//! intent 1本の match。export と pointer 選択は同じ入口の続きなので分けない。

use std::collections::{HashMap, HashSet};
#[cfg(target_os = "macos")]
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use motolii_audio::{AudioProgram, PcmCache, CANONICAL_SAMPLE_RATE};
use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, PixelSize, Quality, RationalTime};
use motolii_doc::{
    build_document_frame_graph, layer_names_for_item, prepare_set_transform_param_key_value,
    Affine2D, Clip, ClipSource, Command, DocParam, DocValue, EffectId, EvaluationTime,
    ItemEnvelope, KeyframeId, LayerId, ParentLocator, ResolvedLayerParams, ScalarPropertyId,
    TrackItem,
};
use motolii_eval::{DataTracks, Interp};
use motolii_export::{export_document_video, ExportJob, VideoSourceBinder};
use motolii_gpu::GpuCtx;
use motolii_plugins_firstparty::{first_party_catalog, first_party_runtime};
use motolii_render::{
    render_graph_cached, validate_render_graph_wiring, RenderGraphInputs, RenderSession,
};
use motolii_transport::PlaybackSession;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_os = "macos")]
use wgpu::{
    Color, CompositeAlphaMode, CurrentSurfaceTexture, LoadOp, Operations, PresentMode,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Surface, SurfaceConfiguration,
    SurfaceTargetUnsafe, TextureFormat, TextureUsages,
};

use crate::document_edit_runtime::{
    prepare_set_effect_param_command, prepare_set_source_param_command, AddPositionKeyRequest,
    AddTransformParamKeyRequest, AttachEffectRequest, DocumentEditQueue, DocumentEditRuntime,
    DocumentEditRuntimeError, PlaceEllipseRequest, PlaceMediaRequest, PlaceRectangleRequest,
    PlaceVismRequest, PublishedDocument, RemovePositionKeyRequest, SetEffectParamRequest,
    SetOpacityRequest, SetPositionConstRequest, SetPositionKeyInterpRequest,
    SetPositionKeyTimeRequest, SetPositionKeyValueRequest, SetSourceParamRequest,
};
use crate::media_library::{LibraryProjection, MediaLibrary};
use crate::shell::{open_project_runtime, ShellError};
use crate::stage_geometry_projection::{project_stage_geometry, StageLayerProjection};
use crate::stage_hit_test::{
    hit_test_projected_layers, view_local_in_stage, view_local_to_canonical, StageHit,
    StageHitTestReject,
};
use crate::stage_overlay_gpu::{overlay_dimensions_match, overlay_dirty, OverlayUploadKey};
use crate::stage_overlay_raster::{raster_selection_outline, StageOverlayRaster};
#[cfg(target_os = "macos")]
use crate::static_preview::{bootstrap_frame_desc, prepare_in_setup_worker, StaticPreview};
use crate::timeline_move_gesture::TimelineMoveRequest;
use crate::timeline_trim_gesture::TimelineTrimRequest;
use crate::{CommandId, DocumentCommandRequest, DomainIntent, InputPhase, RouterOutput};

use super::app_api::*;
use super::error::*;
#[cfg(target_os = "macos")]
use super::ffi::*;
use super::ffi_surface::*;
use super::gpu_draw::*;
use super::gpu_ops::*;
use super::gpu_surface::*;
use super::host::*;
use super::key_projection::*;
use super::playback::*;
use super::projection::*;
use super::registry::*;
use super::stage_projection::*;
use super::surfaces::*;
use super::timeline_gpu::*;
use super::wire::*;
use super::wire_io::*;
#[cfg(target_os = "macos")]
use super::MAX_PROJECT_PATH_BYTES;
use super::{
    HOST_TO_RN, MAX_DIAGNOSTICS, MAX_EFFECTS_PER_LAYER, MAX_JSON_BYTES, MAX_POSITION_KEYS,
    MAX_SNAPSHOT_JSON_BYTES, MAX_SOURCE_PARAMS_PER_LAYER, MAX_STAGE_BOUNDS, MAX_STAGE_SELECTION,
    PRODUCT_ROLE, RN_TO_HOST, WIRE_VERSION,
};

impl RnProductHost {
    pub(super) fn dispatch_intent(
        &mut self,
        host_handle: u64,
        intent: WireIntentEnvelope,
    ) -> WireIntentResponse {
        if self.destroyed {
            return reject(
                diagnostic(
                    RnHostReasonCode::DestroyedHostHandle,
                    Some(host_handle),
                    None,
                    None,
                    None,
                ),
                None,
            );
        }

        if intent.host_handle != host_handle.to_string() {
            return reject(
                diagnostic(
                    RnHostReasonCode::InvalidIntent,
                    Some(host_handle),
                    intent
                        .stage_handle
                        .as_ref()
                        .and_then(|value| value.parse().ok()),
                    None,
                    None,
                ),
                None,
            );
        }

        if let Some(expected) = intent.projection_generation.as_deref() {
            if expected != self.projection_generation.to_string() {
                return reject(
                    diagnostic(
                        RnHostReasonCode::StaleProjectionGeneration,
                        Some(host_handle),
                        intent
                            .stage_handle
                            .as_ref()
                            .and_then(|value| value.parse().ok()),
                        Some(self.projection_generation.to_string()),
                        Some(expected.to_owned()),
                    ),
                    None,
                );
            }
        }

        match intent.kind.as_str() {
            "read_snapshot" => self.accept_live_snapshot(host_handle, None),
            "toggle_playback" => self.ensure_playback(host_handle, self.playback_session.is_none()),
            "shuttle_forward" => self.ensure_playback(host_handle, true),
            "shuttle_stop" => self.ensure_playback(host_handle, false),
            "shuttle_reverse" => self.shuttle_reverse(host_handle),
            "set_time" => {
                // frame index だけを受け、Composition.fps で RationalTime へ解決する。
                // 負・duration 超過・try_from_frame 失敗は暗黙 clamp せず typed 拒否。
                let Some(frame) = intent.frame else {
                    return self.invalid_intent(host_handle);
                };
                self.seek_to_frame(host_handle, frame)
            }
            "place_rectangle" | "place_ellipse" => {
                let Some(position) = intent.position else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(playhead) = intent.playhead else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                if !position.iter().all(|value| value.is_finite()) {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                let mut queue = DocumentEditQueue::default();
                if intent.kind == "place_ellipse" {
                    queue.push_place_ellipse(PlaceEllipseRequest { position, playhead });
                } else {
                    queue.push_place_rectangle(PlaceRectangleRequest { position, playhead });
                }
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "place_vism" => {
                let Some(plugin_id) = intent.plugin_id.filter(|id| !id.is_empty()) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(position) = intent.position else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(playhead) = intent.playhead else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                if !position.iter().all(|value| value.is_finite()) {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                let mut queue = DocumentEditQueue::default();
                queue.push_place_vism(PlaceVismRequest {
                    plugin_id,
                    position,
                    playhead,
                });
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "place_media" => {
                let Some(item_id) = intent.item_id.filter(|id| !id.is_empty()) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(position) = intent.position else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(playhead) = intent.playhead else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                if !position.iter().all(|value| value.is_finite()) {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                let Some(file) = self.media_library.resolve(&item_id) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let mut queue = DocumentEditQueue::default();
                queue.push_place_media(PlaceMediaRequest {
                    path: file.path,
                    name: file.name,
                    kind: file.kind.to_owned(),
                    asset_type: file.asset_type.to_owned(),
                    position,
                    playhead,
                });
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "attach_effect" => {
                // runtime は current_primary へ attach する(document_edit_runtime AttachEffect)。
                // wire は target を明示送付し、primary 不一致は typed 拒否。
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(plugin_id) = intent.plugin_id.filter(|id| !id.is_empty()) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                if self.primary != Some(target) {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                let mut queue = DocumentEditQueue::default();
                queue.push_attach_effect(AttachEffectRequest { plugin_id });
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "preview_effect_param" => {
                let document = self.runtime.snapshot();
                let Some((request, same_as_live)) =
                    effect_param_request_from_intent(document.as_ref(), &intent)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        Some(self.snapshot_wire(host_handle)),
                    );
                };
                if same_as_live {
                    return self.accept_inspector_preview_cancel(host_handle);
                }
                let Some(command) = prepare_set_effect_param_command(document.as_ref(), &request)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        Some(self.snapshot_wire(host_handle)),
                    );
                };
                if !self.try_set_inspector_preview(command) {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::ProjectionGenerationExhausted,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        Some(self.snapshot_wire(host_handle)),
                    );
                }
                accept(self.snapshot_wire(host_handle))
            }
            "set_effect_param" => {
                // definition_id / plugin_id / effect_version は Document から解決し、呼び手に運ばせない。
                let document = self.runtime.snapshot();
                let Some((request, same_as_live)) =
                    effect_param_request_from_intent(document.as_ref(), &intent)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                if same_as_live {
                    return self.accept_inspector_preview_cancel(host_handle);
                }
                let mut queue = DocumentEditQueue::default();
                queue.push_set_effect_param(request);
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => {
                        self.clear_inspector_preview();
                        self.accept_live_snapshot(host_handle, Some(published))
                    }
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "preview_source_param" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return self.invalid_intent(host_handle);
                };
                let Some(new_value) = source_param_intent_value(&intent) else {
                    return self.invalid_intent(host_handle);
                };
                let Some(param_id) = intent.param_id.filter(|id| !id.is_empty()) else {
                    return self.invalid_intent(host_handle);
                };
                let document = self.runtime.snapshot();
                if source_param_matches_document(document.as_ref(), target, &param_id, &new_value) {
                    return self.accept_inspector_preview_cancel(host_handle);
                }
                let Some(command) = prepare_set_source_param_command(
                    document.as_ref(),
                    &SetSourceParamRequest::new(target, param_id, new_value),
                ) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        Some(self.snapshot_wire(host_handle)),
                    );
                };
                if !self.try_set_inspector_preview(command) {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::ProjectionGenerationExhausted,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                accept(self.snapshot_wire(host_handle))
            }
            "set_source_param" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return self.invalid_intent(host_handle);
                };
                let Some(new_value) = source_param_intent_value(&intent) else {
                    return self.invalid_intent(host_handle);
                };
                let Some(param_id) = intent.param_id.filter(|id| !id.is_empty()) else {
                    return self.invalid_intent(host_handle);
                };
                if source_param_matches_document(
                    self.runtime.snapshot().as_ref(),
                    target,
                    &param_id,
                    &new_value,
                ) {
                    return self.accept_inspector_preview_cancel(host_handle);
                }
                let mut queue = DocumentEditQueue::default();
                queue
                    .push_set_source_param(SetSourceParamRequest::new(target, param_id, new_value));
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => {
                        self.clear_inspector_preview();
                        self.accept_live_snapshot(host_handle, Some(published))
                    }
                    Ok(None) => self.invalid_intent(host_handle),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "set_opacity" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return self.invalid_intent(host_handle);
                };
                if self.primary != Some(target) {
                    return self.invalid_intent(host_handle);
                }
                let Some(value) = intent
                    .value
                    .filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
                else {
                    return self.invalid_intent(host_handle);
                };
                let snapshot = self.runtime.snapshot();
                let Some(envelope) = find_envelope_in_document(snapshot.as_ref(), target) else {
                    return self.invalid_intent(host_handle);
                };
                match &envelope.opacity {
                    DocParam::Const(DocValue::F64(_)) | DocParam::Keyframes(_) => {}
                    _ => return self.invalid_intent(host_handle),
                }
                let mut queue = DocumentEditQueue::default();
                queue.push_set_opacity_at(SetOpacityRequest { target, value }, self.current_time);
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "add_param_key" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return self.invalid_intent(host_handle);
                };
                let Some(time) = intent.time else {
                    return self.invalid_intent(host_handle);
                };
                let Some(property) = intent.property.as_deref().and_then(wire_transform_property)
                else {
                    return self.invalid_intent(host_handle);
                };
                if self.primary != Some(target) {
                    return self.invalid_intent(host_handle);
                }
                let mut queue = DocumentEditQueue::default();
                queue.push_add_transform_param_key(AddTransformParamKeyRequest {
                    target,
                    property,
                    time,
                });
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "set_param_key_value" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return self.invalid_intent(host_handle);
                };
                let Some(key) = intent
                    .key_id
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(KeyframeId::from_raw)
                else {
                    return self.invalid_intent(host_handle);
                };
                let Some(property) = intent.property.as_deref().and_then(wire_transform_property)
                else {
                    return self.invalid_intent(host_handle);
                };
                if self.primary != Some(target) {
                    return self.invalid_intent(host_handle);
                }
                if property == ScalarPropertyId::Opacity {
                    let Some(value) = intent.value.filter(|value| value.is_finite()) else {
                        return self.invalid_intent(host_handle);
                    };
                    let snapshot = self.runtime.snapshot();
                    let Some(time) =
                        find_envelope_in_document(snapshot.as_ref(), target).and_then(|envelope| {
                            let DocParam::Keyframes(track) = &envelope.opacity else {
                                return None;
                            };
                            track.get_by_id(key).map(|existing| existing.t)
                        })
                    else {
                        return self.invalid_intent(host_handle);
                    };
                    let mut queue = DocumentEditQueue::default();
                    queue.push_set_opacity_at(SetOpacityRequest { target, value }, time);
                    return match self.runtime.process_next(
                        &mut queue,
                        self.primary,
                        self.projection_generation,
                    ) {
                        Ok(Some(published)) => {
                            self.accept_live_snapshot(host_handle, Some(published))
                        }
                        Ok(None) => self.accept_live_snapshot(host_handle, None),
                        Err(error) => self.reject_document_runtime(host_handle, &error),
                    };
                }
                let new = match property {
                    ScalarPropertyId::Scale => {
                        let Some(new) = intent
                            .new
                            .filter(|value| value.iter().all(|component| component.is_finite()))
                        else {
                            return self.invalid_intent(host_handle);
                        };
                        DocValue::Vec2(new)
                    }
                    ScalarPropertyId::Rotation => {
                        let Some(value) = intent.value.filter(|value| value.is_finite()) else {
                            return self.invalid_intent(host_handle);
                        };
                        DocValue::F64(value)
                    }
                    _ => return self.invalid_intent(host_handle),
                };
                match prepare_set_transform_param_key_value(
                    self.runtime.snapshot().as_ref(),
                    target,
                    property,
                    key,
                    new,
                ) {
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Ok(Some(command)) => self.commit_stage_property(host_handle, command),
                    Err(_) => self.invalid_intent(host_handle),
                }
            }
            "add_position_key" | "set_position_key_value" | "set_position_key_interp" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(time) = intent.time else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let new = match intent.kind.as_str() {
                    "set_position_key_value" => {
                        let Some(new) = intent.new else {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        };
                        if !new.iter().all(|value| value.is_finite()) {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        }
                        Some(new)
                    }
                    _ => None,
                };
                let interp = if intent.kind == "set_position_key_interp" {
                    let Some(interp) = intent.interp else {
                        return reject(
                            diagnostic(
                                RnHostReasonCode::InvalidIntent,
                                Some(host_handle),
                                None,
                                None,
                                None,
                            ),
                            None,
                        );
                    };
                    Some(interp)
                } else {
                    None
                };

                if self.primary != Some(target) {
                    return self.invalid_intent(host_handle);
                }

                let mut queue = DocumentEditQueue::default();
                match intent.kind.as_str() {
                    "add_position_key" => {
                        queue.push_add_position_key(AddPositionKeyRequest { target, time });
                    }
                    "set_position_key_value" => {
                        let Some((key, old)) =
                            position_key_at(self.runtime.snapshot().as_ref(), target, time)
                        else {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        };
                        queue.push_set_position_key_value(SetPositionKeyValueRequest {
                            target,
                            key,
                            old,
                            new: new.expect("validated position value"),
                        });
                    }
                    "set_position_key_interp" => {
                        let Some((key, _)) =
                            position_key_at(self.runtime.snapshot().as_ref(), target, time)
                        else {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        };
                        queue.push_set_position_key_interp(SetPositionKeyInterpRequest {
                            target,
                            key,
                            interp: interp.expect("validated interpolation"),
                        });
                    }
                    _ => unreachable!("matched position key intent"),
                }
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "set_position_key_time" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(key) = intent
                    .key_id
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(KeyframeId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(new) = intent.time else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(old) = position_key_time_at(self.runtime.snapshot().as_ref(), target, key)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let mut queue = DocumentEditQueue::default();
                queue.push_set_position_key_time(SetPositionKeyTimeRequest {
                    target,
                    key,
                    old,
                    new,
                });
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "remove_position_key" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(key) = intent
                    .key_id
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(KeyframeId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let mut queue = DocumentEditQueue::default();
                queue.push_remove_position_key(RemovePositionKeyRequest { target, key });
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "set_clip_start" | "trim_clip_in" | "trim_clip_out" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(time) = intent.time else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let mut queue = DocumentEditQueue::default();
                match intent.kind.as_str() {
                    "set_clip_start" => {
                        queue.push_move_clip(TimelineMoveRequest {
                            layer: target,
                            new_start: time,
                        });
                    }
                    "trim_clip_in" => {
                        queue.push_trim_clip(TimelineTrimRequest::In {
                            layer: target,
                            new_start: time,
                        });
                    }
                    "trim_clip_out" => {
                        queue.push_trim_clip(TimelineTrimRequest::Out {
                            layer: target,
                            new_end: time,
                        });
                    }
                    _ => unreachable!("matched clip edit intent"),
                }
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "select_layer" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let mut queue = DocumentEditQueue::default();
                queue.push_replace_primary(target);
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Err(DocumentEditRuntimeError::SelectionTargetNotFound(_))
                    | Err(DocumentEditRuntimeError::ProjectionGenerationExhausted) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "clear_selection" => {
                let mut queue = DocumentEditQueue::default();
                queue.push_clear_primary();
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "delete_layer" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let document = self.runtime.snapshot();
                let Some((parent, index, item)) =
                    find_track_item_location(document.as_ref(), target)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Ok(layer_names) = layer_names_for_item(document.as_ref(), &item) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Ok(request) = DocumentCommandRequest::try_new(
                    DomainIntent::DeleteTargetedItems,
                    vec![Command::RemoveTrackItem {
                        parent,
                        index,
                        layer_names,
                        item,
                    }],
                ) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let output = RouterOutput::Intent {
                    phase: InputPhase::Click,
                    id: CommandId::try_new("motolii.rn.delete_layer").expect("static command id"),
                    intent: DomainIntent::DeleteTargetedItems,
                };
                let mut queue = DocumentEditQueue::default();
                if queue.push_prepared(output, Some(request)).is_err() {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "split" | "duplicate" | "mute" | "solo" | "reparent_clip" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let mut queue = DocumentEditQueue::default();
                match intent.kind.as_str() {
                    "split" => {
                        let Some(at) = intent.time else {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        };
                        queue.push_split_clip(target, at);
                    }
                    "duplicate" => queue.push_duplicate_layer(target),
                    "mute" => queue.push_toggle_visible(target),
                    "solo" => queue.push_toggle_solo(target),
                    "reparent_clip" => {
                        let Some(dest) = intent
                            .dest
                            .as_deref()
                            .and_then(|value| value.parse::<u64>().ok())
                            .map(LayerId::from_raw)
                        else {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        };
                        queue.push_reparent_clip(target, dest, intent.time);
                    }
                    _ => unreachable!("matched nle intent"),
                }
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "export_document" => self.export_document(host_handle, intent.output_path),
            "undo" | "redo" => {
                let (intent, id) = if intent.kind == "undo" {
                    (DomainIntent::Undo, "motolii.rn.undo")
                } else {
                    (DomainIntent::Redo, "motolii.rn.redo")
                };
                let output = RouterOutput::Intent {
                    phase: InputPhase::Press,
                    id: CommandId::try_new(id).expect("static command id"),
                    intent,
                };
                let mut queue = DocumentEditQueue::default();
                if queue.push_prepared(output, None).is_err() {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(DocumentEditRuntimeError::NothingToUndo)
                    | Err(DocumentEditRuntimeError::NothingToRedo) => reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    ),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "move_layer_by" => {
                let Some(target) = intent
                    .target
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(LayerId::from_raw)
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(delta) = intent.delta else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                if !delta.iter().all(|value| value.is_finite()) {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                }
                let snapshot = self.runtime.snapshot();
                let Some(envelope) = find_envelope_in_document(snapshot.as_ref(), target) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let tracks = DataTracks::new();
                let projection = match project_stage_geometry(
                    snapshot.as_ref(),
                    EvaluationTime::new(self.current_time),
                    &tracks,
                ) {
                    Ok(projection) => projection,
                    Err(_) => {
                        return reject(
                            diagnostic(
                                RnHostReasonCode::InvalidIntent,
                                Some(host_handle),
                                None,
                                None,
                                None,
                            ),
                            None,
                        );
                    }
                };
                let Some(StageLayerProjection::Available(geo)) = projection.get(target).cloned()
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(local_delta) = world_delta_to_position_local(geo.world, delta) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let mut queue = DocumentEditQueue::default();
                match &envelope.transform.position {
                    DocParam::Const(DocValue::Vec2(old)) => {
                        let new = [old[0] + local_delta[0], old[1] + local_delta[1]];
                        if !new.iter().all(|value| value.is_finite()) {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        }
                        queue.push_set_position_const(SetPositionConstRequest {
                            target,
                            old: *old,
                            new,
                        });
                    }
                    DocParam::Keyframes(_) => {
                        let Some((key, old)) =
                            position_key_at(snapshot.as_ref(), target, self.current_time)
                        else {
                            // U4b-0V: off-key は Auto Key せず typed 拒否。
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        };
                        let new = [old[0] + local_delta[0], old[1] + local_delta[1]];
                        if !new.iter().all(|value| value.is_finite()) {
                            return reject(
                                diagnostic(
                                    RnHostReasonCode::InvalidIntent,
                                    Some(host_handle),
                                    None,
                                    None,
                                    None,
                                ),
                                None,
                            );
                        }
                        queue.push_set_position_key_value(SetPositionKeyValueRequest {
                            target,
                            key,
                            old,
                            new,
                        });
                    }
                    _ => {
                        return reject(
                            diagnostic(
                                RnHostReasonCode::InvalidIntent,
                                Some(host_handle),
                                None,
                                None,
                                None,
                            ),
                            None,
                        );
                    }
                }
                match self.runtime.process_next(
                    &mut queue,
                    self.primary,
                    self.projection_generation,
                ) {
                    Ok(Some(published)) => self.accept_live_snapshot(host_handle, Some(published)),
                    Ok(None) => self.accept_live_snapshot(host_handle, None),
                    Err(error) => self.reject_document_runtime(host_handle, &error),
                }
            }
            "stage_mount" | "stage_resize" | "stage_focus" | "stage_unmount" | "stage_pointer" => {
                let Some(stage_handle) = intent
                    .stage_handle
                    .as_deref()
                    .and_then(|value| value.parse::<u64>().ok())
                else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        None,
                    );
                };
                let Some(stage) = self.stages.get_mut(&stage_handle) else {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::UnknownStageHandle,
                            Some(host_handle),
                            Some(stage_handle),
                            None,
                            None,
                        ),
                        None,
                    );
                };
                if stage.destroyed {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::LateLifecycleEvent,
                            Some(host_handle),
                            Some(stage_handle),
                            None,
                            None,
                        ),
                        None,
                    );
                }
                // unmount 後の late pointer は lifecycle と同じ late route で拒否する。
                if intent.kind == "stage_pointer" && !stage.mounted {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::LateLifecycleEvent,
                            Some(host_handle),
                            Some(stage_handle),
                            None,
                            None,
                        ),
                        None,
                    );
                }
                let payload_is_valid = match intent.kind.as_str() {
                    "stage_resize" => matches!(
                        (intent.width, intent.height, intent.scale_factor),
                        (Some(width), Some(height), Some(scale_factor))
                            if width > 0
                                && height > 0
                                && scale_factor.is_finite()
                                && scale_factor > 0.0
                    ),
                    "stage_focus" => intent.focused.is_some(),
                    "stage_mount" | "stage_unmount" => true,
                    "stage_pointer" => {
                        let phase_ok = matches!(
                            intent.phase.as_deref(),
                            Some("down" | "drag" | "up" | "cancel")
                        );
                        let coords_ok = matches!(
                            (intent.view_local_x, intent.view_local_y),
                            (Some(x), Some(y)) if x.is_finite() && y.is_finite()
                        );
                        phase_ok && coords_ok && intent.sequence.is_some()
                    }
                    _ => false,
                };
                if !payload_is_valid {
                    return reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            Some(stage_handle),
                            None,
                            None,
                        ),
                        None,
                    );
                }
                // pointer の selection は stage borrow 解放後に行う。
                let pointer_down = match intent.kind.as_str() {
                    "stage_mount" => {
                        stage.mounted = true;
                        None
                    }
                    "stage_resize" => {
                        stage.width = intent.width.expect("validated resize width");
                        stage.height = intent.height.expect("validated resize height");
                        stage.scale_factor =
                            intent.scale_factor.expect("validated resize scale factor");
                        #[cfg(target_os = "macos")]
                        {
                            stage.gpu.overlay_upload_key = None;
                        }
                        None
                    }
                    "stage_focus" => {
                        stage.focused = intent.focused.expect("validated focus state");
                        None
                    }
                    "stage_unmount" => {
                        #[cfg(target_os = "macos")]
                        stage.gpu_detach_surface();
                        stage.mounted = false;
                        None
                    }
                    "stage_pointer" => {
                        // selection 成否と独立に transient を先に記録する（grain 2）。
                        let phase = intent.phase.expect("validated pointer phase");
                        let view_local_x = intent.view_local_x.expect("validated view_local_x");
                        let view_local_y = intent.view_local_y.expect("validated view_local_y");
                        let sequence = intent.sequence.expect("validated pointer sequence");
                        let width = stage.width;
                        let height = stage.height;
                        stage.pointer = Some(StagePointerTransient {
                            phase: phase.clone(),
                            view_local_x,
                            view_local_y,
                            sequence,
                        });
                        if phase == "down" {
                            Some((view_local_x, view_local_y, width, height))
                        } else {
                            // drag / up / cancel は selection を変更しない。
                            None
                        }
                    }
                    _ => None,
                };
                if let Some((view_local_x, view_local_y, width, height)) = pointer_down {
                    if let Some(response) = self.apply_stage_pointer_selection(
                        host_handle,
                        stage_handle,
                        view_local_x,
                        view_local_y,
                        width,
                        height,
                    ) {
                        return response;
                    }
                }
                accept(self.snapshot_wire(host_handle))
            }
            _ => reject(
                diagnostic(
                    RnHostReasonCode::InvalidIntent,
                    Some(host_handle),
                    intent
                        .stage_handle
                        .as_ref()
                        .and_then(|value| value.parse().ok()),
                    None,
                    None,
                ),
                None,
            ),
        }
    }

    pub(super) fn export_document(
        &mut self,
        host_handle: u64,
        output_path: Option<String>,
    ) -> WireIntentResponse {
        let snapshot = self.snapshot_wire(host_handle);
        let Some(output_path) = output_path.filter(|path| !path.trim().is_empty()) else {
            return with_message(
                reject(
                    diagnostic(
                        RnHostReasonCode::InvalidIntent,
                        Some(host_handle),
                        None,
                        None,
                        None,
                    ),
                    Some(snapshot),
                ),
                "output path is required",
            );
        };
        let output_path = PathBuf::from(output_path.trim());
        if self.stage_frame_runtime.is_none() {
            match first_party_runtime() {
                Ok(runtime) => self.stage_frame_runtime = Some(Arc::new(runtime)),
                Err(error) => {
                    return with_message(
                        reject(
                            diagnostic(
                                RnHostReasonCode::InvalidIntent,
                                Some(host_handle),
                                None,
                                None,
                                None,
                            ),
                            Some(snapshot),
                        ),
                        error.to_string(),
                    );
                }
            }
        }
        let plugin_runtime = self
            .stage_frame_runtime
            .as_ref()
            .expect("stage_frame_runtime initialized")
            .clone();
        let document = self.runtime.snapshot();
        let project_root = self.runtime.project_root();
        let gpu = match GpuCtx::new_headless() {
            Ok(gpu) => gpu,
            Err(error) => {
                return with_message(
                    reject(
                        diagnostic(
                            RnHostReasonCode::InvalidIntent,
                            Some(host_handle),
                            None,
                            None,
                            None,
                        ),
                        Some(snapshot),
                    ),
                    error.to_string(),
                );
            }
        };
        match export_document_video(
            &gpu,
            &ExportJob {
                doc: document.as_ref(),
                runtime: plugin_runtime.as_ref(),
                output_path: &output_path,
                project_root: project_root.as_deref(),
                frame_count: None,
                qp0: false,
                data_tracks: DataTracks::new(),
            },
        ) {
            Ok(report) => with_message(
                accept(snapshot),
                format!(
                    "wrote {} frames to {}",
                    report.frames_written,
                    output_path.display()
                ),
            ),
            Err(error) => with_message(
                reject(
                    diagnostic(
                        RnHostReasonCode::InvalidIntent,
                        Some(host_handle),
                        None,
                        None,
                        None,
                    ),
                    Some(snapshot),
                ),
                error.to_string(),
            ),
        }
    }

    /// `stage_pointer` down の hit-test → 既存 selection writer。
    /// typed 拒否時だけ `Some(reject)`。受理・no-op は `None`（呼び出し側が snapshot を返す）。
    pub(super) fn apply_stage_pointer_selection(
        &mut self,
        host_handle: u64,
        stage_handle: u64,
        view_local_x: f64,
        view_local_y: f64,
        width: u32,
        height: u32,
    ) -> Option<WireIntentResponse> {
        let canonical = match view_local_to_canonical(view_local_x, view_local_y, width, height) {
            Ok(point) => point,
            Err(StageHitTestReject::ZeroStageExtent) => {
                return Some(reject(
                    diagnostic(
                        RnHostReasonCode::InvalidIntent,
                        Some(host_handle),
                        Some(stage_handle),
                        None,
                        None,
                    ),
                    None,
                ));
            }
        };

        // product path と同じ空 DataTracks。runtime に格納口は無い。
        let tracks = DataTracks::new();
        let document = self.runtime.snapshot();
        let projection = match project_stage_geometry(
            document.as_ref(),
            EvaluationTime::new(self.current_time),
            &tracks,
        ) {
            Ok(projection) => projection,
            Err(_) => {
                // 幾何失敗を選択解除の意思に読み替えない。
                return Some(reject(
                    diagnostic(
                        RnHostReasonCode::InvalidIntent,
                        Some(host_handle),
                        Some(stage_handle),
                        None,
                        None,
                    ),
                    None,
                ));
            }
        };

        let hit = if view_local_in_stage(view_local_x, view_local_y, width, height) {
            hit_test_projected_layers(canonical, &projection)
        } else {
            StageHit::Miss
        };

        let mut queue = DocumentEditQueue::default();
        match hit {
            StageHit::Layer(layer) => queue.push_replace_primary(layer),
            StageHit::Miss => queue.push_clear_primary(),
        }

        match self
            .runtime
            .process_next(&mut queue, self.primary, self.projection_generation)
        {
            Ok(None) => {
                // 存在拒否以外の same-id / already-clear no-op。generation は進めない。
                None
            }
            Ok(Some(published)) => {
                // accepted 変更だけを Host transient へ反映する（直接代入で意図を捏造しない）。
                self.adopt_published(published);
                None
            }
            Err(error) => Some(self.reject_document_runtime(host_handle, &error)),
        }
    }
}
