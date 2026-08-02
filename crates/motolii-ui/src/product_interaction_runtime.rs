#[derive(Debug, Clone, Copy)]
struct SnapCandidate {
    delta: RationalTime,
    target_kind: SnapTargetKind,
    target_time: RationalTime,
    other_layer: Option<LayerId>,
    target_edge: IntervalEdge,
    moving_edge: IntervalEdge,
}

fn rational_abs(value: RationalTime) -> Option<RationalTime> {
    if value < RationalTime::ZERO {
        value.try_neg().ok()
    } else {
        Some(value)
    }
}

fn snap_candidate_is_better(candidate: SnapCandidate, current: SnapCandidate) -> bool {
    let Some(candidate_distance) = rational_abs(candidate.delta) else {
        return false;
    };
    let Some(current_distance) = rational_abs(current.delta) else {
        return true;
    };
    let kind_rank = |kind| match kind {
        SnapTargetKind::OtherClip => 0,
        SnapTargetKind::Frame => 1,
    };
    let edge_rank = |edge| match edge {
        IntervalEdge::In => 0,
        IntervalEdge::Out => 1,
    };
    candidate_distance
        .cmp(&current_distance)
        .then_with(|| kind_rank(candidate.target_kind).cmp(&kind_rank(current.target_kind)))
        .then_with(|| candidate.target_time.cmp(&current.target_time))
        .then_with(|| candidate.other_layer.cmp(&current.other_layer))
        .then_with(|| edge_rank(candidate.target_edge).cmp(&edge_rank(current.target_edge)))
        .then_with(|| edge_rank(candidate.moving_edge).cmp(&edge_rank(current.moving_edge)))
        .is_lt()
}

fn interval_preview_candidate(
    active: ActiveIntervalGesture,
    pointer_time: RationalTime,
    layout: NativeHostLayout,
    timeline: &ProductTimelineProjection,
    document: &motolii_doc::Document,
) -> Option<TimelineIntervalPreview> {
    let pointer_delta = pointer_time.try_sub(active.grab_time).ok()?;
    let (raw_start, raw_end, moving_edges) = match active.target.kind {
        IntervalGestureKind::Move => {
            let start = active.target.start.try_add(pointer_delta).ok()?;
            let end = active.target.end.try_add(pointer_delta).ok()?;
            (
                start,
                end,
                [
                    Some((IntervalEdge::In, start)),
                    Some((IntervalEdge::Out, end)),
                ],
            )
        }
        IntervalGestureKind::TrimIn => {
            let start = active.target.start.try_add(pointer_delta).ok()?;
            (
                start,
                active.target.end,
                [Some((IntervalEdge::In, start)), None],
            )
        }
        IntervalGestureKind::TrimOut => {
            let end = active.target.end.try_add(pointer_delta).ok()?;
            (
                active.target.start,
                end,
                [Some((IntervalEdge::Out, end)), None],
            )
        }
    };

    let mut best = None;
    for (moving_edge, moving_time) in moving_edges.into_iter().flatten() {
        if let Ok(frame) = moving_time.try_to_frame_round(document.composition.fps) {
            if let Ok(target_time) = RationalTime::try_from_frame(frame, document.composition.fps) {
                consider_snap_candidate(
                    &mut best,
                    SnapCandidate {
                        delta: target_time.try_sub(moving_time).ok()?,
                        target_kind: SnapTargetKind::Frame,
                        target_time,
                        other_layer: None,
                        target_edge: IntervalEdge::In,
                        moving_edge,
                    },
                    moving_time,
                    layout,
                    timeline,
                );
            }
        }
        for bar in timeline.projection.bars() {
            if bar.layer == active.target.layer {
                continue;
            }
            for (target_edge, target_time) in
                [(IntervalEdge::In, bar.start), (IntervalEdge::Out, bar.end)]
            {
                consider_snap_candidate(
                    &mut best,
                    SnapCandidate {
                        delta: target_time.try_sub(moving_time).ok()?,
                        target_kind: SnapTargetKind::OtherClip,
                        target_time,
                        other_layer: Some(bar.layer),
                        target_edge,
                        moving_edge,
                    },
                    moving_time,
                    layout,
                    timeline,
                );
            }
        }
    }
    let adjustment = best.map_or(RationalTime::ZERO, |candidate| candidate.delta);
    let start = raw_start.try_add(adjustment).ok()?;
    let end = raw_end.try_add(adjustment).ok()?;
    (end > start).then_some(TimelineIntervalPreview {
        layer: active.target.layer,
        start,
        end,
    })
}

fn consider_snap_candidate(
    best: &mut Option<SnapCandidate>,
    candidate: SnapCandidate,
    moving_time: RationalTime,
    layout: NativeHostLayout,
    timeline: &ProductTimelineProjection,
) {
    if candidate.target_time < timeline.viewport.start
        || candidate.target_time > timeline.viewport.end
    {
        return;
    }
    let Some(moving_x) = timeline.logical_x(moving_time, layout) else {
        return;
    };
    let Some(target_x) = timeline.logical_x(candidate.target_time, layout) else {
        return;
    };
    if (target_x - moving_x).abs() > SNAP_DISTANCE_LOGICAL_PX {
        return;
    }
    if best.is_none_or(|current| snap_candidate_is_better(candidate, current)) {
        *best = Some(candidate);
    }
}

#[derive(Debug, Default)]
struct PlaceTerminalDelivery {
    delivered_high_water: Option<u64>,
}

impl PlaceTerminalDelivery {
    fn deliver(&mut self, terminal: &ClassifiedPlaceTerminal) -> Option<PendingStageDrop> {
        if terminal.cause != PlaceTerminalCause::NoNonCommitCause
            || self
                .delivered_high_water
                .is_some_and(|high_water| terminal.generation <= high_water)
        {
            return None;
        }
        let (Some(layout_epoch), Some(ndc)) = (terminal.layout_epoch, terminal.stage_ndc) else {
            return None;
        };
        self.delivered_high_water = Some(terminal.generation);
        Some(PendingStageDrop {
            source: terminal.source.clone(),
            generation: terminal.generation,
            layout_epoch,
            ndc,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlaceTerminalCause {
    Escape,
    OutsideStage,
    CaptureLoss,
    NoNonCommitCause,
}

#[derive(Debug, Clone, PartialEq)]
struct ClassifiedPlaceTerminal {
    source: BrowserPlaceIntent,
    generation: u64,
    cause: PlaceTerminalCause,
    layout_epoch: Option<u64>,
    stage_ndc: Option<[f64; 2]>,
}

impl ClassifiedPlaceTerminal {
    fn released(
        source: BrowserPlaceIntent,
        generation: u64,
        position: [f64; 2],
        layout: NativeHostLayout,
    ) -> Self {
        let stage_ndc = layout.stage_ndc(position);
        Self {
            source,
            generation,
            cause: if stage_ndc.is_some() {
                PlaceTerminalCause::NoNonCommitCause
            } else {
                PlaceTerminalCause::OutsideStage
            },
            layout_epoch: Some(layout.epoch),
            stage_ndc,
        }
    }

    fn cancelled(source: BrowserPlaceIntent, generation: u64, reason: HostPointerCancel) -> Self {
        Self {
            source,
            generation,
            cause: match reason {
                HostPointerCancel::Escape => PlaceTerminalCause::Escape,
                HostPointerCancel::CaptureLost => PlaceTerminalCause::CaptureLoss,
            },
            layout_epoch: None,
            stage_ndc: None,
        }
    }
}

#[derive(Debug, Default)]
struct PlaceTerminalAdmission {
    active_generation: Option<u64>,
    retired_high_water: Option<u64>,
}

impl PlaceTerminalAdmission {
    fn begin(&mut self, generation: u64) -> bool {
        if self.active_generation.is_some()
            || self
                .retired_high_water
                .is_some_and(|high_water| generation <= high_water)
        {
            return false;
        }
        self.active_generation = Some(generation);
        true
    }

    fn admit(&mut self, terminal: &ClassifiedPlaceTerminal) -> bool {
        if self.active_generation != Some(terminal.generation) {
            return false;
        }
        self.active_generation = None;
        self.retired_high_water = Some(
            self.retired_high_water
                .map_or(terminal.generation, |high_water| {
                    high_water.max(terminal.generation)
                }),
        );
        terminal.cause == PlaceTerminalCause::NoNonCommitCause
    }

    fn retire_active(&mut self) {
        if let Some(generation) = self.active_generation.take() {
            self.retired_high_water = Some(
                self.retired_high_water
                    .map_or(generation, |high_water| high_water.max(generation)),
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PlacePreviewProgress {
    source: BrowserPlaceIntent,
    generation: u64,
    layout_epoch: u64,
    stage_ndc: Option<[f64; 2]>,
}

#[derive(Debug, Default)]
struct PlacePreviewPhase {
    latest: Option<PlacePreviewProgress>,
}

impl PlacePreviewPhase {
    fn deliver(
        &mut self,
        source: &BrowserPlaceIntent,
        generation: u64,
        position: [f64; 2],
        layout: NativeHostLayout,
    ) {
        self.latest = Some(PlacePreviewProgress {
            source: source.clone(),
            generation,
            layout_epoch: layout.epoch,
            stage_ndc: layout.stage_ndc(position),
        });
    }

    fn clear(&mut self) {
        self.latest = None;
    }
}
