# Rerun固定commit ソース資産inventory(2026-07-20)

ステータス: **調査台帳(観察)**。egui採用([判断](2026-07-18-m3-egui-selection.md))後の学習素材として、Rerun(rerun-io/rerun)の固定commitとソース資産を棚卸しする。読む対象と所在の記録であり、**依存追加・コード移植・製品実装の許可ではない**。方向の提案は[先例調査・方向決定](2026-07-20-rerun-prior-art-direction.md)、読む順序は[学習・転移計画](2026-07-20-rerun-learning-transfer-plan.md)が持つ。

## 1. 固定点

| 項目 | 値 |
|---|---|
| 対象 | [rerun-io/rerun](https://github.com/rerun-io/rerun) |
| 固定release | [0.34.1](https://github.com/rerun-io/rerun/releases/tag/0.34.1)(2026-07-07公開。調査日時点の最新安定) |
| 固定commit | `4efb18f17f6f0e41985cda99a2bdcd012febc8d5`(tag 0.34.1) |
| ライセンス | MIT OR Apache-2.0(dual。re_ui等の個別crateでも確認) |
| 保守元 | Rerun Technologies AB(open-coreモデル。OSS viewer + 商用Hub) |
| MSRV | Rust 1.92(workspace Cargo.toml) |

固定理由: (1) 調査日時点の最新安定版であること。(2) workspace依存が **egui 0.35.0 / eframe 0.35.0 / egui-wgpu 0.35.0 / egui_tiles 0.16.0 / wgpu 29.0 / winit 0.30.13** で、[egui採用判断](2026-07-18-m3-egui-selection.md)の実測構成(egui/eframe/egui-wgpu 0.35、egui_tiles 0.16、wgpu 29)と一致し、version差の読み替えなしに学習できること。(3) `[patch.crates-io]`は節として存在するが全行コメントアウトで、**egui系はcrates.io公開版のみに依存**しており、fork前提の読み替えが不要なこと。

以後の引用・行番号・ファイルパスは全てこのcommitを基準にする。mainは日次で動くため、固定commitなしの引用を根拠にしない。

## 2. リポジトリ概況

- 一行定義(README): "The data layer for physical AI"。ロボティクス/マルチモーダルデータのlog・query・可視化基盤で、その可視化部分(Rerun Viewer)が**現在実運用されているegui製アプリとして最大級**
- 規模: workspace member約60+。`crates/{build,store,top,utils,viewer}`の5系統+`rerun_py`等
- release頻度(直近): 0.31.2(4/8)→0.31.3(4/14)→0.31.4(4/29)→0.32.0(5/13)→0.32.1(5/18)→0.32.2(5/20)→0.33.0(5/29)→0.33.1(6/22)→0.34.0(7/6)→0.34.1(7/7)。**minorが概ね月次+随時patch**
- 安定性の自己申告(README): "We are in active development. There are many features we want to add, and the API is still evolving. _Expect breaking changes!_"
- データ互換(ARCHITECTURE.md): `.rrd`は「**直前版で生成したfileは開ける**」保証のみで、完全な前方/後方互換はない
- 描画方式(ARCHITECTURE.md): immediate mode。"each rendered frame it will query the in-RAM data store, massage the results, and feed it to the renderer" — 毎frameのin-RAM store問い合わせを前提に最適化を続ける構造

## 3. workspace依存(Motolii関心分のみ)

| crate | version | 備考 |
|---|---|---|
| egui / eframe / egui-wgpu | 0.35.0 | crates.io版。git patchなし |
| egui_tiles | 0.16.0 | rerun-io保守。Motolii採用済みversionと同一 |
| egui_table | 0.9.0 | rerun-io保守。大量行table(未評価の再利用候補) |
| wgpu | 29.0 | Motoliiと同一major |
| winit | 0.30.13 | Motolii実測(0.30.13)と同一 |
| ffmpeg-sidecar | (re_video経由) | **MotoliiのB-2本命と同一crate**をH.264デコードに使用 |
| dav1d(re_rav1d) | (re_video経由) | AV1ソフトデコード。Linux ARM64除外、`nasm` featureで高速化 |

## 4. crate inventory(区分と学習関連度)

ARCHITECTURE.mdの区分に従う。「関連」列はMotoliiから見た区分: ◎=学習主対象 / ○=参考 / −=対象外。

### 4.1 viewer UI(`crates/viewer/`)

| crate | 役割(一行) | 関連 |
|---|---|---|
| re_viewer | viewer本体。"This is the main crate with all the GUI"。native+WASM | ◎ |
| re_viewport | 中央viewport panel(view群のtiling表示) | ◎ |
| re_viewport_blueprint | viewport blueprint の**データモデル**(layout正本) | ◎ |
| re_blueprint_tree | 左panelのblueprint tree UI | ○ |
| re_time_panel | 下部time panel(timeline UI) | ◎ |
| re_time_ruler | 時間ルーラ | ◎ |
| re_selection_panel | 右の選択/inspector panel | ○ |
| re_context_menu | 右クリックmenu | ○ |
| re_ui | テーマ・font・icon・UI helper(design system) | ◎ |
| re_viewer_context | viewer横断の共有context。**`gpu_bridge`(re_renderer↔egui接合)を含む** | ◎ |
| re_renderer | wgpu renderer。"tailored towards re_viewer's needs"だが"can be used standalone"、re_viewer/store非依存 | ○ |
| re_view / re_view_spatial / re_view_time_series / re_view_dataframe / re_view_text_log 等 | view種別ごとの実装(2D/3D、plot、table、log…) | ○ |
| re_data_ui / re_component_ui / re_component_fallbacks | データ→UI投影とfallback | ○ |
| re_viewer_mcp | viewerのMCP口(0.34.0新設。LLM agentからviewerを操作) | ○ |
| re_gamepad / re_plot / re_memory_view / re_recording_panel / re_chunk_store_ui / re_dataframe_ui / re_arrow_ui / re_redap_browser / re_view_* 残り | 周辺panel・view | − |

### 4.2 store / data flow(`crates/store/`ほか)

| crate | 役割(一行) | 関連 |
|---|---|---|
| re_chunk / re_chunk_store | 列指向(Arrow)チャンクの低レベルstore | ○(毎frame queryを成立させる土台として) |
| re_query | storeへのquery層(latest-at等) | ○ |
| re_entity_db | application-level store | ○ |
| re_log_types / re_types_core / re_sorbet | 型・log表現 | − |
| re_data_source / re_grpc_client / re_grpc_server / re_web_viewer_server / re_mp4_reader ほか | 入出力・通信 | − |

### 4.3 utils(`crates/utils/`)

| crate | 役割(一行) | 関連 |
|---|---|---|
| re_video | "Crate for parsing video containers and decoding their contents"。`default = ["av1", "ffmpeg"]`、H.264=ffmpeg-sidecar(CLI)、AV1=dav1d、web=WebCodecs | ◎ |
| re_tracing | "Helpers for tracing/spans/flamegraphs and such"(計測基盤) | ○ |
| re_memory / re_byte_size | メモリ計測・会計 | ○ |
| re_string_interner / re_tuid / re_span / re_format ほか | 汎用基盤 | − |
| re_crash_handler / re_analytics / re_auth ほか | 運用系 | − |

### 4.4 その他

- `crates/top/`: rerun-cli、rerun(SDK+viewer shim)、rerun_c — 対象外
- `crates/build/`: re_build_tools、re_types_builder等のcodegen — 対象外(codegen規律の参考程度)

## 5. 資産ファイル(学習の起点)

固定commit配下で所在確認済みの具体ファイル。

### 5.1 design token / font / icon(`crates/viewer/re_ui/data/`)

- `dark_theme.ron` / `light_theme.ron` / `color_table.ron` — **テーマ実値をcodeでなく.ronデータとして保持**(MotoliiのU0e token JSON→生成方式の先例)
- `Inter-Medium.otf` + `OFL.txt` — 同梱fontは**Interのみ。CJK glyphを含まない**。CJK表示はissue [#12770](https://github.com/rerun-io/rerun/issues/12770)(2026-05-14起票、open、label: blocked/egui/bug)で豆腐表示が報告され、"Requires egui/eframe work"とされている — [egui採用判断§4](2026-07-18-m3-egui-selection.md)の「同梱またはOS font resolver必須」判断と整合する外部事例
- `icons/` — icon資産一式

### 5.2 time panel(`crates/viewer/re_time_panel/src/`)

`time_panel.rs` / `time_axis.rs` / `time_selection_ui.rs` / `time_control_ui.rs` / `data_density_graph.rs`(時間軸上のデータ密度描画)/ `streams_tree_data.rs` / `recursive_chunks_per_timeline_subscriber.rs`(timeline別chunk購読)

### 5.3 re_renderer↔egui接合(`crates/viewer/re_viewer_context/src/gpu_bridge/`)

`re_renderer_callback.rs` / `image_to_gpu.rs` / `colormap.rs` / `mod.rs`。`ReRendererCallback`が`egui_wgpu::CallbackTrait`を実装し、`prepare()`で`ViewBuilder::draw()`のcommand bufferを返し、`paint()`で`set_viewport`+`ViewBuilder::composite()`によりeguiのrender passへ直接合成する(egui-wgpu既定のviewport clampを意図的に回避する記述あり)。**Motoliiのnative texture方式とは別系の接合**であり、扱いは[方向決定§4](2026-07-20-rerun-prior-art-direction.md)参照。

### 5.4 blueprint(公式docs + `crates/viewer/re_viewport_blueprint/`)

公式doc(`docs/content/concepts/visualization/blueprints.md`)より:

- 保持対象: container(Grid/Horizontal/Vertical/Tabs)によるview配置、view種別と設定、背景色・zoom・時間範囲等の視覚property、各panel(blueprint/selection/time)の開閉
- "In general, if you can modify an aspect of how something looks through the Viewer, you are actually modifying the blueprint."
- "blueprints are just data. They are structured using the same Entity Component System as your recordings, but with blueprint-specific archetypes and a separate blueprint timeline."
- 保存/読込: `.rbl` fileとしてSave/Open/drag&drop。"portable and can be version-controlled alongside your code"
- 毎frameの導出: active blueprint+active recordingから、blueprint queryでview仕様を得て、recording queryを引き、描画する(宣言的投影)

viewer本体のUndo/Redoはissue [#3135](https://github.com/rerun-io/rerun/issues/3135)(0.21で完了)で導入済み。regression [#10304](https://github.com/rerun-io/rerun/issues/10304)(0.24.0で修正)からも、undo対象がegui状態でなくviewer編集(=blueprint側のデータ)であることが読み取れるが、**実装機構の内訳は未読**(学習計画の対象)。

## 6. crates.io公開状態

- viewer系crate(re_ui等)はcrates.ioへ**公開されている**(re_ui: 0.34.1、2026-07-07更新)
- ただしre_uiの累計公開版数は278に達し、release candidateを含め**releaseごとに全crateへ新版を発行する運用**。repo全体の"Expect breaking changes!"宣言と合わせ、外部依存には高いchurnコストがかかる(採否判断は[方向決定](2026-07-20-rerun-prior-art-direction.md))

## 7. 未検証・調査の限界

- crate個別(re_ui/re_viewer等)のsemver方針の明文は未発見。README全体の"Expect breaking changes!"のみ確認
- 総行数・crate数の正確な集計は未実施(「約60+ workspace member」はCargo.toml目視)
- time panelの描画量制御(culling/仮想化)の実装内容、chunk store/queryのcache構造、re_tracingが包む具体profiler、blueprintのversioning/migration方針、音声の扱いは**未読**。[学習・転移計画](2026-07-20-rerun-learning-transfer-plan.md)の残課題に登録
- 本文書の要約はWeb取得したファイル・公式docへの依存であり、ソース全文の精読はこれから。引用の再確認は固定commitで行えるようにした

## 8. 一次資料

- release/tag: [0.34.1 release](https://github.com/rerun-io/rerun/releases/tag/0.34.1) / [releases一覧](https://github.com/rerun-io/rerun/releases)
- workspace: [Cargo.toml @0.34.1](https://github.com/rerun-io/rerun/blob/0.34.1/Cargo.toml) / [ARCHITECTURE.md @0.34.1](https://github.com/rerun-io/rerun/blob/0.34.1/ARCHITECTURE.md) / [README @0.34.1](https://github.com/rerun-io/rerun/blob/0.34.1/README.md)
- blueprint: [Blueprints(公式doc)](https://github.com/rerun-io/rerun/blob/0.34.1/docs/content/concepts/visualization/blueprints.md)
- 接合: [gpu_bridge/re_renderer_callback.rs @0.34.1](https://github.com/rerun-io/rerun/blob/0.34.1/crates/viewer/re_viewer_context/src/gpu_bridge/re_renderer_callback.rs)
- video: [re_video Cargo.toml @0.34.1](https://github.com/rerun-io/rerun/blob/0.34.1/crates/utils/re_video/Cargo.toml)
- font/theme: [re_ui/data @0.34.1](https://github.com/rerun-io/rerun/tree/0.34.1/crates/viewer/re_ui/data)
- issue: [#12770 CJK font support](https://github.com/rerun-io/rerun/issues/12770) / [#3135 Undo(and redo) in viewer](https://github.com/rerun-io/rerun/issues/3135) / [#10304 Undo is broken](https://github.com/rerun-io/rerun/issues/10304)
- ecosystem: [egui_tiles](https://github.com/rerun-io/egui_tiles)(0.16.0、egui 0.35対応) / [egui_table](https://github.com/rerun-io/egui_table)(0.9.0、egui 0.35対応) / [crates.io re_ui](https://crates.io/crates/re_ui)
