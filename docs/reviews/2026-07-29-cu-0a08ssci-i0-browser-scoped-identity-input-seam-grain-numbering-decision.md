# CU-0A08SSCI-I0 Browser scoped identity input seam grain numbering

- 日付: 2026-07-29
- 状態: **決定**
- 対象grain: **CU-0A08SSCI-I0**

## 1. 目的

未採番前提(I)を `CU-0A08SSCI-I` として採番し、次のdocs-only裁定粒が閉じる唯一の問いを固定する。本粒は問いに答えない。

## 2. 事実

1. `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx` は1276行。`function CandidateCreateBrowser()`（537行）は module-private で props を受けない。file の export は1198行 `export function DiscoveryBrowserCandidate` の1つだけ。
2. `const elementProps = (itemId) => ({...})`（559行）が各 card への既存props配布所有点で、`itemId` は bare 文字列。
3. `CU-0A08SSCD` は候補(B) `CandidateCreateBrowser` 境界をVS-1 Rectangleに限定採択済み。`CU-0A08SSCSD` は候補(A) 内部source seamのみをVS-1 Rectangleに限定採択済み。raw input/decode、Host transport、D2、drop終端は非目標として確定済み。
4. `CU-0A08SSCI-P` / `CU-0A08SSCI-P1` により前提(P)は authority と guard 実装の両面で閉じている（commit `08e974bb`）。
5. `docs/implementation-ledger.md` 「現在の並列レーン」で状態セルが完全一致 `` `DO` `` の行は `CU-0A08SSCI-I0` の1件のみ。`CU-0A08SSCI` は `` `WAIT` ``。
6. 「発注依存証跡」で `CU-0A08SSC` / `CU-0A08SSCD` / `CU-0A08SSCS` / `CU-0A08SSCSD` / `CU-0A08SSCI-P` / `CU-0A08SSCI-P1` は状態セルが完全一致 `` `DONE` ``。`CU-0A08SSCI-I0` の行は存在しない。

## 3. 採番

前提 **(I)** = `CU-0A08SSCI-I`（PRODUCT-ASSET / M3 / VS-1 / SPEC / docs-only）。

前提 **(T)** は**未採番のまま**とし、IDを与えない。

## 4. `CU-0A08SSCI-I` が閉じる唯一の問い

`CandidateCreateBrowser`がdecode済み`(scope_ref, item_id)`を受ける最小private input seamの契約形を、既存React component境界内でどこまでに限定するか。

## 5. 変わらないもの

- React source byte（`ui/`配下すべて）、guard期待値・literal・hash、`source-provenance.json` 実データ、公開API。
- Document / journal / serde / 永続形式 / Undo単位 / plugin契約 / Place owner。
- bare `itemId` drag payload と JSX `identity` literal の `S`。
- `CU-0A08BT` / `CU-0A08IT` / `CU-0A08RM` / `U2c-2` / `U3a-2Q-V` の状態と依存セル、W0/W1表、`U4a-2`、M3仕様本文、`docs/mocks-ui`配下、`docs/mocks/`。

## 6. 非目標

§5 に加えて、型・callback・event・payload・props名・module・export・wire・transport・decoder名を決めるか命名すること、(I)の問いに答えること、(T)を採番すること、`CU-0A08SSCI` を `DO` へ上げること、raw input/decode・Host transport・D2・drop終端を範囲へ入れること。

## 7. 必須負例

1. (I) の問いに答える、候補(A)(B)を並記して裁定する、契約形を1つ選ぶ。
2. bare `itemId` または JSX `identity` literal を scoped identity として肯定する／`S` を解消済みと書く。
3. 完全一致 `` `DO` `` を0件または2件以上にする／`CU-0A08SSCI` を `DO` へ上げる／前提(T)を採番する。
4. `CU-0A08SSCI` または `CU-0A08SSCI-I` を「発注依存証跡」へ追加する。
5. allowlist外の stale mirror（M3仕様、docs/README.md 他）を本粒で書き換える／過去裁定本文を改変する。
6. `ui/` 配下、guard、fixture、golden、threshold、`package.json`、lockfile を1 byteでも変える／`node_modules` を install する／test を skip・削除・期待値変更する／lint 抑制コメントを足す／TODO stub を残す。

## 8. 同期した current mirror

1. `docs/implementation-ledger.md` M3行（`| M3 | **VS-1 Rectangle配置とUndo** |` で始まる行）
2. `docs/implementation-ledger.md` 「M3への入場判定」末尾の運用判断散文（`したがって現在の短い運用判断は、`で始まる行）
3. `docs/decision-index.md` M3 VS-1 縦slice 総括行
4. `docs/decision-index.md` `CU-110S CU-110D ...` 行
5. `docs/decision-index.md` `CU-0A08RS0 ... CU-0A08SSCI ...` 系列行
6. `docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md` journal durability行
7. 同 selection / Undo再投影行

同じ7箇所を、docs-only `CU-0A08SSCI-I0` は `` `DONE` ``、次の唯一の `` `DO` `` はPRODUCT-ASSET/SPEC docs-only `CU-0A08SSCI-I`（1件）、`CU-0A08SSCI` は `` `WAIT` `` 継続、前提(T)は未採番、PRODUCT-ASSETの製品実装（コード変更を伴う）完全一致 `` `DO` `` は0件へ同期した。

## 9. allowlist外に残る stale mirror

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`
- 既存選定文書 §5 handoff 表の `CU-0A08SSCI-I0` 行（`DO` 表記のまま。過去裁定本文は変更しない）

## 10. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSCI-I0` | **DONE** | 前提(I)を `CU-0A08SSCI-I` として採番し、次粒の唯一の問いを固定 |
| `CU-0A08SSCI-I` | **DO** | 最小private input seamの契約形をdocs-onlyで裁定 |
| `CU-0A08SSCI` | **WAIT** | (I)/(T)未完了 |
| (T) | 未採番 | IDを与えない |

## 11. STOP条件

1. 問いに答える／契約形を選ぶ／型・event・payload・props名・decoder名・module pathを決めないと文が閉じない。
2. `ui/` の byte、guard期待値、`source-provenance.json` を変えないと oracle が緑にならない。
3. 公開API・Document・serde・永続形式・plugin契約・Place owner の変更が必要になる。
4. bare `itemId` または既存JSX `identity` literal を scoped identity として肯定しないと文が閉じない。
5. 「現在の並列レーン」の完全一致 `` `DO` `` を `CU-0A08SSCI-I` の1件に保てない。
6. allowlist外のfileを変更する必要が生じた。過去裁定本文を変えないと整合が取れない。
7. 前提(T)へIDを与える必要が生じた。
8. 既存選定文書 §3 の問いと逐語一致させられない、または新規docが既存選定文書と矛盾する「現行」記述になる。
