# CU-G09R Browser decoder拒否優先順決定

- 日付: 2026-07-26
- 状態: 決定
- 粒: CU-G09R DONE

## §1 適用範囲

本書は [CU-G09 Browser catalog projection契約決定](2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md) と [CU-G09O Browser decoder output契約決定](2026-07-26-cu-g09o-browser-decoder-output-contract-decision.md) を**延長**する追補である。`CU-G09` と `CU-G09O` の本文は1バイトも編集しない。本書が閉じるのは、一つの不正入力を一つの拒否familyへ決定的に帰属させる優先規則と、`CU-G09O I2` が閉じていなかった残余ID/ref境界だけである。

## §2 site-owned family

次の3 family は、該当 site において**唯一**の拒否familyとする。同 site の違反は他familyへ再帰属しない。

| Family | 独占 site | 規則 |
|---|---|---|
| `B1` | top-level `catalog_revision` | `CU-G09 §7 B1` と同義。`catalog_revision: "1"` / `2` / `1.5` は型を問わず `B1`（`B14`／`B9` にしない） |
| `B2` | top-level key集合 | `CU-G09 §7 B2` と同義。top-level のキー欠落・過剰は `B2`（`B11`／`B3` にしない） |
| `B4` | 各 item の `preview_kind` | `CU-G09 §7 B4` と同義。`preview_kind` の非enum値は型を問わず `B4` |

## §3 判定順

### 3.1 走査順

`CU-G09 §6` の宣言順に従う。

1. top-level `catalog_revision`（`B1`／`B2` は site-owned のためここで確定）
2. `vocabularies` 配下を次の表順: `scopes` → `taxonomies` → `providers` → `packs` → `install_states` → `impact_units` → `tags`
3. 各語彙表の entry を index 昇順
4. `catalogs` を index 昇順、各 catalog 内 `items` を index 昇順

### 3.2 node単位の段階順

各 node（object／配列要素）について、前段の gate を通過し、その family が判定可能な node にだけ次を適用する。

`B11` → `B3` → `B13` → `B10` → `B14` → `B15` → `B9`

- `B10`（string byte上限）は string 型が確定した後にのみ適用する
- `B10`（container件数上限）は container 型（`B13`）が確定した後にのみ適用する

### 3.3 snapshot全体の関係検査

全 node の段階順を通過した後、snapshot 全体に対して次を適用する。

`B5` → `B6` → `B7`

### 3.4 一意性

走査順・段階順の合成により、**最初に成立した違反1件だけ**を拒否理由とする。一つの不正入力に対する拒否familyは常に厳密に1つである。

## §4 family境界の明確化

### 4.1 `B13`／`B14`（container／scalar 排他）

| 条件 | Family |
|---|---|
| 期待値または実値のいずれかが container（object／array）である型不一致 | `B13` |
| 期待値と実値がともに非container scalar（string／number／boolean／null）である型不一致 | `B14` |

scalar期待・container実値（例: `item_id: {}`）は `B13` へ一意に落ちる。

### 4.2 `B9`／`B14`（primitive class と非finite）

| 条件 | Family |
|---|---|
| primitive class の不一致（string／number／boolean／null の期待と異なる class） | `B14` |
| 既に number である値の非finite違反（`NaN` / `Infinity` / `-Infinity`） | `B9` |

`catalog_revision` への safe-integer制約は §2 site-owned の `B1` が常に所有する。他の数値欄へ safe-integer 規則を追加しない。

### 4.3 `CU-G09O` 既存不合格例の保存

| 不合格例 | Family（本決定後） |
|---|---|
| `catalogs` が object | `B13` |
| `items` が string | `B13` |
| `impact.measures` が object | `B13` |
| `item_id` が number | `B14` |
| `display_name` が number | `B14` |
| `label` が boolean | `B14` |
| `scope_ref` が number | `B14` |
| 語彙entryから `scope_ref` key を省略 | `B11` |

## §5 B8の処分

`CU-G09 §7 B8` の一般化された旧帰属（語彙entry不備: `{ id: null }` / `label` 非string）は、本決定が**明示的に差し替える**。`B8` 行は削除せず、**予約・到達不能。ID再利用禁止**とする。

到達不能の根拠:

- 語彙entry key省略（`scope_ref` 省略）は `CU-G09O §5 B11` の明示例
- `label` 非string は `CU-G09O §5 B14` の明示例
- `{ id: null }` は `B14` の nullability 不一致
- 語彙entryの過剰key は `B3`
- 語彙表内の重複 `id` は `B5`（`CU-G09O §5 B5(a)`）

`B8` に残る入力は存在しない。

## §6 空ID/ref境界と B15 新設

### 6.1 `B15`（空 ID／ref 文字列）— 新設

次の field／要素が **空文字列**（0 UTF-8 bytes）のとき `B15` とする。

- `item_id`
- catalog の `scope_ref`
- 語彙entry `id`
- `provider_ref`、`pack_ref`、`install_state_ref`
- `taxonomy_refs[]` 各要素
- `tag_refs[]` 各要素
- `impact.measures[].unit_ref`
- 語彙entryの**非null** `scope_ref`（値が `null` の場合は `B15` 対象外）

### 6.2 対象外（空文字を受理）

- `display_name`（`CU-G09O I3`: 空／`null` をそのまま保存）
- 語彙entry `label`（`CU-G09O O3`: verbatim 運搬）

空文字を `B14` へ相乗りさせない。`B14` は scalar／string／nullability **型**不一致を指し、空文字は正しい string 型である。

### 6.3 上限との対

ID/ref の UTF-8 byte 上限128は `B10`、下限1 byte（すなわち非空）は `B15` と対にする。`label` の上限1024は従来どおり `B10` だが、空文字は §6.2 のとおり受理する。

## §7 scope参照

### 7.1 非scoped語彙entryの `scope_ref: null`

`scope_ref: null` は「**全 catalog scope から参照可能**」を意味する。非scoped語彙entryは任意の catalog `scope_ref` から参照できる。

### 7.2 `scopes` 表 entry の `scope_ref`

`scopes` 表の各 entry の `scope_ref` は **`null` でなければならない**。非null 値は `B7`（cross-scope／scoped reference 違反）として拒否する。scope の入れ子や自己参照の例外節は設けない。

## §8 CU-G09 / CU-G09O との関係表

| Family | 本書との関係 |
|---|---|
| `B1` | 延長（無改変）。§2 site-owned で優先 |
| `B2` | 延長（無改変）。§2 site-owned で優先 |
| `B3` | 延長（無改変）。§3 段階順で `B11` の後 |
| `B4` | 延長（無改変）。§2 site-owned で優先 |
| `B5` | 延長（無改変）。§3 snapshot 関係検査 |
| `B6` | 延長（無改変）。§3 snapshot 関係検査 |
| `B7` | 明確化（`CU-G09O §5` の scoped reference 拡張に加え、§7 scopes表entry 非null `scope_ref` を包含） |
| `B8` | **差し替え**（旧一般帰属を退役・ID予約。§5） |
| `B9` | 延長（無改変）。§4.2 で `B14` と排他 |
| `B10` | 延長（無改変）。§3 段階順で型 gate 後 |
| `B11` | 明確化（`CU-G09O §5` 正本。§3 段階順の先頭） |
| `B12` | 延長（無改変。`CU-G09O` の非throw不変条件） |
| `B13` | 明確化（`CU-G09O §5` 正本。§4.1 container/scalar 排他） |
| `B14` | 明確化（`CU-G09O §5` 正本。§4.1／§4.2） |
| `B15` | **追加**（§6。`CU-G09`／`CU-G09O` 本文には未記載のまま） |

## §9 非目標

- `S` 意味決定・`S` 追加、drag payload、Host transport、typed intent、JSX binding、公開API／export、Rust／schema／plugin／community契約、Document意味、serde面とserde default、Undo、selection、threshold、golden、fixture、guard test、decoder コード、Media Browser、Browser tab分類 P41、`CU-0A08BP` 実装、`CU-0A08BT`、`CU-0A08IT`、`CU-109`、`CU-104`、`U2h-1`、`U4a-2`、`docs/reviews/2026-07-25-parallel-lane-readiness-map.md` の編集、`CU-G09`／`CU-G09O` 本文の編集。

## §10 未決（S）

`CU-G09 §10` をそのまま維持し、**`S` を1件も解決せず1件も追加しない**。本書は拒否family帰属の優先順と残余ID/ref境界だけを閉じ、語彙内容・tag owner・P41・bare `itemId` drag payload・`thumbnail`/`kind`/`type`/`mode` 等の `S` 行には触れない。
