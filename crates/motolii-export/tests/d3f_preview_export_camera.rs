//! M2-D3f: 非既定 Document camera について preview と export が
//! 同一 FINAL render 経路（`built.camera` 経由）で画素一致することの意味審判。

use std::collections::BTreeMap;
use std::path::Path;

use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, resolve_asset_path, Asset, AssetId, Clip, ClipSource,
    CompCameraDoc, Composition, DocParam, Document, EvaluationTime, ItemEnvelope, Track, TrackItem,
    RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_export::{export_document_video, ExportJob, EXPORT_DOWNLOAD_TIMEOUT};
use motolii_gpu::{download_rgba, GpuCtx, RgbaDownloader, YuvToRgba};
use motolii_media::{probe, Encoder, FrameReader};
use motolii_plugin::TextureRef;
use motolii_plugins_firstparty::first_party_runtime;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession, TextureId};
use motolii_testkit::{
    assert_rgba_close, compare_rgba, ffmpeg_or_skip, gpu_or_skip, tmp_dir, tol, RgbaImageDesc,
};

const W: u32 = 32;
const H: u32 = 24;
const FPS: Fps = match Fps::try_new(12, 1) {
    Ok(fps) => fps,
    Err(_) => panic!("invalid const fps"),
};

fn image_desc() -> RgbaImageDesc {
    RgbaImageDesc {
        width: W,
        height: H,
    }
}

fn non_default_camera() -> CompCameraDoc {
    // zoom-in は枠外透明を出さず H.264 復号の opaque alpha と衝突しない
    CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([0.0, 0.0]),
        roll_radians: DocParam::const_f64(0.0),
        height: DocParam::const_f64(0.75),
    }
}

fn make_bg_video(path: &Path) {
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut enc = Encoder::open(path, &desc, FPS, true).unwrap();
    let mut frame = vec![0u8; desc.data_size()];
    for px in frame.chunks_exact_mut(4) {
        px.copy_from_slice(&[200, 40, 40, 255]);
    }
    // 均一赤だけでは camera 差が RGB に出ないため、H.264 往復を崩さない最小コーナー信号を置く
    frame[0..4].copy_from_slice(&[255, 255, 255, 255]);
    enc.write_frame(&frame).unwrap();
    enc.finish().unwrap();
}

fn camera_document(project_root: &Path, video_name: &str, camera: CompCameraDoc) -> Document {
    let mut doc = Document::new_current();
    doc.composition = Composition::try_new(
        W as i64,
        H as i64,
        RationalTime::try_new(1, 1).unwrap(),
        FPS,
    )
    .unwrap();
    doc.composition.camera = camera;

    let video_id = AssetId::from_raw(0);
    doc.assets
        .insert(Asset {
            id: video_id,
            name: "bg".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:d3f-bg".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: Some(video_name.into()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();

    let bg_layer = doc.layers.allocate("bg").unwrap();
    let overlay_layer = doc.layers.allocate("overlay").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();

    let mut bg = Clip {
        envelope: ItemEnvelope::new(bg_layer),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::asset_video_only(video_id),
    };
    bg.envelope.layer_id = bg_layer;

    let mut overlay = Clip {
        envelope: ItemEnvelope::new(overlay_layer),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: RECT_LAYER_SOURCE.into(),
            effect_version: 1,
            params: BTreeMap::from([
                ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                ("size".into(), DocParam::const_vec2([0.2, 0.2])),
                ("color".into(), DocParam::const_color([1.0, 1.0, 1.0, 0.0])),
            ]),
            extra: Default::default(),
        },
    };
    overlay.envelope.layer_id = overlay_layer;

    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(bg), TrackItem::Clip(overlay)],
    });
    doc.validate().unwrap();
    let _ = project_root;
    doc
}

fn render_preview_composition_rgba(gpu: &GpuCtx, doc: &Document, project_root: &Path) -> Vec<u8> {
    let runtime = first_party_runtime().unwrap();
    let timeline = RationalTime::ZERO;
    let asset_id = AssetId::from_raw(0);
    let asset = doc.assets.get(asset_id).expect("video asset");
    let path = resolve_asset_path(asset, Some(project_root)).expect("video path");
    let info = probe(&path).unwrap();
    let desc = FrameDesc::packed(
        info.width,
        info.height,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    );
    let built = build_document_frame_graph(
        doc,
        EvaluationTime::new(timeline),
        desc,
        &DataTracks::new(),
        &runtime,
        Some(project_root),
    )
    .unwrap();

    let mut yuv = YuvToRgba::new(gpu);
    let mut backgrounds = Vec::with_capacity(built.video_slots.len());
    for slot in &built.video_slots {
        let asset = doc.assets.get(slot.asset).expect("slot asset");
        let path = resolve_asset_path(asset, Some(project_root)).expect("slot path");
        let info = probe(&path).unwrap();
        let mut reader = FrameReader::open(&path, &info, 0).unwrap();
        let frame = reader.next_frame().unwrap().expect("video frame");
        backgrounds.push(yuv.convert(gpu, &frame).unwrap());
    }
    let video_inputs: Vec<(TextureId, TextureRef<'_>)> = built
        .video_slots
        .iter()
        .zip(backgrounds.iter())
        .map(|(slot, texture)| (slot.texture_id, TextureRef { texture, desc }))
        .collect();

    let camera = built.camera;

    let mut session = RenderSession::new(gpu);
    let rendered = render_graph_cached(
        gpu,
        &mut session,
        timeline,
        &built.graph,
        &RenderGraphInputs {
            camera,
            video_sources: &video_inputs,
            source_time: Some(built.source_time),
            plugins: Some(runtime.executors()),
        },
        Quality::FINAL,
    )
    .unwrap();
    download_rgba(gpu, &rendered.texture).unwrap()
}

fn decode_exported_rgba(
    gpu: &GpuCtx,
    path: &Path,
    yuv: &mut YuvToRgba,
    downloader: &mut RgbaDownloader,
) -> Vec<u8> {
    let info = probe(path).unwrap();
    let mut reader = FrameReader::open(path, &info, 0).unwrap();
    let frame = reader.next_frame().unwrap().expect("exported frame");
    let texture = yuv.convert(gpu, &frame).unwrap();
    downloader
        .download(gpu, &texture, EXPORT_DOWNLOAD_TIMEOUT)
        .unwrap()
}

#[test]
fn non_default_camera_preview_and_export_share_final_render_path() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else {
        return;
    };
    let dir = tmp_dir("d3f-preview-export-camera");
    let video = dir.join("bg.mp4");
    let output = dir.join("non-default-camera.mp4");
    make_bg_video(&video);

    let doc = camera_document(&dir, "bg.mp4", non_default_camera());
    let runtime = first_party_runtime().unwrap();

    export_document_video(
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

    let preview = render_preview_composition_rgba(&gpu, &doc, &dir);
    let mut yuv = YuvToRgba::new(&gpu);
    let mut downloader = RgbaDownloader::new();
    let exported = decode_exported_rgba(&gpu, &output, &mut yuv, &mut downloader);
    assert_rgba_close(
        "d3f-non-default-preview-export",
        image_desc(),
        &preview,
        &exported,
        tol::GPU_RASTER,
    );
}

#[test]
fn non_default_camera_preview_differs_from_default_camera_preview() {
    let Some(gpu) = gpu_or_skip() else {
        return;
    };
    let dir = tmp_dir("d3f-camera-preview-diff");
    let video = dir.join("bg.mp4");
    make_bg_video(&video);

    let default_doc = camera_document(&dir, "bg.mp4", CompCameraDoc::default_planar_orthographic());
    let non_default_doc = camera_document(&dir, "bg.mp4", non_default_camera());

    let default_preview = render_preview_composition_rgba(&gpu, &default_doc, &dir);
    let non_default_preview = render_preview_composition_rgba(&gpu, &non_default_doc, &dir);
    let diff = compare_rgba(image_desc(), &default_preview, &non_default_preview).unwrap();
    assert!(
        diff.stats.max_abs_diff > tol::GPU_RASTER,
        "default vs non-default camera must differ: max={}",
        diff.stats.max_abs_diff
    );
}
