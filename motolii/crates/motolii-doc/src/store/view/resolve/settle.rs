//! 解いた列を整える手 — 層が出揃ってから、並びと値を順に直す。
//! 順番そのものが法(背景を後ろへ → 面へ乗せる → つなぐ線を前へ → 型紙を配る →
//! 格子へ寄せる → 見つけた格子へ寄せる → Field を掛ける)。ここを入れ替えると絵が変わる。

use super::*;

impl<'a> StoreView<'a> {
    /// 解いた列を、決まった順で整える。並べ替えと描き順の番号付けまでが一組。
    /// 解く側はこの一口だけ使う(手を 1 つずつ呼ばせると、順番が呼ぶ側の記憶になる)。
    pub(super) fn settle(&self, out: &mut Vec<ResolvedLayer>, t: RationalTime) -> Result<(), StoreError> {
        self.put_backgrounds_behind(out, t)?;
        self.put_on_planes(out, t)?;
        self.put_connectors_in_front(out, t)?;
        self.hand_out_stencils(out)?;
        self.snap_to_grids(out, t)?;
        self.snap_to_found_grids(out, t)?;
        self.apply_fields(out, t)?;
        out.sort_by_key(|layer| (layer.placement.order, layer.source != crate::doc::store::LayerSource::Group));
        // 描き順は並べた順の番号(同じ order の写し同士を描く側の同点に委ねると、描き順が揺れる)。
        for (rank, layer) in out.iter_mut().enumerate() {
            layer.placement.order = i32::try_from(rank).unwrap_or(i32::MAX);
        }
        Ok(())
    }

    /// Field(C4D の Fields の Box): 指した層の箱からの距離で、画面の上の大きさ・不透明度・押し出しを変える。写しは 1 枚ずつ。
    fn apply_fields(&self, out: &mut [ResolvedLayer], t: RationalTime) -> Result<(), StoreError> {
        use crate::doc::store::layout::{FIELD, FIELD_FALLOFF, FIELD_OPACITY, FIELD_PUSH, FIELD_SCALE};
        let mut field_boxes: HashMap<LayerId, Option<(glam::Vec2, glam::Vec2)>> = HashMap::new();
        for i in 0..out.len() {
            let layer = &out[i];
            if layer.ghost || matches!(layer.source, crate::doc::store::LayerSource::Group | crate::doc::store::LayerSource::Camera | crate::doc::store::LayerSource::Null) {
                continue;
            }
            let field = match self.value_at(layer.id, &PropertyId::new(FIELD)?, t)? {
                Some(Value::LayerId(id)) if id != 0 && id != layer.id.0 => LayerId(id),
                Some(Value::F64(v)) if v >= 1.0 && v.round() as u64 != layer.id.0 => LayerId(v.round() as u64),
                _ => continue,
            };
            // 場の箱(画面)は場の層 1 つにつき 1 回。
            if !field_boxes.contains_key(&field) {
                let found = out.iter().find(|l| l.id == field && !l.ghost && l.copy == 0).map(|l| l.placement.transform);
                let b = match found {
                    Some(transform) => self.screen_box_of(field, transform, t)?,
                    None => None,
                };
                field_boxes.insert(field, b);
            }
            let Some((f_lo, f_hi)) = field_boxes[&field] else { continue };
            let Some((lo, hi)) = self.screen_box_of(layer.id, layer.placement.transform, t)? else { continue };
            let centre = (lo + hi) * 0.5;
            // 箱までの距離(中にいれば 0)。
            let outside = (f_lo - centre).max(centre - f_hi).max(glam::Vec2::ZERO);
            let falloff = self.number(layer.id, FIELD_FALLOFF, 200.0, t)?.max(0.0) as f32;
            let u = if falloff > 1e-3 { (1.0 - outside.length() / falloff).clamp(0.0, 1.0) } else if outside.length() <= 0.0 { 1.0 } else { 0.0 };
            let strength = u * u * (3.0 - 2.0 * u);
            if strength <= 0.0 {
                continue;
            }
            let scale = 1.0 + (self.number(layer.id, FIELD_SCALE, 1.0, t)? as f32 - 1.0) * strength;
            let opacity = 1.0 + (self.number(layer.id, FIELD_OPACITY, 1.0, t)? as f32 - 1.0) * strength;
            let away = centre - (f_lo + f_hi) * 0.5;
            let push = self.number(layer.id, FIELD_PUSH, 0.0, t)? as f32 * strength;
            let shift = if away.length() > 1e-3 { away.normalize() * push } else { glam::Vec2::ZERO };
            let adjust = glam::Affine2::from_translation(centre + shift) * glam::Affine2::from_scale(glam::Vec2::splat(scale)) * glam::Affine2::from_translation(-centre);
            let layer = &mut out[i];
            layer.placement.transform = adjust * layer.placement.transform;
            layer.placement.opacity = (layer.placement.opacity * opacity).clamp(0.0, 1.0);
            if let Some(world) = layer.placement.world_transform {
                let adjust3 = glam::Affine3A::from_translation((centre + shift).extend(0.0)) * glam::Affine3A::from_scale(glam::vec3(scale, scale, 1.0)) * glam::Affine3A::from_translation((-centre).extend(0.0));
                layer.placement.world_transform = Some(adjust3 * world);
            }
        }
        Ok(())
    }

    /// 層の箱を、与えた画面の変換で画面の上に(軸に沿った箱)。形は伸ばす前の輪郭。
    fn screen_box_of(&self, layer: LayerId, transform: glam::Affine2, t: RationalTime) -> Result<Option<(glam::Vec2, glam::Vec2)>, StoreError> {
        let Some(b) = self.layer_box(layer, t)? else { return Ok(None) };
        let corners = [[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| transform.transform_point2(glam::Vec2::from(c)));
        Ok(Some((corners.iter().fold(glam::Vec2::MAX, |a, p| a.min(*p)), corners.iter().fold(glam::Vec2::MIN, |a, p| a.max(*p)))))
    }

    /// 見つけた格子へ寄せる(2026-09-15 利用者「ものは動かします。グリッドが動的に動くと Tracery のような効果になる」):
    /// 寄せる気のある効果に、拾った物の箱の辺を渡して線を立ててもらい、その線へ画面の上で寄せる。
    /// 線は寄せる前の箱から立てる(寄せた結果を読み直さない)。どの辺が同じ線に乗るかはコアの知る所ではない。
    fn snap_to_found_grids(&self, out: &mut [ResolvedLayer], t: RationalTime) -> Result<(), StoreError> {
        // 相手を拾う効果は層に 1 つ(最初の 1 つ)。それが寄せる気でなければ、この層は寄せない。
        let snappers: Vec<(LayerId, crate::doc::store::kind::Snapping)> = out.iter().filter(|l| !l.ghost && l.copy == 0).filter_map(|l| {
            let effect = l.effects.iter().find(|e| self.is_snap_effect(&e.plugin_id))?;
            Some((l.id, self.snapping_of(&effect.plugin_id, &effect.params)?))
        }).collect();
        for (overlay_layer, snap) in snappers {
            let scope = self.overlay_scope(overlay_layer, out, t)?;
            if scope.is_empty() {
                continue;
            }
            let xs: Vec<f32> = scope.iter().flat_map(|(_, b)| [b[0], b[2]]).collect();
            let ys: Vec<f32> = scope.iter().flat_map(|(_, b)| [b[1], b[3]]).collect();
            let (lx, ly) = ((snap.lines)(&xs, snap.merge), (snap.lines)(&ys, snap.merge));
            for (k, &(index, b)) in scope.iter().enumerate() {
                if out[index].source == crate::doc::store::LayerSource::Group {
                    continue;
                }
                let dx = ((lx[2 * k] - b[0]) + (lx[2 * k + 1] - b[2])) * 0.5 * snap.strength;
                let dy = ((ly[2 * k] - b[1]) + (ly[2 * k + 1] - b[3])) * 0.5 * snap.strength;
                let layer = &mut out[index];
                layer.placement.transform = glam::Affine2::from_translation(glam::vec2(dx, dy)) * layer.placement.transform;
                if let Some(world) = layer.placement.world_transform {
                    layer.placement.world_transform = Some(glam::Affine3A::from_translation(glam::vec3(dx, dy, 0.0)) * world);
                }
            }
        }
        Ok(())
    }

    /// 格子へ吸い付く(2026-09-15 利用者「無作為の配置も、グリッドで整えると意図した物という観点が付与される」):
    /// Grid の Group の子(流れの外の子、Repeater の写しは 1 枚ずつ)の箱の左上を、一番近い升目の角へ Snap to Grid の強さで寄せる。
    /// Snap Size が Fields なら、右下も一番近い升目の終わりへ(大きさを升目の倍数に)。寄せるのは画面の変換だけ(書類の値は変えない)。
    fn snap_to_grids(&self, out: &mut [ResolvedLayer], t: RationalTime) -> Result<(), StoreError> {
        use crate::doc::store::layout::{SNAP_SIZE, SNAP_TO_GRID};
        let mut frame = None;
        for layer in out.iter_mut() {
            if layer.ghost || matches!(layer.source, crate::doc::store::LayerSource::Group | crate::doc::store::LayerSource::Camera | crate::doc::store::LayerSource::Null) {
                continue;
            }
            let strength = self.number(layer.id, SNAP_TO_GRID, 0.0, t)?.clamp(0.0, 1.0) as f32;
            if strength <= 0.0 {
                continue;
            }
            let Some(parent) = self.attrs(layer.id)?.unwrap_or_default().parent else { continue };
            if self.display(parent, t)? != 2 {
                continue;
            }
            let frame = match &frame { Some(f) => f, None => { frame = Some(self.layout_frame(t)?); frame.as_ref().unwrap() } };
            let Some((columns, rows)) = frame.fields.get(&parent) else { continue };
            if columns.is_empty() || rows.is_empty() {
                continue;
            }
            let b = match layer.source {
                crate::doc::store::LayerSource::Shape => crate::doc::store::layout::stretched_shape_box(&self.shapes_at(layer.id, t)?, layer.shape_stretch),
                _ => self.layer_box(layer.id, t)?,
            };
            let Some(b) = b else { continue };
            let group = self.world_2d(parent, t)?;
            let to_local = group.inverse() * layer.placement.transform;
            let corners = [[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| to_local.transform_point2(glam::Vec2::from(c)));
            let lo = corners.iter().fold(glam::Vec2::MAX, |a, p| a.min(*p));
            let hi = corners.iter().fold(glam::Vec2::MIN, |a, p| a.max(*p));
            let nearest = |value: f32, candidates: &mut dyn Iterator<Item = f32>| candidates.min_by(|a, b| (a - value).abs().total_cmp(&(b - value).abs())).unwrap_or(value);
            let start = glam::vec2(nearest(lo.x, &mut columns.iter().map(|c| c.0)), nearest(lo.y, &mut rows.iter().map(|r| r.0)));
            let size = hi - lo;
            let target_size = if self.choice(layer.id, SNAP_SIZE, t)? == 1 {
                let end_x = nearest(hi.x, &mut columns.iter().map(|c| c.1).filter(|e| *e > start.x));
                let end_y = nearest(hi.y, &mut rows.iter().map(|r| r.1).filter(|e| *e > start.y));
                glam::vec2(end_x - start.x, end_y - start.y)
            } else {
                size
            };
            let k = glam::vec2(if size.x > 1e-3 { target_size.x / size.x } else { 1.0 }, if size.y > 1e-3 { target_size.y / size.y } else { 1.0 });
            let at = lo.lerp(start, strength);
            let scale = glam::Vec2::ONE.lerp(k, strength);
            let adjust = glam::Affine2::from_translation(at) * glam::Affine2::from_scale(scale) * glam::Affine2::from_translation(-lo);
            layer.placement.transform = group * adjust * group.inverse() * layer.placement.transform;
            if let Some(world) = layer.placement.world_transform {
                let group3 = self.world_transform3d(parent, t).unwrap_or(glam::Affine3A::IDENTITY);
                let adjust3 = glam::Affine3A::from_translation(at.extend(0.0)) * glam::Affine3A::from_scale(scale.extend(1.0)) * glam::Affine3A::from_translation((-lo).extend(0.0));
                layer.placement.world_transform = Some(group3 * adjust3 * group3.inverse() * world);
            }
        }
        Ok(())
    }

    /// Stencil / Silhouette の層(クリッピングマスクの逆、2026-09-15 利用者裁定): 自分の形を、範囲の層へ matte として配る。
    /// 範囲は clip していれば自分のクリップの土台(束の上の層は土台の形で切られるので一緒に切れる)、していなければ
    /// 同じ Group の中で自分より下の層(と、その子孫)。自分の matte(clip の土台)は外す — 土台を切る側なので巡る。
    /// 1 つの層に matte は 1 つ: 既に track matte を持つ層は切らない。複数の Stencil が重なる層は、一番近い(order の低い)物。
    fn hand_out_stencils(&self, out: &mut [ResolvedLayer]) -> Result<(), StoreError> {
        let mut stencils: Vec<(LayerId, i16, bool, crate::doc::store::MatteMode, Option<LayerId>)> = Vec::new();
        for layer in out.iter().filter(|l| l.blend_mode.is_stencil() && !l.ghost && l.copy == 0) {
            let Some(meta) = self.meta(layer.id)? else { continue };
            let mode = if layer.blend_mode == crate::doc::store::BlendMode::SilhouetteAlpha {
                crate::doc::store::MatteMode::InvertedAlpha
            } else {
                crate::doc::store::MatteMode::Alpha
            };
            stencils.push((layer.id, meta.order, layer.clip_to_below, mode, self.attrs(layer.id)?.unwrap_or_default().parent));
        }
        if stencils.is_empty() {
            return Ok(());
        }
        // 近い物が勝つよう、order の高い Stencil から配って低い物で上書きする。
        stencils.sort_by_key(|s| std::cmp::Reverse(s.1));
        let handed: std::collections::HashSet<LayerId> = std::collections::HashSet::new();
        let mut handed = handed;
        for &(stencil, order, clipped, mode, parent) in &stencils {
            let matte = crate::doc::store::Matte { layer: stencil, mode };
            let base = if clipped { self.clipping_base(stencil)? } else { None };
            for i in 0..out.len() {
                let layer = &out[i];
                if layer.id == stencil || layer.ghost || layer.clip_to_below || layer.plate.is_some() {
                    continue;
                }
                if layer.matte.is_some() && !handed.contains(&layer.id) {
                    continue;
                }
                // 板にならない Group は自分では描かないので、切るのはその子孫(板の Group は板ごと切る)。
                let is_plate = |id: LayerId| out.iter().any(|l| l.plate == Some(id));
                if layer.source == crate::doc::store::LayerSource::Group && !is_plate(layer.id) {
                    continue;
                }
                // 祖先を辿る(自分自身を含む)。
                let mut chain = vec![layer.id];
                let mut guard = 0;
                while let Some(up) = self.attrs(*chain.last().unwrap_or(&layer.id))?.unwrap_or_default().parent {
                    chain.push(up);
                    guard += 1;
                    if guard > 64 { break; }
                }
                let inside = if clipped {
                    base.is_some_and(|b| chain.contains(&b))
                } else {
                    // 自分の親の直下の祖先の order が自分より下なら範囲。
                    let mut found = None;
                    for (k, id) in chain.iter().enumerate() {
                        let up = chain.get(k + 1).copied();
                        if up == parent {
                            found = Some(*id);
                            break;
                        }
                    }
                    match found {
                        Some(top) => top != stencil && self.meta(top)?.is_some_and(|m| m.order < order),
                        None => false,
                    }
                };
                if inside {
                    out[i].matte = Some(matte);
                    handed.insert(out[i].id);
                }
            }
            if let Some(layer) = out.iter_mut().find(|l| l.id == stencil) {
                layer.matte = None;
            }
        }
        Ok(())
    }

    /// 並べる Group の面に乗る物へ、その面の基準点を付ける(箱の奥行きの法 3)。面は、z・Tilt・Depth の無い層を
    /// 親へ辿って届く一番外の Display の Group(面の Group 自身は傾いてよい)。
    fn put_on_planes(&self, out: &mut [ResolvedLayer], t: RationalTime) -> Result<(), StoreError> {
        let flat: HashMap<LayerId, (bool, Option<glam::Affine3A>)> = out
            .iter()
            .filter(|l| !l.ghost && l.copy == 0)
            .map(|l| {
                let flat = l.placement.z == 0.0 && l.placement.rotation_x == 0.0 && l.placement.rotation_y == 0.0 && l.depth == 0.0;
                (l.id, (flat, l.placement.world_transform))
            })
            .collect();
        // 面の基準点は面の箱の中心(角だと、傾いた面同士で奥の面の角の方が camera に近くなる)。
        let centre = |id: LayerId, world: glam::Affine3A| {
            let b = self.layer_box(id, t).ok().flatten().unwrap_or([0.0; 4]);
            world.transform_point3(glam::vec3((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5, 0.0)).to_array()
        };
        let mut planes = HashMap::new();
        for layer in out.iter() {
            if planes.contains_key(&layer.id) {
                continue;
            }
            let mut plane = None;
            let mut here = layer.id;
            let mut seen = HashSet::new();
            while seen.insert(here) {
                let Some(&(flat_here, world)) = flat.get(&here) else { break };
                // 面の Group 自身はどう傾いてもよい。面から浮くのは、その下で z・Tilt・Depth を持つ物。
                if self.layout_display(here, t).unwrap_or(0) != 0 {
                    plane = world.map(|w| centre(here, w));
                }
                if !flat_here {
                    break;
                }
                match self.attrs(here)?.unwrap_or_default().parent {
                    Some(parent) => here = parent,
                    None => break,
                }
            }
            planes.insert(layer.id, plane);
        }
        for layer in out.iter_mut() {
            layer.placement.plane = planes.get(&layer.id).copied().flatten();
        }
        Ok(())
    }

    /// つなぐ線となぞる形は、結ぶ相手の奥行きを継ぐ(関係の線は相手より手前。2.5D では線は面を持たず既定の奥に落ち、
    /// 部屋の背景の下に隠れていた — 2026-09-16 の穴、利用者の裁定 2026-09-18)。面は相手の面、描き順は相手の 1 つ上。
    fn put_connectors_in_front(&self, out: &mut [ResolvedLayer], t: RationalTime) -> Result<(), StoreError> {
        let ends: Vec<(usize, LayerId, Option<LayerId>)> = out.iter().enumerate().filter(|(_, l)| l.copy == 0).filter_map(|(i, l)| {
            match self.connection(l.id, t) {
                Ok(Some((from, to))) => Some((i, from, Some(to))),
                _ => match self.tracing(l.id, t) { Ok(Some((target, _))) => Some((i, target, None)), _ => None },
            }
        }).collect();
        for (i, from, to) in ends {
            let find = |id: LayerId| out.iter().find(|l| l.id == id && l.copy == 0).map(|l| (l.placement.order, l.placement.plane));
            let (Some(a), b) = (find(from), to.and_then(find)) else { continue };
            let order = b.map_or(a.0, |b| a.0.max(b.0));
            let plane = a.1.or(b.and_then(|b| b.1));
            out[i].placement.order = order + 1;
            if plane.is_some() {
                out[i].placement.plane = plane;
            }
        }
        Ok(())
    }

    /// 並べる Group の背景は子孫の一番奥の、さらに 1 つ下に積む(同じ番号だと描き順の鍵が同点になる)。
    fn put_backgrounds_behind(&self, out: &mut [ResolvedLayer], t: RationalTime) -> Result<(), StoreError> {
        let mut deepest: HashMap<LayerId, i32> = HashMap::new();
        for layer in out.iter() {
            let mut seen = HashSet::from([layer.id]);
            let mut next = self.attrs(layer.id)?.unwrap_or_default().parent;
            while let Some(group) = next.filter(|g| seen.insert(*g)) {
                let order = deepest.entry(group).or_insert(layer.placement.order);
                *order = (*order).min(layer.placement.order);
                next = self.attrs(group)?.unwrap_or_default().parent;
            }
        }
        for layer in out.iter_mut() {
            if layer.source == crate::doc::store::LayerSource::Group {
                if let Some(order) = deepest.get(&layer.id).filter(|_| self.layout_display(layer.id, t).unwrap_or(0) != 0) {
                    layer.placement.order = layer.placement.order.min(order.saturating_sub(1));
                }
            }
        }
        Ok(())
    }
}
