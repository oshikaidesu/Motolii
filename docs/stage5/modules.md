# UIモジュールの責任

依存の正本は[modules.json](modules.json)。`scripts/motolii-ui.sh check`で参照方向と所有を確認する。ファイル数や行数だけを完成条件にしない。

## コアと拡張

再生側は作品の読み取りと時刻を受け、編集UIを知らずに描く。編集側はDocument/Intentを通して書き込みとUndoを所有する。拡張は操作要求または共通の入出力型を使い、Documentの第二の所有者にならない。読み取り・保存・編集は同じcrate内だが、`editing` featureを外すとDocument/Intent・Undo・保存・編集fixtureをコンパイルしない。描画の通常依存はこの読み取り専用構成で、編集用fixtureはdev-dependencyだけが有効にする。評価・組み込み効果の分離は未完。現行の分離は静的リンクであり、任意のネイティブプラグインを安全に実行する仕組みではない。

| 所有者 | 実装 | 接続する口 |
|---|---|---|
| 再生時計・音声transport | `motolii-render/src/playback.rs` | `StoreView`の読み取りだけ。編集Document/Intent、Flutter、パネルを受け取らない |
| 作品の保存・編集 | `motolii-doc/src/store/document.rs`、`Intent`、`persist.rs` | Document/Intent、共通の値・効果宣言型 |
| 保存作品・処理用の読み取り所有 | `motolii-doc/src/store/recording.rs` | `Recording::load`と`view`。編集・Undoを公開せず、RRD decoderはDocumentと共有。Documentからの変換は確定した編集位置を保持し、一時編集を含めない |
| 作品の読み取り・読取cache | `motolii-doc/src/store/view.rs`、`read.rs` | 編集命令を解釈せず、記録と読み取り用のプレビュー値を参照する。命令から値への変換・検証は編集側の責任 |
| 共通ID・版情報 | `motolii-doc/src/store/ids.rs`、`read.rs` | 読む側と書く側が共有する値の型。編集実装の子モジュールには置かない |
| 閲覧状態 | `ui/native/src/viewer.rs` | 選択・閲覧時刻・再生時計・観測カメラ・表示範囲・ポインタなど。作品を所有せず、読み取りViewから初期化する。主選択は一覧の末尾から導出し二重保存しない |
| JS実行器・作者用関数 | `motolii/ui/extensions/script` (`motolii-script`) | `Host::command` と読み取り専用 `Host::query`。doc/render/Flutterへの依存なし |
| スクリプトと編集の接続 | `ui/native/src/editor/script.rs` | 操作の検証・Document/Undo・ファイルの再実行。JS VMは所有しない |
| 書き出し・Freezeの仕事 | `motolii/ui/extensions/jobs` (`motolii-jobs`) | 受付条件を検証後、呼出し側の一回限りの関数からRecordingを受け取る。作品コピーの作成はnative editorが所有し、jobsはDocument/Intentに依存しない。状態とキャンセルを提供する |
| 同梱効果の組み立て | `motolii-render/src/extensions/mod.rs` | 共通 `Kind` 宣言を既存の描画catalogへ渡す |
| Text Morph | `motolii-render/src/extensions/text.rs` と `text/morph.rs` | 効果パラメータと輪郭を受け、変形後の輪郭を返す。保存コアは実装を知らない |
| WGSL効果 | `motolii-render/vism/` | 既存manifestとstageの入出力 |

**残る分離**: 配置・パス・解析などの組み込み効果は `motolii-doc/src/extensions` に分離したが、レイアウトと効果評価の組み立てはまだdoc内にある。`motolii-doc`全体が最小コアになったとは扱わない。保存形式の変更や、9月17日の未決定の「idと時刻だけ」案の採用は、この移動に含めない。

Recordingの検収: doc単体128件、編集transaction19件、Undoを呼べないcompile-fail testが成功し、jobsを含むbuild checkが成功。RRDからの同値読み込み、Undo後の確定位置、一時値を除く変換、workerへ移せるSendを確認した。新しい型を使った実窓でのexport/Freeze完走は未検収。型の権限分離であり、編集実装をビルド依存から外すcrate分離ではない。

ビルド境界の追加検収: `cargo check -p motolii-doc --no-default-features`と`cargo check -p motolii-render --lib`が成功。文字描画で使う書記素・style IDの読み取りは`text_read`へ分け、編集helperは同じ関数を再公開する。Cargo featureは加算されるため、編集アプリ全体ではeditingを含む。物理的なcrate分割・ビルド時間短縮の実証とは区別する。専用入口は`scripts/motolii-ui.sh check-read-only`。

jobs入口の追加検収: native editor接続check、jobsの受付拒否・snapshot失敗4テスト、`check-read-only`での描画/jobsビルドが成功。jobs内にDocument依存を仮挿入すると構成検査が拒否した。compiler dep-infoでも読み取り専用docの入力からdocument/persist/text_editが外れ、編集構成には含まれることを確認。実窓でのexport/Freeze完走と時間短縮の実測は未検収。

実窓の追加検収: Recording経由のexportは640×360・30fps・300フレームのH.264を生成し、出力画像の白い矩形を確認。平面ShapeのFreezeは画像を持たない描画用meshを受けて保存ゼロでも完了していたため、素材画像取得の共通口で画像化を要求し、jobsは保存ゼロを失敗にする。jobs5テスト、埋め込みshader構成のFreeze2テストが成功し、画像化変更を外す対照ではShapeの保存失敗を再現。通常の開発構成の実窓でも300画像＋300metadataの生成、Frame 1での表示、解除後の表示保持とcache削除を確認し、元の保全作品へ戻した。解析5テスト中Track Overlayの枠線位置1件は失敗し、画像化変更を外した対照でも同じ失敗を確認。debug shader監視はdev停止後の単独テストでもFSEvents開始待ちを再現したため、実窓の成功とは別の未解消事項として扱う。全素材・全効果のFreeze対応を証明したものではない。

`owned_budget`は未合格: compute pipeline 2/0、shader module 2/0、bind-group layout 2/0、render pass 4/3、GPU無期限待機18/4、naga parser 4/3(実測/上限)。Recording変更はこれらを追加していない。上限は据え置き、描画側の残件として扱う。

プレビューは編集入力の変更時に一度、値・時刻・属性・形・文字の読み取り用データへ投影する。ViewがIntentの列を毎回走査する経路は持たない。同じ件数のプレビューでも値が変われば表示の版は変わり、取消・失敗時の復元・UndoはDocumentが所有する。

現行アプリは一つのViewerStateを使い、従来の窓間連動を保つ。独立したViewerStateを複数作っても作品・Undoが変化しない境界を試験するが、窓ごとに独立した選択・閲覧時刻へ接続する製品機能は未実装。Flutter固有のフォーカスと配置はFlutter側が持つ。ドキュメント時間軸は作品の値、現在どの時刻を見るかは閲覧状態である。

UI接続の寿命は `macos/Runner/WindowAttachment.swift` が識別する。`EditorSession.dispose` は自分の接続・テクスチャ・通知だけを外し、共有Documentのcloseや再生pauseを送らない。古い接続の遅れたdetachと通知完了は、新しい接続に作用しない。Documentの終了はアプリ側の明示的な終了処理が所有する。接続識別子は寿命管理用であり、第三者コードの権限制御やサンドボックスではない。

共有再生の描画駆動は主窓だけが持ち、別窓は再生操作を要求する。ランタイムの交換は `runtimeEpoch` で通知し、Stageの表示窓とsurfaceのキャッシュを無効化する。作品の版やUI接続識別子とは別の寿命である。新規レイヤーの投影設定は、設定復元・ランタイム交換・投影設定の変更で同期し、無関係なDeskプリセットの保存では送信しない。

2026-09-19の検収: Swiftの接続世代チェック、Flutterの破棄・再接続・描画所有・投影設定同期の関連10テスト、静的解析が成功。全Flutterテストは177成功・3失敗(全体レイアウト2件、作品選択変更時のレイアウト1件)で未完。棚の選択は内容・タグ入力を再利用し、500項目のクリック検査はbuild 87・layout 20で既存上限内。入力保持と操作先の更新、狭幅のタグ欄を回帰試験したが、この棚変更の実窓操作は未検収。Easeの寸法検査は曲線描画を特定し、Colorsは棚への誘導・Documentへの追加位置要求・識別済みstopのプロパティ編集を検査する。実窓ではInspectorの別窓表示、閉じた後の再生継続(1639→1947)、UI再起動後の選択・値・Frame 3・画像の保持、同サイズ作品の再open後のStage/Camera画像を確認した。別窓からの再生開始は実窓未検収。タブの `No Material widget found` は共通操作部品への置換で解消し、実窓とテストでタブ操作を確認。これは全機能や性能の検証ではない。

棚変更の追加実窓検収: FontsのAl Bayan→Al Nileの選択で枠・名前・タグ欄が更新され、狭幅でoverflowしないことを確認。Create表示へ戻し、作品のレイヤーは変更していない。入力途中の保持・追加削除先の更新はwidget testで確認した範囲であり、実窓でのタグ保存は未検収。

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
