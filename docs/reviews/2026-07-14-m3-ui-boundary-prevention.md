# M3 UI境界汚染の予防(2026-07-14)

ステータス: **運用手順**(反対側レビュー反映済み)。採否と縮小理由は[反対側レビュー](2026-07-14-m3-ui-boundary-counter-review.md)が正本。個別機能の意味論・依存・審判割当は[M3仕様](../specs/M3-ui-integration.md)が正本であり、本手順は未決事項を決定しない。

## 前提

- UIはM2 `Document`の投影であり、別の制作データ正本ではない
- DPI・ウィンドウ・入力頻度・描画頻度は環境依存で、Document・評価・公開プラグイン契約へ流さない
- 本手順は[M2恒久焼き込み予防](2026-07-12-m2-permanence-prevention.md)を置き換えない。Documentスキーマへ触るタスクは両方を適用する
- 全タスクへ同じチェックを課さない。各タスクが満たす審判はM3仕様の「GR-UI審判割当表」で決める

## 規律8本

### GR-UI-1. 状態の所有者を先に決める

実装前に状態を4層へ分類する。

| 層 | 例 | 現時点の規約 |
|---|---|---|
| Document | クリップ配置、パラメータ、キーフレーム、グループ | D2コマンド経由で保存・ジャーナル対象 |
| User settings | キーマップ、テーマ等 | Document外。GAP-6でbase+user delta・version・移行を決めるまで形式を焼かない |
| Workspace/session候補 | パネル幅、scroll、timeline zoom、開いていたpanel | Document外。保存寿命・project/globalの帰属はM3確定ゲートで決める |
| Transient interaction | 選択、hover、IME preedit、ドラッグ途中、popup | Document・ジャーナル対象外。選択/hover/IMEは#103決定済み |

「Document外」と「永続化しない」は同義ではない。workspace/session候補をその場の都合でUser settingsやDocumentへ混ぜない。

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

これらはD2完成後、U2aでQt型macro/mergeの区別を参考に型とプロパティテストを先に決める。

### GR-UI-3. UIスレッドを待たせず、最新要求だけを表示する

- GPUデバイスはコアが作り、egui shellは`WgpuSetup::Existing`で借りる
- render/decodeはworkerで行い、egui状態の投影はevent-loop threadへ戻す
- render requestは最新値置換mailbox。UIからの送信はblockしない
- requestへ単調増加generationを付け、古いcompleted frameをUIが表示しない
- 実行中GPU workの強制cancelは要求しない
- UI共有デバイスで`device.poll(Wait)`、`download_rgba`、フレームごとのGPUリソース生成をしない
- native textureはdisplay pool生成時に登録し、毎frame sampler/bind groupを作らない
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

審判はCargo metadataの依存検査と公開型走査。callback adapterそのものをUIクレート外へ追い出すことは要求しない。

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

## 停止条件

- 状態の4層分類ができない: M3仕様改訂へ戻る
- D2 API完成前にgesture transaction型が必要: U2aまで実装を止める
- GAP-13未決: カスタムplugin UIと`ParamDef`拡張を止める。自動生成fallbackは進めてよい
- GAP-6未決: 恒久keymap形式を焼かない
- IME不合格: shortcut special-caseで隠さず入力経路の仕様改訂へ戻る
- performance測定条件未記録: 60fps達成/未達を完了報告に使わない
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
8. GAP-6/GAP-13等の未決を実装defaultで埋めていないか
9. 主要状態を文字だけ/色だけで表していないか。新規componentが既存token体系から逸脱していないか

## 改訂記録

- 2026-07-14: 初版
- 2026-07-14: 反対側レビューR1〜R9を反映。状態を4層化、gesture API固定を撤回、最新値mailbox+generation、性能測定2層、タスク別審判へ縮小
- 2026-07-14: GR-UI-8を追加。操作参照と視覚参照を分離し、Timeline View限定、Arrangement View非採用、意味色と既存UIへの馴染みを審判化
- 2026-07-18: [egui採用判断](2026-07-18-m3-egui-selection.md)を反映。GR-UI-3〜7をegui/native texture/egui_tiles境界へ再翻訳
