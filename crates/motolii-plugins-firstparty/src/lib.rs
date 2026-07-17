//! 製品入口ごとの登録漏れを防ぐため、first-party既定組み立てを一箇所に閉じる。

use std::sync::Arc;

use motolii_plugin::{
    reference::{reference_catalog, register_reference_plugins},
    PluginCatalog, PluginContractError, PluginError, PluginRegistry, PluginRuntime,
    PluginRuntimeError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FirstPartyError {
    #[error(transparent)]
    Contract(#[from] PluginContractError),
    #[error(transparent)]
    Plugin(#[from] PluginError),
    #[error(transparent)]
    Runtime(#[from] PluginRuntimeError),
}

pub fn first_party_catalog() -> Result<PluginCatalog, PluginContractError> {
    reference_catalog()
}

pub fn first_party_registry() -> Result<PluginRegistry, PluginError> {
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry)?;
    Ok(registry)
}

pub fn first_party_runtime() -> Result<PluginRuntime, FirstPartyError> {
    let catalog = Arc::new(first_party_catalog()?);
    let registry = first_party_registry()?;
    Ok(PluginRuntime::try_new(catalog, registry)?)
}
