//! 確定済みStage React chromeをnative viewportの上下へ載せるprivate Host。

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use wry::{Rect, WebView, WebViewBuilder};

use crate::browser_host_runtime::product_asset_response;
use crate::native_host_layout::LogicalRect;

const PROTOCOL: &str = "motolii-stage";
const HEADER_URL: &str = "motolii-stage://product/stage-header.html";
const TRANSPORT_URL: &str = "motolii-stage://product/stage-transport.html";

pub(crate) struct StageChromeHostRuntime {
    header: WebView,
    transport: WebView,
    latest_layout_epoch: Option<u64>,
    easing_inbox: Arc<Mutex<StageEasingInbox>>,
    wake: Arc<Mutex<Option<StageEasingWake>>>,
}

type StageEasingWake = Arc<dyn Fn() + Send + Sync>;
type StageEasingCallback = (
    Arc<Mutex<StageEasingInbox>>,
    Arc<Mutex<Option<StageEasingWake>>>,
);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct StageEasingIntent {
    pub(crate) anchor: LogicalRect,
    pub(crate) layout_epoch: u64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StageEasingMessage {
    kind: StageEasingKind,
    anchor: StageEasingAnchor,
    #[serde(rename = "layoutEpoch")]
    layout_epoch: u64,
}

#[derive(Debug, serde::Deserialize)]
enum StageEasingKind {
    #[serde(rename = "open-position-easing")]
    OpenPositionEasing,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StageEasingAnchor {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Debug, Default)]
struct StageEasingInbox {
    pending: VecDeque<StageEasingIntent>,
}

impl StageEasingInbox {
    fn accept(&mut self, raw: &str) -> Result<(), StageEasingIntentError> {
        let message: StageEasingMessage = serde_json::from_str(raw)?;
        let _ = message.kind;
        let anchor = LogicalRect {
            x: message.anchor.x,
            y: message.anchor.y,
            width: message.anchor.width,
            height: message.anchor.height,
        };
        if message.layout_epoch == 0
            || !anchor.x.is_finite()
            || !anchor.y.is_finite()
            || !anchor.width.is_finite()
            || !anchor.height.is_finite()
            || anchor.width < 0.0
            || anchor.height < 0.0
        {
            return Err(StageEasingIntentError::Invalid);
        }
        if !self.pending.is_empty() {
            return Err(StageEasingIntentError::InboxFull);
        }
        self.pending.push_back(StageEasingIntent {
            anchor,
            layout_epoch: message.layout_epoch,
        });
        Ok(())
    }

    fn take(&mut self) -> Option<StageEasingIntent> {
        self.pending.pop_front()
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum StageEasingIntentError {
    #[error("Stage Easing intent JSON failed")]
    Json(#[from] serde_json::Error),
    #[error("Stage Easing intent is invalid")]
    Invalid,
    #[error("Stage Easing inbox is full")]
    InboxFull,
    #[error("Stage Easing inbox is poisoned")]
    InboxPoisoned,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub(crate) struct StageTransportSnapshot {
    mode: &'static str,
    timecode: &'static str,
    #[serde(rename = "barPosition")]
    bar_position: &'static str,
    #[serde(rename = "tempoStatus")]
    tempo_status: &'static str,
    #[serde(rename = "qualityStatus")]
    quality_status: &'static str,
    #[serde(rename = "activeInterval")]
    active_interval: Option<StageActiveInterval>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
struct StageActiveInterval {
    #[serde(rename = "objectName")]
    object_name: String,
    channel: &'static str,
}

impl StageTransportSnapshot {
    pub(crate) fn with_position_active_interval(object_name: Option<String>) -> Self {
        Self {
            mode: "RECTANGLE",
            timecode: "00:00.0",
            bar_position: "BAR 0.0.00",
            tempo_status: "120 BPM · SNAP BEAT",
            quality_status: "DRAFT · FP16 · 1/2",
            active_interval: object_name
                .filter(|name| !name.is_empty())
                .map(|object_name| StageActiveInterval {
                    object_name,
                    channel: "Position",
                }),
        }
    }
}

impl StageChromeHostRuntime {
    pub(crate) fn new(
        window: &winit::window::Window,
        snapshot: &StageTransportSnapshot,
    ) -> Result<Self, StageChromeHostRuntimeError> {
        let created_at = std::time::Instant::now();
        let header_initialization_script = r#"window.__MOTOLII_STAGE_HOST__=Object.freeze({
snapshot:Object.freeze({
mode:"RECTANGLE",
timecode:"00:00.0",
barPosition:"BAR 0.0.00",
tempoStatus:"120 BPM · SNAP BEAT",
qualityStatus:"DRAFT · FP16 · 1/2"
})
});"#;
        let encoded_snapshot = javascript_json_parse_argument(snapshot)?;
        let transport_initialization_script = format!(
            r#"window.__MOTOLII_STAGE_HOST__=(()=>{{
let listener=null;
let current=JSON.parse({encoded_snapshot});
return Object.freeze({{
get snapshot(){{return current;}},
subscribe:(next)=>{{if(typeof next!=="function"||listener!==null)throw new TypeError("invalid Stage transport subscriber");listener=next;}},
publish:(next)=>{{current=next;if(listener!==null)listener(current);}}
}});
}})();"#
        );
        let easing_inbox = Arc::new(Mutex::new(StageEasingInbox::default()));
        let wake = Arc::new(Mutex::new(None::<StageEasingWake>));
        let callback_inbox = Arc::clone(&easing_inbox);
        let callback_wake = Arc::clone(&wake);
        let header = build_stage_webview(
            window,
            HEADER_URL,
            header_initialization_script,
            "stage-header",
            None,
        )?;
        let transport = build_stage_webview(
            window,
            TRANSPORT_URL,
            &transport_initialization_script,
            "stage-transport",
            Some((callback_inbox, callback_wake)),
        )?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=stage-chrome event=created elapsed_ms={:.3}",
            created_at.elapsed().as_secs_f64() * 1_000.0,
        ));
        Ok(Self {
            header,
            transport,
            latest_layout_epoch: None,
            easing_inbox,
            wake,
        })
    }

    pub(crate) fn publish(
        &self,
        snapshot: &StageTransportSnapshot,
    ) -> Result<(), StageChromeHostRuntimeError> {
        let encoded_snapshot = javascript_json_parse_argument(snapshot)?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=stage-transport event=publish payload_bytes={}",
            encoded_snapshot.len(),
        ));
        self.transport.evaluate_script(&format!(
            "window.__MOTOLII_STAGE_HOST__.publish(JSON.parse({encoded_snapshot}));"
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
        self.transport.evaluate_script(&format!(
            "window.__MOTOLII_STAGE_EASING__=Object.freeze({{layoutEpoch:{layout_epoch},postMessage:(message)=>window.ipc.postMessage(message)}});"
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

    pub(crate) fn register_easing_wake(
        &self,
        wake: StageEasingWake,
    ) -> Result<(), StageEasingIntentError> {
        *self
            .wake
            .lock()
            .map_err(|_| StageEasingIntentError::InboxPoisoned)? = Some(wake);
        Ok(())
    }

    pub(crate) fn take_easing_intent(
        &self,
    ) -> Result<Option<StageEasingIntent>, StageEasingIntentError> {
        Ok(self
            .easing_inbox
            .lock()
            .map_err(|_| StageEasingIntentError::InboxPoisoned)?
            .take())
    }
}

fn javascript_json_parse_argument<T: serde::Serialize>(
    snapshot: &T,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(&serde_json::to_string(snapshot)?)
}

fn build_stage_webview(
    window: &winit::window::Window,
    entry_url: &'static str,
    initialization_script: &str,
    surface: &'static str,
    easing: Option<StageEasingCallback>,
) -> Result<WebView, wry::Error> {
    let builder = WebViewBuilder::new()
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
    let builder = if let Some((inbox, wake)) = easing {
        builder.with_ipc_handler(move |request| {
            let accepted = inbox
                .lock()
                .map_err(|_| StageEasingIntentError::InboxPoisoned)
                .and_then(|mut inbox| inbox.accept(request.body()));
            if accepted.is_ok() {
                if let Ok(wake) = wake.lock() {
                    if let Some(wake) = wake.as_ref() {
                        wake();
                    }
                }
            }
        })
    } else {
        builder
    };
    builder.build_as_child(window)
}

fn set_webview_bounds(webview: &WebView, rect: LogicalRect) -> Result<(), wry::Error> {
    webview.set_bounds(Rect {
        position: wry::dpi::LogicalPosition::new(rect.x, rect.y).into(),
        size: wry::dpi::LogicalSize::new(rect.width, rect.height).into(),
    })
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum StageChromeHostRuntimeError {
    #[error("Stage chrome WebView failed")]
    WebView(#[from] wry::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_transport_snapshot_projects_only_non_empty_position_interval_names() {
        let active = serde_json::to_value(StageTransportSnapshot::with_position_active_interval(
            Some("Rectangle".to_owned()),
        ))
        .unwrap();
        assert_eq!(
            active,
            serde_json::json!({
                "mode": "RECTANGLE",
                "timecode": "00:00.0",
                "barPosition": "BAR 0.0.00",
                "tempoStatus": "120 BPM · SNAP BEAT",
                "qualityStatus": "DRAFT · FP16 · 1/2",
                "activeInterval": { "objectName": "Rectangle", "channel": "Position" }
            })
        );
        for name in [None, Some(String::new())] {
            assert_eq!(
                serde_json::to_value(StageTransportSnapshot::with_position_active_interval(name))
                    .unwrap()["activeInterval"],
                serde_json::Value::Null,
            );
        }
    }
}
