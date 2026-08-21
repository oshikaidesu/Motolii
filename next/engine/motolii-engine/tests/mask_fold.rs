//! 裁定160 発注γ(MK2)のオラクル — engine 統合: `ResolvedMask` 列を order で畳んで
//! 最終 coverage を作る。
//!
//! MK1(`mask_rasterize.rs`)が縛った「1つの mask 形状 → 1枚の coverage」を土台に、
//! ここでは複数 mask を `motolii_vector::coverage` の代数で order 通りに畳む所だけを見る。
//! GPU 合成(coverage × layer 本体)は MK3(NON-GOALS、このレーンでは触らない) —
//! `fold_masks` は coverage を返すだけで compositor は一切呼ばない。

use motolii_engine::mask::fold_masks;
use motolii_store::{MaskMode, Path, PathVertex, ResolvedMask};
use motolii_vector::Canvas;

/// 中央原点、一辺 `2*half` の軸平行正方形マスク。
fn square_mask(mode: MaskMode, inverted: bool, opacity: f32, half: f64) -> ResolvedMask {
    let corner = |x: f64, y: f64| PathVertex {
        point: [x, y],
        in_tangent: [0.0, 0.0],
        out_tangent: [0.0, 0.0],
    };
    ResolvedMask {
        mode,
        inverted,
        opacity,
        shape: Path {
            vertices: vec![
                corner(-half, -half),
                corner(half, -half),
                corner(half, half),
                corner(-half, half),
            ],
            closed: true,
        },
    }
}

fn canvas_100() -> Canvas {
    Canvas::centered(100, 100)
}

fn coverage_at(coverage: &motolii_vector::coverage::Coverage, x: u32, y: u32) -> u8 {
    coverage.bytes[(y * coverage.width + x) as usize]
}

/// mask が1枚も無ければ全通過 — layer は覆いを一切受けない
/// (R9「先頭は常に無mask=全通過」の 0 枚ケース)。
#[test]
fn no_masks_means_full_coverage_everywhere() {
    let coverage = fold_masks(&[], &canvas_100()).expect("空リストは畳めるはず");
    assert_eq!(coverage.width, 100);
    assert_eq!(coverage.height, 100);
    assert!(
        coverage.bytes.iter().all(|&b| b == 255),
        "mask 0枚なのに全通過(255)になっていない画素がある"
    );
}

/// 単体 mask(mode=Add)は、そのまま mask 形状の coverage になる —
/// 先頭マスクの単位元が add/lighten では `0`(空)であることの確認。
#[test]
fn a_single_add_mask_alone_reduces_to_its_own_shape() {
    let mask = square_mask(MaskMode::Add, false, 1.0, 25.0);
    let coverage = fold_masks(&[mask], &canvas_100()).expect("1枚は畳めるはず");

    assert_eq!(
        coverage_at(&coverage, 50, 50),
        255,
        "中心(内側)が不透明でない"
    );
    assert_eq!(coverage_at(&coverage, 5, 5), 0, "左上(外側)が透明でない");
}

/// 単体 mask(mode=Subtract)だけを置くと、**外側が見える穴あき**になる —
/// 先頭マスクの単位元が subtract 系では `255`(全通過)であることの確認
/// (AE の「先頭マスクが Subtract だと内側に穴が開く」実機挙動と同型、
/// `coverage.rs` module doc 参照)。
#[test]
fn a_single_subtract_mask_alone_punches_a_hole_leaving_the_outside_visible() {
    let mask = square_mask(MaskMode::Subtract, false, 1.0, 25.0);
    let coverage = fold_masks(&[mask], &canvas_100()).expect("1枚は畳めるはず");

    assert_eq!(
        coverage_at(&coverage, 50, 50),
        0,
        "中心(mask 内側)に穴が開いていない"
    );
    assert_eq!(
        coverage_at(&coverage, 5, 5),
        255,
        "外側が全通過のままでない"
    );
}

/// (c) engine 統合オラクル本体 — 2 mask(Add → Subtract)の合成が手計算 golden と一致。
///
/// mask1: Add、半径35(pixel [15,85] の正方形)。
/// mask2: Subtract、半径25(pixel [25,75] の正方形)。境界から5px以上離れた点だけを
/// 見る(MK1 `mask_rasterize.rs` の「境界1px帯」注記と同じく、AA の余地を避ける)。
///
/// 手計算:
/// - 先頭(Add)の単位元は空(`0`)なので、mask1 畳み込み後の accumulator は
///   mask1 の coverage そのもの(mask1 の内側で 255、外側で 0)。
/// - 続く Subtract は `acc.saturating_sub(mask2)` — mask1 の内側から mask2 の
///   coverage を引く。両方の内側(中央)は 255-255=0、mask1 内側だが mask2 外側は
///   255-0=255、mask1 の外側はそもそも 0 のまま(0-0=0)。
#[test]
fn two_masks_add_then_subtract_match_hand_computed_golden() {
    let mask1 = square_mask(MaskMode::Add, false, 1.0, 35.0);
    let mask2 = square_mask(MaskMode::Subtract, false, 1.0, 25.0);
    let coverage = fold_masks(&[mask1, mask2], &canvas_100()).expect("2枚は畳めるはず");

    // 中央(mask1 の内側 かつ mask2 の内側) — 引かれて 0。
    assert_eq!(
        coverage_at(&coverage, 50, 50),
        0,
        "中央が穴になっていない(golden: 0)"
    );
    // mask1 の内側(pixel 20 は [15,85] 内、境界から5px)だが mask2 の外側
    // (pixel 20 は [25,75] 外) — 255-0=255 のまま残るはず。
    assert_eq!(
        coverage_at(&coverage, 20, 50),
        255,
        "mask1 内側 かつ mask2 外側の輪が golden(255)と一致しない"
    );
    // mask1 の外側(pixel 5 は [15,85] 外) — Add 前から 0 のまま。
    assert_eq!(
        coverage_at(&coverage, 5, 5),
        0,
        "mask1 の外側が golden(0)と一致しない"
    );
}
