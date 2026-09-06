
mod transform;

use std::collections::{HashMap, HashSet};

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;

use crate::doc::store::{
    property, LayerId, LayerPlacement, PropertyId, ResolvedEffect, ResolvedLayer, ResolvedMask,
    StoreError, TextDocument,
};

use super::StoreView;

impl<'a> StoreView<'a> {
    pub fn resolve_camera(&self, t: RationalTime) -> Result<crate::doc::core::ResolvedCamera, StoreError> {
        let frame = self.composition()?.map(|c| t.try_to_frame_floor(c.fps)).transpose()
            .map_err(|e| StoreError::Property(e.to_string()))?.unwrap_or(0);
        let mut cameras = Vec::new();
        for id in self.layers() {
            if let Some(meta) = self.meta(id)? {
                if meta.source == crate::doc::store::LayerSource::Camera && meta.timing.covers(frame) {
                    let attrs = self.attrs(id)?.unwrap_or_default();
                    if !self.resolved_hidden(id, t, attrs.hidden)? { cameras.push((self.resolved_solo(id, t, attrs.solo)?, meta.order, id)); }
                }
            }
        }
        cameras.sort();
        if let Some((_, _, id)) = cameras.last() {
            let get = |name| self.value_at(*id, &PropertyId::new(name)?, t);
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
        }))
    }

    fn any_solo(&self, t: RationalTime) -> Result<bool, StoreError> {
        for layer in self.layers() {
            if self.meta(layer)?.is_some_and(|m| m.source == crate::doc::store::LayerSource::Camera) { continue; }
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
        let mut out = Vec::new();
        for layer in layers {
            if let Some(resolved) =
                self.resolve_with_solo(layer, t, any_solo, &present, &world_transforms, &mut memo, &mut visiting)?
            {
                out.push(resolved);
            }
        }
        out.sort_by_key(|layer| layer.placement.order);
        Ok(out)
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
