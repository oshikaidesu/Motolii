# CU-107N CU-107 狭い名前付き前提の閉集合
- 日付: 2026-07-28
- 状態: **決定**
- CU-107N: **DONE**

## 1. 目的

`CU-107R` が `load-bearing` と確定した7 clause（preview 配送 / terminal 配送 / Esc / outside / capture loss / stale / duplicate）を、**実装可能で狭い名前付き前提の閉集合**へ分割し、(1) 個数、(2) 各 clause の単一 owner 割当、(3) 依存順、(4) 次の唯一の PRODUCT-ASSET `DO` を docs へ確定する。

## 2. 事実

1. 対象 worktree の `BASE_REF` = `refs/heads/codex/cu107n-prerequisite-set-20260728`、`BASE_SHA` = `417da301157c68fd6fc5204c6baba23e7abd1995`。着手時 working tree は clean。
2. `docs/implementation-ledger.md` 「現在の並列レーン」で `CU-107N` 行の状態は完全一致 `DO`（1行のみ）。PRODUCT-ASSET lane の他行は `DONE` または `WAIT`（`U3a-2Q-V` / `CU-0A08BT` / `CU-0A08IT` / `U2c-2`）。
3. 同「発注依存証跡」に `CU-109` = `DONE`（PR #425、実装 commit `356d703f`、merge commit `32cf8902bf5c96fc60400a91335e72a9886cf304`）、`CU-G04SC` = `DONE`、`CU-110S` = `DONE`、`CU-110D` = `DONE`、`CU-107S` = `DONE`、`CU-107D` = `DONE`、`CU-107R` = `DONE` の一意行がある。`CU-107N` 行は着手時点では未登録。
4. `CU-107R` §3 R-1 / §4: W0 `CU-107` 完了条件の8 clause のうち (a) preview 配送 / (b) terminal 配送 / (c) Esc / (d) outside / (e) capture loss / (f) stale / (g) duplicate の7つが `load-bearing`、(h)「D2 未接続で検証」posture は `not load-bearing`。
5. `CU-107R` §3 R-7: 7 clause の子粒への割り方・個数・実装順は `CU-107R` では決めておらず、本粒へ繰り延べられている。
6. 判定に使ってよい根拠述語は `P1`〜`P4` のみ（`CU-107R` §2 事実10）。**P1** = `CU-110` の W0 完了条件（preview 中 D2 0、valid drop で `AddTrackItem` / `apply_macro` 各1、失敗 / cancel 0）。**P2** = `CU-102` の採択済み不変条件。**P3** = 採択済み Host Transient 不変条件 = 歴史 D2 回収 §3.3 の1文だけ（dedupe key は `(webview_instance_epoch, drag_ordinal, event_sequence, layout_epoch)` **相当**、一 active drag、bounded terminal detail、高水位 eviction 後の再適用も拒否、**exact wire は WebView Host contract 側で再固定**、drag ID / epoch を D2・Document・journal へ保存しない）。**P4** = 到達性 gate = `CU-106` 決定 §3 の5項目。
7. `CU-0B05` は W0 表で `WAIT`（依存: `CU-0B04N/R`）であり未解決。`CU-107` は `CU-0B05` に依存する（`CU-107R` R-3）。
8. `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` / `CU-107W` は `docs/` および `scripts/` のどこにも存在しない未使用 ID。
9. 着手時点で `./scripts/check-docs.sh` は `OK: docs整合チェック全項目通過`、`node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` は 118 tests / 118 pass / 0 fail。
10. `docs/mocks-ui/node_modules` および `ui/motolii-web/node_modules` は不在であり、`npm run test:reference-guard` は着手前から `@babel/parser` の `ERR_MODULE_NOT_FOUND` で9 suite中6 suiteが失敗する。これは本粒の変更と無関係の既存環境事実であり、**本粒で修復・install・回避しない**。

## 3. 裁定

### N-1（答え）
`CU-110` が必要とする7 load-bearing clause は、**ちょうど4件**の狭い名前付き前提からなる閉集合へ分割する。

### N-2（個数4の導出。2件/3件を既定にしない）
個数は次の4理由から導かれるのであって、既定値ではない。
- (c) Esc / (d) outside / (e) capture loss は出力責任が同一（候補 terminal への原因分類）であり、別 owner へ分けると同じ分類責任を持つ重複 owner を作る。よって1 owner。
- (f) stale / (g) duplicate は出力責任が同一（admitted を高々1件に抑える）であり、`P3` の一 active drag / eviction 後再適用拒否と同じ責任面。よって1 owner。
- (a) preview 配送 と (b) terminal 配送 は `P1` の述語が異なり（preview 中 D2 = 0 と valid drop で各1）、lifecycle 位相も入力責任も異なる。よって併合しない。
- (b) を (f)(g) と同一 owner にすると「terminal を正確に1回」が単一粒の責任へ戻り、at-most-once の owner と重複する。よって併合しない。

### N-3（割当は重複0・欠落0）
§4-2 の7 clause は §4 の4 ID のいずれかちょうど1つへ割り当てられる。1 clause が2 owner に現れてはならず、owner の無い clause があってはならない。

### N-4（責任重複の禁止規則）
各前提は**自分の出力責任だけ**を主張し、上流・下流の前提が持つ保証を再主張しない。「terminal を正確に1回」を単一前提の責任として書かない。exactly-once は `CU-107AD` の at-most-once と `CU-107TD` の「admit ごとにちょうど1回配送」の**合成としてのみ**成立する。

### N-5（依存順）
`CU-107PV` → `CU-107TC` → `CU-107AD` → `CU-107TD` の一本鎖とする。これは候補 terminal → 分類 → admission → accepted delivery の因果と一致する。すなわち **admission は accepted terminal 配送より前**、かつ**原因分類より後**である。分類前の admission、および admission 前の accepted 配送を許す順序を書かない。

### N-6（次の唯一の PRODUCT-ASSET `DO`）
docs-only `CU-107W`（W0 表と `CU-110` 依存リストを本閉集合の名前へ書き換える裁定。`CU-110D` §3 B-3 が繰り延べた残件）。本粒では `CU-107W` の**中身は決めない**。4前提の実装粒は `CU-107` が `CU-0B05` 待ちである限り `WAIT` であり、**本粒で lane 表へ登録しない**。

### N-7（据え置き）
`CU-0B05` は `CU-107` の依存のまま `WAIT`。`CU-107` / `CU-110` / `CU-111` の `WAIT` を解かない。W0 表と `CU-110` 依存リストは本粒で一切書き換えない。

### N-8（自己整合）
§4 は見出し行を除きちょうど4行、§4-2 は見出し行を除きちょうど7行、§4-2 の owner 列に現れる ID は §4 の4 ID だけであり、4 ID すべてが §4-2 に最低1回現れる。

## 4. 閉集合

| 前提ID | 名前 | 入力責任 | 出力責任 | 依存 | 根拠述語 |
|---|---|---|---|---|---|
| `CU-107PV` | preview 配送 | 一 active drag の非 terminal な preview 進行を受け取る | 非空虚な preview phase が存在し、preview 配送が terminal を生じさせずに完結すること | なし（閉集合の先頭） | P1 |
| `CU-107TC` | 候補 terminal の原因分類 | `CU-107PV` が確立した active drag に対して生じた候補 terminal | 各候補 terminal へ、認可済みの非 commit 原因（Esc / outside / capture loss）のちょうど一つを付すか、そのいずれでもないと分類すること。分類は排他かつ網羅 | `CU-107PV` | P1 |
| `CU-107AD` | admission（at-most-once） | `CU-107TC` が「認可済み非 commit 原因のいずれでもない」と分類した候補 terminal だけ | 一 active drag につき admitted を高々1件に抑え、stale および duplicate の候補を admit しないこと | `CU-107TC` | P1 / P2 / P3 |
| `CU-107TD` | accepted terminal 配送 | `CU-107AD` が admit した terminal だけ | admit ごとにちょうど1回、単一の下流 commit 境界へ配送し、admit されていない候補を配送しないこと | `CU-107AD` | P1 / P2 |

### 4-2 clause → owner 割当

| CU-107 clause | 単一 owner |
|---|---|
| (a) preview 配送 | `CU-107PV` |
| (b) terminal 配送 | `CU-107TD` |
| (c) Esc | `CU-107TC` |
| (d) outside | `CU-107TC` |
| (e) capture loss | `CU-107TC` |
| (f) stale | `CU-107AD` |
| (g) duplicate | `CU-107AD` |

## 5. 非目標

- exact wire、event shape、WebView contract、exact dedupe tuple、verdict enum / 値、公開 API 名、visibility、bounded table size、閾値、rejection precedence の決定。
- W0 表、`CU-110` 依存リスト、`CU-107` / `CU-110` / `CU-111` / `CU-0B05` の状態の書換え。
- `CU-107W` の中身（どの行をどう書き換えるか）の決定。
- 4前提の実装、promotion、lane 表への実装粒行の追加、`WAIT` 解除、`CU-0B05` の解決宣言。
- 2026-07-21 rectangle drop D2 contract options を採択済みとして扱う、または同文書から exact wire を確定として引くこと。
- Rust / TypeScript / React / JSX / CSS / fixture / test / guard / schema / Document / journal / 公開 API / plugin 契約の変更。
- 隣接チケット（`U3a-2Q-V` / `CU-0A08BT` / `CU-0A08IT` / `U2c-2` / `CU-106P` / `CU-106F` / `U2h-1P` / `CU-108` / `CU-5A03` / `GAP-25` / `G0-6H`）の状態変更。
- 既存 decision 文書・発注依存証跡の既存行の意味の書換え。
- allowlist 外ファイルへの一切の変更。

## 6. 必須負例

1. §4 が4行でない、§4-2 が7行でない、列を増減する、`CU-107R` §4 の判定を1行でも書き換える。
2. 1つの clause を2つ以上の owner へ割り当てる、または owner の無い clause を残す。
3. exactly-once を単一前提の出力責任として書く（`CU-107TD` に at-most-once を、`CU-107AD` に配送保証を持たせる記述を含む）。
4. 依存順を `CU-107PV` → `CU-107TC` → `CU-107AD` → `CU-107TD` 以外にする、または admission を accepted 配送より後・原因分類より前に置く記述。
5. 個数を「概ね」「実質的に」「2〜3件」等の曖昧語で濁す、または個数の導出（N-2 の4理由）を落とす。
6. `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` 以外の前提 ID を新設する、または5件以上・3件以下へ変える。
7. event 名、verdict 値、API 名、閾値、表サイズ、WebView wire、exact dedupe tuple を発明する。
8. `(webview_instance_epoch, drag_ordinal, event_sequence, layout_epoch)` を `相当` 抜きの確定 wire として書く。
9. transport ID / drag epoch / layout epoch を D2 / Document / journal へ保存する余地を残す記述。
10. `CU-0B05` を解決済み・不要・射程内として扱う、または `CU-107` の依存から外す。
11. `CU-110` の依存リストまたは W0 表を書き換える、`CU-107` を `DONE` 扱いする、`CU-110` / `CU-111` / `CU-0B05` の `WAIT` を解く。
12. 根拠述語欄に `P1`〜`P4` 以外を書く、または未採択文書（2026-07-21 options 等）を根拠に加える。
13. test / dummy / smoke / `#[cfg(test)]` / lint 抑制 / env-gated smoke を到達性の証拠として数える記述。
14. `CU-107W` を発注依存証跡へ `DONE` として追加する（自分の order で後続を完了にする迂回）。
15. PRODUCT-ASSET lane の完全一致 `DO` を 0 件または 2 件以上にする。lane へ `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` の行を追加する。
16. §5 の8箇所のいずれかを更新し忘れる。特に `docs/decision-index.md` の31行目と45行目のうち片方だけ、または vertical-slice 決定の107行目と110行目のうち片方だけを更新する。
17. `docs/reviews/README.md` 索引登録の省略、または `docs/decision-index.md` への重複行作成。
18. 固定語彙（決定 / 縮小採用 / 延期 / 棄却 / 撤回 / 未統一 / 観察 / 比較中 / 停止線）外の状態語を新設する。
19. `check-docs.sh` や guard を通すための索引除外、個別 ID 除外、lint 抑制、既存期待値・golden の書換え、fixture 特例。
20. `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` の期待値・件数の変更、または同 test の skip。
21. `node_modules` の install、`package.json` の変更、環境依存で既に失敗している6 suite の修復・迂回。
22. raw な文字列 / JSON 走査で typed 境界を迂回する記述、公開 raw mutation API の追加、重複 planner / helper の新設を提案する記述。
23. TODO stub、空節、「後で埋める」記述。
24. allowlist 外ファイルへの変更。

## 7. STOP 条件

1. 着手時に `CU-107N` lane 行が完全一致 `DO` でない、または `CU-109` / `CU-G04SC` / `CU-110S` / `CU-110D` / `CU-107S` / `CU-107D` / `CU-107R` のいずれかが発注依存証跡で `DONE` でない。
2. AUTHORITY 行の SHA256 が worktree 実ファイルと一致しない。
3. §3 の確定裁定が、採択済み authority（W0 表、`CU-107D`、`CU-110D`、`CU-107R`、§3.3、`CU-102`、`CU-106` §3）のいずれかと矛盾していると読める。**書き換えず STOP する。**
4. 文書を成立させるために、event shape・WebView wire・exact dedupe tuple・API 名・verdict 値・visibility・表サイズ・閾値のいずれかを決める必要が出た。
5. §3.3 の1文を越えて exact wire を書かないと文書が成立しないように見えた。
6. `P1`〜`P4` 以外の述語、または未採択文書を根拠に加えないと §4 が書けない。
7. 4前提のいずれかが、7 clause のどれとも対応しない責任、または他前提と重複する責任を持たないと書けない。
8. `CU-0B05` を解決済みとして扱う、または `CU-107` / `CU-110` / `CU-111` の `WAIT` を解かないと整合しない。
9. `CU-110` の依存リスト、W0 表、公開 API、Document、永続形式、plugin 契約のいずれかを変える必要が出た。
10. PRODUCT-ASSET の完全一致 `DO` が 0 件または 2 件以上にしか収まらない。
11. `CU-107W` 以外の後続粒 ID が必要に見えた、または `CU-107W` / `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` のいずれかが既に別意味で使われていた。
12. allowlist の7ファイルだけでは §5 を満たせない。
13. gate を通すために、状態語彙の新設、索引除外、個別 ID 除外、lint 抑制、既存期待値・golden・fixture の書換え、`node_modules` の install が必要になった。

## 8. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-107N` | **DONE** | 7 load-bearing clause を4前提の閉集合へ分割し、単一 owner 割当と依存順を確定 |
| `CU-107W` | **DO** | W0 表と `CU-110` 依存リストを本閉集合の名前へ書き換える裁定（docs-only） |
| `CU-107PV` | **WAIT** | `CU-107` 経由で `CU-0B05` 待ち |
| `CU-107TC` | **WAIT** | `CU-107` 経由で `CU-0B05` 待ち |
| `CU-107AD` | **WAIT** | `CU-107` 経由で `CU-0B05` 待ち |
| `CU-107TD` | **WAIT** | `CU-107` 経由で `CU-0B05` 待ち |
| `CU-107` | **WAIT** | 据え置き |
| `CU-110` | **WAIT** | 据え置き |
| `CU-111` | **WAIT** | 据え置き |
| `CU-0B05` | **WAIT** | 据え置き、未解決の依存 |
