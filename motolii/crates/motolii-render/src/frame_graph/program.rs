use std::collections::{BTreeMap, BTreeSet};

use crate::doc::store::StoreView;

use super::{AnalysisProgram, AnalysisProgramError, CameraProgram, CameraProgramError, ContentProgram, ContentProgramError, DynamicInput, EffectProgram, EffectProgramError, EvaluationContext, FlowProgram, FlowProgramError, GraphNode, GroupBackgroundProgram, GroupBackgroundProgramError, LookbehindProgram, LookbehindProgramError, MaskProgram, MaskProgramError, MotionProgram, MotionProgramError, NodeInputs, NodeKey, NodeValue, OverlayProgram, OverlayProgramError, ParticleProgram, ParticleProgramError, PlacementProgram, PlacementProgramError, PropertyProgram, PropertyProgramError, RelationProgram, RelationProgramError, SceneNodeError, SolverProgram, SolverProgramError, SceneNodeProgram, SceneProgramNodes, TextProgram, TextProgramError, TransformProgram, TransformProgramError, VisibilityProgram, VisibilityProgramError};

#[derive(Debug)]
pub enum SceneProgramError {
    Analysis(AnalysisProgramError),
    Property(PropertyProgramError),
    Content(ContentProgramError),
    Transform(TransformProgramError),
    Flow(FlowProgramError),
    Text(TextProgramError),
    Scene(SceneNodeError),
    Camera(CameraProgramError),
    Effect(EffectProgramError),
    Mask(MaskProgramError),
    Group(GroupBackgroundProgramError),
    Visibility(VisibilityProgramError),
    Placement(PlacementProgramError),
    Motion(MotionProgramError),
    Lookbehind(LookbehindProgramError),
    Particle(ParticleProgramError),
    Overlay(OverlayProgramError),
    Relation(RelationProgramError),
    Solver(SolverProgramError),
    Unsupported(super::NodeKind),
}
impl std::fmt::Display for SceneProgramError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for SceneProgramError {}
impl From<AnalysisProgramError> for SceneProgramError { fn from(value: AnalysisProgramError) -> Self { Self::Analysis(value) } }
impl From<PropertyProgramError> for SceneProgramError { fn from(value: PropertyProgramError) -> Self { Self::Property(value) } }
impl From<ContentProgramError> for SceneProgramError { fn from(value: ContentProgramError) -> Self { Self::Content(value) } }
impl From<TransformProgramError> for SceneProgramError { fn from(value: TransformProgramError) -> Self { Self::Transform(value) } }
impl From<FlowProgramError> for SceneProgramError { fn from(value: FlowProgramError) -> Self { Self::Flow(value) } }
impl From<TextProgramError> for SceneProgramError { fn from(value: TextProgramError) -> Self { Self::Text(value) } }
impl From<SceneNodeError> for SceneProgramError { fn from(value: SceneNodeError) -> Self { Self::Scene(value) } }
impl From<CameraProgramError> for SceneProgramError { fn from(value: CameraProgramError) -> Self { Self::Camera(value) } }
impl From<EffectProgramError> for SceneProgramError { fn from(value: EffectProgramError) -> Self { Self::Effect(value) } }
impl From<MaskProgramError> for SceneProgramError { fn from(value: MaskProgramError) -> Self { Self::Mask(value) } }
impl From<GroupBackgroundProgramError> for SceneProgramError { fn from(value: GroupBackgroundProgramError) -> Self { Self::Group(value) } }
impl From<VisibilityProgramError> for SceneProgramError { fn from(value: VisibilityProgramError) -> Self { Self::Visibility(value) } }
impl From<PlacementProgramError> for SceneProgramError { fn from(value: PlacementProgramError) -> Self { Self::Placement(value) } }
impl From<MotionProgramError> for SceneProgramError { fn from(value: MotionProgramError) -> Self { Self::Motion(value) } }
impl From<LookbehindProgramError> for SceneProgramError { fn from(value: LookbehindProgramError) -> Self { Self::Lookbehind(value) } }
impl From<ParticleProgramError> for SceneProgramError { fn from(value: ParticleProgramError) -> Self { Self::Particle(value) } }
impl From<OverlayProgramError> for SceneProgramError { fn from(value: OverlayProgramError) -> Self { Self::Overlay(value) } }
impl From<RelationProgramError> for SceneProgramError { fn from(value: RelationProgramError) -> Self { Self::Relation(value) } }
impl From<SolverProgramError> for SceneProgramError { fn from(value: SolverProgramError) -> Self { Self::Solver(value) } }

/// One immutable revision program. Compiler bindings are the only place that
/// remembers layer ids; runtime execution follows content-addressed edges.
pub struct SceneProgram {
    analysis: AnalysisProgram,
    properties: PropertyProgram,
    content: ContentProgram,
    transforms: TransformProgram,
    flow: FlowProgram,
    text: TextProgram,
    scene: SceneNodeProgram,
    camera: CameraProgram,
    effects: EffectProgram,
    masks: MaskProgram,
    groups: GroupBackgroundProgram,
    visibility: VisibilityProgram,
    placements: PlacementProgram,
    motion: MotionProgram,
    lookbehind: LookbehindProgram,
    particles: ParticleProgram,
    overlay: OverlayProgram,
    relations: RelationProgram,
    solver: SolverProgram,
    nodes: BTreeMap<NodeKey, GraphNode>,
    roots: BTreeSet<NodeKey>,
}

impl SceneProgram {
    pub fn compile(view: &StoreView<'_>) -> Result<Self, SceneProgramError> {
        let properties = PropertyProgram::compile(view)?;
        let visibility = VisibilityProgram::compile(view, &properties)?;
        let content = ContentProgram::compile(view, &properties)?;
        let flow = FlowProgram::compile(view, &properties, &content)?;
        let transforms = TransformProgram::compile(view, &properties, &flow)?;
        let effects = EffectProgram::compile(view, &properties)?;
        let motion = MotionProgram::compile(view, &properties, &effects, &transforms, &flow)?;
        let particles = ParticleProgram::compile(view, &properties)?;
        let text = TextProgram::compile(view, &content, &flow, &properties)?;
        let groups = GroupBackgroundProgram::compile(view, &properties, &flow)?;
        let masks = MaskProgram::compile(view, &properties)?;
        let analysis = AnalysisProgram::compile(
            view, &properties, &content, &transforms, &visibility, &effects, &masks, &text, &groups, &particles,
        )?;
        let placements = PlacementProgram::compile(view, &properties, &effects, &transforms, &analysis)?;
        let relations = RelationProgram::compile(view, &properties)?;
        let solver = SolverProgram::compile(view, &properties, &relations, &flow)?;
        let scene = SceneNodeProgram::compile(view, &properties, &content, &transforms, &flow, &text, &groups, &effects, &masks, &visibility, &placements, &motion, &particles)?;
        let lookbehind = LookbehindProgram::compile(view, scene.output().scene)?;
        let camera = CameraProgram::compile(view, &properties, &transforms)?;
        let overlay = OverlayProgram::compile(view, &effects, lookbehind.key(), solver.key(), camera.key())?;
        let mut nodes = BTreeMap::new();
        for node in properties.nodes().chain(visibility.nodes()).chain(content.nodes()).chain(transforms.nodes()).chain(flow.nodes()).chain(effects.nodes()).chain(motion.nodes()).chain(particles.nodes()).chain(text.nodes()).chain(groups.nodes()).chain(masks.nodes()).chain(analysis.nodes()).chain(placements.nodes()).chain(relations.nodes()).chain(std::iter::once(solver.node())).chain(scene.nodes()).chain(std::iter::once(lookbehind.node())).chain(std::iter::once(camera.node())).chain(overlay.nodes()) { nodes.insert(node.key(), node); }
        let roots = BTreeSet::from([lookbehind.key(), camera.key(), solver.key()]);
        Ok(Self { analysis, properties, content, transforms, flow, text, scene, camera, effects, masks, groups, visibility, placements, motion, lookbehind, particles, overlay, relations, solver, nodes, roots })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn roots(&self) -> impl ExactSizeIterator<Item = NodeKey> + '_ { self.roots.iter().copied() }
    pub fn analysis(&self) -> &AnalysisProgram { &self.analysis }
    pub fn properties(&self) -> &PropertyProgram { &self.properties }
    pub fn content(&self) -> &ContentProgram { &self.content }
    pub fn transforms(&self) -> &TransformProgram { &self.transforms }
    pub fn flow(&self) -> &FlowProgram { &self.flow }
    pub fn text(&self) -> &TextProgram { &self.text }
    pub fn scene(&self) -> SceneProgramNodes { SceneProgramNodes { scene: self.lookbehind.key() } }
    pub fn base_scene(&self) -> SceneProgramNodes { self.scene.output() }
    pub fn camera(&self) -> NodeKey { self.camera.key() }
    pub fn visibility(&self) -> &VisibilityProgram { &self.visibility }
    pub fn placements(&self) -> &PlacementProgram { &self.placements }
    pub fn motion(&self) -> &MotionProgram { &self.motion }
    pub fn particles(&self) -> &ParticleProgram { &self.particles }
    pub fn overlay(&self) -> &OverlayProgram { &self.overlay }
    pub fn relations(&self) -> &RelationProgram { &self.relations }
    pub fn solver(&self) -> &SolverProgram { &self.solver }

    pub fn dynamic_inputs(&self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Result<Vec<DynamicInput>, SceneProgramError> {
        if let Some(requests) = self.analysis.dynamic_inputs(node, inputs, context) {
            return requests.map_err(Into::into);
        }
        if let Some(requests) = self.placements.dynamic_inputs(node, inputs, context) {
            return requests.map_err(Into::into);
        }
        if let Some(requests) = self.motion.dynamic_inputs(node, inputs, context) {
            return requests.map_err(Into::into);
        }
        if let Some(requests) = self.text.dynamic_inputs(node, inputs, context) {
            return requests.map_err(Into::into);
        }
        if let Some(requests) = self.lookbehind.dynamic_inputs(node, inputs, context) {
            return requests.map_err(Into::into);
        }
        if let Some(requests) = self.overlay.dynamic_inputs(node, inputs, context) {
            return requests.map_err(Into::into);
        }
        if let Some(requests) = self.particles.dynamic_inputs(node, inputs, context) {
            return requests.map_err(Into::into);
        }
        if let Some(requests) = self.scene.dynamic_inputs(node, inputs, context) {
            return requests.map_err(Into::into);
        }
        Ok(Vec::new())
    }

    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Result<NodeValue, SceneProgramError> {
        if let Some(value) = self.properties.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.content.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.transforms.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.flow.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.text.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.scene.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.camera.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.effects.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.masks.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.groups.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.visibility.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.analysis.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.placements.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.motion.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.lookbehind.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.overlay.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.particles.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.relations.execute(node, inputs, context) { return value.map_err(Into::into); }
        if let Some(value) = self.solver.execute(node, inputs, context) { return value.map_err(Into::into); }
        Err(SceneProgramError::Unsupported(node.identity().kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::eval::Value;
    use crate::doc::store::{LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> { self.0.execute(node, &inputs, &context) }
    }

    #[test]
    fn one_revision_program_evaluates_authored_values_and_shared_content_without_store_reads() {
        let mut doc = Document::new();
        let cube = LayerId(1);
        let opacity = PropertyId::new(crate::doc::store::property::OPACITY).unwrap();
        doc.apply_all([
            Intent::AddLayer(cube),
            Intent::SetMeta { layer: cube, meta: LayerMeta { source: LayerSource::File { path: "/builtins/cube-v1.obj".into(), fingerprint: Some("cube-v1".into()) }, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetConstant { layer: cube, property: opacity.clone(), value: Value::F64(0.75) },
        ]).unwrap();
        let program = SceneProgram::compile(&doc.view()).unwrap();
        let topology = GraphTopology::try_new(program.nodes(), program.roots().collect()).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let frame = graph.evaluate(&mut executor, crate::doc::core::RationalTime::ZERO, FrameQuality::Export, Generation::new(1)).unwrap();
        let property = program.properties().node_for(cube, &opacity).unwrap();
        let material = program.content().binding(cube).unwrap().material.unwrap();
        assert_eq!(frame.value(property).and_then(|value| value.downcast_ref::<Value>()), Some(&Value::F64(0.75)));
        assert!(frame.value(material).and_then(|value| value.downcast_ref::<super::super::MaterialValue>()).is_some());
        let scene = frame.value(program.scene().scene).and_then(|value| value.downcast_ref::<super::super::SceneValue>()).unwrap();
        assert_eq!(scene.layers.len(), 1);
        assert_eq!(scene.layers[0].opacity, 0.75);
        assert!(matches!(scene.layers[0].content, super::super::SceneContentValue::Material(_)));
    }


    #[test]
    fn scene_visibility_matches_the_legacy_resolver_for_hidden_solo_and_trim() {
        fn ids(scene: &crate::frame_graph::SceneValue) -> Vec<LayerId> {
            scene.layers.iter().map(|layer| layer.layer).collect()
        }

        let fps = crate::doc::core::Fps::try_new(30, 1).unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(crate::doc::store::Composition {
            width: 640,
            height: 360,
            fps,
            duration_frames: 120,
            background: [0.0; 4],
        })).unwrap();

        let normal = LayerId(20);
        let solo = LayerId(21);
        for (layer, order, timing) in [
            (normal, 0, LayerTiming::place(0, None, 120)),
            (solo, 1, LayerTiming { start: 60, duration: 30, source_in: 0, speed: crate::doc::store::Speed::NORMAL }),
        ] {
            doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Null, order, timing } },
            ]).unwrap();
        }
        doc.apply(Intent::SetAttrs {
            layer: solo,
            patch: crate::doc::store::LayerAttrsPatch { solo: Some(true), ..Default::default() },
        }).unwrap();

        for (generation, frame_no) in [(1, 0), (2, 60), (3, 90)] {
            let at = crate::doc::core::RationalTime::try_from_frame(frame_no, fps).unwrap();
            let expected = crate::picture::resolve::resolved_layers(&doc.view(), at).unwrap()
                .into_iter().filter(|layer| !matches!(layer.source, LayerSource::Camera | LayerSource::Stage))
                .map(|layer| layer.id).collect::<Vec<_>>();

            let program = SceneProgram::compile(&doc.view()).unwrap();
            let root = program.scene().scene;
            let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
            let mut graph = CompiledGraph::with_topology(GraphRevision::new(generation), topology);
            let mut executor = Executor(&program);
            let evaluated = graph.evaluate(&mut executor, at, FrameQuality::Export, Generation::new(generation)).unwrap();
            let scene = evaluated.value(root).and_then(|value| value.downcast_ref::<crate::frame_graph::SceneValue>()).unwrap();
            assert_eq!(ids(scene), expected, "frame {frame_no}");
        }

        doc.apply(Intent::SetAttrs {
            layer: solo,
            patch: crate::doc::store::LayerAttrsPatch { hidden: Some(true), ..Default::default() },
        }).unwrap();
        let at = crate::doc::core::RationalTime::try_from_frame(60, fps).unwrap();
        let expected = crate::picture::resolve::resolved_layers(&doc.view(), at).unwrap()
            .into_iter().map(|layer| layer.id).collect::<Vec<_>>();
        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.scene().scene;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(4), topology);
        let mut executor = Executor(&program);
        let evaluated = graph.evaluate(&mut executor, at, FrameQuality::Export, Generation::new(4)).unwrap();
        let scene = evaluated.value(root).and_then(|value| value.downcast_ref::<crate::frame_graph::SceneValue>()).unwrap();
        assert_eq!(ids(scene), expected);
    }

    #[test]
    fn media_frame_uses_layer_timing_and_disappears_outside_the_trim() {
        let mut doc = Document::new();
        let clip = LayerId(7);
        let fps = crate::doc::core::Fps::try_new(30, 1).unwrap();
        doc.apply_all([
            Intent::SetComposition(crate::doc::store::Composition {
                width: 640,
                height: 360,
                fps,
                duration_frames: 120,
                background: [0.0, 0.0, 0.0, 1.0],
            }),
            Intent::AddLayer(clip),
            Intent::SetMeta {
                layer: clip,
                meta: LayerMeta {
                    source: LayerSource::File {
                        path: "/motolii-test/clip.mp4".into(),
                        fingerprint: Some("clip-v1".into()),
                    },
                    order: 0,
                    timing: LayerTiming {
                        start: 10,
                        duration: 20,
                        source_in: 5,
                        speed: crate::doc::store::Speed::try_new(2, 1).unwrap(),
                    },
                },
            },
        ])
        .unwrap();

        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.scene().scene;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);

        let at = crate::doc::core::RationalTime::try_from_frame(12, fps).unwrap();
        let frame = graph
            .evaluate(&mut executor, at, FrameQuality::Export, Generation::new(1))
            .unwrap();
        let scene = frame
            .value(root)
            .and_then(|value| value.downcast_ref::<crate::frame_graph::SceneValue>())
            .unwrap();
        match &scene.layers[0].content {
            crate::frame_graph::SceneContentValue::Media { time, .. } => {
                assert_eq!(*time, crate::doc::core::RationalTime::try_from_frame(9, fps).unwrap());
            }
            other => panic!("in-range file frame did not lower to media: {other:?}"),
        }

        let before = crate::doc::core::RationalTime::try_from_frame(9, fps).unwrap();
        let frame = graph
            .evaluate(&mut executor, before, FrameQuality::Export, Generation::new(2))
            .unwrap();
        let scene = frame
            .value(root)
            .and_then(|value| value.downcast_ref::<crate::frame_graph::SceneValue>())
            .unwrap();
        assert!(matches!(scene.layers[0].content, crate::frame_graph::SceneContentValue::None));
    }

    #[test]
    fn media_frame_time_remap_overrides_speed_like_the_legacy_resolver() {
        let mut doc = Document::new();
        let clip = LayerId(8);
        let fps = crate::doc::core::Fps::try_new(30, 1).unwrap();
        doc.apply_all([
            Intent::SetComposition(crate::doc::store::Composition {
                width: 640,
                height: 360,
                fps,
                duration_frames: 120,
                background: [0.0, 0.0, 0.0, 1.0],
            }),
            Intent::AddLayer(clip),
            Intent::SetMeta {
                layer: clip,
                meta: LayerMeta {
                    source: LayerSource::File {
                        path: "/motolii-test/remap.mp4".into(),
                        fingerprint: Some("clip-remap".into()),
                    },
                    order: 0,
                    timing: LayerTiming {
                        start: 0,
                        duration: 120,
                        source_in: 3,
                        speed: crate::doc::store::Speed::try_new(2, 1).unwrap(),
                    },
                },
            },
            Intent::SetConstant {
                layer: clip,
                property: PropertyId::new(crate::doc::store::property::TIME_REMAP).unwrap(),
                value: Value::F64(42.75),
            },
        ])
        .unwrap();

        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.scene().scene;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let at = crate::doc::core::RationalTime::try_from_frame(10, fps).unwrap();
        let frame = graph
            .evaluate(&mut executor, at, FrameQuality::Export, Generation::new(1))
            .unwrap();
        let scene = frame
            .value(root)
            .and_then(|value| value.downcast_ref::<crate::frame_graph::SceneValue>())
            .unwrap();
        match &scene.layers[0].content {
            crate::frame_graph::SceneContentValue::Media { time, .. } => {
                assert_eq!(*time, crate::doc::core::RationalTime::try_from_frame(42, fps).unwrap());
            }
            other => panic!("remapped file frame did not lower to media: {other:?}"),
        }
    }


    #[test]
    fn file_frame_reaches_the_scene_as_timed_media_instead_of_a_material() {
        let mut doc = Document::new();
        let clip = LayerId(9);
        let path = "/motolii-test/nonexistent-clip.mp4";
        let fps = crate::doc::core::Fps::try_new(30, 1).unwrap();
        doc.apply_all([
            Intent::SetComposition(crate::doc::store::Composition {
                width: 640,
                height: 360,
                fps,
                duration_frames: 90,
                background: [0.0, 0.0, 0.0, 1.0],
            }),
            Intent::AddLayer(clip),
            Intent::SetMeta {
                layer: clip,
                meta: LayerMeta {
                    source: LayerSource::File {
                        path: path.into(),
                        fingerprint: Some("fixture-v1".into()),
                    },
                    order: 0,
                    timing: LayerTiming::place(0, None, 90),
                },
            },
        ])
        .unwrap();

        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.scene().scene;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let at = crate::doc::core::RationalTime::try_from_frame(15, fps).unwrap();
        let frame = graph
            .evaluate(&mut executor, at, FrameQuality::Export, Generation::new(1))
            .unwrap();
        let scene = frame
            .value(root)
            .and_then(|value| value.downcast_ref::<crate::frame_graph::SceneValue>())
            .unwrap();

        match &scene.layers[0].content {
            crate::frame_graph::SceneContentValue::Media { source, time } => {
                assert_eq!(source.path, path);
                assert_eq!(*time, at);
            }
            other => panic!("file frame lowered to the wrong scene content: {other:?}"),
        }
    }

}
