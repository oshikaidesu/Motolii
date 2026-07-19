# Rerun先例調査: egui製プロダクトの資産地図(2026-07-20)

ステータス: **決定**。M3 UI基盤はeguiを継続し、Rerunを制作ソフト級の外観・window/panel構成・時間面・GPU viewportを実装するための主要な製品先例とする(2026-07-20ユーザー決定)。Reactモックは引き続きMotolii固有の視覚・操作要求の正本候補であり、Rerunはそれをeguiへ移す際の実装資産地図として使う。

この方向決定は、Rerunの各実装を無条件に採用する決定ではない。本文書の事実表から個別資産を移す時は[レビュー規律](README.md)規律1〜6に従い、crate依存の追加・fork・アルゴリズム移植は§5の反対側レビューと該当ゲート(M3入場PR、M5 decision PR)で判定する。[egui採用審査録(2026-07-18)](2026-07-18-m3-egui-selection.md)を置換せず、外観と実装先例の不足を事後補強する。

具体的な学習順、RR-0〜9の成果物、既存M3/M5タスクへの接続、`DEPEND/VENDOR/PORT/PATTERN/REJECT`分類は[Rerun → Motolii学習・転移計画](2026-07-20-rerun-learning-transfer-plan.md)を運用正本とする。

## 1. 決定と経緯

### 1.1 決定

- UI toolkitの再選定はここで止め、現行のegui + 同一wgpu device/native texture構成を継続する
- Rerunを単なる一般可視化アプリではなく、**制作ソフトに近い高密度viewer shellの先例**として扱う。可変panel、time panel、時間移動・再生、密度表示、選択、GPU viewport、テーマをMotoliiのBrowser / Timeline / Preview / Inspectorへ転移可能か調べる
- Rerunの画面を複製するのではなく、Reactモックが定めるMotolii固有の情報設計と操作を、Rerunで実証されたegui構成・component・描画パターンへ翻訳する
- Motoliiの製品上の大きな軸を、**Rerunが技術データ可視化で成立させた構造を、映像制作のポップで直接操作可能な言語へ再翻訳すること**に置く。これはRerunの機能名や画面を模倣する意味ではなく、時間・view・selection・density・GPU sceneを編集可能な作品操作へ変換する命題である
- Dioxus Native / GPUI / React+Wry / Qt shellは現行実装候補から外し、eguiで視覚審判を満たせない、またはGPU/IME/入力の停止条件が実測で発火した場合だけ再検討する

### 1.2 Motoliiへの再翻訳

この命題は、Rerunの実装をそのまま映像編集へ転用できるという主張ではない。Rerun側で実証された構造と、Motolii側で新たに所有する編集意味を次のように分ける。

| Rerunで実証された構造 | Motoliiでの再翻訳 | そのまま持ち込まないもの |
|---|---|---|
| Time Panel、時間navigation、density graph | clip/keyframeを読む前から全体密度と現在地が分かるTimeline / Score | recording閲覧を編集timelineと同一視しない |
| View / Blueprint / `egui_tiles`投影 | 利用者が組み替えられるBrowser / Preview / Timeline / Inspector | Rerun store、Blueprint schema、生`Tree`の永続化 |
| entity selection、query、outline | 素材・Object・Effect・文字要素をまたぐ選択と編集対象の可視化 | query結果をDocument正本にしない |
| GPU viewport、picking、compositor | VRAM常駐previewとStage上の直接操作 | Rerunの3D意味論、picking readback方式の無裁定輸入 |
| 高密度な技術UIと`re_ui` component | 初見でも読める意味色・icon・spacingによるポップな制作UI | Rerun固有の情報階層や専門家向け語彙 |

ここでいう「Rerunの発明」はMotoliiの製品仮説を表す言葉であり、個々のwidget・アルゴリズムの独占的な発明帰属を主張しない。一次資料で確認した事実と、Motoliiが行う意味の再翻訳を混同しない。

### 1.3 経緯と調査の問い

UI基盤の再検討(「UIはCPU描画でもよいのでは、プレビューだけGPU」の検討、2026-07-20)から派生した。その調査自体の結論は次の3点で、egui採用判断を補強した。

1. 設計軸は「CPU描画かGPU描画か」ではなく「ネイティブサーフェスが1枚か2枚か」。2枚方式(成熟toolkit+GPU子窓)はプレビュー上の同一ウィンドウ内オーバーレイが壊れる古典的airspace問題([WPF公式記録](https://learn.microsoft.com/en-us/archive/blogs/dwayneneed/mitigating-airspace-issues-in-wpf-applications))とWayland非対応([Qt Window Embedding対応表](https://doc.qt.io/qt-6/qtdoc-demos-windowembedding-example.html))を抱える。1枚方式(アプリ所有の単一swapchainへUIを合成)ならこの問題は原理的に存在しない
2. 現行egui構成(同一deviceの`register_native_texture`)は既に1枚方式であり、ゲームミドルウェア(CEF OSR / Ultralight / Coherent Gameface)が実運用で実証済みのパターンと同型
3. KDABのCXX-QtはQt Widgets APIを提供せずQML/Qt Quick統合を対象とするため、Qt Widgetsを採る場合にMotoliiが要求する成熟した公式Rust経路は確認できなかった([cxx-qt README](https://github.com/KDAB/cxx-qt))。少なくともCXX-Qtだけでは完結せず、C++シェルまたは別bindingの保守評価が要る

この過程で、egui製の大規模製品Rerunが**Motoliiのegui採用構成と同世代の依存で出荷されている**ことを確認した。

## 2. 検証済み事実(一次資料つき)

| 事実 | 出典 |
|---|---|
| Rerunはロボティクス/CV向けマルチモーダルデータ可視化基盤。リポジトリはMIT/Apache-2.0デュアルで「Everything in this repository will stay open source and free」と明言 | [rerun-io/rerun README](https://github.com/rerun-io/rerun) |
| 商用はオープンコア型(データカタログ/クラウド側)。資金は2025-03公表のシード | [Rerun公式ブログ](https://rerun.io/blog/physical-ai-data) |
| egui作者Emil ErnerfeldtがRerun共同創業者であり、egui開発はRerunがスポンサー | [emilk GitHub profile](https://github.com/emilk)、[egui README](https://github.com/emilk/egui) |
| mainのworkspace依存は **wgpu 29 / egui 0.35 / eframe 0.35 / egui-wgpu 0.35 / egui_tiles 0.16** — [egui採用審査録](2026-07-18-m3-egui-selection.md)の検証構成と完全同世代 | [rerun Cargo.toml](https://github.com/rerun-io/rerun/blob/main/Cargo.toml) |
| 最新リリース0.34.1(2026-07-07)、通算80リリース、開発活発 | [Releases](https://github.com/rerun-io/rerun/releases) |
| `re_ui`(テーマ・フォント・アイコン・widgetヘルパー)は`(MIT OR Apache-2.0) AND OFL-1.1`でcrates.io公開(0.34.1、2026-07-07。crates.io APIで確認)。カタログは`cargo r -p re_ui --example re_ui_example` | [crates.io/crates/re_ui](https://crates.io/crates/re_ui)、[re_ui README](https://github.com/rerun-io/rerun/blob/main/crates/viewer/re_ui/README.md) |
| re_uiからコードを抜き出してegui外観を改善する第三者crateの前例がある | [Gui-Yom/egui_ui_refresh](https://github.com/Gui-Yom/egui_ui_refresh) |
| `re_renderer`は自前wgpuレンダラ(Vello不使用)で「スタンドアロン利用可・viewer非依存」をREADMEが明言 | [re_renderer README](https://github.com/rerun-io/rerun/blob/main/crates/viewer/re_renderer/README.md) |
| wgpu統合は`egui_wgpu::CallbackTrait`でeguiと同一RenderPassへ直接描画(同一device/queue)。テクスチャ登録方式とは別解 | [re_renderer_callback.rs](https://github.com/rerun-io/rerun/blob/main/crates/viewer/re_viewer_context/src/gpu_bridge/re_renderer_callback.rs) |
| Blueprint: 表示レイアウトをrecordingと別の独立ストアのデータとして持ち、毎フレーム`egui_tiles::Tree`へ投影、変更はdeferred commandでフレーム末尾適用 | [Blueprints concept](https://rerun.io/docs/concepts/blueprints)、[ViewportBlueprint実装](https://github.com/rerun-io/rerun/blob/main/crates/viewer/re_viewport_blueprint/src/viewport_blueprint.rs) |
| viewerのundoはblueprintストアのタイムトラベルとして実装(0.21) | [公式ブログ](https://rerun.io/blog/graphs) |
| time panelは個別イベントを描かず「UIピクセル単位の密度ヒストグラム+ブラー+前フレーム最大値による動的正規化」で大規模イベントをピクセル数比例コストで描画 | [data_density_graph.rs](https://github.com/rerun-io/rerun/blob/main/crates/viewer/re_time_panel/src/data_density_graph.rs) |
| re_rendererのrenderer群: mesh(glTF)・point_cloud・depth_cloud・lines・rectangles・world_grid・compositor等。draw_phases配下に`picking_layer.rs`(GPU picking)と`outlines.rs`(選択輪郭) | [renderer/](https://github.com/rerun-io/rerun/tree/main/crates/viewer/re_renderer/src/renderer)、[draw_phases/](https://github.com/rerun-io/rerun/tree/main/crates/viewer/re_renderer/src/draw_phases) |
| 動画はVideoAsset(mp4)/VideoStreamの再生対応(H.264/265はFFmpeg経由、AV1ソフトデコーダ内蔵)。再生専用でフレーム編集・エンコードは範囲外 | [Video reference](https://rerun.io/docs/reference/video) |
| ダーク専用だったviewerへライトモードを後付けした事例(0.24) | [0.24リリースブログ](https://rerun.io/blog/release-0.24) |
| 全`re_*`crateはリリースごとにロックステップ更新で、semver安定性の約束はない | [Releases](https://github.com/rerun-io/rerun/releases)の各リリースノート |

## 3. 資産地図

### 流用候補(採否は各ゲートで判定)

| 対象 | 想定用途 | 条件 |
|---|---|---|
| `egui_tiles` 0.16 | panel投影(採用済み) | Rerun本体と同版で実戦検証されているという追加証拠 |
| `re_ui`(vendoring) | M3シェルのテーマ・design tokens・list item・DnDヘルパー。モック再現コストの圧縮 | crate依存でなく**vendoring**(毎リリース破壊的変更前提)。フォントはOFL-1.1でG0-6のフォント決定と接続 |
| `data_density_graph.rs`のアルゴリズム | Timelineの密度俯瞰描画(zoom-out時のイベント密度表示) | ファイル/アルゴリズム単位の移植。キーフレーム個別編集レーンとは別物であることを明記 |
| `CallbackTrait`統合パターン | `register_native_texture`方式との比較材料 | 同一RenderPass直描きはoffscreen textureをsampleする経路自体を変える別解であり、`register_native_texture`に存在しないコピーを削減するとは数えない。クランプ・composite・lifecycleを含めM3入場PRの比較実装で判定 |
| re_rendererのM5配管(mesh/point_cloud/depth_cloud/lines/world_grid/picking_layer/outlines) | M5ビューポートの描画基盤の参照または部分fork | §4の未裁定(picking readback)とVello二本立て保守費の裁定が先 |

### 参考(設計だけ読む)

- **Blueprintシステム全体**: 「レイアウト=独立ストアのデータ、毎フレーム投影、deferred command、undo=タイムトラベル」。[egui採用審査録](2026-07-18-m3-egui-selection.md)の「Motolii所有の安定layout modelから`egui_tiles`へ投影し、`Tree`を正本にしない」決定の実証済み先行事例。実装はRerunのECS/chunk storeに結合しており抽出不可
- **クエリキャッシュ層**: 即時モード+毎フレームstoreクエリの再計算コストをキャッシュで20-30倍改善した記録([公式ブログ](https://rerun.io/blog/fast-plots))。Motoliiの評価snapshot/mailbox設計の転移条件確認に使う
- **re_video**: FFmpeg/WebCodecs/AV1ソフトデコーダの構成調査。再生専用のため設計参照に留める
- **ライトモード後付け事例(0.24)**: 「Dark既定・Light同格」方針([ui-visual-language.md](../ui-visual-language.md))の工程参考

### 流用不可(Motolii固有として自前)

- `re_time_panel`そのもの(chunk store深結合、キーフレーム編集要件を満たさない)
- recordingの編集系全般: Document編集・D2 command/単一writer型undo・clip/keyframe作成・カーブエディタ・書き出し。Rerunのtime panelには時間移動・再生・密度表示があるが、Motoliiの編集timelineそのものではない。Blueprint等のviewer状態編集を、映像Document編集の先例へ読み替えない
- M5の意味論の心臓部: 遮蔽ポリシー3方式・soft alpha対応意味論([M5 spec](../specs/M5-3d-and-post.md)方針)・`Preserve Appearance`解析補正・Depth Rail UI言語。Rerunに同種の問題設定が存在しない
- IME実証: RerunにはIMEを酷使する画面がなく、eguiのCJK IME既知問題([egui#3060](https://github.com/emilk/egui/issues/3060)、[Linux #5544](https://github.com/emilk/egui/issues/5544))の反証にならない。[egui採用審査録](2026-07-18-m3-egui-selection.md)のmacOS実機確認+Windows運用確認の方針を変えない

## 4. 台帳整合

**補強される既決事項と今回の方向決定**:

- egui採用([decision-index](../decision-index.md)「UI基盤 egui Slint toolkit」行): 同世代構成の大規模製品実証が存在する
- 安定layout model→`egui_tiles`投影([egui採用審査録](2026-07-18-m3-egui-selection.md)§6): Blueprintが同構造の実証
- panel可変レイアウト(P48/P49): egui_tiles 0.16の実戦検証
- Rerunを主要な製品先例としてegui実装を継続する。Reactモックの視覚・操作要求とRerunの実装資産を混同せず、前者を「何を作るか」、後者を「eguiでどう成立させた先例があるか」に分ける
- Rerunの技術データ可視化構造を、Motoliiでは編集可能でポップな映像制作言語へ再翻訳する。Rerunの語彙・schema・画面構成そのものを製品契約へ焼かない

**未裁定(採用前に決定が必要)**:

- `picking_layer.rs`は**非同期GPU readback**でpickingする。[M5 spec](../specs/M5-3d-and-post.md)「再生中の動的表示と範囲」は表示値のGPU readbackを禁止しているが、pointer picking一回のasync readbackが同じ禁止に含まれるかは未裁定。M5 decision PRで裁定するまでre_renderer picking方式を持ち込まない

## 5. 反対側レビューへ送る論点

規律2に従い、採用判定前に独立レビューで以下を再判定する。

1. **vendoring追従コスト**: re_uiはsemver約束なしのロックステップ更新。egui本体のバージョンをMotoliiが上げるたびvendoredコードの手動追従が要る。追従を放棄した場合の固定費用と、自作した場合の初期費用の比較
2. **描画基盤の二本立て**: M5でre_renderer(fork)を使うとVello系と自前3D系の二本の描画基盤を保守することになる。参照のみに留めて自前実装する案との比較。re_rendererはWebGL互換tier等Motoliiに不要な複雑さも含む
3. **blueprint式undoの転移条件**: 「undo=ストアのタイムトラベル」はRerunでは表示設定のみが対象。MotoliiはDocument編集undo(D2/単一writer/ジャーナル)が主で、Workspace-session候補(panel配置等)はUndo対象外と既決([P48/P49](2026-07-19-m3-interaction-prototype-decision-ledger.md))。輸入すべき部分が実は無い可能性
4. **即時モード+巨大Documentの再計算**: Rerunはクエリキャッシュ層で緩和した。Motoliiの評価snapshot設計で同種のフレーム毎コストがどこに出るかをfixtureで確認するまで「Rerunが証明した」と言わない
5. **単一実例依存**: 「eguiの大規模製品実証」は実質Rerun一件で、egui作者がRerun CTOであることは「eguiの優先順位がRerunの需要に引っ張られる」リスクと表裏。反例(egui採用を撤回した製品)の探索が未実施
6. **CJK/IMEの空白**(§3記載): Rerunの実績はこの領域の証拠として使用禁止

## 6. 停止線

今回の方向決定だけでは次を**行わない**。

- workspace依存へのre_*系crate追加、re_ui/re_rendererコードのvendoring開始(M3入場PR以降、上記反対側レビュー通過後)
- `register_native_texture`方式から`CallbackTrait`方式への変更(M3入場PRの比較実装で判定)
- M5仕様への転記(picking裁定・二本立て裁定が先)
- Web/Wasm版の目標化(Rerunが技術的実在を示したという記録のみ。デスクトップ優先の現方針を変えない)
- Rerunのtime panelをMotoliiの編集timelineとしてそのまま移植すること。密度描画、時間navigation、panel構成の転移と、clip/keyframe編集モデルの自前実装を分離する
