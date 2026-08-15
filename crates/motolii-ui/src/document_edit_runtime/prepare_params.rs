//! 確定requestを既存Document commandへ落とす。param/effectだけ。

use std::collections::BTreeMap;

use motolii_core::RationalTime;
use motolii_doc::{
    ClipSource, Command, DocParam, DocValue, Document, DocumentPluginError, DocumentWriter,
    DraftDocParam, EffectDefinitionDraft, EffectId, LayerId, PreparedPluginRecipe,
    ScalarPropertyId, TrackItem,
};
use motolii_plugin::{PluginCatalog, PluginKind};

use super::error::DocumentEditRuntimeError;
use super::requests::{
    AttachEffectRequest, SetEffectParamRequest, SetOpacityRequest, SetPositionConstRequest,
    SetSourceParamRequest,
};

pub(crate) fn prepare_set_effect_param_command(
    document: &Document,
    request: &SetEffectParamRequest,
) -> Option<Command> {
    if !doc_param_numeric_finite(&request.new_value) {
        return None;
    }
    let effect_use = document.find_effect_use(request.layer_id, request.effect_use_id)?;
    if effect_use.definition_id != request.definition_id {
        return None;
    }
    let definition = document.effect_definition(effect_use.definition_id)?;
    if definition.plugin_id != request.plugin_id
        || definition.effect_version != request.effect_version
    {
        return None;
    }
    let old_value = definition.params.get(&request.param_id)?;
    if !effect_param_types_match(old_value, &request.new_value) {
        return None;
    }
    if old_value == &request.new_value {
        return None;
    }
    Some(Command::SetProperty {
        target: request.layer_id,
        property: ScalarPropertyId::EffectParam(request.effect_use_id, request.param_id.clone()),
        old_value: old_value.clone(),
        new_value: request.new_value.clone(),
    })
}

pub(super) fn effect_param_types_match(old: &DocParam, new: &DocParam) -> bool {
    matches!(
        (old, new),
        (
            DocParam::Const(DocValue::F64(_)),
            DocParam::Const(DocValue::F64(_))
        ) | (
            DocParam::Const(DocValue::Color(_)),
            DocParam::Const(DocValue::Color(_))
        )
    )
}

pub(crate) fn prepare_set_source_param_command(
    document: &Document,
    request: &SetSourceParamRequest,
) -> Option<Command> {
    if request.param_id.is_empty() || !doc_param_numeric_finite(&request.new_value) {
        return None;
    }
    let old_value = clip_plugin_param(document, request.layer_id, &request.param_id)?;
    if old_value == request.new_value {
        return None;
    }
    Some(Command::SetProperty {
        target: request.layer_id,
        property: ScalarPropertyId::SourceParam(request.param_id.clone()),
        old_value,
        new_value: request.new_value.clone(),
    })
}

pub(super) fn clip_plugin_param(
    document: &Document,
    layer: LayerId,
    param_id: &str,
) -> Option<DocParam> {
    fn walk<'a>(items: &'a [TrackItem], layer: LayerId, param_id: &str) -> Option<DocParam> {
        for item in items {
            match item {
                TrackItem::Clip(clip) if clip.envelope.layer_id == layer => {
                    let ClipSource::Plugin { params, .. } = &clip.source else {
                        return None;
                    };
                    return params.get(param_id).cloned();
                }
                TrackItem::Group(group) => {
                    if let Some(found) = walk(&group.children, layer, param_id) {
                        return Some(found);
                    }
                }
                TrackItem::Clip(_) => {}
            }
        }
        None
    }
    document
        .tracks
        .iter()
        .find_map(|track| walk(&track.items, layer, param_id))
}

pub(super) fn doc_param_numeric_finite(param: &DocParam) -> bool {
    match param {
        DocParam::Const(DocValue::F64(value)) => value.is_finite(),
        DocParam::Const(DocValue::Vec2(value)) => {
            value.iter().all(|component| component.is_finite())
        }
        DocParam::Const(DocValue::Vec3(value)) => {
            value.iter().all(|component| component.is_finite())
        }
        DocParam::Const(DocValue::Color(value)) => {
            value.iter().all(|component| component.is_finite())
        }
        _ => true,
    }
}

pub(super) fn prepare_set_position_const_command(
    writer: &DocumentWriter,
    request: SetPositionConstRequest,
) -> Option<Command> {
    if !request.new.iter().all(|value| value.is_finite()) {
        return None;
    }
    let envelope = writer.find_envelope(request.target)?;
    let DocParam::Const(DocValue::Vec2(old)) = &envelope.transform.position else {
        return None;
    };
    if *old != request.old || request.old == request.new {
        return None;
    }
    Some(Command::SetProperty {
        target: request.target,
        property: ScalarPropertyId::Position,
        old_value: DocParam::const_vec2(request.old),
        new_value: DocParam::const_vec2(request.new),
    })
}

pub(super) fn prepare_set_opacity_command(
    writer: &DocumentWriter,
    request: SetOpacityRequest,
    time: RationalTime,
) -> Option<Command> {
    if !request.value.is_finite() || !(0.0..=1.0).contains(&request.value) {
        return None;
    }
    let envelope = writer.find_envelope(request.target)?;
    match &envelope.opacity {
        DocParam::Const(DocValue::F64(old)) => {
            if !old.is_finite() || !(0.0..=1.0).contains(old) || *old == request.value {
                return None;
            }
            Some(Command::SetProperty {
                target: request.target,
                property: ScalarPropertyId::Opacity,
                old_value: DocParam::const_f64(*old),
                new_value: DocParam::const_f64(request.value),
            })
        }
        DocParam::Keyframes(track) => {
            let key = track.keys().iter().find(|key| key.t == time)?;
            writer
                .prepare_set_transform_param_key_value(
                    request.target,
                    ScalarPropertyId::Opacity,
                    key.id,
                    DocValue::F64(request.value),
                )
                .ok()?
        }
        _ => None,
    }
}

pub(super) fn prepare_attach_effect_command(
    writer: &DocumentWriter,
    catalog: &PluginCatalog,
    target: LayerId,
    index: usize,
    request: AttachEffectRequest,
) -> Result<(Command, EffectId), DocumentEditRuntimeError> {
    let current_version = catalog
        .get(&request.plugin_id)
        .ok_or_else(|| DocumentPluginError::ContractMissing {
            plugin_id: request.plugin_id.clone(),
        })?
        .node
        .version;
    let recipe = motolii_doc::prepare_plugin_recipe(
        &request.plugin_id,
        PluginKind::Filter,
        current_version,
        &BTreeMap::new(),
        catalog,
    )?;
    let draft = attach_effect_draft(recipe)?;
    let command = writer.prepare_create_effect(target, index, draft)?;
    let created_effect_use = match &command {
        Command::CreateEffect { use_, .. } => use_.id,
        _ => return Err(DocumentEditRuntimeError::AttachPrepareCommandMismatch),
    };
    Ok((command, created_effect_use))
}

pub(super) fn attach_effect_draft(
    recipe: PreparedPluginRecipe,
) -> Result<EffectDefinitionDraft, DocumentEditRuntimeError> {
    let mut params = BTreeMap::new();
    for (name, param) in recipe.params {
        let motolii_doc::DocParam::Const(value) = param else {
            return Err(DocumentEditRuntimeError::AttachDefaultNotConst { param: name });
        };
        params.insert(name, DraftDocParam::Const(value));
    }
    Ok(EffectDefinitionDraft {
        plugin_id: recipe.plugin_id,
        effect_version: recipe.current_version,
        enabled: true,
        params,
        extra: Default::default(),
    })
}
