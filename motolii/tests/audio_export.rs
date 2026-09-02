mod testkit;

use std::process::Command;

use motolii::doc::export::{export, export_with_cancel, Cancel, ExportError, ExportJob};
use motolii::doc::store::{
    Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource,
    LayerTiming,
};
use motolii::render::engine::Engine;

#[test]
fn export_muxes_the_document_audio_program_into_the_mp4() {
    if !testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = testkit::tmp_dir("audio-export");
    let tone = dir.join("tone.wav");
    let movie = dir.join("movie.mp4");
    let made = Command::new("ffmpeg")
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=1",
            "-ac",
            "2",
        ])
        .arg(&tone)
        .status()
        .unwrap();
    assert!(
        made.success(),
        "FFmpeg could not generate the official tone fixture"
    );

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 16,
        height: 16,
        fps: Fps::try_new(10, 1).unwrap(),
        duration_frames: 10,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let audio = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(audio),
        Intent::SetMeta {
            layer: audio,
            meta: LayerMeta {
                source: LayerSource::File {
                    path: tone.to_string_lossy().into_owned(),
                    fingerprint: None,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 10),
            },
        },
        Intent::SetAttrs {
            layer: audio,
            patch: LayerAttrsPatch {
                name: Some("tone".to_owned()),
                ..Default::default()
            },
        },
    ])
    .unwrap();

    let project = dir.join("project.rrd");
    doc.save(&project).unwrap();
    let doc = Document::load(&project).expect("saved audio project reopens");

    let report = export(
        &mut Engine::new().unwrap(),
        &doc.view(),
        &ExportJob {
            out_path: movie.clone(),
            qp0: false,
        },
    )
    .unwrap();
    assert_eq!(report.frames_written, 10);

    let probed = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_type,codec_name,sample_rate,channels",
            "-of",
            "json",
        ])
        .arg(&movie)
        .output()
        .unwrap();
    assert!(probed.status.success());
    let json: serde_json::Value = serde_json::from_slice(&probed.stdout).unwrap();
    let streams = json["streams"].as_array().expect("ffprobe streams");
    assert!(streams.iter().any(|stream| stream["codec_type"] == "video"));
    let audio_stream = streams
        .iter()
        .find(|stream| stream["codec_type"] == "audio")
        .expect("exported MP4 has no audio stream");
    assert_eq!(audio_stream["codec_name"], "aac");
    assert_eq!(audio_stream["sample_rate"], "48000");
    assert_eq!(audio_stream["channels"], 2);

    let decoded = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(&movie)
        .args([
            "-map", "0:a:0", "-f", "f32le", "-ac", "2", "-ar", "48000", "-",
        ])
        .output()
        .unwrap();
    assert!(decoded.status.success());
    let non_silent = decoded
        .stdout
        .chunks_exact(4)
        .any(|bytes| f32::from_le_bytes(bytes.try_into().expect("four-byte f32")).abs() > 0.001);
    assert!(non_silent, "the muxed audio stream decodes to silence");
}

#[test]
fn cancelling_export_keeps_an_existing_destination() {
    if !testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = testkit::tmp_dir("cancelled-export");
    let movie = dir.join("movie.mp4");
    let sentinel = b"previous completed movie";
    std::fs::write(&movie, sentinel).unwrap();

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 16,
        height: 16,
        fps: Fps::try_new(10, 1).unwrap(),
        duration_frames: 1,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let cancel = Cancel::new();
    cancel.cancel();
    let result = export_with_cancel(
        &mut Engine::new().unwrap(),
        &doc.view(),
        &ExportJob {
            out_path: movie.clone(),
            qp0: false,
        },
        &cancel,
    );
    assert!(matches!(result, Err(ExportError::Cancelled)));
    assert_eq!(std::fs::read(&movie).unwrap(), sentinel);
}
