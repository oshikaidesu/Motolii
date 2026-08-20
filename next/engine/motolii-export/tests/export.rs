//! 書き出しの契約 — **報告 = 現物**、中断で残骸なし、そして **preview = export**。

use motolii_compositor::CompSpec;
use motolii_core::Fps;
use motolii_engine::Engine;
use motolii_export::{export, export_with_cancel, Cancel, ExportError, ExportJob};
use motolii_media::probe;
use motolii_store::{Composition, Document, Intent, LayerId, LayerMeta, LayerSource, RationalTime};
use motolii_store::LayerTiming;
use motolii_testkit::{ffmpeg_or_skip, tmp_dir};

const W: u32 = 64;
const FRAMES: i64 = 30;
const H: u32 = 64;

fn comp() -> CompSpec {
    CompSpec {
        width: W,
        height: H,
    }
}

fn fps() -> Fps {
    Fps::try_new(30, 1).unwrap()
}

/// 動く白い板。時間で絵が変わるので、書き出しが本当に各フレームを回したか分かる。
fn moving_document() -> Document {
    use motolii_store::{Composition, property, Interp, Keyframe, KeyframeTrack, PropertyId, Value};

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: FRAMES,
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
fn report_matches_the_artifact() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-report");
    let out = dir.join("report.mp4");
    let doc = moving_document();
    let mut engine = Engine::new().unwrap();

    let job = ExportJob {
        out_path: out.clone(),
        qp0: false,
    };
    let report = export(&mut engine, &doc.view(), &job).unwrap();

    assert_eq!(report.frames_written, 30);
    assert!(out.exists(), "成果物が無い");

    let info = probe(&out).expect("書き出した物を probe できない");
    assert_eq!(info.width, W);
    assert_eq!(info.height, H);
    assert_eq!(
        info.nb_frames,
        Some(report.frames_written),
        "報告 {} と現物 {:?} が食い違う",
        report.frames_written,
        info.nb_frames
    );
}

#[test]
fn cancel_leaves_no_artifact() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-cancel");
    let out = dir.join("cancelled.mp4");
    let doc = moving_document();
    let mut engine = Engine::new().unwrap();

    let cancel = Cancel::new();
    cancel.cancel();

    let job = ExportJob {
        out_path: out.clone(),
        qp0: false,
    };
    let result = export_with_cancel(&mut engine, &doc.view(), &job, &cancel);

    assert!(matches!(result, Err(ExportError::Cancelled)));
    assert!(
        !out.exists(),
        "中断したのに壊れた成果物が残っている: {}",
        out.display()
    );
}

/// **M15 / D1 — preview = export**。
///
/// 書き出したフレームは `Engine::render_frame` の出力そのものである(構造的にそう
/// 作ってある)。ここではそれを**現物から**確かめる: 可逆(qp0)で書き出し、
/// 出来た file を decode し直して preview と突き合わせる。
///
/// h264 は YUV420 を通るので RGB とは完全一致しない(色差の間引きがある)。
/// よって「同一」ではなく「**間引きで説明できる範囲**」で判定する。
#[test]
fn exported_frames_are_the_preview_frames() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-same");
    let out = dir.join("same.mp4");
    let doc = moving_document();
    let mut engine = Engine::new().unwrap();

    let job = ExportJob {
        out_path: out.clone(),
        // 可逆。codec の量子化を判定から外す。
        qp0: true,
    };
    export(&mut engine, &doc.view(), &job).unwrap();

    let info = probe(&out).unwrap();
    for frame in [0i64, 2, 4] {
        let preview = engine
            .render_frame(&doc.view(), RationalTime::try_new(frame, 30).unwrap())
            .unwrap();

        let decoded = motolii_media::read_frame_at(&out, &info, frame).unwrap();
        // decode は YUV420p。Y 面だけで比べる(色差は間引かれている)。
        let luma = &decoded.data[..(W * H) as usize];

        let mut worst = 0i32;
        for y in 0..H {
            for x in 0..W {
                let i = ((y * W + x) * 4) as usize;
                // preview は premultiplied sRGB RGBA。limited range の Y へ写す。
                let r = preview[i] as f64;
                let g = preview[i + 1] as f64;
                let b = preview[i + 2] as f64;
                let expected =
                    16.0 + (0.2126 * r + 0.7152 * g + 0.0722 * b) * 219.0 / 255.0;
                let got = luma[(y * W + x) as usize] as f64;
                worst = worst.max((expected - got).abs().round() as i32);
            }
        }

        assert!(
            worst <= 8,
            "frame {frame}: preview と書き出しの Y が最大 {worst} ずれた。\
             可逆書き出しなので、これは評価経路が2本ある兆候"
        );
    }
}
