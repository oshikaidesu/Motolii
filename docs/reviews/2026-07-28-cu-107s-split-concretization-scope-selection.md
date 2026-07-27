# CU-107S CU-107 分割具体化範囲の選定

- 日付: 2026-07-28
- 状態: **決定**
- CU-107S: **DONE**

## 1. 目的

`CU-110D` が候補 **(B)**（`CU-107` を狭い名前付き前提へ分割する方針）を採択したため、その具体化に必要な次の docs-only 裁定範囲を一問へ限定して選定する。本粒は答えを出さない。

## 2. 事実

- [implementation ledger](../implementation-ledger.md) の PRODUCT-ASSET lane では、状態が完全一致 `DO` の行は docs-only `CU-107S` の1件だけであった。他の PRODUCT-ASSET 行は `DONE` または `WAIT`（`U3a-2Q-V` / `CU-0A08BT` / `CU-0A08IT` / `U2c-2`）である。
- [発注依存証跡](../implementation-ledger.md#発注依存証跡) には `CU-109`（PR #425、実装 commit `356d703f`、merge commit `32cf8902bf5c96fc60400a91335e72a9886cf304`）、`CU-G04SC`、`CU-110S`、`CU-110D` の一意な `DONE` 行がある。`CU-107S` 行は着手時点では未登録であった。
- [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) の W0 表で、`CU-107` = `CORE / WAIT`（依存: 既存 D&D spike、CU-0B05）、`CU-110` = `CORE / WAIT`（依存: `CU-102/107/109`）、`CU-111` = `PRODUCT / WAIT` である。`CU-0B05` = `E2E / WAIT` である。
- `CU-102` と `CU-109` は発注依存証跡で `DONE`。`CU-107` は W0 表で `WAIT` のまま（`CU-0B05` 待ち）である。
- 採択済み authority の上限は [歴史 D2 回収 §3.3](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#33-transportとdurabilityの分離) の1文だけ: dedupe key は Host Transient の `(webview_instance_epoch, drag_ordinal, event_sequence, layout_epoch)` **相当**とし、一 active drag、bounded terminal detail、高水位で eviction 後の再適用も拒否する。**exact wire は WebView Host contract 側で再固定する**。drag ID や epoch を D2、Document、journal へ保存しない。
- [rectangle drop D2 contract options](2026-07-21-m3-rectangle-drop-d2-contract-options.md) は options 文書であり、全面採択済みではない。§2 の回収表が示す採択範囲を越えて引用・確定化しない。
- 到達性 gate は [CU-106 selection consumer 分割決定](2026-07-27-cu-106-selection-consumer-split-decision.md) §3 の5項目を再利用する。lint 抑制、dummy 参照、`#[cfg(test)]` 到達性、env-gated smoke を製品 caller として数えない。[CU-110D 決定](2026-07-28-cu-110d-cu-107-dependency-scope-decision.md) §3 B-2 の「non-test production drop source」もこの gate と同じ意味で使う。
- [CU-110D 決定](2026-07-28-cu-110d-cu-107-dependency-scope-decision.md) §3 B-3 が `CU-107S` へ繰り延べた集合は、狭い前提の名前・個数・責任分担・実装順、および W0 表と `CU-110` 依存リストの実際の書換えである。本粒はこの集合のどれも決めず、このうち「次に一問だけ閉じる範囲」を選ぶ。

## 3. CU-107D が閉じる唯一の問い

以下は次粒の問いであり、本粒の回答ではない。

`CU-107` を狭い名前付き前提へ具体化するにあたり、最初に docs-only で閉じる裁定は、**(i) 狭い名前付き前提の閉集合と個数**か、それとも **(ii) `CU-110` が必要とする `CU-107` 責任範囲の限定**か。

## 4. 可能な候補（優劣を付けない）

**(A) 閉集合と個数を先に裁定する。**

次の docs-only 粒で、狭い名前付き前提の閉集合と個数だけを先に確定する案。**本粒ではその名前も個数も書かない。**

**(B) `CU-110` が必要とする責任範囲の限定を先に裁定する。**

`CU-107` 全体のうち `CU-110` が実際に依存する責任だけを先に絞り、閉集合と個数を後続へ送る案。**本粒ではその責任の中身を書かない。**

## 5. 非目標

- 上記の一つの問いへの答えを書くこと。
- `CU-107` 配下の子粒の名前、個数、責任分担、実装順を書くこと。
- event shape、WebView wire、exact dedupe tuple、verdict enum / 値、公開 API 名、visibility、bounded table size、閾値、rejection precedence を決めること。
- `CU-110` の既存依存リスト（`CU-102/107/109`）の書換え・削除・再解釈。W0 表の書換え。
- `CU-107` / `CU-110` / `CU-111` の実装、promotion、`WAIT` 解除、`CU-0B05` の解決宣言。
- [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) の編集。
- [rectangle drop D2 contract options](2026-07-21-m3-rectangle-drop-d2-contract-options.md) を全面採択済みとして扱う、または同文書から exact wire を確定として引くこと。
- Rust / TypeScript / React / fixture / test / guard / schema / Document / journal / 公開 API / plugin 契約の変更。
- 既存 decision 文書・発注依存証跡の既存行の意味の書換え。
- allowlist 外ファイルへの一切の変更。

## 6. 必須負例

- 候補 (A)/(B) のどちらかを推奨・採用・「有力」「望ましい」等と書く、または片方だけ厚く書いて実質的に選好を出す。
- 一つの問いを2問以上へ増やす、または `CU-107D` に追加の裁定対象を足す。
- `CU-107` の子粒名、個数、event 名、verdict 値、API 名、閾値、表サイズを発明する。
- `(webview_instance_epoch, drag_ordinal, event_sequence, layout_epoch)` を「相当」抜きの確定 wire として書く、または 2026-07-21 options 文書を採択済みとして引用する。
- transport ID / drag epoch を D2 / Document / journal へ保存する余地を残す記述。
- `CU-110` の依存リストを書き換える、`CU-107` や `CU-0B05` を `DONE` / 解決済み扱いする、`CU-110` の `WAIT` を解く。
- `CU-107D` を発注依存証跡へ `DONE` として追加する（自分の order で後続を完了にする迂回）。
- PRODUCT-ASSET `DO` を 0 件または 2 件以上にする。
- 既存 decision 文書または発注依存証跡の既存行の意味を書き換える。
- [reviews 索引](README.md) または [decision-index](../decision-index.md) への登録を省く、あるいは decision-index へ重複行を作る。
- 固定語彙外の状態語を新設する。
- test / dummy / smoke / lint 抑制を到達性の証拠として数える記述。
- lint 抑制、テスト期待値・golden の書換え、fixture 特例、guard の個別 ID 除外。
- allowlist 外ファイルへの変更、TODO stub、隣接チケットへの拡張。

## 7. STOP 条件

1. 選定文書を書くために、一つの問いの答えを決めなければならなくなった。
2. `CU-110` の依存リスト、W0 表、公開 API、Document、永続形式、plugin 契約のいずれかを変える必要が出た。
3. PRODUCT-ASSET `DO` を 0 件または 2 件以上にしないと整合しない。
4. allowlist の7ファイルだけでは要求を満たせない（特に [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) の編集が必要に見えた場合）。
5. `CU-107S` lane 行が `DO` でない、または `CU-109` / `CU-G04SC` / `CU-110S` / `CU-110D` のいずれかが発注依存証跡で `DONE` でないことを着手時に発見した。
6. 採択済み Host Transient 不変条件を越えて exact wire を書かないと文書が成立しないように見えた。

## 8. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-107S` | **DONE** | `CU-107` 分割具体化の未決範囲を docs-only `CU-107D` へ選定 |
| `CU-107D` | **DO** | 閉集合と個数を先に閉じるか、`CU-110` が必要とする責任範囲の限定を先に閉じるかを一問だけ裁定 |
| `CU-107` | **WAIT** | 据え置き |
| `CU-110` | **WAIT** | 据え置き、non-test production drop source 待ち |
| `CU-111` | **WAIT** | 据え置き |
