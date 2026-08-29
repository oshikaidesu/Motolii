use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;

use crate::fixture::{inspector_data_from_doc, InspectorData, PropRow};
use crate::playback::Clock;
use motolii_store::{Document, LayerId, RationalTime};

pub fn prop_row(p: &PropRow) -> Element {
    let cells = p.cells.iter().zip(p.dims).map(|(c, dim)| {
        let class = if c.is_empty() {
            "v blank"
        } else if dim {
            "v z"
        } else {
            "v"
        };
        rsx!(span { class: "{class}", "{c}" })
    });
    let key_class = if p.keyed { "glyph on" } else { "glyph" };
    rsx!(
        div { class: "prow",
            span { class: "n", "{p.label}" }
            {cells}
            span { class: "{key_class}", if p.keyed { "◆" } else { "◇" } }
        }
    )
}

pub fn inspector_panel(
    doc: &Arc<Mutex<Document>>,
    selection: Option<LayerId>,
    clock: &Clock,
) -> Element {
    let empty = InspectorData {
        ident_name: "No selection".to_string(),
        ident_sub: String::new(),
        transform: Vec::new(),
        appearance: Vec::new(),
        has_effects: false,
    };
    let t = RationalTime::try_new((clock.now_sec() * 3000.0) as i64, 3000).unwrap_or(RationalTime::ZERO);
    let data = match selection {
        Some(layer) => inspector_data_from_doc(&doc.lock().unwrap().view(), layer, t),
        None => empty,
    };
    let inspector = &data;

    let transform_rows = inspector.transform.iter().map(prop_row);
    let appearance_rows = inspector.appearance.iter().map(prop_row);
    let fx_label = if inspector.has_effects { "" } else { "No shared FX" };

    rsx!(
        div { id: "inspector",
            div { class: "ptitle",
                span { class: "way", style: "background:var(--way-inspector);" }
                "Inspector"
                em { "solid" }
            }
            div { class: "ident",
                div {
                    b { "{inspector.ident_name}" }
                    span { class: "sub", "{inspector.ident_sub}" }
                }
                div { class: "sp",
                    span { class: "glyph", "M" }
                    span { class: "glyph on", "S" }
                }
            }
            div { class: "cols",
                span { class: "pn", "Property" }
                span { "X" }
                span { "Y" }
                span { "Z" }
                span { class: "k", "Key" }
            }
            div { class: "sec", "TRANSFORM" }
            {transform_rows}
            div { class: "sec", "APPEARANCE" }
            {appearance_rows}
            div { class: "sec", "EFFECTS" }
            div { class: "prow",
                span { class: "n empty", "{fx_label}" }
                span { class: "v blank", "" }
                span { class: "v blank", "" }
                span { class: "v blank", "" }
                span { "" }
            }
            div { class: "hint", "Drag to scrub · double-click to type · Esc to cancel" }
        }
    )
}
