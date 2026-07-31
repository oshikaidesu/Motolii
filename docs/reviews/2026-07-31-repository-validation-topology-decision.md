# Repository validation topology 決定（2026-07-31）

状態: **決定**

## 1. 問題

`cargo test --workspace` はRust workspaceの強い横断回帰であり、今後も必要である。しかし現在の
Motoliiには、Rust以外にもdocs整合、監督runner、protected oracle、React product asset、
Playwright、platform、実機、人間審判がある。`cargo test`をrepository全体の完了ownerとして
書き続けると、次の二つを同時に誤る。

1. Rust testが緑なら、React、docs、製品E2E、実機まで完了したように見える
2. docs-onlyまたはReact-onlyの粒にも、変更面を観測できないoracleを形式的に要求する

したがって、`cargo test`を廃止・置換するのではなく、**Rust laneへ限定する**。taskの意味完了、
repository machine validation、独立review、human／hardware evidenceを別の証拠classとして扱う。

## 2. 決定

各taskは完了条件に次の三項目を持つ。

```text
PRIMARY_ORACLE: <変更した契約を直接失敗させられる既存command>
REPO_LANES: docs | policy | tooling | rust | web-build | web-contract | web-visual
EXTERNAL_GATES: NONE | <platform / product E2E / human / hardwareの名前付きgate>
```

- `PRIMARY_ORACLE`は自由な合格文ではなく、対象fixture、test、guardまたはcheck commandを完全に
  記す。変更面を観測できないcommandはoracleではない
- `REPO_LANES`は下表の閉集合から、粒が触る面を列挙する。pathから自動推測しない
- `EXTERNAL_GATES: NONE`は正本spec／decisionが外部審判を要求しない時だけ使う。実装者の都合で
  human／hardware gateを消さない
- machine green、独立review、main到達、human／hardware evidenceは相互代替しない
- `cargo test --locked --workspace`は`rust` laneの必要条件であり、repositoryまたは製品完成の
  十分条件ではない

## 3. lane閉集合

| lane | owner / command | 証明すること | 証明しないこと |
|---|---|---|---|
| `docs` | `scripts/check-docs.sh` | review索引、リンク、状態語彙、入口規律 | code、UI、製品route |
| `policy` | `check-protected-diff.sh`、`check-golden-update-policy.sh` | PR diffがprotected oracleを迂回しない | oracle意味の正しさ、push単独 |
| `tooling` | 監督runner専用試験、UI toolkit依存方向guard | 開発toolingの局所契約 | 製品機能、外部model可用性 |
| `rust` | fmt、clippy、`cargo test --locked --workspace` | Rust workspaceの静的・動的回帰 | React、docs、実機、人間審判 |
| `web-build` | 両npm packageのclean install／build／host bundle check | React sourceとcommit済みHost bundleがclean checkoutから再現可能 | reference、visual、native接続 |
| `web-contract` | reference guard、reference/current-route check | 固定source／provenance／publicationと固定browser capture bytes | 人間の外観審判、native WebView |
| `web-visual` | 既存Playwright suite | 固定browser／fixtureでのvisual・interaction回帰 | native window、実GPU、全OS |

Windows session lockなどOS固有jobは**platform axis**であり、上のportable laneへ隠さない。
product E2E、real audio device、real GPU、IME／Accessibility、人間審判も名前付き
`EXTERNAL_GATES`のまま保持する。

## 4. dispatcherの責任

`scripts/validate.sh`は上表の既存commandを呼ぶだけの薄いadapterとする。

- laneの並列scheduler、変更path判定、retry、cache、process supervisorを持たない
- 未知lane、引数欠落、既知commandの失敗をfail closedする
- `policy`のbase refはoption形を拒否し、commitとして解決できることをdispatch前に確認する
- reference／golden生成、期待値更新、threshold変更をしない
- local profileは便利な反復集合であり、CIまたはtask完了の別名にしない
- local profileは開発checkoutを暗黙に再生成しない`docs rust`だけを順に実行する。
  Playwright、Web build、policy、platform、human／hardwareを含まない
- `test-local.sh`は既存のlocal ffmpeg環境を読んだ後、このprofileへ委譲する
- `policy`のbaseにprotected pathがまだ無いbootstrap時は、そのsubgateだけを適用外として明記し、
  golden policyは必ず実行する。laneのPASSは実行対象になったgateが全て通った意味であり、
  protected gateを実行済みと数えない

`scripts/test-validate.sh`はlane閉集合、local profile順序、未知lane、空引数、
`policy`のbase欠落／option形／未解決ref、必須command不在を負例で固定する。
dispatcher自身のtestが製品testのownerにはならない。

## 5. CIへの接続

CI jobとlaneを対応させる。既存のpolicyとWindows gateは独立のまま維持し、portable test jobは
dispatcherの`docs`、`tooling`、`rust`を順に呼ぶ。`web-build`はclean checkoutの独立jobとし、
commit済みHost bundleを検査してから再生成し、生成後のtracked差分とuntracked pathを拒否する。

2026-07-31の統合基線では、`docs`、`tooling`、`rust`、`web-build`、`web-contract`を個別に
green確認した。`web-contract`は固定Chromiumを使うため、CI runnerへのbrowser provisioningと
clean-runner greenを別に閉じるまでblocking CIへ昇格しない。`web-visual`も同じく未昇格であり、
未実行または未完走をgreenや製品完成と数えない。

本決定による`AGENTS.md`の正本変更は、Inspector read-model inventoryが固定する
`AUTHORITY_SHA256["AGENTS.md"]`を変更後byteから再計算し、同じcommitで1 literalだけ更新する。
旧hash併記、照合緩和、assertion変更、他authority entryの変更は行わない。

この決定の回収元`cc49c91d`が記録したrustfmt差、UI compile error、authority hash driftは、
その後のmain統合で個別に修復済みである。歴史上のredを期待値変更で消したとは扱わず、現在の
green証跡と、未昇格のbrowser／visual gateを分離して報告する。

## 6. 責任処分

```text
RESPONSIBILITY DISPOSITION: WRAP
EXISTING ROUTE: Cargo, rustfmt, clippy, npm, Node test, Playwright,既存shell guards, GitHub Actions
OWNED RESIDUE: lane名、taskからoracleへの対応、証拠class分離、fail-closed dispatch
IMPORTED RESPONSIBILITY: 追加dependencyなし。既存toolchain/version/license/build責任を維持
EXIT: scripts/validate.shとCI stepだけを交換境界にし、各fixtureは既存ownerのまま
RETIREMENT: scripts/test-local.shのCargo単独ownerを廃止。既存runner/test frameworkは廃止しない
```

`cargo-nextest`、Make／just、新しいgeneric test frameworkは本決定では採択しない。Rust laneの速度が
実測上の阻害になった時、同じfixtureとfailure semanticsで独立比較する。

## 7. 負例とSTOP

次を拒否する。

1. 変更面を失敗させられない`PRIMARY_ORACLE`
2. local profile greenをCI、platform、human／hardware greenと報告する
3. toolchain不在、timeout、中断、0件実行をpassと数える
4. unknown laneを無視する、lane失敗を`|| true`等で隠す
5. visual／golden／referenceを検証commandから再生成する
6. task実装者が必要な`EXTERNAL_GATES`を`NONE`へ落とす
7. dispatcherへchange detector、scheduler、retry、background serviceを足す
8. 既存redを通すためにoracle hash、期待値、threshold、除外、lint抑制を変える
9. `web-contract`／`web-visual`をclean runnerでgreenにする前にblocking CIへ昇格する
10. `cargo test`を削除する、またはRust契約の代わりに別laneを使う

## 8. 完了条件

- `AGENTS.md`と`docs/specs/README.md`が`cargo test`単独ownerを撤回する
- decision index／review index／docs入口から本決定を逆引きできる
- dispatcher負例、`docs`、`tooling`、`rust`、`web-build`がgreen
- CIがdispatcherの`tooling`／`rust`と新しい`docs`／`web-build`を呼ぶ
- `AGENTS.md`変更後byteとInspector read-model inventoryの固定SHAが一致する
- policyとWindows gateが独立のまま残る
- `git diff --check`と`scripts/check-docs.sh`がgreen
