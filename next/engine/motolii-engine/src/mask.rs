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
//!
//! # MK2 — 複数 mask の被覆合成
//!
//! [`fold_masks`] がこの切片の仕事。`ResolvedMask` の列を **order 通り**に畳んで
//! 1枚の最終 coverage を作る。代数そのもの(add/subtract/intersect/lighten/darken/
//! difference の画素毎の式と単位元)は `motolii_vector::coverage` の純関数
//! (裁定160 発注γ「代数は motolii-vector」)——ここは `mode`/`inverted`/`opacity`
//! を読んで、その代数へ配る「畳み込み」だけを持つ。GPU 合成(coverage × layer 本体)
//! は引き続き MK3 の仕事(NON-GOALS、このレーンでは触らない)。

use motolii_store::{MaskMode, Path as EvalPath, ResolvedMask};
use motolii_vector::coverage::{self, Coverage};
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

// ---------------------------------------------------------------------------
// MK2 — 複数 mask の被覆合成
// ---------------------------------------------------------------------------

/// [`fold_masks`] の失敗理由。ラスタライズ失敗([`VectorError`])と、代数側の寸法
/// 不一致([`coverage::CoverageSizeMismatch`])の両方をここへ畳む。寸法不一致は
/// 通常起き得ない(全 mask を同じ `canvas` からラスタライズしているので coverage は
/// 全部同じ寸法になる)が、黙って panic させず経路だけ持たせておく(裁定37と同じ形)。
#[derive(Debug, thiserror::Error)]
pub enum MaskFoldError {
    #[error(transparent)]
    Rasterize(#[from] VectorError),
    #[error(transparent)]
    CoverageSize(#[from] coverage::CoverageSizeMismatch),
}

/// マスク1枚の raw coverage(shape の内外だけ)へ、そのマスク自身の `inverted`/
/// `opacity` を適用してから mode の畳み込みへ渡す。
///
/// - `inverted`: 覆いの内外を入れ替える(`255 - byte`)。**`Subtract` とは別物**
///   ([`Mask::inverted`](motolii_store::Mask) の doc と同じ注記— 反転はこの
///   マスク自身の coverage を裏返すだけで、手前の accumulator へは触れない)。
/// - `opacity`: 比を掛けて丸める。`ResolvedMask::opacity` は既に `0.0..=1.0` へ
///   clamp 済み(`motolii-store` の `view.rs` `resolved_masks` 参照)なので、
///   ここでの掛け算は飽和(255超え)を起こさない。
fn adjust_for_composition(raw: &Coverage, inverted: bool, opacity: f32) -> Coverage {
    let bytes = raw
        .bytes
        .iter()
        .map(|&b| {
            let v = if inverted { 255 - b } else { b };
            (v as f32 * opacity).round().clamp(0.0, 255.0) as u8
        })
        .collect();
    Coverage {
        width: raw.width,
        height: raw.height,
        bytes,
    }
}

/// 先頭マスクの「見えない手前の覆い」— mode ごとの単位元。
/// `motolii_vector::coverage` module doc の「先頭マスクの単位元」節に理由がある。
fn identity_for(mode: MaskMode, width: u32, height: u32) -> Coverage {
    match mode {
        MaskMode::Add | MaskMode::Lighten => Coverage::empty(width, height),
        MaskMode::Subtract | MaskMode::Intersect | MaskMode::Darken | MaskMode::Difference => {
            Coverage::full(width, height)
        }
    }
}

/// `mode` へ応じて `motolii_vector::coverage` の対応する純関数へ配るだけの窓口。
fn apply_mode(
    mode: MaskMode,
    acc: &Coverage,
    mask: &Coverage,
) -> Result<Coverage, coverage::CoverageSizeMismatch> {
    match mode {
        MaskMode::Add => coverage::add(acc, mask),
        MaskMode::Subtract => coverage::subtract(acc, mask),
        MaskMode::Intersect => coverage::intersect(acc, mask),
        MaskMode::Lighten => coverage::lighten(acc, mask),
        MaskMode::Darken => coverage::darken(acc, mask),
        MaskMode::Difference => coverage::difference(acc, mask),
    }
}

/// `masks` を **order 通り**に畳んで、layer 全体に効く最終 coverage を1枚返す。
///
/// マスクが1枚も無ければ [`Coverage::full`](全通過 — layer は覆いを一切受けない)。
/// 1枚以上あれば、各マスクを `canvas` へラスタライズ → `inverted`/`opacity` を
/// 適用([`adjust_for_composition`]) → 手前までの accumulator と自分の `mode` で
/// 合成([`apply_mode`])、を順番に繰り返す。先頭マスクの「手前の accumulator」は
/// 固定値ではなく [`identity_for`] — `motolii_vector::coverage` module doc の
/// 「先頭マスクの単位元」節が理由を持つ。
///
/// GPU へは上げない(module doc 参照)。coverage をどう layer 本体と合成するかは
/// MK3 の仕事で、ここは畳んだ coverage を返すだけ。
pub fn fold_masks(masks: &[ResolvedMask], canvas: &Canvas) -> Result<Coverage, MaskFoldError> {
    let mut acc: Option<Coverage> = None;
    for mask in masks {
        let raster = rasterize_mask_coverage(mask, canvas)?;
        let raw = Coverage::from_raster_alpha(&raster);
        let adjusted = adjust_for_composition(&raw, mask.inverted, mask.opacity);
        let next = match acc.as_ref() {
            Some(prev) => apply_mode(mask.mode, prev, &adjusted)?,
            None => {
                let identity = identity_for(mask.mode, canvas.width, canvas.height);
                apply_mode(mask.mode, &identity, &adjusted)?
            }
        };
        acc = Some(next);
    }
    Ok(acc.unwrap_or_else(|| Coverage::full(canvas.width, canvas.height)))
}
