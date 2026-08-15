//! Blitz / dioxus-native 採否プローブ。spikes/ 完結で製品 Document/schema には触れない。
//!
//! 判定2点:
//!   P2  日本語 IME 4項目(`spikes/ime-acceptance` のチェックリストを流用) — 人手審判
//!   P3  Timeline 形状の DOM を毎フレーム更新したときの実測フレーム時間
//!
//! P1(自前 wgpu::Texture を DOM へ)は 0.7.10 で API を確認済みだが、
//! パネル用途では不要と判断したため本体から外した。所見は README を参照。

use std::time::Instant;

use dioxus::prelude::*;

/// P2: `spikes/ime-acceptance` と同一の4項目。
const IME_CHECKLIST: [&str; 4] = [
    "1. preedit下線表示 — ローマ字入力→変換前の未確定表示が出るか",
    "2. 候補ウィンドウ追従 — カーソル移動で候補が追従するか",
    "3. Enter未食い — 未確定のままEnter→下のログに出なければ合格",
    "4. 長文歌詞連続入力 — 下のサンプルを貼付/連続入力",
];

const LONG_LYRIC_SAMPLE: &str = "夜明けの街を歩きながら、君の言葉を思い出していた。";

/// P3: Timeline を模した規模。天井を探すため env で掃引できるようにする。
///   BLITZ_PROBE_SCALE=1  → 8 track / 12 clip / 24 key (約424ノード)
/// 倍率を上げるとノード数がほぼ線形に増える。
fn scale() -> usize {
    std::env::var("BLITZ_PROBE_SCALE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
        .max(1)
}
fn tracks() -> usize {
    8 * scale()
}
fn clips_per_track() -> usize {
    12
}
fn keys_per_track() -> usize {
    24
}
/// P3 測定モード。
///   left      … 各要素の `left:` を毎フレーム再計算(最悪ケース。初回測定と同じ)
///   transform … コンテナ1個に transform を掛け、子は据え置き(実装するならこちら)
fn transform_mode() -> bool {
    std::env::var("BLITZ_PROBE_MODE").map(|v| v == "transform").unwrap_or(false)
}
/// P3 自動駆動のフレーム数。
fn auto_frames() -> usize {
    std::env::var("BLITZ_PROBE_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(400)
}

fn main() {
    dioxus_native::launch(app);
}

fn app() -> Element {
    // P3: 再レンダー間隔。drag 中は概ねフレーム間隔に一致する。
    let mut last = use_signal(|| None::<Instant>);
    let mut last_ms = use_signal(|| 0.0f64);
    let mut max_ms = use_signal(|| 0.0f64);
    let mut renders = use_signal(|| 0u64);

    // playhead を drag すると Timeline 全体が作り直される。
    let mut playhead = use_signal(|| 30i32);
    let mut zoom = use_signal(|| 100i32);

    // P2
    let mut key_log = use_signal(Vec::<String>::new);
    let mut text = use_signal(String::new);

    // P3 自動駆動: 人の操作なしで playhead を動かし続け、
    // 統計が溜まったら stderr へ出す。scrub を手で掴んだのと同じ負荷になる。
    use_future(move || async move {
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        let mut samples: Vec<f64> = Vec::new();
        for i in 0..auto_frames() {
            playhead.set((i % 100) as i32);
            zoom.set(100 + ((i / 3) % 120) as i32);
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            let ms = last_ms();
            if i > 20 && ms > 0.0 {
                samples.push(ms);
            }
        }
        if samples.is_empty() {
            eprintln!("P3 RESULT: no samples");
            return;
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = samples.len();
        let mean = samples.iter().sum::<f64>() / n as f64;
        eprintln!(
            "P3 RESULT: nodes~{} samples={} mean={:.2}ms p50={:.2}ms p95={:.2}ms max={:.2}ms",
            tracks() * (clips_per_track() * 2 + keys_per_track()) + 40,
            n,
            mean,
            samples[n / 2],
            samples[n * 95 / 100],
            samples[n - 1],
        );
    });

    // 再レンダーのたびに間隔を測る。
    {
        let now = Instant::now();
        if let Some(prev) = last() {
            let ms = now.duration_since(prev).as_secs_f64() * 1000.0;
            last_ms.set(ms);
            let n = renders() + 1;
            renders.set(n);
            // 最初の数回は初期化コストなので最大値から除く。
            if n > 10 && ms > max_ms() {
                max_ms.set(ms);
            }
        }
        last.set(Some(now));
    }

    let node_estimate = tracks() * (clips_per_track() * 2 + keys_per_track()) + 40;
    let ph = playhead();
    let zm = zoom();

    rsx! {
        style { {CSS} }
        div { class: "wrap",
            h1 { "Blitz probe — P2 IME / P3 Timeline 形状の動的更新" }

            // ---- P3 ----
            section {
                h2 { "P3: Timeline を DOM で組んだときのフレーム時間" }
                p { class: "hint",
                    "playhead と zoom を掴んで動かしながら数値を見る。"
                    "16.7ms を大きく超え続けるなら、Timeline を DOM で持つのは苦しい。"
                }
                div { class: "ctl",
                    span { class: "k", "playhead" }
                    input {
                        r#type: "range", min: "0", max: "100", value: "{ph}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() { playhead.set(v); }
                        }
                    }
                    span { class: "k", "zoom" }
                    input {
                        r#type: "range", min: "40", max: "400", value: "{zm}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<i32>() { zoom.set(v); }
                        }
                    }
                }
                p { class: "stat",
                    "frame: {last_ms():.2} ms / max(warm): {max_ms():.2} ms / "
                    "renders: {renders()} / DOMノード概算: {node_estimate}"
                }

                // ---- Timeline 本体 ----
                div {
                    class: "tl",
                    // transform モードの核心: 子は据え置きで、この1ノードだけ毎フレーム変わる。
                    style: if transform_mode() {
                        format!("transform: translateX({}px) scaleX({})",
                            -(ph as f64) * 2.0, zm as f64 / 100.0)
                    } else {
                        String::new()
                    },
                    // ruler
                    div { class: "ruler",
                        for t in 0..21 {
                            div {
                                class: "tick",
                                style: if transform_mode() {
                                    format!("left: {}px", t * 25)
                                } else {
                                    format!("left: {}px", t * zm as usize / 4)
                                },
                                "{t * 5}"
                            }
                        }
                    }
                    // tracks
                    for tr in 0..tracks() {
                        div { class: "track", key: "tr{tr}",
                            div { class: "tname", "layer {tr}" }
                            div { class: "lane",
                                for c in 0..clips_per_track() {
                                    div {
                                        class: "clip",
                                        key: "c{tr}-{c}",
                                        style: if transform_mode() {
                                            format!("left: {}px; width: 33px", c * 7 + tr)
                                        } else {
                                            format!("left: {}px; width: {}px",
                                                (c * 7 + tr) * zm as usize / 100, zm as usize / 3)
                                        },
                                        "c{c}"
                                    }
                                }
                                for k in 0..keys_per_track() {
                                    div {
                                        class: "key",
                                        key: "k{tr}-{k}",
                                        style: if transform_mode() {
                                            format!("left: {}px", k * 4 + tr * 2)
                                        } else {
                                            format!("left: {}px", (k * 4 + tr * 2) * zm as usize / 100)
                                        },
                                    }
                                }
                            }
                        }
                    }
                    // playhead
                    div { class: "ph", style: "left: {ph as usize * zm as usize / 100}px" }
                }
            }

            // ---- P2 ----
            section {
                h2 { "P2: 日本語 IME (ime-acceptance の4項目)" }
                ul { class: "check",
                    for item in IME_CHECKLIST {
                        li { "{item}" }
                    }
                }
                p { class: "hint", "長文サンプル: {LONG_LYRIC_SAMPLE}" }
                input {
                    r#type: "text",
                    class: "ime",
                    placeholder: "ここに日本語を入力",
                    value: "{text}",
                    oninput: move |e| text.set(e.value()),
                    onkeydown: move |e| {
                        // 「Enter未食い」の判定材料。変換確定のEnterがここに出たら不合格。
                        let mut log = key_log.write();
                        log.push(format!("{:?}", e.key()));
                        if log.len() > 12 { log.remove(0); }
                    },
                }
                p { class: "stat", "入力値: {text}" }
                p { class: "stat", "keydown log: {key_log.read().join(\" | \")}" }
            }
        }
    }
}

const CSS: &str = r#"
body { margin: 0; background: #2a2a2a; color: #d6d6d6;
       font-family: sans-serif; font-size: 12px; }
.wrap { padding: 12px; }
h1 { font-size: 14px; margin: 0 0 10px 0; color: #ffad56; }
h2 { font-size: 12px; margin: 0 0 6px 0; color: #ffad56; }
section { margin-bottom: 16px; padding: 10px; background: #363636; border-radius: 4px; }
.hint { color: #919191; margin: 0 0 6px 0; }
.stat { color: #96aadb; font-family: monospace; margin: 6px 0; }
.ctl { display: flex; align-items: center; gap: 8px; }
.k { color: #919191; }
.check { margin: 0 0 6px 0; padding-left: 18px; }
.ime { width: 320px; padding: 6px; background: #242424; color: #d6d6d6;
       border: 1px solid #464646; }

.tl { position: relative; background: #242424; padding: 0; margin-top: 6px;
      height: 240px; overflow: hidden; }
.ruler { position: relative; height: 18px; background: #2f2f2f; }
.tick { position: absolute; top: 3px; color: #919191; font-size: 9px; }
.track { display: flex; height: 26px; border-bottom: 1px solid #2f2f2f; }
.tname { width: 70px; color: #919191; padding: 6px 4px; background: #2f2f2f; }
.lane { position: relative; flex: 1; }
.clip { position: absolute; top: 4px; height: 18px; background: #96aadb;
        color: #141414; font-size: 9px; padding: 2px; }
.key { position: absolute; top: 9px; width: 7px; height: 7px; background: #ffad56; }
.ph { position: absolute; top: 0; width: 1px; height: 240px; background: #e7e7e7; }
"#;
