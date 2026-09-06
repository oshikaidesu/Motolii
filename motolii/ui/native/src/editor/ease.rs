use crate::doc::store::{Intent, Interp, KeyframeTrack, PropertyId};
use crate::editor::session::KeySel;
/// 選んだキーから作る区間。同じ層・同じ属性で時刻が隣り合う2つが1区間。
/// 1つしか選んでいない時は「そのキーから次まで」を区間とみなす。
/// 区間の形は**始まりのキー**が持つ(評価がそう読む)。
pub(crate) fn segments(keys: &[KeySel]) -> Vec<KeySel> {
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

pub(crate) fn apply(doc: &mut crate::doc::store::Document, starts: &[KeySel], shape: Interp) -> Result<usize, String> {
    apply_at(doc, starts, EaseEdit::Preset(shape))
}

/// AE の F9 一族。Easy Ease In(⇧F9)は**キーへ入る側** = 前の区間の終わりを寝かせる。
/// Easy Ease Out(⌘F9)はキーから出る側 = この区間の始まり。形はキーが持つので、In は前のキーへ書く。
pub(crate) fn apply_easy(
    doc: &mut crate::doc::store::Document,
    starts: &[KeySel],
    side: crate::editor::keymap::EaseSide,
) -> Result<usize, String> {
    apply_at(doc, starts, EaseEdit::Keys(side))
}

#[derive(Clone, Copy)]
enum EaseEdit {
    Preset(Interp),
    Keys(crate::editor::keymap::EaseSide),
}

fn apply_at(
    doc: &mut crate::doc::store::Document,
    starts: &[KeySel],
    edit: EaseEdit,
) -> Result<usize, String> {
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
                EaseEdit::Preset(_) => { if index + 1 < track.keys().len() { targets.insert(index, 3); } }
                EaseEdit::Keys(side) => {
                    use crate::editor::keymap::EaseSide;
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
mod tests {
    use super::*;
    use crate::doc::store::*;
    #[test]
    fn legacy_interval_application_is_one_undo_and_has_no_terminal_interval() {
        let mut doc = crate::doc::store::blank_project();
        let layer = LayerId(1);
        let fps = Fps::try_new(30, 1).unwrap();
        doc.apply_all(crate::editor::create::new_layer_intents(layer, 0, 0, 60, fps, (1920.0,1080.0), crate::editor::create::NewKind::Rectangle)).unwrap();
        let property = PropertyId::new(property::OPACITY).unwrap();
        let mut track = KeyframeTrack::new();
        for frame in [0, 24, 48] { track.insert(Keyframe {t:RationalTime::try_from_frame(frame,fps).unwrap(),value:Value::F64(frame as f64/48.0),interp:Interp::Linear,spatial:None}); }
        doc.apply(Intent::SetTrack{layer,property:property.clone(),track:track.clone()}).unwrap();
        let selected:Vec<_> = [0,24,48].into_iter().map(|frame|KeySel{layer,property:Some(property.clone()),at_sec:frame as f64/30.0}).collect();
        apply(&mut doc,&segments(&selected),Interp::Hold).unwrap();
        let actual=doc.view().track(layer,&property).unwrap().unwrap();
        assert_eq!(actual.keys().iter().map(|k|k.interp).collect::<Vec<_>>(),vec![Interp::Hold,Interp::Hold,Interp::Linear]);
        assert!(doc.undo());
        assert_eq!(doc.view().track(layer,&property).unwrap().unwrap(),track);
        assert!(apply(&mut doc,&selected[2..],Interp::Hold).is_err());
        assert!(doc.redo());
        assert_eq!(doc.view().track(layer,&property).unwrap().unwrap(),actual);
    }
}
