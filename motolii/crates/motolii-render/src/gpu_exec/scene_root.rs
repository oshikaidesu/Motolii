use crate::frame_graph::{CanonicalEncoder, NodeKey};

use super::graph::{GpuGraphError, GpuResourceGraph};
use super::types::{
    GpuIdentitySource, GpuPassDesc, GpuPassIdentity, GpuPassKey, GpuPassKind,
    GpuResourceClass, GpuResourceDesc, GpuResourceIdentity, GpuResourceKey,
    GpuResourceLifetime, GpuResourceVersion,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuSinkKind {
    Present,
    Readback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuSceneRoot {
    pub resource: GpuResourceKey,
    pub pass: GpuPassKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuSinkRoot {
    pub resource: GpuResourceKey,
    pub pass: GpuPassKey,
}

/// Build the final ordered scene dependency root.
///
/// The SceneComposite resource is deliberately a deferred scene result, not a
/// forced full-frame texture. Mixed 2D/3D composition still depends on the
/// sink's camera/window. Its producer therefore establishes ordering and
/// reachability; the concrete sink executor consumes that ordered scene plan.
pub(crate) fn lower_scene_root(
    graph: &mut GpuResourceGraph,
    scene_node: NodeKey,
    ordered: &[GpuResourceKey],
) -> Result<GpuSceneRoot, GpuGraphError> {
    let mut encoded = CanonicalEncoder::new();
    encoded.node_key(scene_node);
    for key in ordered {
        encoded.u64(key.0);
        if let Some(version) = graph.version(*key) {
            encoded.u64(version.as_u64());
        }
    }
    let version = GpuResourceVersion::from_canonical(&encoded);
    let identity = GpuResourceIdentity {
        source: GpuIdentitySource::Semantic(scene_node),
        class: GpuResourceClass::Composite,
        slot: u32::MAX,
    };
    let resource = identity.key();
    graph.upsert_resource(GpuResourceDesc {
        identity,
        version,
        dependencies: ordered.to_vec(),
        lifetime: GpuResourceLifetime::Frame,
        alias_class: None,
        estimated_bytes: 0,
    })?;
    let desc = GpuPassDesc {
        identity: GpuPassIdentity {
            kind: GpuPassKind::Composite,
            tag: scene_node.as_u64() ^ 0x5343454e45,
            reads: ordered.to_vec(),
            writes: vec![resource],
        },
        after: Vec::new(),
        cacheable: false,
        side_effect: false,
    };
    let pass = desc.key();
    graph.insert_pass(desc)?;
    Ok(GpuSceneRoot { resource, pass })
}

pub(crate) fn lower_sink(
    graph: &mut GpuResourceGraph,
    scene_node: NodeKey,
    scene: GpuSceneRoot,
    kind: GpuSinkKind,
) -> Result<GpuSinkRoot, GpuGraphError> {
    let (slot, pass_kind, tag) = match kind {
        GpuSinkKind::Present => (0, GpuPassKind::Present, 0x50524553454e54),
        GpuSinkKind::Readback => (1, GpuPassKind::Readback, 0x52454144424143),
    };
    let identity = GpuResourceIdentity {
        source: GpuIdentitySource::Semantic(scene_node),
        class: GpuResourceClass::Sink,
        slot,
    };
    let resource = identity.key();
    graph.upsert_resource(GpuResourceDesc {
        identity,
        version: graph.version(scene.resource).unwrap_or(GpuResourceVersion::STATIC),
        dependencies: vec![scene.resource],
        lifetime: GpuResourceLifetime::Frame,
        alias_class: None,
        estimated_bytes: 0,
    })?;
    let desc = GpuPassDesc {
        identity: GpuPassIdentity {
            kind: pass_kind,
            tag: scene_node.as_u64() ^ tag,
            reads: vec![scene.resource],
            writes: vec![resource],
        },
        after: vec![scene.pass],
        cacheable: false,
        side_effect: true,
    };
    let pass = desc.key();
    graph.insert_pass(desc)?;
    Ok(GpuSinkRoot { resource, pass })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_graph::{NodeIdentity, NodeKind};
    use crate::gpu_exec::types::{GpuResourceDesc, GpuResourceIdentity};

    #[test]
    fn present_and_readback_share_one_upstream_scene_root() {
        let mut graph = GpuResourceGraph::default();
        let contribution_node = NodeKey::for_identity(&NodeIdentity::new(NodeKind::Custom(201), vec![]));
        let contribution_identity = GpuResourceIdentity::semantic(contribution_node, GpuResourceClass::Composite, 0);
        let contribution = contribution_identity.key();
        graph.upsert_resource(GpuResourceDesc {
            identity: contribution_identity,
            version: GpuResourceVersion::new(7),
            dependencies: vec![],
            lifetime: GpuResourceLifetime::Persistent,
            alias_class: None,
            estimated_bytes: 0,
        }).unwrap();
        let scene_node = NodeKey::for_identity(&NodeIdentity::new(NodeKind::SceneComposite, vec![]));
        let scene = lower_scene_root(&mut graph, scene_node, &[contribution]).unwrap();
        let present = lower_sink(&mut graph, scene_node, scene, GpuSinkKind::Present).unwrap();
        let readback = lower_sink(&mut graph, scene_node, scene, GpuSinkKind::Readback).unwrap();
        assert_eq!(graph.pass(present.pass).unwrap().identity.reads, vec![scene.resource]);
        assert_eq!(graph.pass(readback.pass).unwrap().identity.reads, vec![scene.resource]);
    }
}
