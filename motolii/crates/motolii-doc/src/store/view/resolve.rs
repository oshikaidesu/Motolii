
pub mod camera;
pub mod copies;
pub mod settle;
pub mod effects;
pub mod mask;
pub mod text;
pub mod transform;

use std::collections::{HashMap, HashSet};

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;

use crate::doc::store::{
    property, LayerId, LayerPlacement, PropertyId, ResolvedEffect, ResolvedLayer,
    ResolvedMask, StoreError, TextDocument,
};

use super::StoreView;

pub fn resolve(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
) -> Result<Option<ResolvedLayer>, StoreError> {
    let any_solo = any_solo(view, t)?;
    let world_transforms = crate::doc::store::view::resolve::transform::world_transforms3d(view, t)?;
    let present: HashSet<LayerId> = view.layers().into_iter().collect();
    let mut memo = HashMap::new();
    let mut visiting = HashSet::new();
    resolve_with_solo(view, layer, t, any_solo, &present, &world_transforms, &mut memo, &mut visiting)
}

pub(crate) fn resolve_with_solo(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
    any_solo: bool,
    present: &HashSet<LayerId>,
    world_transforms: &HashMap<LayerId, glam::Affine3A>,
    memo: &mut HashMap<LayerId, glam::Affine2>,
    visiting: &mut HashSet<LayerId>,
) -> Result<Option<ResolvedLayer>, StoreError> {
    let Some(meta) = view.meta(layer)? else {
        return Ok(None);
    };

    let attrs = view.attrs(layer)?.unwrap_or_default();
    let hidden = resolved_hidden(view, layer, t, attrs.hidden)?;
    if hidden {
        return Ok(None);
    }

    let solo = resolved_solo(view, layer, t, attrs.solo)?;
    if any_solo && !solo {
        return Ok(None);
    }

    let Some(composition) = view.composition()? else {
        return Ok(None);
    };
    let comp_frame = t
        .try_to_frame_floor(composition.fps)
        .map_err(|e| StoreError::Property(e.to_string()))?;
    let Some(mut source_frame) = meta.timing.source_frame(comp_frame) else {
        return Ok(None);
    };
    if let Some(speed_track) = view.track(layer, &PropertyId::new(property::SPEED)?)? {
        if let Some(v) =
            meta.timing
                .source_frame_with_speed_track(comp_frame, &speed_track, composition.fps)?
        {
            source_frame = v;
        }
    }
    if let Some(remap) = view.value_at(layer, &PropertyId::new(property::TIME_REMAP)?, t)? {
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
        match view.value_at(layer, &property, t)? {
            Some(Value::F64(v)) => Ok(v as f32),
            Some(other) => Err(StoreError::Property(format!(
                "{name} に数値でない値が入っている: {other:?}"
            ))),
            None => Ok(default),
        }
    };

    let transform = crate::doc::store::view::resolve::transform::world_affine(view, layer, t, present, memo, visiting)?;
    let mut effects = crate::doc::store::view::resolve::effects::resolved_effects(view, layer, t)?;
    let (handed, plate) = crate::doc::store::view::resolve::effects::handed_down(view, layer, t, present)?;
    effects.extend(handed);

    Ok(Some(ResolvedLayer {
        id: layer,
        placement: LayerPlacement {
            transform,
            world_transform: world_transforms.get(&layer).copied(),
            opacity: scalar(property::OPACITY, 1.0)?.clamp(0.0, 1.0),
            order: i32::from(meta.order),
            z: scalar(property::POSITION_Z, 0.0)? + crate::doc::store::layout::frame::laid_out(view, layer, t)?.map_or(0.0, |slot| slot.z),
            rotation_x: scalar(property::ROTATION_X, 0.0)? + crate::doc::store::layout::frame::laid_out(view, layer, t)?.map_or(0.0, |slot| slot.rotation[0]),
            rotation_y: scalar(property::ROTATION_Y, 0.0)? as f32 + crate::doc::store::layout::frame::laid_out(view, layer, t)?.map_or(0.0, |slot| slot.rotation[1]),
            plane: None,
        },
        declared_size: size,
        source: meta.source,
        source_frame,
        source_time: RationalTime::try_from_frame(source_frame, composition.fps)
            .map_err(|e| StoreError::Property(e.to_string()))?,
        masks: crate::doc::store::view::resolve::mask::masks_of(view, layer, t, present, memo, visiting)?,
        effects,
        blend_mode: resolved_blend_mode(view, layer, t, attrs.blend_mode)?,
        matte: if attrs.clip_to_below {
            view.clipping_base(layer)?.map(|base| crate::doc::store::Matte {
                layer: base,
                mode: crate::doc::store::MatteMode::Alpha,
            })
        } else {
            resolved_matte(view, layer, t, attrs.matte)?
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
        shape_stretch: crate::doc::store::layout::frame::laid_out(view, layer, t)?.map_or([1.0, 1.0], |slot| slot.stretch),
        glyph_offsets: crate::doc::store::layout::text::glyph_offsets(view, layer, t)?,
        flow_around: crate::doc::store::layout::text::flow_around(view, layer, t)?,
    }))
}



/// Track Overlay(Detection Method = Layers)が拾う物: 同じ親で自分より下の、描かれる層(Repeater の写しは 1 枚ずつ)と、その画面の上の箱。
/// 並べる Group(Display を持つ箱)は 1 つの箱として拾う(提案 2026-09-15)。線は立てるが寄せない(子と、箱を読むつなぐ線・札が寄せた位置を知らない)。
pub fn overlay_scope(view: &StoreView<'_>, overlay: LayerId, resolved: &[ResolvedLayer], t: RationalTime) -> Result<Vec<(usize, [f32; 4])>, StoreError> {
    let Some(meta) = view.meta(overlay)? else { return Ok(Vec::new()) };
    let parent = view.attrs(overlay)?.unwrap_or_default().parent;
    let mut out = Vec::new();
    for (index, layer) in resolved.iter().enumerate() {
        if layer.id == overlay || layer.ghost || layer.placement.opacity <= 0.0 || layer.blend_mode.is_stencil()
            || matches!(layer.source, crate::doc::store::LayerSource::Camera | crate::doc::store::LayerSource::Null | crate::doc::store::LayerSource::Stage)
            || (layer.source == crate::doc::store::LayerSource::Group && view.display(layer.id, t)? == 0) {
            continue;
        }
        if view.attrs(layer.id)?.unwrap_or_default().parent != parent || !view.meta(layer.id)?.is_some_and(|m| m.order < meta.order) {
            continue;
        }
        let b = match layer.source {
            crate::doc::store::LayerSource::Shape => crate::doc::store::layout::stretched_shape_box(&view.shapes_at(layer.id, t)?, layer.shape_stretch),
            _ => crate::doc::store::layout::boxes::layer_box(view, layer.id, t)?,
        };
        let Some(b) = b else { continue };
        let corners = [[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| layer.placement.transform.transform_point2(glam::Vec2::from(c)));
        let lo = corners.iter().fold(glam::Vec2::MAX, |a, p| a.min(*p));
        let hi = corners.iter().fold(glam::Vec2::MIN, |a, p| a.max(*p));
        out.push((index, [lo.x, lo.y, hi.x, hi.y]));
    }
    Ok(out)
}






/// 直下の子を重ね順で。
pub(crate) fn children_in_order(view: &StoreView<'_>, group: LayerId, present: &HashSet<LayerId>) -> Result<Vec<LayerId>, StoreError> {
    let mut children = Vec::new();
    for id in present {
        if view.attrs(*id)?.unwrap_or_default().parent == Some(group) {
            children.push((view.meta(*id)?.map_or(0, |m| m.order), *id));
        }
    }
    children.sort();
    Ok(children.into_iter().map(|(_, id)| id).collect())
}

/// 層とその子孫(自分が先)。
pub(crate) fn subtree(view: &StoreView<'_>, root: LayerId, present: &HashSet<LayerId>) -> Result<Vec<LayerId>, StoreError> {
    let mut out = vec![root];
    let mut i = 0;
    while i < out.len() {
        let here = out[i];
        i += 1;
        for child in children_in_order(view, here, present)? {
            if !out.contains(&child) {
                out.push(child);
            }
        }
    }
    Ok(out)
}

/// 配置効果を持つグループの子孫。単独では描かず、配置を通してだけ出る。
pub(crate) fn handed_out_by_a_group(view: &StoreView<'_>, present: &HashSet<LayerId>, t: RationalTime) -> Result<HashSet<LayerId>, StoreError> {
    let mut hidden = HashSet::new();
    for id in present {
        if !view.meta(*id)?.is_some_and(|m| m.source == crate::doc::store::LayerSource::Group) { continue; }
        for effect in view.effects(*id)? {
            if view.placement_program(&effect.plugin_id).is_some() && crate::doc::store::view::resolve::effects::effect_enabled(view, *id, effect.id, t)? {
                hidden.extend(subtree(view, *id, present)?.into_iter().skip(1));
                break;
            }
        }
    }
    Ok(hidden)
}

pub(crate) fn any_solo(view: &StoreView<'_>, t: RationalTime) -> Result<bool, StoreError> {
    for layer in view.layers() {
        if view.meta(layer)?.is_some_and(|m| matches!(m.source, crate::doc::store::LayerSource::Camera | crate::doc::store::LayerSource::Stage)) { continue; }
        let static_solo = view.attrs(layer)?.unwrap_or_default().solo;
        if resolved_solo(view, layer, t, static_solo)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn resolved_solo(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
    static_value: bool,
) -> Result<bool, StoreError> {
    match view.value_at(layer, &PropertyId::solo(), t)? {
        Some(Value::Bool(v)) => Ok(v),
        Some(other) => Err(StoreError::Property(format!(
            "solo に真偽でない値が入っている(track が壊れている): {other:?}"
        ))),
        None => Ok(static_value),
    }
}

pub(crate) fn resolved_hidden(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
    static_value: bool,
) -> Result<bool, StoreError> {
    match view.value_at(layer, &PropertyId::hidden(), t)? {
        Some(Value::Bool(v)) => Ok(v),
        Some(other) => Err(StoreError::Property(format!(
            "hidden に真偽でない値が入っている(track が壊れている): {other:?}"
        ))),
        None => Ok(static_value),
    }
}

pub(crate) fn resolved_blend_mode(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
    static_value: crate::doc::store::BlendMode,
) -> Result<crate::doc::store::BlendMode, StoreError> {
    match view.value_at(layer, &PropertyId::blend_mode(), t)? {
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

pub(crate) fn resolved_matte(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
    static_value: Option<crate::doc::store::Matte>,
) -> Result<Option<crate::doc::store::Matte>, StoreError> {
    let Some(mut matte) = static_value else {
        return Ok(None);
    };
    match view.value_at(layer, &PropertyId::matte_mode(), t)? {
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

pub fn resolved_layers(view: &StoreView<'_>, t: RationalTime) -> Result<Vec<ResolvedLayer>, StoreError> {
    let any_solo = any_solo(view, t)?;
    let world_transforms = crate::doc::store::view::resolve::transform::world_transforms3d(view, t)?;
    let layers = view.layers();
    let present: HashSet<LayerId> = layers.iter().copied().collect();
    let mut memo = HashMap::new();
    let mut visiting = HashSet::new();
    let handed_out = handed_out_by_a_group(view, &present, t)?;
    let mut out = Vec::new();
    for layer in layers {
        if handed_out.contains(&layer) {
            continue;
        }
        // ゴーストは元より先に積む(同じ重ね順なら後の物が上に描かれるので、元が手前に来る)。
        crate::doc::store::view::resolve::copies::push_ghosts(view, layer, t, any_solo, &present, &mut out)?;
        if let Some(resolved) =
            resolve_with_solo(view, layer, t, any_solo, &present, &world_transforms, &mut memo, &mut visiting)?
        {
            crate::doc::store::view::resolve::copies::push_copies(view, resolved, t, any_solo, &present, &world_transforms, &mut memo, &mut visiting, &mut out)?;
        }
    }
    crate::doc::store::view::resolve::settle::settle(view, &mut out, t)?;
    Ok(out)
}
