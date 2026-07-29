# CU-0A08SSCI-T0 Browser private component verification harness grain numbering

- 日付: 2026-07-29
- 状態: **決定**
- 対象grain: **CU-0A08SSCI-T0**

## 1. 目的

未採番前提 **(T)**「module-private `CandidateCreateBrowser` を、公開export追加・新依存追加なしで検証するharness」を **`CU-0A08SSCI-T`** として採番し、次のdocs-only裁定粒が閉じる**唯一の問い**を1文で固定する。本粒は問いに答えない。harness形、検証手段、依存、file配置、test名、assertion形を決めない・命名しない・例示しない。

## 2. 事実

1. `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx` = 1276行。`function CandidateCreateBrowser()` は537行で **module-private**、propsを受けない。`function ElementCard({` は474行で module-private。file唯一のexportは1198行 `export function DiscoveryBrowserCandidate`。
2. `const elementProps = (itemId) => ({ ... })` は559行。`itemId` は bare 文字列。Rectangle cardは681行 `<ElementCard {...elementProps("rectangle")} element="rectangle" name="Rectangle" ... />`。JSX `identity` literal は686行 `identity="motion-kit.type-pulse"` の1箇所のみでRectangleには無い。`<CandidateCreateBrowser />` の呼び出しは747行、引数なし。
3. `ui/motolii-web/src/index.js` のexportは `DiscoveryBrowserCandidate` / `EasingTriggerCandidate` / `InspectorCandidate`+`InspectorContext` / `KeyToolsCandidate` の4行のみ。`CandidateCreateBrowser` / `ElementCard` はpackage公開面に存在しない。
4. `ui/motolii-web/package.json` は `"exports": { ".": "./src/index.js" }`、`dependencies` は `html-react-parser` と `react` のみ、`devDependencies` なし、`scripts` なし。`ui/motolii-web` 配下にvite / babel / bundler設定fileは存在しない。
5. `docs/mocks-ui/package.json` の `test:reference-guard` は `node --test guard-tests/*.test.mjs` → `node scripts/reference-guard.mjs check-registry src/main.jsx` → `node scripts/reference-guard.mjs check-manifest reference-provenance.json` → `node --test ../../ui/motolii-web/guard-tests/browser-ownership.test.mjs` の直列。devDependenciesは `@babel/parser` 7.29.7 / `@babel/traverse` 7.29.7 / `@vitejs/plugin-react` / `vite` ^6.0.11 / `@playwright/test` / `storybook` 系 / `pixelmatch` / `pngjs` / `postcss` / `@fontsource/inter`。**jsdom、@testing-library、react-dom/test-utils専用runner、vitest、react-test-rendererは存在しない。**
6. `ui/motolii-web/guard-tests/browser-ownership.test.mjs` は `node:test` のみでrunnerを構成し、`createRequire(docs/mocks-ui/package.json)` 経由で `@babel/parser` を読み、`.jsx` source をAST / hash / provenanceの静的oracleとして検査する。`test(` は3件。`CandidateCreateBrowser` / `ElementCard` / `elementProps` への言及は無い。
7. `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` と `inspector-read-model-decoder.test.mjs` は `node:test` + `node:assert/strict` だけで、`ui/motolii-web/src/read-model/*.js`（**JSXでない plain ESM**）を相対importして正負を判定する。JSX変換もbundlerも介さない。
8. 本worktreeに `docs/mocks-ui/node_modules` と `ui/motolii-web/node_modules` は**存在しない**。`@babel/parser` を要するguard（`browser-ownership.test.mjs`、`npm run test:reference-guard`、`inspector-read-model-inventory.test.mjs`）は本worktreeで実行できない。installしない。
9. BASE_SHAで `./scripts/check-docs.sh` = exit 0（`OK: docs整合チェック全項目通過`）、`browser-catalog-decoder.test.mjs` = 118 pass / 0 fail、`inspector-read-model-decoder.test.mjs` = 39 pass / 0 fail。worktreeはclean、`git diff --check` 出力なし。
10. `docs/implementation-ledger.md` 「現在の並列レーン」〜「発注依存証跡」区間で状態セル完全一致 `` `DO` `` は `CU-0A08SSCI-T0` の1件のみ。`CU-0A08SSCI` は `` `WAIT` ``。
11. 「発注依存証跡」で `CU-0A08SSCI-I` / `CU-0A08SSCI-I0` / `CU-0A08SSCI-P1` / `CU-0A08SSCSD` / `CU-0A08SSCD` は状態セル完全一致 `` `DONE` ``。`CU-0A08SSCI-T0` と `CU-0A08SSCI` の行は存在しない。
12. `CU-0A08SSCI-I` §3で候補(A)採択済み: decode済み `(scope_ref, item_id)` を **1件だけ** module-private `CandidateCreateBrowser` のprivate component inputとして受け、既存 `elementProps` から **VS-1 Rectangle cardだけ** へ同じ2-field identityを非推測で透過する。型 / callback / event / payload / props名 / module path / export / wire / transport / decoder名は未決のまま。

## 3. 採番

前提 **(T)** = `CU-0A08SSCI-T`（PRODUCT-ASSET / M3 / VS-1 / SPEC / docs-only）。

`CU-0A08SSCI-T` 以外の新IDを与えない。

## 4. `CU-0A08SSCI-T` が閉じる唯一の問い

公開export追加・新module追加・新依存追加・DOM / CSS / golden / fixture / threshold変更なしで、module-private `CandidateCreateBrowser` の private input seam を正例と負例の両方で検証できるharness境界を、既存のNode test / guard構成の内側でどこまでに限定するか。

## 5. 候補境界の母集団（裁定しない）

BASE_SHAで既に存在する検証機構だけを事実として列挙する。採否・優劣・推奨・順位は書かない。

- **(a)** `ui/motolii-web/guard-tests/browser-ownership.test.mjs` が用いる `@babel/parser` による `.jsx` source のAST静的検査境界。runnerは `node:test`。`@babel/parser` は `docs/mocks-ui/package.json` の devDependency で、`createRequire` 経由の解決と `docs/mocks-ui/node_modules` 前提を要する。
- **(b)** `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` と `inspector-read-model-decoder.test.mjs` が用いる plain ESM 直接import境界。runnerは `node:test` + `node:assert/strict` のみで、JSX変換・bundler・追加devDependencyを介さない。
- **(c)** `docs/mocks-ui` package の既存 `test:reference-guard` 直列に載せる境界。上記 guard-tests、reference-guard script、product側 `browser-ownership.test.mjs` を同一npm script内で順に実行する。`@babel/parser` / `@babel/traverse` と `docs/mocks-ui/node_modules` 前提を含む。
- **(d)** 上記 (a)〜(c) のいずれにも該当しないため、新依存 / 新module / 公開exportを要し本粒の制約（公開export追加・新依存追加なし）を満たさない境界。現行 `ui/motolii-web` に devDependencies・test runner・JSX実行環境が無い事実と整合する不成立事実として記載する。

## 6. 変わらないもの

- React source byte（`ui/` 配下すべて）、`ui/motolii-web/src/index.js` の公開export、`.css`、guard期待値・literal・hash、`source-provenance.json` 実データ。
- `docs/mocks-ui` 配下すべて（guard-tests、fixture、golden、threshold、scripts、`package.json`）、`docs/mocks/`、lockfile、`node_modules`。
- 公開API / Document / journal / serde / 永続形式 / Undo単位 / plugin契約 / Place owner、`CU-101` / `CU-102` / `CU-110` / `CU-111` の意味。
- `CU-0A08SSCI-I` の候補(A)裁定、VS-1 Rectangle 1件限定、decode済み `(scope_ref, item_id)` 2-fieldの非推測透過。
- bare `itemId` drag payload と JSX `identity` literal の `S`（**未解消のまま**）。
- ReactへDocument / selection / Undo 正本を置かない規律。
- `CU-0A08BT` / `CU-0A08IT` / `CU-0A08RM` / `U2c-2` / `U3a-2Q-V` / `CU-110` / `CU-111` の状態・依存セル、W0/W1表、`U4a-2`、M3仕様本文。
- 過去裁定本文（`CU-0A08SSCI-I` / `-I0` / `CU-0A08SSCSD` / `CU-0A08SSCD` / `CU-0A08SSC` / `CU-0A08SSCI` 前提順序 / `CU-G09` 系）。

## 7. 非目標

- (T)の問いに答えること。harness形・検証手段・file配置・test名・assertion形・依存・runner・実行順を決める、命名する、例示する、推奨する、順位づけること。
- 型 / callback / event / payload / props名 / module path / export / wire / transport / decoder名を決める・命名する・例示すること。
- `CU-0A08SSCI-T` 以外のIDを与えること、(T)本体を採番済みと書くこと、`CU-0A08SSCI` を `DO` へ上げること、`CU-0A08SSCI` または `CU-0A08SSCI-T` を「発注依存証跡」へ追加すること。
- raw input / decode、Host transport、D2、drop終端、typed intent、JSX binding、`S` 行を範囲へ入れること。
- 隣接チケット（`CU-0A08BT` / `CU-0A08IT` / `CU-110` / `CU-111` / `U3a-2Q-V` / `CU-0A08RM`）の状態・依存・意味へ触れること。
- VS-1 Rectangle以外のcardへ一般化すること。
- allowlist外 stale mirrorの修復、M3仕様・`docs/README.md` の書き換え。
- 新しい状態語彙・新しいlane・新しい台帳表の追加。

## 8. 必須負例

1. (T)の問いに答える / harness形を1つ選ぶ / 候補境界を採択・推奨・順位づけする / 第5の候補境界を発明する。
2. 型 / event / payload / props名 / decoder名 / module path / test file path / test名 / assertion形を決める・命名する・例示する。
3. 公開export、新module、新依存、新npm script、新guard、新fixtureを決める・追加する・例示する。
4. bare `itemId` または JSX `identity` literal を scoped identity として肯定する / `S` を解消済みと書く。
5. ReactへDocument / selection / Undo 正本を追加する、または追加を許容する文を書く。
6. 区間内の完全一致 `` `DO` `` を0件または2件以上にする / `CU-0A08SSCI` を `DO` へ上げる / (T)本体のharness形を裁定する。
7. `CU-0A08SSCI` または `CU-0A08SSCI-T` を「発注依存証跡」へ追加する。
8. allowlist外のfileを1 byteでも変更する / 過去裁定本文を改変する / stale mirrorを本粒で直す。
9. `ui/` 配下、`docs/mocks-ui` 配下、guard、fixture、golden、threshold、`package.json`、lockfileを1 byteでも変える / `node_modules` を install する / testをskip・削除・期待値変更する / lint抑制コメントを足す / TODO stubを残す。
10. 7 mirrorの部分同期（1〜6箇所だけ更新）。
11. 状態語彙の固定集合外の語を新設する / 台帳の他lane行の状態・順序・意味を書き換える。
12. 差分に行末空白を残す。

## 9. 同期した current mirror

1. `docs/implementation-ledger.md` M3行（`| M3 | **VS-1 Rectangle配置とUndo** |` で始まる行）
2. `docs/implementation-ledger.md` 「M3への入場判定」末尾の運用判断散文（`したがって現在の短い運用判断は、` で始まる行）
3. `docs/decision-index.md` M3 VS-1 縦slice 総括行
4. `docs/decision-index.md` `CU-110S CU-110D ...` 行
5. `docs/decision-index.md` `CU-0A08RS0 ... CU-0A08SSCI ...` 系列行
6. `docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md` journal durability行
7. 同 selection / Undo再投影行

同じ7箇所を、docs-only `CU-0A08SSCI-T0` は `` `DONE` ``、次の唯一の `` `DO` `` はPRODUCT-ASSET/SPEC docs-only `CU-0A08SSCI-T`（1件）、`CU-0A08SSCI` は `` `WAIT` `` 継続、(T)本体のharness形は未決、PRODUCT-ASSETの製品実装（コード変更を伴う）完全一致 `` `DO` `` は0件へ同期した。

## 10. allowlist外に残る stale mirror

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`
- `CU-0A08SSCI-I` §9 handoff表の `CU-0A08SSCI-T0` 行（`DO` 表記のまま。過去裁定本文は変更しない）

## 11. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSCI-T0` | **DONE** | 前提(T)を `CU-0A08SSCI-T` として採番し、次docs-only裁定粒の唯一の問いを固定 |
| `CU-0A08SSCI-T` | **DO** | private component verification harness境界をdocs-onlyで裁定 |
| `CU-0A08SSCI` | **WAIT** | (T)本体のharness形未決のまま |
| (T)本体のharness形 | 未決 | harness形・検証手段・依存は決めない |

## 12. STOP条件

1. §3のauthorityと事実だけでは、(T)の唯一の問いを1文へ閉じられない。
2. 問いを閉じるためにharness形・依存・module path・test名・assertion形を決める必要が生じた。
3. 候補境界の母集団を、BASE_SHAに存在しない機構を発明せずには列挙できない。
4. `ui/` のbyte、`docs/mocks-ui` のbyte、guard期待値・literal・hash、`source-provenance.json`、`package.json`、lockfileを変えないとoracleが緑にならない。
5. 公開API / 公開export / Document / serde / 永続形式 / plugin契約 / Place owner の変更が必要になる。
6. bare `itemId` または既存JSX `identity` literal を scoped identity として肯定しないと文が閉じない。
7. 区間内の完全一致 `` `DO` `` を `CU-0A08SSCI-T` の1件に保てない。
8. allowlist外のfileを変更しないと整合が取れない、または過去裁定本文の改変が必要になる。
9. `CU-0A08SSCI-T` 以外のIDを増やす、または(T)本体の実装・harnessを本粒へ混ぜる必要が生じた。
10. `CU-0A08SSCI-I` §3の候補(A)裁定と矛盾する「現行」記述になる、または新規docが既存裁定と衝突する。
