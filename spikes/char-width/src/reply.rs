//! A parser for the replies of the terminal to the probe queries.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// Cursor position report, 1-based.
    Cpr { row: u16, col: u16 },
    /// DECRPM: the state of a private mode.
    Mode { mode: u16, value: u16 },
    /// Primary device attributes. The probe sends it last, as a sentinel.
    Da1,
    /// XTVERSION: the name and version of the terminal.
    Version(String),
}

/// Parses the complete replies at the start of `buf`.
/// Returns them and the number of bytes used. An incomplete reply stays in the buffer.
pub fn parse(buf: &[u8]) -> (Vec<Reply>, usize) {
    let mut out = Vec::new();
    let mut i = 0;
    while i < buf.len() {
        if buf[i] != 0x1b {
            i += 1;
            continue;
        }
        let Some(&kind) = buf.get(i + 1) else { break };
        match kind {
            b'[' => {
                let start = i + 2;
                let Some(end) = (start..buf.len()).find(|&j| (0x40..=0x7e).contains(&buf[j]))
                else {
                    break;
                };
                let params = std::str::from_utf8(&buf[start..end]).unwrap_or("");
                if let Some(r) = csi(params, buf[end]) {
                    out.push(r);
                }
                i = end + 1;
            }
            b'P' => {
                let start = i + 2;
                let Some(end) = (start..buf.len().saturating_sub(1))
                    .find(|&j| buf[j] == 0x1b && buf[j + 1] == b'\\')
                else {
                    break;
                };
                let body = String::from_utf8_lossy(&buf[start..end]);
                if let Some(v) = body.strip_prefix(">|") {
                    out.push(Reply::Version(v.to_string()));
                }
                i = end + 2;
            }
            _ => i += 1,
        }
    }
    (out, i)
}

fn csi(params: &str, fin: u8) -> Option<Reply> {
    let nums = |s: &str| -> Vec<u16> { s.split(';').filter_map(|n| n.parse().ok()).collect() };
    match fin {
        b'R' => match nums(params)[..] {
            [row, col] => Some(Reply::Cpr { row, col }),
            _ => None,
        },
        b'y' => {
            let p = params.strip_prefix('?')?.strip_suffix('$')?;
            match nums(p)[..] {
                [mode, value] => Some(Reply::Mode { mode, value }),
                _ => None,
            }
        }
        b'c' if params.starts_with('?') => Some(Reply::Da1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_each_reply() {
        let buf = b"\x1b[3;7R\x1b[?2027;2$y\x1bP>|ghostty 1.2\x1b\\\x1b[?62;22c";
        let (r, used) = parse(buf);
        assert_eq!(
            r,
            [
                Reply::Cpr { row: 3, col: 7 },
                Reply::Mode {
                    mode: 2027,
                    value: 2
                },
                Reply::Version("ghostty 1.2".into()),
                Reply::Da1,
            ]
        );
        assert_eq!(used, buf.len());
    }

    #[test]
    fn keeps_an_incomplete_reply() {
        let (r, used) = parse(b"\x1b[3;7R\x1b[4;");
        assert_eq!(r, [Reply::Cpr { row: 3, col: 7 }]);
        assert_eq!(used, 6);
    }
}
