use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;

use crate::fixture::{fmt_timecode, LayerRow};
use crate::playback::Clock;
use crate::session::Selection;
use motolii_store::{Document, Intent, LayerAttrsPatch, LayerId};

pub fn timeline_shell(
    clock: Arc<Clock>,
    mut playing: Signal<bool>,
    doc: Arc<Mutex<Document>>,
    mut attrs: Signal<Vec<(bool, bool, bool)>>,
    layer_rows_data: &[LayerRow],
    timeline_attr: CustomWidgetAttr,
    selection: Selection,
    mut selected: Signal<Option<LayerId>>,
) -> Element {
    let layer_rows = layer_rows_data.iter().enumerate().map(|(i, row)| {
        let layer = row.layer;
        let doc = doc.clone();
        let selection = selection.clone();
        let is_selected = selected() == Some(layer);
        let lsurface_style = if is_selected {
            format!("background:{};box-shadow:inset 0 0 0 2px var(--way-inspector);", row.color)
        } else {
            format!("background:{};", row.color)
        };
        let glyph = |bit: u8, label: &'static str| {
            let (hidden, solo, locked) = attrs.read()[i];
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
        rsx!(
            div { class: "lrow",
                span {
                    class: "lsurface",
                    style: "{lsurface_style}",
                    onclick: move |_| {
                        selection.set(Some(layer));
                        selected.set(Some(layer));
                    },
                    "{row.name}"
                }
                div { class: "lctrl",
                    {glyph(0, "M")}
                    {glyph(1, "S")}
                    {glyph(2, "L")}
                }
            }
        )
    });

    let timecode = fmt_timecode(clock.now_sec());
    let layer_count = layer_rows_data.len();

    rsx!(
        div { id: "timelineshell",
            div { id: "tp",
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
            div { id: "timeline",
                div { id: "layers",
                    div { class: "lhead", "OBJECT" }
                    {layer_rows}
                }
                object { "data": timeline_attr }
            }
        }
    )
}
