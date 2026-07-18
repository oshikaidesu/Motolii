//! M2-D3e P7: previewの正規合成
//! (`build_document_frame_graph` + `render_graph_cached`) と
//! `export_document_video` の同一実経路・同一 Quality::FINAL 画素一致。
//! M3の製品preview entrypointはまだ存在しないため、その追加時には同entrypointへ審判を移す。

use std::collections::BTreeMap;
use std::path::Path;

use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, resolve_asset_path, Asset, AssetId, Clip, ClipSource, Composition,
    DocParam, Document, EffectDefinition, EffectDefinitionId, EffectId, EffectUse, EvaluationTime,
    ItemEnvelope, Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_export::{export_document_video, ExportJob, EXPORT_DOWNLOAD_TIMEOUT};
use motolii_gpu::{download_rgba, GpuCtx, RgbaDownloader, YuvToRgba};
use motolii_media::{probe, Encoder, FrameReader};
use motolii_plugin::TextureRef;
use motolii_plugins_firstparty::first_party_runtime;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession, TextureId};
use motolii_testkit::{
    assert_rgba_close, ffmpeg_or_skip, gpu_or_skip, tmp_dir, tol, RgbaImageDesc,
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

fn make_bg_video(path: &Path) {
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut enc = Encoder::open(path, &desc, FPS, true).unwrap();
    let mut frame = vec![0u8; desc.data_size()];
    for px in frame.chunks_exact_mut(4) {
        px.copy_from_slice(&[200, 40, 40, 255]);
    }
    enc.write_frame(&frame).unwrap();
    enc.finish().unwrap();
}

fn shared_effect_document(project_root: &Path, video_name: &str) -> Document {
    let mut doc = Document::new_current();
    doc.composition = Composition::try_new(
        W as i64,
        H as i64,
        RationalTime::try_new(1, 1).unwrap(),
        FPS,
    )
    .unwrap();

    let video_id = AssetId::from_raw(0);
    doc.assets
        .insert(Asset {
            id: video_id,
            name: "bg".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:d3e-bg".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: Some(video_name.into()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();

    let layer = doc.layers.allocate("bg").unwrap();
    let overlay_layer = doc.layers.allocate("transparent-overlay").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let definition_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        definition_id,
        "core.filter.tint",
        1,
        true,
        BTreeMap::from([("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0]))]),
        Default::default(),
    ));
    let use_id = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let mut envelope = ItemEnvelope::new(layer);
    envelope.effects.push(EffectUse {
        id: use_id,
        definition_id,
    });
    let overlay_use_id = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let mut overlay_envelope = ItemEnvelope::new(overlay_layer);
    overlay_envelope.effects.push(EffectUse {
        id: overlay_use_id,
        definition_id,
    });
    doc.tracks.push(Track {
        id: track,
        items: vec![
            TrackItem::Clip(Clip {
                envelope,
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(1, 1).unwrap(),
                time_map: TimeMap::identity(),
                source: ClipSource::asset_video_only(video_id),
            }),
            TrackItem::Clip(Clip {
                envelope: overlay_envelope,
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
            }),
        ],
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

    let mut session = RenderSession::new(gpu);
    let rendered = render_graph_cached(
        gpu,
        &mut session,
        timeline,
        &built.graph,
        &RenderGraphInputs {
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
fn p7_preview_and_export_share_final_render_path() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };
    let dir = tmp_dir("d3e-preview-export");
    let video = dir.join("bg.mp4");
    let output = dir.join("shared-effect.mp4");
    make_bg_video(&video);

    let doc = shared_effect_document(&dir, "bg.mp4");
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
        "d3e-p7-preview-export",
        image_desc(),
        &preview,
        &exported,
        tol::GPU_RASTER,
    );
}
