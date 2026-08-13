use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime};
use motolii_doc::{
    build_document_frame_graph, eval_vector_recipe_path, pathgeom, Clip, ClipSource, Composition,
    DocParam, Document, EvaluationTime, ItemEnvelope, ResolvedLayerParams, StandardShape, Track,
    TrackItem, VectorContent, VectorRecipe,
};
use motolii_eval::DataTracks;
use motolii_gpu::download_rgba;
use motolii_nodes::OverlayShape;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession, RenderStep};
use motolii_testkit::cpu_reference::{
    expected_circle_over_pattern, expected_ellipse_frame, expected_rect_frame,
};
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

fn ellipse_document(width: f64, height: f64) -> Document {
    vector_document(StandardShape::Ellipse {
        width: DocParam::const_f64(width),
        height: DocParam::const_f64(height),
    })
}

fn build_graph(document: &Document) -> motolii_doc::DocumentFrameGraph {
    let runtime = motolii_plugins_firstparty::first_party_runtime().unwrap();
    build_document_frame_graph(
        document,
        EvaluationTime::new(RationalTime::ZERO),
        desc(),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap()
}

fn overlay_shapes(graph: &motolii_render::LinearRenderGraph) -> Vec<OverlayShape> {
    graph
        .steps
        .iter()
        .filter_map(|step| match step {
            RenderStep::Overlay { shape, .. } => Some(*shape),
            _ => None,
        })
        .collect()
}

fn render_download(built: &motolii_doc::DocumentFrameGraph, gpu: &motolii_gpu::GpuCtx) -> Vec<u8> {
    let runtime = motolii_plugins_firstparty::first_party_runtime().unwrap();
    let mut session = RenderSession::new(gpu);
    let rendered = render_graph_cached(
        gpu,
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
    download_rgba(gpu, &rendered.texture).unwrap()
}

#[test]
fn vector_ellipse_never_lowers_to_an_axis_aligned_rect_overlay() {
    for (w, h) in [(0.5, 0.5), (0.5, 0.3)] {
        let built = build_graph(&ellipse_document(w, h));
        assert!(
            !built
                .graph
                .steps
                .iter()
                .any(|step| matches!(step, RenderStep::OverlayRect { .. })),
            "ellipse {w}x{h} must not fake an AABB rect overlay"
        );
    }
}

#[test]
fn circular_vector_ellipse_lowers_to_one_overlay_circle() {
    let built = build_graph(&ellipse_document(0.5, 0.5));
    let shapes = overlay_shapes(&built.graph);
    assert_eq!(shapes.len(), 1);
    let OverlayShape::Circle(circle) = shapes[0] else {
        panic!("expected circle overlay, got {:?}", shapes[0]);
    };
    assert_eq!(circle.center.x, 0.0);
    assert_eq!(circle.center.y, 0.0);
    assert_eq!(circle.radius, 0.25);
    assert_eq!(circle.color, [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn non_uniform_vector_ellipse_lowers_to_one_overlay_ellipse() {
    let built = build_graph(&ellipse_document(0.5, 0.3));
    let shapes = overlay_shapes(&built.graph);
    assert_eq!(shapes.len(), 1);
    let OverlayShape::Ellipse(ellipse) = shapes[0] else {
        panic!("expected ellipse overlay, got {:?}", shapes[0]);
    };
    assert_eq!(ellipse.center.x, 0.0);
    assert_eq!(ellipse.center.y, 0.0);
    assert_eq!(ellipse.radius_x, 0.25);
    assert_eq!(ellipse.radius_y, 0.15);
    assert_eq!(ellipse.color, [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn vector_ellipse_rasters_through_the_canonical_gpu_render_path() {
    let Some(gpu) = gpu_or_skip() else { return };
    let transparent = vec![0u8; desc().data_size()];

    let circle = render_download(&build_graph(&ellipse_document(0.5, 0.5)), &gpu);
    assert_rgba_close(
        "cu103-vector-ellipse-circle",
        RgbaImageDesc {
            width: WIDTH,
            height: HEIGHT,
        },
        &circle,
        &expected_circle_over_pattern(desc(), &transparent, [255, 255, 255, 255], [0.0, 0.0], 0.25),
        tol::GPU_RASTER,
    );

    let ellipse = render_download(&build_graph(&ellipse_document(0.5, 0.3)), &gpu);
    assert_rgba_close(
        "cu103-vector-ellipse-non-uniform",
        RgbaImageDesc {
            width: WIDTH,
            height: HEIGHT,
        },
        &ellipse,
        &expected_ellipse_frame(
            desc(),
            [0, 0, 0, 0],
            [255, 255, 255, 255],
            [0.0, 0.0],
            [0.25, 0.15],
        ),
        tol::GPU_RASTER,
    );
}

#[test]
fn vector_rect_evaluates_through_pathgeom_before_overlay() {
    let recipe = VectorRecipe {
        content: VectorContent::StandardShape {
            shape: StandardShape::Rect {
                width: DocParam::const_f64(0.5),
                height: DocParam::const_f64(0.5),
            },
        },
        modifiers: Vec::new(),
    };
    let path = eval_vector_recipe_path(
        &recipe,
        RationalTime::ZERO,
        &DataTracks::new(),
        &ResolvedLayerParams::default(),
    )
    .unwrap();
    let (center, width, height) = pathgeom::axis_aligned_rect(&path).expect("rect path");
    assert_eq!(center, pathgeom::Point { x: 0.0, y: 0.0 });
    assert_eq!(width, 0.5);
    assert_eq!(height, 0.5);
    assert!(pathgeom::axis_aligned_ellipse(&path).is_none());
}

#[test]
fn vector_ellipse_evaluates_as_path_without_becoming_a_rect() {
    let recipe = VectorRecipe {
        content: VectorContent::StandardShape {
            shape: StandardShape::Ellipse {
                width: DocParam::const_f64(0.5),
                height: DocParam::const_f64(0.5),
            },
        },
        modifiers: Vec::new(),
    };
    let path = eval_vector_recipe_path(
        &recipe,
        RationalTime::ZERO,
        &DataTracks::new(),
        &ResolvedLayerParams::default(),
    )
    .unwrap();
    assert!(pathgeom::axis_aligned_rect(&path).is_none());
    assert!(path.contours[0]
        .vertices
        .iter()
        .any(|v| v.out_tangent != pathgeom::Point::ZERO));
    let (center, width, height) = pathgeom::axis_aligned_ellipse(&path).expect("ellipse path");
    assert_eq!(center, pathgeom::Point { x: 0.0, y: 0.0 });
    assert_eq!(width, 0.5);
    assert_eq!(height, 0.5);
}
