
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

    /// Camera 層 `id` が時刻 `t` に見ている姿勢。層ターゲットが在れば、その層の局所原点の world 点を注視点にする(描画側は bounds の中心で上書きする)。
    pub fn camera_of_layer(&self, id: LayerId, t: RationalTime) -> Result<crate::doc::core::ResolvedCamera, StoreError> {
        let get = |name| self.value_at(id, &PropertyId::new(name)?, t);
        let vec2 = |v: Option<Value>, d: [f32; 2]| match v { Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32], _ => d };
        let f = |v: Option<Value>, d: f32| match v { Some(Value::F64(v)) if v.is_finite() => v as f32, _ => d };
        let mut camera = crate::doc::core::ResolvedCamera {
            center: vec2(get(property::CAMERA_CENTER)?, [0.0, 0.0]),
            target_z: f(get(property::CAMERA_TARGET_Z)?, 0.0),
            orbit_degrees: vec2(get(property::CAMERA_ORBIT)?, [0.0, 0.0]),
            distance_scale: f(get(property::CAMERA_DISTANCE)?, 1.0).max(0.01),
            zoom: f(get(property::CAMERA_ZOOM)?, 1.0),
            roll_degrees: f(get(property::CAMERA_ROLL)?, 0.0),
            near_fade: f(get(property::CAMERA_NEAR_FADE)?, 0.0).max(0.0),
        };
        if let Some(framed) = self.framed_camera(id, t, camera)? {
            return Ok(framed);
        }
        if let Some(target) = self.camera_target_layer(id, t)? {
            if let Some(comp) = self.composition()? {
                let comp = comp.spec();
                let present = self.layers().into_iter().collect();
                if let Some(world) = self.world_transform3d_chain(target, t, &present)?.get(&target) {
                    let point = world.transform_point3(glam::Vec3::ZERO);
                    camera.center = [point.x - comp.width as f32 * 0.5, point.y - comp.height as f32 * 0.5];
                    camera.target_z = point.z;
                }
            }
        }
        Ok(camera)
    }

    /// Framing Size が 0 より大きく Target があれば、箱を画面に収めたカメラ。Camera 層の Transition があれば、少し前の時刻の
    /// 収め方(注視点・奥行き・Distance の対数)を区間の重みで混ぜる(Target を替えると箱から箱へ滑る)。
    fn framed_camera(&self, id: LayerId, t: RationalTime, authored: crate::doc::core::ResolvedCamera) -> Result<Option<crate::doc::core::ResolvedCamera>, StoreError> {
        let framing = match self.value_at(id, &PropertyId::new(property::CAMERA_FRAMING)?, t)? {
            Some(Value::F64(v)) if v > 0.0 => v as f32,
            _ => return Ok(None),
        };
        let Some(now) = self.frame_of(id, t, framing, authored)? else { return Ok(None) };
        let samples = self.transition_samples(id, t)?;
        if samples.is_empty() {
            return Ok(Some(now));
        }
        let (mut center, mut z, mut log_distance, mut total) = (glam::Vec2::ZERO, 0.0f32, 0.0f32, 0.0f32);
        for (at, w) in samples {
            let Some(past) = self.frame_of(id, at, framing, authored)? else { continue };
            center += glam::Vec2::from(past.center) * w;
            z += past.target_z * w;
            log_distance += past.distance_scale.ln() * w;
            total += w;
        }
        if total <= 1e-6 {
            return Ok(Some(now));
        }
        Ok(Some(crate::doc::core::ResolvedCamera { center: (center / total).to_array(), target_z: z / total, distance_scale: (log_distance / total).exp(), ..now }))
    }

    /// その時刻の Target の箱(世界、軸に沿った箱)を画面の `framing` の割合に収めるカメラ。
    fn frame_of(&self, id: LayerId, t: RationalTime, framing: f32, authored: crate::doc::core::ResolvedCamera) -> Result<Option<crate::doc::core::ResolvedCamera>, StoreError> {
        let Some(target) = self.camera_target_layer(id, t)? else { return Ok(None) };
        let Some(comp) = self.composition()? else { return Ok(None) };
        let present = self.layers().into_iter().collect();
        let Some(world) = self.world_transform3d_chain(target, t, &present)?.get(&target).copied() else { return Ok(None) };
        let Some(b) = self.layer_box(target, t)? else { return Ok(None) };
        let corners = [[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| world.transform_point3(glam::vec3(c[0], c[1], 0.0)));
        let lo = corners.iter().fold(glam::Vec3::MAX, |a, p| a.min(*p));
        let hi = corners.iter().fold(glam::Vec3::MIN, |a, p| a.max(*p));
        let (w, h) = ((hi.x - lo.x).max(1.0), (hi.y - lo.y).max(1.0));
        let middle = (lo + hi) * 0.5;
        // 注視点の面で、画面の倍率 = Zoom / Distance。箱が割合 framing に収まる倍率へ Distance を解く。
        let magnify = (framing * comp.width as f32 / w).min(framing * comp.height as f32 / h);
        Ok(Some(crate::doc::core::ResolvedCamera {
            center: [middle.x - comp.width as f32 * 0.5, middle.y - comp.height as f32 * 0.5],
            target_z: middle.z,
            distance_scale: (authored.zoom.max(1e-3) / magnify.max(1e-3)).clamp(0.01, 100.0),
            ..authored
        }))
    }

    /// `camera.target` が指す、いま在る別の層。0・消えた層・自分自身は無し。
    pub fn camera_target_layer(&self, id: LayerId, t: RationalTime) -> Result<Option<LayerId>, StoreError> {
        Ok(match self.value_at(id, &PropertyId::new(property::CAMERA_TARGET)?, t)? {
            Some(Value::LayerId(raw)) if raw != 0 && raw != id.0 && self.layers().contains(&LayerId(raw)) => Some(LayerId(raw)),
            _ => None,
        })
    }

    /// 時刻 `t` に効いている Camera 層。無ければ comp 常在のカメラ track が効いている。
    pub fn active_camera_layer(&self, t: RationalTime) -> Result<Option<LayerId>, StoreError> {
        self.active_guide(crate::doc::store::LayerSource::Camera, t)
    }

    pub fn resolve_camera(&self, t: RationalTime) -> Result<crate::doc::core::ResolvedCamera, StoreError> {
        if let Some(id) = self.active_camera_layer(t)? {
            return self.camera_of_layer(id, t);
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

    /// 解いた文字の書類。横が Fill で並ぶ文字は、並べた幅で折り返す。
    pub fn resolved_text_document(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Option<TextDocument>, StoreError> {
        let Some(mut document) = self.authored_text_document(layer, t)? else { return Ok(None) };
        if let (Some(wrap), Some(comp)) = (self.laid_out(layer, t)?.and_then(|slot| slot.wrap), self.composition()?) {
            document.wrap_size = Some([wrap.max(1.0), comp.height as f32]);
        }
        Ok(Some(document))
    }

    /// 書類に書かれた値だけで解いた文字(並べる前。並べる計算が箱を測る時はこちら)。
    pub(crate) fn authored_text_document(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<Option<TextDocument>, StoreError> {
        let Some(mut document) = self.text_document(layer)? else {
            return Ok(None);
        };
        // Readout: 文字の `#` を関係の値に(`#` が無ければ全部)。1 つの書体の文字として組み直す。
        if let Some(value) = self.readout(layer, t)? {
            let written = document.content.eval(t);
            let content = if written.contains('#') { written.replace('#', &value) } else { value };
            let mut track = crate::doc::store::text::ContentTrack::new();
            track.insert(crate::doc::store::text::ContentKeyframe { t: RationalTime::ZERO, content: content.clone() });
            document.content = track;
            if let Some(style) = document.runs.first().map(|r| r.style) {
                document.runs = vec![crate::doc::store::text::TextRun { len: unicode_segmentation::UnicodeSegmentation::graphemes(content.as_str(), true).count() as u32, style }];
            }
        }

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

        // 文字組みの 3 法(CSS の語)。値は選択肢の番、無ければ CSS の初期値。
        let laws = &mut document.alignment;
        let enum_at = |property: PropertyId| -> Result<Option<i64>, StoreError> {
            match self.value_at(layer, &property, t)? {
                Some(Value::Enum(v)) => Ok(Some(v)),
                Some(other) => Err(StoreError::Property(format!("`{}` に enum でない値が入っている: {other:?}", property.name()))),
                None => Ok(None),
            }
        };
        if let Some(v) = enum_at(PropertyId::text_autospace())? { laws.autospace = crate::doc::store::TextAutospace::from_enum_value(v).unwrap_or_default(); }
        if let Some(v) = enum_at(PropertyId::text_spacing_trim())? { laws.spacing_trim = crate::doc::store::TextSpacingTrim::from_enum_value(v).unwrap_or_default(); }
        if let Some(v) = enum_at(PropertyId::hanging_punctuation())? { laws.hanging = crate::doc::store::HangingPunctuation::from_enum_value(v).unwrap_or_default(); }

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
                frame: crate::doc::store::MaskFrame::Layer,
            });
        }
        Ok(out)
    }

    fn effect_enabled(&self, layer: LayerId, effect: crate::store::EffectId, t: RationalTime) -> Result<bool, StoreError> {
        match self.value_at(layer, &crate::store::PropertyId::effect_enabled(effect), t)? {
            Some(Value::Bool(value)) => Ok(value),
            Some(other) => Err(StoreError::Property(format!("effect {effect} の enabled に真偽でない値が入っている: {other:?}"))),
            None => Ok(true),
        }
    }

    pub fn resolved_effects(
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
            if !self.effect_enabled(layer, effect.id, t)? {
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
            let scope = match self.value_at(layer, &crate::doc::store::PropertyId::effect_scope(effect.id), t)? {
                Some(Value::Enum(v)) => crate::doc::store::EffectScope::from_enum_value(v).ok_or_else(|| {
                    StoreError::Property(format!("effect {} の scope に未知の値が入っている: {v}", effect.id))
                })?,
                Some(other) => {
                    return Err(StoreError::Property(format!(
                        "effect {} の scope に enum でない値が入っている: {other:?}",
                        effect.id
                    )))
                }
                None => crate::doc::store::EffectScope::default(),
            };
            out.push(ResolvedEffect {
                plugin_id: effect.plugin_id,
                params,
                scope,
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
            masks: self.clipped_masks(layer, t, self.resolved_masks(layer, t)?, present, memo, visiting)?,
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

    /// グループが子へ配る効果と、子を 1 枚にしてから掛ける効果。配置効果は数える側なので入らない。
    /// Whole が 1 つ現れた所から下は、もう子が無い(板)ので全部 Whole 扱い。
    fn group_effects(&self, group: LayerId, t: RationalTime) -> Result<(Vec<ResolvedEffect>, Vec<ResolvedEffect>), StoreError> {
        let (mut each, mut whole) = (Vec::new(), Vec::new());
        for effect in self.resolved_effects(group, t)? {
            if self.placement_program(&effect.plugin_id).is_some() {
                continue;
            }
            if !whole.is_empty() || effect.scope == crate::doc::store::EffectScope::Whole {
                whole.push(effect);
            } else {
                each.push(effect);
            }
        }
        Ok((each, whole))
    }

    /// 親のグループから降りてくる効果。効果は 1 枚に掛かるとしか書かれていないので、子/全体は
    /// ここで解く(裁定 2026-09-11)。返すのは (自分に足す効果, 板になる最寄りのグループとその効果)。
    /// 板のグループへ祖先から配られた効果は、板の絵に掛かる。
    fn handed_down(&self, layer: LayerId, t: RationalTime, present: &HashSet<LayerId>) -> Result<(Vec<ResolvedEffect>, Option<(LayerId, Vec<ResolvedEffect>)>), StoreError> {
        let mut each = Vec::new();
        let mut seen = HashSet::from([layer]);
        let mut next = self.attrs(layer)?.unwrap_or_default().parent.filter(|p| present.contains(p));
        while let Some(group) = next {
            if !seen.insert(group) || !self.meta(group)?.is_some_and(|m| m.source == crate::doc::store::LayerSource::Group) {
                break;
            }
            let (own_each, own_whole) = self.group_effects(group, t)?;
            each.extend(own_each);
            if !own_whole.is_empty() {
                let (above, _) = self.handed_down(group, t, present)?;
                let mut plate = own_whole;
                plate.extend(above);
                return Ok((each, Some((group, plate))));
            }
            next = self.attrs(group)?.unwrap_or_default().parent.filter(|p| present.contains(p));
        }
        Ok((each, None))
    }

    /// 配置効果を持つ層を、その配置の数だけ増やす。配置効果より上の効果は各配置の素材に、
    /// 下の効果は `after_effects` として全体に残す。時刻のずれた配置は、その時刻の姿を取り直す。
    /// 配置効果を持つ層を、その配置の数だけ増やす。配置効果より上の効果は各配置の素材に、
    /// 下の効果は `after_effects` として全体に残す。時刻のずれた配置は、その時刻の姿を取り直す。
    /// グループなら子が素材の袋で、配置ごとに 1 つ引いた子の部分木を置く(裁定 2026-09-07)。
    #[allow(clippy::too_many_arguments)]
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

    /// 見つけた格子へ寄せる(2026-09-15 利用者「ものは動かします。グリッドが動的に動くと Tracery のような効果になる」):
    /// Track Overlay(Layers、Snap Strength > 0)が、拾った物の箱の辺から立てた格子の線(`overlay::edge_lines`)へ、物を画面の上で寄せる。
    /// 線は寄せる前の箱から立てる(寄せた結果を読み直さない)。
    fn snap_to_found_grids(&self, out: &mut [ResolvedLayer], t: RationalTime) -> Result<(), StoreError> {
        use crate::doc::extensions::overlay;
        let overlays: Vec<(LayerId, f32, f32)> = out.iter().filter(|l| !l.ghost && l.copy == 0).filter_map(|l| {
            let effect = l.effects.iter().find(|e| overlay::is_track_overlay(&e.plugin_id))?;
            let params = overlay::with_defaults(&effect.plugin_id, &effect.params);
            let strength = overlay::number_of(&params, "grid_snap").clamp(0.0, 1.0) as f32;
            (overlay::number_of(&params, "method").round() as i64 == 2 && strength > 0.0)
                .then(|| (l.id, strength, overlay::number_of(&params, "grid_merge").max(0.0) as f32))
        }).collect();
        for (overlay_layer, strength, merge) in overlays {
            let scope = self.overlay_scope(overlay_layer, out, t)?;
            if scope.is_empty() {
                continue;
            }
            let xs: Vec<f32> = scope.iter().flat_map(|(_, b)| [b[0], b[2]]).collect();
            let ys: Vec<f32> = scope.iter().flat_map(|(_, b)| [b[1], b[3]]).collect();
            let (lx, ly) = (overlay::edge_lines(&xs, merge), overlay::edge_lines(&ys, merge));
            for (k, &(index, b)) in scope.iter().enumerate() {
                if out[index].source == crate::doc::store::LayerSource::Group {
                    continue;
                }
                let dx = ((lx[2 * k].0 - b[0]) + (lx[2 * k + 1].0 - b[2])) * 0.5 * strength;
                let dy = ((ly[2 * k].0 - b[1]) + (ly[2 * k + 1].0 - b[3])) * 0.5 * strength;
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
        let (tune, mask) = motion::settings(&base.effects[at_index].params);
        let below = base.effects.split_off(at_index + 1);
        base.effects.pop();
        base.after_effects.splice(0..0, below);
        let frame_seconds = self.composition()?.map_or(0.0, |c| c.fps.den() as f64 / c.fps.num() as f64);
        if tune <= 0.0 || frame_seconds <= 0.0 || !mask.contains(&true) || base.source == crate::doc::store::LayerSource::Group {
            out.push(base);
            return Ok(());
        }
        let layer = base.id;
        let parent = self.attrs(layer)?.unwrap_or_default().parent.filter(|p| present.contains(p));
        let parent2 = parent.map(|p| self.world_affine(p, t, present, memo, visiting)).transpose()?.unwrap_or(glam::Affine2::IDENTITY);
        let parent3 = parent.and_then(|p| world_transforms.get(&p).copied()).unwrap_or(glam::Affine3A::IDENTITY);
        let now_inverse = self.local_placement_transform(layer, t)?.inverse();
        let local_delta = |at: RationalTime| -> Result<glam::Affine2, StoreError> {
            Ok(self.local_placement_transform_sampled(layer, t, Some((at, mask)))? * now_inverse)
        };
        // 枚数は 1 コマの両端で、層の四隅がどれだけ動くかから。
        let edges = [-0.5, 0.5].map(|s| {
            RationalTime::try_new((s * tune * frame_seconds * 1_000_000.0).round() as i64, 1_000_000).ok().and_then(|o| t.try_add(o).ok())
        });
        let [Some(early), Some(late)] = edges else {
            out.push(base);
            return Ok(());
        };
        let (early, late) = (parent2 * local_delta(early)? * parent2.inverse(), parent2 * local_delta(late)? * parent2.inverse());
        // 形・文字は宣言の大きさを持たない(描くまで分からない)。その時は層の原点のまわり 200 px 四方で数える
        // (1 点に潰れると回転の道のりが 0 になり、写しが足りずに段が出る)。
        let [w, h] = base.declared_size;
        let (lo, hi) = if w > 0.0 && h > 0.0 { ([0.0, 0.0], [w, h]) } else { ([-100.0, -100.0], [100.0, 100.0]) };
        let travel = [[lo[0], lo[1]], [hi[0], lo[1]], [lo[0], hi[1]], [hi[0], hi[1]]]
            .map(|corner| {
                let p = base.placement.transform.transform_point2(glam::Vec2::from(corner));
                early.transform_point2(p).distance(late.transform_point2(p))
            })
            .into_iter()
            .fold(0.0f32, f32::max);
        let count = motion::sample_count(travel);
        if count <= 1 {
            out.push(base);
            return Ok(());
        }
        for (k, at) in motion::sample_times(t, frame_seconds, tune, count).into_iter().enumerate() {
            let local = local_delta(at)?;
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
        self.put_backgrounds_behind(&mut out, t)?;
        self.put_on_planes(&mut out, t)?;
        self.put_connectors_in_front(&mut out, t)?;
        self.hand_out_stencils(&mut out)?;
        self.snap_to_grids(&mut out, t)?;
        self.snap_to_found_grids(&mut out, t)?;
        self.apply_fields(&mut out, t)?;
        out.sort_by_key(|layer| (layer.placement.order, layer.source != crate::doc::store::LayerSource::Group));
        // 描き順は並べた順の番号(同じ order の写し同士を描く側の同点に委ねると、描き順が揺れる)。
        for (rank, layer) in out.iter_mut().enumerate() {
            layer.placement.order = i32::try_from(rank).unwrap_or(i32::MAX);
        }
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

    /// Overflow が Clip の並べる Group の子孫は、その箱で切る: 箱を層の素材座標へ写した角丸の矩形を Intersect で足す。
    /// 祖先の箱の切りは箱の枠に付く(`MaskFrame::Box`)、自分の inset は自分の枠(`Layer`)。
    fn clipped_masks(
        &self,
        layer: LayerId,
        t: RationalTime,
        mut masks: Vec<ResolvedMask>,
        present: &HashSet<LayerId>,
        memo: &mut HashMap<LayerId, glam::Affine2>,
        visiting: &mut HashSet<LayerId>,
    ) -> Result<Vec<ResolvedMask>, StoreError> {
        // 奥行きを持つ網・点群は 2D の mask で切れない(切り口は面の法の宿題)。
        if let Some(crate::doc::store::LayerMeta { source: crate::doc::store::LayerSource::File { path, .. }, .. }) = self.meta(layer)? {
            if self.analysis().and_then(|a| a.extent(&path)).is_some_and(|e| e[2] > 0.0) {
                return Ok(masks);
            }
        }
        let mut seen = HashSet::new();
        // inset は自分の背景も切る(CSS の clip-path は要素ごと)。Overflow は箱の外の子孫だけ。
        let mut next = Some(layer);
        let mut own: Option<glam::Affine2> = None;
        while let Some(group) = next.filter(|g| seen.insert(*g) && present.contains(g)) {
            let cuts = if group == layer { [None, self.clip_inset(group, t)?] } else { [self.clip_box(group, t)?, self.clip_inset(group, t)?] };
            for (b, radius) in cuts.into_iter().flatten() {
                let world = match own {
                    Some(w) => w,
                    None => *own.insert(self.world_affine(layer, t, present, memo, visiting)?),
                };
                let to = world.inverse() * self.world_affine(group, t, present, memo, visiting)?;
                masks.push(ResolvedMask {
                    mode: crate::doc::store::MaskMode::Intersect,
                    inverted: false,
                    opacity: 1.0,
                    expansion: 0.0,
                    shape: crate::doc::store::layout::rounded_rect_path(b, radius, to),
                    frame: if group == layer { crate::doc::store::MaskFrame::Layer } else { crate::doc::store::MaskFrame::Box },
                });
            }
            next = self.attrs(group)?.unwrap_or_default().parent;
        }
        Ok(masks)
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
mod camera_target_contract {
    use crate::doc::core::{camera_projection, ResolvedCamera};
    use crate::doc::store::*;

    fn put(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
        doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    }
    fn add(doc: &mut Document, id: u64, source: LayerSource) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all(vec![
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source, order: id as i16, timing: LayerTiming::place(0, None, 100) } },
        ]).unwrap();
        layer
    }
    /// world 点が出力の枠中央に来るか(注視点は常に中央)。
    fn centred(comp: crate::doc::core::CompSpec, camera: ResolvedCamera, point: glam::Vec3) -> bool {
        let projection = camera_projection(comp, camera);
        let clip = projection.projection_matrix() * projection.view_matrix() * point.extend(1.0);
        clip.w > 0.0 && (clip.x / clip.w).abs() < 1e-3 && (clip.y / clip.w).abs() < 1e-3
    }

    /// Framing Size: Target の箱の中心を注視点にし、箱が画面のその割合に収まる距離へ(Cinemachine の Group Framing Size)。
    #[test]
    fn framing_size_fits_the_target_box_on_screen_and_follows_it() {
        let mut doc = blank_project();
        let comp = doc.view().composition().unwrap().unwrap().spec();
        let card = add(&mut doc, 2, LayerSource::Shape);
        doc.apply(Intent::SetShapes { layer: card, shapes: vec![rect_shape([255; 4], [200.0, 100.0])] }).unwrap();
        put(&mut doc, card, property::POSITION, Value::Vec2([300.0, 200.0]));
        let camera = add(&mut doc, 1, LayerSource::Camera);
        put(&mut doc, camera, property::CAMERA_TARGET, Value::LayerId(card.0));
        put(&mut doc, camera, property::CAMERA_FRAMING, Value::F64(0.5));
        let on_screen = |doc: &Document| {
            let view = doc.view();
            let resolved = view.resolve_camera(RationalTime::ZERO).unwrap();
            let projection = camera_projection(comp, resolved);
            let matrix = projection.projection_matrix() * projection.view_matrix();
            let world = view.world_transform3d(card, RationalTime::ZERO).unwrap();
            let b = view.layer_box(card, RationalTime::ZERO).unwrap().unwrap();
            let ndc: Vec<glam::Vec2> = [[b[0], b[1]], [b[2], b[3]]].iter().map(|c| {
                let clip = matrix * world.transform_point3(glam::vec3(c[0], c[1], 0.0)).extend(1.0);
                glam::vec2(clip.x / clip.w, clip.y / clip.w)
            }).collect();
            (((ndc[1].x - ndc[0].x).abs() * 0.5), ((ndc[1].y - ndc[0].y).abs() * 0.5), (ndc[0] + ndc[1]) * 0.5)
        };
        let (w, h, middle) = on_screen(&doc);
        assert!(middle.length() < 1e-3, "the box's centre is the centre of the frame: {middle:?}");
        assert!(((w.max(h)) - 0.5).abs() < 0.01 && w.max(h) >= w.min(h), "the tighter side takes half the frame: {w} {h}");
        put(&mut doc, card, property::POSITION, Value::Vec2([900.0, 700.0]));
        put(&mut doc, card, property::SCALE, Value::Vec2([2.0, 2.0]));
        let (w2, h2, middle2) = on_screen(&doc);
        assert!(middle2.length() < 1e-3 && ((w2.max(h2)) - 0.5).abs() < 0.01, "moving and growing the box keeps it framed: {w2} {h2} {middle2:?}");
    }

    /// AE の Point of Interest を rerun の球面座標で持つ: 注視点・軌道・距離が Camera 層から解決へ流れ、eye は導出。
    #[test]
    fn orbit_distance_and_target_z_flow_from_the_camera_layer() {
        let mut doc = blank_project();
        let comp = doc.view().composition().unwrap().unwrap().spec();
        let camera = add(&mut doc, 1, LayerSource::Camera);
        put(&mut doc, camera, property::CAMERA_CENTER, Value::Vec2([120.0, -40.0]));
        put(&mut doc, camera, property::CAMERA_TARGET_Z, Value::F64(300.0));
        put(&mut doc, camera, property::CAMERA_ORBIT, Value::Vec2([-20.0, 35.0]));
        put(&mut doc, camera, property::CAMERA_DISTANCE, Value::F64(2.0));
        let resolved = doc.view().resolve_camera(RationalTime::ZERO).unwrap();
        assert_eq!(resolved, ResolvedCamera { center: [120.0, -40.0], target_z: 300.0, orbit_degrees: [-20.0, 35.0], distance_scale: 2.0, ..Default::default() });
        let target = resolved.target(comp);
        assert!(centred(comp, resolved, target), "the point of interest sits under the frame centre");
        let eye = camera_projection(comp, resolved).eye;
        let front = camera_projection(comp, ResolvedCamera { orbit_degrees: [0.0; 2], ..resolved }).eye;
        assert!((eye.distance(target) - front.distance(target)).abs() < 0.01, "orbit keeps the distance");
        assert!(eye.distance(front) > 1.0, "orbit moves the eye");
    }

    /// 層ターゲット: null を動かせば注視点が追う。無い層・0・自分自身は無視して center に戻る。
    #[test]
    fn a_target_layer_moves_the_point_of_interest_with_its_position() {
        let mut doc = blank_project();
        let comp = doc.view().composition().unwrap().unwrap().spec();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |frame| RationalTime::try_from_frame(frame, fps).unwrap();
        let camera = add(&mut doc, 1, LayerSource::Camera);
        let null = add(&mut doc, 2, LayerSource::Null);
        put(&mut doc, camera, property::CAMERA_CENTER, Value::Vec2([500.0, 500.0]));
        put(&mut doc, camera, property::CAMERA_TARGET, Value::LayerId(2));
        put(&mut doc, null, property::POSITION_Z, Value::F64(250.0));
        let mut track = KeyframeTrack::new();
        for (frame, xy) in [(0, [100.0, 200.0]), (10, [300.0, 400.0])] {
            track.insert(Keyframe { t: at(frame), value: Value::Vec2(xy), interp: Interp::Linear, spatial: None });
        }
        doc.apply(Intent::SetTrack { layer: null, property: PropertyId::new(property::POSITION).unwrap(), track }).unwrap();
        for (frame, expect) in [(0, glam::vec3(100.0, 200.0, 250.0)), (5, glam::vec3(200.0, 300.0, 250.0)), (10, glam::vec3(300.0, 400.0, 250.0))] {
            let resolved = doc.view().resolve_camera(at(frame)).unwrap();
            assert!(resolved.target(comp).distance(expect) < 1e-3, "frame {frame}: {:?} != {expect:?}", resolved.target(comp));
            assert!(centred(comp, resolved, expect));
        }
        // parent を挟んでも world の位置を見る
        let parent = add(&mut doc, 3, LayerSource::Null);
        put(&mut doc, parent, property::POSITION, Value::Vec2([1000.0, 0.0]));
        doc.apply(Intent::SetAttrs { layer: null, patch: LayerAttrsPatch { parent: Some(Some(parent)), ..Default::default() } }).unwrap();
        assert!(doc.view().resolve_camera(at(0)).unwrap().target(comp).distance(glam::vec3(1100.0, 200.0, 250.0)) < 1e-3);
        for dead in [Value::LayerId(0), Value::LayerId(1), Value::LayerId(99)] {
            put(&mut doc, camera, property::CAMERA_TARGET, dead.clone());
            assert_eq!(doc.view().resolve_camera(at(0)).unwrap().center, [500.0, 500.0], "{dead:?} falls back to center");
        }
        put(&mut doc, camera, property::CAMERA_TARGET, Value::LayerId(2));
        doc.apply(Intent::RemoveLayer(null)).unwrap();
        assert_eq!(doc.view().resolve_camera(at(0)).unwrap().center, [500.0, 500.0], "a removed target falls back to center");
    }
}

#[cfg(test)]
mod group_scope_contract {
    use crate::doc::store::*;

    fn add(doc: &mut Document, id: u64, order: i16, source: LayerSource, parent: Option<LayerId>) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source, order, timing: LayerTiming::place(0, None, 300) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(parent), ..Default::default() } },
        ]).unwrap();
        layer
    }

    fn effect(doc: &mut Document, layer: LayerId, id: u32, plugin: &str, whole: bool) {
        let mut effects = doc.view().effects(layer).unwrap();
        effects.push(EffectInstance { id: EffectId(id), plugin_id: plugin.to_owned() });
        doc.apply(Intent::SetEffects { layer, effects }).unwrap();
        if whole {
            doc.apply(Intent::SetConstant { layer, property: PropertyId::effect_scope(EffectId(id)), value: Value::Enum(EffectScope::Whole.enum_value()) }).unwrap();
        }
    }

    fn find(out: &[ResolvedLayer], id: LayerId) -> &ResolvedLayer {
        out.iter().find(|l| l.id == id).expect("layer resolved")
    }

    /// 効果は 1 枚に掛かるとしか書かれていない。Each は子それぞれの効果列の末尾に降り、
    /// Whole は最寄りの板にまとまり、板の上の祖先から配られた効果も板に掛かる。
    #[test]
    fn a_groups_effects_go_to_each_child_or_to_one_plate() {
        let mut doc = blank_project();
        let outer = add(&mut doc, 1, 0, LayerSource::Group, None);
        let inner = add(&mut doc, 2, 1, LayerSource::Group, Some(outer));
        let leaf = add(&mut doc, 3, 2, LayerSource::Shape, Some(inner));
        let sibling = add(&mut doc, 4, 3, LayerSource::Shape, Some(outer));
        effect(&mut doc, leaf, 0, "leaf.own", false);
        effect(&mut doc, inner, 0, "inner.each", false);
        effect(&mut doc, outer, 0, "outer.each", false);
        let out = doc.view().resolved_layers(RationalTime::ZERO).unwrap();
        let plugins = |l: &ResolvedLayer| l.effects.iter().map(|e| e.plugin_id.clone()).collect::<Vec<_>>();
        assert_eq!(plugins(find(&out, leaf)), ["leaf.own", "inner.each", "outer.each"], "own first, then nearest group, then the one above");
        assert_eq!(plugins(find(&out, sibling)), ["outer.each"]);
        assert!(out.iter().all(|l| l.plate.is_none() && l.after_effects.is_empty()), "no Whole, no plate");

        // inner に Whole を積む: leaf は inner の板の一部。板には inner の Whole と、outer から inner へ配られた効果が掛かる。
        effect(&mut doc, inner, 1, "inner.whole", true);
        effect(&mut doc, inner, 2, "inner.after", false);
        let out = doc.view().resolved_layers(RationalTime::ZERO).unwrap();
        let leaf_r = find(&out, leaf);
        assert_eq!(plugins(leaf_r), ["leaf.own", "inner.each"], "the outer group's Each now lands on the plate, not the leaf");
        assert_eq!(leaf_r.plate, Some(inner));
        assert_eq!(leaf_r.after_effects.iter().map(|e| e.plugin_id.as_str()).collect::<Vec<_>>(), ["inner.whole", "inner.after", "outer.each"], "below a Whole everything is Whole");
        assert!(find(&out, sibling).plate.is_none());

        // 単層の scope は無意味: 値を書いても板にはならない。
        doc.apply(Intent::SetConstant { layer: sibling, property: PropertyId::effect_scope(EffectId(0)), value: Value::Enum(1) }).unwrap();
        let out = doc.view().resolved_layers(RationalTime::ZERO).unwrap();
        assert!(find(&out, sibling).plate.is_none());
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

    /// Repeater が Group の子を 1 つずつ引く時、写しは番号の順に重なる(子ごとにまとまらない)。Cavalry の Concentrick で踏んだ。
    #[test]
    fn picked_copies_stack_by_their_number_not_by_child() {
        let mut doc = blank_project();
        let group = add(&mut doc, 9, 5, None, false);
        let ink = add(&mut doc, 2, 1, Some(group), false);
        let paper = add(&mut doc, 3, 2, Some(group), false);
        doc.apply(Intent::SetEffects { layer: group, effects: vec![EffectInstance { id: EffectId(1), plugin_id: crate::doc::extensions::placement::REPEAT.to_owned() }] }).unwrap();
        for (name, value) in [("count", 4.0), ("pick", 1.0)] {
            doc.apply(Intent::SetConstant { layer: group, property: PropertyId::effect_param(EffectId(1), name).unwrap(), value: Value::F64(value) }).unwrap();
        }
        let resolved = doc.view().resolved_layers(RationalTime::ZERO).unwrap();
        let drawn: Vec<(LayerId, u32)> = resolved.iter().filter(|l| l.id != group).map(|l| (l.id, l.copy)).collect();
        assert_eq!(drawn, vec![(ink, 0), (paper, 1), (ink, 2), (paper, 3)], "ink, paper, ink, paper from the bottom up");
        let orders: Vec<i32> = resolved.iter().map(|l| l.placement.order).collect();
        assert!(orders.windows(2).all(|w| w[0] < w[1]), "and each is drawn at its own step, never a tie: {orders:?}");
    }

    /// Stencil はクリッピングマスクの逆: 自分の形で下を切る。範囲は clip していれば束、していなければ同じ Group の下だけ。
    #[test]
    fn a_stencil_cuts_its_group_below_it_or_only_its_clipping_stack() {
        let mut doc = blank_project();
        let background = add(&mut doc, 1, 0, None, false);
        let group = add(&mut doc, 9, 5, None, false);
        let low = add(&mut doc, 4, 1, Some(group), false);
        let photo = add(&mut doc, 5, 3, Some(group), false);
        let stencil = add(&mut doc, 6, 4, Some(group), false);
        let above = add(&mut doc, 7, 8, Some(group), false);
        doc.apply(Intent::SetAttrs { layer: stencil, patch: LayerAttrsPatch { blend_mode: Some(BlendMode::StencilAlpha), ..Default::default() } }).unwrap();
        let matte_of = |doc: &Document, id: LayerId| doc.view().resolved_layers(RationalTime::ZERO).unwrap().into_iter().find(|l| l.id == id).and_then(|l| l.matte);
        let cut = Some(Matte { layer: stencil, mode: MatteMode::Alpha });
        assert_eq!((matte_of(&doc, low), matte_of(&doc, photo)), (cut, cut), "without clip: everything below it in the group");
        assert_eq!((matte_of(&doc, above), matte_of(&doc, background)), (None, None), "not what is above, not outside the group");

        doc.apply(Intent::SetAttrs { layer: stencil, patch: LayerAttrsPatch { clip_to_below: Some(true), ..Default::default() } }).unwrap();
        assert_eq!(matte_of(&doc, photo), cut, "clipped: only its clipping base");
        assert_eq!(matte_of(&doc, low), None, "the rest of the group is left alone");
        assert_eq!(matte_of(&doc, stencil), None, "the stencil does not clip itself to the base it cuts");

        doc.apply(Intent::SetAttrs { layer: stencil, patch: LayerAttrsPatch { blend_mode: Some(BlendMode::SilhouetteAlpha), ..Default::default() } }).unwrap();
        assert_eq!(matte_of(&doc, photo), Some(Matte { layer: stencil, mode: MatteMode::InvertedAlpha }), "Silhouette punches a hole");
    }

    #[test]
    fn clipping_cache_preserves_equal_order_and_preview_isolation() {
        let mut doc = blank_project();
        let base = add(&mut doc, 1, 0, None, false);
        let tied = add(&mut doc, 2, 0, None, false);
        let top = add(&mut doc, 3, 10, None, true);
        assert_eq!(doc.view().clipping_base(tied).unwrap(), None);
        assert_eq!(doc.view().clipping_base(top).unwrap(), Some(tied));
        let owner = doc.begin_preview();
        doc.preview_edits(owner, &[Intent::SetAttrs { layer: tied, patch: LayerAttrsPatch { clip_to_below: Some(true), ..Default::default() } }]).unwrap();
        assert_eq!(doc.view().clipping_base(top).unwrap(), Some(base));
        assert_eq!(doc.view().without_transients().clipping_base(top).unwrap(), Some(tied));
        doc.clear_preview_edits(owner);
        assert_eq!(doc.view().clipping_base(top).unwrap(), Some(tied));
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
