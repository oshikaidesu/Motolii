//! VSM-A0I-2: raw plugin recipeを変更しないprepared解決。

use std::collections::BTreeMap;

use motolii_eval::Value;
use motolii_plugin::{
    F64Domain, MigrationOp, ParamDef, PluginCatalog, PluginKind, PluginRuntime, ValueType,
};

use crate::doc_value::DocValue;
use crate::param::DocParam;
use crate::param_expect::{ExpectedValueType, ParamConstraints};
use crate::schema::{ClipSource, TrackItem};
use crate::stable_id::EffectDefinitionId;
use crate::validate::{validate_param, DocumentError};
use crate::{
    open_project_with_limits, AssetId, Document, LayerId, OpenProjectOutcome, ProjectError,
    ResourceLimits,
};

const RECT_LAYER_SOURCE_ID: &str = "doc.layer_source.rect";
const RECT_LAYER_SOURCE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PluginSlotId {
    LayerSource(LayerId),
    EffectDefinition(EffectDefinitionId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedPluginRecipe {
    pub plugin_id: String,
    pub saved_version: u32,
    pub current_version: u32,
    pub params: BTreeMap<String, DocParam>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginDiagnosticReason {
    ContractMissing,
    FutureVersion {
        current_version: u32,
        saved_version: u32,
    },
    MigrationStepMissing {
        from_version: u32,
    },
    MigrationConflict {
        from: String,
        to: String,
    },
    ContractViolation,
    ExecutorMissing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginDiagnostic {
    pub slot: PluginSlotId,
    pub plugin_id: String,
    pub reason: PluginDiagnosticReason,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedDocumentPlugins {
    recipes: BTreeMap<PluginSlotId, PreparedPluginRecipe>,
    diagnostics: Vec<PluginDiagnostic>,
}

#[derive(Debug)]
pub struct ResolvedOpenProjectOutcome {
    pub recovered: OpenProjectOutcome,
    pub plugins: PreparedDocumentPlugins,
}

impl PreparedDocumentPlugins {
    pub fn get(&self, slot: &PluginSlotId) -> Option<&PreparedPluginRecipe> {
        self.recipes.get(slot)
    }

    pub fn diagnostics(&self) -> &[PluginDiagnostic] {
        &self.diagnostics
    }

    pub fn is_fully_prepared(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn execution_diagnostics(&self, runtime: &PluginRuntime) -> Vec<PluginDiagnostic> {
        self.recipes
            .iter()
            .filter_map(|(slot, recipe)| {
                let executors = runtime.executors();
                let available = match slot {
                    PluginSlotId::LayerSource(_) => {
                        executors.layer_source_by_name(&recipe.plugin_id).is_some()
                    }
                    PluginSlotId::EffectDefinition(_) => {
                        executors.filter_by_name(&recipe.plugin_id).is_some()
                    }
                };
                (!available).then(|| PluginDiagnostic {
                    slot: slot.clone(),
                    plugin_id: recipe.plugin_id.clone(),
                    reason: PluginDiagnosticReason::ExecutorMissing,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DocumentPluginError {
    #[error(transparent)]
    Structural(#[from] DocumentError),
    #[error(
        "plugin `{plugin_id}` kind mismatch: expected {expected:?}, contract declares {actual:?}"
    )]
    KindMismatch {
        plugin_id: String,
        expected: PluginKind,
        actual: PluginKind,
    },
    #[error("plugin contract is missing for `{plugin_id}`")]
    ContractMissing { plugin_id: String },
    #[error(
        "plugin `{plugin_id}` recipe version {saved_version} is newer than contract {current_version}"
    )]
    FutureVersion {
        plugin_id: String,
        current_version: u32,
        saved_version: u32,
    },
    #[error("plugin `{plugin_id}` has no migration step from version {from_version}")]
    MigrationStepMissing {
        plugin_id: String,
        from_version: u32,
    },
    #[error("plugin `{plugin_id}` migration conflicts: both `{from}` and `{to}` exist")]
    MigrationConflict {
        plugin_id: String,
        from: String,
        to: String,
    },
    #[error("plugin `{plugin_id}` contract violation at `{param}`: {source}")]
    ContractViolation {
        plugin_id: String,
        param: String,
        #[source]
        source: DocumentError,
    },
    #[error("plugin `{plugin_id}` default for `{param}` cannot be represented in Document")]
    InvalidDefault { plugin_id: String, param: String },
}

impl Document {
    pub fn prepare_plugins(
        &self,
        catalog: &PluginCatalog,
    ) -> Result<PreparedDocumentPlugins, DocumentPluginError> {
        self.validate()?;
        let mut prepared = PreparedDocumentPlugins::default();
        for track in &self.tracks {
            collect_item_recipes(self, &track.items, catalog, &mut prepared)?;
        }
        for definition in &self.effect_definitions {
            let slot = PluginSlotId::EffectDefinition(definition.id);
            prepare_document_slot(
                self,
                slot,
                &definition.plugin_id,
                PluginKind::Filter,
                definition.effect_version,
                &definition.params,
                catalog,
                &mut prepared,
            )?;
        }
        Ok(prepared)
    }
}

pub fn open_project_resolved(
    document_path: &std::path::Path,
    limits: &ResourceLimits,
    catalog: &PluginCatalog,
) -> Result<ResolvedOpenProjectOutcome, ProjectError> {
    let recovered = open_project_with_limits(document_path, limits)?;
    let plugins = recovered.document.prepare_plugins(catalog)?;
    Ok(ResolvedOpenProjectOutcome { recovered, plugins })
}

pub fn prepare_plugin_recipe(
    plugin_id: &str,
    expected_kind: PluginKind,
    saved_version: u32,
    params: &BTreeMap<String, DocParam>,
    catalog: &PluginCatalog,
) -> Result<PreparedPluginRecipe, DocumentPluginError> {
    prepare_recipe(plugin_id, expected_kind, saved_version, params, catalog)
}

fn collect_item_recipes(
    doc: &Document,
    items: &[TrackItem],
    catalog: &PluginCatalog,
    prepared: &mut PreparedDocumentPlugins,
) -> Result<(), DocumentPluginError> {
    for item in items {
        match item {
            TrackItem::Clip(clip) => {
                if let ClipSource::Plugin {
                    plugin_id,
                    effect_version,
                    params,
                    ..
                } = &clip.source
                {
                    prepare_document_slot(
                        doc,
                        PluginSlotId::LayerSource(clip.envelope.layer_id),
                        plugin_id,
                        PluginKind::LayerSource,
                        *effect_version,
                        params,
                        catalog,
                        prepared,
                    )?;
                }
            }
            TrackItem::Group(group) => {
                collect_item_recipes(doc, &group.children, catalog, prepared)?;
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn prepare_document_slot(
    doc: &Document,
    slot: PluginSlotId,
    plugin_id: &str,
    expected_kind: PluginKind,
    saved_version: u32,
    params: &BTreeMap<String, DocParam>,
    catalog: &PluginCatalog,
    prepared: &mut PreparedDocumentPlugins,
) -> Result<(), DocumentPluginError> {
    if plugin_id == RECT_LAYER_SOURCE_ID {
        if expected_kind != PluginKind::LayerSource {
            return Err(DocumentPluginError::KindMismatch {
                plugin_id: plugin_id.to_string(),
                expected: expected_kind,
                actual: PluginKind::LayerSource,
            });
        }
        if saved_version > RECT_LAYER_SOURCE_VERSION {
            prepared.diagnostics.push(PluginDiagnostic {
                slot,
                plugin_id: plugin_id.to_string(),
                reason: PluginDiagnosticReason::FutureVersion {
                    current_version: RECT_LAYER_SOURCE_VERSION,
                    saved_version,
                },
            });
        }
        return Ok(());
    }

    match prepare_recipe(plugin_id, expected_kind, saved_version, params, catalog) {
        Ok(recipe) => {
            validate_prepared_recipe(doc, &recipe, catalog)?;
            prepared.recipes.insert(slot, recipe);
        }
        Err(error) => {
            let reason = match &error {
                DocumentPluginError::ContractMissing { .. } => {
                    Some(PluginDiagnosticReason::ContractMissing)
                }
                DocumentPluginError::FutureVersion {
                    current_version,
                    saved_version,
                    ..
                } => Some(PluginDiagnosticReason::FutureVersion {
                    current_version: *current_version,
                    saved_version: *saved_version,
                }),
                DocumentPluginError::MigrationStepMissing { from_version, .. } => {
                    Some(PluginDiagnosticReason::MigrationStepMissing {
                        from_version: *from_version,
                    })
                }
                DocumentPluginError::MigrationConflict { from, to, .. } => {
                    Some(PluginDiagnosticReason::MigrationConflict {
                        from: from.clone(),
                        to: to.clone(),
                    })
                }
                _ => None,
            };
            if let Some(reason) = reason {
                prepared.diagnostics.push(PluginDiagnostic {
                    slot,
                    plugin_id: plugin_id.to_string(),
                    reason,
                });
            } else {
                return Err(error);
            }
        }
    }
    Ok(())
}

fn prepare_recipe(
    plugin_id: &str,
    expected_kind: PluginKind,
    saved_version: u32,
    params: &BTreeMap<String, DocParam>,
    catalog: &PluginCatalog,
) -> Result<PreparedPluginRecipe, DocumentPluginError> {
    let contract = catalog
        .get(plugin_id)
        .ok_or_else(|| DocumentPluginError::ContractMissing {
            plugin_id: plugin_id.to_string(),
        })?;
    if contract.kind != expected_kind {
        return Err(DocumentPluginError::KindMismatch {
            plugin_id: plugin_id.to_string(),
            expected: expected_kind,
            actual: contract.kind,
        });
    }
    if saved_version > contract.node.version {
        return Err(DocumentPluginError::FutureVersion {
            plugin_id: plugin_id.to_string(),
            current_version: contract.node.version,
            saved_version,
        });
    }

    let mut migrated = params.clone();
    let mut version = saved_version;
    while version < contract.node.version {
        let step = contract
            .migrations
            .iter()
            .find(|step| step.from_version == version)
            .ok_or_else(|| DocumentPluginError::MigrationStepMissing {
                plugin_id: plugin_id.to_string(),
                from_version: version,
            })?;
        for op in &step.ops {
            match op {
                MigrationOp::RenameParam { from, to } => {
                    if migrated.contains_key(*from) && migrated.contains_key(*to) {
                        return Err(DocumentPluginError::MigrationConflict {
                            plugin_id: plugin_id.to_string(),
                            from: (*from).to_string(),
                            to: (*to).to_string(),
                        });
                    }
                    let value = migrated.remove(*from).ok_or_else(|| {
                        DocumentPluginError::ContractViolation {
                            plugin_id: plugin_id.to_string(),
                            param: (*from).to_string(),
                            source: DocumentError::ParamTypeMismatch {
                                path: format!("{plugin_id}.{from}"),
                                expected: "parameter present in saved schema".to_string(),
                                got: "missing".to_string(),
                            },
                        }
                    })?;
                    migrated.insert((*to).to_string(), value);
                }
            }
        }
        version = step.to_version;
    }

    for name in migrated.keys() {
        if !contract.node.params.iter().any(|param| param.id == name) {
            return Err(DocumentPluginError::ContractViolation {
                plugin_id: plugin_id.to_string(),
                param: name.clone(),
                source: DocumentError::ParamTypeMismatch {
                    path: format!("{plugin_id}.{name}"),
                    expected: "parameter defined by current PluginContract".to_string(),
                    got: "unknown parameter".to_string(),
                },
            });
        }
    }
    for param in &contract.node.params {
        if !migrated.contains_key(param.id) {
            migrated.insert(
                param.id.to_string(),
                DocParam::Const(default_doc_value(plugin_id, param)?),
            );
        }
    }

    Ok(PreparedPluginRecipe {
        plugin_id: plugin_id.to_string(),
        saved_version,
        current_version: contract.node.version,
        params: migrated,
    })
}

fn validate_prepared_recipe(
    doc: &Document,
    recipe: &PreparedPluginRecipe,
    catalog: &PluginCatalog,
) -> Result<(), DocumentPluginError> {
    let contract =
        catalog
            .get(&recipe.plugin_id)
            .ok_or_else(|| DocumentPluginError::ContractMissing {
                plugin_id: recipe.plugin_id.clone(),
            })?;
    for definition in &contract.node.params {
        let value = recipe.params.get(definition.id).ok_or_else(|| {
            DocumentPluginError::ContractViolation {
                plugin_id: recipe.plugin_id.clone(),
                param: definition.id.to_string(),
                source: DocumentError::ParamTypeMismatch {
                    path: format!("{}.{}", recipe.plugin_id, definition.id),
                    expected: definition.value_type.to_string(),
                    got: "missing".to_string(),
                },
            }
        })?;
        validate_param(
            doc,
            value,
            constraints_for(definition),
            &format!("{}.{}", recipe.plugin_id, definition.id),
        )
        .map_err(|source| DocumentPluginError::ContractViolation {
            plugin_id: recipe.plugin_id.clone(),
            param: definition.id.to_string(),
            source,
        })?;
    }
    Ok(())
}

fn constraints_for(param: &ParamDef) -> ParamConstraints {
    let expected = match param.value_type {
        ValueType::F64 => ExpectedValueType::F64,
        ValueType::Vec2 => ExpectedValueType::Vec2,
        ValueType::Vec3 => ExpectedValueType::Vec3,
        ValueType::Color => ExpectedValueType::Color,
        ValueType::AssetRef => ExpectedValueType::AssetRef,
    };
    let mut constraints = if expected == ExpectedValueType::Color {
        ParamConstraints::color()
    } else {
        ParamConstraints::typed(expected)
    };
    if let Some(F64Domain {
        min_inclusive,
        max_inclusive,
        integer,
    }) = param.f64_domain
    {
        constraints.min = min_inclusive;
        constraints.max = max_inclusive;
        constraints.integer = integer;
    }
    constraints
}

fn default_doc_value(plugin_id: &str, param: &ParamDef) -> Result<DocValue, DocumentPluginError> {
    match &param.default {
        Value::F64(value) => Ok(DocValue::F64(*value)),
        Value::Vec2(value) => Ok(DocValue::Vec2(*value)),
        Value::Vec3(value) => Ok(DocValue::Vec3(*value)),
        Value::Color(value) => Ok(DocValue::Color(*value)),
        Value::AssetRef(value) => Ok(DocValue::AssetRef(AssetId::from_raw(*value))),
        #[allow(unreachable_patterns)]
        _ => Err(DocumentPluginError::InvalidDefault {
            plugin_id: plugin_id.to_string(),
            param: param.id.to_string(),
        }),
    }
}
