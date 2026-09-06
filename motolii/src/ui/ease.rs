use dioxus_native::prelude::*;

use crate::doc::store::{Intent, Interp, KeyframeTrack, PropertyId};

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
    apply_at(session, starts, EaseEdit::Preset(shape))
}

/// AE の F9 一族。Easy Ease In(⇧F9)は**キーへ入る側** = 前の区間の終わりを寝かせる。
/// Easy Ease Out(⌘F9)はキーから出る側 = この区間の始まり。形はキーが持つので、In は前のキーへ書く。
pub(super) fn apply_easy(
    session: &Session,
    starts: &[KeySel],
    side: crate::ui::keymap::EaseSide,
) -> Result<usize, String> {
    apply_at(session, starts, EaseEdit::Keys(side))
}

#[derive(Clone, Copy)]
enum EaseEdit {
    Preset(Interp),
    Keys(crate::ui::keymap::EaseSide),
}

fn apply_at(
    session: &Session,
    starts: &[KeySel],
    edit: EaseEdit,
) -> Result<usize, String> {
    let mut doc = session.doc.lock().unwrap();
    let fps = doc
        .view()
        .composition()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Composition has no frame rate".to_owned())?
        .fps;

    let mut selected: std::collections::BTreeMap<
        (crate::doc::store::LayerId, PropertyId),
        std::collections::BTreeSet<i64>,
    > = Default::default();
    for sel in starts {
        let frame = (sel.at_sec * fps.as_f64()).round() as i64;
        let properties = match &sel.property {
            Some(p) => vec![p.clone()],
            None => doc.view().properties(sel.layer),
        };
        for property in properties {
            selected.entry((sel.layer, property)).or_default().insert(frame);
        }
    }
    let mut intents = Vec::new();
    for ((layer, property), frames) in selected {
        let Some(track) = doc.view().track(layer, &property).map_err(|e| e.to_string())? else {
            continue;
        };
        let mut targets: std::collections::BTreeMap<usize, u8> = Default::default();
        for (index, key) in track.keys().iter().enumerate() {
            let frame = key.t.try_to_frame_round(fps).map_err(|e| e.to_string())?;
            if !frames.contains(&frame) {
                continue;
            }
            match edit {
                EaseEdit::Preset(_) => { targets.insert(index, 3); }
                EaseEdit::Keys(side) => {
                    use crate::ui::keymap::EaseSide;
                    if side != EaseSide::Out {
                        if let Some(previous) = index.checked_sub(1) {
                            *targets.entry(previous).or_default() |= 2;
                        }
                    }
                    if side != EaseSide::In && index + 1 < track.keys().len() {
                        *targets.entry(index).or_default() |= 1;
                    }
                }
            }
        }
        if targets.is_empty() {
            continue;
        }
        let mut next = KeyframeTrack::new();
        for (index, key) in track.keys().iter().enumerate() {
            let mut key = key.clone();
            if let Some(&endpoints) = targets.get(&index) {
                key.interp = match edit {
                    EaseEdit::Preset(shape) => shape,
                    EaseEdit::Keys(_) => ease_endpoints(key.interp, endpoints),
                };
            }
            next.insert(key);
        }
        intents.push(Intent::SetTrack { layer, property, track: next });
    }

    let count = intents.len();
    if count == 0 {
        return Err("当てる区間が無い".to_string());
    }
    doc.apply_all(intents).map_err(|e| e.to_string())?;
    Ok(count)
}

fn ease_endpoints(shape: Interp, endpoints: u8) -> Interp {
    let (mut x1, mut y1, mut x2, mut y2) = match shape {
        Interp::Bezier { x1, y1, x2, y2 } => (x1, y1, x2, y2),
        _ => (0.0, 0.0, 1.0, 1.0),
    };
    if endpoints & 1 != 0 {
        x1 = 1.0 / 3.0;
        y1 = 0.0;
    }
    if endpoints & 2 != 0 {
        x2 = 2.0 / 3.0;
        y2 = 1.0;
    }
    Interp::Bezier { x1, y1, x2, y2 }
}

#[cfg(test)]
fn easy_ease(side: crate::ui::keymap::EaseSide) -> Interp {
    use crate::ui::keymap::EaseSide;
    match side {
        EaseSide::Both => Interp::Bezier {
            x1: 1.0 / 3.0,
            y1: 0.0,
            x2: 2.0 / 3.0,
            y2: 1.0,
        },
        // In は前の区間の**終わり**を寝かせる(x2 側)。Out はこの区間の始まり(x1 側)。
        EaseSide::In => Interp::Bezier {
            x1: 0.0,
            y1: 0.0,
            x2: 2.0 / 3.0,
            y2: 1.0,
        },
        EaseSide::Out => Interp::Bezier {
            x1: 1.0 / 3.0,
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
    shape: std::sync::Arc<std::sync::Mutex<Interp>>,
    preset_focus: crate::ui::ease_widget::PresetFocus,
    mut revision: Signal<u32>,
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
            div {
                class: if live { "epresets" } else { "epresets off" },
                tabindex: if live { "0" } else { "-1" },
                role: "grid",
                aria_label: "Easing presets",
                onkeydown: {
                    let session = session.clone();
                    move |evt: KeyboardEvent| {
                        if !live {
                            return;
                        }
                        let moved = crate::ui::ease_widget::move_preset_focus(
                            &session,
                            &preset_focus,
                            &evt.key(),
                        );
                        let activate = matches!(evt.key(), Key::Enter)
                            || matches!(evt.key(), Key::Character(c) if c == " ");
                        if !moved && !activate {
                            return;
                        }
                        evt.prevent_default();
                        evt.stop_propagation();
                        if activate {
                            crate::ui::ease_widget::apply_focused_preset(
                                &session,
                                &shape,
                                &preset_focus,
                            );
                        }
                        *revision.write() += 1;
                    }
                },
                object { "data": presets }
            }
            div { class: "ename", "{name}" }
        }
    )
}

#[cfg(test)]
mod easy {
    use super::*;
    use crate::doc::store::{property, Keyframe, RationalTime, Value};
    use crate::ui::keymap::EaseSide;

    fn keyed_session() -> (
        Session,
        crate::doc::store::LayerId,
        PropertyId,
        crate::doc::store::Fps,
    ) {
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
        doc.apply(Intent::SetTrack {
            layer,
            property: prop.clone(),
            track,
        })
        .unwrap();
        (
            Session::new(doc, loaded.duration_sec, loaded.ui),
            layer,
            prop,
            fps,
        )
    }

    #[test]
    fn selected_intervals_on_one_track_ease_together_and_undo_together() {
        let (session, layer, prop, fps) = keyed_session();
        let before = {
            let mut doc = session.doc.lock().unwrap();
            let mut track = doc.view().track(layer, &prop).unwrap().unwrap();
            let mut key = track.keys()[1].clone();
            key.spatial = Some(crate::doc::store::SpatialTangent {
                out_tangent: [4.0, 8.0],
                in_tangent: [-3.0, -2.0],
            });
            track.insert(key);
            doc.apply(Intent::SetTrack { layer, property: prop.clone(), track: track.clone() }).unwrap();
            track
        };
        let selection: Vec<_> = [0, 24, 48].into_iter().map(|frame| KeySel {
            layer,
            property: Some(prop.clone()),
            at_sec: frame as f64 / fps.as_f64(),
        }).collect();
        let mut starts = segments(&selection);
        starts.push(selection[0].clone());
        let shape = easy_ease(EaseSide::Both);
        assert_eq!(apply(&session, &starts, shape).unwrap(), 1);
        let mut doc = session.doc.lock().unwrap();
        let after = doc.view().track(layer, &prop).unwrap().unwrap();
        let mut expected = before.clone();
        for old in &before.keys()[..2] {
            let mut key = old.clone();
            key.interp = shape;
            expected.insert(key);
        }
        assert_eq!(after, expected);
        assert!(doc.undo());
        assert_eq!(doc.view().track(layer, &prop).unwrap().unwrap(), before);
        assert!(doc.redo());
        assert_eq!(doc.view().track(layer, &prop).unwrap().unwrap(), after);
    }

    #[test]
    fn grouped_ease_rejection_preserves_tracks_and_the_redo_branch() {
        let (session, layer, prop, fps) = keyed_session();
        let starts: Vec<_> = [0, 24].into_iter().map(|frame| KeySel {
            layer,
            property: Some(prop.clone()),
            at_sec: frame as f64 / fps.as_f64(),
        }).collect();
        let before = session.doc.lock().unwrap().view().track(layer, &prop).unwrap().unwrap();
        apply(&session, &starts, Interp::Hold).unwrap();
        assert!(session.doc.lock().unwrap().undo());
        let invalid = Interp::Bezier { x1: -1.0, y1: 0.0, x2: 1.0, y2: 1.0 };
        assert!(apply(&session, &starts, invalid).is_err());
        {
            let mut doc = session.doc.lock().unwrap();
            assert_eq!(doc.view().track(layer, &prop).unwrap().unwrap(), before);
            assert!(doc.redo());
            assert!(doc.undo());
            doc.apply(Intent::SetAttrs {
                layer,
                patch: crate::doc::store::LayerAttrsPatch { locked: Some(true), ..Default::default() },
            }).unwrap();
        }
        assert!(apply(&session, &starts, Interp::Hold).is_err());
        let mut doc = session.doc.lock().unwrap();
        assert_eq!(doc.view().track(layer, &prop).unwrap().unwrap(), before);
        assert!(doc.undo());
        assert!(!doc.view().attrs(layer).unwrap().unwrap().locked);
    }

    // Oracle: https://helpx.adobe.com/my_en/after-effects/desktop/animate-in-after-effects/speed-between-keyframes/speed.html
    #[test]
    fn easy_ease_targets_selected_key_endpoints_and_preserves_neighbor_handles() {
        let (session, layer, prop, fps) = keyed_session();
        let original = Interp::Bezier { x1: 0.2, y1: 0.4, x2: 0.8, y2: 0.6 };
        let before = {
            let mut doc = session.doc.lock().unwrap();
            let mut track = doc.view().track(layer, &prop).unwrap().unwrap();
            for old in track.keys().to_vec() {
                track.insert(Keyframe { interp: original, ..old });
            }
            doc.apply(Intent::SetTrack { layer, property: prop.clone(), track: track.clone() }).unwrap();
            track
        };
        let selected = |frame| KeySel {
            layer, property: Some(prop.clone()), at_sec: frame as f64 / fps.as_f64(),
        };
        apply_easy(&session, &[selected(24)], EaseSide::Both).unwrap();
        {
            let mut doc = session.doc.lock().unwrap();
            let after = doc.view().track(layer, &prop).unwrap().unwrap();
            assert_eq!(after.keys()[0].interp, Interp::Bezier { x1: 0.2, y1: 0.4, x2: 2.0 / 3.0, y2: 1.0 });
            assert_eq!(after.keys()[1].interp, Interp::Bezier { x1: 1.0 / 3.0, y1: 0.0, x2: 0.8, y2: 0.6 });
            assert_eq!(after.keys()[2], before.keys()[2]);
            assert!(doc.undo());
            assert_eq!(doc.view().track(layer, &prop).unwrap().unwrap(), before);
        }
        apply_easy(&session, &[selected(0), selected(24)], EaseSide::In).unwrap();
        {
            let mut doc = session.doc.lock().unwrap();
            let after = doc.view().track(layer, &prop).unwrap().unwrap();
            assert_eq!(after.keys()[0].interp, Interp::Bezier { x1: 0.2, y1: 0.4, x2: 2.0 / 3.0, y2: 1.0 });
            assert_eq!(after.keys()[1..], before.keys()[1..]);
            assert!(doc.undo());
        }
        apply_easy(&session, &[selected(0), selected(24)], EaseSide::Both).unwrap();
        let mut doc = session.doc.lock().unwrap();
        let after = doc.view().track(layer, &prop).unwrap().unwrap();
        assert_eq!(after.keys()[0].interp, easy_ease(EaseSide::Both));
        assert_eq!(after.keys()[1].interp, Interp::Bezier { x1: 1.0 / 3.0, y1: 0.0, x2: 0.8, y2: 0.6 });
        assert!(doc.undo());
        assert_eq!(doc.view().track(layer, &prop).unwrap().unwrap(), before);
        assert!(doc.redo());
        assert_eq!(doc.view().track(layer, &prop).unwrap().unwrap(), after);
    }

    #[test]
    fn easy_ease_converts_non_bezier_interpolation_and_undo_restores_it() {
        for source in [Interp::Hold, Interp::Bounce { first_dip: 0.27, dip: 0.2 }] {
            let (session, layer, prop, fps) = keyed_session();
            let selected = |frame| KeySel {
                layer, property: Some(prop.clone()), at_sec: frame as f64 / fps.as_f64(),
            };
            apply(&session, &[selected(0)], source).unwrap();
            let before = session.doc.lock().unwrap().view().track(layer, &prop).unwrap().unwrap();
            apply_easy(&session, &[selected(24)], EaseSide::Both).unwrap();
            let mut doc = session.doc.lock().unwrap();
            let after = doc.view().track(layer, &prop).unwrap().unwrap();
            // Non-Bezier intervals convert to Bezier with a neutral opposite handle.
            assert_eq!(after.keys()[0].interp, easy_ease(EaseSide::In));
            assert_eq!(after.keys()[1].interp, easy_ease(EaseSide::Out));
            for (old, new) in before.keys().iter().zip(after.keys()) {
                assert_eq!(old.t, new.t);
                assert_eq!(old.value, new.value);
                assert_eq!(old.spatial, new.spatial);
            }
            assert_eq!(after.keys()[2], before.keys()[2]);
            assert!(doc.undo());
            assert_eq!(doc.view().track(layer, &prop).unwrap().unwrap(), before);
            assert!(doc.redo());
            assert_eq!(doc.view().track(layer, &prop).unwrap().unwrap(), after);
        }
    }

    /// ⇧F9 は選んだキーへ**入る**区間(前のキーが持つ形)を寝かせ、⌘F9 は出る区間を寝かせる。
    #[test]
    fn ease_in_lands_on_the_previous_segment_and_out_on_this_one() {
        let (session, layer, prop, fps) = keyed_session();
        let middle = KeySel {
            layer,
            property: Some(prop.clone()),
            at_sec: 24.0 / fps.as_f64(),
        };
        apply_easy(&session, &[middle.clone()], EaseSide::In).unwrap();
        let keys = session
            .doc
            .lock()
            .unwrap()
            .view()
            .track(layer, &prop)
            .unwrap()
            .unwrap()
            .keys()
            .to_vec();
        assert_eq!(
            keys[0].interp,
            easy_ease(EaseSide::In),
            "In must shape the segment before the key"
        );
        assert_eq!(keys[1].interp, Interp::Linear);

        apply_easy(&session, &[middle], EaseSide::Out).unwrap();
        let keys = session
            .doc
            .lock()
            .unwrap()
            .view()
            .track(layer, &prop)
            .unwrap()
            .unwrap()
            .keys()
            .to_vec();
        assert_eq!(
            keys[1].interp,
            easy_ease(EaseSide::Out),
            "Out must shape the segment after the key"
        );
    }
}
