//! The walk of game text.

use mu_config::ansi::{SCREEN, Sink, walk};

#[derive(Default)]
struct Seen {
    text: String,
    sgr: Vec<Vec<Vec<u16>>>,
}

impl Sink for Seen {
    fn char(&mut self, c: char) {
        self.text.push(c);
    }

    fn sgr(&mut self, groups: &[&[u16]]) {
        self.sgr
            .push(groups.iter().map(|group| group.to_vec()).collect());
    }
}

fn seen(text: &str) -> Seen {
    let mut sink = Seen::default();
    walk(text, &SCREEN, &mut sink);
    sink
}

#[test]
fn the_walk_gives_the_printable_characters() {
    assert_eq!(seen("Welcome, aria. é").text, "Welcome, aria. é");
}

#[test]
fn the_walk_gives_each_sgr_sequence() {
    let walked = seen("\u{1b}[1;31mred\u{1b}[38:2:1:2:3mrgb\u{1b}[m");
    assert_eq!(
        (walked.text.as_str(), walked.sgr),
        (
            "redrgb",
            vec![
                vec![vec![1], vec![31]],
                vec![vec![38, 2, 1, 2, 3]],
                vec![vec![0]],
            ]
        )
    );
}

#[test]
fn the_walk_drops_other_escape_sequences() {
    let walked = seen("a\u{1b}[2Jb\u{1b}]0;title\u{7}c\u{1b}(Bd");
    assert_eq!((walked.text.as_str(), walked.sgr.len()), ("abcd", 0));
}

#[test]
fn the_walk_maps_the_control_characters() {
    assert_eq!(seen("a\tb\u{7}c\u{10EEEE}").text, "a b\u{fffd}c\u{fffd}");
}
