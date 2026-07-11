//! 合成モードのCPU参照(受け入れテストの期待画素)。

pub fn to_u8(v: f64) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub fn premul_over_u8(bg: [u8; 4], fg: [u8; 4]) -> [u8; 4] {
    let bg = bg.map(|v| v as f64 / 255.0);
    let fg = fg.map(|v| v as f64 / 255.0);
    let inv_a = 1.0 - fg[3];
    [
        to_u8(fg[0] + bg[0] * inv_a),
        to_u8(fg[1] + bg[1] * inv_a),
        to_u8(fg[2] + bg[2] * inv_a),
        to_u8(fg[3] + bg[3] * inv_a),
    ]
}

pub fn premul_add_u8(bg: [u8; 4], fg: [u8; 4]) -> [u8; 4] {
    let bg = bg.map(|v| v as f64 / 255.0);
    let fg = fg.map(|v| v as f64 / 255.0);
    [
        to_u8((fg[0] + bg[0]).min(1.0)),
        to_u8((fg[1] + bg[1]).min(1.0)),
        to_u8((fg[2] + bg[2]).min(1.0)),
        to_u8((fg[3] + bg[3]).min(1.0)),
    ]
}

pub fn premul_multiply_u8(bg: [u8; 4], fg: [u8; 4]) -> [u8; 4] {
    let bg = bg.map(|v| v as f64 / 255.0);
    let fg = fg.map(|v| v as f64 / 255.0);
    let inv_fg_a = 1.0 - fg[3];
    let inv_bg_a = 1.0 - bg[3];
    [
        to_u8(fg[0] * inv_bg_a + bg[0] * inv_fg_a + fg[0] * bg[0]),
        to_u8(fg[1] * inv_bg_a + bg[1] * inv_fg_a + fg[1] * bg[1]),
        to_u8(fg[2] * inv_bg_a + bg[2] * inv_fg_a + fg[2] * bg[2]),
        to_u8(fg[3] + bg[3] * inv_fg_a),
    ]
}
