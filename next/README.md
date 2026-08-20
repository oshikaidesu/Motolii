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
| `core/motolii-core` | `owns:` | 有理数フレーム時刻と frame 記述(旧 workspace から移植) |
| `core/motolii-eval` | `owns:` | keyframe 補間と bezier 分割(同上) |
| `core/motolii-store` | `owns:` | Document の意味。保存と検索は `EntityDb` に寄せる |
| `core/motolii-testkit` | `owns:` | 外部ツールが無い時のスキップ方針(旧 8,106行から使う分だけ) |
| `engine/motolii-compositor` | `wraps:` | `re_renderer` の口 |
| `engine/motolii-engine` | `wraps:` | **1フレームを出す唯一の経路** |
| `engine/motolii-media` | `owns:` | フレーム正確 decode / encode / mux(移植) |
| `engine/motolii-export` | `wraps:` | 回して書いて報告するだけ。**compositor を引かない**(背骨2) |
| `probes/r0-store-edit` | `owns:` | store が編集に耐えるか |
| `probes/r1-frame-throughput` | `owns:` | 1080p 40枚が破綻しないか |
| `probes/r2-view-projection` | `owns:` | 毎フレーム投影が予算に収まるか |

`shell/`(iced)はまだ無い。

## 時間の予算を測る

R1(合成のスループット)は GPU を単独で使う必要があるので既定の `cargo test` では走らない。

```sh
cargo test --release -p r1-frame-throughput -- --ignored --nocapture --test-threads=1
```

他の GPU 試験と並列に走らせると等倍40枚が 40ms → 77ms へ倍近く伸びる。
予算を緩めて通すと見張りとして死ぬので、単独で走らせる方を選んでいる。

## 裁定

[DECISIONS.md](DECISIONS.md) に追記だけする。1裁定1行、リンクを張らない。
