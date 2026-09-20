//! 切り — 層に掛かるマスクを解く。自分の書いたマスク、行列の Matte、
//! Overflow が Clip の祖先の箱。どれも「どの形で、どの枠に付いて、どう足すか」の 3 つで決まる。

use super::*;
#[allow(unused_imports)]
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};

/// この層に掛かる切りを全部: 自分の書いたマスク・行列の Matte に、祖先の箱の切りを足したもの。
/// 解く側はこの一口だけ使う(順番を間違えると、祖先の箱が自分のマスクの前に掛かる)。
pub fn masks_of(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
    present: &HashSet<LayerId>,
    memo: &mut HashMap<LayerId, glam::Affine2>,
    visiting: &mut HashSet<LayerId>,
) -> Result<Vec<ResolvedMask>, StoreError> {
    let masks = resolved_masks(view, layer, t)?;
    clipped_masks(view, layer, t, masks, present, memo, visiting)
}

pub fn resolved_masks(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
) -> Result<Vec<ResolvedMask>, StoreError> {
    let mut out = Vec::new();
    for mask in view.masks(layer)? {
        let mode = match view.value_at(layer, &PropertyId::mask_mode(mask.id), t)? {
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

        let inverted = match view.value_at(layer, &PropertyId::mask_inverted(mask.id), t)? {
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
        let shape = match view.value_at(layer, &shape_property, t)? {
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
        let opacity = match view.value_at(layer, &opacity_property, t)? {
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
        let expansion = match view.value_at(layer, &expansion_property, t)? {
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

/// Overflow が Clip の並べる Group の子孫は、その箱で切る: 箱を層の素材座標へ写した角丸の矩形を Intersect で足す。
/// 祖先の箱の切りは箱の枠に付く(`MaskFrame::Box`)、自分の inset は自分の枠(`Layer`)。
pub fn clipped_masks(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
    mut masks: Vec<ResolvedMask>,
    present: &HashSet<LayerId>,
    memo: &mut HashMap<LayerId, glam::Affine2>,
    visiting: &mut HashSet<LayerId>,
) -> Result<Vec<ResolvedMask>, StoreError> {
    // 奥行きを持つ網・点群は 2D の mask で切れない(切り口は面の法の宿題)。
    if let Some(crate::doc::store::LayerMeta { source: crate::doc::store::LayerSource::File { path, .. }, .. }) = view.meta(layer)? {
        if view.analysis().and_then(|a| a.extent(&path)).is_some_and(|e| e[2] > 0.0) {
            return Ok(masks);
        }
    }
    let mut seen = HashSet::new();
    // inset は自分の背景も切る(CSS の clip-path は要素ごと)。Overflow は箱の外の子孫だけ。
    let mut next = Some(layer);
    let mut own: Option<glam::Affine2> = None;
    while let Some(group) = next.filter(|g| seen.insert(*g) && present.contains(g)) {
        let cuts = if group == layer { [None, crate::picture::boxes::clip_inset(view, group, t)?] } else { [crate::picture::boxes::clip_box(view, group, t)?, crate::picture::boxes::clip_inset(view, group, t)?] };
        for (b, radius) in cuts.into_iter().flatten() {
            let world = match own {
                Some(w) => w,
                None => *own.insert(crate::picture::resolve::transform::world_affine(view, layer, t, present, memo, visiting)?),
            };
            let to = world.inverse() * crate::picture::resolve::transform::world_affine(view, group, t, present, memo, visiting)?;
            masks.push(ResolvedMask {
                mode: crate::doc::store::MaskMode::Intersect,
                inverted: false,
                opacity: 1.0,
                expansion: 0.0,
                shape: crate::picture::path::rounded_rect_path(b, radius, to),
                frame: if group == layer { crate::doc::store::MaskFrame::Layer } else { crate::doc::store::MaskFrame::Box },
            });
        }
        next = view.attrs(group)?.unwrap_or_default().parent;
    }
    Ok(masks)
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

    /// 子を 1 つずつ引く配置効果。この検査が要る物だけを置く — コアは同梱の効果を知らない。
    const FOUR_COPIES: &str = "test.four-copies";

    fn four_copies(_input: &kind::PlacementInput<'_>) -> Vec<kind::PlacementOutput> {
        (0..4)
            .map(|index| kind::Placement {
                index,
                offset: [0.0; 2],
                rotation_degrees: 0.0,
                scale: 1.0,
                offset_z: 0.0,
                opacity: 1.0,
                time_offset: RationalTime::ZERO,
                stretch: [1.0; 2],
            })
            .map(Into::into)
            .collect()
    }

    fn picking_programs() -> kind::Programs {
        fn program(plugin_id: &str) -> Option<kind::PlacementProgram> {
            (plugin_id == FOUR_COPIES).then_some(kind::PlacementProgram {
                plugin_id: FOUR_COPIES,
                needs_position: false,
                evaluate: four_copies,
                pick: kind::pick_in_turn,
                moves_whole: kind::never_moves_whole,
            })
        }
        kind::Programs { placement: program, ..kind::Programs::NONE }
    }

    /// 配置効果が Group の子を 1 つずつ引く時、写しは番号の順に重なる(子ごとにまとまらない)。Cavalry の Concentrick で踏んだ。
    #[test]
    fn picked_copies_stack_by_their_number_not_by_child() {
        let mut doc = blank_project().with_programs(picking_programs());
        let group = add(&mut doc, 9, 5, None, false);
        let ink = add(&mut doc, 2, 1, Some(group), false);
        let paper = add(&mut doc, 3, 2, Some(group), false);
        doc.apply(Intent::SetEffects { layer: group, effects: vec![EffectInstance { id: EffectId(1), plugin_id: FOUR_COPIES.to_owned() }] }).unwrap();
        let resolved = crate::picture::resolve::resolved_layers(&doc.view(), RationalTime::ZERO).unwrap();
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
        let matte_of = |doc: &Document, id: LayerId| crate::picture::resolve::resolved_layers(&doc.view(), RationalTime::ZERO).unwrap().into_iter().find(|l| l.id == id).and_then(|l| l.matte);
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
        let resolved = crate::picture::resolve::resolve(&doc.view(), second, RationalTime::ZERO).unwrap().unwrap();
        assert!(resolved.clip_to_below);
        assert_eq!(resolved.matte, Some(Matte { layer: base, mode: MatteMode::Alpha }));

        doc.apply(Intent::SetOrder { layer: base, order: 25 }).unwrap();
        assert_eq!(doc.view().clipping_base(second).unwrap(), None);
        let orphan = crate::picture::resolve::resolve(&doc.view(), second, RationalTime::ZERO).unwrap().unwrap();
        assert!(orphan.clip_to_below && orphan.matte.is_none());
        assert!(doc.undo());
        assert_eq!(doc.view().clipping_base(second).unwrap(), Some(base));

        let inserted = add(&mut doc, 6, 15, None, false);
        assert_eq!(doc.view().clipping_base(first).unwrap(), Some(base));
        assert_eq!(doc.view().clipping_base(second).unwrap(), Some(inserted));
        doc.apply(Intent::SetAttrs { layer: inserted, patch: LayerAttrsPatch {
            hidden: Some(true), ..Default::default()
        }}).unwrap();
        assert!(crate::picture::resolve::resolve(&doc.view(), inserted, RationalTime::ZERO).unwrap().is_none());
        assert_eq!(crate::picture::resolve::resolve(&doc.view(), second, RationalTime::ZERO).unwrap().unwrap().matte,
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
        assert_eq!(crate::picture::resolve::resolve(&doc.view(), layer, RationalTime::ZERO).unwrap().unwrap().matte,
            Some(Matte { layer: base, mode: MatteMode::Alpha }));
        assert!(doc.undo());
        let previous = crate::picture::resolve::resolve(&doc.view(), layer, RationalTime::ZERO).unwrap().unwrap();
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
        assert_eq!(crate::picture::resolve::resolve(&loaded.view(), layer, RationalTime::ZERO).unwrap().unwrap().matte,
            Some(Matte { layer: base, mode: MatteMode::Alpha }));
    }
}
