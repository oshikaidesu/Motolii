# UIモジュールの責任

依存の正本は[modules.json](modules.json)。`scripts/motolii-ui.sh check`で参照方向と所有を確認する。ファイル数や行数だけを完成条件にしない。

## コアと拡張

再生側は作品の読み取りと時刻を受け、編集UIを知らずに描く。編集側はDocument/Intentを通して書き込みとUndoを所有する。拡張は操作要求または共通の入出力型を使い、Documentの第二の所有者にならない。Documentの読み取り・保存・編集はまだ同じcrate内で、再生だけの最小コアへの分離は未完。現行の分離は静的リンクであり、任意のネイティブプラグインを安全に実行する仕組みではない。

| 所有者 | 実装 | 接続する口 |
|---|---|---|
| 再生時計・音声transport | `motolii-render/src/playback.rs` | `StoreView`の読み取りだけ。編集Document/Intent、Flutter、パネルを受け取らない |
| 作品の保存・編集 | `motolii-doc/src/store/document.rs`、`Intent`、`persist.rs` | Document/Intent、共通の値・効果宣言型 |
| JS実行器・作者用関数 | `motolii/ui/extensions/script` (`motolii-script`) | `Host::command` と読み取り専用 `Host::query`。doc/render/Flutterへの依存なし |
| スクリプトと編集の接続 | `ui/native/src/editor/script.rs` | 操作の検証・Document/Undo・ファイルの再実行。JS VMは所有しない |
| 書き出し・Freezeの仕事 | `motolii/ui/extensions/jobs` (`motolii-jobs`) | Documentスナップショットを受けて処理し、状態とキャンセルを提供。ライブのEditorRuntimeを知らない |
| 同梱効果の組み立て | `motolii-render/src/extensions/mod.rs` | 共通 `Kind` 宣言を既存の描画catalogへ渡す |
| Text Morph | `motolii-render/src/extensions/text.rs` と `text/morph.rs` | 効果パラメータと輪郭を受け、変形後の輪郭を返す。保存コアは実装を知らない |
| WGSL効果 | `motolii-render/vism/` | 既存manifestとstageの入出力 |

**残る分離**: 配置・パス・解析などの組み込み効果は `motolii-doc/src/extensions` に分離したが、レイアウトと効果評価の組み立てはまだdoc内にある。`motolii-doc`全体が最小コアになったとは扱わない。保存形式の変更や、9月17日の未決定の「idと時刻だけ」案の採用は、この移動に含めない。

## UIの所有

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
