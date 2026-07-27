# CU-109S Undo / Redo prepared-action 順序再確認

- 日付: 2026-07-27
- 状態: **決定**
- CU-109S: **DONE**

## 1. 目的

[CU-109S0 選定](2026-07-27-cu-109s0-readiness-recheck-selection-decision.md) §3 の二択を、
既存 authority と BASE_SHA `92656b26232e481c2383a040a86dfb26aea22b72` のコード事実だけで再確認する。

- 候補 (a): `CU-109` 実装を次 PRODUCT-ASSET `DO` にできる。
- 候補 (b): `CU-109` より前に、`CU-111` が所有する prepared-action 順序だけを閉じる docs-only 前提粒が必要である。

本粒は型・API・実装・順序の中身を設計しない。上記のどちらか一方だけを裁定する。

## 2. authority と参照行

| ID | 内容 | 参照 |
|---|---|---|
| A1 | `CU-109S` が決めてよい候補は (a)/(b) の二つのみ。一意に導けなければ `STOP` | [CU-109S0](2026-07-27-cu-109s0-readiness-recheck-selection-decision.md) §3 |
| A2 | entry gate 5 点と必須負例 N1〜N6 | [CU-109S0](2026-07-27-cu-109s0-readiness-recheck-selection-decision.md) §5・§6 |
| A3 | accepted action ごとに non-live prepare → durable commit → live apply → Transient reconcile → atomic publish。Undo/Redo preflight は単一 `Command` を live writer 変更前に確定 | [CU-G03](2026-07-26-cu-g03-edit-durability-ordering-decision.md) §3 |
| A4 | Undo/Redo command 取得は `CU-111` が非公開 typed prepared-action 境界として閉じる。`Healthy / Poisoned` は `CU-109` 所有 | [CU-G03](2026-07-26-cu-g03-edit-durability-ordering-decision.md) §4 末尾 |
| A5 | §6 必須負例 9 行、§7 後続境界（`CU-109` 配線 / `CU-110` Place / `CU-111` 変換・配送）、§8 STOP 6 点 | [CU-G03](2026-07-26-cu-g03-edit-durability-ordering-decision.md) §6〜§8 |
| A6 | `CU-109` は本決定を authority にできるが prepared-action 順序再確認まで自動着手しない | [CU-G03](2026-07-26-cu-g03-edit-durability-ordering-decision.md) §9 |
| A7 | `CU-109` CORE/WAIT 依存 = `CU-G03D`/`R`、`U2b`/`D1m`。`CU-111` PRODUCT/WAIT 依存 = **`CU-109`**、`U0c`/`U2b` | [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) §8 W1 表 |
| A8 | journal durability 行 = `CU-G03 DONE` / `CU-109S DO` / `CU-109 WAIT` | [縦slice実行方針](2026-07-24-m3-vertical-slice-execution-decision.md) §4 blocking table |
| A9 | `CU-109S` は `DO`、PRODUCT-ASSET の他 `DO` は 0 件 | [implementation ledger](../implementation-ledger.md)「現在の並列レーン」 |

## 3. 現行コード事実（BASE_SHA `92656b26`）

| ID | 事実 |
|---|---|
| CF1 | `crates/motolii-doc/src/undo.rs:134-139` — `undo_stack` / `redo_stack` は **private field** |
| CF2 | `undo.rs:141-243` — 公開 method は `new` / `from_restored` / `undo_len` / `redo_len` / `can_undo` / `can_redo` / `push` / `undo` / `redo` のみ。**次に undo/redo される `Macro` または `Command` を読む口は無い** |
| CF3 | `undo.rs:33-36` — `Macro` は公開型、`lib.rs:112` で re-export。型は公開だが CF2 のとおり **live history から取り出す経路が無い** |
| CF4 | `lib.rs:463-477` — `DocumentWriter` は `can_undo` / `can_redo` / `undo_len` / `redo_len` のみ。`undo()` / `redo()` は **mutate 後に `Result` を返す** |
| CF5 | `command.rs:899` — `Command::inverse` は存在 |
| CF6 | `undo.rs:225-242` — `undo` / `redo` は stack pop 後に適用し、内部失敗時に pop 済み `Macro` を戻さない |
| CF7 | `lib.rs:412-442` — `apply_macro` は失敗時に doc / undo / revision / gesture を呼出前へ戻す |
| CF8 | `document_edit_runtime.rs:80-123` — `DocumentEditRuntime` は `pub(crate)`。Undo/Redo は `writer.undo()` / `writer.redo()` を直接呼ぶ。**non-live preflight も journal 接続も無い** |
| CF9 | `motolii-ui/src` に `ProjectSession` / `save_with_journal` / `commit_edit` の使用は無い（enum variant 名のみ） |
| CF10 | `journal/session.rs:72` `ProjectSession`、`session.rs:129` `save_with_journal`、`wal.rs:151` `commit_edit` は存在 |

CF1〜CF10 は現状の写像であり、それ自体は順序の肯定証拠ではない（N2）。

## 4. 事実 / 推論 / 助言 / 改善の分離

### FACT

- [CU-G03](2026-07-26-cu-g03-edit-durability-ordering-decision.md) §3 step 1 は、Undo は undo stack 先頭の単一 forward から得る inverse `Command`、Redo は redo stack 先頭の単一 forward `Command` を、live writer を変更せず 1 件だけ確定すると規定する（A3）。
- 同 §4 末尾は、Undo/Redo command の取得を `CU-111` の非公開 typed prepared-action 境界が閉じ、raw stack / raw writer / 汎用 peek を公開しないと規定する（A4）。
- 同 §7 は、`CU-109` を本順序の ProjectSession / journal / poison / publish 配線、`CU-111` を製品 Undo/Redo `CommandId` と単一 command top macro の非公開 typed prepared action への変換・配送と分ける（A5）。
- [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) は `CU-111` の依存に **`CU-109`** を列挙する（A7）。
- CF2・CF4・CF8 は、現行 live path に Undo/Redo の non-live 単一 `Command` 確定経路が無いことを示す。

### INFERENCE

- **Q1（必要か）**: 必要。根拠は A3（§3 step 1 の Undo/Redo preflight）と CF2・CF4（live writer 変更前に次 command を読む公開口が無い）。必要な読み取り経路の所有は A4 により **`CU-111`** に固定される。A5 §7 は `CU-109` が配線、`CU-111` が変換・配送と分離する。
- **Q2（循環か）**: 循環する。`CU-109` は A5 §7・A6 により CU-G03 順序全体（step 1 含む）の製品配線を担う。A4 は同順序の Undo/Redo command 確定を `CU-111` 境界へ固定する。A7 は `CU-111` 実装が **`CU-109` 完了後**とする。よって「`CU-109` 配線完了」と「`CU-111` 境界による Undo/Redo preflight 確定」が相互に先後を要求する。
- **Q3（解き方）**: (b)。(a) は A4 の「`CU-111` が閉じる」を `CU-109` 側へ移す再解釈となり、本粒では選べない（order §4 Q3・S3）。
- **Q4（候補）**: 候補 **(b)**。R2（Q1=必要、Q2=循環、Q3=(b)）に落ちる。

### ADVICE / OPPORTUNITY

- 拘束力は無い。採用・実装順は後続粒（`CU-109SP`、`CU-109`、`CU-111` 等）の判断とする。
- `CU-109SP` は順序前提の docs 閉包のみを想定し、typed shape・private/public API・配送の設計は後続へ委ねる。

## 5. 判定と結論

- **判定規則**: **R2**
- **結論**: 候補 **(b)**
- **次 PRODUCT-ASSET `DO`**: docs-only **`CU-109SP`**（`CU-111` 所有 prepared-action 順序前提だけを対象とする。中身は本粒で決めない）
- **`CU-109`**: `WAIT` 維持（`CU-G03` §7 / 粒度化 W1 行を参照。複製・再定義しない）

## 6. 非目標（CU-109S0 §4 を維持）

- Rust / JS / fixture / guard / golden の変更
- prepared-action の型、公開 API、private API、payload の決定
- Document、serde、journal、plugin 契約、永続形式の変更
- `Healthy / Poisoned` の具体 state、復旧 UI、再 open 規則の実装
- `CU-109` / `CU-110` / `CU-111` の実装または同一粒への束ね
- `U3a-2Q-V`、`CU-106P/F`、製品 window、G0-6H、U4a / VS-2 の状態変更
- code / caller / field / API の不在を順序の肯定証拠にすること

## 7. 必須負例（してはならない）

- **N1**: 本粒だけで `CU-109` 実装を ready と裁定する。
- **N2**: absence-as-positive-evidence — 不在を順序や所有の肯定証拠にする。
- **N3**: dependency-DONE shortcut — 依存 `DONE` だけで prepared-action 順序の再確認を省略する。
- **N4**: `CU-111` 所有の prepared-action 内容・境界・所有を `CU-109S` または `CU-109` 側へ持ち込む。
- **N5**: `G0-6H`、製品 window、G0-9、`U0e-3` を `CU-109S` の依存へ足す。
- **N6**: multiple DOs — `CU-109` / `CU-110` / `CU-111` 等を同時に PRODUCT-ASSET `DO` にする。
- **N7**: external-model advice as authority — 外部 model 助言・旧粒度化分類・過去 order 文面を authority にする。
- **N8**: historical receipt rewriting — 発注依存証跡の既存行・過去 decision の本文・PR・hash を書き換える（新規 1 行追加のみ可）。
- **N9**: ledger / spec / README の散文や別 phase 状態から依存や次粒を推測する（機械判定は発注依存証跡表と現在の並列レーン表のみ）。
- **N10**: raw stack / raw writer / 汎用 peek / 公開 raw mutation API を「必要」として本文へ書く。
- **N11**: guard test / golden / fixture / 期待値 / threshold / 固定 hash を触る、lint 抑制・個別除外を足す、重複 planner/helper/同名 docs 粒を新設する。
- **N12**: 結論を書かず TODO / 保留 / 後続任せで両候補を残す。

## 8. STOP 条件

- **S1**: 候補 (a) と (b) の両方、またはどちらでもない結論（R3）。
- **S2**: 結論に typed prepared-action の形、API、payload、journal 形式、`Healthy / Poisoned` 具体 state、poison 実装が必要になる。
- **S3**: A4 の「Undo/Redo command 取得は `CU-111` が閉じる」を書き換える必要が見える。
- **S4**: `CU-110` 配置、UI surface、`G0-6H`、製品 window、`U4a` / VS-2 の状態を動かす必要が出る。
- **S5**: 許可 file 以外へ 1 byte でも変更が要る。
- **S6**: guard test / check-docs を通すために期待値・threshold・除外・hash・fixture を触りたくなる。
- **S7**: PRODUCT-ASSET lane に `DO` が 2 件以上残る形しか書けない。
- **S8**: 本 order・AUTHORITY・CF 以外（会話履歴、横断調査、未指定公開境界探索、他 model 出力）が判断に必要になる。

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| **`CU-109SP`** | **DO** | `CU-111` 所有 prepared-action 順序前提だけを docs で閉じる（中身・API・型は決めない） |
| `CU-109` | **WAIT** | `CU-109SP` 完了後に実装順を再判定（本粒では ready にしない） |
| `CU-110` | **WAIT** | 据え置き |
| `CU-111` | **WAIT** | 据え置き |
| `U3a-2Q-V` | **WAIT** | actual consumer surface evidence 待ち（据え置き） |
| `CU-106P` / `CU-106F` | **WAIT** | 据え置き |

PRODUCT-ASSET lane の `DO` は **`CU-109SP` ただ一件**とする。
