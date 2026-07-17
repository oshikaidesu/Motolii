//! motolii-plugin: 静的リンク版のプラグインホスト契約。
//!
//! v1はdylibロードを持たず、同一バイナリ内で種別レジストリに登録する。
//! Render系の境界は最初からGPUテクスチャのみで、CPUフレームを受け渡す経路は作らない。

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::f64::consts::TAU;
use std::sync::{Arc, OnceLock};

pub use bytemuck;
pub use motolii_core::{CompCamera, Fps, FrameDesc, Quality, RationalTime};
pub use motolii_eval::{DataTrack, Value};
pub use motolii_gpu::{GpuCtx, PipelineCache, PipelineCacheKey};
pub use wgpu;

use motolii_core::RationalTimeError;
use serde::{Deserialize, Serialize};

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
    /// 予約: 逐次状態シミュレーション(布・液体・パーティクル)。
    /// 状態はホストが所有しStateTrackへベイクする。設計はdocs/simulation-model.md、実装はv1.x。
    Simulation,
    /// 予約: v2以降の型付き式/WASM。
    ScriptWasm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    F64,
    Vec2,
    Vec3,
    Color,
    /// アセットID参照(F-10予約。実装結線はM2 D1)。
    AssetRef,
}

impl ValueType {
    pub fn as_str(self) -> &'static str {
        match self {
            ValueType::F64 => "F64",
            ValueType::Vec2 => "Vec2",
            ValueType::Vec3 => "Vec3",
            ValueType::Color => "Color",
            ValueType::AssetRef => "AssetRef",
        }
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// `Value` の実行時型名(エラー表示用)。
pub fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::F64(_) => "F64",
        Value::Vec2(_) => "Vec2",
        Value::Vec3(_) => "Vec3",
        Value::Color(_) => "Color",
        Value::AssetRef(_) => "AssetRef",
    }
}

pub fn value_matches_type(value_type: ValueType, value: &Value) -> bool {
    matches!(
        (value_type, value),
        (ValueType::F64, Value::F64(_))
            | (ValueType::Vec2, Value::Vec2(_))
            | (ValueType::Vec3, Value::Vec3(_))
            | (ValueType::Color, Value::Color(_))
            | (ValueType::AssetRef, Value::AssetRef(_))
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamDef {
    pub id: &'static str,
    pub value_type: ValueType,
    pub default: Value,
    /// 値そのものの意味域。UI slider範囲ではない。
    /// `ValueType::F64`以外では必ず`None`。
    pub f64_domain: Option<F64Domain>,
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

/// F64 parameterの意味域。境界は両端包含。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct F64Domain {
    pub min_inclusive: Option<f64>,
    pub max_inclusive: Option<f64>,
    pub integer: bool,
}

impl F64Domain {
    pub const fn new(
        min_inclusive: Option<f64>,
        max_inclusive: Option<f64>,
        integer: bool,
    ) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
            integer,
        }
    }

    pub const fn unit() -> Self {
        Self::new(Some(0.0), Some(1.0), false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationOp {
    RenameParam {
        from: &'static str,
        to: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationStep {
    pub from_version: u32,
    pub to_version: u32,
    pub ops: Vec<MigrationOp>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PluginContract {
    pub kind: PluginKind,
    pub node: NodeDesc,
    pub migrations: Vec<MigrationStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainError {
    NonFiniteBound,
    ReversedBounds,
    NonF64Parameter,
    DefaultOutsideDomain,
    DefaultTypeMismatch,
    NonFiniteDefault,
    ColorDefaultOutsideUnitInterval,
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::NonFiniteBound => "domain bound must be finite",
            Self::ReversedBounds => "domain minimum exceeds maximum",
            Self::NonF64Parameter => "f64 domain is only valid for F64 parameters",
            Self::DefaultOutsideDomain => "default is outside the declared domain",
            Self::DefaultTypeMismatch => "default type does not match ValueType",
            Self::NonFiniteDefault => "default contains a non-finite number",
            Self::ColorDefaultOutsideUnitInterval => {
                "Color default components must be in the inclusive range 0..=1"
            }
        };
        f.write_str(message)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationPlanError {
    ZeroVersion,
    NonAdjacentVersions,
    DuplicateFromVersion,
    BeyondCurrentVersion,
    EmptyParamName,
    SameParamName,
    DuplicateRenameSource,
    DuplicateRenameDestination,
}

impl std::fmt::Display for MigrationPlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::ZeroVersion => "migration versions start at 1",
            Self::NonAdjacentVersions => "migration step must be N to N+1",
            Self::DuplicateFromVersion => "migration from_version is duplicated",
            Self::BeyondCurrentVersion => "migration target exceeds current contract version",
            Self::EmptyParamName => "migration parameter name is empty",
            Self::SameParamName => "migration source and destination are identical",
            Self::DuplicateRenameSource => "migration source is used more than once",
            Self::DuplicateRenameDestination => "migration destination is used more than once",
        };
        f.write_str(message)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PluginContractError {
    #[error("duplicate plugin contract id: {id}")]
    DuplicateContract { id: &'static str },
    #[error("plugin `{plugin}` has duplicate parameter `{param}`")]
    DuplicateParam {
        plugin: &'static str,
        param: &'static str,
    },
    #[error("plugin `{plugin}` parameter `{param}` has invalid domain: {reason}")]
    InvalidDomain {
        plugin: &'static str,
        param: &'static str,
        reason: DomainError,
    },
    #[error("plugin `{plugin}` migration {from_version}->{to_version} is invalid: {reason}")]
    InvalidMigration {
        plugin: &'static str,
        from_version: u32,
        to_version: u32,
        reason: MigrationPlanError,
    },
    #[error(transparent)]
    InvalidNodeDesc(#[from] PluginError),
}

#[derive(Debug, Default)]
pub struct PluginCatalogBuilder {
    contracts: BTreeMap<PluginId, PluginContract>,
}

impl PluginCatalogBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, contract: PluginContract) -> Result<(), PluginContractError> {
        validate_plugin_contract(&contract)?;
        let id = contract.node.id.clone();
        if self.contracts.contains_key(&id) {
            return Err(PluginContractError::DuplicateContract { id: id.0 });
        }
        self.contracts.insert(id, contract);
        Ok(())
    }

    pub fn build(self) -> Result<PluginCatalog, PluginContractError> {
        Ok(PluginCatalog {
            contracts: self.contracts,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PluginCatalog {
    contracts: BTreeMap<PluginId, PluginContract>,
}

impl PluginCatalog {
    pub fn get(&self, id: &str) -> Option<&PluginContract> {
        self.contracts
            .iter()
            .find(|(plugin_id, _)| plugin_id.0 == id)
            .map(|(_, contract)| contract)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PluginId, &PluginContract)> {
        self.contracts.iter()
    }

    pub fn len(&self) -> usize {
        self.contracts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.contracts.is_empty()
    }
}

fn validate_plugin_contract(contract: &PluginContract) -> Result<(), PluginContractError> {
    let plugin = contract.node.id.0;
    let mut param_ids = BTreeSet::new();
    for param in &contract.node.params {
        if !param_ids.insert(param.id) {
            return Err(PluginContractError::DuplicateParam {
                plugin,
                param: param.id,
            });
        }
        validate_param_contract(plugin, param)?;
    }
    validate_node_desc(contract.kind, &contract.node)?;
    validate_migration_plan(contract)
}

fn validate_param_contract(
    plugin: &'static str,
    param: &ParamDef,
) -> Result<(), PluginContractError> {
    let reject = |reason| PluginContractError::InvalidDomain {
        plugin,
        param: param.id,
        reason,
    };
    if !value_matches_type(param.value_type, &param.default) {
        return Err(reject(DomainError::DefaultTypeMismatch));
    }
    let finite = match &param.default {
        Value::F64(value) => value.is_finite(),
        Value::Vec2(value) => value.iter().all(|v| v.is_finite()),
        Value::Vec3(value) => value.iter().all(|v| v.is_finite()),
        Value::Color(value) => value.iter().all(|v| v.is_finite()),
        Value::AssetRef(_) => true,
    };
    if !finite {
        return Err(reject(DomainError::NonFiniteDefault));
    }
    if let Value::Color(value) = &param.default {
        if value.iter().any(|v| !(0.0..=1.0).contains(v)) {
            return Err(reject(DomainError::ColorDefaultOutsideUnitInterval));
        }
    }
    let Some(domain) = param.f64_domain else {
        return Ok(());
    };
    if param.value_type != ValueType::F64 {
        return Err(reject(DomainError::NonF64Parameter));
    }
    if domain
        .min_inclusive
        .into_iter()
        .chain(domain.max_inclusive)
        .any(|v| !v.is_finite())
    {
        return Err(reject(DomainError::NonFiniteBound));
    }
    if matches!(
        (domain.min_inclusive, domain.max_inclusive),
        (Some(min), Some(max)) if min > max
    ) {
        return Err(reject(DomainError::ReversedBounds));
    }
    let Value::F64(default) = param.default else {
        return Err(reject(DomainError::DefaultTypeMismatch));
    };
    if domain.min_inclusive.is_some_and(|min| default < min)
        || domain.max_inclusive.is_some_and(|max| default > max)
        || (domain.integer && default.fract() != 0.0)
    {
        return Err(reject(DomainError::DefaultOutsideDomain));
    }
    Ok(())
}

fn validate_migration_plan(contract: &PluginContract) -> Result<(), PluginContractError> {
    let plugin = contract.node.id.0;
    let mut from_versions = BTreeSet::new();
    for step in &contract.migrations {
        let reject = |reason| PluginContractError::InvalidMigration {
            plugin,
            from_version: step.from_version,
            to_version: step.to_version,
            reason,
        };
        if step.from_version == 0 || step.to_version == 0 {
            return Err(reject(MigrationPlanError::ZeroVersion));
        }
        if step.to_version != step.from_version.saturating_add(1) {
            return Err(reject(MigrationPlanError::NonAdjacentVersions));
        }
        if !from_versions.insert(step.from_version) {
            return Err(reject(MigrationPlanError::DuplicateFromVersion));
        }
        if step.to_version > contract.node.version {
            return Err(reject(MigrationPlanError::BeyondCurrentVersion));
        }
        let mut sources = BTreeSet::new();
        let mut destinations = BTreeSet::new();
        for op in &step.ops {
            match op {
                MigrationOp::RenameParam { from, to } => {
                    if from.is_empty() || to.is_empty() {
                        return Err(reject(MigrationPlanError::EmptyParamName));
                    }
                    if from == to {
                        return Err(reject(MigrationPlanError::SameParamName));
                    }
                    if !sources.insert(*from) {
                        return Err(reject(MigrationPlanError::DuplicateRenameSource));
                    }
                    if !destinations.insert(*to) {
                        return Err(reject(MigrationPlanError::DuplicateRenameDestination));
                    }
                }
            }
        }
    }
    Ok(())
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

    /// サイレントフォールバックは「もっともらしく間違う絵」の温床(M2E-8)。新規コードは`require_f64`。
    #[deprecated(note = "use require_f64; silent fallback hides type mistakes")]
    pub fn f64_or(&self, id: &'static str, fallback: f64) -> f64 {
        self.get(id).and_then(Value::as_f64).unwrap_or(fallback)
    }

    pub fn require_f64(&self, plugin: &str, id: &'static str) -> Result<f64, PluginError> {
        match self.get(id) {
            Some(Value::F64(v)) => Ok(*v),
            Some(v) => Err(PluginError::param_type(
                plugin,
                id,
                ValueType::F64,
                value_type_name(v),
            )),
            None => Err(PluginError::param_missing(plugin, id, ValueType::F64)),
        }
    }

    pub fn require_color(&self, plugin: &str, id: &'static str) -> Result<[f64; 4], PluginError> {
        match self.get(id) {
            Some(Value::Color(v)) => Ok(*v),
            Some(v) => Err(PluginError::param_type(
                plugin,
                id,
                ValueType::Color,
                value_type_name(v),
            )),
            None => Err(PluginError::param_missing(plugin, id, ValueType::Color)),
        }
    }

    pub fn require_vec2(&self, plugin: &str, id: &'static str) -> Result<[f64; 2], PluginError> {
        match self.get(id) {
            Some(Value::Vec2(v)) => Ok(*v),
            Some(v) => Err(PluginError::param_type(
                plugin,
                id,
                ValueType::Vec2,
                value_type_name(v),
            )),
            None => Err(PluginError::param_missing(plugin, id, ValueType::Vec2)),
        }
    }
}

impl NodeDesc {
    /// 生JSON params を desc に照合して解決する(M2E-8)。
    /// 未知ID→Err、型不一致→Err、欠落→`ParamDef.default` 充填。
    pub fn resolve_params(
        &self,
        raw: &HashMap<String, Value>,
    ) -> Result<ResolvedParams, PluginError> {
        let plugin = self.id.0;
        let known: BTreeSet<&str> = self.params.iter().map(|p| p.id).collect();
        for key in raw.keys() {
            if !known.contains(key.as_str()) {
                return Err(PluginError::Param {
                    plugin: plugin.to_string(),
                    id: key.clone(),
                    expected: "defined in NodeDesc".into(),
                    got: "unknown".into(),
                });
            }
        }

        let mut params = ResolvedParams::new();
        for def in &self.params {
            let value = match raw.get(def.id) {
                Some(v) if value_matches_type(def.value_type, v) => v.clone(),
                Some(v) => {
                    return Err(PluginError::param_type(
                        plugin,
                        def.id,
                        def.value_type,
                        value_type_name(v),
                    ));
                }
                None => def.default.clone(),
            };
            params.insert(def.id, value);
        }
        Ok(params)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureRef<'a> {
    pub texture: &'a wgpu::Texture,
    pub desc: FrameDesc,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamDriverContext {
    /// サンプル列の開始時刻(タイムライン)。
    pub start: RationalTime,
    /// 総尺。半開区間 `[start, start+duration)` を覆う(M2E-17)。
    /// サンプル添字は `0..sample_count`（終端ちょうどは範囲外）。
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
    #[error("invalid NodeDesc for `{id}`: {reason}")]
    InvalidDesc { id: String, reason: String },
    #[error("plugin render failed: {0}")]
    Render(String),
    #[error("param migrate failed for {plugin}: {reason}")]
    Migrate { plugin: String, reason: String },
    /// 型不一致・未知キー・欠落(require時)。サイレントデフォルトの代替。
    #[error("plugin `{plugin}` param `{id}`: expected {expected}, got {got}")]
    Param {
        plugin: String,
        id: String,
        expected: String,
        got: String,
    },
    #[error(transparent)]
    RationalTime(#[from] RationalTimeError),
}

impl PluginError {
    fn param_type(plugin: &str, id: &str, expected: ValueType, got: &str) -> Self {
        Self::Param {
            plugin: plugin.to_string(),
            id: id.to_string(),
            expected: expected.to_string(),
            got: got.to_string(),
        }
    }

    fn param_missing(plugin: &str, id: &str, expected: ValueType) -> Self {
        Self::Param {
            plugin: plugin.to_string(),
            id: id.to_string(),
            expected: expected.to_string(),
            got: "missing".into(),
        }
    }
}

/// 複製インスタンスの評価コンテキスト口(F-7予約。配線はM2以降)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstanceIndex {
    pub index: u32,
    pub count: u32,
}

/// 合体結果の別時刻参照(F-11予約。実装はM4後)。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CompLookbehind {
    /// グループIDまたはコンポルートの安定文字列。
    pub target: String,
    /// 負のフレームオフセット列(例: [-1, -2])。
    pub offsets: Vec<i32>,
    /// 自己参照切断用のエフェクトID列。
    pub exclude: Vec<String>,
}

/// 前後フレーム/サブフレーム要求の静的宣言(F-12予約。解決はホスト)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TemporalFootprint {
    pub frames_before: u32,
    pub frames_after: u32,
    /// モーションブラー用。上限は`Quality::effect_samples`。
    pub subframe_samples: u32,
}

/// Filter/Composite の per-call 文脈(M2E-7)。
///
/// `#[non_exhaustive]` — Quality・予約口の追加で既存プラグインのシグネチャを壊さない。
/// 外部クレートは`RenderCtx::new`経由で構築する。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RenderCtx {
    pub t: RationalTime,
    /// Draft/Final 判別と effect_samples の口。解像度畳み込み後の TextureRef.desc だけでは読めない。
    pub quality: Quality,
    /// F-7 予約。Repeater 配線まで常に None。
    pub instance: Option<InstanceIndex>,
    /// F-11 予約。M4 配線まで常に None。
    pub lookbehind: Option<CompLookbehind>,
    /// F-12 予約。窓テクスチャの解決はホスト側(現状はデフォルト=ゼロ窓)。
    pub temporal_footprint: TemporalFootprint,
}

impl RenderCtx {
    pub fn new(t: RationalTime, quality: Quality) -> Self {
        Self {
            t,
            quality,
            instance: None,
            lookbehind: None,
            temporal_footprint: TemporalFootprint::default(),
        }
    }
}

/// `NodeDesc`必須欄の機械判定(INF-7c、plugin-authoring §2)。
///
/// レジストリの`register_*`が必ず呼ぶため、テストを通るプラグインは検証済みになる
/// (§7チェックリスト「メタデータ完備」の目視を不要化)。
pub fn validate_node_desc(kind: PluginKind, desc: &NodeDesc) -> Result<(), PluginError> {
    let invalid = |reason: String| PluginError::InvalidDesc {
        id: desc.id.0.to_string(),
        reason,
    };
    let ident_ok = |s: &str| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    };

    let segments: Vec<&str> = desc.id.0.split('.').collect();
    if segments.len() != 3 || !segments.iter().all(|s| ident_ok(s)) {
        return Err(invalid(format!(
            "id must be `vendor.kind.name` (lowercase ascii), got `{}`",
            desc.id.0
        )));
    }
    // id中央セグメントは登録PluginKindと一致させる(core.param.* をFilterに登録する抜けを塞ぐ)。
    let expected_kind_seg = match kind {
        PluginKind::Filter => Some("filter"),
        PluginKind::ParamDriver => Some("param"),
        PluginKind::LayerSource => Some("layer_source"),
        PluginKind::Composite => Some("composite"),
        // 予約種別はレジストリ登録経路が無い。将来の口に合わせて緩めに置く。
        PluginKind::Input => Some("input"),
        PluginKind::Simulation => Some("simulation"),
        PluginKind::ScriptWasm => Some("script_wasm"),
    };
    if let Some(expected) = expected_kind_seg {
        if segments[1] != expected {
            return Err(invalid(format!(
                "id kind segment `{}` does not match {kind:?} (expected `{expected}`)",
                segments[1]
            )));
        }
    }
    if desc.version == 0 {
        return Err(invalid("version must be >= 1".into()));
    }
    if desc.display_name.trim().is_empty() {
        return Err(invalid("display_name is empty".into()));
    }
    if desc.category.trim().is_empty() {
        return Err(invalid("category is empty".into()));
    }
    if desc.tags.is_empty() {
        return Err(invalid("tags must not be empty (discovery/F-8)".into()));
    }
    if let Some(tag) = desc.tags.iter().find(|t| {
        t.is_empty()
            || !t
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    }) {
        return Err(invalid(format!(
            "tag `{tag}` must be short lowercase ascii"
        )));
    }
    let mut param_ids = BTreeSet::new();
    for param in &desc.params {
        if param.id.trim().is_empty() {
            return Err(invalid("param id is empty".into()));
        }
        if !param_ids.insert(param.id) {
            return Err(invalid(format!("duplicate param id `{}`", param.id)));
        }
        if !value_matches_type(param.value_type, &param.default) {
            return Err(invalid(format!(
                "param `{}` default does not match value_type {:?}",
                param.id, param.value_type
            )));
        }
    }
    if desc.min_inputs > desc.max_inputs {
        return Err(invalid(format!(
            "min_inputs {} > max_inputs {}",
            desc.min_inputs, desc.max_inputs
        )));
    }
    // 入出力アリティは種別の契約(plugin-authoring §1)そのもの。
    let arity_ok = match kind {
        PluginKind::LayerSource | PluginKind::ParamDriver => {
            desc.min_inputs == 0 && desc.max_inputs == 0
        }
        PluginKind::Filter => desc.min_inputs == 1 && desc.max_inputs == 1,
        PluginKind::Composite => desc.min_inputs >= 2,
        // 予約種別はレジストリ登録経路が無いため、ここでは制約しない。
        PluginKind::Input | PluginKind::Simulation | PluginKind::ScriptWasm => true,
    };
    if !arity_ok {
        return Err(invalid(format!(
            "inputs [{}, {}] violate {kind:?} arity contract",
            desc.min_inputs, desc.max_inputs
        )));
    }
    Ok(())
}

/// プラグインparamの版間移行(G-1 / FG-C4)。
///
/// `from_version` → `to_version` へ `params` を破壊的に書き換える。
/// 未知プラグインは何もしない(呼び出し側がF-9パススルーを担当)。
pub fn migrate_plugin_params(
    plugin_id: &str,
    from_version: u32,
    to_version: u32,
    params: &mut HashMap<String, Value>,
) -> Result<(), PluginError> {
    if from_version == to_version {
        return Ok(());
    }
    if from_version > to_version {
        return Err(PluginError::Migrate {
            plugin: plugin_id.to_string(),
            reason: format!("cannot downgrade params {from_version} → {to_version}"),
        });
    }
    match plugin_id {
        "core.param.sine" => migrate_sine_params(from_version, to_version, params),
        _ => Ok(()),
    }
}

fn migrate_sine_params(
    from_version: u32,
    to_version: u32,
    params: &mut HashMap<String, Value>,
) -> Result<(), PluginError> {
    // v1→v2: `amp` を `amplitude` に改名(参照プラグインの破壊的変更デモ)。
    if from_version < 2 && to_version >= 2 {
        if let Some(v) = params.remove("amp") {
            if params.contains_key("amplitude") {
                return Err(PluginError::Migrate {
                    plugin: "core.param.sine".into(),
                    reason: "both amp and amplitude present during migrate".into(),
                });
            }
            params.insert("amplitude".into(), v);
        }
    }
    Ok(())
}

pub trait FilterPlugin: Send + Sync {
    fn desc(&self) -> &NodeDesc;

    // プラグイン契約の引数集合(GPU/文脈/params/入出力)が閾値を超えるのは構造上のもの。
    #[allow(clippy::too_many_arguments)]
    fn render(
        &self,
        gpu: &GpuCtx,
        pipelines: &mut PipelineCache,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &RenderCtx,
        params: &ResolvedParams,
        input: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), PluginError>;
}

pub trait LayerSourcePlugin: Send + Sync {
    fn desc(&self) -> &NodeDesc;

    #[allow(clippy::too_many_arguments)]
    fn render(
        &self,
        gpu: &GpuCtx,
        pipelines: &mut PipelineCache,
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

    #[allow(clippy::too_many_arguments)]
    fn render(
        &self,
        gpu: &GpuCtx,
        pipelines: &mut PipelineCache,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &RenderCtx,
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

/// M1-T12用の最小参照プラグイン群。
///
/// Filter/CompositeはGPU render passだけを発行する。CPUピクセル処理の迂回路は持たない。
pub mod reference {
    use super::*;

    pub static CLEAR_FILTER: ClearFilter = ClearFilter;
    pub static TINT_FILTER: TintFilter = TintFilter;
    /// INF-7g 実演: LLMが new-plugin 型紙から肉付けした参照Filter。
    pub static OPACITY_FILTER: OpacityFilter = OpacityFilter;
    pub static CLEAR_LAYER_SOURCE: ClearLayerSource = ClearLayerSource;
    pub static SINE_PARAM_DRIVER: SineParamDriver = SineParamDriver;
    pub static CLEAR_COMPOSITE: ClearComposite = ClearComposite;

    pub fn register_reference_plugins(registry: &mut PluginRegistry) -> Result<(), PluginError> {
        registry.register_layer_source(&CLEAR_LAYER_SOURCE)?;
        registry.register_filter(&CLEAR_FILTER)?;
        registry.register_filter(&TINT_FILTER)?;
        registry.register_filter(&OPACITY_FILTER)?;
        registry.register_param_driver(&SINE_PARAM_DRIVER)?;
        registry.register_composite(&CLEAR_COMPOSITE)?;
        Ok(())
    }

    pub fn register_reference_contracts(
        catalog: &mut PluginCatalogBuilder,
    ) -> Result<(), PluginContractError> {
        for (kind, node, migrations) in [
            (PluginKind::LayerSource, clear_layer_source_desc(), vec![]),
            (PluginKind::Filter, clear_filter_desc(), vec![]),
            (PluginKind::Filter, tint_filter_desc(), vec![]),
            (PluginKind::Filter, opacity_filter_desc(), vec![]),
            (
                PluginKind::ParamDriver,
                sine_param_desc(),
                vec![MigrationStep {
                    from_version: 1,
                    to_version: 2,
                    ops: vec![MigrationOp::RenameParam {
                        from: "amp",
                        to: "amplitude",
                    }],
                }],
            ),
            (PluginKind::Composite, clear_composite_desc(), vec![]),
        ] {
            catalog.register(PluginContract {
                kind,
                node: node.clone(),
                migrations,
            })?;
        }
        Ok(())
    }

    pub fn reference_catalog() -> Result<PluginCatalog, PluginContractError> {
        let mut builder = PluginCatalogBuilder::new();
        register_reference_contracts(&mut builder)?;
        builder.build()
    }

    pub struct ClearFilter;

    impl FilterPlugin for ClearFilter {
        fn desc(&self) -> &NodeDesc {
            clear_filter_desc()
        }

        fn render(
            &self,
            _gpu: &GpuCtx,
            _pipelines: &mut PipelineCache,
            encoder: &mut wgpu::CommandEncoder,
            _ctx: &RenderCtx,
            params: &ResolvedParams,
            _input: TextureRef<'_>,
            output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            clear_texture(
                encoder,
                output,
                color_from_params("core.filter.clear", params)?,
            );
            Ok(())
        }
    }

    /// PipelineCache実証用の実Filter(所見2/F-10)。入力をcolorで乗算する。
    pub struct TintFilter;

    impl FilterPlugin for TintFilter {
        fn desc(&self) -> &NodeDesc {
            tint_filter_desc()
        }

        fn render(
            &self,
            gpu: &GpuCtx,
            pipelines: &mut PipelineCache,
            encoder: &mut wgpu::CommandEncoder,
            _ctx: &RenderCtx,
            params: &ResolvedParams,
            input: TextureRef<'_>,
            output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            use motolii_gpu::PipelineCacheKey;

            let cached = pipelines.get_or_create_tex_sample_uniform4(
                gpu,
                PipelineCacheKey {
                    id: "core.filter.tint",
                    wgsl: TINT_WGSL,
                },
            );
            // UI/APIのcolorはstraight。シェーダ側でunpremul→乗算→premulする。
            let [r, g, b, a] = params.require_color("core.filter.tint", "color")?;
            let color = [r as f32, g as f32, b as f32, a as f32];
            gpu.queue
                .write_buffer(&cached.uniform_buffer, 0, bytemuck::bytes_of(&color));
            let input_view = input
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let output_view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            // bind group / view は入力テクスチャがフレームごとに差し替わるため都度生成
            // (OverlayNodeと同じ。バッファ/パイプラインはキャッシュ済み)。
            let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("core.filter.tint.bg"),
                layout: &cached.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&cached.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: cached.uniform_buffer.as_entire_binding(),
                    },
                ],
            });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("core.filter.tint.pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    multiview_mask: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&cached.pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
            Ok(())
        }
    }

    /// 不透明度乗算(INF-7g)。premul RGBA 全体に `amount` を掛ける。
    pub struct OpacityFilter;

    impl FilterPlugin for OpacityFilter {
        fn desc(&self) -> &NodeDesc {
            opacity_filter_desc()
        }

        fn render(
            &self,
            gpu: &GpuCtx,
            pipelines: &mut PipelineCache,
            encoder: &mut wgpu::CommandEncoder,
            _ctx: &RenderCtx,
            params: &ResolvedParams,
            input: TextureRef<'_>,
            output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            use motolii_gpu::PipelineCacheKey;

            let cached = pipelines.get_or_create_tex_sample_uniform4(
                gpu,
                PipelineCacheKey {
                    id: "core.filter.opacity",
                    wgsl: OPACITY_WGSL,
                },
            );
            let amount = params
                .require_f64("core.filter.opacity", "amount")?
                .clamp(0.0, 1.0) as f32;
            let uniform = [amount, 0.0, 0.0, 0.0];
            gpu.queue
                .write_buffer(&cached.uniform_buffer, 0, bytemuck::bytes_of(&uniform));
            let input_view = input
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let output_view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("core.filter.opacity.bg"),
                layout: &cached.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&cached.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: cached.uniform_buffer.as_entire_binding(),
                    },
                ],
            });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("core.filter.opacity.pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    multiview_mask: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&cached.pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
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
            _pipelines: &mut PipelineCache,
            encoder: &mut wgpu::CommandEncoder,
            _t: RationalTime,
            params: &ResolvedParams,
            ctx: LayerSourceContext,
            output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            ctx.camera.validate().map_err(PluginError::Render)?;
            clear_texture(
                encoder,
                output,
                color_from_params("core.layer_source.clear", params)?,
            );
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
            _pipelines: &mut PipelineCache,
            encoder: &mut wgpu::CommandEncoder,
            _ctx: &RenderCtx,
            params: &ResolvedParams,
            _inputs: &[TextureRef<'_>],
            output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            clear_texture(
                encoder,
                output,
                color_from_params("core.composite.clear", params)?,
            );
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
            let amplitude = params.require_f64("core.param.sine", "amplitude")?;
            let frequency_hz = params.require_f64("core.param.sine", "frequency_hz")?;
            let offset = params.require_f64("core.param.sine", "offset")?;
            let samples = sample_count(ctx.duration, ctx.sample_rate)?;
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

    fn tint_filter_desc() -> &'static NodeDesc {
        static DESC: OnceLock<NodeDesc> = OnceLock::new();
        DESC.get_or_init(|| NodeDesc {
            id: PluginId("core.filter.tint"),
            version: 1,
            display_name: "Tint",
            category: "Color",
            tags: &["tint", "color", "reference"],
            params: vec![ParamDef {
                id: "color",
                value_type: ValueType::Color,
                default: Value::Color([1.0, 1.0, 1.0, 1.0]),
                f64_domain: None,
            }],
            min_inputs: 1,
            max_inputs: 1,
        })
    }

    fn opacity_filter_desc() -> &'static NodeDesc {
        static DESC: OnceLock<NodeDesc> = OnceLock::new();
        DESC.get_or_init(|| NodeDesc {
            id: PluginId("core.filter.opacity"),
            version: 1,
            display_name: "Opacity",
            category: "Color",
            tags: &["opacity", "alpha", "reference"],
            params: vec![ParamDef {
                id: "amount",
                value_type: ValueType::F64,
                default: Value::F64(1.0),
                f64_domain: Some(F64Domain::unit()),
            }],
            min_inputs: 1,
            max_inputs: 1,
        })
    }

    const TINT_WGSL: &str = r#"
struct TintUniform {
    color: vec4<f32>,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<uniform> tint: TintUniform;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    let p = positions[vertex_index];
    var out: VsOut;
    out.pos = vec4<f32>(p, 0.0, 1.0);
    out.uv = p * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let tin = textureSample(input_tex, tex_sampler, in.uv);
    let t = tint.color;
    let out_a = tin.a * t.a;
    let rgb = select(tin.rgb / max(tin.a, 1e-5), vec3<f32>(0.0), tin.a == 0.0) * t.rgb;
    return vec4<f32>(rgb * out_a, out_a);
}
"#;

    const OPACITY_WGSL: &str = r#"
struct OpacityUniform {
    // x = amount (0..1). yzw unused (tex_sample_uniform4 スロットに合わせる)。
    amount: vec4<f32>,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<uniform> opacity: OpacityUniform;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    let p = positions[vertex_index];
    var out: VsOut;
    out.pos = vec4<f32>(p, 0.0, 1.0);
    out.uv = p * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let tin = textureSample(input_tex, tex_sampler, in.uv);
    return tin * opacity.amount.x;
}
"#;

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
            // v2: `amp` → `amplitude`(FG-C4 migrate実証)。
            version: 2,
            display_name: "Sine",
            category: "Generate",
            tags: &["lfo", "oscillator", "reference"],
            params: vec![
                ParamDef {
                    id: "amplitude",
                    value_type: ValueType::F64,
                    default: Value::F64(1.0),
                    f64_domain: None,
                },
                ParamDef {
                    id: "frequency_hz",
                    value_type: ValueType::F64,
                    default: Value::F64(1.0),
                    f64_domain: None,
                },
                ParamDef {
                    id: "offset",
                    value_type: ValueType::F64,
                    default: Value::F64(0.0),
                    f64_domain: None,
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
            f64_domain: None,
        }]
    }

    fn color_from_params(
        plugin: &str,
        params: &ResolvedParams,
    ) -> Result<wgpu::Color, PluginError> {
        let [r, g, b, a] = params.require_color(plugin, "color")?;
        Ok(wgpu::Color { r, g, b, a })
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

    /// 半開 `[0, duration)` 上の等間隔サンプル数。`duration` は総尺(M2E-17)。
    /// サンプル時刻は `i / rate`（i = 0,1,…）で、`i/rate < duration` を満たす個数
    /// = 有理数の厳密 `ceil(duration × rate)`。整数境界では旧 `floor` と同じ。
    /// 無条件の `floor+1`(フェンスポスト)には戻さない。
    pub(super) fn sample_count(
        duration: RationalTime,
        sample_rate: Fps,
    ) -> Result<usize, RationalTimeError> {
        // ceil((d.num/d.den)*(r.num/r.den)) = ceil((d.num*r.num)/(d.den*r.den))
        let num = (duration.num() as i128)
            .checked_mul(sample_rate.num() as i128)
            .ok_or(RationalTimeError::Overflow)?;
        let den = (duration.den() as i128)
            .checked_mul(sample_rate.den() as i128)
            .ok_or(RationalTimeError::Overflow)?;
        if den <= 0 {
            return Err(RationalTimeError::ZeroDenominator);
        }
        if num <= 0 {
            return Ok(0);
        }
        let n = num
            .checked_add(den - 1)
            .ok_or(RationalTimeError::Overflow)?
            / den;
        usize::try_from(n).map_err(|_| RationalTimeError::Overflow)
    }
}

// 公開APIのパニック禁止(INF-7b)は本番コードにlintを効かせ、テストmodだけ免除する。
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::reference::{register_reference_plugins, CLEAR_LAYER_SOURCE, SINE_PARAM_DRIVER};
    use super::*;

    #[test]
    fn registry_keeps_plugin_kinds_separate() {
        let mut registry = PluginRegistry::new();
        register_reference_plugins(&mut registry).unwrap();

        assert_eq!(registry.len(PluginKind::LayerSource), 1);
        assert_eq!(registry.len(PluginKind::Filter), 3);
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
        assert!(registry.composite_by_name("core.composite.clear").is_some());
        assert!(registry
            .layer_source_by_name("core.layer_source.clear")
            .is_some());
        assert!(registry.param_driver_by_name("core.param.sine").is_some());
        assert!(registry.filter_by_name("missing").is_none());

        assert_eq!(registry.iter(PluginKind::Filter).count(), 3);
        assert_eq!(registry.iter(PluginKind::ParamDriver).count(), 1);
        assert_eq!(registry.iter(PluginKind::LayerSource).count(), 1);
        assert_eq!(registry.iter(PluginKind::Composite).count(), 1);
        assert_eq!(registry.iter(PluginKind::Input).count(), 0);
        let filter_ids: Vec<&str> = registry
            .iter(PluginKind::Filter)
            .map(|(id, _)| id.0)
            .collect();
        assert!(filter_ids.contains(&"core.filter.clear"));
        assert!(filter_ids.contains(&"core.filter.tint"));
        assert!(filter_ids.contains(&"core.filter.opacity"));
    }

    #[test]
    fn runtime_rejects_kind_mismatch_even_if_catalog_was_not_built_normally() {
        let node = super::reference::OPACITY_FILTER.desc().clone();
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
            .register_filter(&super::reference::OPACITY_FILTER)
            .unwrap();
        let err = PluginRuntime::try_new(catalog, executors).unwrap_err();
        assert!(matches!(
            err,
            PluginRuntimeError::KindMismatch {
                id: "core.filter.opacity",
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
                    sample_rate: Fps::try_new(4, 1).unwrap(),
                },
                &params,
            )
            .unwrap();

        // M2E-17: 半開 [0,1) @ 4fps → 4サンプル(旧 fence-post の5を廃止)
        assert_eq!(track.values.len(), 4);
        assert_eq!(track.values[0], Value::F64(10.0));
        assert!((track.values[1].as_f64().unwrap() - 12.0).abs() < 1e-9);
    }

    #[test]
    fn sample_count_is_half_open_excluding_end() {
        // duration=1s @ 4fps → 半開 [0,1) はサンプル 0..4（旧 fence-post の5ではない）
        assert_eq!(
            reference::sample_count(RationalTime::from_seconds(1), Fps::try_new(4, 1).unwrap())
                .unwrap(),
            4
        );
        // 総尺ちょうど(= end)のフレーム添字は範囲外: 90フレーム総尺ならサンプル数90
        let fps = Fps::try_new(30, 1).unwrap();
        let duration = RationalTime::try_from_frame(90, fps).unwrap();
        assert_eq!(reference::sample_count(duration, fps).unwrap(), 90);
        // 右端時刻は半開で範囲外。最終内包フレームは89
        assert_eq!(RationalTime::try_from_frame(90, fps).unwrap(), duration);
        assert!(RationalTime::try_from_frame(89, fps).unwrap() < duration);
    }

    #[test]
    fn sample_count_ceil_keeps_in_range_samples_off_grid() {
        let rate = Fps::try_new(4, 1).unwrap();
        // 0.3s @ 4Hz: 区間内は 0, 0.25 → 2。floor(1.2)=1 では落とす。
        assert_eq!(
            reference::sample_count(RationalTime::try_new(3, 10).unwrap(), rate).unwrap(),
            2
        );
        // 0.1s @ 4Hz: 区間内は t=0 のみ → 1。floor(0.4)=0 では空になる。
        assert_eq!(
            reference::sample_count(RationalTime::try_new(1, 10).unwrap(), rate).unwrap(),
            1
        );
        // 整数境界は ceil=floor。無条件 floor+1 には戻らない。
        assert_eq!(
            reference::sample_count(RationalTime::from_seconds(1), rate).unwrap(),
            4
        );
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
        assert_eq!(driver.version, 2);
    }

    #[test]
    fn migrate_sine_renames_amp_to_amplitude() {
        let mut params = HashMap::new();
        params.insert("amp".into(), Value::F64(0.5));
        params.insert("frequency_hz".into(), Value::F64(2.0));
        migrate_plugin_params("core.param.sine", 1, 2, &mut params).unwrap();
        assert_eq!(params.get("amplitude"), Some(&Value::F64(0.5)));
        assert!(!params.contains_key("amp"));
        assert_eq!(params.get("frequency_hz"), Some(&Value::F64(2.0)));
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
        use super::reference::{
            CLEAR_COMPOSITE, CLEAR_FILTER, CLEAR_LAYER_SOURCE, OPACITY_FILTER, TINT_FILTER,
        };
        validate_node_desc(PluginKind::Filter, CLEAR_FILTER.desc()).unwrap();
        validate_node_desc(PluginKind::Filter, TINT_FILTER.desc()).unwrap();
        validate_node_desc(PluginKind::Filter, OPACITY_FILTER.desc()).unwrap();
        validate_node_desc(PluginKind::LayerSource, CLEAR_LAYER_SOURCE.desc()).unwrap();
        validate_node_desc(PluginKind::ParamDriver, SINE_PARAM_DRIVER.desc()).unwrap();
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

    #[test]
    fn resolve_params_fills_defaults_and_rejects_unknown_or_mismatch() {
        let desc = SINE_PARAM_DRIVER.desc();
        let empty = HashMap::new();
        let filled = desc.resolve_params(&empty).unwrap();
        assert_eq!(
            filled.require_f64("core.param.sine", "amplitude").unwrap(),
            1.0
        );
        assert_eq!(
            filled
                .require_f64("core.param.sine", "frequency_hz")
                .unwrap(),
            1.0
        );
        assert_eq!(
            filled.require_f64("core.param.sine", "offset").unwrap(),
            0.0
        );

        let mut unknown = HashMap::new();
        unknown.insert("nope".into(), Value::F64(1.0));
        let err = desc.resolve_params(&unknown).unwrap_err();
        assert!(
            matches!(
                err,
                PluginError::Param {
                    ref id,
                    ref got,
                    ..
                } if id == "nope" && got == "unknown"
            ),
            "{err:?}"
        );

        let mut mismatch = HashMap::new();
        mismatch.insert("amplitude".into(), Value::Vec2([0.0, 1.0]));
        let err = desc.resolve_params(&mismatch).unwrap_err();
        assert!(
            matches!(
                err,
                PluginError::Param {
                    ref id,
                    ref expected,
                    ref got,
                    ..
                } if id == "amplitude" && expected == "F64" && got == "Vec2"
            ),
            "{err:?}"
        );
    }

    #[test]
    fn require_f64_rejects_wrong_type_and_missing() {
        let mut params = ResolvedParams::new();
        params.insert("amplitude", Value::Vec2([1.0, 2.0]));
        let err = params
            .require_f64("core.param.sine", "amplitude")
            .unwrap_err();
        assert!(
            matches!(
                err,
                PluginError::Param {
                    ref expected,
                    ref got,
                    ..
                } if expected == "F64" && got == "Vec2"
            ),
            "{err:?}"
        );

        let empty = ResolvedParams::new();
        let err = empty
            .require_f64("core.param.sine", "amplitude")
            .unwrap_err();
        assert!(
            matches!(
                err,
                PluginError::Param {
                    ref got,
                    ..
                } if got == "missing"
            ),
            "{err:?}"
        );
    }

    #[test]
    fn reference_impl_does_not_call_silent_f64_fallback() {
        // 完了条件: 参照実装からサイレントf64フォールバック呼び出しが消えている(M2E-8)。
        let src = include_str!("lib.rs");
        let start = src
            .find("pub mod reference")
            .expect("reference module marker");
        // テストmod自身の文字列に引っかからないよう、参照モジュール本体だけを見る。
        let body = &src[start..];
        let end = body.find("\n#[cfg(test)]").unwrap_or(body.len());
        let reference = &body[..end];
        let forbidden = format!(".{}(", "f64_or");
        assert!(
            !reference.contains(&forbidden),
            "reference plugins must use require_* instead of silent f64 fallback"
        );
    }
}

/// M2E-10: `new-plugin` 生成物を自己クレート配置でコンパイル検証する口。
/// 実体は OUT_DIR(build.rs)。`MOTOLII_SCAFFOLD_FIXTURE` 未設定時は空モジュール。
/// ソースに欠落 `#[path]` を置かない(rustfmt が cfg を無視するため)。
pub mod scaffold_fixture {
    include!(concat!(env!("OUT_DIR"), "/scaffold_fixture_mods.rs"));
}
