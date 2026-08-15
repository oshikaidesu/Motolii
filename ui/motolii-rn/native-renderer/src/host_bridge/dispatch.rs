use motolii_ui::{
    AppStageTransformEdit, host_commit_stage_transform_for_app,
    host_preview_stage_transform_for_app,
};

use super::parse_wire::snapshot_has_position_key;
use super::projection::{rational_time_parts_from_bar, try_read_timeline_projection};
use super::slot::host_slot;
use super::terminal::dispatch_intent_json_terminal;
use super::types::{HostStageGeometry, HostTerminalDiagnostic, HostTerminalResult};

#[cfg(test)]
use std::sync::atomic::Ordering;

#[cfg(test)]
use super::slot::{
    TEST_KEYMAP_DELETE_LAYER_COUNT, TEST_KEYMAP_REMOVE_POSITION_KEY_COUNT,
    TEST_MOVE_LAYER_BY_DISPATCH_COUNT, TEST_SELECTION_DISPATCH_COUNT,
    TEST_SET_POSITION_KEY_TIME_DISPATCH_COUNT,
};

/// process host slot経由でset_timeを送り、同じHost応答を返す。host不在はNone。
pub(crate) fn try_dispatch_set_time(frame: i64) -> Option<HostTerminalResult> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = frame;
        None
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let intent = format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","host_handle":"{}","frame":{}}}"#,
            slot.handle, frame
        );
        dispatch_intent_json_terminal(slot.handle, &intent)
    }
}

pub(crate) fn try_dispatch_timeline_edit(
    commit: &crate::timeline_skia::TimelineEditCommit,
) -> Option<HostTerminalResult> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = commit;
        None
    }
    #[cfg(target_os = "macos")]
    {
        use crate::timeline_skia::TimelineEditCommit;
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let intent = match commit {
            TimelineEditCommit::SetClipStart { layer_id, bar } => {
                let (num, den) = rational_time_parts_from_bar(f64::from(*bar));
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"set_clip_start","#,
                        r#""host_handle":"{}","target":"{}","time":{{"num":{},"den":{}}}}}"#
                    ),
                    slot.handle, layer_id, num, den
                )
            }
            TimelineEditCommit::TrimClipIn { layer_id, bar } => {
                let (num, den) = rational_time_parts_from_bar(f64::from(*bar));
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"trim_clip_in","#,
                        r#""host_handle":"{}","target":"{}","time":{{"num":{},"den":{}}}}}"#
                    ),
                    slot.handle, layer_id, num, den
                )
            }
            TimelineEditCommit::TrimClipOut { layer_id, bar } => {
                let (num, den) = rational_time_parts_from_bar(f64::from(*bar));
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"trim_clip_out","#,
                        r#""host_handle":"{}","target":"{}","time":{{"num":{},"den":{}}}}}"#
                    ),
                    slot.handle, layer_id, num, den
                )
            }
            TimelineEditCommit::SetPositionKeyTime {
                layer_id,
                key_id,
                bar,
            } => {
                // param_keys は diamond だけ。position_keys に無い id は commit しない。
                if !snapshot_has_position_key(slot.handle, layer_id, *key_id) {
                    return None;
                }
                #[cfg(test)]
                TEST_SET_POSITION_KEY_TIME_DISPATCH_COUNT.fetch_add(1, Ordering::SeqCst);
                let (num, den) = rational_time_parts_from_bar(f64::from(*bar));
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"set_position_key_time","#,
                        r#""host_handle":"{}","target":"{}","key_id":"{}","time":{{"num":{},"den":{}}}}}"#
                    ),
                    slot.handle, layer_id, key_id, num, den
                )
            }
            TimelineEditCommit::ReparentClip {
                layer_id,
                dest_layer_id,
                bar,
            } => {
                let (num, den) = rational_time_parts_from_bar(f64::from(*bar));
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"reparent_clip","#,
                        r#""host_handle":"{}","target":"{}","dest":"{}","time":{{"num":{},"den":{}}}}}"#
                    ),
                    slot.handle, layer_id, dest_layer_id, num, den
                )
            }
            TimelineEditCommit::ToggleMute { layer_id } => format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"mute","#,
                    r#""host_handle":"{}","target":"{}"}}"#
                ),
                slot.handle, layer_id
            ),
            TimelineEditCommit::ToggleSolo { layer_id } => format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"solo","#,
                    r#""host_handle":"{}","target":"{}"}}"#
                ),
                slot.handle, layer_id
            ),
            TimelineEditCommit::RemovePositionKey { layer_id, key_id } => {
                // param_keys diamond の Delete を Position 削除へ流さない。
                if !snapshot_has_position_key(slot.handle, layer_id, *key_id) {
                    return None;
                }
                format!(
                    concat!(
                        r#"{{"version":1,"direction":"rn-to-host","kind":"remove_position_key","#,
                        r#""host_handle":"{}","target":"{}","key_id":"{}"}}"#
                    ),
                    slot.handle, layer_id, key_id
                )
            }
        };
        dispatch_intent_json_terminal(slot.handle, &intent)
    }
}

pub(crate) fn try_dispatch_remove_position_key(
    layer_id: &str,
    key_id: u64,
) -> Option<HostTerminalResult> {
    #[cfg(test)]
    TEST_KEYMAP_REMOVE_POSITION_KEY_COUNT.fetch_add(1, Ordering::SeqCst);
    try_dispatch_timeline_edit(
        &crate::timeline_skia::TimelineEditCommit::RemovePositionKey {
            layer_id: layer_id.to_string(),
            key_id,
        },
    )
}

pub(crate) fn try_dispatch_timeline_selection(
    commit: &crate::timeline_skia::TimelineSelectionCommit,
) -> Option<HostTerminalResult> {
    #[cfg(test)]
    TEST_SELECTION_DISPATCH_COUNT.fetch_add(1, Ordering::SeqCst);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = commit;
        None
    }
    #[cfg(target_os = "macos")]
    {
        use crate::timeline_skia::TimelineSelectionCommit;
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let intent = match commit {
            TimelineSelectionCommit::SelectLayer { layer_id } => format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"select_layer","#,
                    r#""host_handle":"{}","target":"{}"}}"#
                ),
                slot.handle, layer_id
            ),
            TimelineSelectionCommit::ClearSelection => format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"clear_selection","#,
                    r#""host_handle":"{}"}}"#
                ),
                slot.handle
            ),
        };
        dispatch_intent_json_terminal(slot.handle, &intent)
    }
}

pub(crate) fn try_dispatch_move_layer_by(
    target: &str,
    delta: [f64; 2],
) -> Option<HostTerminalResult> {
    #[cfg(test)]
    TEST_MOVE_LAYER_BY_DISPATCH_COUNT.fetch_add(1, Ordering::SeqCst);
    if !delta.iter().all(|value| value.is_finite()) {
        return None;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = target;
        None
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let intent = format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"move_layer_by","#,
                r#""host_handle":"{}","target":"{}","delta":[{},{}]}}"#
            ),
            slot.handle, target, delta[0], delta[1]
        );
        dispatch_intent_json_terminal(slot.handle, &intent)
    }
}

/// mount/resize 済みの論理 viewport。未設定は None。
pub(crate) fn try_stage_logical_size() -> Option<(f64, f64)> {
    let Ok(guard) = host_slot().lock() else {
        return None;
    };
    let slot = guard.as_ref()?;
    if slot.stage_logical_width > 0.0 && slot.stage_logical_height > 0.0 {
        Some((slot.stage_logical_width, slot.stage_logical_height))
    } else {
        None
    }
}

pub(crate) fn try_preview_stage_transform(
    expected_revision: u64,
    target: &str,
    edit: AppStageTransformEdit,
) -> Result<HostStageGeometry, String> {
    let target = target
        .parse::<u64>()
        .map_err(|_| "The selected layer identity is invalid".to_owned())?;
    let handle = host_slot()
        .lock()
        .map_err(|_| "Stage host is unavailable".to_owned())?
        .as_ref()
        .map(|slot| slot.handle)
        .ok_or_else(|| "Stage host is unavailable".to_owned())?;
    let preview = host_preview_stage_transform_for_app(handle, expected_revision, target, edit)
        .map_err(|error| error.to_string())?;
    Ok(preview.geometry.into())
}

pub(crate) fn try_commit_stage_transform(
    expected_revision: u64,
    target: &str,
    edit: AppStageTransformEdit,
) -> Result<(), String> {
    let result = dispatch_commit_stage_transform(expected_revision, target, edit);
    if result.accepted {
        Ok(())
    } else {
        Err(result
            .feedback()
            .unwrap_or("Stage transform rejected")
            .to_owned())
    }
}

pub(crate) fn dispatch_commit_stage_transform(
    expected_revision: u64,
    target: &str,
    edit: AppStageTransformEdit,
) -> HostTerminalResult {
    let Ok(target) = target.parse::<u64>() else {
        return rejected_terminal_result(
            "invalid_layer_identity",
            "The selected layer identity is invalid".to_owned(),
        );
    };
    let Some(handle) = host_slot()
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|slot| slot.handle))
    else {
        return rejected_terminal_result(
            "host_unavailable",
            "Stage host is unavailable".to_owned(),
        );
    };
    let result = host_commit_stage_transform_for_app(handle, expected_revision, target, edit);
    let diagnostic = result.as_ref().err().map(|error| HostTerminalDiagnostic {
        reason: stage_transform_reason(error).to_owned(),
        host_handle: Some(handle.to_string()),
        stage_handle: None,
        timeline_handle: None,
        expected_projection_generation: None,
        actual_projection_generation: None,
    });
    HostTerminalResult {
        accepted: result.is_ok(),
        diagnostics: diagnostic.into_iter().collect(),
        message: result.as_ref().err().map(ToString::to_string),
        projection: try_read_timeline_projection(),
    }
}

fn rejected_terminal_result(reason: &str, message: String) -> HostTerminalResult {
    HostTerminalResult {
        accepted: false,
        diagnostics: vec![HostTerminalDiagnostic {
            reason: reason.to_owned(),
            host_handle: None,
            stage_handle: None,
            timeline_handle: None,
            expected_projection_generation: None,
            actual_projection_generation: None,
        }],
        message: Some(message),
        projection: try_read_timeline_projection(),
    }
}

fn stage_transform_reason(error: &motolii_ui::AppStageTransformError) -> &'static str {
    use motolii_ui::AppStageTransformError;
    match error {
        AppStageTransformError::HostUnavailable => "host_unavailable",
        AppStageTransformError::StaleDocument => "stale_document",
        AppStageTransformError::TargetUnavailable => "target_unavailable",
        AppStageTransformError::TransformUnavailable => "transform_unavailable",
        AppStageTransformError::OffKeyframe => "off_keyframe",
        AppStageTransformError::UnsupportedProperty => "unsupported_property",
        AppStageTransformError::NonFinite => "non_finite",
        AppStageTransformError::NoChange => "no_change",
        AppStageTransformError::Preview(_) => "preview",
        AppStageTransformError::Render(_) => "render",
        AppStageTransformError::Commit(_) => "commit",
    }
}

/// Timeline Delete: real key選択中なら remove_position_key、否則 delete_layer。
pub(crate) fn try_timeline_keymap_delete(
    scene: &crate::timeline_skia::TimelineScene,
) -> Option<HostTerminalResult> {
    if let Some(crate::timeline_skia::TimelineEditCommit::RemovePositionKey { layer_id, key_id }) =
        crate::timeline_skia::remove_position_key_commit(scene)
    {
        return try_dispatch_remove_position_key(&layer_id, key_id);
    }
    #[cfg(test)]
    TEST_KEYMAP_DELETE_LAYER_COUNT.fetch_add(1, Ordering::SeqCst);
    try_dispatch_keymap("delete_layer")
}

/// keymap: undo / redo / delete_layer(現primary=RemoveTrackItem) / duplicate(現primary) / toggle_playback。primaryなしのdelete/duplicateは何もしない。
pub(crate) fn try_dispatch_keymap(kind: &str) -> Option<HostTerminalResult> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = kind;
        None
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let intent = match kind {
            "undo" | "redo" | "toggle_playback" | "shuttle_forward" | "shuttle_reverse"
            | "shuttle_stop" => format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"{kind}","host_handle":"{}"}}"#,
                slot.handle
            ),
            "delete_layer" | "duplicate" | "split" | "mute" | "solo" | "trim_clip_in"
            | "trim_clip_out" => {
                drop(guard);
                let Some(projection) = try_read_timeline_projection() else {
                    return None;
                };
                let Some(target) = projection.primary_layer_id else {
                    return Some(HostTerminalResult {
                        accepted: true,
                        diagnostics: Vec::new(),
                        message: None,
                        projection: Some(projection),
                    });
                };
                let Ok(guard) = host_slot().lock() else {
                    return None;
                };
                let Some(slot) = guard.as_ref() else {
                    return None;
                };
                let intent = if kind == "split" || kind == "trim_clip_in" || kind == "trim_clip_out"
                {
                    let (num, den) = projection.current_time;
                    format!(
                        concat!(
                            r#"{{"version":1,"direction":"rn-to-host","kind":"{}","#,
                            r#""host_handle":"{}","target":"{}","time":{{"num":{},"den":{}}}}}"#
                        ),
                        kind, slot.handle, target, num, den
                    )
                } else {
                    format!(
                        concat!(
                            r#"{{"version":1,"direction":"rn-to-host","kind":"{}","#,
                            r#""host_handle":"{}","target":"{}"}}"#
                        ),
                        kind, slot.handle, target
                    )
                };
                return dispatch_intent_json_terminal(slot.handle, &intent);
            }
            _ => return None,
        };
        dispatch_intent_json_terminal(slot.handle, &intent)
    }
}
