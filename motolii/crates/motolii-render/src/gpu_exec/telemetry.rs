use super::types::{GpuPassKey, GpuResourceKey};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuFrameCounters {
    pub resources_total: u64,
    pub resources_resident: u64,
    pub logical_passes_total: u64,
    pub planned_passes: u64,
    pub cache_skips: u64,
    pub aliased_resources: u64,
    pub physical_scratch_slots: u64,
    pub readback_passes: u64,
    pub present_passes: u64,
}

#[derive(Default)]
pub(crate) struct GpuTelemetry {
    pub frame: GpuFrameCounters,
    pub resource_rebuilds: std::collections::BTreeMap<GpuResourceKey, u64>,
    pub pass_runs: std::collections::BTreeMap<GpuPassKey, u64>,
}

impl GpuTelemetry {
    pub fn begin_frame(&mut self) { self.frame = GpuFrameCounters::default(); }

    pub fn resource_rebuilt(&mut self, key: GpuResourceKey) {
        *self.resource_rebuilds.entry(key).or_default() += 1;
    }

    pub fn pass_ran(&mut self, key: GpuPassKey) {
        *self.pass_runs.entry(key).or_default() += 1;
    }
}
