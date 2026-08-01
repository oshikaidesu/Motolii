# VSM-A4S — 外部crate作者scaffold責任仕様

状態: **決定**（仕様のみ。VSM-A4I実装、local Vism、package／install／loaderは未成立）

## 1. 利用者成果と停止線

VSM-A4Sが閉じる成果は、first-party参照Vismを作者が別crateへforkし、公開`motolii-plugin` façadeだけで編集・検査し、Hostへ明示登録してrebuild／restartできるところまでである。Radial Repeaterを代表fixtureとする。

これはstatic bundled first-party開発の足場であり、第三者package、動的load、local Vism、TypeScript authoring、製品内IDE、Document保存形式を成立させない。`ScriptWasm`予約variantをruntime証拠として扱わない。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [Vism実装計画 VSM-A4S/A4I](2026-07-17-vism-implementation-plan.md)、[作者journey](2026-07-27-vism-authoring-journey-decision.md)、[plugin authoring](../plugin-authoring.md) |
| `INTERNAL TARGET` | `motolii-plugin`の公開re-export／trait、`plugins/motolii-plugin-radial-repeater`、`motolii-testkit`のHost側purity／golden oracle、`motolii-plugins-firstparty`の明示composition root |
| `OWNER` | scaffold生成はHost developer tooling、検査はHost test infrastructure、登録はfirst-party composition root。plugin crateはDocument、testkit、install、loaderを所有しない |
| `WRITE ROUTE` | 新しい外部crate用CLIが指定出力directoryへcrateを生成する。作者がsourceを編集し、Host側fixtureがcompile／conformance／purity／goldenを検査し、採用時だけ人または後続粒がcomposition rootへ明示登録する |
| `GAP` | 現行`scripts/new-plugin.sh`は`motolii-plugin`内部へ貼るsourceとtestkit用testを生成し、`motolii_core`等の内部importを含む。独立Cargo crate、公開façade限定依存、Host側out-of-tree検査入口は生成しない |
| `RESOLUTION ROUTE` | 既存in-tree入口は維持し、外部crate専用入口を別toolとして追加する。公開façadeはRadial Repeaterで通常依存`motolii-plugin`一つの成立を確認済みで、新しい公開APIを作らない |
| `DISPOSITION` | `PASS / SPEC`。実装はVSM-A4Iへ分離。LANG-TS-F0／SDK-S0は本仕様がmainへ到達するまで`WAIT` |

## 3. A4Iが生成する最小closure

外部crate用入口は既存`scripts/new-plugin.sh`を拡張せず、`scripts/new-plugin-crate.sh`から`scripts/new_plugin_crate.py`を呼ぶ別toolとする。A4Iの最小対応は`--from core.layer_source.radial_repeater`、`name`、`vendor`、明示`--out-dir`とし、Radial Repeaterのsource forkを新しいcrate／plugin identityへ機械生成する。他kindや任意first-party pluginへの対応をA4I完了条件へ広げず、暗黙にworkspaceやregistryを書き換えない。

生成先は次だけを持つ。

- `Cargo.toml`: 通常依存は`motolii-plugin`だけ。dev／build dependencyと`build.rs`を持たない。
- `src/lib.rs`: `motolii_plugin::*`の公開面だけで一つのplugin contractとtrait実装を定義する。実装未完は型付き`PluginError`でfail closedする。
- `AUTHORING.md`: Host検査command、明示登録、rebuild／restart、package未成立の注意だけを示す。install manifestやloader設定を生成しない。

plugin crate内へtestkitを入れない。conformance／purity／goldenはHost所有の専用fixtureが、一時Host harnessの依存として生成crateと`motolii-testkit`を別々に参照して実行する。生成crateの`motolii-plugin` path解決はgeneratorが置換可能な明示値として書き、Host harnessが検査対象repositoryの正確なpathを注入する。repository rootのambient workspace継承に依存させない。golden期待値が未設定なら必ず失敗し、雛形の透明画像や既定値で合格させない。

first-party無特権gateはCargo dependency closureとsource上の直接参照を検査する開発時hygiene gateであり、悪性native Rustを封じ込めるsandboxではない。`std::fs`／`std::net`／`std::process`／`std::env`、`unsafe`、FFIを含むambient authorityの実行時封じ込めはVSM-C2以降の責任で、A4I合格から第三者native codeの安全性を称さない。

## 4. 10分fork oracle

機械fixtureに加え、同じ席で次の順序を一度通す。時間値は製品SLOではなく、作者journeyの発見性を比較するprobeである。

1. Radial Repeaterを参照候補として選ぶ。
2. 新規identityの外部crateを生成する。
3. parameter一つとWGSL表現一つを変更する。
4. Host側compile／contract／purity／goldenを実行し、失敗時にcrate、file、plugin ID、検査段を診断する。
5. first-party composition rootへ明示登録する。
6. rebuild／restart後、標準parameter projectionとrender経路で変更を確認する。

自動登録、hot reload、package D&D、製品内編集をこのprobeへ足してはならない。

## 5. A4I変更許可面

A4Iは次の面だけを変更できる。名称変更が必要ならA4S改訂へ戻し、実装中に広げない。

- `scripts/new-plugin-crate.sh`（新規）
- `scripts/new_plugin_crate.py`（新規）
- `crates/motolii-plugin/tests/external_plugin_scaffold.rs`（新規。生成物と依存closureの機械検査）
- `docs/plugin-authoring.md`（既存in-tree入口との使い分け追記）
- `docs/reviews/2026-07-17-vism-implementation-plan.md`
- `docs/implementation-ledger.md`

`motolii-plugin`公開API、Document／serde、plugin package契約、first-party registry、既存`new-plugin`実装・期待値は変更禁止とする。生成crateのHost実行fixtureに既存testkit APIだけでは不足する場合、A4Iを止め、必要なHost側試験面を別粒で仕様化する。

## 6. 必須負例

A4Iの機械検査は少なくとも次を拒否する。

- 通常依存に`motolii-plugin`以外、またはdev／build dependency、`build.rs`がある。
- `motolii_core`、`motolii_eval`、`motolii_gpu`、UI toolkit、OS／vendor API、`motolii-testkit`をplugin sourceが直接参照する。
- `std::fs`／`std::net`／`std::process`／`std::env`、`unsafe`、FFIの直接参照を含む。これはA4I生成物のhygiene拒否であり、native sandboxの証明ではない。
- 生成がworkspace member、composition root、registry、既存fileを暗黙変更する。
- package、install manifest、loader、署名、権限schemaを生成する。
- duplicate／invalid plugin ID、invalid `NodeDesc`、未設定goldenを成功扱いする。
- 既存`scripts/new-plugin.sh`のfixture出力が変化する。
- isolated生成crateがrepository rootのambient dependencyや未申告pathに依存する。

## 7. A4S完了と後続入口

A4Sの完了条件は、本文の責任、生成closure、Host検査、10分fork、変更許可面、負例、STOPが独立レビューでP0/P1=0となり、mainへ統合されることである。文書作成branchでのdocs greenはmain到達ではない。本稿作成時の外部レビュー呼び出しは二回とも空出力で、票として未取得である。

その後もVSM-A4Iは別実装粒であり、A4I完了はlocal Vism／第三者配布の完成を意味しない。LANG-TS-F0／SDK-S0はA4S main到達後に、言語非依存Path2D意味fixtureとTypeScript frontendを分けた仕様粒として再入場する。
