
mod transform;

use std::collections::{HashMap, HashSet};

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;

use crate::doc::store::{
    placement, property, LayerId, LayerPlacement, PropertyId, ResolvedEffect, ResolvedLayer,
    ResolvedMask, StoreError, TextDocument,
};

use super::StoreView;

impl<'a> StoreView<'a> {
    /// Camera と同じ規則で active な非描画層を選ぶ: 区間内・可視・solo が優先・最上位。
    fn active_guide(&self, source: crate::doc::store::LayerSource, t: RationalTime) -> Result<Option<LayerId>, StoreError> {
        let frame = self.composition()?.map(|c| t.try_to_frame_floor(c.fps)).transpose()
            .map_err(|e| StoreError::Property(e.to_string()))?.unwrap_or(0);
        let mut guides = Vec::new();
        for id in self.layers() {
            if let Some(meta) = self.meta(id)? {
                if meta.source == source && meta.timing.covers(frame) {
                    let attrs = self.attrs(id)?.unwrap_or_default();
                    if !self.resolved_hidden(id, t, attrs.hidden)? { guides.push((self.resolved_solo(id, t, attrs.solo)?, meta.order, id)); }
                }
            }
        }
        guides.sort();
        Ok(guides.last().map(|(_, _, id)| *id))
    }

    pub fn resolve_stage_extent(&self, t: RationalTime) -> Result<crate::doc::store::StageExtent, StoreError> {
        let Some(id) = self.active_guide(crate::doc::store::LayerSource::Stage, t)? else { return Ok(Default::default()) };
        let mut margins = [0.0; 4];
        for (margin, name) in margins.iter_mut().zip(property::STAGE_MARGINS) {
            if let Some(Value::F64(v)) = self.value_at(id, &PropertyId::new(name)?, t)? { *margin = v.max(0.0) as f32; }
        }
        Ok(crate::doc::store::StageExtent { layer: Some(id), margins })
    }

    pub fn resolve_camera(&self, t: RationalTime) -> Result<crate::doc::core::ResolvedCamera, StoreError> {
        if let Some(id) = self.active_guide(crate::doc::store::LayerSource::Camera, t)? {
            let get = |name| self.value_at(id, &PropertyId::new(name)?, t);
            let center = match get(property::CAMERA_CENTER)? { Some(Value::Vec2(v)) => [v[0] as f32,v[1] as f32], _ => [0.0,0.0] };
            let zoom = match get(property::CAMERA_ZOOM)? { Some(Value::F64(v)) => v as f32, _ => 1.0 };
            let roll_degrees = match get(property::CAMERA_ROLL)? { Some(Value::F64(v)) => v as f32, _ => 0.0 };
            return Ok(crate::doc::core::ResolvedCamera { center, zoom, roll_degrees, ..Default::default() });
        }
        let center_property = PropertyId::camera(property::CAMERA_CENTER)?;
        let center = match self.camera_value_at(&center_property, t)? {
            Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32],
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に2成分でない値が入っている: {other:?}",
                    property::CAMERA_CENTER
                )))
            }
            None => [0.0, 0.0],
        };

        let zoom_property = PropertyId::camera(property::CAMERA_ZOOM)?;
        let zoom = match self.camera_value_at(&zoom_property, t)? {
            Some(Value::F64(v)) => v as f32,
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に数値でない値が入っている: {other:?}",
                    property::CAMERA_ZOOM
                )))
            }
            None => 1.0,
        };

        let roll_property = PropertyId::camera(property::CAMERA_ROLL)?;
        let roll_degrees = match self.camera_value_at(&roll_property, t)? {
            Some(Value::F64(v)) => v as f32,
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に数値でない値が入っている: {other:?}",
                    property::CAMERA_ROLL
                )))
            }
            None => 0.0,
        };

        Ok(crate::doc::core::ResolvedCamera {
            center,
            zoom,
            roll_degrees, ..Default::default() })
    }

    pub fn resolved_text_document(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Option<TextDocument>, StoreError> {
        let Some(mut document) = self.text_document(layer)? else {
            return Ok(None);
        };

        let justify_property = PropertyId::text_justify();
        match self.value_at(layer, &justify_property, t)? {
            Some(Value::Enum(v)) => {
                document.justify = crate::doc::store::TextJustify::from_enum_value(v).ok_or_else(|| {
                    StoreError::Property(format!(
                        "`text_justify` track に未知の enum 値が入っている: {v}"
                    ))
                })?;
            }
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "`text_justify` に enum でない値が入っている(track が壊れている): {other:?}"
                )))
            }
            None => {}
        }

        for style in &mut document.styles {
            let size_property = PropertyId::text_style_size(style.id);
            if let Some(value) = self.value_at(layer, &size_property, t)? {
                match value {
                    Value::F64(v) => style.size = v as f32,
                    other => {
                        return Err(StoreError::Property(format!(
                            "text_style.{}.size に数値でない値が入っている: {other:?}",
                            style.id
                        )))
                    }
                }
            }

            let line_height_property = PropertyId::text_style_line_height(style.id);
            if let Some(value) = self.value_at(layer, &line_height_property, t)? {
                match value {
                    Value::F64(v) => style.line_height = Some(v as f32),
                    other => {
                        return Err(StoreError::Property(format!(
                            "text_style.{}.line_height に数値でない値が入っている: {other:?}",
                            style.id
                        )))
                    }
                }
            }

            let tracking_property = PropertyId::text_style_tracking(style.id);
            if let Some(value) = self.value_at(layer, &tracking_property, t)? {
                match value {
                    Value::F64(v) => style.tracking = v as f32,
                    other => {
                        return Err(StoreError::Property(format!(
                            "text_style.{}.tracking に数値でない値が入っている: {other:?}",
                            style.id
                        )))
                    }
                }
            }

            let fill_property = PropertyId::text_style_fill_color(style.id);
            if let Some(value) = self.value_at(layer, &fill_property, t)? {
                match value {
                    Value::Color(c) => style.fill = c,
                    other => {
                        return Err(StoreError::Property(format!(
                            "text_style.{}.fill_color に色でない値が入っている: {other:?}",
                            style.id
                        )))
                    }
                }
            }

            let stroke_property = PropertyId::text_style_stroke_color(style.id);
            if let Some(value) = self.value_at(layer, &stroke_property, t)? {
                match value {
                    Value::Color(c) => style.stroke_color = Some(c),
                    other => {
                        return Err(StoreError::Property(format!(
                            "text_style.{}.stroke_color に色でない値が入っている: {other:?}",
                            style.id
                        )))
                    }
                }
            }
        }

        Ok(Some(document))
    }

    fn resolved_masks(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Vec<ResolvedMask>, StoreError> {
        let mut out = Vec::new();
        for mask in self.masks(layer)? {
            let mode = match self.value_at(layer, &PropertyId::mask_mode(mask.id), t)? {
                Some(Value::Enum(v)) => crate::doc::store::MaskMode::from_enum_value(v).ok_or_else(|| {
                    StoreError::Property(format!(
                        "マスク {} の mode track に未知の enum 値が入っている: {v}",
                        mask.id
                    ))
                })?,
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の mode に enum でない値が入っている(track が壊れている): {other:?}",
                        mask.id
                    )))
                }
                None => mask.mode,
            };

            let inverted = match self.value_at(layer, &PropertyId::mask_inverted(mask.id), t)? {
                Some(Value::Bool(v)) => v,
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の inverted に真偽でない値が入っている(track が壊れている): {other:?}",
                        mask.id
                    )))
                }
                None => mask.inverted,
            };

            let shape_property = PropertyId::mask_shape(mask.id);
            let shape = match self.value_at(layer, &shape_property, t)? {
                Some(Value::Path(path)) => path,
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の形状にパスでない値が入っている: {other:?}",
                        mask.id
                    )))
                }
                None => {
                    return Err(StoreError::Property(format!(
                        "マスク {} に形状が無い(`mask.{}.shape` が未設定)",
                        mask.id, mask.id
                    )))
                }
            };

            let opacity_property = PropertyId::mask_opacity(mask.id);
            let opacity = match self.value_at(layer, &opacity_property, t)? {
                Some(Value::F64(v)) => v as f32,
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の不透明度に数値でない値が入っている: {other:?}",
                        mask.id
                    )))
                }
                None => 1.0,
            };

            let expansion_property = PropertyId::mask_expansion(mask.id);
            let expansion = match self.value_at(layer, &expansion_property, t)? {
                Some(Value::F64(v)) if v.is_finite() => v,
                Some(Value::F64(v)) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の膨張に有限でない値が入っている: {v}",
                        mask.id
                    )))
                }
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "マスク {} の膨張に数値でない値が入っている: {other:?}",
                        mask.id
                    )))
                }
                None => 0.0,
            };

            out.push(ResolvedMask {
                mode,
                inverted,
                opacity: opacity.clamp(0.0, 1.0),
                expansion,
                shape,
            });
        }
        Ok(out)
    }

    fn resolved_effects(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Vec<ResolvedEffect>, StoreError> {
        let effects = self.effects(layer)?;
        if effects.is_empty() {
            return Ok(Vec::new());
        }

        let properties = self.properties(layer);

        let mut out = Vec::with_capacity(effects.len());
        for effect in effects {
            let enabled_property = crate::doc::store::PropertyId::effect_enabled(effect.id);
            let enabled = match self.value_at(layer, &enabled_property, t)? {
                Some(Value::Bool(v)) => v,
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "effect {} の enabled に真偽でない値が入っている: {other:?}",
                        effect.id
                    )))
                }
                None => true,
            };
            if !enabled {
                continue;
            }
            let prefix = format!("{}{}.param.", property::EFFECT_PREFIX, effect.id);
            let mut params = Vec::new();
            for candidate in &properties {
                let Some(param_name) = candidate.name().strip_prefix(prefix.as_str()) else {
                    continue;
                };
                if let Some(value) = self.value_at(layer, candidate, t)? {
                    params.push((param_name.to_owned(), value));
                }
            }
            out.push(ResolvedEffect {
                plugin_id: effect.plugin_id,
                params,
            });
        }
        Ok(out)
    }

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

        Ok(Some(ResolvedLayer {
            id: layer,
            placement: LayerPlacement {
                transform,
                world_transform: world_transforms.get(&layer).copied(),
                opacity: scalar(property::OPACITY, 1.0)?.clamp(0.0, 1.0),
                order: meta.order,
                z: scalar(property::POSITION_Z, 0.0)?,
                rotation_x: scalar(property::ROTATION_X, 0.0)?,
                rotation_y: scalar(property::ROTATION_Y, 0.0)? as f32,
            },
            declared_size: size,
            source: meta.source,
            source_frame,
            masks: self.resolved_masks(layer, t)?,
            effects: self.resolved_effects(layer, t)?,
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
            ghost: false,
            copy: 0,
            after_effects: Vec::new(),
        }))
    }

    /// 配置効果を持つ層を、その配置の数だけ増やす。配置効果より上の効果は各配置の素材に、
    /// 下の効果は `after_effects` として全体に残す。時刻のずれた配置は、その時刻の姿を取り直す。
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
        let Some(first) = base.effects.iter().position(|e| placement::kind(&e.plugin_id).is_some()) else {
            out.push(base);
            return Ok(());
        };
        let kind = placement::kind(&base.effects[first].plugin_id).expect("found above");
        let params = &base.effects[first].params;
        let placements = placement::placements(kind, params);
        let layer = base.id;
        let parent = self.attrs(layer)?.unwrap_or_default().parent.filter(|p| present.contains(p));
        let is_group = base.source == crate::doc::store::LayerSource::Group;
        let children = if is_group { self.children_in_order(layer, present)? } else { Vec::new() };
        if is_group && children.is_empty() {
            return Ok(());
        }
        let whole = is_group && placement::whole_group(params);
        let picks = placement::picks(params, &children.iter().map(|c| c.0).collect::<Vec<_>>(), placements.len());
        for placement in placements {
            let Ok(at) = t.try_sub(placement.time_offset) else { continue };
            let shifted = at != t;
            let subjects: Vec<LayerId> = if whole {
                self.subtree(layer, present)?.into_iter().skip(1).collect()
            } else if is_group {
                self.subtree(children[picks[placement.index as usize]], present)?
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
            }
            let worlds = if shifted { &worlds_at } else { world_transforms };
            let memo = if shifted { &mut memo_at } else { &mut *memo };
            let visiting = if shifted { &mut visiting_at } else { &mut *visiting };
            let parent2 = parent.map(|p| self.world_affine(p, at, present, memo, visiting)).transpose()?.unwrap_or(glam::Affine2::IDENTITY);
            let parent3 = parent.and_then(|p| worlds.get(&p).copied()).unwrap_or(glam::Affine3A::IDENTITY);
            let pivot = glam::Vec2::from(self.resolve_position(layer, at)?);
            for subject in subjects {
                let mut copy = if !is_group && !shifted {
                    base.clone()
                } else {
                    let Some(copy) = self.resolve_with_solo(subject, at, any_solo, present, worlds, memo, visiting)? else { continue };
                    copy
                };
                if !is_group {
                    let Some(split) = copy.effects.iter().position(|e| placement::kind(&e.plugin_id).is_some()) else {
                        out.push(copy);
                        continue;
                    };
                    copy.after_effects = copy.effects.split_off(split + 1);
                    copy.effects.pop();
                }
                copy.copy = placement.index;
                copy.placement.transform =
                    parent2 * placement.affine2(pivot) * parent2.inverse() * copy.placement.transform;
                if let Some(world) = copy.placement.world_transform {
                    copy.placement.world_transform = Some(
                        parent3 * placement.affine3(pivot.extend(copy.placement.z)) * parent3.inverse() * world,
                    );
                }
                copy.placement.opacity = (copy.placement.opacity * placement.opacity).clamp(0.0, 1.0);
                out.push(copy);
            }
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
    fn handed_out_by_a_group(&self, present: &HashSet<LayerId>) -> Result<HashSet<LayerId>, StoreError> {
        let mut hidden = HashSet::new();
        for id in present {
            if self.meta(*id)?.is_some_and(|m| m.source == crate::doc::store::LayerSource::Group)
                && self.effects(*id)?.iter().any(|e| placement::kind(&e.plugin_id).is_some())
            {
                hidden.extend(self.subtree(*id, present)?.into_iter().skip(1));
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
        let handed_out = self.handed_out_by_a_group(&present)?;
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
                self.push_placements(resolved, t, any_solo, &present, &world_transforms, &mut memo, &mut visiting, &mut out)?;
            }
        }
        out.sort_by_key(|layer| layer.placement.order);
        Ok(out)
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
mod stage_extent_contract {
    use crate::doc::store::*;

    /// Boxcam の working comp / AE の guide layer: 区間内の最上位が効き、区間の外では出力枠に戻る。
    #[test]
    fn the_topmost_stage_layer_in_range_widens_the_frame_and_nothing_else_does() {
        let mut doc = blank_project();
        let comp = doc.view().composition().unwrap().unwrap().spec();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |frame| RationalTime::try_from_frame(frame, fps).unwrap();
        for (id, order, start, margins) in [(1u64, 0i16, 0i64, [100.0, 0.0, 100.0, 0.0]), (2, 5, 10, [0.0, 400.0, 0.0, 400.0])] {
            let layer = LayerId(id);
            let mut intents = vec![
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Stage, order, timing: LayerTiming::place(start, None, 10) } },
            ];
            for (name, value) in property::STAGE_MARGINS.iter().zip(margins) {
                intents.push(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value: Value::F64(value) });
            }
            doc.apply_all(intents).unwrap();
        }
        let view = doc.view();
        assert_eq!(view.resolve_stage_extent(at(3)).unwrap().rect(comp), [-100.0, 0.0, comp.width as f32 + 200.0, comp.height as f32]);
        assert_eq!(view.resolve_stage_extent(at(12)).unwrap().rect(comp), [0.0, -400.0, comp.width as f32, comp.height as f32 + 800.0]);
        assert_eq!(view.resolve_stage_extent(at(25)).unwrap(), StageExtent::default());
        assert_eq!(view.resolve_camera(at(3)).unwrap(), crate::doc::core::ResolvedCamera::default(), "a stage layer is not a camera");
    }
}

#[cfg(test)]
mod clipping_contract {
    use crate::doc::store::*;

    fn add(doc: &mut Document, id: u64, order: i16, parent: Option<LayerId>, clipped: bool) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta {
                source: if id == 9 { LayerSource::Group } else { LayerSource::Shape },
                order,
                timing: LayerTiming::place(0, None, 300),
            }},
            Intent::SetAttrs { layer, patch: LayerAttrsPatch {
                parent: Some(parent), clip_to_below: Some(clipped), ..Default::default()
            }},
        ]).unwrap();
        layer
    }

    #[test]
    fn clipping_stack_retargets_on_reorder_without_crossing_parent_boundaries() {
        let mut doc = blank_project();
        let base = add(&mut doc, 1, 0, None, false);
        let first = add(&mut doc, 2, 10, None, true);
        let second = add(&mut doc, 3, 20, None, true);
        let group = add(&mut doc, 9, 30, None, false);
        let child_base = add(&mut doc, 4, 5, Some(group), false);
        let child = add(&mut doc, 5, 15, Some(group), true);
        assert_eq!(doc.view().clipping_base(first).unwrap(), Some(base));
        assert_eq!(doc.view().clipping_base(second).unwrap(), Some(base));
        assert_eq!(doc.view().clipping_base(child).unwrap(), Some(child_base));
        assert_eq!(doc.view().clipping_base(child_base).unwrap(), None);
        assert_eq!(doc.view().clipping_base(LayerId(999)).unwrap(), None);
        let resolved = doc.view().resolve(second, RationalTime::ZERO).unwrap().unwrap();
        assert!(resolved.clip_to_below);
        assert_eq!(resolved.matte, Some(Matte { layer: base, mode: MatteMode::Alpha }));

        doc.apply(Intent::SetOrder { layer: base, order: 25 }).unwrap();
        assert_eq!(doc.view().clipping_base(second).unwrap(), None);
        let orphan = doc.view().resolve(second, RationalTime::ZERO).unwrap().unwrap();
        assert!(orphan.clip_to_below && orphan.matte.is_none());
        assert!(doc.undo());
        assert_eq!(doc.view().clipping_base(second).unwrap(), Some(base));

        let inserted = add(&mut doc, 6, 15, None, false);
        assert_eq!(doc.view().clipping_base(first).unwrap(), Some(base));
        assert_eq!(doc.view().clipping_base(second).unwrap(), Some(inserted));
        doc.apply(Intent::SetAttrs { layer: inserted, patch: LayerAttrsPatch {
            hidden: Some(true), ..Default::default()
        }}).unwrap();
        assert!(doc.view().resolve(inserted, RationalTime::ZERO).unwrap().is_none());
        assert_eq!(doc.view().resolve(second, RationalTime::ZERO).unwrap().unwrap().matte,
            Some(Matte { layer: inserted, mode: MatteMode::Alpha }));
    }

    #[test]
    fn clipping_toggle_preserves_explicit_matte_lock_and_saved_relationship() {
        let mut doc = blank_project();
        let base = add(&mut doc, 1, 0, None, false);
        let layer = add(&mut doc, 2, 10, None, false);
        let explicit = Matte { layer: base, mode: MatteMode::Luma };
        doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch {
            matte: Some(Some(explicit)), ..Default::default()
        }}).unwrap();
        doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch {
            clip_to_below: Some(true), ..Default::default()
        }}).unwrap();
        assert_eq!(doc.view().attrs(layer).unwrap().unwrap().matte, Some(explicit));
        assert_eq!(doc.view().resolve(layer, RationalTime::ZERO).unwrap().unwrap().matte,
            Some(Matte { layer: base, mode: MatteMode::Alpha }));
        assert!(doc.undo());
        let previous = doc.view().resolve(layer, RationalTime::ZERO).unwrap().unwrap();
        assert!(!previous.clip_to_below);
        assert_eq!(previous.matte, Some(explicit));
        assert!(doc.redo());
        doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch {
            locked: Some(true), ..Default::default()
        }}).unwrap();
        let history = doc.history_depth();
        assert!(doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch {
            clip_to_below: Some(false), ..Default::default()
        }}).is_err());
        assert_eq!(doc.history_depth(), history);
        assert!(doc.view().attrs(layer).unwrap().unwrap().clip_to_below);

        let path = std::env::temp_dir().join(format!("motolii-clipping-{}-{}.rrd",
            std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        doc.save(&path).unwrap();
        let loaded = Document::load(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert!(loaded.view().attrs(layer).unwrap().unwrap().clip_to_below);
        assert_eq!(loaded.view().attrs(layer).unwrap().unwrap().matte, Some(explicit));
        assert_eq!(loaded.view().clipping_base(layer).unwrap(), Some(base));
        assert_eq!(loaded.view().resolve(layer, RationalTime::ZERO).unwrap().unwrap().matte,
            Some(Matte { layer: base, mode: MatteMode::Alpha }));
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
