use std::collections::BTreeMap;

use super::graph::GpuResourceGraph;
use super::planner::GpuExecutionPlan;
use super::types::{GpuPassDesc, GpuPassKey, GpuPhysicalSlot, GpuResourceKey};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuExecutionStats {
    pub passes_executed: u64,
    pub passes_skipped_cached: u64,
    pub aliased_resources: u64,
    pub physical_scratch_slots: u64,
}

#[derive(Debug)]
pub(crate) enum GpuExecuteError<E> {
    MissingPass(GpuPassKey),
    Backend(E),
}

/// Concrete backend boundary. wgpu/re_renderer implementation belongs behind
/// this trait; planning/invalidation must never leak back into the backend.
pub(crate) trait GpuBackend {
    type Error;

    fn execute_pass(
        &mut self,
        pass: &GpuPassDesc,
        aliases: &BTreeMap<GpuResourceKey, GpuPhysicalSlot>,
    ) -> Result<(), Self::Error>;
}

#[derive(Default)]
pub(crate) struct GpuExecutor;

impl GpuExecutor {
    pub fn execute<B: GpuBackend>(
        &self,
        graph: &mut GpuResourceGraph,
        plan: &GpuExecutionPlan,
        generation: u64,
        backend: &mut B,
    ) -> Result<GpuExecutionStats, GpuExecuteError<B::Error>> {
        let mut stats = GpuExecutionStats {
            passes_skipped_cached: plan.skipped_cached.len() as u64,
            aliased_resources: plan.aliases.len() as u64,
            physical_scratch_slots: plan
                .aliases
                .values()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len() as u64,
            ..Default::default()
        };

        for pass_key in &plan.passes {
            let pass = graph
                .pass(*pass_key)
                .cloned()
                .ok_or(GpuExecuteError::MissingPass(*pass_key))?;
            backend
                .execute_pass(&pass, &plan.aliases)
                .map_err(GpuExecuteError::Backend)?;
            graph.mark_pass_outputs_resident(*pass_key, generation);
            stats.passes_executed += 1;
        }

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_graph::{NodeIdentity, NodeKey, NodeKind};
    use crate::gpu_exec::graph::GpuResourceGraph;
    use crate::gpu_exec::planner::GpuPlanner;
    use crate::gpu_exec::types::{
        GpuIdentitySource, GpuPassIdentity, GpuPassKind, GpuResourceClass, GpuResourceDesc,
        GpuResourceIdentity, GpuResourceLifetime, GpuResourceVersion,
    };

    #[derive(Default)]
    struct Recorder(Vec<GpuPassKey>);

    impl GpuBackend for Recorder {
        type Error = ();

        fn execute_pass(
            &mut self,
            pass: &GpuPassDesc,
            _aliases: &BTreeMap<GpuResourceKey, GpuPhysicalSlot>,
        ) -> Result<(), Self::Error> {
            self.0.push(pass.key());
            Ok(())
        }
    }

    fn node(tag: u16) -> NodeKey {
        NodeKey::for_identity(&NodeIdentity::new(NodeKind::Custom(tag), vec![]))
    }

    #[test]
    fn executor_only_runs_the_precomputed_plan_and_commits_residency() {
        let mut graph = GpuResourceGraph::default();
        let resource = GpuResourceDesc {
            identity: GpuResourceIdentity {
                source: GpuIdentitySource::Semantic(node(1)),
                class: GpuResourceClass::Content,
                slot: 0,
            },
            version: GpuResourceVersion::new(9),
            dependencies: vec![],
            lifetime: GpuResourceLifetime::Persistent,
            alias_class: None,
            estimated_bytes: 4,
        };
        let resource_key = resource.key();
        graph.upsert_resource(resource).unwrap();

        let upload = GpuPassDesc {
            identity: GpuPassIdentity {
                kind: GpuPassKind::Upload,
                tag: 1,
                reads: vec![],
                writes: vec![resource_key],
            },
            after: vec![],
            cacheable: true,
            side_effect: false,
        };
        let upload_key = upload.key();
        graph.insert_pass(upload).unwrap();

        let plan = GpuPlanner.plan(&graph, [upload_key]).unwrap();
        let mut recorder = Recorder::default();
        let stats = GpuExecutor
            .execute(&mut graph, &plan, 3, &mut recorder)
            .unwrap();

        assert_eq!(recorder.0, vec![upload_key]);
        assert_eq!(stats.passes_executed, 1);
        assert!(graph.is_current(resource_key));

        let cached = GpuPlanner.plan(&graph, [upload_key]).unwrap();
        assert!(cached.passes.is_empty());
        assert_eq!(cached.skipped_cached, vec![upload_key]);
    }
}
