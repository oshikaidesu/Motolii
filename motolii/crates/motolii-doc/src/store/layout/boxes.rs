//! 箱と座標 — 層が画面のどこにどれだけの大きさで在るか。
//! 素材座標の箱、親の空間へ移した箱、奥行き、切り、付いて置く物のずれ。
//! 並べる側が枠を決めた後の話で、枠そのものは flow が持つ。

use super::*;

/// 形の層の素材座標の箱(伸ばした後)。
pub(crate) fn stretched_shape_box(shapes: &[crate::doc::vector::ShapeNode], stretch: [f32; 2]) -> Option<[f32; 4]> {
    if stretch == [1.0, 1.0] { shape_box(shapes) } else { shape_box(&crate::doc::vector::stretch_outline(shapes, stretch)) }
}

fn shape_box(shapes: &[crate::doc::vector::ShapeNode]) -> Option<[f32; 4]> {
    let canvas = crate::doc::vector::content_canvas(shapes).ok().flatten()?;
    let b = crate::doc::vector::content_bounds(shapes).ok().flatten()?;
    let (ox, oy) = (canvas.origin_x as f64, canvas.origin_y as f64);
    Some([(b[0] + ox) as f32, (b[1] + oy) as f32, (b[2] + ox) as f32, (b[3] + oy) as f32])
}

/// 箱(素材座標の x, y と奥行き z)を、アンカーのまわりで拡縮・回した時の軸に沿った範囲。回し方は層の変換と同じ
/// (Tilt X・Tilt Y の後に Rotation と Scale)。回転が 0 なら拡縮した箱そのもの。
fn footprint(bounds: [f32; 4], depth: [f32; 2], anchor: [f32; 2], scale: [f32; 3], rotation: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let turn = glam::Quat::from_rotation_x(rotation[0].to_radians()) * glam::Quat::from_rotation_y(rotation[1].to_radians()) * glam::Quat::from_rotation_z(rotation[2].to_radians());
    let (mut lo, mut hi) = (glam::Vec3::splat(f32::INFINITY), glam::Vec3::splat(f32::NEG_INFINITY));
    for x in [bounds[0], bounds[2]] {
        for y in [bounds[1], bounds[3]] {
            for z in depth {
                let p = turn * (glam::vec3(x - anchor[0], y - anchor[1], z) * glam::Vec3::from(scale));
                lo = lo.min(p);
                hi = hi.max(p);
            }
        }
    }
    (lo.to_array(), hi.to_array())
}

/// 並べる側へ返す、層が親の空間で占める場所。枠へ合わせるのに要る物を 1 度に渡す
/// (部品で訊くと、奥行き・回し方・アンカーの読み方が呼ぶ側ごとにずれる)。
pub(super) struct Extent {
    pub lo: [f32; 3],
    pub hi: [f32; 3],
    pub anchor: [f32; 2],
    pub rotation: [f32; 3],
}

/// アンカーを原点に置いた時の範囲。子の素の大きさを測る時に使う(まだ枠が無い)。
pub(super) fn natural_extent(view: &StoreView<'_>, layer: LayerId, t: RationalTime, bounds: [f32; 4], scale: [f32; 2]) -> Result<Extent, StoreError> {
    extent(view, layer, t, bounds, scale, [0.0, 0.0])
}

/// 層が書いたアンカーで置いた時の範囲。決まった枠へ合わせる時に使う。
pub(super) fn placed_extent(view: &StoreView<'_>, layer: LayerId, t: RationalTime, bounds: [f32; 4], scale: [f32; 2]) -> Result<Extent, StoreError> {
    let anchor = item_anchor(view, layer, t, bounds)?;
    extent(view, layer, t, bounds, scale, anchor)
}

pub(crate) fn extent(view: &StoreView<'_>, layer: LayerId, t: RationalTime, bounds: [f32; 4], scale: [f32; 2], anchor: [f32; 2]) -> Result<Extent, StoreError> {
    let rotation = layout_rotation(view, layer, t)?;
    let (lo, hi) = footprint(bounds, raw_depth(view, layer, t)?, anchor, [scale[0], scale[1], 1.0], rotation);
    Ok(Extent { lo, hi, anchor, rotation })
}

/// 押し合いの奥行きのずれ(移り方を混ぜた後)。
pub(crate) fn nudge_z(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<f32, StoreError> {
    let now = view.layout_frame(t)?.nudges_z.get(&layer).copied().unwrap_or(0.0);
    let samples = view.transition_samples(layer, t)?;
    if samples.is_empty() {
        return Ok(now);
    }
    let (mut acc, mut total) = (0.0f32, 0.0f32);
    for (at, weight) in samples {
        acc += view.layout_frame(at)?.nudges_z.get(&layer).copied().unwrap_or(0.0) * weight;
        total += weight;
    }
    Ok(if total > 1e-6 { acc / total } else { now })
}

/// 付いて置く物の、書いた位置からのずれ(親の空間)。Position Area が None か、相手が居なければ None。
pub(crate) fn anchored(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<Option<[f32; 2]>, StoreError> {
    let area = view.choice(layer, POSITION_AREA, t)?;
    if area <= 0 {
        return Ok(None);
    }
    let anchor = match view.value_at(layer, &PropertyId::new(POSITION_ANCHOR)?, t)? {
        Some(Value::LayerId(id)) if id != 0 => LayerId(id),
        Some(Value::F64(v)) if v >= 1.0 => LayerId(v.round() as u64),
        _ => return Ok(None),
    };
    if anchor == layer || !view.here(anchor, t)? {
        return Ok(None);
    }
    // 付き合いが輪になっていれば、2 度目は付かない。
    let key = (layer.0, t.num(), t.den());
    if !view.layout_memo().borrow_mut().anchoring.insert(key) {
        return Ok(None);
    }
    let result = anchored_inner(view, layer, anchor, area, t);
    view.layout_memo().borrow_mut().anchoring.remove(&key);
    result
}

/// `target` の画面の上の箱を、`from` の親の空間で(軸に沿った箱)。Blob Track の層は ID の一番小さい塊。
/// 並べて伸ばした形は伸ばした後の箱。付いて置く札とつなぐ線が、相手の箱を読む口。
pub(crate) fn box_seen_from(view: &StoreView<'_>, target: LayerId, from: LayerId, t: RationalTime) -> Result<Option<(glam::Vec2, glam::Vec2)>, StoreError> {
    let bound = |points: &[glam::Vec2]| points.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    let marks = view.analysis().and_then(|a| a.blobs(target, crate::doc::store::EffectId(0), t)).filter(|m| !m.is_empty());
    let (lo, hi) = if let Some(mark) = marks.and_then(|m| m.iter().min_by_key(|m| m.id)) {
        let (c, h) = (glam::Vec2::from(mark.center), glam::Vec2::from(mark.size) * 0.5);
        (c - h, c + h)
    } else {
        let stretch = view.laid_out(target, t)?.map(|s| s.stretch).filter(|s| *s != [1.0, 1.0]);
        let b = match (stretch, view.meta(target)?.map(|m| m.source)) {
            (Some(stretch), Some(LayerSource::Shape)) => shape_box(&crate::doc::vector::stretch_outline(&view.shapes_at(target, t)?, stretch)),
            _ => layer_box(view, target, t)?,
        };
        let Some(b) = b else { return Ok(None) };
        let world = world_2d(view, target, t)?;
        bound(&[[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| world.transform_point2(glam::Vec2::from(c))))
    };
    // 物理が動かした分を足す(つなぐ線の端が、解き手が動かした箱に付いて行く)。
    Ok(Some(match view.attrs(from)?.unwrap_or_default().parent {
        Some(parent) => {
            let inverse = world_2d(view, parent, t)?.inverse();
            bound(&[lo, glam::vec2(hi.x, lo.y), glam::vec2(lo.x, hi.y), hi].map(|p| inverse.transform_point2(p)))
        }
        None => (lo, hi),
    }))
}

pub(crate) fn anchored_inner(view: &StoreView<'_>, layer: LayerId, anchor: LayerId, area: i64, t: RationalTime) -> Result<Option<[f32; 2]>, StoreError> {
    let bound = |points: &[glam::Vec2]| points.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    let Some((mut a_lo, mut a_hi)) = box_seen_from(view, anchor, layer, t)? else { return Ok(None) };
    let second = match view.value_at(layer, &PropertyId::new(POSITION_ANCHOR_2)?, t)? {
        Some(Value::LayerId(id)) if id != 0 && id != layer.0 => Some(LayerId(id)),
        Some(Value::F64(v)) if v >= 1.0 && v.round() as u64 != layer.0 => Some(LayerId(v.round() as u64)),
        _ => None,
    };
    if let Some(second) = second.filter(|s| view.here(*s, t).unwrap_or(false)) {
        if let Some((b_lo, b_hi)) = box_seen_from(view, second, layer, t)? {
            // 軸ごとに、重なっていれば重なり、離れていれば間。
            let (lo, hi) = (a_lo.max(b_lo), a_hi.min(b_hi));
            let (gap_lo, gap_hi) = (a_hi.min(b_hi), a_lo.max(b_lo));
            a_lo = glam::vec2(if lo.x <= hi.x { lo.x } else { gap_lo.x }, if lo.y <= hi.y { lo.y } else { gap_lo.y });
            a_hi = glam::vec2(if lo.x <= hi.x { hi.x } else { gap_hi.x }, if lo.y <= hi.y { hi.y } else { gap_hi.y });
        }
    }
    // 自分の箱の、位置からの広がり(親の空間)。
    let Some(b) = layer_box(view, layer, t)? else { return Ok(None) };
    let authored = crate::doc::store::view::resolve::transform::resolve_position(view, layer, t)?;
    let local = view.authored_local(layer, t)?;
    let (o_lo, o_hi) = bound(&[[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| local.transform_point2(glam::Vec2::from(c)) - glam::Vec2::from(authored)));
    let margin = view.number(layer, MARGIN, 0.0, t)? as f32;
    let (col, row) = ((area - 1) % 3, (area - 1) / 3);
    let along = |side: i64, a_lo: f32, a_hi: f32, o_lo: f32, o_hi: f32| match side {
        0 => a_lo - margin - o_hi,
        2 => a_hi + margin - o_lo,
        _ => (a_lo + a_hi) * 0.5 - (o_lo + o_hi) * 0.5,
    };
    let target = [along(col, a_lo.x, a_hi.x, o_lo.x, o_hi.x), along(row, a_lo.y, a_hi.y, o_lo.y, o_hi.y)];
    Ok(Some([target[0] - authored[0], target[1] - authored[1]]))
}

/// 並べる Group の箱の大きさ(移り方を混ぜた後)。背景・切り抜き・層の箱が読む。
pub(crate) fn group_size(view: &StoreView<'_>, group: LayerId, t: RationalTime) -> Result<Option<[f32; 2]>, StoreError> {
    let Some(now) = view.layout_frame(t)?.sizes.get(&group).copied() else { return Ok(None) };
    let mut acc = [0.0f32; 2];
    let mut total = 0.0f32;
    for (at, weight) in view.transition_samples(group, t)? {
        if let Some(past) = view.layout_frame(at)?.sizes.get(&group) {
            acc[0] += past[0] * weight;
            acc[1] += past[1] * weight;
            total += weight;
        }
    }
    Ok(Some(if total > 1e-6 { acc.map(|v| v / total) } else { now }))
}

/// 容器の外で押し合ったずれ(移り方を混ぜた後)。
pub(crate) fn nudge(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
    let shift = |at: RationalTime| -> Result<[f32; 2], StoreError> {
        let pushed = view.layout_frame(at)?.nudges.get(&layer).copied().unwrap_or([0.0; 2]);
        let anchored = anchored(view, layer, at)?.unwrap_or([0.0; 2]);
        Ok([pushed[0] + anchored[0], pushed[1] + anchored[1]])
    };
    let now = shift(t)?;
    let samples = view.transition_samples(layer, t)?;
    if samples.is_empty() {
        return Ok(now);
    }
    let mut acc = [0.0f32; 2];
    let mut total = 0.0f32;
    for (at, weight) in samples {
        let past = shift(at)?;
        acc[0] += past[0] * weight;
        acc[1] += past[1] * weight;
        total += weight;
    }
    Ok(if total > 1e-6 { acc.map(|v| v / total) } else { now })
}

/// 層の箱(素材座標)。形は輪郭の canvas、文字は行の箱、並べない Group は子の箱を合わせた物。
/// 効果の広がりは含めない。箱を持たない層(Null・Camera・画・動画はまだ)は `None`。
pub fn layer_box(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<Option<[f32; 4]>, StoreError> {
    let Some(meta) = view.meta(layer)? else { return Ok(None) };
    Ok(match meta.source {
        LayerSource::Shape => shape_box(&view.shapes_at(layer, t)?),
        LayerSource::Text => super::text::text_box(view, layer, t, None)?,
        LayerSource::File { path, .. } => view.analysis().and_then(|a| a.extent(&path)).filter(|e| e[0] > 0.0 && e[1] > 0.0).map(|e| [0.0, 0.0, e[0], e[1]]),
        LayerSource::Group => {
            if view.display(layer, t)? != 0 {
                return Ok(group_size(view, layer, t)?.map(|s| [CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + s[0], CANVAS_MARGIN + s[1]]));
            }
            let mut acc: Option<[f32; 4]> = None;
            for child in view.layers() {
                // 中の Display の Group は数えない(並べる途中でここへ来るので、解き直すと巡る)。
                if view.attrs(child)?.unwrap_or_default().parent != Some(layer) || !view.here(child, t)? || view.display(child, t)? != 0 {
                    continue;
                }
                let Some(b) = layer_box(view, child, t)? else { continue };
                let local = crate::doc::store::view::resolve::transform::local_transform(view, child, t)?;
                for corner in [[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]] {
                    let p = local.transform_point2(glam::Vec2::from(corner));
                    acc = Some(match acc {
                        None => [p.x, p.y, p.x, p.y],
                        Some(a) => [a[0].min(p.x), a[1].min(p.y), a[2].max(p.x), a[3].max(p.y)],
                    });
                }
            }
            acc
        }
        _ => None,
    })
}

/// Shape Outside が宣言する形(comp の多角形)。Margin Box = 箱 + Margin、Content = 輪郭か Blob の塊の箱。
pub(super) fn declared_shape(view: &StoreView<'_>, layer: LayerId, mode: i64, t: RationalTime) -> Result<Vec<Vec<glam::Vec2>>, StoreError> {
    let rect = |b: [f32; 4]| vec![glam::vec2(b[0], b[1]), glam::vec2(b[2], b[1]), glam::vec2(b[2], b[3]), glam::vec2(b[0], b[3])];
    if mode == 2 {
        if let Some(marks) = view.analysis().and_then(|a| a.blobs(layer, crate::doc::store::EffectId(0), t)) {
            return Ok(marks.iter().map(|mark| {
                let (c, h) = (glam::Vec2::from(mark.center), glam::Vec2::from(mark.size) * 0.5);
                rect([c.x - h.x, c.y - h.y, c.x + h.x, c.y + h.y])
            }).collect());
        }
    }
    let world = world_2d(view, layer, t)?;
    let local = outline(view, layer, t)?;
    if mode == 1 {
        let grow = view.number(layer, MARGIN, 0.0, t)? as f32;
        let (lo, hi) = local.iter().flatten().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
        if lo.x > hi.x {
            return Ok(Vec::new());
        }
        return Ok(vec![rect([lo.x - grow, lo.y - grow, hi.x + grow, hi.y + grow]).into_iter().map(|p| world.transform_point2(p)).collect()]);
    }
    Ok(local.into_iter().map(|poly| poly.into_iter().map(|p| world.transform_point2(p)).collect()).collect())
}

/// 層の輪郭(素材座標の多角形)。形は曲線を刻んだ輪郭(並べた伸びを込み)、他は層の箱。
pub(crate) fn outline(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<Vec<Vec<glam::Vec2>>, StoreError> {
    let Some(meta) = view.meta(layer)? else { return Ok(Vec::new()) };
    if meta.source == LayerSource::Shape {
        let stretch = view.laid_out(layer, t)?.map_or([1.0, 1.0], |slot| slot.stretch);
        let shapes = crate::doc::vector::stretch_outline(&view.shapes_at(layer, t)?, stretch);
        let Ok(Some(canvas)) = crate::doc::vector::content_canvas(&shapes) else { return Ok(Vec::new()) };
        let origin = glam::vec2(canvas.origin_x as f32, canvas.origin_y as f32);
        let mut out = Vec::new();
        for shape in crate::doc::vector::flatten(&shapes).unwrap_or_default() {
            for instance in crate::doc::vector::resolve(&shape).unwrap_or_default() {
                for contour in &instance.path {
                    let n = contour.vertices.len();
                    let mut poly = Vec::new();
                    let p = |v: crate::doc::vector::Point| glam::vec2(v.x as f32, v.y as f32);
                    for i in 0..if contour.closed { n } else { n.saturating_sub(1) } {
                        let (a, b) = (&contour.vertices[i], &contour.vertices[(i + 1) % n]);
                        let (p0, p3) = (p(a.point), p(b.point));
                        let (p1, p2) = (p0 + p(a.out_tangent), p3 + p(b.in_tangent));
                        for k in 0..8 {
                            let u = k as f32 / 8.0;
                            let w = 1.0 - u;
                            poly.push(p0 * w * w * w + p1 * 3.0 * w * w * u + p2 * 3.0 * w * u * u + p3 * u * u * u + origin);
                        }
                    }
                    out.push(poly);
                }
            }
        }
        return Ok(out);
    }
    Ok(layer_box(view, layer, t)?.map(|b| vec![vec![glam::vec2(b[0], b[1]), glam::vec2(b[2], b[1]), glam::vec2(b[2], b[3]), glam::vec2(b[0], b[3])]]).unwrap_or_default())
}

/// Display の Group の背景(Background の色が透明でなければ)。角は Border Radius。描くのは形の層と同じ道。
pub fn background_shapes(view: &StoreView<'_>, group: LayerId, t: RationalTime) -> Result<Option<Vec<crate::doc::vector::ShapeNode>>, StoreError> {
    use crate::doc::vector::{Brush, Fill, FillRule, OpKind, PathSource, Point, RepeaterTransform, Rgb, Shape, ShapeGroup, ShapeNode, ShapeOp};
    if view.display(group, t)? == 0 {
        return Ok(None);
    }
    let color = match view.value_at(group, &PropertyId::new(BACKGROUND)?, t)? {
        Some(Value::Color(c)) => c,
        _ => [0.0; 4],
    };
    let shadow = match view.value_at(group, &PropertyId::new(SHADOW_COLOR)?, t)? {
        Some(Value::Color(c)) => c,
        _ => [0.0; 4],
    };
    let Some(size) = group_size(view, group, t)? else { return Ok(None) };
    if (color[3] <= 0.0 && shadow[3] <= 0.0) || size[0] <= 0.0 || size[1] <= 0.0 {
        return Ok(None);
    }
    let radius = view.number(group, BORDER_RADIUS, 0.0, t)?.max(0.0);
    let (w, h) = (f64::from(size[0]), f64::from(size[1]));
    // 箱を grow だけ広げた角丸の矩形(中心は箱の中心 + ずれ)。
    let rounded = |grow: f64, offset: [f64; 2], rgba: [f64; 4]| -> Option<ShapeNode> {
        let (sw, sh) = (w + 2.0 * grow, h + 2.0 * grow);
        if sw <= 0.0 || sh <= 0.0 || rgba[3] <= 0.0 {
            return None;
        }
        let r = (radius + grow).max(0.0).min(sw.min(sh) * 0.5);
        Some(ShapeNode::Group(ShapeGroup {
            transform: RepeaterTransform { position: Point { x: w * 0.5 + offset[0], y: h * 0.5 + offset[1] }, ..RepeaterTransform::IDENTITY },
            children: vec![ShapeNode::Leaf(Shape {
                source: PathSource::Rectangle { size: Point { x: sw, y: sh } },
                ops: if r > 0.0 { vec![ShapeOp::new(OpKind::RoundedCorners { radius: r })] } else { Vec::new() },
                fill: Some(Fill { brush: Brush::Solid(Rgb { r: rgba[0], g: rgba[1], b: rgba[2] }), rule: FillRule::NonZero, opacity: rgba[3], hidden: false }),
                stroke: None,
            })],
        }))
    };
    let mut out = Vec::new();
    if shadow[3] > 0.0 {
        let offset = view.pair(group, SHADOW_OFFSET, [0.0, 12.0], t)?;
        let offset = [f64::from(offset[0]), f64::from(offset[1])];
        let blur = view.number(group, SHADOW_BLUR, 24.0, t)?.max(0.0);
        let spread = view.number(group, SHADOW_SPREAD, 0.0, t)?;
        if blur <= 0.5 {
            out.extend(rounded(spread, offset, shadow));
        } else {
            // CSS のぼかしは標準偏差 blur / 2 のガウス。影の縁の前後 ±blur を外から内へ N 枚の輪で覆い、
            // 重なった後の濃さが外の 0 から内の Shadow Color の α まで滑らかな段(smoothstep)で上がるよう、輪ごとの α を解く。
            const RINGS: usize = 32;
            let ramp = |n: usize| { let u = n as f64 / RINGS as f64; shadow[3] * u * u * (3.0 - 2.0 * u) };
            for k in 0..RINGS {
                let grow = spread + blur * (1.0 - 2.0 * (k as f64 + 0.5) / RINGS as f64);
                let (before, after) = (ramp(k), ramp(k + 1));
                let each = if before >= 1.0 { 0.0 } else { 1.0 - (1.0 - after) / (1.0 - before) };
                out.extend(rounded(grow, offset, [shadow[0], shadow[1], shadow[2], each.clamp(0.0, 1.0)]));
            }
        }
    }
    out.extend(rounded(0.0, [0.0, 0.0], color));
    Ok(Some(out))
}

/// 層の 2D の world(親を辿る)。塊を Group の素材座標へ戻す時。
pub(crate) fn world_2d(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<glam::Affine2, StoreError> {
    let mut world = crate::doc::store::view::resolve::transform::local_transform(view, layer, t)?;
    let mut seen = std::collections::HashSet::from([layer]);
    let mut next = view.attrs(layer)?.unwrap_or_default().parent;
    while let Some(parent) = next.filter(|p| seen.insert(*p)) {
        world = crate::doc::store::view::resolve::transform::local_transform(view, parent, t)? * world;
        next = view.attrs(parent)?.unwrap_or_default().parent;
    }
    Ok(world)
}

/// Overflow が Clip の並べる Group なら、その箱(素材座標)と角の丸み。
pub(crate) fn clip_box(view: &StoreView<'_>, group: LayerId, t: RationalTime) -> Result<Option<([f32; 4], f32)>, StoreError> {
    if view.display(group, t)? == 0 || view.choice(group, OVERFLOW, t)? != 1 {
        return Ok(None);
    }
    let Some(size) = group_size(view, group, t)? else { return Ok(None) };
    let b = [CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + size[0], CANVAS_MARGIN + size[1]];
    Ok(Some((b, view.number(group, BORDER_RADIUS, 0.0, t)?.max(0.0) as f32)))
}

/// `clip-path: inset()`: 4 辺のどれかが 0 でない並べる Group なら、削った箱(素材座標)と角の丸み(Clip Radius)。
/// 辺が向かい合う辺を越えたら空の箱(CSS と同じ、何も見えない)。
pub(crate) fn clip_inset(view: &StoreView<'_>, group: LayerId, t: RationalTime) -> Result<Option<([f32; 4], f32)>, StoreError> {
    if view.display(group, t)? == 0 {
        return Ok(None);
    }
    let mut edges = [0.0f32; 4];
    for (edge, name) in edges.iter_mut().zip([CLIP_TOP, CLIP_RIGHT, CLIP_BOTTOM, CLIP_LEFT]) {
        *edge = view.number(group, name, 0.0, t)? as f32;
    }
    let [top, right, bottom, left] = edges;
    if edges.iter().all(|e| e.abs() <= 1e-6) {
        return Ok(None);
    }
    let Some(size) = group_size(view, group, t)? else { return Ok(None) };
    let (x0, y0) = (CANVAS_MARGIN + left, CANVAS_MARGIN + top);
    let (x1, y1) = ((CANVAS_MARGIN + size[0] - right).max(x0), (CANVAS_MARGIN + size[1] - bottom).max(y0));
    Ok(Some(([x0, y0, x1, y1], view.number(group, CLIP_RADIUS, 0.0, t)?.max(0.0) as f32)))
}

/// 層の奥行きの範囲(素材座標の z、[手前, 奥])。平らな物は [0, 0]、押し出しは [0, Depth]、網・点群は bounds の
/// 奥行きを中心に、並べる Group は [-奥行き, 0]。Scale Z(と 3D の Object Fit)を掛ける。
pub(super) fn depth_range(view: &StoreView<'_>, layer: LayerId, t: RationalTime, frame: &Frame) -> Result<[f32; 2], StoreError> {
    let Some(meta) = view.meta(layer)? else { return Ok([0.0; 2]) };
    let scale_z = view.number(layer, property::SCALE_Z, 1.0, t)? as f32 * frame.slots.get(&layer).map_or(1.0, |s| s.scale_z);
    let range = match &meta.source {
        LayerSource::Shape | LayerSource::Text => [0.0, view.number(layer, property::DEPTH, 0.0, t)?.max(0.0) as f32],
        LayerSource::File { path, .. } => {
            // 描く側と同じ: 奥行きは xy の拡縮の平均で伸びる(depth_scaled)。
            let scale = frame.slots.get(&layer).map_or(view.pair(layer, property::SCALE, [1.0, 1.0], t)?, |s| s.scale);
            let d = view.analysis().and_then(|a| a.extent(path)).map_or(0.0, |e| e[2]) * (scale[0].abs() + scale[1].abs()) * 0.5;
            [-d * 0.5, d * 0.5]
        }
        LayerSource::Group => frame.depths.get(&layer).copied().unwrap_or([0.0, 0.0]),
        _ => [0.0, 0.0],
    };
    let range = [range[0] * scale_z, range[1] * scale_z];
    match frame.slots.get(&layer).map(|s| s.rotation).filter(|r| *r != [0.0; 3]) {
        Some(rotation) => {
            let bounds = match meta.source {
                LayerSource::Group => frame.sizes.get(&layer).map_or([0.0; 4], |s| [0.0, 0.0, s[0], s[1]]),
                _ => layer_box(view, layer, t)?.unwrap_or([0.0; 4]),
            };
            let scale = view.pair(layer, property::SCALE, [1.0, 1.0], t)?;
            let anchor = item_anchor(view, layer, t, bounds)?;
            let (lo, hi) = footprint(bounds, [range[0], range[1]], anchor, [scale[0], scale[1], 1.0], rotation);
            Ok([lo[2], hi[2]])
        }
        None => Ok(range),
    }
}

/// Transform Origin が Anchor 以外なら、箱の中のその点(素材座標)。
pub(crate) fn origin_in(view: &StoreView<'_>, layer: LayerId, t: RationalTime, bounds: [f32; 4]) -> Result<Option<[f32; 2]>, StoreError> {
    let origin = view.choice(layer, TRANSFORM_ORIGIN, t)?;
    if origin <= 0 {
        return Ok(None);
    }
    let (col, row) = ((origin - 1) % 3, (origin - 1) / 3);
    let at = |k: i64, lo: f32, hi: f32| lo + (hi - lo) * k as f32 * 0.5;
    Ok(Some([at(col, bounds[0], bounds[2]), at(row, bounds[1], bounds[3])]))
}

/// 並ばない層の中心: Transform Origin があれば箱から、無ければ Anchor の値。
pub(crate) fn free_anchor(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
    if view.choice(layer, TRANSFORM_ORIGIN, t)? > 0 {
        if let Some(b) = layer_box(view, layer, t)? {
            if let Some(origin) = origin_in(view, layer, t, b)? {
                return Ok(origin);
            }
        }
    }
    view.pair(layer, property::ANCHOR, [0.0, 0.0], t)
}

/// 並ぶ子の拡縮・回転の中心。Transform Origin があれば箱の中のその点、Anchor を書いた層はその値、書いていなければ箱の中心(CSS の transform-origin)。
pub(crate) fn item_anchor(view: &StoreView<'_>, layer: LayerId, t: RationalTime, bounds: [f32; 4]) -> Result<[f32; 2], StoreError> {
    if let Some(origin) = origin_in(view, layer, t, bounds)? {
        return Ok(origin);
    }
    Ok(match view.value_at(layer, &PropertyId::new(property::ANCHOR)?, t)? {
        Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32],
        _ => [(bounds[0] + bounds[2]) * 0.5, (bounds[1] + bounds[3]) * 0.5],
    })
}

/// 並べる前の奥行きの範囲(Scale Z 込み、Object Fit と回転は無し)。葉の箱を測る時。
pub(crate) fn raw_depth(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
    depth_range(view, layer, t, &Frame::default())
}

/// 回転の欄(Tilt X・Tilt Y・Rotation)。
pub(crate) fn layout_rotation(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<[f32; 3], StoreError> {
    Ok([view.number(layer, LAYOUT_TILT_X, 0.0, t)? as f32, view.number(layer, LAYOUT_TILT_Y, 0.0, t)? as f32, view.number(layer, LAYOUT_ROTATION, 0.0, t)? as f32])
}

/// Depth Alignment: 子の奥行きの最大が Group の奥行き。Back = 子の奥を面(z = 0)に、Front = 子の手前を
/// Group の手前(-奥行き)に、Center = 中心を揃える(visionOS の depthAlignment)。
pub(super) fn align_depth(view: &StoreView<'_>, group: LayerId, t: RationalTime, children: &HashMap<LayerId, Vec<(i16, LayerId)>>, frame: &mut Frame) -> Result<(), StoreError> {
    let alignment = view.choice(group, DEPTH_ALIGNMENT, t)?;
    let mut ranges = Vec::new();
    for &(_, child) in children.get(&group).map(Vec::as_slice).unwrap_or(&[]) {
        if frame.slots.contains_key(&child) {
            ranges.push((child, depth_range(view, child, t, frame)?));
        }
    }
    if view.display(group, t)? == 1 && view.choice(group, FLEX_DIRECTION, t)? == DIRECTION_DEPTH {
        // 奥へ積む: 重ね順の上(番号の大きい物)が一番手前、面(z = 0)に手前を合わせ、奥行き + Gap ずつ奥へ。
        let gap = view.number(group, GAP, 0.0, t)? as f32;
        let mut cursor = 0.0f32;
        for (child, [front, back]) in ranges.into_iter().rev() {
            if let Some(slot) = frame.slots.get_mut(&child) {
                slot.z = cursor - front;
            }
            cursor += back - front + gap;
        }
        frame.depths.insert(group, [0.0, (cursor - gap).max(0.0)]);
        return Ok(());
    }
    let depth = ranges.iter().map(|(_, r)| r[1] - r[0]).fold(0.0f32, f32::max);
    for (child, [front, back]) in ranges {
        let z = match alignment {
            1 => -depth * 0.5 - (front + back) * 0.5,
            2 => -depth - front,
            _ => -back,
        };
        if let Some(slot) = frame.slots.get_mut(&child) {
            slot.z = z;
        }
    }
    frame.depths.insert(group, [-depth, 0.0]);
    Ok(())
}
