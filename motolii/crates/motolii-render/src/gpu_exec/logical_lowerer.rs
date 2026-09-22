use std::collections::{BTreeMap, BTreeSet};

use super::graph::{GpuGraphError, GpuResourceGraph};
use super::lowerer::{GpuContributionInput, GpuContributionResources, GpuLowerer};
use super::types::{
    GpuIdentitySource, GpuPassDesc, GpuPassIdentity, GpuPassKind, GpuResourceClass,
    GpuResourceDesc, GpuResourceIdentity, GpuResourceKey, GpuResourceLifetime,
    GpuResourceVersion,
};

#[derive(Debug)]
pub(crate) enum LogicalLowerError {
    Graph(GpuGraphError),
    RelationArity { inputs: usize, resources: usize },
}

impl From<GpuGraphError> for LogicalLowerError {
    fn from(value: GpuGraphError) -> Self { Self::Graph(value) }
}

pub(crate) struct GpuSceneRelations {
    pub ordered_outputs: Vec<GpuResourceKey>,
    pub operations: Vec<(super::types::GpuPassKey, super::engine_backend::EngineGpuOperation)>,
}

/// Production-independent lowerer. A contribution is lowered to a prepared
/// resource first; cross-contribution relations are lowered only after every
/// contribution is known.
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

    fn relation_resource(
        graph: &mut GpuResourceGraph,
        identity: GpuResourceIdentity,
        version: GpuResourceVersion,
        dependencies: Vec<GpuResourceKey>,
    ) -> Result<GpuResourceKey, GpuGraphError> {
        let desc = GpuResourceDesc {
            identity,
            version,
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
    ) -> Result<super::types::GpuPassKey, GpuGraphError> {
        let pass = GpuPassDesc {
            identity: GpuPassIdentity { kind, tag, reads, writes },
            after: Vec::new(),
            cacheable: true,
            side_effect: false,
        };
        let key = pass.key();
        graph.insert_pass(pass)?;
        Ok(key)
    }

    fn relation_version(
        graph: &GpuResourceGraph,
        tag: u64,
        reads: &[GpuResourceKey],
        extra: Option<&str>,
    ) -> GpuResourceVersion {
        let mut encoded = crate::frame_graph::CanonicalEncoder::new();
        encoded.u64(tag);
        for key in reads {
            encoded.u64(key.0);
            encoded.u64(
                graph
                    .version(*key)
                    .unwrap_or(GpuResourceVersion::STATIC)
                    .as_u64(),
            );
        }
        if let Some(extra) = extra {
            let _ = encoded.string(extra);
        }
        GpuResourceVersion::from_canonical(&encoded)
    }

    /// Lower clip/matte consumption after all contributions have a prepared
    /// output. The stable contribution key is only a relation handle; it never
    /// becomes a hidden whole-scene execution resource.
    pub(crate) fn lower_relations(
        &mut self,
        graph: &mut GpuResourceGraph,
        inputs: &[GpuContributionInput],
        resources: &[GpuContributionResources],
    ) -> Result<GpuSceneRelations, LogicalLowerError> {
        if inputs.len() != resources.len() {
            return Err(LogicalLowerError::RelationArity {
                inputs: inputs.len(),
                resources: resources.len(),
            });
        }

        let by_contribution: BTreeMap<_, _> = resources
            .iter()
            .enumerate()
            .map(|(index, value)| (value.contribution, index))
            .collect();
        let mut current: Vec<_> = resources.iter().map(|value| value.prepared).collect();
        let mut removed = vec![false; resources.len()];
        let mut operations = Vec::new();

        // Clipping mutates the visible base in source order. A clipped upper is
        // consumed and never reaches the scene root as a separate output.
        for (index, input) in inputs.iter().enumerate() {
            if !input.clip_to_below || input.stencil {
                continue;
            }
            let Some(base_handle) = input.clip_base else {
                removed[index] = true;
                continue;
            };
            let Some(base_index) = by_contribution.get(&base_handle).copied() else {
                removed[index] = true;
                continue;
            };
            let (Some(source), Some(base)) = (current[index], current[base_index]) else {
                removed[index] = true;
                continue;
            };
            let reads = vec![source, base];
            let version = Self::relation_version(
                graph,
                input.contribution.node.as_u64() ^ 0x434c4950 ^ u64::from(input.instance),
                &reads,
                None,
            );
            let identity = GpuResourceIdentity {
                source: GpuIdentitySource::Semantic(input.contribution.node),
                class: GpuResourceClass::Composite,
                slot: input.instance | 0x4000_0000,
            };
            let output = Self::relation_resource(graph, identity, version, reads.clone())?;
            let pass = Self::producer(
                graph,
                input.contribution.node.as_u64() ^ 0x434c4950 ^ u64::from(input.instance),
                GpuPassKind::Composite,
                reads,
                vec![output],
            )?;
            operations.push((
                pass,
                super::engine_backend::EngineGpuOperation::Clip {
                    source,
                    base,
                    output,
                    version,
                },
            ));
            current[base_index] = Some(output);
            removed[index] = true;
        }

        // Track matte sources are consumed after all clip relations, matching
        // the proven previous owner.
        let matte_sources: BTreeSet<_> = inputs
            .iter()
            .filter(|input| !input.clip_to_below)
            .filter_map(|input| input.matte_source)
            .collect();

        for (index, input) in inputs.iter().enumerate() {
            if removed[index] || input.clip_to_below {
                continue;
            }
            let Some(source_handle) = input.matte_source else { continue };
            let Some(source_index) = by_contribution.get(&source_handle).copied() else {
                removed[index] = true;
                continue;
            };
            let (Some(target), Some(source)) = (current[index], current[source_index]) else {
                removed[index] = true;
                continue;
            };
            let Some(mode) = input.matte_mode else {
                removed[index] = true;
                continue;
            };
            let reads = vec![target, source];
            let mode_text = format!("{mode:?}");
            let version = Self::relation_version(
                graph,
                input.contribution.node.as_u64() ^ 0x4d41545445,
                &reads,
                Some(&mode_text),
            );
            let identity = GpuResourceIdentity {
                source: GpuIdentitySource::Semantic(input.contribution.node),
                class: GpuResourceClass::Matte,
                slot: input.instance,
            };
            let output = Self::relation_resource(graph, identity, version, reads.clone())?;
            let pass = Self::producer(
                graph,
                input.contribution.node.as_u64() ^ 0x4d41545445,
                GpuPassKind::Render,
                reads,
                vec![output],
            )?;
            operations.push((
                pass,
                super::engine_backend::EngineGpuOperation::Matte {
                    target,
                    source,
                    output,
                    version,
                    mode,
                },
            ));
            current[index] = Some(output);
        }

        let ordered_outputs = resources
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                (!removed[index]
                    && !inputs[index].stencil
                    && !matte_sources.contains(&value.contribution))
                    .then_some(current[index])
                    .flatten()
            })
            .collect();

        Ok(GpuSceneRelations { ordered_outputs, operations })
    }
}

impl GpuLowerer for LogicalGpuLowerer {
    type Error = LogicalLowerError;

    fn declare_contribution(
        &mut self,
        _graph: &mut GpuResourceGraph,
        input: &GpuContributionInput,
    ) -> Result<GpuResourceKey, Self::Error> {
        // Stable relation handle only. It is deliberately not inserted into the
        // GPU resource graph: the scene root depends on the real final output.
        Ok(GpuResourceIdentity {
            source: GpuIdentitySource::Semantic(input.contribution.node),
            class: GpuResourceClass::Composite,
            slot: input.instance,
        }
        .key())
    }

    fn lower_contribution(
        &mut self,
        graph: &mut GpuResourceGraph,
        input: &GpuContributionInput,
    ) -> Result<GpuContributionResources, Self::Error> {
        let contribution = GpuResourceIdentity {
            source: GpuIdentitySource::Semantic(input.contribution.node),
            class: GpuResourceClass::Composite,
            slot: input.instance,
        }
        .key();

        let content = if let Some(content) = input.content {
            let key =
                Self::resource(graph, content, GpuResourceClass::Content, input.instance, vec![])?;
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

        let operations = Vec::new();
        let mut snapshot_rows = Vec::with_capacity(input.direct_effects.len());
        let mut current = content;

        for (index, effect) in input.direct_effects.iter().copied().enumerate() {
            let mut reads = current.into_iter().collect::<Vec<_>>();
            let mut snapshots = Vec::new();
            if let Some(row) = input.image_sources.get(index) {
                for (image_slot, source) in row.iter().cloned().enumerate() {
                    let resources = super::image_source::lower_image_source(
                        graph,
                        &super::image_source::GpuImageSourceInput {
                            effect: effect.node,
                            pass_slot: index as u32,
                            image_slot: image_slot as u32,
                            source,
                        },
                    )?;
                    reads.push(resources.snapshot);
                    snapshots.push(resources.snapshot);
                }
            }
            snapshot_rows.push(snapshots);
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

        // Placement is a separate resource; this node is only the placed
        // contribution before after-effects.
        if let Some(source) = current {
            let output = Self::resource(
                graph,
                input.contribution,
                GpuResourceClass::Composite,
                input.instance | 0x1000_0000,
                vec![source, placement],
            )?;
            Self::producer(
                graph,
                input.contribution.node.as_u64() ^ 0x504c41434544 ^ u64::from(input.instance),
                GpuPassKind::Composite,
                vec![source, placement],
                vec![output],
            )?;
            current = Some(output);
        }

        for (index, effect) in input.after_effects.iter().copied().enumerate() {
            let reads = current.into_iter().collect::<Vec<_>>();
            let output = Self::resource(
                graph,
                effect,
                GpuResourceClass::Effect,
                0x8000_0000u32 | index as u32,
                reads.clone(),
            )?;
            Self::producer(
                graph,
                effect.node.as_u64() ^ 0x4146544552 ^ index as u64,
                GpuPassKind::Render,
                reads,
                vec![output],
            )?;
            current = Some(output);
        }

        // Uniform prepared handle used by relation lowering and concrete
        // materialization regardless of whether after-effects exist.
        let prepared = if let Some(source) = current {
            let output = Self::resource(
                graph,
                input.contribution,
                GpuResourceClass::Composite,
                input.instance | 0x2000_0000,
                vec![source],
            )?;
            Self::producer(
                graph,
                input.contribution.node.as_u64() ^ 0x50524550415245 ^ u64::from(input.instance),
                GpuPassKind::Copy,
                vec![source],
                vec![output],
            )?;
            Some(output)
        } else {
            None
        };

        Ok(GpuContributionResources {
            content,
            placement,
            contribution,
            prepared,
            snapshot_rows,
            operations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_graph::{NodeIdentity, NodeKey, NodeKind};
    use crate::gpu_exec::planner::GpuPlanner;
    use crate::gpu_exec::{EngineGpuOperation, VersionedSemantic};

    fn semantic(tag: u16, version: u64) -> VersionedSemantic {
        VersionedSemantic {
            node: NodeKey::for_identity(&NodeIdentity::new(NodeKind::Custom(tag), vec![])),
            version: GpuResourceVersion::new(version),
        }
    }

    fn input(tag: u16, content: u16) -> GpuContributionInput {
        GpuContributionInput {
            contribution: semantic(tag, 1),
            instance: 0,
            content: Some(semantic(content, 10)),
            placement: semantic(tag + 100, 20),
            direct_effects: vec![],
            after_effects: vec![],
            image_sources: vec![],
            masks: vec![],
            matte_source: None,
            clip_base: None,
            clip_to_below: false,
            matte_mode: None,
            stencil: false,
            plate: None,
        }
    }

    #[test]
    fn placement_version_change_does_not_schedule_content_producer() {
        let mut graph = GpuResourceGraph::default();
        let mut lowerer = LogicalGpuLowerer;
        let first = input(1, 2);
        let resources = lowerer.lower_contribution(&mut graph, &first).unwrap();
        let content_key = resources.content.unwrap();
        let placement_key = resources.placement;
        let content_pass = graph.producer(content_key).unwrap();
        let placement_pass = graph.producer(placement_key).unwrap();
        graph.mark_resident(content_key, GpuResourceVersion::new(10), 1);
        graph.mark_resident(placement_key, GpuResourceVersion::new(20), 1);

        let second = GpuContributionInput { placement: semantic(101, 21), ..first };
        lowerer.lower_contribution(&mut graph, &second).unwrap();

        let sink = graph.producer(placement_key).unwrap();
        let plan = GpuPlanner.plan(&graph, [sink]).unwrap();
        assert!(!plan.passes.contains(&content_pass));
        assert!(plan.passes.contains(&placement_pass));
    }

    #[test]
    fn clip_replaces_base_output_and_consumes_upper() {
        let mut graph = GpuResourceGraph::default();
        let mut lowerer = LogicalGpuLowerer;
        let base = input(1, 11);
        let base_handle = lowerer.declare_contribution(&mut graph, &base).unwrap();
        let upper = GpuContributionInput {
            clip_base: Some(base_handle),
            clip_to_below: true,
            ..input(2, 12)
        };
        let inputs = vec![base, upper];
        let resources: Vec<_> = inputs
            .iter()
            .map(|input| lowerer.lower_contribution(&mut graph, input).unwrap())
            .collect();
        let relations = lowerer.lower_relations(&mut graph, &inputs, &resources).unwrap();
        assert_eq!(relations.ordered_outputs.len(), 1);
        assert_ne!(relations.ordered_outputs[0], resources[0].prepared.unwrap());
        assert!(matches!(
            relations.operations.as_slice(),
            [(_, EngineGpuOperation::Clip { .. })]
        ));
    }
}
