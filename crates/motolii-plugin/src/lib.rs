//! motolii-plugin: 静的リンク版のプラグインホスト契約。
//!
//! v1はdylibロードを持たず、同一バイナリ内で種別レジストリに登録する。
//! Render系の境界は最初からGPUテクスチャのみで、CPUフレームを受け渡す経路は作らない。
//!
//! 責任は各moduleが持ち、ここは公開pathの組み立てだけを行う。

// A1S §2.1: 外部plugin crateが別versionのwgpu/bound型分裂を避け、単一依存でtrait実装できる公開面。
pub use bytemuck;
pub use motolii_core::{CompCamera, Fps, FrameDesc, Quality, RationalTime};
pub use motolii_eval::{DataTrack, Value};
pub use motolii_gpu::{GpuCtx, PipelineCache, PipelineCacheKey};
pub use wgpu;

mod context;
mod contract;
mod params;
mod registry;
mod traits;

pub use context::{
    CompLookbehind, InstanceIndex, LayerSourceContext, ParamDriverContext, RenderCtx,
    TemporalFootprint, TextureRef,
};
pub use contract::{
    validate_node_desc, value_matches_type, value_type_name, DomainError, ElementType, F64Domain,
    MigrationOp,
    MigrationPlanError, MigrationStep, NodeDesc, ParamDef, PluginCatalog, PluginCatalogBuilder,
    PluginContract, PluginContractError, PluginError, PluginId, PluginKind, ValueType,
};
pub use params::ResolvedParams;
pub use registry::{DynPlugin, PluginRegistry, PluginRuntime, PluginRuntimeError};
pub use traits::{CompositePlugin, FilterPlugin, LayerSourcePlugin, ParamDriverPlugin};

pub mod reference;

// 公開APIのパニック禁止(INF-7b)は本番コードにlintを効かせ、テストmodだけ免除する。
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::{Arc, OnceLock};

    use super::reference::{register_reference_plugins, CLEAR_LAYER_SOURCE};
    use super::*;

    #[test]
    fn registry_keeps_plugin_kinds_separate() {
        let mut registry = PluginRegistry::new();
        register_reference_plugins(&mut registry).unwrap();

        assert_eq!(registry.len(PluginKind::LayerSource), 1);
        assert_eq!(registry.len(PluginKind::Filter), 2);
        assert_eq!(registry.len(PluginKind::ParamDriver), 0);
        assert_eq!(registry.len(PluginKind::Composite), 1);
        assert!(registry
            .layer_source(&PluginId("core.layer_source.clear"))
            .is_some());
        assert!(registry.filter(&PluginId("core.filter.clear")).is_some());
        assert!(registry
            .composite(&PluginId("core.composite.clear"))
            .is_some());
        assert!(registry.filter_by_name("core.filter.clear").is_some());
        assert!(registry.composite_by_name("core.composite.clear").is_some());
        assert!(registry
            .layer_source_by_name("core.layer_source.clear")
            .is_some());
        assert!(registry.param_driver_by_name("core.param.sine").is_none());
        assert!(registry.filter_by_name("missing").is_none());

        assert_eq!(registry.iter(PluginKind::Filter).count(), 2);
        assert_eq!(registry.iter(PluginKind::ParamDriver).count(), 0);
        assert_eq!(registry.iter(PluginKind::LayerSource).count(), 1);
        assert_eq!(registry.iter(PluginKind::Composite).count(), 1);
        assert_eq!(registry.iter(PluginKind::Input).count(), 0);
        let filter_ids: Vec<&str> = registry
            .iter(PluginKind::Filter)
            .map(|(id, _)| id.0)
            .collect();
        assert!(filter_ids.contains(&"core.filter.clear"));
        assert!(filter_ids.contains(&"core.filter.tint"));
    }

    #[test]
    fn runtime_rejects_kind_mismatch_even_if_catalog_was_not_built_normally() {
        let node = super::reference::CLEAR_FILTER.desc().clone();
        let mut contracts = BTreeMap::new();
        contracts.insert(
            node.id.clone(),
            PluginContract {
                kind: PluginKind::LayerSource,
                node,
                migrations: vec![],
            },
        );
        let catalog = Arc::new(PluginCatalog { contracts });
        let mut executors = PluginRegistry::new();
        executors
            .register_filter(&super::reference::CLEAR_FILTER)
            .unwrap();
        let err = PluginRuntime::try_new(catalog, executors).unwrap_err();
        assert!(matches!(
            err,
            PluginRuntimeError::KindMismatch {
                id: "core.filter.clear",
                contract: PluginKind::LayerSource,
                executor: PluginKind::Filter,
            }
        ));
    }

    #[test]
    fn registry_rejects_duplicate_layer_source_within_kind() {
        let mut registry = PluginRegistry::new();
        registry.register_layer_source(&CLEAR_LAYER_SOURCE).unwrap();
        let err = registry
            .register_layer_source(&CLEAR_LAYER_SOURCE)
            .unwrap_err();
        assert!(matches!(
            err,
            PluginError::Duplicate {
                kind: PluginKind::LayerSource,
                id: "core.layer_source.clear"
            }
        ));
    }

    #[test]
    fn registry_rejects_duplicate_across_kinds() {
        let mut registry = PluginRegistry::new();
        registry
            .register_filter(&super::reference::CLEAR_FILTER)
            .unwrap();

        // 同一PluginId文字列を別種別に流用すると、kindセグメント検証が先に弾く
        // (vendor.kind.name 規約下では ensure_id_free の前に InvalidDesc になる)。
        struct ClashComposite;
        impl CompositePlugin for ClashComposite {
            fn desc(&self) -> &NodeDesc {
                static DESC: OnceLock<NodeDesc> = OnceLock::new();
                DESC.get_or_init(|| NodeDesc {
                    id: PluginId("core.filter.clear"),
                    version: 1,
                    display_name: "Clash",
                    category: "Composite",
                    tags: &["test"],
                    params: vec![],
                    min_inputs: 2,
                    max_inputs: 2,
                })
            }

            fn render(
                &self,
                _gpu: &GpuCtx,
                _pipelines: &mut PipelineCache,
                _encoder: &mut wgpu::CommandEncoder,
                _ctx: &RenderCtx,
                _params: &ResolvedParams,
                _inputs: &[TextureRef<'_>],
                _output: TextureRef<'_>,
            ) -> Result<(), PluginError> {
                Ok(())
            }
        }

        static CLASH: ClashComposite = ClashComposite;
        let err = registry.register_composite(&CLASH).unwrap_err();
        assert!(
            matches!(err, PluginError::InvalidDesc { .. }),
            "expected InvalidDesc for kind/id mismatch, got {err:?}"
        );
    }

    #[test]
    fn registry_rejects_duplicate_within_kind() {
        let mut registry = PluginRegistry::new();
        registry
            .register_filter(&super::reference::CLEAR_FILTER)
            .unwrap();
        let err = registry
            .register_filter(&super::reference::CLEAR_FILTER)
            .unwrap_err();
        assert!(matches!(
            err,
            PluginError::Duplicate {
                kind: PluginKind::Filter,
                id: "core.filter.clear"
            }
        ));
    }

    #[test]
    fn reference_plugins_expose_discovery_metadata() {
        let mut registry = PluginRegistry::new();
        register_reference_plugins(&mut registry).unwrap();

        let filter = registry
            .filter(&PluginId("core.filter.clear"))
            .unwrap()
            .desc();
        assert_eq!(filter.version, 1);
        assert_eq!(filter.category, "Utility");
        assert!(filter.tags.contains(&"reference"));
        assert!(!filter.display_name.is_empty());
    }

    #[test]
    fn reserved_lookbehind_and_instance_index_serde() {
        let idx = InstanceIndex { index: 2, count: 8 };
        let look = CompLookbehind {
            target: "root".into(),
            offsets: vec![-1, -2],
            exclude: vec!["core.filter.echo".into()],
        };
        let idx_json = serde_json::to_string(&idx).unwrap();
        let look_json = serde_json::to_string(&look).unwrap();
        assert_eq!(
            serde_json::from_str::<InstanceIndex>(&idx_json).unwrap(),
            idx
        );
        assert_eq!(
            serde_json::from_str::<CompLookbehind>(&look_json).unwrap(),
            look
        );
    }

    #[test]
    fn render_ctx_carries_quality_and_reserved_defaults() {
        use motolii_core::Quality;
        let ctx = RenderCtx::new(RationalTime::from_seconds(1), Quality::DRAFT);
        assert_eq!(ctx.t, RationalTime::from_seconds(1));
        assert_eq!(ctx.quality, Quality::DRAFT);
        assert!(ctx.instance.is_none());
        assert!(ctx.lookbehind.is_none());
        assert_eq!(ctx.temporal_footprint, TemporalFootprint::default());
        let footprint = TemporalFootprint {
            frames_before: 1,
            frames_after: 2,
            subframe_samples: 4,
        };
        let json = serde_json::to_string(&footprint).unwrap();
        assert_eq!(
            serde_json::from_str::<TemporalFootprint>(&json).unwrap(),
            footprint
        );
    }

    /// INF-7c: 参照プラグイン全desc + 検証の負例(欠落メタデータが赤になる証明)。
    #[test]
    fn validate_node_desc_accepts_reference_plugins() {
        use super::reference::{CLEAR_COMPOSITE, CLEAR_FILTER, CLEAR_LAYER_SOURCE, TINT_FILTER};
        validate_node_desc(PluginKind::Filter, CLEAR_FILTER.desc()).unwrap();
        validate_node_desc(PluginKind::Filter, TINT_FILTER.desc()).unwrap();
        validate_node_desc(PluginKind::LayerSource, CLEAR_LAYER_SOURCE.desc()).unwrap();
        validate_node_desc(PluginKind::Composite, CLEAR_COMPOSITE.desc()).unwrap();
    }

    #[test]
    fn validate_node_desc_rejects_incomplete_metadata() {
        let valid = NodeDesc {
            id: PluginId("core.filter.probe"),
            version: 1,
            display_name: "Probe",
            category: "Utility",
            tags: &["test"],
            params: vec![],
            min_inputs: 1,
            max_inputs: 1,
        };
        validate_node_desc(PluginKind::Filter, &valid).unwrap();

        let cases: &[(&str, NodeDesc)] = &[
            (
                "empty display_name",
                NodeDesc {
                    display_name: "  ",
                    ..valid.clone()
                },
            ),
            (
                "empty category",
                NodeDesc {
                    category: "",
                    ..valid.clone()
                },
            ),
            (
                "empty tags",
                NodeDesc {
                    tags: &[],
                    ..valid.clone()
                },
            ),
            (
                "version 0",
                NodeDesc {
                    version: 0,
                    ..valid.clone()
                },
            ),
            (
                "bad id",
                NodeDesc {
                    id: PluginId("Not.Valid.ID"),
                    ..valid.clone()
                },
            ),
            (
                "arity",
                NodeDesc {
                    min_inputs: 0,
                    max_inputs: 0,
                    ..valid.clone()
                },
            ),
            (
                "kind segment mismatch",
                NodeDesc {
                    id: PluginId("core.param.evil"),
                    ..valid.clone()
                },
            ),
        ];
        for (label, desc) in cases {
            let err = validate_node_desc(PluginKind::Filter, desc).unwrap_err();
            assert!(
                matches!(err, PluginError::InvalidDesc { .. }),
                "{label}: {err:?}"
            );
        }
    }

    #[test]
    fn registry_rejects_invalid_desc_at_registration() {
        struct BadFilter;
        impl FilterPlugin for BadFilter {
            fn desc(&self) -> &NodeDesc {
                static DESC: OnceLock<NodeDesc> = OnceLock::new();
                DESC.get_or_init(|| NodeDesc {
                    id: PluginId("core.filter.bad"),
                    version: 1,
                    display_name: "Bad",
                    category: "Utility",
                    tags: &[],
                    params: vec![],
                    min_inputs: 1,
                    max_inputs: 1,
                })
            }

            fn render(
                &self,
                _gpu: &GpuCtx,
                _pipelines: &mut PipelineCache,
                _encoder: &mut wgpu::CommandEncoder,
                _ctx: &RenderCtx,
                _params: &ResolvedParams,
                _input: TextureRef<'_>,
                _output: TextureRef<'_>,
            ) -> Result<(), PluginError> {
                Ok(())
            }
        }
        static BAD: BadFilter = BadFilter;
        let mut registry = PluginRegistry::new();
        let err = registry.register_filter(&BAD).unwrap_err();
        assert!(matches!(err, PluginError::InvalidDesc { .. }));
    }
}

/// M2E-10: `new-plugin` 生成物を自己クレート配置でコンパイル検証する口。
/// 実体は OUT_DIR(build.rs)。`MOTOLII_SCAFFOLD_FIXTURE` 未設定時は空モジュール。
/// ソースに欠落 `#[path]` を置かない(rustfmt が cfg を無視するため)。
pub mod scaffold_fixture {
    include!(concat!(env!("OUT_DIR"), "/scaffold_fixture_mods.rs"));
}
