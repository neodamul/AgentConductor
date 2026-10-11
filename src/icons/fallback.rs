//! Original monogram artwork, MIT. No vendor artwork or desktop installation needed.
use crate::protocol::Provider;
pub fn fallback_icon(provider: Provider) -> Option<String> {
    let (color, letters): ([u8; 3], [[u8; 7]; 2]) = match provider {
        Provider::Claude => (
            [177, 100, 76],
            [[14, 17, 16, 16, 16, 17, 14], [16, 16, 16, 16, 16, 16, 31]],
        ),
        Provider::Codex => (
            [52, 103, 91],
            [[14, 17, 16, 16, 16, 17, 14], [17, 17, 10, 4, 10, 17, 17]],
        ),
        Provider::OpenCode => (
            [74, 82, 106],
            [[14, 17, 17, 17, 17, 17, 14], [14, 17, 16, 16, 16, 17, 14]],
        ),
        Provider::Custom => return None,
    };
    let mut rgba = vec![0u8; 32 * 32 * 4];
    for y in 0..32 {
        for x in 0..32 {
            let at = (y * 32 + x) * 4;
            rgba[at..at + 3].copy_from_slice(&color);
            rgba[at + 3] = if (!(3..=28).contains(&x)) && (!(3..=28).contains(&y)) {
                0
            } else {
                255
            };
        }
    }
    for (index, glyph) in letters.iter().enumerate() {
        for (y, row) in glyph.iter().enumerate() {
            for x in 0..5 {
                if row & (1 << (4 - x)) != 0 {
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let at = ((9 + y * 2 + dy) * 32 + 5 + index * 12 + x * 2 + dx) * 4;
                            rgba[at..at + 4].copy_from_slice(&[250, 250, 244, 255]);
                        }
                    }
                }
            }
        }
    }
    super::png_data_url(32, 32, &rgba).ok()
}
