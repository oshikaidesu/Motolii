# Motolii(リセット後)

MV 制作のためのモーショングラフィック指向コンポジットツール。
**構造としては、rerun store + re_renderer + iced + FFmpeg の薄いラッパーである。**

裁定の経緯は [../docs/reviews/2026-08-20-reset-to-one-axis.md](../docs/reviews/2026-08-20-reset-to-one-axis.md)。

## 軸(1本)

| 層 | 誰が持つか |
|---|---|
| Document(identity・履歴・undo) | **rerun store**(`re_entity_db` / `re_chunk_store`)。undo = `edit` timeline の時間移動 |
| 合成・GPU | **`re_renderer`** |
| front | **iced のみ**。pane は store への query の投影であり、独自の状態を持たない |
| 素材 IO | **FFmpeg** |
| Motolii が持つもの | AE の意味(component 定義)、評価器(comp 時間 → 値)、製品 policy、拡張の口1本 |

## 規律 — たった1つ

**各 crate の根(`lib.rs` / `main.rs`)の1行目 doc コメントが `//! wraps:` か `//! owns:` で始まること。**

```rust
//! wraps: re_entity_db::EntityDb — Document の実体。undo は edit timeline の latest-at。
```

```rust
//! owns: keyframe の eased 補間。rerun の latest-at は step 補間しか持たない(実測 R0-3)。
```

- `wraps:` = 上流機構の薄い口。**中身を知りたければ上流を読む**。ここに機構を書き足さない
- `owns:` = 上流に無いと**主張**している。この行だけがレビュー対象であり、
  「読んでいなかったから再発明した」は `owns:` の一覧を見れば全部そこに出る

`./check.sh` が (1) marker の書き忘れ (2) `owns:` の全一覧を**行数つきで** (3) `wraps:` の一覧 を出す。
行数を並べるのは、3,000行の `owns:` と 50行の `owns:` が同じ重さの主張ではないため。
**リンク台帳も索引も持たない** — ラッパーに必要なのは「どの上流を包んだか」だけで、
それはコードの隣にあるのが最も腐りにくい。

## 現在の crate

| crate | marker | 中身 |
|---|---|---|
| `core/motolii-store` | `wraps:` | `EntityDb` の口。Document / StoreView / Intent / undo / redo |
| `core/motolii-core` | `owns:` | 有理数フレーム時刻。rerun の `TimeInt` は有理 fps を持てない(旧 workspace から移植) |
| `core/motolii-eval` | `owns:` | keyframe 補間と bezier 分割。rerun の latest-at は step 補間のみ(旧 workspace から移植) |
| `engine/motolii-compositor` | `wraps:` | `re_renderer` の口。layer = 板、preview と export は同じ `render()` |
| `engine/motolii-engine` | `wraps:` | store と compositor を繋ぐ。**1フレームを出す唯一の経路** |
| `probes/r0-store-edit` | `owns:` | 軸が立つことの実測。fork の rev を上げたら回す |

`engine/` の media・export と、`shell/`(iced)はまだ無い。

## 裁定

[DECISIONS.md](DECISIONS.md) に追記だけする。1裁定1行、リンクを張らない。
