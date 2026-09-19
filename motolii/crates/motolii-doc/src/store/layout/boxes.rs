//! 箱と座標 — 層が画面のどこにどれだけの大きさで在るか。
//! 素材座標の箱、親の空間へ移した箱、奥行き、切り、付いて置く物のずれ。
//! 並べる側が枠を決めた後の話で、枠そのものは flow が持つ。

use super::*;

/// 形の層の素材座標の箱(伸ばした後)。
pub(crate) fn stretched_shape_box(shapes: &[crate::doc::vector::ShapeNode], stretch: [f32; 2]) -> Option<[f32; 4]> {
    if stretch == [1.0, 1.0] { shape_box(shapes) } else { shape_box(&crate::doc::vector::stretch_outline(shapes, stretch)) }
}

pub(super) fn shape_box(shapes: &[crate::doc::vector::ShapeNode]) -> Option<[f32; 4]> {
    let canvas = crate::doc::vector::content_canvas(shapes).ok().flatten()?;
    let b = crate::doc::vector::content_bounds(shapes).ok().flatten()?;
    let (ox, oy) = (canvas.origin_x as f64, canvas.origin_y as f64);
    Some([(b[0] + ox) as f32, (b[1] + oy) as f32, (b[2] + ox) as f32, (b[3] + oy) as f32])
}

/// 箱(素材座標の x, y と奥行き z)を、アンカーのまわりで拡縮・回した時の軸に沿った範囲。回し方は層の変換と同じ
/// (Tilt X・Tilt Y の後に Rotation と Scale)。回転が 0 なら拡縮した箱そのもの。
pub(super) fn footprint(bounds: [f32; 4], depth: [f32; 2], anchor: [f32; 2], scale: [f32; 3], rotation: [f32; 3]) -> ([f32; 3], [f32; 3]) {
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

impl StoreView<'_> {
    /// 押し合いの奥行きのずれ(移り方を混ぜた後)。
    pub(crate) fn nudge_z(&self, layer: LayerId, t: RationalTime) -> Result<f32, StoreError> {
        let now = self.layout_frame(t)?.nudges_z.get(&layer).copied().unwrap_or(0.0);
        let samples = self.transition_samples(layer, t)?;
        if samples.is_empty() {
            return Ok(now);
        }
        let (mut acc, mut total) = (0.0f32, 0.0f32);
        for (at, weight) in samples {
            acc += self.layout_frame(at)?.nudges_z.get(&layer).copied().unwrap_or(0.0) * weight;
            total += weight;
        }
        Ok(if total > 1e-6 { acc / total } else { now })
    }

    /// 付いて置く物の、書いた位置からのずれ(親の空間)。Position Area が None か、相手が居なければ None。
    fn anchored(&self, layer: LayerId, t: RationalTime) -> Result<Option<[f32; 2]>, StoreError> {
        let area = self.choice(layer, POSITION_AREA, t)?;
        if area <= 0 {
            return Ok(None);
        }
        let anchor = match self.value_at(layer, &PropertyId::new(POSITION_ANCHOR)?, t)? {
            Some(Value::LayerId(id)) if id != 0 => LayerId(id),
            Some(Value::F64(v)) if v >= 1.0 => LayerId(v.round() as u64),
            _ => return Ok(None),
        };
        if anchor == layer || !self.here(anchor, t)? {
            return Ok(None);
        }
        // 付き合いが輪になっていれば、2 度目は付かない。
        let key = (layer.0, t.num(), t.den());
        if !self.layout_memo().borrow_mut().anchoring.insert(key) {
            return Ok(None);
        }
        let result = self.anchored_inner(layer, anchor, area, t);
        self.layout_memo().borrow_mut().anchoring.remove(&key);
        result
    }

    /// `target` の画面の上の箱を、`from` の親の空間で(軸に沿った箱)。Blob Track の層は ID の一番小さい塊。
    /// 並べて伸ばした形は伸ばした後の箱。付いて置く札とつなぐ線が、相手の箱を読む口。
    pub(crate) fn box_seen_from(&self, target: LayerId, from: LayerId, t: RationalTime) -> Result<Option<(glam::Vec2, glam::Vec2)>, StoreError> {
        let bound = |points: &[glam::Vec2]| points.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
        let marks = self.analysis().and_then(|a| a.blobs(target, crate::doc::store::EffectId(0), t)).filter(|m| !m.is_empty());
        let (lo, hi) = if let Some(mark) = marks.and_then(|m| m.iter().min_by_key(|m| m.id)) {
            let (c, h) = (glam::Vec2::from(mark.center), glam::Vec2::from(mark.size) * 0.5);
            (c - h, c + h)
        } else {
            let stretch = self.laid_out(target, t)?.map(|s| s.stretch).filter(|s| *s != [1.0, 1.0]);
            let b = match (stretch, self.meta(target)?.map(|m| m.source)) {
                (Some(stretch), Some(LayerSource::Shape)) => shape_box(&crate::doc::vector::stretch_outline(&self.shapes_at(target, t)?, stretch)),
                _ => self.layer_box(target, t)?,
            };
            let Some(b) = b else { return Ok(None) };
            let world = self.world_2d(target, t)?;
            bound(&[[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| world.transform_point2(glam::Vec2::from(c))))
        };
        // 物理が動かした分を足す(つなぐ線の端が、解き手が動かした箱に付いて行く)。
        Ok(Some(match self.attrs(from)?.unwrap_or_default().parent {
            Some(parent) => {
                let inverse = self.world_2d(parent, t)?.inverse();
                bound(&[lo, glam::vec2(hi.x, lo.y), glam::vec2(lo.x, hi.y), hi].map(|p| inverse.transform_point2(p)))
            }
            None => (lo, hi),
        }))
    }

    fn anchored_inner(&self, layer: LayerId, anchor: LayerId, area: i64, t: RationalTime) -> Result<Option<[f32; 2]>, StoreError> {
        let bound = |points: &[glam::Vec2]| points.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
        let Some((mut a_lo, mut a_hi)) = self.box_seen_from(anchor, layer, t)? else { return Ok(None) };
        let second = match self.value_at(layer, &PropertyId::new(POSITION_ANCHOR_2)?, t)? {
            Some(Value::LayerId(id)) if id != 0 && id != layer.0 => Some(LayerId(id)),
            Some(Value::F64(v)) if v >= 1.0 && v.round() as u64 != layer.0 => Some(LayerId(v.round() as u64)),
            _ => None,
        };
        if let Some(second) = second.filter(|s| self.here(*s, t).unwrap_or(false)) {
            if let Some((b_lo, b_hi)) = self.box_seen_from(second, layer, t)? {
                // 軸ごとに、重なっていれば重なり、離れていれば間。
                let (lo, hi) = (a_lo.max(b_lo), a_hi.min(b_hi));
                let (gap_lo, gap_hi) = (a_hi.min(b_hi), a_lo.max(b_lo));
                a_lo = glam::vec2(if lo.x <= hi.x { lo.x } else { gap_lo.x }, if lo.y <= hi.y { lo.y } else { gap_lo.y });
                a_hi = glam::vec2(if lo.x <= hi.x { hi.x } else { gap_hi.x }, if lo.y <= hi.y { hi.y } else { gap_hi.y });
            }
        }
        // 自分の箱の、位置からの広がり(親の空間)。
        let Some(b) = self.layer_box(layer, t)? else { return Ok(None) };
        let authored = self.resolve_position(layer, t)?;
        let local = self.authored_local(layer, t)?;
        let (o_lo, o_hi) = bound(&[[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| local.transform_point2(glam::Vec2::from(c)) - glam::Vec2::from(authored)));
        let margin = self.number(layer, MARGIN, 0.0, t)? as f32;
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
    pub(crate) fn group_size(&self, group: LayerId, t: RationalTime) -> Result<Option<[f32; 2]>, StoreError> {
        let Some(now) = self.layout_frame(t)?.sizes.get(&group).copied() else { return Ok(None) };
        let mut acc = [0.0f32; 2];
        let mut total = 0.0f32;
        for (at, weight) in self.transition_samples(group, t)? {
            if let Some(past) = self.layout_frame(at)?.sizes.get(&group) {
                acc[0] += past[0] * weight;
                acc[1] += past[1] * weight;
                total += weight;
            }
        }
        Ok(Some(if total > 1e-6 { acc.map(|v| v / total) } else { now }))
    }

    /// 容器の外で押し合ったずれ(移り方を混ぜた後)。
    pub(crate) fn nudge(&self, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
        let shift = |at: RationalTime| -> Result<[f32; 2], StoreError> {
            let pushed = self.layout_frame(at)?.nudges.get(&layer).copied().unwrap_or([0.0; 2]);
            let anchored = self.anchored(layer, at)?.unwrap_or([0.0; 2]);
            Ok([pushed[0] + anchored[0], pushed[1] + anchored[1]])
        };
        let now = shift(t)?;
        let samples = self.transition_samples(layer, t)?;
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
    pub fn layer_box(&self, layer: LayerId, t: RationalTime) -> Result<Option<[f32; 4]>, StoreError> {
        let Some(meta) = self.meta(layer)? else { return Ok(None) };
        Ok(match meta.source {
            LayerSource::Shape => shape_box(&self.shapes_at(layer, t)?),
            LayerSource::Text => self.text_box(layer, t, None)?,
            LayerSource::File { path, .. } => self.analysis().and_then(|a| a.extent(&path)).filter(|e| e[0] > 0.0 && e[1] > 0.0).map(|e| [0.0, 0.0, e[0], e[1]]),
            LayerSource::Group => {
                if self.display(layer, t)? != 0 {
                    return Ok(self.group_size(layer, t)?.map(|s| [CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + s[0], CANVAS_MARGIN + s[1]]));
                }
                let mut acc: Option<[f32; 4]> = None;
                for child in self.layers() {
                    // 中の Display の Group は数えない(並べる途中でここへ来るので、解き直すと巡る)。
                    if self.attrs(child)?.unwrap_or_default().parent != Some(layer) || !self.here(child, t)? || self.display(child, t)? != 0 {
                        continue;
                    }
                    let Some(b) = self.layer_box(child, t)? else { continue };
                    let local = self.local_transform(child, t)?;
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
    pub(super) fn declared_shape(&self, layer: LayerId, mode: i64, t: RationalTime) -> Result<Vec<Vec<glam::Vec2>>, StoreError> {
        let rect = |b: [f32; 4]| vec![glam::vec2(b[0], b[1]), glam::vec2(b[2], b[1]), glam::vec2(b[2], b[3]), glam::vec2(b[0], b[3])];
        if mode == 2 {
            if let Some(marks) = self.analysis().and_then(|a| a.blobs(layer, crate::doc::store::EffectId(0), t)) {
                return Ok(marks.iter().map(|mark| {
                    let (c, h) = (glam::Vec2::from(mark.center), glam::Vec2::from(mark.size) * 0.5);
                    rect([c.x - h.x, c.y - h.y, c.x + h.x, c.y + h.y])
                }).collect());
            }
        }
        let world = self.world_2d(layer, t)?;
        let local = self.outline(layer, t)?;
        if mode == 1 {
            let grow = self.number(layer, MARGIN, 0.0, t)? as f32;
            let (lo, hi) = local.iter().flatten().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
            if lo.x > hi.x {
                return Ok(Vec::new());
            }
            return Ok(vec![rect([lo.x - grow, lo.y - grow, hi.x + grow, hi.y + grow]).into_iter().map(|p| world.transform_point2(p)).collect()]);
        }
        Ok(local.into_iter().map(|poly| poly.into_iter().map(|p| world.transform_point2(p)).collect()).collect())
    }

    /// 層の輪郭(素材座標の多角形)。形は曲線を刻んだ輪郭(並べた伸びを込み)、他は層の箱。
    fn outline(&self, layer: LayerId, t: RationalTime) -> Result<Vec<Vec<glam::Vec2>>, StoreError> {
        let Some(meta) = self.meta(layer)? else { return Ok(Vec::new()) };
        if meta.source == LayerSource::Shape {
            let stretch = self.laid_out(layer, t)?.map_or([1.0, 1.0], |slot| slot.stretch);
            let shapes = crate::doc::vector::stretch_outline(&self.shapes_at(layer, t)?, stretch);
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
        Ok(self.layer_box(layer, t)?.map(|b| vec![vec![glam::vec2(b[0], b[1]), glam::vec2(b[2], b[1]), glam::vec2(b[2], b[3]), glam::vec2(b[0], b[3])]]).unwrap_or_default())
    }

    /// Display の Group の背景(Background の色が透明でなければ)。角は Border Radius。描くのは形の層と同じ道。
    pub fn background_shapes(&self, group: LayerId, t: RationalTime) -> Result<Option<Vec<crate::doc::vector::ShapeNode>>, StoreError> {
        use crate::doc::vector::{Brush, Fill, FillRule, OpKind, PathSource, Point, RepeaterTransform, Rgb, Shape, ShapeGroup, ShapeNode, ShapeOp};
        if self.display(group, t)? == 0 {
            return Ok(None);
        }
        let color = match self.value_at(group, &PropertyId::new(BACKGROUND)?, t)? {
            Some(Value::Color(c)) => c,
            _ => [0.0; 4],
        };
        let shadow = match self.value_at(group, &PropertyId::new(SHADOW_COLOR)?, t)? {
            Some(Value::Color(c)) => c,
            _ => [0.0; 4],
        };
        let Some(size) = self.group_size(group, t)? else { return Ok(None) };
        if (color[3] <= 0.0 && shadow[3] <= 0.0) || size[0] <= 0.0 || size[1] <= 0.0 {
            return Ok(None);
        }
        let radius = self.number(group, BORDER_RADIUS, 0.0, t)?.max(0.0);
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
            let offset = self.pair(group, SHADOW_OFFSET, [0.0, 12.0], t)?;
            let offset = [f64::from(offset[0]), f64::from(offset[1])];
            let blur = self.number(group, SHADOW_BLUR, 24.0, t)?.max(0.0);
            let spread = self.number(group, SHADOW_SPREAD, 0.0, t)?;
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
    pub(crate) fn world_2d(&self, layer: LayerId, t: RationalTime) -> Result<glam::Affine2, StoreError> {
        let mut world = self.local_transform(layer, t)?;
        let mut seen = std::collections::HashSet::from([layer]);
        let mut next = self.attrs(layer)?.unwrap_or_default().parent;
        while let Some(parent) = next.filter(|p| seen.insert(*p)) {
            world = self.local_transform(parent, t)? * world;
            next = self.attrs(parent)?.unwrap_or_default().parent;
        }
        Ok(world)
    }

    /// Overflow が Clip の並べる Group なら、その箱(素材座標)と角の丸み。
    pub(crate) fn clip_box(&self, group: LayerId, t: RationalTime) -> Result<Option<([f32; 4], f32)>, StoreError> {
        if self.display(group, t)? == 0 || self.choice(group, OVERFLOW, t)? != 1 {
            return Ok(None);
        }
        let Some(size) = self.group_size(group, t)? else { return Ok(None) };
        let b = [CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + size[0], CANVAS_MARGIN + size[1]];
        Ok(Some((b, self.number(group, BORDER_RADIUS, 0.0, t)?.max(0.0) as f32)))
    }

    /// `clip-path: inset()`: 4 辺のどれかが 0 でない並べる Group なら、削った箱(素材座標)と角の丸み(Clip Radius)。
    /// 辺が向かい合う辺を越えたら空の箱(CSS と同じ、何も見えない)。
    pub(crate) fn clip_inset(&self, group: LayerId, t: RationalTime) -> Result<Option<([f32; 4], f32)>, StoreError> {
        if self.display(group, t)? == 0 {
            return Ok(None);
        }
        let mut edges = [0.0f32; 4];
        for (edge, name) in edges.iter_mut().zip([CLIP_TOP, CLIP_RIGHT, CLIP_BOTTOM, CLIP_LEFT]) {
            *edge = self.number(group, name, 0.0, t)? as f32;
        }
        let [top, right, bottom, left] = edges;
        if edges.iter().all(|e| e.abs() <= 1e-6) {
            return Ok(None);
        }
        let Some(size) = self.group_size(group, t)? else { return Ok(None) };
        let (x0, y0) = (CANVAS_MARGIN + left, CANVAS_MARGIN + top);
        let (x1, y1) = ((CANVAS_MARGIN + size[0] - right).max(x0), (CANVAS_MARGIN + size[1] - bottom).max(y0));
        Ok(Some(([x0, y0, x1, y1], self.number(group, CLIP_RADIUS, 0.0, t)?.max(0.0) as f32)))
    }

    /// 層の奥行きの範囲(素材座標の z、[手前, 奥])。平らな物は [0, 0]、押し出しは [0, Depth]、網・点群は bounds の
    /// 奥行きを中心に、並べる Group は [-奥行き, 0]。Scale Z(と 3D の Object Fit)を掛ける。
    pub(super) fn depth_range(&self, layer: LayerId, t: RationalTime, frame: &Frame) -> Result<[f32; 2], StoreError> {
        let Some(meta) = self.meta(layer)? else { return Ok([0.0; 2]) };
        let scale_z = self.number(layer, property::SCALE_Z, 1.0, t)? as f32 * frame.slots.get(&layer).map_or(1.0, |s| s.scale_z);
        let range = match &meta.source {
            LayerSource::Shape | LayerSource::Text => [0.0, self.number(layer, property::DEPTH, 0.0, t)?.max(0.0) as f32],
            LayerSource::File { path, .. } => {
                // 描く側と同じ: 奥行きは xy の拡縮の平均で伸びる(depth_scaled)。
                let scale = frame.slots.get(&layer).map_or(self.pair(layer, property::SCALE, [1.0, 1.0], t)?, |s| s.scale);
                let d = self.analysis().and_then(|a| a.extent(path)).map_or(0.0, |e| e[2]) * (scale[0].abs() + scale[1].abs()) * 0.5;
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
                    _ => self.layer_box(layer, t)?.unwrap_or([0.0; 4]),
                };
                let scale = self.pair(layer, property::SCALE, [1.0, 1.0], t)?;
                let anchor = self.item_anchor(layer, t, bounds)?;
                let (lo, hi) = footprint(bounds, [range[0], range[1]], anchor, [scale[0], scale[1], 1.0], rotation);
                Ok([lo[2], hi[2]])
            }
            None => Ok(range),
        }
    }

    /// Transform Origin が Anchor 以外なら、箱の中のその点(素材座標)。
    pub(crate) fn origin_in(&self, layer: LayerId, t: RationalTime, bounds: [f32; 4]) -> Result<Option<[f32; 2]>, StoreError> {
        let origin = self.choice(layer, TRANSFORM_ORIGIN, t)?;
        if origin <= 0 {
            return Ok(None);
        }
        let (col, row) = ((origin - 1) % 3, (origin - 1) / 3);
        let at = |k: i64, lo: f32, hi: f32| lo + (hi - lo) * k as f32 * 0.5;
        Ok(Some([at(col, bounds[0], bounds[2]), at(row, bounds[1], bounds[3])]))
    }

    /// 並ばない層の中心: Transform Origin があれば箱から、無ければ Anchor の値。
    pub(crate) fn free_anchor(&self, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
        if self.choice(layer, TRANSFORM_ORIGIN, t)? > 0 {
            if let Some(b) = self.layer_box(layer, t)? {
                if let Some(origin) = self.origin_in(layer, t, b)? {
                    return Ok(origin);
                }
            }
        }
        self.pair(layer, property::ANCHOR, [0.0, 0.0], t)
    }

    /// 並ぶ子の拡縮・回転の中心。Transform Origin があれば箱の中のその点、Anchor を書いた層はその値、書いていなければ箱の中心(CSS の transform-origin)。
    pub(super) fn item_anchor(&self, layer: LayerId, t: RationalTime, bounds: [f32; 4]) -> Result<[f32; 2], StoreError> {
        if let Some(origin) = self.origin_in(layer, t, bounds)? {
            return Ok(origin);
        }
        Ok(match self.value_at(layer, &PropertyId::new(property::ANCHOR)?, t)? {
            Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32],
            _ => [(bounds[0] + bounds[2]) * 0.5, (bounds[1] + bounds[3]) * 0.5],
        })
    }

    /// 並べる前の奥行きの範囲(Scale Z 込み、Object Fit と回転は無し)。葉の箱を測る時。
    pub(super) fn raw_depth(&self, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
        self.depth_range(layer, t, &Frame::default())
    }

    /// 回転の欄(Tilt X・Tilt Y・Rotation)。
    pub(super) fn layout_rotation(&self, layer: LayerId, t: RationalTime) -> Result<[f32; 3], StoreError> {
        Ok([self.number(layer, LAYOUT_TILT_X, 0.0, t)? as f32, self.number(layer, LAYOUT_TILT_Y, 0.0, t)? as f32, self.number(layer, LAYOUT_ROTATION, 0.0, t)? as f32])
    }

    /// Depth Alignment: 子の奥行きの最大が Group の奥行き。Back = 子の奥を面(z = 0)に、Front = 子の手前を
    /// Group の手前(-奥行き)に、Center = 中心を揃える(visionOS の depthAlignment)。
    pub(super) fn align_depth(&self, group: LayerId, t: RationalTime, children: &HashMap<LayerId, Vec<(i16, LayerId)>>, frame: &mut Frame) -> Result<(), StoreError> {
        let alignment = self.choice(group, DEPTH_ALIGNMENT, t)?;
        let mut ranges = Vec::new();
        for &(_, child) in children.get(&group).map(Vec::as_slice).unwrap_or(&[]) {
            if frame.slots.contains_key(&child) {
                ranges.push((child, self.depth_range(child, t, frame)?));
            }
        }
        if self.display(group, t)? == 1 && self.choice(group, FLEX_DIRECTION, t)? == DIRECTION_DEPTH {
            // 奥へ積む: 重ね順の上(番号の大きい物)が一番手前、面(z = 0)に手前を合わせ、奥行き + Gap ずつ奥へ。
            let gap = self.number(group, GAP, 0.0, t)? as f32;
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
}
