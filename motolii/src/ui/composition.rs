//! 枠(Composition)の設定。寸法・fps・尺・背景は作品の物なので Document に書く(Intent::SetComposition)。
//! AE の Composition Settings(⌘K)、Premiere の Sequence Settings、CapCut の比率。
//! **9:16 と曲の長さの尺**が無ければ歌詞動画は 1 本も出ない(第 5 波 L1)。

use dioxus_native::prelude::*;

use crate::doc::store::{Composition, Fps, Intent};
use crate::ui::semantic_menu::SemanticButton;
use crate::ui::session::Session;

/// 比率の preset。幅×高さ。
const PRESETS: &[(&str, u32, u32)] = &[("16:9", 1920, 1080), ("9:16", 1080, 1920), ("1:1", 1080, 1080), ("4K", 3840, 2160)];

#[component]
pub(super) fn CompositionSheet(session: Session, revision: Signal<u32>) -> Element {
    let _ = revision();
    let current = session.doc.lock().unwrap().view().composition().ok().flatten();
    let Some(current) = current else {
        return rsx!(div { class: "settings-sheet", div { class: "sec", "Composition" } div { class: "rcount", "No composition" } });
    };
    let fps_value = current.fps.as_f64();
    let seconds = current.duration_frames as f64 / fps_value.max(1e-9);
    let longest = longest_layer_end_sec(&session);
    let write = {
        let session = session.clone();
        move |next: Composition| {
            let mut revision = revision;
            match session.doc.lock().unwrap().apply(Intent::SetComposition(next)) {
                Ok(_) => *revision.write() += 1,
                Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
            }
        }
    };
    let size_row = |label: &'static str, value: u32, apply: fn(&Composition, u32) -> Composition| {
        let base = current.clone();
        let write = write.clone();
        rsx!(div { class: "prow",
            span { class: "pname", "{label}" }
            input {
                class: "csheet-in",
                r#type: "text",
                value: "{value}",
                onchange: move |evt: FormEvent| {
                    if let Ok(v) = evt.value().trim().parse::<u32>() {
                        if (16..=8192).contains(&v) {
                            write(apply(&base, v));
                        }
                    }
                },
            }
        })
    };
    rsx!(
        div { class: "settings-sheet",
            div { class: "sec", "Composition" }
            div { class: "prow",
                span { class: "pname", "Preset" }
                div { class: "zoomctl", role: "group", aria_label: "Aspect presets",
                    for (label , w , h) in PRESETS.iter().copied() {
                        SemanticButton {
                            class: if current.width == w && current.height == h { "chip on" } else { "chip" },
                            selected: current.width == w && current.height == h,
                            onclick: {
                                let base = current.clone();
                                let write = write.clone();
                                move |_| write(Composition { width: w, height: h, ..base.clone() })
                            },
                            "{label}"
                        }
                    }
                }
            }
            {size_row("Width", current.width, |c, v| Composition { width: v, ..c.clone() })}
            {size_row("Height", current.height, |c, v| Composition { height: v, ..c.clone() })}
            div { class: "prow",
                span { class: "pname", "Frame rate" }
                div { class: "zoomctl", role: "group", aria_label: "Frame rate",
                    for fps in [24u32, 25, 30, 60] {
                        SemanticButton {
                            class: if (fps_value - fps as f64).abs() < 1e-6 { "chip on" } else { "chip" },
                            selected: (fps_value - fps as f64).abs() < 1e-6,
                            onclick: {
                                let base = current.clone();
                                let write = write.clone();
                                move |_| {
                                    let Ok(next_fps) = Fps::try_new(fps as i64, 1) else { return };
                                    // 尺は秒で保つ。fps を変えてもコマ数で伸び縮みしない。
                                    let frames = (base.duration_frames as f64 / base.fps.as_f64() * fps as f64).round() as i64;
                                    write(Composition { fps: next_fps, duration_frames: frames.max(1), ..base.clone() })
                                }
                            },
                            "{fps}"
                        }
                    }
                }
            }
            div { class: "prow",
                span { class: "pname", "Duration" }
                input {
                    class: "csheet-in",
                    r#type: "text",
                    value: "{format_duration(seconds)}",
                    title: "m:ss or seconds",
                    onchange: {
                        let base = current.clone();
                        let write = write.clone();
                        move |evt: FormEvent| {
                            let Some(secs) = parse_duration(&evt.value()) else { return };
                            let frames = (secs * base.fps.as_f64()).round() as i64;
                            if frames >= 1 {
                                write(Composition { duration_frames: frames, ..base.clone() });
                            }
                        }
                    },
                }
                if let Some(end) = longest {
                    if (end - seconds).abs() > 0.5 / fps_value {
                        SemanticButton {
                            class: "chip",
                            title: "Set the duration to the end of the longest layer",
                            onclick: {
                                let base = current.clone();
                                let write = write.clone();
                                move |_| {
                                    let frames = (end * base.fps.as_f64()).round() as i64;
                                    write(Composition { duration_frames: frames.max(1), ..base.clone() })
                                }
                            },
                            "Fit to layers · {format_duration(end)}"
                        }
                    }
                }
            }
        }
    )
}

/// 一番遅く終わる層の尻(秒)。曲を入れたら尺をそこへ伸ばす為の物。
fn longest_layer_end_sec(session: &Session) -> Option<f64> {
    let d = session.doc.lock().unwrap();
    let view = d.view();
    let fps = view.composition().ok().flatten()?.fps.as_f64();
    view.layers()
        .into_iter()
        .filter_map(|l| view.meta(l).ok().flatten())
        .map(|m| (m.timing.start + m.timing.duration) as f64 / fps)
        .fold(None, |acc: Option<f64>, v| Some(acc.map_or(v, |a| a.max(v))))
}

/// `m:ss` / `m:ss.s` / `秒`。
pub(super) fn parse_duration(text: &str) -> Option<f64> {
    let t = text.trim();
    if let Some((m, s)) = t.split_once(':') {
        let m: f64 = m.trim().parse().ok()?;
        let s: f64 = s.trim().parse().ok()?;
        return Some(m * 60.0 + s).filter(|v| *v > 0.0);
    }
    t.parse::<f64>().ok().filter(|v| *v > 0.0)
}

pub(super) fn format_duration(secs: f64) -> String {
    let whole = secs.floor() as i64;
    let frac = secs - whole as f64;
    let base = format!("{}:{:02}", whole / 60, whole % 60);
    if frac > 1e-3 {
        format!("{base}{}", format!("{frac:.2}").trim_start_matches('0'))
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_round_trip() {
        assert_eq!(parse_duration("3:24"), Some(204.0));
        assert_eq!(parse_duration("90"), Some(90.0));
        assert_eq!(parse_duration("0"), None);
        assert_eq!(format_duration(204.0), "3:24");
        assert_eq!(format_duration(20.5), "0:20.50");
    }
}
