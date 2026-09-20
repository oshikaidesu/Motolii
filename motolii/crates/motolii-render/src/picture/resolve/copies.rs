//! 写しを積む手 — 1 枚の層が、絵の上では何枚にもなる所。
//! 配置効果の数だけ、Motion Blur のサンプルの数だけ、Split の単位の数だけ、Ghost の数だけ。
//! どれも「元をずらした時刻・切り方で解き直して積む」で、書類に子の層は作らない。

use super::*;

/// 解けた 1 枚を、写しに展開して積む。Split の単位になるならそちらだけ、
/// ならなければ配置効果の数だけ(その中で Motion Blur の写しも)。
#[allow(clippy::too_many_arguments)]
pub fn push_copies(
    view: &StoreView<'_>,
    resolved: ResolvedLayer,
    t: RationalTime,
    any_solo: bool,
    present: &HashSet<LayerId>,
    world_transforms: &HashMap<LayerId, glam::Affine3A>,
    memo: &mut HashMap<LayerId, glam::Affine2>,
    visiting: &mut HashSet<LayerId>,
    out: &mut Vec<ResolvedLayer>,
) -> Result<(), StoreError> {
    if push_split(view, &resolved, t, any_solo, present, out)? {
        return Ok(());
    }
    push_placements(view, resolved, t, any_solo, present, world_transforms, memo, visiting, out)
}

/// 配置効果を持つ層を、その配置の数だけ増やす。配置効果より上の効果は各配置の素材に、
/// 下の効果は `after_effects` として全体に残す。時刻のずれた配置は、その時刻の姿を取り直す。
/// グループなら子が素材の袋で、配置ごとに 1 つ引いた子の部分木を置く(裁定 2026-09-07)。
#[allow(clippy::too_many_arguments)]
pub fn push_placements(
    view: &StoreView<'_>,
    base: ResolvedLayer,
    t: RationalTime,
    any_solo: bool,
    present: &HashSet<LayerId>,
    world_transforms: &HashMap<LayerId, glam::Affine3A>,
    memo: &mut HashMap<LayerId, glam::Affine2>,
    visiting: &mut HashSet<LayerId>,
    out: &mut Vec<ResolvedLayer>,
) -> Result<(), StoreError> {
    let blur = base.effects.iter().position(|e| view.is_sampling_effect(&e.plugin_id));
    let places = |e: &ResolvedEffect| view.placement_program(&e.plugin_id).is_some();
    let Some(first) = base.effects.iter().position(places) else {
        return match blur {
            Some(at) => push_motion_blur(view, base, at, t, present, world_transforms, memo, visiting, out),
            None => { out.push(base); Ok(()) }
        };
    };
    let params = &base.effects[first].params;
    let layer = base.id;
    // 形の素材は輪郭を伸ばし(線は太らない)、それ以外は置き場所で伸ばす。
    let stretch_outline = base.source == crate::doc::store::LayerSource::Shape;
    let program = view.placement_program(&base.effects[first].plugin_id).expect("matched placement program");
    let placements = (program.evaluate)(&crate::doc::store::kind::PlacementInput {
        params, layer, time: t, stretch_outline, analysis: view.analysis(),
        position: if program.needs_position { crate::picture::resolve::transform::resolve_position(view, layer, t)? } else { [0.0; 2] },
    });
    let parent = view.attrs(layer)?.unwrap_or_default().parent.filter(|p| present.contains(p));
    let is_group = base.source == crate::doc::store::LayerSource::Group;
    let children = if is_group { crate::picture::resolve::children_in_order(view, layer, present)? } else { Vec::new() };
    if is_group && children.is_empty() {
        return Ok(());
    }
    let whole = is_group && base.effects[first].scope == crate::doc::store::EffectScope::Whole;
    let whole_transform = (program.moves_whole)(params);
    let picks = (program.pick)(params, &children.iter().map(|c| c.0).collect::<Vec<_>>(), placements.len());
    // 引いた子の写しは番号の順に重ねる(AE の Repeater・Cavalry の Duplicator)。子の order のまま並べ直すと、同じ子の写しが全部まとまって重なる。
    let copies_slot = if is_group && !whole {
        children.iter().filter_map(|c| view.meta(*c).ok().flatten().map(|m| m.order)).min()
    } else {
        None
    };
    for (ordinal, result) in placements.into_iter().enumerate() {
        let (placement, outline) = (result.placement, result.outline_stretch);
        let Ok(at) = t.try_sub(placement.time_offset) else { continue };
        let shifted = at != t;
        let subjects: Vec<LayerId> = if whole {
            crate::picture::resolve::subtree(view, layer, present)?.into_iter().skip(1).collect()
        } else if is_group {
            crate::picture::resolve::subtree(view, children[picks[ordinal]], present)?
        } else {
            vec![layer]
        };
        let mut worlds_at = HashMap::new();
        let (mut memo_at, mut visiting_at) = (HashMap::new(), HashSet::new());
        if shifted {
            for subject in &subjects {
                worlds_at.extend(crate::picture::resolve::transform::world_transform3d_chain(view, *subject, at, present)?);
            }
            if let Some(p) = parent {
                worlds_at.extend(crate::picture::resolve::transform::world_transform3d_chain(view, p, at, present)?);
            }
            if whole_transform {
                worlds_at.extend(crate::picture::resolve::transform::world_transform3d_chain(view, layer, at, present)?);
            }
        }
        let worlds = if shifted { &worlds_at } else { world_transforms };
        let memo = if shifted { &mut memo_at } else { &mut *memo };
        let visiting = if shifted { &mut visiting_at } else { &mut *visiting };
        let parent2 = parent.map(|p| crate::picture::resolve::transform::world_affine(view, p, at, present, memo, visiting)).transpose()?.unwrap_or(glam::Affine2::IDENTITY);
        let parent3 = parent.and_then(|p| worlds.get(&p).copied()).unwrap_or(glam::Affine3A::IDENTITY);
        let pivot = glam::Vec2::from(crate::picture::resolve::transform::resolve_position(view, layer, at)?);
        // Each: ずれは親の空間、回転と大きさは層の位置が中心(複製 1 つずつ)。
        // Whole: ずれは層自身の空間、中心は層のアンカー — 層を回すと並びごと回る。
        let (frame2, frame3, around) = if whole_transform {
            let anchor = match view.value_at(layer, &crate::doc::store::PropertyId::new(crate::doc::store::property::ANCHOR)?, at)? {
                Some(crate::doc::store::Value::Vec2(v)) => glam::Vec2::new(v[0] as f32, v[1] as f32),
                _ => glam::Vec2::ZERO,
            };
            (crate::picture::resolve::transform::world_affine(view, layer, at, present, memo, visiting)?, worlds.get(&layer).copied().unwrap_or(glam::Affine3A::IDENTITY), anchor)
        } else {
            (parent2, parent3, pivot)
        };
        for subject in subjects {
            let mut copy = if !is_group && !shifted {
                base.clone()
            } else {
                let Some(copy) = crate::picture::resolve::resolve_with_solo(view, subject, at, any_solo, present, worlds, memo, visiting)? else { continue };
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
pub fn push_motion_blur(
    view: &StoreView<'_>,
    mut base: ResolvedLayer,
    at_index: usize,
    t: RationalTime,
    present: &HashSet<LayerId>,
    world_transforms: &HashMap<LayerId, glam::Affine3A>,
    memo: &mut HashMap<LayerId, glam::Affine2>,
    visiting: &mut HashSet<LayerId>,
    out: &mut Vec<ResolvedLayer>,
) -> Result<(), StoreError> {
    let frame_seconds = view.composition()?.map_or(0.0, |c| c.fps.den() as f64 / c.fps.num() as f64);
    let effect = &base.effects[at_index];
    let shutter = if base.source == crate::doc::store::LayerSource::Group || frame_seconds <= 0.0 { None }
        else { view.shutter_of(&effect.plugin_id, &effect.params) };
    let below = base.effects.split_off(at_index + 1);
    base.effects.pop();
    base.after_effects.splice(0..0, below);
    let Some(shutter) = shutter else {
        out.push(base);
        return Ok(());
    };
    let layer = base.id;
    let parent = view.attrs(layer)?.unwrap_or_default().parent.filter(|p| present.contains(p));
    let parent2 = parent.map(|p| crate::picture::resolve::transform::world_affine(view, p, t, present, memo, visiting)).transpose()?.unwrap_or(glam::Affine2::IDENTITY);
    let parent3 = parent.and_then(|p| world_transforms.get(&p).copied()).unwrap_or(glam::Affine3A::IDENTITY);
    let now_inverse = crate::picture::resolve::transform::local_placement_transform(view, layer, t)?.inverse();
    let local_delta = |at: RationalTime| -> Result<glam::Affine2, StoreError> {
        Ok(crate::picture::resolve::transform::local_placement_transform_sampled(view, layer, t, Some((at, shutter.channels)))? * now_inverse)
    };
    let samples = shutter.deltas(t, frame_seconds, base.declared_size, base.placement.transform, parent2, local_delta)?;
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

/// 文字の Split(GSAP SplitText / CSS `sibling-index()`): 字・語・行の単位を、その文字の層の Stagger でずれた時刻に解いた
/// 写しとして積む。書類に子の層は作らない — 写し = 層全体をずれた時刻で解き、単位の箱を Intersect の mask で切り、
/// 拡縮・回転の中心を単位の箱の中心へ移す(SplitText の char が自分の中心で回るのと同じ)。時刻の純関数。
pub fn push_split(
    view: &StoreView<'_>,
    base: &ResolvedLayer,
    t: RationalTime,
    any_solo: bool,
    present: &HashSet<LayerId>,
    out: &mut Vec<ResolvedLayer>,
) -> Result<bool, StoreError> {
    use crate::doc::store::layout::STAGGER;
    if base.source != crate::doc::store::LayerSource::Text || view.number(base.id, STAGGER, 0.0, t)? <= 0.0 {
        return Ok(false);
    }
    let units = crate::picture::text::text_units(view, base.id, t)?;
    if units.len() < 2 {
        return Ok(false);
    }
    let layer = base.id;
    let n = units.len();
    let parent = view.attrs(layer)?.unwrap_or_default().parent.filter(|p| present.contains(p));
    for (k, b) in units.into_iter().enumerate() {
        let at = view.schedule_shift(layer, k, n, t)?;
        let (mut memo, mut visiting) = (HashMap::new(), HashSet::new());
        let worlds = crate::picture::resolve::transform::world_transform3d_chain(view, layer, at, present)?;
        let Some(mut copy) = crate::picture::resolve::resolve_with_solo(view, layer, at, any_solo, present, &worlds, &mut memo, &mut visiting)? else { continue };
        // 中心を単位の箱の中心へ: 親の空間で T(c − a) を局所の変換に共役で掛ける(a = 層のアンカー、c = 箱の中心)。
        let anchor = glam::Vec2::from(crate::picture::boxes::free_anchor(view, layer, at)?);
        let shift = glam::vec2((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5) - anchor;
        let parent2 = parent.map(|p| crate::picture::resolve::transform::world_affine(view, p, at, present, &mut memo, &mut visiting)).transpose()?.unwrap_or(glam::Affine2::IDENTITY);
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
            shape: crate::picture::path::rounded_rect_path(b, 0.0, glam::Affine2::IDENTITY),
            frame: crate::doc::store::MaskFrame::Layer,
        });
        out.push(copy);
    }
    Ok(true)
}

/// ゴースト: 層を遅れ d だけ後に見た姿を 1 枚、同じ id で `ghost = true` にして積む。
/// 層に Repeater が掛かっていれば、その時刻の配置がそのまま増える。
pub fn push_ghosts(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
    any_solo: bool,
    present: &HashSet<LayerId>,
    out: &mut Vec<ResolvedLayer>,
) -> Result<(), StoreError> {
    let attrs = view.attrs(layer)?.unwrap_or_default();
    if attrs.environment { return Ok(()) }
    let Some(composition) = view.composition()? else { return Ok(()) };
    let fps = composition.fps;
    let parent = attrs.parent;
    for delay in attrs.ghost {
        let Ok(shift) = RationalTime::try_from_frame(delay.abs(), fps) else { continue };
        let at = if delay >= 0 { t.try_sub(shift) } else { t.try_add(shift) };
        let Ok(at) = at else { continue };
        let mut worlds = crate::picture::resolve::transform::world_transform3d_chain(view, layer, at, present)?;
        if let Some(parent) = parent {
            worlds.extend(crate::picture::resolve::transform::world_transform3d_chain(view, parent, at, present)?);
        }
        let (mut memo, mut visiting) = (HashMap::new(), HashSet::new());
        let Some(resolved) = crate::picture::resolve::resolve_with_solo(view, layer, at, any_solo, present, &worlds, &mut memo, &mut visiting)? else {
            continue;
        };
        let mut placed = Vec::new();
        push_placements(view, resolved, at, any_solo, present, &worlds, &mut memo, &mut visiting, &mut placed)?;
        for mut copy in placed {
            copy.ghost = true;
            out.push(copy);
        }
    }
    Ok(())
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
        crate::picture::resolve::resolved_layers(&doc.view(), t).unwrap().into_iter().filter(|l| l.id == id).map(|l| (l.ghost, l.placement.transform.translation.x)).collect()
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
