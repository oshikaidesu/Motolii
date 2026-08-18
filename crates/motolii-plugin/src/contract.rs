//! plugin契約のスキーマとカタログ検証。

use std::collections::{BTreeMap, BTreeSet};

use motolii_core::RationalTimeError;
use motolii_eval::Value;

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

/// `List` の要素型。要素は既存のスカラ・ベクトル型に限る。
///
/// `ValueType`と別の型にしてあるのは、`List`の入れ子を**型で**禁じるためである
/// (決定 2.1「`List`の入れ子は許さない」)。`List(Box<ValueType>)`にすると
/// `Copy`が実装できず(`E0204`)、`as_str`が`&'static str`を返せなくなる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementType {
    F64,
    Vec2,
    Vec3,
    Color,
    AssetRef,
}

impl ElementType {
    pub fn as_str(self) -> &'static str {
        self.as_value_type().as_str()
    }

    /// 要素型を単体の`ValueType`として見る。
    pub fn as_value_type(self) -> ValueType {
        match self {
            ElementType::F64 => ValueType::F64,
            ElementType::Vec2 => ValueType::Vec2,
            ElementType::Vec3 => ValueType::Vec3,
            ElementType::Color => ValueType::Color,
            ElementType::AssetRef => ValueType::AssetRef,
        }
    }
}

impl std::fmt::Display for ElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    F64,
    Vec2,
    Vec3,
    Color,
    /// アセットID参照(F-10予約。実装結線はM2 D1)。
    AssetRef,
    /// 同種のものが可変個並ぶ型。長さの検査は plugin 側の責任。
    List(ElementType),
}

impl ValueType {
    pub fn as_str(self) -> &'static str {
        match self {
            ValueType::F64 => "F64",
            ValueType::Vec2 => "Vec2",
            ValueType::Vec3 => "Vec3",
            ValueType::Color => "Color",
            ValueType::AssetRef => "AssetRef",
            ValueType::List(ElementType::F64) => "List<F64>",
            ValueType::List(ElementType::Vec2) => "List<Vec2>",
            ValueType::List(ElementType::Vec3) => "List<Vec3>",
            ValueType::List(ElementType::Color) => "List<Color>",
            ValueType::List(ElementType::AssetRef) => "List<AssetRef>",
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
        // 要素型は先頭要素から名乗る。空listは要素型を名乗れない。
        Value::List(items) => match items.first() {
            Some(Value::F64(_)) => "List<F64>",
            Some(Value::Vec2(_)) => "List<Vec2>",
            Some(Value::Vec3(_)) => "List<Vec3>",
            Some(Value::Color(_)) => "List<Color>",
            Some(Value::AssetRef(_)) => "List<AssetRef>",
            Some(Value::List(_)) => "List<List>",
            None => "List",
        },
    }
}

pub fn value_matches_type(value_type: ValueType, value: &Value) -> bool {
    match (value_type, value) {
        // 空listは長さの問題であって型の問題ではないので、どの要素型にも一致する。
        (ValueType::List(element), Value::List(items)) => items
            .iter()
            .all(|item| value_matches_type(element.as_value_type(), item)),
        _ => matches!(
            (value_type, value),
            (ValueType::F64, Value::F64(_))
                | (ValueType::Vec2, Value::Vec2(_))
                | (ValueType::Vec3, Value::Vec3(_))
                | (ValueType::Color, Value::Color(_))
                | (ValueType::AssetRef, Value::AssetRef(_))
        ),
    }
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
    // 試験が検証を迂回した不正カタログを組み立てるため。
    pub(super) contracts: BTreeMap<PluginId, PluginContract>,
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
    if !value_is_finite(&param.default) {
        return Err(reject(DomainError::NonFiniteDefault));
    }
    if !colors_in_unit_interval(&param.default) {
        return Err(reject(DomainError::ColorDefaultOutsideUnitInterval));
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

/// `List`は要素へ降りて検査する(入れ子は`value_matches_type`が既に弾いている)。
fn value_is_finite(value: &Value) -> bool {
    match value {
        Value::F64(value) => value.is_finite(),
        Value::Vec2(value) => value.iter().all(|v| v.is_finite()),
        Value::Vec3(value) => value.iter().all(|v| v.is_finite()),
        Value::Color(value) => value.iter().all(|v| v.is_finite()),
        Value::AssetRef(_) => true,
        Value::List(items) => items.iter().all(value_is_finite),
    }
}

fn colors_in_unit_interval(value: &Value) -> bool {
    match value {
        Value::Color(value) => value.iter().all(|v| (0.0..=1.0).contains(v)),
        Value::List(items) => items.iter().all(colors_in_unit_interval),
        _ => true,
    }
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
    pub(super) fn param_type(plugin: &str, id: &str, expected: ValueType, got: &str) -> Self {
        Self::Param {
            plugin: plugin.to_string(),
            id: id.to_string(),
            expected: expected.to_string(),
            got: got.to_string(),
        }
    }

    pub(super) fn param_missing(plugin: &str, id: &str, expected: ValueType) -> Self {
        Self::Param {
            plugin: plugin.to_string(),
            id: id.to_string(),
            expected: expected.to_string(),
            got: "missing".into(),
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
