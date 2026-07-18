//! 製品入口ごとの登録漏れを防ぐため、first-party既定組み立てを一箇所に閉じる。

use std::sync::Arc;

use motolii_plugin::{
    reference::{register_reference_contracts, register_reference_plugins},
    PluginCatalog, PluginCatalogBuilder, PluginContractError, PluginError, PluginRegistry,
    PluginRuntime, PluginRuntimeError,
};
use motolii_plugin_opacity::{opacity_contract, OPACITY_FILTER};
use thiserror::Error;

/// v1 Host必須capability。Document graphがenvelope opacityをこのIDへlowerする。
const REQUIRED_CAPABILITIES: &[&str] = &["core.filter.opacity"];

#[derive(Debug, Error)]
pub enum FirstPartyError {
    #[error(transparent)]
    Contract(#[from] PluginContractError),
    #[error(transparent)]
    Plugin(#[from] PluginError),
    #[error(transparent)]
    Runtime(#[from] PluginRuntimeError),
    #[error("required capability missing from first-party assembly: {id}")]
    MissingRequiredCapability { id: &'static str },
}

pub fn first_party_catalog() -> Result<PluginCatalog, PluginContractError> {
    let mut builder = PluginCatalogBuilder::new();
    register_reference_contracts(&mut builder)?;
    builder.register(opacity_contract())?;
    builder.build()
}

pub fn first_party_registry() -> Result<PluginRegistry, PluginError> {
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry)?;
    registry.register_filter(&OPACITY_FILTER)?;
    Ok(registry)
}

pub fn first_party_runtime() -> Result<PluginRuntime, FirstPartyError> {
    let catalog = Arc::new(first_party_catalog()?);
    let registry = first_party_registry()?;
    let runtime = PluginRuntime::try_new(catalog, registry)?;
    ensure_required_capabilities(runtime.catalog(), runtime.executors())?;
    Ok(runtime)
}

fn ensure_required_capabilities(
    catalog: &PluginCatalog,
    registry: &PluginRegistry,
) -> Result<(), FirstPartyError> {
    for &id in REQUIRED_CAPABILITIES {
        if catalog.get(id).is_none() {
            return Err(FirstPartyError::MissingRequiredCapability { id });
        }
        let has_executor = registry.filter_by_name(id).is_some()
            || registry.param_driver_by_name(id).is_some()
            || registry.layer_source_by_name(id).is_some()
            || registry.composite_by_name(id).is_some();
        if !has_executor {
            return Err(FirstPartyError::MissingRequiredCapability { id });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_plugin::reference::reference_catalog;

    #[test]
    fn missing_catalog_capability_is_rejected() {
        let catalog = reference_catalog().unwrap();
        let registry = first_party_registry().unwrap();
        let err = ensure_required_capabilities(&catalog, &registry).unwrap_err();
        assert!(matches!(
            err,
            FirstPartyError::MissingRequiredCapability {
                id: "core.filter.opacity"
            }
        ));
    }

    #[test]
    fn missing_executor_capability_is_rejected() {
        use motolii_plugin::reference::register_reference_plugins;

        let catalog = first_party_catalog().unwrap();
        let mut registry = PluginRegistry::new();
        register_reference_plugins(&mut registry).unwrap();
        let err = ensure_required_capabilities(&catalog, &registry).unwrap_err();
        assert!(matches!(
            err,
            FirstPartyError::MissingRequiredCapability {
                id: "core.filter.opacity"
            }
        ));
    }
}
