# M3: UI統合

ステータス: **ドラフト / UI責任境界・surface topology決定、platform受入比較中**([M2基盤再締結ゲート](../reviews/2026-07-15-m2-foundation-reclosure-gate.md)はmainで解除済み。[UI runtime責任境界](../ui-runtime-architecture.md)はReact chrome + native Stage/Timeline + headless interactionへ固定し、通常windowは[1 top-level wgpu Surface + 2 native viewport + opaque child WebView islands](../reviews/2026-07-21-ui-surface-topology-decision.md)へ固定した。React所有面は[直接移管契約](../reviews/2026-07-22-m3-react-product-asset-promotion-contract.md)R0〜R6に従うproduct-owned packageとmock consumer化だけ先行可。現行mainのegui shell等は比較基準として保持し、G0-9実機spikeまではWebView/native製品統合とegui撤去を停止する。plugin UI公開契約はG0-9と分離したG0-3 / GAP-13の判断まで停止する。toolkit/renderer非依存の状態所有、layout/hit-test、domain intent、Command境界、Rust coreは各既存依存に従い進行可)

> **着手前規約**: [M3 UI境界汚染の予防](../reviews/2026-07-14-m3-ui-boundary-prevention.md)のうち、後掲「GR-UI審判割当表」で対象タスクへ割り当てた項目を先に通す。全製品UIは[UI操作言語](../ui-interaction-language.md)、外観を伴うタスクは[UI視覚言語](../ui-visual-language.md)、[UI参照地図](../ui-reference-map.md)、[React製品資産の直接移管契約](../reviews/2026-07-22-m3-react-product-asset-promotion-contract.md)、時間面・Timelineを伴うタスクは[時間面UI構成モデル](../ui-score-model.md)も適用する。React所有面は固定source assetを直接移管し、縮約再実装しない。モックの具体値・未決機能の意味論はDocument/公開契約ではない。非該当項目を形式的にYesにしない。Documentスキーマへ触る場合は[M2恒久焼き込みの予防](../reviews/2026-07-12-m2-permanence-prevention.md)も同時適用する。

## 目的(退治する落とし穴)

A-1(egui候補は既存device/native texture共有を[採用時の実機証拠](../reviews/2026-07-18-m3-egui-selection.md)で確認済み。React/WebView/hybrid候補は[G0-9](../reviews/2026-07-21-m3-react-webview-runtime-reconsideration.md)でCPU bridgeなしのStage接合を再審判)、D-3(OpenCut流用の期待値管理)。

## M3仕様確定ゲート(G0)

| ID | 内容 | 状態 | 確定条件 |
|---|---|---|---|
| G0-1 | eguiと既存wgpu共有の成立性 | **測定完了 / 採否はG0-9再評価中** | [egui採用判断](../reviews/2026-07-18-m3-egui-selection.md)。Apple M4 / Metalでcore-first device、native texture、lifecycle、日本語IMEを実機確認。証拠はG0-9比較入力として保持 |
| G0-2 | 入力/キーマップ/アクセシビリティ最小意味論(GAP-6) | **完了** | [着手前決定§2](../reviews/2026-07-16-m3-preflight-decisions.md#2-g0-2-inputとui状態の意味)。全shortcut再割当、安定Command、状態寿命、version付きJSON keymap、v1保証/非保証を固定 |
| G0-3 | plugin UIモデル(GAP-13) | **再評価中 / 自動panel fallbackは維持** | [軸分離決定](../reviews/2026-07-22-m3-surface-extension-axis-separation.md)に従い、first/third-party pluginの公開kit、sandbox、権限、互換、配布をG0-9の製品surface合否と分離して比較する。G0-9証拠は入力にできるが完了だけで解除しない。`NodeDesc`自動panelだけで全保存paramを編集できる条件は維持し、比較前に自由UI公開契約を実装しない |
| G0-4 | 性能測定プロトコル | **完了** | [着手前決定§4](../reviews/2026-07-16-m3-preflight-decisions.md#4-g0-4-性能測定プロトコル)。絶対閾値は初回実測後の独立改訂 |
| G0-5 | UI境界規約の反対側レビュー | **完了** | [R1〜R9](../reviews/2026-07-14-m3-ui-boundary-counter-review.md)を予防文書・本仕様へ反映 |
| G0-6 | 視覚言語tokenと認知審判 | **手順完了・目視待ち** | [着手前決定§5](../reviews/2026-07-16-m3-preflight-decisions.md#5-g0-6-見た目はuxの投影として導出する)。U0e-1の生成機構、U0e-2Rの固定React比較baseline再結合、U0e-2のreference fixtureだけ先行可。具体token値と製品componentを入れるU0e-3はG0-6Hの人間審判まで待つ |
| G0-7 | 操作単純化・共通componentゲート | **完了** | [UI操作言語](../ui-interaction-language.md)と[着手前決定§6](../reviews/2026-07-16-m3-preflight-decisions.md#6-g0-7-操作文法を共通部品の契約にする)をU2c conformanceへ固定 |
| G0-8 | resource予算presetとpreview縮退設定 | **意味完了・実測待ち** | [着手前決定§7](../reviews/2026-07-16-m3-preflight-decisions.md#7-g0-8-resource値はm4の事実から決める)。具体値だけG0-4+M4-K1a後に決定 |
| G0-9 | React chrome + native Stage/Timelineのsurface統合 | **責任境界・topology決定 / React asset直接移管可 / platform受入継続** | [UI runtime責任境界](../ui-runtime-architecture.md)、[軸分離決定](../reviews/2026-07-22-m3-surface-extension-axis-separation.md)、[React直接移管契約](../reviews/2026-07-22-m3-react-product-asset-promotion-contract.md)、[surface topology決定](../reviews/2026-07-21-ui-surface-topology-decision.md)を正本とする。R0〜R6のproduct-owned React packageとmock consumer化は先行可。その後のbuilt-in WebView Hostは[歴史回収で再採択した不変条件](../reviews/2026-07-23-historical-react-webview-lineage-recovery.md#3-built-in-webview-hostへ再採択する不変条件)に従い、offline bundle、closed typed transport、Host epoch、strict origin/lifecycleを一境界ずつ再契約する。旧H1のversion・role・wire値は現行contractではない。通常windowは1 top-level wgpu Surface内のStage/Timeline viewportとopaque child WebView islandsへ固定し、macOS公式wry sampleの実機合成・resize・Web focus/AXを確認済み。残る審判はdirect wgpu(+Vello局所)対egui同条件baseline、IME/VoiceOver、100回resize/DPI/capture/lost、WebView crash、Windows実機。合格までWebView/native製品統合・egui撤去を停止する。plugin sandbox／公開契約はG0-3で別判定し、G0-9合格へ含めない |

以下は**M3入場(U0a完了)後**の論理依存表である。G0自体はM3全コードを一括停止する門ではないが、初回Uシリーズは下表の論理依存に加えて本書の直列運用を優先する。U1aはU0bの5層所有とdomain intentを待ち、custom UI追加タスクはG0-3の判定後に初めて起票する。U0〜U9を一括または並走発注しない。

## 方針(2026-07-21: UI責任境界・surface topology決定、G0-9 platform受入比較中)

以下のegui節は2026-07-18採用時のbaseline仕様と成立証拠として保持する。責任境界は[正本](../ui-runtime-architecture.md)へ移り、G0-9完了前のWebView/native統合やegui撤去の許可ではない。toolkit横断のDocument/command/thread/座標/preview規律は引き続き現行である。

- **egui候補**。[採用判断](../reviews/2026-07-18-m3-egui-selection.md)時の初期統合はegui/eframe/egui-wgpu/egui-winit 0.35、egui_tiles 0.16、wgpu 29の組合せをadapter内で固定した。versionはDocument/plugin契約へ出さない
- プレビューは同一device上の`Rgba8Unorm` `TextureView`を`egui_wgpu::Renderer::register_native_texture`へ登録して表示する。display slot生成時にtextureと安定viewを一度作り、rendererを得られる`eframe::CreationContext`で一度だけnative texture登録する。frame更新、resize、DPI変更、minimize/restoreごとに登録し直さない
- Stage上の2D handle、selection outline、3D gizmo、Depth rail/axisは、toolkitにかかわらずcanonical display texture後段のnative wgpu presentation overlayで描く。canonical render/exportへ焼かず、少数固定形状のhit-testはCPU解析幾何、確定はD2とする。Webはtoolbar/control/a11y proxyを所有できるが、transparent WebViewをgizmo要件にしない。実装許可ではなく、合格条件は[native Stage所有境界](../reviews/2026-07-21-native-stage-gizmo-ownership.md)を正とする
- **wgpuバージョンはcoreとegui-wgpuで単一化**する。更新PRではCargo treeの重複拒否、既存device共有、native texture、resize/minimize/restoreを再検証する
- egui/eframe/winitのAPI変更は`motolii-ui`内のadapterで吸収する。0.35で実際に発生した`App::update`→`App::ui`等の変更をdomain modelや他製品crateへ波及させない
- OpenCut(MIT)はコード流用不可となった(React前提のため)。**操作仕様・レイアウトの参考のみ**に格下げ。操作動線はOpenCut、Flow/Alight Motion、一般的なトラック型UIを参照する。既知の外殻、操作トポロジー、共通component契約は[UI操作言語](../ui-interaction-language.md)を正本とする
- 外観は[UI視覚言語](../ui-visual-language.md)を正本とする。Abletonは一画面、固定されたView役割、選択→詳細、Info View、評価順と配置の一致という**操作トポロジー**と、Timeline Viewの視覚言語だけを参照し、Arrangement Viewの画面構成やDAW意味論は参照しない。Ableton/Apple風とはdark neutral、抑制した面、明確な階層、一貫したicon、意味色を指し、装飾gradient/glass/neon/card乱用を指さない
- 制作機能を大きな余白や段階的開示の奥へ隠さない。asset、preview、property、effect stack、driver、timeline、transportを高密度に一覧でき、右下/status領域へ短いcontext説明を出せる構造にする。Blenderは文脈ヘルプだけの先例で、全体UIは模倣しない
- AEのように無彩色と文字だけへ識別を寄せない。選択・種別・mute/disabled・keyframe・data mapping・bake・warning等は文字を読む前に位置/形/icon/意味色で識別でき、かつ色だけにも依存しない
- UIの色値は固定実装にしない。組み込みLight/Darkと将来のcustom themeを同じsemantic token schemaで解決し、設定画面から選択・永続化する。初回既定はDark(土台dark neutralの規約と一致)、theme異常時も診断してDarkへfallbackする
- タイムラインUIの状態管理はM2ドキュメントモデルに直結(UI独自の編集状態を二重に持たない)。編集操作は全てM2コマンドを発行する形
- キーフレーム編集UIは**AE式の値グラフエディタを作らない**。**Flow/アライトモーション式の区間イージングエディタ**を採る: 2キーフレーム間を選択→ボタン→cubic-bezierイージングをポップアップ編集(プリセット+ハンドル)。データは`motolii-eval`の`Interp::Bezier{x1,y1,x2,y2}`(区間正規化位置に対する連続曲線=fps/解像度非依存)を編集するだけでスキーマ変更不要。オーバーシュートはyの[0,1]外で表現(y非クランプ維持)。詳細と根拠はconcept.md決定事項。シーケンス操作の参考にTheatre.jsは見てよい(AGPLのstudioコードは読まない・流用しない)。**空間モーションパス(位置の2D曲線)は時間イージングとは別概念**で、v1コアには入れない(プラグイン領域/v1後半)
- パネルレイアウトは利用者が分割、tab化、resize、表示/非表示、復帰を選べる。`egui_tiles`をruntime投影先の第一候補とするが、その`Tree`/`TileId`/serde形を正本へせず、Motolii所有の安定layout modelから投影する。panelの初期配置は組み込みpresetであり固定契約ではない

## デバイスとスレッドの規約(第2回レビュー#1/#2を受けた確定事項)

1. **デバイスはコアが作り、egui shellは借りる**: `GpuCtx::new_for_ui()`がコンポジタ要件
   (`motolii_gpu::required_features()`/`check_minimum_limits()` — 単一の情報源。limitは最低ライン4096を検証した上でアダプタ実力値を要求する)を明示してデバイスを生成し、
   `egui_wgpu::WgpuSetup::Existing`でshellに渡す。逆(shellが作ったデバイスをコアが借りる)は通常経路にしない
   — feature/limitは生成時に確定し後から足せないため、M3統合直前に「必要featureが無い」で
   詰む。要件を増やす時は`required_features()`/`check_minimum_limits()`を更新する
2. **UIスレッドはMotolii frameをレンダしない**: `render_frame()`はレンダ専用スレッドで実行する(egui自身のUI paintはevent-loop threadで行ってよい)。render requestは
   blockしない最新値置換mailbox(Tokio `watch`相当の意味。依存採用は未決)で渡し、各request/resultに
   単調増加generationを付ける。UIは最新要求より古い結果を表示せず、event-loop threadで最新display
   poolの`TextureId`だけを投影する。wgpuのDevice/Queue/TextureはSend+Syncだが、eguiのUI状態をworkerから
   直接更新しない
3. **共有デバイスでの同期読み戻し禁止**: `download_rgba`(`device.poll(Wait)`)はUIと共有中の
   デバイスではUIごと止める。プレビュー中の読み戻しは行わない。**書き出しは別のヘッドレス
   デバイス(`new_headless()`)で実行する**(プレビューと書き出しの分離はB-4の設計と整合)
4. **GPU健全性監視とコールバック所有者**: レンダ専用スレッドの毎フレーム入口
   (`render_graph_cached`等)で`GpuCtx::check_health()`を呼び、device lost / uncaptured error
   を型付き`GpuRuntimeError`として検出する。wgpuの`set_device_lost_callback` /
   `on_uncaptured_error`は**デバイスあたり1スロット**のため、**コア(`GpuCtx::new_for_ui` /
   `from_device_queue`)が唯一の登録者**とし、egui/eframe側での別登録は禁止(後登録は黙って置換する)

## Stage / Output Frame / 統一Camera

正本は[統一カメラ設計](../reviews/2026-07-14-unified-stage-camera-design.md)。M3は完成画像だけを表示するpreview panelではなく、固定サイズを持たないStage上で同じworld/cameraを編集する。

- 全CompositionにM2-D1jの`CompCamera`が常在する。通常UIで「3D cameraを追加する」操作は作らない
- `Output Frame`は`CompCamera`のprojection aperture。frameの移動・ズーム・回転はDocument cameraをD2 commandで編集し、書き出しへ影響する
- `Stage View`のpan/zoom/`Fit Output / Selection / All`はworkspace/session候補で、Document serializeと書き出しへ影響しない。別preview cameraとしてdomainへ出さない
- 2D objectも`z=0`の同じworld objectで、Output Frame外でもbounds、anchor、選択、hit-test、snapを維持する
- 枠外は不透明グレーで隠さず、同じ時刻・camera・world評価の保守的Draftへ半透明scrimを重ねる。RoD/RoI最適化はM4-K0で後付けし、U1fの見た目をK0待ちにしない。Final出力範囲を広げず、GPU同期readbackでvisible boundsを求めない
- Camera toolとHand/Stage View toolはicon、frame形状、操作結果で識別でき、色だけ/labelだけへ依存しない
- UIでは平面配置の`Position X/Y`と前後配置の`Depth Z`を独立した操作groupへ投影する。これは同じ正準XYZの`position.z`を編集するUI上の意味分離であり、Depth専用field、第二の所有者、暗黙の3D modeを追加しない。`Depth Z`の平行移動と`Rotation Z`（Z軸まわりの回転）も別control・別automation channelとして識別可能にする
- Depth Railは現在時刻の評価へ追従し、明示的に開いた時だけ直接操作できる。automation中のdragは現在時刻のZ keyを更新または追加し、静的Depthのdragだけではautomationを開始しない。Cameraはworld上の文脈marker、Particle系はEmitter／生成元をmarkerとして扱い、camera-space depthやParticle個体群を第二の編集正本にしない。詳細と負例は[時間面UI構成モデル](../ui-score-model.md)を正本とする
- Timeline barの選択は開いているDepth Railの同じstable IDへfocusするが、通常bar clickだけではRailを自動展開しない。bar内または時間面headerの明示Depth iconからRailを開いて対象へfocusできるようにする。同一Zは件数付きstackとして表示し、hover／選択だけの扇状展開や表示衝突回避のための自動Z変更を行わない
- 同じparentの選択Objectを指定した奥端・手前端へauthoring orderで等間隔配置する`Layer Order Distribute`を、context menuではなくDepth Railの常設iconから使えるようにする。Groupは親側の1 markerと`Edit Children`中のparent-local子scopeを混在させず、mixed-parent選択は変更ゼロで拒否理由を示す

## プレビュー出力の寿命(`RenderedFrame` / G-1)

`render_graph_cached`（およびそれを呼ぶ `render_frame` / `render_graph`）が返す `RenderedFrame` について:

1. **セッション中間プールとの分離**: `RenderSession` の ping-pong 中間バッファは次フレームで再利用される。
   `RenderedFrame.texture` は中間プールのエイリアスを返してはならず、**専用の出力コピー**を返す（現実装: `create_owned_output_texture` + GPU copy）。
2. **呼び出し側の保持**: 同一 `RenderSession` で連続 `render_graph_cached` しても、**直前に返した `RenderedFrame` のピクセル内容は上書きされない**（出力コピー契約）。
3. **2026-07-18 egui候補でのU1義務**: レンダスレッドからUIへdisplay textureを公開する前に、**表示用の独立コピー**を確保する（TextureIdを渡すだけでは不十分 — UIがsample中に次フレームが同じ面を触るtearingを防ぐ）。display slotの安定viewはslot生成時に作り、rendererを得られる`eframe::CreationContext`で一度だけnative texture登録する。frame更新、resize、DPI変更、minimize/restoreごとに登録し直さない。G0-9の別候補もCPU pixel bridgeなし、tearingなし、generation破棄、色/alpha一致を同じfixtureで満たす。GR-1 refcountプール前倒し時はこの節を更新する。

製品プレビュー経路の正規入口は `render_graph_cached`（監査 G-3）。

## プラグインパネルの拡張方式(2026-07-22 G0-3 / GAP-13再評価中)

2026-07-12の縮小判断では、v1公開境界を`NodeDesc`からの自動生成パネルだけとした。2026-07-21にHost/コミュニティでcomponent/test語彙を再利用する長期原則を採ったが、2026-07-22の[軸分離決定](../reviews/2026-07-22-m3-surface-extension-axis-separation.md)で標準製品surfaceのG0-9とplugin UI公開判断のG0-3 / GAP-13を分離した。G0-3完了までは従来どおりプラグインUIコードをロードせず、新しい公開契約も実装しない。G0-9完了だけでは解除しない。

**不変条件(将来拡張しても崩さない)**: 標準(自動生成)パネルだけで全パラメータを操作できること。カスタムUI固有にしか存在しない必須操作は禁止。

### G0-9完了まで公開しないもの

| 候補 | 扱い | 理由(要約) |
|---|---|---|
| plugin所有のegui/native UI code | 非公開。v1.xへ自動繰越ししない | Rust/native codeのABI、event、theme、a11y、resource ownershipを第三者契約にし、Host共通componentを迂回するため |
| wgpu自由描画(スコープ・カーブ等) | 延期(同上) | wgpu/WGSLは安全なRust APIでもGPU資源のDoS隔離にはならない。AE Effect UI級の描画・イベント契約を背負う |
| 宣言レイアウト(Blender UILayout型) / ギズモ | 延期(同上) | ホスト所有部品でのレイアウトは有力だが、`NodeDesc`とは別の公開語彙。v1へ急がない |

旧案の「3段構え」(自動生成 / toolkit所有UI / wgpu描画)をそのまま復活させない。自由UI候補は公開kit、sandbox、権限、互換、fallbackをG0-3 / GAP-13で満たす場合だけ仕様改訂へ進める。G0-9の製品surface合格を代用しない。

## プラグインパネルの拡張方式(v1)

GAP-13の2026-07-16縮小判断は安全なfallbackとして残すが、最終的な公開UI runtimeはG0-9で再評価中である。

1. **自動生成パネル(必須fallback・既定体験の大半)**: エフェクトプラグインはパラメータ定義(`NodeDesc`)を宣言するだけで、汎用プロパティパネル(Rustモデル駆動の行リスト: スライダー/カラーピッカー等)が自動生成される。**全保存パラメータはこのパネルだけで編集可能**でなければならない。カスタムUIは操作可能性を追加せず、速度・可視化・専用体験だけを改善する
2. **宣言語彙は型ごとに解凍する**: Host所有のgizmo、curve、gradient、visualization等は、能力不足が実例で確認され、座標/保存/Undo/a11yを宣言できる場合だけ追加する。`ParamDef`/`ValueType`の互換と意味論をM3だけで発明しない
3. **比較前に自由UIを公開しない**: plugin所有のegui/native UI code、wgpu UI texture、任意Web codeは実装しない。Hostと同じkitを使う案も、sandbox・権限・互換・配布の比較を通してから公開範囲を決める

## 編集時Generator hook(one-shot)

上位の製品境界、Shape/SVGの分界、p5.js型表現の翻訳、Materialize/Live/Bakeの責任分担は[ジェネラティブユーザー境界](../generative-user-boundary.md)を正本とする。本節はそのうちMaterialize経路だけをM3の実装契約へ落とす。

JS/p5.jsをDocument・評価器・レンダ契約へ直接入れず、まず**編集時Generatorが型付きD2コマンドbatchを返す汎用hook**を置く。ホストは開始時snapshotに対してbatch全体をpreflightし、単一writerへ1 macroとしてcommitする。成功時は通常のGroup/Clip/VectorRecipeだけが残り、失敗・cancel・制限超過時はDocumentとUndo履歴を一切変えない。Generatorへ`&mut Document`は渡さない。

最初のadapterは**Motolii ShapeScript**とする。Paper.jsの`Project/Layer/Item/Path/Group`型object modelを設計参照にしつつ互換を名乗らず、座標はMotolii正準空間(原点中央・Y-up・高さ=1.0)、shape配置は中心基準、回転保存はradianに固定する。曖昧な位置引数を避け、`center`/`size`等のnamed fieldを使う。命令を通常のvector layer群へ変換し、1実行=1 Group=1 Undoとする。生成物は実行後に通常の編集UIで変更でき、保存・再読込・preview・exportにscript engineを必要としない。script source、runtime名、実行event stream、生成元provenanceはv1 Documentの必須意味にしない。

LLM向けの第2入口として**SVG materialize adapter**を分離する。SVGの公開語彙を入力に利用するが、左上原点・Y-down・viewport単位は入口で正準座標へ変換し、SVG DOM/XMLをDocumentの実行意味にせず通常のGroup/VectorRecipeへ実体化する。

これは**編集操作の量産口**であり、毎frame評価するlive JS layer、AE式expression、WASM Param Pipeline、plugin custom UIとは別境界である。p5.jsで一般的な「canvasをclearせず前frameの画素へ追描きする」表現も、scriptが隠しcanvasを所有する形では模倣しない。ただし表現自体は捨てず、有限loopを事前記録して通常shapeの出現時刻へ畳める場合はone-shot materialize、前出力そのものが必要な場合は[F-11 Feedback](../plugin-resources.md#6-時間参照-lookbehind--フィードバックf-11口の予約のみ)の明示的なホスト所有状態+チェックポイントBakeへ送る。JS engine/sandbox実装の選定はU9aの公開契約へ焼かない。

## タスク分割

この表のIDは能力単位であり、そのまま1 PRにしない。U0b〜U4aの実装は、[UIコンセプトから実装チケットへの分解](../reviews/2026-07-16-m3-ui-concept-to-tickets.md)の枝番を **1 Issue = 1 commit** で発注する。親IDの完了は必要な枝番がすべてmainへ到達した時だけ記録する。

**入場**: [M2基盤再締結ゲート](../reviews/2026-07-15-m2-foundation-reclosure-gate.md)はmainで解除済み。**U0a(egui骨格+依存方向CI)は本入場で完了**。下表はU0b〜U9の論理依存を示すが、初回Uシリーズの発注許可は本書末尾の直列運用で現在選択された1枝番だけに与える。#180/#191≠入場完了の歴史注記は維持する。

| ID | 内容 | 依存 | 完了条件(概要) |
|---|---|---|---|
| U0a | `motolii-ui`クレート骨格+UI toolkit依存方向CI | G0-1 | **完了**: `motolii-ui`以外の製品クレートのegui/eframe/egui-winit/egui-wgpu/egui_tiles直接依存を、Cargo metadataの直接依存検査（`package = "…"` renameを含む解決済みpackage名、`motolii_testkit::ui_toolkit_dep_policy`）が拒否。domain公開型へtoolkit型が無い。egui骨格へ置換済み |
| U0b | UI状態所有表+toolkit非依存domain intent | G0-2, M2-D2 | 代表状態をDocument/User settings/Workspace profile/Project session/Transientの5層へ分類。toolkit型なしintentの単体テスト。新しい所有寿命や恒久workspace/session形式はこのタスクで発明しない |
| U0c | input router+安定`CommandId`+event種別 | U0b, G0-2 | press/release/click/dragを区別し、IME preedit中のshortcut抑止を自動試験。shortcutを持つ全登録commandに安定IDがあり、機能crateの[raw key/modifier分岐](../reviews/2026-07-16-m3-preflight-decisions.md#23-keymap保存)を拒否。物理入力からdomain intentまでtoolkit型なし |
| U0d | **完了: 全shortcutを変更できる**keymap base+user delta JSON永続化 | U0c | builtin base不変。全bindingの追加/置換/複数割当/無効化、[documented JSON](../reviews/2026-07-20-m3-keymap-codec-contract.md) read/validate/write、roundtrip。初版は出荷済み旧版が無いためcurrent v1恒等migration枠+移行前原本面の冪等を固定し、実在しないv0変換を作らない。未知`CommandId`を保持。U0d-3は製品builtin baseを発明せず、合成baseの無効化→別bindingを`InputRouter`へ通して同じintentを発行する全command conformance。競合/OS予約は型付き診断 |
| U0e | DTCG theme token generator+component state+icon体系 | U0a。具体値と製品導入はG0-6H | token JSONからRust/egui adapterへ決定的生成し手編集を拒否。5 reference screenの人間審判後にだけ具体値を製品へ入れ、text/non-text/focus contrast、gradient許可list、component state、icon grid、motion数値検査を通す |
| U0f | **resource policy User settings model+永続化**: VRAM/RAM/disk予算preset/custom、preview解像度auto/fixed、縮退許可をDocument外で保持 | G0-2, G0-8, U0b, M4-K1a | (1)version付きroundtrip/migration冪等 (2)未知field原本保全 (3)設定変更でDocument serialize/journal/Undo不変 (4)custom hard capをK1aへ反映 (5)共有メモリ合算capの矛盾を型付き拒否 (6)egui/backend型を保存形式へ入れない |
| U1a | eguiアプリシェル+既存device共有+静止preview | U0a, U0b, G0-1, M2-D3 | [U1a-1契約](../reviews/2026-07-21-m3-u1a-1-static-viewport-contract.md)どおり同じfixture Documentの独立display slotを中央Stageへ投影し、[U1a-2契約](../reviews/2026-07-21-m3-u1a-2-layout-projection-contract.md)どおりprivate・非serdeの固定5 role layout intentから決定的runtimeを作る。runtime編集はproposalとして全体検証し、Document不変。保存codecはU1a-3、自由dock/別window実機受け入れはU1e |
| U1b | **完了**: render worker+最新値mailbox+generation破棄 | U1a | 100連続seekで送信がblockせず、取得済みresultの配送順を反転しても最新generationだけ表示。共有deviceで同期readbackなし。ownerはevent loop外でjoinし、既存display slotの登録は1回 |
| U1c | 起動/idle memory/input latency計測+開発HUD | U1b, G0-4 | 測定環境とraw結果を保存し、drop/latency/generationをHUD表示。閾値の採択は独立仕様改訂 |
| U1d | 日本語IME深部受け入れ | U1a, U0c | preedit下線、候補位置、変換中Enter非奪取、長文歌詞を対象OS実機で記録。失敗時は入力経路の仕様改訂へ戻る |
| U1e | 別window/別monitor preview spike | U1b | 同一Textureを別surfaceへ表示し、scale/monitor移動でDocument・評価結果不変。成立性と制約を記録 |
| U1f | [#169](https://github.com/oshikaidesu/Motolii/issues/169) Stage View+Output Frame+off-frame透過Draft | U1b, U0e, M2-D1k, M2-D3 | (1)同じcamera/worldからframe内+枠外を保守的に表示 (2)Stage View pan/zoom/fitでDocument serializeとFinal frame pixel不変 (3)Output Frame外を不透明塗潰しせず半透明scrim+形で識別 (4)frame外objectが無言で消えず選択可能 (5)UI thread readbackなし (6)overscan負荷をG0-4手順で測定。最適化なしでも成立し、K0導入後に見た目不変 |
| U1g | **PreviewDeadlineController**: audio/Transport主クロックを保ち、render deadline超過時は古い要求を捨てて最新時刻だけを表示。容量pressureとは別入力 | U1b, U1c, U5, M4-K1d | (1)遅延注入でproject fps/time/audio clock不変 (2)表示frameだけ15/10fps相当へ落ち追いつく (3)旧generationを表示しない (4)容量pressureだけではframe dropしない (5)固定解像度でscale不変 (6)Finalは全frameを評価 (7)UI thread/blocking sendなし |
| U1h | **Performance/Memory settings+pressure HUD**: preset/custom予算、auto/fixed解像度、現在scale・実表示fps・予算使用量・縮退理由を既存settings/HUD語彙で表示 | U0e, U0f, U1g | (1)自動縮退の理由をwarning icon+文言で表示し色だけに依存しない (2)固定設定違反なし (3)HUD非表示でも制御同一 (4)設定操作でDocument/Undo/Final不変 (5)100連続telemetry更新でUI送信非blocking (6)theme外raw color/独自spacingなし |
| U1i | **Activity projection**: background処理を既存status/diagnostic領域へ投影 | U0e, U2c, 各providerの型付きsnapshot | providerを正本にqueued/running/completed/failed/cancelled、進捗不明、cancel可否、typed reason、次の一手をBrief/Context/Inspectへ投影。UI所有queueを作らず、cancel不能処理をcancel可能に見せず、閉じても処理結果不変。export/import-proxy/bake/cache coverageはprovider完成分だけ接続し、未実装分を偽装しない |
| U2a | **完了**: [D2 one-shot atomic macroと、決定済みD2 commandを伴うDocument intent adapter](../reviews/2026-07-20-m3-u2a-1-command-adapter-contract.md) | U0b, M2-D2 | U2a-0で空列／途中失敗を全状態不変にし、U2a-1はrequestごとにatomic macroを1回使う。代表gestureがUndo 1回、異request/異targetはmergeせず、初回適用前Cancelは変更ゼロ。公開gesture lifecycle、適用後Cancel、target解決は個別の型宣言前に実装しない |
| U2b | **U2b-1完了**: normalized UI event→prepared command→private single writer→`Arc<Document>`購読E2E | U1a, U2a | callbackはrequestをqueueするだけ。Apply/Undo/Redo成功時だけ新snapshotをUI/render workerへ渡し、実windowで最終generationを表示。失敗とUI状態変更ではDocument/history不変。selection/targetとUndo/Redo製品commandは後続 |
| U2c | Direct/Tool/Advanced + 共通component/診断conformance harness | G0-7, U2b。枝番の追加依存は[Uシリーズ枝番表](../reviews/2026-07-16-m3-ui-concept-to-tickets.md#34-編集境界と共通操作文法) | 代表操作を存在する複数入口から実行し、同じDocument意味/Undo 1回/Cancel変更ゼロ。U2c-2はU4a-2 DirectとU4c Advancedの実製品入口完成後に実行し、未実装Toolを偽装しない。領域固有rejectionをTransient Diagnostic Envelopeへ適応し、Brief/Context/Inspectで同じreason/subject/factsを保持する。UIイベント列や診断をserializeしない。gray/dimだけのsilent disabled、外部検索必須、Document objectのUI文言依存、巨大error enum、診断componentの直接mutation、同じIntentの局所picker/popup、独自hover/focus/Cancel/error投影をfixtureで拒否する |
| U2d | Camera/Output Frame直接操作+枠外object選択 | U1f, U2c | Camera toolはM2-D1j cameraだけをD2 command化し1 gesture=1 Undo。Hand/Fitはworkspaceだけを変更。frame外objectを選択・移動・snapでき、camera/object操作を混同しない。DPI差で同じ正規化gestureが同じdomain値 |
| U2e | LookAt/Follow/Parent向け**説明付き共通Connection Target Picker** | U2c, U2d, M2-D3 | (1)button/whip/Canvas・Timeline clickを同じConnection Intentへ正規化 (2)選択mode中はカーソル近傍へ「何を・何へ・どう繋ぐか」を常時文表示 (3)期待型、valid強調、invalid dim+理由、hover仮線/from-to手掛かり、確定後semantic badgeを表示し色/iconだけへ依存しない (4)`Idle/Picking/HoverValid/HoverInvalid/Commit/Cancel`はTransientでDocument非保存 (5)`LayerId`だけをD2 command化し、表示名変更で参照不変、自己参照/循環/削除済みtargetを型付き拒否、Cancel変更ゼロ、1選択=1 Undo (6)layer名/property path文字列、pick-whip式、隠れhelper、`force connect`を作らない (7)Advancedは同じ参照の由来/評価順/失敗を検査し、型検査を外す別意味にしない。具体的な高度例外は[操作単純化モデル S-3a](../interaction-simplicity-model.md#s-3a-接続操作はカーソル自身が意味を説明する)の境界を通す |
| U2f | [#168](https://github.com/oshikaidesu/Motolii/issues/168) **modifier+drag one-shot Relative Move**: 安定gesture intentをkeymapから呼び、Position Const/全keyへ同じEdit-Space差分を適用するD2 macro | U0c, U0d, U2a, U2c, M2-D2 | (1)通常drag=現在値、modifier+drag=軌跡全体をHUD/ghostで識別 (2)pointer-upまでtransient、Undo 1回、Escape/capture loss変更ゼロ (3)混合型/削除済み/編集不可を開始前に型付き拒否し部分適用なし (4)時刻/補間/接線不変、既存値だけが変わりhelper/offset/Modifier/expressionを生成しない (5)DataTrack/FollowをBakeしない (6)専用Tool/panelなし (7)物理modifierはkeymapで変更可能 |
| U2g | **Timeline Effect Link**: Effect Definition `out`→各Layer effect-stack Use `in`の常時表示connection gutter | U0e, U2b, U3a, M2-D1l, M2-D3e | (1)非選択時も全接続線をgutter内に表示 (2)from/inをsocket形状+arrowheadで識別し色だけに依存しない (3)折畳み先はstub+件数badgeで接続存在を隠さない (4)drag中は型不一致をdimしstack挿入位置を表示 (5)1 drag=1 Use=Undo 1回、Cancel変更ゼロ (6)timeline順/renameで参照不変 (7)線はclip/key領域を横断しない (8)Group=合成後1回、Explicit=各layer個別適用をUIで混同しない (9)500 use fixtureのrouting/hit-testがUI threadをblockせず、全線またはbundle stubが常時存在 |
| U2h | **Selection/focus+essential command surface** | U0c, U2a, U2b, U2c, U3a | single/additive/range/marqueeを同じTransient selectionへ正規化し、focusはhoverでなく明示click/keyboard移動で変える。Stage/Timeline/Inspectorが同じ安定IDを指し、filter/折畳みで隠れた選択は件数+戻る操作を表示。Delete/Duplicate/Rename/Copy/Pasteは登録済み`CommandId`+preflightを通り、対応D2操作が無い対象はtyped reasonを表示。clipboard payload、cross-document再写像、Shared Effect複製意味をUIで発明しない |
| U3a | toolkit非依存timeline layout/hit-test+dense surface | U0a, U0b, G0-9 | clips 1,000+keys 100,000の固定fixture/viewport/操作列ベンチ。遠景density・中景cluster・近景individualを同じtime range/stable ID投影から作り、zoom境界前後でselection/playhead/visible range不変。Canvas/browser WebGPUの結果は先例baseline、製品枝は[native renderer再選定](../reviews/2026-07-21-native-surface-renderer-reselection.md)どおりdirect wgpu primitive batchとVello path/text局所passを同じwindowed fixtureで比較する。Reactは外側のtoolbar/menu/parameter編集を所有し、density pixelやDOM identityをDocument identityへ使わない。CIは基準比、実画面60fps閾値はG0-4後 |
| U3b | timeline配置/移動/trim操作 | U0c, U2b, U3a | drag操作がD2 commandを発行。ランダム操作列で重複なし・相対位置維持・Undo全巻戻し |
| U3c | 波形表示用derived cache+timeline描画 | U3a, M2-D4 | cacheの持ち場・無効化・上限を仕様化してから実装。波形データをDocumentへ焼かず、seek/zoom fixtureで一致 |
| U3d | timeline視覚統合+認知reference screen | U0e, U3a | 固定fixtureのgolden/lightness差分。5秒識別、grayscale、既存componentとの馴染み、Timeline Viewとの同条件比較を記録 |
| U3e | **Timeline navigation/search/filter** | U0c, U2h, U3a | Fit All/Selection、Go to Playhead、前後のclip/key/snap point、名前/型/animated/error/hidden filterを共通`CommandId`で操作。結果から同じTimeline/Inspectorへ戻り、Document/Undo不変。1000 clip+100000 key fixtureで検索・移動・filterがUI threadをblockせず、filtered selectionを無言で失わない。表示名検索を参照identityへ使わない |
| U3f | **Time-local readiness projection** | U0e, U3a, U1h, M4-K1b/K7/K8 | Timelineへready/rendering/stale/unavailableとDraft/Final-equivalentの由来を既存overlay/diagnostic語彙で投影し、色だけに依存しない。provider snapshotと表示区間が一致し、1000区間更新がnonblocking。表示はcache/bake policy、Document、Final結果を変えず、未取得をreadyとして見せない |
| U4a | `NodeDesc`自動parameter panel | U2b, U0e, U1b | `ValueType → widget → command`対応表。全登録pluginの全保存paramが自動panelから編集可能なconformance。100回連続slider更新でUI送信がblockせず、操作中は最新generationだけをpreview表示し、確定値が一致し、gesture全体がUndo 1回 |
| U4b | keyframe編集+区間Easing Graph View | U4a | [native Easing popup受入契約](../reviews/2026-07-22-m3-native-easing-popup-acceptance.md)に従い、ReactはGraph icon/現在値要約、native wgpu popupはframe/preset/user library/form/curve/grid/handle/drag preview、Hostはanchor/z-order/focus/dismiss/DPIとUser settings codecを所有する。Preview直下のGraph iconはplayheadが隣接key間にある時だけ有効（key上・区間外は無効）。key clickでは開かない。single click=Graph View、double click=単一◎markのお気に入りcurveを現在区間へ即適用し1 Undo（popupを残さない）。お気に入り・user preset変更はUser settingでDocument/Undo不変、最終使用へ自動追従しない。preset thumbnailは保存curveからnativeで再生成し画像を正本化しない。drag中semantic write 0、release 1 Undo、Esc/focus loss変更ゼロ。現在区間の補間切替/cubic-bezier 4値+presetが反映し、既存区間の非対象curve不変プロパティテスト |
| U4c | Advanced意味検査+round-trip | U2c-1, U2c-3, U2c-4, U2c-5, U4a, M2-D1l | 現行DocParamのConst/Keyframes/Data/Vec2Axes/LookAt/Follow、plugin source/version、Effect Definition/Use ID、target、Owned/Explicit scope、policyを検査できる。Direct/Toolで作った状態を開閉してserialize不変。Simple時も非既定意味をbadge表示。未実装Param Pipeline、Composite Set、Backdrop地点をUIだけで捏造しない。U4a-2のDirect入口と本タスクのAdvanced入口が揃った後、U2c-2 conformanceへ渡す |
| U5 | scrub/再生transport UI | U0c, U0e, U1b, U3b, M2-D5 | vsync暴走注入でもTransport同期不変。最新seekのみ表示。低速時のvarispeedは数値試験+別記の聴感確認 |
| U6 | Project Explorer+素材preview内source range+Inbox受取+import/D&D配置 | U0e, U2b | Project assetと外部filesystemを別popupへ分けず、既存Browserの`Project` tab内にある同じExplorer UIで`PROJECT / FILES`を明示切替する。FILESの検索・選択・preview・In/OutはDocument外で、`Add to Inbox`は未配置参照を受け取るだけとする。PROJECTから配置確定した時だけ既存の`Clip.duration = out - in`と`TimeMap.source_start = in`へ変換し、1 Undo。同じrangeからの再配置は明示変更まで同じ結果になり、配置前のrange変更ではDocument/Undo不変。Inboxはasset所有者や全履歴にならず、未配置・未確認状態だけを参照し処理後に外す。動画/SVGを配置し楽曲1本を設定。欠落/不正asset・空/逆転/素材尺外rangeをtyped error表示し、UI threadでdecodeしない |
| U7 | beat grid+ユーザーmarker snap | U3b, GAP-16意味決定+M2実装 | 有理BPM/beat origin/meterから生成したbeat gridに加え、ユーザーが決めたtimeline markerをsnap対象にできる。clip/keyframe snapがfps非依存のRationalTimeで一致し、markerなしでは従来のbeat grid結果と同一。markerの永続型・点/範囲・identity・編集意味はGAP-16とGR-PVを通す前にM3で発明しない |
| U8a | group/clip mask UI | U3b, M2-D7 | grouping/ungrouping/clip modeがD2 command経由でUndo可能 |
| U8b | group仮出力toggle | U8a, M4-K7c | bake発動・区間無効化・再freezeがUIからE2Eで確認でき、toggle前後でDocument/Undo/serialize不変 |
| U9a | **Editor Generator command hook**: 外部generator結果を型付きD2 command batchとして受けるtoolkit/runtime非依存境界 | U2b, M2-D2 | (1)generatorへ`&mut Document`を渡さない (2)開始snapshotに対するbatch全体preflight後だけ単一writerへ1 macro commitし、commit時に現行Documentと一致しない結果はstaleとして拒否 (3)成功=Undo 1回、失敗/cancel/制限超過/stale=Document・履歴変更ゼロ (4)journal/serializeには解決済み通常編集だけが残る (5)script engine無しでsave/reload/preview/export同一 (6)domain公開型にegui/JS/runtime固有型なし |
| U9b | **Motolii ShapeScript one-shot adapter**: Paper.js型object/path/group思想を正準座標で再構成し、通常Group+vector layerへ変換 | U9a, M2-D1i-2 | (1)原点中央/Y-up/高さ1.0、center基準shape、radian、named fieldの固定表 (2)Path/Shape/Group/style/transform stack/unsupported APIの固定表 (3)同一script+明示`u64 seed`で同一command batch、時計/OS entropyなし (4)1実行=1 Group=1 Undo、生成物を通常編集可能 (5)network/filesystem/process/GPU textureへ非接続 (6)実行時間・command数・path点数・nest深度の上限超過を型付き拒否し部分生成なし (7)JS engineをDocument/renderer/plugin契約へ露出しない (8)editor buffer/script sourceの持ち場を分類し、恒久保存形式はこのタスクで発明しない (9)`draw()`/前frame画素/暗黙canvas蓄積が構文不能 |
| U9c | **SVG materialize adapter**: LLM生成SVG→通常Group/VectorRecipe | U9b | (1)SVG viewport/左上原点/Y-downを正準座標へ決定的変換 (2)採用element/style/transformと拒否表を固定 (3)DOM/XML/script/event/外部URLをDocumentへ残さず、外部参照と実行要素を型付き拒否 (4)materialize後はSVG parser/runtime無しでsave/reload/preview/export同一 (5)同じSVGから同じD2 batch、1 import=1 Undo |
| AG-3 | **v1.x追加レーン**: Video+Audio/Video Only import、audio component展開、mute/gain、音声分離macro | AG-1, AG-2, U6, U2c, U3a | 同じClipのmove/trim/retimeでA/V追従、分離前後PCM一致、Undo 1回、別project mode/別timeline schemaを作らない。現行U6のMV最短導線を置換せず追加する |

直列運用（U0a完了後）: U0aはegui骨格+依存方向CIで完了。初回Uシリーズは
ファイル競合の有無にかかわらず1チケットずつ進め、`U0b-1→U0b-2→U0c-1→U0c-2
→U0d-1→U0d-2→U0d-3→U2a-0→U2a-1→U1a-1/2→U1b-1/2→U2b-1→U2c-1→U2c-4`
の順に意味・入力・編集・shell境界を閉じる。実在する複数製品入口が無いU2c-2は
空harnessで完了させず、U0e-3依存のU2c-3/5も先行しない。次にPR #184から生成機構だけを
`U0e-1`へ抽出し、`U0e-2R`で固定React比較baselineをmainへ再結合してから
`U0e-2`のreference fixtureを作る。G0-6Hの人間審判後にだけ
`U0e-3→U2c-3→U2c-5`へ進む。その後
G0-9完了後に`U3a→U4a-*→U4c→U2c-2`を最初の製品縦切りと入口同値審判へ合流させる。論理依存が並列を許す後続も、
初回Uシリーズではこの直列運用を優先する。G0-8+K1a後のU0f、M4依存の
U1g/U1h/U3f/U8b、D5依存のU5、GAP-16依存のU7、未統一Browser P41、
U9bのengine/sandbox/保存判断等へ到達したら、仕様や外部依存を迂回せずSTOPする。

## GR-UI審判割当表

| 規律 | 対象タスク | 自動審判 | 人間実機審判 |
|---|---|---|---|
| GR-UI-1 状態所有 | U0b, U0d, U0f, U1a, U1f, U1h, U1i, U2b, U2d, U2h, U3c, U3e, U3f, U9b, U9c | 状態分類fixture、panel layout操作とStage View/preview resource設定変更時のDocument/Final不変、provider/selection/search/readiness非Document検査、keymap/resource settings roundtrip、script/SVG source非Document検査 | workspace/script復元UXは保存方針決定後 |
| GR-UI-2 command境界 | U0c, U2a, U2b, U2c, U2d, U2e, U2f, U2g, U2h, U3b, U4a, U4b, U4c, U6, U8a, U9a, U9b, U9c, AG-3 | input→intent、入口意味同値、selection/target/definition/use ID、macro/merge/Undo property test、generator batch原子性、`&mut Document`依存検査 | — |
| GR-UI-3 thread/latest | U1a, U1b, U1e, U1f, U1g, U1h, U1i, U3e, U3f, U4a, U5, U6, U9b, AG-3 | non-blocking seek/parameter/overscan/activity/search/readiness/telemetry、deadline時の最新frame、generator実行中のUI応答、generation逆順、同期readback禁止 | 長時間scrub/parameter/camera drag、pressure縮退、generator cancelの体感 |
| GR-UI-4 単位 | U1e, U1f, U2b, U2d, U4a, U4b, U7 | scale注入時domain command一致、UI degree↔Document radian、RationalTime、Stage View非永続 | 別monitor/DPI移動 |
| GR-UI-5 UI toolkit隔離 | U0a, U0b, U1a, U1f, U1i, U2c, U2h, U3a, U3e, U3f, U9a, U9b, U9c | Cargo metadata直接依存検査（rename含む、`ui_toolkit_dep_policy`）+公開型走査、panel layout/provider/selection/search/readiness model、generator/SVG hookのruntime/egui非依存test、windowなしlayout/hit-test test | — |
| GR-UI-6 performance | U1c, U1f, U1g, U2h, U3a, U3e, U3f | 固定fixture/overscan/deadline遅延注入、large selection/search/readiness更新の基準比 | 基準機p50/p95、起動、idle memory、Stage pan/zoom、preview縮退 |
| GR-UI-7 plugin fallback | U4a | 全登録plugin conformance | widget操作性 |
| GR-UI-8 視覚認知 | U0e, U1f, U1h, U1i, U2d, U2e, U2f, U2g, U2h, U3d, U3e, U3f, U4a, U4c, U5, U6 | token生成差分、Output Frame/Stage View/Camera/pressure warning/activity/selection/filtered selection/readiness/target-pick/Relative HUD/from-in接続状態、raw color、contrast、icon/state、通常+lightness/CVD reference画像 | 5秒識別、frame内外/Camera対Hand/pressure理由/activity/selection/readiness/通常drag対Relative/from対in、grayscale/CVD、既存UIとの馴染み |

表にない横断変更を行う場合は、PR前に本表へ審判を追加する。人間実機審判だけで「完了」にせず、自動審判と別の証跡として残す。

## 実装ガード(先行ツールの失敗・ユーザー不満クロスチェック 2026-07-11)

過去のSlint実運用問題、egui採用時の実機調査、出荷済みエディタのタイムライン/プレビュー苦情(FCPX/AE/Kdenlive/Shotcut/Resolve/AviUtl)からガードを抽出した。**先頭2項目は「M3後半に発覚すると設計が覆る」種類のリスクなので、U1d/U3aへ独立割当する。**

本節の「着手前」「最初」はM3入場(U0a完了)を起点とする。U0a以前は、製品コード・公開API・永続形式を変更するスパイクを含めて発注しない。

受容側の対照先例は[reviews/2026-07-16-m3-ui-rapid-acceptance-prior-art.md](../reviews/2026-07-16-m3-ui-rapid-acceptance-prior-art.md): 第一部=プロダクト単位の受容事例(Flow/AviUtl2/VOICEVOX/FCPX両面等 — 界隈の期待リスト)、第二部=**業界収斂した操作語彙の台帳**(本節ガード5の「業界標準の操作」の具体的な中身)とUX原理の一次資料、第三部=後発の勝ち筋「どの操作も直感的」(Ableton先例のAEカウンター分解 — 直感性の要件「結果が100ms以内に見え可逆」は性能の関数であり、performance-modelが前提条件)。仮説メモであり本節のガード・完了条件は変更しない(転移候補は個別M3チケット採択時に判断)。

以下は個別タスクのガードである。横断する状態所有・コマンド境界・GPU/スレッド・単位・UI toolkit隔離・公開契約の停止条件は[GR-UI](../reviews/2026-07-14-m3-ui-boundary-prevention.md)を正本とし、適用先は審判割当表で限定する。

1. **日本語IME受け入れをU1dへ分離する**: [egui採用判断](../reviews/2026-07-18-m3-egui-selection.md)でmacOSの単一行/複数行、Preedit 37件、Commit 5件、候補位置、変換中shortcut漏れ0を実機確認した。製品U1dでも (1) preedit表示 (2) 候補がcaretへ追従 (3) **変換中のEnter/Esc/Spaceがアプリshortcutへ漏れない** (4) 長文連続入力を固定する。Windows MS-IMEとLinux IMEは採用停止線にせず、各platformの最初の配布候補で同じchecklistを運用する。落ちたらshortcut special-caseで隠さず入力adapterを修復する
2. **タイムライン・波形・グラフ類を無仮想化の大量widget/DOMで組まない**: project全量でUI要素数が増える構成はCPU負荷と操作不整合を生む。layout/hit-test/time mappingをtoolkit非依存に置き、WebのCanvas/browser WebGPU結果はbaselineとして保持するが、製品の高頻度Timeline surfaceはdirect wgpuを第一候補、Velloをpath/text局所利用とする。clips 1,000+keys 100,000の同一fixtureをwindow present/input/WebView同居まで測り、**60fpsはG0-4で基準機・操作列・p50/p95を決めた後の製品目標で、hardware未指定のCI閾値にしない**
3. **再生のフレームペースをvsync/egui repaintへ依存させない**: 主クロックは音声(M2 Transport)で、eguiのrepaintは投影要求にすぎない。repaintが過剰でも停止してもフレームペースと音声同期が崩れず、idleでは連続repaintしないことをU5で固定する
4. **egui APIに触れるのは`motolii-ui`だけ(依存方向をCIで強制)**: ArdourはGTK2から移行できず自前フォーク(YTK)を生涯保守する道を選んだ。メディアアプリはカスタムwidget比率が高く、toolkit APIが全域に染みると移行コスト=全書き直しになる。タイムライン/preview描画modelはegui非依存に置き、egui event→domain intent変換adapterだけを`motolii-ui`内に置く。禁止対象は他製品crateのegui family依存とdomain公開型へのegui/eframe/winit型流出
5. **タイムラインの革新的挙動は必ずオプトインし、全shortcutを変更可能にする**: FCPXのマグネティックタイムライン強制は3,700筆超の抗議署名とプロ層の恒久流出を生んだ(「概念として優れていても、訓練されてきた全てに反する」)。既定は業界標準の操作(トラック型、スペース再生、スナップのキートグル)とするが、Space、Delete、tool、modifier+dragを含む全shortcutは初日から追加/置換/無効化可能にする。専用UIが間に合わなくてもversion付きJSONをfallbackとし、機能側のraw key/modifier判定を禁止する
6. **「キーフレームを追加しても既存区間のカーブ形状が変わらない」を不変条件に**: AEグラフエディタの「イージングを入れるとスパイク/ループが出る」「予測可能な調整がほぼ不可能」という定番苦情は、キー追加・移動時に近傍カーブが暗黙に変わることが根因。区間イージング方式(採用済み)はこの罠を大きく回避するが、この性質自体をmotolii-evalのプロパティテストとして固定する → U4b
7. **ランダム編集操作列のプロパティテスト**: Kdenliveは「保存→再起動でクリップが複製され、後続クリップがまとめてズレて音ズレ」等のモデル不整合で「不安定」の評判が定着した(単発操作でなく操作の合成で壊れる)。「ランダムな操作列(配置/移動/トリム/グループ/undo-redo)を数千回適用しても (a)クリップ重複なし (b)グループ内相対位置維持 (c)undo全巻き戻しで初期状態一致」を、M2-D2の単発プロパティテストの系列版としてU2a/U3bのCIに置く
8. **スクラブは「最新要求だけ保持」+generationで旧結果を捨てる+観測可能に**: render requestはblocking容量1 channelでなく最新値置換mailboxにする。実行中GPU workの強制cancelは要求せず、完了した旧generationをUIが表示しない。開発ビルドにdrop/latency/generation HUDを置く — 再生系苦情の大半は「間に合わない時のポリシー未定義」に還元される
9. **可変panelとプレビュー別ウィンドウをU1eでスパイク**: Resolveの「パネル取り外し不可」は10年級の不満。egui_tilesによる分割/tab/resize/hide/restoreと、プレビューの別window/別monitor fullscreenを早期確認する。layout正本をegui_tilesの生serializeにせず、別surfaceでも同じdisplay pool/device ownershipを崩さない
10. **起動時間・アイドルメモリはU1cで測ってから数値目標を採択する**: AviUtl層のユーザーは重さに敏感だが、測定前のN秒/M MBを公約しない。G0-4の基準機と手順でraw値を取り、閾値は独立仕様改訂で固定する
11. **アクセシビリティはG0-2の保証範囲を維持する**: egui/eframeのAccessKit連携を使っても、カスタム描画timelineは自動ではaccessibility treeに乗らない。やらない範囲を明記し、標準controlのlabel/focusとキーボード完結操作を保証する範囲を決めてからU0b/U3bへ配線する

12. **Param PipelineをUIから先に発明しない**: U4a/U4cは現行`DocParam`の出所を編集・検査する範囲なら進めてよい。常設Relative Offset、Generator/Modifier列、DataTrack+手補正の同時適用、評価列並べ替え、汎用parameter pluginのいずれかが必要になった時点で[PP-Gate](../interaction-simplicity-model.md#4-param-pipeline-gatepp-gate)を開始し、M1/M2解凍・migration・意味論golden・反対側レビュー前は実装を止める

    One-Knob Macro Controlは有力なv1.x候補だが、一対多の永続parameter driverでありM3のノブ部品として先行実装しない。[操作単純化モデル§4.1](../interaction-simplicity-model.md#41-v1x候補-one-knob-macro-control)とbacklog MC-0〜2へ送り、PP-Gate後に意味→評価→UIの順で追加する

13. **one-shot Generatorをlive runtimeへ拡張しない**: U9a〜U9cが許すのは、制限付きworkerでcommand batchを生成し、開始snapshotと現行Documentをcommit時に照合して全体preflight後に通常編集として1回だけcommitする経路だけ。新しいtransaction/revision公開APIをこのタスクで発明しない。script/SVG source、runtime、provenanceを必須Document意味へ追加しない。毎frame JS、expression、Param Pipeline、部分commit、暗黙の乱数/時計、未対応APIの黙示fallbackが必要になったら実装を止める。前frame画素への追描きはU9内で隠し状態化せず、F-11 Feedback+K1/K7後のSCR-4へ送る

14. **preview縮退を作品意味へしない**: U0f/U1g/U1hの予算・auto/fixed・実表示fps・pressure reasonはUser settingsまたはTransientであり、Document、journal、Undo、plugin parameter、cache keyへ入れない。frame dropはaudio/Transport時刻へ追いつくための表示省略で、project fps変更・低速再生・Final frame省略として実装しない。自動scale変更は許可設定時だけ行い、固定中は明示的なコマ落ち/拒否へ縮退する

15. **Rerun転移と接合部の後半発覚を枝番内で先に潰す**: [M3 / Rerun実装後半発覚プレモーテム](../reviews/2026-07-20-m3-rerun-late-discovery-premortem.md)に従い、U0e-2のfixture単一正本と保存形式確定、U1a-1の色/alpha/display pool寿命、U1a-2のstable panel identity、U1b-2のpool generation、U3aのsemantic zoom境界を各枝番の負例へ含める。fixture manifestへU0b-1の所有層を与えず、registration/resource計器はtest-only accessorに限る。Rerun調査全体を無関係なUタスクの一括ゲートにせず、参照する枝番だけfile/API単位のjust-in-time transfer packetを持つ

16. **React所有面を製品用に縮約再実装しない**: [直接移管契約](../reviews/2026-07-22-m3-react-product-asset-promotion-contract.md)に従い、固定commitのcomponent/CSS/stable ID/ARIA/testをproduct packageへ直接所有移管し、mockをconsumerへ反転する。sourceが無いInspector等は固定モック内で同形React化とparityを先に通す。別leaf、CSS後追い、skeleton代用、opaque ID分岐、mock/product二重copy、React semantic state、diagnostic routeによる製品画面代用、visual threshold/golden変更が発生した時点でSTOPする

出典: slint-ui/slint#1644・#4097・#8693・#2895 / rust-windowing/winit#2888 / warpdotdev/warp#9383 / variety.com(FCPX抗議署名) / creativecow.net(AEグラフエディタ苦情) / KDE Bug 369505(Kdenliveクリップ複製) / forum.blackmagicdesign.com(Resolveパネル分離) / phoronix.com(Ardour YTK) / theregister.com(Qt LTS商用化) / forum.shotcut.org(プレビューラグ)

## 未決事項

- ~~S1合格によるSlint確定~~ → 2026-07-18にeguiへ変更。[採用判断](../reviews/2026-07-18-m3-egui-selection.md)で既存device/native texture、lifecycle、日本語IMEをApple M4 / Metal実機確認。Slint S1は歴史証拠として維持する
- OpenCutからコードは取り込まない。操作仕様・レイアウトのどの観察を採るかだけをU3a着手前に棚卸しする
- Param Pipelineの具体型は未決。U4cは現行意味の可視化までで、Modifier UIの採否判断ではない
- 枠外overscanの距離別品質・bounds cache・VRAM予算の固定値はU1f着手前spikeで決める。Stage全域を無制限Final描画するdefaultは採らない
- U9bのJS engine、sandbox方式、script保存場所は未決。p5.js互換はv1要件にせず、有限one-shot命令のsyntax sugarが必要ならShapeScript完成後に別判断する
