use dioxus_native::prelude::*;

use crate::fixture::UiData;

pub fn browser_panel(ui: &UiData) -> Element {
    let asset_cards = ui.assets.iter().map(|a| {
        rsx!(
            div { class: "tcard",
                div { class: "thumb", style: "background:{a.thumb};" }
                span { class: "tname", "{a.name}" }
                span { class: "tmeta", "{a.kind}" }
            }
        )
    });
    let first_asset = ui.assets.first().map(|a| a.name.clone()).unwrap_or_default();
    let asset_count = ui.assets.len();

    rsx!(
        div { id: "browser",
            div { class: "ptitle",
                span { class: "way", style: "background:var(--way-browser);" }
                "Browser"
                em { "LOCAL LIBRARY" }
            }
            div { class: "btoolbar",
                span { class: "hbtn", "‹" }
                span { class: "hbtn", "›" }
                span { class: "search", "Search files and tags" }
                span { class: "tbtn", "Filters" }
                span { class: "tbtn", "Tags" }
            }
            div { class: "btabs",
                span { class: "btab on", "Media" }
                span { class: "btab", "Effects" }
                span { class: "btab", "Create" }
                span { class: "btab", "Panels" }
            }
            div { class: "bwork",
                div { class: "bside",
                    div { class: "sh", "LIBRARY" }
                    div { class: "srow on", "All media" }
                    div { class: "srow", "Video" }
                    div { class: "srow", "Images" }
                    div { class: "srow", "Audio" }
                    div { class: "sh", "PLACES" }
                    div { class: "srow", "Comp 1" }
                }
                div { class: "bresults",
                    div { class: "rhead",
                        div {
                            b { "All media" }
                            span { class: "sub", "Library · fixture" }
                        }
                        div { class: "sp",
                            span { class: "glyph", "▦" }
                            span { class: "glyph on", "▤" }
                        }
                    }
                    div { class: "rcount",
                        "Results"
                        em { "{asset_count}" }
                    }
                    div { class: "tgrid", {asset_cards} }
                    div { class: "bfoot",
                        span { class: "dot", style: "background:var(--accent);" }
                        "{first_asset}"
                        em { "Edit tags" }
                    }
                }
            }
        }
    )
}
