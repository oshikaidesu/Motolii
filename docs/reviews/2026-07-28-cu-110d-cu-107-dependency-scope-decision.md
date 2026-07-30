# CU-110D CU-110 の CU-107 依存範囲裁定

- 日付: 2026-07-28
- 状態: **決定**
- CU-110D: **DONE**

## 1. 目的

[CU-110S 選定](2026-07-28-cu-110s-dependency-scope-decision-selection.md) §3 が送った唯一の問い——`CU-110` は `CU-107` 全体の完了を待つ必要があるのか、それとも `CU-110` の既存依存を `CU-107` 配下のより狭い名前付き前提へ分割すべきか——へ docs-only で答えを書き、`CU-110D` を閉じる。実装・promotion・`WAIT` 解除は一切しない。

## 2. 事実

- [implementation ledger](../implementation-ledger.md) の「現在の並列レーン」で `CU-110D` 行の状態は完全一致 `DO`（1行のみ）。PRODUCT-ASSET lane で `DO` は着手時点でこの1件だけであった。
- 同 [発注依存証跡](../implementation-ledger.md#発注依存証跡) に `CU-109` = `DONE`（PR #425、実装 commit `356d703f`、merge `32cf8902bf5c96fc60400a91335e72a9886cf304`）、`CU-G04SC` = `DONE`、`CU-110S` = `DONE` の一意行がある。`CU-110D` 行は着手時点では未登録であった。
- [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) の W0 表で、`CU-107` = `CORE / WAIT`（依存: 既存 D&D spike、CU-0B05）、`CU-110` = `CORE / WAIT`（依存: `CU-102/107/109`）、`CU-111` = `PRODUCT / WAIT` である。
- [CU-110S 選定](2026-07-28-cu-110s-dependency-scope-decision-selection.md) §2 の read-only 調査事実: 製品 `crates/` および `ui/` 配下に Place intent 型・公開 API・production caller は存在せず、drag epoch/sequence/dedupe coordinator の製品 Host 接続実装も存在しない（D&D spike と test harness を除く）。
- `CU-102` と `CU-109` は発注依存証跡で `DONE`。`CU-107` は W0 表で `WAIT` のまま（`CU-0B05` 待ち）である。

## 3. 裁定

[CU-110S 選定](2026-07-28-cu-110s-dependency-scope-decision-selection.md) §4 の候補 **(B)** を採る。

**B-1**: `CU-110` の `CU-107` 依存は「`CU-107` 全体完了待ち」のままにしない。`CU-107` 配下のより狭い**名前付き前提**へ分割する方針を採る。理由は、待ち理由を監査可能にし、`CU-0B05` を含む `CU-107` 全体の未決を `CU-110` 全体の停止理由へ畳み込まないため。

**B-2**: 分割は `CU-110` の `WAIT` 解除ではない。`CU-110` は、**通常製品 route で non-test production drop source が成立する**まで `WAIT` を維持する。ここでの「成立」は製品 `crates/` / `ui/` 上の production caller の存在を指し、test / dummy / smoke / lint 抑制を到達性の証拠に数えない（§2 の CU-110S 調査事実に依拠）。

**B-3**: 具体化（狭い前提の名前・個数・責任分担・実装順、および W0 表と `CU-110` 依存リストの実際の書換え）は本粒では行わず、後続 docs-only 粒 **`CU-107S`** 1件へ送る。`CU-107S` は「`CU-107` 分割の具体化範囲の選定」だけを扱う docs-only 選定粒であり、`CU-107` 配下の子粒そのものではない。

## 4. 非目標

- 子粒名・個数・責任分担・実装順、event shape、WebView wire、verdict enum / 値、公開 API 名、visibility、bounded table size、閾値、rejection precedence の決定。
- `CU-110` の既存依存リスト `CU-102/107/109` の書換え・削除・再解釈。
- `CU-107` / `CU-110` / `CU-111` の実装、promotion、`WAIT` 解除、`CU-0B05` の解決宣言。
- [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) の編集（W0 表の `CU-107` / `CU-110` / `CU-111` 行を含め一切禁止）。
- Rust / TypeScript / React / fixture / test / guard / schema / Document / journal / 公開 API / plugin 契約の変更。
- 既存 decision 文書・発注依存証跡の既存行の意味の書換え。
- allowlist 外ファイルへの一切の変更。
- 隣接チケット（`U3a-2Q-V` / `CU-0A08BT` / `CU-106P` / `CU-106F` / `U2h-1P`）の状態変更。

## 5. STOP 条件

1. 着手時に `CU-110D` lane 行が完全一致 `DO` でない、または `CU-109` / `CU-G04SC` / `CU-110S` のいずれかが発注依存証跡で `DONE` でない。
2. §3 の答えを書くために、子粒名・event shape・API・verdict・visibility・閾値のいずれかを決める必要が出た。
3. `CU-110` の依存リスト、公開 API、Document、永続形式、plugin 契約のいずれかを変える必要が出た。
4. [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) を編集しないと整合しないように見えた。
5. PRODUCT-ASSET `DO` が 0 件または 2 件以上にしか収まらない。
6. `CU-107S` 以外の後続粒 ID が必要に見えた、または `CU-107S` が既に別意味で使われていた。
7. allowlist 7 file だけでは要求を満たせない。
8. `scripts/check-docs.sh` を通すために状態語彙の新設、索引の除外、lint 抑制、既存期待値の書換えが必要になった。

## 6. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-110D` | **DONE** | `CU-107` 全体待ちか、より狭い名前付き前提への分割かを一問だけ裁定（候補 B 採択） |
| `CU-107S` | **DO** | `CU-107` 分割の具体化範囲の選定 |
| `CU-107` | **WAIT** | 据え置き |
| `CU-110` | **WAIT** | 据え置き、non-test production drop source 待ち |
| `CU-111` | **WAIT** | 据え置き |
