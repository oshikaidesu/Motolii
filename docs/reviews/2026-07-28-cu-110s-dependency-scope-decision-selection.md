# CU-110S CU-110 前提範囲（CU-107 依存）判断の選定

- 日付: 2026-07-28
- 状態: **決定**
- CU-110S: **DONE**

## 1. 目的

`CU-110` 発注前提の未決範囲だけを docs-only 粒 `CU-110D` へ選定する。本粒は答えを出さない。

## 2. 事実

- [implementation ledger](../implementation-ledger.md) の PRODUCT-ASSET lane では、状態が完全一致 `DO` の行は docs-only `CU-110S` の1件だけであった。他の PRODUCT-ASSET 行は `DONE` または `WAIT`（`U3a-2Q-V` / `CU-0A08BT` / `CU-0A08IT` / `U2c-2`）である。
- [発注依存証跡](../implementation-ledger.md#発注依存証跡) には `CU-109`（PR #425、実装 commit `356d703f`、merge commit `32cf8902bf5c96fc60400a91335e72a9886cf304`）と `CU-G04SC` の一意な `DONE` 行がある。`CU-110S` 行は着手時点では未登録であった。
- [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) の W0 表で、`CU-107` = `CORE / WAIT`（依存: 既存 D&D spike、CU-0B05）、`CU-110` = `CORE / WAIT`（依存: `CU-102/107/109`）、`CU-111` = `PRODUCT / WAIT` である。
- `CU-102` と `CU-109` は発注依存証跡で `DONE`。`CU-107` は W0 表で `WAIT` のまま（`CU-0B05` 待ち）である。
- read-only 調査では、製品 `crates/` および `ui/` 配下に Place intent 型・公開 API・production caller が存在せず、drag epoch/sequence/dedupe coordinator の製品 Host 接続実装も存在しない（D&D spike と test harness を除く）。

## 3. CU-110D が閉じる唯一の問い

以下は次粒の問いであり、本粒の回答ではない。

`CU-110` は `CU-107` 全体の完了を待つ必要があるのか、それとも `CU-110` の既存依存を `CU-107` 配下のより狭い名前付き前提へ分割すべきか。

## 4. 可能な候補（優劣を付けない）

**(A) `CU-107` 全体先行を維持する。**

`CU-110` は `CU-107` 全体の完了を待つ前提を維持する。

**(B) より狭い名前付き前提への分割を検討する。**

`CU-107` をより狭い名前付き前提へ分割する案を検討する一方、production 到達性の明示 gate は残す。

## 5. 非目標

- 上記の一つの問いへの答えを書くこと。
- `CU-107` 配下の子粒の名前、個数、責任分担、実装順を書くこと。
- event shape、WebView wire、verdict enum、公開 API、visibility、bounded table size、rejection precedence を決めること。
- Rust / TypeScript / React / fixture / test / guard / schema / Document / journal / 公開 API の変更。
- `CU-107` / `CU-110` / `CU-111` の実装、promotion、`WAIT` 解除。
- `CU-110` の既存依存リスト（`CU-102/107/109`）の変更・削除・再解釈。
- [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) の編集。
- allowlist 外ファイルへの一切の変更。

## 6. 必須負例

- 候補 (A)/(B) のどちらかを推奨・採用・「有力」「望ましい」等と書く、または片方だけ厚く書いて実質的に選好を出す。
- 一つの問いを2問以上へ増やす、または `CU-110D` に追加の裁定対象を足す。
- `CU-107` の子粒名、event 名、verdict 値、API 名、閾値、表サイズを発明する。
- `CU-110` の依存リストを書き換える、`CU-107` を `DONE` 扱いする、`CU-0B05` を解決済みとする。
- `CU-110D` を発注依存証跡へ `DONE` として追加する（自分の order で自分を完了にする迂回）。
- PRODUCT-ASSET `DO` を 0 件または 2 件以上にする。
- 既存 decision 文書または発注依存証跡の既存行の意味を書き換える。
- [reviews 索引](README.md) または [decision-index](../decision-index.md) への登録を省く。
- 固定語彙外の状態語を新設する。
- lint 抑制、テスト期待値・golden の書き換え、fixture 特例、guard の個別 ID 除外。
- allowlist 外ファイルへの変更、TODO stub、隣接チケットへの拡張。

## 7. STOP 条件

1. 選定文書を書くために、一つの問いの答えを決めなければならなくなった。
2. `CU-110` の依存リスト、公開 API、Document、永続形式、plugin 契約のいずれかを変える必要が出た。
3. PRODUCT-ASSET `DO` を2件以上にしないと整合しない。
4. allowlist の7ファイルだけでは要求を満たせない（特に [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) の編集が必要に見えた場合）。
5. `CU-110S` lane 行が `DO` でない、または `CU-109` / `CU-G04SC` が発注依存証跡で `DONE` でないことを着手時に発見した。

## 8. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-110S` | **DONE** | `CU-110` 前提範囲の未決を docs-only `CU-110D` へ選定 |
| `CU-110D` | **DO** | `CU-107` 全体待ちか、より狭い名前付き前提への分割かを一問だけ裁定 |
| `CU-107` | **WAIT** | 据え置き |
| `CU-110` | **WAIT** | 据え置き |
| `CU-111` | **WAIT** | 据え置き |
