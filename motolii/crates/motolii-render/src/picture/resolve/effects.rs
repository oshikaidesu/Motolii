//! 効果の列 — その層に、どの効果が、どの順で掛かるか。
//! 自分の効果、グループから降りてくる効果(子ごと / 1 枚にしてから)、
//! Motion Blur の写しと、その下に残る効果。掛ける仕事は描く側で、ここは列を組むだけ。

use super::*;
#[allow(unused_imports)]
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};

pub fn effect_enabled(view: &StoreView<'_>, layer: LayerId, effect: crate::doc::store::EffectId, t: RationalTime) -> Result<bool, StoreError> {
    match view.value_at(layer, &crate::doc::store::PropertyId::effect_enabled(effect), t)? {
        Some(Value::Bool(value)) => Ok(value),
        Some(other) => Err(StoreError::Property(format!("effect {effect} の enabled に真偽でない値が入っている: {other:?}"))),
        None => Ok(true),
    }
}

pub fn resolved_effects(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
) -> Result<Vec<ResolvedEffect>, StoreError> {
    let effects = view.effects(layer)?;
    if effects.is_empty() {
        return Ok(Vec::new());
    }

    let properties = view.properties(layer);

    let mut out = Vec::with_capacity(effects.len());
    for effect in effects {
        if !effect_enabled(view, layer, effect.id, t)? {
            continue;
        }
        let prefix = format!("{}{}.param.", property::EFFECT_PREFIX, effect.id);
        let mut params = Vec::new();
        for candidate in &properties {
            let Some(param_name) = candidate.name().strip_prefix(prefix.as_str()) else {
                continue;
            };
            if let Some(value) = view.value_at(layer, candidate, t)? {
                params.push((param_name.to_owned(), value));
            }
        }
        let scope = match view.value_at(layer, &crate::doc::store::PropertyId::effect_scope(effect.id), t)? {
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

/// グループが子へ配る効果と、子を 1 枚にしてから掛ける効果。配置効果は数える側なので入らない。
/// Whole が 1 つ現れた所から下は、もう子が無い(板)ので全部 Whole 扱い。
pub fn group_effects(view: &StoreView<'_>, group: LayerId, t: RationalTime) -> Result<(Vec<ResolvedEffect>, Vec<ResolvedEffect>), StoreError> {
    let (mut each, mut whole) = (Vec::new(), Vec::new());
    for effect in resolved_effects(view, group, t)? {
        if view.placement_program(&effect.plugin_id).is_some() {
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
pub fn handed_down(view: &StoreView<'_>, layer: LayerId, t: RationalTime, present: &HashSet<LayerId>) -> Result<(Vec<ResolvedEffect>, Option<(LayerId, Vec<ResolvedEffect>)>), StoreError> {
    let mut each = Vec::new();
    let mut seen = HashSet::from([layer]);
    let mut next = view.attrs(layer)?.unwrap_or_default().parent.filter(|p| present.contains(p));
    while let Some(group) = next {
        if !seen.insert(group) || !view.meta(group)?.is_some_and(|m| m.source == crate::doc::store::LayerSource::Group) {
            break;
        }
        let (own_each, own_whole) = group_effects(view, group, t)?;
        each.extend(own_each);
        if !own_whole.is_empty() {
            let (above, _) = handed_down(view, group, t, present)?;
            let mut plate = own_whole;
            plate.extend(above);
            return Ok((each, Some((group, plate))));
        }
        next = view.attrs(group)?.unwrap_or_default().parent.filter(|p| present.contains(p));
    }
    Ok((each, None))
}

#[cfg(test)]
mod group_scope_contract {
    use crate::picture::resolved::ResolvedLayer;
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
        let out = crate::picture::resolve::resolved_layers(&doc.view(), RationalTime::ZERO).unwrap();
        let plugins = |l: &ResolvedLayer| l.effects.iter().map(|e| e.plugin_id.clone()).collect::<Vec<_>>();
        assert_eq!(plugins(find(&out, leaf)), ["leaf.own", "inner.each", "outer.each"], "own first, then nearest group, then the one above");
        assert_eq!(plugins(find(&out, sibling)), ["outer.each"]);
        assert!(out.iter().all(|l| l.plate.is_none() && l.after_effects.is_empty()), "no Whole, no plate");

        // inner に Whole を積む: leaf は inner の板の一部。板には inner の Whole と、outer から inner へ配られた効果が掛かる。
        effect(&mut doc, inner, 1, "inner.whole", true);
        effect(&mut doc, inner, 2, "inner.after", false);
        let out = crate::picture::resolve::resolved_layers(&doc.view(), RationalTime::ZERO).unwrap();
        let leaf_r = find(&out, leaf);
        assert_eq!(plugins(leaf_r), ["leaf.own", "inner.each"], "the outer group's Each now lands on the plate, not the leaf");
        assert_eq!(leaf_r.plate, Some(inner));
        assert_eq!(leaf_r.after_effects.iter().map(|e| e.plugin_id.as_str()).collect::<Vec<_>>(), ["inner.whole", "inner.after", "outer.each"], "below a Whole everything is Whole");
        assert!(find(&out, sibling).plate.is_none());

        // 単層の scope は無意味: 値を書いても板にはならない。
        doc.apply(Intent::SetConstant { layer: sibling, property: PropertyId::effect_scope(EffectId(0)), value: Value::Enum(1) }).unwrap();
        let out = crate::picture::resolve::resolved_layers(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(find(&out, sibling).plate.is_none());
    }
}
