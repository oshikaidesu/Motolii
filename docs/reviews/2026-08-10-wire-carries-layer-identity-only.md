# RN wireがlayer粒度のidentityしか運ばない（finding）

日付: 2026-08-10
状態: **finding / 未処分。修理は発注していない**

## 0. この文書の扱い

`AGENTS.md`「**findingは権限ではない**」に従い、**報告と分類だけ**を行う。
**本書を根拠に修理・schema変更を発注しない。** 処分はsupervisor席が別途決める。

## 1. 要旨

`WireProductSnapshot`（`crates/motolii-ui/src/rn_product_host.rs`）が運ぶidentityは
**layer粒度だけ**である。

```text
version / direction / role / host_handle / revision / projection_generation
current_time / primary_layer_id / stage{ selection, bounds } / diagnostics
```

**keyframe、effect、effect param のidentityを運ばない。**

したがって、それらを payload に要求する intent は、
**host側の実装が正しくてもRN routeから撃てない。**

## 2. 実測

2026-08-10、RN hostへR2編集4 intentを接続した際に判明した。
実装は正しく、`process_next` の各armへ到達する。しかし呼び側がpayloadを作れない。

| intent | 必要なpayload | RN側が取得できるか |
|---|---|---|
| `add_position_key` | `target` + `time` | **可能**。`target` は `primary_layer_id`、`time` は呼び側が決める |
| `set_position_key_value` | `target` + **`key: KeyframeId`** + old/new | **不可**。key idがwireに無い |
| `set_position_key_interp` | `target` + **`key`** + `interp` | **不可**。同上 |
| `set_effect_param` | `layer_id` + **`effect_use_id` / `definition_id` / `plugin_id` / `effect_version` / `param_id`** | **不可**。いずれもwireに無い |

**4本のうち駆動できるのは1本だけ**である。

## 3. 同じ形の既出

`WireStageBound` は名前に反して `layer_id` + `display_name` だけを持ち、
**幾何を運ばない**（2026-08-09に別途確認済み）。

**wireは一貫してlayer粒度で切られている。** 個別の事象ではなく設計の輪郭である。

## 4. 影響

`R2-INSPECTOR-EDIT` は「RN→host bridgeが無い」ため止まっていると判定されていたが、
**bridgeが成立しても effect identity が wire に無いため撃てない。**
止めているものが1つ増える。

同様に、keyframeを個別に指す操作（値編集、interp変更、削除）は
**すべて同じ壁の向こう**にある。

## 5. 決めるべきこと（本書は決めない）

- wireへidentityを追加するのか、それとも**別の addressing** を採るのか
  （例: keyを時刻で指す、effectをindexで指す）
- 追加するなら、`MAX_JSON_BYTES = 16_384` の制約とどう両立するか
- host→RNの投影粒度を変えることが、`R1` の「三面同一revision」契約へ影響しないか

**追加が唯一の解ではない。** 現行wireは意図的にboundedであり、
`snapshot_wire()` は `.take(16)` で17層目以降を落とす設計である。
identityを増やす方向はその設計と衝突しうる。

## 6. supervisor側の再発防止

本日、orderが閉じていなかったのは4回目である。

1. 接続先が `#[cfg(test)]` だった（`push_undo`）
2. order自身が自己矛盾していた（Spark、fixture）
3. **接続先の事前条件**を書かなかった（`current_primary == target`）
4. **呼び側がpayloadの全fieldを作れるか**を見なかった（本件）

> **payloadの各fieldを、呼び側が手持ちの情報から作れるか。作れないなら契約は閉じていない。**

## 7. 非目標

- 本書を根拠にwire schemaを変更すること
- addressing方式を決めること
- `MAX_JSON_BYTES` を変更すること
