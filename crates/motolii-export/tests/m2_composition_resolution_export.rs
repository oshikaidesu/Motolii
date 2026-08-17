//! 2026-08-17決定「出力解像度はCompositionが所有し、素材はfitで受ける」のexport側審判:
//! `resolution: None` は旧挙動(最初のvideo sourceのnative解像度)を保ち、
//! `Some` は素材のnativeを無視してcomposition解像度で書き出し、aspect不一致の素材は
//! contain fit(中央配置・余白は合成背景)でletterbox/pillarboxされる。
//! fixture生成は既存慣行(`d3e_preview_export_same`)と同じ`motolii_media::Encoder`。

use std::path::Path;

use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime, TimeMap};
use motolii_doc::{
    Asset, AssetId, Clip, ClipSource, Composition, Document, ItemEnvelope, Track, TrackItem,
};
use motolii_eval::DataTracks;
use motolii_export::{export_document_video, ExportJob, EXPORT_DOWNLOAD_TIMEOUT};
use motolii_gpu::{GpuCtx, RgbaDownloader, YuvToRgba};
use motolii_media::{probe, Encoder, FrameReader};
use motolii_plugins_firstparty::first_party_runtime;
use motolii_testkit::{ffmpeg_or_skip, gpu_or_skip, tmp_dir};

const FPS: Fps = match Fps::try_new(12, 1) {
    Ok(fps) => fps,
    Err(_) => panic!("invalid const fps"),
};

/// 単色ソース動画をnative解像度WxHで作る。
fn make_solid_video(path: &Path, width: u32, height: u32, rgba: [u8; 4]) {
    let desc = FrameDesc::packed(width, height, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut enc = Encoder::open(path, &desc, FPS, true).unwrap();
    let mut frame = vec![0u8; desc.data_size()];
    for px in frame.chunks_exact_mut(4) {
        px.copy_from_slice(&rgba);
    }
    enc.write_frame(&frame).unwrap();
    enc.finish().unwrap();
}

/// 動画クリップ1本のDocument。composition aspectは引数、resolutionは呼び出し側が設定する。
fn video_document(video_name: &str, aspect_num: i64, aspect_den: i64) -> Document {
    let mut doc = Document::new_current();
    doc.composition = Composition::try_new(
        aspect_num,
        aspect_den,
        RationalTime::try_new(1, 1).unwrap(),
        FPS,
    )
    .unwrap();

    let video_id = AssetId::from_raw(0);
    doc.assets
        .insert(Asset {
            id: video_id,
            name: "src".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:m2-resolution-src".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: Some(video_name.into()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
            duration: None,
        })
        .unwrap();

    let layer = doc.layers.allocate("src").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::identity(),
            source: ClipSource::asset_video_only(video_id),
        })],
    });
    doc.validate().unwrap();
    doc
}

fn decode_exported_rgba(gpu: &GpuCtx, path: &Path) -> (u32, u32, Vec<u8>) {
    let info = probe(path).unwrap();
    let mut reader = FrameReader::open(path, &info, 0).unwrap();
    let frame = reader.next_frame().unwrap().expect("exported frame");
    let mut yuv = YuvToRgba::new(gpu);
    let texture = yuv.convert(gpu, &frame).unwrap();
    let mut downloader = RgbaDownloader::new();
    let rgba = downloader
        .download(gpu, &texture, EXPORT_DOWNLOAD_TIMEOUT)
        .unwrap();
    (info.width, info.height, rgba)
}

fn pixel(rgba: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let idx = ((y * width + x) * 4) as usize;
    rgba[idx..idx + 4].try_into().unwrap()
}

/// 互換ピン: `resolution: None` は現行導出(最初のvideo sourceのnative)を変えない。
#[test]
fn resolution_none_keeps_deriving_export_desc_from_first_video() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };
    let dir = tmp_dir("m2-resolution-none");
    let video = dir.join("src.mp4");
    make_solid_video(&video, 32, 24, [200, 40, 40, 255]);

    let doc = video_document("src.mp4", 32, 24);
    assert_eq!(doc.composition.resolution(), None);
    let runtime = first_party_runtime().unwrap();
    let output = dir.join("out.mp4");
    let report = export_document_video(
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
    .unwrap();
    assert_eq!((report.desc.width, report.desc.height), (32, 24));
}

/// `Some`: 出力はcomposition解像度。9:16素材は16:9出力へpillarbox
/// (中央列に映像、左右は合成背景=黒)される。
#[test]
fn resolution_some_overrides_native_and_pillarboxes_portrait_source() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };
    let dir = tmp_dir("m2-resolution-some");
    let video = dir.join("portrait.mp4");
    // 9:16縦動画(36x64)。
    make_solid_video(&video, 36, 64, [200, 40, 40, 255]);

    let mut doc = video_document("portrait.mp4", 16, 9);
    doc.composition.set_resolution(Some((64, 36))).unwrap();
    doc.validate().unwrap();
    let runtime = first_party_runtime().unwrap();
    let output = dir.join("out.mp4");
    let report = export_document_video(
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
    .unwrap();
    assert_eq!((report.desc.width, report.desc.height), (64, 36));

    let (width, height, rgba) = decode_exported_rgba(&gpu, &output);
    assert_eq!((width, height), (64, 36));

    // contain fit: 36x64 → 64x36 の嵌め込み幅 = 36 * (36/64) = 20.25px、中央配置
    // (x ∈ [21.9, 42.1])。境界とchroma subsamplingから離れた列で審判する。
    let y = height / 2;
    let center = pixel(&rgba, width, width / 2, y);
    assert!(
        center[0] > 150 && center[1] < 90 && center[2] < 90,
        "center column must carry the source video, got {center:?}"
    );
    for x in [6u32, 57u32] {
        let margin = pixel(&rgba, width, x, y);
        assert!(
            margin[0] < 40 && margin[1] < 40 && margin[2] < 40,
            "pillarbox margin at x={x} must stay composition background, got {margin:?}"
        );
    }
}
