# CU-0A08SSCI-T Browser private component verification harness boundary 裁定

- 日付: 2026-07-29
- 状態: **決定**
- 対象grain: **CU-0A08SSCI-T**

## 1. 目的

[CU-0A08SSCI-T0 §4](2026-07-29-cu-0a08ssci-t0-browser-private-component-verification-harness-grain-numbering-decision.md#4-cu-0a08ssci-t-が閉じる唯一の問い)が固定した唯一の問いに答える。

公開export追加・新module追加・新依存追加・DOM / CSS / golden / fixture / threshold変更なしで、module-private `CandidateCreateBrowser` の private input seam を正例と負例の両方で検証できるharness境界を、既存のNode test / guard構成の内側でどこまでに限定するか。

## 2. 事実

1. `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx` = 1276行。`function CandidateCreateBrowser()` は537行で **module-private**、propsを受けない。`function ElementCard({` は474行で module-private。file唯一のexportは1198行 `export function DiscoveryBrowserCandidate`。
2. `const elementProps = (itemId) => ({ itemId, selected, tags, tagVisible, onSelect })` は559行。`itemId` は bare 文字列。Rectangle card は681行 `<ElementCard {...elementProps("rectangle")} element="rectangle" name="Rectangle" ... />`。JSX `identity` literal は686行 `identity="motion-kit.type-pulse"` の1箇所のみでRectangleには無い。`<CandidateCreateBrowser />` は747行で引数なし。
3. `ui/motolii-web/src/index.js` のexportは `DiscoveryBrowserCandidate` / `EasingTriggerCandidate` / `InspectorCandidate`+`InspectorContext` / `KeyToolsCandidate` の4行のみ。`CandidateCreateBrowser` / `ElementCard` / `elementProps` はpackage公開面に存在しない。
4. `ui/motolii-web/package.json` は `"exports": { ".": "./src/index.js" }`、`dependencies` は `html-react-parser` と `react` のみ。**`devDependencies` なし、`scripts` なし**。`ui/motolii-web` 配下にvite / babel / bundler設定fileは存在しない。
5. `ui/motolii-web/guard-tests/browser-ownership.test.mjs` は `node:test` + `node:assert/strict` でrunnerを構成し、21〜22行の `createRequire(docs/mocks-ui/package.json)` 経由で `@babel/parser` を読み、当該 `.jsx` を **AST / hash / provenance の静的oracle**として検査する。`test(` は338 / 734 / 758行の3件。357行以降で synthetic byte / provenance chain を用いた正負oracleが既にある。`CandidateCreateBrowser` / `ElementCard` / `elementProps` への言及は現時点では無い。
6. `docs/mocks-ui/package.json` の `test:reference-guard` は guard-tests、reference-guard script、product側 `browser-ownership.test.mjs` を同一npm script内で順に実行する。devDependenciesに `@babel/parser` 7.29.7 がある。**jsdom / @testing-library / react-dom test runner / vitest / react-test-renderer は存在しない。**
7. `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` と `inspector-read-model-decoder.test.mjs` は `node:test` + `node:assert/strict` だけで、`ui/motolii-web/src/read-model/*.js`（**JSXでない plain ESM**）を相対importして正負を判定する。JSX変換もbundlerも介さない。
8. 本worktreeに `docs/mocks-ui/node_modules` と `ui/motolii-web/node_modules` は**存在しない**。`@babel/parser` を要するguardは本worktreeで**実行できない**。installしない。
9. BASE_SHAで `./scripts/check-docs.sh` = exit 0、`browser-catalog-decoder.test.mjs` = 118 pass / 0 fail、`inspector-read-model-decoder.test.mjs` = 39 pass / 0 fail。
10. `CU-0A08SSCI-I` §3で候補(A)採択済み: decode済み `(scope_ref, item_id)` を **1件だけ** module-private `CandidateCreateBrowser` の private component input として受け、既存 `elementProps` から **VS-1 Rectangle card だけ** へ同じ2-field identityを非推測で透過する。型 / callback / event / payload / props名 / module path / export / wire / transport / decoder名は**未決**。

## 3. 裁定

候補母集団は [CU-0A08SSCI-T0 §5](2026-07-29-cu-0a08ssci-t0-browser-private-component-verification-harness-grain-numbering-decision.md#5-候補境界の母集団裁定しない) の (a)〜(d) に固定する。

| 候補 | C1 | C2 | C3 | C4 | C5 | C6 | 判定 |
|---|---|---|---|---|---|---|---|
| **(a)** `@babel/parser` による `.jsx` AST静的検査境界（`browser-ownership.test.mjs`、`node:test`） | 満たす。公開export・新module・新依存・DOM/CSS/golden/fixture/threshold変更を境界採択自体は要しない | 満たす。module-private かつ `.jsx` である対象を、package exportを介さず当該file source のASTとして観測できる | 満たす。当該guardは既にsynthetic source / byte を用いた正負oracleを持ち、同一runner内で拡張可能 | 満たす。**一つのclosed boundary**。所有guard = `ui/motolii-web/guard-tests/browser-ownership.test.mjs` | 満たす。VS-1 Rectangle 1件・decode済み2-field非推測透過を崩さない静的検証に限定できる | 満たす。既存 `@babel/parser` devDependency と `createRequire` 解決のみ。install / 新依存 / 公開exportを要しない | **採択** |
| **(b)** plain ESM 直接import境界（decoder guard-tests） | 境界自体は満たす | **不採択**。相対import対象は `read-model/*.js` の plain ESM のみ。`DiscoveryBrowserCandidate.jsx` は `.jsx` で `index.js` にexportが無く、JSX runnerも無いため module-private JSX seam を実際に観測できない | — | — | — | — | 不採択 |
| **(c)** `test:reference-guard` 直列に載せる境界 | (a) と同型 | (a) と同型 | (a) と同型 | **(a) と同一境界の既存実行関係**であり第二のharnessではない。実行script = `docs/mocks-ui/package.json` の `test:reference-guard` | (a) と同型 | (a) と同型 | 単独採択対象ではない |
| **(d)** 新依存 / 新module / 公開exportを要する境界 | **制約違反**。C1を満たさない | — | — | — | — | **拒否**。C6により不採択 | 拒否 |

**採択は (a) の1件のみ**とする。

- **所有guard**: `ui/motolii-web/guard-tests/browser-ownership.test.mjs` が `@babel/parser` による `.jsx` AST静的検査と正負oracleの責任を持つ。
- **実行script**: `docs/mocks-ui/package.json` の `test:reference-guard` が当該guardを既存直列の最終段として実行する責任を持つ。(c) はこの実行関係の記述であり、(a) と別harnessとして数えない。

harness実装、test file path、test名、assertion形、props名、型、event、payload、module pathは本粒で決めない・命名しない・例示しない。

## 4. 次の唯一 DO

binding order §5 の決定規則に従い **N1** を採る。

採択境界 (a) の内側では、BASE_SHAの product byte を1つも変えずに、既存guardが用いる synthetic な正負source構成で正例が緑になり得る（357行以降の既存patternと同型）。

- 次の唯一 `` `DO` `` = **`CU-0A08SSCI-T1`**（採択境界の最小実装粒）。
- `CU-0A08SSCI` は `` `WAIT` `` 維持。
- PRODUCT-ASSETの製品実装（コード変更を伴う）完全一致 `` `DO` `` は **0件** 維持。

`CU-0A08SSCI-T1` の harness実装・test file path・test名・assertion形・props名・型・event・payloadは本粒で決めない。

## 5. 変わらないもの

- React source byte（`ui/` 配下すべて）、`ui/motolii-web/src/index.js` の公開export、`.css`、guard期待値・literal・hash、`source-provenance.json` 実データ。
- `docs/mocks-ui` 配下すべて（guard-tests、fixture、golden、threshold、scripts、`package.json`）、`docs/mocks/`、lockfile、`node_modules`。
- 公開API / Document / journal / serde / 永続形式 / Undo単位 / plugin契約 / Place owner、`CU-101` / `CU-102` / `CU-110` / `CU-111` の意味。
- `CU-0A08SSCI-I` §3の候補(A)裁定、VS-1 Rectangle 1件限定、decode済み `(scope_ref, item_id)` 2-fieldの非推測透過。
- bare `itemId` drag payload と JSX `identity` literal の `S`（**未解消のまま**）。
- ReactへDocument / selection / Undo 正本を置かない規律。
- 過去裁定本文（`CU-0A08SSCI-T0` / `-I` / `-I0` / `CU-0A08SSCSD` / `CU-0A08SSCD` / `CU-0A08SSC` / `CU-0A08SSCI` 前提順序 / `CU-G09` 系）。

## 6. 非目標

- harness実装、test file path新設、test名、assertion形、fixture、npm script追加、guard追加。
- 型 / callback / event / payload / props名 / module path / export / wire / transport / decoder名 を決める・命名する・例示すること。
- `CU-0A08SSCI` を `` `DO` `` へ上げること、`CU-0A08SSCI` または `CU-0A08SSCI-T1` を「発注依存証跡」へ追加すること（本粒完了時は `CU-0A08SSCI-T` のみ追加）。
- raw input / decode、Host transport、D2、drop終端、typed intent、JSX binding、`S` 行を範囲へ入れること。
- VS-1 Rectangle 以外の card への一般化。
- allowlist外 stale mirror の修復。

## 7. 必須負例

1. 採択境界を0件または2件以上にする / 第5の候補境界を発明する / (a) と (c) を二重harnessとして数える。
2. (b) を不採択理由なしで落とす、または (d) を制約違反として拒否せず採る。
3. 型 / event / payload / props名 / decoder名 / module path / test file path / test名 / assertion形を決める・命名する・例示する。
4. bare `itemId` または JSX `identity` literal を scoped identity として肯定する / `S` を解消済みと書く。
5. ReactへDocument / selection / Undo 正本を追加する、または追加を許容する文を書く。
6. 区間内の完全一致 `` `DO` `` を0件または2件以上にする / `CU-0A08SSCI` を `` `DO` `` へ上げる / 製品実装の完全一致 `` `DO` `` を1件以上にする。
7. `CU-0A08SSCI` または `CU-0A08SSCI-T1` を「発注依存証跡」へ追加する（`CU-0A08SSCI-T` 行の追加は本粒の完了操作）。
8. allowlist外のfileを変更する / 過去裁定本文を改変する。
9. `ui/` 配下、`docs/mocks-ui` 配下、guard、fixture、golden、threshold、`package.json`、lockfileを変える / `node_modules` を install する。
10. 7 mirrorの部分同期 / 索引登録を欠く。
11. 差分に行末空白を残す。

## 8. 同期した current mirror

1. `docs/implementation-ledger.md` M3行
2. `docs/implementation-ledger.md` 「M3への入場判定」末尾の運用判断散文
3. `docs/decision-index.md` M3 VS-1 縦slice 総括行
4. `docs/decision-index.md` `CU-110S CU-110D ...` 行
5. `docs/decision-index.md` `CU-0A08RS0 ... CU-0A08SSCI ...` 系列行
6. `docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md` journal durability行
7. 同 selection / Undo再投影行

同じ7箇所を、docs-only `CU-0A08SSCI-T` は `` `DONE` ``、次の唯一の `` `DO` `` はORACLE-GUARD `CU-0A08SSCI-T1`（1件）、`CU-0A08SSCI` は `` `WAIT` `` 継続、PRODUCT-ASSETの製品実装（コード変更を伴う）完全一致 `` `DO` `` は0件へ同期した。

## 9. allowlist外に残る stale mirror

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`
- `CU-0A08SSCI-I` §9 handoff表の `CU-0A08SSCI-T0` 行（`DO` 表記のまま。過去裁定本文は変更しない）

## 10. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSCI-T` | **DONE** | private component verification harness境界を候補(a)で裁定 |
| `CU-0A08SSCI-T1` | **DO** | 採択境界(a)内側で正負検証harnessを実装 |
| `CU-0A08SSCI` | **WAIT** | 製品React実装は継続待ち |
| harness形の詳細 | 未決 | test file path・test名・assertion形・props名・型は `CU-0A08SSCI-T1` で決める |

## 11. STOP条件

1. C1〜C6 を適用しても採択境界が1つに定まらない、または全候補が落ちる。
2. 裁定を閉じるために harness形の詳細、test名、assertion形、props名、型、module path を決める必要が生じる。
3. `ui/` の byte、`docs/mocks-ui` の byte、guard期待値・literal・hashを変えないと結論が閉じない。
4. 公開API / Document / serde / 永続形式 / plugin契約 / Place owner へ波及する。
5. bare `itemId` または既存JSX `identity` literal を scoped identity として肯定しないと文が閉じない。
6. allowlist外のfileを変更しないと整合が取れない。
7. `CU-0A08SSCI-I` §3 の候補(A)裁定や既存裁定と矛盾する「現行」記述になる。
