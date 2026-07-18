# VSM-A0D — plugin契約とmigrationの所有決定

作成日: 2026-07-17

状態: **設計決定／コード変更なし**。本書はfirst-party pluginの既知表とmigrationをどこへ置くかを決める。Rust型、serde形式、Vism manifest、動的loaderは未実装である。公開型と呼出境界は[VSM-A0S](2026-07-17-vism-a0s-contract-catalog-spec.md)で仕様化済みで、実装はVSM-A0I-1〜3を通すまで開始しない。

関連文書: [VSM-A0 inventory](2026-07-17-vism-a0-plugin-boundary-inventory.md)、[VSM-A7 BPM→DataTrack spike](2026-07-17-vism-a7-bpm-datatrack-spike.md)、[Vism実装計画](2026-07-17-vism-implementation-plan.md)、[M2恒久焼き込み予防](2026-07-12-m2-permanence-prevention.md)

## 1. 決定

pluginの責任を四つへ分ける。

| 責任 | 正本の所有者 | 含むもの | 含まないもの |
|---|---|---|---|
| **Document recipe** | `motolii-doc` | plugin entry ID、保存version、`DocParam`列、unknown payload、Project instance identity | 現行plugin version、実行関数、install状態 |
| **Plugin Contract** | plugin作者 | entry ID、kind、現行version、parameter型／値域、連続migration宣言 | Document AST、UI code、GPU executor、package署名 |
| **Contract Catalog** | Host composition root | contract集約、ID一意性、version chain検査、lookup、診断 | 契約意味の手書きmirror、Documentへの自動install |
| **Executor Registry** | Host runtime | Filter等の実行実装、GPU dispatch | 永続契約の正本、Document mutation、migration意味 |

**Plugin ContractとExecutor Registryを分離する。** 実行コードが無いheadless readerでもcontractだけで検査でき、contractが無い作品もunknown recipeとして保持できる構造にする。

plugin作者はparameterとmigrationの**意味**を所有する。Hostはそれを集約し、順序検査し、Document原本ではなく一時的な解決結果へ適用する。Documentはplugin欠落中もraw recipeを失わない。

```text
Document raw recipe（正本・不変）
       │
       ├─ contractなし ─────────────→ preserve + degraded
       │
       └─ contractあり
            ├─ current version ─────→ validate → prepared recipe
            ├─ old + chainあり ─────→ clone → migrate → validate → prepared recipe
            └─ future/chain欠落/失敗 → preserve + degraded

prepared recipe
       ├─ executorあり ─→ preview/export
       └─ executorなし ─→ execution unavailable
```

Project openはinstall、network、build、任意plugin code実行を起こさない。catalogの純データ検査と、Host実装の宣言的migration interpreterだけを許す。

## 2. 四案の比較

| 案 | 長所 | 破れるもの | 判定 |
|---|---|---|---|
| A. 現行`known_plugin_*`と中央`migrate_plugin_params`を維持 | 最小変更、現行testがある | 新pluginごとにHost／Document直書き。first-party特権を固定 | **棄却** |
| B. `PluginRegistry`を唯一の正本にする | mirrorを消せる | executable不在では検査不能。open時code availabilityと永続意味が結合 | **棄却** |
| C. build時にregistryからDocument表を生成する | 手書き乖離を減らす | Host再build前提、contractとexecutorが結合。第三者境界にはならない | **縮小採用**: parity artifact／CIには使えるが正本にしない |
| D. immutable Contract CatalogとExecutor Registryを分離 | missing保持、headless検査、first-party無特権、将来Vismへ追加的 | 新しい公開contractとvalidate接続の仕様が必要 | **採用** |

runtime registryをDocumentへ直接注入する案を採らないのは、pluginの「知っている」と「実行可能」を同義にしないためである。契約だけを読むHost、実行器だけが一時的に壊れたHost、pluginが欠落したProjectを区別する。

## 3. parameter契約の分担

現行`NodeDesc`は型とdefaultを持つ一方、`motolii-doc::param_expect`は値域とDocParam source規則を持つ。これを丸ごとplugin側へ移さない。

### plugin作者が宣言する

- parameter ID
- `ValueType`
- default
- 値そのもののdomain: 最小値、最大値、整数性等
- version間でのparameter migration

### Documentが固定する

- Const／Keyframes／Data／Vec2Axes／LookAt／Followという保存sourceの構造
- keyframe identity、補間、finite、AssetRef解決
- LookAt／Followを置けるDocument slot
- unknown pluginでは構造検査だけを行う規律

plugin contractは`DocParam`や`LayerId`へ依存しない。たとえばOpacity作者は`amount: F64, 0..=1`を宣言するが、それがConstかDataTrackかKeyframesかを処理しない。Hostが各leaf／fallbackへ値domainを適用する。

Colorのstraight sRGB／0〜1意味はDocumentと評価の既決契約であり、pluginごとに再宣言しない。UIのslider幅、単位表記、widget hintはsemantic value domainと別で、M3-U4a／GAP-13の判断前にcontractへ混ぜない。

## 4. migration契約

### 4.1 所有

- 作者: `version N → N+1`の意味と旧fixture。
- Host catalog: stepの連続性、一意性、循環／飛び越し／重複の拒否。
- Host interpreter: cloneへの適用、全step成功後だけprepared recipeを公開。
- Document: raw version／params／extraを変更せず保持。
- Executor: current schemaだけを受け取り、migrationを行わない。

### 4.2 v1の最小migration言語

最初に許す操作は**parameter keyのrenameだけ**とする。

```text
RenameParam {
  from: "amp",
  to: "amplitude",
}
```

Hostが`BTreeMap<String, DocParam>`の値を不透明なまま移すため、Const、Keyframes、Data、Vec2Axes等の内部をplugin codeへ見せない。Sine v1→v2はこの一操作で表現できる。

規則:

1. stepは必ず隣接version `N → N+1`。
2. 入力versionとparameter shapeの組を検査し、versionとshapeが矛盾する場合は拒否する。暗黙の冪等成功にしない。
3. `from`と`to`が同時にある場合はtyped conflict。片方を捨てない。
4. unknown parameter／unknown extraを削除しない。
5. default追加はmigrationにしない。現行descriptorのdefault解決を使う。
6. parameter削除、任意Rust callback、JSON script、値変換はv1 migration言語へ入れない。
7. 画素意味が変わる変更は既存IDのmigrationで隠さず、新entry ID／legacy executorを第一選択にする。

renameだけで表せない実例が出た場合は、そのfixtureを先に追加してmigration algebraを追加的に解凍する。一般scriptを先に許して将来のProject openで任意codeを実行する構造にはしない。

### 4.3 Documentを自動書換えしない

過去versionに完全なmigration chainがあっても、openやpreviewのためにDocument正本を自動更新しない。

1. raw recipeをcloneする。
2. cloneへmigration chainを適用する。
3. current contractで検査する。
4. prepared recipeとしてexecutorへ渡す。
5. raw recipe、revision、Undo、保存bytesは変更しない。

将来Inspectorで旧recipeを編集するときの永続upgradeは別のtyped Document commandで行い、1 Undo、失敗時変更ゼロ、旧recipe復元を満たす必要がある。A0IやA1へ便乗実装しない。

## 5. 状態行列

| contract | executor | saved version | open／save | full validation | preview／export |
|---|---|---:|---|---|---|
| なし | なし | 任意 | 構造検査、原本保持、unknown警告 | 不可 | 不可、export拒否 |
| あり | なし | current | 原本保持 | 可 | execution unavailable、export拒否 |
| あり | あり | current | 原本保持 | 可 | 可 |
| あり | あり | old、chain完全 | 原本保持。cloneをpreparedへ移行 | current contractで可 | 可 |
| あり | あり | old、chain欠落／失敗 | 原本保持、migration unavailable | 不可 | 不可、export拒否 |
| あり | 任意 | future | 原本保持、future警告 | 構造のみ | 不可、export拒否 |
| kind不一致 | 任意 | 任意 | typed hard error、原本不変 | 不可 | 不可 |

contractとexecutorのID／kind／current versionが一致しないHost構成はstartup時にtyped errorとし、Projectごとのdegraded状態へ流さない。

「contractあり、executorなし」を独立させることで、indexer、lint、asset explorer、互換HostがGPU実装をlinkせず作品を理解できる。逆にexecutorだけがありcontractが無い登録はHost構成エラーである。

## 6. 診断

現行のUnknown／Futureに最低限次を加える必要がある。ただしenum名はA0Sで確定する。

- contract missing
- executor missing
- migration step missing
- migration conflict／failure
- contract／executor mismatch

診断はopen可能性とexport可能性を別々に返す。missingやmigration失敗をdefaultで評価して「それらしい絵」を出さない。

## 7. Host組込み表現

`doc.layer_source.rect`はregistry pluginではなく、Document graphへ組み込まれたlegacy sourceである。これを外部plugin contract catalogへ偽装しない。

- 組込みDocument表現の契約はDocument側に残す。
- plugin contractと同じID namespaceを使う間は重複をHost startupで拒否する。
- 将来rectを本物のpluginへ移すなら、別のmigration／pixel同一PRで処分する。

これにより`known_plugin_info`からplugin集合を外しても、Document built-inの既知表まで無理に一般化しない。

## 8. A0S／A0I — 仕様と実装を分ける

A0Dは所有を決めたが、現行`Document::validate()`、save、open、exportのsignatureを黙って変更してよい許可ではない。[VSM-A0S](2026-07-17-vism-a0s-contract-catalog-spec.md)でM2／plugin仕様を改訂済みであり、以後はVSM-A0I-1〜3で一対一に実装する。

### VSM-A0Sの必須成果

1. UI／serde／wgpu非依存のimmutable plugin contract型。
2. parameter value domainの最小型。DocParam source／widget hintを含めない。
3. declarative adjacent-version `RenameParam` plan。
4. Contract CatalogとExecutor Registryの整合validator。
5. raw recipeを変更しないprepared resolution API。
6. §5全状態のdiagnostic。
7. `Document::validate`／save／open／exportのどこがintrinsic structural validation、どこがcatalog validationかを仕様表で確定。
8. `doc.layer_source.rect`をDocument built-inとして残す境界。
9. 追加／変更する公開signature、旧APIの互換処分、crate依存方向。
10. A0Iを1 ticket=1 commitへ収める実装順と拒否fixture。

VSM-A0SではRust型、test、production codeを変更していない。call site監査の結果、A0Iを1 ticketへ収める案は棄却し、contract/runtime、Document resolution、製品実行入口の3 ticketへ直列化した。型名、signature、旧API処分、拒否fixtureはA0Sを正本とする。

### VSM-A0I-1〜3の自動完了条件

- Opacity contractの`amount 0..=1`をdoc側ID matchなしで拒否できる。
- Sine v1 `amp`をclone上だけ`amplitude`へrenameし、raw Document bytes／revision不変。
- both-key conflict、step欠落、future、contract欠落、executor欠落が別diagnostic。
- contractだけのheadless検査がGPU executorなしで通る。
- executorだけの登録、ID／kind／version不一致、duplicate contractがstartup時に赤。
- current reference registry全件と現行Document期待表のparity fixture。
- unknown pluginのopen/save round-tripとdegraded export拒否を維持。
- `cargo test --workspace`全緑。

A0IはDocument永続shapeを変更しない。A0Sで採択された公開signatureと検査責任だけを実装し、実装PRで新しい意味やfieldを発明しない。

## 9. A1／A2の発注条件

### VSM-A1 Opacity

A0S／A0I完了後に発注する。

- 外部workspace crateがOpacity executorとcontractを公開境界だけで提供。
- `core.filter.opacity`、version 1、pixelを不変。
- `amount 0..=1`は外部crate所有contractが正本。`known_plugin_param`へのID追記なし。
- Host composition rootはcontractとexecutorを明示登録。
- envelope opacityの内部参照も同じcatalog／registryを通る。
- private Motolii crate、Slint、OS／vendor API依存を拒否。

### VSM-A2 Sine

A1後に発注する。

- 外部workspace crateがSine executor、contract、v1→v2 rename planを提供。
- 中央`migrate_plugin_params`のSine ID matchを削除。
- old `amp` recipeはprepared cloneだけが`amplitude`へ移り、raw recipe不変。
- migration欠落時はdegradedで開け、exportを拒否。
- `ProjectV1` CLI専用migrationとDocument recipe migrationを混同せず、旧CLI fixtureの処分を明記。

## 10. 非目標

- `.vism` manifest／container／署名を決めない。
- contractをDocumentへ埋め込まない。
- plugin欠落時にcontractをnetwork取得しない。
- arbitrary migration callbackやscriptを許さない。
- parameter UI hintをsemantic domainへ混ぜない。
- consumer input port、BeatEvents、Kit schemaを追加しない。
- old recipeの永続upgrade commandをA0S／A0Iで実装しない。
- plugin contract catalogをglobal mutable singletonにしない。

## 11. 審判

この決定は「pluginが自分で全部する」設計ではない。作者が意味を宣言し、Hostが安全に解釈し、Documentが原本を保持する。

```text
作者の自由
  = parameter意味とversion進化を宣言できる

Hostの責任
  = 実行前検査、transaction、診断、欠落時停止

Documentの責任
  = pluginが無くても作品を壊さない
```

first-party無特権とは、Hostが責任を放棄することではない。OpacityやSineも第三者と同じcontract登録、同じmissing lifecycle、同じmigration interpreterを通ることである。
