# M3 UI境界汚染の予防(2026-07-14)

ステータス: **運用手順**(反対側レビュー反映済み)。採否と縮小理由は[反対側レビュー](2026-07-14-m3-ui-boundary-counter-review.md)が正本。個別機能の意味論・依存・審判割当は[M3仕様](../specs/M3-ui-integration.md)が正本であり、本手順は未決事項を決定しない。

## 前提

- UIはM2 `Document`の投影であり、別の制作データ正本ではない
- DPI・ウィンドウ・入力頻度・描画頻度は環境依存で、Document・評価・公開プラグイン契約へ流さない
- 本手順は[M2恒久焼き込み予防](2026-07-12-m2-permanence-prevention.md)を置き換えない。Documentスキーマへ触るタスクは両方を適用する
- 全タスクへ同じチェックを課さない。各タスクが満たす審判はM3仕様の「GR-UI審判割当表」で決める

## 規律9本

### GR-UI-1. 状態の所有者を先に決める

実装前に状態を5層へ分類する。所有と寿命の正本は
[M3着手前決定 G0-2](2026-07-16-m3-preflight-decisions.md#22-状態の持ち場と寿命)であり、
本表はその語彙を縮約しない。

| 層 | 例 | 現時点の規約 |
|---|---|---|
| Document | クリップ配置、パラメータ、キーフレーム、グループ | D2コマンド経由で保存・ジャーナル対象 |
| User settings | キーマップ、テーマ等 | Document外。keymapはG0-2と[U0d-2 codec契約](2026-07-20-m3-keymap-codec-contract.md)のbase+user delta・version・原本保全だけを焼く。theme等は各決定待ち |
| Workspace profile | パネル幅、開いていたpanel、Timeline density等の作業配置 | user単位。Document外で、壊れた場合に既定へ全resetできる |
| Project session | Stage View pan/zoom/fit、Timeline scroll/zoom、選択中panel等 | project identity単位のbest-effort cache。欠落・破損時は安全な既定へ戻す |
| Transient | 選択、hover、IME preedit、ドラッグ途中、popup | Document・ジャーナル対象外。選択/hover/IMEは#103決定済み |

「Document外」と「永続化しない」は同義ではない。Workspace profileとProject sessionを
その場の都合で統合したり、User settingsやDocumentへ混ぜたりしない。U0b-1はこの5層を
型とfixtureへ写す実装であり、新しい所有層・寿命・永続形式を決める場ではない。

### GR-UI-2. 永続編集はD2コマンドだけを通す

固定済みの契約:

- atomic commandは1対象・1プロパティ
- 1 gestureは1 macro、Undo 1回
- 同一gesture・同一対象・同一プロパティの更新はD2のmerge keyで結合
- ジャーナルへポインタ軌跡やUIイベントを記録せず、決定済み値を記録
- UIは`Arc<Document>`を読み、`&mut Document`を持たない

未決のため固定しないもの:

- `begin/update/commit/cancel`等の公開型
- ドラッグ途中をDocumentへ仮適用するか、UI overlayで表示するか
- cancel/フォーカス喪失/ウィンドウ終了時のtransaction意味論

これらはD2完成後も一括して発明しない。U2a-1は
[gesture command adapter契約](2026-07-20-m3-u2a-1-command-adapter-contract.md)により、
決定済みcommandを伴うruntime-only requestと初回適用前Cancelだけへ縮小した。
適用後Cancel、drag途中の仮適用／overlay、公開gesture lifecycleは、必要になる個別
チケットで型とプロパティテストを先に決める。

### GR-UI-3. UIスレッドを待たせず、最新要求だけを表示する

- GPUデバイスはコアが作り、egui shellは`WgpuSetup::Existing`で借りる
- render/decodeはworkerで行い、egui状態の投影はevent-loop threadへ戻す
- render requestは最新値置換mailbox。UIからの送信はblockしない
- requestへ単調増加generationを付け、古いcompleted frameをUIが表示しない
- 実行中GPU workの強制cancelは要求しない
- UI共有デバイスで`device.poll(Wait)`、`download_rgba`、フレームごとのGPUリソース生成をしない
- native textureはdisplay slot生成時に安定viewを作り、rendererを得られる
  `eframe::CreationContext`で一度だけ登録する。毎frame、resize、DPI変更、
  minimize/restoreごとにsampler/bind groupを作らない
- 再生クロックをvsync/egui repaintへ従属させない

mailboxはTokio `watch`相当の意味を要求するが、Tokio採用自体は決定しない。単なる容量1のblocking channelで代用しない。

### GR-UI-4. UI単位を永続層へ流さない

- 空間値: 正準座標(原点中央・Y-up・高さ=1.0)
- 回転: Document/commandはラジアン。度は表示変換のみ
- 時刻: M2の`RationalTime`等
- 色: UI pickerも保存空間を変えず、色変換はレンダ直前の1箇所
- egui point/物理px、DPI scale、window座標: UI adapter内だけ

scale変更の自動審判は注入したscaleで同一操作から同一domain command/Documentが得られること。実モニタ移動は人間実機審判として分ける。

### GR-UI-5. UI toolkitをadapter境界へ封じ込める

製品UIクレート名は`motolii-ui`とする。

- `motolii-ui`はegui/winit eventをtoolkit非依存のdomain intent/commandへ変換してよい
- `motolii-ui`以外の製品クレートは`egui` / `eframe` / `egui-winit` / `egui-wgpu` / `egui_tiles`へ依存しない
- egui/eframe/winit型をdomain intent、Document command、core/eval/render/pluginの公開APIへ出さない
- timeline layout/hit-test/render modelはウィンドウなしでテスト可能なtoolkit非依存moduleに置く
- panel layoutはMotolii所有modelから`egui_tiles` runtime treeへ投影し、`Tree`/`TileId`/crateのserde形を保存正本にしない

審判はCargo metadataの直接依存検査（`package = "…"` renameを含む解決済みpackage名）と公開型走査。callback adapterそのものをUIクレート外へ追い出すことは要求しない。

### GR-UI-6. 負荷と測定方法を先に固定する

timeline/波形/keyframeを項目ごとのegui widgetで作らず、単一wgpu面へ描画する。大規模listは仮想化する。負荷データは「clips 1,000 + keyframes 100,000」を固定する。

測定を2層に分ける。

1. CI: 固定viewport・固定操作列のlayout/hit-test benchmarkを基準比で回帰判定
2. 基準機: 実画面の解像度、GPU/CPU/OS、warm-up、測定時間、pan/zoom操作列、p50/p95 frame timeを記録

60fpsは製品目標であり、hardware未指定のCI合否ではない。U1c/U3aの初回実測後に基準機と閾値をM3仕様へ固定する。

### GR-UI-7. 自動生成パネルを必須fallbackにする

- 全保存パラメータは`NodeDesc`自動生成パネルだけで編集可能にする
- カスタムUIは操作可能性を追加せず、速度・可視化・専用体験だけを改善する
- `ValueType → 標準widget → command`対応表をU4aで作り、全登録pluginをconformanceで走査する
- plugin所有のegui/native UI code、自由wgpu UI、`ParamDef::WidgetHint`、DPI/toolkit型を渡すAPIへ着手しない

「自動生成が9割」は既定利用率の見込みであり、残り1割が操作不能でよいという意味には使わない。

### GR-UI-8. 文字を読む前の識別と既存UIへの馴染みを審判する

外観の正本は[UI視覚言語](../ui-visual-language.md)とする。

- 操作動線はOpenCut、Flow/Alight Motion、一般的なトラック型UIを参照する
- AbletonはTimeline Viewの視覚言語だけを参照し、Arrangement Viewの構成やDAW操作モデルを輸入しない
- 選択・種別・状態は位置/形/icon/意味色を組み合わせ、文字だけにも色だけにも依存しない
- 装飾gradient、glassmorphism、neon glow、card/pillの乱用を禁止する
- 新規componentは既存のspacing、radius、stroke、icon grid、意味色へ馴染ませる
- 任意のtrack/clip色を見た目の都合だけでDocumentへ追加しない。必要ならGR-PVへ戻る

自動審判はtheme外raw color、contrast、gradient許可list、component state matrix、reference screen golden/lightness差分。5秒識別、grayscale、既存UIとの馴染み、Timeline Viewとの同一fixture比較は人間審判として別に記録する。5秒は普遍的な認知研究の主張ではなく、M3内の比較条件である。

### GR-UI-9. UI実装は数値ログと数値審判を残す

layout、座標変換、DPI、hit-test、drag、render viewport、起動時間、frame time、resource量を
変更するUI実装は、見た目だけで完了判定しない。入力から最終domain値までの変換境界を
同じgeneration / revision / layout epochで結べる構造化数値ログと、代表点・境界・異なる
scale factorを固定した自動試験を先に用意する。

- 座標経路はraw platform point、platform座標向き、logical point、layout rect、NDC、正準値を記録する。
- 性能経路は対象build、hardware、warm-up、試行数、p50 / p95とraw sampleの保存場所を記録する。
- visual比較は入力寸法、device scale、viewport、source / output解像度を記録する。
- 通常製品UIの縦sliceは起動phase、window / DPI / layout、WebView load / IPC / focus、
  typed intent、preview submit / result、Document revision、Stage投影、Timeline projection /
  hit、Inspector publish、Undo / Redo、surface recovery / failureを一つの構造化ログ系列で記録する。
- 毎frameやpollの同値行を反復せず、連続値は変化時、状態は遷移時に記録する。ログ量でUIを
  遅くしたり重要eventを埋没させない。
- debug製品UIは数値ログを常時出す。release buildはdevelopment診断入口から明示的に有効化する。
- ログはTransient診断であり、Document、journal、公開API、plugin契約、永続形式へ入れない。
- 秘密情報、ユーザー入力本文、project内容を記録せず、数値と不透明なgeneration / revisionだけにする。

ログの存在だけでは合格にしない。期待値をassertする自動試験を審判とし、実機の目視確認は
ログの同一操作列と対応づけた補助証跡にする。必要な変換境界を数値化できない場合は、UI実装を
進めず観測点の所有境界を先に決める。

通常の実機確認は`./scripts/run-ui-trace.sh <project.json>`から起動する。stderrの構造化ログを
`target/ui-traces/motolii-ui-<UTC timestamp>.log`へ同時保存し、確認後にstartup、failure、
対象操作のgeneration / revision / layout epochを読む。`target/`内のログは開発証跡であり
commitしない。PRへ証跡を添える場合は秘密情報を含まない該当行と測定条件だけをレビュー文書へ
転記し、生ログ全体やproject内容をrepositoryへ保存しない。

既存UIテストも、対象実装を変更する粒で同時に更新する。全テストへ無関係なダミー数値を
追加するのではなく、layout / DPI / 入力 / hit-test / 描画 / 性能 / resource量を扱うテストは
raw値と期待値を明示し、文字列codec、状態機械、公開境界だけを扱うテストは従来の意味審判を
維持する。数値ログの必須field自体も契約テストで固定する。

## 停止条件

- 状態の5層分類ができない: M3仕様改訂へ戻る
- D2 API完成前にgesture transaction型が必要: U2aまで実装を止める
- GAP-13未決: カスタムplugin UIと`ParamDef`拡張を止める。自動生成fallbackは進めてよい
- GAP-6/U0d-2 codec契約の外: 未決field・旧版変換・保存場所・presetを恒久keymap形式へ焼かない
- IME不合格: shortcut special-caseで隠さず入力経路の仕様改訂へ戻る
- performance測定条件未記録: 60fps達成/未達を完了報告に使わない
- UI変換経路の数値ログまたは期待値assertがない: 目視だけで完了にせず、観測点を先に追加する
- Documentまたは公開plugin契約の変更が必要: GR-PVまたは解凍手続きを先に行う

## エージェントの着手前チェック

M3仕様の審判割当表で対象タスクに割り当てられた項目だけを確認する。非該当項目を形式的にYesにしない。

1. 状態の所有層が決まっているか
2. 永続編集がD2 command/単一writerだけを通るか
3. UI threadをblockする処理やblocking sendがないか
4. px/DPI/度/window座標がdomainへ漏れていないか
5. egui/eframe/winit依存と型が`motolii-ui`の外へ漏れていないか
6. 自動審判のfixture、command、合否条件がタスク完了条件にあるか
7. IME/別monitor/聴感等の人間審判を自動試験で代用していないか
8. GAP-6のU0d-2 codec契約外やGAP-13等の未決を実装defaultで埋めていないか
9. 主要状態を文字だけ/色だけで表していないか。新規componentが既存token体系から逸脱していないか
10. UIの入力・変換・出力を同じgeneration / revision / layout epochで数値追跡でき、期待値をassertしているか

## 改訂記録

- 2026-07-14: 初版
- 2026-07-14: 反対側レビューR1〜R9を反映。状態を4層化、gesture API固定を撤回、最新値mailbox+generation、性能測定2層、タスク別審判へ縮小
- 2026-07-14: GR-UI-8を追加。操作参照と視覚参照を分離し、Timeline View限定、Arrangement View非採用、意味色と既存UIへの馴染みを審判化
- 2026-07-18: [egui採用判断](2026-07-18-m3-egui-selection.md)を反映。GR-UI-3〜7をegui/native texture/egui_tiles境界へ再翻訳
- 2026-07-20: G0-2で確定済みの5層所有語彙へ同期し、U0b-1が意味決定を発明しない停止線を明記
- 2026-07-30: GR-UI-9を追加。全UI実装を構造化数値ログ、同一世代の変換追跡、数値assertで審判し、目視だけの完了判定を禁止
