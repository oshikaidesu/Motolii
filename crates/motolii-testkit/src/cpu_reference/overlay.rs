//! 矩形オーバーレイのCPU期待値。正準→pxはここに式を書き下ろす(製品変換に依存しない)。

use motolii_core::FrameDesc;

/// 正準空間の矩形(`center`/`size`、高さ=1.0基準・Y-up・原点中央)をRGBAに焼き込む。
///
/// 変換式は`ViewportTransform`と同等だが、参照側が製品コードを呼ぶ循環を避けるため
/// ここへ複製する(監査E-6)。式を変えるときは製品側とセットでレビューする。
pub fn expected_rect_frame(
    desc: FrameDesc,
    bg: [u8; 4],
    fg: [u8; 4],
    center: [f64; 2],
    size: [f64; 2],
) -> Vec<u8> {
    let h = desc.height as f64;
    let center_x = desc.width as f64 * 0.5 + center[0] * h;
    let center_y = desc.height as f64 * 0.5 - center[1] * h;
    let size_w = size[0] * h;
    let size_h = size[1] * h;
    let min_x = center_x - size_w * 0.5;
    let max_x = center_x + size_w * 0.5;
    let min_y = center_y - size_h * 0.5;
    let max_y = center_y + size_h * 0.5;
    let mut out = vec![0u8; desc.data_size()];
    for y in 0..desc.height {
        for x in 0..desc.width {
            let cx = x as f64 + 0.5;
            let cy = y as f64 + 0.5;
            let inside = cx >= min_x && cx < max_x && cy >= min_y && cy < max_y;
            let i = ((y * desc.width + x) * 4) as usize;
            out[i..i + 4].copy_from_slice(if inside { &fg } else { &bg });
        }
    }
    out
}

/// 既存パターン上に正準矩形を上書きした期待値。
pub fn expected_rect_over_pattern(
    desc: FrameDesc,
    base: &[u8],
    color: [u8; 4],
    center: [f64; 2],
    size: [f64; 2],
) -> Vec<u8> {
    let mut out = base.to_vec();
    let h = desc.height as f64;
    let center_x = desc.width as f64 * 0.5 + center[0] * h;
    let center_y = desc.height as f64 * 0.5 - center[1] * h;
    let rect_w = size[0] * h;
    let rect_h = size[1] * h;
    let min_x = center_x - rect_w * 0.5;
    let max_x = center_x + rect_w * 0.5;
    let min_y = center_y - rect_h * 0.5;
    let max_y = center_y + rect_h * 0.5;

    for y in 0..desc.height {
        for x in 0..desc.width {
            let px = x as f64 + 0.5;
            let py = y as f64 + 0.5;
            if px >= min_x && px < max_x && py >= min_y && py < max_y {
                let i = ((y * desc.width + x) * 4) as usize;
                out[i..i + 4].copy_from_slice(&color);
            }
        }
    }
    out
}

/// 既存パターン上に正準円を上書きした期待値。
pub fn expected_circle_over_pattern(
    desc: FrameDesc,
    base: &[u8],
    color: [u8; 4],
    center: [f64; 2],
    radius: f64,
) -> Vec<u8> {
    let mut out = base.to_vec();
    let h = desc.height as f64;
    let center_x = desc.width as f64 * 0.5 + center[0] * h;
    let center_y = desc.height as f64 * 0.5 - center[1] * h;
    let radius_px = radius * h;
    let radius_sq = radius_px * radius_px;

    for y in 0..desc.height {
        for x in 0..desc.width {
            let px = x as f64 + 0.5;
            let py = y as f64 + 0.5;
            let dx = px - center_x;
            let dy = py - center_y;
            if dx * dx + dy * dy < radius_sq {
                let i = ((y * desc.width + x) * 4) as usize;
                out[i..i + 4].copy_from_slice(&color);
            }
        }
    }
    out
}

/// 既存パターン上に正準線分を上書きした期待値。
///
/// しきい値±EPSの不定域は`actual`を採用して比較から外す(mesa丸め対策)。
pub fn expected_line_over_pattern(
    desc: FrameDesc,
    base: &[u8],
    color: [u8; 4],
    start: [f64; 2],
    end: [f64; 2],
    width: f64,
    actual: &[u8],
) -> Vec<u8> {
    const EDGE_EPS: f32 = 1e-3;
    let mut out = base.to_vec();
    let h = desc.height as f32;
    let center_x = desc.width as f32 * 0.5;
    let center_y = desc.height as f32 * 0.5;
    let start_x = center_x + start[0] as f32 * h;
    let start_y = center_y - start[1] as f32 * h;
    let end_x = center_x + end[0] as f32 * h;
    let end_y = center_y - end[1] as f32 * h;
    let half_width = width as f32 * h * 0.5;

    for y in 0..desc.height {
        for x in 0..desc.width {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let dist = dist_to_segment_f32(px, py, start_x, start_y, end_x, end_y);
            let i = ((y * desc.width + x) * 4) as usize;
            if (dist - half_width).abs() < EDGE_EPS {
                out[i..i + 4].copy_from_slice(&actual[i..i + 4]);
            } else if dist < half_width {
                out[i..i + 4].copy_from_slice(&color);
            }
        }
    }
    out
}

fn dist_to_segment_f32(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let abx = bx - ax;
    let aby = by - ay;
    let denom = abx * abx + aby * aby;
    let (cx, cy) = if denom <= 1e-8 {
        (ax, ay)
    } else {
        let t = ((px - ax) * abx + (py - ay) * aby) / denom;
        let t = t.clamp(0.0, 1.0);
        (ax + abx * t, ay + aby * t)
    };
    let dx = px - cx;
    let dy = py - cy;
    (dx * dx + dy * dy).sqrt()
}
