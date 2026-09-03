use std::rc::Rc;
use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;

use crate::ui::fixture::{inspector_data_from_doc, InspectorData, PropRow};
use crate::ui::playback::Clock;
use crate::ui::semantic_menu::{Field, SemanticButton};
use crate::ui::session::{FieldAt, Focus, OpenField, Session};
use crate::ui::fixture::ColorRow;
use crate::doc::store::{
    property, BlendMode, ContentKeyframe, Document, Intent, Interp, Keyframe,
    LayerAttrsPatch, LayerId, LayerSource, Matte, MatteMode, PropertyId, RationalTime, StoreError,
    StoreView, Value,
};

/// 窓に並べる合成モード。**W3C Compositing の16 mix + `plus`(Add)**で、
/// 順番は語彙の並び(reference/compositing-coverage.tsv と同じ)。
const ANCHOR_SPOTS: [(f64, f64); 9] = [
    (0.0, 0.0), (0.5, 0.0), (1.0, 0.0),
    (0.0, 0.5), (0.5, 0.5), (1.0, 0.5),
    (0.0, 1.0), (0.5, 1.0), (1.0, 1.0),
];

pub(super) const BLEND_MODES: &[(BlendMode, &str)] = &[
    (BlendMode::Normal, "Normal"),
    (BlendMode::Add, "Add"),
    (BlendMode::Multiply, "Multiply"),
    (BlendMode::Screen, "Screen"),
    (BlendMode::Overlay, "Overlay"),
    (BlendMode::Darken, "Darken"),
    (BlendMode::Lighten, "Lighten"),
    (BlendMode::ColorDodge, "Color Dodge"),
    (BlendMode::ColorBurn, "Color Burn"),
    (BlendMode::HardLight, "Hard Light"),
    (BlendMode::SoftLight, "Soft Light"),
    (BlendMode::Difference, "Difference"),
    (BlendMode::Exclusion, "Exclusion"),
    (BlendMode::Hue, "Hue"),
    (BlendMode::Saturation, "Saturation"),
    (BlendMode::Color, "Color"),
    (BlendMode::Luminosity, "Luminosity"),
];

fn blend_label(mode: BlendMode) -> &'static str {
    BLEND_MODES
        .iter()
        .find(|(m, _)| *m == mode)
        .map(|(_, label)| *label)
        .unwrap_or("Normal")
}

pub(super) fn write_blend(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    mode: BlendMode,
) -> Result<(), crate::doc::store::StoreError> {
    let patch = LayerAttrsPatch {
        blend_mode: Some(mode),
        ..Default::default()
    };
    doc.lock().unwrap().apply(Intent::SetAttrs { layer, patch })
}

const RANGE_SPAN_PX: f64 = 300.0;

fn increment(property: &str, range: Option<(f64, f64)>) -> f64 {
    if let Some((min, max)) = range {
        return (max - min) / RANGE_SPAN_PX;
    }
    match property {
        p if p == property::SCALE => 0.005,
        p if p == property::ROTATION => 0.5,
        p if p == property::OPACITY => 0.005,
        _ => 1.0, // position / anchor
    }
}

fn nudge(value: &Value, vec2: bool, axis: usize, delta: f64, range: Option<(f64, f64)>) -> Value {
    match (vec2, value) {
        (true, Value::Vec2([x, y])) => {
            let mut v = [*x, *y];
            v[axis] += delta;
            Value::Vec2(v)
        }
        (false, Value::F64(v)) => {
            let v = v + delta;
            Value::F64(match range {
                Some((min, max)) => v.clamp(min, max),
                None => v,
            })
        }
        _ => value.clone(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ValueDrag {
    layer: LayerId,
    /// 一緒に選んでいる層と、それぞれの掴んだ時の値。同じ差分で動く(AE の複数選択)。
    others: Vec<(LayerId, Value)>,
    property: String,
    vec2: bool,
    axis: usize,
    start_x: f64,
    start_value: Value,
    range: Option<(f64, f64)>,
    last_dx: f64,
}

fn put_axis(value: &Value, vec2: bool, axis: usize, v: f64, range: Option<(f64, f64)>) -> Value {
    match (vec2, value) {
        (true, Value::Vec2([x, y])) => {
            let mut a = [*x, *y];
            a[axis] = v;
            Value::Vec2(a)
        }
        (false, Value::F64(_)) => Value::F64(match range {
            Some((min, max)) => v.clamp(min, max),
            None => v,
        }),
        _ => value.clone(),
    }
}

fn write_key(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    property: &str,
    value: Value,
    t: RationalTime,
) -> Result<(), crate::doc::store::StoreError> {
    let Ok(prop) = PropertyId::new(property) else {
        return Ok(());
    };
    let mut doc = doc.lock().unwrap();
    let intent = doc.place(layer, &prop, value, t);
    doc.apply(intent)
}

/// 時間の世界を開ける。**今の時刻に1つだけ**キーを立てる。
/// `write_key` は「既に開いている物へ打つ」道なので、開ける時はこちら。
fn open_time(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    property: &str,
    value: Value,
    t: RationalTime,
) -> Result<(), crate::doc::store::StoreError> {
    let Ok(prop) = PropertyId::new(property) else {
        return Ok(());
    };
    let mut track = crate::doc::store::KeyframeTrack::new();
    track.insert(Keyframe { t, value, interp: Interp::Linear, spatial: None });
    doc.lock()
        .unwrap()
        .apply(Intent::SetTrack { layer, property: prop, track })
}

/// 今の時刻にキーが在れば外し、無ければ今の値で 1 つ立てる。
fn toggle_key_at(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    property: &str,
    value: Value,
    t: RationalTime,
) -> Result<(), crate::doc::store::StoreError> {
    let Ok(prop) = PropertyId::new(property) else {
        return Ok(());
    };
    let mut d = doc.lock().unwrap();
    let here = d
        .view()
        .track(layer, &prop)?
        .is_some_and(|track| track.keys().iter().any(|k| k.t == t));
    let intents = if here {
        crate::ui::timeline_widget::keyframe_delete_intents(&d, layer, Some(&prop), t.as_seconds_f64())?
    } else {
        vec![d.place(layer, &prop, value, t)]
    };
    d.apply_all(intents).map(|_| ())
}

/// 時間の世界を閉じる。キーを捨てて、**今見えている値だけ**を残す。
fn close_time(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    property: &str,
    value: Value,
) -> Result<(), crate::doc::store::StoreError> {
    let Ok(prop) = PropertyId::new(property) else {
        return Ok(());
    };
    doc.lock()
        .unwrap()
        .apply(Intent::SetConstant { layer, property: prop, value })
}

/// エフェクトを層から外す。**param は触れるのに本体を外せない**という
/// Q0b 違反(触れる物は全て編集可能)を閉じる。
fn remove_effect(doc: &Arc<Mutex<Document>>, layer: LayerId, id: u32) {
    let mut d = doc.lock().unwrap();
    let mut effects = d.view().effects(layer).unwrap_or_default();
    effects.retain(|e| e.id.0 != id);
    match d.apply(Intent::SetEffects { layer, effects }) {
        Ok(_) => println!("PROBE room=write verdict=applied RemoveEffect layer={layer:?} id={id}"),
        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
    }
}

fn commit_drag(doc: &Arc<Mutex<Document>>, d: &ValueDrag, t: RationalTime) {
    if let Ok(prop) = PropertyId::new(&d.property) {
        doc.lock().unwrap().clear_transient(d.layer, &prop);
    }
    if d.last_dx == 0.0 {
        return;
    }
    let delta = d.last_dx * increment(&d.property, d.range);
    let targets = std::iter::once((d.layer, d.start_value.clone())).chain(d.others.iter().cloned());
    for (layer, start) in targets {
        if let Ok(prop) = PropertyId::new(&d.property) {
            doc.lock().unwrap().clear_transient(layer, &prop);
        }
        let new_value = nudge(&start, d.vec2, d.axis, delta, d.range);
        if let Err(e) = write_key(doc, layer, &d.property, new_value, t) {
            println!("PROBE room=write verdict=apply-error {e}");
        }
    }
}

/// 擦りを終える。窓の外で放した時も同じ道(host の release から)。
pub(super) fn end_scrub(session: &Session) -> bool {
    let Some(d) = session.scrub.lock().unwrap().take() else { return false };
    commit_drag(&session.doc, &d, session.clock.current_time());
    true
}

/// 擦りを取り消す。値は掴む前へ戻り、Undo には何も残らない。
pub(super) fn cancel_scrub(session: &Session) -> bool {
    let Some(d) = session.scrub.lock().unwrap().take() else { return false };
    if let Ok(prop) = PropertyId::new(&d.property) {
        let mut doc = session.doc.lock().unwrap();
        doc.clear_transient(d.layer, &prop);
        for (layer, _) in &d.others {
            doc.clear_transient(*layer, &prop);
        }
    }
    true
}

/// 選んでいる他の層の、同じ property の今の値。複数選択で一緒に動かす為。
fn others_at(session: &Session, primary: LayerId, property: &str, t: RationalTime) -> Vec<(LayerId, Value)> {
    let Ok(prop) = PropertyId::new(property) else { return Vec::new() };
    // 錠の判定は doc の lock を取る。ここで lock を握る前に済ませる(同じ lock は二度取れない)。
    let editable = session.editable_selection();
    let doc = session.doc.lock().unwrap();
    let view = doc.view();
    editable
        .into_iter()
        .filter(|l| *l != primary)
        .filter_map(|l| value_with_default(&view, l, &prop, property, t).map(|v| (l, v)))
        .collect()
}

/// 明示の値が無い property は既定値。Inspector の行と同じ物を見る。
pub(super) fn value_with_default(
    view: &StoreView<'_>,
    layer: LayerId,
    prop: &PropertyId,
    property: &str,
    t: RationalTime,
) -> Option<Value> {
    if let Ok(Some(v)) = view.value_at(layer, prop, t) {
        return Some(v);
    }
    crate::ui::fixture::inspector_data_from_doc(view, layer, t)
        .transform
        .into_iter()
        .find(|row| row.property.as_deref() == Some(property))
        .map(|row| row.value)
}

pub(super) fn write_content(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    t: RationalTime,
    content: String,
) -> Result<(), crate::doc::store::StoreError> {
    let mut doc = doc.lock().unwrap();
    let Some(mut document) = doc.view().text_document(layer)? else {
        return Ok(());
    };
    // 文字は時間を開けていない限り 1 つ。キーが 1 つ以下なら差し替え、2 つ以上なら今の時刻に足す。
    let keys = document.content.keys();
    if keys.len() <= 1 {
        let at = keys.first().map_or(t, |k| k.t);
        let mut only = crate::doc::store::ContentTrack::new();
        only.insert(ContentKeyframe { t: at, content });
        document.content = only;
    } else {
        document.content.insert(ContentKeyframe { t, content });
    }
    doc.apply(Intent::SetTextDocument { layer, document })
}

fn prop_row(
    p: &PropRow,
    layer: LayerId,
    t: RationalTime,
    doc: &Arc<Mutex<Document>>,
    session: &Session,
    mut revision: Signal<u32>,
) -> Element {
    // 1 値の行は X の列に置く(AE・Figma)。data は 3 番目に持つので描く順だけ入れ替える。
    let solo = p.cells[0].is_empty() && p.cells[1].is_empty() && !p.cells[2].is_empty();
    let order: [usize; 3] = if solo { [2, 0, 1] } else { [0, 1, 2] };
    let cells = order.into_iter().map(|i| (i, (&p.cells[i], p.dims[i]))).map(|(i, (c, dim))| {
        let class = if c.is_empty() {
            "v blank"
        } else if dim {
            "v z"
        } else {
            "v"
        };
        let target = match &p.axis[i] {
            Some((property, value)) => Some((property.clone(), value.clone(), false)),
            None if !c.is_empty() && (if p.vec2 { i < 2 } else { i == 2 }) => p
                .property
                .clone()
                .map(|property| (property, p.value.clone(), p.vec2)),
            None => None,
        };
        if let Some((property, start_value, vec2)) = target {
            let range = p.range;
            let at = FieldAt::Number { layer, property: property.clone(), axis: i };
            if session.field_at(&at).is_some() {
                let doc_commit = doc.clone();
                let commit_session = session.clone();
                let start_value = start_value.clone();
                return rsx!(Field {
                    session: session.clone(),
                    class: "{class} typing",
                    revision,
                    oncommit: move |f: OpenField| {
                        let FieldAt::Number { layer, property, axis } = f.at else { return };
                        let Ok(v) = f.draft.trim().parse::<f64>() else { return };
                        let others = others_at(&commit_session, layer, &property, t);
                        let targets = std::iter::once((layer, start_value.clone())).chain(others);
                        let mut wrote = false;
                        for (layer, base) in targets {
                            let value = put_axis(&base, vec2, axis, v, range);
                            // 同じ値は書かない。複数選択では層ごとに見る(主の層だけで早帰りしない)。
                            if value == base {
                                continue;
                            }
                            match write_key(&doc_commit, layer, &property, value, t) {
                                Ok(()) => wrote = true,
                                Err(err) => println!("PROBE room=write verdict=apply-error {err}"),
                            }
                        }
                        if wrote {
                            *revision.write() += 1;
                        }
                    },
                });
            }
            let opener = session.clone();
            let grabber = session.clone();
            let open = (property.clone(), start_value.clone());
            let cell = c.clone();
            rsx!(span {
                class: "{class}",
                onmousedown: move |evt| {
                    let x = evt.data().client_coordinates().x;
                    let others = others_at(&grabber, layer, &property, t);
                    *grabber.scrub.lock().unwrap() = Some(ValueDrag {
                        layer,
                        others,
                        property: property.clone(),
                        vec2,
                        axis: i,
                        start_x: x,
                        start_value: start_value.clone(),
                        range,
                        last_dx: 0.0,
                    });
                },
                ondoubleclick: move |_| {
                    // 擦りかけの下書き(transient)を残さない。
                    cancel_scrub(&opener);
                    opener.open_field(
                        FieldAt::Number { layer, property: open.0.clone(), axis: i },
                        cell.clone(),
                    );
                    *revision.write() += 1;
                },
                "{c}"
            })
        } else {
            rsx!(span { class: "{class}", "{c}" })
        }
    });
    let key_class = if p.keyed { "glyph on" } else { "glyph" };
    let key_glyph = if p.keyed { "◆" } else { "◇" };
    let key_click = p.property.clone().map(|property| {
        let value = p.value.clone();
        let doc = doc.clone();
        let keyed = p.keyed;
        // ◇ は今の時刻に 1 つ立てる。◆ は今の時刻のキーだけ外す(AE のナビゲータ)。
        // Alt+◆ で時間の世界を閉じ、今の値だけを残す(AE のストップウォッチ)。
        move |evt: MouseEvent| {
            let done = if keyed && evt.modifiers().alt() {
                close_time(&doc, layer, &property, value.clone())
            } else if keyed {
                toggle_key_at(&doc, layer, &property, value.clone(), t)
            } else {
                open_time(&doc, layer, &property, value.clone(), t)
            };
            match done {
                Ok(_) => {
                    println!(
                        "PROBE room=write verdict=key-{} layer={:?} prop={} t={:?}",
                        if keyed { "closed" } else { "opened" },
                        layer,
                        property,
                        t
                    );
                    *revision.write() += 1;
                }
                Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
            }
        }
    });
    rsx!(
        div { class: "prow",
            span { class: "n", "{p.label}" }
            {cells}
            if let Some(on_click) = key_click {
                SemanticButton { class: "{key_class}", selected: p.keyed, aria_label: if p.keyed { "Remove the keyframe at this time · Alt removes all" } else { "Add a keyframe at this time" }, onclick: on_click, "{key_glyph}" }
            } else {
                span { class: "{key_class}", "{key_glyph}" }
            }
        }
    )
}

fn content_row(
    p: &PropRow,
    layer: LayerId,
    t: RationalTime,
    doc: &Arc<Mutex<Document>>,
    session: &Session,
    mut revision: Signal<u32>,
) -> Element {
    let key_class = if p.keyed { "glyph on" } else { "glyph" };
    let key_glyph = if p.keyed { "◆" } else { "◇" };
    let current = p.cells[2].clone();
    if session.field_at(&FieldAt::Content(layer)).is_some() {
        let doc_commit = doc.clone();
        let unchanged = current.clone();
        return rsx!(
            div { class: "prow content-row",
                span { class: "n", "{p.label}" }
                Field {
                    session: session.clone(),
                    class: "v content",
                    multiline: true,
                    revision,
                    oncommit: move |f: OpenField| {
                        if f.draft == unchanged {
                            return;
                        }
                        match write_content(&doc_commit, layer, t, f.draft) {
                            Ok(_) => {
                                println!("PROBE room=write verdict=content-commit layer={:?} t={:?}", layer, t);
                                *revision.write() += 1;
                            }
                            Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                        }
                    },
                }
                span { class: "{key_class}", "{key_glyph}" }
            }
        );
    }
    let opener = session.clone();
    rsx!(
        div { class: "prow content-row",
            span { class: "n", "{p.label}" }
            span {
                class: "v content",
                ondoubleclick: move |_| {
                    opener.open_field(FieldAt::Content(layer), current.clone());
                    *revision.write() += 1;
                },
                "{p.cells[2]}"
            }
            span { class: "{key_class}", "{key_glyph}" }
        }
    )
}

fn descends_from(view: &StoreView<'_>, layer: LayerId, ancestor: LayerId) -> bool {
    let mut cur = Some(layer);
    for _ in 0..64 {
        let Some(l) = cur else { return false };
        if l == ancestor {
            return true;
        }
        cur = view.attrs(l).ok().flatten().and_then(|a| a.parent);
    }
    false
}

fn layer_label(view: &StoreView<'_>, layer: LayerId) -> String {
    view.attrs(layer)
        .ok()
        .flatten()
        .map(|attrs| attrs.name)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("Layer {}", layer.0))
}

fn parent_candidates(view: &StoreView<'_>, layer: LayerId) -> Vec<(LayerId, String)> {
    view.layers()
        .into_iter()
        .filter(|candidate| *candidate != layer && !descends_from(view, *candidate, layer))
        .map(|candidate| (candidate, layer_label(view, candidate)))
        .collect()
}

fn matte_chain_reaches(view: &StoreView<'_>, start: LayerId, target: LayerId) -> bool {
    let mut current = Some(start);
    let mut seen = std::collections::HashSet::new();
    while let Some(layer) = current {
        if layer == target {
            return true;
        }
        if !seen.insert(layer) {
            return false;
        }
        current = view
            .attrs(layer)
            .ok()
            .flatten()
            .and_then(|attrs| attrs.matte)
            .map(|matte| matte.layer);
    }
    false
}

fn is_texture_matte_source(source: &LayerSource) -> bool {
    match source {
        LayerSource::Shape | LayerSource::Text => true,
        LayerSource::File { path, .. } => std::path::Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .and_then(crate::render::media::asset_type_for_extension)
            .is_some_and(|kind| kind.starts_with("image/") || kind.starts_with("video/")),
        LayerSource::Null | LayerSource::Group => false,
    }
}

fn matte_candidates(view: &StoreView<'_>, target: LayerId) -> Vec<(LayerId, String)> {
    view.layers()
        .into_iter()
        .filter(|candidate| *candidate != target)
        .filter(|candidate| !matte_chain_reaches(view, *candidate, target))
        .filter(|candidate| {
            view.meta(*candidate)
                .ok()
                .flatten()
                .is_some_and(|meta| is_texture_matte_source(&meta.source))
        })
        .map(|candidate| (candidate, layer_label(view, candidate)))
        .collect()
}

fn set_parent(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    parent: Option<LayerId>,
) -> Result<(), StoreError> {
    let patch = LayerAttrsPatch { parent: Some(parent), ..Default::default() };
    doc.lock().unwrap().apply(Intent::SetAttrs { layer, patch })
}

fn set_matte_source(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    source: Option<LayerId>,
) -> Result<(), StoreError> {
    let mut doc = doc.lock().unwrap();
    let mode = doc
        .view()
        .attrs(layer)?
        .and_then(|attrs| attrs.matte)
        .map_or(MatteMode::Alpha, |matte| matte.mode);
    let matte = source.map(|source| Matte { layer: source, mode });
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch { matte: Some(matte), ..Default::default() },
    })
}

fn set_matte_mode(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    mode: MatteMode,
) -> Result<(), StoreError> {
    let mut doc = doc.lock().unwrap();
    let Some(mut matte) = doc.view().attrs(layer)?.and_then(|attrs| attrs.matte) else {
        return Err(StoreError::Property("Choose a matte source first".to_owned()));
    };
    matte.mode = mode;
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch { matte: Some(Some(matte)), ..Default::default() },
    })
}

#[derive(Clone, PartialEq)]
struct LayerChoice {
    layer: Option<LayerId>,
    label: String,
}

#[derive(Clone)]
struct LayerChoiceAction(Rc<dyn Fn(Option<LayerId>) -> Result<(), StoreError>>);

impl PartialEq for LayerChoiceAction {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ChoiceId {
    Parent,
    MatteSource,
    MatteMode,
}

#[component]
pub(super) fn ChoiceDismiss(mut open: Signal<Option<ChoiceId>>) -> Element {
    if open().is_none() {
        return rsx! {};
    }
    rsx!(div {
        class: "choice-dismiss",
        role: "presentation",
        onmousedown: move |evt| {
            evt.stop_propagation();
            open.set(None);
        },
    })
}

#[component]
fn ChoiceTrigger(current: String, id: ChoiceId, mut open: Signal<Option<ChoiceId>>) -> Element {
    let shown = open() == Some(id);
    rsx!(button {
        class: "v content choice-trigger",
        aria_expanded: if shown { "true" } else { "false" },
        onclick: move |_| open.set(if open() == Some(id) { None } else { Some(id) }),
        onkeydown: move |evt| {
            if evt.key() == Key::Escape {
                evt.prevent_default();
                evt.stop_propagation();
                open.set(None);
            }
        },
        "{current}"
    })
}

#[derive(Clone, PartialEq, Props)]
struct LayerChoiceRowProps {
    id: ChoiceId,
    label: &'static str,
    current: String,
    choices: Vec<LayerChoice>,
    action: LayerChoiceAction,
    open: Signal<Option<ChoiceId>>,
}

#[allow(non_snake_case)]
fn LayerChoiceRow(props: LayerChoiceRowProps) -> Element {
    let mut open = props.open;
    let mut rejection = use_signal(String::new);
    let shown = open() == Some(props.id);
    rsx!(
        div { class: "prow",
            span { class: "n", "{props.label}" }
            ChoiceTrigger { current: props.current.clone(), id: props.id, open }
            span { class: "glyph", "◇" }
        }
        if shown {
            for choice in props.choices.iter().cloned() {
                SemanticButton {
                    class: "prow blend-pick",
                    onclick: {
                        let action = props.action.clone();
                        move |_| {
                            match (action.0)(choice.layer) {
                                Ok(()) => {
                                    rejection.set(String::new());
                                    open.set(None);
                                }
                                Err(error) => rejection.set(error.to_string()),
                            }
                        }
                    },
                    span { class: "n", "" }
                    span { class: "v content", "{choice.label}" }
                }
            }
        }
        if !rejection().is_empty() {
            div { class: "hint", "{rejection}" }
        }
    )
}

const MATTE_MODES: &[(MatteMode, &str)] = &[
    (MatteMode::Alpha, "Alpha"),
    (MatteMode::InvertedAlpha, "Inverted Alpha"),
    (MatteMode::Luma, "Luma"),
    (MatteMode::InvertedLuma, "Inverted Luma"),
];

fn matte_mode_label(mode: MatteMode) -> &'static str {
    MATTE_MODES
        .iter()
        .find(|(candidate, _)| *candidate == mode)
        .map(|(_, label)| *label)
        .unwrap_or("Alpha")
}

#[derive(Clone)]
struct MatteModeAction(Rc<dyn Fn(MatteMode) -> Result<(), StoreError>>);

impl PartialEq for MatteModeAction {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, PartialEq, Props)]
struct MatteModeRowProps {
    current: MatteMode,
    action: MatteModeAction,
    open: Signal<Option<ChoiceId>>,
}

#[allow(non_snake_case)]
fn MatteModeRow(props: MatteModeRowProps) -> Element {
    let mut open = props.open;
    let mut rejection = use_signal(String::new);
    let shown = open() == Some(ChoiceId::MatteMode);
    rsx!(
        div { class: "prow",
            span { class: "n", "Mode" }
            ChoiceTrigger {
                current: matte_mode_label(props.current).to_owned(),
                id: ChoiceId::MatteMode,
                open,
            }
            span { class: "glyph", "◇" }
        }
        if shown {
            for (mode, label) in MATTE_MODES.iter().copied() {
                SemanticButton {
                    class: if mode == props.current { "prow blend-pick on" } else { "prow blend-pick" },
                    selected: mode == props.current,
                    onclick: {
                        let action = props.action.clone();
                        move |_| {
                            match (action.0)(mode) {
                                Ok(()) => {
                                    rejection.set(String::new());
                                    open.set(None);
                                }
                                Err(error) => rejection.set(error.to_string()),
                            }
                        }
                    },
                    span { class: "n", "" }
                    span { class: "v content", "{label}" }
                }
            }
        }
        if !rejection().is_empty() {
            div { class: "hint", "{rejection}" }
        }
    )
}

pub(super) fn inspector_panel(
    doc: &Arc<Mutex<Document>>,
    selection: Option<LayerId>,
    clock: &Clock,
    mut revision: Signal<u32>,
    session: &Session,
    choice_open: Signal<Option<ChoiceId>>,
    playhead: Signal<f64>,
    selected_size: &Arc<Mutex<Option<[f32; 2]>>>,
    focus: &Arc<Mutex<Option<Focus>>>,
    live_focus: Option<Focus>,
) -> Element {
    let blend_focused = matches!(
        (selection, &live_focus),
        (Some(layer), Some(Focus::Blend(at))) if *at == layer
    );
    let color_focus = match live_focus {
        Some(Focus::Color(slot)) => Some(slot),
        _ => None,
    };
    let box_size = *selected_size.lock().unwrap();
    let _ = revision(); // Document書き換え後の再描画をここで購読する(値そのものは使わない)
    // 再生位置が動いた時も描き直す。**値は時刻で決まる**ので、
    // ここを購読しないと絵だけ動いて数字が止まる。
    let _ = playhead();

    let empty = InspectorData {
        blend: BlendMode::Normal,
        ident_name: "No selection".to_string(),
        ident_sub: String::new(),
        text: Vec::new(),
        transform: Vec::new(),
        effects: Vec::new(),
        has_effects: false,
        colors: Vec::new(),
    };
    let t = RationalTime::try_new((clock.now_sec() * 3000.0) as i64, 3000).unwrap_or(RationalTime::ZERO);
    let mut data = match selection {
        Some(layer) => inspector_data_from_doc(&doc.lock().unwrap().view(), layer, t),
        None => empty,
    };
    // 複数選択: Transform の共通行だけ。違う値の欄は空にして、書けば全部へ届く(§1)。
    let chosen = session.selection.all();
    if chosen.len() > 1 {
        let d = doc.lock().unwrap();
        let view = d.view();
        let siblings: Vec<_> = chosen
            .iter()
            .filter(|l| Some(**l) != selection)
            .map(|l| inspector_data_from_doc(&view, *l, t))
            .collect();
        for row in &mut data.transform {
            for i in 0..3 {
                let agree = siblings.iter().all(|s| {
                    s.transform.iter().any(|r| r.label == row.label && r.cells[i] == row.cells[i])
                });
                // 違う値は「—」(Figma の Mixed)。空にすると掴む口が消えるので文字で残す。
                if !agree && !row.cells[i].is_empty() {
                    row.cells[i] = "—".to_owned();
                }
            }
            row.keyed = false;
        }
        data.ident_name = format!("{} layers", chosen.len());
        data.text.clear();
        data.colors.clear();
        data.effects.clear();
        data.has_effects = true;
    }
    let inspector = &data;

    // 文字の行のうち、property を持つ物(級数)は数の行。本文だけが文の行。
    let text_rows = inspector.text.iter().map(|p| {
        if p.property.is_some() {
            prop_row(p, selection.unwrap_or(LayerId(0)), t, doc, session, revision)
        } else {
            content_row(p, selection.unwrap_or(LayerId(0)), t, doc, session, revision)
        }
    });
    let transform_rows = inspector
        .transform
        .iter()
        .map(|p| prop_row(p, selection.unwrap_or(LayerId(0)), t, doc, session, revision));
    let effect_blocks: Vec<_> = inspector
        .effects
        .iter()
        .map(|block| {
            (
                block.id,
                block.plugin_id.clone(),
                block
                    .params
                    .iter()
                    .map(|p| prop_row(p, selection.unwrap_or(LayerId(0)), t, doc, session, revision))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let fx_label = if inspector.has_effects { "" } else { "No effects shared by the selection" };

    let (parent_label, parent_choices, matte_source_label, matte_choices, matte) = match selection {
        Some(layer) => {
            let d = doc.lock().unwrap();
            let view = d.view();
            let attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
            let parent_label = attrs
                .parent
                .map(|parent| layer_label(&view, parent))
                .unwrap_or_else(|| "None".to_owned());
            let mut parent_choices = vec![LayerChoice { layer: None, label: "None".to_owned() }];
            parent_choices.extend(
                parent_candidates(&view, layer)
                    .into_iter()
                    .map(|(candidate, label)| LayerChoice { layer: Some(candidate), label }),
            );
            let matte = attrs.matte;
            let matte_source_label = matte
                .map(|matte| layer_label(&view, matte.layer))
                .unwrap_or_else(|| "None".to_owned());
            let mut matte_choices = vec![LayerChoice { layer: None, label: "None".to_owned() }];
            matte_choices.extend(
                matte_candidates(&view, layer)
                    .into_iter()
                    .map(|(candidate, label)| LayerChoice { layer: Some(candidate), label }),
            );
            (parent_label, parent_choices, matte_source_label, matte_choices, matte)
        }
        None => ("None".to_owned(), Vec::new(), "None".to_owned(), Vec::new(), None),
    };
    let has_children = match selection {
        Some(layer) => {
            let d = doc.lock().unwrap();
            let view = d.view();
            view.layers().into_iter().any(|l| {
                view.attrs(l).ok().flatten().and_then(|a| a.parent) == Some(layer)
            })
        }
        None => false,
    };
    let frozen = match selection {
        Some(layer) => doc
            .lock()
            .unwrap()
            .view()
            .attrs(layer)
            .ok()
            .flatten()
            .is_some_and(|a| a.frozen),
        None => false,
    };

    let parent_action = selection.map(|layer| {
        let doc = doc.clone();
        let revision_signal = revision;
        LayerChoiceAction(Rc::new(move |parent| {
            set_parent(&doc, layer, parent)?;
            let mut revision = revision_signal;
            *revision.write() += 1;
            println!("PROBE room=write verdict=applied SetAttrs parent={parent:?} layer={layer:?}");
            Ok(())
        }))
    });
    let matte_source_action = selection.map(|layer| {
        let doc = doc.clone();
        let revision_signal = revision;
        LayerChoiceAction(Rc::new(move |source| {
            set_matte_source(&doc, layer, source)?;
            let mut revision = revision_signal;
            *revision.write() += 1;
            println!("PROBE room=write verdict=applied SetAttrs matte-source={source:?} layer={layer:?}");
            Ok(())
        }))
    });
    let matte_mode_action = selection.map(|layer| {
        let doc = doc.clone();
        let revision_signal = revision;
        MatteModeAction(Rc::new(move |mode| {
            set_matte_mode(&doc, layer, mode)?;
            let mut revision = revision_signal;
            *revision.write() += 1;
            println!("PROBE room=write verdict=applied SetAttrs matte-mode={mode:?} layer={layer:?}");
            Ok(())
        }))
    });

    let scrub_move = session.clone();
    let scrub_up = session.clone();
    rsx!(
        div {
            id: "inspector",
            onmousemove: move |evt| {
                if evt.data().held_buttons().is_empty() {
                    if end_scrub(&scrub_move) {
                        *revision.write() += 1;
                    }
                    return;
                }
                let state = scrub_move.scrub.lock().unwrap().as_mut().map(|d| {
                    let x = evt.data().client_coordinates().x;
                    let dx = x - d.start_x;
                    let changed = dx != d.last_dx;
                    d.last_dx = dx;
                    let mut targets = vec![(d.layer, d.start_value.clone())];
                    targets.extend(d.others.iter().cloned());
                    (changed, targets, d.property.clone(), d.vec2, d.axis, d.range, dx)
                });
                let Some((changed, targets, property, vec2, axis, range, dx)) = state else { return };
                if !changed {
                    return;
                }
                let Ok(prop) = PropertyId::new(&property) else { return };
                let mut doc = scrub_move.doc.lock().unwrap();
                for (layer, start) in targets {
                    let new_value = nudge(&start, vec2, axis, dx * increment(&property, range), range);
                    doc.set_transient(layer, prop.clone(), new_value);
                }
                drop(doc);
                *revision.write() += 1;
            },
            onmouseup: move |_| {
                if end_scrub(&scrub_up) {
                    *revision.write() += 1;
                }
            },
            div { class: "ident",
                div {
                    b { "{inspector.ident_name}" }
                    span { class: "sub", "{inspector.ident_sub}" }
                }
            }
            div { class: "cols",
                span { class: "pn", "Property" }
                span { "X" }
                span { "Y" }
                span { "Z" }
                span { class: "k", "Key" }
            }
            div { class: "sec", "Transform" }
            {transform_rows}
            // 升の並びそのものが意味なので、言葉は置かない(裁定451)。
            if let (Some(layer), Some(size)) = (selection, box_size) {
                div { class: "prow anchor",
                    span { class: "n", "anchor" }
                    div { class: "anchorgrid",
                        for (fx , fy) in ANCHOR_SPOTS.iter().copied() {
                            SemanticButton {
                                class: "aspot",
                                aria_label: "Set anchor {fx} {fy}",
                                onclick: {
                                    let doc = doc.clone();
                                    move |_| {
                                        crate::ui::utility::move_anchor(&doc, layer, size, t, fx, fy);
                                        *revision.write() += 1;
                                    }
                                },
                                span { class: "adot" }
                            }
                        }
                    }
                }
            }
            div { class: "iscroll",
            if let Some(layer) = selection {
                div { class: "sec", "Blend" }
                // 値は文字で選ばない。行を光らせ、机がサムネイルの格子を出す。
                SemanticButton {
                    class: if blend_focused { "prow focus on" } else { "prow focus" },
                    selected: blend_focused,
                    aria_label: "Focus blend",
                    onclick: {
                        let focus = focus.clone();
                        let asker = session.clone();
                        move |_| {
                            *focus.lock().unwrap() = Some(Focus::Blend(layer));
                            // COLOR 行と同じ扱い: 応える所(机)を前に出す。
                            asker.ask_panel(crate::ui::dock::Panel::Desk);
                            *revision.write() += 1;
                        }
                    },
                    span { class: "n", "mode" }
                    span { class: "v content", "{blend_label(inspector.blend)}" }
                    span { class: "glyph", "◇" }
                }
            }
            if let Some(layer) = selection {
                div { class: "sec", "Parent" }
                if let Some(action) = parent_action.clone() {
                    LayerChoiceRow {
                        id: ChoiceId::Parent,
                        label: "Parent",
                        current: parent_label.clone(),
                        choices: parent_choices.clone(),
                        action,
                        open: choice_open,
                    }
                }
                if has_children {
                    SemanticButton {
                        class: "prow",
                        selected: frozen,
                        onclick: {
                            let doc = doc.clone();
                            move |_| {
                                let intent = if frozen {
                                    Intent::Unfreeze { group: layer }
                                } else {
                                    Intent::Freeze { group: layer }
                                };
                                match doc.lock().unwrap().apply(intent) {
                                    Ok(_) => *revision.write() += 1,
                                    Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                                }
                            }
                        },
                        span { class: "n", "Group" }
                        span { class: "v chip", if frozen { "Unfreeze" } else { "Freeze" } }
                    }
                }
            }
            if selection.is_some() {
                div { class: "sec", "Matte" }
                if let Some(action) = matte_source_action.clone() {
                    LayerChoiceRow {
                        id: ChoiceId::MatteSource,
                        label: "Source",
                        current: matte_source_label.clone(),
                        choices: matte_choices.clone(),
                        action,
                        open: choice_open,
                    }
                }
                if let (Some(matte), Some(action)) = (matte, matte_mode_action.clone()) {
                    MatteModeRow { current: matte.mode, action, open: choice_open }
                }
            }
            if !inspector.text.is_empty() {
                div { class: "sec", "Text" }
                {text_rows}
            }
            if !inspector.colors.is_empty() {
                div { class: "sec", "Color" }
                for ColorRow { label , hex , slot } in inspector.colors.iter() {
                    SemanticButton {
                        class: if color_focus.as_ref() == Some(slot) { "prow color focus on" } else { "prow color focus" },
                        selected: color_focus.as_ref() == Some(slot),
                        aria_label: "Focus {label} color",
                        onclick: {
                            let focus = focus.clone();
                            let slot = slot.clone();
                            let asker = session.clone();
                            move |_| {
                                *focus.lock().unwrap() = Some(Focus::Color(slot.clone()));
                                // 押した所と応える所を離さない。輪の居る Colors を前に出す。
                                asker.ask_panel(crate::ui::dock::Panel::Colors);
                                *revision.write() += 1;
                            }
                        },
                        span { class: "n", "{label}" }
                        span { class: "v swatch",
                            span { class: "dot", style: "background:{hex};" }
                            "{hex}"
                        }
                        span { class: "v blank", "" }
                        span { class: "v blank", "" }
                        span { class: "glyph", "◇" }
                    }
                }
            }
            div { class: "sec", "Effects" }
            if inspector.effects.is_empty() {
                div { class: "prow",
                    span { class: "n empty", "{fx_label}" }
                    span { class: "v blank", "" }
                    span { class: "v blank", "" }
                    span { class: "v blank", "" }
                    span { "" }
                }
            } else if let Some(layer) = selection {
                for (id , plugin_id , rows) in effect_blocks.into_iter() {
                    div { class: "prow fxhead",
                        span { class: "n", "{plugin_id}" }
                        SemanticButton {
                            class: "v fxdrop",
                            aria_label: "Remove {plugin_id}",
                            onclick: {
                                let doc = doc.clone();
                                move |_| {
                                    remove_effect(&doc, layer, id);
                                    *revision.write() += 1;
                                }
                            },
                            "×"
                        }
                    }
                    {rows.into_iter()}
                }
            }
            }
            div { class: "hint", "Drag to scrub · double-click to type · Esc to cancel" }
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{Composition, Fps, LayerMeta, LayerTiming};

    fn add_layer(doc: &mut Document, id: u64, source: LayerSource, name: &str) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source,
                    order: id as i16,
                    timing: LayerTiming::place(0, None, 30),
                },
            },
            Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch { name: Some(name.to_owned()), ..Default::default() },
            },
        ])
        .unwrap();
        layer
    }

    fn document() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 32,
            height: 32,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 30,
            background: [0.0, 0.0, 0.0, 0.0],
        }))
        .unwrap();
        doc
    }

    #[test]
    fn matte_candidates_exclude_self_cycles_and_non_texture_sources() {
        let mut doc = document();
        let target = add_layer(&mut doc, 1, LayerSource::Shape, "Target");
        let shape = add_layer(&mut doc, 2, LayerSource::Shape, "Shape");
        let image = add_layer(
            &mut doc,
            3,
            LayerSource::File { path: "image.png".into(), fingerprint: None },
            "Image",
        );
        let video = add_layer(
            &mut doc,
            4,
            LayerSource::File { path: "movie.mp4".into(), fingerprint: None },
            "Video",
        );
        let mesh = add_layer(
            &mut doc,
            5,
            LayerSource::File { path: "mesh.obj".into(), fingerprint: None },
            "Mesh",
        );
        let points = add_layer(
            &mut doc,
            6,
            LayerSource::File { path: "cloud.ply".into(), fingerprint: None },
            "Points",
        );
        let group = add_layer(&mut doc, 7, LayerSource::Group, "Group");
        let null = add_layer(&mut doc, 8, LayerSource::Null, "Null");
        let cycle = add_layer(&mut doc, 9, LayerSource::Text, "Cycle");
        doc.apply(Intent::SetAttrs {
            layer: cycle,
            patch: LayerAttrsPatch {
                matte: Some(Some(Matte { layer: target, mode: MatteMode::Alpha })),
                ..Default::default()
            },
        })
        .unwrap();

        let ids: Vec<_> = matte_candidates(&doc.view(), target)
            .into_iter()
            .map(|(layer, _)| layer)
            .collect();
        assert!(ids.contains(&shape));
        assert!(ids.contains(&image));
        assert!(ids.contains(&video));
        for rejected in [target, mesh, points, group, null, cycle] {
            assert!(!ids.contains(&rejected), "unexpected matte source: {rejected:?}");
        }
    }

    #[test]
    fn none_and_all_four_modes_round_trip_one_undo_at_a_time() {
        for mode in [
            MatteMode::Alpha,
            MatteMode::InvertedAlpha,
            MatteMode::Luma,
            MatteMode::InvertedLuma,
        ] {
            let mut doc = document();
            let target = add_layer(&mut doc, 1, LayerSource::Shape, "Target");
            let source = add_layer(&mut doc, 2, LayerSource::Shape, "Source");
            let doc = Arc::new(Mutex::new(doc));

            set_matte_source(&doc, target, Some(source)).unwrap();
            set_matte_mode(&doc, target, mode).unwrap();
            assert_eq!(doc.lock().unwrap().view().attrs(target).unwrap().unwrap().matte,
                Some(Matte { layer: source, mode }));
            assert!(doc.lock().unwrap().undo());
            assert_eq!(doc.lock().unwrap().view().attrs(target).unwrap().unwrap().matte,
                Some(Matte { layer: source, mode: MatteMode::Alpha }));

            set_matte_source(&doc, target, None).unwrap();
            assert_eq!(doc.lock().unwrap().view().attrs(target).unwrap().unwrap().matte, None);
            assert!(doc.lock().unwrap().undo());
            assert_eq!(doc.lock().unwrap().view().attrs(target).unwrap().unwrap().matte,
                Some(Matte { layer: source, mode: MatteMode::Alpha }));
        }
    }
}
