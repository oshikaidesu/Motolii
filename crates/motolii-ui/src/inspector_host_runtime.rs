//! product-owned Inspectorをnative shellのopaque childへ載せるprivate Host。

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use wry::{Rect, WebView, WebViewBuilder};

use crate::browser_host_runtime::product_asset_response;
use crate::document_edit_runtime::SetEffectParamRequest;
use crate::native_host_layout::LogicalRect;
use crate::{map_parameter_control, HostParameterControl};

const PROTOCOL: &str = "motolii-inspector";
const ENTRY_URL: &str = "motolii-inspector://product/inspector.html";
const MAX_PENDING_TERMINALS: usize = 64;
const MAX_PENDING_POSITION_KEYS: usize = 8;

type InspectorWake = Arc<dyn Fn() + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub(crate) struct InspectorPositionControl {
    pub(crate) layer_id: motolii_doc::LayerId,
    pub(crate) projection_generation: u64,
    pub(crate) key_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AddPositionKeyIntent {
    pub(crate) layer_id: motolii_doc::LayerId,
    pub(crate) projection_generation: u64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AddPositionKeyMessage {
    #[serde(rename = "kind")]
    _kind: AddPositionKeyKind,
    layer_id: motolii_doc::LayerId,
    projection_generation: u64,
}

#[derive(Debug, serde::Deserialize)]
enum AddPositionKeyKind {
    #[serde(rename = "add-position-key")]
    AddPositionKey,
}

#[derive(Debug)]
struct AddPositionKeyInbox {
    expected: Option<AddPositionKeyIntent>,
    pending: VecDeque<AddPositionKeyIntent>,
}

impl AddPositionKeyInbox {
    fn new(expected: Option<AddPositionKeyIntent>) -> Self {
        Self {
            expected,
            pending: VecDeque::new(),
        }
    }

    fn reconcile(&mut self, expected: Option<AddPositionKeyIntent>) {
        if self.expected != expected {
            self.expected = expected;
            self.pending.clear();
        }
    }

    fn accept(&mut self, message: AddPositionKeyMessage) -> Result<(), AddPositionKeyIntentError> {
        let intent = AddPositionKeyIntent {
            layer_id: message.layer_id,
            projection_generation: message.projection_generation,
        };
        if self.expected != Some(intent) {
            return Err(AddPositionKeyIntentError::IdentityMismatch);
        }
        if self.pending.len() >= MAX_PENDING_POSITION_KEYS {
            return Err(AddPositionKeyIntentError::InboxFull);
        }
        self.pending.push_back(intent);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InspectorGestureUpdate {
    pub(crate) session: u64,
    pub(crate) sequence: u64,
    pub(crate) identity: InspectorGestureIdentity,
    pub(crate) value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InspectorGestureTerminal {
    pub(crate) session: u64,
    pub(crate) sequence: u64,
    pub(crate) identity: InspectorGestureIdentity,
    pub(crate) cause: InspectorGestureTerminalCause,
}

impl InspectorGestureTerminal {
    pub(crate) fn into_set_effect_param_request(self) -> Option<SetEffectParamRequest> {
        match self.cause {
            InspectorGestureTerminalCause::Cancel => None,
            InspectorGestureTerminalCause::Commit(value) => Some(SetEffectParamRequest::new(
                self.identity.layer_id,
                self.identity.effect_use_id,
                self.identity.definition_id,
                self.identity.plugin_id,
                self.identity.effect_version,
                self.identity.param_id,
                value,
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum InspectorGestureTerminalCause {
    Commit(f64),
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InspectorGestureIdentity {
    layer_id: motolii_doc::LayerId,
    effect_use_id: motolii_doc::EffectId,
    definition_id: motolii_doc::EffectDefinitionId,
    plugin_id: String,
    effect_version: u32,
    param_id: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InspectorGestureMessage {
    #[serde(rename = "kind")]
    _kind: InspectorGestureKind,
    phase: InspectorGesturePhase,
    session: u64,
    sequence: u64,
    layer_id: motolii_doc::LayerId,
    effect_use_id: motolii_doc::EffectId,
    definition_id: motolii_doc::EffectDefinitionId,
    plugin_id: String,
    effect_version: u32,
    param_id: String,
    #[serde(default, deserialize_with = "deserialize_gesture_value")]
    value: InspectorGestureValue,
}

#[derive(Debug, Default)]
enum InspectorGestureValue {
    #[default]
    Missing,
    Null,
    Number(f64),
}

fn deserialize_gesture_value<'de, D>(deserializer: D) -> Result<InspectorGestureValue, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(
        match <Option<f64> as serde::Deserialize>::deserialize(deserializer)? {
            Some(value) => InspectorGestureValue::Number(value),
            None => InspectorGestureValue::Null,
        },
    )
}

impl InspectorGestureMessage {
    fn identity(&self) -> InspectorGestureIdentity {
        InspectorGestureIdentity {
            layer_id: self.layer_id,
            effect_use_id: self.effect_use_id,
            definition_id: self.definition_id,
            plugin_id: self.plugin_id.clone(),
            effect_version: self.effect_version,
            param_id: self.param_id.clone(),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
enum InspectorGestureKind {
    #[serde(rename = "effect-param-gesture")]
    EffectParamGesture,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum InspectorGesturePhase {
    Start,
    Update,
    Commit,
    Cancel,
}

#[derive(Debug)]
struct ActiveInspectorGesture {
    session: u64,
    last_sequence: u64,
}

#[derive(Debug)]
struct InspectorGestureInbox {
    expected: Option<InspectorGestureIdentity>,
    last_session: u64,
    active: Option<ActiveInspectorGesture>,
    latest_update: Option<InspectorGestureUpdate>,
    terminals: VecDeque<InspectorGestureTerminal>,
}

impl InspectorGestureInbox {
    fn new(expected: Option<InspectorGestureIdentity>) -> Self {
        Self {
            expected,
            last_session: 0,
            active: None,
            latest_update: None,
            terminals: VecDeque::new(),
        }
    }

    fn reconcile(&mut self, expected: Option<InspectorGestureIdentity>) {
        if self.expected != expected {
            self.expected = expected;
            self.active = None;
            self.latest_update = None;
        }
    }

    fn accept(&mut self, raw: &str) -> Result<(), InspectorGestureError> {
        let message: InspectorGestureMessage = serde_json::from_str(raw)?;
        let Some(expected) = &self.expected else {
            return Err(InspectorGestureError::NoActiveTarget);
        };
        let identity = message.identity();
        if &identity != expected {
            return Err(InspectorGestureError::IdentityMismatch);
        }
        if message.session == 0 || message.sequence == 0 {
            return Err(InspectorGestureError::NonPositiveOrder);
        }
        let value = match (message.phase, message.value) {
            (InspectorGesturePhase::Cancel, InspectorGestureValue::Missing) => None,
            (InspectorGesturePhase::Cancel, _) => {
                return Err(InspectorGestureError::UnexpectedValue)
            }
            (_, InspectorGestureValue::Number(value))
                if value.is_finite() && (0.0..=1.0).contains(&value) =>
            {
                Some(value)
            }
            _ => return Err(InspectorGestureError::InvalidValue),
        };

        match message.phase {
            InspectorGesturePhase::Start => {
                if self.active.is_some() {
                    return Err(InspectorGestureError::GestureAlreadyActive);
                }
                if message.session <= self.last_session || message.sequence != 1 {
                    return Err(InspectorGestureError::StaleOrReordered);
                }
                self.last_session = message.session;
                self.active = Some(ActiveInspectorGesture {
                    session: message.session,
                    last_sequence: message.sequence,
                });
            }
            InspectorGesturePhase::Update => {
                self.advance(message.session, message.sequence)?;
                self.latest_update = Some(InspectorGestureUpdate {
                    session: message.session,
                    sequence: message.sequence,
                    identity,
                    value: value.expect("validated update value"),
                });
            }
            InspectorGesturePhase::Commit | InspectorGesturePhase::Cancel => {
                if self.terminals.len() >= MAX_PENDING_TERMINALS {
                    return Err(InspectorGestureError::TerminalInboxFull);
                }
                self.advance(message.session, message.sequence)?;
                self.latest_update = None;
                self.active = None;
                self.terminals.push_back(InspectorGestureTerminal {
                    session: message.session,
                    sequence: message.sequence,
                    identity,
                    cause: match message.phase {
                        InspectorGesturePhase::Commit => InspectorGestureTerminalCause::Commit(
                            value.expect("validated commit value"),
                        ),
                        InspectorGesturePhase::Cancel => InspectorGestureTerminalCause::Cancel,
                        _ => unreachable!(),
                    },
                });
            }
        }
        Ok(())
    }

    fn advance(&mut self, session: u64, sequence: u64) -> Result<(), InspectorGestureError> {
        let Some(active) = &mut self.active else {
            return Err(InspectorGestureError::GestureNotActive);
        };
        if active.session != session || sequence <= active.last_sequence {
            return Err(InspectorGestureError::StaleOrReordered);
        }
        active.last_sequence = sequence;
        Ok(())
    }
}

pub(crate) struct InspectorHostRuntime {
    gesture_inbox: Arc<Mutex<InspectorGestureInbox>>,
    position_key_inbox: Arc<Mutex<AddPositionKeyInbox>>,
    wake: Arc<Mutex<Option<InspectorWake>>>,
    webview: WebView,
    latest_layout_epoch: Option<u64>,
}

impl InspectorHostRuntime {
    pub(crate) fn new(
        window: &winit::window::Window,
        document: &motolii_doc::Document,
        primary: Option<motolii_doc::LayerId>,
        active_effect_use: Option<motolii_doc::EffectId>,
    ) -> Result<Self, InspectorHostRuntimeError> {
        let created_at = std::time::Instant::now();
        let snapshot = snapshot_json(document, primary, active_effect_use)?;
        let gesture_identity = gesture_identity(document, primary, active_effect_use)?;
        let position_control = position_control(document, primary, 0);
        let encoded_snapshot = javascript_json_parse_argument(&snapshot)?;
        let initialization_script = format!(
            r#"window.__MOTOLII_INSPECTOR_HOST__=(()=>{{
let listener=null;
let current=JSON.parse({encoded_snapshot});
return Object.freeze({{
get snapshot(){{return current;}},
subscribe:(next)=>{{if(typeof next!=="function"||listener!==null)throw new TypeError("invalid Inspector subscriber");listener=next;listener(current);}},
publish:(next)=>{{current=next;if(listener!==null)listener(current);}},
postMessage:(message)=>window.ipc.postMessage(message)
}});
}})();"#
        );
        let gesture_inbox = Arc::new(Mutex::new(InspectorGestureInbox::new(gesture_identity)));
        let position_key_inbox = Arc::new(Mutex::new(AddPositionKeyInbox::new(
            position_control.map(position_intent),
        )));
        let wake = Arc::new(Mutex::new(None::<InspectorWake>));
        let callback_inbox = Arc::clone(&gesture_inbox);
        let callback_position_key_inbox = Arc::clone(&position_key_inbox);
        let callback_wake = Arc::clone(&wake);
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
            .with_ipc_handler(move |request| {
                let raw = request.body();
                let accepted = match serde_json::from_str::<AddPositionKeyMessage>(raw) {
                    Ok(message) => callback_position_key_inbox
                        .lock()
                        .map_err(|_| AddPositionKeyIntentError::InboxPoisoned)
                        .and_then(|mut inbox| inbox.accept(message))
                        .map_err(InspectorIpcError::from),
                    Err(_) => callback_inbox
                        .lock()
                        .map_err(|_| InspectorGestureError::InboxPoisoned)
                        .and_then(|mut inbox| inbox.accept(raw))
                        .map_err(InspectorIpcError::from),
                };
                match accepted {
                    Ok(()) => {
                        crate::ui_numeric_trace::emit(format_args!(
                            "kind=webview surface=inspector event=ipc-accepted bytes={}",
                            raw.len(),
                        ));
                        if let Ok(wake) = callback_wake.lock() {
                            if let Some(wake) = wake.as_ref() {
                                wake();
                            }
                        }
                    }
                    Err(error) => eprintln!("Inspector Host rejected message: {error}"),
                }
            })
            .build_as_child(window)?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=inspector event=created elapsed_ms={:.3} primary_present={}",
            created_at.elapsed().as_secs_f64() * 1_000.0,
            primary.is_some(),
        ));
        Ok(Self {
            gesture_inbox,
            position_key_inbox,
            wake,
            webview,
            latest_layout_epoch: None,
        })
    }

    pub(crate) fn register_wake(&self, wake: InspectorWake) -> Result<(), InspectorGestureError> {
        let mut slot = self
            .wake
            .lock()
            .map_err(|_| InspectorGestureError::InboxPoisoned)?;
        *slot = Some(wake);
        Ok(())
    }

    pub(crate) fn take_latest_update(
        &self,
    ) -> Result<Option<InspectorGestureUpdate>, InspectorGestureError> {
        Ok(self
            .gesture_inbox
            .lock()
            .map_err(|_| InspectorGestureError::InboxPoisoned)?
            .latest_update
            .take())
    }

    pub(crate) fn take_terminal(
        &self,
    ) -> Result<Option<InspectorGestureTerminal>, InspectorGestureError> {
        Ok(self
            .gesture_inbox
            .lock()
            .map_err(|_| InspectorGestureError::InboxPoisoned)?
            .terminals
            .pop_front())
    }

    pub(crate) fn take_add_position_key(
        &self,
    ) -> Result<Option<AddPositionKeyIntent>, AddPositionKeyIntentError> {
        Ok(self
            .position_key_inbox
            .lock()
            .map_err(|_| AddPositionKeyIntentError::InboxPoisoned)?
            .pending
            .pop_front())
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
            crate::ui_numeric_trace::emit(format_args!(
                "kind=webview surface=inspector event=bounds layout_epoch={} visible=true \
                 x={:.3} y={:.3} width={:.3} height={:.3}",
                layout_epoch, rect.x, rect.y, rect.width, rect.height,
            ));
        } else {
            crate::ui_numeric_trace::emit(format_args!(
                "kind=webview surface=inspector event=bounds layout_epoch={} visible=false",
                layout_epoch,
            ));
        }
        self.webview.set_visible(rect.is_some())?;
        self.latest_layout_epoch = Some(layout_epoch);
        Ok(())
    }

    pub(crate) fn publish(
        &self,
        document: &motolii_doc::Document,
        primary: Option<motolii_doc::LayerId>,
        active_effect_use: Option<motolii_doc::EffectId>,
        projection_generation: u64,
    ) -> Result<(), InspectorHostRuntimeError> {
        let snapshot = snapshot_json_with_position_generation(
            document,
            primary,
            active_effect_use,
            projection_generation,
        )?;
        let gesture_identity = gesture_identity(document, primary, active_effect_use)?;
        let position_control = position_control(document, primary, projection_generation);
        self.gesture_inbox
            .lock()
            .map_err(|_| InspectorGestureError::InboxPoisoned)?
            .reconcile(gesture_identity);
        self.position_key_inbox
            .lock()
            .map_err(|_| AddPositionKeyIntentError::InboxPoisoned)?
            .reconcile(position_control.map(position_intent));
        let encoded_snapshot = javascript_json_parse_argument(&snapshot)?;
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=inspector event=publish primary_present={} payload_bytes={}",
            primary.is_some(),
            encoded_snapshot.len(),
        ));
        self.webview.evaluate_script(&format!(
            "window.__MOTOLII_INSPECTOR_HOST__.publish(JSON.parse({encoded_snapshot}));"
        ))?;
        Ok(())
    }
}

pub(crate) fn resolve_effect_param_preview_command(
    document: &motolii_doc::Document,
    identity: &InspectorGestureIdentity,
    new_value: f64,
) -> Option<motolii_doc::Command> {
    if identity.plugin_id != "core.filter.opacity"
        || identity.effect_version != 1
        || identity.param_id != "amount"
        || !new_value.is_finite()
        || !(0.0..=1.0).contains(&new_value)
    {
        return None;
    }
    let effect_use = document.find_effect_use(identity.layer_id, identity.effect_use_id)?;
    if effect_use.definition_id != identity.definition_id {
        return None;
    }
    let definition = document.effect_definition(effect_use.definition_id)?;
    if definition.plugin_id != identity.plugin_id
        || definition.effect_version != identity.effect_version
        || definition.params.len() != 1
    {
        return None;
    }
    let old_value = definition.params.get("amount")?;
    if !matches!(
        old_value,
        motolii_doc::DocParam::Const(motolii_doc::DocValue::F64(value))
            if value.is_finite() && (0.0..=1.0).contains(value)
    ) {
        return None;
    }
    Some(motolii_doc::Command::SetProperty {
        target: identity.layer_id,
        property: motolii_doc::ScalarPropertyId::EffectParam(
            identity.effect_use_id,
            identity.param_id.clone(),
        ),
        old_value: old_value.clone(),
        new_value: motolii_doc::DocParam::const_f64(new_value),
    })
}

fn gesture_identity(
    document: &motolii_doc::Document,
    primary: Option<motolii_doc::LayerId>,
    active_effect_use: Option<motolii_doc::EffectId>,
) -> Result<Option<InspectorGestureIdentity>, InspectorHostRuntimeError> {
    let Some(active_effect_use) = active_effect_use else {
        return Ok(None);
    };
    let Some(primary) = primary else {
        return Err(InspectorHostRuntimeError::ActiveEffectWithoutPrimary { active_effect_use });
    };
    let effect_use = document.find_effect_use(primary, active_effect_use).ok_or(
        InspectorHostRuntimeError::ActiveEffectNotFound {
            primary,
            active_effect_use,
        },
    )?;
    let definition = document.effect_definition(effect_use.definition_id).ok_or(
        InspectorHostRuntimeError::ActiveEffectDefinitionNotFound {
            definition_id: effect_use.definition_id,
        },
    )?;
    let amount = definition.params.get("amount");
    if definition.plugin_id != "core.filter.opacity"
        || definition.effect_version != 1
        || definition.params.len() != 1
        || !matches!(
            amount,
            Some(motolii_doc::DocParam::Const(motolii_doc::DocValue::F64(value)))
                if value.is_finite() && (0.0..=1.0).contains(value)
        )
    {
        return Err(InspectorHostRuntimeError::ActiveEffectContractMismatch {
            definition_id: definition.id,
        });
    }
    Ok(Some(InspectorGestureIdentity {
        layer_id: primary,
        effect_use_id: active_effect_use,
        definition_id: definition.id,
        plugin_id: definition.plugin_id.clone(),
        effect_version: definition.effect_version,
        param_id: "amount".to_owned(),
    }))
}

fn javascript_json_parse_argument(
    snapshot: &serde_json::Value,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(&serde_json::to_string(snapshot)?)
}

fn snapshot_json(
    document: &motolii_doc::Document,
    primary: Option<motolii_doc::LayerId>,
    active_effect_use: Option<motolii_doc::EffectId>,
) -> Result<serde_json::Value, InspectorHostRuntimeError> {
    snapshot_json_with_position_generation(document, primary, active_effect_use, 0)
}

fn snapshot_json_with_position_generation(
    document: &motolii_doc::Document,
    primary: Option<motolii_doc::LayerId>,
    active_effect_use: Option<motolii_doc::EffectId>,
    projection_generation: u64,
) -> Result<serde_json::Value, InspectorHostRuntimeError> {
    if let Some(active_effect_use) = active_effect_use {
        let Some(primary) = primary else {
            return Err(InspectorHostRuntimeError::ActiveEffectWithoutPrimary {
                active_effect_use,
            });
        };
        if document
            .find_effect_use(primary, active_effect_use)
            .is_none()
        {
            return Err(InspectorHostRuntimeError::ActiveEffectNotFound {
                primary,
                active_effect_use,
            });
        }
    }
    let Some(primary) = primary else {
        return Ok(serde_json::Value::Null);
    };

    let catalog = motolii_plugins_firstparty::first_party_catalog()?;
    let nodes = catalog
        .iter()
        .map(|(_, contract)| {
            let params = contract
                .node
                .params
                .iter()
                .map(|param| {
                    let mapped = map_parameter_control(param)?;
                    Ok(InspectorParameter {
                        id: param.id,
                        value_type: param.value_type.as_str(),
                        default: doc_value_from_plugin(&param.default),
                        f64_domain: param.f64_domain.map(|domain| InspectorF64Domain {
                            min_inclusive: domain.min_inclusive,
                            max_inclusive: domain.max_inclusive,
                            integer: domain.integer,
                        }),
                        control: control_name(mapped.control()),
                    })
                })
                .collect::<Result<Vec<_>, crate::ParameterControlError>>()?;
            Ok(InspectorNode {
                id: contract.node.id.0,
                effect_version: contract.node.version,
                params,
            })
        })
        .collect::<Result<Vec<_>, crate::ParameterControlError>>()?;

    Ok(serde_json::to_value(InspectorSnapshot {
        fixture_revision: 1,
        document,
        nodes,
        target: InspectorTarget { layer_id: primary },
        active_effect_use_id: active_effect_use,
        position_control: position_control(document, Some(primary), projection_generation),
    })?)
}

fn position_intent(control: InspectorPositionControl) -> AddPositionKeyIntent {
    AddPositionKeyIntent {
        layer_id: control.layer_id,
        projection_generation: control.projection_generation,
    }
}

fn position_control(
    document: &motolii_doc::Document,
    primary: Option<motolii_doc::LayerId>,
    projection_generation: u64,
) -> Option<InspectorPositionControl> {
    let layer_id = primary?;
    let position = &find_envelope(document, layer_id)?.transform.position;
    let key_count = match position {
        motolii_doc::DocParam::Const(motolii_doc::DocValue::Vec2(_)) => 0,
        motolii_doc::DocParam::Keyframes(track) => track.keys().len(),
        _ => return None,
    };
    Some(InspectorPositionControl {
        layer_id,
        projection_generation,
        key_count,
    })
}

fn find_envelope(
    document: &motolii_doc::Document,
    target: motolii_doc::LayerId,
) -> Option<&motolii_doc::ItemEnvelope> {
    fn in_items(
        items: &[motolii_doc::TrackItem],
        target: motolii_doc::LayerId,
    ) -> Option<&motolii_doc::ItemEnvelope> {
        for item in items {
            let envelope = match item {
                motolii_doc::TrackItem::Clip(clip) => &clip.envelope,
                motolii_doc::TrackItem::Group(group) => &group.envelope,
            };
            if envelope.layer_id == target {
                return Some(envelope);
            }
            if let motolii_doc::TrackItem::Group(group) = item {
                if let Some(found) = in_items(&group.children, target) {
                    return Some(found);
                }
            }
        }
        None
    }

    document
        .tracks
        .iter()
        .find_map(|track| in_items(&track.items, target))
}

fn doc_value_from_plugin(value: &motolii_plugin::Value) -> motolii_doc::DocValue {
    match value {
        motolii_plugin::Value::F64(value) => motolii_doc::DocValue::F64(*value),
        motolii_plugin::Value::Vec2(value) => motolii_doc::DocValue::Vec2(*value),
        motolii_plugin::Value::Vec3(value) => motolii_doc::DocValue::Vec3(*value),
        motolii_plugin::Value::Color(value) => motolii_doc::DocValue::Color(*value),
        motolii_plugin::Value::AssetRef(value) => {
            motolii_doc::DocValue::AssetRef(motolii_doc::AssetId::from_raw(*value))
        }
    }
}

fn control_name(control: HostParameterControl) -> &'static str {
    match control {
        HostParameterControl::F64 { .. } => "F64",
        HostParameterControl::Vec2 => "Vec2",
        HostParameterControl::Vec3 => "Vec3",
        HostParameterControl::Color => "Color",
    }
}

#[derive(serde::Serialize)]
struct InspectorSnapshot<'a> {
    #[serde(rename = "fixtureRevision")]
    fixture_revision: u8,
    document: &'a motolii_doc::Document,
    nodes: Vec<InspectorNode>,
    target: InspectorTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_effect_use_id: Option<motolii_doc::EffectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position_control: Option<InspectorPositionControl>,
}

#[derive(serde::Serialize)]
struct InspectorNode {
    id: &'static str,
    effect_version: u32,
    params: Vec<InspectorParameter>,
}

#[derive(serde::Serialize)]
struct InspectorParameter {
    id: &'static str,
    value_type: &'static str,
    default: motolii_doc::DocValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    f64_domain: Option<InspectorF64Domain>,
    control: &'static str,
}

#[derive(serde::Serialize)]
struct InspectorF64Domain {
    #[serde(skip_serializing_if = "Option::is_none")]
    min_inclusive: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_inclusive: Option<f64>,
    integer: bool,
}

#[derive(serde::Serialize)]
struct InspectorTarget {
    layer_id: motolii_doc::LayerId,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum InspectorHostRuntimeError {
    #[error("Inspector active Effect Use {active_effect_use:?} has no primary layer")]
    ActiveEffectWithoutPrimary {
        active_effect_use: motolii_doc::EffectId,
    },
    #[error(
        "Inspector active Effect Use {active_effect_use:?} was not found under primary layer {primary:?}"
    )]
    ActiveEffectNotFound {
        primary: motolii_doc::LayerId,
        active_effect_use: motolii_doc::EffectId,
    },
    #[error("Inspector active Effect Definition {definition_id:?} was not found")]
    ActiveEffectDefinitionNotFound {
        definition_id: motolii_doc::EffectDefinitionId,
    },
    #[error("Inspector active Effect Definition {definition_id:?} does not match Opacity amount")]
    ActiveEffectContractMismatch {
        definition_id: motolii_doc::EffectDefinitionId,
    },
    #[error(transparent)]
    Gesture(#[from] InspectorGestureError),
    #[error(transparent)]
    PositionKey(#[from] AddPositionKeyIntentError),
    #[error(transparent)]
    Catalog(#[from] motolii_plugin::PluginContractError),
    #[error(transparent)]
    ParameterControl(#[from] crate::ParameterControlError),
    #[error("Inspector Host JSON could not be encoded")]
    Json(#[from] serde_json::Error),
    #[error("Inspector Host WebView failed")]
    WebView(#[from] wry::Error),
}

#[derive(Debug, thiserror::Error)]
enum InspectorIpcError {
    #[error(transparent)]
    Gesture(#[from] InspectorGestureError),
    #[error(transparent)]
    PositionKey(#[from] AddPositionKeyIntentError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AddPositionKeyIntentError {
    #[error("Inspector Add Position Key identity does not match the current projection")]
    IdentityMismatch,
    #[error("Inspector Add Position Key inbox is full")]
    InboxFull,
    #[error("Inspector Add Position Key inbox lock is poisoned")]
    InboxPoisoned,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum InspectorGestureError {
    #[error("Inspector gesture JSON was rejected")]
    Json(#[from] serde_json::Error),
    #[error("Inspector gesture has no active target")]
    NoActiveTarget,
    #[error("Inspector gesture identity does not match the active target")]
    IdentityMismatch,
    #[error("Inspector gesture session and sequence must be positive")]
    NonPositiveOrder,
    #[error("Inspector gesture value must be finite and in [0,1]")]
    InvalidValue,
    #[error("Inspector cancel must not carry a value")]
    UnexpectedValue,
    #[error("Inspector gesture is already active")]
    GestureAlreadyActive,
    #[error("Inspector gesture is not active")]
    GestureNotActive,
    #[error("Inspector gesture is stale or reordered")]
    StaleOrReordered,
    #[error("Inspector terminal inbox is full")]
    TerminalInboxFull,
    #[error("Inspector gesture inbox lock is poisoned")]
    InboxPoisoned,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document_with_effects() -> (
        motolii_doc::Document,
        motolii_doc::LayerId,
        motolii_doc::EffectId,
        motolii_doc::LayerId,
        motolii_doc::EffectId,
    ) {
        let mut document = motolii_doc::Document::new_current();
        let primary = motolii_doc::LayerId::from_raw(10);
        let primary_effect = motolii_doc::EffectId::from_raw(20);
        let other = motolii_doc::LayerId::from_raw(11);
        let other_effect = motolii_doc::EffectId::from_raw(21);
        let mut primary_envelope = motolii_doc::ItemEnvelope::new(primary);
        primary_envelope.effects.push(motolii_doc::EffectUse {
            id: primary_effect,
            definition_id: motolii_doc::EffectDefinitionId::from_raw(30),
        });
        let mut other_envelope = motolii_doc::ItemEnvelope::new(other);
        other_envelope.effects.push(motolii_doc::EffectUse {
            id: other_effect,
            definition_id: motolii_doc::EffectDefinitionId::from_raw(31),
        });
        document.tracks.push(motolii_doc::Track {
            id: motolii_doc::TrackId::from_raw(1),
            items: vec![
                motolii_doc::TrackItem::Group(motolii_doc::Group {
                    envelope: primary_envelope,
                    children: Vec::new(),
                }),
                motolii_doc::TrackItem::Group(motolii_doc::Group {
                    envelope: other_envelope,
                    children: Vec::new(),
                }),
            ],
        });
        (document, primary, primary_effect, other, other_effect)
    }

    #[test]
    fn absent_primary_projects_null_without_inventing_selection() {
        let document = motolii_doc::Document::new_current();
        assert_eq!(
            snapshot_json(&document, None, None).unwrap(),
            serde_json::Value::Null
        );
    }

    #[test]
    fn active_effect_without_primary_is_a_typed_error() {
        let active_effect_use = motolii_doc::EffectId::from_raw(20);
        let error = snapshot_json(
            &motolii_doc::Document::new_current(),
            None,
            Some(active_effect_use),
        )
        .unwrap_err();
        match error {
            InspectorHostRuntimeError::ActiveEffectWithoutPrimary {
                active_effect_use: actual,
            } => assert_eq!(actual, active_effect_use),
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn missing_and_cross_layer_active_effects_are_typed_errors() {
        let (document, primary, _, _, other_effect) = document_with_effects();
        for active_effect_use in [motolii_doc::EffectId::from_raw(999), other_effect] {
            let error =
                snapshot_json(&document, Some(primary), Some(active_effect_use)).unwrap_err();
            match error {
                InspectorHostRuntimeError::ActiveEffectNotFound {
                    primary: actual_primary,
                    active_effect_use: actual_effect,
                } => {
                    assert_eq!(actual_primary, primary);
                    assert_eq!(actual_effect, active_effect_use);
                }
                other => panic!("unexpected error variant: {other:?}"),
            }
        }
    }

    #[test]
    fn exact_primary_active_effect_and_catalog_metadata_are_projected() {
        let (document, primary, primary_effect, _, _) = document_with_effects();
        let snapshot = snapshot_json(&document, Some(primary), Some(primary_effect)).unwrap();
        assert_eq!(snapshot["fixtureRevision"], 1);
        assert_eq!(snapshot["target"]["layer_id"], primary.get());
        assert_eq!(snapshot["active_effect_use_id"], primary_effect.get());

        let catalog = motolii_plugins_firstparty::first_party_catalog().unwrap();
        let nodes = snapshot["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), catalog.len());
        for (node, (_, contract)) in nodes.iter().zip(catalog.iter()) {
            assert_eq!(node["id"], contract.node.id.0);
            assert_eq!(node["effect_version"], contract.node.version);
            let params = node["params"].as_array().unwrap();
            assert_eq!(params.len(), contract.node.params.len());
            for (projected, source) in params.iter().zip(&contract.node.params) {
                assert_eq!(projected["id"], source.id);
                assert_eq!(projected["value_type"], source.value_type.as_str());
                assert_eq!(
                    projected["default"],
                    serde_json::to_value(doc_value_from_plugin(&source.default)).unwrap()
                );
                assert_eq!(
                    projected["control"],
                    control_name(map_parameter_control(source).unwrap().control())
                );
                match source.f64_domain {
                    Some(domain) => {
                        let projected_domain = projected["f64_domain"].as_object().unwrap();
                        match domain.min_inclusive {
                            Some(min) => assert_eq!(projected_domain["min_inclusive"], min),
                            None => assert!(!projected_domain.contains_key("min_inclusive")),
                        }
                        match domain.max_inclusive {
                            Some(max) => assert_eq!(projected_domain["max_inclusive"], max),
                            None => assert!(!projected_domain.contains_key("max_inclusive")),
                        }
                        assert_eq!(projected_domain["integer"], domain.integer);
                    }
                    None => assert!(!projected.as_object().unwrap().contains_key("f64_domain")),
                }
            }
        }

        let opacity = nodes
            .iter()
            .find(|node| node["id"] == "core.filter.opacity")
            .unwrap();
        assert_eq!(opacity["effect_version"], 1);
        assert_eq!(
            opacity["params"],
            serde_json::json!([{
                "id": "amount",
                "value_type": "F64",
                "default": { "F64": 1.0 },
                "f64_domain": {
                    "min_inclusive": 0.0,
                    "max_inclusive": 1.0,
                    "integer": false
                },
                "control": "F64",
            }])
        );
    }

    #[test]
    fn inactive_snapshot_omits_active_effect_identity() {
        let (document, primary, _, _, _) = document_with_effects();
        let snapshot = snapshot_json(&document, Some(primary), None).unwrap();
        assert!(!snapshot
            .as_object()
            .unwrap()
            .contains_key("active_effect_use_id"));
        assert_eq!(snapshot["position_control"]["layer_id"], primary.get());
        assert_eq!(snapshot["position_control"]["projection_generation"], 0);
        assert_eq!(snapshot["position_control"]["key_count"], 0);
    }

    #[test]
    fn add_position_key_inbox_rejects_stale_identity_and_clears_on_projection_change() {
        let expected = AddPositionKeyIntent {
            layer_id: motolii_doc::LayerId::from_raw(5),
            projection_generation: 9,
        };
        let message = || AddPositionKeyMessage {
            _kind: AddPositionKeyKind::AddPositionKey,
            layer_id: expected.layer_id,
            projection_generation: expected.projection_generation,
        };
        let mut inbox = AddPositionKeyInbox::new(Some(expected));
        inbox.accept(message()).unwrap();
        assert_eq!(inbox.pending.pop_front(), Some(expected));
        inbox.accept(message()).unwrap();
        inbox.reconcile(Some(AddPositionKeyIntent {
            projection_generation: 10,
            ..expected
        }));
        assert!(inbox.pending.is_empty());
        assert!(matches!(
            inbox.accept(message()),
            Err(AddPositionKeyIntentError::IdentityMismatch)
        ));
    }

    #[test]
    fn bridge_argument_is_a_quoted_json_string_not_an_object_literal() {
        let encoded = javascript_json_parse_argument(&serde_json::json!({"target": 7})).unwrap();
        assert_eq!(encoded, r#""{\"target\":7}""#);
    }

    fn gesture_fixture() -> (
        motolii_doc::Document,
        motolii_doc::LayerId,
        motolii_doc::EffectId,
        InspectorGestureIdentity,
    ) {
        let mut document = motolii_doc::Document::new_current();
        let layer_id = motolii_doc::LayerId::from_raw(5);
        let effect_use_id = motolii_doc::EffectId::from_raw(17);
        let definition_id = motolii_doc::EffectDefinitionId::from_raw(23);
        let mut envelope = motolii_doc::ItemEnvelope::new(layer_id);
        envelope.effects.push(motolii_doc::EffectUse {
            id: effect_use_id,
            definition_id,
        });
        document.tracks.push(motolii_doc::Track {
            id: motolii_doc::TrackId::from_raw(1),
            items: vec![motolii_doc::TrackItem::Group(motolii_doc::Group {
                envelope,
                children: Vec::new(),
            })],
        });
        document
            .effect_definitions
            .push(motolii_doc::EffectDefinition::new(
                definition_id,
                "core.filter.opacity",
                1,
                true,
                std::collections::BTreeMap::from([(
                    "amount".to_owned(),
                    motolii_doc::DocParam::const_f64(0.72),
                )]),
                serde_json::Map::new(),
            ));
        let identity = gesture_identity(&document, Some(layer_id), Some(effect_use_id))
            .unwrap()
            .unwrap();
        (document, layer_id, effect_use_id, identity)
    }

    fn gesture_message(
        identity: &InspectorGestureIdentity,
        phase: &str,
        session: u64,
        sequence: u64,
        value: Option<f64>,
    ) -> String {
        let mut message = serde_json::json!({
            "kind": "effect-param-gesture",
            "phase": phase,
            "session": session,
            "sequence": sequence,
            "layer_id": identity.layer_id,
            "effect_use_id": identity.effect_use_id,
            "definition_id": identity.definition_id,
            "plugin_id": identity.plugin_id,
            "effect_version": identity.effect_version,
            "param_id": identity.param_id,
        });
        if let Some(value) = value {
            message["value"] = serde_json::json!(value);
        }
        serde_json::to_string(&message).unwrap()
    }

    #[test]
    fn terminal_conversion_preserves_exact_commit_and_drops_cancel() {
        let (_, _, _, identity) = gesture_fixture();
        let expected = SetEffectParamRequest::new(
            identity.layer_id,
            identity.effect_use_id,
            identity.definition_id,
            identity.plugin_id.clone(),
            identity.effect_version,
            identity.param_id.clone(),
            0.41,
        );
        let commit = InspectorGestureTerminal {
            session: 7,
            sequence: 9,
            identity: identity.clone(),
            cause: InspectorGestureTerminalCause::Commit(0.41),
        };
        assert_eq!(commit.into_set_effect_param_request(), Some(expected));

        let cancel = InspectorGestureTerminal {
            session: 8,
            sequence: 3,
            identity,
            cause: InspectorGestureTerminalCause::Cancel,
        };
        assert_eq!(cancel.into_set_effect_param_request(), None);
    }

    #[test]
    fn hundred_updates_replace_one_latest_slot_and_terminal_is_preserved() {
        let (_, _, _, identity) = gesture_fixture();
        let mut inbox = InspectorGestureInbox::new(Some(identity.clone()));
        inbox
            .accept(&gesture_message(&identity, "start", 1, 1, Some(0.72)))
            .unwrap();
        for sequence in 2..=101 {
            inbox
                .accept(&gesture_message(
                    &identity,
                    "update",
                    1,
                    sequence,
                    Some((sequence - 1) as f64 / 100.0),
                ))
                .unwrap();
        }
        let latest = inbox.latest_update.as_ref().unwrap();
        assert_eq!(latest.sequence, 101);
        assert_eq!(latest.value, 1.0);
        inbox
            .accept(&gesture_message(&identity, "commit", 1, 102, Some(1.0)))
            .unwrap();
        assert!(inbox.latest_update.is_none());
        assert_eq!(inbox.terminals.len(), 1);
        assert_eq!(
            inbox.terminals.front().unwrap().cause,
            InspectorGestureTerminalCause::Commit(1.0)
        );
    }

    #[test]
    fn stale_prior_session_and_reordered_sequence_are_rejected() {
        let (_, _, _, identity) = gesture_fixture();
        let mut inbox = InspectorGestureInbox::new(Some(identity.clone()));
        inbox
            .accept(&gesture_message(&identity, "start", 1, 1, Some(0.72)))
            .unwrap();
        inbox
            .accept(&gesture_message(&identity, "cancel", 1, 2, None))
            .unwrap();
        inbox
            .accept(&gesture_message(&identity, "start", 2, 1, Some(0.72)))
            .unwrap();
        for raw in [
            gesture_message(&identity, "update", 1, 3, Some(0.8)),
            gesture_message(&identity, "update", 2, 1, Some(0.8)),
            gesture_message(&identity, "start", 2, 1, Some(0.8)),
        ] {
            assert!(matches!(
                inbox.accept(&raw),
                Err(InspectorGestureError::StaleOrReordered)
                    | Err(InspectorGestureError::GestureAlreadyActive)
            ));
        }
        assert!(inbox.latest_update.is_none());
    }

    #[test]
    fn malformed_identity_value_and_phase_shape_fail_closed() {
        let (_, _, _, identity) = gesture_fixture();
        let mut inbox = InspectorGestureInbox::new(Some(identity.clone()));
        for raw in [
            gesture_message(&identity, "start", 0, 1, Some(0.5)),
            gesture_message(&identity, "start", 1, 1, Some(1.1)),
            gesture_message(&identity, "cancel", 1, 1, Some(0.5)),
            gesture_message(&identity, "unknown", 1, 1, Some(0.5)),
            gesture_message(
                &InspectorGestureIdentity {
                    effect_use_id: motolii_doc::EffectId::from_raw(999),
                    ..identity.clone()
                },
                "start",
                1,
                1,
                Some(0.5),
            ),
        ] {
            assert!(inbox.accept(&raw).is_err());
        }
        assert!(inbox.active.is_none());
        assert!(inbox.latest_update.is_none());
        assert!(inbox.terminals.is_empty());
    }

    #[test]
    fn cancel_rejects_explicit_null_without_advancing_the_active_session() {
        let (_, _, _, identity) = gesture_fixture();
        let mut inbox = InspectorGestureInbox::new(Some(identity.clone()));
        inbox
            .accept(&gesture_message(&identity, "start", 1, 1, Some(0.5)))
            .unwrap();
        let mut cancel: serde_json::Value =
            serde_json::from_str(&gesture_message(&identity, "cancel", 1, 2, None)).unwrap();
        cancel["value"] = serde_json::Value::Null;
        assert!(matches!(
            inbox.accept(&serde_json::to_string(&cancel).unwrap()),
            Err(InspectorGestureError::UnexpectedValue)
        ));
        assert_eq!(inbox.active.as_ref().unwrap().last_sequence, 1);
        assert!(inbox.terminals.is_empty());
    }

    #[test]
    fn resolve_preview_command_reads_snapshot_old_value_and_exact_identity() {
        let (document, _, _, identity) = gesture_fixture();
        let command =
            resolve_effect_param_preview_command(&document, &identity, 0.41).expect("command");
        match command {
            motolii_doc::Command::SetProperty {
                target,
                property,
                old_value,
                new_value,
            } => {
                assert_eq!(target, identity.layer_id);
                assert_eq!(
                    property,
                    motolii_doc::ScalarPropertyId::EffectParam(
                        identity.effect_use_id,
                        "amount".to_owned()
                    )
                );
                assert_eq!(old_value, motolii_doc::DocParam::const_f64(0.72));
                assert_eq!(new_value, motolii_doc::DocParam::const_f64(0.41));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn resolve_preview_command_rejects_mismatched_contract_without_guessing() {
        let (document, _, _, identity) = gesture_fixture();
        let mismatches = [
            InspectorGestureIdentity {
                layer_id: motolii_doc::LayerId::from_raw(999),
                ..identity.clone()
            },
            InspectorGestureIdentity {
                effect_use_id: motolii_doc::EffectId::from_raw(999),
                ..identity.clone()
            },
            InspectorGestureIdentity {
                definition_id: motolii_doc::EffectDefinitionId::from_raw(999),
                ..identity.clone()
            },
            InspectorGestureIdentity {
                plugin_id: "core.filter.other".to_owned(),
                ..identity.clone()
            },
            InspectorGestureIdentity {
                effect_version: 2,
                ..identity.clone()
            },
            InspectorGestureIdentity {
                param_id: "opacity".to_owned(),
                ..identity.clone()
            },
        ];
        for mismatched in mismatches {
            assert!(resolve_effect_param_preview_command(&document, &mismatched, 0.5).is_none());
        }
        assert!(resolve_effect_param_preview_command(&document, &identity, 1.1).is_none());
    }

    #[test]
    fn terminal_batch_coalesces_cancel_before_exact_commit() {
        let (document, layer_id, effect_use_id, identity) = gesture_fixture();
        let terminals = [
            InspectorGestureTerminal {
                session: 1,
                sequence: 2,
                identity: identity.clone(),
                cause: InspectorGestureTerminalCause::Cancel,
            },
            InspectorGestureTerminal {
                session: 2,
                sequence: 3,
                identity: identity.clone(),
                cause: InspectorGestureTerminalCause::Commit(0.55),
            },
        ];
        let mut needs_baseline = false;
        let mut pending_commit = None;
        for terminal in terminals {
            match terminal.cause {
                InspectorGestureTerminalCause::Cancel => needs_baseline = true,
                InspectorGestureTerminalCause::Commit(value) => {
                    if let Some(command) =
                        resolve_effect_param_preview_command(&document, &terminal.identity, value)
                    {
                        pending_commit = Some(command);
                        break;
                    }
                    needs_baseline = true;
                }
            }
        }
        assert!(needs_baseline);
        assert_eq!(
            pending_commit,
            Some(motolii_doc::Command::SetProperty {
                target: layer_id,
                property: motolii_doc::ScalarPropertyId::EffectParam(
                    effect_use_id,
                    "amount".to_owned()
                ),
                old_value: motolii_doc::DocParam::const_f64(0.72),
                new_value: motolii_doc::DocParam::const_f64(0.55),
            })
        );
    }
}
