
use dioxus_native::prelude::*;

use crate::doc::store::{Fps, Interp, Intent, KeyframeTrack, PropertyId, RationalTime};

use crate::ui::session::{KeySel, Session};

const FPS: f64 = 30.0;

/// 選んだキーから作る区間。同じ層・同じ属性で時刻が隣り合う2つが1区間。
/// 1つしか選んでいない時は「そのキーから次まで」を区間とみなす。
/// 区間の形は**始まりのキー**が持つ(評価がそう読む)。
pub(super) fn segments(keys: &[KeySel]) -> Vec<KeySel> {
    let mut by_track: std::collections::BTreeMap<(u64, String), Vec<f64>> = Default::default();
    for key in keys {
        let track = (
            key.layer.0,
            key.property.as_ref().map(|p| p.name().to_string()).unwrap_or_default(),
        );
        by_track.entry(track).or_default().push(key.at_sec);
    }
    let mut out = Vec::new();
    for key in keys {
        let track = (
            key.layer.0,
            key.property.as_ref().map(|p| p.name().to_string()).unwrap_or_default(),
        );
        let times = &by_track[&track];
        let is_last = times.iter().all(|t| *t <= key.at_sec);
        // 1つだけの時は最後でも区間の始まりとして扱う
        if !is_last || times.len() == 1 {
            out.push(key.clone());
        }
    }
    out
}

pub(super) fn apply(session: &Session, starts: &[KeySel], shape: Interp) -> Result<usize, String> {
    let mut doc = session.doc.lock().unwrap();
    let fps = Fps::try_new(FPS as i64, 1).map_err(|e| e.to_string())?;

    let mut intents = Vec::new();
    for sel in starts {
        let at = RationalTime::try_new((sel.at_sec * FPS).round() as i64, FPS as i64)
            .map_err(|e| e.to_string())?;
        let properties: Vec<PropertyId> = match &sel.property {
            Some(p) => vec![p.clone()],
            None => doc.view().properties(sel.layer),
        };
        for property in properties {
            let Ok(Some(track)) = doc.view().track(sel.layer, &property) else {
                continue;
            };
            let mut next = KeyframeTrack::new();
            let mut touched = false;
            for key in track.keys() {
                let mut key = key.clone();
                let same = key
                    .t
                    .try_to_frame_round(fps)
                    .ok()
                    .zip(at.try_to_frame_round(fps).ok())
                    .is_some_and(|(a, b)| a == b);
                if same {
                    key.interp = shape;
                    touched = true;
                }
                next.insert(key);
            }
            if touched {
                intents.push(Intent::SetTrack { layer: sel.layer, property, track: next });
            }
        }
    }

    let count = intents.len();
    if count == 0 {
        return Err("当てる区間が無い".to_string());
    }
    doc.apply_all(intents).map_err(|e| e.to_string())?;
    Ok(count)
}

/// 区間の始まりと終わりの時刻(秒)。終わりは次のキー。
pub(super) fn span_of(session: &Session, start: &KeySel) -> Option<(f64, f64)> {
    let doc = session.doc.lock().unwrap();
    let view = doc.view();
    let properties: Vec<PropertyId> = match &start.property {
        Some(p) => vec![p.clone()],
        None => view.properties(start.layer),
    };
    for property in properties {
        let Ok(Some(track)) = view.track(start.layer, &property) else {
            continue;
        };
        let times: Vec<f64> = track.keys().iter().map(|k| k.t.as_seconds_f64()).collect();
        let Some(here) = times.iter().position(|t| (t - start.at_sec).abs() < 1e-6) else {
            continue;
        };
        if let Some(next) = times.get(here + 1) {
            return Some((start.at_sec, *next));
        }
    }
    None
}

/// 区間が今持っている形。無ければ直線。
pub(super) fn shape_of(session: &Session, start: &KeySel) -> Interp {
    let doc = session.doc.lock().unwrap();
    let view = doc.view();
    let Ok(fps) = Fps::try_new(FPS as i64, 1) else {
        return Interp::Linear;
    };
    let properties: Vec<PropertyId> = match &start.property {
        Some(p) => vec![p.clone()],
        None => view.properties(start.layer),
    };
    let at = ((start.at_sec * FPS).round()) as i64;
    for property in properties {
        let Ok(Some(track)) = view.track(start.layer, &property) else {
            continue;
        };
        for key in track.keys() {
            if key.t.try_to_frame_round(fps).ok() != Some(at) {
                continue;
            }
            return key.interp;
        }
    }
    Interp::Linear
}

/// 文字を置かない。上が盤、下が形の棚。掴めば区間へ乗る。
pub(super) fn ease_panel(
    session: &Session,
    editor: dioxus_native::CustomWidgetAttr,
    presets: dioxus_native::CustomWidgetAttr,
    revision: Signal<u32>,
) -> Element {
    let _ = revision();
    let live = !segments(&session.selected_keys.lock().unwrap()).is_empty();
    rsx!(
        div { id: "ease",
            div { class: "ecurve", object { "data": editor } }
            div { class: if live { "epresets" } else { "epresets off" },
                object { "data": presets }
            }
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{
        Composition, Document, Keyframe, LayerId, LayerMeta, LayerSource, LayerTiming, Value,
    };

    fn key(frame: i64) -> Keyframe {
        Keyframe {
            t: RationalTime::try_new(frame, FPS as i64).unwrap(),
            value: Value::F64(frame as f64),
            interp: Interp::Linear,
            spatial: None,
        }
    }

    fn session_with_two_keys() -> (Session, PropertyId) {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(FPS as i64, 1).unwrap(),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Shape,
                    order: 0,
                    timing: LayerTiming::place(0, None, 100),
                },
            },
        ])
        .unwrap();
        let property = PropertyId::new("opacity").unwrap();
        let mut track = KeyframeTrack::new();
        track.insert(key(0));
        track.insert(key(30));
        doc.apply(Intent::SetTrack { layer, property: property.clone(), track }).unwrap();
        let ui = crate::ui::fixture::load_fixture().ui;
        (Session::new(doc, 10.0, ui), property)
    }

    fn sel(property: &PropertyId, at_sec: f64) -> KeySel {
        KeySel { layer: LayerId(1), property: Some(property.clone()), at_sec }
    }

    #[test]
    fn two_keys_make_one_segment_and_the_later_one_is_not_a_start() {
        let property = PropertyId::new("opacity").unwrap();
        let starts = segments(&[sel(&property, 0.0), sel(&property, 1.0)]);
        assert_eq!(starts.len(), 1, "2つ選んだら区間は1つ");
        assert_eq!(starts[0].at_sec, 0.0, "区間の形は始まりのキーが持つ");
    }

    #[test]
    fn one_key_alone_is_treated_as_the_start_of_its_segment() {
        let property = PropertyId::new("opacity").unwrap();
        assert_eq!(segments(&[sel(&property, 0.0)]).len(), 1);
    }

    #[test]
    fn applying_shapes_only_the_starting_key_of_the_segment() {
        let (session, property) = session_with_two_keys();
        let starts = segments(&[sel(&property, 0.0), sel(&property, 1.0)]);
        apply(
            &session,
            &starts,
            Interp::Bezier { x1: 0.42, y1: 0.0, x2: 0.58, y2: 1.0 },
        )
        .expect("当たるはず");

        let doc = session.doc.lock().unwrap();
        let track = doc.view().track(LayerId(1), &property).unwrap().unwrap();
        let keys = track.keys();
        assert!(
            matches!(keys[0].interp, Interp::Bezier { x1, y1, x2, y2 }
                if (x1 - 0.42).abs() < 1e-9 && y1 == 0.0 && (x2 - 0.58).abs() < 1e-9 && y2 == 1.0),
            "始まりのキーに曲線が乗っていない: {:?}",
            keys[0].interp
        );
        assert!(
            matches!(keys[1].interp, Interp::Linear),
            "区間の外まで書き換えている: {:?}",
            keys[1].interp
        );
    }
}
