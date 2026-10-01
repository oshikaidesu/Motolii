//! 種から決まる乱れ — 同じ (種, 番号, 欄) には必ず同じ値。時刻も順番も持たない純関数。
//! 配置のばらつき、形の縁の揺らぎ、粒の散り方が同じ源を使う(別々に書くと、同じ種で違う絵になる)。

/// [-1, 1] の一様な値。`channel` は 1 つの物が複数の乱れを要る時の欄(x と y、大きさと角度)。
pub fn noise(seed: u64, index: u32, channel: u64) -> f64 {
    let mut z = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(u64::from(index).wrapping_mul(0xBF58_476D_1CE4_E5B9))
        .wrapping_add(channel.wrapping_mul(0x94D0_49BB_1331_11EB))
        .wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
}
