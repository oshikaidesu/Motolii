# CU-107D CU-110 が必要とする CU-107 責任範囲の先行限定

- 日付: 2026-07-28
- 状態: **決定**
- CU-107D: **DONE**

## 1. 目的

[CU-107S 選定](2026-07-28-cu-107s-split-concretization-scope-selection.md) §3 が送った唯一の問い——`CU-107` を狭い名前付き前提へ具体化するにあたり、最初に docs-only で閉じる裁定は **(i) 狭い名前付き前提の閉集合と個数** か、それとも **(ii) `CU-110` が必要とする `CU-107` 責任範囲の限定** か——へ答えを書き、`CU-107D` を閉じる。実装・promotion・`WAIT` 解除は一切しない。

## 2. 事実

1. [implementation ledger](../implementation-ledger.md) 「現在の並列レーン」で `CU-107D` 行の状態は完全一致 `DO`（1行のみ）。PRODUCT-ASSET lane で `DO` は着手時点でこの1件だけであり、他行は `DONE` または `WAIT`（`U3a-2Q-V` / `CU-0A08BT` / `CU-0A08IT` / `U2c-2`）。
2. 同 [発注依存証跡](../implementation-ledger.md#発注依存証跡) に `CU-109` = `DONE`（PR #425、実装 commit `356d703f`、merge commit `32cf8902bf5c96fc60400a91335e72a9886cf304`）、`CU-G04SC` = `DONE`、`CU-110S` = `DONE`、`CU-110D` = `DONE`、`CU-107S` = `DONE` の一意行がある。`CU-107D` 行は着手時点では未登録であった。
3. [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) W0 表で `CU-107` = `CORE / WAIT`（目的: drag epoch / sequence / dedupe coordinator を製品 Host へ接続。依存: 既存 D&D spike、`CU-0B05`。完了条件: preview / terminal 配送、Esc / outside / capture loss、stale / duplicate を D2 未接続で検証）、`CU-110` = `CORE / WAIT`（依存: `CU-102/107/109`）、`CU-111` = `PRODUCT / WAIT`、`CU-0B05` = `E2E / WAIT`（依存: `CU-0B04N/R`）である。
4. 同 W0 表で `CU-110` の完了条件は「preview 中 D2 0、valid drop で `AddTrackItem` / `apply_macro` 各1、失敗 / cancel 0」である。
5. `CU-102` は発注依存証跡および W0 表（`SPEC / DONE`）で `DONE` であり、fresh live-next 一致 + live 不在、live mint 0、失敗時 counter / history / revision 不変、既存 journal 互換を採択済みの不変条件として持つ。
6. 採択済み Host Transient authority の上限は [歴史 D2 回収 §3.3](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#33-transportとdurabilityの分離) の1文だけ: dedupe key は `(webview_instance_epoch, drag_ordinal, event_sequence, layout_epoch)` **相当**とし、一 active drag、bounded terminal detail、高水位で eviction 後の再適用も拒否する。**exact wire は WebView Host contract 側で再固定する**。drag ID や epoch を D2 / Document / journal へ保存しない。
7. [2026-07-21 rectangle drop D2 contract options](2026-07-21-m3-rectangle-drop-d2-contract-options.md) は options 文書であり、全面採択済みではない。§2 の回収表が示す採択範囲を越えて引用・確定化しない。
8. 到達性 gate は [CU-106 selection consumer 分割決定](2026-07-27-cu-106-selection-consumer-split-decision.md) §3 の5項目を再利用する。lint 抑制、dummy 参照、`#[cfg(test)]` 到達性、env-gated smoke を製品 caller として数えない。`CU-110D` §3 B-2 の「non-test production drop source」も同じ意味で使う。
9. [CU-110D 決定](2026-07-28-cu-110d-cu-107-dependency-scope-decision.md) §3 B-3 が `CU-107S` へ繰り延べた集合は、狭い前提の名前・個数・責任分担・実装順、および W0 表と `CU-110` 依存リストの実際の書換えである。`CU-107S` はそのうち「次に一問だけ閉じる範囲」の選定だけを行い、[CU-107S 選定](2026-07-28-cu-107s-split-concretization-scope-selection.md) §4 に候補 (A) 閉集合と個数を先に裁定 / (B) `CU-110` が必要とする責任範囲の限定を先に裁定 を優劣なしで並べた。

## 3. 裁定

**D-1**: [CU-107S 選定](2026-07-28-cu-107s-split-concretization-scope-selection.md) §4 の候補 **(B)** を明示採択する。すなわち `CU-107` 全体のうち `CU-110` が実際に依存する責任だけを先に限定し、狭い名前付き前提の閉集合と個数は後続へ送る。

**D-2**: 理由は次の二つであり、両方書く。

- **理由1（供給側に閉包述語がない）**: `CU-107` の依存 `CU-0B05` は W0 表で `WAIT`（依存: `CU-0B04N/R`）であり未解決である。したがって `CU-107` 側には「何をもって閉じたと言えるか」の述語が採択済み authority に存在せず、狭い名前付き前提の個数を先に決めれば「もっともらしいデフォルト」を発明することになる。これは [AGENTS.md](../../AGENTS.md)「常時規律」「計画と実装」の、実在targetを推測せず未決契約をコードで補わない規律に反する。
- **理由2（需要側には述語がある）**: 一方、需要側は採択済み事実だけで述語を組める。`CU-110` の既存 W0 完了条件（preview 中 D2 0、valid drop で `AddTrackItem` / `apply_macro` 各1、失敗 / cancel 0）と、`CU-102` の採択済み不変条件と、§3.3 の採択済み Host Transient 不変条件（`相当` 表現、一 active drag、bounded terminal detail、高水位 eviction 後再適用拒否、exact wire は WebView Host contract 側）が揃っており、新しい意味を発明せずに「`CU-110` が必要とする責任」を判定できる。

**D-3**: (B) の採択は順序の裁定であって、範囲を狭める判定ではない。後続 `CU-107R` が authority を再照合した結果、`CU-110` が必要とする責任範囲は `CU-107` 全体であると答えることは正当であり、本粒はその答えを先取りしない。「狭くなる」ことを結論として書かない。

**D-4**: 「`CU-110` が必要とする」の判定基準は、製品 `crates/` / `ui/` 上の production caller の存在とする（`CU-110D` §3 B-2 と `CU-106` §3 の5項目に依拠）。test / dummy / smoke / lint 抑制を到達性の証拠に数えない。

**D-5**: 本粒は責任範囲の**限定という順序**だけを決める。子粒の名前・個数・責任分担・実装順、event shape、WebView wire、exact dedupe tuple、verdict enum / 値、公開 API 名、visibility、bounded table size、閾値、rejection precedence は決めない。W0 表と `CU-110` 依存リストは一切書き換えない。`CU-107` / `CU-110` / `CU-111` / `CU-0B05` の `WAIT` は据え置く。

## 4. 非目標

- `CU-107` 配下の子粒の名前、個数、責任分担、実装順を書くこと。
- event shape、WebView wire、exact dedupe tuple、verdict enum / 値、公開 API 名、visibility、bounded table size、閾値、rejection precedence の決定。
- `CU-110` の既存依存リスト（`CU-102/107/109`）の書換え・削除・再解釈。
- 快適利用粒度化 W0 表の編集（`CU-107` / `CU-110` / `CU-111` / `CU-0B05` 行を含め一切禁止）。
- `CU-107` / `CU-110` / `CU-111` の実装、promotion、`WAIT` 解除、`CU-0B05` の解決宣言。
- 2026-07-21 rectangle drop D2 contract options を全面採択済みとして扱う、または同文書から exact wire を確定として引くこと。
- Rust / TypeScript / React / fixture / test / guard / schema / Document / journal / 公開 API / plugin 契約の変更。
- 既存 decision 文書・発注依存証跡の既存行の意味の書換え。
- 隣接チケット（`U3a-2Q-V` / `CU-0A08BT` / `CU-0A08IT` / `U2c-2` / `CU-106P` / `CU-106F` / `U2h-1P`）の状態変更。
- allowlist 外ファイルへの一切の変更。

## 5. STOP 条件

1. 着手時に `CU-107D` lane 行が完全一致 `DO` でない、または `CU-109` / `CU-G04SC` / `CU-110S` / `CU-110D` / `CU-107S` のいずれかが発注依存証跡で `DONE` でない。
2. §3 の裁定を書くために、子粒名・個数・event shape・WebView wire・API 名・verdict 値・visibility・表サイズ・閾値のいずれかを決める必要が出た。
3. 採択済み Host Transient 不変条件（§3.3 の1文）を越えて exact wire を書かないと文書が成立しないように見えた。
4. `CU-110` の依存リスト、W0 表、公開 API、Document、永続形式、plugin 契約のいずれかを変える必要が出た。
5. `CU-0B05` を解決済みとして扱う、または `CU-107` / `CU-110` / `CU-111` の `WAIT` を解かないと整合しない。
6. PRODUCT-ASSET `DO` が 0 件または 2 件以上にしか収まらない。
7. `CU-107R` 以外の後続粒 ID が必要に見えた、または `CU-107R` が既に別意味で使われていた。
8. allowlist の7ファイルだけでは要求を満たせない（特に快適利用粒度化の編集が必要に見えた場合）。
9. `./scripts/check-docs.sh` または reference guard を通すために、状態語彙の新設、索引の除外、個別 ID 除外、lint 抑制、既存期待値・golden の書換えが必要になった。

## 6. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-107D` | **DONE** | 候補 (B) を採択し、`CU-110` が必要とする `CU-107` 責任範囲の限定を先に閉じる順序を裁定 |
| `CU-107R` | **DO** | `CU-110` が必要とする `CU-107` 責任範囲の限定（docs-only） |
| `CU-107` | **WAIT** | 据え置き |
| `CU-110` | **WAIT** | 据え置き、non-test production drop source 待ち |
| `CU-111` | **WAIT** | 据え置き |
