//! 4ジェスチャの**判断部分だけ**を、窓を開かずに確かめる。
//!
//! ここが成立するのは iced 側の構造のおかげである。`update()` はモデルと message しか
//! 触らず、`hit_test` / `zoom_at` / `snap` は純関数なので、`egui::Context` にあたる物を
//! 用意しなくてもテストが書ける。egui 版の同じ判断は 1,724 行の `show(&mut self, ui)` の
//! 中に埋まっていて、`ui` 無しでは呼べない。

use crate::app::{App, Drag};
use crate::message::TimelineMsg;
use crate::model::*;

fn app_with(clips: Vec<Clip>, tracks: usize) -> App {
    let mut app = App::new(1);
    app.timeline = Timeline { tracks, clips };
    app.playhead = 0;
    app
}

fn clip(id: ClipId, track: usize, start: i64, len: i64) -> Clip {
    Clip {
        id,
        track,
        start,
        len,
    }
}

// ── hit test / trim 幅 ─────────────────────────────────────────────────────

#[test]
fn 端は左右それぞれ8px_内側は本体() {
    let app = app_with(vec![clip(0, 0, 0, 100)], 1);
    // px_per_frame = 4.0 なので clip は x = 0..400。
    let y = RULER_H + 5.0;
    assert_eq!(
        hit_test(&app.timeline, &app.viewport, 1.0, y),
        Hit::Clip {
            id: 0,
            grab: Grab::LeftEdge
        }
    );
    assert_eq!(
        hit_test(&app.timeline, &app.viewport, 399.0, y),
        Hit::Clip {
            id: 0,
            grab: Grab::RightEdge
        }
    );
    assert_eq!(
        hit_test(&app.timeline, &app.viewport, 200.0, y),
        Hit::Clip {
            id: 0,
            grab: Grab::Body
        }
    );
    // 8px ちょうどは端、8.1px は本体。
    assert!(matches!(
        hit_test(&app.timeline, &app.viewport, 8.0, y),
        Hit::Clip {
            grab: Grab::LeftEdge,
            ..
        }
    ));
    assert!(matches!(
        hit_test(&app.timeline, &app.viewport, 8.1, y),
        Hit::Clip {
            grab: Grab::Body,
            ..
        }
    ));
}

#[test]
fn 細い_clip_には端を作らない() {
    // 幅 5 フレーム × 4px = 20px < TRIM_MIN_BAR_W(24px)。
    let app = app_with(vec![clip(0, 0, 0, 5)], 1);
    let y = RULER_H + 5.0;
    for x in [0.5, 10.0, 19.5] {
        assert!(
            matches!(
                hit_test(&app.timeline, &app.viewport, x, y),
                Hit::Clip {
                    grab: Grab::Body,
                    ..
                }
            ),
            "x={x} が端になってしまった"
        );
    }
}

#[test]
fn ruler帯と行の隙間を取り違えない() {
    let app = app_with(vec![clip(0, 1, 0, 100)], 3);
    assert_eq!(hit_test(&app.timeline, &app.viewport, 50.0, 3.0), Hit::Ruler);
    // 行0の直下の 1px の隙間。
    let gap_y = RULER_H + ROW_H + 0.5;
    assert_eq!(
        hit_test(&app.timeline, &app.viewport, 50.0, gap_y),
        Hit::Empty
    );
}

// ── 移動 ───────────────────────────────────────────────────────────────────

#[test]
fn 移動はフレームグリッドへ吸い付く() {
    let mut app = app_with(vec![clip(0, 0, 10, 20)], 1);
    app.update(TimelineMsg::ClipGrabbed {
        id: 0,
        grab: Grab::Body,
        at_frame: 15.0,
        additive: false,
    });
    // 3.4 フレーム分ずらす → +3 に丸まる。
    app.update(TimelineMsg::PointerMoved { frame: 18.4 });
    assert_eq!(app.timeline.get(0).unwrap().start, 13);
    // 3.6 なら +4。
    app.update(TimelineMsg::PointerMoved { frame: 18.6 });
    assert_eq!(app.timeline.get(0).unwrap().start, 14);
}

#[test]
fn 移動を往復させても丸め誤差が溜まらない() {
    let mut app = app_with(vec![clip(0, 0, 10, 20)], 1);
    app.update(TimelineMsg::ClipGrabbed {
        id: 0,
        grab: Grab::Body,
        at_frame: 15.0,
        additive: false,
    });
    // 押した点からの絶対差分で置くので、何度往復しても出発点に戻る。
    for i in 0..200 {
        app.update(TimelineMsg::PointerMoved {
            frame: 15.0 + (i as f64 * 0.37).sin() * 9.0,
        });
    }
    app.update(TimelineMsg::PointerMoved { frame: 15.0 });
    assert_eq!(app.timeline.get(0).unwrap().start, 10);
}

#[test]
fn 複数選択は相対配置を保ったまま動く() {
    let mut app = app_with(vec![clip(0, 0, 10, 20), clip(1, 1, 100, 20)], 2);
    app.update(TimelineMsg::ClipGrabbed {
        id: 0,
        grab: Grab::Body,
        at_frame: 15.0,
        additive: true,
    });
    app.update(TimelineMsg::PointerReleased);
    app.update(TimelineMsg::ClipGrabbed {
        id: 1,
        grab: Grab::Body,
        at_frame: 105.0,
        additive: true,
    });
    assert_eq!(app.selection.len(), 2);
    app.update(TimelineMsg::PointerMoved { frame: 130.0 });
    assert_eq!(app.timeline.get(0).unwrap().start, 35);
    assert_eq!(app.timeline.get(1).unwrap().start, 125);
}

#[test]
fn 左へ押し込んでも集団の相対配置が崩れない() {
    let mut app = app_with(vec![clip(0, 0, 5, 20), clip(1, 1, 40, 20)], 2);
    app.selection.insert(0);
    app.selection.insert(1);
    // 既に選択済みなので素のクリックで掴む(Cmd を足すと選択が外れてしまう)。
    app.update(TimelineMsg::ClipGrabbed {
        id: 1,
        grab: Grab::Body,
        at_frame: 45.0,
        additive: false,
    });
    // 大きく左へ。先頭が 0 で止まり、後続は 35 の差を保つ。
    app.update(TimelineMsg::PointerMoved { frame: -500.0 });
    assert_eq!(app.timeline.get(0).unwrap().start, 0);
    assert_eq!(app.timeline.get(1).unwrap().start, 35);
}

#[test]
fn 選択済みの_clip_を素で押しても選択が潰れない() {
    let mut app = app_with(vec![clip(0, 0, 0, 20), clip(1, 1, 0, 20)], 2);
    app.selection.insert(0);
    app.selection.insert(1);
    app.update(TimelineMsg::ClipGrabbed {
        id: 1,
        grab: Grab::Body,
        at_frame: 5.0,
        additive: false,
    });
    assert_eq!(app.selection.len(), 2, "複数移動が始められなくなる");
}

// ── トリム ─────────────────────────────────────────────────────────────────

#[test]
fn 右端トリムは長さだけを変える() {
    let mut app = app_with(vec![clip(0, 0, 10, 20)], 1);
    app.update(TimelineMsg::ClipGrabbed {
        id: 0,
        grab: Grab::RightEdge,
        at_frame: 30.0,
        additive: false,
    });
    app.update(TimelineMsg::PointerMoved { frame: 37.0 });
    let c = *app.timeline.get(0).unwrap();
    assert_eq!((c.start, c.len), (10, 27));
}

#[test]
fn 左端トリムは頭と長さを逆向きに動かす() {
    let mut app = app_with(vec![clip(0, 0, 10, 20)], 1);
    app.update(TimelineMsg::ClipGrabbed {
        id: 0,
        grab: Grab::LeftEdge,
        at_frame: 10.0,
        additive: false,
    });
    app.update(TimelineMsg::PointerMoved { frame: 16.0 });
    let c = *app.timeline.get(0).unwrap();
    assert_eq!((c.start, c.end()), (16, 30), "出し点が動いてはいけない");
}

#[test]
fn トリムは1フレーム未満に潰れないし負にも出ない() {
    let mut app = app_with(vec![clip(0, 0, 10, 20)], 1);
    app.update(TimelineMsg::ClipGrabbed {
        id: 0,
        grab: Grab::LeftEdge,
        at_frame: 10.0,
        additive: false,
    });
    app.update(TimelineMsg::PointerMoved { frame: 999.0 });
    assert_eq!(app.timeline.get(0).unwrap().len, 1);
    app.update(TimelineMsg::PointerMoved { frame: -999.0 });
    assert_eq!(app.timeline.get(0).unwrap().start, 0);

    let mut app = app_with(vec![clip(0, 0, 10, 20)], 1);
    app.update(TimelineMsg::ClipGrabbed {
        id: 0,
        grab: Grab::RightEdge,
        at_frame: 30.0,
        additive: false,
    });
    app.update(TimelineMsg::PointerMoved { frame: -999.0 });
    assert_eq!(app.timeline.get(0).unwrap().len, 1);
}

// ── スクラブ ───────────────────────────────────────────────────────────────

#[test]
fn スクラブはフレームへ丸めて負に出ない() {
    let mut app = app_with(vec![], 1);
    app.update(TimelineMsg::ScrubStarted { frame: 12.6 });
    assert_eq!(app.playhead, 13);
    app.update(TimelineMsg::PointerMoved { frame: -4.0 });
    assert_eq!(app.playhead, 0);
    app.update(TimelineMsg::PointerReleased);
    // 離した後の移動は playhead を動かさない。
    app.update(TimelineMsg::PointerMoved { frame: 400.0 });
    assert_eq!(app.playhead, 0);
}

// ── zoom / パン / 縦スクロール ────────────────────────────────────────────

#[test]
fn zoomはカーソル下のフレームを動かさない() {
    let mut app = app_with(vec![], 1);
    app.viewport.scroll_x = 37.5;
    let anchor_x = 250.0;
    let before = app.viewport.x_to_frame(anchor_x);
    for _ in 0..8 {
        app.update(TimelineMsg::ZoomedAt {
            anchor_x,
            notches: 1.0,
        });
        let after = app.viewport.x_to_frame(anchor_x);
        assert!(
            (after - before).abs() < 1e-6,
            "掴んだ時刻が動いた: {before} -> {after}"
        );
    }
}

#[test]
fn zoomは上限と下限で止まる() {
    let mut app = app_with(vec![], 1);
    for _ in 0..200 {
        app.update(TimelineMsg::ZoomedAt {
            anchor_x: 100.0,
            notches: 1.0,
        });
    }
    assert_eq!(app.viewport.px_per_frame, PX_PER_FRAME_MAX);
    assert!(app.at_zoom_ceiling());
    for _ in 0..400 {
        app.update(TimelineMsg::ZoomedAt {
            anchor_x: 100.0,
            notches: -1.0,
        });
    }
    assert_eq!(app.viewport.px_per_frame, PX_PER_FRAME_MIN);
}

#[test]
fn 横パンは負に出ない_縦スクロールは中身の高さで止まる() {
    let mut app = app_with(vec![], 8);
    app.update(TimelineMsg::PannedX { notches: 20.0 });
    assert_eq!(app.viewport.scroll_x, 0.0);

    app.update(TimelineMsg::ScrolledY { notches: -100.0 });
    let max = 8.0 * (ROW_H + ROW_GAP) - ROW_H;
    assert_eq!(app.viewport.scroll_y, max);
    app.update(TimelineMsg::ScrolledY { notches: 100.0 });
    assert_eq!(app.viewport.scroll_y, 0.0);
}

// ── 矢印キー(製品 F-04 と同じ意味)──────────────────────────────────────

#[test]
fn 矢印は選択があれば_clip_無ければ_playhead_を動かす() {
    let mut app = app_with(vec![clip(0, 0, 10, 20)], 1);
    app.playhead = 50;

    // 選択なし → playhead。
    app.update(TimelineMsg::Nudged { frames: 1 });
    assert_eq!(app.playhead, 51);
    app.update(TimelineMsg::Nudged { frames: -10 });
    assert_eq!(app.playhead, 41);
    assert_eq!(app.timeline.get(0).unwrap().start, 10);

    // 選択あり → clip。
    app.selection.insert(0);
    app.update(TimelineMsg::Nudged { frames: 10 });
    assert_eq!(app.timeline.get(0).unwrap().start, 20);
    assert_eq!(app.playhead, 41, "playhead は動いてはいけない");
}

#[test]
fn 矢印でも左端で止まる() {
    let mut app = app_with(vec![clip(0, 0, 3, 20), clip(1, 1, 8, 20)], 2);
    app.selection.insert(0);
    app.selection.insert(1);
    app.update(TimelineMsg::Nudged { frames: -10 });
    assert_eq!(app.timeline.get(0).unwrap().start, 0);
    assert_eq!(app.timeline.get(1).unwrap().start, 5);

    let mut app = app_with(vec![], 1);
    app.playhead = 3;
    app.update(TimelineMsg::Nudged { frames: -10 });
    assert_eq!(app.playhead, 0);
}

// ── ジェスチャの寿命 ───────────────────────────────────────────────────────

#[test]
fn cmdクリックで選択を外したときはドラッグを始めない() {
    let mut app = app_with(vec![clip(0, 0, 10, 20), clip(1, 1, 40, 20)], 2);
    app.selection.insert(0);
    app.selection.insert(1);
    // Cmd+クリックで 1 を選択から外す。
    app.update(TimelineMsg::ClipGrabbed {
        id: 1,
        grab: Grab::Body,
        at_frame: 45.0,
        additive: true,
    });
    assert!(!app.selection.contains(&1));
    assert!(app.drag.is_none(), "外した物を掴んだことにしてはいけない");
    app.update(TimelineMsg::PointerMoved { frame: 200.0 });
    assert_eq!(app.timeline.get(0).unwrap().start, 10, "残りの選択が動いた");
}

#[test]
fn 離したらジェスチャは残らない() {
    let mut app = app_with(vec![clip(0, 0, 10, 20)], 1);
    app.update(TimelineMsg::ClipGrabbed {
        id: 0,
        grab: Grab::Body,
        at_frame: 15.0,
        additive: false,
    });
    assert!(matches!(app.drag, Some(Drag::Move { .. })));
    app.update(TimelineMsg::PointerReleased);
    assert!(app.drag.is_none());
    // 離した後の移動はモデルを触らない。
    app.update(TimelineMsg::PointerMoved { frame: 900.0 });
    assert_eq!(app.timeline.get(0).unwrap().start, 10);
}

#[test]
fn デモデータは重ならない() {
    let tl = Timeline::demo(500);
    assert_eq!(tl.clips.len(), 500);
    for t in 0..tl.tracks {
        let mut row: Vec<&Clip> = tl.clips.iter().filter(|c| c.track == t).collect();
        row.sort_by_key(|c| c.start);
        for w in row.windows(2) {
            assert!(w[0].end() <= w[1].start, "clip が重なっている: {w:?}");
        }
    }
}
