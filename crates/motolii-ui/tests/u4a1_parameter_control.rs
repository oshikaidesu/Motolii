//! U4a-1: toolkit非依存のparameter mapping contract。

use motolii_doc::{CommandKind, EffectId, ScalarPropertyId};
use motolii_eval::Value;
use motolii_plugin::{F64Domain, ParamDef, ValueType};
use motolii_plugins_firstparty::first_party_catalog;
use motolii_ui::{
    map_parameter_control, HostParameterControl, ParameterControlError, ParameterControlSpec,
};

#[test]
fn map_parameter_control_maps_f64_with_domain_preserved() {
    let param = ParamDef {
        id: "amplitude",
        value_type: ValueType::F64,
        default: Value::F64(0.5),
        f64_domain: Some(F64Domain::new(Some(-1.0), Some(1.0), false)),
    };
    let mapped = map_parameter_control(&param).unwrap();
    assert_eq!(mapped.param_id(), "amplitude");
    assert_eq!(
        mapped.control(),
        HostParameterControl::F64 {
            domain: Some(F64Domain::new(Some(-1.0), Some(1.0), false))
        }
    );
}

#[test]
fn map_parameter_control_maps_vec2() {
    let param = ParamDef {
        id: "position",
        value_type: ValueType::Vec2,
        default: Value::Vec2([0.0, 0.0]),
        f64_domain: None,
    };
    let mapped = map_parameter_control(&param).unwrap();
    assert_eq!(mapped.param_id(), "position");
    assert_eq!(mapped.control(), HostParameterControl::Vec2);
}

#[test]
fn map_parameter_control_maps_vec3() {
    let param = ParamDef {
        id: "axis",
        value_type: ValueType::Vec3,
        default: Value::Vec3([0.0, 0.0, 0.0]),
        f64_domain: None,
    };
    let mapped = map_parameter_control(&param).unwrap();
    assert_eq!(mapped.param_id(), "axis");
    assert_eq!(mapped.control(), HostParameterControl::Vec3);
}

#[test]
fn map_parameter_control_maps_color() {
    let param = ParamDef {
        id: "tint",
        value_type: ValueType::Color,
        default: Value::Color([0.0, 0.0, 1.0, 1.0]),
        f64_domain: None,
    };
    let mapped = map_parameter_control(&param).unwrap();
    assert_eq!(mapped.param_id(), "tint");
    assert_eq!(mapped.control(), HostParameterControl::Color);
}

#[test]
fn map_parameter_control_rejects_asset_ref() {
    let param = ParamDef {
        id: "asset",
        value_type: ValueType::AssetRef,
        default: Value::AssetRef(0),
        f64_domain: None,
    };
    assert!(matches!(
        map_parameter_control(&param),
        Err(ParameterControlError::UnsupportedValueType {
            param_id: "asset",
            value_type: ValueType::AssetRef,
        })
    ));
}

#[test]
fn map_parameter_control_returns_set_property_and_effect_property_route() {
    let param = ParamDef {
        id: "amount",
        value_type: ValueType::F64,
        default: Value::F64(0.25),
        f64_domain: None,
    };
    let mapped = map_parameter_control(&param).unwrap();

    assert_eq!(mapped.command_kind(), CommandKind::SetProperty);
    assert_eq!(
        mapped.effect_property(EffectId::from_raw(123)),
        ScalarPropertyId::EffectParam(EffectId::from_raw(123), "amount".to_owned())
    );
    assert_eq!(
        mapped.source_property(),
        ScalarPropertyId::SourceParam("amount".to_owned())
    );
}

#[test]
fn map_parameter_control_allows_duplicate_param_ids_across_plugins() {
    let first = ParamDef {
        id: "shared",
        value_type: ValueType::F64,
        default: Value::F64(0.0),
        f64_domain: None,
    };
    let second = ParamDef {
        id: "shared",
        value_type: ValueType::Color,
        default: Value::Color([1.0, 1.0, 1.0, 1.0]),
        f64_domain: None,
    };

    let first_spec = map_parameter_control(&first).unwrap();
    let second_spec = map_parameter_control(&second).unwrap();

    assert_eq!(first_spec.param_id(), "shared");
    assert_eq!(second_spec.param_id(), "shared");
    assert_ne!(first_spec.control(), second_spec.control());
}

#[test]
fn map_parameter_control_handles_all_first_party_catalog_params() {
    for (id, contract) in first_party_catalog().unwrap().iter() {
        for param in &contract.node.params {
            let mapped = map_parameter_control(param);
            match mapped {
                Ok(spec) => {
                    assert_eq!(spec.param_id(), param.id);
                    match (param.value_type, spec.control()) {
                        (ValueType::F64, HostParameterControl::F64 { domain }) => {
                            assert_eq!(domain, param.f64_domain);
                        }
                        (ValueType::Vec2, HostParameterControl::Vec2) => {}
                        (ValueType::Vec3, HostParameterControl::Vec3) => {}
                        (ValueType::Color, HostParameterControl::Color) => {}
                        (ValueType::AssetRef, _) => {
                            panic!(
                                "{:?}: AssetRef should be rejected, got mapped control for {}",
                                id.0, param.id
                            )
                        }
                        _ => panic!(
                            "{:?}: unexpected mapping for value type {:?}",
                            id.0, param.value_type
                        ),
                    }
                }
                Err(ParameterControlError::UnsupportedValueType {
                    param_id,
                    value_type,
                }) => {
                    assert_eq!(param_id, param.id);
                    assert_eq!(value_type, ValueType::AssetRef);
                }
            }
        }
    }
}

#[test]
fn map_parameter_control_keeps_public_return_shape() {
    let f64_domain = Some(F64Domain::new(Some(0.0), Some(1.0), true));
    let param = ParamDef {
        id: "opacity",
        value_type: ValueType::F64,
        default: Value::F64(0.5),
        f64_domain,
    };
    let spec: ParameterControlSpec = map_parameter_control(&param).unwrap();
    assert_eq!(spec.param_id(), "opacity");
    assert_eq!(
        spec.control(),
        HostParameterControl::F64 { domain: f64_domain }
    );
    assert_eq!(
        spec.control(),
        HostParameterControl::F64 { domain: f64_domain }
    );
}
