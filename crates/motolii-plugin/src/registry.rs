//! 実行器の登録とcontract照合。

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::contract::{
    validate_node_desc, NodeDesc, PluginCatalog, PluginError, PluginId, PluginKind,
};
use crate::traits::{CompositePlugin, FilterPlugin, LayerSourcePlugin, ParamDriverPlugin};

#[derive(Default)]
pub struct PluginRegistry {
    layer_sources: BTreeMap<PluginId, &'static dyn LayerSourcePlugin>,
    filters: BTreeMap<PluginId, &'static dyn FilterPlugin>,
    param_drivers: BTreeMap<PluginId, &'static dyn ParamDriverPlugin>,
    composites: BTreeMap<PluginId, &'static dyn CompositePlugin>,
}

impl std::fmt::Debug for PluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRegistry")
            .field("layer_sources", &self.layer_sources.len())
            .field("filters", &self.filters.len())
            .field("param_drivers", &self.param_drivers.len())
            .field("composites", &self.composites.len())
            .finish()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_layer_source(
        &mut self,
        plugin: &'static dyn LayerSourcePlugin,
    ) -> Result<(), PluginError> {
        validate_node_desc(PluginKind::LayerSource, plugin.desc())?;
        let id = plugin.desc().id.clone();
        self.ensure_id_free(&id)?;
        insert_unique(&mut self.layer_sources, PluginKind::LayerSource, id, plugin)
    }

    pub fn register_filter(
        &mut self,
        plugin: &'static dyn FilterPlugin,
    ) -> Result<(), PluginError> {
        validate_node_desc(PluginKind::Filter, plugin.desc())?;
        let id = plugin.desc().id.clone();
        self.ensure_id_free(&id)?;
        insert_unique(&mut self.filters, PluginKind::Filter, id, plugin)
    }

    pub fn register_param_driver(
        &mut self,
        plugin: &'static dyn ParamDriverPlugin,
    ) -> Result<(), PluginError> {
        validate_node_desc(PluginKind::ParamDriver, plugin.desc())?;
        let id = plugin.desc().id.clone();
        self.ensure_id_free(&id)?;
        insert_unique(&mut self.param_drivers, PluginKind::ParamDriver, id, plugin)
    }

    pub fn register_composite(
        &mut self,
        plugin: &'static dyn CompositePlugin,
    ) -> Result<(), PluginError> {
        validate_node_desc(PluginKind::Composite, plugin.desc())?;
        let id = plugin.desc().id.clone();
        self.ensure_id_free(&id)?;
        insert_unique(&mut self.composites, PluginKind::Composite, id, plugin)
    }

    /// 種別をまたいでも PluginId は一意(ディスパッチの曖昧さを排除)。
    fn ensure_id_free(&self, id: &PluginId) -> Result<(), PluginError> {
        let kind = if self.layer_sources.contains_key(id) {
            Some(PluginKind::LayerSource)
        } else if self.filters.contains_key(id) {
            Some(PluginKind::Filter)
        } else if self.param_drivers.contains_key(id) {
            Some(PluginKind::ParamDriver)
        } else if self.composites.contains_key(id) {
            Some(PluginKind::Composite)
        } else {
            None
        };
        if let Some(kind) = kind {
            return Err(PluginError::Duplicate { kind, id: id.0 });
        }
        Ok(())
    }

    pub fn filter(&self, id: &PluginId) -> Option<&'static dyn FilterPlugin> {
        self.filters.get(id).copied()
    }

    pub fn param_driver(&self, id: &PluginId) -> Option<&'static dyn ParamDriverPlugin> {
        self.param_drivers.get(id).copied()
    }

    /// JSON等の動的なプラグインID文字列から参照する。
    pub fn param_driver_by_name(&self, name: &str) -> Option<&'static dyn ParamDriverPlugin> {
        by_name(&self.param_drivers, name)
    }

    pub fn filter_by_name(&self, name: &str) -> Option<&'static dyn FilterPlugin> {
        by_name(&self.filters, name)
    }

    pub fn composite_by_name(&self, name: &str) -> Option<&'static dyn CompositePlugin> {
        by_name(&self.composites, name)
    }

    pub fn layer_source_by_name(&self, name: &str) -> Option<&'static dyn LayerSourcePlugin> {
        by_name(&self.layer_sources, name)
    }

    pub fn composite(&self, id: &PluginId) -> Option<&'static dyn CompositePlugin> {
        self.composites.get(id).copied()
    }

    pub fn layer_source(&self, id: &PluginId) -> Option<&'static dyn LayerSourcePlugin> {
        self.layer_sources.get(id).copied()
    }

    pub fn len(&self, kind: PluginKind) -> usize {
        match kind {
            PluginKind::LayerSource => self.layer_sources.len(),
            PluginKind::Filter => self.filters.len(),
            PluginKind::ParamDriver => self.param_drivers.len(),
            PluginKind::Composite => self.composites.len(),
            PluginKind::Input | PluginKind::Simulation | PluginKind::ScriptWasm => 0,
        }
    }

    /// 登録済みプラグインを種別ごとに列挙する(M2E-9: 一括purityの前提)。
    pub fn iter(&self, kind: PluginKind) -> impl Iterator<Item = (&PluginId, DynPlugin)> + '_ {
        let items: Vec<(&PluginId, DynPlugin)> = match kind {
            PluginKind::LayerSource => self
                .layer_sources
                .iter()
                .map(|(id, p)| (id, DynPlugin::LayerSource(*p)))
                .collect(),
            PluginKind::Filter => self
                .filters
                .iter()
                .map(|(id, p)| (id, DynPlugin::Filter(*p)))
                .collect(),
            PluginKind::ParamDriver => self
                .param_drivers
                .iter()
                .map(|(id, p)| (id, DynPlugin::ParamDriver(*p)))
                .collect(),
            PluginKind::Composite => self
                .composites
                .iter()
                .map(|(id, p)| (id, DynPlugin::Composite(*p)))
                .collect(),
            PluginKind::Input | PluginKind::Simulation | PluginKind::ScriptWasm => Vec::new(),
        };
        items.into_iter()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PluginRuntimeError {
    #[error("executor `{id}` ({kind:?}) has no plugin contract")]
    ExecutorContractMissing { id: &'static str, kind: PluginKind },
    #[error(
        "executor `{id}` kind differs from contract: contract={contract:?}, executor={executor:?}"
    )]
    KindMismatch {
        id: &'static str,
        contract: PluginKind,
        executor: PluginKind,
    },
    #[error(
        "executor `{id}` version differs from contract: contract={contract}, executor={executor}"
    )]
    VersionMismatch {
        id: &'static str,
        contract: u32,
        executor: u32,
    },
    #[error("executor `{id}` NodeDesc differs from its contract")]
    DescriptorMismatch { id: &'static str },
}

/// Contractとexecutorの整合を構築時に固定した実行環境。
///
/// contractだけのentryは許すが、executorだけのentryは`try_new`で拒否する。
pub struct PluginRuntime {
    catalog: Arc<PluginCatalog>,
    executors: PluginRegistry,
}

impl std::fmt::Debug for PluginRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRuntime")
            .field("catalog_len", &self.catalog.len())
            .field("executors", &self.executors)
            .finish()
    }
}

impl PluginRuntime {
    pub fn try_new(
        catalog: Arc<PluginCatalog>,
        executors: PluginRegistry,
    ) -> Result<Self, PluginRuntimeError> {
        for kind in [
            PluginKind::LayerSource,
            PluginKind::Filter,
            PluginKind::ParamDriver,
            PluginKind::Composite,
        ] {
            for (id, executor) in executors.iter(kind) {
                let Some(contract) = catalog.get(id.0) else {
                    return Err(PluginRuntimeError::ExecutorContractMissing { id: id.0, kind });
                };
                if contract.kind != executor.kind() {
                    return Err(PluginRuntimeError::KindMismatch {
                        id: id.0,
                        contract: contract.kind,
                        executor: executor.kind(),
                    });
                }
                let desc = executor.desc();
                if contract.node.version != desc.version {
                    return Err(PluginRuntimeError::VersionMismatch {
                        id: id.0,
                        contract: contract.node.version,
                        executor: desc.version,
                    });
                }
                if contract.node != *desc {
                    return Err(PluginRuntimeError::DescriptorMismatch { id: id.0 });
                }
            }
        }
        Ok(Self { catalog, executors })
    }

    pub fn catalog(&self) -> &PluginCatalog {
        &self.catalog
    }

    pub fn executors(&self) -> &PluginRegistry {
        &self.executors
    }
}

/// `PluginRegistry::iter` が返す動的プラグイン参照。
#[derive(Clone, Copy)]
pub enum DynPlugin {
    LayerSource(&'static dyn LayerSourcePlugin),
    Filter(&'static dyn FilterPlugin),
    ParamDriver(&'static dyn ParamDriverPlugin),
    Composite(&'static dyn CompositePlugin),
}

impl DynPlugin {
    pub fn desc(&self) -> &NodeDesc {
        match self {
            DynPlugin::LayerSource(p) => p.desc(),
            DynPlugin::Filter(p) => p.desc(),
            DynPlugin::ParamDriver(p) => p.desc(),
            DynPlugin::Composite(p) => p.desc(),
        }
    }

    pub fn kind(&self) -> PluginKind {
        match self {
            DynPlugin::LayerSource(_) => PluginKind::LayerSource,
            DynPlugin::Filter(_) => PluginKind::Filter,
            DynPlugin::ParamDriver(_) => PluginKind::ParamDriver,
            DynPlugin::Composite(_) => PluginKind::Composite,
        }
    }
}

fn insert_unique<T: ?Sized>(
    map: &mut BTreeMap<PluginId, &'static T>,
    kind: PluginKind,
    id: PluginId,
    plugin: &'static T,
) -> Result<(), PluginError> {
    if map.contains_key(&id) {
        return Err(PluginError::Duplicate { kind, id: id.0 });
    }
    map.insert(id, plugin);
    Ok(())
}

fn by_name<T: ?Sized>(map: &BTreeMap<PluginId, &'static T>, name: &str) -> Option<&'static T> {
    map.iter()
        .find(|(id, _)| id.0 == name)
        .map(|(_, plugin)| *plugin)
}
