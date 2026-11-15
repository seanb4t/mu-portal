//! Spike: the output pane on the alternate screen.
//!
//! `output-pane bench [lines]` measures the store, the re-wrap and both renderers.
//! `output-pane run [lines] [ratatui|direct]` opens the pane in the terminal.

mod cells;
mod diacritics;
mod generate;
mod kitty;
mod render;
mod selection;
mod store;
mod style;
mod view;
mod wrap;

use std::io::{self, Write as _};
use std::process::Command;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEventKind,
};
use crossterm::execute;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style as RStyle};
use ratatui::widgets::Widget;

use crate::render::{Direct, Pane};
use crate::selection::Selection;
use crate::store::LineStore;
use crate::view::{Pos, Viewport, VisibleRow};
use crate::wrap::{Row, row_count};

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let lines = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);
    match args.first().map(String::as_str) {
        Some("bench") => {
            bench(lines);
            Ok(())
        }
        Some("run") => run(lines, args.get(2).is_some_and(|r| r == "direct")),
        _ => {
            eprintln!("usage: output-pane bench|run [lines] [ratatui|direct]");
            Ok(())
        }
    }
}

fn rss_mib() -> f64 {
    let out = Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output();
    out.ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<f64>().ok())
        .map_or(f64::NAN, |kib| kib / 1024.0)
}

fn stats(v: &mut [Duration]) -> String {
    v.sort_unstable();
    let pick = |q: f64| v[((v.len() - 1) as f64 * q) as usize];
    format!(
        "p50 {:>9.1?}  p99 {:>9.1?}  max {:>9.1?}  (n={})",
        pick(0.5),
        pick(0.99),
        v[v.len() - 1],
        v.len()
    )
}

fn ratatui_frame(
    store: &LineStore,
    rows: &[VisibleRow],
    area: Rect,
    prev: &mut Buffer,
    cur: &mut Buffer,
    sink: &mut Vec<u8>,
) -> usize {
    cur.reset();
    Pane {
        store,
        rows,
        sel: None,
    }
    .render(area, cur);
    let diff = prev.diff(cur);
    sink.clear();
    CrosstermBackend::new(&mut *sink)
        .draw(diff.into_iter())
        .expect("draw to memory");
    std::mem::swap(prev, cur);
    sink.len()
}

fn bench(n: usize) {
    let rss0 = rss_mib();
    let t = Instant::now();
    let mut store = LineStore::new();
    generate::fill(&mut store, n, 42);
    let ingest = t.elapsed();
    let rss1 = rss_mib();
    let text: usize = (store.first_id()..store.end_id())
        .map(|id| store.line(id).text.len())
        .sum();
    println!("lines            {n}");
    println!(
        "ingest           {ingest:.1?} ({:.0} ns/line)",
        ingest.as_nanos() as f64 / n as f64
    );
    println!(
        "store            {:.1} MiB by capacity, text {:.1} MiB, {:.1} B/line overhead",
        store.bytes() as f64 / 1048576.0,
        text as f64 / 1048576.0,
        (store.bytes() - text) as f64 / n as f64
    );
    println!("rss              {rss0:.1} -> {rss1:.1} MiB");

    let mut scratch: Vec<Row> = Vec::new();
    for w in [80u16, 200] {
        let t = Instant::now();
        let total: usize = (store.first_id()..store.end_id())
            .map(|id| row_count(&store.line(id), w, &mut scratch))
            .sum();
        println!(
            "full re-wrap {w:>3} {total} rows in {:.1?} (one thread)",
            t.elapsed()
        );
    }

    let height = 60u16;
    let widths: Vec<u16> = (40..=200).chain((40..200).rev()).collect();
    let mut rows: Vec<VisibleRow> = Vec::new();
    for (name, follow) in [("middle", false), ("follow", true)] {
        let mut vp = Viewport {
            top: Pos {
                line: n / 2,
                byte: 0,
            },
            follow,
        };
        let (mut lay, mut rat, mut dir) = (Vec::new(), Vec::new(), Vec::new());
        let mut direct = Direct::default();
        let mut backend: Vec<u8> = Vec::with_capacity(1 << 20);
        for &w in &widths {
            let t = Instant::now();
            vp.layout(&store, w, height, &mut rows, &mut scratch);
            lay.push(t.elapsed());
            let area = Rect::new(0, 0, w, height);
            let (mut prev, mut cur) = (Buffer::empty(area), Buffer::empty(area));
            let t = Instant::now();
            ratatui_frame(&store, &rows, area, &mut prev, &mut cur, &mut backend);
            rat.push(lay[lay.len() - 1] + t.elapsed());
            let t = Instant::now();
            direct.frame(&store, &rows, w, height, None, "status");
            dir.push(lay[lay.len() - 1] + t.elapsed());
        }
        println!("resize {name} layout          {}", stats(&mut lay));
        println!("resize {name} layout+ratatui  {}", stats(&mut rat));
        println!("resize {name} layout+direct   {}", stats(&mut dir));
    }

    let (w, frames) = (200u16, 2000);
    let area = Rect::new(0, 0, w, height);
    let mut vp = Viewport::default();
    vp.layout(&store, w, height, &mut rows, &mut scratch);
    let (mut prev, mut cur) = (Buffer::empty(area), Buffer::empty(area));
    let mut backend: Vec<u8> = Vec::with_capacity(1 << 20);
    let mut direct = Direct::default();
    let (mut rat, mut dir, mut idle) = (Vec::new(), Vec::new(), Vec::new());
    let (mut rat_bytes, mut dir_bytes) = (0usize, 0usize);
    for _ in 0..frames {
        vp.scroll(&store, w, -1, &mut scratch);
        let t = Instant::now();
        vp.layout(&store, w, height, &mut rows, &mut scratch);
        let lay = t.elapsed();
        let t = Instant::now();
        rat_bytes += ratatui_frame(&store, &rows, area, &mut prev, &mut cur, &mut backend);
        rat.push(lay + t.elapsed());
        let t = Instant::now();
        dir_bytes += direct.frame(&store, &rows, w, height, None, "status").len();
        dir.push(lay + t.elapsed());
        let t = Instant::now();
        ratatui_frame(&store, &rows, area, &mut prev, &mut cur, &mut backend);
        idle.push(t.elapsed());
    }
    println!(
        "scroll 1 row {w}x{height} ratatui  {}  {} B/frame",
        stats(&mut rat),
        rat_bytes / frames
    );
    println!(
        "scroll 1 row {w}x{height} direct   {}  {} B/frame",
        stats(&mut dir),
        dir_bytes / frames
    );
    println!("no change    {w}x{height} ratatui  {}", stats(&mut idle));

    let mut vp = Viewport::default();
    vp.layout(&store, w, height, &mut rows, &mut scratch);
    let (mut prev, mut cur) = (Buffer::empty(area), Buffer::empty(area));
    ratatui_frame(&store, &rows, area, &mut prev, &mut cur, &mut backend);
    let sent = String::from_utf8_lossy(&backend);
    let want = usize::from(generate::IMAGE.cols * generate::IMAGE.rows) * 2;
    println!(
        "placeholders     ratatui wrote {} of {want} (two images on screen), image id color present: {}",
        sent.matches(cells::PLACEHOLDER).count(),
        sent.contains("\x1b[38;2;0;0;1;")
    );
}

struct App {
    store: LineStore,
    vp: Viewport,
    rows: Vec<VisibleRow>,
    scratch: Vec<Row>,
    sel: Option<Selection>,
    dragging: bool,
    direct: Option<Direct>,
    frame: Duration,
    worst_resize: Duration,
    stream: Option<generate::Rng>,
    msg: String,
}

impl App {
    fn status(&self, w: u16, h: u16) -> String {
        format!(
            " {} | {} lines | {w}x{h} | last frame {:.1?} | worst resize {:.1?} | {} | {} | q quit, r renderer, t stream, drag to copy",
            if self.direct.is_some() {
                "direct"
            } else {
                "ratatui"
            },
            self.store.end_id() - self.store.first_id(),
            self.frame,
            self.worst_resize,
            if self.vp.follow { "follow" } else { "scrolled" },
            self.msg,
        )
    }
}

fn draw(app: &mut App, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let t = Instant::now();
    let (w, h) = crossterm::terminal::size()?;
    let ph = h.saturating_sub(1);
    let sel = app.sel.map(|s| s.range());
    app.vp
        .layout(&app.store, w, ph, &mut app.rows, &mut app.scratch);
    let status = app.status(w, h);
    if let Some(direct) = app.direct.as_mut() {
        let bytes = direct.frame(&app.store, &app.rows, w, ph, sel, &status);
        let mut out = io::stdout().lock();
        out.write_all(bytes)?;
        out.flush()?;
    } else {
        terminal.draw(|f| {
            let area = f.area();
            let pane = Rect {
                height: area.height.saturating_sub(1),
                ..area
            };
            f.render_widget(
                Pane {
                    store: &app.store,
                    rows: &app.rows,
                    sel,
                },
                pane,
            );
            let bar = Rect {
                y: pane.height,
                height: 1,
                ..area
            };
            f.buffer_mut()
                .set_style(bar, RStyle::new().add_modifier(Modifier::REVERSED));
            f.buffer_mut().set_stringn(
                0,
                bar.y,
                &status,
                usize::from(bar.width),
                RStyle::new().add_modifier(Modifier::REVERSED),
            );
        })?;
    }
    app.frame = t.elapsed();
    Ok(())
}

fn run(lines: usize, direct: bool) -> io::Result<()> {
    let mut store = LineStore::new();
    generate::fill(&mut store, lines, 42);
    let mut app = App {
        store,
        vp: Viewport::default(),
        rows: Vec::new(),
        scratch: Vec::new(),
        sel: None,
        dragging: false,
        direct: direct.then(Direct::default),
        frame: Duration::ZERO,
        worst_resize: Duration::ZERO,
        stream: None,
        msg: String::new(),
    };
    let mut terminal = ratatui::init();
    let result = event_loop(&mut app, &mut terminal);
    let mut out = io::stdout();
    out.write_all(kitty::DELETE_ALL)?;
    execute!(out, DisableMouseCapture)?;
    ratatui::restore();
    result
}

fn event_loop(app: &mut App, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let mut out = io::stdout();
    execute!(out, EnableMouseCapture)?;
    let img = generate::IMAGE;
    out.write_all(&kitty::transmit(
        img.id,
        192,
        64,
        &kitty::picture(192, 64),
        img.cols,
        img.rows,
    ))?;
    out.flush()?;
    let mut dirty = true;
    let mut resized = false;
    loop {
        if dirty {
            draw(app, terminal)?;
            if resized {
                app.worst_resize = app.worst_resize.max(app.frame);
            }
            dirty = false;
            resized = false;
        }
        let tick = if app.stream.is_some() { 50 } else { 500 };
        if !event::poll(Duration::from_millis(tick))? {
            if let Some(rng) = app.stream.as_mut() {
                let (mut text, mut spans) = (String::new(), Vec::new());
                for _ in 0..20 {
                    generate::push_line(&mut app.store, rng, &mut text, &mut spans);
                }
                app.store.trim(2_000_000);
                dirty = true;
            }
            continue;
        }
        let (w, h) = crossterm::terminal::size()?;
        let ph = h.saturating_sub(1);
        dirty = true;
        match event::read()? {
            Event::Resize(..) => resized = true,
            Event::Key(k) if k.kind == KeyEventKind::Press => match k.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('r') => {
                    app.direct = match app.direct.take() {
                        Some(_) => {
                            terminal.clear()?;
                            None
                        }
                        None => Some(Direct::default()),
                    };
                }
                KeyCode::Char('t') => {
                    app.stream = match app.stream {
                        Some(_) => None,
                        None => Some(generate::Rng::new(7)),
                    };
                }
                KeyCode::Up => app.vp.scroll(&app.store, w, -1, &mut app.scratch),
                KeyCode::Down => app.vp.scroll(&app.store, w, 1, &mut app.scratch),
                KeyCode::PageUp => {
                    app.vp
                        .scroll(&app.store, w, -(ph as isize - 1), &mut app.scratch)
                }
                KeyCode::PageDown => {
                    app.vp
                        .scroll(&app.store, w, ph as isize - 1, &mut app.scratch)
                }
                KeyCode::Home => app.vp.home(&app.store),
                KeyCode::End => app.vp.follow = true,
                _ => dirty = false,
            },
            Event::Mouse(m) => match m.kind {
                MouseEventKind::ScrollUp => app.vp.scroll(&app.store, w, -3, &mut app.scratch),
                MouseEventKind::ScrollDown => app.vp.scroll(&app.store, w, 3, &mut app.scratch),
                MouseEventKind::Down(MouseButton::Left) => {
                    app.sel = selection::hit(&app.store, &app.rows, m.column, m.row)
                        .map(|p| Selection { anchor: p, head: p });
                    app.dragging = app.sel.is_some();
                }
                MouseEventKind::Drag(MouseButton::Left) if app.dragging => {
                    let y = m.row.min(ph.saturating_sub(1));
                    if let (Some(s), Some(p)) = (
                        app.sel.as_mut(),
                        selection::hit(&app.store, &app.rows, m.column, y),
                    ) {
                        s.head = p;
                    }
                }
                MouseEventKind::Up(MouseButton::Left) if app.dragging => {
                    app.dragging = false;
                    if let Some(s) = app.sel {
                        let text = selection::extract(&app.store, &s);
                        out.write_all(selection::osc52(&text).as_bytes())?;
                        out.flush()?;
                        app.msg = format!("copied {} bytes", text.len());
                    }
                }
                _ => dirty = false,
            },
            _ => dirty = false,
        }
    }
}
