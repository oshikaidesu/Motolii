//! BT.709 limitedのグレーRGB→輝度Yの理論値。

/// グレーRGB`(g,g,g)`のBT.709 limited輝度: `Y = 16 + 219*g/255`(グレーは係数に依らない)。
pub fn expected_luma(gray: u8) -> i32 {
    (16.0 + 219.0 * gray as f64 / 255.0).round() as i32
}
