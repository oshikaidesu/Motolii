//! product-owned Browser bundleをnative shellのopaque childへ載せるHost。

use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use wry::http::{header::CONTENT_TYPE, Response};
use wry::{NewWindowResponse, PageLoadEvent, Rect, WebView, WebViewBuilder};

use crate::browser_host::{BrowserHostSession, BrowserPlaceIntent};
use crate::host_pointer_capture::{
    HostPointerCandidate, PlatformPointerCapture, PlatformPointerCaptureError,
};
use crate::native_host_layout::LogicalRect;

const PROTOCOL: &str = "motolii-browser";
const ENTRY_URL: &str = "motolii-browser://product/host.html";
const HOST_HTML: &[u8] = include_bytes!("../../../ui/motolii-web/generated-host/host.html");
const HOST_JS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/host-Ur5hKlzh.js");
const HOST_CSS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/host-B6RM5CLf.css");

pub(crate) struct BrowserHostRuntime {
    session: Arc<Mutex<BrowserHostSession>>,
    island: Arc<Mutex<BrowserIslandState>>,
    webview: WebView,
    pointer_capture: Mutex<PlatformPointerCapture>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestedFocusOwner {
    Parent,
    Browser,
}

#[derive(Debug)]
struct BrowserIslandState {
    instance_epoch: u64,
    initial_projection_ready: bool,
    latest_layout_epoch: Option<u64>,
    requested_focus_owner: Option<RequestedFocusOwner>,
}

impl BrowserIslandState {
    fn new(instance_epoch: u64) -> Self {
        Self {
            instance_epoch,
            initial_projection_ready: false,
            latest_layout_epoch: None,
            requested_focus_owner: None,
        }
    }

    fn observe_initial_projection(&mut self, instance_epoch: u64) -> bool {
        if instance_epoch != self.instance_epoch || self.initial_projection_ready {
            return false;
        }
        self.initial_projection_ready = true;
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

    fn should_transfer_focus(&self, owner: RequestedFocusOwner) -> bool {
        self.can_transfer_focus() && self.requested_focus_owner != Some(owner)
    }

    fn needs_initial_focus(&self) -> bool {
        self.can_transfer_focus() && self.requested_focus_owner.is_none()
    }

    fn commit_focus(&mut self, owner: RequestedFocusOwner) {
        self.requested_focus_owner = Some(owner);
    }
}

impl BrowserHostRuntime {
    pub(crate) fn new(
        window: &winit::window::Window,
        wake: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<Self, BrowserHostRuntimeError> {
        let elapsed = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let epoch = u64::try_from(elapsed.as_millis())
            .map_err(|_| BrowserHostRuntimeError::EpochOverflow)?;
        let source = BrowserPlaceIntent {
            // scopeはHost sessionが発行するopaque identityで、表示値から導かない。
            scope_ref: format!("builtin-{epoch}"),
            item_id: "rectangle".to_owned(),
        };
        let session = BrowserHostSession::new(epoch, 0, source);
        let snapshot = session.snapshot_json()?;
        let encoded_snapshot = serde_json::to_string(&snapshot)?;
        let initialization_script = format!(
            r#"window.__MOTOLII_BUILTIN_HOST__=Object.freeze({{
snapshot:JSON.parse({encoded_snapshot}),
postMessage:(message)=>window.ipc.postMessage(message)
}});"#
        );
        let session = Arc::new(Mutex::new(session));
        let island = Arc::new(Mutex::new(BrowserIslandState::new(epoch)));
        let callback_session = Arc::clone(&session);
        let callback_island = Arc::clone(&island);
        let load_island = Arc::clone(&island);
        let load_wake = Arc::clone(&wake);
        let webview = WebViewBuilder::new()
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
                if matches!(event, PageLoadEvent::Finished) && url == ENTRY_URL {
                    let changed = load_island
                        .lock()
                        .map(|mut island| island.observe_initial_projection(epoch))
                        .unwrap_or(false);
                    if changed {
                        load_wake();
                    }
                }
            })
            .with_ipc_handler(move |request| {
                let raw = request.body();
                match callback_session.lock() {
                    Ok(mut session) => match session.accept(raw) {
                        Ok(()) => {
                            if let Ok(mut island) = callback_island.lock() {
                                island.observe_initial_projection(epoch);
                            }
                            wake();
                        }
                        Err(error) => eprintln!("Browser Host rejected message: {error}"),
                    },
                    Err(_) => eprintln!("Browser Host inbox lock is poisoned"),
                }
            })
            .build_as_child(window)?;
        let pointer_capture = Mutex::new(PlatformPointerCapture::new(window)?);
        Ok(Self {
            session,
            island,
            webview,
            pointer_capture,
        })
    }

    pub(crate) fn take_place_intent(
        &self,
    ) -> Result<Option<BrowserPlaceIntent>, BrowserHostRuntimeError> {
        let intent = self
            .session
            .lock()
            .map_err(|_| BrowserHostRuntimeError::InboxPoisoned)
            .map(|mut session| session.pop())?;
        if intent.is_some() {
            self.transfer_focus(RequestedFocusOwner::Parent)?;
            self.pointer_capture
                .lock()
                .map_err(|_| BrowserHostRuntimeError::PointerCapturePoisoned)?
                .arm();
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

    pub(crate) fn ensure_initial_focus(&self) -> Result<(), BrowserHostRuntimeError> {
        let should_focus = self
            .island
            .lock()
            .map_err(|_| BrowserHostRuntimeError::IslandStatePoisoned)?
            .needs_initial_focus();
        if should_focus {
            self.transfer_focus(RequestedFocusOwner::Browser)?;
        }
        Ok(())
    }

    pub(crate) fn set_bounds(
        &self,
        layout_epoch: u64,
        rect: LogicalRect,
    ) -> Result<(), BrowserHostRuntimeError> {
        if !self
            .island
            .lock()
            .map_err(|_| BrowserHostRuntimeError::IslandStatePoisoned)?
            .should_apply_layout(layout_epoch)
        {
            return Ok(());
        }
        self.webview.set_bounds(Rect {
            position: wry::dpi::LogicalPosition::new(rect.x, rect.y).into(),
            size: wry::dpi::LogicalSize::new(rect.width, rect.height).into(),
        })?;
        self.island
            .lock()
            .map_err(|_| BrowserHostRuntimeError::IslandStatePoisoned)?
            .commit_layout(layout_epoch);
        Ok(())
    }

    fn transfer_focus(&self, owner: RequestedFocusOwner) -> Result<(), BrowserHostRuntimeError> {
        if !self
            .island
            .lock()
            .map_err(|_| BrowserHostRuntimeError::IslandStatePoisoned)?
            .can_transfer_focus()
        {
            return Err(BrowserHostRuntimeError::InitialProjectionNotReady);
        }
        if !self
            .island
            .lock()
            .map_err(|_| BrowserHostRuntimeError::IslandStatePoisoned)?
            .should_transfer_focus(owner)
        {
            return Ok(());
        }
        match owner {
            RequestedFocusOwner::Parent => self.webview.focus_parent()?,
            RequestedFocusOwner::Browser => self.webview.focus()?,
        }
        self.island
            .lock()
            .map_err(|_| BrowserHostRuntimeError::IslandStatePoisoned)?
            .commit_focus(owner);
        Ok(())
    }
}

fn product_asset_response(path: &str) -> Response<Cow<'static, [u8]>> {
    let (content_type, body) = match path {
        "/" | "/host.html" => ("text/html; charset=utf-8", HOST_HTML),
        "/assets/host-Ur5hKlzh.js" => ("text/javascript; charset=utf-8", HOST_JS),
        "/assets/host-B6RM5CLf.css" => ("text/css; charset=utf-8", HOST_CSS),
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
    #[error("Browser Host island state lock is poisoned")]
    IslandStatePoisoned,
    #[error("Browser Host initial projection is not ready for focus transfer")]
    InitialProjectionNotReady,
    #[error(transparent)]
    PointerCapture(#[from] PlatformPointerCaptureError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_host_html_references_the_embedded_assets() {
        let html = std::str::from_utf8(HOST_HTML).expect("generated Host HTML is UTF-8");

        for path in ["/assets/host-Ur5hKlzh.js", "/assets/host-B6RM5CLf.css"] {
            assert!(html.contains(path));
            let response = product_asset_response(path);
            assert_eq!(response.status(), 200);
            assert!(!response.body().is_empty());
        }
    }

    #[test]
    fn product_asset_response_rejects_unknown_paths() {
        let response = product_asset_response("/assets/host-stale.js");

        assert_eq!(response.status(), 404);
        assert!(response.body().is_empty());
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
        assert!(island.needs_initial_focus());
    }

    #[test]
    fn focus_state_records_only_an_explicit_host_request() {
        let mut island = BrowserIslandState::new(7);

        assert_eq!(island.requested_focus_owner, None);
        island.commit_focus(RequestedFocusOwner::Parent);
        assert!(!island.needs_initial_focus());
        assert_eq!(
            island.requested_focus_owner,
            Some(RequestedFocusOwner::Parent)
        );
        island.commit_focus(RequestedFocusOwner::Browser);
        assert_eq!(
            island.requested_focus_owner,
            Some(RequestedFocusOwner::Browser)
        );
    }
}
