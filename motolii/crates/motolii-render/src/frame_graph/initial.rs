//! The deliberately coarse first vertical slice for F2.
//!
//! These are typed node handles, not renderer values. They make the one
//! shared evaluation graph explicit before the existing evaluators are moved
//! behind node adapters.

use super::{
    CompilerOutput, GraphBuilder, NodeIdentity, NodeKey, NodeKind, TimeDependency, TopologyError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedWorld(pub NodeKey);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextDocuments(pub NodeKey);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeDocuments(pub NodeKey);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentCamera(pub NodeKey);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedScene(pub NodeKey);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CameraRoot(pub NodeKey);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageRoot(pub NodeKey);

/// The first cutover topology. The world is evaluated once at the frame time,
/// then text, shapes, and camera read that one result. The two view roots are
/// produced only after the scene has been assembled.
pub struct InitialTopology {
    pub output: CompilerOutput,
    pub world: ResolvedWorld,
    pub text: TextDocuments,
    pub shapes: ShapeDocuments,
    pub document_camera: DocumentCamera,
    pub scene: SharedScene,
    pub camera: CameraRoot,
    pub stage: StageRoot,
}

pub fn build_initial_topology() -> Result<InitialTopology, TopologyError> {
    let mut builder = GraphBuilder::new();

    let world = ResolvedWorld(builder.intern(time_dependent(NodeKind::ResolvedWorld, Vec::new()))?);
    let text =
        TextDocuments(builder.intern(time_dependent(NodeKind::TextDocuments, vec![world.0]))?);
    let shapes =
        ShapeDocuments(builder.intern(time_dependent(NodeKind::ShapeDocuments, vec![world.0]))?);
    let document_camera =
        DocumentCamera(builder.intern(time_dependent(NodeKind::DocumentCamera, vec![world.0]))?);

    let contribution = builder.intern(time_dependent(
        NodeKind::SharedScene,
        vec![world.0, text.0, shapes.0, document_camera.0],
    ))?;
    builder.set_contributions([contribution]);
    let output = builder.finish(Some(document_camera.0))?;

    Ok(InitialTopology {
        world,
        text,
        shapes,
        document_camera,
        scene: SharedScene(output.scene),
        camera: CameraRoot(output.camera),
        stage: StageRoot(output.stage),
        output,
    })
}

fn time_dependent(kind: NodeKind, inputs: Vec<NodeKey>) -> NodeIdentity {
    let mut identity = NodeIdentity::new(kind, inputs);
    identity.time_dependency = TimeDependency::Exact;
    identity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_topology_has_one_shared_scene_and_two_view_roots() {
        let initial = build_initial_topology().unwrap();
        assert_eq!(
            initial.output.topology.roots(),
            &[initial.camera.0, initial.stage.0]
        );
        assert_eq!(
            initial
                .output
                .topology
                .node(initial.text.0)
                .unwrap()
                .identity()
                .inputs,
            vec![initial.world.0]
        );
        assert_eq!(
            initial
                .output
                .topology
                .node(initial.shapes.0)
                .unwrap()
                .identity()
                .inputs,
            vec![initial.world.0]
        );
        assert_eq!(
            initial
                .output
                .topology
                .node(initial.document_camera.0)
                .unwrap()
                .identity()
                .inputs,
            vec![initial.world.0]
        );
        for key in [initial.scene.0, initial.camera.0, initial.stage.0] {
            assert_eq!(
                initial
                    .output
                    .topology
                    .node(key)
                    .unwrap()
                    .identity()
                    .time_dependency,
                TimeDependency::Exact
            );
        }
    }
}
