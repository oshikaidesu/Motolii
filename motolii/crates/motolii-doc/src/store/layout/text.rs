//! 文字の箱と流れ — 何文字がどこに落ちるか、行がどこで折れるか、
//! 回り込む相手の箱をどう避けるか。字面そのものは vector/text.rs、ここは置き場の話。

use super::flow::Measure;
use super::*;

/// 文字の折り返しの移り方: Transition を持つ文字は、少し前のコマの組の字の位置との差を区間の重みで混ぜる。
/// 字は元の文字の byte で対にする。前のコマに無い字は動かさない。差が無ければ None。
pub(crate) fn glyph_offsets(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<Option<std::sync::Arc<Vec<[f32; 2]>>>, StoreError> {
    if !view.meta(layer)?.is_some_and(|m| m.source == LayerSource::Text) {
        return Ok(None);
    }
    let samples = view.transition_samples(layer, t)?;
    if samples.is_empty() {
        return Ok(None);
    }
    let Some(comp) = view.composition()? else { return Ok(None) };
    let canvas = crate::doc::vector::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
    // 字は元の文字の byte で対にする(組み直しで行頭の空白が落ちても、隣の字と取り違えない)。
    let glyphs = |at: RationalTime| -> Result<Vec<(usize, [f32; 2])>, StoreError> {
        let Some(document) = view.resolved_text_document(layer, at)? else { return Ok(Vec::new()) };
        let around = flow_around(view, layer, at)?;
        let Ok(Some(shaped)) = crate::doc::store::text_frame::shape_document_around(&document, at, &canvas, around.as_deref().map_or(&[], Vec::as_slice)) else { return Ok(Vec::new()) };
        Ok(shaped.lines.iter().flat_map(|line| line.glyph_bytes.iter().zip(&line.glyph_xs).map(move |(b, x)| (*b, [*x, line.baseline_y]))).collect())
    };
    let now = glyphs(t)?;
    let mut sum = vec![[0.0f32; 2]; now.len()];
    let mut weight = vec![0.0f32; now.len()];
    for (at, w) in samples {
        let past: HashMap<usize, [f32; 2]> = glyphs(at)?.into_iter().collect();
        for (n, (byte, here)) in now.iter().enumerate() {
            if let Some(p) = past.get(byte) {
                sum[n][0] += (p[0] - here[0]) * w;
                sum[n][1] += (p[1] - here[1]) * w;
                weight[n] += w;
            }
        }
    }
    let offsets: Vec<[f32; 2]> = sum.iter().zip(&weight).map(|(s, w)| if *w > 1e-6 { [s[0] / w, s[1] / w] } else { [0.0, 0.0] }).collect();
    Ok(offsets.iter().any(|o| o[0].abs() > 1e-3 || o[1].abs() > 1e-3).then(|| std::sync::Arc::new(offsets)))
}

/// 文字の Split の単位の箱(素材座標、読む順)。Split が None なら空。
pub(crate) fn text_units(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<Vec<[f32; 4]>, StoreError> {
    let split = view.choice(layer, crate::doc::store::names::TEXT_SPLIT, t)?;
    if split == 0 || !view.meta(layer)?.is_some_and(|m| m.source == LayerSource::Text) {
        return Ok(Vec::new());
    }
    let (Some(document), Some(comp)) = (view.resolved_text_document(layer, t)?, view.composition()?) else { return Ok(Vec::new()) };
    let canvas = crate::doc::vector::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
    let Some(shaped) = crate::doc::store::text_frame::shape_document(&document, t, &canvas).ok().flatten() else { return Ok(Vec::new()) };
    Ok(crate::doc::store::text_frame::split_boxes(&document, &shaped, &canvas, document.content.eval(t), split))
}

/// 折り返す文字が避ける物(同じ親で Shape Outside を持つ兄弟)を、文字の枠の座標の多角形で。無ければ None。
/// 物が Transition を持てば、少し前の時刻の形も重みつきで渡す(文字の組みは重みの過半が覆う所を避ける)。
pub(crate) fn flow_around(view: &StoreView<'_>, text: LayerId, t: RationalTime) -> Result<Option<std::sync::Arc<Vec<crate::doc::store::text_frame::Obstacle>>>, StoreError> {
    if !view.meta(text)?.is_some_and(|m| m.source == LayerSource::Text) {
        return Ok(None);
    }
    let parent = view.attrs(text)?.unwrap_or_default().parent;
    let mut to_text: Option<glam::Affine2> = None;
    let mut out = Vec::new();
    for layer in view.layers() {
        if layer == text || view.attrs(layer)?.unwrap_or_default().parent != parent {
            continue;
        }
        let mode = view.choice(layer, SHAPE_OUTSIDE, t)?;
        if mode == 0 || !view.here(layer, t)? {
            continue;
        }
        if to_text.is_none() {
            if view.resolved_text_document(text, t)?.and_then(|d| d.wrap_size).is_none() {
                return Ok(None);
            }
            to_text = Some(view.world_2d(text, t)?.inverse());
        }
        let to_text = to_text.unwrap_or(glam::Affine2::IDENTITY);
        let margin = view.number(layer, SHAPE_MARGIN, 0.0, t)?.max(0.0) as f32;
        let mut samples = view.transition_samples(layer, t)?;
        if samples.is_empty() {
            samples.push((t, 1.0));
        }
        for (at, weight) in samples {
            for poly in view.declared_shape(layer, mode, at)? {
                if poly.len() >= 3 {
                    out.push(crate::doc::store::text_frame::Obstacle { margin, weight, points: poly.into_iter().map(|p| to_text.transform_point2(p).to_array()).collect() });
                }
            }
        }
    }
    Ok((!out.is_empty()).then(|| std::sync::Arc::new(out)))
}

/// 文字の行の箱。`wrap` があればその幅で折り返して組み、横は [0, wrap](CSS の block の幅、揃えはその中)。
/// 文字の層の字形の輪郭(`text_box` と同じ座標)。文字はベクターなので、当たりは絵の透過ではなく
/// 形の層と同じ輪郭の道で取る(利用者 2026-09-16「文字の透過は svg ルートなんだから普通にできそう」)。
pub fn text_outline(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<Option<Vec<crate::doc::vector::Contour>>, StoreError> {
    let (Some(document), Some(comp)) = (view.authored_text_document(layer, t)?, view.composition()?) else { return Ok(None) };
    let canvas = crate::doc::vector::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
    Ok(crate::doc::store::text_frame::shape_document(&document, t, &canvas).ok().flatten().map(|shaped| shaped.contours))
}

pub(super) fn text_box(view: &StoreView<'_>, layer: LayerId, t: RationalTime, wrap: Option<f32>) -> Result<Option<[f32; 4]>, StoreError> {
    let (Some(mut document), Some(comp)) = (view.authored_text_document(layer, t)?, view.composition()?) else { return Ok(None) };
    let canvas = crate::doc::vector::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
    if let Some(width) = wrap {
        document.wrap_size = Some([width.max(1.0), comp.height as f32]);
    }
    let shaped = crate::doc::store::text_frame::shape_document(&document, t, &canvas).ok().flatten();
    Ok(shaped.and_then(|shaped| crate::doc::store::text_frame::line_box(&document, &shaped, &canvas)).map(|b| match wrap {
        Some(width) => [0.0, b[1], width.max(1.0), b[3]],
        None => b,
    }))
}

/// 横が Fill の文字: 決まった幅で折り返した高さ(Scale の zoom 込み)。
/// 幅が決まっているならその幅で折り返した時の大きさ、決まっていないなら折り返さない大きさ。
/// 高さを渡されたらそれをそのまま返す(並べる側が既に決めている)。
pub(super) fn measure_text(view: &StoreView<'_>, measure: Measure, width: Option<f32>, height: Option<f32>, t: RationalTime) -> [f32; 2] {
    let wrap = width.map(|w| w / measure.scale);
    match text_box(view, measure.layer, t, wrap).ok().flatten() {
        Some(b) => [width.unwrap_or((b[2] - b[0]) * measure.scale), height.unwrap_or((b[3] - b[1]) * measure.scale)],
        None => [0.0, 0.0],
    }
}
