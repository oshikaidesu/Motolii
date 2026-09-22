use std::collections::{BTreeMap, BTreeSet};

use super::graph::GpuResourceGraph;
use super::types::{
    GpuAliasClass, GpuPassKey, GpuPhysicalSlot, GpuResourceKey, GpuResourceLifetime,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GpuPlanError {
    MissingPass(GpuPassKey),
    MissingResource(GpuResourceKey),
    MissingProducer(GpuResourceKey),
    Cycle(GpuPassKey),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuResourceInterval {
    pub first_pass: usize,
    pub last_pass: usize,
}

/// Immutable work list for one sink/generation. The executor records this
/// order; it must not rediscover semantic dependencies.
#[derive(Clone, Debug, Default)]
pub(crate) struct GpuExecutionPlan {
    pub passes: Vec<GpuPassKey>,
    pub skipped_cached: Vec<GpuPassKey>,
    pub liveness: BTreeMap<GpuResourceKey, GpuResourceInterval>,
    pub aliases: BTreeMap<GpuResourceKey, GpuPhysicalSlot>,
}

#[derive(Default)]
pub(crate) struct GpuPlanner;

impl GpuPlanner {
    pub fn plan(
        &self,
        graph: &GpuResourceGraph,
        roots: impl IntoIterator<Item = GpuPassKey>,
    ) -> Result<GpuExecutionPlan, GpuPlanError> {
        let mut ordered = Vec::new();
        let mut skipped = Vec::new();
        let mut visiting = BTreeSet::new();
        let mut complete = BTreeSet::new();

        fn visit(
            graph: &GpuResourceGraph,
            pass_key: GpuPassKey,
            visiting: &mut BTreeSet<GpuPassKey>,
            complete: &mut BTreeSet<GpuPassKey>,
            ordered: &mut Vec<GpuPassKey>,
            skipped: &mut Vec<GpuPassKey>,
        ) -> Result<(), GpuPlanError> {
            if complete.contains(&pass_key) {
                return Ok(());
            }
            if !visiting.insert(pass_key) {
                return Err(GpuPlanError::Cycle(pass_key));
            }

            let pass = graph.pass(pass_key).ok_or(GpuPlanError::MissingPass(pass_key))?;

            // A resident cacheable producer is a cut in the execution graph:
            // none of its producer chain is needed for this generation.
            let cache_hit = pass.cacheable
                && !pass.side_effect
                && !pass.identity.writes.is_empty()
                && pass.identity.writes.iter().all(|key| graph.is_current(*key));
            if cache_hit {
                skipped.push(pass_key);
                visiting.remove(&pass_key);
                complete.insert(pass_key);
                return Ok(());
            }

            for dependency in &pass.after {
                visit(graph, *dependency, visiting, complete, ordered, skipped)?;
            }

            for input in &pass.identity.reads {
                let resource = graph.resource(*input).ok_or(GpuPlanError::MissingResource(*input))?;
                if graph.is_current(*input) {
                    continue;
                }
                if matches!(resource.lifetime, GpuResourceLifetime::Frame)
                    || !graph.is_current(*input)
                {
                    let producer = graph.producer(*input)
                        .ok_or(GpuPlanError::MissingProducer(*input))?;
                    visit(graph, producer, visiting, complete, ordered, skipped)?;
                }
            }

            visiting.remove(&pass_key);
            complete.insert(pass_key);
            ordered.push(pass_key);
            Ok(())
        }

        for root in roots {
            visit(
                graph,
                root,
                &mut visiting,
                &mut complete,
                &mut ordered,
                &mut skipped,
            )?;
        }

        let liveness = resource_liveness(graph, &ordered)?;
        let aliases = assign_aliases(graph, &liveness);
        Ok(GpuExecutionPlan {
            passes: ordered,
            skipped_cached: skipped,
            liveness,
            aliases,
        })
    }
}

fn resource_liveness(
    graph: &GpuResourceGraph,
    passes: &[GpuPassKey],
) -> Result<BTreeMap<GpuResourceKey, GpuResourceInterval>, GpuPlanError> {
    let mut liveness = BTreeMap::new();
    for (index, pass_key) in passes.iter().copied().enumerate() {
        let pass = graph.pass(pass_key).ok_or(GpuPlanError::MissingPass(pass_key))?;
        for resource in pass.identity.reads.iter().chain(&pass.identity.writes) {
            let entry = liveness.entry(*resource).or_insert(GpuResourceInterval {
                first_pass: index,
                last_pass: index,
            });
            entry.first_pass = entry.first_pass.min(index);
            entry.last_pass = entry.last_pass.max(index);
        }
    }
    Ok(liveness)
}

fn assign_aliases(
    graph: &GpuResourceGraph,
    liveness: &BTreeMap<GpuResourceKey, GpuResourceInterval>,
) -> BTreeMap<GpuResourceKey, GpuPhysicalSlot> {
    #[derive(Clone, Copy)]
    struct Slot {
        physical: GpuPhysicalSlot,
        class: GpuAliasClass,
        last_pass: usize,
    }

    let mut candidates: Vec<_> = liveness.iter().filter_map(|(key, interval)| {
        let resource = graph.resource(*key)?;
        (matches!(resource.lifetime, GpuResourceLifetime::Frame))
            .then_some(resource.alias_class)
            .flatten()
            .map(|class| (*key, *interval, class))
    }).collect();
    candidates.sort_by_key(|(_, interval, _)| interval.first_pass);

    let mut slots: Vec<Slot> = Vec::new();
    let mut out = BTreeMap::new();
    for (key, interval, class) in candidates {
        if let Some(slot) = slots
            .iter_mut()
            .find(|slot| slot.class == class && slot.last_pass < interval.first_pass)
        {
            slot.last_pass = interval.last_pass;
            out.insert(key, slot.physical);
            continue;
        }
        let physical = GpuPhysicalSlot(slots.len() as u32);
        slots.push(Slot { physical, class, last_pass: interval.last_pass });
        out.insert(key, physical);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_graph::{NodeIdentity, NodeKey, NodeKind};
    use crate::gpu_exec::graph::GpuResourceGraph;
    use crate::gpu_exec::types::{
        GpuAliasClass, GpuIdentitySource, GpuPassDesc, GpuPassIdentity, GpuPassKind,
        GpuResourceClass, GpuResourceDesc, GpuResourceIdentity, GpuResourceVersion,
    };

    fn node(tag: u16) -> NodeKey {
        NodeKey::for_identity(&NodeIdentity::new(NodeKind::Custom(tag), vec![]))
    }

    fn resource(
        tag: u16,
        class: GpuResourceClass,
        version: u64,
        lifetime: GpuResourceLifetime,
        alias_class: Option<GpuAliasClass>,
    ) -> GpuResourceDesc {
        GpuResourceDesc {
            identity: GpuResourceIdentity {
                source: GpuIdentitySource::Semantic(node(tag)),
                class,
                slot: 0,
            },
            version: GpuResourceVersion::new(version),
            dependencies: Vec::new(),
            lifetime,
            alias_class,
            estimated_bytes: 0,
        }
    }

    fn pass(
        kind: GpuPassKind,
        tag: u64,
        reads: Vec<GpuResourceKey>,
        writes: Vec<GpuResourceKey>,
        after: Vec<GpuPassKey>,
        cacheable: bool,
        side_effect: bool,
    ) -> GpuPassDesc {
        GpuPassDesc {
            identity: GpuPassIdentity { kind, tag, reads, writes },
            after,
            cacheable,
            side_effect,
        }
    }

    #[test]
    fn transform_change_does_not_rebuild_resident_content() {
        let mut graph = GpuResourceGraph::default();
        let content = resource(1, GpuResourceClass::Content, 1, GpuResourceLifetime::Persistent, None);
        let content_key = content.key();
        graph.upsert_resource(content).unwrap();

        let placement_v1 = resource(2, GpuResourceClass::Placement, 1, GpuResourceLifetime::Persistent, None);
        let placement_key = placement_v1.key();
        graph.upsert_resource(placement_v1).unwrap();

        let output = resource(3, GpuResourceClass::Composite, 1, GpuResourceLifetime::Frame, None);
        let output_key = output.key();
        graph.upsert_resource(output).unwrap();

        let upload = pass(GpuPassKind::Upload, 1, vec![], vec![content_key], vec![], true, false);
        let upload_key = upload.key();
        graph.insert_pass(upload).unwrap();

        let place = pass(GpuPassKind::Compute, 2, vec![], vec![placement_key], vec![], true, false);
        let place_key = place.key();
        graph.insert_pass(place).unwrap();

        let composite = pass(
            GpuPassKind::Composite,
            3,
            vec![content_key, placement_key],
            vec![output_key],
            vec![],
            false,
            false,
        );
        let composite_key = composite.key();
        graph.insert_pass(composite).unwrap();

        graph.mark_resident(content_key, GpuResourceVersion::new(1), 1);
        graph.mark_resident(placement_key, GpuResourceVersion::new(1), 1);

        let placement_v2 = resource(2, GpuResourceClass::Placement, 2, GpuResourceLifetime::Persistent, None);
        graph.upsert_resource(placement_v2).unwrap();

        let plan = GpuPlanner.plan(&graph, [composite_key]).unwrap();
        assert!(!plan.passes.contains(&upload_key));
        assert!(plan.passes.contains(&place_key));
        assert!(plan.passes.contains(&composite_key));
    }

    #[test]
    fn advancing_video_does_not_rebuild_static_overlay() {
        let mut graph = GpuResourceGraph::default();
        let video = resource(10, GpuResourceClass::Content, 1, GpuResourceLifetime::Persistent, None);
        let video_key = video.key();
        graph.upsert_resource(video).unwrap();
        let overlay = resource(11, GpuResourceClass::Content, 1, GpuResourceLifetime::Persistent, None);
        let overlay_key = overlay.key();
        graph.upsert_resource(overlay).unwrap();
        let out = resource(12, GpuResourceClass::Composite, 1, GpuResourceLifetime::Frame, None);
        let out_key = out.key();
        graph.upsert_resource(out).unwrap();

        let video_upload = pass(GpuPassKind::Upload, 10, vec![], vec![video_key], vec![], true, false);
        let video_upload_key = video_upload.key();
        graph.insert_pass(video_upload).unwrap();
        let overlay_upload = pass(GpuPassKind::Upload, 11, vec![], vec![overlay_key], vec![], true, false);
        let overlay_upload_key = overlay_upload.key();
        graph.insert_pass(overlay_upload).unwrap();
        let composite = pass(
            GpuPassKind::Composite,
            12,
            vec![video_key, overlay_key],
            vec![out_key],
            vec![],
            false,
            false,
        );
        let composite_key = composite.key();
        graph.insert_pass(composite).unwrap();

        graph.mark_resident(video_key, GpuResourceVersion::new(1), 1);
        graph.mark_resident(overlay_key, GpuResourceVersion::new(1), 1);
        graph.upsert_resource(resource(
            10,
            GpuResourceClass::Content,
            2,
            GpuResourceLifetime::Persistent,
            None,
        )).unwrap();

        let plan = GpuPlanner.plan(&graph, [composite_key]).unwrap();
        assert!(plan.passes.contains(&video_upload_key));
        assert!(!plan.passes.contains(&overlay_upload_key));
    }

    #[test]
    fn non_overlapping_frame_resources_alias_the_same_slot() {
        let mut graph = GpuResourceGraph::default();
        let class = GpuAliasClass { tag: 7, bytes: 4096, alignment: 256 };
        let a = resource(20, GpuResourceClass::Scratch, 1, GpuResourceLifetime::Frame, Some(class));
        let a_key = a.key();
        graph.upsert_resource(a).unwrap();
        let b = resource(21, GpuResourceClass::Scratch, 1, GpuResourceLifetime::Frame, Some(class));
        let b_key = b.key();
        graph.upsert_resource(b).unwrap();
        let terminal = resource(22, GpuResourceClass::Sink, 1, GpuResourceLifetime::Frame, None);
        let terminal_key = terminal.key();
        graph.upsert_resource(terminal).unwrap();

        let make_a = pass(GpuPassKind::Render, 20, vec![], vec![a_key], vec![], false, false);
        let make_a_key = make_a.key();
        graph.insert_pass(make_a).unwrap();

        let consume_a = pass(GpuPassKind::Compute, 21, vec![a_key], vec![], vec![], false, true);
        let consume_a_key = consume_a.key();
        graph.insert_pass(consume_a).unwrap();

        let make_b = pass(GpuPassKind::Render, 22, vec![], vec![b_key], vec![consume_a_key], false, false);
        let make_b_key = make_b.key();
        graph.insert_pass(make_b).unwrap();

        let consume_b = pass(
            GpuPassKind::Composite,
            23,
            vec![b_key],
            vec![terminal_key],
            vec![make_b_key],
            false,
            false,
        );
        let consume_b_key = consume_b.key();
        graph.insert_pass(consume_b).unwrap();

        let plan = GpuPlanner.plan(&graph, [consume_b_key]).unwrap();
        assert!(plan.passes.contains(&make_a_key));
        assert_eq!(plan.aliases.get(&a_key), plan.aliases.get(&b_key));
    }
}
