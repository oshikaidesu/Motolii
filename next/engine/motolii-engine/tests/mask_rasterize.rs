//! 裁定160 発注γ(MK1)のオラクル。
//!
//! `motolii-vector` の CPU ラスタライザを engine から呼ぶ最初の切片。ここで縛るのは
//! 「1つの mask 形状 → 1枚の coverage(alpha-only) Raster」だけ — mode/inverted/opacity
//! の適用(被覆代数)は MK2、GPU 合成は MK3(NON-GOALS、このレーンでは触らない)。

use motolii_engine::mask::rasterize_mask_coverage;
use motolii_store::{MaskMode, Path, PathVertex, ResolvedMask};
use motolii_vector::Canvas;

/// 中央原点の正方形マスク(一辺 `2*half`)。頂点はコーナー(タンジェント無し)なので
/// 辺は直線 — ラスタライズ結果が軸平行矩形になることを前提に試験できる。
fn square_mask(half: f64) -> ResolvedMask {
    let corner = |x: f64, y: f64| PathVertex {
        point: [x, y],
        in_tangent: [0.0, 0.0],
        out_tangent: [0.0, 0.0],
    };
    ResolvedMask {
        mode: MaskMode::Add,
        inverted: false,
        opacity: 1.0,
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

/// 100x100、局所原点が中央(裁定EXACT TARGETの例と同じ)。
fn canvas_100() -> Canvas {
    Canvas::centered(100, 100)
}

fn alpha_at(raster: &motolii_vector::Raster, x: u32, y: u32) -> u8 {
    let i = ((y * raster.width + x) * 4 + 3) as usize;
    raster.premultiplied_rgba8[i]
}

/// (b) 内側=255・外側=0。中心50x50の矩形(canvas 中央±25)。
#[test]
fn inside_is_opaque_outside_is_transparent() {
    let mask = square_mask(25.0);
    let raster = rasterize_mask_coverage(&mask, &canvas_100()).expect("rasterize failed");

    assert_eq!(raster.width, 100);
    assert_eq!(raster.height, 100);
    assert_eq!(raster.premultiplied_rgba8.len(), (100 * 100 * 4) as usize);

    // 中心 — 完全に内側。
    assert_eq!(alpha_at(&raster, 50, 50), 255, "中心が不透明でない");
    // 四隅近く — 完全に外側(矩形は 25..75 なので 5,5 は十分外)。
    assert_eq!(alpha_at(&raster, 5, 5), 0, "左上隅が透明でない");
    assert_eq!(alpha_at(&raster, 94, 5), 0, "右上隅が透明でない");
    assert_eq!(alpha_at(&raster, 5, 94), 0, "左下隅が透明でない");
    assert_eq!(alpha_at(&raster, 94, 94), 0, "右下隅が透明でない");
}

/// (b) 境界1px帯の外は完全な二値(0 か 255 のどちらかのみ)。矩形の辺はピクセル格子
/// (25 / 75)にちょうど乗るので、AA は起きても境界の直近1pxに限られるはず。
#[test]
fn coverage_is_binary_outside_a_one_pixel_edge_band() {
    let mask = square_mask(25.0);
    let raster = rasterize_mask_coverage(&mask, &canvas_100()).expect("rasterize failed");

    // 矩形境界: x/y ∈ {25, 75}。境界から1px以上離れた画素だけを二値チェックする。
    let is_edge_band = |v: u32| -> bool {
        let edges = [25i64, 75i64];
        edges.iter().any(|e| (v as i64 - e).abs() <= 1)
    };

    for y in 0..raster.height {
        for x in 0..raster.width {
            if is_edge_band(x) || is_edge_band(y) {
                continue;
            }
            let a = alpha_at(&raster, x, y);
            assert!(a == 0 || a == 255, "境界帯の外({x},{y})が二値でない: {a}");
        }
    }
}

/// (b) トータル被覆面積が期待どおり(50x50 = 2500 に対し、AA 誤差は境界1px帯ぶんに有界)。
#[test]
fn total_coverage_area_matches_the_square() {
    let mask = square_mask(25.0);
    let raster = rasterize_mask_coverage(&mask, &canvas_100()).expect("rasterize failed");

    let covered: f64 = (0..raster.height)
        .flat_map(|y| (0..raster.width).map(move |x| (x, y)))
        .map(|(x, y)| alpha_at(&raster, x, y) as f64 / 255.0)
        .sum();

    let expected = 50.0 * 50.0;
    // 境界は一辺あたり最大1px帯 x 4辺ぶんの誤差(200px)を許す。
    assert!(
        (covered - expected).abs() <= 200.0,
        "被覆面積が期待とずれすぎている: got={covered} expected={expected}"
    );
}

/// (c) 同一入力は byte 一致(tiny-skia 側は上流が決定論と自認、ここはその素通しを縛る)。
#[test]
fn same_input_renders_byte_identical_twice() {
    let mask = square_mask(25.0);
    let canvas = canvas_100();
    let a = rasterize_mask_coverage(&mask, &canvas).expect("first render");
    let b = rasterize_mask_coverage(&mask, &canvas).expect("second render");
    assert_eq!(a.premultiplied_rgba8, b.premultiplied_rgba8);
}
