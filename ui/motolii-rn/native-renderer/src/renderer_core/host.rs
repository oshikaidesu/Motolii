use super::types::NativeHostTerminalEvent;
use crate::timeline_skia::TimelineScene;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum HostStageGeometryCommand {
    Apply(crate::host_bridge::HostStageGeometry),
    Clear,
    Noop,
}

pub(super) fn host_stage_geometry_command(
    previous: Option<&crate::host_bridge::HostStageGeometry>,
    projection: Option<&crate::host_bridge::HostTimelineProjection>,
) -> HostStageGeometryCommand {
    let next = projection.and_then(|next| next.stage_geometry.as_ref());
    match (previous, next) {
        (Some(current), Some(next)) if current == next => HostStageGeometryCommand::Noop,
        (Some(_), Some(next)) => HostStageGeometryCommand::Apply(next.clone()),
        (None, Some(next)) => HostStageGeometryCommand::Apply(next.clone()),
        (Some(_), None) => HostStageGeometryCommand::Clear,
        (None, None) => HostStageGeometryCommand::Noop,
    }
}

pub(super) fn stage_selection_commit(
    selected_entity_path: Option<&str>,
) -> crate::timeline_skia::TimelineSelectionCommit {
    selected_entity_path
        .and_then(crate::rerun_stage::host_layer_id_from_entity_path)
        .map_or(
            crate::timeline_skia::TimelineSelectionCommit::ClearSelection,
            |layer_id| crate::timeline_skia::TimelineSelectionCommit::SelectLayer {
                layer_id: layer_id.to_owned(),
            },
        )
}

fn terminal_projection_is_stale(
    current_host_handle: Option<&str>,
    current_generation: Option<&str>,
    projection: &crate::host_bridge::HostTimelineProjection,
) -> bool {
    let (Some(current_host), Some(next_host)) = (
        current_host_handle.and_then(|value| value.parse::<u64>().ok()),
        projection
            .host_handle
            .as_deref()
            .and_then(|value| value.parse::<u64>().ok()),
    ) else {
        return false;
    };
    if next_host != current_host {
        return next_host < current_host;
    }
    let (Some(current_generation), Some(next_generation)) = (
        current_generation.and_then(|value| value.parse::<u64>().ok()),
        projection.projection_generation.parse::<u64>().ok(),
    ) else {
        return false;
    };
    next_generation < current_generation
}

#[derive(Default)]
pub(super) struct HostTerminalLatch(Option<NativeHostTerminalEvent>);

impl HostTerminalLatch {
    pub(super) fn record(&mut self, result: &crate::host_bridge::HostTerminalResult) {
        self.0 = Some(NativeHostTerminalEvent {
            accepted: result.accepted,
            message: result.feedback().unwrap_or_default().to_owned(),
        });
    }

    pub(super) fn take(&mut self) -> Option<NativeHostTerminalEvent> {
        self.0.take()
    }

    pub(super) fn record_if_current(
        &mut self,
        current_host_handle: Option<&str>,
        current_generation: Option<&str>,
        result: &crate::host_bridge::HostTerminalResult,
    ) -> bool {
        if result.projection.as_ref().is_some_and(|projection| {
            terminal_projection_is_stale(current_host_handle, current_generation, projection)
        }) {
            return false;
        }
        self.record(result);
        true
    }
}

pub(super) fn timeline_scene_from_projection(
    existing_scene: &TimelineScene,
    projection: &crate::host_bridge::HostTimelineProjection,
) -> TimelineScene {
    let fallback_song_bars = (10.0f64 / crate::timeline_skia::SECONDS_PER_BAR) as f32;
    let song_bars = projection
        .timeline_duration
        .and_then(|(num, den)| {
            if den <= 0 || num < 0 {
                None
            } else {
                Some((num as f64 / den as f64 / crate::timeline_skia::SECONDS_PER_BAR) as f32)
            }
        })
        .filter(|bars| bars.is_finite())
        .unwrap_or(fallback_song_bars);
    let layers = crate::host_bridge::snapshot_layers_from_projection(projection);
    let mut scene = TimelineScene::from_snapshot_with_song_bars(
        &layers,
        projection.primary_layer_id.as_deref(),
        song_bars,
    );
    if let Some((num, den)) = projection.fps {
        scene = scene.with_fps(num, den);
    }
    if let Some(timeline_layers) = &projection.timeline_layers {
        scene.apply_layer_mute_solo(
            timeline_layers
                .iter()
                .map(|layer| (layer.visible, layer.solo, layer.effects.len())),
        );
    }
    // real同士の差し替えではlocal viewを維持。fixture→real初回はfrom_snapshotの0..song_bars。
    if existing_scene.real {
        scene.view_a = existing_scene.view_a;
        scene.view_b = existing_scene.view_b;
        let span = scene.view_b - scene.view_a;
        if scene.view_a < 0.0 {
            scene.view_a = 0.0;
            scene.view_b = span.min(scene.song_bars);
        }
        if scene.view_b > scene.song_bars {
            scene.view_b = scene.song_bars;
            scene.view_a = (scene.song_bars - span).max(0.0);
        }
        // revision再投影でkeyのselが落ちるとDeleteがlayer削除へ化ける。key_id一致で引き継ぐ。
        if let Some((layer_id, key_id)) = crate::timeline_skia::selected_real_key(existing_scene) {
            if projection.primary_layer_id.as_deref() == Some(layer_id.as_str()) {
                let _ = crate::timeline_skia::restore_key_selection(
                    &mut scene,
                    layer_id.as_str(),
                    key_id,
                );
            }
        }
    }
    scene
}

pub(super) fn timeline_projection_selected_flat(
    projection: &crate::host_bridge::HostTimelineProjection,
) -> i32 {
    let Some(primary) = projection.primary_layer_id.as_deref() else {
        return -1;
    };
    let position = projection.timeline_layers.as_ref().map_or_else(
        || {
            projection
                .bounds
                .iter()
                .position(|(layer_id, _)| layer_id == primary)
        },
        |layers| layers.iter().position(|layer| layer.layer_id == primary),
    );
    position
        .and_then(|index| i32::try_from(index).ok())
        .unwrap_or(-1)
}
