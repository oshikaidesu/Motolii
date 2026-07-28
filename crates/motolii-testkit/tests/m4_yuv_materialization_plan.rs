//! M4 YUV materialization lane選定のtest-only契約。

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FrameSize {
    width: u32,
    height: u32,
}

impl FrameSize {
    const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Lane {
    size: FrameSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BatchPlan {
    lane_indices: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
enum PlanError {
    #[error("YUV materialization demand {requested} exceeds injected lane cap {cap}")]
    LaneCapExceeded { requested: usize, cap: usize },
}

#[derive(Debug)]
struct MaterializationPlanner {
    cap: usize,
    lanes: Vec<Lane>,
    growth_events: BTreeMap<FrameSize, usize>,
}

impl MaterializationPlanner {
    fn new(cap: usize) -> Self {
        Self {
            cap,
            lanes: Vec::new(),
            growth_events: BTreeMap::new(),
        }
    }

    fn plan(&mut self, demands: &[FrameSize]) -> Result<BatchPlan, PlanError> {
        if demands.len() > self.cap {
            return Err(PlanError::LaneCapExceeded {
                requested: demands.len(),
                cap: self.cap,
            });
        }

        let mut candidate_lanes = self.lanes.clone();
        let mut used = vec![false; candidate_lanes.len()];
        let mut lane_indices = Vec::with_capacity(demands.len());
        let mut growth_by_size = BTreeMap::<FrameSize, usize>::new();
        for &size in demands {
            let lane_index = candidate_lanes
                .iter()
                .enumerate()
                .find_map(|(index, lane)| (!used[index] && lane.size == size).then_some(index))
                .unwrap_or_else(|| {
                    let index = candidate_lanes.len();
                    candidate_lanes.push(Lane { size });
                    used.push(false);
                    *growth_by_size.entry(size).or_default() += 1;
                    index
                });
            used[lane_index] = true;
            lane_indices.push(lane_index);
        }

        self.lanes = candidate_lanes;
        for size in growth_by_size.keys() {
            *self.growth_events.entry(*size).or_default() += 1;
        }
        Ok(BatchPlan { lane_indices })
    }
}

const HD: FrameSize = FrameSize::new(1280, 720);
const FHD: FrameSize = FrameSize::new(1920, 1080);

#[test]
fn batch_assigns_one_distinct_lane_per_live_source() {
    let mut planner = MaterializationPlanner::new(4);
    let plan = planner.plan(&[FHD, FHD, FHD, FHD]).unwrap();

    let mut unique = plan.lane_indices.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), 4);
}

#[test]
fn shrinking_then_regrowing_reuses_high_watermark_lanes() {
    let mut planner = MaterializationPlanner::new(4);
    let first = planner.plan(&[FHD, FHD, FHD]).unwrap();
    let shrunk = planner.plan(&[FHD]).unwrap();
    let regrown = planner.plan(&[FHD, FHD, FHD]).unwrap();

    assert_eq!(planner.lanes.len(), 3);
    assert_eq!(planner.growth_events.get(&FHD), Some(&1));
    assert_eq!(first.lane_indices, regrown.lane_indices);
    assert_eq!(shrunk.lane_indices, vec![first.lane_indices[0]]);
}

#[test]
fn alternating_mixed_size_order_stops_growing_after_warmup() {
    let mut planner = MaterializationPlanner::new(4);
    planner.plan(&[FHD, HD, FHD]).unwrap();
    planner.plan(&[HD, FHD, HD]).unwrap();
    let warmed_lane_count = planner.lanes.len();
    let warmed_growth = planner.growth_events.clone();

    for _ in 0..5 {
        planner.plan(&[FHD, HD, FHD]).unwrap();
        planner.plan(&[HD, FHD, HD]).unwrap();
    }

    assert_eq!(warmed_lane_count, 4);
    assert_eq!(planner.lanes.len(), warmed_lane_count);
    assert_eq!(planner.growth_events, warmed_growth);
}

#[test]
fn cap_refusal_is_typed_and_does_not_partially_grow() {
    let mut planner = MaterializationPlanner::new(3);
    let before = planner.plan(&[HD]).unwrap();
    let lane_count = planner.lanes.len();
    let growth = planner.growth_events.clone();

    let error = planner.plan(&[FHD, FHD, FHD, FHD]).unwrap_err();

    assert_eq!(
        error,
        PlanError::LaneCapExceeded {
            requested: 4,
            cap: 3
        }
    );
    assert_eq!(before.lane_indices, vec![0]);
    assert_eq!(planner.lanes.len(), lane_count);
    assert_eq!(planner.growth_events, growth);
}

#[test]
fn failure_does_not_prevent_a_later_valid_batch() {
    let mut planner = MaterializationPlanner::new(2);
    assert!(planner.plan(&[HD, HD, HD]).is_err());

    let plan = planner.plan(&[HD, FHD]).unwrap();
    assert_eq!(plan.lane_indices, vec![0, 1]);
}
