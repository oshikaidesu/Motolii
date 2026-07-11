//! 固定レンダグラフのCPU期待値。

use motolii_core::FrameDesc;

use super::composite::premul_over_u8;

/// `centered_request`相当の固定オーバーレイ合成の理論RGBA。
pub fn expected_fixed_graph(desc: FrameDesc) -> Vec<u8> {
    let bg = [0u8, 128, 0, 128];
    let fg = [128u8, 0, 0, 128];
    let over = premul_over_u8(bg, fg);
    let mut out = vec![0u8; desc.data_size()];
    for y in 0..desc.height {
        for x in 0..desc.width {
            let inside = (3..5).contains(&x) && (1..3).contains(&y);
            let i = ((y * desc.width + x) * 4) as usize;
            out[i..i + 4].copy_from_slice(if inside { &over } else { &bg });
        }
    }
    out
}
