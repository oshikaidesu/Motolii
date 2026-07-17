#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use motolii_eval::Value;
use motolii_plugin::reference::{reference_catalog, register_reference_plugins, OPACITY_FILTER};
use motolii_plugin::{
    DomainError, F64Domain, FilterPlugin, MigrationOp, MigrationPlanError, MigrationStep, NodeDesc,
    ParamDef, PluginCatalogBuilder, PluginContract, PluginContractError, PluginId, PluginKind,
    PluginRegistry, PluginRuntime, PluginRuntimeError, ValueType,
};

fn filter_contract(id: &'static str, version: u32, params: Vec<ParamDef>) -> PluginContract {
    PluginContract {
        kind: PluginKind::Filter,
        node: NodeDesc {
            id: PluginId(id),
            version,
            display_name: "Test Filter",
            category: "Utility",
            tags: &["test"],
            params,
            min_inputs: 1,
            max_inputs: 1,
        },
        migrations: vec![],
    }
}

fn scalar(id: &'static str, default: f64, domain: Option<F64Domain>) -> ParamDef {
    ParamDef {
        id,
        value_type: ValueType::F64,
        default: Value::F64(default),
        f64_domain: domain,
    }
}

#[test]
fn reference_catalog_owns_opacity_domain_and_sine_migration() {
    let catalog = reference_catalog().unwrap();
    assert_eq!(catalog.len(), 6);

    let opacity = catalog.get("core.filter.opacity").unwrap();
    assert_eq!(opacity.kind, PluginKind::Filter);
    assert_eq!(opacity.node.params[0].f64_domain, Some(F64Domain::unit()));

    let sine = catalog.get("core.param.sine").unwrap();
    assert_eq!(
        sine.migrations,
        vec![MigrationStep {
            from_version: 1,
            to_version: 2,
            ops: vec![MigrationOp::RenameParam {
                from: "amp",
                to: "amplitude",
            }],
        }]
    );
}

#[test]
fn catalog_rejects_duplicate_id() {
    let mut builder = PluginCatalogBuilder::new();
    builder
        .register(filter_contract("test.filter.duplicate", 1, vec![]))
        .unwrap();
    let err = builder
        .register(filter_contract("test.filter.duplicate", 1, vec![]))
        .unwrap_err();
    assert!(matches!(
        err,
        PluginContractError::DuplicateContract {
            id: "test.filter.duplicate"
        }
    ));
}

#[test]
fn catalog_rejects_invalid_numeric_domains() {
    let cases = [
        (
            scalar(
                "amount",
                0.5,
                Some(F64Domain::new(Some(f64::NAN), Some(1.0), false)),
            ),
            DomainError::NonFiniteBound,
        ),
        (
            scalar(
                "amount",
                0.5,
                Some(F64Domain::new(Some(2.0), Some(1.0), false)),
            ),
            DomainError::ReversedBounds,
        ),
        (
            scalar("amount", 1.5, Some(F64Domain::unit())),
            DomainError::DefaultOutsideDomain,
        ),
    ];

    for (param, expected) in cases {
        let mut builder = PluginCatalogBuilder::new();
        let err = builder
            .register(filter_contract("test.filter.domain", 1, vec![param]))
            .unwrap_err();
        assert!(matches!(
            err,
            PluginContractError::InvalidDomain { reason, .. } if reason == expected
        ));
    }
}

#[test]
fn catalog_rejects_non_f64_domain_and_non_finite_default() {
    let color_with_domain = ParamDef {
        id: "color",
        value_type: ValueType::Color,
        default: Value::Color([0.0, 0.0, 0.0, 1.0]),
        f64_domain: Some(F64Domain::unit()),
    };
    let mut builder = PluginCatalogBuilder::new();
    let err = builder
        .register(filter_contract(
            "test.filter.color_domain",
            1,
            vec![color_with_domain],
        ))
        .unwrap_err();
    assert!(matches!(
        err,
        PluginContractError::InvalidDomain {
            reason: DomainError::NonF64Parameter,
            ..
        }
    ));

    let mut builder = PluginCatalogBuilder::new();
    let err = builder
        .register(filter_contract(
            "test.filter.non_finite",
            1,
            vec![scalar("amount", f64::INFINITY, None)],
        ))
        .unwrap_err();
    assert!(matches!(
        err,
        PluginContractError::InvalidDomain {
            reason: DomainError::NonFiniteDefault,
            ..
        }
    ));
}

#[test]
fn catalog_rejects_invalid_migration_plan() {
    let invalid_steps = [
        (
            MigrationStep {
                from_version: 1,
                to_version: 3,
                ops: vec![],
            },
            MigrationPlanError::NonAdjacentVersions,
        ),
        (
            MigrationStep {
                from_version: 2,
                to_version: 3,
                ops: vec![],
            },
            MigrationPlanError::BeyondCurrentVersion,
        ),
        (
            MigrationStep {
                from_version: 1,
                to_version: 2,
                ops: vec![MigrationOp::RenameParam {
                    from: "same",
                    to: "same",
                }],
            },
            MigrationPlanError::SameParamName,
        ),
    ];

    for (step, expected) in invalid_steps {
        let mut contract = filter_contract("test.filter.migrate", 2, vec![]);
        contract.migrations.push(step);
        let mut builder = PluginCatalogBuilder::new();
        let err = builder.register(contract).unwrap_err();
        assert!(matches!(
            err,
            PluginContractError::InvalidMigration { reason, .. } if reason == expected
        ));
    }
}

#[test]
fn catalog_rejects_duplicate_migration_source_version() {
    let mut contract = filter_contract("test.filter.duplicate_step", 3, vec![]);
    contract.migrations = vec![
        MigrationStep {
            from_version: 1,
            to_version: 2,
            ops: vec![],
        },
        MigrationStep {
            from_version: 1,
            to_version: 2,
            ops: vec![],
        },
    ];
    let mut builder = PluginCatalogBuilder::new();
    let err = builder.register(contract).unwrap_err();
    assert!(matches!(
        err,
        PluginContractError::InvalidMigration {
            reason: MigrationPlanError::DuplicateFromVersion,
            ..
        }
    ));
}

#[test]
fn runtime_allows_contract_only_catalog() {
    let catalog = Arc::new(reference_catalog().unwrap());
    let runtime = PluginRuntime::try_new(catalog, PluginRegistry::new()).unwrap();
    assert_eq!(runtime.catalog().len(), 6);
    assert_eq!(runtime.executors().len(PluginKind::Filter), 0);
}

#[test]
fn runtime_rejects_executor_without_contract() {
    let catalog = Arc::new(PluginCatalogBuilder::new().build().unwrap());
    let mut executors = PluginRegistry::new();
    executors.register_filter(&OPACITY_FILTER).unwrap();
    let err = PluginRuntime::try_new(catalog, executors).unwrap_err();
    assert!(matches!(
        err,
        PluginRuntimeError::ExecutorContractMissing {
            id: "core.filter.opacity",
            kind: PluginKind::Filter,
        }
    ));
}

#[test]
fn runtime_rejects_version_and_descriptor_mismatch() {
    let mut version_contract = filter_contract(
        "core.filter.opacity",
        2,
        OPACITY_FILTER.desc().params.clone(),
    );
    version_contract.node.display_name = OPACITY_FILTER.desc().display_name;
    version_contract.node.category = OPACITY_FILTER.desc().category;
    version_contract.node.tags = OPACITY_FILTER.desc().tags;
    let mut builder = PluginCatalogBuilder::new();
    builder.register(version_contract).unwrap();
    let mut executors = PluginRegistry::new();
    executors.register_filter(&OPACITY_FILTER).unwrap();
    let err = PluginRuntime::try_new(Arc::new(builder.build().unwrap()), executors).unwrap_err();
    assert!(matches!(
        err,
        PluginRuntimeError::VersionMismatch {
            id: "core.filter.opacity",
            contract: 2,
            executor: 1,
        }
    ));

    let mut descriptor_contract = PluginContract {
        kind: PluginKind::Filter,
        node: OPACITY_FILTER.desc().clone(),
        migrations: vec![],
    };
    descriptor_contract.node.display_name = "Different Name";
    let mut builder = PluginCatalogBuilder::new();
    builder.register(descriptor_contract).unwrap();
    let mut executors = PluginRegistry::new();
    executors.register_filter(&OPACITY_FILTER).unwrap();
    let err = PluginRuntime::try_new(Arc::new(builder.build().unwrap()), executors).unwrap_err();
    assert!(matches!(
        err,
        PluginRuntimeError::DescriptorMismatch {
            id: "core.filter.opacity"
        }
    ));
}

#[test]
fn reference_catalog_and_executor_registry_form_valid_runtime() {
    let catalog = Arc::new(reference_catalog().unwrap());
    let mut executors = PluginRegistry::new();
    register_reference_plugins(&mut executors).unwrap();
    let runtime = PluginRuntime::try_new(catalog, executors).unwrap();
    assert_eq!(runtime.catalog().len(), 6);
    assert_eq!(runtime.executors().len(PluginKind::Filter), 3);
}
