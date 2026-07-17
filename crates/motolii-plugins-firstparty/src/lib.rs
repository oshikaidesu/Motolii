//! first-party 既定プラグインの composition root。
use std::sync::Arc;

use motolii_plugin::reference::{register_reference_contracts, register_reference_plugins};
use motolii_plugin::{
    PluginCatalog, PluginCatalogBuilder, PluginContractError, PluginError, PluginKind,
    PluginRegistry, PluginRuntime, PluginRuntimeError,
};
use motolii_plugin_opacity::{opacity_contract, OPACITY_FILTER};
use motolii_plugin_sine::{sine_contract, SINE_PARAM_DRIVER};

const REQUIRED_HOST_CAPABILITIES: &[&str] = &["core.filter.opacity"];

#[derive(Debug, thiserror::Error)]
pub enum FirstPartyError {
    #[error(transparent)]
    Contract(#[from] PluginContractError),
    #[error(transparent)]
    Plugin(#[from] PluginError),
    #[error(transparent)]
    Runtime(#[from] PluginRuntimeError),
    #[error("required host capability `{id}` missing from first-party assembly")]
    RequiredCapabilityMissing { id: &'static str },
}

pub fn first_party_catalog() -> Result<PluginCatalog, PluginContractError> {
    let mut builder = PluginCatalogBuilder::new();
    register_reference_contracts(&mut builder)?;
    builder.register(opacity_contract())?;
    builder.register(sine_contract())?;
    builder.build()
}

pub fn first_party_registry() -> Result<PluginRegistry, PluginError> {
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry)?;
    registry.register_filter(&OPACITY_FILTER)?;
    registry.register_param_driver(&SINE_PARAM_DRIVER)?;
    Ok(registry)
}

pub fn first_party_runtime() -> Result<PluginRuntime, FirstPartyError> {
    let catalog = Arc::new(first_party_catalog()?);
    let registry = first_party_registry()?;
    let runtime = PluginRuntime::try_new(catalog, registry)?;
    ensure_required_host_capabilities(runtime.catalog(), runtime.executors())?;
    Ok(runtime)
}

fn ensure_required_host_capabilities(
    catalog: &PluginCatalog,
    executors: &PluginRegistry,
) -> Result<(), FirstPartyError> {
    for &id in REQUIRED_HOST_CAPABILITIES {
        if catalog.get(id).is_none() || !executor_has_id(executors, id) {
            return Err(FirstPartyError::RequiredCapabilityMissing { id });
        }
    }
    Ok(())
}

fn executor_has_id(executors: &PluginRegistry, id: &str) -> bool {
    [
        PluginKind::LayerSource,
        PluginKind::Filter,
        PluginKind::ParamDriver,
        PluginKind::Composite,
    ]
    .into_iter()
    .any(|kind| executors.iter(kind).any(|(plugin_id, _)| plugin_id.0 == id))
}

#[cfg(test)]
mod tests {
    use motolii_plugin::reference::reference_catalog;

    use super::*;

    #[test]
    fn missing_opacity_executor_is_rejected_before_graph_eval() {
        let catalog = first_party_catalog().unwrap();
        let registry = PluginRegistry::new();

        let err = ensure_required_host_capabilities(&catalog, &registry).unwrap_err();
        assert!(matches!(
            err,
            FirstPartyError::RequiredCapabilityMissing {
                id: "core.filter.opacity"
            }
        ));
    }

    #[test]
    fn missing_opacity_contract_is_rejected_before_graph_eval() {
        let catalog = reference_catalog().unwrap();
        let registry = first_party_registry().unwrap();

        let err = ensure_required_host_capabilities(&catalog, &registry).unwrap_err();
        assert!(matches!(
            err,
            FirstPartyError::RequiredCapabilityMissing {
                id: "core.filter.opacity"
            }
        ));
    }
}
