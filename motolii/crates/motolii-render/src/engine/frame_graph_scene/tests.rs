use super::*;
use crate::doc::store::{
    BlendMode, Composition, Fps, LayerAttrsPatch, LayerId, LayerMeta, LayerSource,
    LayerTiming, Matte, MatteMode, ShapeNode,
};
use crate::doc::vector::{Brush, Fill, PathSource, Point, Rgb, Shape};
use crate::frame_graph::{
    CompiledGraph, EvaluationContext, FrameQuality, Generation, GraphNode, GraphRevision,
    GraphTopology, NodeExecutor, NodeInputs, NodeValue, SceneProgram, SceneProgramError,
};
use motolii_edit::{Document, Intent};

struct Executor<'a>(&'a SceneProgram);
impl NodeExecutor for Executor<'_> {
    type Error = SceneProgramError;
    fn execute(
        &mut self,
        node: &GraphNode,
        inputs: NodeInputs,
        context: EvaluationContext,
    ) -> Result<NodeValue, Self::Error> {
        self.0.execute(node, &inputs, &context)
    }
}

fn document() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 30,
        background: [0.0; 4],
    })).unwrap();
    doc
}

fn rectangle(rgb: Rgb) -> Vec<ShapeNode> {
    vec![ShapeNode::Leaf(Shape {
        source: PathSource::Rectangle { size: Point { x: 32.0, y: 32.0 } },
        ops: Vec::new(),
        stroke: None,
        fill: Some(Fill { brush: Brush::Solid(rgb), ..Default::default() }),
    })]
}

fn add_shape(doc: &mut Document, id: u64, order: i16, rgb: Rgb) -> LayerId {
    let layer = LayerId(id);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Shape,
                order,
                timing: LayerTiming::place(0, None, 30),
            },
        },
        Intent::SetShapes { layer, shapes: rectangle(rgb) },
    ]).unwrap();
    layer
}

fn scene(doc: &Document) -> SceneValue {
    let program = SceneProgram::compile(&doc.view()).unwrap();
    let root = program.scene().scene;
    let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
    let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
    let mut executor = Executor(&program);
    let frame = graph.evaluate(
        &mut executor,
        crate::doc::core::RationalTime::ZERO,
        FrameQuality::Export,
        Generation::new(1),
    ).unwrap();
    frame.value(root).and_then(|value| value.downcast_ref::<SceneValue>()).unwrap().clone()
}

#[test]
fn track_matte_source_is_auxiliary_not_a_second_draw_layer() {
    let mut doc = document();
    let source = add_shape(&mut doc, 1, 0, Rgb { r: 1.0, g: 1.0, b: 1.0 });
    let target = add_shape(&mut doc, 2, 1, Rgb { r: 1.0, g: 0.0, b: 0.0 });
    doc.apply(Intent::SetAttrs {
        layer: target,
        patch: LayerAttrsPatch {
            matte: Some(Some(Matte { layer: source, mode: MatteMode::Alpha })),
            ..Default::default()
        },
    }).unwrap();

    let scene = scene(&doc);
    let mut engine = Engine::new().unwrap();
    let gpu = engine.prepare_gpu_scene(
        &scene,
        doc.view().composition().unwrap().unwrap().spec(),
        ResolvedCamera::default(),
    ).unwrap();
    assert_eq!(gpu.layers.len(), 1, "matte source must be consumed");
}

#[test]
fn clipping_folds_the_upper_picture_into_its_base() {
    let mut doc = document();
    add_shape(&mut doc, 1, 0, Rgb { r: 0.0, g: 1.0, b: 0.0 });
    let upper = add_shape(&mut doc, 2, 1, Rgb { r: 1.0, g: 0.0, b: 0.0 });
    doc.apply(Intent::SetAttrs {
        layer: upper,
        patch: LayerAttrsPatch { clip_to_below: Some(true), ..Default::default() },
    }).unwrap();

    let scene = scene(&doc);
    let mut engine = Engine::new().unwrap();
    assert!(engine.compositor.effect_programs.is_empty());
    assert!(!engine.compositor.blend_vism.is_compiled());
    assert!(!engine.compositor.matte_vism.is_compiled());
    let gpu = engine.prepare_gpu_scene(
        &scene,
        doc.view().composition().unwrap().unwrap().spec(),
        ResolvedCamera::default(),
    ).unwrap();
    assert_eq!(gpu.layers.len(), 1, "clip upper is not a separate contribution");
    assert!(engine.compositor.effect_programs.is_empty(), "clipping must not compile unrelated effects");
    assert!(engine.compositor.blend_vism.is_compiled());
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
}

#[test]
fn stencil_is_built_as_a_matte_source_and_never_drawn_itself() {
    let mut doc = document();
    add_shape(&mut doc, 1, 0, Rgb { r: 1.0, g: 0.0, b: 0.0 });
    let stencil = add_shape(&mut doc, 2, 1, Rgb { r: 1.0, g: 1.0, b: 1.0 });
    doc.apply(Intent::SetAttrs {
        layer: stencil,
        patch: LayerAttrsPatch { blend_mode: Some(BlendMode::StencilAlpha), ..Default::default() },
    }).unwrap();

    let scene = scene(&doc);
    assert_eq!(
        scene.layers.iter().find(|layer| layer.layer == LayerId(1)).unwrap().matte,
        Some(Matte { layer: stencil, mode: MatteMode::Alpha }),
    );

    let mut engine = Engine::new().unwrap();
    let gpu = engine.prepare_gpu_scene(
        &scene,
        doc.view().composition().unwrap().unwrap().spec(),
        ResolvedCamera::default(),
    ).unwrap();
    assert_eq!(gpu.layers.len(), 1, "stencil itself is auxiliary");
}

#[test]
fn a_lowered_matte_draws_real_pixels_through_the_cassette() {
    let red = |mode| {
        let mut doc = document();
        let source = add_shape(&mut doc, 1, 1, Rgb { r: 1.0, g: 1.0, b: 1.0 });
        let target = add_shape(&mut doc, 2, 0, Rgb { r: 1.0, g: 0.0, b: 0.0 });
        doc.apply(Intent::SetAttrs {
            layer: target,
            patch: LayerAttrsPatch { matte: Some(Some(Matte { layer: source, mode })), ..Default::default() },
        }).unwrap();
        let mut engine = Engine::new().unwrap();
        let pixels = engine.render_with_camera_override(&doc.view(), crate::doc::core::RationalTime::ZERO, true, None).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        assert_eq!(pixels.len(), 64 * 64 * 4);
        pixels.chunks_exact(4).filter(|p| p[0] > 200 && p[1] < 40 && p[2] < 40).count()
    };
    assert!(red(MatteMode::Alpha) > 0, "the target shows where its source covers");
    assert_eq!(red(MatteMode::InvertedAlpha), 0, "and nowhere else");
}


#[test]
fn a_frame_that_only_moves_things_is_not_prepared_again() {
    use crate::doc::store::{property, Interp, KeyframeTrack, PropertyId, Value};
    use crate::doc::eval::Keyframe;
    let mut doc = document();
    let layer = add_shape(&mut doc, 1, 0, Rgb { r: 1.0, g: 1.0, b: 1.0 });
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe { t: crate::doc::core::RationalTime::ZERO, value: Value::Vec2([0.0, 0.0]), interp: Interp::Linear, spatial: None });
    track.insert(Keyframe { t: crate::doc::core::RationalTime::try_from_frame(10, Fps::try_new(30, 1).unwrap()).unwrap(), value: Value::Vec2([30.0, 0.0]), interp: Interp::Linear, spatial: None });
    doc.apply(Intent::SetTrack { layer, property: PropertyId::new(property::POSITION).unwrap(), track }).unwrap();
    let at = |frame| crate::doc::core::RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap();
    let mut engine = Engine::new().unwrap();
    let first = engine.render_with_camera_override(&doc.view(), at(0), true, None).unwrap();
    let prepared = engine.full_prepares;
    let moved = engine.render_with_camera_override(&doc.view(), at(5), true, None).unwrap();
    assert_eq!(engine.full_prepares, prepared, "only the placement changed: nothing is lowered or prepared again");
    assert_ne!(first, moved, "and the picture still moves");
    let fresh = Engine::new().unwrap().render_with_camera_override(&doc.view(), at(5), true, None).unwrap();
    assert_eq!(moved, fresh, "the reused frame is the frame a fresh engine prepares");
}
