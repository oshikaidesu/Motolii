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
| `probes/r3-pointcloud` | `owns:` | 実データの PLY 点群が point_cloud renderer で撮れ、カメラ移動で視差が出るか(D12) |

`shell/`(iced)は骨が立っている(store 投影+Session のみ。2026-08-20 実機起動済み)。

## 旧ステージの理想文書

実装の正本はこの workspace だが、**理想・概念(vism 候補・空間モデル・UI 品質バー・拡張の憲法)は旧 `docs/` に正本が残っている**。
どれがまだ拘束するかは [reference/CANON.md](reference/CANON.md) — **発注前に該当領域の行を確認する**こと(2.5D 見落とし事故の再発防止、2026-08-20)。

## Lottie を地図として使う

`reference/lottie.schema.json` は **Lottie 公式の機械可読スキーマを上流そのまま**置いたもの。
Lottie は Bodymovin が After Effects のデータ模型を吐いた物なので、**実質 OSS の AE 解析**である。

`reference/lottie-coverage.tsv` はそこから機械生成した**全語彙 656項目の地図**で、
1項目ずつ「採用済 / 採用予定 / 不採用 / 未判定」を書く。

**「作る瞬間に読む」方式は採らない** — 読まなかった物が構造的に見えないから。
先に全部並べて、`cargo test -p motolii-store --test lottie_coverage` が
「スキーマにあって表に無い」「表にあってスキーマに無い」を落とす。
`./check.sh` が毎回 **未判定の数**と**発注単位ごとの残り**を出す。

地図の列は6つ。**新しい台帳を作らないための形**である:

| 列 | 役割 |
|---|---|
| `status` | 採用済 / 採用予定 / 不採用 / 未判定 / 該当なし |
| `note` | 判断の理由。**不採用の理由を必ず書く** |
| `evidence` | 採用済 の行が指す**コード中に実在する識別子**。試験が grep するので**自己申告にならない** |
| `unit` | 採用予定 の行が属する**発注単位**。束は地図の見え方の1つで、別の台帳ではない |

**束の完了条件 = その束の行が全部 `採用済` になり、evidence がコードに実在すること。**
だから完了は機械で判定でき、発注もそのまま切り出せる。

### コンポーネント契約から粒を導出する

意味を持つコンポーネントの実装ファイルには `motolii-component` 契約を置く。
契約の `entry / meaning / evaluation / render / observable` は、利用者の1つの意味を
検収する5粒である。`scripts/derive_components.py` が契約を読み、
`reference/generated/components.tsv` と `components.md` を生成する。

契約ブロック以外のコードから各証拠名を探すため、名前を契約に書いただけでは緑にならない。
実装・結線・評価・描画・観測のどれかが欠ければその粒だけ赤になる。
`maps = []` は外部地図に対応しないMotolii固有の意味に限る。

重みの初期定数は、機能の大きさではなく優先順位を表す:

| weight | 値 | 意味 |
|---|---:|---|
| `truth_safety` | 5 | 保存・破壊防止・結果の正しさ |
| `core_edit` / `render_export` | 4 | 通常編集経路・画面と書出しの真実性 |
| `fanout` | 3 | 他の意味へ及ぼす波及 |
| `frequency` / `portability` | 2 | 使用頻度・再開や受け渡し |
| `convenience` | 1 | ショートカットや補助入口 |

この数値は外部製品の事実ではなく、Motoliiの優先順位である。外部資料は `maps` の採否を
支え、PageRank は `fanout` の判断を補助するが、どちらも重みそのものを決めない。

コンポーネントの切り分けは、**独立した意味・状態遷移・失敗方針・検収結果**を1単位とする。
別の undo/recovery 方針、別 owner、別の観測結果を持つなら分ける。単独では利用者に意味が
見えないUI部品や補助関数は、意味コンポーネントに数えず内部実装に留める。

## 現在の実装ルート

実装は `plan_steps.py` が示す**最も早い未通過step**を起点にする。stepを進める途中で、
必要な意味コンポーネントが無い、または契約の粒が赤なら、そのコンポーネントを先に閉じる。
コンポーネントが緑になったらstepへ戻り、入口から利用者が観測できる結果まで通す。

したがって、stepとcomponentは競合しない。stepが需要を決め、componentがその需要の依存を
閉じる。componentだけを先回りして増やさない。前半で複数stepに効く基盤を作り、後半ほど
新規実装量を減らす。ただし後半の表現・再リンク・受け渡しは、量が少なくても意味と検収が
重くなりうるため、5粒の赤を残したまま完了とはしない。

## 並列レーンへの発注テンプレート

レーンごとの発注は、現場監督の作文ではなく次の固定項目で作る。`plan_waves.py` が出す
write-set と、component契約の赤粒をそのまま埋める。

```text
OUTCOME: 状態の変化を1つ(「確認する」ではなく「何が変わるか」)
STEP: procedures/P*.md の手順番号・map id
COMPONENT: component id と、閉じる5粒(entry/meaning/evaluation/render/observable)
TARGET: 変更してよいファイルの絶対的な範囲
WRITE-SET: plan_waves.py の責任ファイル集合
WIRE-SET: `//! responsibility: wire` を持つ結線ファイル(意味レーンから除外)
DO-NOT-TOUCH: 他レーンのファイルと先回りの機能
STATIC: inventory / derive_components / derive_entries / coherence / rehearse_parallel / diff --check
CARGO: 原則なし。例外は enum 網羅性・公開型境界・借用生存期間・波末の消費点だけ
RETURN: 変更ファイル、evidence(file:line)、残った赤、実行した検査と終了コード
```

エージェントは通常 `cargo` を回さない。静的検査で答えられる問いを先に閉じ、supervisorが
波の終端または消費点で、影響crateをまとめて `cargo check --tests` する。`cargo test` は
実窓・push・引き継ぎ前、または前波の実行時挙動に依存する束の前だけに置く。これでcargoの
lock待ちを並列の上限にしない。

`Shell` rootのような結線ハブは意味componentのwrite-setへ混ぜない。コード側に
`//! responsibility: wire` を宣言し、意味レーン完了後のWIRE結線へ送る。外部上流の欠如は
偽の責任ファイルを割り当てず、`(外部依存)`として残す。本当に所有者が無い穴だけを
`(責任ファイル未記入)`として赤にする。

現行Shellの実装境界もこの形に揃えている。`render.rs` は描画・読み取り口だけ、
`settings_ops.rs`/`gizmo_ops.rs`/`marker_ops.rs` は各意味更新、
`render_dispatch.rs` はMessage分配だけのWIREである。Inspectorの横断値ドラッグも
`value_drag.rs` を共通gestureのWIRE adapterとし、Composition・Settings/Background・
Text色のdraft/commitは `value_drag_composition.rs`/`value_drag_settings.rs`/
`value_drag_color.rs` へ分けている。`check_responsibility.py` がWIREのDocument書き込みと、
描画moduleへの書き戻りを赤にする。

並列を開始する前は `rehearse_parallel.py` を実行する。これは実装を変更せず、意味レーンを
同時に読み、write-setの交差・WIREの漏れ・全レーンの完了を検査する。40レーンが終わらない
場合は、エージェントを増やす前に責任境界か台帳の誤りを直す。


### Lottie は**書き出し専用**(2026-08-23 実測・取り込みは未決)

`import_lottie`/`from_lottie`/`parse_lottie` は workspace のどこにも無い。
`.json` を吐けるが読み戻せない。**これは妥当な設計でもある**(保存形式は
`persist.rs` が別に持っており、Lottie は出力先) — が、**明文が無い**ので
「決めた」のか「忘れた」のか区別がつかない状態だった。裁定54 は `.aep` の
import を「将来の別問題」と書くが、Lottie 取り込みには触れていない。

**帰結として見落としやすい形が1つ在る**: 型・描画・書き出しが揃っているのに
UI に入口が無い語彙(polystar / gradient / trim / repeater / 角丸)は、
取り込みも無いため**利用者が到達する経路がゼロ**。試験の中にしか存在しない。
安い穴(材料が全部在り、要るのは入口1本)なので早い段階で拾える。


## 答えは静的な物の中に在る — 高価な探査で出し直さない

同じ一手が3回効いた。**動的に走らせて確かめていた問いが、すでに手元の静的な物から
出せた**という形が繰り返し当たる。

| 前(動的に確かめていた) | 後(静的な物から出す) |
|---|---|
| 何が在るか・どう呼ぶか → 適当に書いて `cargo` に怒られて直す | `generated/inventory.tsv` の7列目に型 |
| どれが重い・先か → 勘、あるいは `cargo tree` | `rank_load_bearing.py`(**台帳の辺に PageRank**) |
| 並列にできるか → 走らせて壊れるか見る | `plan_waves.py`(write-set が交わるか) |

**PageRank が cargo を減らしたのではない。** 別々の問いに同じ置き換えを当てただけ。
だが効き方は繋がっていて、**cargo に聞く回数が減った分そのままレーンが待たなくなり、
並列の上限が cargo の lock ではなく write-set の交差で決まるようになった**。

cargo にしか答えられないのは2つだけ — **網羅性**(`match` の腕・enum に足した時)と
**借用・生存期間**。だから `cargo check --tests` は**書き終わってから1回**(裁定220/228)。

**次にこの形を疑う場所**: 「確かめるために走らせている」物すべて。走らせる前に、
その答えがすでにどこかのファイルに書いてあるかを見る。

## 時間の予算を測る

R1(合成のスループット)は GPU を単独で使う必要があるので既定の `cargo test` では走らない。

```sh
cargo test --release -p r1-frame-throughput -- --ignored --nocapture --test-threads=1
```

他の GPU 試験と並列に走らせると等倍40枚が 40ms → 77ms へ倍近く伸びる。
予算を緩めて通すと見張りとして死ぬので、単独で走らせる方を選んでいる。

## 裁定

[DECISIONS.md](DECISIONS.md) に追記だけする。1裁定1行、リンクを張らない。
