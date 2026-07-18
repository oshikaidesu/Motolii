# 反対側レビュー: M3 UI境界規約を実装可能な最小形へ縮小する(2026-07-14)

ステータス: **批判レビュー完了・判定反映済み**。対象は初版の[M3 UI境界汚染の予防](2026-07-14-m3-ui-boundary-prevention.md)。2026-07-14の批判ラウンドで挙がったP0/P1を、既存決定と公式資料に照らして再判定した。

追記(2026-07-18): 本書はSlint採用時点の反対側レビューである。状態所有、単一writer、thread、単位、toolkit隔離、性能審判の結論は維持し、Slint固有のAPI・scale注入・計測手段は[egui採用判断](2026-07-18-m3-egui-selection.md)と改訂済み[予防手順](2026-07-14-m3-ui-boundary-prevention.md)で置換する。

## 判定基準

1. M2または既存M3仕様で決定済みか
2. UI都合をDocument・評価・公開プラグイン契約へ不可逆に焼くか
3. 審判をタスクとコマンドへ割り当てられるか
4. より小さい境界で同じ事故を防げるか

判定語は[レビュー文書の規律](README.md)どおり、採用・縮小・延期・棄却を使う。

## 判定

### R1. 反対側レビュー前の運用正本化 — **縮小**

初版は作成と同時にAGENTS/M3仕様から強制参照され、レビュー文書の規律2・6を飛ばした。本レビューを採否記録とし、予防文書には**本レビューで採用・縮小した項目だけ**を残す。未決を決める項目は停止条件またはM3確定ゲートへ戻す。

### R2. UI状態を3群へ固定 — **縮小**

#103で決定済みなのは選択・hover・IME中間がDocument外という範囲であり、パネル幅・ズーム・スクロールの再起動後復元は未決である。「Documentへ保存しない」と「永続化しない」を混同しないため、持ち場を次の4層へ分ける。

1. Document: 制作結果の意味
2. user settings: 複数プロジェクトにまたがる設定
3. workspace/session候補: パネル配置・表示範囲等。保存寿命と形式はM3で別途決定
4. transient interaction: hover・IME preedit・ドラッグ途中等

Blenderも制作データとは別のPreferencesにkeymap等を保存する([Blender Defaults](https://docs.blender.org/manual/en/3.0/getting_started/configuration/defaults.html))。ただしMotoliiのworkspace保存形式までBlenderから転写しない。

### R3. `begin/update/commit/cancel`の固定 — **延期**

#103の採用範囲はatomic command、gesture macro、同一対象・同一プロパティのmergeまで。Qt Undo Frameworkもcommand compressionとmacroを別機構として提供するが、ポインタgestureのtransaction API自体はアプリ側の判断である([QUndoStack](https://doc.qt.io/qt-6/qundostack.html)、[QUndoCommand::mergeWith](https://doc.qt.io/qt-6/qundocommand.html#mergeWith))。

したがってM3規約は「D2コマンドだけを正本へ適用」「1 gesture=1 Undo」を固定し、ドラッグ途中をDocumentへ仮適用するかoverlayで見せるか、cancelをどのAPIで表すかはD2完成後のM3境界タスクで型として決める。

### R4. 最新要求の容量1チャネル — **縮小**

単なるbounded channelは満杯時に送信側を待たせ得る。標準`sync_channel`も満杯時sendはblockする([Rust `sync_channel`](https://doc.rust-lang.org/std/sync/mpsc/fn.sync_channel.html))。必要な意味はTokio `watch`と同じ「最新値だけ保持し中間値を落とす」mailboxである([Tokio watch](https://docs.rs/tokio/latest/tokio/sync/watch/))。

採用する契約:

- render requestは最新値置換mailbox。UI送信はblockしない
- 実行中のGPU workは無理にcancelせず、requestへ単調増加generationを付ける
- completed frameもgenerationを持ち、UIは最新要求より古い結果を表示しない
- 具体クレートとしてTokioを採用する決定ではない。意味が同じ小実装でもよい

Slint componentはevent-loop threadでのみ更新し、workerからは公式の`invoke_from_event_loop`または`Weak::upgrade_in_event_loop`相当で戻す([Slint threading](https://docs.rs/slint/latest/slint/)、[`invoke_from_event_loop`](https://docs.rs/slint/latest/slint/fn.invoke_from_event_loop.html))。

### R5. px/DPI不変条件 — **採用**

Slintの`px`は論理pixelでdevice pixel ratioへ自動追従し、`phx`が物理pixelである([Slint positioning](https://docs.slint.dev/latest/docs/slint/guide/language/coding/positioning-and-layouts/))。UIのscaleをDocument・評価・プラグイン処理へ流さない既存反対側レビューの判定を維持する。

自動審判は実モニタ移動そのものではなく、`SLINT_SCALE_FACTOR`等でscaleを注入した同一入力のdomain command/Document一致とする。別モニタ表示は人間実機審判として分離する。

### R6. Slint隔離の範囲 — **縮小**

「コマンド生成をUIシェル外へ置く」は広すぎる。UI callbackをdomain intentへ変換するadapterはUIクレートに置いてよい。禁止するのは次の2点に縮小する。

- Slint型がdomain intent、Document command、core/eval/render/pluginの公開APIへ出ること
- `motolii-ui`以外の製品クレートがSlintへ依存すること

これならCargo metadataと公開型走査で機械判定できる。

### R7. 1,000 clips + 100,000 keys = 60fps — **縮小**

負荷データは固定するが、ハードウェア・viewport・操作軌跡・統計量なしの60fpsをCI公約にしない。

- CI: レイアウト/ヒットテストの固定入力ベンチと基準比回帰
- 基準機: 実画面のpan/zoom操作軌跡、解像度、warm-up、測定時間、p50/p95 frame timeを記録
- 製品目標60fps: U1c/U3aの初回実測後に、基準機と許容値をM3仕様へ採択

Slint自身もperformance表示機能を提供するが、描画が発生したframeだけを測るモード等があるため測定条件を明記する([Slint debugging techniques](https://docs.slint.dev/latest/docs/slint/guide/development/debugging_techniques/))。

### R8. 自動生成パネルの「9割」と「完全操作可能」の矛盾 — **採用(仕様訂正)**

カスタムUIの有無でプロジェクトの操作可能性が変わらないよう、**全保存パラメータは自動生成パネルから編集可能**を必須fallbackにする。「9割」は利用頻度/体験の既定を指す表現に改め、操作可能性の保証には使わない。

ただし「完全操作可能」の審判には`ValueType → 標準widget`対応表が必要であり、U4aへ割り当てる。WidgetHintやカスタムUI APIはGAP-13決着前に足さない。

### R9. 全タスク一律チェック — **縮小**

全項目Yes方式は非該当タスクを偽装させる。M3仕様にタスク別審判表を置き、各タスクは割り当てられた審判だけを満たす。表にない横断変更を行う場合は、該当規律を追加して仕様改訂する。

## 結論

初版の7原則そのものは残すが、次を除去する。

- 未決workspace状態を「保存しない」と固定すること
- 未決gesture transaction APIを規約で発明すること
- hardware未指定の60fpsをCI審判にすること
- 検査方法のない「到達不能」「全操作可能」を完了条件と呼ぶこと

有効化条件は、修正版予防文書とM3仕様のタスク別審判表が同時に更新されること。本レビュー単独ではコード実装を解禁しない。
