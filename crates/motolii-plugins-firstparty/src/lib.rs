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
const RESERVED_BUILTIN_IDS: &[&str] = &["doc.layer_source.rect"];

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
    #[error("plugin id `{id}` is reserved for a document built-in")]
    ReservedBuiltinId { id: &'static str },
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
    ensure_no_reserved_builtin_ids(&catalog, &registry)?;
    let runtime = PluginRuntime::try_new(catalog, registry)?;
    ensure_required_host_capabilities(runtime.catalog(), runtime.executors())?;
    Ok(runtime)
}

fn ensure_no_reserved_builtin_ids(
    catalog: &PluginCatalog,
    registry: &PluginRegistry,
) -> Result<(), FirstPartyError> {
    for &id in RESERVED_BUILTIN_IDS {
        if catalog.get(id).is_some() || executor_has_id(registry, id) {
            return Err(FirstPartyError::ReservedBuiltinId { id });
        }
    }
    Ok(())
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
    use std::sync::OnceLock;

    use motolii_plugin::reference::reference_catalog;
    use motolii_plugin::{
        wgpu, GpuCtx, LayerSourceContext, LayerSourcePlugin, NodeDesc, PipelineCache,
        PluginContract, PluginId, PluginKind, RationalTime, ResolvedParams, TextureRef,
    };

    use super::*;

    fn rect_layer_source_contract() -> PluginContract {
        PluginContract {
            kind: PluginKind::LayerSource,
            node: NodeDesc {
                id: PluginId("doc.layer_source.rect"),
                version: 1,
                display_name: "Rect",
                category: "Generate",
                tags: &["rect", "test"],
                params: vec![],
                min_inputs: 0,
                max_inputs: 0,
            },
            migrations: vec![],
        }
    }

    fn rect_layer_source_desc() -> &'static NodeDesc {
        static DESC: OnceLock<NodeDesc> = OnceLock::new();
        DESC.get_or_init(|| rect_layer_source_contract().node)
    }

    struct RectLayerSourceStub;

    impl LayerSourcePlugin for RectLayerSourceStub {
        fn desc(&self) -> &NodeDesc {
            rect_layer_source_desc()
        }

        fn render(
            &self,
            _gpu: &GpuCtx,
            _pipelines: &mut PipelineCache,
            _encoder: &mut wgpu::CommandEncoder,
            _t: RationalTime,
            _params: &ResolvedParams,
            _ctx: LayerSourceContext,
            _output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            Ok(())
        }
    }

    static RECT_LAYER_SOURCE_STUB: RectLayerSourceStub = RectLayerSourceStub;

    #[test]
    fn ensure_no_reserved_builtin_ids_accepts_first_party_assembly() {
        let catalog = first_party_catalog().unwrap();
        let registry = first_party_registry().unwrap();

        ensure_no_reserved_builtin_ids(&catalog, &registry).unwrap();
    }

    #[test]
    fn n6_rect_catalog_only_registration_rejected() {
        let mut builder = PluginCatalogBuilder::new();
        builder.register(rect_layer_source_contract()).unwrap();
        let catalog = builder.build().unwrap();
        let registry = PluginRegistry::new();

        let err = ensure_no_reserved_builtin_ids(&catalog, &registry).unwrap_err();
        assert!(matches!(
            err,
            FirstPartyError::ReservedBuiltinId {
                id: "doc.layer_source.rect"
            }
        ));
    }

    #[test]
    fn n6_rect_registry_only_registration_rejected() {
        let catalog = reference_catalog().unwrap();
        let mut registry = PluginRegistry::new();
        registry
            .register_layer_source(&RECT_LAYER_SOURCE_STUB)
            .unwrap();

        let err = ensure_no_reserved_builtin_ids(&catalog, &registry).unwrap_err();
        assert!(matches!(
            err,
            FirstPartyError::ReservedBuiltinId {
                id: "doc.layer_source.rect"
            }
        ));
    }

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
