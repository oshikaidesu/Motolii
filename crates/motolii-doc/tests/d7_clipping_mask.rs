#![allow(deprecated)]

//! D7: クリッピングマスク合成ゴールデン(#145)。
//!
//! 「下のレイヤーにクリップ」(クリスタ方式)を doc→グラフ→GPU で各 MaskMode を固定する。
//! マスク形状レイヤーは visible=false にしてブレンドを避け、モード差だけを審判する。
//! sRGB ブレンド依存の見た目検証は別テスト(provisional / regenerate マーカー)。
//!
//! MOTOLII_GOLDEN_CLASS: provisional
//! MOTOLII_REGENERATE_WHEN: srgb-blend-to-linear

use std::collections::BTreeMap;

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, ClippingMaskSettings, DocParam, Document,
    EvaluationTime, ItemEnvelope, MaskMode, Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_gpu::download_rgba;
use motolii_nodes::ClippingMaskMode;
use motolii_plugin::reference::register_reference_plugins;
use motolii_plugin::PluginRegistry;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession, RenderStep};
use motolii_testkit::clipping_mask::{clipping_mask_frame, clipping_mask_mul_u8, ClippingMaskRef};
use motolii_testkit::cpu_reference::{expected_rect_frame, premul_over_u8};
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

const W: u32 = 16;
const H: u32 = 8;

fn desc() -> FrameDesc {
    FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true)
}

fn rect_clip(layer: u64, center: [f64; 2], size: [f64; 2], color: [f64; 4]) -> Clip {
    Clip {
        envelope: ItemEnvelope::new(motolii_doc::LayerId::from_raw(layer)),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: RECT_LAYER_SOURCE.into(),
            effect_version: 1,
            params: BTreeMap::from([
                ("center".into(), DocParam::const_vec2(center)),
                ("size".into(), DocParam::const_vec2(size)),
                ("color".into(), DocParam::const_color(color)),
            ]),
            extra: Default::default(),
        },
    }
}

/// A4: 重なりは別 Track。マスク下・コンテンツ上が flatten 順で「直下」になる。
fn clipped_doc(
    mask_center: [f64; 2],
    mask_size: [f64; 2],
    mask_color: [f64; 4],
    content_color: [f64; 4],
    mode: MaskMode,
    mask_visible: bool,
) -> Document {
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let mask_layer = doc.layers.allocate("mask").unwrap();
    let content_layer = doc.layers.allocate("content").unwrap();
    let track_mask = doc.track_ids.allocate("V1").unwrap();
    let track_content = doc.track_ids.allocate("V2").unwrap();

    let mut mask = rect_clip(mask_layer.get(), mask_center, mask_size, mask_color);
    mask.envelope.layer_id = mask_layer;
    mask.envelope.visible = mask_visible;

    let mut content = rect_clip(content_layer.get(), [0.0, 0.0], [1.0, 1.0], content_color);
    content.envelope.layer_id = content_layer;
    content.envelope.clipping_mask = ClippingMaskSettings {
        enabled: true,
        mode,
    };

    doc.tracks.push(Track {
        id: track_mask,
        items: vec![TrackItem::Clip(mask)],
    });
    doc.tracks.push(Track {
        id: track_content,
        items: vec![TrackItem::Clip(content)],
    });
    doc.validate()
        .expect("D7 golden document must validate (A4)");
    doc
}

fn render_doc(doc: &Document) -> Option<Vec<u8>> {
    let gpu = gpu_or_skip()?;
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry).unwrap();
    let built = build_document_frame_graph(
        doc,
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &registry,
        None,
    )
    .unwrap();
    let mut session = RenderSession::new(&gpu);
    let rendered = render_graph_cached(
        &gpu,
        &mut session,
        RationalTime::ZERO,
        &built.graph,
        &RenderGraphInputs {
            video_sources: &[],
            source_time: Some(built.source_time),
            plugins: Some(&registry),
        },
        Quality::FINAL,
    )
    .unwrap();
    Some(download_rgba(&gpu, &rendered.texture).unwrap())
}

fn assert_mode_maps(mode: MaskMode, want: ClippingMaskMode) {
    let doc = clipped_doc(
        [-0.25, 0.0],
        [0.5, 1.0],
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        mode,
        false,
    );
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry).unwrap();
    let built = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &registry,
        None,
    )
    .unwrap();
    let got = built
        .graph
        .steps
        .iter()
        .find_map(|s| match s {
            RenderStep::ApplyMask { mode, .. } => Some(*mode),
            _ => None,
        })
        .expect("clipping_mask.enabled must emit ApplyMask");
    assert_eq!(got, want);
}

#[test]
fn mask_mode_maps_one_to_one_onto_clipping_mask_mode() {
    assert_mode_maps(MaskMode::Alpha, ClippingMaskMode::Alpha);
    assert_mode_maps(MaskMode::Luminance, ClippingMaskMode::Luminance);
    assert_mode_maps(MaskMode::InvertAlpha, ClippingMaskMode::InvertAlpha);
    assert_mode_maps(MaskMode::InvertLuminance, ClippingMaskMode::InvertLuminance);
}

#[test]
fn alpha_clip_hides_content_outside_shape_below() {
    let Some(_) = gpu_or_skip() else { return };
    let doc = clipped_doc(
        [-0.25, 0.0],
        [0.5, 1.0],
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        MaskMode::Alpha,
        false,
    );
    let actual = render_doc(&doc).expect("gpu");
    let expected = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [0, 255, 0, 255],
        [-0.25, 0.0],
        [0.5, 1.0],
    );
    assert_rgba_close(
        "d7-alpha-clip",
        RgbaImageDesc {
            width: W,
            height: H,
        },
        &actual,
        &expected,
        tol::GPU_RASTER,
    );
}

#[test]
fn invert_alpha_clip_shows_content_outside_shape_below() {
    let Some(_) = gpu_or_skip() else { return };
    let doc = clipped_doc(
        [-0.25, 0.0],
        [0.5, 1.0],
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        MaskMode::InvertAlpha,
        false,
    );
    let actual = render_doc(&doc).expect("gpu");
    let mask = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [255, 255, 255, 255],
        [-0.25, 0.0],
        [0.5, 1.0],
    );
    let content = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [0, 255, 0, 255],
        [0.0, 0.0],
        [1.0, 1.0],
    );
    let expected = clipping_mask_frame(&content, &mask, ClippingMaskRef::InvertAlpha);
    assert_rgba_close(
        "d7-invert-alpha-clip",
        RgbaImageDesc {
            width: W,
            height: H,
        },
        &actual,
        &expected,
        tol::GPU_RASTER,
    );
}

#[test]
fn luminance_clip_uses_premul_bt709_not_alpha() {
    let Some(_) = gpu_or_skip() else { return };
    // 不透明赤: alpha=1 だが luma≈0.2126 → Alpha と差が出る。
    let doc = clipped_doc(
        [-0.25, 0.0],
        [0.5, 1.0],
        [1.0, 0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0, 1.0],
        MaskMode::Luminance,
        false,
    );
    let actual = render_doc(&doc).expect("gpu");
    let mask = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [255, 0, 0, 255],
        [-0.25, 0.0],
        [0.5, 1.0],
    );
    let content = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [255, 255, 255, 255],
        [0.0, 0.0],
        [1.0, 1.0],
    );
    let expected = clipping_mask_frame(&content, &mask, ClippingMaskRef::Luminance);
    // Alpha なら左半分が白全量になるので、ルミナンス期待と一致しないことを番兵にする。
    let alpha_trap = clipping_mask_frame(&content, &mask, ClippingMaskRef::Alpha);
    assert_ne!(
        expected, alpha_trap,
        "luminance golden must differ from alpha for red mask"
    );
    assert_rgba_close(
        "d7-luminance-clip",
        RgbaImageDesc {
            width: W,
            height: H,
        },
        &actual,
        &expected,
        tol::GPU_RASTER,
    );
}

#[test]
fn invert_luminance_clip_inverts_bt709_coverage() {
    let Some(_) = gpu_or_skip() else { return };
    let doc = clipped_doc(
        [-0.25, 0.0],
        [0.5, 1.0],
        [1.0, 0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0, 1.0],
        MaskMode::InvertLuminance,
        false,
    );
    let actual = render_doc(&doc).expect("gpu");
    let mask = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [255, 0, 0, 255],
        [-0.25, 0.0],
        [0.5, 1.0],
    );
    let content = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [255, 255, 255, 255],
        [0.0, 0.0],
        [1.0, 1.0],
    );
    let expected = clipping_mask_frame(&content, &mask, ClippingMaskRef::InvertLuminance);
    assert_rgba_close(
        "d7-invert-luminance-clip",
        RgbaImageDesc {
            width: W,
            height: H,
        },
        &actual,
        &expected,
        tol::GPU_RASTER,
    );
}

/// クリスタ方式: 下のシェイプは描画され、上がそのシルエットにクリップされて合成される。
/// Normal 合成は C-1 暫定 sRGB ブレンド依存 → provisional / regenerate 必須。
#[test]
fn krista_style_visible_shape_below_composites_with_clipped_content() {
    let Some(_) = gpu_or_skip() else { return };
    let doc = clipped_doc(
        [-0.25, 0.0],
        [0.5, 1.0],
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        MaskMode::Alpha,
        true,
    );
    let actual = render_doc(&doc).expect("gpu");
    let mask_layer = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [255, 255, 255, 255],
        [-0.25, 0.0],
        [0.5, 1.0],
    );
    let content = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [0, 255, 0, 255],
        [0.0, 0.0],
        [1.0, 1.0],
    );
    let clipped = clipping_mask_frame(&content, &mask_layer, ClippingMaskRef::Alpha);
    let expected: Vec<u8> = mask_layer
        .chunks_exact(4)
        .zip(clipped.chunks_exact(4))
        .flat_map(|(b, f)| premul_over_u8(b.try_into().unwrap(), f.try_into().unwrap()))
        .collect();
    assert_rgba_close(
        "d7-krista-visible-shape-below",
        RgbaImageDesc {
            width: W,
            height: H,
        },
        &actual,
        &expected,
        tol::GPU_RASTER,
    );
}

#[test]
fn clipping_mask_mul_matches_shader_coefficients_for_red() {
    let content = [255, 255, 255, 255];
    let mask = [255, 0, 0, 255];
    let got = clipping_mask_mul_u8(content, mask, ClippingMaskRef::Luminance);
    let f = 0.2126_f64;
    let want = [
        (255.0 * f).round() as u8,
        (255.0 * f).round() as u8,
        (255.0 * f).round() as u8,
        (255.0 * f).round() as u8,
    ];
    assert_eq!(got, want);
    assert_eq!(
        clipping_mask_mul_u8(content, mask, ClippingMaskRef::Alpha),
        [255, 255, 255, 255]
    );
}
