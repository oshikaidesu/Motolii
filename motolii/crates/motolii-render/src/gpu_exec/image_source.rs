use crate::frame_graph::{CanonicalEncoder, NodeKey, SceneContentValue, SceneImageSourceValue};

use super::graph::{GpuGraphError, GpuResourceGraph};
use super::types::{
    GpuIdentitySource, GpuPassDesc, GpuPassIdentity, GpuPassKind, GpuResourceClass,
    GpuResourceDesc, GpuResourceIdentity, GpuResourceKey, GpuResourceLifetime,
    GpuResourceVersion,
};

#[derive(Clone, Debug)]
pub(crate) struct GpuImageSourceInput {
    pub effect: NodeKey,
    pub pass_slot: u32,
    pub image_slot: u32,
    pub source: SceneImageSourceValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuImageSourceResources {
    pub source: GpuResourceKey,
    pub snapshot: GpuResourceKey,
}

pub(crate) fn lower_image_source(
    graph: &mut GpuResourceGraph,
    input: &GpuImageSourceInput,
) -> Result<GpuImageSourceResources, GpuGraphError> {
    let slot = input.pass_slot.wrapping_mul(0x10000).wrapping_add(input.image_slot);
    let version = source_version(&input.source);
    let source_identity = GpuResourceIdentity {
        source: GpuIdentitySource::Semantic(input.effect),
        class: GpuResourceClass::ImageSource,
        slot,
    };
    let source_key = source_identity.key();
    graph.upsert_resource(GpuResourceDesc {
        identity: source_identity,
        version,
        dependencies: Vec::new(),
        lifetime: GpuResourceLifetime::Frame,
        alias_class: None,
        estimated_bytes: 0,
    })?;

    let snapshot_identity = GpuResourceIdentity {
        source: GpuIdentitySource::Semantic(input.effect),
        class: GpuResourceClass::Snapshot,
        slot,
    };
    let snapshot_key = snapshot_identity.key();
    graph.upsert_resource(GpuResourceDesc {
        identity: snapshot_identity,
        version,
        dependencies: vec![source_key],
        lifetime: GpuResourceLifetime::Temporal { retain_generations: 8 },
        alias_class: None,
        estimated_bytes: 0,
    })?;
    graph.insert_pass(GpuPassDesc {
        identity: GpuPassIdentity {
            kind: GpuPassKind::Copy,
            tag: input.effect.as_u64() ^ 0x534e4150 ^ u64::from(slot),
            reads: vec![source_key],
            writes: vec![snapshot_key],
        },
        after: Vec::new(),
        cacheable: true,
        side_effect: false,
    })?;

    Ok(GpuImageSourceResources { source: source_key, snapshot: snapshot_key })
}

fn source_version(source: &SceneImageSourceValue) -> GpuResourceVersion {
    let mut encoded = CanonicalEncoder::new();
    match source {
        SceneImageSourceValue::Content { layer, content, time, namespace } => {
            encoded.u8(1).u64(layer.0).rational_time(*time).u64(*namespace);
            encode_content(&mut encoded, content);
        }
        SceneImageSourceValue::Scene { scene, background, time, namespace } => {
            encoded.u8(2).rational_time(*time).u64(*namespace);
            for value in background { let _ = encoded.f32(*value); }
            encoded.u64(scene.layers.len() as u64);
            for layer in &scene.layers {
                encoded.u64(layer.layer.0).u32(layer.instance);
                encode_content(&mut encoded, &layer.content);
            }
        }
    }
    GpuResourceVersion::from_canonical(&encoded)
}

fn encode_content(encoded: &mut CanonicalEncoder, content: &SceneContentValue) {
    let bytes = format!("{content:?}");
    let _ = encoded.string(&bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::core::RationalTime;
    use crate::doc::store::LayerId;
    use crate::frame_graph::{NodeIdentity, NodeKind};

    #[test]
    fn same_temporal_request_reuses_snapshot_identity_and_version() {
        let effect = NodeKey::for_identity(&NodeIdentity::new(NodeKind::Custom(90), vec![]));
        let source = SceneImageSourceValue::Content {
            layer: LayerId(7),
            content: SceneContentValue::None,
            time: RationalTime::try_new(1, 2).unwrap(),
            namespace: 12,
        };
        let input = GpuImageSourceInput { effect, pass_slot: 1, image_slot: 2, source };
        let mut graph = GpuResourceGraph::default();
        let first = lower_image_source(&mut graph, &input).unwrap();
        let version = graph.version(first.snapshot).unwrap();
        graph.mark_resident(first.snapshot, version, 1);
        let second = lower_image_source(&mut graph, &input).unwrap();
        assert_eq!(first, second);
        assert!(graph.is_current(second.snapshot));
    }
}
