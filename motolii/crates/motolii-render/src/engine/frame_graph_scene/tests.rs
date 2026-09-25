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
    fn dynamic_inputs(&mut self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Result<Vec<crate::frame_graph::DynamicInput>, Self::Error> {
        self.0.dynamic_inputs(node, inputs, context)
    }
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
        background: [0.0; 4], look: Default::default()
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
        &mut super::Preparation::new(doc.view().composition().unwrap().unwrap().spec(), Default::default(), super::LegacyCameraSeam::new(ResolvedCamera::default())),
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
        &mut super::Preparation::new(doc.view().composition().unwrap().unwrap().spec(), Default::default(), super::LegacyCameraSeam::new(ResolvedCamera::default())),
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
        &mut super::Preparation::new(doc.view().composition().unwrap().unwrap().spec(), Default::default(), super::LegacyCameraSeam::new(ResolvedCamera::default())),
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


/// A shape's colour is its `shape.fill_color` property; the document's brush is only the default.
/// A property written over a white brush draws in its colour, on every projection, and a keyed one
/// changes the picture over time (the legacy `shapes_at` is the oracle).
#[test]
fn a_shape_draws_its_fill_color_property_over_the_documents_brush() {
    use crate::doc::store::{property, Interp, KeyframeTrack, LayerProjection, PropertyId, Value};
    use crate::doc::eval::Keyframe;
    let fps = Fps::try_new(30, 1).unwrap();
    let at = |frame| crate::doc::core::RationalTime::try_from_frame(frame, fps).unwrap();
    let fill = PropertyId::new(property::SHAPE_FILL_COLOR).unwrap();
    for projection in [LayerProjection::TwoD, LayerProjection::TwoPointFiveD, LayerProjection::ThreeD] {
        let mut doc = document();
        let layer = add_shape(&mut doc, 1, 0, Rgb { r: 1.0, g: 1.0, b: 1.0 });
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe { t: at(0), value: Value::Color([1.0, 0.0, 0.0, 1.0]), interp: Interp::Linear, spatial: None });
        track.insert(Keyframe { t: at(10), value: Value::Color([0.0, 0.0, 1.0, 1.0]), interp: Interp::Linear, spatial: None });
        doc.apply_all([
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(projection), ..Default::default() } },
            Intent::SetTrack { layer, property: fill.clone(), track },
        ]).unwrap();
        let oracle = crate::picture::shapes::shapes_at(&doc.view(), layer, at(0)).unwrap();
        let program = SceneProgram::compile(&doc.view()).unwrap();
        let key = program.content().binding(layer).and_then(|binding| binding.content).unwrap();
        let topology = GraphTopology::try_new(program.nodes(), vec![key]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let frame = graph.evaluate(&mut Executor(&program), at(0), FrameQuality::Export, Generation::new(1)).unwrap();
        assert_eq!(frame.value(key).and_then(|value| value.downcast_ref::<Vec<ShapeNode>>()), Some(&oracle), "{projection:?}: the geometry is the legacy shape at t");

        let mut engine = Engine::new().unwrap();
        let count = |pixels: &[u8], red: bool| pixels.chunks_exact(4).filter(|p| if red { p[0] > 200 && p[2] < 40 } else { p[2] > 200 && p[0] < 40 }).count();
        let first = engine.render_with_camera_override(&doc.view(), at(0), true, None).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        assert!(count(&first, true) > 50, "{projection:?}: the property's red, not the brush's white");
        let last = engine.render_with_camera_override(&doc.view(), at(10), true, None).unwrap();
        assert!(count(&last, false) > 50, "{projection:?}: the keyed colour changes the picture");
    }
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
    let prepared = engine.prepared_contributions;
    let moved = engine.render_with_camera_override(&doc.view(), at(5), true, None).unwrap();
    assert_eq!(engine.prepared_contributions, prepared, "only the placement changed: nothing is lowered or prepared again");
    assert!(engine.frame_claims().iter().all(|c| c.stage != "prepare"), "and nothing claims to have prepared: {:?}", engine.frame_claims());
    assert_ne!(first, moved, "and the picture still moves");
    let fresh = Engine::new().unwrap().render_with_camera_override(&doc.view(), at(5), true, None).unwrap();
    assert_eq!(moved, fresh, "the reused frame is the frame a fresh engine prepares");
}


#[test]
fn only_the_contribution_that_changed_is_prepared_again() {
    use crate::doc::store::{EffectId, EffectInstance, Interp, KeyframeTrack, PropertyId, Value};
    use crate::doc::eval::Keyframe;
    let fps = Fps::try_new(30, 1).unwrap();
    let at = |frame| crate::doc::core::RationalTime::try_from_frame(frame, fps).unwrap();
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: 64, height: 64, fps, duration_frames: 30, background: [0.0; 4], look: Default::default() })).unwrap();
    add_shape(&mut doc, 1, 0, Rgb { r: 0.0, g: 1.0, b: 0.0 });
    let blurred = add_shape(&mut doc, 2, 1, Rgb { r: 1.0, g: 0.0, b: 0.0 });
    doc.apply(Intent::SetEffects { layer: blurred, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.blur".into() }] }).unwrap();
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe { t: at(0), value: Value::F64(1.0), interp: Interp::Linear, spatial: None });
    track.insert(Keyframe { t: at(10), value: Value::F64(8.0), interp: Interp::Linear, spatial: None });
    doc.apply(Intent::SetTrack { layer: blurred, property: PropertyId::effect_param(EffectId(0), "radius").unwrap(), track }).unwrap();
    let mut engine = Engine::new().unwrap();
    engine.render_with_camera_override(&doc.view(), at(0), true, None).unwrap();
    let before = engine.prepared_contributions;
    let frame = engine.render_with_camera_override(&doc.view(), at(5), true, None).unwrap();
    assert_eq!(engine.prepared_contributions - before, 1, "the still shape is reused; only the animated blur is prepared");
    let prepared: Vec<_> = engine.frame_claims().iter().filter(|c| c.stage == "prepare").collect();
    assert_eq!(prepared.len(), 1, "exactly one contribution names itself: {prepared:?}");
    assert_eq!((prepared[0].who.as_str(), prepared[0].why.as_str()), ("layer 2", "an effect value changed"));
    let fresh = Engine::new().unwrap().render_with_camera_override(&doc.view(), at(5), true, None).unwrap();
    assert_eq!(frame, fresh, "the mixed frame is the frame a fresh engine prepares");
}


/// The FrameGraph form of the legacy contract `a_repeater_on_a_group_hands_out_the_children_instead_of_the_group`.
#[test]
fn a_repeater_on_a_group_copies_its_children() {
    use crate::doc::store::{EffectId, EffectInstance, EffectScope, PropertyId, Value};
    use crate::extensions::placement;
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: 400, height: 200, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 30, background: [0.0; 4], look: Default::default() })).unwrap();
    let group = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(group),
        Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 30) } },
    ]).unwrap();
    let circle = add_shape(&mut doc, 2, 1, Rgb { r: 1.0, g: 0.0, b: 0.0 });
    let square = add_shape(&mut doc, 3, 2, Rgb { r: 0.0, g: 1.0, b: 0.0 });
    let put = |doc: &mut Document, layer, at: [f64; 2]| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(crate::doc::store::property::POSITION).unwrap(), value: Value::Vec2(at) }).unwrap();
    for child in [circle, square] {
        doc.apply(Intent::SetAttrs { layer: child, patch: LayerAttrsPatch { parent: Some(Some(group)), ..Default::default() } }).unwrap();
    }
    put(&mut doc, group, [100.0, 100.0]);
    put(&mut doc, circle, [10.0, 0.0]);
    put(&mut doc, square, [0.0, 10.0]);
    let repeat = EffectId(0);
    doc.apply_all([
        Intent::SetEffects { layer: group, effects: vec![EffectInstance { id: repeat, plugin_id: placement::REPEAT.to_owned() }] },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "count").unwrap(), value: Value::F64(4.0) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "pick").unwrap(), value: Value::F64(1.0) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "position_each").unwrap(), value: Value::Vec2([50.0, 0.0]) },
    ]).unwrap();
    let placed = |doc: &Document| {
        let mut seen: Vec<(LayerId, u32, [f32; 2])> = scene(doc).layers.iter()
            .filter(|l| l.layer != group)
            .map(|l| (l.layer, l.instance, l.transform.affine.translation.to_array().map(|v| v.round())))
            .collect();
        seen.sort_by_key(|(id, copy, _)| (id.0, *copy));
        seen
    };
    assert_eq!(placed(&doc), [
        (circle, 0, [110.0, 100.0]), (circle, 2, [210.0, 100.0]),
        (square, 1, [150.0, 110.0]), (square, 3, [250.0, 110.0]),
    ], "Each: the copies take the children in turn; the children are not drawn on their own");
    doc.apply(Intent::SetConstant { layer: group, property: PropertyId::effect_scope(repeat), value: Value::Enum(EffectScope::Whole.enum_value()) }).unwrap();
    let whole = placed(&doc);
    assert_eq!(whole.len(), 8, "Whole: every copy carries both children");
    assert!(whole.contains(&(square, 3, [250.0, 110.0])));

    // An effect after the Repeater reads its result: one plate of every copy, drawn once.
    let glow = EffectId(1);
    doc.apply(Intent::SetEffects { layer: group, effects: vec![
        EffectInstance { id: repeat, plugin_id: placement::REPEAT.to_owned() },
        EffectInstance { id: glow, plugin_id: "group.after".into() },
    ] }).unwrap();
    let scene = scene(&doc);
    let plates: Vec<_> = scene.layers.iter().filter(|l| matches!(l.content, crate::frame_graph::SceneContentValue::Plate(_))).collect();
    assert_eq!(plates.len(), 1, "the copies are composed into one picture");
    assert_eq!(plates[0].after_effects.iter().map(|e| e.plugin_id.as_str()).collect::<Vec<_>>(), ["group.after"]);
    let crate::frame_graph::SceneContentValue::Plate(plate) = &plates[0].content else { unreachable!() };
    assert_eq!(plate.members.iter().filter(|m| m.layer.is_some()).count(), 8);
    assert!(plate.members.iter().filter_map(|m| m.layer.as_ref()).all(|l| l.effects.is_empty()), "no copy carries the effect itself");
}
