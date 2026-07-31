//! product-owned Inspectorをnative shellのopaque childへ載せるprivate Host。

use wry::{Rect, WebView, WebViewBuilder};

use crate::browser_host_runtime::product_asset_response;
use crate::native_host_layout::LogicalRect;
use crate::{map_parameter_control, HostParameterControl};

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
        active_effect_use: Option<motolii_doc::EffectId>,
    ) -> Result<Self, InspectorHostRuntimeError> {
        let created_at = std::time::Instant::now();
        let snapshot = snapshot_json(document, primary, active_effect_use)?;
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
        crate::ui_numeric_trace::emit(format_args!(
            "kind=webview surface=inspector event=created elapsed_ms={:.3} primary_present={}",
            created_at.elapsed().as_secs_f64() * 1_000.0,
            primary.is_some(),
        ));
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
    ) -> Result<(), InspectorHostRuntimeError> {
        let snapshot = snapshot_json(document, primary, active_effect_use)?;
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
    })?)
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
    #[error(transparent)]
    Catalog(#[from] motolii_plugin::PluginContractError),
    #[error(transparent)]
    ParameterControl(#[from] crate::ParameterControlError),
    #[error("Inspector Host JSON could not be encoded")]
    Json(#[from] serde_json::Error),
    #[error("Inspector Host WebView failed")]
    WebView(#[from] wry::Error),
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
    }

    #[test]
    fn bridge_argument_is_a_quoted_json_string_not_an_object_literal() {
        let encoded = javascript_json_parse_argument(&serde_json::json!({"target": 7})).unwrap();
        assert_eq!(encoded, r#""{\"target\":7}""#);
    }
}
