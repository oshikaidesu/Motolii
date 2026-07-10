//! motolii-plugin: 静的リンク版のプラグインホスト契約。
//!
//! v1はdylibロードを持たず、同一バイナリ内で種別レジストリに登録する。
//! Render系の境界は最初からGPUテクスチャのみで、CPUフレームを受け渡す経路は作らない。

use std::collections::{BTreeMap, HashMap};
use std::f64::consts::TAU;
use std::sync::OnceLock;

use motolii_core::{CompCamera, Fps, FrameDesc, RationalTime};
use motolii_eval::{DataTrack, Value};
use motolii_gpu::GpuCtx;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginId(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginKind {
    /// 予約: デコード/メディアソース。M1ではmotolii-media境界として扱う。
    Input,
    /// 入力なしでレイヤーのRGBAテクスチャを生成する。
    LayerSource,
    /// テクスチャ in/out のGPUエフェクト。
    Filter,
    /// 値・時系列データを生成し、ParamSource/DataTrack側を駆動する。
    ParamDriver,
    /// 複数テクスチャ入力を合成して1テクスチャへ書く。
    Composite,
    /// 予約: v2以降の型付き式/WASM。
    ScriptWasm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    F64,
    Vec2,
    Vec3,
    Color,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamDef {
    pub id: &'static str,
    pub value_type: ValueType,
    pub default: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeDesc {
    pub id: PluginId,
    /// パラメータスキーマの互換バージョン。破壊的変更で上げる(F-9)。
    pub version: u32,
    pub display_name: &'static str,
    /// UIブラウザ用カテゴリ(F-8)。例: "Color" / "Generate" / "Composite"。
    pub category: &'static str,
    /// 検索・発見用タグ(F-8)。将来サムネイル口とは別。
    pub tags: &'static [&'static str],
    pub params: Vec<ParamDef>,
    pub min_inputs: usize,
    pub max_inputs: usize,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ResolvedParams {
    values: HashMap<&'static str, Value>,
}

impl ResolvedParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, id: &'static str, value: Value) {
        self.values.insert(id, value);
    }

    pub fn get(&self, id: &'static str) -> Option<&Value> {
        self.values.get(id)
    }

    pub fn f64_or(&self, id: &'static str, fallback: f64) -> f64 {
        self.get(id).and_then(Value::as_f64).unwrap_or(fallback)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureRef<'a> {
    pub texture: &'a wgpu::Texture,
    pub desc: FrameDesc,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamDriverContext {
    pub start: RationalTime,
    pub duration: RationalTime,
    pub sample_rate: Fps,
}

#[derive(Debug, Clone, Copy)]
pub struct LayerSourceContext {
    /// v1ではコンポ全体で共有される単一カメラ。
    pub camera: CompCamera,
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("duplicate {kind:?} plugin id: {id}")]
    Duplicate { kind: PluginKind, id: &'static str },
    #[error("plugin render failed: {0}")]
    Render(String),
}

pub trait FilterPlugin: Send + Sync {
    fn desc(&self) -> &NodeDesc;

    fn render(
        &self,
        gpu: &GpuCtx,
        encoder: &mut wgpu::CommandEncoder,
        t: RationalTime,
        params: &ResolvedParams,
        input: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), PluginError>;
}

pub trait LayerSourcePlugin: Send + Sync {
    fn desc(&self) -> &NodeDesc;

    fn render(
        &self,
        gpu: &GpuCtx,
        encoder: &mut wgpu::CommandEncoder,
        t: RationalTime,
        params: &ResolvedParams,
        ctx: LayerSourceContext,
        output: TextureRef<'_>,
    ) -> Result<(), PluginError>;
}

pub trait ParamDriverPlugin: Send + Sync {
    fn desc(&self) -> &NodeDesc;

    fn build_track(
        &self,
        ctx: ParamDriverContext,
        params: &ResolvedParams,
    ) -> Result<DataTrack, PluginError>;
}

pub trait CompositePlugin: Send + Sync {
    fn desc(&self) -> &NodeDesc;

    fn render(
        &self,
        gpu: &GpuCtx,
        encoder: &mut wgpu::CommandEncoder,
        t: RationalTime,
        params: &ResolvedParams,
        inputs: &[TextureRef<'_>],
        output: TextureRef<'_>,
    ) -> Result<(), PluginError>;
}

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
        insert_unique(
            &mut self.layer_sources,
            PluginKind::LayerSource,
            plugin.desc().id.clone(),
            plugin,
        )
    }

    pub fn register_filter(
        &mut self,
        plugin: &'static dyn FilterPlugin,
    ) -> Result<(), PluginError> {
        insert_unique(
            &mut self.filters,
            PluginKind::Filter,
            plugin.desc().id.clone(),
            plugin,
        )
    }

    pub fn register_param_driver(
        &mut self,
        plugin: &'static dyn ParamDriverPlugin,
    ) -> Result<(), PluginError> {
        insert_unique(
            &mut self.param_drivers,
            PluginKind::ParamDriver,
            plugin.desc().id.clone(),
            plugin,
        )
    }

    pub fn register_composite(
        &mut self,
        plugin: &'static dyn CompositePlugin,
    ) -> Result<(), PluginError> {
        insert_unique(
            &mut self.composites,
            PluginKind::Composite,
            plugin.desc().id.clone(),
            plugin,
        )
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
            PluginKind::Input | PluginKind::ScriptWasm => 0,
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

fn by_name<T: ?Sized>(
    map: &BTreeMap<PluginId, &'static T>,
    name: &str,
) -> Option<&'static T> {
    map.iter()
        .find(|(id, _)| id.0 == name)
        .map(|(_, plugin)| *plugin)
}

/// M1-T12用の最小参照プラグイン群。
///
/// Filter/CompositeはGPU render passだけを発行する。CPUピクセル処理の迂回路は持たない。
pub mod reference {
    use super::*;

    pub static CLEAR_FILTER: ClearFilter = ClearFilter;
    pub static CLEAR_LAYER_SOURCE: ClearLayerSource = ClearLayerSource;
    pub static SINE_PARAM_DRIVER: SineParamDriver = SineParamDriver;
    pub static CLEAR_COMPOSITE: ClearComposite = ClearComposite;

    pub fn register_reference_plugins(registry: &mut PluginRegistry) -> Result<(), PluginError> {
        registry.register_layer_source(&CLEAR_LAYER_SOURCE)?;
        registry.register_filter(&CLEAR_FILTER)?;
        registry.register_param_driver(&SINE_PARAM_DRIVER)?;
        registry.register_composite(&CLEAR_COMPOSITE)?;
        Ok(())
    }

    pub struct ClearFilter;

    impl FilterPlugin for ClearFilter {
        fn desc(&self) -> &NodeDesc {
            clear_filter_desc()
        }

        fn render(
            &self,
            _gpu: &GpuCtx,
            encoder: &mut wgpu::CommandEncoder,
            _t: RationalTime,
            params: &ResolvedParams,
            _input: TextureRef<'_>,
            output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            clear_texture(encoder, output, color_from_params(params));
            Ok(())
        }
    }

    pub struct ClearLayerSource;

    impl LayerSourcePlugin for ClearLayerSource {
        fn desc(&self) -> &NodeDesc {
            clear_layer_source_desc()
        }

        fn render(
            &self,
            _gpu: &GpuCtx,
            encoder: &mut wgpu::CommandEncoder,
            _t: RationalTime,
            params: &ResolvedParams,
            ctx: LayerSourceContext,
            output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            ctx.camera.validate().map_err(PluginError::Render)?;
            clear_texture(encoder, output, color_from_params(params));
            Ok(())
        }
    }

    pub struct ClearComposite;

    impl CompositePlugin for ClearComposite {
        fn desc(&self) -> &NodeDesc {
            clear_composite_desc()
        }

        fn render(
            &self,
            _gpu: &GpuCtx,
            encoder: &mut wgpu::CommandEncoder,
            _t: RationalTime,
            params: &ResolvedParams,
            _inputs: &[TextureRef<'_>],
            output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            clear_texture(encoder, output, color_from_params(params));
            Ok(())
        }
    }

    pub struct SineParamDriver;

    impl ParamDriverPlugin for SineParamDriver {
        fn desc(&self) -> &NodeDesc {
            sine_param_desc()
        }

        fn build_track(
            &self,
            ctx: ParamDriverContext,
            params: &ResolvedParams,
        ) -> Result<DataTrack, PluginError> {
            let amplitude = params.f64_or("amplitude", 1.0);
            let frequency_hz = params.f64_or("frequency_hz", 1.0);
            let offset = params.f64_or("offset", 0.0);
            let samples = sample_count(ctx.duration, ctx.sample_rate);
            let values = (0..samples)
                .map(|i| {
                    let secs = i as f64 / ctx.sample_rate.as_f64();
                    Value::F64(offset + amplitude * (TAU * frequency_hz * secs).sin())
                })
                .collect();
            Ok(DataTrack {
                start: ctx.start,
                sample_rate: ctx.sample_rate,
                values,
            })
        }
    }

    fn clear_filter_desc() -> &'static NodeDesc {
        static DESC: OnceLock<NodeDesc> = OnceLock::new();
        DESC.get_or_init(|| NodeDesc {
            id: PluginId("core.filter.clear"),
            version: 1,
            display_name: "Clear",
            category: "Utility",
            tags: &["clear", "fill", "reference"],
            params: color_params(),
            min_inputs: 1,
            max_inputs: 1,
        })
    }

    fn clear_layer_source_desc() -> &'static NodeDesc {
        static DESC: OnceLock<NodeDesc> = OnceLock::new();
        DESC.get_or_init(|| NodeDesc {
            id: PluginId("core.layer_source.clear"),
            version: 1,
            display_name: "Clear Layer Source",
            category: "Generate",
            tags: &["clear", "fill", "reference"],
            params: color_params(),
            min_inputs: 0,
            max_inputs: 0,
        })
    }

    fn clear_composite_desc() -> &'static NodeDesc {
        static DESC: OnceLock<NodeDesc> = OnceLock::new();
        DESC.get_or_init(|| NodeDesc {
            id: PluginId("core.composite.clear"),
            version: 1,
            display_name: "Clear Composite",
            category: "Composite",
            tags: &["clear", "fill", "reference"],
            params: color_params(),
            min_inputs: 2,
            max_inputs: usize::MAX,
        })
    }

    fn sine_param_desc() -> &'static NodeDesc {
        static DESC: OnceLock<NodeDesc> = OnceLock::new();
        DESC.get_or_init(|| NodeDesc {
            id: PluginId("core.param.sine"),
            version: 1,
            display_name: "Sine",
            category: "Generate",
            tags: &["lfo", "oscillator", "reference"],
            params: vec![
                ParamDef {
                    id: "amplitude",
                    value_type: ValueType::F64,
                    default: Value::F64(1.0),
                },
                ParamDef {
                    id: "frequency_hz",
                    value_type: ValueType::F64,
                    default: Value::F64(1.0),
                },
                ParamDef {
                    id: "offset",
                    value_type: ValueType::F64,
                    default: Value::F64(0.0),
                },
            ],
            min_inputs: 0,
            max_inputs: 0,
        })
    }

    fn color_params() -> Vec<ParamDef> {
        vec![ParamDef {
            id: "color",
            value_type: ValueType::Color,
            default: Value::Color([0.0, 0.0, 0.0, 0.0]),
        }]
    }

    fn color_from_params(params: &ResolvedParams) -> wgpu::Color {
        match params.get("color") {
            Some(Value::Color([r, g, b, a])) => wgpu::Color {
                r: *r,
                g: *g,
                b: *b,
                a: *a,
            },
            _ => wgpu::Color::TRANSPARENT,
        }
    }

    fn clear_texture(
        encoder: &mut wgpu::CommandEncoder,
        output: TextureRef<'_>,
        color: wgpu::Color,
    ) {
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("motolii-plugin-clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            multiview_mask: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }

    fn sample_count(duration: RationalTime, sample_rate: Fps) -> usize {
        let seconds = duration.as_seconds_f64().max(0.0);
        (seconds * sample_rate.as_f64()).floor() as usize + 1
    }
}

#[cfg(test)]
mod tests {
    use super::reference::{register_reference_plugins, CLEAR_LAYER_SOURCE, SINE_PARAM_DRIVER};
    use super::*;

    #[test]
    fn registry_keeps_plugin_kinds_separate() {
        let mut registry = PluginRegistry::new();
        register_reference_plugins(&mut registry).unwrap();

        assert_eq!(registry.len(PluginKind::LayerSource), 1);
        assert_eq!(registry.len(PluginKind::Filter), 1);
        assert_eq!(registry.len(PluginKind::ParamDriver), 1);
        assert_eq!(registry.len(PluginKind::Composite), 1);
        assert!(registry
            .layer_source(&PluginId("core.layer_source.clear"))
            .is_some());
        assert!(registry.filter(&PluginId("core.filter.clear")).is_some());
        assert!(registry
            .param_driver(&PluginId("core.param.sine"))
            .is_some());
        assert!(registry
            .composite(&PluginId("core.composite.clear"))
            .is_some());
        assert!(registry.filter_by_name("core.filter.clear").is_some());
        assert!(registry
            .composite_by_name("core.composite.clear")
            .is_some());
        assert!(registry
            .layer_source_by_name("core.layer_source.clear")
            .is_some());
        assert!(registry.param_driver_by_name("core.param.sine").is_some());
        assert!(registry.filter_by_name("missing").is_none());
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
    fn registry_rejects_duplicate_within_kind() {
        let mut registry = PluginRegistry::new();
        registry.register_param_driver(&SINE_PARAM_DRIVER).unwrap();
        let err = registry
            .register_param_driver(&SINE_PARAM_DRIVER)
            .unwrap_err();
        assert!(matches!(
            err,
            PluginError::Duplicate {
                kind: PluginKind::ParamDriver,
                id: "core.param.sine"
            }
        ));
    }

    #[test]
    fn sine_param_driver_builds_typed_data_track() {
        let mut params = ResolvedParams::new();
        params.insert("amplitude", Value::F64(2.0));
        params.insert("frequency_hz", Value::F64(1.0));
        params.insert("offset", Value::F64(10.0));

        let track = SINE_PARAM_DRIVER
            .build_track(
                ParamDriverContext {
                    start: RationalTime::ZERO,
                    duration: RationalTime::from_seconds(1),
                    sample_rate: Fps::new(4, 1),
                },
                &params,
            )
            .unwrap();

        assert_eq!(track.values.len(), 5);
        assert_eq!(track.values[0], Value::F64(10.0));
        assert!((track.values[1].as_f64().unwrap() - 12.0).abs() < 1e-9);
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

        let driver = registry
            .param_driver(&PluginId("core.param.sine"))
            .unwrap()
            .desc();
        assert_eq!(driver.category, "Generate");
        assert!(driver.tags.contains(&"lfo"));
    }
}
