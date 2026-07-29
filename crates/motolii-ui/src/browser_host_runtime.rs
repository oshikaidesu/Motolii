//! product-owned Browser bundleをnative shellのopaque childへ載せるHost。

use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use wry::http::{header::CONTENT_TYPE, Response};
use wry::{NewWindowResponse, Rect, WebView, WebViewBuilder};

use crate::browser_host::{BrowserHostSession, BrowserPlaceIntent};

const PROTOCOL: &str = "motolii-browser";
const ENTRY_URL: &str = "motolii-browser://product/host.html";
const HOST_HTML: &[u8] = include_bytes!("../../../ui/motolii-web/generated-host/host.html");
const HOST_JS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/host-CRv7Qcif.js");
const HOST_CSS: &[u8] =
    include_bytes!("../../../ui/motolii-web/generated-host/assets/host-Dbykilq0.css");

pub(crate) struct BrowserHostRuntime {
    _session: Arc<Mutex<BrowserHostSession>>,
    webview: WebView,
}

impl BrowserHostRuntime {
    pub(crate) fn new(window: &winit::window::Window) -> Result<Self, BrowserHostRuntimeError> {
        let elapsed = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let epoch = u64::try_from(elapsed.as_millis())
            .map_err(|_| BrowserHostRuntimeError::EpochOverflow)?;
        let source = BrowserPlaceIntent {
            // scopeはHost sessionが発行するopaque identityで、表示値から導かない。
            scope_ref: format!("builtin-{epoch}"),
            item_id: "rectangle".to_owned(),
        };
        let session = BrowserHostSession::new(epoch, 0);
        let snapshot = session.snapshot_json(&source)?;
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
                    Ok(mut session) => {
                        if let Err(error) = session.accept(raw) {
                            eprintln!("Browser Host rejected message: {error}");
                        }
                    }
                    Err(_) => eprintln!("Browser Host inbox lock is poisoned"),
                }
            })
            .build_as_child(window)?;
        Ok(Self {
            _session: session,
            webview,
        })
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
        "/assets/host-CRv7Qcif.js" => ("text/javascript; charset=utf-8", HOST_JS),
        "/assets/host-Dbykilq0.css" => ("text/css; charset=utf-8", HOST_CSS),
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
}
