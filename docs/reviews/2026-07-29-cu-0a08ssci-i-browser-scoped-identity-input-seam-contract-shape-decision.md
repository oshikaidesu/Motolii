# CU-0A08SSCI-I Browser scoped identity input seam 契約形 裁定

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08SSCI-I: **DONE**

## 1. 目的

[CU-0A08SSCI-I0採番 §4](2026-07-29-cu-0a08ssci-i0-browser-scoped-identity-input-seam-grain-numbering-decision.md#4-cu-0a08ssci-i-が閉じる唯一の問い)が固定した唯一の問いに答える。

`CandidateCreateBrowser`がdecode済み`(scope_ref, item_id)`を受ける最小private input seamの契約形を、既存React component境界内でどこまでに限定するか。

## 2. 事実

1. `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx` は1276行。`function CandidateCreateBrowser()` は537行で **module-private**、propsを受けない。file の export は1198行 `export function DiscoveryBrowserCandidate` の1つだけ。
2. `const elementProps = (itemId) => ({ itemId, selected, tags, tagVisible, onSelect })` は559行。`itemId` は bare 文字列。
3. Rectangle card は681行 `<ElementCard {...elementProps("rectangle")} element="rectangle" name="Rectangle" ... />`。`ElementCard` は474行で module-private。
4. JSX `identity` literal は686行 `identity="motion-kit.type-pulse"` の1箇所のみで、Rectangle には無い。
5. `CandidateCreateBrowser` は747行で `CandidateBrowserTabs` から引数なしで呼ばれる。`developmentProjection` 定義時は `CandidateBrowserTabsProjectionOnly`（717行）が描画され、`decodeProjection` 出力は `CandidateProjectBrowser` へしか渡らない。
6. 当該 `.jsx` の SHA-256 は `866124a69caaa168fa19c67e6c723db97fec67a61071bdbe66973576266c42f4` で、`ui/motolii-web/source-provenance.json` の `postPromotionChanges` 1 entry の `currentSha256` と一致する。
7. `docs/mocks-ui/node_modules` および `ui/motolii-web/node_modules` は本worktreeに存在しない。`@babel/parser` を要する guard（`browser-ownership.test.mjs`、`inspector-read-model-inventory.test.mjs`、`npm run test:reference-guard`）は本worktreeで実行できない。install もしない。
8. BASE_SHA で `./scripts/check-docs.sh` = OK、`browser-catalog-decoder.test.mjs` = 118/118、`inspector-read-model-decoder.test.mjs` = 39/39。

## 3. 裁定

候補は次の2つのみとし、第三候補は設けない。

- **候補(A)**: VS-1 Rectangle用の scoped identity **1件だけ** を private component input として受け、既存 `elementProps` から **Rectangle card だけ** へ同じ2-field identityを非推測で透過する。
- **候補(B)**: decode済みcatalog全体 / items配列 / lookup・mapを `CandidateCreateBrowser` へ持ち込み、component内でRectangleを検索する。

**候補(A)** を採択する。

| 判定 | 候補(A) | 候補(B) |
|---|---|---|
| C1 ([CU-0A08SSCSD §3](2026-07-29-cu-0a08sscsd-browser-place-source-seam-implementation-scope-decision.md)): VS-1 Rectangle 1件に限り、raw input / decode / Host transport / D2 / drop終端を含まない | 満たす。1件のdecode済みidentityだけを受け、Rectangle card だけへ配る | 不採択。catalog全体・items配列の持込はVS-1 Rectangle 1件限定の実装範囲を越える |
| C2 ([CU-G09](2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md)): identityは `(scope_ref, item_id)` の2 fieldのみ。catalog ID / label / thumbnail token から導出しない | 満たす。供給済み2 fieldを非推測で透過するだけ | 不採択。component内検索・lookup・mapは token 由来の導出に当たる |
| C3 ([CU-0A08SSCD](2026-07-29-cu-0a08sscd-browser-place-source-seam-contract-concretization-decision.md)): module-private `CandidateCreateBrowser` 境界の内側に閉じ、公開export・新module・新依存を増やさない | 満たす | 満たすが C1/C2 で不採択 |
| C4 ([CU-0A08SSCI-I0 §5](2026-07-29-cu-0a08ssci-i0-browser-scoped-identity-input-seam-grain-numbering-decision.md#5-変わらないもの)): React に Document / selection / Undo 正本を追加しない。bare `itemId` と JSX `identity` literal の `S` を解消済みとしない | 満たす | 満たすが C1/C2 で不採択 |
| C5 ([CU-0A08SSCI-I0 §6](2026-07-29-cu-0a08ssci-i0-browser-scoped-identity-input-seam-grain-numbering-decision.md#6-非目標)): `(scope_ref, item_id)` 以外のfield、fallback / default を契約へ入れない | 満たす | 満たすが C1/C2 で不採択 |

採択内容は契約形の限定のみである。decode済み `(scope_ref, item_id)` を **1件** だけ module-private `CandidateCreateBrowser` の private component input として受け、既存 `elementProps` から VS-1 Rectangle card だけへ同じ2-field identityを非推測で透過する。型、callback、event、payload、props名、module path、export、wire、transport、decoder名は本粒で決めない・命名しない・例示しない。

## 4. 変わらないもの

- React source byte（`ui/` 配下すべて）、guard期待値・literal・hash、`source-provenance.json` 実データ、公開API。
- Document / journal / serde / 永続形式 / Undo単位 / plugin契約 / Place owner、`CU-101` / `CU-102` / `CU-110` の意味。
- bare `itemId` drag payload と JSX `identity` literal の `S`（未解消のまま）。
- `CU-0A08BT` / `CU-0A08IT` / `CU-0A08RM` / `U2c-2` / `U3a-2Q-V` の状態と依存セル、W0/W1表、`U4a-2`、M3仕様本文、`docs/mocks-ui` 配下、`docs/mocks/`。
- 過去裁定本文（`CU-0A08SSCI-I0` / `CU-0A08SSCSD` / `CU-0A08SSCD` / `CU-0A08SSC` / `CU-0A08SSCI` 前提順序 / `CU-G09` 系）。allowlist外の stale mirror 5件も変更しない。
- `docs/mocks-ui/guard-tests/*`、fixture、golden、threshold、`package.json`、lockfile、`node_modules`。

## 5. 非目標

- 型 / callback / event / payload / props名 / module path / export / wire / transport / decoder名 を決めること、命名すること、または例示として書くこと。
- (T) 本体の harness 形・検証手段・依存を決めること。`CU-0A08SSCI-T0` 以外のIDを与えること。
- `CU-0A08SSCI` を `DO` へ上げること、または「発注依存証跡」へ `CU-0A08SSCI` を追加すること。
- raw input / decode、Host transport、D2、drop終端、typed intent、JSX binding、`S` 行を範囲へ入れること。
- 隣接チケット（`CU-0A08BT` / `CU-0A08IT` / `CU-110` / `CU-111` / `U3a-2Q-V` / `CU-0A08RM`）の状態・依存・意味へ触れること。
- VS-1 Rectangle 以外の card へ一般化すること。
- allowlist外 stale mirror の修復、M3仕様・`docs/README.md` の書き換え。

## 6. 必須負例

1. 候補(A)(B)を両方採択する、どちらも採択しない、または第三の候補を新設する。
2. `(scope_ref, item_id)` 以外のfield、fallback / default、catalog ID / label / thumbnail からの検索・lookup・map を契約へ入れる。
3. 公開export、新module、新依存、新props名、新型、新event、新payload、新callback を決める・命名する・例示する。
4. bare `itemId` または JSX `identity` literal を scoped identity として肯定する / `S` を解消済みと書く。
5. React へ Document / selection / Undo 正本を追加する、または追加を許容する文を書く。
6. 完全一致 `` `DO` `` を0件または2件以上にする / `CU-0A08SSCI` を `DO` へ上げる / (T) 本体を採番する。
7. `CU-0A08SSCI` を「発注依存証跡」へ追加する。
8. allowlist外のfileを変更する / 過去裁定本文を改変する。
9. `node_modules` を install する / test を skip・削除・期待値変更する / lint抑制コメントを足す / TODO stub を残す。
10. 台帳の意味・順序・他lane行の状態を、許可された行操作以外で書き換える（7 mirror の部分同期も含む）。
11. 状態語彙の固定集合外の語を新設する。
12. 差分に行末空白を残す。

## 7. 同期した current mirror

1. `docs/implementation-ledger.md` M3行（`| M3 | **VS-1 Rectangle配置とUndo** |` で始まる行）
2. `docs/implementation-ledger.md` 「M3への入場判定」末尾の運用判断散文（`したがって現在の短い運用判断は、` で始まる行）
3. `docs/decision-index.md` M3 VS-1 縦slice 総括行
4. `docs/decision-index.md` `CU-110S CU-110D ...` 行
5. `docs/decision-index.md` `CU-0A08RS0 ... CU-0A08SSCI ...` 系列行
6. `docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md` journal durability行
7. 同 selection / Undo再投影行

同じ7箇所を、docs-only `CU-0A08SSCI-I` は `` `DONE` ``、次の唯一の `` `DO` `` はPRODUCT-ASSET/SPEC docs-only `CU-0A08SSCI-T0`（1件）、`CU-0A08SSCI` は `` `WAIT` `` 継続、前提(T)本体は未採番、PRODUCT-ASSETの製品実装（コード変更を伴う）完全一致 `` `DO` `` は0件へ同期した。

## 8. allowlist外に残る stale mirror

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`
- 既存選定文書 §5 handoff 表の `CU-0A08SSCI-I0` 行（`DO` 表記のまま。過去裁定本文は変更しない）

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSCI-I` | **DONE** | 最小private input seamの契約形を候補(A)で裁定 |
| `CU-0A08SSCI-T0` | **DO** | 未採番前提(T)を採番し、次docs-only裁定粒の唯一の問いを固定 |
| `CU-0A08SSCI` | **WAIT** | (T)本体未採番のまま |
| (T)本体 | 未採番 | harness形・検証手段・依存は決めない |

## 10. STOP条件

1. C1〜C5を適用しても候補が一意に定まらない、または両候補が残る。
2. 契約形を閉じるために型 / event / payload / props名 / decoder名 / module path を決める必要が生じる。
3. `ui/` の byte、guard期待値・literal・hash、`source-provenance.json` を変えないと oracle が緑にならない。
4. 公開API / Document / serde / 永続形式 / plugin契約 / Place owner の変更が必要になる。
5. bare `itemId` または既存JSX `identity` literal を scoped identity として肯定しないと文が閉じない。
6. 「現在の並列レーン」の完全一致 `` `DO` `` を `CU-0A08SSCI-T0` の1件に保てない。
7. allowlist外のfileを変更しないと整合が取れない、または過去裁定本文の改変が必要になる。
8. (T) 本体へIDを与える、または `CU-0A08SSCI-T0` 以外の新IDを増やす必要が生じた。
9. `CU-0A08SSCI-I0` §4 の問いと逐語一致させられない、または新規docが既存裁定と矛盾する「現行」記述になる。
10. authority の実hashがBASE_SHAで order の `AUTHORITY:` 値と一致しない。
