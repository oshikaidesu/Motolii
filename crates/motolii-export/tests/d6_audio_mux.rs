#![allow(deprecated)]

//! D6: Document書き出しの楽曲mux + D1f書き出し厳格化。

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime, TimeMap};
use motolii_doc::{
    Asset, AssetId, Clip, ClipSource, Composition, DocParam, Document, EffectDefinitionId,
    EffectId, EffectInstance, ItemEnvelope, PluginDiagnosticReason, Soundtrack, Track, TrackItem,
    RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_export::{export_document_video, ExportError, ExportJob};
use motolii_media::{probe, Encoder};
use motolii_plugin::reference::reference_catalog;
use motolii_plugin::{PluginRegistry, PluginRuntime};
use motolii_plugins_firstparty::{first_party_catalog, first_party_runtime};
use motolii_testkit::{ffmpeg_or_skip, gpu_or_skip, tmp_dir};

const W: u32 = 32;
const H: u32 = 24;
const FPS: Fps = match Fps::try_new(12, 1) {
    Ok(fps) => fps,
    Err(_) => panic!("invalid const fps"),
};
const N_FRAMES: usize = 12; // 1秒

fn reference_runtime() -> PluginRuntime {
    first_party_runtime().unwrap()
}

fn contract_only_runtime() -> PluginRuntime {
    PluginRuntime::try_new(
        std::sync::Arc::new(first_party_catalog().unwrap()),
        PluginRegistry::new(),
    )
    .unwrap()
}

fn make_bg_video(path: &Path) {
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut enc = Encoder::open(path, &desc, FPS, true).unwrap();
    for i in 0..N_FRAMES {
        let g = (i * 10) as u8;
        let mut data = vec![0u8; desc.data_size()];
        for px in data.chunks_exact_mut(4) {
            px.copy_from_slice(&[g, g, g, 255]);
        }
        enc.write_frame(&data).unwrap();
    }
    enc.finish().unwrap();
}

fn make_aac(path: &Path, seconds: f64) {
    let status = Command::new("ffmpeg")
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!("sine=frequency=880:sample_rate=48000:duration={seconds}"),
            "-c:a",
            "aac",
            "-b:a",
            "128k",
        ])
        .arg(path)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success(), "aac fixture failed");
}

fn extract_pcm(path: &Path, out: &Path, start: Option<&str>, duration: &str) {
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-v", "error", "-y"]);
    if let Some(ss) = start {
        cmd.args(["-ss", ss]);
    }
    cmd.arg("-i").arg(path);
    cmd.args([
        "-t", duration, "-vn", "-ac", "1", "-ar", "48000", "-f", "s16le",
    ])
    .arg(out);
    let status = cmd.status().expect("spawn ffmpeg extract");
    assert!(
        status.success(),
        "pcm extract failed for {}",
        path.display()
    );
}

fn build_doc(video_name: &str, audio_name: Option<(&str, RationalTime)>) -> Document {
    let mut doc = Document::new_v1();
    doc.version = 2;
    doc.min_reader_version = 2;
    doc.composition = Composition::try_new(
        W as i64,
        H as i64,
        RationalTime::try_new(N_FRAMES as i64, 12).unwrap(),
        FPS,
    )
    .unwrap();

    let video_id = AssetId::from_raw(0);
    doc.assets
        .insert(Asset {
            id: video_id,
            name: "bg".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:d6-bg".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: Some(video_name.into()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();

    if let Some((audio_name, offset)) = audio_name {
        let audio_id = AssetId::from_raw(1);
        doc.assets
            .insert(Asset {
                id: audio_id,
                name: "song".into(),
                asset_type: "audio/mp4".into(),
                content_hash: "sha256:d6-audio".into(),
                path_absolute: None,
                path_project_relative: None,
                file_name: Some(audio_name.into()),
                size_bytes: None,
                head_hash: None,
                tail_hash: None,
            })
            .unwrap();
        doc.soundtrack = Some(Soundtrack::try_new(audio_id, offset, 1.0).unwrap());
    }

    let layer = doc.layers.allocate("bg").unwrap();
    let overlay_layer = doc.layers.allocate("overlay").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let clip_duration = RationalTime::try_new(N_FRAMES as i64, 12).unwrap();
    // 単一VideoSourceだけだと graph が transparent を未使用書き込みのまま残すため、
    // exit_demo と同型の rect overlay を載せて Composite 経路にする。
    doc.tracks.push(Track {
        id: track_id,
        items: vec![
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(layer),
                start: RationalTime::ZERO,
                duration: clip_duration,
                time_map: TimeMap::identity(),
                source: ClipSource::asset_video_only(video_id),
            }),
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(overlay_layer),
                start: RationalTime::ZERO,
                duration: clip_duration,
                time_map: TimeMap::identity(),
                source: ClipSource::Plugin {
                    plugin_id: RECT_LAYER_SOURCE.into(),
                    effect_version: 1,
                    params: BTreeMap::from([
                        ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                        ("size".into(), DocParam::const_vec2([0.2, 0.2])),
                        ("color".into(), DocParam::const_color([1.0, 0.0, 0.0, 0.0])),
                    ]),
                    extra: Default::default(),
                },
            }),
        ],
    });
    doc
}

#[test]
fn export_muxes_soundtrack_sample_exact_stream_copy() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };

    let dir = tmp_dir("export-d6-mux");
    let video = dir.join("bg.mp4");
    let audio = dir.join("song.m4a");
    let output = dir.join("out.mp4");
    make_bg_video(&video);
    make_aac(&audio, 2.0);

    let doc = build_doc("bg.mp4", Some(("song.m4a", RationalTime::ZERO)));
    doc.validate().unwrap();

    let report = export_document_video(
        &gpu,
        &ExportJob {
            doc: &doc,
            runtime: &reference_runtime(),
            output_path: &output,
            project_root: Some(&dir),
            frame_count: Some(N_FRAMES),
            qp0: true,
            data_tracks: DataTracks::new(),
        },
    )
    .unwrap();
    assert_eq!(report.frames_written, N_FRAMES);

    // 映像が残っていること
    let info = probe(&output).unwrap();
    assert_eq!((info.width, info.height), (W, H));
    assert_eq!(info.fps, FPS);

    // 音声が元素材とサンプル一致(ストリームコピー相当)。
    // -shortest のAACフレーム境界差は許容し、重なり先頭の一致を審判にする。
    let got = dir.join("got.pcm");
    let want = dir.join("want.pcm");
    extract_pcm(&output, &got, None, "1");
    extract_pcm(&audio, &want, None, "1");
    let got_bytes = std::fs::read(&got).unwrap();
    let want_bytes = std::fs::read(&want).unwrap();
    let n = got_bytes.len().min(want_bytes.len());
    assert!(n > 48_000, "expected ~1s of pcm, got {n}");
    assert_eq!(&got_bytes[..n], &want_bytes[..n]);
    assert!((got_bytes.len() as i64 - want_bytes.len() as i64).abs() < 8_192);

    // 一時映像ファイルが残っていないこと
    assert!(!dir.join("out.video-only.tmp.mp4").exists());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn export_mux_respects_start_offset_and_stays_in_sync() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };

    let dir = tmp_dir("export-d6-offset");
    let video = dir.join("bg.mp4");
    let audio = dir.join("song.m4a");
    let output = dir.join("out.mp4");
    make_bg_video(&video);
    make_aac(&audio, 3.0);
    let offset = RationalTime::try_new(1, 2).unwrap(); // 0.5s

    let doc = build_doc("bg.mp4", Some(("song.m4a", offset)));
    doc.validate().unwrap();

    export_document_video(
        &gpu,
        &ExportJob {
            doc: &doc,
            runtime: &reference_runtime(),
            output_path: &output,
            project_root: Some(&dir),
            frame_count: Some(N_FRAMES),
            qp0: true,
            data_tracks: DataTracks::new(),
        },
    )
    .unwrap();

    let got = dir.join("got.pcm");
    let want = dir.join("want.pcm");
    extract_pcm(&output, &got, None, "1");
    extract_pcm(&audio, &want, Some("0.5"), "1");
    let got_bytes = std::fs::read(&got).unwrap();
    let want_bytes = std::fs::read(&want).unwrap();
    let n = got_bytes.len().min(want_bytes.len());
    assert!(n > 48_000, "expected ~1s of pcm, got {n}");
    assert_eq!(&got_bytes[..n], &want_bytes[..n]);
    assert!((got_bytes.len() as i64 - want_bytes.len() as i64).abs() < 8_192);

    // 映像尺≈音声尺(shortest): ずれて無音パディングが混ざっていない
    let probe = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=start_time,duration",
            "-of",
            "csv=p=0",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(probe.status.success());
    let text = String::from_utf8_lossy(&probe.stdout);
    let parts: Vec<&str> = text.trim().split(',').collect();
    assert!(parts.len() >= 2, "ffprobe audio fields: {text}");
    let start: f64 = parts[0].parse().unwrap_or(0.0);
    let dur: f64 = parts[1].parse().unwrap();
    assert!(start.abs() < 0.05, "audio start_time drifted: {start}");
    assert!(
        (dur - 1.0).abs() < 0.15,
        "audio duration not matched to video: {dur}"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn export_refuses_degraded_plugins() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };

    let dir = tmp_dir("export-d6-degraded");
    let video = dir.join("bg.mp4");
    let output = dir.join("out.mp4");
    make_bg_video(&video);

    let mut doc = build_doc("bg.mp4", None);
    let eid = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let did = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    let (use_, def) = EffectInstance {
        id: eid,
        definition_id: did,
        plugin_id: "vendor.filter.unknown_for_export".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::from([("amount".into(), DocParam::const_f64(0.25))]),
        extra: Default::default(),
    }
    .into_use_and_definition();
    doc.effect_definitions.push(def);
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        clip.envelope.effects.push(use_);
    }
    doc.version = doc
        .version
        .max(motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS);
    doc.min_reader_version = doc
        .min_reader_version
        .max(motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS);
    doc.validate()
        .expect("unknown plugin must still validate (open side)");
    assert!(!doc
        .prepare_plugins(&reference_catalog().unwrap())
        .unwrap()
        .diagnostics()
        .is_empty());

    let err = export_document_video(
        &gpu,
        &ExportJob {
            doc: &doc,
            runtime: &reference_runtime(),
            output_path: &output,
            project_root: Some(&dir),
            frame_count: Some(1),
            qp0: true,
            data_tracks: DataTracks::new(),
        },
    )
    .unwrap_err();
    assert!(
        matches!(err, ExportError::DegradedPlugins(_)),
        "expected DegradedPlugins, got {err:?}"
    );
    assert!(!output.exists(), "refused export must not leave output");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn export_refuses_contract_without_executor() {
    let Some(gpu) = gpu_or_skip() else { return };
    let dir = tmp_dir("export-a0i3-executor-missing");
    let output = dir.join("out.mp4");
    let mut doc = build_doc("missing.mp4", None);
    let eid = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let did = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    let (use_, def) = EffectInstance {
        id: eid,
        definition_id: did,
        plugin_id: "core.filter.opacity".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
        extra: Default::default(),
    }
    .into_use_and_definition();
    doc.effect_definitions.push(def);
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        clip.envelope.effects.push(use_);
    }
    doc.version = doc
        .version
        .max(motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS);
    doc.min_reader_version = doc
        .min_reader_version
        .max(motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS);
    let runtime = contract_only_runtime();

    let error = export_document_video(
        &gpu,
        &ExportJob {
            doc: &doc,
            runtime: &runtime,
            output_path: &output,
            project_root: Some(&dir),
            frame_count: Some(1),
            qp0: true,
            data_tracks: DataTracks::new(),
        },
    )
    .unwrap_err();

    let ExportError::DegradedPlugins(diagnostics) = error else {
        panic!("expected degraded export, got {error:?}");
    };
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.plugin_id == "core.filter.opacity"
            && diagnostic.reason == PluginDiagnosticReason::ExecutorMissing
    }));
    assert!(!output.exists());
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn export_refuses_future_version_rect_layer_source() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };

    let dir = tmp_dir("export-d6-future-rect");
    let video = dir.join("bg.mp4");
    let output = dir.join("out.mp4");
    make_bg_video(&video);

    let mut doc = build_doc("bg.mp4", None);
    // build_doc は現行v1の rect overlay を含む。未来版へ書き換え、D1f degraded にする。
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[1] {
        if let ClipSource::Plugin {
            plugin_id,
            effect_version,
            ..
        } = &mut clip.source
        {
            assert_eq!(plugin_id, RECT_LAYER_SOURCE);
            *effect_version = 2;
        }
    }
    doc.validate()
        .expect("future rect version must still open (D1f)");
    let prepared = doc.prepare_plugins(&reference_catalog().unwrap()).unwrap();
    let warnings = prepared.diagnostics();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(warnings[0].plugin_id, RECT_LAYER_SOURCE);
    assert_eq!(
        warnings[0].reason,
        PluginDiagnosticReason::FutureVersion {
            current_version: 1,
            saved_version: 2,
        }
    );

    let err = export_document_video(
        &gpu,
        &ExportJob {
            doc: &doc,
            runtime: &reference_runtime(),
            output_path: &output,
            project_root: Some(&dir),
            frame_count: Some(1),
            qp0: true,
            data_tracks: DataTracks::new(),
        },
    )
    .unwrap_err();
    assert!(
        matches!(err, ExportError::DegradedPlugins(_)),
        "future rect must not bypass DegradedPlugins, got {err:?}"
    );
    assert!(!output.exists());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn current_version_rect_alone_is_not_degraded() {
    let doc = build_doc("bg.mp4", None);
    assert!(
        doc.prepare_plugins(&reference_catalog().unwrap())
            .unwrap()
            .diagnostics()
            .is_empty(),
        "v1 rect is a known built-in contract: {:?}",
        doc.prepare_plugins(&reference_catalog().unwrap())
            .unwrap()
            .diagnostics()
    );
}
