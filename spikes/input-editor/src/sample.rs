/// Paragraphs of a long pose, with the kinds of text a game player pastes:
/// accents, CJK, emoji sequences and a long word that cannot wrap.
const PARAGRAPHS: &[&str] = &[
    "Aria leans against the rail of the airship, her coat snapping in the wind. \"You said the storm would pass by dusk,\" she calls over her shoulder, \"and yet here we are.\"",
    "The navigator, a wiry man named Théo, only shrugs and taps the barometer twice. Its needle shivers and settles a hair lower than before.",
    "Below them the lanterns of 港町 flicker on one by one, a string of amber beads along the black water. Somewhere a bell rings the hour: 一、二、三。",
    "She pulls a folded letter from her sleeve and reads it again by the glow of the binnacle 🕯️, lips moving. The family crest 👨‍👩‍👧 is pressed into the wax, and a small flag 🇯🇵 is inked in one corner.",
    "Ref: https://wiki.example.org/Aria_Valcourt/Correspondence/Letters_from_the_Southern_Reach_Volume_Two",
    "\"Théo,\" she says at last, folding the paper with careful, deliberate creases, \"if we turn north now, how long before we reach the cliffs?\"",
];

/// A pose of exactly `bytes` bytes or one character less, cut at a character
/// boundary. Paragraphs are separated by one empty line.
pub fn pose(bytes: usize) -> String {
    let mut out = String::new();
    'fill: for round in 0.. {
        for p in PARAGRAPHS {
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            if round > 0 {
                out.push_str(&format!("({round}) "));
            }
            for c in p.chars() {
                if out.len() + c.len_utf8() > bytes {
                    break 'fill;
                }
                out.push(c);
            }
        }
    }
    while out.len() > bytes {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pose_fills_the_size() {
        let p = pose(4096);
        assert!(p.len() <= 4096 && p.len() > 4090, "{}", p.len());
    }
}
