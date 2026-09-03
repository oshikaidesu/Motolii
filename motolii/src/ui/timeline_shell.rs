use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;

use crate::doc::store::{Document, Intent, LayerAttrsPatch, LayerId};
use crate::ui::fixture::LayerRow;
use crate::ui::playback::Clock;
use crate::ui::session::{FieldAt, OpenField, Selection, Session};
use crate::ui::semantic_menu::{Field, SemanticButton};
use crate::ui::timeline_widget::TimelineMsg;

/// 再生の道具。タブの帯の右へ乗る(帯を2段にしないため)。
pub(super) fn transport(
    clock: Arc<Clock>,
    mut playing: Signal<bool>,
    playhead: Signal<f64>,
    layer_count: usize,
) -> Element {
    // 再生位置の鏡を購読する。読むだけだと、再生中この字は描き直されず
    // **絵は動いているのに時刻が止まって見える**。
    let _ = playhead();
    let timecode = clock.format_timecode();
    let composition = clock.composition_label();
    let audio_health = clock.health().visible_label();
    let layer_word = if layer_count == 1 { "layer" } else { "layers" };
    rsx!(
        div { class: "ptools",
            SemanticButton {
                class: "play",
                selected: playing(),
                aria_label: if playing() { "Pause" } else { "Play" },
                title: "Space",
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
            em {
                "{layer_count} {layer_word} · {composition}"
                if let Some(label) = audio_health { " · {label}" }
            }
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
    session: &Session,
    mut revision: Signal<u32>,
) -> Element {
    let layer_rows = layer_rows_data.iter().enumerate().map(|(i, row)| {
        let layer = row.layer;
        let doc = doc.clone();
        let doc_twirl = doc.clone();
        let timeline_tx_row = timeline_tx.clone();
        let mut layer_rows_sig = layer_rows_sig;
        let selection = selection.clone();
        let is_primary = layer.is_some() && selected() == layer;
        let is_secondary =
            !is_primary && layer.is_some_and(|l| selection.contains(l));
        // 色は左の帯、選択は行の背景(AE・Resolve)。色の壁の上に文字を置かない。
        let is_selected = layer.is_some_and(|l| selection.contains(l));
        let lsurface_style = if is_primary {
            format!("border-left-color:{};background:var(--raised);box-shadow:inset 2px 0 0 var(--accent);", row.color)
        } else if is_secondary {
            format!("border-left-color:{};background:var(--raised);", row.color)
        } else {
            format!("border-left-color:{};", row.color)
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
                SemanticButton {
                    class: "{class}",
                    selected: lit,
                    aria_label: match bit { 0 => "M · hide layer", 1 => "S · solo layer", _ => "L · lock layer" },
                    title: match bit { 0 => "Hide layer", 1 => "Solo layer", _ => "Lock layer" },
                    onclick: move |_| {
                        let Some(layer) = layer else { return };
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
        let editing_name = layer.is_some_and(|l| session.field_at(&FieldAt::Name(l)).is_some());
        let opener = session.clone();
        let field_session = session.clone();
        let unchanged = row.name.clone();
        let doc_rename = doc.clone();
        rsx!(
            div { class: "lrow", style: "{indent}",
                SemanticButton {
                    class: "twirl",
                    selected: expanded,
                    aria_label: if expanded { "Collapse row" } else { "Expand row" },
                    onclick: {
                        let timeline_tx = timeline_tx_row.clone();
                        let doc = doc_twirl.clone();
                        move |_| {
                            match layer {
                                Some(l) => crate::ui::fixture::toggle_expanded(l),
                                None => crate::ui::fixture::toggle_camera_open(),
                            }
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
                if editing_name {
                    Field {
                        label: "Layer name",
                        session: field_session,
                        class: "lsurface",
                        style: "{lsurface_style}",
                        revision,
                        oncommit: move |f: OpenField| {
                            let FieldAt::Name(layer) = f.at else { return };
                            let name = f.draft;
                            if name.trim().is_empty() || name == unchanged {
                                return;
                            }
                            let patch = LayerAttrsPatch {
                                name: Some(name.clone()),
                                ..Default::default()
                            };
                            match doc_rename
                                .lock()
                                .unwrap()
                                .apply(Intent::SetAttrs { layer, patch })
                            {
                                Ok(_) => {
                                    println!("PROBE room=write verdict=applied Rename layer={layer:?} name={name:?}");
                                    *revision.write() += 1;
                                }
                                Err(e) => {
                                    println!("PROBE room=write verdict=apply-error {e}")
                                }
                            }
                        },
                    }
                } else {
                    span {
                        class: "lsurface",
                        style: "{lsurface_style}",
                        tabindex: "0",
                        role: "option",
                        aria_selected: if is_selected { "true" } else { "false" },
                        onclick: move |evt| {
                            let Some(l) = layer else { return };
                            if evt.modifiers().intersects(Modifiers::META | Modifiers::SUPER) {
                                selection.toggle(l);
                            } else {
                                selection.set(Some(l));
                            }
                            selected.set(selection.get());
                        },
                        ondoubleclick: {
                            let name = row.name.clone();
                            move |_| {
                                if let Some(l) = layer {
                                    opener.open_field(FieldAt::Name(l), name.clone());
                                    *revision.write() += 1;
                                }
                            }
                        },
                        "{row.name}"
                    }
                }
                if layer.is_some() {
                    div { class: "lctrl",
                        {glyph(0, "M")}
                        {glyph(1, "S")}
                        {glyph(2, "L")}
                    }
                } else {
                    // カメラに表示と独奏は無い。錠だけ在る —— 掛けると枠を掴めない。
                    div { class: "lctrl",
                        SemanticButton {
                            class: if row.locked { "glyph lit" } else { "glyph" },
                            selected: row.locked,
                            aria_label: "Toggle camera lock",
                            onclick: move |_| {
                                crate::ui::fixture::toggle_camera_locked();
                                *revision.write() += 1;
                            },
                            "L"
                        }
                    }
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
                    div { class: "lhead", "Layer" }
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
