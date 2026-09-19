
mod camera;
mod settle;
mod effects;
mod mask;
mod text;
mod transform;

use std::collections::{HashMap, HashSet};

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;

use crate::doc::store::{
    property, LayerId, LayerPlacement, PropertyId, ResolvedEffect, ResolvedLayer,
    ResolvedMask, StoreError, TextDocument,
};
use crate::doc::extensions::{placement};

use super::StoreView;

impl<'a> StoreView<'a> {





    pub fn resolve(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Option<ResolvedLayer>, StoreError> {
        let any_solo = self.any_solo(t)?;
        let world_transforms = self.world_transforms3d(t)?;
        let present: HashSet<LayerId> = self.layers().into_iter().collect();
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        self.resolve_with_solo(layer, t, any_solo, &present, &world_transforms, &mut memo, &mut visiting)
    }

    fn resolve_with_solo(
        &self,
        layer: LayerId,
        t: RationalTime,
        any_solo: bool,
        present: &HashSet<LayerId>,
        world_transforms: &HashMap<LayerId, glam::Affine3A>,
        memo: &mut HashMap<LayerId, glam::Affine2>,
        visiting: &mut HashSet<LayerId>,
    ) -> Result<Option<ResolvedLayer>, StoreError> {
        let Some(meta) = self.meta(layer)? else {
            return Ok(None);
        };

        let attrs = self.attrs(layer)?.unwrap_or_default();
        let hidden = self.resolved_hidden(layer, t, attrs.hidden)?;
        if hidden {
            return Ok(None);
        }

        let solo = self.resolved_solo(layer, t, attrs.solo)?;
        if any_solo && !solo {
            return Ok(None);
        }

        let Some(composition) = self.composition()? else {
            return Ok(None);
        };
        let comp_frame = t
            .try_to_frame_floor(composition.fps)
            .map_err(|e| StoreError::Property(e.to_string()))?;
        let Some(mut source_frame) = meta.timing.source_frame(comp_frame) else {
            return Ok(None);
        };
        if let Some(speed_track) = self.track(layer, &PropertyId::new(property::SPEED)?)? {
            if let Some(v) =
                meta.timing
                    .source_frame_with_speed_track(comp_frame, &speed_track, composition.fps)?
            {
                source_frame = v;
            }
        }
        if let Some(remap) = self.value_at(layer, &PropertyId::new(property::TIME_REMAP)?, t)? {
            match remap {
                Value::F64(v) => source_frame = v.floor() as i64,
                other => {
                    return Err(StoreError::Property(format!(
                        "{} に数値でない値が入っている: {other:?}",
                        property::TIME_REMAP
                    )))
                }
            }
        }
        let size = meta.source.declared_size().unwrap_or([0.0, 0.0]);

        let scalar = |name: &str, default: f32| -> Result<f32, StoreError> {
            let property = PropertyId::new(name)?;
            match self.value_at(layer, &property, t)? {
                Some(Value::F64(v)) => Ok(v as f32),
                Some(other) => Err(StoreError::Property(format!(
                    "{name} に数値でない値が入っている: {other:?}"
                ))),
                None => Ok(default),
            }
        };

        let transform = self.world_affine(layer, t, present, memo, visiting)?;
        let mut effects = self.resolved_effects(layer, t)?;
        let (handed, plate) = self.handed_down(layer, t, present)?;
        effects.extend(handed);

        Ok(Some(ResolvedLayer {
            id: layer,
            placement: LayerPlacement {
                transform,
                world_transform: world_transforms.get(&layer).copied(),
                opacity: scalar(property::OPACITY, 1.0)?.clamp(0.0, 1.0),
                order: i32::from(meta.order),
                z: scalar(property::POSITION_Z, 0.0)? + self.laid_out(layer, t)?.map_or(0.0, |slot| slot.z),
                rotation_x: scalar(property::ROTATION_X, 0.0)? + self.laid_out(layer, t)?.map_or(0.0, |slot| slot.rotation[0]),
                rotation_y: scalar(property::ROTATION_Y, 0.0)? as f32 + self.laid_out(layer, t)?.map_or(0.0, |slot| slot.rotation[1]),
                plane: None,
            },
            declared_size: size,
            source: meta.source,
            source_frame,
            source_time: RationalTime::try_from_frame(source_frame, composition.fps)
                .map_err(|e| StoreError::Property(e.to_string()))?,
            masks: self.masks_of(layer, t, present, memo, visiting)?,
            effects,
            blend_mode: self.resolved_blend_mode(layer, t, attrs.blend_mode)?,
            matte: if attrs.clip_to_below {
                self.clipping_base(layer)?.map(|base| crate::doc::store::Matte {
                    layer: base,
                    mode: crate::doc::store::MatteMode::Alpha,
                })
            } else {
                self.resolved_matte(layer, t, attrs.matte)?
            },
            clip_to_below: attrs.clip_to_below,
            projection: attrs.projection,
            flatten: attrs.flatten,
            environment: attrs.environment,
            depth: scalar(property::DEPTH, 0.0)?.max(0.0),
            blocks_light: attrs.blocks_light,
            ghost: false,
            copy: 0,
            after_effects: plate.as_ref().map(|(_, effects)| effects.clone()).unwrap_or_default(),
            plate: plate.map(|(group, _)| group),
            averaged: 0,
            shape_stretch: self.laid_out(layer, t)?.map_or([1.0, 1.0], |slot| slot.stretch),
            glyph_offsets: self.glyph_offsets(layer, t)?,
            flow_around: self.flow_around(layer, t)?,
        }))
    }



    /// Track Overlay(Detection Method = Layers)が拾う物: 同じ親で自分より下の、描かれる層(Repeater の写しは 1 枚ずつ)と、その画面の上の箱。
    /// 並べる Group(Display を持つ箱)は 1 つの箱として拾う(提案 2026-09-15)。線は立てるが寄せない(子と、箱を読むつなぐ線・札が寄せた位置を知らない)。
    pub fn overlay_scope(&self, overlay: LayerId, resolved: &[ResolvedLayer], t: RationalTime) -> Result<Vec<(usize, [f32; 4])>, StoreError> {
        let Some(meta) = self.meta(overlay)? else { return Ok(Vec::new()) };
        let parent = self.attrs(overlay)?.unwrap_or_default().parent;
        let mut out = Vec::new();
        for (index, layer) in resolved.iter().enumerate() {
            if layer.id == overlay || layer.ghost || layer.placement.opacity <= 0.0 || layer.blend_mode.is_stencil()
                || matches!(layer.source, crate::doc::store::LayerSource::Camera | crate::doc::store::LayerSource::Null | crate::doc::store::LayerSource::Stage)
                || (layer.source == crate::doc::store::LayerSource::Group && self.display(layer.id, t)? == 0) {
                continue;
            }
            if self.attrs(layer.id)?.unwrap_or_default().parent != parent || !self.meta(layer.id)?.is_some_and(|m| m.order < meta.order) {
                continue;
            }
            let b = match layer.source {
                crate::doc::store::LayerSource::Shape => crate::doc::store::layout::stretched_shape_box(&self.shapes_at(layer.id, t)?, layer.shape_stretch),
                _ => self.layer_box(layer.id, t)?,
            };
            let Some(b) = b else { continue };
            let corners = [[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| layer.placement.transform.transform_point2(glam::Vec2::from(c)));
            let lo = corners.iter().fold(glam::Vec2::MAX, |a, p| a.min(*p));
            let hi = corners.iter().fold(glam::Vec2::MIN, |a, p| a.max(*p));
            out.push((index, [lo.x, lo.y, hi.x, hi.y]));
        }
        Ok(out)
    }




    /// 配置効果を持つ層を、その配置の数だけ増やす。配置効果より上の効果は各配置の素材に、
    /// 下の効果は `after_effects` として全体に残す。時刻のずれた配置は、その時刻の姿を取り直す。
    /// グループなら子が素材の袋で、配置ごとに 1 つ引いた子の部分木を置く(裁定 2026-09-07)。
    #[allow(clippy::too_many_arguments)]
    fn push_placements(
        &self,
        base: ResolvedLayer,
        t: RationalTime,
        any_solo: bool,
        present: &HashSet<LayerId>,
        world_transforms: &HashMap<LayerId, glam::Affine3A>,
        memo: &mut HashMap<LayerId, glam::Affine2>,
        visiting: &mut HashSet<LayerId>,
        out: &mut Vec<ResolvedLayer>,
    ) -> Result<(), StoreError> {
        let blur = base.effects.iter().position(|e| crate::doc::extensions::motion::is_motion_blur(&e.plugin_id));
        let places = |e: &ResolvedEffect| self.placement_program(&e.plugin_id).is_some();
        let Some(first) = base.effects.iter().position(places) else {
            return match blur {
                Some(at) => self.push_motion_blur(base, at, t, present, world_transforms, memo, visiting, out),
                None => { out.push(base); Ok(()) }
            };
        };
        let params = &base.effects[first].params;
        let layer = base.id;
        // 形の素材は輪郭を伸ばし(線は太らない)、それ以外は置き場所で伸ばす。
        let stretch_outline = base.source == crate::doc::store::LayerSource::Shape;
        let program = self.placement_program(&base.effects[first].plugin_id).expect("matched placement program");
        let placements = (program.evaluate)(&crate::store::kind::PlacementInput {
            params, layer, time: t, stretch_outline, analysis: self.analysis(),
            position: if program.needs_position { self.resolve_position(layer, t)? } else { [0.0; 2] },
        });
        let parent = self.attrs(layer)?.unwrap_or_default().parent.filter(|p| present.contains(p));
        let is_group = base.source == crate::doc::store::LayerSource::Group;
        let children = if is_group { self.children_in_order(layer, present)? } else { Vec::new() };
        if is_group && children.is_empty() {
            return Ok(());
        }
        let whole = is_group && base.effects[first].scope == crate::doc::store::EffectScope::Whole;
        let whole_transform = params.iter().any(|(n, v)| n == "transform" && matches!(v, crate::doc::store::Value::F64(x) if x.round() == f64::from(placement::TRANSFORM_WHOLE)));
        let picks = placement::picks(params, &children.iter().map(|c| c.0).collect::<Vec<_>>(), placements.len());
        // 引いた子の写しは番号の順に重ねる(AE の Repeater・Cavalry の Duplicator)。子の order のまま並べ直すと、同じ子の写しが全部まとまって重なる。
        let copies_slot = if is_group && !whole {
            children.iter().filter_map(|c| self.meta(*c).ok().flatten().map(|m| m.order)).min()
        } else {
            None
        };
        for (ordinal, result) in placements.into_iter().enumerate() {
            let (placement, outline) = (result.placement, result.outline_stretch);
            let Ok(at) = t.try_sub(placement.time_offset) else { continue };
            let shifted = at != t;
            let subjects: Vec<LayerId> = if whole {
                self.subtree(layer, present)?.into_iter().skip(1).collect()
            } else if is_group {
                self.subtree(children[picks[ordinal]], present)?
            } else {
                vec![layer]
            };
            let mut worlds_at = HashMap::new();
            let (mut memo_at, mut visiting_at) = (HashMap::new(), HashSet::new());
            if shifted {
                for subject in &subjects {
                    worlds_at.extend(self.world_transform3d_chain(*subject, at, present)?);
                }
                if let Some(p) = parent {
                    worlds_at.extend(self.world_transform3d_chain(p, at, present)?);
                }
                if whole_transform {
                    worlds_at.extend(self.world_transform3d_chain(layer, at, present)?);
                }
            }
            let worlds = if shifted { &worlds_at } else { world_transforms };
            let memo = if shifted { &mut memo_at } else { &mut *memo };
            let visiting = if shifted { &mut visiting_at } else { &mut *visiting };
            let parent2 = parent.map(|p| self.world_affine(p, at, present, memo, visiting)).transpose()?.unwrap_or(glam::Affine2::IDENTITY);
            let parent3 = parent.and_then(|p| worlds.get(&p).copied()).unwrap_or(glam::Affine3A::IDENTITY);
            let pivot = glam::Vec2::from(self.resolve_position(layer, at)?);
            // Each: ずれは親の空間、回転と大きさは層の位置が中心(複製 1 つずつ)。
            // Whole: ずれは層自身の空間、中心は層のアンカー — 層を回すと並びごと回る。
            let (frame2, frame3, around) = if whole_transform {
                let anchor = match self.value_at(layer, &crate::doc::store::PropertyId::new(crate::doc::store::property::ANCHOR)?, at)? {
                    Some(crate::doc::store::Value::Vec2(v)) => glam::Vec2::new(v[0] as f32, v[1] as f32),
                    _ => glam::Vec2::ZERO,
                };
                (self.world_affine(layer, at, present, memo, visiting)?, worlds.get(&layer).copied().unwrap_or(glam::Affine3A::IDENTITY), anchor)
            } else {
                (parent2, parent3, pivot)
            };
            for subject in subjects {
                let mut copy = if !is_group && !shifted {
                    base.clone()
                } else {
                    let Some(copy) = self.resolve_with_solo(subject, at, any_solo, present, worlds, memo, visiting)? else { continue };
                    copy
                };
                if !is_group {
                    let Some(split) = copy.effects.iter().position(places) else {
                        out.push(copy);
                        continue;
                    };
                    copy.after_effects = copy.effects.split_off(split + 1);
                    copy.effects.pop();
                }
                copy.copy = placement.index;
                if let Some(slot) = copies_slot {
                    copy.placement.order = i32::from(slot);
                }
                copy.shape_stretch = outline;
                copy.placement.transform =
                    frame2 * placement.affine2(around) * frame2.inverse() * copy.placement.transform;
                if let Some(world) = copy.placement.world_transform {
                    let depth = if whole_transform { 0.0 } else { copy.placement.z };
                    copy.placement.world_transform = Some(
                        frame3 * placement.affine3(around.extend(depth)) * frame3.inverse() * world,
                    );
                    copy.placement.z += placement.offset_z;
                }
                copy.placement.opacity = (copy.placement.opacity * placement.opacity).clamp(0.0, 1.0);
                out.push(copy);
            }
        }
        Ok(())
    }

    /// Motion Blur: 1 コマの中のずらした時刻で位置・大きさ・角度だけを取り直した写しを、平均する枚数の印を付けて並べる。
    /// Motion Blur より下の効果は、平均した 1 枚に掛かる(`after_effects`)。
    #[allow(clippy::too_many_arguments)]
    fn push_motion_blur(
        &self,
        mut base: ResolvedLayer,
        at_index: usize,
        t: RationalTime,
        present: &HashSet<LayerId>,
        world_transforms: &HashMap<LayerId, glam::Affine3A>,
        memo: &mut HashMap<LayerId, glam::Affine2>,
        visiting: &mut HashSet<LayerId>,
        out: &mut Vec<ResolvedLayer>,
    ) -> Result<(), StoreError> {
        use crate::doc::extensions::motion;
        let frame_seconds = self.composition()?.map_or(0.0, |c| c.fps.den() as f64 / c.fps.num() as f64);
        let sampling = if base.source == crate::doc::store::LayerSource::Group { None }
            else { motion::sampling(&base.effects[at_index].params, t, frame_seconds) };
        let below = base.effects.split_off(at_index + 1);
        base.effects.pop();
        base.after_effects.splice(0..0, below);
        let Some(sampling) = sampling else {
            out.push(base);
            return Ok(());
        };
        let layer = base.id;
        let parent = self.attrs(layer)?.unwrap_or_default().parent.filter(|p| present.contains(p));
        let parent2 = parent.map(|p| self.world_affine(p, t, present, memo, visiting)).transpose()?.unwrap_or(glam::Affine2::IDENTITY);
        let parent3 = parent.and_then(|p| world_transforms.get(&p).copied()).unwrap_or(glam::Affine3A::IDENTITY);
        let now_inverse = self.local_placement_transform(layer, t)?.inverse();
        let local_delta = |at: RationalTime| -> Result<glam::Affine2, StoreError> {
            Ok(self.local_placement_transform_sampled(layer, t, Some((at, sampling.channels)))? * now_inverse)
        };
        let samples = sampling.deltas(base.declared_size, base.placement.transform, parent2, local_delta)?;
        let count = samples.len() as u32;
        if count <= 1 {
            out.push(base);
            return Ok(());
        }
        for (k, local) in samples.into_iter().enumerate() {
            let local = local?;
            let mut copy = base.clone();
            copy.copy = k as u32;
            copy.averaged = count;
            copy.placement.transform = parent2 * local * parent2.inverse() * base.placement.transform;
            if let Some(world) = base.placement.world_transform {
                let local3 = glam::Affine3A::from_mat3_translation(glam::Mat3::from_mat2(local.matrix2), local.translation.extend(0.0).into());
                copy.placement.world_transform = Some(parent3 * local3 * parent3.inverse() * world);
            }
            copy.placement.opacity = base.placement.opacity / count as f32;
            out.push(copy);
        }
        Ok(())
    }

    /// 直下の子を重ね順で。
    fn children_in_order(&self, group: LayerId, present: &HashSet<LayerId>) -> Result<Vec<LayerId>, StoreError> {
        let mut children = Vec::new();
        for id in present {
            if self.attrs(*id)?.unwrap_or_default().parent == Some(group) {
                children.push((self.meta(*id)?.map_or(0, |m| m.order), *id));
            }
        }
        children.sort();
        Ok(children.into_iter().map(|(_, id)| id).collect())
    }

    /// 層とその子孫(自分が先)。
    fn subtree(&self, root: LayerId, present: &HashSet<LayerId>) -> Result<Vec<LayerId>, StoreError> {
        let mut out = vec![root];
        let mut i = 0;
        while i < out.len() {
            let here = out[i];
            i += 1;
            for child in self.children_in_order(here, present)? {
                if !out.contains(&child) {
                    out.push(child);
                }
            }
        }
        Ok(out)
    }

    /// 配置効果を持つグループの子孫。単独では描かず、配置を通してだけ出る。
    fn handed_out_by_a_group(&self, present: &HashSet<LayerId>, t: RationalTime) -> Result<HashSet<LayerId>, StoreError> {
        let mut hidden = HashSet::new();
        for id in present {
            if !self.meta(*id)?.is_some_and(|m| m.source == crate::doc::store::LayerSource::Group) { continue; }
            for effect in self.effects(*id)? {
                if self.placement_program(&effect.plugin_id).is_some() && self.effect_enabled(*id, effect.id, t)? {
                    hidden.extend(self.subtree(*id, present)?.into_iter().skip(1));
                    break;
                }
            }
        }
        Ok(hidden)
    }

    fn any_solo(&self, t: RationalTime) -> Result<bool, StoreError> {
        for layer in self.layers() {
            if self.meta(layer)?.is_some_and(|m| matches!(m.source, crate::doc::store::LayerSource::Camera | crate::doc::store::LayerSource::Stage)) { continue; }
            let static_solo = self.attrs(layer)?.unwrap_or_default().solo;
            if self.resolved_solo(layer, t, static_solo)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn resolved_solo(
        &self,
        layer: LayerId,
        t: RationalTime,
        static_value: bool,
    ) -> Result<bool, StoreError> {
        match self.value_at(layer, &PropertyId::solo(), t)? {
            Some(Value::Bool(v)) => Ok(v),
            Some(other) => Err(StoreError::Property(format!(
                "solo に真偽でない値が入っている(track が壊れている): {other:?}"
            ))),
            None => Ok(static_value),
        }
    }

    fn resolved_hidden(
        &self,
        layer: LayerId,
        t: RationalTime,
        static_value: bool,
    ) -> Result<bool, StoreError> {
        match self.value_at(layer, &PropertyId::hidden(), t)? {
            Some(Value::Bool(v)) => Ok(v),
            Some(other) => Err(StoreError::Property(format!(
                "hidden に真偽でない値が入っている(track が壊れている): {other:?}"
            ))),
            None => Ok(static_value),
        }
    }

    fn resolved_blend_mode(
        &self,
        layer: LayerId,
        t: RationalTime,
        static_value: crate::doc::store::BlendMode,
    ) -> Result<crate::doc::store::BlendMode, StoreError> {
        match self.value_at(layer, &PropertyId::blend_mode(), t)? {
            Some(Value::Enum(v)) => crate::doc::store::BlendMode::from_enum_value(v).ok_or_else(|| {
                StoreError::Property(format!(
                    "`blend_mode` track に未知の enum 値が入っている: {v}"
                ))
            }),
            Some(other) => Err(StoreError::Property(format!(
                "`blend_mode` に enum でない値が入っている(track が壊れている): {other:?}"
            ))),
            None => Ok(static_value),
        }
    }

    fn resolved_matte(
        &self,
        layer: LayerId,
        t: RationalTime,
        static_value: Option<crate::doc::store::Matte>,
    ) -> Result<Option<crate::doc::store::Matte>, StoreError> {
        let Some(mut matte) = static_value else {
            return Ok(None);
        };
        match self.value_at(layer, &PropertyId::matte_mode(), t)? {
            Some(Value::Enum(v)) => {
                matte.mode = crate::doc::store::MatteMode::from_enum_value(v).ok_or_else(|| {
                    StoreError::Property(format!(
                        "`matte_mode` track に未知の enum 値が入っている: {v}"
                    ))
                })?;
            }
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "`matte_mode` に enum でない値が入っている(track が壊れている): {other:?}"
                )))
            }
            None => {}
        }
        Ok(Some(matte))
    }

    pub fn resolved_layers(&self, t: RationalTime) -> Result<Vec<ResolvedLayer>, StoreError> {
        let any_solo = self.any_solo(t)?;
        let world_transforms = self.world_transforms3d(t)?;
        let layers = self.layers();
        let present: HashSet<LayerId> = layers.iter().copied().collect();
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        let handed_out = self.handed_out_by_a_group(&present, t)?;
        let mut out = Vec::new();
        for layer in layers {
            if handed_out.contains(&layer) {
                continue;
            }
            // ゴーストは元より先に積む(同じ重ね順なら後の物が上に描かれるので、元が手前に来る)。
            self.push_ghosts(layer, t, any_solo, &present, &mut out)?;
            if let Some(resolved) =
                self.resolve_with_solo(layer, t, any_solo, &present, &world_transforms, &mut memo, &mut visiting)?
            {
                if self.push_split(&resolved, t, any_solo, &present, &mut out)? {
                    continue;
                }
                self.push_placements(resolved, t, any_solo, &present, &world_transforms, &mut memo, &mut visiting, &mut out)?;
            }
        }
        self.settle(&mut out, t)?;
        Ok(out)
    }

    /// 文字の Split(GSAP SplitText / CSS `sibling-index()`): 字・語・行の単位を、その文字の層の Stagger でずれた時刻に解いた
    /// 写しとして積む。書類に子の層は作らない — 写し = 層全体をずれた時刻で解き、単位の箱を Intersect の mask で切り、
    /// 拡縮・回転の中心を単位の箱の中心へ移す(SplitText の char が自分の中心で回るのと同じ)。時刻の純関数。
    fn push_split(
        &self,
        base: &ResolvedLayer,
        t: RationalTime,
        any_solo: bool,
        present: &HashSet<LayerId>,
        out: &mut Vec<ResolvedLayer>,
    ) -> Result<bool, StoreError> {
        use crate::doc::store::layout::STAGGER;
        if base.source != crate::doc::store::LayerSource::Text || self.number(base.id, STAGGER, 0.0, t)? <= 0.0 {
            return Ok(false);
        }
        let units = self.text_units(base.id, t)?;
        if units.len() < 2 {
            return Ok(false);
        }
        let layer = base.id;
        let n = units.len();
        let parent = self.attrs(layer)?.unwrap_or_default().parent.filter(|p| present.contains(p));
        for (k, b) in units.into_iter().enumerate() {
            let at = self.schedule_shift(layer, k, n, t)?;
            let (mut memo, mut visiting) = (HashMap::new(), HashSet::new());
            let worlds = self.world_transform3d_chain(layer, at, present)?;
            let Some(mut copy) = self.resolve_with_solo(layer, at, any_solo, present, &worlds, &mut memo, &mut visiting)? else { continue };
            // 中心を単位の箱の中心へ: 親の空間で T(c − a) を局所の変換に共役で掛ける(a = 層のアンカー、c = 箱の中心)。
            let anchor = glam::Vec2::from(self.free_anchor(layer, at)?);
            let shift = glam::vec2((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5) - anchor;
            let parent2 = parent.map(|p| self.world_affine(p, at, present, &mut memo, &mut visiting)).transpose()?.unwrap_or(glam::Affine2::IDENTITY);
            let shift2 = glam::Affine2::from_translation(shift);
            copy.placement.transform = parent2 * shift2 * parent2.inverse() * copy.placement.transform * shift2.inverse();
            if let Some(world) = copy.placement.world_transform {
                let parent3 = parent.and_then(|p| worlds.get(&p).copied()).unwrap_or(glam::Affine3A::IDENTITY);
                let shift3 = glam::Affine3A::from_translation(shift.extend(0.0));
                copy.placement.world_transform = Some(parent3 * shift3 * parent3.inverse() * world * shift3.inverse());
            }
            copy.copy = k as u32;
            copy.masks.push(ResolvedMask {
                mode: crate::doc::store::MaskMode::Intersect,
                inverted: false,
                opacity: 1.0,
                expansion: 0.0,
                shape: crate::doc::store::layout::rounded_rect_path(b, 0.0, glam::Affine2::IDENTITY),
                frame: crate::doc::store::MaskFrame::Layer,
            });
            out.push(copy);
        }
        Ok(true)
    }





    /// ゴースト: 層を遅れ d だけ後に見た姿を 1 枚、同じ id で `ghost = true` にして積む。
    /// 層に Repeater が掛かっていれば、その時刻の配置がそのまま増える。
    fn push_ghosts(
        &self,
        layer: LayerId,
        t: RationalTime,
        any_solo: bool,
        present: &HashSet<LayerId>,
        out: &mut Vec<ResolvedLayer>,
    ) -> Result<(), StoreError> {
        let attrs = self.attrs(layer)?.unwrap_or_default();
        if attrs.environment { return Ok(()) }
        let Some(composition) = self.composition()? else { return Ok(()) };
        let fps = composition.fps;
        let parent = attrs.parent;
        for delay in attrs.ghost {
            let Ok(shift) = RationalTime::try_from_frame(delay.abs(), fps) else { continue };
            let at = if delay >= 0 { t.try_sub(shift) } else { t.try_add(shift) };
            let Ok(at) = at else { continue };
            let mut worlds = self.world_transform3d_chain(layer, at, present)?;
            if let Some(parent) = parent {
                worlds.extend(self.world_transform3d_chain(parent, at, present)?);
            }
            let (mut memo, mut visiting) = (HashMap::new(), HashSet::new());
            let Some(resolved) = self.resolve_with_solo(layer, at, any_solo, present, &worlds, &mut memo, &mut visiting)? else {
                continue;
            };
            let mut placed = Vec::new();
            self.push_placements(resolved, at, any_solo, present, &worlds, &mut memo, &mut visiting, &mut placed)?;
            for mut copy in placed {
                copy.ghost = true;
                out.push(copy);
            }
        }
        Ok(())
    }
}




#[cfg(test)]
mod ghost_contract {
    use crate::doc::store::*;

    fn document() -> (Document, Fps) {
        let fps = Fps::try_new(10, 1).unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: 100, height: 100, fps, duration_frames: 40, background: [0.0; 4] })).unwrap();
        (doc, fps)
    }

    fn xs(doc: &Document, id: LayerId, frame: i64, fps: Fps) -> Vec<(bool, f32)> {
        let t = RationalTime::try_from_frame(frame, fps).unwrap();
        doc.view().resolved_layers(t).unwrap().into_iter().filter(|l| l.id == id).map(|l| (l.ghost, l.placement.transform.translation.x)).collect()
    }

    /// ゴーストは同じ層を d だけ遅れて見た姿が 1 枚。行は増えず同じ id で、元より先に積まれ(奥)、元は ghost = false。
    /// 層の帯の外(t − d が始まる前)では居ない。
    #[test]
    fn ghosts_are_the_layer_seen_d_frames_earlier_behind_the_layer() {
        let (mut doc, fps) = document();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Null, order: 0, timing: LayerTiming { start: 0, duration: 30, source_in: 0, speed: Speed::NORMAL } } },
        ]).unwrap();
        let track = crate::doc::eval::KeyframeTrack::try_from_keys(vec![
            crate::doc::eval::Keyframe { t: RationalTime::ZERO, value: Value::Vec2([0.0, 0.0]), interp: crate::doc::eval::Interp::Linear, spatial: None },
            crate::doc::eval::Keyframe { t: RationalTime::try_from_frame(10, fps).unwrap(), value: Value::Vec2([100.0, 0.0]), interp: crate::doc::eval::Interp::Linear, spatial: None },
        ]).unwrap();
        doc.apply(Intent::SetTrack { layer, property: PropertyId::new(property::POSITION).unwrap(), track }).unwrap();
        doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch { ghost: Some(Some(5)), ..Default::default() } }).unwrap();

        assert_eq!(xs(&doc, layer, 8, fps), vec![(true, 30.0), (false, 80.0)], "d = 5 は 3 コマ目の姿、元が最後(手前)");
        assert_eq!(xs(&doc, layer, 3, fps), vec![(false, 30.0)], "t − d が帯の前ならゴーストは居ない");
        doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch { hidden: Some(true), ..Default::default() } }).unwrap();
        assert!(xs(&doc, layer, 8, fps).is_empty(), "隠せばゴーストも消える");
    }
}
