# U2h-1 primary selection implementation split決定

- 日付: 2026-07-27
- 状態: **決定**
- U2h-1: **SPLIT**
- U2h-1S: **DONE**
- U2h-1I: **DONE**
- U2h-1P: **WAIT**（[CU-106 selection consumer分割決定](2026-07-27-cu-106-selection-consumer-split-decision.md)によりCU-106Pへ統合）

## 0. 語彙（本決定内で一貫）

- **歴史的・基盤前提（historical/foundational prerequisite）**: M3仕様 U2h行に既に載る依存family。`U0c` / `U2a` / `U2b`。family単位の`DONE`断定はしない。台帳§主クリティカルパスで `U0c-1`/`U0c-2`/`U2a-0`/`U2a-1`/`U2b-1` が `DONE` である事実だけを引く。`U2b-2` Placeは `CU-101`/`CU-110` へ分離済みでU2h-1の入場条件にしない。
- **現在の入場証跡（current entry evidence）**: 完了済みU2c entryの証跡として引く行は `U2c-1` と `U2c-4` の2件**のみ**。
- **現在の未解決gate（current unresolved gate）**: U2h-1着手を止める未完了条件。現時点で**0件**。

## 1. 分割の結論

`U2h-1`（Host Transient primary selection + publish envelope実装）を、既存private Apply/Undo/Redo publication経路とselection-only入力面へ分割する。CU-104はowner・visibility・field閉集合・generation規則・reconcile時点をdocsで閉じ済み。本粒は実装分割と陳腐化表現の解消だけを行い、新しい製品意味・公開契約は作らない。

| ID | 状態 | 一成果 | 依存（U2h行・本表と同語） | 合格 | STOP |
|---|---|---|---|---|---|
| U2h-1 | `SPLIT` | 親。分割証跡のみ | `U0c`,`U2a`,`U2b`,`U2c-1`,`U2c-4`（§0の三分を厳守） | 発注依存証跡に`SPLIT`行が一意 | — |
| U2h-1S | `DONE` | 本決定で§4を正本化 | `CU-104`（意味・契約の閉じ済み正本）, `U2c-1`,`U2c-4`（§0・現在の入場証跡） | 本文書§0〜§4だけで`U2h-1I`/`U2h-1P`の境界が一意 | — |
| U2h-1I | `DONE` | 既存private Apply/Undo/Redo publication経路へのfield追加とreconcile | `U2h-1S` | CU-104 §7 **P1/P2/P3** | 第2 publish path・公開API化 |
| U2h-1P | `WAIT` | selection-only `ReplacePrimary`/`ClearPrimary`入力面の受入ID | `U2h-1I`, CU-106S, CU-106Pの実consumer surface | CU-104 §7 **P5** | producer-only実装、lint抑制、dummy caller |

U2h-1I完了時点までの未解決gateは0件だった。U2h-1P事前審査後のCU-105R/CU-106Sは完了し、[CU-106 selection consumer分割決定](2026-07-27-cu-106-selection-consumer-split-decision.md)によりCU-106Pの実consumer surfaceが現在gateである。`U2c-2`/`U2c-3`/`U2c-5`、`U0e-3`、`G0-6H`、`U3a-1`は現在gateではない（「不要」「完了」とは書かない）。

## 2. 現行コード事実

BASE_SHA `733c622b70f02ca19e09ca0dbc705e73bfa64162` 時点。本節以外の探索を`U2h-1I` closed orderの根拠にしない。

| # | 事実 |
|---|---|
| 1 | `crates/motolii-ui/src/document_edit_runtime.rs:26-31` `DocumentEditActionKind` は `Apply` / `Undo` / `Redo` のみ |
| 2 | 同 `:121-125` `PublishedDocument` は `kind` / `revision` / `snapshot` の3 field、`pub(crate)` |
| 3 | `PublishedDocument` 構築siteは同 `:108` の**1箇所だけ** |
| 4 | `crates/` 全体に `ReplacePrimary` / `ClearPrimary` / `projection_generation` は0件 |
| 5 | `crates/motolii-doc/src/lib.rs:533` `pub fn find_envelope(&self, target: LayerId)` が再帰存在oracle |
| 6 | U2h-1は全体未実装。既存publish経路はApply/Undo/Redo成功時のみ |

## 3. 依存の三分（ambiguity 1）

M3仕様 U2h行、本決定、台帳lane行は同じことを同じ語で言う。

- U2h行の依存セルから `U0c` / `U2a` / `U2b` を**削除しない**（歴史的・基盤前提）。
- 裸の `U2c` は `U2c-1`,`U2c-4` へ絞る（現在の入場証跡）。
- `U2c-1`/`U2c-4` を「唯一の依存」「唯一の歴史的依存」と**断定しない**。
- 「U2c ownershipの整合待ち」表現は陳腐化。現在の入場証跡は `U2c-1`/`U2c-4` の完了で満たされている。

## 4. U2h-1I（完了済み実装粒）

- 対象は**既存の private Apply/Undo/Redo publication経路**のみ。
- `PublishedDocument` へ `primary: Option<LayerId>` と `projection_generation: u64` を CU-104 §4の閉集合どおり追加（private・`pub(crate)` 維持）。
- reconcile-before-publish（CU-104 §6 D4）と Redo非復元を含む。
- 必須正例: CU-104 §7 **P1 / P2 / P3**。
- 再利用: 単一構築site（`document_edit_runtime.rs:108`）と `find_envelope`。新publish経路、第2 selection構造体、`LayerIdTable::contains` 相当の別存在checkは作らない。

## 5. U2h-1P（CU-106P待ち）

- selection-onlyの `ReplacePrimary` / `ClearPrimary` 入力面のみ。
- private action / kind の追加と、unknown / table-only IDのtyped rejectを含む。
- 必須正例: CU-104 §7 **P5**。
- production caller不在の単独producer粒にはせず、[CU-106 selection consumer分割決定](2026-07-27-cu-106-selection-consumer-split-decision.md)どおりCU-106Pへ統合する。
- **現行順序**: `U2h-1I` / CU-105R / CU-106S `DONE` → U3a-2S再確認 → 実consumer surface成立後のCU-106P内U2h-1P。`CU-109`/`CU-110`/`CU-111`の相互順序は固定しない。

## 6. P4 の帰属

CU-104 §7 **P4 Place receipt** は `CU-110` に留める。成功receiptはCU-110まで存在せず、U2h-1I/U2h-1Pのoracleを未実在物へ依存させない。

## 7. 必須正例（帰属）

| ID | 帰属 |
|---|---|
| P1 Apply成功 | `U2h-1I` |
| P2 Undo成功でprimary dangling | `U2h-1I` |
| P3 Redo成功（非復元） | `U2h-1I` |
| P4 Place receipt | `CU-110` |
| P5 selection-only valid ReplacePrimary | `U2h-1P` |

## 8. 必須負例

- SN1〜SN7（CU-104 §8）を`U2h-1I`/`U2h-1P`実装で破らない。
- 歴史的・基盤前提と現在の入場証跡を混同した記述、または「U2h-1の依存は`U2c-1`と`U2c-4`だけ」型の断定。

## 9. 非目標

- `CU-109` / `CU-110` / `CU-111` / `CU-106` / `CU-105` / `U2h-2` の実装または意味変更。
- 本粒での Rust / JS / fixture / guard test / golden 変更。
- `U2h-1P`より後のticket順序決定。
- 公開API、`Document`、serde、journal、Undo/history、ProjectSession、plugin契約の変更。
- CU-104が閉じた owner / visibility / field閉集合 / generation規則 / reconcile時点の再決定。
- 発注依存証跡への `U0c` / `U2a` / `U2b` 行の新設。
- `U2h-1S` の後続同名docs粒。

## 10. STOP

1. 分割を書くために公開API・Document意味・serde・plugin契約の新規決定が必要
2. allowlist外fileの変更が必要
3. `crates/` の編集が必要（`U2h-1S`はdocs-only）
4. P4をU2h-1I/U2h-1Pへ移す必要が見える
5. `U2h-1P` と他ticketの相互順序を本粒で固定しないと書けないと判明
6. §0の三語を混ぜた記述が unavoidable
7. 台帳の `U2h-1I` または `CU-104`/`U2c-1`/`U2c-4` が `DONE` 以外

## 11. U2h-1I完了後の引き渡し

`U2h-1I`は、既存private Apply/Undo/Redo経路へCU-104どおりのfieldとreconcileを実装し、[../implementation-ledger.md](../implementation-ledger.md)の`発注依存証跡`で`DONE`になった。`U2h-1P`はowner/visibility・P1〜P3帰属・§0語彙を変更せず、§5のselection-only入力面とP5だけをCU-106P内で閉じる。`U2h-1S`の後続同名docs粒は作らない。
