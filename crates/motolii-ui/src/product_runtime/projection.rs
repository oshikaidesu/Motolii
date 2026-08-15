//! Stage表示世代とTimeline投影。range stateはDocument envelopeが正本。

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use motolii_audio::{AudioProgram, PcmCache, CANONICAL_SAMPLE_RATE};
use motolii_core::{CanonicalPoint, Fps, FpsError, Quality, RationalTime, RationalTimeError};
use motolii_doc::{
    Command, DocParam, DocValue, EffectId, EvaluationTime, KeyframeId, LayerId, TrackItem,
};
use motolii_eval::DataTracks;
use motolii_gpu::GpuCtx;
use motolii_transport::{FramePlan, PlaybackSession, PlaybackSessionError, TransportError};
use winit::dpi::LogicalSize;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::Window;

use crate::app::canonical_drop_from_ndc;
use crate::browser_host::BrowserPlaceIntent;
use crate::browser_host_runtime::{
    BrowserFocusTarget, BrowserHostRuntime, BrowserHostRuntimeError, BrowserLifecycleEvent,
};
use crate::document_edit_runtime::{
    AddPositionKeyRequest, AttachEffectRequest, DocumentEditDispatchError, DocumentEditQueue,
    DocumentEditRuntime, DocumentEditRuntimeError, PlaceRectangleRequest, PublishedDocument,
    SetPositionKeyInterpRequest, SetPositionKeyValueRequest,
};
use crate::host_pointer_capture::{HostPointerCancel, HostPointerCandidate};
use crate::inspector_host_runtime::{
    resolve_effect_param_preview_command, InspectorGestureTerminal, InspectorGestureTerminalCause,
    InspectorHostRuntime, InspectorHostRuntimeError, InspectorPositionAxis,
    InspectorPositionGestureStart, InspectorPositionGestureTerminal,
    InspectorPositionGestureTerminalCause,
};
use crate::layout_authority::LayoutAuthority;
use crate::native_host_layout::{
    key_tools_logical_rect, timeline_ruler_logical_rect, timeline_time_surface_logical_rect,
    LogicalRect, NativeHostLayout, PhysicalRect,
};
use crate::product_easing_popup::{
    PopupTerminal, ProductEasingPopup, ProductEasingPopupError, ProductEasingPopupOpen,
};
use crate::render_worker::{
    RenderGeneration, RenderRequest, RenderWorker, RenderWorkerClient, RenderWorkerError,
};
use crate::stage_chrome_host_runtime::{
    StageChromeHostRuntime, StageChromeHostRuntimeError, StageEasingIntent, StageEasingIntentError,
    StagePlaybackIntentError, StagePlaybackState, StagePlaybackToggle, StageTransportSnapshot,
};
use crate::static_preview::{bootstrap_frame_desc, prepare_in_setup_worker, StaticPreview};
use crate::timeline_move_gesture::{TimelineMoveGesture, TimelineMoveRequest};
use crate::timeline_projection::{
    project_timeline, TimelineHit, TimelineMetrics, TimelineProjection, TimelineProjectionError,
    TimelineViewport,
};
use crate::timeline_tools_host_runtime::{TimelineToolsHostRuntime, TimelineToolsHostRuntimeError};
use crate::timeline_trim_gesture::{TimelineTrimEdge, TimelineTrimGesture};
use crate::{
    builtin_command_registry, default_user_keymap_override_path, load_user_keymap_override,
    product_builtin_keymap, resolve_keymap, CommandIdError, CommandRegistry, CommandRegistryError,
    DomainIntent, EffectiveTrigger, ImeGateState, InputPhase, InputRouter, InputRouterError,
    KeyToken, KeymapResolution, ModifierError, Modifiers, NormalizedInput,
    PlatformBindingConstraints, PlatformCommandModifier, RouterOutput, SafetyInterrupt,
};

#[derive(Debug, Clone)]
pub(super) struct PendingStageDrop {
    pub(super) source: BrowserPlaceIntent,
    pub(super) generation: u64,
    pub(super) layout_epoch: u64,
    pub(super) ndc: [f64; 2],
}

#[derive(Debug, Default)]
pub(super) struct ProductStageProjection {
    pub(super) last_displayed_generation: Option<RenderGeneration>,
}

impl ProductStageProjection {
    pub(super) fn accepts(
        &self,
        result_generation: RenderGeneration,
        latest_accepted_generation: Option<RenderGeneration>,
    ) -> bool {
        Some(result_generation) == latest_accepted_generation
            && self
                .last_displayed_generation
                .is_none_or(|displayed| result_generation > displayed)
    }

    pub(super) fn commit(&mut self, generation: RenderGeneration) {
        self.last_displayed_generation = Some(generation);
    }
}

#[derive(Debug, Clone)]
pub(super) struct ProductTimelineProjection {
    pub(super) projection: TimelineProjection,
    pub(super) band_span: f64,
    pub(super) composition_duration: RationalTime,
    pub(super) preview: Option<TimelineProjection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProductTimelineHit {
    Key { layer: LayerId, key: KeyframeId },
    Left { layer: LayerId },
    Right { layer: LayerId },
    Body { layer: LayerId },
    None,
}

impl ProductTimelineProjection {
    pub(super) fn from_document(
        document: &motolii_doc::Document,
    ) -> Result<Self, TimelineProjectionError> {
        let duration_seconds = document.composition.duration.as_seconds_f64();
        let projection = project_timeline(
            document,
            &TimelineMetrics {
                band_height: 1.0,
                units_per_second: duration_seconds.recip(),
                key_half_extent: 1.0,
            },
            &TimelineViewport {
                start: RationalTime::ZERO,
                end: document.composition.duration,
            },
        )?;
        let band_span = projection
            .bars()
            .iter()
            .map(|bar| bar.y_bottom)
            .fold(1.0, f64::max);
        Ok(Self {
            projection,
            band_span,
            composition_duration: document.composition.duration,
            preview: None,
        })
    }

    pub(super) fn render_projection(&self) -> &TimelineProjection {
        self.preview.as_ref().unwrap_or(&self.projection)
    }

    pub(super) fn set_move_preview(&mut self, request: Option<TimelineMoveRequest>) {
        self.preview = request.and_then(|request| {
            self.projection.preview_move(
                request.layer,
                request.new_start,
                self.composition_duration,
            )
        });
    }

    pub(super) fn clear_move_preview(&mut self) {
        self.preview = None;
    }

    pub(super) fn set_trim_preview(
        &mut self,
        document: Option<&motolii_doc::Document>,
    ) -> Result<(), TimelineProjectionError> {
        self.preview = document
            .map(Self::from_document)
            .transpose()?
            .map(|projection| projection.projection);
        Ok(())
    }

    pub(super) fn hit_test(
        &self,
        position: [f64; 2],
        layout: NativeHostLayout,
    ) -> Option<ProductTimelineHit> {
        self.hit_test_pair(position, layout)
            .map(|(_, private_hit)| private_hit)
    }

    pub(super) fn hit_test_pair(
        &self,
        position: [f64; 2],
        layout: NativeHostLayout,
    ) -> Option<(TimelineHit, ProductTimelineHit)> {
        let time_surface = timeline_time_surface_logical_rect(layout)?;
        if !position.iter().all(|value| value.is_finite())
            || !time_surface.x.is_finite()
            || !time_surface.y.is_finite()
            || !time_surface.width.is_finite()
            || time_surface.width <= 0.0
            || !time_surface.height.is_finite()
            || time_surface.height <= 0.0
            || !self.band_span.is_finite()
            || self.band_span <= 0.0
            || !time_surface.contains(position)
        {
            return None;
        }
        let x = (position[0] - time_surface.x) / time_surface.width;
        let y = ((position[1] - time_surface.y) / time_surface.height) * self.band_span;
        let public_hit = self.render_projection().hit_test(x, y);
        let private_hit = match public_hit {
            TimelineHit::Key { layer, key } => ProductTimelineHit::Key { layer, key },
            TimelineHit::None => ProductTimelineHit::None,
            TimelineHit::Bar { layer } => {
                let Some(bar) = self
                    .render_projection()
                    .bars()
                    .iter()
                    .find(|bar| bar.layer == layer)
                else {
                    return Some((public_hit, ProductTimelineHit::Body { layer }));
                };
                let bar_width = (bar.x_end - bar.x_start) * time_surface.width;
                let bar_height = time_surface.height / self.band_span;
                if !bar.x_start.is_finite()
                    || !bar.x_end.is_finite()
                    || !bar_width.is_finite()
                    || bar_width < 25.0
                    || !bar_height.is_finite()
                    || bar_height < 16.0
                {
                    ProductTimelineHit::Body { layer }
                } else {
                    let bar_left = time_surface.x + bar.x_start * time_surface.width;
                    let local_x = position[0] - bar_left;
                    let edge_width = 15.0_f64.min(bar_width / 4.0);
                    if !bar_left.is_finite() || !local_x.is_finite() {
                        ProductTimelineHit::Body { layer }
                    } else if local_x <= edge_width {
                        ProductTimelineHit::Left { layer }
                    } else if local_x >= bar_width - edge_width {
                        ProductTimelineHit::Right { layer }
                    } else {
                        ProductTimelineHit::Body { layer }
                    }
                }
            }
        };
        Some((public_hit, private_hit))
    }

    pub(super) fn time_at(
        &self,
        position: [f64; 2],
        layout: NativeHostLayout,
    ) -> Option<RationalTime> {
        let time_surface = timeline_time_surface_logical_rect(layout)?;
        if !position.iter().all(|value| value.is_finite()) || !time_surface.contains(position) {
            return None;
        }
        let normalized = (position[0] - time_surface.x) / time_surface.width;
        if !normalized.is_finite() {
            return None;
        }
        let fraction = RationalTime::try_from_decimal_str(&format!("{normalized:.9}")).ok()?;
        self.composition_duration.try_mul(fraction).ok()
    }

    pub(super) fn ruler_time_at(
        &self,
        position: [f64; 2],
        layout: NativeHostLayout,
        require_ruler_hit: bool,
    ) -> Option<RationalTime> {
        let ruler = timeline_ruler_logical_rect(layout)?;
        if !position.iter().all(|value| value.is_finite())
            || !ruler.x.is_finite()
            || !ruler.y.is_finite()
            || !ruler.width.is_finite()
            || ruler.width <= 0.0
            || !ruler.height.is_finite()
            || ruler.height <= 0.0
        {
            return None;
        }
        let right = ruler.x + ruler.width;
        let bottom = ruler.y + ruler.height;
        if !right.is_finite()
            || !bottom.is_finite()
            || (require_ruler_hit
                && (position[0] < ruler.x
                    || position[0] > right
                    || position[1] < ruler.y
                    || position[1] >= bottom))
        {
            return None;
        }
        let normalized = ((position[0] - ruler.x) / ruler.width).clamp(0.0, 1.0);
        let fraction = RationalTime::try_from_decimal_str(&format!("{normalized:.9}")).ok()?;
        self.composition_duration.try_mul(fraction).ok()
    }
}
