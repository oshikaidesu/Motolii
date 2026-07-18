//! M1 出口デモ(ヒーロー)のE2Eゴールデン。
//!
//! 「実写(生成)背景 + Bezierイージングで右へ流れる四角シェイプ」を
//! Document→レンダグラフ(D3)経由でmp4化し、**出力mp4をデコードして中身を検証**する。

use std::collections::BTreeMap;
use std::path::Path;

use motolii_cli::export_document;
use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime, TimeMap};
use motolii_doc::{
    Asset, AssetId, Clip, ClipSource, Composition, DocKeyframe, DocKeyframeTrack, DocParam,
    DocValue, Document, ItemEnvelope, KeyframeId, ProjectSession, ResourceLimits, SaveOptions,
    Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_eval::Interp;
use motolii_media::{probe, read_frame_at, Encoder};
use motolii_nodes::ViewportTransform;
use motolii_testkit::cpu_reference::yuv_to_rgba_reference;
use motolii_testkit::{ffmpeg_or_skip, gpu_or_skip, tmp_dir};

const W: u32 = 64;
const H: u32 = 48;
const FPS: Fps = match Fps::try_new(12, 1) {
    Ok(fps) => fps,
    Err(_) => panic!("invalid const fps"),
};
const N_FRAMES: usize = 13;
const BG_GRAY: u8 = 120;

fn eased_center_track() -> DocKeyframeTrack {
    let mut track = DocKeyframeTrack::new();
    track.insert(DocKeyframe {
        id: KeyframeId::from_raw(0),
        t: RationalTime::ZERO,
        value: DocValue::Vec2([-0.3, 0.0]),
        interp: Interp::Bezier {
            x1: 0.42,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        },
    });
    track.insert(DocKeyframe {
        id: KeyframeId::from_raw(1),
        t: RationalTime::try_new(1, 1).unwrap(),
        value: DocValue::Vec2([0.3, 0.0]),
        interp: Interp::Linear,
    });
    track
}

fn build_exit_demo_document(input_name: &str) -> Document {
    let mut doc = Document::new_current();
    // KeyframeId 0,1 を使うためカウンタを先に進める(A8)。
    let _ = doc.next_stable_id.allocate().unwrap();
    let _ = doc.next_stable_id.allocate().unwrap();
    doc.composition = Composition::try_new(
        W as i64,
        H as i64,
        RationalTime::try_new(N_FRAMES as i64, 12).unwrap(),
        FPS,
    )
    .unwrap();

    let asset_id = AssetId::from_raw(0);
    doc.assets
        .insert(Asset {
            id: asset_id,
            name: "bg".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:exit-demo".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: Some(input_name.into()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();

    let bg_layer = doc.layers.allocate("bg").unwrap();
    let overlay_layer = doc.layers.allocate("overlay").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let clip_duration = RationalTime::try_new(N_FRAMES as i64, 12).unwrap();

    let bg_clip = Clip {
        envelope: ItemEnvelope::new(bg_layer),
        start: RationalTime::ZERO,
        duration: clip_duration,
        time_map: TimeMap::identity(),
        source: ClipSource::asset_video_only(asset_id),
    };

    let center = DocParam::Keyframes(eased_center_track());
    let overlay_clip = Clip {
        envelope: ItemEnvelope::new(overlay_layer),
        start: RationalTime::ZERO,
        duration: clip_duration,
        time_map: TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: RECT_LAYER_SOURCE.into(),
            effect_version: 1,
            params: BTreeMap::from([
                ("center".into(), center),
                ("size".into(), DocParam::const_vec2([0.3, 0.4])),
                ("color".into(), DocParam::const_color([1.0, 0.15, 0.1, 1.0])),
            ]),
            extra: Default::default(),
        },
    };

    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(bg_clip), TrackItem::Clip(overlay_clip)],
    });
    doc
}

fn make_bg_video(path: &Path) {
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut enc = Encoder::open(path, &desc, FPS, true).unwrap();
    for _ in 0..N_FRAMES {
        let mut data = vec![0u8; desc.data_size()];
        for px in data.chunks_exact_mut(4) {
            px.copy_from_slice(&[BG_GRAY, BG_GRAY, BG_GRAY, 255]);
        }
        enc.write_frame(&data).unwrap();
    }
    enc.finish().unwrap();
}

fn pixel(rgba: &[u8], w: u32, x: i64, y: i64) -> [u8; 4] {
    let xi = x.clamp(0, w as i64 - 1) as usize;
    let yi = y.clamp(0, H as i64 - 1) as usize;
    let i = (yi * w as usize + xi) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
}

#[test]
fn exit_demo_video_bg_plus_eased_rect_matches_golden() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };

    let dir = tmp_dir("exit-demo");
    let input = dir.join("input.mp4");
    let output = dir.join("exit-demo.mp4");
    let document_path = dir.join("document.json");

    make_bg_video(&input);

    let doc = build_exit_demo_document("input.mp4");
    doc.validate().unwrap();
    {
        let mut session =
            ProjectSession::acquire(&document_path, &ResourceLimits::production()).unwrap();
        session
            .save_document(&doc, &SaveOptions::default())
            .unwrap();
    }

    let report = export_document(&gpu, &document_path, &output, Some(N_FRAMES), true).unwrap();
    assert_eq!(report.frames_written, N_FRAMES);
    assert_eq!((report.desc.width, report.desc.height), (W, H));

    // 検証用に同じキーフレーム軌道を再構築(IDは評価に影響しない)。
    let mut center_track = DocKeyframeTrack::new();
    center_track.insert(DocKeyframe {
        id: KeyframeId::from_raw(0),
        t: RationalTime::ZERO,
        value: DocValue::Vec2([-0.3, 0.0]),
        interp: Interp::Bezier {
            x1: 0.42,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        },
    });
    center_track.insert(DocKeyframe {
        id: KeyframeId::from_raw(1),
        t: RationalTime::try_new(1, 1).unwrap(),
        value: DocValue::Vec2([0.3, 0.0]),
        interp: Interp::Linear,
    });
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let tx = ViewportTransform::from_desc(&desc).expect("non-zero FrameDesc");

    let info = probe(&output).unwrap();
    assert_eq!((info.width, info.height), (W, H));

    let samples = [(0i64, "start"), (6, "mid"), (12, "end")];
    let mut centers_x = Vec::new();

    for (idx, label) in samples {
        let t = RationalTime::try_from_frame(idx, FPS).unwrap();
        let center = match center_track.eval(t) {
            motolii_eval::Value::Vec2(v) => v,
            other => panic!("center keyframes must be Vec2, got {other:?}"),
        };
        let c = tx.point_to_px(motolii_nodes::CanonicalPoint {
            x: center[0],
            y: center[1],
        });
        centers_x.push(c.x);

        let frame = read_frame_at(&output, &info, idx).unwrap();
        assert_eq!(frame.desc.format, PixelFormat::Yuv420p);
        let rgba = yuv_to_rgba_reference(&frame);

        let center_px = pixel(&rgba, W, c.x.round() as i64, c.y.round() as i64);
        assert!(
            center_px[0] > 150 && center_px[1] < 100 && center_px[2] < 100,
            "{label}: rect center expected red-ish, got {center_px:?} at ({:.1},{:.1})",
            c.x,
            c.y
        );

        let corner = pixel(&rgba, W, 2, 2);
        let gray_ok = corner[..3]
            .iter()
            .all(|&v| (v as i32 - BG_GRAY as i32).abs() < 45);
        assert!(
            gray_ok,
            "{label}: corner should stay near bg gray {BG_GRAY}, got {corner:?}"
        );
    }

    assert!(
        centers_x[0] < centers_x[1] && centers_x[1] < centers_x[2],
        "eased center.x should advance left→right across samples: {centers_x:?}"
    );
}
