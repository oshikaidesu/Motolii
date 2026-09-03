use dioxus_native::prelude::*;

use crate::doc::store::{Intent, Interp, KeyframeTrack, PropertyId, RationalTime};

use crate::ui::session::{KeySel, Session};

/// 選んだキーから作る区間。同じ層・同じ属性で時刻が隣り合う2つが1区間。
/// 1つしか選んでいない時は「そのキーから次まで」を区間とみなす。
/// 区間の形は**始まりのキー**が持つ(評価がそう読む)。
pub(super) fn segments(keys: &[KeySel]) -> Vec<KeySel> {
    let mut by_track: std::collections::BTreeMap<(u64, String), Vec<f64>> = Default::default();
    for key in keys {
        let track = (
            key.layer.0,
            key.property
                .as_ref()
                .map(|p| p.name().to_string())
                .unwrap_or_default(),
        );
        by_track.entry(track).or_default().push(key.at_sec);
    }
    let mut out = Vec::new();
    for key in keys {
        let track = (
            key.layer.0,
            key.property
                .as_ref()
                .map(|p| p.name().to_string())
                .unwrap_or_default(),
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
    apply_at(session, starts, shape, false)
}

/// AE の F9 一族。Easy Ease In(⇧F9)は**キーへ入る側** = 前の区間の終わりを寝かせる。
/// Easy Ease Out(⌘F9)はキーから出る側 = この区間の始まり。形はキーが持つので、In は前のキーへ書く。
pub(super) fn apply_easy(session: &Session, starts: &[KeySel], side: crate::ui::keymap::EaseSide) -> Result<usize, String> {
    let previous = side == crate::ui::keymap::EaseSide::In;
    apply_at(session, starts, easy_ease(side), previous)
}

fn apply_at(session: &Session, starts: &[KeySel], shape: Interp, previous: bool) -> Result<usize, String> {
    let mut doc = session.doc.lock().unwrap();
    let fps = doc
        .view()
        .composition()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Composition has no frame rate".to_owned())?
        .fps;

    let mut intents = Vec::new();
    for sel in starts {
        let at = RationalTime::try_from_frame((sel.at_sec * fps.as_f64()).round() as i64, fps)
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
            let keys = track.keys();
            let hit = keys.iter().position(|key| {
                key.t
                    .try_to_frame_round(fps)
                    .ok()
                    .zip(at.try_to_frame_round(fps).ok())
                    .is_some_and(|(a, b)| a == b)
            });
            let target = match (hit, previous) {
                (Some(i), true) => i.checked_sub(1),
                (hit, false) => hit,
                (None, true) => None,
            };
            for (i, key) in keys.iter().enumerate() {
                let mut key = key.clone();
                if Some(i) == target {
                    key.interp = shape;
                    touched = true;
                }
                next.insert(key);
            }
            if touched {
                intents.push(Intent::SetTrack {
                    layer: sel.layer,
                    property,
                    track: next,
                });
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

/// AE の F9 一族。両側 / 入り / 出 を寝かせる。
pub(super) fn easy_ease(side: crate::ui::keymap::EaseSide) -> Interp {
    use crate::ui::keymap::EaseSide;
    match side {
        EaseSide::Both => Interp::Bezier {
            x1: 0.33,
            y1: 0.0,
            x2: 0.67,
            y2: 1.0,
        },
        // In は前の区間の**終わり**を寝かせる(x2 側)。Out はこの区間の始まり(x1 側)。
        EaseSide::In => Interp::Bezier {
            x1: 0.0,
            y1: 0.0,
            x2: 0.67,
            y2: 1.0,
        },
        EaseSide::Out => Interp::Bezier {
            x1: 0.33,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        },
    }
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
    let Ok(Some(composition)) = view.composition() else {
        return Interp::Linear;
    };
    let fps = composition.fps;
    let properties: Vec<PropertyId> = match &start.property {
        Some(p) => vec![p.clone()],
        None => view.properties(start.layer),
    };
    let at = (start.at_sec * fps.as_f64()).round() as i64;
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
    kind_name: Signal<String>,
    revision: Signal<u32>,
) -> Element {
    let _ = revision();
    let chosen = segments(&session.selected_keys.lock().unwrap());
    let live = !chosen.is_empty();
    // **今なにを編集しているか**を書く。選べていなくても盤は普通に曲線を描き、
    // 形を押しても叱られないので、値を1コマずつ読むまで空振りに気づけない。
    let target = if live {
        let doc = session.doc.lock().unwrap();
        let view = doc.view();
        let fps = view
            .composition()
            .ok()
            .flatten()
            .map(|composition| composition.fps)
            .unwrap_or_else(|| crate::doc::store::Fps::try_new(30, 1).expect("30fps"));
        let layer = chosen[0].layer;
        let name = view
            .attrs(layer)
            .ok()
            .flatten()
            .map(|a| a.name)
            .unwrap_or_default();
        let property = chosen[0]
            .property
            .as_ref()
            .map(|p| p.name().to_string())
            .unwrap_or_else(|| "all".to_string());
        let mut at: Vec<f64> = chosen.iter().map(|k| k.at_sec).collect();
        at.sort_by(f64::total_cmp);
        let (a, b) = (at[0], *at.last().expect("空でない"));
        format!(
            "{name} · {property} · {}–{}",
            crate::ui::fixture::fmt_timecode(a, fps),
            crate::ui::fixture::fmt_timecode(b, fps)
        )
    } else {
        "Pick a key on the timeline".to_string()
    };
    // 説明書は形を名前で指す(「弾性」)。棚は形しか描けないので、
    // **指している間だけ**名前を返す。常に置くと8つの字が形を覆う。
    let name = kind_name();
    rsx!(
        div { id: if live { "ease" } else { "ease idle" },
            div { class: "etarget", "{target}" }
            div { class: "ecurve", object { "data": editor } }
            div { class: if live { "epresets" } else { "epresets off" },
                object { "data": presets }
            }
            div { class: "ename", "{name}" }
        }
    )
}

#[cfg(test)]
mod easy {
    use super::*;
    use crate::doc::store::{property, Keyframe, Value};
    use crate::ui::keymap::EaseSide;

    fn keyed_session() -> (Session, crate::doc::store::LayerId, PropertyId, crate::doc::store::Fps) {
        let loaded = crate::ui::fixture::load_fixture();
        let mut doc = loaded.doc;
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let layer = doc.view().layers()[0];
        let prop = PropertyId::new(property::OPACITY).unwrap();
        let mut track = KeyframeTrack::new();
        for frame in [0, 24, 48] {
            track.insert(Keyframe {
                t: RationalTime::try_from_frame(frame, fps).unwrap(),
                value: Value::F64(frame as f64),
                interp: Interp::Linear,
                spatial: None,
            });
        }
        doc.apply(Intent::SetTrack { layer, property: prop.clone(), track }).unwrap();
        (Session::new(doc, loaded.duration_sec, loaded.ui), layer, prop, fps)
    }

    /// ⇧F9 は選んだキーへ**入る**区間(前のキーが持つ形)を寝かせ、⌘F9 は出る区間を寝かせる。
    #[test]
    fn ease_in_lands_on_the_previous_segment_and_out_on_this_one() {
        let (session, layer, prop, fps) = keyed_session();
        let middle = KeySel { layer, property: Some(prop.clone()), at_sec: 24.0 / fps.as_f64() };
        apply_easy(&session, &[middle.clone()], EaseSide::In).unwrap();
        let keys = session.doc.lock().unwrap().view().track(layer, &prop).unwrap().unwrap().keys().to_vec();
        assert_eq!(keys[0].interp, easy_ease(EaseSide::In), "In must shape the segment before the key");
        assert_eq!(keys[1].interp, Interp::Linear);

        apply_easy(&session, &[middle], EaseSide::Out).unwrap();
        let keys = session.doc.lock().unwrap().view().track(layer, &prop).unwrap().unwrap().keys().to_vec();
        assert_eq!(keys[1].interp, easy_ease(EaseSide::Out), "Out must shape the segment after the key");
    }
}
