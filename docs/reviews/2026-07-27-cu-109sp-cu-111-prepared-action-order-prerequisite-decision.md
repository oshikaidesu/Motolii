# CU-109SP CU-109 / CU-111 prepared-action 順序前提

- 日付: 2026-07-27
- 状態: **決定**
- CU-109SP: **DONE**

## 1. 目的

[CU-109S 順序再確認](2026-07-27-cu-109s-undo-redo-prepared-action-order-recheck.md) §4 INFERENCE Q2 が記録した
**`CU-109` と `CU-111` の相互先後要求（循環）**を、既存 authority を書き換えずに解く **precedence 一件だけ**を裁定する（Q(SP)）。

本粒は prepared-action の形・公開／非公開 API・payload・transport・journal 形式・`Healthy / Poisoned` の具体 state を設計しない。

## 2. authority と参照行

| ID | 内容 | 参照 |
|---|---|---|
| A3 | accepted action ごとに non-live prepare → durable commit → live apply → Transient reconcile → atomic publish。Undo/Redo preflight は単一 `Command` を live writer 変更前に 1 件だけ確定 | [CU-G03](2026-07-26-cu-g03-edit-durability-ordering-decision.md) §3 |
| A4 | Undo/Redo command 取得は `CU-111` が非公開 typed prepared-action 境界として閉じる。raw stack / raw writer / 汎用 peek を公開しない。`Healthy / Poisoned` edit authority は `CU-109` が session 内 Transient に一箇所だけ所有 | [CU-G03](2026-07-26-cu-g03-edit-durability-ordering-decision.md) §4 末尾 |
| A5 | §6 必須負例、§7 後続境界（`CU-109` = 本順序の ProjectSession / journal / poison / publish 配線、`CU-110` = Place、`CU-111` = 製品 Undo/Redo `CommandId` と単一 command top macro の非公開 typed prepared action への変換・配送）、§8 STOP | [CU-G03](2026-07-26-cu-g03-edit-durability-ordering-decision.md) §6〜§8 |
| A7 | `CU-109` 依存 = `CU-G03D` / `CU-G03R` / `U2b` / `D1m`。`CU-111` 依存 = **`CU-109`**、`U0c` / `U2b` | [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) §8 W1 表 |
| A9 | `CU-109S` §4 Q2・§5（R2・候補 (b)）、§9 handoff（`CU-109SP` = 循環前提の docs 閉包） | [CU-109S 順序再確認](2026-07-27-cu-109s-undo-redo-prepared-action-order-recheck.md) |
| A10 | 「現在の並列レーン」「発注依存証跡」 | [implementation ledger](../implementation-ledger.md) |

## 3. 現行コード事実（BASE_SHA `19491b69`）

不在（caller / field / API が無いこと）は順序・所有の肯定証拠にしない（§6 N2）。

| ID | 事実 |
|---|---|
| CF1 | `crates/motolii-doc/src/undo.rs:134-139` — `UndoHistory` の `undo_stack` / `redo_stack` は private field |
| CF2 | `undo.rs:142-242` — 公開 method は `new` / `from_restored` / `undo_len` / `redo_len` / `can_undo` / `can_redo` / `push` / `undo` / `redo` のみ。次に undo/redo される `Macro` または `Command` を読む公開口は無い |
| CF3 | `crates/motolii-doc/src/lib.rs:463-481` — `DocumentWriter` の公開面は `can_undo` / `can_redo` / `undo_len` / `redo_len` と、mutate 後に `Result` を返す `undo()` / `redo()` |
| CF4 | `crates/motolii-ui/src/document_edit_runtime.rs:80-123` — `DocumentEditRuntime` は crate 内限定公開。`process_next` は `Apply(request)` / `Undo` / `Redo` を `writer.apply_macro` / `writer.undo` / `writer.redo` へ直接渡し、成功後に revision・snapshot・reconcile 済み primary・`projection_generation` を 1 envelope で返す。non-live preflight も journal 接続も無い |
| CF5 | `crates/motolii-doc/src/journal/session.rs:72` `ProjectSession`、`session.rs:129` `save_with_journal`、`crates/motolii-doc/src/journal/wal.rs:151` `commit_edit` は存在する |
| CF6 | `crates/motolii-ui/src` 内で `ProjectSession` / `save_with_journal` / `commit_edit` を使う箇所は無い（`state_ownership.rs:9`、`domain_intent.rs:44` の variant 名のみ） |
| CF7 | [implementation ledger](../implementation-ledger.md)「現在の並列レーン」で `状態` が `DO` の行は全 lane 通算 1 件、`CU-109SP` だけである |
| CF8 | 同「発注依存証跡」に `CU-109S` / `CU-109S0` / `CU-G03D` / `CU-G03R` / `D1m` / `D2` / `U2b-1` / `CU-104` / `CU-104E` の `DONE` 行が存在する。`CU-109` / `CU-110` / `CU-111` の行は無い |
| CF9 | `./scripts/check-docs.sh` は `OK`、`node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` は 118 pass / 0 fail |
| CF10 | `node --test docs/mocks-ui/guard-tests/inspector-read-model-inventory.test.mjs` は `node_modules` 未設置の worktree で `ERR_MODULE_NOT_FOUND: '@babel/parser'` で失敗する。BASE_SHA 時点の環境前提であり、本粒の diff が原因ではない |

## 4. 事実 / 推論の分離

### FACT

- [CU-109S 順序再確認](2026-07-27-cu-109s-undo-redo-prepared-action-order-recheck.md) §4 INFERENCE Q2 は、`CU-109` 配線完了と `CU-111` 境界による Undo/Redo preflight 確定が相互に先後を要求する循環を記録した（A9）。
- A3〜A5・A7 は上表のとおり既存 authority に固定され、本粒では改変しない（§8 S1）。

### INFERENCE — Q(SP) 候補 × admissibility（AD1〜AD6）

| 候補 | 内容 | AD1 | AD2 | AD3 | AD4 | AD5 | AD6 | 受理 |
|---|---|---|---|---|---|---|---|---|
| **P1** | `CU-109` が accepted action 全体の共有 ProjectSession / journal / poison / reconcile / publish 配線を所有したまま先行し、acceptance evidence を `CU-111` 入力を要しない **Apply roundtrip に限定**する。`CU-111` は後続で同一配線へ変換・配送だけを接続する | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | **可** |
| **P2** | `CU-109` の acceptance evidence に Undo/Redo durable roundtrip を含める（次 Undo/Redo command の取得を `CU-109` 側で成立させる） | ✓ | **AD2** | ✓ | **AD4** | ✓ | ✓ | 不可 |
| **P3** | `CU-111`（またはその一部）を `CU-109` より先に置き、Undo/Redo durable 配線・poison・journal・reconcile・publish を `CU-111` diff へ移す | ✓ | ✓ | **AD3** | **AD4** | ✓ | ✓ | 不可 |
| **P4** | 上記以外の新しい所有分割 | — | — | **AD3**（および **AD5**） | — | — | — | 不可 |

- **P2 違反**: AD2（Undo/Redo command 取得は `CU-111` 境界が閉じる）、AD4（`CU-111` は `CU-109` 完了後）。
- **P3 違反**: AD3（共有配線を `CU-111` diff へ移さない）、AD4。
- **P4 違反**: AD3・AD5（新分割は typed shape / API 等を要する）。

admissible 候補は **P1 のみ**（厳密に 1 件）。

## 5. 判定と結論

- **判定規則**: admissible が 1 件のときだけ結論とする。
- **結論**: **P1**
- **次 PRODUCT-ASSET `DO`**: **`CU-109`**（実装粒。acceptance evidence は Apply roundtrip に限定）

P1 が admissible な場合の限定（authority 非改変）:

1. acceptance evidence の限定は **evidence の範囲だけ**であり、`CU-109` の所有範囲を縮めない。Undo/Redo の durable 配線・poison・journal・reconcile・publish は `CU-109` 所有に留まる。
2. `CU-111` は後続で、`CU-109` 所有配線へ接続するだけとし、2 本目の durability / publish 経路を作らない。
3. prepared-action の形・API・payload・配送機構は本粒で決めない（後続 `CU-111` 粒の範囲）。

## 6. 非目標（本発注書 §6 を維持）

- prepared-action の型、公開 API、非公開 API、payload、transport、journal 形式の決定
- `Healthy / Poisoned` の具体 state、復旧 UI、再 open 規則の設計・実装
- `CU-109` / `CU-110` / `CU-111` の実装、または同一粒への束ね
- Rust / JS / fixture / guard test / golden / `package.json` / script の変更
- Document、journal、plugin 契約、永続形式、公開境界の変更
- `U3a-2Q-V`、`CU-106P` / `CU-106F`、`CU-0A08BT` / `CU-0A08IT`、`U2c-2`、製品 window、`G0-6H`、`G0-9*`、`U4a` / VS-2 の状態変更
- 隣接チケット（`CU-110` Place、`U2h-1P`、`U3a-2*`）への拡張
- 不在を順序・所有の肯定証拠にすること

## 7. 必須負例（してはならない）

- **N1**: 本粒で `CU-109` / `CU-110` / `CU-111` の実装差分を書く、または着手する。
- **N2**: 不在（CF1〜CF6 の「無い」）を順序・所有の肯定証拠として結論に使う。
- **N3**: 依存が `DONE` であることだけを根拠に Q(SP) の判定を省略する。
- **N4**: `CU-111` 所有の prepared-action の内容・境界・所有を `CU-109SP` または `CU-109` 側へ持ち込む。
- **N5**: `G0-6H`、製品 window、`G0-9`、`U0e-3` を本粒の依存へ足す。
- **N6**: PRODUCT-ASSET `DO` を 2 件以上にする。
- **N7**: 外部 model 助言、旧粒度化の候補分類、過去 order 文面を authority にする。
- **N8**: 「発注依存証跡」の既存行、過去 decision の本文・PR・hash を書き換える（新規 1 行追記のみ可）。
- **N9**: ledger / spec / README の散文や別 phase の状態から依存・次粒を推測する（機械判定は「発注依存証跡」表と「現在の並列レーン」表のみ）。
- **N10**: raw stack / raw writer / 汎用 peek / 公開 raw mutation API を「必要」として本文へ書く。
- **N11**: guard test / golden / fixture / 期待値 / threshold / 固定 hash を触る、lint 抑制や個別除外を足す、重複 planner / helper / 同名 docs 粒 / 新 script を新設する、暗黙 default を発明する。
- **N12**: 結論を書かず TODO / 保留 / 後続任せで複数候補を残す、または一部の file だけ更新して他を stale にする。

## 8. STOP 条件

- **S1**: A3 / A4 / A5 / A7 のいずれかを書き換える必要が見える。
- **S2**: 結論に typed shape、API、payload、transport、journal 形式、`Healthy / Poisoned` 具体 state、poison 実装が必要になる。
- **S3**: admissible 候補が 0 件、または 2 件以上になる。
- **S4**: 共有配線の一部を `CU-111` 側へ移さないと閉じられない。
- **S5**: 許可 file 以外へ 1 byte でも変更が要る。
- **S6**: guard test / `check-docs.sh` を通すために期待値・threshold・除外・固定 hash・fixture・依存を触りたくなる。
- **S7**: 変更後に `DO` 行が 2 件以上残る形しか書けない。
- **S8**: 本発注書・AUTHORITY・CF 以外が判断に必要になる。
- **S9**: `CU-110` 配置、UI surface、`G0-6H`、製品 window、`U4a` / VS-2 の状態を動かす必要が出る。

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| **`CU-109SP`** | **DONE** | P1 precedence を docs で閉じた |
| **`CU-109`** | **DO** | 共有配線の実装粒。acceptance evidence は Apply roundtrip に限定 |
| `CU-110` / `CU-111` | **WAIT** | 据え置き |
| `U3a-2Q-V` / `CU-106P` / `CU-106F` | **WAIT** | 据え置き |

PRODUCT-ASSET lane の `DO` は **`CU-109` ただ一件**とする。
