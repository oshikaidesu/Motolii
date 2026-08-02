//! 確定済みStage React chromeをnative viewportの上下へ載せるprivate Host。

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use wry::{Rect, WebView, WebViewBuilder};

use crate::browser_host_runtime::product_asset_response;
use crate::native_host_layout::LogicalRect;
use crate::timeline_projection::project_position_interval;
use motolii_core::RationalTime;

const PROTOCOL: &str = "motolii-stage";
const HEADER_URL: &str = "motolii-stage://product/stage-header.html";
const TRANSPORT_URL: &str = "motolii-stage://product/stage-transport.html";
const MAX_PENDING_EASING_OPENS: usize = 8;
const MAX_PENDING_PLAYBACK_TOGGLES: usize = 8;
const MAX_PENDING_PROJECT_ACTIONS: usize = 8;

type StageChromeWake = Arc<dyn Fn() + Send + Sync>;

#[derive(Debug, Clone, Copy)]
pub(crate) struct StageChromeStatus<'a> {
    pub(crate) project: &'a str,
    pub(crate) export: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StageChromeProjection<'a> {
    pub(crate) easing_pressed: bool,
    pub(crate) playing: bool,
    pub(crate) status: StageChromeStatus<'a>,
}

impl<'a> StageChromeProjection<'a> {
    pub(crate) const fn new(
        easing_pressed: bool,
        playing: bool,
        status: StageChromeStatus<'a>,
    ) -> Self {
        Self {
            easing_pressed,
            playing,
            status,
        }
    }
}

impl<'a> StageChromeStatus<'a> {
    pub(crate) const fn new(project: &'a str, export: &'a str) -> Self {
        Self { project, export }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct OpenEasingRequest {
    pub(crate) layer_id: motolii_doc::LayerId,
    pub(crate) left_key_id: motolii_doc::KeyframeId,
    pub(crate) right_key_id: motolii_doc::KeyframeId,
    pub(crate) projection_generation: u64,
    pub(crate) layout_epoch: u64,
    pub(crate) anchor: LogicalRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OpenEasingIdentity {
    layer_id: motolii_doc::LayerId,
    left_key_id: motolii_doc::KeyframeId,
    right_key_id: motolii_doc::KeyframeId,
    projection_generation: u64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenEasingMessage {
    #[serde(rename = "kind")]
    _kind: OpenEasingKind,
    #[serde(rename = "layerId")]
    layer_id: motolii_doc::LayerId,
    #[serde(rename = "leftKeyId")]
    left_key_id: motolii_doc::KeyframeId,
    #[serde(rename = "rightKeyId")]
    right_key_id: motolii_doc::KeyframeId,
    #[serde(rename = "projectionGeneration")]
    projection_generation: u64,
    #[serde(rename = "layoutEpoch")]
    layout_epoch: u64,
    anchor: OpenEasingAnchor,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenEasingAnchor {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Debug, serde::Deserialize)]
enum OpenEasingKind {
    #[serde(rename = "open-easing")]
    OpenEasing,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TogglePlaybackMessage {
    #[serde(rename = "kind")]
    _kind: TogglePlaybackKind,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectActionMessage {
    #[serde(rename = "kind")]
    _kind: ProjectActionKind,
}

#[derive(Debug, serde::Deserialize)]
enum ProjectActionKind {
    #[serde(rename = "save-project")]
    SaveProject,
    #[serde(rename = "export-project")]
    ExportProject,
}

#[derive(Debug, serde::Deserialize)]
enum TogglePlaybackKind {
    #[serde(rename = "toggle-play")]
    TogglePlay,
}

#[derive(Debug)]
struct TogglePlaybackInbox {
    pending: VecDeque<()>,
}

impl TogglePlaybackInbox {
    fn new() -> Self {
        Self {
            pending: VecDeque::new(),
        }
    }

    fn accept(&mut self, raw: &str) -> Result<(), StageChromeIntentError> {
        let _: TogglePlaybackMessage = serde_json::from_str(raw)?;
        if self.pending.len() >= MAX_PENDING_PLAYBACK_TOGGLES {
            return Err(StageChromeIntentError::PlaybackInboxFull);
        }
        self.pending.push_back(());
        Ok(())
    }

    fn take(&mut self) -> bool {
        self.pending.pop_front().is_some()
    }
}

#[derive(Debug, Default)]
struct ProjectActionInbox {
    pending: VecDeque<()>,
}

impl ProjectActionInbox {
    fn accept(&mut self, raw: &str) -> Result<(), StageChromeIntentError> {
        let _: ProjectActionMessage = serde_json::from_str(raw)?;
        if self.pending.len() >= MAX_PENDING_PROJECT_ACTIONS {
            return Err(StageChromeIntentError::ProjectInboxFull);
        }
        self.pending.push_back(());
        Ok(())
    }

    fn take(&mut self) -> bool {
        self.pending.pop_front().is_some()
    }
}

#[derive(Debug)]
struct OpenEasingInbox {
    expected: Option<OpenEasingIdentity>,
    layout_epoch: u64,
    pending: VecDeque<OpenEasingRequest>,
}

impl OpenEasingInbox {
    fn new(expected: Option<OpenEasingIdentity>) -> Self {
        Self {
            expected,
            layout_epoch: 0,
            pending: VecDeque::new(),
        }
    }

    fn reconcile(&mut self, expected: Option<OpenEasingIdentity>) {
        if self.expected != expected {
            self.expected = expected;
            self.pending.clear();
        }
    }

    fn set_layout_epoch(&mut self, layout_epoch: u64) {
        if layout_epoch > self.layout_epoch {
            self.layout_epoch = layout_epoch;
            self.pending.clear();
        }
    }

    fn accept(&mut self, raw: &str) -> Result<(), StageChromeIntentError> {
        let message: OpenEasingMessage = serde_json::from_str(raw)?;
        let identity = OpenEasingIdentity {
            layer_id: message.layer_id,
            left_key_id: message.left_key_id,
            right_key_id: message.right_key_id,
            projection_generation: message.projection_generation,
        };
        if self.expected != Some(identity) || message.layout_epoch != self.layout_epoch {
            return Err(StageChromeIntentError::IdentityMismatch);
        }
        let anchor = LogicalRect {
            x: message.anchor.x,
            y: message.anchor.y,
            width: message.anchor.width,
            height: message.anchor.height,
        };
        if ![anchor.x, anchor.y, anchor.width, anchor.height]
            .iter()
            .all(|value| value.is_finite())
            || anchor.x < 0.0
            || anchor.y < 0.0
            || anchor.width <= 0.0
            || anchor.height <= 0.0
        {
            return Err(StageChromeIntentError::InvalidAnchor);
        }
        if self.pending.len() >= MAX_PENDING_EASING_OPENS {
            return Err(StageChromeIntentError::InboxFull);
        }
        self.pending.push_back(OpenEasingRequest {
            layer_id: identity.layer_id,
            left_key_id: identity.left_key_id,
            right_key_id: identity.right_key_id,
            projection_generation: identity.projection_generation,
            layout_epoch: message.layout_epoch,
            anchor,
        });
        Ok(())
    }
}

pub(crate) struct StageChromeHostRuntime {
    easing_inbox: Arc<Mutex<OpenEasingInbox>>,
    playback_inbox: Arc<Mutex<TogglePlaybackInbox>>,
    save_inbox: Arc<Mutex<ProjectActionInbox>>,
    export_inbox: Arc<Mutex<ProjectActionInbox>>,
    wake: Arc<Mutex<Option<StageChromeWake>>>,
    header: WebView,
    transport: WebView,
    latest_layout_epoch: Option<u64>,
}

impl StageChromeHostRuntime {
    pub(crate) fn new(
        window: &winit::window::Window,
        document: &motolii_doc::Document,
        selected_position_key: Option<(motolii_doc::LayerId, motolii_doc::KeyframeId)>,
        projection_generation: u64,
        playhead: RationalTime,
    ) -> Result<Self, StageChromeHostRuntimeError> {
        let created_at = std::time::Instant::now();
        let (snapshot, expected) = snapshot_json(
            document,
            selected_position_key,
            projection_generation,
            StageChromeProjection::new(false, false, StageChromeStatus::new("READY", "READY")),
            0,
            playhead,
        )?;
        let encoded = javascript_json_parse_argument(&snapshot)?;
        let initialization_script = format!(
            r#"window.__MOTOLII_STAGE_HOST__=(()=>{{
let listener=null;
let current=JSON.parse({encoded});
return Object.freeze({{
get snapshot(){{return current;}},
subscribe:(next)=>{{if(typeof next!=="function"||listener!==null)throw new TypeError("invalid Stage subscriber");listener=next;listener(current);return()=>{{listener=null;}};}},
publish:(next)=>{{current=next;if(listener!==null)listener(current);}},
setLayoutEpoch:(layoutEpoch)=>{{current=Object.freeze({{...current,layoutEpoch}});if(listener!==null)listener(current);}},
openEasing:(interval,anchor)=>window.ipc.postMessage(JSON.stringify({{kind:"open-easing",layerId:interval.layerId,leftKeyId:interval.leftKeyId,rightKeyId:interval.rightKeyId,projectionGeneration:interval.projectionGeneration,layoutEpoch:current.layoutEpoch,anchor:{{x:anchor.x,y:anchor.y,width:anchor.width,height:anchor.height}}}})),
togglePlayback:()=>window.ipc.postMessage(JSON.stringify({{kind:"toggle-play"}})),
saveProject:()=>window.ipc.postMessage(JSON.stringify({{kind:"save-project"}})),
exportProject:()=>window.ipc.postMessage(JSON.stringify({{kind:"export-project"}}))
}});
}})();"#
        );
        let easing_inbox = Arc::new(Mutex::new(OpenEasingInbox::new(expected)));
        let playback_inbox = Arc::new(Mutex::new(TogglePlaybackInbox::new()));
        let save_inbox = Arc::new(Mutex::new(ProjectActionInbox::default()));
        let export_inbox = Arc::new(Mutex::new(ProjectActionInbox::default()));
        let wake = Arc::new(Mutex::new(None::<StageChromeWake>));
        let header = build_stage_webview(
            window,
            HEADER_URL,
            &initialization_script,
            "stage-header",
            None,
        )?;
        let callback_inbox = Arc::clone(&easing_inbox);
        let callback_playback = Arc::clone(&playback_inbox);
        let callback_save = Arc::clone(&save_inbox);
        let callback_export = Arc::clone(&export_inbox);
        let callback_wake = Arc::clone(&wake);
        let transport = build_stage_webview(
            window,
            TRANSPORT_URL,
            &initialization_script,
            "stage-transport",
            Some(Box::new(move |raw| {
                let accepted = accept_stage_message(
                    raw,
                    &callback_inbox,
                    &callback_playback,
                    &callback_save,
                    &callback_export,
                );
                match accepted {
                    Ok(()) => {
                        if let Ok(wake) = callback_wake.lock() {
                            if let Some(wake) = wake.as_ref() {
                                wake();
                            }
                        }
                    }
                    Err(error) => eprintln!("Stage Host rejected message: {error}"),
                }
            })),
        )?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=stage-chrome event=created elapsed_ms={:.3}",
            created_at.elapsed().as_secs_f64() * 1_000.0,
        ));
        Ok(Self {
            easing_inbox,
            playback_inbox,
            save_inbox,
            export_inbox,
            wake,
            header,
            transport,
            latest_layout_epoch: None,
        })
    }

    pub(crate) fn register_wake(
        &self,
        wake: StageChromeWake,
    ) -> Result<(), StageChromeIntentError> {
        *self
            .wake
            .lock()
            .map_err(|_| StageChromeIntentError::InboxPoisoned)? = Some(wake);
        Ok(())
    }

    pub(crate) fn take_open_easing(
        &self,
    ) -> Result<Option<OpenEasingRequest>, StageChromeIntentError> {
        Ok(self
            .easing_inbox
            .lock()
            .map_err(|_| StageChromeIntentError::InboxPoisoned)?
            .pending
            .pop_front())
    }

    pub(crate) fn take_toggle_playback(&self) -> Result<bool, StageChromeIntentError> {
        Ok(self
            .playback_inbox
            .lock()
            .map_err(|_| StageChromeIntentError::InboxPoisoned)?
            .take())
    }

    pub(crate) fn take_save_project(&self) -> Result<bool, StageChromeIntentError> {
        Ok(self
            .save_inbox
            .lock()
            .map_err(|_| StageChromeIntentError::InboxPoisoned)?
            .take())
    }

    pub(crate) fn take_export_project(&self) -> Result<bool, StageChromeIntentError> {
        Ok(self
            .export_inbox
            .lock()
            .map_err(|_| StageChromeIntentError::InboxPoisoned)?
            .take())
    }

    pub(crate) fn publish(
        &self,
        document: &motolii_doc::Document,
        selected_position_key: Option<(motolii_doc::LayerId, motolii_doc::KeyframeId)>,
        projection_generation: u64,
        projection: StageChromeProjection<'_>,
        playhead: RationalTime,
    ) -> Result<(), StageChromeHostRuntimeError> {
        let (snapshot, expected) = snapshot_json(
            document,
            selected_position_key,
            projection_generation,
            projection,
            self.latest_layout_epoch.unwrap_or(0),
            playhead,
        )?;
        self.easing_inbox
            .lock()
            .map_err(|_| StageChromeIntentError::InboxPoisoned)?
            .reconcile(expected);
        let encoded = javascript_json_parse_argument(&snapshot)?;
        self.transport.evaluate_script(&format!(
            "window.__MOTOLII_STAGE_HOST__.publish(JSON.parse({encoded}));"
        ))?;
        Ok(())
    }

    pub(crate) fn set_bounds(
        &mut self,
        layout_epoch: u64,
        header: LogicalRect,
        transport: LogicalRect,
    ) -> Result<(), StageChromeHostRuntimeError> {
        if self
            .latest_layout_epoch
            .is_some_and(|latest| layout_epoch <= latest)
        {
            return Ok(());
        }
        set_webview_bounds(&self.header, header)?;
        set_webview_bounds(&self.transport, transport)?;
        self.easing_inbox
            .lock()
            .map_err(|_| StageChromeIntentError::InboxPoisoned)?
            .set_layout_epoch(layout_epoch);
        self.transport.evaluate_script(&format!(
            "window.__MOTOLII_STAGE_HOST__.setLayoutEpoch({layout_epoch});"
        ))?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=stage-chrome event=bounds layout_epoch={} \
             header_x={:.3} header_y={:.3} header_width={:.3} header_height={:.3} \
             transport_x={:.3} transport_y={:.3} transport_width={:.3} transport_height={:.3}",
            layout_epoch,
            header.x,
            header.y,
            header.width,
            header.height,
            transport.x,
            transport.y,
            transport.width,
            transport.height,
        ));
        self.latest_layout_epoch = Some(layout_epoch);
        Ok(())
    }
}

type StageIpcHandler = Box<dyn Fn(&str) + Send + 'static>;

fn build_stage_webview(
    window: &winit::window::Window,
    entry_url: &'static str,
    initialization_script: &str,
    surface: &'static str,
    ipc_handler: Option<StageIpcHandler>,
) -> Result<WebView, wry::Error> {
    let mut builder = WebViewBuilder::new()
        .with_bounds(Rect {
            position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
            size: wry::dpi::LogicalSize::new(1.0, 1.0).into(),
        })
        .with_initialization_script(initialization_script)
        .with_custom_protocol(PROTOCOL.to_owned(), move |_webview_id, request| {
            product_asset_response(request.uri().path())
        })
        .with_url(entry_url)
        .with_navigation_handler(move |target| {
            let accepted = target.starts_with("motolii-stage:");
            if !accepted {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=webview surface={} event=navigation-rejected",
                    surface,
                ));
            }
            accepted
        });
    if let Some(ipc_handler) = ipc_handler {
        builder = builder.with_ipc_handler(move |request| ipc_handler(request.body()));
    }
    builder.build_as_child(window)
}

fn set_webview_bounds(webview: &WebView, rect: LogicalRect) -> Result<(), wry::Error> {
    webview.set_bounds(Rect {
        position: wry::dpi::LogicalPosition::new(rect.x, rect.y).into(),
        size: wry::dpi::LogicalSize::new(rect.width, rect.height).into(),
    })
}

fn snapshot_json(
    document: &motolii_doc::Document,
    selected_position_key: Option<(motolii_doc::LayerId, motolii_doc::KeyframeId)>,
    projection_generation: u64,
    projection: StageChromeProjection<'_>,
    layout_epoch: u64,
    playhead: RationalTime,
) -> Result<(serde_json::Value, Option<OpenEasingIdentity>), serde_json::Error> {
    let projected = selected_position_key.and_then(|(layer, left_key)| {
        let interval = project_position_interval(document, layer, left_key)?;
        let object_name = document.layers.display_name(layer)?.to_owned();
        let request = OpenEasingIdentity {
            layer_id: layer,
            left_key_id: interval.left_key,
            right_key_id: interval.right_key,
            projection_generation,
        };
        let value = serde_json::json!({
            "objectName": object_name,
            "channel": "Position",
            "layerId": layer,
            "leftKeyId": interval.left_key,
            "rightKeyId": interval.right_key,
            "projectionGeneration": projection_generation,
            "interp": interval.interp,
        });
        Some((value, request))
    });
    let active_interval = projected.as_ref().map(|(value, _)| value.clone());
    let expected = projected.map(|(_, request)| request);
    Ok((
        serde_json::json!({
            "mode": "RECTANGLE",
            "timecode": format_stage_timecode(playhead),
            "barPosition": "BAR 0.0.00",
            "tempoStatus": "120 BPM · SNAP BEAT",
            "qualityStatus": "DRAFT · FP16 · 1/2",
            "activeInterval": active_interval,
            "easingPressed": projection.easing_pressed,
            "playing": projection.playing,
            "projectStatus": projection.status.project,
            "exportStatus": projection.status.export,
            "layoutEpoch": layout_epoch,
        }),
        expected,
    ))
}

fn format_stage_timecode(time: RationalTime) -> String {
    let total_tenths = (time.as_seconds_f64().max(0.0) * 10.0).floor() as u64;
    let tenths = total_tenths % 10;
    let total_seconds = total_tenths / 10;
    let seconds = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    if total_minutes < 60 {
        format!("{total_minutes:02}:{seconds:02}.{tenths}")
    } else {
        let minutes = total_minutes % 60;
        let hours = total_minutes / 60;
        format!("{hours:02}:{minutes:02}:{seconds:02}.{tenths}")
    }
}

fn javascript_json_parse_argument(
    snapshot: &serde_json::Value,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(&serde_json::to_string(snapshot)?)
}

fn accept_stage_message(
    raw: &str,
    easing_inbox: &Arc<Mutex<OpenEasingInbox>>,
    playback_inbox: &Arc<Mutex<TogglePlaybackInbox>>,
    save_inbox: &Arc<Mutex<ProjectActionInbox>>,
    export_inbox: &Arc<Mutex<ProjectActionInbox>>,
) -> Result<(), StageChromeIntentError> {
    let value = serde_json::from_str::<serde_json::Value>(raw)?;
    let kind = value.get("kind").and_then(serde_json::Value::as_str);
    match kind {
        Some("toggle-play") => playback_inbox
            .lock()
            .map_err(|_| StageChromeIntentError::InboxPoisoned)?
            .accept(raw),
        Some("save-project") => save_inbox
            .lock()
            .map_err(|_| StageChromeIntentError::InboxPoisoned)?
            .accept(raw),
        Some("export-project") => export_inbox
            .lock()
            .map_err(|_| StageChromeIntentError::InboxPoisoned)?
            .accept(raw),
        _ => easing_inbox
            .lock()
            .map_err(|_| StageChromeIntentError::InboxPoisoned)?
            .accept(raw),
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum StageChromeIntentError {
    #[error("Stage chrome message is invalid")]
    Json(#[from] serde_json::Error),
    #[error("Stage chrome easing interval identity is stale or mismatched")]
    IdentityMismatch,
    #[error("Stage chrome easing anchor is invalid")]
    InvalidAnchor,
    #[error("Stage chrome easing intent inbox is full")]
    InboxFull,
    #[error("Stage chrome playback intent inbox is full")]
    PlaybackInboxFull,
    #[error("Stage chrome project action inbox is full")]
    ProjectInboxFull,
    #[error("Stage chrome easing intent inbox is unavailable")]
    InboxPoisoned,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum StageChromeHostRuntimeError {
    #[error(transparent)]
    Intent(#[from] StageChromeIntentError),
    #[error("Stage chrome snapshot JSON could not be encoded")]
    Json(#[from] serde_json::Error),
    #[error("Stage chrome WebView failed")]
    WebView(#[from] wry::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(generation: u64) -> OpenEasingRequest {
        OpenEasingRequest {
            layer_id: motolii_doc::LayerId::from_raw(7),
            left_key_id: motolii_doc::KeyframeId::from_raw(11),
            right_key_id: motolii_doc::KeyframeId::from_raw(12),
            projection_generation: generation,
            layout_epoch: 5,
            anchor: LogicalRect {
                x: 90.0,
                y: 4.0,
                width: 28.0,
                height: 22.0,
            },
        }
    }

    fn identity(generation: u64) -> OpenEasingIdentity {
        OpenEasingIdentity {
            layer_id: motolii_doc::LayerId::from_raw(7),
            left_key_id: motolii_doc::KeyframeId::from_raw(11),
            right_key_id: motolii_doc::KeyframeId::from_raw(12),
            projection_generation: generation,
        }
    }

    fn message(generation: u64) -> String {
        format!(
            r#"{{"kind":"open-easing","layerId":7,"leftKeyId":11,"rightKeyId":12,"projectionGeneration":{generation},"layoutEpoch":5,"anchor":{{"x":90.0,"y":4.0,"width":28.0,"height":22.0}}}}"#
        )
    }

    #[test]
    fn easing_open_accepts_only_the_current_projected_interval() {
        let expected = request(3);
        let mut inbox = OpenEasingInbox::new(Some(identity(3)));
        inbox.set_layout_epoch(5);

        inbox.accept(&message(3)).unwrap();

        assert_eq!(inbox.pending.pop_front(), Some(expected));
        assert!(inbox.pending.is_empty());
    }

    #[test]
    fn easing_open_rejects_stale_unknown_and_unprojected_messages() {
        let mut inbox = OpenEasingInbox::new(Some(identity(3)));
        inbox.set_layout_epoch(5);
        assert!(matches!(
            inbox.accept(&message(2)),
            Err(StageChromeIntentError::IdentityMismatch)
        ));
        assert!(matches!(
            inbox.accept(
                r#"{"kind":"open-easing","layerId":7,"leftKeyId":11,"rightKeyId":12,"projectionGeneration":3,"layoutEpoch":5,"anchor":{"x":90.0,"y":4.0,"width":28.0,"height":22.0},"extra":true}"#
            ),
            Err(StageChromeIntentError::Json(_))
        ));
        inbox.reconcile(None);
        assert!(matches!(
            inbox.accept(&message(3)),
            Err(StageChromeIntentError::IdentityMismatch)
        ));
        assert!(inbox.pending.is_empty());
    }

    #[test]
    fn changing_projected_interval_discards_queued_stale_open() {
        let mut inbox = OpenEasingInbox::new(Some(identity(3)));
        inbox.set_layout_epoch(5);
        inbox.accept(&message(3)).unwrap();

        inbox.reconcile(Some(identity(4)));

        assert!(inbox.pending.is_empty());
        assert_eq!(inbox.expected, Some(identity(4)));
    }

    #[test]
    fn playback_toggle_accepts_only_the_closed_message() {
        let mut inbox = TogglePlaybackInbox::new();

        inbox.accept(r#"{"kind":"toggle-play"}"#).unwrap();
        assert!(inbox.take());
        assert!(!inbox.take());
        assert!(matches!(
            inbox.accept(r#"{"kind":"toggle-play","extra":true}"#),
            Err(StageChromeIntentError::Json(_))
        ));
    }

    #[test]
    fn project_actions_are_strict_and_bounded() {
        let mut inbox = ProjectActionInbox::default();
        inbox.accept(r#"{"kind":"save-project"}"#).unwrap();
        assert!(inbox.take());
        assert!(!inbox.take());
        assert!(matches!(
            inbox.accept(r#"{"kind":"export-project","extra":true}"#),
            Err(StageChromeIntentError::Json(_))
        ));
    }

    #[test]
    fn stage_timecode_projects_the_host_playhead() {
        assert_eq!(format_stage_timecode(RationalTime::ZERO), "00:00.0");
        assert_eq!(
            format_stage_timecode(RationalTime::try_new(125, 10).unwrap()),
            "00:12.5"
        );
        assert_eq!(
            format_stage_timecode(RationalTime::try_new(3_661, 1).unwrap()),
            "01:01:01.0"
        );
    }
}
