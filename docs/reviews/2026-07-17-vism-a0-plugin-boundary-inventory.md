# VSM-A0 — 現行plugin境界inventory

作成日: 2026-07-17

状態: **VSM-A0調査完了／コード・公開API・Document schemaの変更許可ではない**。本書は現行の静的plugin境界を、Vism実装前のコード事実として固定する。`.vism` package、loader、manifest、typed port、Kit schemaをここでは設計しない。

関連文書: [Vism実装計画](2026-07-17-vism-implementation-plan.md)、[Vism-ready反対側レビュー採否](2026-07-17-vism-ready-counter-review-disposition.md)、[Vism / Kitモデル](../vism-kit-model.md)、[プラグイン作者向け規約](../plugin-authoring.md)

## 1. 結論

現行実装は「Vismが将来包む実行核」の一部をすでに持つが、第三者配布境界ではない。

```text
作者コード
  └─ motolii-plugin::reference 内の静的singleton
       └─ register_reference_plugins()
            └─ PluginRegistry<&'static dyn Plugin>
                 ├─ Document graph → RenderStep::Plugin → GPU dispatch
                 └─ ProjectV1 ParamDriver → DataTrack → ParamSource::Data

Document load / validate
  └─ motolii-doc::param_expect の手書き既知表
       ├─ plugin id / kind / current version
       └─ parameter type / constraint

Document export
  ├─ 手書き既知表によるmissing / future判定
  ├─ degradedならexport拒否
  └─ fresh PluginRegistryへ参照pluginを再登録して評価
```

既に使えるものは、静的登録、種別別dispatch、自己記述parameter、GPU texture境界、purity検査、未知／未来版の保持とexport拒否である。一方、外部作者を塞いでいる継ぎ目は次の五つに集約できる。

1. plugin identityと実装参照が`'static`で、登録集合はHost側関数へ直書きされる。
2. migrationは中央関数が具体plugin IDをmatchし、一般Document loadには接続されていない。
3. Documentの既知契約はruntime descriptorと別の手書き表である。
4. ParamDriverは値を生成できるが入力portを持たず、汎用provider→consumer接続はない。
5. `NodeDesc`はparameterと無名のtexture arityを記述するが、typed port、resource要求、capability、migration、diagnostic契約を記述しない。

したがって次はVism package実装ではなく、現行BPMを既存DataTrack結線だけで試すVSM-A7と、migration／Document既知表の正本を決めるVSM-A0Dである。

## 2. 公開境界の全inventory

### 2.1 共通型

| 公開要素 | 現行の意味 | 現在の所有者 | Vism前の判定 |
|---|---|---|---|
| `PluginId(pub &'static str)` | `vendor.kind.name`形式の実行入口ID | `motolii-plugin` | 静的registryには十分。package／entry／instance identityではない |
| `PluginKind` | Input、LayerSource、Filter、ParamDriver、Composite、Simulation、ScriptWasm | `motolii-plugin` | 実行分類。Input／Simulation／ScriptWasmは予約で登録口なし |
| `ValueType` / `Value` | F64、Vec2、Vec3、Color、AssetRef | plugin / eval | parameter型として実装済み。structured eventやdomain型ではない |
| `ParamDef` | id、型、default | plugin実装 | 最小parameter schema。範囲、単位、widget hint、可視条件は持たない |
| `NodeDesc` | id、version、表示名、category、tags、params、入力枚数 | plugin実装 | discoveryとparameter解決の核。出力型、名前付きport、resource、migration、capabilityは持たない |
| `ResolvedParams` | `NodeDesc`で検査・default補完された値 | Hostが構築 | 型付きaccessorあり。keyは`&'static str` |
| `PluginError` | descriptor、render、migration、parameter、時刻の失敗 | plugin API | 型付きだが共通diagnostic code、severity、recovery情報はない |
| `PluginRegistry` | kind別の静的trait object集合 | 各Host呼び出し元 | 全ID一意、descriptor検査、列挙可能。発見・install・version併存はない |
| `DynPlugin` | registry列挙用の四種union | plugin API | purity一括検査に利用。予約kindは表現できない |

`PluginId`、`ParamDef.id`、`ResolvedParams`のkey、登録trait objectがすべて`'static`なのは、現在の同一binary内singletonに適した実装である。これをVismの永続identityや動的load方式として昇格してはならない。

### 2.2 trait別の入出力と責任

| trait | 入力 | 出力 | 時間／品質 | resource | 現在の不足 |
|---|---|---|---|---|---|
| `FilterPlugin` | texture 1枚、params、`RenderCtx` | texture 1枚 | `t`、`Quality`、予約instance/lookbehind/temporal footprint | `GpuCtx`、`PipelineCache`、encoderをHostから借用 | texture portは無名。resource要求とcache寄与を宣言しない |
| `CompositePlugin` | texture 2枚以上、params、`RenderCtx` | texture 1枚 | Filterと同じ | Filterと同じ | 可変arityは枚数だけ。port roleや型がない |
| `LayerSourcePlugin` | params、camera、個別の`t` | texture 1枚 | `RenderCtx`ではなく`t`直渡し。`Quality`なし | Filterと同じ | 他のrender traitとcontext形が不統一。Draft/Final契約を直接読めない |
| `ParamDriverPlugin` | params、start、duration、sample rate | `DataTrack<Value>` 1本 | 区間を一括生成 | GPU resourceなし | 入力portなし、出力型のdescriptorなし、複数track／structured eventなし |

全traitは`Send + Sync`と`&self`を使い、状態をHostへ隠す口を持たない。Simulationはenum予約と設計文書だけで、trait／StateTrack実コードはまだない。

`GpuCtx`、`PipelineCache`、`wgpu::CommandEncoder`、`wgpu::Texture`は公開render契約へ直接現れる。これは「見せるGPUはwgpu/WGSLのみ」という現行規律には適合するが、将来のsource／WASM／native payload分類や権限宣言を表すresource modelではない。

## 3. 参照registryの六実装

`register_reference_plugins`が製品Hostへ登録する集合を全件分類する。

| ID | kind / version | parameter | 入力→出力 | migration | 現在の製品経路 | 処分 |
|---|---|---|---|---|---|---|
| `core.filter.clear` | Filter / 1 | `color: Color` | texture 1→1 | なし | Document Effectから一般Filter経路で到達可能 | 境界fixture。独立Vism候補とは未決 |
| `core.filter.tint` | Filter / 1 | `color: Color` | texture 1→1 | なし | Document Effectから到達可能 | 実用Filterの既存証拠 |
| `core.filter.opacity` | Filter / 1 | `amount: F64` | texture 1→1 | なし | Document Effectに加えenvelope opacityの内部実装がIDで直接参照 | VSM-A1候補。ただしHost内部利用の依存を保つ必要あり |
| `core.layer_source.clear` | LayerSource / 1 | `color: Color` | 0→texture 1 | なし | `Document::ClipSource::Plugin`でこのIDだけ明示許可 | 最小source fixture |
| `core.param.sine` | ParamDriver / 2 | `amplitude`、`frequency_hz`、`offset` | 0→`DataTrack<F64>`相当 | v1 `amp`→v2 `amplitude` | 旧`ProjectV1` CLIの`param_drivers`だけ。Document Effect graphには入らない | VSM-A2候補。A0D前は外部化しない |
| `core.composite.clear` | Composite / 1 | `color: Color` | texture 2以上→1 | なし | render dispatchとテストのみ。Document graphはplugin Compositeへ二入力を構築しない | 境界fixture。製品接続済みと称さない |

`motolii-nodes`のnormal compositeも`CompositePlugin`を実装するが、参照registryには登録されず、Documentの通常合成は専用`RenderStep::Composite`を使う。従って「Composite traitがある」と「一般Composite pluginを作品から利用できる」は別の証拠である。

### 3.1 実行経路の非対称

- Filter: `EffectDefinition.plugin_id`をregistryで解決し、一入力`RenderStep::Plugin`へ変換する。
- LayerSource: Document graphは`core.layer_source.clear`だけを特別に許し、それ以外を`UnsupportedSourcePlugin`にする。
- Composite: render実行器は対応するが、Document graphのEffectは常に一入力なので、二入力Compositeの正規な生成口がない。
- ParamDriver: `motolii-cli::ProjectV1`が独自宣言を読み、export loop前にDataTrackを構築する。現行Document schemaのplugin評価経路とは別である。

この非対称はVSM-A3の表現選定と、将来のconformance matrixで明示的に扱う。traitが存在するだけで製品境界が完成したと判定しない。

## 4. 登録・保存・検査・移行の責任

### 4.1 登録

製品経路は必要な場所で空の`PluginRegistry`を作り、`register_reference_plugins`を呼ぶ。

- `motolii-export::export_document_video`
- `motolii-cli`のProject正規化とDataTrack構築
- render／doc／testkitの各テストfixture

登録集合を注入するapplication composition rootはまだ一つに集約されていない。exportは毎回fresh registryを作るため、別crateへ実装を移すだけでは製品へ参加できず、Host側の登録関数変更を必ず伴う。

### 4.2 Document保存と未知保持

Documentはplugin recipeを概ね次で保存する。

- source: `plugin_id`、`effect_version`、params、unknown `extra`
- effect: `EffectDefinition`にplugin ID、version、enabled、params、unknown payload
- use: `EffectUse`がdefinitionを安定IDで参照

未知IDと未来versionは開け、原本を保持し、警告を返す。exportは警告が一つでもあれば型付きで拒否する。このlifecycleはVism欠落時の下部契約として再利用できるが、install／resolve／再導入の仕組みではない。

### 4.3 Document既知表

`motolii-doc::param_expect`は次を手書きで持つ。

- `known_plugin_param`: ID×parameterの型と制約
- `known_plugin_ids`: reference registryと比較するID集合
- `known_plugin_info`: ID×kind×現行version
- `DocPluginKind`: plugin kindのDocument側mirror

`d1h_plugin_expect_table`がreference registryと表の乖離を検出するため、現在のfirst-party集合内では「忘れたらテストが落ちる」。ただし第三者pluginが自分の契約を登録できる境界ではない。

さらに、`param_expect.rs`のコメントは「motolii-docは本番コードでmotolii-pluginに依存しないためmirrorする」と説明するが、現行`motolii-doc/Cargo.toml`は`motolii-plugin`へ通常依存し、`graph.rs`も公開型を使用する。これはコメントまたは依存方針が古い証拠であり、A0Dでは現状の二重表を不可避な層分離として前提化しない。

`known_plugin_info`にはregistry外の組込み`doc.layer_source.rect`も存在する。一方、`known_plugin_ids`はreference registry六件だけである。したがって「Documentが知る表現」と「registry登録plugin」は既に同一集合ではない。

### 4.4 migration

`motolii-plugin::migrate_plugin_params`は具体ID `core.param.sine`をmatchし、v1からv2へのrenameを実行する。migrationをdescriptorやplugin実装が提供する登録口はない。

コード上の製品call siteは旧`ProjectV1` CLIのParamDriver正規化だけである。一般Documentのload／validate／graph構築はこの関数を呼ばない。従って現在証明されているのは「旧CLI ProjectのSineを移行できる」までであり、「保存済みDocument plugin recipeの一般migration境界」ではない。

A0Dは少なくとも次を比較する必要がある。

1. v1静的集合のまま、migration registryとDocument既知契約を一つのHost-owned catalogへ寄せる。
2. plugin実装がmigrationを提供し、Hostがinstall済み実装なしでも旧Projectを扱える補助情報を別に保持する。
3. 永続意味に効く既知表はDocument側へ残し、runtime descriptorから生成・検証するbuild-time artifactを導入する。
4. 現状を維持し、外部pluginを常にunknownとして保持するだけに留める、より小さいv1案。

決定条件は、plugin欠落中のload、未来版、downgrade拒否、旧Projectの移行、決定的validate、再導入復元、first-party無特権である。

## 5. DataTrackとBPMの現在地

現行の値接続は次だけである。

```text
ParamDriverPlugin
  → DataTrack { start, sample_rate, Vec<Value> }
  → DataTracks[DataTrackId(String)]
  → ParamSource::Data / DocParam::Data
  → 既存parameter評価
```

`ParamDriverContext`に入力trackや別plugin出力はない。`NodeDesc`にもDataTrack出力型の宣言はない。従って、BPM由来の拍を値列にして既存parameterへ渡すVSM-A7は可能だが、次を称してはならない。

- BeatEventsという新しい公開型
- provider Vismとconsumer Vismの接続
- consumer pluginのtyped input port
- Kit materialize

`Document.bpm`は正の有理数による固定BPMで、tempo map／meter mapではない。A7はこの意味を変更せず、`RationalTime`上の決定的な拍位置または値列をfixtureとして作る。公開enum、schema、migrationを追加しない。

## 6. UI、resource、diagnosticの現在地

| 領域 | 実装済み | 未実装／未決 |
|---|---|---|
| discovery | display name、category、tags | Asset Explorer／検索への製品接続 |
| parameter UI | `ValueType`とdefault | M3-U4a自動Inspector。範囲、単位、widget hint |
| custom UI | なし | v1では意図的に公開しない |
| GPU resource | Host所有`PipelineCache`を借りる | 宣言的resource要求、budget、権限、payload別sandbox |
| failure | `PluginError`、unknown/future warning、export拒否 | 安定diagnostic code、回復操作、plugin由来表示 |
| quality | Filter/Compositeの`RenderCtx.quality` | LayerSourceとのcontext統一、ParamDriver縮退契約 |
| state | render traitへの隠れ状態禁止 | SimulationPlugin／StateTrack実コード |

NodeDescから標準UIを作る方針は決定済みだが、製品パネルは未実装である。「全保存parameterをcustom UIなしで操作可能」はM3-U4aの完了条件であり、A1/A2のcrate分離だけでは証明できない。

## 7. 既にある審判は再実装しない

- `validate_node_desc`: ID形式、kind segment、version、表示metadata、parameter重複／default型、arity。
- registry全ID一意とkind別列挙。
- `motolii-testkit::purity`: 登録済みpluginの一括純関数検査。
- conformance: vendor／OS固有API、panic／unwrap等の禁止。
- scaffold: `scripts/new-plugin.sh`と生成物検査。
- GPUゴールデン／DataTrack値列テスト。
- Document未知／未来版round-trip、kind mismatch拒否、degraded export拒否。
- Document既知表とreference registryの乖離テスト。

Vism作業はこれらを捨てて新しいvalidatorを作るのではなく、将来のpackage／entry／artifact検査へ合成する。

## 8. 機械fixture案

VSM-A0のinventoryをコード変更へ昇格する場合は、次の一つの正規化snapshotで「登録したが分類を忘れた」を赤にする。A0時点では案に留める。

```text
InventoryRow {
  plugin_id,
  kind,
  version,
  params[(id, ValueType)],
  min_inputs,
  max_inputs,
  product_route,
  migration_owner,
  document_contract_owner,
}
```

完了条件:

1. `PluginRegistry::iter`でreference registry全件を列挙し、各IDがexactly onceでinventoryにある。
2. `PluginKind`と`DynPlugin`をexhaustive matchし、新kind追加時はcompile errorまたはfixture failureになる。
3. descriptor由来欄は手書きせず実値と比較する。
4. `product_route`等の意味欄は明示enumにし、空文字や「その他」を許さない。
5. Document既知表の余剰／欠落も同じ試験から検出するが、registry外組込み表現を別集合として許す。
6. snapshot更新だけで新pluginを「製品接続済み」にできない。routeの実行fixtureを別に要求する。

このfixture自体を恒久公開APIにせず、まずtest-only inventoryとして置く。Vism manifestのfieldをここから逆算しない。

## 9. 次の発注粒

### VSM-A7 — 完了

目的: 現行`Document.bpm`を変更せず、既存DataTrack→parameter接続だけで拍同期値を作れるか反証する。

必須条件:

- 固定有理BPM、開始時刻、sample rateから値列が決定的。
- frame rateとBPMが割り切れないfixtureを含む。
- seek順、Draft/Final、preview/exportで同じ時刻意味。
- `DocParam::Data`または既存`ParamSource::Data`以外のconsumer口を追加しない。
- Document byte意味、schema version、公開plugin traitを変更しない。
- 結果をBeatEvents、consumer Vism、Kitと呼ばない。

結果と自動fixtureは[VSM-A7 BPM→DataTrack意味spike](2026-07-17-vism-a7-bpm-datatrack-spike.md)に固定した。120.35 BPM×30000/1001 fps、forward／reverse seek、Draft／Final同値、Document bytes不変が緑である。

### VSM-A0D — 決定完了

目的: Sineを別crateへ移す前に、migrationとDocument既知契約の正本を決める。コードは触らない。

必須比較:

- §4.4の四案。
- plugin実装が欠落した状態でのload／validate／save。
- old／current／future version、downgrade、再導入。
- `doc.layer_source.rect`のようなHost組込み表現との境界。
- static v1の最小解と、Vism v2へ追加的に移れる解。

採否は[VSM-A0D contract／migration所有決定](2026-07-17-vism-a0d-contract-migration-ownership-decision.md)に固定した。immutable Contract CatalogとExecutor Registryを分離し、plugin作者がparameter／migration意味、Hostが集約／transaction、Documentがraw recipe保持を所有する。[VSM-A0S](2026-07-17-vism-a0s-contract-catalog-spec.md)で仕様を改訂済みで、コード化はVSM-A0I-1〜3で直列に行う。

### VSM-A1 — まだWAIT

Opacityの別workspace crate化はA0I完了後に発注する。移動時は、ID／version／pixel、envelope opacityの内部参照、contract catalog、export時登録、purity／conformanceを同時に保つ。単にsource fileを別crateへ移しただけではfirst-party無特権と判定しない。

## 10. A0の停止線

- `PluginId`をVism package identityへ流用しない。
- `NodeDesc`へmanifest、author、license、dependency、capabilityを足さない。
- ParamDriverへ入力portを足さない。
- Sine migrationをplugin本体へ移す実装を先行しない。
- Document既知表をruntime registryへ即置換しない。
- LayerSourceのcontext不統一をA0ついでに修正しない。
- Composite traitの存在を製品接続の証明にしない。
- `.vism` reader、archive、dynamic loaderを作らない。

A0が得た最大の成果は、Vismが「既存pluginを包むファイル形式」ではなく、登録、永続契約、migration、typed connection、製品投影という複数の責任を再配置する仕事だと、コードの継ぎ目から確定したことである。
