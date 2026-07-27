# CU-107R CU-110 required-responsibility scope decision
- 日付: 2026-07-28
- 状態: **決定**
- CU-107R: **DONE**

## 1. 目的
`CU-110` が必要とする `CU-107` 責任範囲を `CU-107` 全体ではなく厳密な部分集合として確定し、現況鏡像を同期して `CU-107N` へ閉集合を移す。

## 2. 事実

1. 対象 worktree: `BASE_REF` = `refs/heads/codex/cu107r-required-responsibility-20260728`、`BASE_SHA` = `bf604c63808605364c2553526ec615a02989374e`。着手時点の working tree は clean である。
2. [implementation ledger](../implementation-ledger.md) 「現在の並列レーン」で `CU-107R` 行の状態は完全一致 `DO`（1行のみ）。PRODUCT-ASSET lane の他行は `DONE` または `WAIT`（`U3a-2Q-V` / `CU-0A08BT` / `CU-0A08IT` / `U2c-2`）。
3. 同 [発注依存証跡](../implementation-ledger.md#発注依存証跡) に `CU-109` = `DONE`（PR #425、実装 commit `356d703f`、merge commit `32cf8902bf5c96fc60400a91335e72a9886cf304`）、`CU-G04SC` = `DONE`、`CU-110S` = `DONE`、`CU-110D` = `DONE`、`CU-107S` = `DONE`、`CU-107D` = `DONE` の一意行がある。`CU-107R` 行は着手時点では未登録であった。
4. [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) W0 表: `CU-107` = `CORE / WAIT`（目的: drag epoch / sequence / dedupe coordinator を製品 Host へ接続。依存: 既存 D&D spike、`CU-0B05`。完了条件: preview / terminal 配送、Esc / outside / capture loss、stale / duplicate を D2 未接続で検証）、`CU-110` = `CORE / WAIT`（依存: `CU-102/107/109`。完了条件: preview 中 D2 0、valid drop で `AddTrackItem` / `apply_macro` 各1、失敗 / cancel 0）、`CU-111` = `PRODUCT / WAIT`、`CU-0B05` = `E2E / WAIT`（依存: `CU-0B04N/R`）、`CU-102` = `SPEC / DONE`。
5. 採択済み Host Transient authority の上限は [歴史 D2 回収 §3.3](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#33-transportとdurabilityの分離) の1文だけ: dedupe key は Host Transient の `(webview_instance_epoch, drag_ordinal, event_sequence, layout_epoch)` **相当**とし、一 active drag、bounded terminal detail、高水位で eviction 後の再適用も拒否する。**exact wire は WebView Host contract 側で再固定する**。drag ID や epoch を D2、Document、journal へ保存しない。
6. 到達性 gate は [CU-106 selection consumer 分割決定](2026-07-27-cu-106-selection-consumer-split-decision.md) §3 の5項目を再利用する（`TimelineHit` 相当の non-test caller、production 入力到達、同一差分での producer + caller、lint 抑制 / dummy / `#[cfg(test)]` / env-gated smoke を製品 caller に数えないこと、公開契約不変）。`CU-110D` §3 B-2 の「non-test production drop source」も同じ意味で使う。
7. [CU-110D 決定](2026-07-28-cu-110d-cu-107-dependency-scope-decision.md) §2 が採録する read-only 調査事実として、製品 `crates/` / `ui/` 配下に Place intent 型・公開 API・production caller は存在せず、drag epoch / sequence / dedupe coordinator の製品 Host 接続実装も存在しない（D&D spike と test harness を除く）。本粒でこの調査をやり直さない。
8. [2026-07-21 rectangle drop D2 contract options](2026-07-21-m3-rectangle-drop-d2-contract-options.md) は options 文書であり、全面採択済みではない。§2 の回収表が示す採択範囲を越えて引用・確定化しない。
9. 着手時点で `./scripts/check-docs.sh` は `OK: docs整合チェック全項目通過`、`node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` は 118 tests / 118 pass / 0 fail。`ui/motolii-web/node_modules` は不在。`CU-107N` は `docs/` および `scripts/` のどこにも存在しない未使用 ID。
10. 本粒の判定に使ってよい根拠述語は `P1`〜`P4` のみ（[CU-107D 決定](2026-07-28-cu-107d-cu-110-required-responsibility-scope-decision.md) §3 D-2 理由2 の需要側閉集合）。
    - **P1**: `CU-110` の W0 完了条件（preview 中 D2 0、valid drop で `AddTrackItem` / `apply_macro` 各1、失敗 / cancel 0）
    - **P2**: `CU-102` の採択済み不変条件（fresh live-next 一致 + live 不在、live mint 0、失敗時 counter / history / revision 不変、既存 journal 互換）
    - **P3**: 採択済み Host Transient 不変条件 = 歴史 D2 回収 §3.3 の1文だけ（上記事実5）
    - **P4**: 到達性 gate = `CU-106` 決定 §3 の5項目（上記事実6）

## 3. 裁定
### R-1（答え）
`CU-110` が必要とする `CU-107` 責任範囲は **`CU-107` 全体ではなく厳密な部分集合**である。W0 `CU-107` 完了条件の8 clause のうち **7 clause が load-bearing、1 clause が not load-bearing** である。

### R-2（根拠の限定）
判定は §2 事実10 の `P1`〜`P4` だけから導く。他の source ID を根拠欄に書かない。

### R-3（`CU-0B05`）
`CU-0B05` は `CU-107` の**依存であって責任ではない**。本粒で解決済み・不要・射程内として扱わず、`WAIT` のまま据え置く。

### R-4（第8 clause の disposition）
「D2 未接続で検証」は `CU-110` が消費する load-bearing な**責任**ではなく、**接続前の検証 posture** である。黙って落とさず、`not load-bearing` として理由付きで明示 disposition する。

### R-6（自己整合）
R-1 が「厳密な部分集合」であることと、§4 の表に `not load-bearing` 行が (h) の1行だけ存在することは一致していなければならない。

### R-7（狭め方の限界）
本粒は**責任境界の限定**だけを書く。7 load-bearing clause を子粒へどう割るか、いくつに割るか、どの順で実装するかは書かない。`CU-107D` §3 D-5 の禁止集合をそのまま維持する。

## 4. 責任境界と除外
| # | CU-107 clause | 判定 | 根拠述語 | 理由（骨子） |
|---|---|---|---|---|
| (a) | preview 配送 | `load-bearing` | P1 | preview 配送が無ければ「preview 中 D2 0」を判定できる preview phase 自体が存在せず、P1 の当該条件が空虚に真になる。非空虚な preview D2=0 phase の成立に必要 |
| (b) | terminal 配送 | `load-bearing` | P1 / P2 | valid drop が単一の commit 境界へ到達する経路が無ければ、`AddTrackItem` / `apply_macro` 各1（P1）と P2 の原子性・失敗時不変を成立させられない |
| (c) | Esc | `load-bearing` | P1 | 認可された cancel terminal cause であり、その D2 件数が 0 であることが P1 の「失敗 / cancel 0」に直接含まれる |
| (d) | outside | `load-bearing` | P1 | 認可された cancel terminal cause であり、その D2 件数が 0 であることが P1 の「失敗 / cancel 0」に直接含まれる |
| (e) | capture loss | `load-bearing` | P1 | 認可された failure terminal cause であり、その D2 件数が 0 であることが P1 の「失敗 / cancel 0」に直接含まれる |
| (f) | stale | `load-bearing` | P1 / P2 / P3 | stale 抑制が無ければ at-most-once が崩れ、`AddTrackItem` / `apply_macro` が各1を超え得る。P3 の高水位 eviction 後再適用拒否と同じ責任面 |
| (g) | duplicate | `load-bearing` | P1 / P2 / P3 | duplicate 抑制が無ければ at-most-once が崩れ、`AddTrackItem` / `apply_macro` が各1を超え得る。P3 の一 active drag / dedupe key 相当と同じ責任面 |
| (h) | 「D2 未接続で検証」posture | `not load-bearing` | P1 | これは (a)〜(g) の**検証をいつどの接続状態で行うか**という posture であり、責任の中身ではない。`CU-110` は定義上 D2 を接続する粒であるため、この posture を外しても P1〜P4 のどの完了条件も導出不能にならない |

## 5. 非目標

- `CU-107` 配下の子粒の名前、個数、責任分担、実装順。7 clause から子粒数を示唆する記述も含む。
- event shape、WebView wire、exact dedupe tuple、verdict enum / 値、公開 API 名、visibility、bounded table size、閾値、rejection precedence。
- W0 表、`CU-110` 依存リストの書換え。
- `CU-107` / `CU-110` / `CU-111` の実装、promotion、`WAIT` 解除、`CU-0B05` の解決宣言。
- [2026-07-21 rectangle drop D2 contract options](2026-07-21-m3-rectangle-drop-d2-contract-options.md) を採択済みとして扱う、または同文書から exact wire を確定として引くこと。
- Rust / TypeScript / React / JSX / CSS / fixture / test / guard / schema の変更。
- 隣接チケット（`U3a-2Q-V` / `CU-0A08BT` / `CU-0A08IT` / `U2c-2` / `CU-106P` / `CU-106F` / `U2h-1P` / `CU-108` / `CU-5A03`）の状態変更。
- allowlist 外ファイルへの一切の変更。

## 6. 必須負例

1. R-1 の答えを「全体」へ変える、曖昧語（「概ね」「実質的に」「要検討」）で濁す、または書かない。
2. §4 の表が8行でない、行を併合 / 追加 / 削除する、判定を1行でも変える、根拠述語欄に `P1`〜`P4` 以外を書く、`not load-bearing` 行に理由が無い。
3. R-1（厳密な部分集合）と表（`not load-bearing` は (h) の1行のみ）が自己矛盾する。
4. (h) を黙って表から落とす、または責任として load-bearing 扱いにする。
5. `CU-107` の子粒名、個数、event 名、verdict 値、API 名、閾値、表サイズを発明する。7 clause を子粒数として数える。
6. `(webview_instance_epoch, drag_ordinal, event_sequence, layout_epoch)` を `相当` 抜きの確定 wire として書く。
7. [2026-07-21 rectangle drop D2 contract options](2026-07-21-m3-rectangle-drop-d2-contract-options.md) を採択済みとして引用する、またはそこから exact wire を確定として引く。
8. transport ID / drag epoch / layout epoch を D2 / Document / journal へ保存する余地を残す記述。
9. `CU-0B05` を解決済み・不要・射程内として扱う、または `CU-107` の依存から外す。
10. `CU-110` の依存リストを書き換える、`CU-107` を `DONE` 扱いする、`CU-110` / `CU-111` / `CU-0B05` の `WAIT` を解く。
11. test / dummy / smoke / `#[cfg(test)]` / lint 抑制 / env-gated smoke を到達性の証拠として数える記述。
12. `CU-107N` を発注依存証跡へ `DONE` として追加する（自分の order で後続を完了にする迂回）。
13. PRODUCT-ASSET の完全一致 `DO` を 0 件または 2 件以上にする。
14. M6 と M7 のうち片方だけ更新する。M8 の2行のうち片方だけ更新する。
15. §4-3 の8箇所9行のいずれかを更新し忘れる。
16. `docs/reviews/README.md` 索引登録の省略、または `docs/decision-index.md` への重複行作成。
17. 固定語彙外の状態語を新設する。
18. `check-docs.sh` を通すための索引除外、個別 ID 除外、lint 抑制、既存期待値・golden の書換え、fixture 特例。
19. `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` の期待値・件数の変更、または同 test の skip。
20. raw な文字列 / JSON 走査で typed 境界を迂回する記述、公開 raw mutation API の追加、重複 planner / helper の新設を提案する記述。
21. TODO stub、空節、「後で埋める」記述。
22. allowlist 外ファイルへの変更（`git status` に8ファイル目が現れる）。

## 7. STOP 条件

1. 着手時に `CU-107R` lane 行が完全一致 `DO` でない、または `CU-109` / `CU-G04SC` / `CU-110S` / `CU-110D` / `CU-107S` / `CU-107D` のいずれかが発注依存証跡で `DONE` でない。
2. AUTHORITY 行の SHA256 が worktree 実ファイルと一致しない。
3. §3 の確定裁定が、採択済み authority（W0 表、`CU-107D`、`CU-110D`、§3.3、`CU-102`、`CU-106` §3）のいずれかと矛盾していると読める。**書き換えず STOP する。**
4. 文書を成立させるために、子粒名・個数・event shape・WebView wire・exact dedupe tuple・API 名・verdict 値・visibility・表サイズ・閾値のいずれかを決める必要が出た。
5. §3.3 の1文を越えて exact wire を書かないと文書が成立しないように見えた。
6. P1〜P4 以外の述語、または未採択文書（2026-07-21 options 等）を根拠に加えないと表が書けない。
7. `CU-0B05` を解決済みとして扱う、または `CU-107` / `CU-110` / `CU-111` の `WAIT` を解かないと整合しない。
8. `CU-110` の依存リスト、W0 表、公開 API、Document、永続形式、plugin 契約のいずれかを変える必要が出た。
9. PRODUCT-ASSET `DO` が 0 件または 2 件以上にしか収まらない。
10. `CU-107N` 以外の後続粒 ID が必要に見えた、または `CU-107N` が既に別意味で使われていた。
11. allowlist の7ファイルだけでは §4-3 を満たせない。
12. gate を通すために、状態語彙の新設、索引の除外、個別 ID 除外、lint 抑制、既存期待値・golden・fixture の書換えが必要になった。

## 8. handoff
| ID | 状態 | 内容 |
|---|---|---|
| `CU-107R` | **DONE** | `CU-110` が必要とする `CU-107` 責任範囲は厳密な部分集合（8 clause 中 7 が load-bearing） |
| `CU-107N` | **DO** | 狭い名前付き前提の閉集合と個数の裁定（docs-only） |
| `CU-107` | **WAIT** | 据え置き |
| `CU-110` | **WAIT** | 据え置き、non-test production drop source 待ち |
| `CU-111` | **WAIT** | 据え置き |
| `CU-0B05` | **WAIT** | 据え置き、未解決の依存 |
