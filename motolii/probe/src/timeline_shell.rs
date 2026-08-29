use std::sync::Arc;

use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;

use crate::fixture::{fmt_timecode, LayerRow};
use crate::playback::Clock;

pub fn timeline_shell(
    clock: Arc<Clock>,
    mut playing: Signal<bool>,
    mut msl: Signal<std::collections::HashSet<(usize, u8)>>,
    layer_rows_data: &[LayerRow],
    timeline_attr: CustomWidgetAttr,
) -> Element {
    let layer_rows = layer_rows_data.iter().enumerate().map(|(i, row)| {
        let glyph = |bit: u8, label: &'static str| {
            let lit = msl.read().contains(&(i, bit));
            let class = if lit { "glyph lit" } else { "glyph" };
            rsx!(
                span {
                    class: "{class}",
                    onclick: move |_| {
                        let mut set = msl.write();
                        if !set.remove(&(i, bit)) {
                            set.insert((i, bit));
                        }
                    },
                    "{label}"
                }
            )
        };
        rsx!(
            div { class: "lrow",
                span { class: "lsurface", style: "background:{row.color};", "{row.name}" }
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
