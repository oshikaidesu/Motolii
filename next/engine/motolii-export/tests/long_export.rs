//! 長い書き出しが**メモリで死なない**こと。
//!
//! MV は3〜5分 = 1080p30 で 5,400〜9,000フレーム(製品命題)。
//! 途中でフレームを溜め込む設計だと、ここで初めて露見する。

use motolii_compositor::CompSpec;
use motolii_core::Fps;
use motolii_engine::Engine;
use motolii_export::{export, ExportJob};
use motolii_store::{Composition, Document, Intent, LayerId, LayerMeta, LayerSource};
use motolii_store::LayerTiming;
use motolii_testkit::{ffmpeg_or_skip, tmp_dir};
use std::process::Command;

const W: u32 = 320;
const H: u32 = 240;
const FRAMES: i64 = 300;

/// 実素材を持つ Document。**素材があると engine はフレームを持つ**ので、
/// 単色だけで測ると溜め込みを見逃す。
fn document_with_media(path: String) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: FRAMES,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Media {
                path,
                fingerprint: None,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
    doc
}

fn make_video(dir: &std::path::Path, seconds: u32) -> String {
    let path = dir.join("long.mp4");
    let out = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!("testsrc=size=320x240:duration={seconds}:rate=30"),
            "-pix_fmt",
            "yuv420p",
            "-c:v",
            "libx264",
            "-crf",
            "23",
        ])
        .arg(&path)
        .output()
        .expect("ffmpeg");
    assert!(out.status.success());
    path.to_string_lossy().into_owned()
}

/// **落ちる試験**: 素材つきで長く書き出すと、engine が全フレームの texture を
/// 抱えたままになる。300フレーム(10秒)でも GPU メモリを 300枚ぶん持つ。
///
/// 判定は「持っているフレーム数が窓の大きさを超えないこと」。
#[test]
fn long_export_does_not_accumulate_frames() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("export-long");
    let src = make_video(&dir, 10);
    let out = dir.join("long-out.mp4");
    let doc = document_with_media(src);

    let mut engine = Engine::new().unwrap();
    let job = ExportJob {
        out_path: out.clone(),
        qp0: false,
    };
    let report = export(&mut engine, &doc.view(), &job).unwrap();
    assert_eq!(report.frames_written, 300);

    // 判定は「**フレーム数に比例して伸びない**」こと。マジックナンバーではなく
    // 性質を見る — 上限の値そのものは「同時に描ける media layer の枚数」で決まり、
    // 変わりうるから。
    assert!(
        engine.cached_frame_count() <= Engine::FRAME_CACHE_LIMIT,
        "300フレーム書き出し後に {} フレームぶんの texture を抱えている(上限 {})",
        engine.cached_frame_count(),
        Engine::FRAME_CACHE_LIMIT
    );
    let after_300 = engine.cached_frame_count();

    // 倍のフレーム数を回しても抱える量が増えないこと。
    // (増えるなら上限が効いておらず、3〜5分の MV でメモリが死ぬ)
    let long_job = ExportJob {
        out_path: dir.join("long-out-2.mp4"),
        qp0: false,
    };
    let src2 = make_video(&dir, 10);
    let doc2 = document_with_media(src2);
    export(&mut engine, &doc2.view(), &long_job).unwrap();

    assert!(
        engine.cached_frame_count() <= Engine::FRAME_CACHE_LIMIT,
        "600フレームぶん回した後に {} 枚(300フレーム時点は {after_300} 枚)。\
         フレーム数に比例して伸びている",
        engine.cached_frame_count()
    );
}
