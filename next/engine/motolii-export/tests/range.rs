//! 範囲付き書き出しの契約 — フレーム数一致・境界(最短1フレーム)・全範囲との整合。

use motolii_core::Fps;
use motolii_engine::Engine;
use motolii_export::{export, export_range, export_range_with_cancel, Cancel, ExportError, ExportJob};
use motolii_media::probe;
use motolii_store::{Composition, Document, Intent, LayerId, LayerMeta, LayerSource};
use motolii_store::LayerTiming;
use motolii_testkit::{ffmpeg_or_skip, tmp_dir};

const W: u32 = 64;
const H: u32 = 64;
const FRAMES: i64 = 30;

fn fps() -> Fps {
    Fps::try_new(30, 1).unwrap()
}

/// 動く白い板。時間で絵が変わるので、範囲の途中から書き出しても
/// 正しい区間の絵になっているか(将来の目視/golden 追加時に)確かめられる。
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

#[test]
fn range_frame_count_matches_the_artifact() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-range-count");
    let out = dir.join("range.mp4");
    let doc = moving_document();
    let mut engine = Engine::new().unwrap();

    let job = ExportJob {
        out_path: out.clone(),
        qp0: false,
    };
    // 半開 [10, 20) — 10フレームぶんだけ。
    let report = export_range(&mut engine, &doc.view(), &job, 10..20).unwrap();

    assert_eq!(report.frames_written, 10);
    assert!(out.exists(), "成果物が無い");

    let info = probe(&out).expect("書き出した物を probe できない");
    assert_eq!(
        info.nb_frames,
        Some(report.frames_written),
        "報告 {} と現物 {:?} が食い違う",
        report.frames_written,
        info.nb_frames
    );
}

#[test]
fn range_of_a_single_frame_writes_exactly_one_frame() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-range-min");
    let out = dir.join("one.mp4");
    let doc = moving_document();
    let mut engine = Engine::new().unwrap();

    let job = ExportJob {
        out_path: out.clone(),
        qp0: false,
    };
    // 最短の有効範囲: start..start+1。
    let report = export_range(&mut engine, &doc.view(), &job, 5..6).unwrap();

    assert_eq!(report.frames_written, 1);
    let info = probe(&out).expect("書き出した物を probe できない");
    assert_eq!(info.nb_frames, Some(1));
}

#[test]
fn empty_range_writes_zero_frames_not_an_error() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-range-empty");
    let out = dir.join("empty.mp4");
    let doc = moving_document();
    let mut engine = Engine::new().unwrap();

    let job = ExportJob {
        out_path: out.clone(),
        qp0: false,
    };
    // start >= end は「0フレーム書いた」成功として報告する(妥当性検査ではない)。
    let report = export_range(&mut engine, &doc.view(), &job, 12..12).unwrap();
    assert_eq!(report.frames_written, 0);
}

/// [`export_with_cancel`] は `0..duration_frames` を範囲書き出しへ委譲した
/// だけであることを、範囲版を直接呼んだ結果と突き合わせて確かめる —
/// 「全範囲 = 範囲書き出しの特殊形」という実装の主張そのもの。
#[test]
fn full_range_agrees_with_export_with_cancel() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-range-full-agree");
    let doc = moving_document();

    let out_a = dir.join("via-export.mp4");
    let mut engine_a = Engine::new().unwrap();
    let job_a = ExportJob {
        out_path: out_a.clone(),
        qp0: false,
    };
    let report_a = export(&mut engine_a, &doc.view(), &job_a).unwrap();

    let out_b = dir.join("via-range.mp4");
    let mut engine_b = Engine::new().unwrap();
    let job_b = ExportJob {
        out_path: out_b.clone(),
        qp0: false,
    };
    let report_b = export_range(&mut engine_b, &doc.view(), &job_b, 0..FRAMES).unwrap();

    assert_eq!(report_a.frames_written, report_b.frames_written);
    assert_eq!(report_a.frames_written, FRAMES);
}

#[test]
fn cancel_leaves_no_artifact_for_a_range_too() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-range-cancel");
    let out = dir.join("cancelled-range.mp4");
    let doc = moving_document();
    let mut engine = Engine::new().unwrap();

    let cancel = Cancel::new();
    cancel.cancel();

    let job = ExportJob {
        out_path: out.clone(),
        qp0: false,
    };
    let result = export_range_with_cancel(&mut engine, &doc.view(), &job, 0..FRAMES, &cancel);

    assert!(matches!(result, Err(ExportError::Cancelled)));
    assert!(
        !out.exists(),
        "中断したのに壊れた成果物が残っている: {}",
        out.display()
    );
}
