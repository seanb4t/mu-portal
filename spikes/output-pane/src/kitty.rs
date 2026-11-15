//! Upload of an image with the kitty graphics protocol, as a virtual
//! placement for unicode placeholders (`U=1`).

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

/// The escape sequences that upload `rgba` as image `id` and make it fill
/// `cols` by `rows` placeholder cells.
pub fn transmit(id: u32, w: u32, h: u32, rgba: &[u8], cols: u16, rows: u16) -> Vec<u8> {
    let data = STANDARD.encode(rgba);
    let chunks: Vec<&[u8]> = data.as_bytes().chunks(4096).collect();
    let mut out = Vec::new();
    for (i, c) in chunks.iter().enumerate() {
        let more = u8::from(i + 1 < chunks.len());
        if i == 0 {
            out.extend_from_slice(
                format!("\x1b_Ga=T,q=2,f=32,s={w},v={h},i={id},U=1,c={cols},r={rows},m={more};")
                    .as_bytes(),
            );
        } else {
            out.extend_from_slice(format!("\x1b_Gm={more};").as_bytes());
        }
        out.extend_from_slice(c);
        out.extend_from_slice(b"\x1b\\");
    }
    out
}

/// Deletes every image this client uploaded.
pub const DELETE_ALL: &[u8] = b"\x1b_Ga=d,d=A,q=2\x1b\\";

/// A test picture: a color gradient with a ring.
pub fn picture(w: u32, h: u32) -> Vec<u8> {
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    for y in 0..h {
        for x in 0..w {
            let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
            let ring = (d - cy * 0.7).abs() < cy * 0.12;
            let (r, g, b) = if ring {
                (255, 255, 255)
            } else {
                ((x * 255 / w) as u8, (y * 255 / h) as u8, 160)
            };
            px.extend_from_slice(&[r, g, b, 255]);
        }
    }
    px
}
