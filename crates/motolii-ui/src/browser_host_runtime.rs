//! product-owned Browser bundleをnative shellのopaque childへ載せるHost。

use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use wry::http::{header::CONTENT_TYPE, Response};
#[cfg(target_os = "macos")]
use wry::WebViewBuilderExtDarwin;
use wry::{NewWindowResponse, PageLoadEvent, Rect, WebView, WebViewBuilder};

use crate::browser_host::{BrowserAttachEffectIntent, BrowserHostSession, BrowserPlaceIntent};
use crate::host_pointer_capture::{
    HostPointerCandidate, HostPointerClick, PlatformPointerCapture, PlatformPointerCaptureError,
};
use crate::native_host_layout::LogicalRect;

const PROTOCOL: &str = "motolii-browser";
const ENTRY_URL: &str = "motolii-browser://product/host.html";
const HOST_HTML: &[u8] = include_bytes!("../../../ui/motolii-web/generated-host/host.html");
const HOST_JS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/host-eLLhPPap.js");
const HOST_CSS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/host-BTNUEeQC.css");
const INSPECTOR_HTML: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/inspector.html");
const INSPECTOR_JS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/inspector-DHxJYfhL.js");
const INSPECTOR_CSS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/inspector-fMh9jxYJ.css");
const STAGE_HEADER_HTML: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/stage-header.html");
const STAGE_TRANSPORT_HTML: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/stage-transport.html");
const TIMELINE_TOOLS_HTML: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/timeline-tools.html");
const STAGE_HEADER_JS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/stageHeader-CbQfY9Sr.js");
const STAGE_TRANSPORT_JS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/stageTransport-BGTTL1QB.js");
const STAGE_HOST_JS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/stageHostBridge-BPiRulR-.js");
const STAGE_HOST_CSS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/stageHostBridge-ETwwNc59.css");
const TIMELINE_TOOLS_JS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/timelineTools-Dg87OVI1.js");
const TIMELINE_TOOLS_CSS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/timelineTools-CEMKvxwx.css");
const SHARED_JS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/tokens-CcZ3RUC1.js");
const SHARED_CSS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/tokens-_d9c2vBB.css");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BrowserLifecycleEvent {
    ReloadStarted { instance_epoch: u64 },
    ProcessTerminated { instance_epoch: u64 },
}

impl BrowserLifecycleEvent {
    pub(crate) fn instance_epoch(self) -> u64 {
        match self {
            Self::ReloadStarted { instance_epoch } | Self::ProcessTerminated { instance_epoch } => {
                instance_epoch
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BrowserFocusTarget {
    Parent,
    Browser,
}

pub(crate) struct BrowserHostRuntime {
    session: Arc<Mutex<BrowserHostSession>>,
    island: Arc<Mutex<BrowserIslandState>>,
    webview: WebView,
    pointer_capture: Mutex<PlatformPointerCapture>,
}

#[derive(Debug)]
struct BrowserIslandState {
    instance_epoch: u64,
    initial_projection_ready: bool,
    latest_layout_epoch: Option<u64>,
    requested_focus_target: Option<BrowserFocusTarget>,
    reload_reported: bool,
}

impl BrowserIslandState {
    fn new(instance_epoch: u64) -> Self {
        Self {
            instance_epoch,
            initial_projection_ready: false,
            latest_layout_epoch: None,
            requested_focus_target: None,
            reload_reported: false,
        }
    }

    fn observe_initial_projection(&mut self, instance_epoch: u64) -> bool {
        if instance_epoch != self.instance_epoch || self.initial_projection_ready {
            return false;
        }
        self.initial_projection_ready = true;
        self.reload_reported = false;
        true
    }

    fn observe_reload_started(&mut self, instance_epoch: u64) -> bool {
        if instance_epoch != self.instance_epoch
            || !self.initial_projection_ready
            || self.reload_reported
        {
            return false;
        }
        self.initial_projection_ready = false;
        self.reload_reported = true;
        true
    }

    fn should_apply_layout(&self, epoch: u64) -> bool {
        self.latest_layout_epoch.is_none_or(|latest| epoch > latest)
    }

    fn commit_layout(&mut self, epoch: u64) {
        if self.should_apply_layout(epoch) {
            self.latest_layout_epoch = Some(epoch);
        }
    }

    fn can_transfer_focus(&self) -> bool {
        self.initial_projection_ready
    }

    fn should_transfer_focus(&self, target: BrowserFocusTarget) -> bool {
        self.can_transfer_focus() && self.requested_focus_target != Some(target)
    }

    fn commit_focus(&mut self, target: BrowserFocusTarget) {
        self.requested_focus_target = Some(target);
    }
}

impl BrowserHostRuntime {
    pub(crate) fn fresh_instance_epoch() -> Result<u64, BrowserHostRuntimeError> {
        let elapsed = SystemTime::now().duration_since(UNIX_EPOCH)?;
        u64::try_from(elapsed.as_millis()).map_err(|_| BrowserHostRuntimeError::EpochOverflow)
    }

    pub(crate) fn built_in_rectangle_source(instance_epoch: u64) -> BrowserPlaceIntent {
        BrowserPlaceIntent {
            // scopeはHost projectionが発行するopaque identityで、表示値から導かない。
            scope_ref: format!("builtin-{instance_epoch}"),
            item_id: "rectangle".to_owned(),
        }
    }

    pub(crate) fn new(
        window: &winit::window::Window,
        instance_epoch: u64,
        source: BrowserPlaceIntent,
        wake: Arc<dyn Fn() + Send + Sync>,
        lifecycle: Arc<dyn Fn(BrowserLifecycleEvent) + Send + Sync>,
    ) -> Result<Self, BrowserHostRuntimeError> {
        let created_at = std::time::Instant::now();
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=browser event=create-start instance_epoch={}",
            instance_epoch,
        ));
        let session = BrowserHostSession::new(instance_epoch, 0, source);
        let snapshot = session.snapshot_json()?;
        let encoded_snapshot = serde_json::to_string(&snapshot)?;
        let initialization_script = format!(
            r#"window.__MOTOLII_BUILTIN_HOST__=Object.freeze({{
snapshot:JSON.parse({encoded_snapshot}),
postMessage:(message)=>window.ipc.postMessage(message)
}});"#
        );
        let session = Arc::new(Mutex::new(session));
        let island = Arc::new(Mutex::new(BrowserIslandState::new(instance_epoch)));
        let callback_session = Arc::clone(&session);
        let callback_island = Arc::clone(&island);
        let load_island = Arc::clone(&island);
        let load_wake = Arc::clone(&wake);
        let pointer_capture_wake = Arc::clone(&wake);
        let load_lifecycle = Arc::clone(&lifecycle);
        let webview_builder = WebViewBuilder::new()
            .with_bounds(Rect {
                position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
                size: wry::dpi::LogicalSize::new(1.0, 1.0).into(),
            })
            .with_initialization_script(&initialization_script)
            .with_custom_protocol(PROTOCOL.to_owned(), move |_webview_id, request| {
                product_asset_response(request.uri().path())
            })
            .with_url(ENTRY_URL)
            .with_navigation_handler(|target| target.starts_with("motolii-browser:"))
            .with_new_window_req_handler(|_url, _features| NewWindowResponse::Deny)
            .with_download_started_handler(|_url, _destination| false)
            .with_on_page_load_handler(move |event, url| {
                if url != ENTRY_URL {
                    return;
                }
                match event {
                    PageLoadEvent::Started => {
                        crate::ui_numeric_trace::emit(format_args!(
                            "kind=webview surface=browser event=load-started instance_epoch={} elapsed_ms={:.3}",
                            instance_epoch,
                            created_at.elapsed().as_secs_f64() * 1_000.0,
                        ));
                        let reload = load_island
                            .lock()
                            .map(|mut island| island.observe_reload_started(instance_epoch))
                            .unwrap_or(false);
                        if reload {
                            load_lifecycle(BrowserLifecycleEvent::ReloadStarted { instance_epoch });
                        }
                    }
                    PageLoadEvent::Finished => {
                        crate::ui_numeric_trace::emit(format_args!(
                            "kind=webview surface=browser event=load-finished instance_epoch={} elapsed_ms={:.3}",
                            instance_epoch,
                            created_at.elapsed().as_secs_f64() * 1_000.0,
                        ));
                        let changed = load_island
                            .lock()
                            .map(|mut island| island.observe_initial_projection(instance_epoch))
                            .unwrap_or(false);
                        if changed {
                            load_wake();
                        }
                    }
                }
            });
        #[cfg(target_os = "macos")]
        let webview_builder = {
            let terminated_lifecycle = Arc::clone(&lifecycle);
            webview_builder.with_on_web_content_process_terminate_handler(move || {
                terminated_lifecycle(BrowserLifecycleEvent::ProcessTerminated { instance_epoch });
            })
        };
        let webview = webview_builder
            .with_ipc_handler(move |request| {
                let raw = request.body();
                match callback_session.lock() {
                    Ok(mut session) => match session.accept(raw) {
                        Ok(()) => {
                            crate::ui_numeric_trace::emit(format_args!(
                                "kind=webview surface=browser event=ipc-accepted instance_epoch={} bytes={}",
                                instance_epoch,
                                raw.len(),
                            ));
                            if let Ok(mut island) = callback_island.lock() {
                                island.observe_initial_projection(instance_epoch);
                            }
                            wake();
                        }
                        Err(error) => eprintln!("Browser Host rejected message: {error}"),
                    },
                    Err(_) => eprintln!("Browser Host inbox lock is poisoned"),
                }
            })
            .build_as_child(window)?;
        let pointer_capture =
            Mutex::new(PlatformPointerCapture::new(window, pointer_capture_wake)?);
        Ok(Self {
            session,
            island,
            webview,
            pointer_capture,
        })
    }

    pub(crate) fn take_place_intent(
        &self,
        generation: u64,
    ) -> Result<Option<(BrowserPlaceIntent, u64)>, BrowserHostRuntimeError> {
        let intent = self
            .session
            .lock()
            .map_err(|_| BrowserHostRuntimeError::InboxPoisoned)
            .map(|mut session| session.pop())?;
        let Some(intent) = intent else {
            return Ok(None);
        };
        let generation = self
            .pointer_capture
            .lock()
            .map_err(|_| BrowserHostRuntimeError::PointerCapturePoisoned)?
            .arm(generation)
            .then_some(generation)
            .ok_or(BrowserHostRuntimeError::PointerCaptureAlreadyActive)?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=browser event=intent-dequeued generation={} scope_bytes={} item_bytes={}",
            generation,
            intent.scope_ref.len(),
            intent.item_id.len(),
        ));
        Ok(Some((intent, generation)))
    }

    pub(crate) fn take_attach_effect_intent(
        &self,
    ) -> Result<Option<BrowserAttachEffectIntent>, BrowserHostRuntimeError> {
        let intent = self
            .session
            .lock()
            .map_err(|_| BrowserHostRuntimeError::InboxPoisoned)
            .map(|mut session| session.pop_attach_effect())?;
        if let Some(intent) = &intent {
            crate::ui_numeric_trace::emit(format_args!(
                "kind=webview surface=browser event=attach-effect-dequeued scope_bytes={} item_bytes={}",
                intent.scope_ref.len(),
                intent.item_id.len(),
            ));
        }
        Ok(intent)
    }

    pub(crate) fn poll_pointer_candidate(
        &self,
    ) -> Result<Option<HostPointerCandidate>, BrowserHostRuntimeError> {
        self.pointer_capture
            .lock()
            .map_err(|_| BrowserHostRuntimeError::PointerCapturePoisoned)?
            .poll()
            .map_err(Into::into)
    }

    pub(crate) fn pointer_capture_is_active(&self) -> Result<bool, BrowserHostRuntimeError> {
        self.pointer_capture
            .lock()
            .map_err(|_| BrowserHostRuntimeError::PointerCapturePoisoned)
            .map(|capture| capture.is_active())
    }

    pub(crate) fn poll_host_click(
        &self,
    ) -> Result<Option<HostPointerClick>, BrowserHostRuntimeError> {
        self.pointer_capture
            .lock()
            .map_err(|_| BrowserHostRuntimeError::PointerCapturePoisoned)?
            .poll_click()
            .map_err(Into::into)
    }

    pub(crate) fn poll_host_command(
        &self,
    ) -> Result<Option<crate::EffectiveTrigger>, BrowserHostRuntimeError> {
        self.pointer_capture
            .lock()
            .map_err(|_| BrowserHostRuntimeError::PointerCapturePoisoned)?
            .poll_command()
            .map_err(Into::into)
    }

    pub(crate) fn instance_epoch(&self) -> Result<u64, BrowserHostRuntimeError> {
        self.island
            .lock()
            .map_err(|_| BrowserHostRuntimeError::IslandStatePoisoned)
            .map(|island| island.instance_epoch)
    }

    pub(crate) fn ensure_focus(
        &self,
        target: BrowserFocusTarget,
    ) -> Result<(), BrowserHostRuntimeError> {
        if !self
            .island
            .lock()
            .map_err(|_| BrowserHostRuntimeError::IslandStatePoisoned)?
            .can_transfer_focus()
        {
            return Ok(());
        }
        self.transfer_focus(target)?;
        self.pointer_capture
            .lock()
            .map_err(|_| BrowserHostRuntimeError::PointerCapturePoisoned)?
            .set_host_commands_enabled(target == BrowserFocusTarget::Parent);
        Ok(())
    }

    pub(crate) fn set_bounds(
        &self,
        layout_epoch: u64,
        rect: impl Into<Option<LogicalRect>>,
    ) -> Result<(), BrowserHostRuntimeError> {
        if !self
            .island
            .lock()
            .map_err(|_| BrowserHostRuntimeError::IslandStatePoisoned)?
            .should_apply_layout(layout_epoch)
        {
            return Ok(());
        }
        let rect = rect.into();
        if let Some(rect) = rect {
            self.webview.set_bounds(Rect {
                position: wry::dpi::LogicalPosition::new(rect.x, rect.y).into(),
                size: wry::dpi::LogicalSize::new(rect.width, rect.height).into(),
            })?;
            crate::ui_numeric_trace::emit(format_args!(
                "kind=webview surface=browser event=bounds layout_epoch={} visible=true \
                 x={:.3} y={:.3} width={:.3} height={:.3}",
                layout_epoch, rect.x, rect.y, rect.width, rect.height,
            ));
        } else {
            crate::ui_numeric_trace::emit(format_args!(
                "kind=webview surface=browser event=bounds layout_epoch={} visible=false",
                layout_epoch,
            ));
        }
        self.webview.set_visible(rect.is_some())?;
        self.island
            .lock()
            .map_err(|_| BrowserHostRuntimeError::IslandStatePoisoned)?
            .commit_layout(layout_epoch);
        Ok(())
    }

    fn transfer_focus(&self, target: BrowserFocusTarget) -> Result<(), BrowserHostRuntimeError> {
        if !self
            .island
            .lock()
            .map_err(|_| BrowserHostRuntimeError::IslandStatePoisoned)?
            .should_transfer_focus(target)
        {
            return Ok(());
        }
        match target {
            BrowserFocusTarget::Parent => self.webview.focus_parent()?,
            BrowserFocusTarget::Browser => self.webview.focus()?,
        }
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=browser event=focus target={target:?}"
        ));
        self.island
            .lock()
            .map_err(|_| BrowserHostRuntimeError::IslandStatePoisoned)?
            .commit_focus(target);
        Ok(())
    }
}

pub(crate) fn product_asset_response(path: &str) -> Response<Cow<'static, [u8]>> {
    let (content_type, body) = match path {
        "/" | "/host.html" => ("text/html; charset=utf-8", HOST_HTML),
        "/inspector.html" => ("text/html; charset=utf-8", INSPECTOR_HTML),
        "/stage-header.html" => ("text/html; charset=utf-8", STAGE_HEADER_HTML),
        "/stage-transport.html" => ("text/html; charset=utf-8", STAGE_TRANSPORT_HTML),
        "/timeline-tools.html" => ("text/html; charset=utf-8", TIMELINE_TOOLS_HTML),
        "/assets/host-eLLhPPap.js" => ("text/javascript; charset=utf-8", HOST_JS),
        "/assets/host-BTNUEeQC.css" => ("text/css; charset=utf-8", HOST_CSS),
        "/assets/inspector-DHxJYfhL.js" => ("text/javascript; charset=utf-8", INSPECTOR_JS),
        "/assets/inspector-fMh9jxYJ.css" => ("text/css; charset=utf-8", INSPECTOR_CSS),
        "/assets/stageHeader-CbQfY9Sr.js" => ("text/javascript; charset=utf-8", STAGE_HEADER_JS),
        "/assets/stageTransport-BGTTL1QB.js" => {
            ("text/javascript; charset=utf-8", STAGE_TRANSPORT_JS)
        }
        "/assets/stageHostBridge-BPiRulR-.js" => ("text/javascript; charset=utf-8", STAGE_HOST_JS),
        "/assets/stageHostBridge-ETwwNc59.css" => ("text/css; charset=utf-8", STAGE_HOST_CSS),
        "/assets/timelineTools-Dg87OVI1.js" => {
            ("text/javascript; charset=utf-8", TIMELINE_TOOLS_JS)
        }
        "/assets/timelineTools-CEMKvxwx.css" => ("text/css; charset=utf-8", TIMELINE_TOOLS_CSS),
        "/assets/tokens-CcZ3RUC1.js" => ("text/javascript; charset=utf-8", SHARED_JS),
        "/assets/tokens-_d9c2vBB.css" => ("text/css; charset=utf-8", SHARED_CSS),
        _ => {
            return Response::builder()
                .status(404)
                .body(Cow::Borrowed(&[] as &[u8]))
                .expect("constant 404 response");
        }
    };
    Response::builder()
        .header(CONTENT_TYPE, content_type)
        .body(Cow::Borrowed(body))
        .expect("constant product asset response")
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum BrowserHostRuntimeError {
    #[error("Browser Host clock is before the Unix epoch")]
    Clock(#[from] std::time::SystemTimeError),
    #[error("Browser Host epoch exceeds u64")]
    EpochOverflow,
    #[error("Browser Host JSON could not be encoded")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Codec(#[from] crate::browser_host::BrowserHostError),
    #[error("Browser Host WebView failed")]
    WebView(#[from] wry::Error),
    #[error("Browser Host inbox lock is poisoned")]
    InboxPoisoned,
    #[error("Browser Host pointer capture lock is poisoned")]
    PointerCapturePoisoned,
    #[error("Browser Host pointer capture is already active")]
    PointerCaptureAlreadyActive,
    #[error("Browser Host island state lock is poisoned")]
    IslandStatePoisoned,
    #[error(transparent)]
    PointerCapture(#[from] PlatformPointerCaptureError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_host_html_references_the_embedded_assets() {
        let html = std::str::from_utf8(HOST_HTML).expect("generated Host HTML is UTF-8");

        for path in [
            "/assets/host-eLLhPPap.js",
            "/assets/host-BTNUEeQC.css",
            "/assets/tokens-CcZ3RUC1.js",
            "/assets/tokens-_d9c2vBB.css",
        ] {
            assert!(html.contains(path));
            let response = product_asset_response(path);
            assert_eq!(response.status(), 200);
            assert!(!response.body().is_empty());
        }

        let inspector =
            std::str::from_utf8(INSPECTOR_HTML).expect("generated Inspector HTML is UTF-8");
        for path in [
            "/assets/inspector-DHxJYfhL.js",
            "/assets/inspector-fMh9jxYJ.css",
            "/assets/tokens-CcZ3RUC1.js",
            "/assets/tokens-_d9c2vBB.css",
        ] {
            assert!(inspector.contains(path));
            let response = product_asset_response(path);
            assert_eq!(response.status(), 200);
            assert!(!response.body().is_empty());
        }
    }

    #[test]
    fn embedded_stage_chrome_references_only_served_product_assets() {
        for (html, paths) in [
            (
                STAGE_HEADER_HTML,
                [
                    "/assets/stageHeader-CbQfY9Sr.js",
                    "/assets/stageHostBridge-BPiRulR-.js",
                    "/assets/stageHostBridge-ETwwNc59.css",
                    "/assets/tokens-CcZ3RUC1.js",
                    "/assets/tokens-_d9c2vBB.css",
                ],
            ),
            (
                STAGE_TRANSPORT_HTML,
                [
                    "/assets/stageTransport-BGTTL1QB.js",
                    "/assets/stageHostBridge-BPiRulR-.js",
                    "/assets/stageHostBridge-ETwwNc59.css",
                    "/assets/tokens-CcZ3RUC1.js",
                    "/assets/tokens-_d9c2vBB.css",
                ],
            ),
        ] {
            let html = std::str::from_utf8(html).expect("generated Stage HTML is UTF-8");
            for path in paths {
                assert!(html.contains(path));
                let response = product_asset_response(path);
                assert_eq!(response.status(), 200);
                assert!(!response.body().is_empty());
            }
        }
    }

    #[test]
    fn embedded_timeline_tools_references_only_served_product_assets() {
        let html = std::str::from_utf8(TIMELINE_TOOLS_HTML)
            .expect("generated Timeline Tools HTML is UTF-8");
        for path in [
            "/assets/timelineTools-Dg87OVI1.js",
            "/assets/timelineTools-CEMKvxwx.css",
            "/assets/tokens-CcZ3RUC1.js",
            "/assets/tokens-_d9c2vBB.css",
        ] {
            assert!(html.contains(path));
            let response = product_asset_response(path);
            assert_eq!(response.status(), 200);
            assert!(!response.body().is_empty());
        }
    }

    #[test]
    fn product_asset_response_rejects_unknown_paths() {
        for path in [
            "/assets/host-U2PKaKi1.js",
            "/assets/inspector-AsuNEztV.js",
            "/assets/stageHeader-Dc3Ipo6y.js",
            "/assets/stageTransport-B3FCEEvz.js",
            "/assets/stageHostBridge-DA8O00-R.js",
            "/assets/timelineTools-uxYDKYfL.js",
            "/assets/tokens-Bna1XoBQ.js",
            "/assets/tokens-Dq8978N5.css",
            "/assets/host-BKMdnRh7.js",
            "/assets/inspector-CFJXvMrF.js",
            "/assets/stageHeader-DjP6cf6c.js",
            "/assets/stageTransport-Byb7DYH4.js",
            "/assets/stageHostBridge-ChgEsvda.js",
            "/assets/timelineTools-VIpwtVDm.js",
            "/assets/tokens-aRJyBw67.js",
            "/assets/host-D4A5P1qx.js",
            "/assets/inspector-DBEprllO.js",
            "/assets/stageHeader-CtMdRMr4.js",
            "/assets/stageTransport-C2weCYBm.js",
            "/assets/stageHostBridge-CVlt-md6.js",
            "/assets/timelineTools-BK7xXweN.js",
            "/assets/tokens-BQ3aCSlZ.js",
            "/assets/host-xrDzqY7Q.js",
            "/assets/inspector-CZmr_AtC.js",
            "/assets/stageHeader-B5JMD-q0.js",
            "/assets/stageTransport-033AeWO0.js",
            "/assets/stageHostBridge-39CF9q4X.js",
            "/assets/timelineTools-CIvM4ocO.js",
            "/assets/tokens--Xo9xXv8.js",
        ] {
            let response = product_asset_response(path);
            assert_eq!(response.status(), 404);
            assert!(response.body().is_empty());
        }
    }

    #[test]
    fn island_rejects_duplicate_and_stale_geometry_epochs() {
        let mut island = BrowserIslandState::new(7);

        assert!(island.should_apply_layout(10));
        island.commit_layout(10);
        assert!(!island.should_apply_layout(10));
        assert!(!island.should_apply_layout(9));
        assert!(island.should_apply_layout(11));
    }

    #[test]
    fn initial_projection_is_single_shot_and_instance_scoped() {
        let mut island = BrowserIslandState::new(7);

        assert!(!island.can_transfer_focus());
        assert!(!island.observe_initial_projection(6));
        assert!(island.observe_initial_projection(7));
        assert!(!island.observe_initial_projection(7));
        assert!(island.can_transfer_focus());
        assert!(island.should_transfer_focus(BrowserFocusTarget::Browser));
    }

    #[test]
    fn focus_state_records_only_an_explicit_host_request() {
        let mut island = BrowserIslandState::new(7);

        assert_eq!(island.requested_focus_target, None);
        island.commit_focus(BrowserFocusTarget::Parent);
        assert_eq!(
            island.requested_focus_target,
            Some(BrowserFocusTarget::Parent)
        );
        island.commit_focus(BrowserFocusTarget::Browser);
        assert_eq!(
            island.requested_focus_target,
            Some(BrowserFocusTarget::Browser)
        );
    }

    #[test]
    fn reload_is_reported_only_after_a_live_projection() {
        let mut island = BrowserIslandState::new(7);

        assert!(!island.observe_reload_started(7));
        assert!(island.observe_initial_projection(7));
        assert!(island.observe_reload_started(7));
        assert!(!island.observe_reload_started(7));
        assert!(!island.can_transfer_focus());
        assert!(!island.observe_initial_projection(6));
    }
}
