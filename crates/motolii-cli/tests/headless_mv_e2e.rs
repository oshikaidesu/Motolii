//! 完成条件の鎖をheadlessで通すE2E: new-project → import(動画+音声) → place →
//! set-soundtrack → export → 出力に映像・音声両streamが実在することをffprobeで審判。
//!
//! CLIの意味は全てライブラリ側(`document_edit`)にあり、binはargvの写像だけ。
//! fixtureは既存慣行(admission_probe / ag4_audio_export)に合わせてffmpegで生成し、
//! 不在時のskipは`ffmpeg_or_skip` / `gpu_or_skip`を通す。

use std::path::Path;
use std::process::Command;

use motolii_cli::document_edit::{import_asset, new_project, place_clip, set_soundtrack};
use motolii_cli::export_document;
use motolii_gpu::GpuCtx;
use motolii_media::probe_audio;
use motolii_testkit::{ffmpeg_or_skip, gpu_or_skip, tmp_dir};

fn run_ffmpeg(args: &[&str]) {
    let status = Command::new("ffmpeg")
        .args(["-v", "error", "-y"])
        .args(args)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success(), "ffmpeg failed: {args:?}");
}

fn make_video(path: &Path) {
    run_ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "color=c=red:s=640x360:d=1.0:r=24",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        path.to_str().unwrap(),
    ]);
}

fn make_music(path: &Path) {
    run_ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:sample_rate=48000:duration=1.0",
        "-c:a",
        "aac",
        "-b:a",
        "128k",
        path.to_str().unwrap(),
    ]);
}

/// ffprobeで映像streamの存在を独立確認する(muxの自己申告に依らない)。
fn has_video_stream(path: &Path) -> bool {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_type",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .expect("spawn ffprobe");
    String::from_utf8_lossy(&output.stdout).trim() == "video"
}

#[test]
fn full_chain_import_place_soundtrack_export() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(_gpu_probe) = gpu_or_skip() else {
        return;
    };

    let dir = tmp_dir("headless_mv_e2e");
    let video = dir.join("clip.mp4");
    let music = dir.join("music.m4a");
    make_video(&video);
    make_music(&music);

    // 1. 新規プロジェクト(空のDocument + 既定Track 1本)。
    let project = dir.join("mv.json");
    new_project(&project).expect("new project");

    // 2. 素材2本をimport(probe→AssetDraft→既存admission経路)。
    let video_asset = import_asset(&project, &video).expect("import video");
    let music_asset = import_asset(&project, &music).expect("import music");
    assert_ne!(video_asset, music_asset);

    // 3. 動画を配置(t=0からcomposition endまで、CU-101の既定)。
    let layer = place_clip(&project, video_asset, 0.0).expect("place video");

    // 4. 曲を貼る(SetSoundtrack)。
    set_soundtrack(&project, music_asset, 0.0, 1.0).expect("set soundtrack");

    // 5. 再オープンして永続化を確認(catalogから再構成した状態で審判)。
    {
        let runtime = motolii_plugins_firstparty::first_party_runtime().expect("runtime");
        let opened = motolii_doc::open_project_resolved(
            &project,
            &motolii_doc::ResourceLimits::production(),
            runtime.catalog(),
        )
        .expect("reopen");
        let doc = &opened.recovered.document;
        assert_eq!(doc.assets.iter().count(), 2, "both assets persisted");
        assert!(
            doc.soundtrack.is_some(),
            "soundtrack persisted through save/reopen"
        );
        let placed = doc
            .tracks
            .iter()
            .flat_map(|track| track.items.iter())
            .filter_map(|item| match item {
                motolii_doc::TrackItem::Clip(clip) => Some(clip),
                _ => None,
            })
            .find(|clip| clip.envelope.layer_id == layer);
        assert!(placed.is_some(), "placed clip persisted with its LayerId");
    }

    // 6. 書き出し(既存のexport経路そのまま)。
    let gpu = GpuCtx::new_headless().expect("headless gpu");
    let out = dir.join("out.mp4");
    export_document(&gpu, &project, &out, None, false).expect("export");

    // 7. 出力の審判: 映像streamと音声streamが両方実在する。
    assert!(has_video_stream(&out), "exported file must carry video");
    let audio = probe_audio(&out).expect("exported file must carry audio");
    assert!(
        audio.sample_rate.unwrap_or(0) > 0,
        "audio stream must have a sample rate"
    );
}
