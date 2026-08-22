//! owns: `Vec<motolii_vector::ShapeNode>`(store の意味、`Layer:shapes` component、
//! `StoreView::shapes` が返す形)→ `motolii-vector::render_tree` → premultiplied
//! RGBA8。`text.rs`(裁定190 切片2)と同じ形の橋渡しだが、shape は既に
//! fill/stroke/ops まで確定した記述(`motolii_vector::Shape`/`ShapeNode`)なので、
//! テキストのシェイピング(cosmic-text)に相当する段が要らない分、薄い。
//!
//! # なぜここが必要だったか(発注: シェイプが画に出るようにする)
//!
//! 土台(`motolii-vector` のパス演算子・`Shape`/`ShapeNode`/`Fill`/`Stroke`・
//! [`motolii_vector::render_tree`])は shape-1/2/3・裁定173 H4 で既に揃っており、
//! `mask.rs::rasterize_mask_coverage` がマスクのラスタライズで実証済みだった。
//! それでも `motolii-engine::Engine::texture_for` の `LayerSource::Shape` 枝は
//! 「まだ描画に繋いでいない」という注記付きで常に `(None, [0.0, 0.0])` を返して
//! いた——ここはその「土台はあるが繋がっていない」穴を塞ぐ、`render_tree` を
//! 呼ぶだけの薄いラッパー。呼び手は `lib.rs` の
//! [`Engine::shape_texture_from_shapes`](../struct.Engine.html)。
//!
//! # canvas は `Canvas::centered`(text とは逆の選択)
//!
//! `text.rs` は左上原点の canvas を使う(組版のペン位置がそのまま画素の位置を
//! 決めるので、左上原点が自然)。shape は逆——パス源(`rect`/`ellipse`/
//! `polystar`、`motolii_vector::geom` 参照)は**すべて局所原点 `(0,0)` を中心に
//! 生成される**ので、`Canvas::centered` を使わないと既定(position track の無い)
//! shape layer が comp の左上 1/4 だけに描かれてしまう(中心から負の座標へ伸びる
//! 半分が canvas の外へ落ちる)。
//!
//! # 時刻(`t`)を取らない理由
//!
//! `motolii_vector::Shape`/`ShapeNode` は Hold/`KeyframeTrack` のような時間評価を
//! 一切持たない静的な記述——`TextDocument::content`(Hold 評価、`text.rs`
//! 参照)や mask の `PropertyId::mask_shape`(`Value::Path` の property track)と
//! 違い、`StoreView::shapes(layer)` 自体が時刻を引数に取らない(`motolii-store`
//! の `view.rs` 参照)。つまり shape の頂点が時間で動くとしたら、それは
//! **shape 自身の記述が変わる**からではなく、**layer の transform
//! (`ResolvedLayer.placement`、position/rotation/scale がキーフレーム可能)が
//! 動く**からで、これは `render_with_camera_override`/`layers_from_resolved` が
//! `Layer.placement` をそのまま持ち回る既存の経路がそのまま担う——ここで
//! 新しい時間軸を作る必要は無い(`tests/shape_layer.rs::keyframed_position_
//! moves_the_rendered_shape_over_time` が実測で固定する)。

use motolii_store::ShapeNode;
use motolii_vector::{Canvas, Raster, VectorError};

/// `shapes` を `canvas` へラスタライズする。**空なら `Ok(None)`**(`SetShapes` が
/// 一度も呼ばれていない、または空配列で明示的に差し替えられた——どちらもエラー
/// ではない、`text::rasterize_text_document` の「描く物が無い」扱いと同じ形)。
pub fn rasterize_shapes(
    shapes: &[ShapeNode],
    canvas: &Canvas,
) -> Result<Option<Raster>, VectorError> {
    if shapes.is_empty() {
        return Ok(None);
    }
    Ok(Some(motolii_vector::render_tree(shapes, canvas)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_vector::{Brush, Fill, FillRule, PathSource, Point, Rgb, Shape, Stroke};

    fn canvas(w: u32, h: u32) -> Canvas {
        Canvas::centered(w, h)
    }

    fn visible_pixels(raster: &Raster) -> usize {
        raster
            .premultiplied_rgba8
            .chunks_exact(4)
            .filter(|p| p[3] > 0)
            .count()
    }

    fn filled_rect(size: f64, rgb: Rgb) -> Shape {
        Shape {
            source: PathSource::Rectangle {
                size: Point { x: size, y: size },
            },
            ops: Vec::new(),
            fill: Some(Fill {
                brush: Brush::Solid(rgb),
                rule: FillRule::NonZero,
                opacity: 1.0,
                hidden: false,
            }),
            stroke: None,
        }
    }

    #[test]
    fn empty_shape_list_yields_no_texture() {
        let result = rasterize_shapes(&[], &canvas(64, 64)).expect("no error");
        assert!(result.is_none());
    }

    #[test]
    fn rectangle_with_fill_produces_visible_pixels() {
        let shape = filled_rect(
            40.0,
            Rgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
        );
        let raster = rasterize_shapes(&[ShapeNode::Leaf(shape)], &canvas(64, 64))
            .expect("render")
            .expect("非空の shape 列は Some のはず");
        assert!(visible_pixels(&raster) > 1_000);
        // fill 色(赤)が premultiplied のまま画素へ届いている。
        assert!(raster
            .premultiplied_rgba8
            .chunks_exact(4)
            .any(|p| p[0] > 200 && p[1] == 0 && p[2] == 0 && p[3] > 200));
    }

    #[test]
    fn ellipse_with_stroke_only_produces_visible_pixels() {
        let shape = Shape {
            source: PathSource::Ellipse {
                size: Point { x: 40.0, y: 40.0 },
            },
            ops: Vec::new(),
            fill: None,
            stroke: Some(Stroke {
                brush: Brush::Solid(Rgb {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                }),
                width: 6.0,
                opacity: 1.0,
                ..Stroke::default()
            }),
        };
        let raster = rasterize_shapes(&[ShapeNode::Leaf(shape)], &canvas(64, 64))
            .expect("render")
            .expect("非空の shape 列は Some のはず");
        assert!(visible_pixels(&raster) > 50);
    }

    /// **中心 canvas の証拠**: 局所原点 `(0,0)` に置かれた矩形は、`Canvas::centered`
    /// なら canvas 中央付近に画素を持つ——左上原点だったら見えるはずの
    /// canvas 左上隅(0,0 近傍)には画素が無いことも合わせて確かめる。
    #[test]
    fn shape_is_centered_on_the_canvas_not_anchored_to_the_top_left() {
        let shape = filled_rect(
            20.0,
            Rgb {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
        );
        let raster = rasterize_shapes(&[ShapeNode::Leaf(shape)], &canvas(64, 64))
            .expect("render")
            .expect("非空の shape 列は Some のはず");
        let pixel = |x: u32, y: u32| -> [u8; 4] {
            let i = ((y * raster.width + x) * 4) as usize;
            [
                raster.premultiplied_rgba8[i],
                raster.premultiplied_rgba8[i + 1],
                raster.premultiplied_rgba8[i + 2],
                raster.premultiplied_rgba8[i + 3],
            ]
        };
        assert!(pixel(32, 32)[3] > 200, "canvas 中央には塗りが乗るはず");
        assert_eq!(pixel(1, 1)[3], 0, "canvas 左上隅は 20x20 の矩形の外のはず");
    }
}
