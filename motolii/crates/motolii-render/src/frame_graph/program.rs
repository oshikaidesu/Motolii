use std::collections::{BTreeMap, BTreeSet};

use crate::doc::store::StoreView;

use super::{CameraProgram, CameraProgramError, ContentProgram, ContentProgramError, EffectProgram, EffectProgramError, EvaluationContext, FlowProgram, FlowProgramError, GraphNode, GroupBackgroundProgram, GroupBackgroundProgramError, MaskProgram, MaskProgramError, NodeInputs, NodeKey, NodeValue, PropertyProgram, PropertyProgramError, SceneNodeError, SceneNodeProgram, SceneProgramNodes, TextProgram, TextProgramError, TransformProgram, TransformProgramError, VisibilityProgram, VisibilityProgramError};

#[derive(Debug)]
pub enum SceneProgramError {
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
    Unsupported(super::NodeKind),
}
impl std::fmt::Display for SceneProgramError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for SceneProgramError {}
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

/// One immutable revision program. Compiler bindings are the only place that
/// remembers layer ids; runtime execution follows content-addressed edges.
pub struct SceneProgram {
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
        let text = TextProgram::compile(view, &content, &flow)?;
        let groups = GroupBackgroundProgram::compile(view, &properties, &flow)?;
        let effects = EffectProgram::compile(view, &properties)?;
        let masks = MaskProgram::compile(view, &properties)?;
        let scene = SceneNodeProgram::compile(view, &properties, &content, &transforms, &text, &groups, &effects, &masks, &visibility)?;
        let camera = CameraProgram::compile(view, &properties, &transforms)?;
        let mut nodes = BTreeMap::new();
        for node in properties.nodes().chain(visibility.nodes()).chain(content.nodes()).chain(transforms.nodes()).chain(flow.nodes()).chain(text.nodes()).chain(groups.nodes()).chain(effects.nodes()).chain(masks.nodes()).chain(scene.nodes()).chain(std::iter::once(camera.node())) { nodes.insert(node.key(), node); }
        let roots = BTreeSet::from([scene.output().scene, camera.key()]);
        Ok(Self { properties, content, transforms, flow, text, scene, camera, effects, masks, groups, visibility, nodes, roots })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn roots(&self) -> impl ExactSizeIterator<Item = NodeKey> + '_ { self.roots.iter().copied() }
    pub fn properties(&self) -> &PropertyProgram { &self.properties }
    pub fn content(&self) -> &ContentProgram { &self.content }
    pub fn transforms(&self) -> &TransformProgram { &self.transforms }
    pub fn flow(&self) -> &FlowProgram { &self.flow }
    pub fn text(&self) -> &TextProgram { &self.text }
    pub fn scene(&self) -> SceneProgramNodes { self.scene.output() }
    pub fn camera(&self) -> NodeKey { self.camera.key() }
    pub fn visibility(&self) -> &VisibilityProgram { &self.visibility }

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
