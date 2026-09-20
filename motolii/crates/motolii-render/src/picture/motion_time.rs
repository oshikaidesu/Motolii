//! 移り方 — 値が新しい所へ移るまでの、前のコマの姿を覗く窓。
//! 順番の札が距離でずらす規則は**並べた結果**を読むので、時刻の背骨ではなくこちら側。

use std::collections::HashMap;

use crate::doc::core::RationalTime;
use crate::doc::store::layout::*;
use crate::doc::store::{LayerId, StoreError, StoreView};


/// 移り方(Transition)が遡るコマ数の最大。host は解析の入力(Blob の塊)をこのコマ数だけ前まで置く。
/// 移り方は重なる(折り返しの移り方が前の時刻で組み、その時刻の避ける物がさらに前の形を混ぜる)ので、長い順に 2 つの和と遅れ 2 つ分。
pub fn transition_reach(view: &StoreView<'_>, t: RationalTime) -> Result<i64, StoreError> {
    let Some(comp) = view.composition()? else { return Ok(0) };
    let mut longest = [0.0f64; 2];
    let mut waits = 0.0f64;
    for layer in view.layers() {
        waits = waits.max(view.number(layer, TRANSITION_DELAY, 0.0, t)? + view.number(layer, STAGGER, 0.0, t)?);
        let d = view.number(layer, TRANSITION_DURATION, 0.0, t)?;
        if d > longest[0] {
            longest = [d, longest[0]];
        } else if d > longest[1] {
            longest[1] = d;
        }
    }
    Ok(((longest[0] + longest[1] + 2.0 * waits) * comp.fps.as_f64()).round() as i64 + 1)
}

/// Transition の標本: (時刻, 重み)。位置(t) = Σ (E(uₖ₊₁) − E(uₖ)) · 行き先(t − 遅れ − D·uₖ)。時刻はコマに丸める。
/// 遅れがコマの途中なら、前後のコマへ重みを分ける(遅れが時刻で変わっても位置が跳ばない)。
/// Duration も遅れも 0 なら空(今の行き先そのまま)。
pub fn transition_samples(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<Vec<(RationalTime, f32)>, StoreError> {
    let Some(comp) = view.composition()? else { return Ok(Vec::new()) };
    let fps = comp.fps;
    let base = base_samples(view, layer, t)?;
    // 自分の移り方を持たない並ぶ子は、容器の移り方を借りる: 容器の箱が移る途中は、その見えている箱の中で並ぶ
    // (CSS で幅が移る間、中身は毎コマその幅で並び直すのと同じ。揃え・伸びは箱の大きさに線形なので、同じ重みで混ぜれば一致する)。
    if base.is_empty() && view.number(layer, TRANSITION_DELAY, 0.0, t)? <= 0.0 {
        if let Some(parent) = view.attrs(layer)?.unwrap_or_default().parent.filter(|p| view.display(*p, t).is_ok_and(|d| d != 0)) {
            return transition_samples(view, parent, t);
        }
    }
    let delay = transition_delay(view, layer, t, &base)? * fps.as_f64();
    if delay <= 1e-6 {
        return base.into_iter().map(|(back, w)| Ok((frame_time(view, back, t)?, w))).collect();
    }
    let base = if base.is_empty() { vec![(0.0, 1.0)] } else { base };
    let mut out = Vec::with_capacity(base.len() * 2);
    for (back, w) in base {
        let at = back + delay;
        let lo = at.floor();
        let frac = (at - lo) as f32;
        out.push((frame_time(view, lo, t)?, w * (1.0 - frac)));
        if frac > 1e-4 {
            out.push((frame_time(view, lo + 1.0, t)?, w * frac));
        }
    }
    Ok(out)
}

/// 遅れの無い標本: (遡るコマ数, 重み)。
pub fn base_samples(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<Vec<(f64, f32)>, StoreError> {
    let duration = view.number(layer, TRANSITION_DURATION, 0.0, t)?;
    let Some(comp) = view.composition()? else { return Ok(Vec::new()) };
    let frames = (duration * comp.fps.as_f64()).round();
    if frames < 1.0 {
        return Ok(Vec::new());
    }
    let easing = view.choice(layer, TRANSITION_EASING, t)?;
    // コマごとに 1 つ(時刻をずらしても重みの形が変わらない、畳み込みとして滑らか)。長い移り方だけ間引く。
    let n = frames.min(120.0) as usize;
    Ok((0..n).map(|k| {
        let (u0, u1) = (k as f64 / n as f64, (k + 1) as f64 / n as f64);
        ((frames * u0).round(), (ease(easing, u1) - ease(easing, u0)) as f32)
    }).collect())
}

/// 今のコマから `back` コマ前の時刻(0 より前は 0)。
pub fn frame_time(view: &StoreView<'_>, back: f64, t: RationalTime) -> Result<RationalTime, StoreError> {
    let Some(comp) = view.composition()? else { return Ok(t) };
    let now = t.try_to_frame_round(comp.fps).map_err(|e| StoreError::Property(e.to_string()))?;
    RationalTime::try_from_frame((now - back as i64).max(0), comp.fps).map_err(|e| StoreError::Property(e.to_string()))
}

/// 移り方の遅れ(秒): 自分の Transition Delay + 並べる親の Stagger が配る分。
/// 配る分は、遅れと長さの窓より前(t − Stagger − Duration)に並んでいた場所の、起点からの距離で決める
/// (GSAP の stagger が今居る場所で測るのと同じ)。動いている途中の位置で測ると、動くほど遅れが変わって戻る。
pub fn transition_delay(view: &StoreView<'_>, layer: LayerId, t: RationalTime, base: &[(f64, f32)]) -> Result<f64, StoreError> {
    let own = view.number(layer, TRANSITION_DELAY, 0.0, t)?.max(0.0);
    let Some(parent) = view.attrs(layer)?.unwrap_or_default().parent else { return Ok(own) };
    let stagger = view.number(parent, STAGGER, 0.0, t)?.max(0.0);
    if stagger <= 0.0 || view.display(parent, t)? == 0 {
        return Ok(own);
    }
    let Some(comp) = view.composition()? else { return Ok(own) };
    let window = base.iter().map(|(back, _)| *back).fold(0.0, f64::max) + 1.0 + ((stagger + own) * comp.fps.as_f64()).ceil();
    let before = crate::picture::frame::layout_frame(view, frame_time(view, window, t)?)?;
    let (Some(size), Some(slot)) = (before.sizes.get(&parent).copied(), before.slots.get(&layer).copied()) else { return Ok(own) };
    let p = glam::Vec2::from(slot.position);
    let (w, h) = (size[0].max(1e-3), size[1].max(1e-3));
    let corner = glam::vec2(CANVAS_MARGIN, CANVAS_MARGIN);
    let diagonal = glam::vec2(w, h).length();
    let from_centre = (p - corner - glam::vec2(w, h) * 0.5).length() / (diagonal * 0.5);
    let reach = match view.choice(parent, STAGGER_FROM, t)? {
        1 => from_centre,
        2 => (p - corner - glam::vec2(w, h)).length() / diagonal,
        3 => 1.0 - from_centre,
        _ => (p - corner).length() / diagonal,
    };
    Ok(own + stagger * f64::from(reach.clamp(0.0, 1.0)))
}

/// CSS の timing function(区間の形)。Ease / Linear / Ease In / Ease Out / Ease In Out。
pub fn ease(kind: i64, u: f64) -> f64 {
    let (x1, y1, x2, y2) = match kind {
        1 => return u.clamp(0.0, 1.0),
        2 => (0.42, 0.0, 1.0, 1.0),
        3 => (0.0, 0.0, 0.58, 1.0),
        4 => (0.42, 0.0, 0.58, 1.0),
        _ => (0.25, 0.1, 0.25, 1.0),
    };
    let bezier = |a: f64, b: f64, s: f64| 3.0 * a * s * (1.0 - s).powi(2) + 3.0 * b * s * s * (1.0 - s) + s.powi(3);
    let u = u.clamp(0.0, 1.0);
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..40 {
        let mid = (lo + hi) * 0.5;
        if bezier(x1, x2, mid) < u { lo = mid } else { hi = mid }
    }
    bezier(y1, y2, (lo + hi) * 0.5)
}
