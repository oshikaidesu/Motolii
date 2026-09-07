//! 配置効果を**展開**する — 配置を N 層に切る(⌘D と同じく独立)。1 回の undo で戻る。
use std::collections::{HashMap, HashSet};

use crate::doc::store::{
    placement, property, Document, EffectId, Intent, KeyframeTrack, LayerId, LayerTiming,
    Placement, PropertyId, RationalTime, StoreError, StoreView, Value,
};
use crate::editor::timeline_edit::{copy_layers, remap_clipboard_intent};

pub(crate) fn expand_intents(
    doc: &Document,
    layer: LayerId,
    effect: EffectId,
    at: RationalTime,
) -> Result<(Vec<Intent>, Vec<LayerId>), StoreError> {
    let view = doc.view();
    let effects = view.effects(layer)?;
    let instance = effects
        .iter()
        .find(|e| e.id == effect)
        .ok_or_else(|| StoreError::Property("Effect missing".into()))?;
    let kind = placement::kind(&instance.plugin_id)
        .ok_or_else(|| StoreError::Property("Not a placement effect".into()))?;
    let params: Vec<(String, Value)> = kind
        .params
        .iter()
        .filter_map(|p| {
            let prop = PropertyId::effect_param(effect, p.name).ok()?;
            view.value_at(layer, &prop, at).ok().flatten().map(|v| (p.name.to_owned(), v))
        })
        .collect();
    let placements = placement::placements(kind, &params);
    let fps = view
        .composition()?
        .ok_or_else(|| StoreError::Property("No composition".into()))?
        .fps;
    let meta = view
        .meta(layer)?
        .ok_or_else(|| StoreError::Property("Layer missing".into()))?;
    if meta.source == crate::doc::store::LayerSource::Group {
        return Err(StoreError::Property("Expand does not apply to a group Repeater yet".into()));
    }
    let timing = meta.timing;
    let remaining: Vec<_> = effects.iter().filter(|e| e.id != effect).cloned().collect();

    let clipboard = copy_layers(doc, &[layer])?;
    if !clipboard.slots.is_empty() {
        return Err(StoreError::Property("Expand needs a layer without shared slots".into()));
    }
    let roots: HashSet<_> = clipboard.roots.iter().copied().collect();
    let mut next_order = view
        .layers()
        .into_iter()
        .filter(|l| view.attrs(*l).ok().flatten().and_then(|a| a.parent).is_none())
        .filter_map(|l| view.meta(l).ok().flatten().map(|m| m.order))
        .max()
        .unwrap_or(-1)
        .saturating_add(1);
    let first = view.next_layer_id();

    let mut intents = Vec::new();
    let mut made = vec![layer];
    for (k, placement) in placements.iter().enumerate() {
        let target = if k == 0 {
            layer
        } else {
            let base = first + (k as u64 - 1) * clipboard.copies.len() as u64;
            let ids: HashMap<_, _> = clipboard
                .copies
                .iter()
                .enumerate()
                .map(|(i, old)| (*old, LayerId(base + i as u64)))
                .collect();
            for intent in clipboard.intents.iter().cloned() {
                intents.push(remap_clipboard_intent(intent, &ids, &HashMap::new(), &roots, &mut next_order)?);
            }
            let root = clipboard.roots.first().copied().ok_or_else(|| StoreError::Property("Nothing to copy".into()))?;
            let id = ids[&root];
            made.push(id);
            id
        };
        intents.push(Intent::SetEffects { layer: target, effects: remaining.clone() });
        intents.extend(baked_transform(&view, layer, target, placement, at)?);
        if placement.time_offset != RationalTime::ZERO {
            let frames = placement
                .time_offset
                .try_to_frame_round(fps)
                .map_err(|e| StoreError::Property(e.to_string()))?;
            intents.push(Intent::SetTiming {
                layer: target,
                timing: LayerTiming { start: timing.start + frames, ..timing.clone() },
            });
        }
    }
    Ok((intents, made))
}

/// 配置のずれを `source` の値へ足した物を `target` に書く。キーがあれば全部のキーへ足す。
fn baked_transform(
    view: &StoreView<'_>,
    source: LayerId,
    target: LayerId,
    placement: &Placement,
    at: RationalTime,
) -> Result<Vec<Intent>, StoreError> {
    let mut out = Vec::new();
    let [dx, dy] = placement.offset.map(f64::from);
    let split = view.property_source(source, &PropertyId::new(property::POSITION)?)?.is_none()
        && (view.property_source(source, &PropertyId::new(property::POSITION_X)?)?.is_some()
            || view.property_source(source, &PropertyId::new(property::POSITION_Y)?)?.is_some());
    if dx != 0.0 || dy != 0.0 {
        if split {
            for (name, d) in [(property::POSITION_X, dx), (property::POSITION_Y, dy)] {
                out.extend(mapped(view, source, target, name, at, Value::F64(0.0), |v| add_f64(v, d))?);
            }
        } else {
            out.extend(mapped(view, source, target, property::POSITION, at, Value::Vec2([0.0, 0.0]), |v| match v {
                Value::Vec2([x, y]) => Value::Vec2([x + dx, y + dy]),
                other => other,
            })?);
        }
    }
    let rotation = f64::from(placement.rotation_degrees);
    if rotation != 0.0 {
        out.extend(mapped(view, source, target, property::ROTATION, at, Value::F64(0.0), |v| add_f64(v, rotation))?);
    }
    let scale = f64::from(placement.scale);
    if scale != 1.0 {
        out.extend(mapped(view, source, target, property::SCALE, at, Value::Vec2([1.0, 1.0]), |v| match v {
            Value::Vec2([x, y]) => Value::Vec2([x * scale, y * scale]),
            other => other,
        })?);
    }
    let opacity = f64::from(placement.opacity);
    if opacity != 1.0 {
        out.extend(mapped(view, source, target, property::OPACITY, at, Value::F64(1.0), |v| match v {
            Value::F64(o) => Value::F64((o * opacity).clamp(0.0, 1.0)),
            other => other,
        })?);
    }
    Ok(out)
}

fn add_f64(value: Value, delta: f64) -> Value {
    match value {
        Value::F64(v) => Value::F64(v + delta),
        other => other,
    }
}

fn mapped(
    view: &StoreView<'_>,
    source: LayerId,
    target: LayerId,
    name: &str,
    at: RationalTime,
    default: Value,
    f: impl Fn(Value) -> Value,
) -> Result<Option<Intent>, StoreError> {
    let property = PropertyId::new(name)?;
    Ok(Some(if let Some(track) = view.track(source, &property)? {
        let mut mapped = KeyframeTrack::new();
        for key in track.keys() {
            let mut key = key.clone();
            key.value = f(key.value);
            mapped.insert(key);
        }
        Intent::SetTrack { layer: target, property, track: mapped }
    } else {
        let current = view.value_at(source, &property, at)?.unwrap_or(default);
        Intent::SetConstant { layer: target, property, value: f(current) }
    }))
}

#[cfg(test)]
mod expand_contract {
    use super::*;
    use crate::doc::store::{
        Composition, EffectInstance, Fps, LayerAttrsPatch, LayerMeta, LayerSource,
    };

    fn document() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 60,
            background: [0.0; 4],
        }))
        .unwrap();
        let layer = LayerId(1);
        let effect = EffectId(0);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 60) },
            },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { name: Some("paper".into()), ..Default::default() } },
            Intent::SetConstant {
                layer,
                property: PropertyId::new(property::POSITION).unwrap(),
                value: Value::Vec2([5.0, 5.0]),
            },
            Intent::SetEffects {
                layer,
                effects: vec![
                    EffectInstance { id: effect, plugin_id: placement::REPEAT.to_owned() },
                    EffectInstance { id: EffectId(1), plugin_id: "motolii.blur".to_owned() },
                ],
            },
            Intent::SetConstant { layer, property: PropertyId::effect_param(effect, "count").unwrap(), value: Value::F64(3.0) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(effect, "position_each").unwrap(), value: Value::Vec2([10.0, 0.0]) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(effect, "delay_each").unwrap(), value: Value::F64(1.0) },
        ])
        .unwrap();
        doc
    }

    #[test]
    fn expanding_cuts_the_copies_into_independent_layers_in_one_undo() {
        let mut doc = document();
        let (undo_before, _) = doc.history_depth();
        let (intents, made) = expand_intents(&doc, LayerId(1), EffectId(0), RationalTime::ZERO).unwrap();
        doc.apply_all(intents).unwrap();
        assert_eq!(made.len(), 3);
        assert_eq!(doc.history_depth().0, undo_before + 1);
        let view = doc.view();
        assert_eq!(view.layers().len(), 3);
        for (k, id) in made.iter().enumerate() {
            let effects = view.effects(*id).unwrap();
            assert_eq!(effects.iter().map(|e| e.plugin_id.as_str()).collect::<Vec<_>>(), ["motolii.blur"]);
            let position = view.value_at(*id, &PropertyId::new(property::POSITION).unwrap(), RationalTime::ZERO).unwrap();
            assert_eq!(position, Some(Value::Vec2([5.0 + 10.0 * k as f64, 5.0])));
            assert_eq!(view.meta(*id).unwrap().unwrap().timing.start, 30 * k as i64);
        }
        // 1.5 秒: 元(0-2s)と 1 秒遅れの複製(1-3s)が居て、2 秒遅れの複製はまだ出ない。
        let visible = doc.view().resolved_layers(RationalTime::try_new(3, 2).unwrap()).unwrap();
        assert_eq!(visible.iter().map(|l| l.id).collect::<Vec<_>>(), made[..2]);
        assert!(visible.iter().all(|l| l.copy == 0 && l.after_effects.is_empty()));
    }
}
