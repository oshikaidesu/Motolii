
mod transform;

use std::collections::{HashMap, HashSet};

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;

use crate::doc::store::{
    property, LayerId, LayerPlacement, PropertyId, ResolvedEffect, ResolvedLayer, ResolvedMask,
    StoreError, TextDocument,
};

#[cfg(test)]
use crate::doc::store::Document;
use super::StoreView;
#[cfg(test)]
use transform::{reset_world_affine_compute_count, world_affine_compute_count};

impl<'a> StoreView<'a> {
    pub fn resolve_camera(&self, t: RationalTime) -> Result<crate::doc::core::ResolvedCamera, StoreError> {
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
            roll_degrees,
        })
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
        let present: HashSet<LayerId> = self.layers().into_iter().collect();
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        self.resolve_with_solo(layer, t, any_solo, &present, &mut memo, &mut visiting)
    }

    fn resolve_with_solo(
        &self,
        layer: LayerId,
        t: RationalTime,
        any_solo: bool,
        present: &HashSet<LayerId>,
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
            matte: self.resolved_matte(layer, t, attrs.matte)?,
            pinned: attrs.pinned,
            flatten: attrs.flatten,
        }))
    }

    fn any_solo(&self, t: RationalTime) -> Result<bool, StoreError> {
        for layer in self.layers() {
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
        let layers = self.layers();
        let present: HashSet<LayerId> = layers.iter().copied().collect();
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        let mut out = Vec::new();
        for layer in layers {
            if let Some(resolved) =
                self.resolve_with_solo(layer, t, any_solo, &present, &mut memo, &mut visiting)?
            {
                out.push(resolved);
            }
        }
        out.sort_by_key(|layer| layer.placement.order);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{Composition, Fps, Intent, LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming};

    fn t(frame: i64) -> RationalTime {
        RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
    }

    fn place(doc: &mut Document, layer: LayerId, parent: Option<LayerId>) {
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Null,
                    order: layer.0 as i16,
                    timing: LayerTiming::place(0, None, 300),
                },
            },
        ])
        .unwrap();
        if let Some(parent) = parent {
            doc.apply(Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch {
                    parent: Some(Some(parent)),
                    ..Default::default()
                },
            })
            .unwrap();
        }
    }

    #[test]
    fn shared_ancestor_is_resolved_exactly_once_across_siblings() {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: Composition::default_background(),
        }))
        .unwrap();

        let (a, b, c1, c2) = (LayerId(1), LayerId(2), LayerId(3), LayerId(4));
        place(&mut doc, a, None);
        place(&mut doc, b, Some(a));
        place(&mut doc, c1, Some(b));
        place(&mut doc, c2, Some(b));

        let view = doc.view();
        let present: HashSet<LayerId> = view.layers().into_iter().collect();
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        reset_world_affine_compute_count();

        view.world_affine(c1, t(0), &present, &mut memo, &mut visiting)
            .unwrap();
        assert_eq!(
            world_affine_compute_count(),
            3,
            "root/mid/leaf1 の3層でちょうど3回のはず(まだ誰も共有していない)"
        );

        view.world_affine(c2, t(0), &present, &mut memo, &mut visiting)
            .unwrap();
        assert_eq!(
            world_affine_compute_count(),
            4,
            "B(と A)が2人目の子 C2 のために再計算されてしまっている(メモ化が効いていない)"
        );
    }

    #[test]
    fn position_z_track_resolves_into_placement_z() {
        use crate::doc::store::{Interp, Keyframe, KeyframeTrack};

        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: Composition::default_background(),
        }))
        .unwrap();

        let (with_z, without_z) = (LayerId(1), LayerId(2));
        place(&mut doc, with_z, None);
        place(&mut doc, without_z, None);

        let mut z_track = KeyframeTrack::new();
        z_track.insert(Keyframe {
            t: t(0),
            value: Value::F64(42.0),
            interp: Interp::Hold,
            spatial: None,
        });
        let mut xy_track = KeyframeTrack::new();
        xy_track.insert(Keyframe {
            t: t(0),
            value: Value::Vec2([10.0, 20.0]),
            interp: Interp::Hold,
            spatial: None,
        });
        doc.apply_all([
            Intent::SetTrack {
                layer: with_z,
                property: PropertyId::new(property::POSITION_Z).unwrap(),
                track: z_track,
            },
            Intent::SetTrack {
                layer: with_z,
                property: PropertyId::new(property::POSITION).unwrap(),
                track: xy_track,
            },
        ])
        .unwrap();

        let view = doc.view();
        let layers = view.resolved_layers(t(0)).unwrap();
        let placement_of = |id: LayerId| {
            layers
                .iter()
                .find(|l| l.id == id)
                .unwrap()
                .placement
        };

        assert_eq!(placement_of(with_z).z, 42.0, "position.z を書いた層に出ていない");
        assert_eq!(
            placement_of(without_z).z,
            0.0,
            "書いていない層は LayerPlacement::default() どおり 0.0 のはず"
        );
        let (tx, ty) = placement_of(with_z).transform.translation.into();
        assert_eq!((tx, ty), (10.0, 20.0), "z を足しても x/y の解決が変わってはいけない");
    }
}
