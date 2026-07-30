# CU-G09O Browser decoder output契約決定

- 日付: 2026-07-26
- 状態: 決定
- 粒: CU-G09O DONE

## §1 適用範囲

本書は [CU-G09 Browser catalog projection契約決定](2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md) を**延長**する追補であり、`CU-G09` 本文は1バイトも編集しない。`CU-G09 §1`〜`§10` は本書が明示的に差し替える箇所を除いてそのまま現行authorityである。本書が差し替えるのは次の3箇所だけである。

- `CU-G09 §6` の語彙entry shape 文言 `{ id, label, scope_ref? }`
- `CU-G09 §7` の `B11` 行の文言「optional省略」
- `CU-G09 §7` の `B12` 行を入力拒否例として読める旧記述

## §2 decoder出力契約（`O1`〜`O6`）

| ID | 規則 |
|---|---|
| `O1` | decoder出力は、`CU-G09 §6` の全検証に合格した strict top-level snapshot の **deep non-aliasing clone** とする。入力側のobject／array／containerと同一参照を1つも共有しない |
| `O2` | 出力は `catalog_revision`、`vocabularies`、`catalogs`、それら配下のkey名、および validated 値を保存する。key の追加・削除・改名・並べ替えによる意味付けを行わない |
| `O3` | vocabulary の `label` は verbatim で運搬する。比較・parse・正規化・trim・既定値化・意味分岐のいずれにも使わない |
| `O4` | 全 ref は dangling／cross-scope 検証後も **opaque string** のままとする。分解・prefix判定・scope推定をしない |
| `O5` | `CU-G09 §6a`（`CU-0A08IS §6a` 由来）の decoder出力禁止key閉集合を出力へ適用し続ける。とくに literal key `availability` / `availability_lifecycle` を出力しない |
| `O6` | 出力へ新しい wrapper、派生field、derived index、正規化済みlookup表を追加しない |

## §3 vocabulary entry shape（`CU-G09 §6` の当該文言を差し替える）

- `V1`: 語彙entryは厳密3キー `{ id, label, scope_ref }` とする。`scope_ref` は **key必須・値nullable**。非scoped語彙は `scope_ref: null` を明示して持つ。`scope_ref` key の省略は受理しない。
- 差し替え前の文言（`{ id, label, scope_ref? }`、`scope_ref` は scoped 語彙のみ）を**撤回**する。`CU-G09 §6` の当該行は受理入力契約の歴史記録として残るが、decoder出力と将来decoder実装の正本は本 `V1` である。

## §4 item key の存在則（`CU-G09 §6` を延長）

- `I1`: `CU-G09 §6` の item 9キーは常に全て存在する。値の `null` は `CU-G09 §6` が許した箇所だけに使う。
- `I2`: `item_id` と catalog の `scope_ref` は **非空**の bounded string とする（上限は `CU-G09 §6` の ID 128 UTF-8 bytes を継承）。空文字列は受理しない。
- `I3`: `display_name` が空文字列または `null` の場合、その値をそのまま保存する。fallback 文字列・既定文字列・placeholder を作らない。

## §5 拒否規則（`CU-G09 §7` の改訂と追加）

- `B11` **撤回と差し替え**: `CU-G09 §7 B11` の「optional省略」という族名と例示を撤回する。`B11` は以後「**必須keyの省略**」を指す。不合格例: `items[0]` から `provider_ref` key を省略した／語彙entryから `scope_ref` key を省略した。
- `B12` **改訂**: `B12` は入力を拒否する rule ではなく、**非throwの不変条件**である。decoder が fallback・既定値・補完値・placeholder を生成してはならない、という禁止を意味する。空／`null` の `display_name` に `"Unknown"` を入れる実装は `B12` 違反（実装欠陥）であり、入力の拒否理由ではない。**必須keyが省略された入力は `B11` として拒否する。**
- `B13` **新設**: 構造／container 型不一致。不合格例: `catalogs` が object、`items` が string、`impact.measures` が object。
- `B14` **新設**: scalar／string／nullability 不一致。不合格例: `item_id` が number、`display_name` が number、`label` が boolean、`scope_ref` が number。`B9`（非finite数値）とは重複させず、`B9` は数値欄へ入った非finite値、`B14` は型そのものの不一致を担当する。
- `B5` **明確化**: 重複は3種を含む。(a) 各語彙表内の重複 `id`、(b) catalog 間の重複 `scope_ref`、(c) 同一 catalog 内の重複 `item_id`。
- `B7` **明確化**: cross-scope 拒否は `taxonomy_refs` に限らず、**あらゆる scoped reference** に適用する。

`CU-G09 §7` の `B1`〜`B10` は本書の差し替え対象外であり、受理入力の拒否規則として延長（無改変）する。

## §6 CU-G09との関係表

| CU-G09 節 | 本書との関係 |
|---|---|
| §1 FACTS | 延長（無改変） |
| §2 分類語彙 | 延長（無改変） |
| §3 catalog範囲 | 延長（無改変） |
| §4 可視要素インベントリ | 延長（無改変） |
| §5 catalog item identity | 延長（無改変） |
| §6 受理入力契約（語彙entry shape 行） | **差し替え**（`V1` が decoder出力の正本。受理入力の歴史文言は `CU-G09` 本文のまま） |
| §6a 禁止output-key | 延長（無改変）。出力へ `O5` で適用 |
| §7 `B11` | **差し替え**（必須key省略。旧「optional省略」は撤回） |
| §7 `B12` | **差し替え**（非throw不変条件。旧入力拒否例は撤回） |
| §7 `B13` / `B14` | **追加**（本書 §5 で新設。`CU-G09` 本文には未記載のまま） |
| §7 `B5` / `B7` | **明確化**（本書 §5 の文言が decoder実装の正本） |
| §7 `B1`〜`B10`（`B11`/`B12` 除く） | 延長（無改変） |
| §8 fixture oracle | 延長（無改変）。本書 `O1`〜`O6` / `I1`〜`I3` / `V1` / 改訂 `B11`〜`B14` を decoder出力oracleへ追加 |
| §9 非目標とSTOP | 延長（無改変） |
| §10 未決（S） | 延長（無改変） |

## §7 非目標

- `S` 語彙の意味決定、`S` 値の追加、drag payload の設計、Host transport、typed intent、JSX binding、公開API／export、Rust／schema／plugin／community契約、Document 意味、serde 面と serde default、Undo、selection、threshold、golden、fixture、guard test、decoder コード、Media Browser、Browser tab 分類 P41、`CU-0A08BP` の実装、`CU-0A08BT`、`CU-0A08IT`、`CU-109`、`CU-104`、`U2h-1`、`U4a-2`、`docs/reviews/2026-07-25-parallel-lane-readiness-map.md` の編集、`CU-G09` 受理入力契約文書そのものの編集。

## §8 未決（`S`）

`CU-G09 §10` をそのまま維持し、**`S` を1件も解決せず1件も追加しない**。本書は decoder出力shapeと拒否family写像だけを閉じ、語彙内容・tag owner・P41・bare `itemId` drag payload・`thumbnail`/`kind`/`type`/`mode` 等の `S` 行には触れない。
