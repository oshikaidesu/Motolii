# 最小コア化 — 2026-09-19進捗・引き継ぎ

この文書は本作業の到達点を固定した記録。作成前のmainは `4b1ba7a87`、作業ツリーはクリーン。以下の実装はmainへ反映済み。責任の現行定義は[modules.md](modules.md)、機械検査の対象は[modules.json](modules.json)。

## 結論

**小さい読み取り・再生基盤に、編集と表現を拡張として載せる方向へ進んだ。ただし最小コア化全体は未完。**

利用者のAviUtl的な意図は、プレイヤー以外の機能を捨てることではなく、機能追加で中央の実装を肥大化させず、責任と接続先を見通せる構造にすること。行数・ファイル数・crate数だけで完成を判定しない。進捗率は、全体の分母を定義できていないため算出しない。

大きな到達点は次の3つ。

1. **作品と閲覧の責任分離**：Document/Intent/UndoとViewerStateを分離し、UIの破棄を作品の終了と切り離した。
2. **編集なしのビルド**：描画とjobsは、編集Document・Intent・Undoを含めずにビルドできる。読み取り用Recordingが実経路に接続された。
3. **効果の計算を外へ**：配置プログラムの入力・出力を定義し、Repeater・Mirror・Blobの配置計算、Motionのサンプル方針を拡張側へ寄せた。

## 定規

- 作品の値・構造・キー・時間軸・UndoはDocumentが唯一の正本。変更はIntent経由。
- 選択・現在見ている時刻・観測カメラ・表示範囲・フォーカスは閲覧側。パネルは第二のDocumentにならない。
- UIの終了、閲覧接続の終了、作品の終了を同一視しない。
- 拡張へは必要な値・読み取り・操作要求の口を渡す。編集権限を丸ごと渡さない。
- 保存形式・既存機能を維持する。未決定の「idと時刻だけ」案は採用していない。
- CPUの常時pollや不変入力の再計算を既定にしない。機能を足すたびに全体を再構築する構造を避ける。
- 静的検査・ビルド成功と実窓の操作成功を区別する。検査の上限を緩めて成功扱いにしない。

## 機能と責任の現在地

以下の実装パスは `motolii/` 以下。

| 機能・責任 | 所有者／接続先 | 到達点・限界 |
|---|---|---|
| 作品の編集・Undo・保存 | `crates/motolii-doc/src/store/document.rs`、Intent、persist | 唯一の書き込み経路を維持。読み取りから編集命令の解釈を除去 |
| 保存作品の読み取り | `store/recording.rs` のRecording | RRD decoderはDocumentと共有。編集・Undoを公開せず、確定した編集位置を保持。一時編集は含めない |
| プレビュー投影・読取cache | 編集側projection → StoreView/read | 入力変更時に読み取り値へ投影。同件数の異なるプレビューも区別。一時値を除くViewでのlayout cache混入を修正 |
| 選択・閲覧時刻・観測・ポインタ | `ui/native/src/viewer.rs` | Documentから分離。製品ではまだ一つの連動ViewerStateを使用 |
| 再生時計・音声transport | `crates/motolii-render/src/playback.rs` | StoreViewを読み、編集Document・Flutter・パネルに依存しない |
| 窓・UIの接続寿命 | Swift WindowAttachment、Dart EditorSession | 古いdetachが後継接続を壊さない。UI破棄で共有作品をclose/pauseしない。共有描画の駆動は主窓 |
| 新しいランタイムへの表示接続 | runtimeEpoch、Stage | 同寸法の作品を開き直した際の描画欠落を修正。接続IDと作品の版は別物 |
| JS実行 | `ui/extensions/script` | Hostのcommand/query経由。JS VMを編集本体から分離 |
| 書き出し・Freeze | `ui/extensions/jobs` | 受付検証後に編集側からRecordingを受け取る。入口・workerともDocument/Intentを直接受け取らない |
| 配置効果 | PlacementProgram／PlacementInput／PlacementOutput | 静的なRust関数をViewへ渡せる。同梱のRepeater・Mirror・Blobも同じ口を使用 |
| Blob Track | 画像解析はrender、配置計算は`extensions/blob.rs` | 解析データを借りて配置を返す。飛び番IDと配列順序を分離。グループの余分な子の表示・無効化時の隠蔽を修正 |
| Motion Blur | `extensions/motion.rs` のSampling | サンプル選択方針を分離。コアは変換を読み、平均合成用の結果へ適用。汎用的な外部Motion登録は未実装 |
| Text Morph・WGSL | render側extensions／vism | Text Morphは保存コアの外。WGSLは既存manifestとstageの契約を利用 |
| UIの構成 | app/workspace/input/foundation/panels/session/bridge | 依存方向を検査。棚の選択で不変の内容・タグ・入力欄を再利用。既存のMaterial祖先エラーを原因修正 |

旧実装1,770ファイルを履歴へ退役させた整理も反映済み。復元先はGit履歴であり、新しい作業で旧UIを再び正本にしない。

## ビルド待ちについて

`motolii-doc`は既定で`editing`を有効にする。読み取り用途は`default-features = false`を指定する。

- 編集なしのdoc、描画、jobsのビルド成功を確認した。
- compiler dep-infoでも、読み取り専用構成から`document.rs`・`persist.rs`・`text_edit.rs`が外れることを確認した。
- 描画・jobsのテスト用fixtureだけはdev-dependencyとして編集機能を使う。
- Cargo featureは加算される。編集アプリ全体のビルドでは編集機能を含む。
- **物理的なcrate分割の完了、ビルド時間短縮の実測、拡張変更で常にコア再ビルド不要になったことは、まだ証明していない。**
- UI変更はhot reload、Rust変更だけnative build。小さい変更で全検査・全構成を毎回ビルドしない。

## 検証結果

結果は各対象の直近の検証時点。全項目を最新mainに対して一括実行した結果ではない。

| 対象 | 確認した結果 |
|---|---|
| 最新のdoc単体／編集transaction／配置・Motion投影 | **129／19／4件成功**。確定値layout cacheの再利用も追加assertionで確認 |
| Recordingの権限制約 | Undoを呼べないcompile-fail test成功。読み込み同値・Undo位置・一時値除外を確認 |
| jobs | **5件成功**。無効な受付でコピーしない、snapshot失敗、保存ゼロとキャンセルを区別 |
| 読み取り専用ビルド | doc成功。描画・jobsも専用入口で成功を確認 |
| Flutter全体 | **177成功・3失敗**。残りは全体layout 2件、作品選択変更時layout 1件 |
| 棚の選択 | build **199→87**、layout **43→20**。既存上限を通過。入力保持・現在の操作先・狭幅を検査 |
| 構成ルール | 通常検査成功。依存ルール9ケースをCIへ追加。明示的editing/default要求を拒否 |
| Freeze GPU検査 | 埋め込みshader構成で形・動画の**2件成功**。画像化変更を外す対照で形の保存失敗を再現 |
| 画像解析GPU検査 | **4成功・1失敗**。Track Overlayの枠線位置。同変更を外した対照でも失敗 |

### 実窓で確認済み

- Dockの赤字`No Material widget found`の解消、タブ操作・ドラッグ。
- Inspectorの別窓表示、別窓を閉じても主窓の再生が継続、UI再起動後の作品・選択・表示保持。
- 同寸法の作品再open後のStage/Camera描画。
- Fonts棚の選択枠・名前・タグ更新と狭幅表示。
- Recording経由のexport：**640×360、30fps、300フレーム、10秒のH.264**。出力画像の矩形も確認。
- 修正後のShape Freeze：**300画像＋300metadata**を生成。別フレームの表示、解除後の表示保持とcache削除を確認。
- 検証後に元の保全作品を開き直し、保全ファイルのハッシュ不変を確認。

これらは全素材・全効果・全操作の検収ではない。

### 配置／Blob／Motion分離後の実窓検収（完了）

`5d5c08e42` のnativeを実窓へ反映して確認した。検証作品は[minimal_core_read_views.js](../../motolii/ui/native/src/editor/script/examples/minimal_core_read_views.js)。

- Blobは Frame 15 でも白い素材へ追従。効果を無効化すると元の子が戻る。
- Edit メニューの Undo で追従配置が復元。
- Motion Blur のサンプルは移動方向の前後両側へ伸びる。中心合わせの方針（[motion.rs:49](../../motolii/crates/motolii-doc/src/extensions/motion.rs:49) の `sample_times` が `-0.5 → +0.5`）どおりで、先例の Alight Motion・AE の既定と一致する。
- 保留していた Cmd+Z は**実装の問題ではなかった**。手では undo・redo・cut・copy・paste・select all すべて動く。反応しなかったのは自動操作側の事情。`MainMenu.xib` の First Responder 宛て key equivalent は無効時に鍵を消費しないため、衝突していない。

## 残件・注意点

1. **最小コアの完成は未証明**：layout・評価・効果列／Group／Motionの組み立てがdoc内に残る。ファイル分割やfeature分離を、完全な物理分離と呼ばない。
2. 独立した選択・閲覧時刻を窓ごとに持つ製品接続は未実装。別窓からの再生開始は実窓未検収。
3. ~~直近のBlob／Motion分離の実窓検収~~ **完了**（後述）。
4. Flutterのlayout上限超過3件。上限は据え置き。
5. Track Overlayの枠線位置の既存失敗。今回のFreeze修正を外しても再現した。
6. debug用shader監視のFSEvents開始待ち。devを閉じた単独テストでも再現。スタックは`FileServer::watch → notify → FSEventStreamStart`で、GPU競合と断定しない。
7. `owned_budget`未合格：compute pipeline **2/0**、shader module **2/0**、bind-group layout **2/0**、render pass **4/3**、GPU無期限待機 **18/4**、naga parser **4/3**（実数/上限）。上限を引き上げていない。
8. 第三者コードの動的ロード・権限・sandbox・障害隔離、GUI登録、配布ABI管理は未実装。今回の関数型や接続IDはセキュリティ境界ではない。
9. payload/snapshotの完全な型付け、Session永続化、全機能の検収も残る。既存監査[#481](https://github.com/oshikaidesu/Motolii/issues/481)全件を解決したものではない。

## 再開時の順序

1. `motolii/AGENTS.md`、この文書、modules.json、Git状態を確認。未コミット変更を保護する。
2. 残るcore内の評価組み立てを依存単位で切る。新しい汎用registryやsandboxを先回りして作らない。
3. layout 3件、Track Overlay、監視待ち、実装量上限を、対象と根拠を分けて処理する。
4. 完了時は目的ごとの証拠を確認し、未検証を未完として残す。ビルド速度は再ビルド範囲を確認してから必要な測定を行う。

軽い検査・対象別の入口：

```sh
python3 scripts/check-stage5.py --self-test
scripts/motolii-ui.sh check
git diff --check
cargo test -p motolii-doc --test placement_programs
cargo test -p motolii-doc --lib --test edit_transactions
scripts/motolii-ui.sh check-read-only
```

Rustの依存環境は[CONTRIBUTING](../../CONTRIBUTING.md)と開発スクリプトに従う。`IS_IN_RERUN_WORKSPACE=no`のGPUテストは上流の埋め込みshader構成であり、debug監視を修復した証拠ではない。監視待ちを隠すため常用設定を書き換えない。

## 主要な追跡点

文書化後の追記：配置programの`needs_position`を表示の識別値へ含める修正。修正前は同一関数の入力条件を変えても識別値が衝突することを再現し、修正後は位置入力による配置の変化と識別の分離を確認した。対象の投影3テスト成功。GPU・実窓の追加検証は行っていない。

続く追記：一時編集を読むViewと除外するViewの表示識別を分離。除外した一時値・プレビューの更新では確定値Viewの識別を変えず、確定値layoutの同一cacheを再利用する。修正前の識別衝突を再現し、修正後の値・識別・cache再利用を検査済み。保存形式は変更していない。

| commit | 内容 |
|---|---|
| `7ca2d8436` | ViewerStateをDocumentから分離 |
| `3704f6f8c` | UI接続寿命・共有描画・runtimeEpoch |
| `37dc3b8bf` | 棚の選択と不変内容の再構築を分離 |
| `11371b29c` | 確定値Viewへの一時layout cache混入を修正 |
| `9de838126` | 読み取り専用Recording |
| `d36608692` | 編集なしのdoc／描画ビルド |
| `9b835de95` | jobs入口から編集Document依存を除去 |
| `858368ab3` | 素材画像取得・Freeze保存ゼロの修正 |
| `de7c0372d`・`7132f8aa4` | 配置programとBlobの分離 |
| `e78290e09` | Motionのサンプル方針を分離 |
| `4b1ba7a87` | 読み取り専用依存の明示feature抜け道を検査 |
| `524de0ea7` | scrub中のヘッドをtimeline全体の再構築から外す |
