# Ease — 形・意味・動きを一緒に読む

2026-09-09、利用者が[操作できる比較案](../reviews/assets/2026-09-09-ease-readable.html)を「それで行こう」と採用。小窓への収容を理由にカーブを縮小・横伸ばししない。比較案は見た目の定規であり、その仮の補間式を製品へ移植しない。

## 採用した配置

- 上段は正方形の編集グラフと、カーブ名・短い動作説明・一往復しない動きのプレビュー・数値調整。
- 下段は標準9種のカーブと省略に頼らない名前。通常幅では3列。カーブの描画面を44px角以上に保ち、名前は必要なら2行へ折り返す。
- 下端はCopy／Save、対象の状態、Overshoot、Apply。狭幅では操作を折り返す。高さ不足には縦スクロールで対応し、意味や見分ける面積を削らない。
- 淡いミントのグラフ面・選択面と濃い描線を共通化する。色だけで選択を伝えず、形・名前・選択状態も維持する。

## 意味と書き込み

`easeModel`とsnapshotの`interp`がDocumentの補間器から返す` samples / handles `を使う。モーション表示・グラフ・Sequenceの点は同じサンプル列を読む。Holdの不連続とOvershootを保ち、Flutterに補間式を増やさない。

プリセットのhover／キーボード比較とPreview motionはUI内だけの一回再生。書類、選択キー、保存カーブ、Stageの再生位置を変更しない。離脱・フォーカス喪失では比較を終了する。Reduce Motionでは経過アニメーションを省略する。

プリセットクリック、Apply、数値確定、ハンドルreleaseは従来の`_model → _commit → ease / sequence`へ通す。キー区間への反映はnativeのDocument/Intentが所有する。ドラッグ中の下書き、Escape／focus lossの取消、Undoの単位を保持する。対象なしの作業カーブと保存プリセットはDesk設定に残る。

## 検証

- `ui/test/ease_interaction_test.dart`: nativeサンプルのHold／Overshoot、hover時の書き込み0、元の曲線への復帰、releaseで1回、Escape／focus loss取消、240px幅での到達性、サムネイルの大きさ・名前の折返し、Reduce Motion。
- `ui/test/desk_workspace_test.dart`: 対象なしの保存と道具の切替後の保持。
- 実窓の検収記録は確認後に追記する。

実装は`motolii/ui/lib/panels/ease_desk.dart`。既存のカーブ型・パラメータ・評価器を増減しない。
