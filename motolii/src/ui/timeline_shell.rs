use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;

use crate::ui::fixture::{fmt_timecode, LayerRow};
use crate::ui::playback::Clock;
use crate::ui::session::Selection;
use crate::ui::timeline_widget::TimelineMsg;
use crate::doc::store::{Document, Intent, LayerAttrsPatch, LayerId};

/// 再生の道具。タブの帯の右へ乗る(帯を2段にしないため)。
pub(super) fn transport(
    clock: Arc<Clock>,
    mut playing: Signal<bool>,
    layer_count: usize,
) -> Element {
    let timecode = fmt_timecode(clock.now_sec());
    rsx!(
        div { class: "ptools",
            button {
                id: "play",
                onclick: {
                    let clock = clock.clone();
                    move |_| {
                        clock.toggle();
                        *playing.write() = clock.playing();
                    }
                },
                if playing() { "■" } else { "▶" }
            }
            span { class: "tc", "{timecode}" }
            em { "{layer_count} rows · 30fps · 60s" }
        }
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn timeline_shell(
    doc: Arc<Mutex<Document>>,
    mut attrs: Signal<Vec<(bool, bool, bool)>>,
    layer_rows_data: &[LayerRow],
    layer_rows_sig: Signal<Vec<LayerRow>>,
    timeline_attr: CustomWidgetAttr,
    selection: Selection,
    mut selected: Signal<Option<LayerId>>,
    scroll_y: Signal<f64>,
    timeline_tx: Sender<TimelineMsg>,
    mut renaming: Signal<Option<(LayerId, String)>>,
    mut revision: Signal<u32>,
) -> Element {
    let layer_rows = layer_rows_data.iter().enumerate().map(|(i, row)| {
        let layer = row.layer;
        let doc = doc.clone();
        let doc_twirl = doc.clone();
        let timeline_tx_row = timeline_tx.clone();
        let mut layer_rows_sig = layer_rows_sig;
        let selection = selection.clone();
        let is_primary = selected() == Some(layer);
        let is_secondary = !is_primary && selection.contains(layer);
        let lsurface_style = if is_primary {
            format!("background:{};box-shadow:inset 0 0 0 2px var(--way-inspector);", row.color)
        } else if is_secondary {
            format!("background:{};box-shadow:inset 0 0 0 1px var(--way-inspector);", row.color)
        } else {
            format!("background:{};", row.color)
        };
        let glyph = |bit: u8, label: &'static str| {
            let (hidden, solo, locked) = attrs.read().get(i).copied().unwrap_or_default();
            let lit = match bit {
                0 => hidden,
                1 => solo,
                _ => locked,
            };
            let class = if lit { "glyph lit" } else { "glyph" };
            let doc = doc.clone();
            rsx!(
                span {
                    class: "{class}",
                    onclick: move |_| {
                        let patch = match bit {
                            0 => LayerAttrsPatch { hidden: Some(!hidden), ..Default::default() },
                            1 => LayerAttrsPatch { solo: Some(!solo), ..Default::default() },
                            _ => LayerAttrsPatch { locked: Some(!locked), ..Default::default() },
                        };
                        let mut doc = doc.lock().unwrap();
                        match doc.apply(Intent::SetAttrs { layer, patch }) {
                            Ok(_) => {
                                let a = doc.view().attrs(layer).ok().flatten().unwrap_or_default();
                                attrs.write()[i] = (a.hidden, a.solo, a.locked);
                                println!("PROBE room=write verdict=applied SetAttrs bit={bit}");
                            }
                            Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                        }
                    },
                    "{label}"
                }
            )
        };
        let indent = format!("padding-left:{}px", row.depth as u32 * 18);
        let folded_children = (!row.expanded).then_some(row.children).filter(|n| *n > 0);
        if let Some(name) = row.prop.clone() {
            return rsx!(
                div { class: "lrow", style: "{indent}",
                    span { class: "lprop", "{name}" }
                }
            );
        }
        let expanded = row.expanded;
        let editing_name = renaming().filter(|(l, _)| *l == layer).map(|(_, n)| n);
        let doc_rename = doc.clone();
        rsx!(
            div { class: "lrow", style: "{indent}",
                span {
                    class: "twirl",
                    onclick: {
                        let timeline_tx = timeline_tx_row.clone();
                        let doc = doc_twirl.clone();
                        move |_| {
                            crate::ui::fixture::toggle_expanded(layer);
                            let d = doc.lock().unwrap();
                            let rows = crate::ui::fixture::layer_rows_from_doc(&d);
                            let canvas = crate::ui::fixture::canvas_rows_from_doc(&d);
                            drop(d);
                            attrs.set(rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect());
                            layer_rows_sig.set(rows);
                            let _ = timeline_tx.send(TimelineMsg::SetRows(canvas));
                            *revision.write() += 1;
                        }
                    },
                    if expanded { "▾" } else { "▸" }
                }
                if let Some(n) = folded_children {
                    span { class: "inside", "{n}" }
                }
                if let Some(draft) = editing_name {
                    input {
                        class: "lsurface",
                        style: "{lsurface_style}",
                        value: "{draft}",
                        autofocus: "true",
                        oninput: move |evt| *renaming.write() = Some((layer, evt.value())),
                        onkeydown: move |evt| match evt.key() {
                            Key::Enter => {
                                evt.prevent_default();
                                if let Some((layer, name)) = renaming.write().take() {
                                    let patch = LayerAttrsPatch { name: Some(name.clone()), ..Default::default() };
                                    match doc_rename.lock().unwrap().apply(Intent::SetAttrs { layer, patch }) {
                                        Ok(_) => {
                                            println!("PROBE room=write verdict=applied Rename layer={layer:?} name={name:?}");
                                            *revision.write() += 1;
                                        }
                                        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                                    }
                                }
                            }
                            Key::Escape => {
                                evt.prevent_default();
                                *renaming.write() = None;
                            }
                            _ => {}
                        },
                    }
                } else {
                    span {
                        class: "lsurface",
                        style: "{lsurface_style}",
                        onclick: move |evt| {
                            if evt.modifiers().meta() {
                                selection.toggle(layer);
                            } else {
                                selection.set(Some(layer));
                            }
                            selected.set(selection.get());
                        },
                        ondoubleclick: {
                            let name = row.name.clone();
                            move |_| *renaming.write() = Some((layer, name.clone()))
                        },
                        "{row.name}"
                    }
                }
                div { class: "lctrl",
                    {glyph(0, "M")}
                    {glyph(1, "S")}
                    {glyph(2, "L")}
                }
            }
        )
    });

    rsx!(
        div { id: "timelineshell",
            div { id: "timeline",
                div {
                    id: "layers",
                    onwheel: move |evt| {
                        let dy = evt.data().delta().strip_units().y;
                        let _ = timeline_tx.send(TimelineMsg::ScrollBy(dy));
                    },
                    div { class: "lhead", "OBJECT" }
                    div {
                        style: "transform: translateY(-{scroll_y()}px);",
                        {layer_rows}
                    }
                }
                object { "data": timeline_attr }
            }
        }
    )
}
