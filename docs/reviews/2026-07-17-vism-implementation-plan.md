# Vism実装計画 — 公開境界の反証から配布へ

作成日: 2026-07-17

状態: **実装ロードマップ案／Vism package実装は未許可**。名称 **Vism** と拡張子 **`.vism`** は決定済みだが、manifest、container、payload、署名、loader、install storeは未決である。本書はそれらをコードから発明せず、どの証拠をどの順で揃えて仕様へ昇格するかを定める。現行境界のコード監査は[VSM-A0 inventory](2026-07-17-vism-a0-plugin-boundary-inventory.md)に固定した。

関連正本: [Vismコンセプト](../vism-package-concept.md)、[Vism / Kitモデル](../vism-kit-model.md)、[Vism-ready反対側レビュー採否](2026-07-17-vism-ready-counter-review-disposition.md)、[プラグイン作者向け規約](../plugin-authoring.md)、[小さなコアと探索可能な拡張](../extensible-core-model.md)、[M2 Document仕様](../specs/M2-document-model.md)、[Simulationモデル](../simulation-model.md)

## 1. 結論

実装順は次の七段とする。

```text
既存静的pluginの監査
  → first-party無特権のコード実証
  → typed provider／Kitとidentity五層のfixture
  → package意味論のfixture
  → container／payload／trustの比較spike
  → .vism検査・導入・解決・実行
  → UIとheadless互換Host／forkで契約を反証
```

最初に`.vism` readerや動的loaderを作らない。先に作るのは、Vismが将来包むことになる**公開plugin境界の実証物**である。loaderを先に作ると、現在の`PluginKind`、`NodeDesc`、Rust ABI、単一entryという偶然がpackage形式へ恒久化される。

v1では静的リンクを維持する。Vismのpackage／loaderはv2であり、前半のpre-Vism作業はv1〜v1.xのplugin品質を直接改善する。

## 2. 現在すでにあるもの

次は再実装しない。

| 能力 | 現在の証拠 | Vism計画での扱い |
|---|---|---|
| Host内部の実行分類 | `PluginKind`とFilter／LayerSource／ParamDriver／Composite trait | package identityとは分離して利用 |
| 自己記述parameter | `NodeDesc`、`ParamDef`、`validate_node_desc` | expression contractのpre-Vism証拠 |
| 作者の型紙 | `scripts/new-plugin.sh`、`scripts/new_plugin.py` | 新規参照実装の入口 |
| 純関数審判 | `motolii_testkit::purity` | Vism conformanceへ後で合成 |
| ベンダー／panic禁止 | `motolii-plugin/tests/conformance.rs` | package形式に関係なく維持 |
| LLMによるFilter実演 | INF-7g `core.filter.opacity` | 同じFilterを作り直さない |
| 未知／未来版の保持 | M2-D1f | Vism欠落lifecycleの下部証拠 |
| degraded時のexport拒否 | M2-D6 | 黙ったfallbackを作らない |
| Simulationの責任分離 | `SimulationPlugin + StateTrack`設計、`PluginKind::Simulation`予約 | 実コード証拠はSIM-1待ち |

一方、現在の証拠だけでは次を言えない。

- first-party実装が`motolii-plugin`クレートの外から、公開APIだけで成立するか。
- Vismが具体providerを知らず型付きinputだけを要求できるか。
- 複数VismをKitがpreflightし1 macroでmaterializeできるか。
- 一つのVismと、一つ以上のcapability entryのidentityをどう分けるか。
- `PluginId`、Vism package identity、Project内instance identityの対応。
- `NodeDesc`のどの情報が表現契約で、どれがMotolii Host固有投影か。
- source、WGSL、WASM、native binary、宣言的recipeのどれを同じ配布規則へ載せられるか。
- install前検査、依存解決、version併存、署名、build、権限の責任。

## 3. 五つのidentityを先に分ける

Vism実装の最初の意味課題はschemaではなく、次の三層を混ぜないfixtureである。

| identity | 寿命 | 例 | 所有者 |
|---|---|---|---|
| **package identity** | 配布・更新をまたぐ | `org.example.beat-glow` | Vism作者／配布系 |
| **capability entry identity** | Hostが実行入口を選ぶ | `filter.glow`、`driver.beat` | Vismの表現契約 |
| **Kit identity** | 接続構成の配布・更新をまたぐ | `org.example.music-reactive-kit` | Kit作者／配布系 |
| **Project instance identity** | 一つの作品内で複製・Undo・参照をまたぐ | Effect Use、Generator instance | Project Document |
| **artifact identity** | build、検証、署名をまたぐ | content hash／provenance | build／trust系 |

`PluginId(pub &'static str)`をそのままVism identityと宣言しない。`'static`は現行静的registryの実装事情であり、配布意味ではない。また、instance identityをpackage versionやTimeline上の並び順から導出しない。

最初のfixtureは最低でも次の三ケースを比較する。

1. 一つのVismがFilter entryを一つ持つ。
2. consumer Vismが`BeatEvents`相当の型だけを要求する。
3. Kitがprovider Vismとconsumer Vismを選んで接続し、Projectへmaterializeする。
4. fork Kitが同じconsumerへ別providerを接続する。

独立して差替えられるproviderとconsumerはKitで構成する。一package複数entryは同じlifecycle／compatibility責任から分離できない場合だけ比較する。ここで決める前にmanifestの`entries: []`等を実装しない。

上の2〜4は将来意味を比較するfixtureであり、現行`ParamDriverPlugin`に入力portがあるという記述ではない。現行コードで可能な最小接続はDataTrackを既存parameterが参照する形だけである。

## 4. 段階計画

### Phase A — pre-Vism公開境界を反証する

目的はpackageを作ることではなく、first-partyと第三者が同じ公開面で表現を作れることを証明すること。

| ID | 内容 | 依存 | 自動完了条件 | 状態 |
|---|---|---|---|---|
| VSM-A0 | 現行plugin APIのinventoryをfixture化。kind、入力、出力、parameter、resource、diagnostic、migration、UI投影、DataTrack結線の対応表を作る | なし | 全登録pluginに責任分類があり、`migrate_plugin_params`のSine直書き、doc側`known_plugin_*`／kind/version mirror、static registry、入力なしParamDriverを明示。未分類field／traitを検出できるfixture案 | **調査完了**: [inventory](2026-07-17-vism-a0-plugin-boundary-inventory.md)。test-only fixture実装は未発注 |
| VSM-A0D | first-party migrationとDocument既知契約表の所有処分を決める。コードは触らない | VSM-A0, GR-PV | migration登録／実行者、param制約の正本、欠落時保持、runtime registryとの依存方向、旧Project互換、より小さい静的v1案を比較し採否。未決ならA1/A2を開始しない | **決定完了**: [contract／migration所有](2026-07-17-vism-a0d-contract-migration-ownership-decision.md) |
| VSM-A0S | immutable Contract Catalog、value domain、declarative rename、prepared resolution、validate接続の仕様追補。コードは触らない | VSM-A0D, GR-PV | A0D §8のA0S必須成果、公開signature／旧API処分／依存方向／拒否fixtureが未決なし | **仕様完了**: [A0S](2026-07-17-vism-a0s-contract-catalog-spec.md) |
| VSM-A0I-1 | contract／catalog／runtime整合を`motolii-plugin`へ実装 | VSM-A0S | domain／migration plan拒否、duplicate、executor-only・kind/version/desc不一致拒否、contract-only許可、workspace全緑 | **完了**: `cb2c9a7` |
| VSM-A0I-2 | Document prepared resolutionとcatalog保持Writer | VSM-A0I-1 | known表撤去、raw不変rename、全degraded分類、unknown round-trip、無関係編集、workspace全緑 | **完了**: `e4f42c6` |
| VSM-A0I-3 | graph／export／製品openをcatalog/runtime必須化 | VSM-A0I-2 | 裸registry経路消滅、export内registry生成撤去、D3/D6/CLI E2E、workspace全緑 | **完了**: `057e2e9` |
| VSM-A1S | Opacity外部crate化の公開façade、依存allowlist、first-party composition root、必須capability、test配置を仕様化 | VSM-A0I-3 | [A1S](2026-07-17-vism-a1-public-crate-boundary-spec.md)に未決なし。コードは触らない | **仕様完了** |
| VSM-A1G | Opacity移動前pixel基線を端点・非一様入力へ強化 | VSM-A1S | amount 0/1/0.5、非単色premul、`MOTOLII_REQUIRE_GPU=1`でskip不可。実装はまだ移動しない | **完了**: 3 pixel case全緑 |
| VSM-A1 | `core.filter.opacity`相当を**別workspace crate**へ移し、公開`motolii-plugin` APIだけで登録・評価する | VSM-A1G | façade経由のみ、normal/dev/build allowlist、既存ID／version／A1G pixel一致、assembled purity、ID parity、Host必須capability検査 | **完了**: A1-1・A1-2・A1-3（façade公開面・first-party composition root・Opacity外部crate化） |
| VSM-A2 | ParamDriver参照実装を別workspace crate化する | VSM-A0I-3, VSM-A1 | Sine v1→v2をA0Iのdeclarative renameだけでprepared解決可能。raw recipe不変、同一`t`+入力で同一値、非有限入力の型付き拒否、private依存拒否 | VSM-A0I-3/A1待ち |
| VSM-A3 | LayerSourceまたはCompositeのうち、`Clear`より実用的な参照表現を公開APIだけで作る | VSM-A0 | VRAM常駐golden、Draft/Final同一関数、正準座標、private依存拒否 | 表現選定待ち |
| VSM-A4 | first-party無特権gateを共通化する | VSM-A1, VSM-A2 | first-party参照crateがHost内部crate／Slint／OS・vendor APIへ依存するとCI赤になる負例 | 仕様昇格待ち |
| VSM-A5 | kind横断の欠落／未来版／未知payload round-trip matrixを追加する | VSM-A1, VSM-A2, M2-D1f/D6 | open成功、byte意味保持、無関係編集成功、export型付き拒否、互換実装再導入後に評価復元 | M2再締結との調整待ち |
| VSM-A6 | L3状態系を公開境界だけで実証する | SIM-1 | StateTrackをHostが所有し、plugin欠落時もDocument recipe保持。render pluginに隠れ可変状態なし | SIM-1待ち |
| VSM-A7 | 現行`Document.bpm`から値列を導出し、既存`DataTrackId`→`DocParam::Data`結線でparameterを駆動する意味fixture。公開型・schemaは追加しない | VSM-A0 | 固定BPM→拍位置／値列がRationalTimeで決定的。既存param結線だけを使用し、「consumer Vism」や入力portを称さない。旧Project byte意味不変 | **完了**: [結果](2026-07-17-vism-a7-bpm-datatrack-spike.md)／test 3件 |

`VSM-A3`ではVism package都合の新traitを足さない。既存公開面で書けないなら、その失敗自体をGAPとして記録し、plugin contractの仕様改訂を先に行う。

Phase Aの出口:

- pixel Filter、値Driver、生成／合成、Host所有状態の少なくとも三種類で同じ規律が通る。
- first-party参照実装がHost private APIを使わない。
- 現行BPMを変更せず、既存DataTrack→parameter結線へ時系列意味を渡すfixtureがある。consumer pluginの成立はまだ公約しない。
- 「missingを保持できる」と「評価・exportできる」を別々にテストできる。
- ここまでは`.vism`ファイルを一つも生成しない。

### Phase B — typed provider、Kit、packageの意味をコードより先に固定する

| ID | 内容 | 依存 | 完了条件 | 状態 |
|---|---|---|---|---|
| VSM-B0 | package／entry／Kit／Project instance／artifactのidentity fixture | VSM-A1, VSM-A2 | §3の4ケースについてrename、update、duplicate、missing、reinstall、fork差替え後のidentity期待値が表になる | Phase A完了待ち |
| VSM-B1 | Vism／Kit／Preset／Asset／Bake／Projectの境界fixture | VSM-B0 | 各成果物の正本、持ち運び、欠落、更新、実行可否を分類。Project openでinstallが発生しない | Phase A完了待ち |
| VSM-B2 | provider→consumerとmaterialize Kitの**方式決定fixture** | VSM-A7, VSM-B0, VSM-B1 | (a)既存DataTrack→param、(b)入力portを持つconsumer plugin、(c)keyframe等を作るAuthoring Toolを比較し採否。(b)は公開API解凍、(c)はlive providerでないと明記。Kit identity採番、循環、欠落、展開後runtimeの意味表。コードは作らない | VSM-A7/B0/B1待ち |
| VSM-B2I | 採用方式のmaterialize Kit実装 | VSM-B2, M2-D2, M3-U9aまたは独立採択された同等のbatch preflight境界 | batch全体を開始snapshotへpreflight後だけ1 macro commit。途中command失敗、型不一致、循環、Cancel、staleでDocument／履歴変更ゼロ。現行`apply_command`逐次適用だけをatomic batchと称さない | **WAIT／atomic batch未実装** |
| VSM-B3 | logical manifest、version、migration、依存解決の意味表。まだ直列化しない | VSM-B0〜B2 | package／entry／Kit／payload versionを分離。各fieldの作者、検査者、互換影響、未指定時とdowngrade／併存／循環の拒否表 | VSM-B2待ち |
| VSM-B4 | payload classとfork capabilityを分類する | VSM-A3, VSM-A6, VSM-B2 | Declarative、WGSL、source+Host build、WASM、nativeとBase／Optional／Fork capabilityの可搬性・権限・再現性・DXを別評価 | VSM-A6待ち |
| VSM-B5 | headless compatible runnerでUI／Document漏れを反証し、ISF／OpenFX adapter範囲を判定する | VSM-B2, VSM-B4 | Motolii UI／Documentなしで最小provider→consumerを評価。非対応能力は型付き診断。import／adapterのloss表 | VSM-B4待ち |
| VSM-B6 | Phase Bの反対側レビュー | VSM-B0〜B5 | 事実、転移条件、より小さい形式、fork分断、供給網、安全性を独立判定し、P0/P1未解決0 | VSM-B0〜B5待ち |

VSM-B0〜B6の意味決定ではRust struct、serde schema、公開enumを作らない。表・fixture・候補データだけで反証し、`BeatEvents`、`KitDefinition`、作者名、license等を現行`NodeDesc`へ足さない。実コードは独立したVSM-B2I以降である。`NodeDesc`はHostの評価／UI記述、manifestは配布責任であり、重なるfieldがあっても同一型とは限らない。

### Phase C — container、payload、trustを隔離spikeで比較する

| ID | 内容 | 依存 | 自動完了条件 | 状態 |
|---|---|---|---|---|
| VSM-C0 | container候補比較spike | VSM-B6 | directory／archive等を同じ論理fixtureで読み、path traversal、symlink escape、重複名、巨大展開、破損、unknown fieldの負例が揃う | VSM-B6待ち |
| VSM-C1 | source+Host build再現性spike | VSM-B4, VSM-B6 | toolchain pin、offline build、cache key、Cancel、失敗rollback、同一入力artifact hashの保証範囲を測定 | VSM-B6待ち |
| VSM-C2 | WGSL／WASM／nativeのsandbox・権限spike | VSM-B4, VSM-B6 | filesystem/network/process/GPU上限、timeout、memory budget、診断を方式別に実測 | VSM-B6待ち |
| VSM-C3 | install前検査と署名／由来モデルspike | VSM-C0〜C2 | 機能正当性、互換性、供給網信頼、安全性を別結果として表示できるfixture | VSM-C0〜C2待ち |
| VSM-C4 | container／manifest／payload採否の反対側レビュー | VSM-C0〜C3 | 採用／縮小／延期／棄却を方式ごとに記録し、P0/P1未解決0 | VSM-C0〜C3待ち |

spikeは製品のplugin registry、Project loader、OS file associationへ接続しない。一時形式を`.vism`の正式形式として保存しない。

### Phase D — `.vism`を導入できる最小製品経路

このPhaseで初めてV2-1を仕様書へ昇格する。1タスク1PRを守り、parser、store、builder、loader、resolverを分離する。

| ID | 内容 | 依存 | 自動完了条件 | 状態 |
|---|---|---|---|---|
| VSM-D0 | 採用済みmanifest/containerの型とvalidator | VSM-C4、v2仕様凍結 | 正例round-trip、unknown保持、全VSM-C0負例拒否、resource limit | WAIT |
| VSM-D1 | transactional install store | VSM-D0 | stage→verify→atomic publish。Cancel/電源断模擬/同版衝突で旧install不変 | WAIT |
| VSM-D2 | dependency/version resolver | VSM-D0, VSM-D1 | version併存、循環、欠落、非互換、lock再現性のfixture | WAIT |
| VSM-D3 | payload build／compile pipeline | VSM-D1、採用payload別spike | untrusted入力を隔離し、artifact provenanceとcache keyを保存。失敗時に未導入へ戻る | WAIT |
| VSM-D4 | runtime loader adapter | VSM-D2, VSM-D3 | static参照実装と同じconformance corpusで結果一致。load/unload失敗がHostをpanicさせない | WAIT |
| VSM-D5 | Project resolver接続 | VSM-D2, VSM-D4, M2 migration仕様改訂 | Project openはinstall/executeしない。欠落保持、再導入復元、必要Vism不能時export拒否 | WAIT |
| VSM-D6 | Vism conformance bundle | VSM-D0〜D5 | manifest、capability、purity、GPU、migration、missing、resource、securityを一コマンドで判定 | WAIT |

`VSM-D5`だけがDocument恒久面へ接続し得る。着手前にGR-PV、M2仕様改訂、migration、旧reader拒否、意味論goldenが必須である。既存`plugin_id + effect_version + extra`で表現できるなら、Vism都合のfieldをDocumentへ足さない。

### Phase E — Host UIへ投影する

M3製品実装停止とV2-8のcustom UI延期を維持する。

| ID | 内容 | 依存 | 完了条件 | 状態 |
|---|---|---|---|---|
| VSM-E0 | installed Vism一覧／検索／診断snapshot | VSM-D2, VSM-D5, M3入場 | UI非依存のread-only snapshot。同期I/OやSlint型を公開しない | WAIT |
| VSM-E1 | Asset Explorer内のVism入口 | VSM-E0, M3-U3a/U4a | Vism名→preview→追加の導線。内部`PluginKind`を主ラベルにしない | WAIT |
| VSM-E2 | install確認とtrust／依存表示 | VSM-D1〜D3, VSM-E0 | Project openと別操作。由来、要求能力、build、権限、非互換理由を実行前に表示 | WAIT |
| VSM-E3 | missing／update／recovery UI | VSM-D5, VSM-E0 | 読む前に状態識別可能、無関係編集可、再導入で同instanceへ復元 | WAIT |
| VSM-E4 | NodeDesc由来の標準Inspector接続 | VSM-D4, M3-U4a | 全保存parameterをHost標準UIから操作可能。custom UIなしでも機能欠落なし | WAIT |

色は意味状態にのみ使い、marketplaceの華やかさを主役にしない。主役はVismがStageへ生む表現であり、install UIを制作画面の常設中心にしない。

### Phase F — 互換forkで能力進化を反証する

| ID | 内容 | 依存 | 完了条件 | 状態 |
|---|---|---|---|---|
| VSM-F0 | packageを読む独立headless互換Host | VSM-D0, VSM-D4, VSM-B5 | MotoliiのDocument／Slint／Timeline rowなしで最低1つのprovider→consumer Kitを評価 | WAIT |
| VSM-F1 | Base／Optional／Fork capability fixture | VSM-F0 | 同一時刻、入力、parameter、seed、Quality契約で許容差内一致。fork固有providerへ差替えてもconsumer不変。非対応は型付き診断 | WAIT |
| VSM-F2 | fork名前空間と上流昇格審判 | VSM-F1 | fork要求が暗黙分岐でなくmanifest要求として観測可能。既存Base意味を変更せず追加的に昇格できる | WAIT |

他製品のためにMotoliiの全機能を共通化しない。独立headless Hostは任意ソフト互換の証明ではなく、Motolii内部UI／Documentへの契約漏れとfork差替えを検出するfixtureである。

## 5. 依存と並列レーン

最短の安全な進行は次の通り。

```text
今:
  VSM-A0（調査完了）
    ├─ VSM-A0D ─ VSM-A0S（仕様完了）─ VSM-A0I-1 ─ A0I-2 ─ A0I-3 ─ VSM-A1/A2 ─ VSM-A4 ─ VSM-A5
    ├─ VSM-A3
    └─ VSM-A7（完了、既存DataTrack→paramだけ）

M4 K1/K7後:
  SIM-1 ─ VSM-A6

公開境界の実証後:
  VSM-A1/A2 ─ VSM-B0 ─ VSM-B1 ─┐
  VSM-A7 ────────────────────────┴─ VSM-B2（方式決定）─ VSM-B3
  VSM-A3/A6 ────────────────────────────────┴─ VSM-B4 ─ VSM-B5 ─ VSM-B6

atomic batch成立後:
  VSM-B2 + M2-D2 + U9a相当 ─ VSM-B2I

v2:
  VSM-C0/C1/C2 ─ VSM-C3 ─ VSM-C4
  VSM-D0 ─ VSM-D1 ─ VSM-D2/D3 ─ VSM-D4 ─ VSM-D5 ─ VSM-D6

M3再入場後:
  VSM-E0 ─ VSM-E1/E2/E3/E4

最小loader成立後:
  VSM-F0 ─ VSM-F1 ─ VSM-F2
```

Phase Aの仕様化とM2基盤再締結作業は並列に検討できるが、M2 Documentテストを変更する`VSM-A5`は再締結責任者と衝突確認する。Phase EはM3入場前に着手しない。Phase C以降はSIM-1を含む責任の異なる実plugin証拠とPhase B反対側レビューを飛ばさない。

## 6. LLM開発者向けの発注単位

LLMへは「Vismを実装する」と発注しない。未決を一度に埋めるため、container、manifest、loader、UIを勝手に束ねる危険が高い。

各Issueは次を必須にする。

1. 上表のIDを一つだけ持つ。
2. 触ってよいcrate／docsと、触ってはいけない恒久面を列挙する。
3. 入出力型、error、resource上限を仕様書に先に書く。
4. 正例だけでなく、欠落、未来版、unknown、Cancel、再導入の負例を与える。
5. 実行コマンドと期待結果を完了条件にする。
6. 新しい公開APIが欲しくなったらコードを止め、GAPと最小fixtureを返す。
7. golden更新で通さず、意味変更なら別の仕様／golden PRへ分離する。

Phase Aの標準提出コマンド候補:

```sh
cargo test -p motolii-plugin
cargo test -p motolii-testkit --test purity
cargo test --workspace
git diff --check
```

各参照plugin crate固有testは、そのcrate追加時に上へ加える。Phase Dではparser fuzz、archive負例、install crash fixtureを別の必須commandとして仕様化する。

## 7. 明確な停止条件

次のいずれかに当たったら実装を止める。

- `PluginId`を根拠なくVism package IDへ流用しようとしている。
- consumer Vismが具体provider VismのIDや表示名を検索している。
- Kitが任意code、独自Undo、常駐runtime、既存Projectの自動更新を持つ。
- 一つのVismが一entryか複数entryかをKit比較前に決めようとしている。
- author、license、signatureを`NodeDesc`へ足してpackage manifestの代用にしている。
- Project openがnetwork、install、build、または新code実行を起こす。
- first-partyだけHost内部APIを使う。
- `.vism`を拡張子だけ決まったZIP／JSON／binaryとして先行実装する。
- native、WASM、WGSL、sourceを「実行できる」の一語へ畳み、trustと権限を分離していない。
- custom UIをVism動作の必須入口にする。
- Simulationを通常Filterの隠れ状態で模倣する。
- M3再締結ゲート、M4/M5依存、GR-PVを飛ばす。

## 8. 最初の五手

直近の実装計画としては次の順が最も小さい。

1. **VSM-A0のinventory**: [コード監査を完了](2026-07-17-vism-a0-plugin-boundary-inventory.md)。test-only機械fixtureは別発注とし、公開面を変更しない。
2. **VSM-A7の意味spike**: [完了](2026-07-17-vism-a7-bpm-datatrack-spike.md)。現行BPMから連続拍位置を導出し、既存`DataTrackId`→`DocParam::Data`でparameterを駆動した。consumer plugin、公開型、Document変更なし。
3. **VSM-A0Dの処分**: [決定完了](2026-07-17-vism-a0d-contract-migration-ownership-decision.md)。plugin作者=contract／migration意味、Host=immutable catalog／transaction、Document=raw recipe保持へ分離した。
4. **VSM-A0Sの仕様PR**: [完了](2026-07-17-vism-a0s-contract-catalog-spec.md)。contract catalogとexecutor registry、宣言的rename、prepared recipe、validate接続の公開signatureと旧API処分を確定した。
5. **VSM-A0I-1〜3の実装PR**: A0Sの写しだけをcontract/runtime→Document resolution→製品実行入口の順で実装する。A0I-3完了がA1/A2の前提。

上の2は「仮のtyped input／consumer Vism」を作る意味ではない。現行`DataTrackId`→`DocParam::Data`で既存parameterを駆動するだけとし、入力portを持つconsumer pluginはVSM-B2の比較前に実装しない。A0Dの所有決定はA0Sで仕様化済みであり、A0I-1〜3でコード化した後、最初の外部crate実証がVSM-A1、その次がVSM-A2である。

この五手で見つかった継ぎ目が、Vismのmanifestやloaderより先に処分すべき本当のplugin／typed port境界である。A0I後にA1/A2を実証しても`.vism` parserへは進まず、A3〜A6とPhase BのKit／identity fixtureを完了してから配布形式を審判する。

## 9. この計画が完了したと呼べる条件

Vism全体の完了は「`.vism`を開けた」ではない。少なくとも次をすべて満たした時である。

- ユーザーが表現名で発見・導入・追加でき、内部kindを理解しなくてもよい。
- Project openとinstall／executeが分離している。
- package、entry、Kit、Project instance、artifactのidentityが更新・複製・欠落をまたいで壊れない。
- Vismは具体providerでなく型を要求し、Kitが接続を1 macroでmaterializeできる。
- first-partyと第三者が同じ公開API、resource、diagnostic、UI、migration規則を通る。
- 欠落時に作品を壊さず、評価不能なexportは黙って似せない。
- pixel、control、generation、Host所有stateの異なる責任をpackageが隠蔽せず宣言できる。
- Motolii内部UI／Documentを使わないheadless互換Hostがprovider→consumerを評価できる。
- 作者が一コマンドのconformanceで正否を知り、LLMも未決を推測せず同じ審判を回せる。

ここまで到達して初めて、Vismは単なるMotolii plugin archiveではなく、持ち運べる映像表現の単位になったと判定する。
