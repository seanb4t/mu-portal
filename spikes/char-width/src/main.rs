//! Spike: emoji and wide-character width.
//!
//! `char-width probe [--out FILE]` measures each grapheme with a cursor position report,
//! learns a profile from the probe set, checks it on the test set, and measures the drift
//! of a ratatui row for each render strategy. It needs a real terminal.
//! `char-width list` prints the class and the `unicode-width` value of each grapheme.

mod class;
mod profile;
mod reply;
mod sets;

use std::fmt::Write as _;
use std::io::{self, Read as _, Write as _};
use std::num::NonZeroU16;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::{Buffer, CellDiffOption};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::class::{Class, char_sum, classify};
use crate::profile::{Profile, unicode};
use crate::reply::{Reply, parse};
use crate::sets::{PROBE, TEST, code_points};

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("list") => {
            for g in PROBE.iter().chain(TEST) {
                println!(
                    "{:<12} uw={} sum={} {}",
                    classify(g).name(),
                    unicode(g),
                    char_sum(g, 0),
                    code_points(g)
                );
            }
            Ok(())
        }
        Some("probe") => {
            let out = args
                .iter()
                .position(|a| a == "--out")
                .and_then(|i| args.get(i + 1));
            let report = probe()?;
            match out {
                Some(path) => std::fs::write(path, &report)?,
                None => print!("{report}"),
            }
            Ok(())
        }
        _ => {
            eprintln!("usage: char-width list | probe [--out FILE]");
            Ok(())
        }
    }
}

struct Tty {
    rx: Receiver<Vec<u8>>,
    pending: Vec<u8>,
}

impl Tty {
    fn open() -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut stdin = io::stdin();
            let mut buf = [0u8; 4096];
            while let Ok(n) = stdin.read(&mut buf) {
                if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
                    break;
                }
            }
        });
        Tty {
            rx,
            pending: Vec::new(),
        }
    }

    /// Writes `q` and a DA1 sentinel, then returns every reply before the DA1 reply.
    fn query(&mut self, q: &str) -> io::Result<Vec<Reply>> {
        let mut out = io::stdout();
        out.write_all(q.as_bytes())?;
        out.write_all(b"\x1b[c")?;
        out.flush()?;
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut replies = Vec::new();
        loop {
            let (r, used) = parse(&self.pending);
            self.pending.drain(..used);
            let done = r.contains(&Reply::Da1);
            replies.extend(r.into_iter().filter(|x| *x != Reply::Da1));
            if done {
                return Ok(replies);
            }
            let left = deadline.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(left) {
                Ok(b) => self.pending.extend(b),
                Err(_) => return Err(io::Error::other("no DA1 reply from the terminal")),
            }
        }
    }

    /// The width the terminal gives each grapheme: the cursor column after it, from column 1.
    fn measure(&mut self, gs: &[&str]) -> io::Result<Vec<u16>> {
        let mut q = String::from("\x1b[H");
        for g in gs {
            write!(q, "\r\x1b[2K{g}\x1b[6n").unwrap();
        }
        q.push_str("\r\x1b[2K");
        let cols: Vec<u16> = self
            .query(&q)?
            .into_iter()
            .filter_map(|r| match r {
                Reply::Cpr { col, .. } => Some(col - 1),
                _ => None,
            })
            .collect();
        if cols.len() != gs.len() {
            return Err(io::Error::other(format!(
                "{} position reports for {} graphemes",
                cols.len(),
                gs.len()
            )));
        }
        Ok(cols)
    }

    fn cursor_col(&mut self) -> io::Result<u16> {
        self.query("\x1b[6n")?
            .into_iter()
            .find_map(|r| match r {
                Reply::Cpr { col, .. } => Some(col),
                _ => None,
            })
            .ok_or_else(|| io::Error::other("no position report"))
    }
}

/// One row: `[`, the grapheme, `]`, `X`. `width` is the width the renderer gives the
/// grapheme: `None` lets ratatui use `unicode-width`, `Some(w)` forces `w` cells.
struct Row<'a> {
    g: &'a str,
    width: Option<u16>,
}

impl Row<'_> {
    fn used(&self) -> u16 {
        self.width.unwrap_or_else(|| unicode(self.g))
    }
}

impl Widget for Row<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (x, y) = (area.x, area.y);
        let w = self.used();
        buf[(x, y)].set_symbol("[");
        match self.width {
            None => {
                buf.set_string(x + 1, y, self.g, Style::default());
            }
            Some(0) => {}
            Some(n) => {
                buf[(x + 1, y)]
                    .set_symbol(self.g)
                    .set_diff_option(CellDiffOption::ForcedWidth(NonZeroU16::new(n).unwrap()));
            }
        }
        buf[(x + 1 + w, y)].set_symbol("]");
        buf[(x + 2 + w, y)].set_symbol("X");
    }
}

#[derive(Clone, Copy)]
enum Strategy {
    /// ratatui with `unicode-width`.
    Naive,
    /// The width from the learned profile, set with `ForcedWidth`.
    Model,
    /// No probe: a risky grapheme always gets at least two cells, so ratatui moves the
    /// cursor before the next cell.
    Reserve,
}

const STRATEGIES: [(Strategy, &str); 3] = [
    (Strategy::Naive, "naive"),
    (Strategy::Model, "model"),
    (Strategy::Reserve, "reserve"),
];

fn width_for(s: Strategy, g: &str, p: &Profile) -> Option<u16> {
    match s {
        Strategy::Naive => None,
        Strategy::Model => Some(p.width(g)),
        Strategy::Reserve => {
            let c = classify(g);
            if c == Class::Zero {
                Some(0)
            } else if c.risky() {
                Some(unicode(g).max(2))
            } else {
                None
            }
        }
    }
}

struct Restore;

impl Drop for Restore {
    fn drop(&mut self) {
        let _ = io::stdout().write_all(b"\x1b[?2027l");
        let _ = crossterm::execute!(io::stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

fn probe() -> io::Result<String> {
    enable_raw_mode()?;
    crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
    let _restore = Restore;
    let mut tty = Tty::open();
    let mut report = String::new();

    let version = tty.query("\x1b[>0q")?.into_iter().find_map(|r| match r {
        Reply::Version(v) => Some(v),
        _ => None,
    });
    let mode = tty
        .query("\x1b[?2027$p")?
        .into_iter()
        .find_map(|r| match r {
            Reply::Mode { mode: 2027, value } => Some(value),
            _ => None,
        });
    let env = |k: &str| std::env::var(k).unwrap_or_default();
    writeln!(
        report,
        "terminal\txtversion={}\tTERM_PROGRAM={} {}\tTERM={}\tmode2027={}",
        version.as_deref().unwrap_or("-"),
        env("TERM_PROGRAM"),
        env("TERM_PROGRAM_VERSION"),
        env("TERM"),
        mode.map_or("-".into(), |v| v.to_string()),
    )
    .unwrap();
    pass(&mut tty, "default", &mut report)?;
    if mode == Some(2) {
        io::stdout().write_all(b"\x1b[?2027h")?;
        pass(&mut tty, "mode-2027", &mut report)?;
        io::stdout().write_all(b"\x1b[?2027l")?;
    }
    Ok(report)
}

fn pass(tty: &mut Tty, name: &str, report: &mut String) -> io::Result<()> {
    let start = Instant::now();
    let probed = tty.measure(PROBE)?;
    writeln!(
        report,
        "pass\t{name}\tprobe set measured in {:.1} ms",
        start.elapsed().as_secs_f64() * 1000.0
    )
    .unwrap();
    let pairs: Vec<(&str, u16)> = PROBE.iter().copied().zip(probed.iter().copied()).collect();
    let profile = Profile::learn(&pairs);
    for (g, m) in &pairs {
        let c = classify(g);
        writeln!(
            report,
            "probe\t{}\tuw={}\tterm={}\trule={}\t{}",
            c.name(),
            unicode(g),
            m,
            profile.rules[&c].name(),
            code_points(g)
        )
        .unwrap();
    }

    let measured = tty.measure(TEST)?;
    let (mut naive_miss, mut model_miss) = (0, 0);
    let mut drift_rows = [0usize; 3];
    let mut overlap_rows = [0usize; 3];
    for (g, &m) in TEST.iter().zip(&measured) {
        let (uw, model) = (unicode(g), profile.width(g));
        naive_miss += usize::from(uw != m);
        model_miss += usize::from(model != m);
        let mut drifts = Vec::new();
        let mut overlaps = Vec::new();
        for (i, (s, _)) in STRATEGIES.iter().enumerate() {
            let row = Row {
                g,
                width: width_for(*s, g, &profile),
            };
            let expected = 4 + row.used();
            let overlap = i32::from(m) - i32::from(row.used());
            overlap_rows[i] += usize::from(overlap > 0);
            overlaps.push(overlap);
            // A new terminal has an empty back buffer. `Terminal::clear` reads the cursor
            // position and would take a reply from the probe reader.
            io::stdout().write_all(b"\x1b[2J")?;
            let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
            terminal.draw(|f| f.render_widget(row, Rect::new(0, 0, f.area().width, 1)))?;
            let d = i32::from(tty.cursor_col()?) - i32::from(expected);
            drift_rows[i] += usize::from(d != 0);
            drifts.push(d);
        }
        writeln!(
            report,
            "test\t{}\tuw={uw}\tterm={m}\tmodel={model}\tdrift naive={} model={} reserve={}\t\
             overlap naive={} model={} reserve={}\t{}",
            classify(g).name(),
            drifts[0],
            drifts[1],
            drifts[2],
            overlaps[0],
            overlaps[1],
            overlaps[2],
            code_points(g)
        )
        .unwrap();
    }
    writeln!(
        report,
        "summary\t{name}\twidth misses: unicode-width {naive_miss}/{n}, profile {model_miss}/{n}\t\
         rows with drift: naive {}, model {}, reserve {}\t\
         rows with overlap: naive {}, model {}, reserve {}",
        drift_rows[0],
        drift_rows[1],
        drift_rows[2],
        overlap_rows[0],
        overlap_rows[1],
        overlap_rows[2],
        n = TEST.len()
    )
    .unwrap();
    io::stdout().write_all(b"\x1b[2J")?;
    Ok(())
}
