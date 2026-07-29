//! product-owned Browser bundleをnative shellのopaque childへ載せるHost。

use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use wry::http::{header::CONTENT_TYPE, Response};
use wry::{NewWindowResponse, Rect, WebView, WebViewBuilder};

use crate::browser_host::{BrowserHostSession, BrowserPlaceIntent};
use crate::host_pointer_capture::{
    HostPointerCandidate, PlatformPointerCapture, PlatformPointerCaptureError,
};

const PROTOCOL: &str = "motolii-browser";
const ENTRY_URL: &str = "motolii-browser://product/host.html";
const HOST_HTML: &[u8] = include_bytes!("../../../ui/motolii-web/generated-host/host.html");
const HOST_JS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/host-Ur5hKlzh.js");
const HOST_CSS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/host-B6RM5CLf.css");

pub(crate) struct BrowserHostRuntime {
    session: Arc<Mutex<BrowserHostSession>>,
    webview: WebView,
    pointer_capture: Mutex<PlatformPointerCapture>,
}

impl BrowserHostRuntime {
    pub(crate) fn new(
        window: &winit::window::Window,
        repaint_context: egui::Context,
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
        let callback_session = Arc::clone(&session);
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
            .with_ipc_handler(move |request| {
                let raw = request.body();
                match callback_session.lock() {
                    Ok(mut session) => match session.accept(raw) {
                        Ok(()) => repaint_context.request_repaint(),
                        Err(error) => eprintln!("Browser Host rejected message: {error}"),
                    },
                    Err(_) => eprintln!("Browser Host inbox lock is poisoned"),
                }
            })
            .build_as_child(window)?;
        let pointer_capture = Mutex::new(PlatformPointerCapture::new(window)?);
        Ok(Self {
            session,
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

    pub(crate) fn set_bounds(&self, rect: egui::Rect) -> Result<(), BrowserHostRuntimeError> {
        self.webview.set_bounds(Rect {
            position: wry::dpi::LogicalPosition::new(f64::from(rect.left()), f64::from(rect.top()))
                .into(),
            size: wry::dpi::LogicalSize::new(f64::from(rect.width()), f64::from(rect.height()))
                .into(),
        })?;
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
}
