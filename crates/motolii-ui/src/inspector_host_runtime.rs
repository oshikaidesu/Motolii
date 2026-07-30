//! product-owned Inspectorをnative shellのopaque childへ載せるprivate Host。

use wry::{Rect, WebView, WebViewBuilder};

use crate::browser_host_runtime::product_asset_response;
use crate::native_host_layout::LogicalRect;

const PROTOCOL: &str = "motolii-inspector";
const ENTRY_URL: &str = "motolii-inspector://product/inspector.html";

pub(crate) struct InspectorHostRuntime {
    webview: WebView,
    latest_layout_epoch: Option<u64>,
}

impl InspectorHostRuntime {
    pub(crate) fn new(
        window: &winit::window::Window,
        document: &motolii_doc::Document,
        primary: Option<motolii_doc::LayerId>,
    ) -> Result<Self, InspectorHostRuntimeError> {
        let snapshot = snapshot_json(document, primary)?;
        let encoded_snapshot = javascript_json_parse_argument(&snapshot)?;
        let initialization_script = format!(
            r#"window.__MOTOLII_INSPECTOR_HOST__=(()=>{{
let listener=null;
let current=JSON.parse({encoded_snapshot});
return Object.freeze({{
get snapshot(){{return current;}},
subscribe:(next)=>{{if(typeof next!=="function"||listener!==null)throw new TypeError("invalid Inspector subscriber");listener=next;listener(current);}},
publish:(next)=>{{current=next;if(listener!==null)listener(current);}}
}});
}})();"#
        );
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
            .with_navigation_handler(|target| target.starts_with("motolii-inspector:"))
            .build_as_child(window)?;
        Ok(Self {
            webview,
            latest_layout_epoch: None,
        })
    }

    pub(crate) fn set_bounds(
        &mut self,
        layout_epoch: u64,
        rect: impl Into<Option<LogicalRect>>,
    ) -> Result<(), InspectorHostRuntimeError> {
        if self
            .latest_layout_epoch
            .is_some_and(|latest| layout_epoch <= latest)
        {
            return Ok(());
        }
        let rect = rect.into();
        if let Some(rect) = rect {
            self.webview.set_bounds(Rect {
                position: wry::dpi::LogicalPosition::new(rect.x, rect.y).into(),
                size: wry::dpi::LogicalSize::new(rect.width, rect.height).into(),
            })?;
        }
        self.webview.set_visible(rect.is_some())?;
        self.latest_layout_epoch = Some(layout_epoch);
        Ok(())
    }

    pub(crate) fn publish(
        &self,
        document: &motolii_doc::Document,
        primary: Option<motolii_doc::LayerId>,
    ) -> Result<(), InspectorHostRuntimeError> {
        let snapshot = snapshot_json(document, primary)?;
        let encoded_snapshot = javascript_json_parse_argument(&snapshot)?;
        self.webview.evaluate_script(&format!(
            "window.__MOTOLII_INSPECTOR_HOST__.publish(JSON.parse({encoded_snapshot}));"
        ))?;
        Ok(())
    }
}

fn javascript_json_parse_argument(
    snapshot: &serde_json::Value,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(&serde_json::to_string(snapshot)?)
}

fn snapshot_json(
    document: &motolii_doc::Document,
    primary: Option<motolii_doc::LayerId>,
) -> Result<serde_json::Value, serde_json::Error> {
    let Some(primary) = primary else {
        return Ok(serde_json::Value::Null);
    };
    Ok(serde_json::json!({
        "fixtureRevision": 1,
        "document": document,
        "nodes": [],
        "target": { "layer_id": primary },
    }))
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum InspectorHostRuntimeError {
    #[error("Inspector Host JSON could not be encoded")]
    Json(#[from] serde_json::Error),
    #[error("Inspector Host WebView failed")]
    WebView(#[from] wry::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_primary_projects_null_without_inventing_selection() {
        let document = motolii_doc::Document::new_current();
        assert_eq!(
            snapshot_json(&document, None).unwrap(),
            serde_json::Value::Null
        );
    }

    #[test]
    fn bridge_argument_is_a_quoted_json_string_not_an_object_literal() {
        let encoded = javascript_json_parse_argument(&serde_json::json!({"target": 7})).unwrap();
        assert_eq!(encoded, r#""{\"target\":7}""#);
    }
}
