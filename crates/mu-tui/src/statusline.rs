//! The statusline: one row of segments about the world in focus.

use mu_config::Segment;
use unicode_segmentation::UnicodeSegmentation as _;

use crate::model::Model;
use crate::text::clean;

/// The text between two segments of one side.
const BETWEEN: &str = " | ";

/// A time of `ms` milliseconds as `M:SS`, or as `H:MM:SS` from one hour. A time before 0 is
/// `0:00`.
#[must_use]
pub fn time(ms: i64) -> String {
    let seconds = ms.max(0) / 1000;
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// The value of `segment` for the world in focus, or none for a segment with no value now.
fn value(model: &Model, segment: Segment) -> Option<String> {
    let world = model.focused_world()?;
    match segment {
        Segment::World => Some(clean(&world.id)),
        Segment::State => world.state.name().map(str::to_owned),
        Segment::Connected => world.connected_ms.map(|since| time(model.now_ms - since)),
        Segment::Idle => world.sent_ms.map(|since| time(model.now_ms - since)),
        Segment::Typed => Some(model.input().chars().count().to_string()),
    }
}

fn side(model: &Model, segments: &[Segment]) -> String {
    segments
        .iter()
        .filter_map(|segment| value(model, *segment))
        .collect::<Vec<_>>()
        .join(BETWEEN)
}

/// The row of the statusline in `width` columns: the left segments from the left edge, and the
/// right segments to the right edge. If the two sides do not fit, the right side shows in
/// full and the left side is cut at its end.
#[must_use]
pub fn row(model: &Model, width: u16) -> String {
    let width = usize::from(width);
    if model.focused_world().is_none() {
        return "no world".to_owned();
    }
    let profile = &model.caps.profile;
    let width_of =
        |text: &str| -> usize { text.graphemes(true).map(|each| profile.width(each)).sum() };
    let left = side(model, &model.segments.0);
    let right = side(model, &model.segments.1);
    if width_of(&right) > width {
        // The right side shows in full as far as the row has room, from its end.
        let mut kept: Vec<&str> = Vec::new();
        let mut used = 0;
        for grapheme in right.graphemes(true).rev() {
            let next = used + profile.width(grapheme);
            if next > width {
                break;
            }
            kept.push(grapheme);
            used = next;
        }
        kept.reverse();
        return format!("{}{}", " ".repeat(width - used), kept.concat());
    }
    let right_width = width_of(&right);
    // One column stays free between the two sides.
    let left_room = if right.is_empty() {
        width
    } else {
        (width - right_width).saturating_sub(1)
    };
    let mut shown = String::new();
    let mut used = 0;
    for grapheme in left.graphemes(true) {
        let next = used + profile.width(grapheme);
        if next > left_room {
            break;
        }
        shown.push_str(grapheme);
        used = next;
    }
    if right.is_empty() {
        return shown;
    }
    format!("{shown}{}{right}", " ".repeat(width - right_width - used))
}
