//! P5: Blitz(dioxus-native)で Timeline の手触りが成立するか — 人手審判。
//!
//! 数値ではなく「掴んで動かして違和感が無いか」を見る。判定は利用者。
//!
//! 見るところ:
//!   1. clip 本体を掴んで左右に動かす — 指に追従するか、ズレ/遅れが無いか
//!   2. clip の端(trim handle)を掴んで伸縮 — 端を正確に掴めるか
//!   3. key(ダイヤ)を掴んで動かす — 小さい対象を掴めるか、clip より先に取れるか
//!   4. playhead を ruler で掴んでスクラブ
//!   5. カーソル言語 — clip=手, trim=左右リサイズ, key=指差し(F2の既決)
//!   6. drag 中に hit 外へ出てもカーソルと掴みが維持されるか(F2の既決)

use std::time::Instant;

use dioxus::prelude::*;

const TRACKS: usize = 8;
const CLIPS_PER_TRACK: usize = 3;
const KEYS_PER_TRACK: usize = 12;
const LANE_LEFT: f64 = 80.0;
const ROW_H: f64 = 28.0;
const ROWS_TOP: f64 = 26.0;
/// trim handle の掴み幅(px)。狭すぎると「触れそうで触れない」になる。
const TRIM_W: f64 = 6.0;
/// key の掴み半径(px)。F7 で 5.6px 視覚一致が既決。
const KEY_R: f64 = 6.0;

#[derive(Clone, Copy, PartialEq)]
enum Grab {
    ClipBody { tr: usize, c: usize, dx: f64 },
    TrimL { tr: usize, c: usize },
    TrimR { tr: usize, c: usize },
    Key { tr: usize, k: usize, dx: f64 },
    Playhead,
}

fn main() {
    dioxus_native::launch(app);
}

fn app() -> Element {
    // clip[tr][c] = (start_px, width_px)
    let mut clips = use_signal(|| {
        (0..TRACKS)
            .map(|tr| {
                (0..CLIPS_PER_TRACK)
                    .map(|c| ((c * 150 + tr * 12) as f64, 110.0))
                    .collect::<Vec<(f64, f64)>>()
            })
            .collect::<Vec<_>>()
    });
    // keys[tr][k] = x_px
    let mut keys = use_signal(|| {
        (0..TRACKS)
            .map(|tr| {
                (0..KEYS_PER_TRACK)
                    .map(|k| (k * 36 + tr * 5) as f64)
                    .collect::<Vec<f64>>()
            })
            .collect::<Vec<_>>()
    });
    let mut playhead = use_signal(|| 120.0f64);
    // ズーム(x方向のみ)と縦横スクロール。world座標 wx を持ち、画面へは
    //   screen_x = LANE_LEFT + wx * zoom - scroll_x
    // で射影する。key の見た目サイズはズームで変えない(Timelineの通例)。
    let mut zoom = use_signal(|| 1.0f64);
    let mut scroll_x = use_signal(|| 0.0f64);
    let mut scroll_y = use_signal(|| 0.0f64);
    // トラックパッドのピンチが届くかを見るためのログ
    let mut wheel_log = use_signal(|| "(まだホイール/ピンチ未検出)".to_string());
    // ホイールイベントに修飾キーが乗らない実測を受け、キーイベント側で状態を持つ。
    // これはキーイベントが届くかどうかの検証も兼ねる。
    let mut mod_zoom = use_signal(|| false);
    let mut key_log = use_signal(|| "(まだキー未検出)".to_string());
    let mut grab = use_signal(|| None::<Grab>);
    let mut hover = use_signal(|| "default".to_string());

    // フレーム時間(再レンダー間隔)
    let mut last = use_signal(|| None::<Instant>);
    let mut ms = use_signal(|| 0.0f64);
    let mut worst = use_signal(|| 0.0f64);
    let mut n = use_signal(|| 0u64);
    {
        let now = Instant::now();
        if let Some(p) = last() {
            let d = now.duration_since(p).as_secs_f64() * 1000.0;
            ms.set(d);
            let c = n() + 1;
            n.set(c);
            if c > 20 && d > worst() {
                worst.set(d);
            }
        }
        last.set(Some(now));
    }

    // どこを掴んだか判定する。key を clip より先に見る(小さい対象を優先)。
    // 画面座標 → world座標
    let to_world = move |x: f64| -> f64 { (x - LANE_LEFT + scroll_x()) / zoom() };

    let hit = move |x: f64, y: f64| -> Option<Grab> {
        if y < ROWS_TOP {
            return Some(Grab::Playhead);
        }
        let tr = ((y - ROWS_TOP + scroll_y()) / ROW_H).floor() as usize;
        if tr >= TRACKS {
            return None;
        }
        let lx = to_world(x);
        // 掴み判定は画面px基準にする(ズームしても掴みやすさを一定に保つ)
        let trim_w = TRIM_W / zoom();
        let key_r = KEY_R / zoom();
        for (k, kx) in keys.read()[tr].iter().enumerate() {
            if (lx - kx).abs() <= key_r {
                return Some(Grab::Key { tr, k, dx: lx - kx });
            }
        }
        for (c, (s, w)) in clips.read()[tr].iter().enumerate() {
            if lx >= *s - trim_w && lx <= *s + trim_w {
                return Some(Grab::TrimL { tr, c });
            }
            if lx >= s + w - trim_w && lx <= s + w + trim_w {
                return Some(Grab::TrimR { tr, c });
            }
            if lx > *s && lx < s + w {
                return Some(Grab::ClipBody { tr, c, dx: lx - s });
            }
        }
        None
    };

    let cursor_for = |g: Option<Grab>| match g {
        Some(Grab::TrimL { .. }) | Some(Grab::TrimR { .. }) => "ew-resize",
        Some(Grab::ClipBody { .. }) => "grab",
        Some(Grab::Key { .. }) => "pointer",
        Some(Grab::Playhead) => "ew-resize",
        None => "default",
    };

    let ph = playhead();
    let cur = hover();
    let grabbing = grab().is_some();

    rsx! {
        style { {CSS} }
        div {
            class: "wrap",
            tabindex: "0",
            onkeydown: move |e| {
                key_log.set(format!("[wrap] key={:?}", e.key()));
            },
            h1 { "P5: Timeline 手触り検証 — 掴んで動かしてください" }
            p { class: "hint",
                "clip本体=移動 / clipの端=trim / ダイヤ=key移動 / 上部ruler=playhead。"
                "掴んだまま外へ出てもカーソルと掴みが維持されるかも見てください。"
            }
            p { class: "stat",
                "frame: {ms():.2} ms / worst: {worst():.2} ms / renders: {n()} / "
                "掴み中: {grabbing} / cursor: {cur}"
            }
            p { class: "stat",
                "zoom: {zoom():.2}x / scrollX: {scroll_x():.0} / scrollY: {scroll_y():.0}"
            }
            p { class: "stat", "wheel: {wheel_log()}" }
            p { class: "stat", "key:   {key_log()}  (z=ズームイン x=ズームアウト)" }
            p { class: "hint",
                "診断: 下のテキスト欄をクリックしてキーを押すと [input] と出るはず。"
                "Timeline領域をクリックして押すと [tl]、外側なら [wrap]。"
                "どれが出るかで、Blitzがキーをどこへ配るかが分かる。"
            }
            input {
                r#type: "text",
                class: "sink",
                placeholder: "ここをクリックしてキーを押す",
                onkeydown: move |e| {
                    let m = e.modifiers();
                    key_log.set(format!(
                        "[input] key={:?} ctrl={} cmd={} shift={} alt={}",
                        e.key(), m.ctrl(), m.meta(), m.shift(), m.alt()
                    ));
                    if m.ctrl() || m.meta() { mod_zoom.set(true); }
                    match e.key() {
                        Key::Character(ref c) if c == "z" => { zoom.set((zoom() * 1.15).clamp(0.2, 8.0)); }
                        Key::Character(ref c) if c == "x" => { zoom.set((zoom() / 1.15).clamp(0.2, 8.0)); }
                        _ => {}
                    }
                },
                onkeyup: move |e| {
                    let m = e.modifiers();
                    if !(m.ctrl() || m.meta()) { mod_zoom.set(false); }
                },
            }

            div {
                class: "tl",
                style: "cursor: {cur}",
                onmousedown: move |e| {
                    let p = e.element_coordinates();
                    if let Some(g) = hit(p.x, p.y) {
                        grab.set(Some(g));
                        // 掴んだ瞬間にカーソルを固定する(F2: drag中はhit外でも維持)
                        hover.set(cursor_for(Some(g)).to_string());
                    }
                },
                onmousemove: move |e| {
                    let p = e.element_coordinates();
                    let lx = to_world(p.x);
                    match grab() {
                        Some(Grab::ClipBody { tr, c, dx }) => {
                            clips.write()[tr][c].0 = (lx - dx).max(0.0);
                        }
                        Some(Grab::TrimL { tr, c }) => {
                            let mut w = clips.write();
                            let (s, wd) = w[tr][c];
                            let right = s + wd;
                            let ns = lx.clamp(0.0, right - 12.0);
                            w[tr][c] = (ns, right - ns);
                        }
                        Some(Grab::TrimR { tr, c }) => {
                            let mut w = clips.write();
                            let (s, _) = w[tr][c];
                            w[tr][c].1 = (lx - s).max(12.0);
                        }
                        Some(Grab::Key { tr, k, dx }) => {
                            keys.write()[tr][k] = (lx - dx).max(0.0);
                        }
                        Some(Grab::Playhead) => playhead.set(lx.max(0.0)),
                        None => {
                            // hover 中だけカーソルを再計算する
                            hover.set(cursor_for(hit(p.x, p.y)).to_string());
                        }
                    }
                },
                onwheel: move |e| {
                    let d = e.delta().strip_units();
                    let m = e.modifiers();
                    wheel_log.set(format!(
                        "dx={:.1} dy={:.1} ctrl={} cmd={} shift={}",
                        d.x, d.y, m.ctrl(), m.meta(), m.shift()
                    ));
                    if m.ctrl() || m.meta() || mod_zoom() {
                        // ズーム。ポインタ位置を固定点にする(Timelineの通例)
                        let anchor_screen = e.element_coordinates().x;
                        let anchor_world = to_world(anchor_screen);
                        let z = (zoom() * (1.0 - d.y * 0.002)).clamp(0.2, 8.0);
                        zoom.set(z);
                        // anchor_world が同じ画面位置に来るよう scroll_x を解き直す
                        scroll_x.set(anchor_world * z - (anchor_screen - LANE_LEFT));
                    } else if m.shift() {
                        scroll_x.set((scroll_x() + d.y).max(0.0));
                    } else {
                        scroll_x.set((scroll_x() + d.x).max(0.0));
                        let max_y = (TRACKS as f64 * ROW_H - 200.0).max(0.0);
                        scroll_y.set((scroll_y() + d.y).clamp(0.0, max_y));
                    }
                },
                tabindex: "0",
                onkeydown: move |e| {
                    let m = e.modifiers();
                    key_log.set(format!(
                        "[tl] key={:?} ctrl={} cmd={} shift={} alt={}",
                        e.key(), m.ctrl(), m.meta(), m.shift(), m.alt()
                    ));
                    if m.ctrl() || m.meta() { mod_zoom.set(true); }
                    // 修飾キーが取れない環境でもズームの手触りを見られるように
                    // z / x を明示のズームキーにする。
                    match e.key() {
                        Key::Character(ref c) if c == "z" => {
                            zoom.set((zoom() * 1.15).clamp(0.2, 8.0));
                        }
                        Key::Character(ref c) if c == "x" => {
                            zoom.set((zoom() / 1.15).clamp(0.2, 8.0));
                        }
                        _ => {}
                    }
                },
                onkeyup: move |e| {
                    let m = e.modifiers();
                    if !(m.ctrl() || m.meta()) { mod_zoom.set(false); }
                },
                onmouseup: move |e| {
                    grab.set(None);
                    let p = e.element_coordinates();
                    // 離した後に再計算する(F2の既決)
                    hover.set(cursor_for(hit(p.x, p.y)).to_string());
                },

                div { class: "ruler",
                    for t in 0..14 {
                        div { class: "tick", style: "left: {LANE_LEFT + t as f64 * 50.0 * zoom() - scroll_x()}px", "{t * 10}" }
                    }
                }

                for tr in 0..TRACKS {
                    div { class: "track", key: "tr{tr}", style: "top: {ROWS_TOP + tr as f64 * ROW_H - scroll_y()}px",
                        div { class: "tname", "layer {tr}" }
                        for (c, (s, w)) in clips.read()[tr].iter().enumerate() {
                            div {
                                class: "clip",
                                key: "c{tr}-{c}",
                                style: "left: {LANE_LEFT + s * zoom() - scroll_x()}px; width: {w * zoom()}px",
                                "clip {c}"
                            }
                        }
                        for (k, kx) in keys.read()[tr].iter().enumerate() {
                            div {
                                class: "key",
                                key: "k{tr}-{k}",
                                style: "left: {LANE_LEFT + kx * zoom() - scroll_x() - 5.0}px",
                            }
                        }
                    }
                }

                div { class: "ph", style: "left: {LANE_LEFT + ph * zoom() - scroll_x()}px" }
            }
        }
    }
}

const CSS: &str = r#"
body { margin: 0; background: #2a2a2a; color: #d6d6d6;
       font-family: sans-serif; font-size: 12px; }
.wrap { padding: 12px; }
h1 { font-size: 14px; margin: 0 0 6px 0; color: #ffad56; }
.hint { color: #919191; margin: 0 0 6px 0; }
.stat { color: #96aadb; font-family: monospace; margin: 0 0 8px 0; }
.tl { position: relative; background: #242424; height: 280px; overflow: hidden;
      user-select: none; }
/* 自前でhit判定するので子はイベントを受けない。これが効かないと
   element_coordinates() が子要素基準になり、clipを押した時に
   y<ROWS_TOP と誤判定される(実測で判明)。 */
.ruler, .tick, .track, .tname, .clip, .key, .ph { pointer-events: none; }
.ruler { position: absolute; top: 0; left: 0; right: 0; height: 26px;
         background: #2f2f2f; }
.tick { position: absolute; top: 6px; color: #919191; font-size: 9px; }
.track { position: absolute; left: 0; right: 0; height: 28px;
         border-bottom: 1px solid #2f2f2f; }
.tname { position: absolute; left: 0; top: 0; width: 74px; height: 27px;
         color: #919191; padding: 7px 4px; background: #2f2f2f; }
.clip { position: absolute; top: 4px; height: 19px; background: #96aadb;
        color: #141414; font-size: 9px; padding: 3px 4px;
        border-left: 2px solid #6f8cc4; border-right: 2px solid #6f8cc4; }
.key { position: absolute; top: 8px; width: 10px; height: 10px;
       background: #ffad56; }
.sink { width: 260px; padding: 6px; background: #242424; color: #d6d6d6;
        border: 1px solid #464646; margin-top: 8px; }
.ph { position: absolute; top: 0; width: 1px; height: 280px; background: #e7e7e7; }
"#;
