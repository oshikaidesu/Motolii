# VSM-A0S — Contract Catalogとprepared plugin解決仕様

作成日: 2026-07-17

状態: **仕様決定／A0I-1〜3実装完了**。本書は[VSM-A0D](2026-07-17-vism-a0d-contract-migration-ownership-decision.md)の所有決定を、実装時に別解を発明できない公開型・呼出境界・拒否fixtureへ落とす。`.vism` manifest、動的loader、UI、永続schemaは対象外である。

関連文書: [VSM-A0 inventory](2026-07-17-vism-a0-plugin-boundary-inventory.md)、[Vism実装計画](2026-07-17-vism-implementation-plan.md)、[M2 Document仕様](../specs/M2-document-model.md)、[plugin作者規約](../plugin-authoring.md)

## 1. 結論

現行の「構造検証」「plugin意味の検査」「実行可能性」を一つの`validate`へ畳まない。

```text
Document bytes
  │
  ├─ intrinsic validation ─→ 保存構造として有効
  │
  └─ Contract Catalog
       └─ prepared resolution ─→ 現行contractで意味が有効
            └─ Executor Registry ─→ このHostで実行可能
```

- `Document::validate()`はintrinsic validationとして残す。pluginが欠落してもraw recipeを開いて保存できるためである。
- pluginのkind、version、parameter型／値域、migrationは`PluginCatalog`だけを正本にする。
- graph、preview、exportは検証済みの`PluginRuntime`を必須引数にする。catalogを渡し忘れた実行経路をコンパイル可能にしない。
- `DocumentWriter`はcatalogを保持する。既知pluginのparameterを編集した直後にcontract検査を行い、first-party ID表へ戻らない。
- load／save／journal recoveryは構造保持の低水準境界としてcatalog非依存を維持する。製品openは別のcatalog必須入口を通る。
- raw Documentはprepared resolutionで変更しない。

## 2. crate依存方向

```text
motolii-core / motolii-eval
          ↑
motolii-plugin
          ↑
motolii-doc
          ↑
motolii-export / product composition root
```

`motolii-plugin`はcontract、catalog、executor registry、runtime整合を所有する。`motolii-doc`は`DocParam`を知るため、raw recipeのcloneとdeclarative migration interpreterを所有する。

禁止する依存:

- `motolii-plugin → motolii-doc`
- contract型からserde、Slint、wgpu texture、OS／vendor APIへの依存
- global mutable catalog
- executor callbackによるProject open時migration

現行`motolii-doc → motolii-plugin`本番依存は既に存在する。`param_expect.rs`の「本番コードでは依存しない」というコメントはA0Iで削除する。

## 3. `motolii-plugin`の公開型

以下の名前とfieldをA0Iの仕様とする。A0Iで同義の別型を追加しない。

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct F64Domain {
    pub min_inclusive: Option<f64>,
    pub max_inclusive: Option<f64>,
    pub integer: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamDef {
    pub id: &'static str,
    pub value_type: ValueType,
    pub default: Value,
    pub f64_domain: Option<F64Domain>,
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

#[derive(Debug, Clone)]
pub struct PluginCatalog { /* private immutable map */ }

#[derive(Debug, Default)]
pub struct PluginCatalogBuilder { /* private mutable map */ }
```

`PluginCatalogBuilder`だけが登録を受け付け、`build(self)`後の`PluginCatalog`にはmutating APIを置かない。

```rust
impl PluginCatalogBuilder {
    pub fn new() -> Self;
    pub fn register(
        &mut self,
        contract: PluginContract,
    ) -> Result<(), PluginContractError>;
    pub fn build(self) -> Result<PluginCatalog, PluginContractError>;
}

impl PluginCatalog {
    pub fn get(&self, id: &str) -> Option<&PluginContract>;
    pub fn iter(&self) -> impl Iterator<Item = (&PluginId, &PluginContract)>;
}
```

### 3.1 value domain

`f64_domain`は`ValueType::F64`だけで`Some`を許す。境界は両端包含で、`integer=true`は有限かつ`fract() == 0.0`を要求する。

builderは次を拒否する。

- NaN／Infの境界またはdefault
- `min_inclusive > max_inclusive`
- F64以外への`f64_domain`
- domain外のdefault
- 重複parameter ID

Colorは既決のstraight sRGB／各成分0〜1をHost共通契約として扱い、pluginごとのdomain再宣言を置かない。Vec2／Vec3の成分domain、単位、UI widget hintはA0Sの外である。

### 3.2 migration

builderは次を拒否する。

- `from_version == 0`または`to_version == 0`
- `to_version != from_version + 1`
- 同じ`from_version`の重複step
- `to_version > node.version`
- 空のparameter名、`from == to`
- 同一step内で同じsourceまたはdestinationを複数回使うrename

version 1にmigrationは不要である。`node.version > 1`でも、version 1からの完全chainを必須にはしない。古いrecipeが実在したときにchain欠落を`MigrationStepMissing`として診断できるためである。未使用の歴史を作者へ捏造させない。

`PluginId(&'static str)`と`&'static str`はpre-Vism静的リンク境界の型であり、package identityや`.vism` wire型ではない。

## 4. Executor Registryとの整合

現行`PluginRegistry`はexecutor集合として残す。登録APIはA0I中の構築用に維持するが、graph／exportは裸のregistryを受け取らない。

```rust
pub struct PluginRuntime {
    catalog: Arc<PluginCatalog>,
    executors: PluginRegistry,
}

impl PluginRuntime {
    pub fn try_new(
        catalog: Arc<PluginCatalog>,
        executors: PluginRegistry,
    ) -> Result<Self, PluginRuntimeError>;

    pub fn catalog(&self) -> &PluginCatalog;
    pub fn executors(&self) -> &PluginRegistry;
}
```

`try_new`は全executorについて同じIDのcontractがあり、kind、`NodeDesc`、current versionが一致することを要求する。executorだけの登録、ID／kind／version／parameter schema不一致はstartup errorである。

contractだけの登録は許す。headless検査と「contractはあるがこのHostにはexecutorがない」という診断に必要だからである。特定recipeの実行時にexecutorが無ければ`ExecutorMissing`となる。

`doc.layer_source.rect`は`PluginCatalog`へ登録しない。Document built-inの判定表を`motolii-doc`に残し、catalog／registryのどちらかが同じIDを使った場合はproduct composition rootの構築エラーとする。

## 5. `motolii-doc`の解決型

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PluginSlotId {
    LayerSource(LayerId),
    EffectDefinition(EffectDefinitionId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedPluginRecipe {
    pub plugin_id: String,
    pub saved_version: u32,
    pub current_version: u32,
    pub params: BTreeMap<String, DocParam>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginDiagnosticReason {
    ContractMissing,
    FutureVersion {
        current_version: u32,
        saved_version: u32,
    },
    MigrationStepMissing {
        from_version: u32,
    },
    MigrationConflict {
        from: String,
        to: String,
    },
    ContractViolation,
    ExecutorMissing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginDiagnostic {
    pub slot: PluginSlotId,
    pub plugin_id: String,
    pub reason: PluginDiagnosticReason,
}

pub struct PreparedDocumentPlugins {
    /* PluginSlotId → PreparedPluginRecipe。private */
    /* diagnostics。private */
}

impl PreparedDocumentPlugins {
    pub fn get(&self, slot: &PluginSlotId) -> Option<&PreparedPluginRecipe>;
    pub fn diagnostics(&self) -> &[PluginDiagnostic];
    pub fn is_fully_prepared(&self) -> bool;
    pub fn execution_diagnostics(
        &self,
        runtime: &PluginRuntime,
    ) -> Vec<PluginDiagnostic>;
}
```

`ContractViolation`の詳細はsource付きの型付きerrorとして保持し、表示のための文字列だけに潰さない。上記enumは分類である。

公開入口:

```rust
impl Document {
    pub fn prepare_plugins(
        &self,
        catalog: &PluginCatalog,
    ) -> Result<PreparedDocumentPlugins, DocumentPluginError>;
}

pub fn prepare_plugin_recipe(
    plugin_id: &str,
    expected_kind: PluginKind,
    saved_version: u32,
    params: &BTreeMap<String, DocParam>,
    catalog: &PluginCatalog,
) -> Result<PreparedPluginRecipe, DocumentPluginError>;
```

後者はDocument本体にまだ保存口のないParamDriverを含むcontract fixture用であり、ProjectV1の`HashMap<String, Value>` migration APIではない。

`prepare_plugins`はraw recipeをcloneし、versionを1段ずつ進め、現行contractで全parameterを検査する。成功時だけprepared recipeを公開する。raw Document bytes、revision、Undo、保存version、unknown `extra`は不変である。

状態の扱い:

| 状態 | `Document::validate()` | `prepare_plugins` | 実行 |
|---|---|---|---|
| contract欠落 | 構造が正しければ成功 | diagnostic | 不可 |
| future version | 成功 | diagnostic | 不可 |
| old + chain完全 | 成功 | prepared | executorがあれば可 |
| old + chain欠落／競合 | 成功 | diagnostic | 不可 |
| kind不一致 | 成功 | typed `DocumentPluginError` | 不可 |
| current + contract違反 | 成功 | typed `DocumentPluginError` | 不可 |
| contractあり／executor欠落 | 成功 | prepared | `ExecutorMissing` |

kind不一致をintrinsic errorから外すのは、kindの正本がcatalogへ移るためである。空ID、非有限値、AssetRef dangling、DocParam AST不正等は引き続きintrinsic errorである。

## 6. migration interpreter

`RenameParam`は`BTreeMap<String, DocParam>`のentryを移すだけで、`DocParam`内部を読まない。

```text
fromあり / toなし → 値をそのまま移動
fromあり / toあり → MigrationConflict
fromなし / toあり → versionとshapeの矛盾としてContractViolation
fromなし / toなし → versionとshapeの矛盾としてContractViolation
```

最後の2行を暗黙成功にしない。保存versionが旧版なのに旧keyがないrecipeを「既に移行済み」と推測すると、version正本を無効にするためである。

全stepはcloneへ適用する。途中で失敗したcloneは破棄し、部分preparedを返さない。default追加はmigrationではなく、現行`NodeDesc::resolve_params`相当のdefault解決で補う。unknown parameterとunknown `extra`は削除しないが、current contractで未知parameterが残れば実行可能とは判定しない。

## 7. 各入口の責任

| 入口 | intrinsic | catalog | executor | 処分 |
|---|---:|---:|---:|---|
| `Document::validate()` | 必須 | なし | なし | 名前／signature維持 |
| `load_document*` | 必須 | なし | なし | 低水準raw loadとして維持 |
| `save_document*` | 必須 | なし | なし | unknown保持用raw saveとして維持 |
| journal recovery内部 | 必須 | なし | なし | recovery中にplugin codeを呼ばない |
| `open_project_resolved` | 必須 | 必須 | 任意 | 製品openの新入口 |
| `DocumentWriter` | 必須 | 必須 | なし | constructorへcatalog追加 |
| `build_document_frame_graph` | 済 | 必須 | 必須 | `PluginRuntime`を必須化 |
| `export_document_video` | 済 | 必須 | 必須 | `ExportJob`へruntime追加 |

製品open:

```rust
pub struct ResolvedOpenProjectOutcome {
    pub recovered: OpenProjectOutcome,
    pub plugins: PreparedDocumentPlugins,
}

pub fn open_project_resolved(
    document_path: &Path,
    limits: &ResourceLimits,
    catalog: &PluginCatalog,
) -> Result<ResolvedOpenProjectOutcome, ProjectError>;
```

既存`open_project`／`open_project_with_limits`はjournal／recovery testとrepair toolの低水準入口として残すが、製品composition rootからの利用をAST gateで拒否する。openはcontract欠落等のdiagnosticを返して成功できる。install、network、build、executor呼出はしない。

Writer:

```rust
impl DocumentWriter {
    pub fn new(
        doc: Document,
        catalog: Arc<PluginCatalog>,
    ) -> Result<Self, DocumentPluginError>;

    pub fn with_undo_limits(
        doc: Document,
        catalog: Arc<PluginCatalog>,
        live_limit: UndoLimit,
        restart_limit: UndoLimit,
    ) -> Result<Self, DocumentPluginError>;
}
```

Writerはunknown／future／migration不可recipeを保持したまま構築できる。無関係な編集も許す。既知recipeを作成・変更するcommandだけは、適用後cloneをcatalogで検査し、失敗時Document全文不変とする。

graph／export:

```rust
pub fn build_document_frame_graph(
    doc: &Document,
    eval: EvaluationTime,
    desc: FrameDesc,
    data_tracks: &DataTracks,
    runtime: &PluginRuntime,
    project_root: Option<&Path>,
) -> Result<DocumentFrameGraph, GraphError>;

pub struct ExportJob<'a> {
    pub doc: &'a Document,
    pub runtime: &'a PluginRuntime,
    /* 既存field */
}
```

`export_document_video`内部でreference registryを生成しない。composition rootが明示的にcatalogとexecutorを集約する。preparedでない、またはdiagnosticが一件でもある使用recipeは型付き拒否する。

## 8. 旧APIの処分

| 現行 | A0I後 |
|---|---|
| `known_plugin_param`／`known_plugin_ids` | plugin entry分を削除。Document built-in／core slot制約だけ別名で残す |
| `KnownPluginInfo`／`known_plugin_info` | plugin entry分を削除。`doc.layer_source.rect`だけDocument built-in表へ移す |
| `DocPluginKind` | 削除し`motolii_plugin::PluginKind`を使用 |
| `PluginOpenWarning`／`PluginDegradation` | `PluginDiagnostic`へ置換 |
| `Document::plugin_open_warnings()` | `prepare_plugins()`のdiagnosticへ置換 |
| `migrate_plugin_params` | A0I-1ではdeprecated。A2で削除し、旧CLI互換は[A2S](2026-07-17-vism-a2-legacy-project-migration-decision.md)のprivate declarative adapterへ移す |
| 裸の`PluginRegistry`をgraphへ渡す | `PluginRuntime`へ置換 |
| export内`register_reference_plugins` | composition rootへ移動 |

`NodeDesc::resolve_params(HashMap<String, Value>)`はexecutor直前の値解決として維持する。Document側contract検査の代替には使わない。

## 9. 実装を三つへ分ける

A0Dの「A0Iを1 ticket」はcall site監査後には大きすぎる。意味順を保った3 ticketへ改訂する。各ticketは1 commitである。

### VSM-A0I-1 — contract／catalog／runtime

- §3、§4を`motolii-plugin`へ実装。
- reference contractを全executorについて登録。
- Document、graph、exportのsignatureはまだ変えない。
- duplicate、domain、migration plan、executor-only、kind／version／desc不一致の拒否fixture。

完了条件: `cargo test -p motolii-plugin`、既存purity、workspace全緑。

**完了**: `cb2c9a7`。`PluginContract`、immutable catalog、validated runtime、reference contractと拒否fixtureを実装した。

### VSM-A0I-2 — Document prepared resolution

- §5、§6を`motolii-doc`へ実装。
- plugin entryの`param_expect`／known table／open warning mirrorを削除。
- `Document::validate()`をintrinsicだけへ純化。
- `open_project_resolved`とcatalog保持Writerを追加。
- Sine rename、raw bytes／revision不変、unknown／future／chain欠落／conflict／kind違いをfixture化。

完了条件: M2 D1f/D6のopen／round-trip意味を維持、`cargo test -p motolii-doc`、workspace全緑。

**完了**: `e4f42c6`。clone-only prepared resolution、catalog保持Writer、resolved open、degraded分類とraw不変fixtureを実装した。

### VSM-A0I-3 — 製品実行入口

- graphを`PluginRuntime`必須へ変更。
- exportの内部reference registry生成を削除。
- export、CLI、製品openのcomposition rootをcatalog必須へ変更。
- contract-only／executor missing／degraded export拒否をfixture化。
- 旧製品入口を使うcall siteをAST gateで拒否。

完了条件: D3／D6／CLI E2E、`cargo test --workspace`全緑。

**完了**: `057e2e9`。graph／ExportJob／製品CLIをruntime必須化し、export内部registry生成と製品raw openを拒否するfixtureを実装した。

A1は[A1S公開crate境界仕様](2026-07-17-vism-a1-public-crate-boundary-spec.md)と移動前pixel基線VSM-A1Gの完了まで開始しない。A2はA1後とし、中央Sine migrationの削除を担当する。

## 10. A0I拒否fixture

最低限、次を自動化する。

1. duplicate contract ID。
2. 非有限／逆転domain、domain外default、F64以外のdomain。
3. 非隣接／重複／不正rename plan。
4. executor-only、kind／version／NodeDesc不一致。
5. contract-only headless検査成功。
6. Opacity `amount=-0.01`／`1.01`をdoc側ID matchなしで拒否。
7. Sine v1 `amp`→v2 `amplitude`がcloneだけを変更。
8. Sine `amp`＋`amplitude` conflict、旧versionなのに両方無いshape矛盾。
9. chain欠落、future、contract欠落、executor欠落を別分類。
10. unknown pluginを含むopen／saveの意味保持と無関係編集成功。
11. kind不一致はcatalogありでtyped hard error。
12. graph／exportが裸registryではコンパイル不能。
13. export内部でreference registryを生成していないAST gate。
14. `doc.layer_source.rect`がcatalog entryでないことと、ID衝突拒否。
15. migration成功／失敗の両方でraw Document bytes、revision、Undoが不変。

golden画像の期待値は変更しない。Opacity／Sineの既存意味を変える必要が出た場合はA0Iを止め、仕様へ戻る。

## 11. 非目標

- `.vism`のpackage／entry identityを`PluginId`で決定しない。
- manifest、container、署名、install store、version solverを作らない。
- arbitrary migration callback、WASM、Rust function pointerをcontractへ置かない。
- old recipeの永続upgrade commandを作らない。
- parameter UI hint、単位、slider幅をcontractへ置かない。
- input port、provider、Kit、BPM gridをA0Iへ混ぜない。
- `doc.layer_source.rect`を見かけ上plugin化しない。

## 12. 審判

A0Sが固定するコアは「表現の種類」ではなく、未知の表現を壊さず保持し、知っている表現だけを宣言で検査し、実行可能な表現だけを明示的に実行する三段境界である。

```text
保存できる ≠ 意味を検証できる ≠ このHostで実行できる
```

この不等号を型で消せないことが、Vismへ進む前の最小条件である。
