//! 撮った絵を目視でなく画素で審判するための道具。
//!
//! 素材の色は「支配的なチャンネル」で見分けられるように選んである。gamma の
//! 掛かり方や合成の丸めで数値が動いても、赤が緑になることはない。

use std::path::Path;

use image::ImageEncoder as _;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hue {
    Red,
    Green,
    Blue,
    Yellow,
    /// Rerun の既定背景のような暗い色。レイヤーが「無い」ことの印。
    Dark,
    Other,
}

impl std::fmt::Display for Hue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Red => "red",
            Self::Green => "green",
            Self::Blue => "blue",
            Self::Yellow => "yellow",
            Self::Dark => "dark",
            Self::Other => "other",
        };
        f.write_str(name)
    }
}

const HIGH: u8 = 140;
const LOW: u8 = 110;

pub fn hue(rgb: [u8; 3]) -> Hue {
    let [r, g, b] = rgb;
    if r < 90 && g < 90 && b < 90 {
        return Hue::Dark;
    }
    match (r >= HIGH, g >= HIGH, b >= HIGH) {
        (true, false, false) if g < LOW && b < LOW => Hue::Red,
        (false, true, false) if r < LOW && b < LOW => Hue::Green,
        (false, false, true) if r < LOW && g < LOW => Hue::Blue,
        (true, true, false) if b < LOW => Hue::Yellow,
        _ => Hue::Other,
    }
}

pub struct Frame {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Frame {
    pub fn new(rgba: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            rgba,
            width,
            height,
        }
    }

    pub fn rgb(&self, x: i32, y: i32) -> Option<[u8; 3]> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        let index = ((y as u32 * self.width + x as u32) * 4) as usize;
        Some([
            self.rgba[index],
            self.rgba[index + 1],
            self.rgba[index + 2],
        ])
    }

    /// 1点だけ見ると縁や補間に当たる。3x3 の多数決で読む。
    pub fn hue_at(&self, x: i32, y: i32) -> Hue {
        let mut counts = [0u32; 6];
        for dy in -1..=1 {
            for dx in -1..=1 {
                let Some(rgb) = self.rgb(x + dx, y + dy) else {
                    continue;
                };
                let index = match hue(rgb) {
                    Hue::Red => 0,
                    Hue::Green => 1,
                    Hue::Blue => 2,
                    Hue::Yellow => 3,
                    Hue::Dark => 4,
                    Hue::Other => 5,
                };
                counts[index] += 1;
            }
        }
        let best = counts
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| **count)
            .map(|(index, _)| index)
            .unwrap_or(5);
        [
            Hue::Red,
            Hue::Green,
            Hue::Blue,
            Hue::Yellow,
            Hue::Dark,
            Hue::Other,
        ][best]
    }

    pub fn hue_histogram(&self) -> [u32; 6] {
        let mut counts = [0u32; 6];
        for pixel in self.rgba.chunks_exact(4) {
            let index = match hue([pixel[0], pixel[1], pixel[2]]) {
                Hue::Red => 0,
                Hue::Green => 1,
                Hue::Blue => 2,
                Hue::Yellow => 3,
                Hue::Dark => 4,
                Hue::Other => 5,
            };
            counts[index] += 1;
        }
        counts
    }

    pub fn digest(&self) -> u64 {
        fnv1a(&self.rgba)
    }

    pub fn write_png(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("create {}: {error}", parent.display()))?;
        }
        let file = std::fs::File::create(path)
            .map_err(|error| format!("create {}: {error}", path.display()))?;
        image::codecs::png::PngEncoder::new(std::io::BufWriter::new(file))
            .write_image(
                &self.rgba,
                self.width,
                self.height,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|error| format!("encode {}: {error}", path.display()))
    }
}

/// 依存を増やさずに「同じ絵か」を言うためだけのハッシュ。
/// 暗号学的な主張はしない。外向きの証拠は PNG の sha256 で取る。
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadrant_colors_classify_as_themselves() {
        assert_eq!(hue([230, 30, 30]), Hue::Red);
        assert_eq!(hue([30, 230, 30]), Hue::Green);
        assert_eq!(hue([30, 30, 230]), Hue::Blue);
        assert_eq!(hue([230, 230, 30]), Hue::Yellow);
    }

    #[test]
    fn gamma_shifted_colors_keep_their_class() {
        // gamma 空間へ持ち上げても支配チャンネルは変わらない。
        assert_eq!(hue([247, 96, 96]), Hue::Red);
        assert_eq!(hue([96, 247, 96]), Hue::Green);
    }

    #[test]
    fn dark_background_is_not_a_layer() {
        assert_eq!(hue([13, 15, 18]), Hue::Dark);
        assert_eq!(hue([40, 44, 52]), Hue::Dark);
    }
}
