# UIモジュールの責任

依存の正本は[modules.json](modules.json)。`scripts/motolii-ui.sh check`で参照方向と所有を確認する。ファイル数や行数だけを完成条件にしない。

| モジュール | 持つ責任 | 持たない責任 |
|---|---|---|
| `app` | 起動、窓のライフサイクル、保存確認、OS窓との接続 | Dock内部、個別キーの判定、作品データ |
| `workspace` | パネル配置、タブ、幅、パネルフォーカス | Document編集、OS操作、具体的パネル実装 |
| `input` | 共通ショートカット、入力の所有、ナビゲーションと慣性 | ファイル操作の実装、描画エンジン |
| `foundation` | 色・寸法、ボタン、メニュー、文字/数値の下書き入力 | Session、Document、個別パネル |
| `panels` | 結果の表示、対象固有の操作入口 | Native transportの直接呼出し、第二の作品状態 |
| `session` | snapshot受信、命令queue、再生要求の調停 | ウィジェットやDockの形 |
| `bridge` | MethodChannel、型付きoperationとwire codec | UI判断、作品の編集意味 |
| Rust `editor` | 選択/clipboard/gestureをDocument/Intentへ変換する編集層 | Flutter widget、作品の第二の保存先 |
| Rust `doc` | 不変条件、親子・キー、transaction、Undo | OS/Flutterの状態 |

`main.dart`はアプリの起動だけ。パネル実体の組み立ては`panels/registry.dart`、一覧の名前は`workspace/panel_ids.dart`。nativeへの命令はEditorSessionを通し、bridge以外でMethodChannelを生成しない。

共通編集helperは`motolii/ui/native/src/editor`が現役。旧Dioxus側は履歴参照であり、修正を両側へ複製しない。由来は[imported-edit-helpers.json](imported-edit-helpers.json)。`keyframe_edit.rs`はキー編集の意味を持ち、widgetではない。

## 検証

- protocolのoperation全件がRust CAPABILITIESと一致し、未知opやop上書きを送らない。
- Dock移動・保存復元後も各panelが一つだけ存在する。
- 入力欄がキーを所有し、繰返すビュー操作も通知される。
- レーン操作と慣性は既存のイベント検証を継続する。
- 禁止依存とtransport迂回を実際に挿入した負例で、構成チェックが拒否することを確認済み。

## 次の境界

operation名は型付きになったが、各payloadとsnapshotの完全な型付けは未完。個別パネル内部の整理、Sessionの永続化、配布用ABI管理も残る。C ABIの既存symbolとMethodChannel名は互換性のため維持し、内部クラス名とは区別する。

Stageの初回表示はレイアウト準備後にEditorSessionへrefreshを要求する。UI再接続の順序が変わっても作品や再生位置を書き換えず画像を受け取る。実窓でRectangleの選択・値・画像・frame27の保持を確認した。

正式採用時に、再生中UI再初期化の再接続も検証した。nativeの時計は維持し、main EditorSessionがcadenceを再開する。専用テストは再接続でplay命令を再送しないことと、pauseで停止することを確認する。
