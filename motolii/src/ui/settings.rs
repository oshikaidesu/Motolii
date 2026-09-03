use dioxus_native::prelude::*;

use crate::ui::semantic_menu::SemanticButton;
use crate::ui::session::Session;

/// 見る側の設定。**作品には入らない**物だけを置く。
/// 散らばっていると探せないので、窓の設定はここへ集める。
#[component]
pub(super) fn SettingsSheet(session: Session, scale_pct: Signal<u32>) -> Element {
    let dim = session.frame_dim.clone();
    let pct = use_signal(|| dim.load(std::sync::atomic::Ordering::Relaxed));
    let dim_step = move |dim: std::sync::Arc<std::sync::atomic::AtomicU32>, mut pct: Signal<u32>, by: i32| {
        let next = (pct() as i32 + by).clamp(0, 100) as u32;
        dim.store(next, std::sync::atomic::Ordering::Relaxed);
        pct.set(next);
    };
    let (dim_a, dim_b) = (dim.clone(), dim.clone());

    let scale_step = move |ui: std::sync::Arc<crate::ui::tokens::UiScale>, mut sig: Signal<u32>, by: i32| {
        let next = (sig() as i32 + by).clamp(50, 200) as u32;
        ui.set_percent(next);
        sig.set(ui.percent());
    };
    let (ui_a, ui_b) = (session.scale.clone(), session.scale.clone());

    rsx!(
        div { class: "settings-sheet",
            div { class: "sec", "View" }
            div { class: "prow",
                span { class: "pname", "Outside dim" }
                div { class: "zoomctl", role: "group", aria_label: "Outside dim",
                    SemanticButton { class: "zbtn", aria_label: "Decrease outside dim", onclick: move |_| dim_step(dim_a.clone(), pct, -5), "−" }
                    span { class: "zval", role: "status", "{pct()}%" }
                    SemanticButton { class: "zbtn", aria_label: "Increase outside dim", onclick: move |_| dim_step(dim_b.clone(), pct, 5), "+" }
                }
            }
            div { class: "sec", "Window" }
            div { class: "prow",
                span { class: "pname", "Scale" }
                div { class: "zoomctl", role: "group", aria_label: "Interface scale",
                    SemanticButton { class: "zbtn", aria_label: "Decrease interface scale", onclick: move |_| scale_step(ui_a.clone(), scale_pct, -5), "−" }
                    span { class: "zval", role: "status", "{scale_pct()}%" }
                    SemanticButton { class: "zbtn", aria_label: "Increase interface scale", onclick: move |_| scale_step(ui_b.clone(), scale_pct, 5), "+" }
                }
            }
        }
    )
}
