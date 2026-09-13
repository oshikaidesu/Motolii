# Stage 5 — Flutter UIへの正式移行

現在の入口はこの文書と `workspace.json`。2026-09-06の利用者承認によりFlutterを本格採用し、通常のUI・開発・検証の経路をここへ統一した。これは製品の全機能・配布品質の完成宣言ではない。

進行中の引き継ぎ作業は[2026-09-11作業表](claude-handoff-2026-09-11.md)に記録する。

## コンセプトと採用事項

[根本コンセプト](../concept.md) → [UIと操作の採用事項](product-contract.md) → [文書の救出・照合](document-map.md)。保存形式・GPU共有・測定の範囲は[技術境界](technical-boundaries.md)。下記は実装の配置と移行状況であり、製品コンセプトの代わりではない。

## 正本

| 責任 | 場所 |
|---|---|
| 作品・親子・キー・Undo | `motolii/crates/motolii-doc` |
| 評価・GPU描画・書き出し | `motolii/crates/motolii-render` |
| Flutter UI | `motolii/ui/lib` |
| 編集命令・スナップショットの接続 | `motolii/ui/native` |
| macOS窓・共有テクスチャ・OS操作 | `motolii/ui/macos` |
| 起動・検証入口 | `scripts/motolii-ui.sh` |
| Cargoの機械入口 | リポジトリrootの`Cargo.toml`。既定は`motolii/ui/native`、targetは`motolii/target` |

Lottieを基幹モデル、Rerun/re_rendererをビューとする。作品の書き込みはDocument/Intentに一本化する。Flutterは作品の第二の正本を持たず、展開・スクロール・フォーカス・入力下書きを持つ。

## 開発

Rust、Flutter SDK、macOS用のXcodeツールが必要。FlutterはPATH、`FLUTTER_BIN`、または無視対象の `.tools/flutter` で指定する。SDK本体・build・targetをGitへ入れない。

```sh
scripts/motolii-ui.sh check
scripts/motolii-ui.sh native       # 初回・Rust変更時だけ
scripts/motolii-ui.sh dev          # 空の作品で起動
scripts/motolii-ui.sh dev /absolute/path/project.rrd
scripts/motolii-ui.sh reload       # 起動中のDart UIへ反映
```

`dev`のターミナルでも `r` がhot reload。`R`（または`restart-ui`）はUI状態を初期化するため通常の反復に使わない。Rust ABI変更時は保存→native build→アプリ再起動。スクリプトはUI変更を理由にRustを自動再ビルドしない。

テスト入口は `scripts/motolii-ui.sh test`。通常の色・余白変更では一式を繰り返さず、hot reloadと実画面で確認する。操作や所有境界変更は対応するテストだけを先に使う。

## FFmpegの準備

動画・音声の読込や書き出しには、macOSで実行可能な`ffmpeg`と`ffprobe`を用意し、開発プロセスのPATHから見えるようにする。既存インストールを使ってよい。空作品・文字・図形の確認と、メディア機能の検収は分ける。

```sh
command -v ffmpeg
command -v ffprobe
ffmpeg -version
ffprobe -version
```

見つからなければ、利用しているパッケージ管理ツールまたはFFmpeg配布元の手順で導入する。Stage 5の起動スクリプトは無断でダウンロードしない。MP4書き出しでは対象のH.264/AAC encoderが利用可能かも確認する。配布時の同梱・ライセンス・署名は未完工程。

## 移行順序

1. **保全と入口**：変更前ソースと検証版を外部checkpointに保全。manifestに場所とGit HEADの記録先を残す。
2. **core統合**：検証版で修正した子孫削除、親子移動、Undo、Document identityを正本docへ統合。Flutter bridgeは正本doc/renderにpath依存し、coreコピーを持たない。
3. **UI構造の再構成**：現在の検証コードを比較基準に、app/session、共通control、入力/選択/gesture、型付きbridge、各panelへ責任を整理する。パネル名で分割するだけでは終了しない。
4. **正式起動・配布**：開発時の絶対パス依存を除去し、配布時はnative libraryをbundleする。署名・終了保存・再接続・複数窓を検収する。
5. **旧経路退役**：新経路の実操作が揃った後、旧UIコードと適応helperの重複を解消。退役日と参照先だけを残し、正本を増やさない。

## 現在の限界

- nativeの現役編集層は`ui/native/src/editor`。旧Dioxus helperは履歴参照であり、二重に保守しない。由来は適応資産のmanifestに残す。
- 起動・workspace・入力・共通部品・session・通信を[モジュール](modules.md)へ分離済み。個別panel内部、payload/snapshotの完全な型付け、UI session永続化は未完。
- BGRA出力は`shared-bgra-output` featureでFlutter hostだけが選ぶ。旧host既定は維持し、両hostを一括feature統合するbuildを通常路にしない。
- macOS共有GPU経路を検証済み。Windows/Linux、配布bundle・署名の成立を意味しない。
- 回転・拡縮を伴う親変更で、子がアニメーションしている一般ケースの補償は未完。無言でキーを破壊しない。
- Browser分類の詳細、3Dギズモ全機能、細部の操作整合は残作業。仮UIを採用仕様と取り違えない。
- **効果の宿題(2026-09-13)**: フリーズフレーム。描いたコマの cache(feedback の効果の逆再生・戻りスクラブ・編集後の再表示を、辿り直しでなく絵で返す。重さの実測は [plugin-resources.md §6-5](../plugin-resources.md))。編集で捨てる feedback の状態を「変わった層だけ」に(今は全部)。他人の機械で build(fork の pin が `file://`、FFmpeg の場所が `.cargo/config.toml` に直書き)。`motolii-ui.sh test` に `--test-threads=2`(GPU の試験は全並列だと落ちる)。Warp 段の撤去(`turbulent_warp.wgsl` は契約としてだけ残る)と `import.ceil_*` の証拠を棚に出すかの判断。速度欄が負(逆再生)を受けるかの確認(書類の `Speed` は既に受ける)。cage の試験 1 本(窓の作業と一緒に)。一覧の正本は [workspace.json](workspace.json) の `pending`。

## 旧資料との関係

`motolii/src/ui`と`motolii-dx.sh`は旧UIの比較・回帰参照用。`app/`、`next/`、ルート`crates/`等の過去世代も現在の開発入口ではない。検証版は外部に保存したまま、本体へ逆流させない。技術選定の現行判断ではStage 5と利用者の最新指示を優先し、古いdecision行の履歴そのものは消さない。

## 最終入口監査での修正

ルートCargoをStage 5 workspaceへ統合し、旧ルートmanifest/lockをhistoryへ保全した。旧Dioxus hostはCargo workspace対象から除外した。`motolii/Cargo.toml`と旧ソースは履歴参照で、実行が必要なら保存した旧環境から扱う。CIは現行構成と文書だけを検証し、旧appの個人checkoutを実行しない。macOS実機／GPU／Flutter操作の検収はCIの緑から推定しない。required checkやbranch protectionは変更していない。

## 本格採用の確認

3視点の最終確認で入口と現役所有を点検し、再生中UI再初期化のcadence再接続漏れを修正。nativeの時計をリセットせず、main sessionが更新要求を再開する。通常のhot reloadと、UI状態を初期化するrestart-uiは区別する。旧hostを通常Cargo対象から除外し、単一のFlutter経路を正式な基準とした。
