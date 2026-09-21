use std::collections::BTreeMap;

use crate::doc::store::LayerId;

use super::{GraphNode, GraphTopology, NodeIdentity, NodeKey, NodeKind, TopologyError};

/// Compiler-local lookup for edits and diagnostics. Layer identity is kept
/// here, outside content-addressed node identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerBinding {
    pub content: NodeKey,
    pub layout: Option<NodeKey>,
    pub transform: NodeKey,
    pub contribution: NodeKey,
}

pub struct CompilerOutput {
    pub topology: GraphTopology,
    pub layers: BTreeMap<LayerId, LayerBinding>,
    pub scene: NodeKey,
    pub camera: NodeKey,
    pub stage: NodeKey,
}

/// Builds one revision-scoped graph. It does not read values at a concrete
/// frame time and does not execute renderer work.
#[derive(Default)]
pub struct GraphBuilder {
    nodes: BTreeMap<NodeKey, GraphNode>,
    layers: BTreeMap<LayerId, LayerBinding>,
    contributions: Vec<NodeKey>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern a complete recipe. Equal work becomes one node; a compact-key
    /// collision between different recipes is rejected.
    pub fn intern(&mut self, identity: NodeIdentity) -> Result<NodeKey, TopologyError> {
        let node = GraphNode::new(identity);
        let key = node.key();
        match self.nodes.get(&key) {
            Some(existing) if existing.identity() == node.identity() => Ok(key),
            Some(_) => Err(TopologyError::HashCollision(key)),
            None => {
                self.nodes.insert(key, node);
                Ok(key)
            }
        }
    }

    pub fn bind_layer(&mut self, layer: LayerId, binding: LayerBinding) {
        self.layers.insert(layer, binding);
    }

    pub fn layer(&self, layer: LayerId) -> Option<LayerBinding> {
        self.layers.get(&layer).copied()
    }

    /// Contributions remain in authored compositing order.
    pub fn set_contributions(&mut self, contributions: impl IntoIterator<Item = NodeKey>) {
        self.contributions = contributions.into_iter().collect();
    }

    /// Finish with one shared scene and two view-only roots. Camera state is an
    /// optional extra input to the Camera projection; Stage reads the scene.
    pub fn finish(
        mut self,
        camera_state: Option<NodeKey>,
    ) -> Result<CompilerOutput, TopologyError> {
        let scene = self.intern(NodeIdentity::new(
            NodeKind::SceneComposite,
            self.contributions.clone(),
        ))?;

        let mut camera_inputs = vec![scene];
        if let Some(camera) = camera_state {
            camera_inputs.push(camera);
        }
        let mut camera_identity = NodeIdentity::new(NodeKind::CameraProjection, camera_inputs);
        camera_identity.parameters.push(0);
        let camera = self.intern(camera_identity)?;

        let mut stage_identity = NodeIdentity::new(NodeKind::CameraProjection, vec![scene]);
        stage_identity.parameters.push(1);
        let stage = self.intern(stage_identity)?;

        let topology = GraphTopology::try_new(self.nodes.into_values(), vec![camera, stage])?;
        Ok(CompilerOutput {
            topology,
            layers: self.layers,
            scene,
            camera,
            stage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(kind: NodeKind, marker: u8) -> NodeIdentity {
        let mut identity = NodeIdentity::new(kind, Vec::new());
        identity.parameters.push(marker);
        identity
    }

    #[test]
    fn equal_content_is_interned_but_layer_bindings_remain_distinct() {
        let mut builder = GraphBuilder::new();
        let shared = builder.intern(node(NodeKind::TextShape, 1)).unwrap();
        let transform_a = builder.intern(node(NodeKind::Transform, 2)).unwrap();
        let transform_b = builder.intern(node(NodeKind::Transform, 3)).unwrap();
        let contribution_a = builder
            .intern(NodeIdentity::new(
                NodeKind::CompositeContribution,
                vec![shared, transform_a],
            ))
            .unwrap();
        let contribution_b = builder
            .intern(NodeIdentity::new(
                NodeKind::CompositeContribution,
                vec![shared, transform_b],
            ))
            .unwrap();
        builder.bind_layer(
            LayerId(1),
            LayerBinding {
                content: shared,
                layout: None,
                transform: transform_a,
                contribution: contribution_a,
            },
        );
        builder.bind_layer(
            LayerId(2),
            LayerBinding {
                content: shared,
                layout: None,
                transform: transform_b,
                contribution: contribution_b,
            },
        );
        builder.set_contributions([contribution_a, contribution_b]);

        let output = builder.finish(None).unwrap();
        assert_eq!(
            output.layers[&LayerId(1)].content,
            output.layers[&LayerId(2)].content
        );
        assert_ne!(
            output.layers[&LayerId(1)].transform,
            output.layers[&LayerId(2)].transform
        );
        assert_eq!(output.topology.roots(), &[output.camera, output.stage]);
    }
}
