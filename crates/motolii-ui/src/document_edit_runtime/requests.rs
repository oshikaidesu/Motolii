//! queueへ載せる確定編集request。意味は既存command準備へ渡すだけ。

use std::path::PathBuf;

use motolii_core::RationalTime;
use motolii_doc::{DocParam, EffectDefinitionId, EffectId, KeyframeId, LayerId, ScalarPropertyId};
use motolii_eval::Interp;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlaceRectangleRequest {
    pub(crate) position: [f64; 2],
    pub(crate) playhead: RationalTime,
}

/// Ellipse配置はRectangleと同じdrop位置とplayheadだけで決まるため、requestを共有する。
pub(crate) type PlaceEllipseRequest = PlaceRectangleRequest;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlaceVismRequest {
    pub(crate) plugin_id: String,
    pub(crate) position: [f64; 2],
    pub(crate) playhead: RationalTime,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlaceMediaRequest {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) asset_type: String,
    pub(crate) position: [f64; 2],
    pub(crate) playhead: RationalTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AttachEffectRequest {
    pub(crate) plugin_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetEffectParamRequest {
    pub(super) layer_id: LayerId,
    pub(super) effect_use_id: EffectId,
    pub(super) definition_id: EffectDefinitionId,
    pub(super) plugin_id: String,
    pub(super) effect_version: u32,
    pub(super) param_id: String,
    pub(super) new_value: DocParam,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetSourceParamRequest {
    pub(super) layer_id: LayerId,
    pub(super) param_id: String,
    pub(super) new_value: DocParam,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AddPositionKeyRequest {
    pub(crate) target: LayerId,
    pub(crate) time: RationalTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AddTransformParamKeyRequest {
    pub(crate) target: LayerId,
    pub(crate) property: ScalarPropertyId,
    pub(crate) time: RationalTime,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SetPositionConstRequest {
    pub(crate) target: LayerId,
    pub(crate) old: [f64; 2],
    pub(crate) new: [f64; 2],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SetOpacityRequest {
    pub(crate) target: LayerId,
    pub(crate) value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SetPositionKeyInterpRequest {
    pub(crate) target: LayerId,
    pub(crate) key: KeyframeId,
    pub(crate) interp: Interp,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SetPositionKeyValueRequest {
    pub(crate) target: LayerId,
    pub(crate) key: KeyframeId,
    pub(crate) old: [f64; 2],
    pub(crate) new: [f64; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SetPositionKeyTimeRequest {
    pub(crate) target: LayerId,
    pub(crate) key: KeyframeId,
    pub(crate) old: RationalTime,
    pub(crate) new: RationalTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RemovePositionKeyRequest {
    pub(crate) target: LayerId,
    pub(crate) key: KeyframeId,
}

impl SetEffectParamRequest {
    pub(crate) fn new(
        layer_id: LayerId,
        effect_use_id: EffectId,
        definition_id: EffectDefinitionId,
        plugin_id: String,
        effect_version: u32,
        param_id: String,
        value: f64,
    ) -> Self {
        Self {
            layer_id,
            effect_use_id,
            definition_id,
            plugin_id,
            effect_version,
            param_id,
            new_value: DocParam::const_f64(value),
        }
    }

    pub(crate) fn with_param(
        layer_id: LayerId,
        effect_use_id: EffectId,
        definition_id: EffectDefinitionId,
        plugin_id: String,
        effect_version: u32,
        param_id: String,
        new_value: DocParam,
    ) -> Self {
        Self {
            layer_id,
            effect_use_id,
            definition_id,
            plugin_id,
            effect_version,
            param_id,
            new_value,
        }
    }
}

impl SetSourceParamRequest {
    pub(crate) fn new(layer_id: LayerId, param_id: String, new_value: DocParam) -> Self {
        Self {
            layer_id,
            param_id,
            new_value,
        }
    }
}
