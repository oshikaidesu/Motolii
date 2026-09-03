//! 層をいじる手(複製・分割)。timeline_widget.rs が 1737 行の天井を越えたので分けた。
use std::sync::{Arc, Mutex};
use crate::doc::store::{
    Document, Intent, KeyframeTrack, LayerAttrs, LayerAttrsPatch, LayerId, LayerMeta,
    LayerTiming, RationalTime};
use crate::ui::timeline_widget::document_fps;

pub(super) fn attrs_to_patch(a: &LayerAttrs) -> LayerAttrsPatch {
    LayerAttrsPatch {
        flatten: Some(a.flatten),
        hidden: Some(a.hidden),
        parent: Some(a.parent),
        blend_mode: Some(a.blend_mode.clone()),
        matte: Some(a.matte.clone()),
        name: Some(a.name.clone()),
        auto_orient: Some(a.auto_orient),
        pinned: Some(a.pinned),
        solo: Some(a.solo),
        locked: Some(a.locked),
        label_color: Some(a.label_color),
    }
}

/// 層をそのまま増やす。中身(尺・見え方・エフェクト・キー)は全部連れていく。
/// 重ね順だけ1つ上へ置く — AE の Cmd+D と同じで、複製は元の上に出る。
pub(super) fn duplicate_layer(doc: &Arc<Mutex<Document>>, layer: LayerId) -> Option<LayerId> {
    let mut doc = doc.lock().unwrap();
    let view = doc.view();
    let meta = view.meta(layer).ok().flatten()?;
    let copy = LayerId(view.next_layer_id());
    let attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
    let effects = view.effects(layer).unwrap_or_default();
    let tracks: Vec<_> = view
        .properties(layer)
        .into_iter()
        .filter_map(|p| view.track(layer, &p).ok().flatten().map(|t| (p, t)))
        .collect();
    let shapes = view.shapes(layer).unwrap_or_default();
    let text = view.text_document(layer).ok().flatten();

    let above: Vec<(LayerId, i16)> = view
        .layers()
        .into_iter()
        .filter(|l| *l != layer)
        .filter_map(|l| view.meta(l).ok().flatten().map(|m| (l, m.order)))
        .filter(|(_, order)| *order > meta.order)
        .collect();
    let mut intents = vec![
        Intent::AddLayer(copy),
        Intent::SetMeta {
            layer: copy,
            meta: LayerMeta { order: meta.order.saturating_add(1), ..meta.clone() },
        },
        Intent::SetAttrs { layer: copy, patch: attrs_to_patch(&attrs) },
    ];
    // 複製は元の**すぐ上**に割り込む。上に居た層は 1 つずつ退く。
    for (l, order) in above {
        intents.push(Intent::SetOrder { layer: l, order: order.saturating_add(1) });
    }
    if !effects.is_empty() {
        intents.push(Intent::SetEffects { layer: copy, effects });
    }
    if !shapes.is_empty() {
        intents.push(Intent::SetShapes { layer: copy, shapes });
    }
    if let Some(document) = text {
        intents.push(Intent::SetTextDocument { layer: copy, document });
    }
    for (property, track) in tracks {
        intents.push(Intent::SetTrack { layer: copy, property, track });
    }

    doc.apply_all(intents).ok()?;
    Some(copy)
}

pub(super) fn split_layer(doc: &Arc<Mutex<Document>>, layer: LayerId, comp_frame: i64) -> Option<LayerId> {
    let mut doc = doc.lock().unwrap();
    let view = doc.view();
    let meta = view.meta(layer).ok().flatten()?;
    if !meta.timing.covers(comp_frame) {
        return None;
    }
    let head_dur = comp_frame - meta.timing.start;
    if head_dur <= 0 {
        return None;
    }

    let tail = LayerId(view.next_layer_id());
    let head_timing = LayerTiming { duration: head_dur, ..meta.timing };
    let tail_timing = LayerTiming {
        start: comp_frame,
        duration: meta.timing.duration - head_dur,
        source_in: meta.timing.source_in + head_dur,
        ..meta.timing
    };

    let attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
    let effects = view.effects(layer).unwrap_or_default();
    let shapes = view.shapes(layer).unwrap_or_default();
    let text = view.text_document(layer).ok().flatten();
    let fps = document_fps(&doc).ok()?;
    let cut = RationalTime::try_from_frame(comp_frame, fps).ok()?;
    let tracks: Vec<_> = view
        .properties(layer)
        .into_iter()
        .filter_map(|p| view.track(layer, &p).ok().flatten().map(|t| (p, t)))
        .collect();

    let mut intents = vec![
        Intent::SetTiming { layer, timing: head_timing },
        Intent::AddLayer(tail),
        Intent::SetMeta { layer: tail, meta: LayerMeta { timing: tail_timing, ..meta } },
        Intent::SetAttrs { layer: tail, patch: attrs_to_patch(&attrs) },
    ];
    if !effects.is_empty() {
        intents.push(Intent::SetEffects { layer: tail, effects });
    }
    // 形と文字も連れていく(複製と同じ)。切らないと歌詞層の尻が空になる。
    if !shapes.is_empty() {
        intents.push(Intent::SetShapes { layer: tail, shapes });
    }
    if let Some(document) = text {
        // 歌詞の切替も切り口で分ける。頭は切り口より前、尻は切り口の行から(両方に全部を配らない)。
        let keys = document.content.keys().to_vec();
        if keys.len() > 1 {
            let at_cut = keys.iter().filter(|k| k.t <= cut).last().or(keys.first()).cloned();
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
                tail_track.insert(crate::doc::store::ContentKeyframe { t: cut, content: k.content.clone() });
            }
            let mut head_doc = document.clone();
            head_doc.content = head_track;
            intents.push(Intent::SetTextDocument { layer, document: head_doc });
            let mut tail_doc = document.clone();
            tail_doc.content = tail_track;
            intents.push(Intent::SetTextDocument { layer: tail, document: tail_doc });
        } else {
            intents.push(Intent::SetTextDocument { layer: tail, document });
        }
    }
    for (property, track) in tracks {
        let (head, tail_track) = split_track_at(&track, cut);
        if let Some(head) = head {
            intents.push(Intent::SetTrack { layer, property: property.clone(), track: head });
        }
        intents.push(Intent::SetTrack { layer: tail, property, track: tail_track });
    }

    doc.apply_all(intents).ok()?;
    Some(tail)
}

/// 切り口を跨ぐ区間のイージングを両側へ分ける(Premiere・Resolve は切っても見た目が変わらない)。
/// 跨ぐ区間が無ければ頭はそのまま(None)、尻は丸ごと写す。
pub(super) fn split_track_at(track: &KeyframeTrack, cut: RationalTime) -> (Option<KeyframeTrack>, KeyframeTrack) {
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

