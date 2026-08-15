//! 確定済みDocument編集をsingle writerへ直列配送するprivate runtime。
//!
//! 責任は各moduleが持ち、ここは組み立てと既存crate経路の再exportだけを行う。

mod action;
mod commit;
mod error;
mod prepare_params;
mod prepare_place;
mod process;
mod process_clips;
mod process_keys;
mod process_params;
mod process_place;
mod requests;
mod runtime;

#[cfg(test)]
mod tests;

pub(crate) use action::{
    DocumentEditAction, DocumentEditActionKind, DocumentEditDispatchError, DocumentEditQueue,
};
pub(crate) use error::{DocumentEditRuntimeError, PublishedDocument};
pub(crate) use prepare_params::{
    prepare_set_effect_param_command, prepare_set_source_param_command,
};
pub(crate) use requests::{
    AddPositionKeyRequest, AddTransformParamKeyRequest, AttachEffectRequest, PlaceEllipseRequest,
    PlaceMediaRequest, PlaceRectangleRequest, PlaceVismRequest, RemovePositionKeyRequest,
    SetEffectParamRequest, SetOpacityRequest, SetPositionConstRequest, SetPositionKeyInterpRequest,
    SetPositionKeyTimeRequest, SetPositionKeyValueRequest, SetSourceParamRequest,
};
pub(crate) use runtime::DocumentEditRuntime;
#[cfg(test)]
pub(crate) use runtime::RuntimeTestFailpoint;
