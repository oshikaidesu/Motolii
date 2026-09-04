use super::atom;
use crate::doc::store::{
    Document, EffectId, EffectInstance, Intent, KeyframeTrack, LayerId, LayerSource, LayerTiming,
    RationalTime, ShapeNode, StoreError,
};
use crate::doc::vector::{Brush, Rgb};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TimingMode {
    Move,
    TrimStart,
    TrimEnd,
    Slip,
}

pub(crate) fn timing_delta(
    orig: LayerTiming,
    mode: TimingMode,
    delta: i64,
) -> Result<LayerTiming, StoreError> {
    let add = |base, delta| {
        atom::translate_frames(base, delta).map_err(|reason| StoreError::Property(reason.into()))
    };
    if orig.duration < 1 || orig.start < 0 || orig.source_in < 0 {
        return Err(StoreError::Property("Invalid layer timing".into()));
    }
    let span = atom::frame_span(orig.start, orig.duration)
        .map_err(|reason| StoreError::Property(reason.into()))?;
    Ok(match mode {
        TimingMode::Move => {
            let shifted = atom::shift_span(span, delta.max(-orig.start))
                .map_err(|reason| StoreError::Property(reason.into()))?;
            LayerTiming {
                start: shifted.start,
                ..orig
            }
        }
        TimingMode::Slip => LayerTiming {
            source_in: add(
                orig.source_in,
                delta
                    .checked_neg()
                    .ok_or_else(|| StoreError::Property("Frame position overflow".into()))?,
            )?
            .max(0),
            ..orig
        },
        TimingMode::TrimStart => {
            let delta = atom::clamp_to_span(delta, -orig.start.min(orig.source_in)..orig.duration)
                .map_err(|reason| StoreError::Property(reason.into()))?;
            LayerTiming {
                start: add(orig.start, delta)?,
                duration: orig.duration - delta,
                source_in: add(orig.source_in, delta)?,
                ..orig
            }
        }
        TimingMode::TrimEnd => {
            let duration = add(orig.duration, delta.max(1 - orig.duration))?;
            atom::frame_span(orig.start, duration)
                .map_err(|reason| StoreError::Property(reason.into()))?;
            LayerTiming { duration, ..orig }
        }
    })
}

/// Add a Browser selection as one effect-stack replacement. Existing plugin
/// identities and duplicates in the requested selection are kept to one
/// instance, so repeated activation is an empty edit rather than another Undo.
pub(crate) fn effect_batch_intents(
    doc: &Document,
    layer: LayerId,
    plugin_ids: &[String],
) -> Result<Vec<Intent>, StoreError> {
    let mut effects = doc.view().effects(layer)?;
    let before_len = effects.len();
    let mut known = std::collections::BTreeSet::new();
    effects.retain(|effect| known.insert(effect.plugin_id.clone()));
    let mut last_id = effects.iter().map(|effect| effect.id.0).max();
    let mut changed = effects.len() != before_len;
    for plugin_id in plugin_ids {
        if !known.insert(plugin_id.clone()) {
            continue;
        }
        let next_id = match last_id {
            Some(id) => id
                .checked_add(1)
                .ok_or_else(|| StoreError::Property("effect id space exhausted".into()))?,
            None => 0,
        };
        effects.push(EffectInstance {
            id: EffectId(next_id),
            plugin_id: plugin_id.clone(),
        });
        last_id = Some(next_id);
        changed = true;
    }
    Ok(changed
        .then_some(Intent::SetEffects { layer, effects })
        .into_iter()
        .collect())
}

fn set_shape_fill_color(node: &mut ShapeNode, brush: Brush) {
    match node {
        ShapeNode::Leaf(shape) => {
            if let Some(fill) = shape.fill.as_mut() {
                fill.brush = brush;
            }
        }
        ShapeNode::Group(group) => {
            for child in group.children.iter_mut() {
                set_shape_fill_color(child, brush.clone());
            }
        }
    }
}

pub(crate) fn color_block(
    doc: &Document,
    layer: LayerId,
    rgba: [u8; 4],
) -> Result<super::compose::Block, StoreError> {
    let supported = matches!(
        doc.view().meta(layer)?.map(|meta| meta.source),
        Some(LayerSource::Text | LayerSource::Shape)
    );
    if !supported {
        return Ok(super::compose::Block::Rejected(
            "This layer has no fill color".into(),
        ));
    }
    Ok(super::compose::Block::Edits(color_intents(
        doc, layer, rgba,
    )?))
}

pub(crate) fn color_intents(
    d: &Document,
    layer: LayerId,
    rgba: [u8; 4],
) -> Result<Vec<Intent>, StoreError> {
    let source = d.view().meta(layer)?.map(|m| m.source);
    let intent = match source {
        Some(LayerSource::Text) => {
            d.view().text_document(layer)?.and_then(|mut document| {
                let before = document.clone();
                // 色を変える手は色だけ触る。α(帯で置いた値)は残す。
                for style in document.styles.iter_mut() {
                    style.fill = [
                        rgba[0] as f64 / 255.0,
                        rgba[1] as f64 / 255.0,
                        rgba[2] as f64 / 255.0,
                        style.fill[3],
                    ];
                }
                (document != before).then_some(Intent::SetTextDocument { layer, document })
            })
        }
        Some(LayerSource::Shape) => {
            let mut shapes = d.view().shapes(layer)?;
            let before = shapes.clone();
            if shapes.is_empty() {
                None
            } else {
                let brush = Brush::Solid(Rgb {
                    r: rgba[0] as f64 / 255.0,
                    g: rgba[1] as f64 / 255.0,
                    b: rgba[2] as f64 / 255.0,
                });
                for node in shapes.iter_mut() {
                    set_shape_fill_color(node, brush.clone());
                }
                (shapes != before).then_some(Intent::SetShapes { layer, shapes })
            }
        }
        _ => None,
    };
    Ok(intent.into_iter().collect())
}

#[derive(Clone, Copy)]
pub(crate) enum LayerFlag {
    Hidden,
    Solo,
    Locked,
}

pub(crate) fn flag_intents(
    doc: &Document,
    layer: LayerId,
    flag: LayerFlag,
    value: bool,
) -> Result<Vec<Intent>, StoreError> {
    let attrs = doc.view().attrs(layer)?.unwrap_or_default();
    let current = match flag {
        LayerFlag::Hidden => attrs.hidden,
        LayerFlag::Solo => attrs.solo,
        LayerFlag::Locked => attrs.locked,
    };
    if current == value {
        return Ok(Vec::new());
    }
    let patch = match flag {
        LayerFlag::Hidden => crate::doc::store::LayerAttrsPatch {
            hidden: Some(value),
            ..Default::default()
        },
        LayerFlag::Solo => crate::doc::store::LayerAttrsPatch {
            solo: Some(value),
            ..Default::default()
        },
        LayerFlag::Locked => crate::doc::store::LayerAttrsPatch {
            locked: Some(value),
            ..Default::default()
        },
    };
    Ok(vec![Intent::SetAttrs { layer, patch }])
}

pub(crate) fn keyframe_shift_intents(
    doc: &Document,
    layer: LayerId,
    delta_frames: i64,
) -> Result<Vec<Intent>, StoreError> {
    let view = doc.view().without_transients();
    let fps = doc
        .view()
        .composition()?
        .ok_or_else(|| StoreError::Property("Composition has no frame rate".into()))?
        .fps;
    let shift = RationalTime::try_from_frame(delta_frames, fps)
        .map_err(|e| StoreError::Property(e.to_string()))?;
    let mut intents = Vec::new();
    for property in view.properties(layer) {
        super::lens::require_local_source(&view, layer, &property)?;
        let Some(track) = view.track(layer, &property)? else {
            continue;
        };
        let mut shifted = KeyframeTrack::new();
        for key in track.keys() {
            let mut key = key.clone();
            key.t = key
                .t
                .try_add(shift)
                .map_err(|e| StoreError::Property(e.to_string()))?;
            shifted.insert(key);
        }
        intents.push(Intent::SetTrack {
            layer,
            property,
            track: shifted,
        });
    }
    // 文字の切替(ContentTrack)は property でなく data。層と一緒に動かさないと歌詞の時刻だけ置き去りになる。
    if let Some(mut text) = view.text_document(layer)? {
        if !text.content.keys().is_empty() {
            let mut moved = crate::doc::store::ContentTrack::new();
            for key in text.content.keys() {
                let t = key
                    .t
                    .try_add(shift)
                    .map_err(|e| StoreError::Property(e.to_string()))?;
                moved.insert(crate::doc::store::ContentKeyframe {
                    t,
                    content: key.content.clone(),
                });
            }
            text.content = moved;
            intents.push(Intent::SetTextDocument {
                layer,
                document: text,
            });
        }
    }
    Ok(intents)
}

pub(crate) fn retime_layer(
    doc: &Document,
    layer: LayerId,
    original: LayerTiming,
    changed: LayerTiming,
    move_keys: bool,
) -> Result<Vec<Intent>, StoreError> {
    if original == changed {
        return Ok(Vec::new());
    }
    let mut intents = vec![Intent::SetTiming {
        layer,
        timing: changed,
    }];
    if move_keys {
        intents.extend(keyframe_shift_intents(
            doc,
            layer,
            changed.start - original.start,
        )?);
    }
    Ok(intents)
}
