//! Document から wire/App 投影。timeline/stage/key を同じ投影口に置く。

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
use super::dispatch::*;
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

pub(super) fn snapshot_for_test(snapshot: WireProductSnapshot) -> RnProductSnapshotForTest {
    RnProductSnapshotForTest {
        revision: snapshot.revision,
        projection_generation: snapshot.projection_generation,
        current_time: snapshot.current_time,
        primary_layer_id: snapshot.primary_layer_id,
        layer_ids: snapshot
            .stage
            .bounds
            .into_iter()
            .map(|bound| bound.layer_id)
            .collect(),
        timeline: RnTimelineProjectionForTest {
            fps: snapshot.timeline.fps,
            layers: snapshot
                .timeline
                .layers
                .into_iter()
                .map(|layer| RnTimelineLayerForTest {
                    layer_id: layer.layer_id,
                    display_name: layer.display_name,
                    start: layer.start,
                    duration: layer.duration,
                    position_keys: layer
                        .position_keys
                        .into_iter()
                        .map(|key| RnTimelinePositionKeyForTest {
                            key_id: key.key_id,
                            time: key.time,
                            value: key.value,
                            interp: key.interp,
                        })
                        .collect(),
                    param_keys: layer
                        .param_keys
                        .into_iter()
                        .map(|key| RnTimelineParamKeyForTest {
                            property: key.property,
                            key_id: key.key_id,
                            time: key.time,
                            value: key.value,
                            vec: key.vec,
                        })
                        .collect(),
                    keys_truncated: layer.keys_truncated,
                })
                .collect(),
            layers_truncated: snapshot.timeline.layers_truncated,
        },
    }
}

pub(super) fn project_timeline(document: &motolii_doc::Document) -> (WireTimelineProjection, u32) {
    let ordered = layers_in_track_order(document);
    let mut truncated_total: u32 = 0;
    if ordered.len() > MAX_STAGE_BOUNDS {
        truncated_total = truncated_total
            .saturating_add(u32::try_from(ordered.len() - MAX_STAGE_BOUNDS).unwrap_or(u32::MAX));
    }
    let layers_truncated = ordered.len() > MAX_STAGE_BOUNDS;
    let layers = ordered
        .into_iter()
        .take(MAX_STAGE_BOUNDS)
        .map(|(layer_id, name)| {
            let (start, duration, visible, solo) = find_first_clip(document, layer_id)
                .map(|clip| {
                    (
                        clip.start,
                        clip.duration,
                        clip.envelope.visible,
                        clip.envelope.solo,
                    )
                })
                .unwrap_or((RationalTime::ZERO, RationalTime::ZERO, true, false));
            let (position_keys, keys_truncated, keys_hidden) =
                project_position_keys(document, layer_id);
            let param_keys = project_param_keys(document, layer_id);
            let (effects, effects_truncated, effects_hidden) =
                project_layer_effects(document, layer_id);
            let (source_params, source_params_truncated, source_hidden) =
                project_layer_source_params(document, layer_id);
            truncated_total = truncated_total
                .saturating_add(keys_hidden)
                .saturating_add(effects_hidden)
                .saturating_add(source_hidden);
            WireTimelineLayer {
                layer_id: layer_id.get().to_string(),
                display_name: name,
                start,
                duration,
                position_keys,
                param_keys,
                keys_truncated,
                effects,
                effects_truncated,
                source_params,
                source_params_truncated,
                visible,
                solo,
            }
        })
        .collect();
    (
        WireTimelineProjection {
            fps: document.composition.fps,
            duration: document.composition.duration,
            layers,
            layers_truncated,
        },
        truncated_total,
    )
}

/// LayerIdTable採番順ではなく Document.tracks の track→item 順。
/// Groupは半対応(自身+childrenを順に列挙)。
pub(super) fn layers_in_track_order(document: &motolii_doc::Document) -> Vec<(LayerId, String)> {
    fn walk(
        items: &[TrackItem],
        document: &motolii_doc::Document,
        out: &mut Vec<(LayerId, String)>,
    ) {
        for item in items {
            match item {
                TrackItem::Clip(clip) => {
                    let id = clip.envelope.layer_id;
                    if let Some(name) = document.layers.display_name(id) {
                        out.push((id, name.to_owned()));
                    }
                }
                TrackItem::Group(group) => {
                    let id = group.envelope.layer_id;
                    if let Some(name) = document.layers.display_name(id) {
                        out.push((id, name.to_owned()));
                    }
                    walk(&group.children, document, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    for track in &document.tracks {
        walk(&track.items, document, &mut out);
    }
    out
}

pub(super) fn project_layer_effects(
    document: &motolii_doc::Document,
    layer_id: LayerId,
) -> (Vec<WireTimelineEffect>, bool, u32) {
    let Some(envelope) = find_envelope_in_document(document, layer_id) else {
        return (Vec::new(), false, 0);
    };
    let hidden = envelope.effects.len().saturating_sub(MAX_EFFECTS_PER_LAYER) as u32;
    let effects_truncated = hidden > 0;
    let effects = envelope
        .effects
        .iter()
        .take(MAX_EFFECTS_PER_LAYER)
        .filter_map(|effect_use| {
            let definition = document.effect_definition(effect_use.definition_id)?;
            let params = definition
                .params
                .iter()
                .filter_map(|(param_id, param)| match param {
                    DocParam::Const(DocValue::F64(value)) if value.is_finite() => {
                        Some(WireTimelineEffectParam {
                            param_id: param_id.clone(),
                            value: *value,
                            color: None,
                        })
                    }
                    DocParam::Const(DocValue::Color(color))
                        if color.iter().all(|component| component.is_finite()) =>
                    {
                        Some(WireTimelineEffectParam {
                            param_id: param_id.clone(),
                            value: 0.0,
                            color: Some(*color),
                        })
                    }
                    _ => None,
                })
                .collect();
            Some(WireTimelineEffect {
                effect_use_id: effect_use.id.get().to_string(),
                plugin_id: definition.plugin_id.clone(),
                params,
            })
        })
        .collect();
    (effects, effects_truncated, hidden)
}

pub(super) fn project_layer_source_params(
    document: &motolii_doc::Document,
    layer_id: LayerId,
) -> (Vec<WireTimelineSourceParam>, bool, u32) {
    let Some(clip) = find_first_clip(document, layer_id) else {
        return (Vec::new(), false, 0);
    };
    let ClipSource::Plugin { params, .. } = &clip.source else {
        return (Vec::new(), false, 0);
    };
    let all: Vec<_> = params
        .iter()
        .filter_map(|(param_id, param)| match param {
            DocParam::Const(DocValue::F64(value)) if value.is_finite() => {
                Some(WireTimelineSourceParam {
                    param_id: param_id.clone(),
                    value: *value,
                    color: None,
                })
            }
            DocParam::Const(DocValue::Color(color)) if color.iter().all(|c| c.is_finite()) => {
                Some(WireTimelineSourceParam {
                    param_id: param_id.clone(),
                    value: 0.0,
                    color: Some(*color),
                })
            }
            _ => None,
        })
        .collect();
    let hidden = all.len().saturating_sub(MAX_SOURCE_PARAMS_PER_LAYER) as u32;
    let source_params_truncated = hidden > 0;
    let source_params = all.into_iter().take(MAX_SOURCE_PARAMS_PER_LAYER).collect();
    (source_params, source_params_truncated, hidden)
}

pub(super) fn const_f64_param(param: &DocParam) -> Option<f64> {
    match param {
        DocParam::Const(DocValue::F64(value)) if value.is_finite() => Some(*value),
        _ => None,
    }
}

pub(super) fn source_param_intent_value(intent: &WireIntentEnvelope) -> Option<DocParam> {
    if let Some(color) = intent.color {
        if color.iter().all(|component| component.is_finite()) {
            return Some(DocParam::const_color(color));
        }
        return None;
    }
    intent
        .value
        .filter(|value| value.is_finite())
        .map(DocParam::const_f64)
}

pub(super) fn source_param_matches_document(
    document: &motolii_doc::Document,
    target: LayerId,
    param_id: &str,
    value: &DocParam,
) -> bool {
    let Some(clip) = find_first_clip(document, target) else {
        return false;
    };
    let ClipSource::Plugin { params, .. } = &clip.source else {
        return false;
    };
    params.get(param_id) == Some(value)
}

pub(super) fn effect_param_request_from_intent(
    document: &motolii_doc::Document,
    intent: &WireIntentEnvelope,
) -> Option<(SetEffectParamRequest, bool)> {
    let target = intent
        .target
        .as_deref()
        .and_then(|value| value.parse::<u64>().ok())
        .map(LayerId::from_raw)?;
    let effect_use_id = intent
        .effect_use_id
        .as_deref()
        .and_then(|value| value.parse::<u64>().ok())
        .map(EffectId::from_raw)?;
    let param_id = intent.param_id.clone().filter(|id| !id.is_empty())?;
    let new_value = source_param_intent_value(intent)?;
    let effect_use = document.find_effect_use(target, effect_use_id)?;
    let definition = document.effect_definition(effect_use.definition_id)?;
    let same_as_live = definition.params.get(&param_id) == Some(&new_value);
    Some((
        SetEffectParamRequest::with_param(
            target,
            effect_use_id,
            effect_use.definition_id,
            definition.plugin_id.clone(),
            definition.effect_version,
            param_id,
            new_value,
        ),
        same_as_live,
    ))
}

pub(super) fn project_selected_doc_params(
    document: &motolii_doc::Document,
    primary: Option<LayerId>,
    time: RationalTime,
) -> Option<WireSelectedDocParams> {
    let primary = primary?;
    let envelope = find_envelope_in_document(document, primary)?;
    let (effects, _, _) = project_layer_effects(document, primary);
    let (source_params, _, _) = project_layer_source_params(document, primary);
    Some(WireSelectedDocParams {
        layer_id: primary.get().to_string(),
        opacity: motolii_doc::param_eval::eval_f64(
            &envelope.opacity,
            time,
            &DataTracks::new(),
            &ResolvedLayerParams::default(),
        )
        .ok()
        .filter(|value| value.is_finite()),
        effects,
        source_params,
    })
}

/// first_party − reference。session 不変なので OnceLock で cache。
pub(super) fn wire_catalog_projection() -> WireCatalogProjection {
    static CACHE: OnceLock<WireCatalogProjection> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let catalog = first_party_catalog().expect("first-party catalog");
            let reference =
                motolii_plugin::reference::reference_catalog().expect("reference catalog");
            let mut effects = catalog
                .iter()
                .filter(|(plugin_id, contract)| {
                    contract.kind == motolii_plugin::PluginKind::Filter
                        && reference.get(plugin_id.0).is_none()
                })
                .map(|(plugin_id, contract)| {
                    let name = if contract.node.display_name.trim().is_empty() {
                        plugin_id.0.to_owned()
                    } else {
                        contract.node.display_name.to_owned()
                    };
                    WireCatalogEffect {
                        plugin_id: plugin_id.0.to_owned(),
                        name,
                        effect_version: contract.node.version,
                    }
                })
                .collect::<Vec<_>>();
            effects.sort_by(|a, b| a.plugin_id.cmp(&b.plugin_id));
            let mut sources = catalog
                .iter()
                .filter(|(plugin_id, contract)| {
                    contract.kind == motolii_plugin::PluginKind::LayerSource
                        && reference.get(plugin_id.0).is_none()
                })
                .map(|(plugin_id, contract)| {
                    let name = if contract.node.display_name.trim().is_empty() {
                        plugin_id.0.to_owned()
                    } else {
                        contract.node.display_name.to_owned()
                    };
                    WireCatalogSource {
                        plugin_id: plugin_id.0.to_owned(),
                        name,
                        effect_version: contract.node.version,
                    }
                })
                .collect::<Vec<_>>();
            sources.sort_by(|a, b| a.plugin_id.cmp(&b.plugin_id));
            WireCatalogProjection { effects, sources }
        })
        .clone()
}

pub(super) fn wire_library_projection(projection: LibraryProjection) -> WireLibraryProjection {
    WireLibraryProjection {
        root: projection.root.map(|root| WireLibraryRoot {
            id: root.id,
            name: root.name,
            path: root.path.to_string_lossy().into_owned(),
        }),
        directories: projection
            .directories
            .into_iter()
            .map(|directory| WireLibraryDirectory {
                id: directory.id,
                name: directory.name,
                path: directory.path,
            })
            .collect(),
        tags: projection
            .tags
            .into_iter()
            .map(|tag| WireLibraryTag {
                id: tag.id,
                label: tag.label,
                count: tag.count,
            })
            .collect(),
        items: projection
            .items
            .into_iter()
            .map(|item| WireLibraryItem {
                id: item.id,
                name: item.name,
                kind: item.kind.to_owned(),
                directory: item.directory,
                tags: item.tags,
            })
            .collect(),
    }
}

pub(super) fn response_for_test(response: WireIntentResponse) -> RnHostTestResponse {
    RnHostTestResponse {
        accepted: response.accepted,
        reason: response
            .diagnostics
            .first()
            .map(|diagnostic| diagnostic.reason),
        snapshot: response.snapshot.map(snapshot_for_test),
        message: response.message,
    }
}
