# M3: UI統合

ステータス: **ドラフト**(INF-1でSlint採用確定済み。下記G0-2〜G0-4/G0-6〜G0-7と各タスクのM2依存完了後に確定)

> **着手前規約**: [M3 UI境界汚染の予防](../reviews/2026-07-14-m3-ui-boundary-prevention.md)のうち、後掲「GR-UI審判割当表」で対象タスクへ割り当てた項目を先に通す。外観を伴うタスクは[UI視覚言語](../ui-visual-language.md)も適用する。非該当項目を形式的にYesにしない。Documentスキーマへ触る場合は[M2恒久焼き込みの予防](../reviews/2026-07-12-m2-permanence-prevention.md)も同時適用する。

## 目的(退治する落とし穴)

A-1(→**Slint転換により構造的に解消見込み**。S1スパイクで最終確認)、D-3(OpenCut流用の期待値管理)。

## M3仕様確定ゲート(G0)

| ID | 内容 | 状態 | 確定条件 |
|---|---|---|---|
| G0-1 | Slint採否とManual wgpu共有 | **完了**(INF-1) | 実機証拠とS1方式が仕様へ反映済み |
| G0-2 | 入力/キーマップ/アクセシビリティ最小意味論(GAP-6) | 未決 | 安定`CommandId`、press/release/click/drag、不変base+user delta、設定version/原本保全migrationを仕様化。panel/zoom等workspace状態の保存寿命と、v1 accessibility保証/非保証を別表で決定 |
| G0-3 | plugin UIモデル(GAP-13) | 未決 | 能力コーパスと判定語付き採否。決着までは自動生成fallback以外の公開契約を実装しない |
| G0-4 | 性能測定プロトコル | 未決 | 基準機、viewport、操作列、warm-up、測定時間、p50/p95、CI基準比を記録。測定前に60fpsをCI閾値へしない |
| G0-5 | UI境界規約の反対側レビュー | **完了** | [R1〜R9](../reviews/2026-07-14-m3-ui-boundary-counter-review.md)を予防文書・本仕様へ反映 |
| G0-6 | 視覚言語tokenと認知審判 | 未決 | [UI視覚言語](../ui-visual-language.md)の意味role、contrast、icon/component状態表を具体tokenへ固定。Stageを含む4 reference screenを同一fixtureで比較し、人間審判を記録 |
| G0-7 | 操作単純化ゲート | 未決 | [操作単純化モデル](../interaction-simplicity-model.md)の代表操作ごとにDomain Intent、永続物、Undo、失敗、存在するDirect/Tool/Advanced入口、Simple表示で残すsemantic badgeを固定。入口違いのserialize意味同値審判を決める |

G0はM3全コードを一括停止する門ではない。各タスクは依存するG0/M2項目だけを満たせば着手できる。例: U1aはG0-2/G0-3を待たずに進められる。custom UI追加タスクはG0-3の判定後に初めて起票する。

## 方針(2026-07-08 改訂: UI基盤をSlintに決定)

- **UI基盤はSlint**(1.17+、`unstable-wgpu-29` feature)。WebView/Tauri案は廃止。理由: 公式wgpu統合により本体レンダラと同一デバイスでテクスチャをUIに直接埋め込め(A-1のブリッジ問題が消滅)、かつ日本語IMEの動作実績がある(将来の日本語UI方針)
- プレビューは`slint::Image::try_from(wgpu::Texture)`で埋め込み。デバイス共有は`BackendSelector::require_wgpu_29(WGPUConfiguration)` + `set_rendering_notifier`で取得(実装例: `spikes/s1-slint/`)
- **wgpuバージョンはSlintの対応版に合わせて固定**(現在29系。本体workspaceを30→29に下げてUI統合する。Slintのwgpu対応更新に追従してまとめて上げる)
- **レンダラ選択はfeatureで固定**: `require_wgpu_29()` を使う場合、`slint` のデフォルト機能(OpenGL系 `renderer-femtovg`)を混在させない。`default-features = false` + `backend-winit` + `renderer-femtovg-wgpu` + `unstable-wgpu-29` を基本とする(2026-07-08 S1実測で、OpenGLレンダラ混入時に `WGPU 29.x rendering is not supported with an OpenGL renderer` を確認)
- OpenCut(MIT)はコード流用不可となった(React前提のため)。**操作仕様・レイアウトの参考のみ**に格下げ。操作動線はOpenCut、Flow/Alight Motion、一般的なトラック型UIを参照する
- 外観は[UI視覚言語](../ui-visual-language.md)を正本とする。Abletonは**Timeline Viewの視覚言語だけ**を参照し、Arrangement Viewの画面構成やDAW操作モデルは参照しない。Ableton/Apple風とはdark neutral、抑制した面、明確な階層、一貫したicon、意味色を指し、装飾gradient/glass/neon/card乱用を指さない
- AEのように無彩色と文字へ識別を寄せない。選択・種別・mute/disabled・keyframe・warning等は文字を読む前に位置/形/icon/意味色で識別でき、かつ色だけに依存しない
- タイムラインUIの状態管理はM2ドキュメントモデルに直結(UI独自の編集状態を二重に持たない)。編集操作は全てM2コマンドを発行する形
- キーフレーム編集UIは**AE式の値グラフエディタを作らない**。**Flow/アライトモーション式の区間イージングエディタ**を採る: 2キーフレーム間を選択→ボタン→cubic-bezierイージングをポップアップ編集(プリセット+ハンドル)。データは`motolii-eval`の`Interp::Bezier{x1,y1,x2,y2}`(区間正規化位置に対する連続曲線=fps/解像度非依存)を編集するだけでスキーマ変更不要。オーバーシュートはyの[0,1]外で表現(y非クランプ維持)。詳細と根拠はconcept.md決定事項。シーケンス操作の参考にTheatre.jsは見てよい(AGPLのstudioコードは読まない・流用しない)。**空間モーションパス(位置の2D曲線)は時間イージングとは別概念**で、v1コアには入れない(プラグイン領域/v1後半)
- パネルレイアウトはまず固定分割で作る(Slintに既製ドッキング機構はないため、可変ドッキングはv1後半以降)

## デバイスとスレッドの規約(第2回レビュー#1/#2を受けた確定事項)

1. **デバイスはコアが作り、Slintは借りる**: `GpuCtx::new_for_ui()`がコンポジタ要件
   (`motolii_gpu::required_features()`/`check_minimum_limits()` — 単一の情報源。limitは最低ライン4096を検証した上でアダプタ実力値を要求する)を明示してデバイスを生成し、
   `WGPUConfiguration::Manual`でSlintに渡す。逆(Slintが作ったデバイスをコアが借りる)は禁止
   — feature/limitは生成時に確定し後から足せないため、M3統合直前に「必要featureが無い」で
   詰む。要件を増やす時は`required_features()`/`check_minimum_limits()`を更新する
2. **UIスレッドはMotolii frameをレンダしない**: `render_frame()`はレンダ専用スレッドで実行する(Slint自身のUI paintはevent-loop threadで行ってよい)。render requestは
   blockしない最新値置換mailbox(Tokio `watch`相当の意味。依存採用は未決)で渡し、各request/resultに
   単調増加generationを付ける。UIは最新要求より古い結果を表示せず、Slint component更新だけを
   event-loop threadへ戻して`Image::try_from`で表示する。wgpuのDevice/Queue/TextureはSend+Syncだが、
   Slint componentをworkerから直接更新しない
3. **共有デバイスでの同期読み戻し禁止**: `download_rgba`(`device.poll(Wait)`)はUIと共有中の
   デバイスではUIごと止める。プレビュー中の読み戻しは行わない。**書き出しは別のヘッドレス
   デバイス(`new_headless()`)で実行する**(プレビューと書き出しの分離はB-4の設計と整合)

## Stage / Output Frame / 統一Camera

正本は[統一カメラ設計](../reviews/2026-07-14-unified-stage-camera-design.md)。M3は完成画像だけを表示するpreview panelではなく、固定サイズを持たないStage上で同じworld/cameraを編集する。

- 全CompositionにM2-D1jの`CompCamera`が常在する。通常UIで「3D cameraを追加する」操作は作らない
- `Output Frame`は`CompCamera`のprojection aperture。frameの移動・ズーム・回転はDocument cameraをD2 commandで編集し、書き出しへ影響する
- `Stage View`のpan/zoom/`Fit Output / Selection / All`はworkspace/session候補で、Document serializeと書き出しへ影響しない。別preview cameraとしてdomainへ出さない
- 2D objectも`z=0`の同じworld objectで、Output Frame外でもbounds、anchor、選択、hit-test、snapを維持する
- 枠外は不透明グレーで隠さず、同じ時刻・camera・world評価の保守的Draftへ半透明scrimを重ねる。RoD/RoI最適化はM4-K0で後付けし、U1fの見た目をK0待ちにしない。Final出力範囲を広げず、GPU同期readbackでvisible boundsを求めない
- Camera toolとHand/Stage View toolはicon、frame形状、操作結果で識別でき、色だけ/labelだけへ依存しない

## プラグインパネルの拡張方式(3段構え)

> **競合注記(2026-07-13、GAP-13)**: 本節の2・3は[plugin-ui-model.md](../plugin-ui-model.md)の設計仮説(宣言語彙のみ・自由描画UIはv1で開けない)と**競合中**。採否判断(同§7の手順: 能力コーパス+AM実機確認→判定語付き決定)が下るまで、**2・3の実装には着手しない**。1は両案共通なので着手可。判断後、本節を同時改訂する。

1. **自動生成パネル(必須fallback・既定体験の大半)**: エフェクトプラグインはパラメータ定義(`NodeDesc`)を宣言するだけで、汎用プロパティパネル(Rustモデル駆動の行リスト: スライダー/カラーピッカー等)が自動生成される。**全保存パラメータはこのパネルだけで編集可能**でなければならない。カスタムUIは操作可能性を追加せず、速度・可視化・専用体験だけを改善する
2. **カスタムパネル(.slint実行時ロード)**: 独自UIが欲しいプラグインは`.slint`ファイルを同梱し、ホストが`slint-interpreter`(v1.17で存在確認済み)で実行時ロードする。プロパティ/コールバックの授受は実行時APIで行い、値の型はmotolii-evalの`Value`に限定する
3. **フルカスタム描画(スコープ・カーブ表示等)**: プラグインがwgpuテクスチャに描き、ホストがプレビューと同じ`Image::try_from`で埋め込む(ゼロコピー)

## 編集時Generator hook(one-shot)

上位の製品境界、Shape/SVGの分界、p5.js型表現の翻訳、Materialize/Live/Bakeの責任分担は[ジェネラティブユーザー境界](../generative-user-boundary.md)を正本とする。本節はそのうちMaterialize経路だけをM3の実装契約へ落とす。

JS/p5.jsをDocument・評価器・レンダ契約へ直接入れず、まず**編集時Generatorが型付きD2コマンドbatchを返す汎用hook**を置く。ホストは開始時snapshotに対してbatch全体をpreflightし、単一writerへ1 macroとしてcommitする。成功時は通常のGroup/Clip/VectorRecipeだけが残り、失敗・cancel・制限超過時はDocumentとUndo履歴を一切変えない。Generatorへ`&mut Document`は渡さない。

最初のadapterは**Motolii ShapeScript**とする。Paper.jsの`Project/Layer/Item/Path/Group`型object modelを設計参照にしつつ互換を名乗らず、座標はMotolii正準空間(原点中央・Y-up・高さ=1.0)、shape配置は中心基準、回転保存はradianに固定する。曖昧な位置引数を避け、`center`/`size`等のnamed fieldを使う。命令を通常のvector layer群へ変換し、1実行=1 Group=1 Undoとする。生成物は実行後に通常の編集UIで変更でき、保存・再読込・preview・exportにscript engineを必要としない。script source、runtime名、実行event stream、生成元provenanceはv1 Documentの必須意味にしない。

LLM向けの第2入口として**SVG materialize adapter**を分離する。SVGの公開語彙を入力に利用するが、左上原点・Y-down・viewport単位は入口で正準座標へ変換し、SVG DOM/XMLをDocumentの実行意味にせず通常のGroup/VectorRecipeへ実体化する。

これは**編集操作の量産口**であり、毎frame評価するlive JS layer、AE式expression、WASM Param Pipeline、plugin custom UIとは別境界である。p5.jsで一般的な「canvasをclearせず前frameの画素へ追描きする」表現も、scriptが隠しcanvasを所有する形では模倣しない。ただし表現自体は捨てず、有限loopを事前記録して通常shapeの出現時刻へ畳める場合はone-shot materialize、前出力そのものが必要な場合は[F-11 Feedback](../plugin-resources.md#6-時間参照-lookbehind--フィードバックf-11口の予約のみ)の明示的なホスト所有状態+チェックポイントBakeへ送る。JS engine/sandbox実装の選定はU9aの公開契約へ焼かない。

## タスク分割

| ID | 内容 | 依存 | 完了条件(概要) |
|---|---|---|---|
| U0a | `motolii-ui`クレート骨格+Slint依存方向CI | G0-1 | `motolii-ui`以外の製品クレートのSlint依存をCargo metadata検査が拒否。domain公開型へSlint型が無い |
| U0b | UI状態所有表+Slint非依存domain intent | G0-2, M2-D2 | 代表操作をDocument/User settings/Workspace/Transientへ分類。Slint型なしintentの単体テスト。恒久workspace形式はこのタスクで発明しない |
| U0c | input router+安定`CommandId`+event種別 | U0b, G0-2 | press/release/click/dragを区別し、IME preedit中のshortcut抑止を自動試験。物理入力からdomain intentまでSlint型なし |
| U0d | keymap base+user delta永続化 | U0c | builtin base不変、追加/置換/無効化deltaのroundtrip、version migration冪等、移行前原本と未知`CommandId`保持 |
| U0e | DTCG theme token generator+component state+icon体系 | G0-6, U0a | token JSONからRust/Slintへ決定的生成し手編集を拒否。text/non-text/focus contrast、gradient許可list、component state、icon grid、motion数値検査が通る |
| U1a | Slintアプリシェル+Manual共有デバイス+静止preview | U0a, S1, M2-D3 | Documentの同一frameをUI内へzero-copy表示。UI threadからrenderを直接呼べない |
| U1b | render worker+最新値mailbox+generation破棄 | U1a | 100連続seekで送信がblockせず、完了順を反転しても最新generationだけ表示。共有deviceで同期readbackなし |
| U1c | 起動/idle memory/input latency計測+開発HUD | U1b, G0-4 | 測定環境とraw結果を保存し、drop/latency/generationをHUD表示。閾値の採択は独立仕様改訂 |
| U1d | 日本語IME深部受け入れ | U1a, U0c | preedit下線、候補位置、変換中Enter非奪取、長文歌詞を対象OS実機で記録。失敗時は入力経路の仕様改訂へ戻る |
| U1e | 別window/別monitor preview spike | U1b | 同一Textureを別surfaceへ表示し、scale/monitor移動でDocument・評価結果不変。成立性と制約を記録 |
| U1f | [#169](https://github.com/oshikaidesu/Motolii/issues/169) Stage View+Output Frame+off-frame透過Draft | U1b, U0e, M2-D1k, M2-D3 | (1)同じcamera/worldからframe内+枠外を保守的に表示 (2)Stage View pan/zoom/fitでDocument serializeとFinal frame pixel不変 (3)Output Frame外を不透明塗潰しせず半透明scrim+形で識別 (4)frame外objectが無言で消えず選択可能 (5)UI thread readbackなし (6)overscan負荷をG0-4手順で測定。最適化なしでも成立し、K0導入後に見た目不変 |
| U2a | D2 command adapter+gesture macro/merge契約 | U0b, M2-D2 | Qt型macro/mergeを区別。公開gesture lifecycleは型宣言後に実装。代表gestureがUndo 1回、異gesture/異targetはmergeしないプロパティテスト |
| U2b | UI→command→writer→`Arc<Document>`購読E2E | U1a, U2a | UI callbackから編集しUndo/Redo込みで往復。UI状態変更だけではDocument serialize結果不変 |
| U2c | Direct/Tool/Advanced conformance harness | G0-7, U2b | 代表操作を存在する複数入口から実行し、同じDocument意味/Undo 1回/Cancel変更ゼロ。hidden helper/itemなし、未実装入口は明示。UIイベント列や入口種別をserializeしない |
| U2d | Camera/Output Frame直接操作+枠外object選択 | U1f, U2c | Camera toolはM2-D1j cameraだけをD2 command化し1 gesture=1 Undo。Hand/Fitはworkspaceだけを変更。frame外objectを選択・移動・snapでき、camera/object操作を混同しない。DPI差で同じ正規化gestureが同じdomain値 |
| U2e | LookAt/Follow/Parent型付きtarget picker | U2c, U2d, M2-D3 | Canvas/Timelineから対象をclickして`LayerId`をD2 command化。表示名変更で参照不変、自己参照/循環/削除済みtargetを型付き拒否、Cancel変更ゼロ、1選択=1 Undo。layer名/property path文字列やpick-whip式を保存しない |
| U2f | [#168](https://github.com/oshikaidesu/Motolii/issues/168) **modifier+drag one-shot Relative Move**: 安定gesture intentをkeymapから呼び、Position Const/全keyへ同じEdit-Space差分を適用するD2 macro | U0c, U0d, U2a, U2c, M2-D2 | (1)通常drag=現在値、modifier+drag=軌跡全体をHUD/ghostで識別 (2)pointer-upまでtransient、Undo 1回、Escape/capture loss変更ゼロ (3)混合型/削除済み/編集不可を開始前に型付き拒否し部分適用なし (4)時刻/補間/接線不変、既存値だけが変わりhelper/offset/Modifier/expressionを生成しない (5)DataTrack/FollowをBakeしない (6)専用Tool/panelなし (7)物理modifierはkeymapで変更可能 |
| U2g | **Timeline Effect Link**: Effect Definition `out`→各Layer effect-stack Use `in`の常時表示connection gutter | U0e, U2b, U3a, M2-D1l, M2-D3e | (1)非選択時も全接続線をgutter内に表示 (2)from/inをsocket形状+arrowheadで識別し色だけに依存しない (3)折畳み先はstub+件数badgeで接続存在を隠さない (4)drag中は型不一致をdimしstack挿入位置を表示 (5)1 drag=1 Use=Undo 1回、Cancel変更ゼロ (6)timeline順/renameで参照不変 (7)線はclip/key領域を横断しない (8)Group=合成後1回、Explicit=各layer個別適用をUIで混同しない (9)500 use fixtureのrouting/hit-testがUI threadをblockせず、全線またはbundle stubが常時存在 |
| U3a | Slint非依存timeline layout/hit-test+単一wgpu面 | U0a, U0b | clips 1,000+keys 100,000の固定fixture/viewport/操作列ベンチ。CIは基準比、実画面60fps閾値はG0-4後 |
| U3b | timeline配置/移動/trim操作 | U0c, U2b, U3a | drag操作がD2 commandを発行。ランダム操作列で重複なし・相対位置維持・Undo全巻戻し |
| U3c | 波形表示用derived cache+timeline描画 | U3a, M2-D4 | cacheの持ち場・無効化・上限を仕様化してから実装。波形データをDocumentへ焼かず、seek/zoom fixtureで一致 |
| U3d | timeline視覚統合+認知reference screen | U0e, U3a | 固定fixtureのgolden/lightness差分。5秒識別、grayscale、既存componentとの馴染み、Timeline Viewとの同条件比較を記録 |
| U4a | `NodeDesc`自動parameter panel | U2b, U0e, U1b | `ValueType → widget → command`対応表。全登録pluginの全保存paramが自動panelから編集可能なconformance。100回連続slider更新でUI送信がblockせず、操作中は最新generationだけをpreview表示し、確定値が一致し、gesture全体がUndo 1回 |
| U4b | keyframe編集+区間easing popup | U4a | key追加/補間切替/cubic-bezier 4値+presetが反映。既存区間の非対象curve不変プロパティテスト |
| U4c | Advanced意味検査+round-trip | U2c, U4a, M2-D1l | 現行DocParamのConst/Keyframes/Data/Vec2Axes/LookAt/Follow、plugin source/version、Effect Definition/Use ID、target、Owned/Explicit scope、policyを検査できる。Direct/Toolで作った状態を開閉してserialize不変。Simple時も非既定意味をbadge表示。未実装Param Pipeline、Composite Set、Backdrop地点をUIだけで捏造しない |
| U5 | scrub/再生transport UI | U0c, U0e, U1b, U3b, M2-D5 | vsync暴走注入でもTransport同期不変。最新seekのみ表示。低速時のvarispeedは数値試験+別記の聴感確認 |
| U6 | asset browser+import/D&D配置 | U0e, U2b | 動画/SVGを配置し楽曲1本を設定。欠落/不正assetをtyped error表示、UI threadでdecodeしない |
| U7 | beat grid+snap | U3b | 有理BPM/beat origin/meterからgrid生成。clip/keyframe snapがfps非依存のRationalTimeで一致 |
| U8a | group/clip mask UI | U3b, M2-D7 | grouping/ungrouping/clip modeがD2 command経由でUndo可能 |
| U8b | group仮出力toggle | U8a, M4-K7 | bake発動・編集時無効化がUIからE2Eで確認できる |
| U9a | **Editor Generator command hook**: 外部generator結果を型付きD2 command batchとして受けるSlint/runtime非依存境界 | U2b, M2-D2 | (1)generatorへ`&mut Document`を渡さない (2)開始snapshotに対するbatch全体preflight後だけ単一writerへ1 macro commitし、commit時に現行Documentと一致しない結果はstaleとして拒否 (3)成功=Undo 1回、失敗/cancel/制限超過/stale=Document・履歴変更ゼロ (4)journal/serializeには解決済み通常編集だけが残る (5)script engine無しでsave/reload/preview/export同一 (6)domain公開型にSlint/JS/runtime固有型なし |
| U9b | **Motolii ShapeScript one-shot adapter**: Paper.js型object/path/group思想を正準座標で再構成し、通常Group+vector layerへ変換 | U9a, M2-D1i-2 | (1)原点中央/Y-up/高さ1.0、center基準shape、radian、named fieldの固定表 (2)Path/Shape/Group/style/transform stack/unsupported APIの固定表 (3)同一script+明示`u64 seed`で同一command batch、時計/OS entropyなし (4)1実行=1 Group=1 Undo、生成物を通常編集可能 (5)network/filesystem/process/GPU textureへ非接続 (6)実行時間・command数・path点数・nest深度の上限超過を型付き拒否し部分生成なし (7)JS engineをDocument/renderer/plugin契約へ露出しない (8)editor buffer/script sourceの持ち場を分類し、恒久保存形式はこのタスクで発明しない (9)`draw()`/前frame画素/暗黙canvas蓄積が構文不能 |
| U9c | **SVG materialize adapter**: LLM生成SVG→通常Group/VectorRecipe | U9b | (1)SVG viewport/左上原点/Y-downを正準座標へ決定的変換 (2)採用element/style/transformと拒否表を固定 (3)DOM/XML/script/event/外部URLをDocumentへ残さず、外部参照と実行要素を型付き拒否 (4)materialize後はSVG parser/runtime無しでsave/reload/preview/export同一 (5)同じSVGから同じD2 batch、1 import=1 Undo |
| AG-3 | **v1.x追加レーン**: Video+Audio/Video Only import、audio component展開、mute/gain、音声分離macro | AG-1, AG-2, U6, U2c, U3a | 同じClipのmove/trim/retimeでA/V追従、分離前後PCM一致、Undo 1回、別project mode/別timeline schemaを作らない。現行U6のMV最短導線を置換せず追加する |

並列レーン: U0aと、G0-2完了後のU0bは並列。G0-6後のU0e、G0-7後のU2c準備は独立可能だが、実装はU2b待ち。U0b後はU0cとU2a、U0c後はU0d。U1a後はU1b/U1d/U2b、U1b+U0e+D1k/D3後にU1f。U2b後はU2c/U3a/U4a/U6/U9a、U9a後にU9b、U9b後にU9c。U0d+U2a+U2c後にU2f、D1l+D3e+U3a後にU2g、U1f+U2c後にU2d、U2d+D3後にU2e、U2c+U4a後にU4c。M2-D3後はM4-K0を独立実施可能。U0e/U3a後にU3d。U3b後はU5/U7/U8aを並列。U8bはM4-K7待ち。AG-3はAG-1/AG-2後のv1.x追加レーン。

## GR-UI審判割当表

| 規律 | 対象タスク | 自動審判 | 人間実機審判 |
|---|---|---|---|
| GR-UI-1 状態所有 | U0b, U0d, U1f, U2b, U2d, U3c, U9b, U9c | 状態分類fixture、Stage View変更時のDocument/Final不変、keymap delta roundtrip、script/SVG source非Document検査 | workspace/script復元UXは保存方針決定後 |
| GR-UI-2 command境界 | U0c, U2a, U2b, U2c, U2d, U2e, U2f, U2g, U3b, U4a, U4b, U4c, U6, U8a, U9a, U9b, U9c, AG-3 | input→intent、入口意味同値、target/definition/use ID、macro/merge/Undo property test、generator batch原子性、`&mut Document`依存検査 | — |
| GR-UI-3 thread/latest | U1a, U1b, U1e, U1f, U4a, U5, U6, U9b, AG-3 | non-blocking seek/parameter/overscan、generator実行中のUI応答、generation逆順、同期readback禁止 | 長時間scrub/parameter/camera drag、generator cancelの体感 |
| GR-UI-4 単位 | U1e, U1f, U2b, U2d, U4a, U4b, U7 | scale注入時domain command一致、UI degree↔Document radian、RationalTime、Stage View非永続 | 別monitor/DPI移動 |
| GR-UI-5 Slint隔離 | U0a, U0b, U1f, U2c, U3a, U9a, U9b, U9c | Cargo metadata+公開型走査、generator/SVG hookのruntime/Slint非依存test、windowなしlayout/hit-test test | — |
| GR-UI-6 performance | U1c, U1f, U3a | 固定fixture/overscan基準比 | 基準機p50/p95、起動、idle memory、Stage pan/zoom |
| GR-UI-7 plugin fallback | U4a | 全登録plugin conformance | widget操作性 |
| GR-UI-8 視覚認知 | U0e, U1f, U2d, U2e, U2f, U2g, U3d, U4a, U4c, U5, U6 | token生成差分、Output Frame/Stage View/Camera/target-pick/Relative HUD/from-in接続状態、raw color、contrast、icon/state、通常+lightness/CVD reference画像 | 5秒識別、frame内外/Camera対Hand/通常drag対Relative/from対in、grayscale/CVD、既存UIとの馴染み |

表にない横断変更を行う場合は、PR前に本表へ審判を追加する。人間実機審判だけで「完了」にせず、自動審判と別の証跡として残す。

## 実装ガード(先行ツールの失敗・ユーザー不満クロスチェック 2026-07-11)

Slint実運用の既知問題と、出荷済みエディタのタイムライン/プレビュー苦情(FCPX/AE/Kdenlive/Shotcut/Resolve/AviUtl)を調査し、既存方針に無いガードを抽出した。**先頭2項目は「M3後半に発覚すると設計が覆る」種類のリスクなので、U1d/U3aへ独立割当する。**

以下は個別タスクのガードである。横断する状態所有・コマンド境界・GPU/スレッド・単位・Slint隔離・公開契約の停止条件は[GR-UI](../reviews/2026-07-14-m3-ui-boundary-prevention.md)を正本とし、適用先は審判割当表で限定する。

1. **日本語IME受け入れをU1dへ分離する**: SlintのTextInputはIME対応後もCJK混在時のカーソル位置ズレが報告され、下層winitにはWaylandで`set_ime_allowed`未呼び出しだとfcitx5/ibusが一切起動しない・候補ウィンドウ位置誤りの既知穴がある。チェックリスト: (1) preedit下線表示 (2) 候補ウィンドウがカーソル位置に追従 (3) **変換中のEnterがアプリのショートカットに食われない** (4) 長文歌詞の連続入力。対象: Windows MS-IME / macOS / Linux(fcitx5+Wayland, ibus+X11)。1つでも落ちたらshortcut special-caseで隠さず、入力経路の仕様改訂へ戻る
2. **タイムライン・波形・グラフ類はSlintエレメントで組まない**: SlintのListViewは1画面超のコンテンツで全面フリッカー(報告者が「自力では改善不能」と書いて未解決クローズ)、カスタム要素170個の破棄に数秒の報告があり、「数百クリップ×数万キーフレーム×毎フレーム更新」はエレメントモデルの想定外。タイムラインは1枚のカスタムレンダリング面(wgpuテクスチャ、プレビューと同じ`Image::try_from`埋め込み)に自前描画し、Slintはイベント受けとシェルに限定する。U3aでclips 1,000+keys 100,000の固定fixtureを測る。**60fpsはG0-4で基準機・操作列・p50/p95を決めた後の製品目標で、hardware未指定のCI閾値にしない**
3. **再生のフレームペースをvsync/描画コールバックに依存させない**: Slintのfemtovg-wgpuレンダラにはvsync破綻(60Hzで約3ms間隔の暴走描画)の未トリアージバグがある。主クロックは音声(M2 Transport)で確定済みだが、「vsyncが暴走してもフレームペースと音声同期が崩れない」ことをU5のテスト観点として明記する
4. **Slint APIに触れるのは`motolii-ui`だけ(依存方向をCIで強制)**: ArdourはGTK2から移行できず自前フォーク(YTK)を生涯保守する道を選んだ — メディアアプリはカスタムウィジェット比率が高く、ツールキットAPIが全域に染みると移行コスト=全書き直しになる(Qtの2020年LTS商用化パニックのようにベンダー方針は10年スパンで変わる)。タイムライン/preview描画modelはSlint非依存に置く。ただしSlint callback→domain intent変換adapterは`motolii-ui`内でよい。禁止対象は他crateのSlint依存とdomain公開型へのSlint型流出
5. **タイムラインの革新的挙動は必ずオプトイン**: FCPXのマグネティックタイムライン強制は3,700筆超の抗議署名とプロ層の恒久流出を生んだ(「概念として優れていても、訓練されてきた全てに反する」)。既定は業界標準の操作(トラック型、スペース再生、スナップのキートグル)とし、ショートカットはAviUtl2同様に初日からカスタマイズ可能にする(AviUtl層の獲得条件)
6. **「キーフレームを追加しても既存区間のカーブ形状が変わらない」を不変条件に**: AEグラフエディタの「イージングを入れるとスパイク/ループが出る」「予測可能な調整がほぼ不可能」という定番苦情は、キー追加・移動時に近傍カーブが暗黙に変わることが根因。区間イージング方式(採用済み)はこの罠を大きく回避するが、この性質自体をmotolii-evalのプロパティテストとして固定する → U4b
7. **ランダム編集操作列のプロパティテスト**: Kdenliveは「保存→再起動でクリップが複製され、後続クリップがまとめてズレて音ズレ」等のモデル不整合で「不安定」の評判が定着した(単発操作でなく操作の合成で壊れる)。「ランダムな操作列(配置/移動/トリム/グループ/undo-redo)を数千回適用しても (a)クリップ重複なし (b)グループ内相対位置維持 (c)undo全巻き戻しで初期状態一致」を、M2-D2の単発プロパティテストの系列版としてU2a/U3bのCIに置く
8. **スクラブは「最新要求だけ保持」+generationで旧結果を捨てる+観測可能に**: render requestはblocking容量1 channelでなく最新値置換mailboxにする。実行中GPU workの強制cancelは要求せず、完了した旧generationをUIが表示しない。開発ビルドにdrop/latency/generation HUDを置く — 再生系苦情の大半は「間に合わない時のポリシー未定義」に還元される
9. **プレビュー別ウィンドウ(マルチモニタ)の成立性をU1eでスパイク**: Resolveの「パネル取り外し不可」は10年級の不満。フルdockingは不要(固定分割方針のまま)だが、「プレビューを別ウィンドウ/別モニタへフルスクリーン表示」の1点だけは、Slintマルチウィンドウ+wgpuテクスチャ共有(別surface/swapchain)の成立性を早期確認する
10. **起動時間・アイドルメモリはU1cで測ってから数値目標を採択する**: AviUtl層のユーザーは重さに敏感だが、測定前のN秒/M MBを公約しない。G0-4の基準機と手順でraw値を取り、閾値は独立仕様改訂で固定する
11. **アクセシビリティはG0-2で保証範囲を決める**: Slintのアクセシビリティ(特にTextInput)は未完成で、カスタム描画timelineは自動ではアクセシビリティtreeに乗らない。やらない範囲も明記し、代替としてキーボード完結操作を保証する範囲を決めてからU0b/U3bへ配線する

12. **Param PipelineをUIから先に発明しない**: U4a/U4cは現行`DocParam`の出所を編集・検査する範囲なら進めてよい。常設Relative Offset、Generator/Modifier列、DataTrack+手補正の同時適用、評価列並べ替え、汎用parameter pluginのいずれかが必要になった時点で[PP-Gate](../interaction-simplicity-model.md#4-param-pipeline-gatepp-gate)を開始し、M1/M2解凍・migration・意味論golden・反対側レビュー前は実装を止める

13. **one-shot Generatorをlive runtimeへ拡張しない**: U9a〜U9cが許すのは、制限付きworkerでcommand batchを生成し、開始snapshotと現行Documentをcommit時に照合して全体preflight後に通常編集として1回だけcommitする経路だけ。新しいtransaction/revision公開APIをこのタスクで発明しない。script/SVG source、runtime、provenanceを必須Document意味へ追加しない。毎frame JS、expression、Param Pipeline、部分commit、暗黙の乱数/時計、未対応APIの黙示fallbackが必要になったら実装を止める。前frame画素への追描きはU9内で隠し状態化せず、F-11 Feedback+K1/K7後のSCR-4へ送る

出典: slint-ui/slint#1644・#4097・#8693・#2895 / rust-windowing/winit#2888 / warpdotdev/warp#9383 / variety.com(FCPX抗議署名) / creativecow.net(AEグラフエディタ苦情) / KDE Bug 369505(Kdenliveクリップ複製) / forum.blackmagicdesign.com(Resolveパネル分離) / phoronix.com(Ardour YTK) / theregister.com(Qt LTS商用化) / forum.shotcut.org(プレビューラグ)

## 未決事項

- ~~S1の実機確認(IME・tearing)の結果次第で退避判断(egui → Tauri)~~ → **INF-1合格でSlint確定**(2026-07-11)。深いIMEは実装ガード1のチェックリストで合否判定する
- OpenCutからコードは取り込まない。操作仕様・レイアウトのどの観察を採るかだけをU3a着手前に棚卸しする
- Param Pipelineの具体型は未決。U4cは現行意味の可視化までで、Modifier UIの採否判断ではない
- 枠外overscanの距離別品質・bounds cache・VRAM予算の固定値はU1f着手前spikeで決める。Stage全域を無制限Final描画するdefaultは採らない
- U9bのJS engine、sandbox方式、script保存場所は未決。p5.js互換はv1要件にせず、有限one-shot命令のsyntax sugarが必要ならShapeScript完成後に別判断する
