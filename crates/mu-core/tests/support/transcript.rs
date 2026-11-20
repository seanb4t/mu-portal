//! The record format of a telnet transcript. The tests and the example `transcript_lines` use
//! this file.

/// The reads of the game in a transcript of the telnet spike, in order.
///
/// Each record is one line: `<` for a read of the game or `>` for a write to it, the seconds
/// since the connect, and the hex of the bytes. A line that starts with `#` is a note.
pub fn transcript_reads(text: &str) -> Vec<Vec<u8>> {
    text.lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let mut fields = line.splitn(3, ' ');
            let direction = fields.next().unwrap();
            assert!(matches!(direction, "<" | ">"), "bad record: {line}");
            let _seconds: f64 = fields.next().unwrap().parse().unwrap();
            let bytes = hex::decode(fields.next().unwrap_or_default()).unwrap();
            (direction == "<").then_some(bytes)
        })
        .collect()
}
