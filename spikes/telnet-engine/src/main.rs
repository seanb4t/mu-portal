//! Spike: telnet engine and recorded transcripts.
//!
//! `telnet-engine record --port N --script FILE --out FILE [--host H] [--family NAME]
//! [--charset NAME]`
//! connects to a server, answers its negotiation as a MU client, sends each line of the
//! script after the server goes quiet, and writes a transcript.
//! `telnet-engine replay FILE...` runs transcripts through the engine and prints the events
//! and a summary of each.
//! `telnet-engine diverge [CASES]` counts where this engine and libmudtelnet differ, for each
//! kind of input outside the streams that the differential test covers.

use std::io::{self, Read as _, Write as _};
use std::net::TcpStream;
use std::time::{Duration, Instant};

use telnet_engine::client::Client;
use telnet_engine::oracle::{self, Token};
use telnet_engine::transcript::{Direction, Record, Transcript};
use telnet_engine::{Event, Side, opt};

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    match args.first().map(String::as_str) {
        Some("record") => {
            let port = flag("--port").ok_or("--port is required")?;
            let script = flag("--script").ok_or("--script is required")?;
            let out = flag("--out").ok_or("--out is required")?;
            let host = flag("--host").unwrap_or_else(|| "127.0.0.1".into());
            let family = flag("--family").unwrap_or_default();
            let charset = flag("--charset").unwrap_or_else(|| "UTF-8".into());
            let script = std::fs::read_to_string(&script).map_err(|e| format!("{script}: {e}"))?;
            let transcript =
                record(&host, &port, &family, &charset, &script).map_err(|e| e.to_string())?;
            std::fs::write(&out, transcript.render()).map_err(|e| format!("{out}: {e}"))?;
            eprintln!("wrote {out}: {} records", transcript.records.len());
            Ok(())
        }
        Some("replay") => {
            for path in &args[1..] {
                let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
                let transcript = Transcript::parse(&text).map_err(|e| format!("{path}: {e}"))?;
                println!("== {path}");
                replay(&transcript);
            }
            Ok(())
        }
        Some("diverge") => {
            let cases = args.get(1).and_then(|n| n.parse().ok()).unwrap_or(2000);
            diverge(cases);
            Ok(())
        }
        _ => Err("usage: telnet-engine record|replay|diverge (see the source header)".into()),
    }
}

const FIRST_QUIET: Duration = Duration::from_millis(2500);
const QUIET: Duration = Duration::from_millis(1200);
const LONGEST_WAIT: Duration = Duration::from_secs(10);

fn record(
    host: &str,
    port: &str,
    family: &str,
    charset: &str,
    script: &str,
) -> io::Result<Transcript> {
    let stream = TcpStream::connect(format!("{host}:{port}"))?;
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    stream.set_nodelay(true)?;
    let start = Instant::now();
    let mut session = Session {
        stream,
        start,
        client: Client::with_charset(charset),
        transcript: Transcript::default(),
        open: true,
    };
    session.transcript.notes.push(format!("family: {family}"));
    session.transcript.notes.push(format!(
        "client: mu-portal telnet-engine spike, window {}x{}, accepts charset {charset}",
        telnet_engine::client::COLUMNS,
        telnet_engine::client::ROWS
    ));
    session.wait_quiet(FIRST_QUIET)?;
    for line in script.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') || !session.open {
            continue;
        }
        let mut bytes = line.as_bytes().to_vec();
        bytes.extend_from_slice(b"\r\n");
        session.client.engine.send_data(&bytes);
        session.flush()?;
        session.wait_quiet(QUIET)?;
    }
    Ok(session.transcript)
}

struct Session {
    stream: TcpStream,
    start: Instant,
    client: Client,
    transcript: Transcript,
    open: bool,
}

impl Session {
    fn note(&mut self, direction: Direction, bytes: &[u8]) {
        self.transcript.records.push(Record {
            direction,
            seconds: self.start.elapsed().as_secs_f64(),
            bytes: bytes.to_vec(),
        });
    }

    fn flush(&mut self) -> io::Result<()> {
        let out = self.client.take_output();
        if !out.is_empty() {
            self.stream.write_all(&out)?;
            self.note(Direction::ToServer, &out);
        }
        Ok(())
    }

    /// Reads until the server sends nothing for `quiet`, or for at most `LONGEST_WAIT`.
    fn wait_quiet(&mut self, quiet: Duration) -> io::Result<()> {
        let begin = Instant::now();
        let mut last = Instant::now();
        let mut buf = [0u8; 8192];
        while self.open && last.elapsed() < quiet && begin.elapsed() < LONGEST_WAIT {
            match self.stream.read(&mut buf) {
                Ok(0) => self.open = false,
                Ok(n) => {
                    self.note(Direction::FromServer, &buf[..n]);
                    for event in self.client.receive(&buf[..n]) {
                        eprintln!("{}", show(&event));
                    }
                    self.flush()?;
                    last = Instant::now();
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) => {}
                Err(e) if e.kind() == io::ErrorKind::ConnectionReset => self.open = false,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

fn option_name(o: u8) -> String {
    let name = match o {
        opt::ECHO => "ECHO",
        opt::SGA => "SGA",
        opt::TTYPE => "TTYPE",
        opt::EOR => "EOR",
        opt::NAWS => "NAWS",
        opt::LINEMODE => "LINEMODE",
        opt::NEW_ENVIRON => "NEW-ENVIRON",
        opt::CHARSET => "CHARSET",
        opt::MSDP => "MSDP",
        opt::MSSP => "MSSP",
        opt::MCCP2 => "MCCP2",
        opt::MCCP3 => "MCCP3",
        opt::MSP => "MSP",
        opt::MXP => "MXP",
        opt::ATCP => "ATCP",
        opt::GMCP => "GMCP",
        _ => return format!("option {o}"),
    };
    name.to_owned()
}

fn command_name(c: u8) -> String {
    match c {
        telnet_engine::GA => "GA".into(),
        telnet_engine::EOR => "EOR".into(),
        telnet_engine::NOP => "NOP".into(),
        246 => "AYT".into(),
        242 => "DM".into(),
        _ => format!("command {c}"),
    }
}

fn printable(bytes: &[u8], limit: usize) -> String {
    let text: String = String::from_utf8_lossy(bytes)
        .chars()
        .flat_map(char::escape_debug)
        .collect();
    if text.chars().count() > limit {
        format!("{}...", text.chars().take(limit).collect::<String>())
    } else {
        text
    }
}

fn show(event: &Event) -> String {
    match event {
        Event::Data(d) => format!("  data {} bytes \"{}\"", d.len(), printable(d, 70)),
        Event::Command(c) => format!("  {}", command_name(*c)),
        Event::Enabled(side, o) => format!("  on {side:?} {}", option_name(*o)),
        Event::Disabled(side, o) => format!("  off {side:?} {}", option_name(*o)),
        Event::Subnegotiation(o, p) => {
            format!("  sb {} \"{}\"", option_name(*o), printable(p, 100))
        }
        Event::CompressionStarted => "  compression on".into(),
        Event::CompressionEnded => "  compression off".into(),
        Event::Error(e) => format!("  ERROR {e:?}"),
    }
}

fn replay(transcript: &Transcript) {
    let mut client = Client::new();
    let mut errors = 0;
    let mut subs: Vec<String> = Vec::new();
    let mut commands: Vec<String> = Vec::new();
    let mut data_bytes = 0usize;
    let mut has_cr_nul = false;
    let mut bare_lf = 0usize;
    let mut crlf = 0usize;
    let mut previous = 0u8;
    for read in transcript.server_reads() {
        for event in client.receive(read) {
            println!("{}", show(&event));
            match &event {
                Event::Error(_) => errors += 1,
                Event::Subnegotiation(o, _) => {
                    let name = option_name(*o);
                    if !subs.contains(&name) {
                        subs.push(name);
                    }
                }
                Event::Command(c) => {
                    let name = command_name(*c);
                    if !commands.contains(&name) {
                        commands.push(name);
                    }
                }
                Event::Data(d) => {
                    data_bytes += d.len();
                    for &b in d {
                        if previous == b'\r' && b == 0 {
                            has_cr_nul = true;
                        }
                        if b == b'\n' {
                            if previous == b'\r' {
                                crlf += 1;
                            } else {
                                bare_lf += 1;
                            }
                        }
                        previous = b;
                    }
                }
                _ => {}
            }
        }
        client.take_output();
    }
    let on = |side| {
        (0..=255u8)
            .filter(|&o| client.engine.is_enabled(side, o))
            .map(option_name)
            .collect::<Vec<_>>()
            .join(" ")
    };
    println!("-- summary");
    println!("server performs: {}", on(Side::Remote));
    println!("client performs: {}", on(Side::Local));
    println!("compressed at end: {}", client.engine.is_compressed());
    println!("subnegotiations: {}", subs.join(" "));
    println!("commands: {}", commands.join(" "));
    println!("data bytes: {data_bytes}, CRLF {crlf}, bare LF {bare_lf}, CR NUL {has_cr_nul}");
    println!("errors: {errors}");
}

/// A small xorshift generator, so that each run of `diverge` sees the same cases.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn bytes(&mut self, max: usize, allow_iac: bool) -> Vec<u8> {
        let len = 1 + self.below(max);
        (0..len)
            .map(|_| {
                let b = (self.next() & 0xff) as u8;
                if b == 255 && !allow_iac { b'x' } else { b }
            })
            .collect()
    }
}

fn base_token(rng: &mut Rng) -> Token {
    match rng.below(5) {
        0 | 1 => Token::Data(rng.bytes(30, false)),
        2 => {
            Token::Prompt([telnet_engine::GA, telnet_engine::EOR, telnet_engine::NOP][rng.below(3)])
        }
        3 => Token::Subnegotiation(opt::GMCP, rng.bytes(30, false)),
        _ => [Token::StartCompression, Token::EndCompression][rng.below(2)].clone(),
    }
}

fn diverge(cases: usize) {
    type Feature = (&'static str, fn(&mut Rng) -> Token);
    let features: [Feature; 4] = [
        ("IAC IAC in data", |_| Token::EscapedIac),
        ("IAC IAC in a subnegotiation", |r| {
            let mut p = r.bytes(20, false);
            p.insert(r.below(p.len() + 1), 255);
            Token::SubnegotiationWithIac(opt::GMCP, p)
        }),
        ("IAC AYT, IAC DM and other two-byte commands", |r| {
            Token::OtherCommand([240u8, 241, 242, 243, 244, 245, 246, 247, 248][r.below(9)])
        }),
        ("none (control)", base_token),
    ];
    println!("cases per feature: {cases}");
    println!("feature | ours wrong | libmudtelnet wrong");
    for (name, make) in features {
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        let (mut ours_wrong, mut theirs_wrong) = (0, 0);
        let mut example: Option<(usize, String)> = None;
        for _ in 0..cases {
            let mut tokens: Vec<Token> = (0..rng.below(12)).map(|_| base_token(&mut rng)).collect();
            let at = rng.below(tokens.len() + 1);
            tokens.insert(at, make(&mut rng));
            let bytes = oracle::encode(&tokens);
            let cuts: Vec<usize> = (0..rng.below(6)).map(|_| rng.next() as usize).collect();
            let reads = oracle::split(&bytes, &cuts);
            let want = oracle::expected(&tokens);
            if oracle::ours(&reads) != want {
                ours_wrong += 1;
            }
            let theirs = oracle::reference(&reads);
            if theirs != want {
                theirs_wrong += 1;
                if example.as_ref().is_none_or(|(len, _)| bytes.len() < *len) {
                    example = Some((
                        bytes.len(),
                        format!(
                            "{} -> {theirs:?}",
                            telnet_engine::transcript::encode_hex(&bytes)
                        ),
                    ));
                }
            }
        }
        println!("{name} | {ours_wrong} | {theirs_wrong}");
        if let Some((_, e)) = example {
            println!("  shortest: {e}");
        }
    }
    server_option_subnegotiation();
}

/// A table set up the way a client sets it up for a server option such as GMCP: supported,
/// and turned on by the server's WILL.
fn server_option_subnegotiation() {
    use libmudtelnet::Parser;
    use libmudtelnet::compatibility::CompatibilityTable;
    use telnet_engine::{IAC, SB, SE, WILL};

    let mut table = CompatibilityTable::new();
    table.support(opt::GMCP);
    let mut parser = Parser::with_support(table);
    let mut stream = vec![IAC, WILL, opt::GMCP, IAC, SB, opt::GMCP];
    stream.extend_from_slice(b"Char.Vitals {}");
    stream.extend_from_slice(&[IAC, SE]);
    let events = parser.receive(&stream);
    let delivered = events
        .iter()
        .any(|e| matches!(e, libmudtelnet::events::TelnetEvents::Subnegotiation(_)));
    println!("libmudtelnet, GMCP after the server's WILL: subnegotiation delivered = {delivered}");
}
