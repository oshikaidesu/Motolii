//! 進捗コールバックの契約 — フレーム数と一致する単調増加、途中キャンセルで止まる、
//! `export_range_with_cancel`(コールバックなし版)は退行しない。

use motolii_core::Fps;
use motolii_engine::Engine;
use motolii_export::{
    export_range_with_cancel, export_range_with_progress, export_with_progress, Cancel,
    ExportError, ExportJob, ExportProgress,
};
use motolii_media::probe;
use motolii_store::LayerTiming;
use motolii_store::{Composition, Document, Intent, LayerId, LayerMeta, LayerSource};
use motolii_testkit::{ffmpeg_or_skip, tmp_dir};

const W: u32 = 64;
const H: u32 = 64;
const FRAMES: i64 = 30;

fn fps() -> Fps {
    Fps::try_new(30, 1).unwrap()
}

/// 動く白い板(他の export 試験と同じ fixture)。
fn moving_document() -> Document {
    use motolii_store::{property, Interp, Keyframe, KeyframeTrack, PropertyId, RationalTime, Value};

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: fps(),
        duration_frames: FRAMES,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [255, 255, 255, 255],
                width: 16,
                height: 16,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    let mut x = KeyframeTrack::new();
    x.insert(Keyframe {
        t: RationalTime::try_new(0, 30).unwrap(),
        value: Value::Vec2([0.0, 0.0]),
        interp: Interp::Linear,
        spatial: None,
    });
    x.insert(Keyframe {
        t: RationalTime::try_new(30, 30).unwrap(),
        value: Value::Vec2([48.0, 0.0]),
        interp: Interp::Linear,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(property::POSITION).unwrap(),
        track: x,
    })
    .unwrap();
    doc
}

/// **進捗は単調増加でフレーム数と一致する**。
///
/// コールバックはフレームを1本書くたびに1回呼ばれるので、呼ばれた回数は
/// `frames_written` と一致し、値は `1, 2, .., total` と厳密に単調増加する
/// (飛びなし・重複なし)。
#[test]
fn progress_is_monotonic_and_matches_frame_count() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-progress-monotonic");
    let out = dir.join("progress.mp4");
    let doc = moving_document();
    let mut engine = Engine::new().unwrap();

    let job = ExportJob {
        out_path: out.clone(),
        qp0: false,
    };
    let cancel = Cancel::new();
    let mut seen: Vec<ExportProgress> = Vec::new();
    let report =
        export_range_with_progress(&mut engine, &doc.view(), &job, 0..FRAMES, &cancel, |p| {
            seen.push(p);
        })
        .unwrap();

    assert_eq!(report.frames_written, FRAMES);
    assert_eq!(
        seen.len() as i64,
        FRAMES,
        "コールバックの呼び出し回数がフレーム数と食い違う"
    );

    let mut last_done = 0i64;
    for (i, p) in seen.iter().enumerate() {
        assert_eq!(p.frames_total, FRAMES, "frame {i}: frames_total が動いている");
        assert_eq!(
            p.frames_done,
            last_done + 1,
            "frame {i}: frames_done が単調増加(+1刻み)でない"
        );
        last_done = p.frames_done;
    }
    assert_eq!(last_done, FRAMES, "最後の進捗が総数と一致しない");

    // 現物とも一致する(報告 = 現物 の精神を進捗にも適用)。
    let info = probe(&out).expect("書き出した物を probe できない");
    assert_eq!(info.nb_frames, Some(report.frames_written));
}

/// 空範囲は `on_progress` を一度も呼ばない(既存の「0フレーム書いた成功」契約と揃える)。
#[test]
fn empty_range_calls_progress_zero_times() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-progress-empty");
    let out = dir.join("empty.mp4");
    let doc = moving_document();
    let mut engine = Engine::new().unwrap();

    let job = ExportJob {
        out_path: out.clone(),
        qp0: false,
    };
    let cancel = Cancel::new();
    let mut calls = 0u32;
    let report =
        export_range_with_progress(&mut engine, &doc.view(), &job, 12..12, &cancel, |_| {
            calls += 1;
        })
        .unwrap();

    assert_eq!(report.frames_written, 0);
    assert_eq!(calls, 0, "空範囲なのにコールバックが呼ばれた");
}

/// **途中キャンセルで進捗が止まる** — 中断チェックと進捗報告は同じループの
/// 同じ点(1フレーム境界)で見ているので、進捗の最後の値が「どこで止まったか」
/// をそのまま表す。ここでは N フレーム目の直後に cancel() を叩き、以降の
/// 呼び出しが来ないこと・成果物が残らないことを確かめる。
#[test]
fn cancel_mid_export_stops_progress_from_advancing_further() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-progress-cancel");
    let out = dir.join("cancelled.mp4");
    let doc = moving_document();
    let mut engine = Engine::new().unwrap();

    let job = ExportJob {
        out_path: out.clone(),
        qp0: false,
    };
    let cancel = Cancel::new();
    const STOP_AFTER: i64 = 5;
    let mut seen: Vec<ExportProgress> = Vec::new();
    let result =
        export_range_with_progress(&mut engine, &doc.view(), &job, 0..FRAMES, &cancel, |p| {
            seen.push(p);
            if p.frames_done == STOP_AFTER {
                cancel.cancel();
            }
        });

    assert!(matches!(result, Err(ExportError::Cancelled)));
    assert_eq!(
        seen.len() as i64,
        STOP_AFTER,
        "cancel() 後にもコールバックが呼ばれた(中断がフレーム境界で止まっていない)"
    );
    assert_eq!(seen.last().unwrap().frames_done, STOP_AFTER);
    assert!(
        !out.exists(),
        "中断したのに壊れた成果物が残っている: {}",
        out.display()
    );
}

/// **既存 golden 非退行**: `export_range_with_cancel`(コールバックなし版)は
/// `export_range_with_progress` へ委譲しただけで、書き出し結果自体は今までどおり。
#[test]
fn export_range_with_cancel_still_works_after_delegating_to_progress() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-progress-no-regression");
    let out = dir.join("no-regression.mp4");
    let doc = moving_document();
    let mut engine = Engine::new().unwrap();

    let job = ExportJob {
        out_path: out.clone(),
        qp0: false,
    };
    let cancel = Cancel::new();
    let report = export_range_with_cancel(&mut engine, &doc.view(), &job, 0..FRAMES, &cancel)
        .unwrap();

    assert_eq!(report.frames_written, FRAMES);
    let info = probe(&out).expect("書き出した物を probe できない");
    assert_eq!(info.nb_frames, Some(report.frames_written));
}

/// `export_with_progress`(全範囲の進捗版)も comp の全区間で回る。
#[test]
fn export_with_progress_covers_the_whole_composition() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-progress-whole");
    let out = dir.join("whole.mp4");
    let doc = moving_document();
    let mut engine = Engine::new().unwrap();

    let job = ExportJob {
        out_path: out.clone(),
        qp0: false,
    };
    let cancel = Cancel::new();
    let mut calls = 0i64;
    let report = export_with_progress(&mut engine, &doc.view(), &job, &cancel, |_| {
        calls += 1;
    })
    .unwrap();

    assert_eq!(report.frames_written, FRAMES);
    assert_eq!(calls, FRAMES);
}
