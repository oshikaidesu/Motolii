# Stage 5 — 保存・描画・検証の境界

製品の意味は[concept](../concept.md)、採用操作は[product-contract](product-contract.md)。ここはコードで確認した技術の現在地を記録する。日付は2026-09-06。

## 保存とモデル

- 現在のプロジェクト保存・読込はRust Documentの`.rrd`経路。Lottieが基幹モデルであることと、ディスク上の形式がLottie JSONであることは同義ではない。
- 新規起動は空のDocument。個人のComparisonファイルを起動時に強制読込しない。既存作品はFile/Openまたは開発コマンドの引数で指定する。
- Undo／Redo、preview／commit／cancel、親子関係、キーはDocumentが所有する。Dart側のMapは読取snapshotであり、保存すべき第二の作品ではない。
- 表示幅・展開・パネル配置などのUI状態と、作品データの保存を分ける。UI session復元は未完であり、hot restartで全UI状態が保持されるとは言わない。
- Lottieの語彙・参照資料は[対応表](../../motolii/reference/lottie-coverage.tsv)と[スキーマ](../../motolii/reference/lottie.schema.json)。過去のcoverage判定をStage 5の実操作完了と見なさない。

## GPU共有経路

```text
Document → 既存Engine / re_renderer
         → IOSurface-backed Metal texture
         → CVPixelBuffer / FlutterTexture
         → Flutterの画面合成
```

- Stageの受け渡しはCPUで画像を読み戻してDartへ送る経路ではない。最終出力の保存領域を共有する。
- Flutterによる画面合成や描画自体が無い、GPU上の全作業がゼロ、という意味ではない。
- `interopCopies: 0`等はbridgeの実装境界に関する表示で、ハードウェアprofilerが全GPU経路を計測した結果ではない。
- 現実装はフレームのGPU完了待ちと共有surfaceの寿命管理を行う。プール化・非同期fence・全負荷下のフレームペーシングが完成したという宣言ではない。
- `shared-bgra-output`はFlutter host用の出力形式。旧hostのRGBA既定と混同しない。Windows/Linuxの共有方式や配布物へのdylib同梱・署名は別の未完工程。
- Stageの滑らかさを、描画結果とは別の滑らかなオーバーレイだけで代用しない。

## 反復の境界

Dart UI変更はhot reload。Rustの編集処理・ABI・macOS接続の変更は必要なbuildと再起動を伴う。UIの変更に紛れて自動で全buildを実行しない。`scripts/motolii-ui.sh`が現在の入口。

Flutter比較検証ではUI変更160msと、Document・選択・再生位置・Texture ID保持を確認した。これは比較検証時の測定で、全変更に対する保証時間ではない。Stage 5移管後は起動・描画・編集とreload入口を確認した。161msの記録はソース差分なしのreassembleであり、変更を含むhot reload測定へ読み替えない。

## 残した検証と適用範囲

| 記録 | 範囲 |
|---|---|
| [workspaceの検証欄](workspace.json) | Stage 5移管時のbuild、23 document unit / 10 transaction / 16 bridge / 5 Flutter操作テスト、実窓。後の変更全ての保証ではない |
| [親子削除](evidence/comparison/validation-parent-ownership.json) | 比較版の実保存データ：28レイヤー一括削除・Undo／Redo、他レイヤー維持 |
| [行移動](evidence/comparison/validation-layer-move.json) | 比較版のnative APIでinside・outdent・Undo・cycle拒否 |
| [アニメ親への移動](evidence/comparison/validation-animated-group-drop.json) | 比較版の実Groupへstatic／animated childを移動し、キー時刻・補間とUndoを確認 |

比較版の記録は原本hash付きで保存し、Stage 5での新規再計測とは区別する。Flutterのpointer-eventテストと人間のトラックパッドの感触も別。見た目の採否・実制作の検収は実画面で行う。

## 未完を制約へ昇格させない

一般の親空間補償、型付きbridge、UI session、全3D操作、配布・他OS・長時間負荷は未完。これらを「意図的に対応しない仕様」と説明しない。詳細は[移行の残作業](README.md)。
