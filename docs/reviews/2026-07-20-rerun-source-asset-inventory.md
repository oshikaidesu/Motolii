# Rerun固定commit ソース資産inventory（2026-07-20）

ステータス: **観察**。Rerun公式リポジトリの固定commitをpackage単位で全量棚卸しし、Motoliiから再調査すべき資産群をコード事実つきで記録する。本文書は個別資産の採用、依存追加、vendoring、移植、公開API・Document・plugin契約の変更を決定しない。

方向決定は[Rerun先例調査](2026-07-20-rerun-prior-art-survey.md)、転移の分類・停止線・発注動線は[Rerun → Motolii学習・転移計画](2026-07-20-rerun-learning-transfer-plan.md)が正本である。本文書は、その計画のRR-0に必要な**母集団と調査入口**を補う。

## 1. なぜ再棚卸ししたか

先行調査は`egui_tiles`、`re_ui`、Time Panelのdensity、egui-wgpu callback、`re_renderer`を上から選んだspot auditだった。しかし、固定commitのworkspaceには139 packageがあり、先行調査で名前を挙げた資産はその一部にすぎない。

この差を残したままフェーズ割当や発注へ進むと、次の誤りが起こる。

- 既にRerunにある拡張・試験・観測の実例を見落とし、Motoliiで手探り実装する
- 反対に、目立つ5資産だけをRerunの全体設計と誤認し、製品要件をRerun側から逆算する
- 公開可能なleaf crate、内部結合したviewer subsystem、単なるexampleを同じ「流用可能資産」と呼ぶ
- Git LFS pointerしか取得できていないsnapshotを、画像内容まで監査済みと誤記する

したがって、まずpackage全量を母集団として固定し、その後に必要なassetだけをfile/API単位へ絞る。

## 2. baselineと再現方法

| 項目 | 観察値 |
|---|---|
| repository | [`rerun-io/rerun`](https://github.com/rerun-io/rerun) |
| commit | [`954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`](https://github.com/rerun-io/rerun/commit/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e)（2026-07-19） |
| 取得物 | GitHub commit archive |
| archive SHA-256 | `a891a52e4a56ced5f9d438527894d295fefe0f0ba9e10bf0d47a219f94f07af4` |
| 展開後容量 | 59 MiB（`du -sh`表示） |
| workspace version | `0.35.0-alpha.1+dev` |
| Rust | 1.95 |
| UI / GPU | egui 0.35、egui_tiles 0.16、wgpu 29 |
| package数 | 139（`cargo metadata --no-deps --format-version 1`） |

再取得とpackage列挙:

```sh
curl -L \
  https://github.com/rerun-io/rerun/archive/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e.tar.gz \
  -o rerun.tar.gz
shasum -a 256 rerun.tar.gz
tar -xzf rerun.tar.gz
cd rerun-954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e
cargo metadata --no-deps --format-version 1 > rerun-metadata.json
```

### 2.1 調査範囲

**全量確認したもの**:

- workspaceの139 packageの名前、manifest位置、説明、license宣言、publish宣言、直接依存
- `crates/viewer`内のRust source量と非コード資産の拡張子別件数
- §5の重点候補について、manifest、公開module、代表example、主要call path

**全量確認したもの(独立再調査 2026-07-20、§2.2)**:

- 直近安定release 0.34.1のtag commit、公開日、workspace依存、`[patch.crates-io]`状態

**全量確認していないもの**:

- 全139 packageの全関数・全testの意味
- Git LFS実体が必要な861 PNG snapshotの画像内容
- dependency closureの全第三者licenseと供給網監査
- 各`re_*` crateのcrates.io公開版と固定commit版のAPI差分
- 実行時性能、Windows/macOS/Linux差、IME入力挙動(CJK**表示**が同梱Inter単一fontで豆腐になる公式open issueは§2.2で確認済み。IME**入力**は未確認のまま)

よって「package-level inventoryは全量」「file/API-level auditは重点候補のみ」が正確な完了表現である。

### 2.2 安定release側anchor(独立再調査 2026-07-20)

§2のmain監査commit(`954bf95a`、workspace `0.35.0-alpha.1+dev`)は**未リリースのalpha**である。出荷実績の主張と更新費の見積りには安定release側のanchorを併用する。以下はWeb一次資料で独立確認した観察値。

| 項目 | 観察値 | 出典 |
|---|---|---|
| 直近安定release | 0.34.1(2026-07-07公開)。tag commit [`4efb18f17f6f0e41985cda99a2bdcd012febc8d5`](https://github.com/rerun-io/rerun/releases/tag/0.34.1) | releases page |
| 0.34.1のworkspace依存 | egui / eframe / egui-wgpu 0.35.0、egui_tiles 0.16.0、egui_table 0.9.0、wgpu 29.0、winit 0.30.13 | [Cargo.toml @0.34.1](https://github.com/rerun-io/rerun/blob/0.34.1/Cargo.toml) |
| 0.34.1の`[patch.crates-io]` | 節は存在するが**全行コメントアウト**。egui系はcrates.io公開版のみに依存(fork patch前提の読み替え不要) | 同上 |
| 0.34.1のMSRV | Rust 1.92(main監査commitは1.95 — alpha側が先行) | 同上 |
| release cadence(直近) | 0.31.2(4/8)→0.31.3(4/14)→0.31.4(4/29)→0.32.0(5/13)→0.32.1(5/18)→0.32.2(5/20)→0.33.0(5/29)→0.33.1(6/22)→0.34.0(7/6)→0.34.1(7/7)。**minor概ね月次+随時patch** | [releases](https://github.com/rerun-io/rerun/releases) |
| `re_ui`のcrates.io累計版数 | 278(2026-07-07の0.34.1公開時点。rc含む全crateロックステップ発行) | [crates.io/crates/re_ui](https://crates.io/crates/re_ui) |
| データ互換の自己申告 | `.rrd`は「現在のversionは**直前のversion**が生成した`.rrd`を常に開ける」保証のみ | [ARCHITECTURE.md @0.34.1](https://github.com/rerun-io/rerun/blob/0.34.1/ARCHITECTURE.md) |
| CJK表示 | 同梱fontはInterのみのため日本語label(`歩行`等)が豆腐表示になるissueがopen(2026-05-14起票、label: blocked / egui / bug。"Requires egui/eframe work") | [#12770](https://github.com/rerun-io/rerun/issues/12770) |
| viewer Undo | [#3135](https://github.com/rerun-io/rerun/issues/3135)で0.21導入完了。regression「Undo is broken」[#10304](https://github.com/rerun-io/rerun/issues/10304)は0.24.0で修正 | 各issue |

このanchorの用途: (1) survey側「同世代依存で出荷されている」主張の裏をalphaでなく**tag付き出荷版**で取る。(2) vendoring追従費の見積りにcadence・累計版数の定量を与える。(3) main監査commitと安定版の差(MSRV等)を、後続調査で別commitを読む時の差分基準にする。

## 3. workspace全量

| 群 | 数 | 主な責任 |
|---|---:|---|
| viewer | 38 | shell、UI component、panel、View、GPU renderer、試験、MCP |
| store | 28 | chunk/query/schema/importer/protocol/server/media container |
| utils | 25 | channel、memory、mutex、trace、video、format、error等 |
| build | 5 | codegen、build metadata、開発tool |
| top | 4 | SDK、CLI、C API、統合crate |
| examples | 28 | SDK利用、viewer拡張、importer、callback等の実例 |
| tests | 8 | integration、stress、density、memory、UI wakeup |
| docs / Python / Wasm | 3 | snippets、Python binding、Wasm runner |
| **計** | **139** |  |

### 3.1 viewer 38

`re_arrow_ui`、`re_blueprint_tree`、`re_chunk_store_ui`、`re_component_fallbacks`、`re_component_ui`、`re_context_menu`、`re_data_ui`、`re_dataframe_ui`、`re_gamepad`、`re_memory_view`、`re_plot`、`re_recording_panel`、`re_redap_browser`、`re_renderer`、`re_renderer_examples`、`re_selection_panel`、`re_test_context`、`re_test_viewport`、`re_time_panel`、`re_time_ruler`、`re_ui`、`re_view`、`re_view_bar_chart`、`re_view_dataframe`、`re_view_graph`、`re_view_map`、`re_view_spatial`、`re_view_state_timeline`、`re_view_tensor`、`re_view_text_document`、`re_view_text_log`、`re_view_time_series`、`re_viewer`、`re_viewer_context`、`re_viewer_mcp`、`re_viewport`、`re_viewport_blueprint`、`re_web_viewer_server`

### 3.2 store 28

`re_chunk`、`re_chunk_store`、`re_data_source`、`re_dataframe`、`re_datafusion`、`re_entity_db`、`re_grpc_client`、`re_grpc_server`、`re_importer`、`re_lenses`、`re_lenses_core`、`re_log_channel`、`re_log_encoding`、`re_log_types`、`re_mcap`、`re_mp4_reader`、`re_parquet`、`re_protos`、`re_query`、`re_redap_client`、`re_redap_tests`、`re_sdk_types`、`re_server`、`re_sorbet`、`re_tf`、`re_types`、`re_types_core`、`re_uri`

### 3.3 utils 25

`re_analytics`、`re_arrow_util`、`re_auth`、`re_backoff`、`re_byte_size`、`re_byte_size_derive`、`re_capabilities`、`re_case`、`re_crash_handler`、`re_error`、`re_format`、`re_grpc_headers`、`re_log`、`re_memory`、`re_mutex`、`re_perf_telemetry`、`re_quota_channel`、`re_ros_msg`、`re_rvl`、`re_span`、`re_string_interner`、`re_test_mocks`、`re_tracing`、`re_tuid`、`re_video`

### 3.4 build / top / examples / tests / その他 48

- build: `re_build_info`、`re_build_tools`、`re_dev_tools`、`re_protos_builder`、`re_types_builder`
- top: `re_sdk`、`rerun`、`rerun-cli`、`rerun_c`
- examples: `animated_urdf`、`blueprint`、`blueprint_stocks`、`clock`、`custom_callback`、`custom_importer`、`custom_store_subscriber`、`custom_view`、`custom_visualizer`、`dataframe_query`、`dna`、`extend_viewer_ui`、`graph_lattice`、`incremental_logging`、`lenses`、`log_file`、`minimal`、`minimal_options`、`minimal_serve`、`objectron`、`raw_mesh`、`rerun-importer-rust-file`、`shared_recording`、`spawn_viewer`、`state_timeline_example`、`stdio`、`template`、`viewer_callbacks`
- tests: `log_benchmark`、`plot_dashboard_stress`、`re_integration_test`、`test_data_density_graph`、`test_image_memory`、`test_label_compaction`、`test_out_of_order_transforms`、`test_ui_wakeup`
- その他: `snippets`、`rerun_py`、`run_wasm`

## 4. 非コード資産とlicense

`crates/viewer`で数えた非コード資産:

| 種別 | 件数 | 観察 |
|---|---:|---|
| PNG | 867 | 861件はarchive内でGit LFS pointer。画像内容は未監査 |
| SVG | 103 | 主に`re_ui/data/icons`。個別採用時は由来・商標性を再確認 |
| WGSL | 36 | `re_renderer/shader`。composite、YUV、picking/outline周辺、mesh、line、point、rect、grid等 |
| OTF | 1 | `Inter-Medium.otf`。OFL-1.1 |
| RON | 3 | dark/light theme等 |
| JSON | 1 | design token生成元 |

repository rootのコードlicenseはMIT OR Apache-2.0。`re_ui` manifestは`(MIT OR Apache-2.0) AND OFL-1.1`で、Interの`OFL.txt`とREADMEを同梱する。したがって、`re_ui`を一語で「MIT/Apache」と扱わず、code、font、icon、snapshot、shaderを分けて判断する。

GitHub archiveはLFS objectを展開しない。snapshot harnessの構造はsourceから読めるが、861枚の期待画像の画風・許容差・coverageをこの取得物だけで監査済みとは言えない。

## 5. 先行調査から漏れていた重要資産

以下の`候補分類`は調査の入口を示すだけで、学習・転移計画§6の最終裁定ではない。実装・発注へ入れる前に反対側レビューとMotolii側gap/oracleが必要である。

### 5.1 Viewer拡張の実例群

| source | コード事実 | Motoliiで読む理由 | 結合・限界 | 候補分類 |
|---|---|---|---|---|
| [`custom_view`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/examples/rust/custom_view) | `App::add_view_class`で独自View、visualizer、component UIを登録 | Hostが拡張点を組み立てる責任分割の実例 | Rerun ViewClass/Blueprint/store型へ深く依存。Motolii plugin契約ではない | `PATTERN`候補 |
| [`custom_visualizer`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/examples/rust/custom_visualizer) | `App::extend_view_class`で既存3D Viewへvisualizerとfallback providerを追加 | 既存surfaceを壊さず能力を追加する登録動線 | GPU/data/component意味がRerun固有。第三者自由UIの根拠にしない | `PATTERN`候補 |
| [`extend_viewer_ui`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/examples/rust/extend_viewer_ui) | `re_viewer::App`を独自eframe UIで包み、追加panelとviewerを同居 | shell内への製品面埋込み、UI/logic分離の実例 | Rerun App全体の埋込み例。MotoliiがRerun Viewerを製品基盤にする根拠ではない | `PATTERN`候補 |
| [`viewer_callbacks`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/examples/rust/viewer_callbacks) | play/pause/time/timeline/selection/openイベントをcallbackで外へ通知 | UI投影イベントと外部統合の観察材料 | callback状態所有はMotoliiのsingle writer/commandを置換しない | `PATTERN`候補 |

この群は「Rerunにplugin systemがあるからMotoliiも同じtraitを採る」という証拠ではない。重要なのは、登録、fallback、既存View拡張、shell埋込み、event出力を**別の境界として実例化している**点である。Motoliiでは既決の`NodeDesc`自動UI、純関数plugin、Vism、toolkit隔離が上位に立つ。

### 5.2 外部Importer plugin

[`re_importer`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/store/re_importer)は内部Importerに加え、nativeで`PATH`から`rerun-importer-` prefixの実行ファイルを発見する。外部processにはapplication/recording/entity/timeの型付き引数を渡し、stdoutからRerun log streamを受け、非対応をexit code 66で区別する。[`external_importer`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/examples/rust/external_importer)が最小実例である。

これは、外部process隔離、能力判定、typed invocation、streaming result、非対応と失敗の分離を読む価値がある。一方で、無条件の`PATH`走査、Rerun log protocol、Entity/Time意味をMotoliiのplugin discoveryやVism loaderへ移すことはできない。Motoliiには既に配布・署名・loader・sidecar・純関数境界の別契約がある。

候補分類は`PATTERN`。外部plugin discoveryとして再利用するかは未裁定であり、Vism契約調査へ直結させる前に[plugin authoring](../plugin-authoring.md)とVSM-A0/A1の現行境界を照合する。

### 5.3 Viewer MCP

[`re_viewer_mcp`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_viewer_mcp)は「LLM agentがRerun Viewerを使う」MCP serverである。592 Rust行、Rerun内部依存は`re_log`と`re_protos`。`egui_mcp`の`query_tree`、screenshot、click等を再利用し、Viewerとの接続はgRPC `ViewerControlService`と`egui_inspection` request/responseで行う。Rerun固有toolとしてviewer state、URL open、time cursor等を重ねる。

Motoliiにとって重要なのは、LLMに生の内部状態や任意Rust UIを渡すのではなく、

1. UI inspection protocol
2. Viewer control service
3. generic egui tools
4. domain固有tools

を分けている点である。これはLLMによる実装容易性、操作試験、アクセシビリティtree、screenshot oracleの調査候補になる。

ただし、この観察からMotolii MCP、遠隔操作API、plugin APIを新設しない。gRPC/protobuf、Rerun store ID、URL、time cursorはMotoliiの公開契約ではない。候補分類は`PATTERN`で、M3 test/inspectionとVism authoring支援を別々に評価する。

### 5.4 Viewer試験基盤

| source | コード事実 | 転移可能性 | 制約 |
|---|---|---|---|
| [`re_test_context`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_test_context) | viewer context、store、selection、time、view registry、component registry、egui/wgpu harnessを組み立てる | 複雑な製品contextをfixture builderへ閉じる試験設計 | 17個の内部`re_*`直接依存。コード移植より構造参照 |
| [`re_test_viewport`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_test_viewport) | View登録、blueprint setup、system実行、snapshot保存を拡張traitで束ねる | component testとGPU View snapshotの接続 | Rerun blueprint/queryを前提 |
| `re_ui::testing` / `egui_kittest` | UI componentを実行・snapshot化 | React referenceとegui catalogの比較 | LFS期待画像未取得、Motolii閾値は別決定 |
| `test_ui_wakeup` | UI wakeupの統合test package | non-blocking UIとrepaint条件の負例探索 | Motolii mailbox/generation fixtureへ翻訳が必要 |

候補分類は`PATTERN`。Rerun test crateへの製品依存は不要で、fixtureの責任分割とsnapshot matrixをMotolii testkitへ翻訳できるかを見る。

### 5.5 State Timeline View

[`re_view_state_timeline`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_view_state_timeline)は状態遷移を水平laneとして時間上に表示し、`StateLane`、`StateLaneGroup`、`StateLanePhase`等を公開する。2,663 Rust行、12個の内部`re_*`直接依存を持ち、`re_time_ruler`、selection、viewport、chunk storeへ結合する。

Effect IN/OUT、Character Score、cache/readiness等の「連続curveではない状態区間」の見せ方を読む候補である。ただしRerunのstate schemaをMotolii Scoreへ輸入せず、既決のclip/key/effect意味から必要なlaneを作る。候補分類は`PATTERN`、局所描画algorithmだけ切り出せる場合に`PORT`を比較する。

### 5.6 dense table / Browser / Inspector部品

`re_dataframe_ui`、`re_recording_panel`、`re_redap_browser`、`re_selection_panel`、`re_context_menu`、`re_blueprint_tree`、`re_component_ui`、`re_data_ui`には、dense list/table、filter/sort、tree、selection、property表示、context actionの実装が分散している。

`re_dataframe_ui`だけで8,884 Rust行、22個の内部`re_*`直接依存があり、DataFusion/Rerun table意味を含む。crate依存の第一候補ではない。一方、row virtualization、truncate、empty/loading/error、wide/narrow property、selection affordance、snapshot matrixはBrowser/Inspectorの反例探索に使える。

候補分類は原則`PATTERN`。leaf widgetが`re_ui`へ閉じ、Motolii fixtureで必要性が立つ場合だけ`VENDOR/PORT`を再審査する。

なおdense tableの**外部leaf crate**として、rerun-io保守の[egui_table](https://github.com/rerun-io/egui_table)(0.34.1 workspace依存は0.9.0。sticky column/header、"millions of rows"、可変行高、egui 0.35対応、MIT OR Apache-2.0)が本体と分離配布されている。`re_dataframe_ui`のような内部結合crateと違い`DEPEND`比較候補になり得るため、Browser大量行の必要が立った時にegui_tiles同様の外部crateとして別途審査する。

### 5.7 `re_ui`の実際の面積

`re_ui`は15,196 Rust行、9個の内部`re_*`直接依存を持つ。先行調査で挙げたtheme/list item以外にも、alert、button、combo item、command palette、design token、DnD、card/group/layout、filter/fuzzy、help、icon text、loading、markdown、menu、modal、notification、form、relative time range、collapsing header、syntax highlight、text edit、time drag、testingを含む。

これは「丸ごとvendorすればUIが完成する」規模ではない。Rerun domain依存を含むmoduleとegui leaf helperを分け、MotoliiのReact component mapから必要なものだけを逆引きする。候補分類はmoduleごとに未裁定。

### 5.8 renderer / shader / resource lifecycle

`re_renderer`は23,347 Rust行で、allocator、GPU readback belt、draw phase、picking、outline、screenshot、error tracking、model importer、texture/YUV/video、resource pool、compositor、各drawableを含む。36 WGSLにはcomposite、YUV変換、copy、debug、depth cloud、skybox、mesh、line、jump-flood outline、point cloud、rect、voxel、world grid等がある。

これはrenderer lifecycleと失敗処理の豊富な先例だが、M3 preview接合とM5 3D sceneとmedia decodeを一つの採用判断へ束ねてはならない。候補分類は、

- egui callback / pool / error isolation: `PATTERN`
- 小さいshader/algorithm: `PORT`比較候補
- crate依存またはfork: 未裁定
- Rerunの3D scene意味: `REJECT`候補

と分割して反対側レビューへ送る。

### 5.9 小さい基盤crate

| crate | コード事実 | Motoliiとの関係 | 候補分類 |
|---|---|---|---|
| `re_quota_channel` | 1,871 Rust行。byte量でbackpressureするsync/async channel、長時間block警告 | decode/import等の容量制御の参考。ただしUIのlatest-value mailboxとは目的が違う | `PATTERN` |
| `re_memory` | accounting allocator、memory tracking、limit、leak callstack | cache/VRAM/host memoryの証拠作り | `PATTERN` |
| `re_mutex` | `parking_lot` wrapper、debug deadlock検出、caller location | lock規律の開発時検査 | `PATTERN` |
| `re_perf_telemetry` / `re_tracing` | tracing、metrics、OpenTelemetry接合 | M4/G0-4の計測区間設計 | `PATTERN` |
| `re_memory_view` | memory treeをflamegraph表示 | 開発観測器のUI候補 | `PATTERN` |
| `re_video` / `re_mp4_reader` | video decode/presentation、MP4読取。0.34.1の`re_video` featureは`default = ["av1", "ffmpeg"]`で、H.264は`ffmpeg-sidecar`経由。MotoliiはM0-S2で同crateを不採用とし、自前ffprobe／ffmpeg pipeを採択済み。AV1は`dav1d`(re_rav1d、Linux ARM64除外、`nasm` featureで高速化)、webはWebCodecs(`web-sys`) | Motolii正本／現行codeとの差を前提に、D5の反例、codec fixture、cancel／backpressureを調査。crate採択の独立収束とは扱わない | 原則`PATTERN` |

これらは小さく見えてもRerun全体の型・logging・featureへ結合する。既存Motolii helper検索と標準/既存crate比較を先に行い、「Rerunにある」だけで依存を増やさない。

## 6. 既知候補を全体地図へ戻す

先行調査の5候補は有効だが、全体内の位置を次のように訂正する。

| 既知候補 | 全体内の位置 | 調査上の訂正 |
|---|---|---|
| `egui_tiles` | Rerun外部依存。`re_viewport_blueprint`がruntime投影に使用 | Rerun独自assetではなく、同版の大規模利用例 |
| `re_ui` | 38 viewer crate中の1つ。15k行+domain依存+font/icon/theme | 一括vendoring候補ではなくmodule inventoryが必要 |
| density graph | `re_time_panel`内の局所algorithm + 専用test package | Time Panel全体と分離して`PORT`可否を見る |
| `CallbackTrait` | `re_viewer_context/gpu_bridge`の接合pattern | renderer全体採用と分離する |
| `re_renderer` | 23k行+36 shaderの独立renderer subsystem | M3/M5/mediaを分割しない丸ごと採用は危険 |

## 7. domain結合が強く、初期転移優先度を下げる群

次はinventoryから消さないが、Motoliiの現行課題へ直接持ち込む優先度は低い。

- RerunのChunk Store、Entity DB、Arrow/Sorbet、DataFusion、query、log encoding、protobuf、gRPC server/client、REDAP/catalog
- robotics固有のTF、URDF、MCAP、ROS message、LeRobot、lenses
- map、tensor、text log、dataframe、bar chart等のView完成品
- gamepad、web viewer server、Python/C SDK、Wasm runner
- analytics/auth/crash reporting/server infrastructure

理由は「品質が低い」ではなく、Rerun固有のデータ意味・配布形態・運用へ最適化されているためである。Motolii側に同じ問題が実コード事実として現れた場合だけ再昇格する。

## 8. Motoliiフェーズへの仮置き

これは実装チケット割当ではなく、次にどの仕様と照合するかの調査routeである。

| 観察資産 | 最初の照合先 | 現時点で禁止する飛躍 |
|---|---|---|
| renderer pool / callback / shader | M1コード事実、M3 viewport、M5 GPU scene | M1基盤を`re_renderer`型へ置換 |
| ownership / Blueprint / selection / event | M2 single writer・Document外state | Entity/BlueprintをDocument schemaへ追加 |
| `re_ui` / dense panels / state timeline | M3 fixture・React reference | Rerun画面を製品要件にする |
| cache / quota / memory / telemetry | M4性能審判 | Rerun cache key・thread modelを輸入 |
| picking / outline / compositor | M5意味論・readback裁定 | Rerunの3D意味とselectionを採用 |
| Viewer MCP / inspection | M3 testability、将来のLLM authoring支援 | 公開control APIやplugin APIを先に作る |
| external importer | Vism/sidecar/distribution既決 | `PATH`走査とstdout protocolをplugin契約にする |

M1/M2にも関係はあるが、役割は既決境界の監査・反証である。Rerunを理由に完了済みの基盤契約を再設計しない。

## 9. 調査から実装へ進む強制動線

個別assetを発注または実装候補へ昇格するには、次を順番に満たす。

1. Motoliiのspec ID・決定・fixtureから問題を定義する
2. 現行Motolii codeにそのgapがあることをcall pathとtestで示す
3. 本inventoryから候補clusterを引き、固定commitのfile/APIを追加監査する
4. dependency closure、license、公開性、更新費、Motolii既存helperを比較する
5. `DEPEND / VENDOR / PORT / PATTERN / REJECT`を1 asset単位で反対側レビューする
6. Motolii oracleと負例を先に固定する
7. [発注の強制動線](2026-07-20-rerun-learning-transfer-plan.md#9-rerun参照を発注へ入れる強制動線)へ渡す

**STOP**:

- 本文書の`候補分類`を最終裁定として発注書へ転載した
- package名だけで採用範囲を決め、対象file/APIとdependency closureが無い
- Rerunに存在する機能をMotoliiの新要件として追加した
- Viewer MCPを理由に公開遠隔操作API、Importerを理由にplugin discovery契約を発明した
- LFS pointerしかないsnapshotを視覚oracleとして使用した

## 10. 次の調査キュー

優先順位は「Rerunで目立つ順」ではなく、Motoliiの既決目的と不透明領域を減らす順にする。

1. **拡張境界matrix**: `custom_view` / `custom_visualizer` / `extend_viewer_ui` / callbacks / importerを、Motolii Host・Vism・NodeDesc・sidecar境界と対照する
2. **LLM/test inspection**: Viewer MCP、egui inspection、kittest、snapshot、accessibility treeを、製品APIを増やさず開発・検収へ使えるか調べる
3. **`re_ui` module inventory**: React component ↔ Motolii fixture ↔ `re_ui` leaf moduleを対応づけ、domain依存とasset licenseを分離する
4. **Time表現群**: density、ruler、state timelineを、navigation / semantic zoom / edit意味へ分解する
5. **renderer責任分解**: callback、resource pool、error isolation、readback、outline、compositorを別assetとして監査する
6. **試験資産取得**: 必要な場合だけGit LFSを導入し、対象snapshotの実体と由来を限定取得する
7. **反対側レビュー**: 各clusterについて「より小さいMotolii自作」「既存helper」「一般crate」を比較する

このキューの完了前にRR-1〜9全体を一括発注しない。1回の調査または1チケットは、1つのMotolii問題と1つのRerun asset境界へ縮める。
