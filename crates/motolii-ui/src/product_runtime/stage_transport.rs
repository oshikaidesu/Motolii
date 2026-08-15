//! Stage transport snapshot。Documentを書かず表示口へ渡す。

use motolii_core::RationalTime;
use motolii_doc::LayerId;

use crate::stage_chrome_host_runtime::StagePlaybackState;
use crate::stage_chrome_host_runtime::StageTransportSnapshot;

use super::position::position_active_interval;

pub(super) fn stage_transport_snapshot(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
) -> StageTransportSnapshot {
    stage_transport_snapshot_with_state(document, primary, playhead, StagePlaybackState::Idle)
}

pub(super) fn stage_transport_snapshot_with_state(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
    playback_state: StagePlaybackState,
) -> StageTransportSnapshot {
    let object_name = position_active_interval(document, primary, playhead).and_then(|interval| {
        document
            .layers
            .display_name(interval.layer)
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
    });
    if playback_state == StagePlaybackState::Idle {
        StageTransportSnapshot::with_position_active_interval(object_name)
    } else {
        StageTransportSnapshot::with_position_active_interval_and_state(object_name, playback_state)
    }
}

#[allow(dead_code)]
pub(super) fn publish_stage_transport_snapshot<E>(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
    publish: impl FnOnce(&StageTransportSnapshot) -> Result<(), E>,
) -> Result<(), E> {
    publish_stage_transport_snapshot_with_state(
        document,
        primary,
        playhead,
        StagePlaybackState::Idle,
        publish,
    )
}

pub(super) fn publish_stage_transport_snapshot_with_state<E>(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    playhead: RationalTime,
    playback_state: StagePlaybackState,
    publish: impl FnOnce(&StageTransportSnapshot) -> Result<(), E>,
) -> Result<(), E> {
    let snapshot = stage_transport_snapshot_with_state(document, primary, playhead, playback_state);
    publish(&snapshot)
}
