use dioxus_native::prelude::*;

use crate::fixture::{InspectorData, PropRow};

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

pub fn inspector_panel(inspector: &InspectorData) -> Element {
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
