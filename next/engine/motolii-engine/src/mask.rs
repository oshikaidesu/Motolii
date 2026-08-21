//! mask 消化(裁定160 発注γ)の第1切片 — `motolii-vector` の CPU ラスタライザを
//! engine から呼ぶ最初の配線。
//!
//! **ここが持つのは「1つの mask 形状 → 1枚の coverage(alpha-only)Raster」だけ**。
//! [`ResolvedMask::mode`] / [`ResolvedMask::inverted`] / [`ResolvedMask::opacity`] は
//! 読まない — 手前までの覆いとどう重ねるかという被覆代数は次の切片(MK2)の仕事で、
//! ここで先取りすると「1枚の shape を塗る」以上の意味をこの module に持たせてしまう
//! (このレーンの EXACT TARGET 2 — mode/inverted/opacity の適用はしない)。
//!
//! GPU へは上げない。`motolii_compositor::Compositor::upload_rgba` を呼ぶのは
//! 合成パスと一緒に配線する MK3 の仕事(R9 の切片割り MK1→MK2→MK3)。

use motolii_store::{Path as EvalPath, ResolvedMask};
use motolii_vector::{Brush, Canvas, Fill, FillRule, PathSource, Raster, Rgb, Shape, VectorError};

/// マスク形状の正本 [`motolii_eval::Path`](motolii_store::Path)(単一輪郭・頂点+接線の列)を、
/// `motolii-vector` の入力形 [`motolii_vector::Path`](輪郭の列)へ橋渡しする。
///
/// **変換であって新しい型ではない** — 両方とも「頂点+接線+閉路フラグ」の同じ中身を
/// 持っており、`motolii_eval::Path` は「1輪郭ぶんの頂点列」、`motolii_vector::Path`
/// (`= Vec<Contour>`)は「輪郭の列」なので、ここでは前者を後者の**1輪郭だけの列**に包む。
/// マスク形状は `Value::Path` 1個(裁定78 の doc「`List` の入れ子は禁止」)なので、
/// 複数輪郭を束ねる必要はここでは生じない。
fn eval_path_to_vector_path(path: &EvalPath) -> motolii_vector::Path {
    let vertices = path
        .vertices
        .iter()
        .map(|v| motolii_vector::Vertex {
            point: motolii_vector::Point {
                x: v.point[0],
                y: v.point[1],
            },
            in_tangent: motolii_vector::Point {
                x: v.in_tangent[0],
                y: v.in_tangent[1],
            },
            out_tangent: motolii_vector::Point {
                x: v.out_tangent[0],
                y: v.out_tangent[1],
            },
        })
        .collect();
    vec![motolii_vector::Contour {
        vertices,
        closed: path.closed,
    }]
}

/// [`ResolvedMask::shape`] を `canvas` の座標系でラスタライズし、被覆率(coverage)だけを
/// 持つ [`Raster`] を返す。
///
/// 塗りは不透明な白の単色 fill を内部で固定して使う。`Raster` は premultiplied RGBA8
/// (`motolii_vector` の唯一の出口 [`motolii_vector::render`] の doc 参照)なので、
/// 白(`r=g=b=1.0`)を不透明度1.0で塗ると **RGB の各チャンネルは alpha と同じ値になる**
/// (`color * alpha` = `1.0 * alpha` = `alpha`) — つまりどのチャンネルを読んでも
/// coverage そのものが読める。呼び手は alpha チャンネル(4バイトごとの4つ目)を
/// coverage として読めばよい。
///
/// `mode` / `inverted` / `opacity` は読まない(module doc 参照 — MK2 の仕事)。
pub fn rasterize_mask_coverage(
    mask: &ResolvedMask,
    canvas: &Canvas,
) -> Result<Raster, VectorError> {
    let shape = Shape {
        source: PathSource::Bezier(eval_path_to_vector_path(&mask.shape)),
        ops: Vec::new(),
        fill: Some(Fill {
            brush: Brush::Solid(Rgb {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            }),
            rule: FillRule::NonZero,
            opacity: 1.0,
            hidden: false,
        }),
        stroke: None,
    };
    motolii_vector::render(&shape, canvas)
}
