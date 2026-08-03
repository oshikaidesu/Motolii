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
| `DISPOSITION` | `PASS / SPEC`。実装はVSM-A4Iへ分離。作者意味SDKのSDK-S0S/Iはartifact／ownerを共有しない別laneであり、本scaffoldを直列依存にしない |

## 3. A4Iが生成する最小closure

外部crate用入口は既存`scripts/new-plugin.sh`を拡張せず、`scripts/new-plugin-crate.sh`から`scripts/new_plugin_crate.py`を呼ぶ別toolとする。A4Iの最小対応は`--from core.layer_source.radial_repeater`、`name`、`vendor`、明示`--out-dir`とし、Radial Repeaterのsource forkを新しいcrate／plugin identityへ機械生成する。他kindや任意first-party pluginへの対応をA4I完了条件へ広げず、暗黙にworkspaceやregistryを書き換えない。

生成先は次だけを持つ。

- `Cargo.toml`: 通常依存は`motolii-plugin`だけ。dev／build dependencyと`build.rs`を持たない。`edition`／`license`／`lints`はworkspace継承でなく生成時の具象値とし、空の`[workspace]`を置いて親repositoryのambient workspaceへ参加しない。
- `src/lib.rs`: `motolii_plugin::*`の公開面だけで一つのplugin contractとtrait実装を定義する。実装未完は型付き`PluginError`でfail closedする。
- `AUTHORING.md`: Host検査command、明示登録、rebuild／restart、package未成立の注意だけを示す。install manifestやloader設定を生成しない。

plugin crate内へtestkitを入れない。conformance／purity／goldenはHost所有の専用fixtureが、一時Host harnessの依存として生成crateと`motolii-testkit`を別々に参照して実行する。一時harnessはcheck scriptがrepository外の一時directoryへ生成し、検査後に破棄してrepositoryと生成crateへ書き残さない。生成crateの`motolii-plugin` path解決はgeneratorが置換可能な明示値として書き、Host harnessが検査対象repositoryの正確なpathを注入する。repository rootのambient workspace継承に依存させない。作者向け検査入口は同じ`scripts/new-plugin-crate.sh --check <crate-dir>`としてHost toolingが所有し、別のpackage／loader commandを新設しない。golden期待値が未設定なら必ず失敗し、雛形の透明画像や既定値で合格させない。purity／goldenを実行する検査入口は`MOTOLII_REQUIRE_GPU=1`相当のGPU必須条件を使い、GPU不在によるskipを成功として報告しない。

`vendor`に既定値を置かない。作者が明示した値だけを受け、first-party／built-in予約namespaceの現行閉集合`core`／`doc`を外部crate生成では拒否する。`doc`はVSM-A3-1bの`ReservedBuiltinId`と`doc.layer_source.rect`を根拠にし、将来の予約追加をgenerator独自判断で行わない。`--out-dir`がMotolii repository木の内側を指す場合も拒否し、`plugins/`への生成や暗黙workspace member化を入力段階で成立させない。

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
- `edition`／`license`／`lints`が親workspace継承のまま、空の`[workspace]`なしで生成される。
- `--out-dir`がMotolii repository木の内側を指す、`vendor`が未指定、または外部作者が予約namespace `core`／`doc`を指定する。
- package、install manifest、loader、署名、権限schemaを生成する。
- duplicate／invalid plugin ID、invalid `NodeDesc`、未設定goldenを成功扱いする。
- GPU不在でpurity／goldenをskipし、その実行を合格扱いする。
- 既存`scripts/new-plugin.sh`のfixture出力が変化する。
- isolated生成crateがrepository rootのambient dependencyや未申告pathに依存する。

## 7. A4S完了と後続入口

A4Sの完了条件は、本文の責任、生成closure、Host検査、10分fork、変更許可面、負例、STOPが独立レビューでP0/P1=0となり、mainへ統合されることである。文書作成branchでのdocs greenはmain到達ではない。

2026-08-01のClaude Code経由Fable 5独立レビューは`REVISE（P0=0、P1=1、P2=3）`だった。P1はGPU不在時の暗黙skipをgolden／purity合格として扱える穴であり、§3と§6へGPU必須／skip非合格を追加した。P2はout-of-tree `Cargo.toml`のworkspace分離、Host検査commandのowner、vendor／出力先の拒否条件であり、同じく§3と§6へ反映した。初回の空出力記録はCLIが返した継続sessionを監督側が追跡しなかった誤判定であり、レビュー不能の証拠として扱わない。

修正後のspot reviewは`ACCEPT（P0=0、P1=0、P2=3）`。追加P2の一時harness owner、`doc`予約namespace、SDK-S0との誤った直列依存をCodexが現行コードへ再照合して採用し、§2／§3／§6／本節へ反映した。これらは公開API、Document、package／runtime意味を増やさない。A4Sは本変更のmain統合で仕様完了とし、VSM-A4Iは引き続き別実装粒とする。

VSM-A4I完了はlocal Vism／第三者配布の完成を意味しない。SDK-S0S/IはM2 PathOpと意味SDK決定を上流に持つ独立laneで、LANG-TS-F0はSDK-S0Iをconsumerにする。外部Rust crate scaffoldの完成を作者意味fixtureの入場条件にしない。
