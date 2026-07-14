//! クリッピングマスク coverage の共有参照(D7)。
//!
//! `cpu_reference/` 外に置く: M2E-2 同時変更ゲートを避けつつ、doc/nodes テストで式を共有する。
//! 係数は `mask_apply.wgsl` と一致(premul RGB の BT.709 輝度)。

use crate::cpu_reference::to_u8;

/// WGSL `ClippingMaskMode` / doc `MaskMode` と 1:1。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClippingMaskRef {
    Alpha,
    Luminance,
    InvertAlpha,
    InvertLuminance,
}

/// premul RGBA マスク画素 → coverage(WGSL `mask_factor` と同式)。
pub fn clipping_mask_factor(mask: [u8; 4], mode: ClippingMaskRef) -> f64 {
    let r = mask[0] as f64 / 255.0;
    let g = mask[1] as f64 / 255.0;
    let b = mask[2] as f64 / 255.0;
    let a = mask[3] as f64 / 255.0;
    let luma = (0.2126 * r + 0.7152 * g + 0.0722 * b).clamp(0.0, 1.0);
    let f = match mode {
        ClippingMaskRef::Alpha => a,
        ClippingMaskRef::Luminance => luma,
        ClippingMaskRef::InvertAlpha => 1.0 - a,
        ClippingMaskRef::InvertLuminance => 1.0 - luma,
    };
    f.clamp(0.0, 1.0)
}

/// premul content × coverage(WGSL `fs_main` と同式)。
pub fn clipping_mask_mul_u8(
    content: [u8; 4],
    mask: [u8; 4],
    mode: ClippingMaskRef,
) -> [u8; 4] {
    let f = clipping_mask_factor(mask, mode);
    [
        to_u8(content[0] as f64 / 255.0 * f),
        to_u8(content[1] as f64 / 255.0 * f),
        to_u8(content[2] as f64 / 255.0 * f),
        to_u8(content[3] as f64 / 255.0 * f),
    ]
}

/// 同サイズ content/mask バッファへ画素ごと適用。
pub fn clipping_mask_frame(content: &[u8], mask: &[u8], mode: ClippingMaskRef) -> Vec<u8> {
    assert_eq!(content.len(), mask.len());
    assert_eq!(content.len() % 4, 0);
    content
        .chunks_exact(4)
        .zip(mask.chunks_exact(4))
        .flat_map(|(c, m)| {
            clipping_mask_mul_u8([c[0], c[1], c[2], c[3]], [m[0], m[1], m[2], m[3]], mode)
        })
        .collect()
}
