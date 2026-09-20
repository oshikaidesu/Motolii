//! 層を切って割る(再生ヘッドでの split)。
#[allow(unused_imports)]
use crate::edit::{Animate, Document, Intent};
use crate::doc::store::{KeyframeTrack, LayerId, LayerTiming, RationalTime, StoreError};
use crate::editor::functions::atom;
use crate::editor::keyframe_edit::document_fps;
use super::copy_paste::copy_plan;

pub(crate) fn split_layers(
    doc: &mut Document,
    layers: &[LayerId],
    comp_frame: i64,
) -> Result<Vec<LayerId>, StoreError> {
    let mut eligible = Vec::new();
    for &layer in layers {
        let view = doc.view().without_transients();
        if let Some(reason) = crate::editor::functions::lens::edit_rejection(&view, layer)? {
            println!(
                "PROBE room=write verdict=split-skip layer={} reason={reason}",
                layer.0
            );
            continue;
        }
        let covers_cut = match view.meta(layer)? {
            Some(meta) => {
                let span = atom::frame_span(meta.timing.start, meta.timing.duration)
                    .map_err(|reason| StoreError::Property(reason.into()))?;
                span.start < comp_frame && span.contains(&comp_frame)
            }
            None => false,
        };
        if covers_cut {
            if !eligible.contains(&layer) {
                eligible.push(layer);
            }
        } else {
            println!(
                "PROBE room=write verdict=split-skip layer={} reason=playhead-outside-layer",
                layer.0
            );
        }
    }
    let mut plan = copy_plan(&doc, &eligible)?;
    for &(source, tail) in &plan.roots {
        plan.intents
            .extend(split_intents(&doc, source, tail, comp_frame, &plan.ids)?);
    }
    doc.apply_all(plan.intents)?;
    Ok(plan.roots.into_iter().map(|(_, tail)| tail).collect())
}

fn split_intents(
    doc: &Document,
    layer: LayerId,
    tail: LayerId,
    comp_frame: i64,
    ids: &std::collections::HashMap<LayerId, LayerId>,
) -> Result<Vec<Intent>, StoreError> {
    let view = doc.view().without_transients();
    let Some(meta) = view.meta(layer)? else {
        return Ok(Vec::new());
    };
    let span = atom::frame_span(meta.timing.start, meta.timing.duration)
        .map_err(|reason| StoreError::Property(reason.into()))?;
    if !span.contains(&comp_frame) || comp_frame == span.start {
        return Ok(Vec::new());
    }
    let (head, tail_span) =
        atom::split_span(span, comp_frame).map_err(|reason| StoreError::Property(reason.into()))?;
    let head_timing = LayerTiming {
        duration: head.end - head.start,
        ..meta.timing
    };
    let tail_timing = LayerTiming {
        start: tail_span.start,
        duration: tail_span.end - tail_span.start,
        source_in: meta
            .timing
            .source_frame(comp_frame)
            .ok_or_else(|| StoreError::Property("Split is outside the source".into()))?,
        ..meta.timing
    };

    let text = view.text_document(layer)?;
    let fps = document_fps(doc)?;
    let cut = RationalTime::try_from_frame(comp_frame, fps)
        .map_err(|error| StoreError::Property(error.to_string()))?;
    let tracks: Vec<_> = view
        .properties(layer)
        .into_iter()
        .filter_map(|p| view.track(layer, &p).ok().flatten().map(|t| (p, t)))
        .collect();

    let mut intents = vec![
        Intent::SetTiming {
            layer,
            timing: head_timing,
        },
        Intent::SetTiming {
            layer: tail,
            timing: tail_timing,
        },
    ];
    if let Some(document) = text {
        // 歌詞の切替も切り口で分ける。頭は切り口より前、尻は切り口の行から(両方に全部を配らない)。
        let keys = document.content.keys().to_vec();
        if keys.len() > 1 {
            let at_cut = keys
                .iter()
                .filter(|k| k.t <= cut)
                .last()
                .or(keys.first())
                .cloned();
            let mut head_track = crate::doc::store::ContentTrack::new();
            let mut tail_track = crate::doc::store::ContentTrack::new();
            for k in &keys {
                if k.t < cut {
                    head_track.insert(k.clone());
                } else {
                    tail_track.insert(k.clone());
                }
            }
            if let Some(k) = at_cut {
                if head_track.keys().is_empty() {
                    head_track.insert(k.clone());
                }
                tail_track.insert(crate::doc::store::ContentKeyframe {
                    t: cut,
                    content: k.content.clone(),
                });
            }
            let mut head_doc = document.clone();
            head_doc.content = head_track;
            intents.push(Intent::SetTextDocument {
                layer,
                document: head_doc,
            });
            let mut tail_doc = document.clone();
            tail_doc.content = tail_track;
            intents.push(Intent::SetTextDocument {
                layer: tail,
                document: tail_doc,
            });
        } else {
            intents.push(Intent::SetTextDocument {
                layer: tail,
                document,
            });
        }
    }
    for (property, track) in tracks {
        let (head, tail_track) = split_track_at(&track, cut);
        if let Some(head) = head {
            intents.push(Intent::SetTrack {
                layer,
                property: property.clone(),
                track: head,
            });
        }
        intents.push(Intent::SetTrack {
            layer: tail,
            property: property.clone(),
            track: tail_track,
        });
        if let Some(source) = view.property_source(layer, &property)? {
            if !source.modulators.is_empty() {
                intents.push(Intent::SetPropertyModulators {
                    layer,
                    property: property.clone(),
                    modulators: source.modulators.clone(),
                });
                let modulators = source
                    .modulators
                    .into_iter()
                    .map(|mut link| {
                        link.source_layer = ids
                            .get(&link.source_layer)
                            .copied()
                            .unwrap_or(link.source_layer);
                        link
                    })
                    .collect();
                intents.push(Intent::SetPropertyModulators {
                    layer: tail,
                    property,
                    modulators,
                });
            }
        }
    }

    Ok(intents)
}

/// 切り口を跨ぐ区間のイージングを両側へ分ける(Premiere・Resolve は切っても見た目が変わらない)。
/// 跨ぐ区間が無ければ頭はそのまま(None)、尻は丸ごと写す。
fn split_track_at(
    track: &KeyframeTrack,
    cut: RationalTime,
) -> (Option<KeyframeTrack>, KeyframeTrack) {
    let keys = track.keys();
    let straddle = keys
        .windows(2)
        .position(|pair| pair[0].t < cut && cut < pair[1].t);
    let Some(i) = straddle else {
        return (None, track.clone());
    };
    let (a, b) = (&keys[i], &keys[i + 1]);
    let progress = (cut.as_seconds_f64() - a.t.as_seconds_f64())
        / (b.t.as_seconds_f64() - a.t.as_seconds_f64()).max(f64::EPSILON);
    let Ok((first, second)) = a.interp.split_at(progress) else {
        return (None, track.clone());
    };
    let at_cut = crate::doc::store::Keyframe {
        t: cut,
        value: track.eval(cut),
        interp: second,
        spatial: None,
    };
    let mut head = KeyframeTrack::new();
    let mut tail = KeyframeTrack::new();
    for (k, key) in keys.iter().enumerate() {
        let mut key = key.clone();
        if k == i {
            key.interp = first;
        }
        if k <= i {
            head.insert(key.clone());
        }
        if k > i {
            tail.insert(key);
        }
    }
    head.insert(at_cut.clone());
    tail.insert(at_cut);
    (Some(head), tail)
}
