//! The kitty graphics command that sends an image to the terminal.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

use crate::store::Image;

/// The most base64 bytes in one chunk of a command.
const CHUNK: usize = 4096;

/// The command that sends the PNG `png` as `image` and makes a virtual placement of the size of
/// `image`, for unicode placeholders. The first chunk holds every key. Each other chunk holds
/// only `m` and `q`. Every chunk has `q=2`, so the terminal sends no reply.
#[must_use]
pub fn transmit(image: Image, png: &[u8]) -> Vec<u8> {
    let encoded = STANDARD.encode(png);
    let chunks: Vec<&[u8]> = if encoded.is_empty() {
        vec![&[]]
    } else {
        encoded.as_bytes().chunks(CHUNK).collect()
    };
    let mut out = Vec::new();
    for (index, chunk) in chunks.iter().enumerate() {
        let more = u8::from(index + 1 < chunks.len());
        out.extend_from_slice(b"\x1b_G");
        if index == 0 {
            out.extend_from_slice(
                format!(
                    "a=T,U=1,f=100,i={},c={},r={},",
                    image.id(),
                    image.columns(),
                    image.rows()
                )
                .as_bytes(),
            );
        }
        out.extend_from_slice(format!("m={more},q=2;").as_bytes());
        out.extend_from_slice(chunk);
        out.extend_from_slice(b"\x1b\\");
    }
    out
}
