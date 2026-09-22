use super::graph::{GpuGraphError, GpuResourceGraph};
use super::lowerer::{GpuContributionInput, GpuContributionResources, GpuLowerer};
use super::types::{
    GpuIdentitySource, GpuPassDesc, GpuPassIdentity, GpuPassKind, GpuResourceClass,
    GpuResourceDesc, GpuResourceIdentity, GpuResourceKey, GpuResourceLifetime,
};

#[derive(Debug)]
pub(crate) enum LogicalLowerError {
    Graph(GpuGraphError),
}

impl From<GpuGraphError> for LogicalLowerError {
    fn from(value: GpuGraphError) -> Self { Self::Graph(value) }
}

/// First production-independent lowerer: turns semantic contribution values
/// into independently versioned logical GPU resources and producer passes.
///
/// Concrete wgpu content creation is intentionally a later backend step.
#[derive(Default)]
pub(crate) struct LogicalGpuLowerer;

impl LogicalGpuLowerer {
    fn resource(
        graph: &mut GpuResourceGraph,
        semantic: super::lowerer::VersionedSemantic,
        class: GpuResourceClass,
        slot: u32,
        dependencies: Vec<GpuResourceKey>,
    ) -> Result<GpuResourceKey, GpuGraphError> {
        let desc = GpuResourceDesc {
            identity: GpuResourceIdentity {
                source: GpuIdentitySource::Semantic(semantic.node),
                class,
                slot,
            },
            version: semantic.version,
            dependencies,
            lifetime: GpuResourceLifetime::Persistent,
            alias_class: None,
            estimated_bytes: 0,
        };
        let key = desc.key();
        graph.upsert_resource(desc)?;
        Ok(key)
    }

    fn producer(
        graph: &mut GpuResourceGraph,
        tag: u64,
        kind: GpuPassKind,
        reads: Vec<GpuResourceKey>,
        writes: Vec<GpuResourceKey>,
    ) -> Result<(), GpuGraphError> {
        graph.insert_pass(GpuPassDesc {
            identity: GpuPassIdentity { kind, tag, reads, writes },
            after: Vec::new(),
            cacheable: true,
            side_effect: false,
        })
    }
}

impl GpuLowerer for LogicalGpuLowerer {
    type Error = LogicalLowerError;

    fn lower_contribution(
        &mut self,
        graph: &mut GpuResourceGraph,
        input: &GpuContributionInput,
    ) -> Result<GpuContributionResources, Self::Error> {
        let content = if let Some(content) = input.content {
            let key = Self::resource(graph, content, GpuResourceClass::Content, input.instance, vec![])?;
            Self::producer(
                graph,
                content.node.as_u64() ^ u64::from(input.instance),
                GpuPassKind::Upload,
                vec![],
                vec![key],
            )?;
            Some(key)
        } else {
            None
        };

        let placement = Self::resource(
            graph,
            input.placement,
            GpuResourceClass::Placement,
            input.instance,
            vec![],
        )?;
        Self::producer(
            graph,
            input.placement.node.as_u64() ^ (u64::from(input.instance) << 32),
            GpuPassKind::Upload,
            vec![],
            vec![placement],
        )?;

        let mut current = content;
        for (index, effect) in input.effects.iter().copied().enumerate() {
            let reads = current.into_iter().collect::<Vec<_>>();
            let output = Self::resource(
                graph,
                effect,
                GpuResourceClass::Effect,
                index as u32,
                reads.clone(),
            )?;
            Self::producer(
                graph,
                effect.node.as_u64() ^ index as u64,
                GpuPassKind::Render,
                reads,
                vec![output],
            )?;
            current = Some(output);
        }

        for (index, mask) in input.masks.iter().copied().enumerate() {
            let reads = current.into_iter().collect::<Vec<_>>();
            let output = Self::resource(
                graph,
                mask,
                GpuResourceClass::Mask,
                index as u32,
                reads.clone(),
            )?;
            Self::producer(
                graph,
                mask.node.as_u64() ^ 0x4d41534b ^ index as u64,
                GpuPassKind::Render,
                reads,
                vec![output],
            )?;
            current = Some(output);
        }

        if let Some(plate) = input.plate {
            let reads = current.into_iter().collect::<Vec<_>>();
            let output = Self::resource(
                graph,
                plate,
                GpuResourceClass::Plate,
                input.instance,
                reads.clone(),
            )?;
            Self::producer(
                graph,
                plate.node.as_u64() ^ 0x504c415445,
                GpuPassKind::Composite,
                reads,
                vec![output],
            )?;
            current = Some(output);
        }

        if let Some(matte) = input.matte_source {
            let mut reads = current.into_iter().collect::<Vec<_>>();
            reads.push(matte);
            let output = Self::resource(
                graph,
                input.contribution,
                GpuResourceClass::Matte,
                input.instance,
                reads.clone(),
            )?;
            Self::producer(
                graph,
                input.contribution.node.as_u64() ^ 0x4d41545445,
                GpuPassKind::Render,
                reads,
                vec![output],
            )?;
            current = Some(output);
        }

        Ok(GpuContributionResources {
            content,
            placement,
            final_image_or_geometry: current,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_graph::{NodeIdentity, NodeKey, NodeKind};
    use crate::gpu_exec::planner::GpuPlanner;
    use crate::gpu_exec::types::GpuResourceVersion;
    use crate::gpu_exec::VersionedSemantic;

    fn semantic(tag: u16, version: u64) -> VersionedSemantic {
        VersionedSemantic {
            node: NodeKey::for_identity(&NodeIdentity::new(NodeKind::Custom(tag), vec![])),
            version: GpuResourceVersion::new(version),
        }
    }

    #[test]
    fn placement_version_change_does_not_schedule_content_producer() {
        let mut graph = GpuResourceGraph::default();
        let mut lowerer = LogicalGpuLowerer;
        let first = GpuContributionInput {
            contribution: semantic(1, 1),
            instance: 0,
            content: Some(semantic(2, 10)),
            placement: semantic(3, 20),
            effects: vec![],
            masks: vec![],
            matte_source: None,
            plate: None,
        };
        let resources = lowerer.lower_contribution(&mut graph, &first).unwrap();
        let content_key = resources.content.unwrap();
        let placement_key = resources.placement;
        let content_pass = graph.producer(content_key).unwrap();
        let placement_pass = graph.producer(placement_key).unwrap();
        graph.mark_resident(content_key, GpuResourceVersion::new(10), 1);
        graph.mark_resident(placement_key, GpuResourceVersion::new(20), 1);

        let second = GpuContributionInput { placement: semantic(3, 21), ..first };
        lowerer.lower_contribution(&mut graph, &second).unwrap();

        let sink = graph.producer(placement_key).unwrap();
        let plan = GpuPlanner.plan(&graph, [sink]).unwrap();
        assert!(!plan.passes.contains(&content_pass));
        assert!(plan.passes.contains(&placement_pass));
    }
}
