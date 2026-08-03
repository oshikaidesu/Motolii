//! parameter-to-UI control mapping for first-party UI host controls.

use motolii_doc::{CommandKind, EffectId, ScalarPropertyId};
use motolii_plugin::{F64Domain, ParamDef, ValueType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HostParameterControl {
    F64 { domain: Option<F64Domain> },
    Vec2,
    Vec3,
    Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParameterControlSpec {
    param_id: &'static str,
    control: HostParameterControl,
}

#[derive(Debug, thiserror::Error)]
pub enum ParameterControlError {
    #[error("unsupported value type `{value_type}` for parameter `{param_id}`")]
    UnsupportedValueType {
        param_id: &'static str,
        value_type: ValueType,
    },
}

impl ParameterControlSpec {
    pub fn param_id(&self) -> &'static str {
        self.param_id
    }

    pub fn control(&self) -> HostParameterControl {
        self.control
    }

    pub fn command_kind(&self) -> CommandKind {
        CommandKind::SetProperty
    }

    pub fn effect_property(&self, effect: EffectId) -> ScalarPropertyId {
        ScalarPropertyId::EffectParam(effect, self.param_id.to_owned())
    }
}

pub fn map_parameter_control(
    param: &ParamDef,
) -> Result<ParameterControlSpec, ParameterControlError> {
    match param.value_type {
        ValueType::F64 => Ok(ParameterControlSpec {
            param_id: param.id,
            control: HostParameterControl::F64 {
                domain: param.f64_domain,
            },
        }),
        ValueType::Vec2 => Ok(ParameterControlSpec {
            param_id: param.id,
            control: HostParameterControl::Vec2,
        }),
        ValueType::Vec3 => Ok(ParameterControlSpec {
            param_id: param.id,
            control: HostParameterControl::Vec3,
        }),
        ValueType::Color => Ok(ParameterControlSpec {
            param_id: param.id,
            control: HostParameterControl::Color,
        }),
        ValueType::AssetRef => Err(ParameterControlError::UnsupportedValueType {
            param_id: param.id,
            value_type: param.value_type,
        }),
    }
}
