//! 書き出しの sheet。範囲(全体 / 印から印)と 1 行のサマリを見てから保存先を選ぶ。
//! 出す前に「1080×1920 · 30 fps · 0:00–1:24 · 2520 frames」が読めれば、枠の間違いはここで気付く。

use dioxus_native::prelude::*;

use crate::ui::composition::format_duration;
use crate::ui::semantic_menu::{SemanticButton, SemanticControl};
use crate::ui::session::Session;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum ExportRange {
    All,
    /// 再生位置を挟む印から印まで(前が無ければ 0、後が無ければ尺の終わり)。
    BetweenMarkers,
}

/// 範囲をコマで解く。
pub(super) fn frames_for(session: &Session, range: ExportRange) -> Option<std::ops::Range<i64>> {
    let d = session.doc.lock().unwrap();
    let view = d.view();
    let comp = view.composition().ok().flatten()?;
    let total = comp.duration_frames;
    match range {
        ExportRange::All => Some(0..total),
        ExportRange::BetweenMarkers => {
            let now = session.clock.current_time().as_seconds_f64();
            let fps = comp.fps.as_f64();
            let marks: Vec<f64> = view
                .markers()
                .unwrap_or_default()
                .iter()
                .map(|m| m.time.as_seconds_f64())
                .collect();
            let start = marks.iter().copied().filter(|m| *m <= now).fold(0.0_f64, f64::max);
            let end = marks.iter().copied().filter(|m| *m > now).fold(f64::INFINITY, f64::min);
            let a = (start * fps).round() as i64;
            let b = if end.is_finite() { (end * fps).round() as i64 } else { total };
            Some(a.clamp(0, total)..b.clamp(0, total).max(a.clamp(0, total) + 1))
        }
    }
}

#[component]
pub(super) fn ExportSheet(
    session: Session,
    revision: Signal<u32>,
    poke: crate::ui::host::Poke,
    window: Option<std::sync::Arc<dyn dioxus_native::winit::window::Window>>,
    mut open: Signal<Option<crate::ui::semantic_menu::MenuId>>,
) -> Element {
    let _ = revision();
    let mut range = use_signal(|| ExportRange::All);
    let comp = session.doc.lock().unwrap().view().composition().ok().flatten();
    let Some(comp) = comp else {
        return rsx!(div { class: "settings-sheet", h3 { class: "sec", "Export" } div { class: "rcount", "No composition" } });
    };
    let frames = frames_for(&session, range()).unwrap_or(0..0);
    let fps = comp.fps.as_f64();
    let summary = format!(
        "{}×{} · {} fps · {}–{} · {} frames · MP4 H.264 + AAC",
        comp.width,
        comp.height,
        crate::ui::export_sheet::fps_label(comp.fps),
        format_duration(frames.start as f64 / fps),
        format_duration(frames.end as f64 / fps),
        frames.end - frames.start
    );
    let busy = session.export.is_active();
    rsx!(
        div { class: "settings-sheet",
            h3 { class: "sec", "Export" }
            div { class: "prow",
                span { class: "pname", "Range" }
                div { class: "zoomctl", role: "group", aria_label: "Export range",
                    for (label , which) in [("All", ExportRange::All), ("Marker to marker", ExportRange::BetweenMarkers)] {
                        SemanticButton {
                            class: if range() == which { "chip on" } else { "chip" },
                            selected: range() == which,
                            onclick: move |_| range.set(which),
                            "{label}"
                        }
                    }
                }
            }
            div { class: "prow", span { class: "pname", "Output" } span { class: "v", "{summary}" } }
            div { class: "vrow menu-section",
                SemanticControl {
                    label: "Export…",
                    disabled: busy,
                    hint: if busy { "Exporting" } else { "" },
                    onclick: {
                        let session = session.clone();
                        let poke = poke.clone();
                        let window = window.clone();
                        move |evt: Event<MouseData>| {
                            evt.stop_propagation();
                            open.set(None);
                            let Some(frames) = frames_for(&session, range()) else { return };
                            start_export(session.clone(), frames, poke.clone(), window.clone());
                        }
                    }
                }
            }
        }
    )
}

pub(super) fn fps_label(fps: crate::doc::store::Fps) -> String {
    if fps.den() == 1 {
        format!("{}", fps.num())
    } else {
        format!("{:.3}", fps.as_f64())
    }
}

/// 保存先を選んで走らせる。dialog の間は Choosing で二度押しを止める。
pub(super) fn start_export(
    session: Session,
    frames: std::ops::Range<i64>,
    poke: crate::ui::host::Poke,
    window: Option<std::sync::Arc<dyn dioxus_native::winit::window::Window>>,
) {
    if !session.export.choosing() {
        return;
    }
    // 出力名は作品名から(Premiere・Resolve)。無ければ Untitled。
    let export_name = session
        .project_path
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .map_or_else(|| "Untitled.mp4".to_owned(), |s| format!("{s}.mp4"));
    dioxus_core::spawn(async move {
        // 種の絞りを付けると、OS が拡張子を**もう一度**足して `x.mp4.mp4` になる。足すのはこちらの仕事。
        let picked = crate::ui::project::sheet(window.as_deref())
            .set_file_name(&export_name)
            .save_file()
            .await;
        let Some(file) = picked else {
            session.export.unchoose();
            poke.poke();
            return;
        };
        let mut out = file.path().to_path_buf();
        if out.extension().is_none_or(|e| !e.eq_ignore_ascii_case("mp4")) {
            out.set_extension("mp4");
        }
        session.export.unchoose();
        if let Err(error) = session.export.start(session.doc.clone(), out, frames, poke.clone()) {
            *session.project_notice.lock().unwrap() = error.clone();
            poke.poke();
            println!("PROBE room=export verdict=start-error {error}");
        }
    });
}
