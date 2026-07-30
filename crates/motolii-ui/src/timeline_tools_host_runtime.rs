//! 確定済みReact Key Toolsをnative Timeline左端へ載せるprivate Host。

use wry::{Rect, WebView, WebViewBuilder};

use crate::browser_host_runtime::product_asset_response;
use crate::native_host_layout::LogicalRect;

const PROTOCOL: &str = "motolii-timeline-tools";
const ENTRY_URL: &str = "motolii-timeline-tools://product/timeline-tools.html";

pub(crate) struct TimelineToolsHostRuntime {
    webview: WebView,
    latest_layout_epoch: Option<u64>,
}

impl TimelineToolsHostRuntime {
    pub(crate) fn new(
        window: &winit::window::Window,
        layer_count: usize,
        key_count: usize,
    ) -> Result<Self, TimelineToolsHostRuntimeError> {
        let snapshot = snapshot_json(layer_count, key_count);
        let encoded = javascript_json_parse_argument(&snapshot)?;
        let initialization_script = format!(
            r#"window.__MOTOLII_TIMELINE_TOOLS_HOST__=(()=>{{
let listener=null;
let current=JSON.parse({encoded});
return Object.freeze({{
get snapshot(){{return current;}},
subscribe:(next)=>{{if(typeof next!=="function"||listener!==null)throw new TypeError("invalid Timeline Tools subscriber");listener=next;listener(current);return()=>{{listener=null;}};}},
publish:(next)=>{{current=next;if(listener!==null)listener(current);}},
reportUnavailable:(target,operation)=>window.ipc.postMessage(JSON.stringify({{kind:"operation-unavailable",target,operation}}))
}});
}})();"#
        );
        let webview = WebViewBuilder::new()
            .with_bounds(Rect {
                position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
                size: wry::dpi::LogicalSize::new(1.0, 1.0).into(),
            })
            .with_initialization_script(&initialization_script)
            .with_ipc_handler(|request| {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=webview surface=timeline-tools event=ipc payload_bytes={}",
                    request.body().len(),
                ));
            })
            .with_custom_protocol(PROTOCOL.to_owned(), move |_webview_id, request| {
                product_asset_response(request.uri().path())
            })
            .with_url(ENTRY_URL)
            .with_navigation_handler(|target| target.starts_with("motolii-timeline-tools:"))
            .build_as_child(window)?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=timeline-tools event=created layers={} keys={}",
            layer_count, key_count,
        ));
        Ok(Self {
            webview,
            latest_layout_epoch: None,
        })
    }

    pub(crate) fn set_bounds(
        &mut self,
        layout_epoch: u64,
        rect: LogicalRect,
    ) -> Result<(), TimelineToolsHostRuntimeError> {
        if self
            .latest_layout_epoch
            .is_some_and(|latest| layout_epoch <= latest)
        {
            return Ok(());
        }
        self.webview.set_bounds(Rect {
            position: wry::dpi::LogicalPosition::new(rect.x, rect.y).into(),
            size: wry::dpi::LogicalSize::new(rect.width, rect.height).into(),
        })?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=timeline-tools event=bounds layout_epoch={} x={:.3} y={:.3} width={:.3} height={:.3}",
            layout_epoch, rect.x, rect.y, rect.width, rect.height,
        ));
        self.latest_layout_epoch = Some(layout_epoch);
        Ok(())
    }

    pub(crate) fn publish(
        &self,
        layer_count: usize,
        key_count: usize,
    ) -> Result<(), TimelineToolsHostRuntimeError> {
        let encoded = javascript_json_parse_argument(&snapshot_json(layer_count, key_count))?;
        self.webview.evaluate_script(&format!(
            "window.__MOTOLII_TIMELINE_TOOLS_HOST__.publish(JSON.parse({encoded}));"
        ))?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=timeline-tools event=publish layers={} keys={}",
            layer_count, key_count,
        ));
        Ok(())
    }
}

fn snapshot_json(layer_count: usize, key_count: usize) -> serde_json::Value {
    serde_json::json!({ "layerCount": layer_count, "keyCount": key_count })
}

fn javascript_json_parse_argument(
    snapshot: &serde_json::Value,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(&serde_json::to_string(snapshot)?)
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TimelineToolsHostRuntimeError {
    #[error("Timeline Tools Host JSON could not be encoded")]
    Json(#[from] serde_json::Error),
    #[error("Timeline Tools Host WebView failed")]
    WebView(#[from] wry::Error),
}
