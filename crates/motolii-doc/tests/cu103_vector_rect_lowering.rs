use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime};
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, Composition, DocParam, Document, EvaluationTime,
    GraphError, ItemEnvelope, StandardShape, Track, TrackItem, VectorContent, VectorRecipe,
};
use motolii_eval::DataTracks;
use motolii_gpu::download_rgba;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession, RenderStep};
use motolii_testkit::cpu_reference::expected_rect_frame;
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

const WIDTH: u32 = 16;
const HEIGHT: u32 = 8;

fn desc() -> FrameDesc {
    FrameDesc::packed(
        WIDTH,
        HEIGHT,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    )
}

fn vector_document(shape: StandardShape) -> Document {
    let mut document = Document::new_current();
    document.composition = Composition::try_new(
        2,
        1,
        document.composition.duration,
        document.composition.fps,
    )
    .unwrap();
    let layer = document.layers.allocate("Rectangle").unwrap();
    let track = document.track_ids.allocate("V1").unwrap();
    document.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: document.composition.duration,
            time_map: Default::default(),
            source: ClipSource::Vector {
                recipe: VectorRecipe {
                    content: VectorContent::StandardShape { shape },
                    modifiers: Vec::new(),
                },
            },
        })],
    });
    document.validate().unwrap();
    document
}

fn rectangle_document() -> Document {
    vector_document(StandardShape::Rect {
        width: DocParam::const_f64(0.5),
        height: DocParam::const_f64(0.5),
    })
}

#[test]
fn vector_rect_lowers_to_one_local_opaque_white_overlay() {
    let runtime = motolii_plugins_firstparty::first_party_runtime().unwrap();
    let built = build_document_frame_graph(
        &rectangle_document(),
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap();
    let overlays: Vec<_> = built
        .graph
        .steps
        .iter()
        .filter_map(|step| match step {
            RenderStep::OverlayRect { overlay, .. } => Some(overlay),
            _ => None,
        })
        .collect();
    assert_eq!(overlays.len(), 1);
    assert_eq!(overlays[0].center.x, 0.0);
    assert_eq!(overlays[0].center.y, 0.0);
    assert_eq!(overlays[0].size.width, 0.5);
    assert_eq!(overlays[0].size.height, 0.5);
    assert_eq!(overlays[0].color, [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn vector_rect_uses_the_canonical_gpu_render_path() {
    let Some(gpu) = gpu_or_skip() else { return };
    let runtime = motolii_plugins_firstparty::first_party_runtime().unwrap();
    let built = build_document_frame_graph(
        &rectangle_document(),
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &runtime,
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
            camera: built.camera,
            video_sources: &[],
            source_time: Some(built.source_time),
            plugins: Some(runtime.executors()),
        },
        Quality::FINAL,
    )
    .unwrap();
    let actual = download_rgba(&gpu, &rendered.texture).unwrap();
    let expected = expected_rect_frame(
        desc(),
        [0, 0, 0, 0],
        [255, 255, 255, 255],
        [0.0, 0.0],
        [0.5, 0.5],
    );
    assert_rgba_close(
        "cu103-vector-rect",
        RgbaImageDesc {
            width: WIDTH,
            height: HEIGHT,
        },
        &actual,
        &expected,
        tol::GPU_RASTER,
    );
}

#[test]
fn unsupported_vector_content_does_not_degrade_to_a_rectangle() {
    let runtime = motolii_plugins_firstparty::first_party_runtime().unwrap();
    let document = vector_document(StandardShape::Ellipse {
        width: DocParam::const_f64(0.5),
        height: DocParam::const_f64(0.5),
    });
    assert!(matches!(
        build_document_frame_graph(
            &document,
            EvaluationTime::new(RationalTime::ZERO),
            desc(),
            &DataTracks::new(),
            &runtime,
            None,
        ),
        Err(GraphError::UnsupportedVectorSource(_))
    ));
}
