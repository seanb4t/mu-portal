//! The kitty graphics command that sends an image.

use mu_tui::kitty::transmit;
use mu_tui::store::Image;

#[test]
fn small_image_is_one_chunk() {
    let command = transmit(Image::new(7, 2, 3).unwrap(), b"PNG");
    assert_eq!(
        command,
        b"\x1b_Ga=T,U=1,f=100,i=7,c=3,r=2,m=0,q=2;UE5H\x1b\\"
    );
}

#[test]
fn large_image_is_chunks_of_4096() {
    // 3072 bytes are exactly 4096 base64 bytes, so 3073 bytes take a second chunk.
    let command = transmit(Image::new(7, 2, 3).unwrap(), &[0; 3073]);
    let text = String::from_utf8(command).unwrap();
    let chunks: Vec<(&str, usize)> = text
        .split_terminator("\x1b\\")
        .map(|chunk| {
            let (keys, data) = chunk.split_once(';').unwrap();
            (keys, data.len())
        })
        .collect();
    assert_eq!(
        chunks,
        [
            ("\x1b_Ga=T,U=1,f=100,i=7,c=3,r=2,m=1,q=2", 4096),
            ("\x1b_Gm=0,q=2", 4),
        ]
    );
}
