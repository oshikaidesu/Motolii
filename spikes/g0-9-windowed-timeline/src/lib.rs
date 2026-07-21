use serde::Serialize;

pub const KEYFRAME_COUNT: usize = 100_000;
pub const DEFAULT_WARMUP_FRAMES: u32 = 120;
pub const DEFAULT_MEASURE_FRAMES: u32 = 100;
pub const DEFAULT_MEASURE_SECONDS: f64 = 30.0;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct KeyInstance {
    pub time_seconds: f32,
    pub track: f32,
    pub selected: u32,
    pub _padding: u32,
}

pub fn make_key_instances(count: usize) -> Vec<KeyInstance> {
    (0..count)
        .map(|index| KeyInstance {
            time_seconds: (index % 10_000) as f32 * 0.01,
            track: (index % 32) as f32,
            selected: u32::from(index % 10 == 0),
            _padding: 0,
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ResourceCreationCounters {
    pub pipelines: u64,
    pub buffers: u64,
    pub bind_groups: u64,
    pub textures: u64,
}

impl ResourceCreationCounters {
    pub fn delta(self, baseline: Self) -> Self {
        Self {
            pipelines: self.pipelines.saturating_sub(baseline.pipelines),
            buffers: self.buffers.saturating_sub(baseline.buffers),
            bind_groups: self.bind_groups.saturating_sub(baseline.bind_groups),
            textures: self.textures.saturating_sub(baseline.textures),
        }
    }

    pub fn is_zero(self) -> bool {
        self == Self::default()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MeasurementSummary {
    pub samples: usize,
    pub median_frame_ms: f64,
    pub p95_frame_ms: f64,
    pub max_frame_ms: f64,
}

pub fn summarize_samples(samples: &[f64]) -> Option<MeasurementSummary> {
    if samples.is_empty()
        || samples
            .iter()
            .any(|sample| !sample.is_finite() || *sample < 0.0)
    {
        return None;
    }
    let mut ordered = samples.to_vec();
    ordered.sort_by(f64::total_cmp);
    let percentile = |fraction: f64| {
        let index = ((ordered.len() - 1) as f64 * fraction).ceil() as usize;
        ordered[index]
    };
    Some(MeasurementSummary {
        samples: ordered.len(),
        median_frame_ms: percentile(0.5),
        p95_frame_ms: percentile(0.95),
        max_frame_ms: *ordered.last().unwrap(),
    })
}

#[derive(Clone, Copy, Debug)]
pub struct AcceptanceInput {
    pub measured_frames: u32,
    pub target_frames: u32,
    pub measured_seconds: f64,
    pub target_seconds: f64,
    pub acquire_count: u64,
    pub present_count: u64,
    pub readback_count: u64,
    pub frame_creations: ResourceCreationCounters,
}

pub fn acceptance_passes(input: AcceptanceInput) -> bool {
    input.measured_frames >= input.target_frames
        && input.measured_seconds >= input.target_seconds
        && input.acquire_count > 0
        && input.acquire_count == input.present_count
        && input.readback_count == 0
        && input.frame_creations.is_zero()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_has_exactly_one_hundred_thousand_stable_instances() {
        let instances = make_key_instances(KEYFRAME_COUNT);
        assert_eq!(instances.len(), KEYFRAME_COUNT);
        assert_eq!(instances[0].time_seconds, 0.0);
        assert_eq!(instances[99_999].time_seconds, 99.99);
        assert_eq!(
            instances.iter().filter(|key| key.selected == 1).count(),
            10_000
        );
    }

    #[test]
    fn summary_is_deterministic_and_rejects_invalid_samples() {
        let summary = summarize_samples(&[4.0, 1.0, 3.0, 2.0]).unwrap();
        assert_eq!(summary.samples, 4);
        assert_eq!(summary.median_frame_ms, 3.0);
        assert_eq!(summary.p95_frame_ms, 4.0);
        assert_eq!(summary.max_frame_ms, 4.0);
        assert!(summarize_samples(&[]).is_none());
        assert!(summarize_samples(&[f64::NAN]).is_none());
    }

    #[test]
    fn acceptance_requires_duration_repetitions_and_zero_hot_loop_creations() {
        let zero = ResourceCreationCounters::default();
        let good = AcceptanceInput {
            measured_frames: 100,
            target_frames: 100,
            measured_seconds: 30.0,
            target_seconds: 30.0,
            acquire_count: 100,
            present_count: 100,
            readback_count: 0,
            frame_creations: zero,
        };
        assert!(acceptance_passes(good));
        assert!(!acceptance_passes(AcceptanceInput {
            measured_frames: 99,
            ..good
        }));
        assert!(!acceptance_passes(AcceptanceInput {
            measured_seconds: 29.9,
            ..good
        }));
        assert!(!acceptance_passes(AcceptanceInput {
            present_count: 99,
            ..good
        }));
        assert!(!acceptance_passes(AcceptanceInput {
            frame_creations: ResourceCreationCounters {
                bind_groups: 1,
                ..zero
            },
            ..good
        }));
    }

    #[test]
    fn render_hot_loop_has_no_tracked_resource_creation_or_readback_call() {
        let source = include_str!("main.rs");
        let render = source
            .split("fn render(&mut self")
            .nth(1)
            .and_then(|tail| tail.split("#[derive(Serialize)]").next())
            .expect("render source section");
        for forbidden in [
            "create_buffer(",
            "create_bind_group(",
            "create_render_pipeline(",
            "create_texture(",
            "copy_texture",
            "map_async",
            "PollType::wait",
        ] {
            assert!(
                !render.contains(forbidden),
                "render hot loop contains forbidden call: {forbidden}",
            );
        }
    }
}
