# U3a-2Q-V ordinary Timeline viewport決定

- 日付: 2026-08-02
- 状態: **決定**
- U3a-2Q-V: **DONE**
- 次実装粒: **CU-206 DO**

## 1. 利用者成果

通常製品windowのnative Timelineを、全barをpanel高へ縮めて並べる表示から、一般的な編集Timelineのviewportへ直す。

- track rowは固定モックの既存値を再利用し、**34 logical px**で固定する。
- 時間軸は固定モックの既存grid値を再利用し、**80 logical px / second**を初期scaleとする。
- clip barのstart / duration / bandはDocument投影値のままにし、clip数やwindow高でbarの高さ・時間scaleを変えない。
- 縦scrollはtrack方向、横scrollはtime方向のviewportだけを動かす。Mac trackpadの2軸deltaを分離し、Shift+縦wheelは横scrollへ写す。
- ruler、bar、key、playhead、hit-test、move / trim / snapは同じviewport変換を使う。

## 2. ownerと寿命

既決の`Timeline scroll/zoom = Project session`をvisible rangeにも適用する。現行fixed-Mac Local AlphaではHost coordinatorのprivate presentation stateとし、Document、journal、Undo/Redo、project JSON、plugin契約へ保存しない。fresh Host coordinatorでprojectをopenした時はscrollを原点へ戻す。resizeや同一sessionのDocument publishではscrollを維持し、新しいcontent boundsへclampする。

## 3. 既存契約接続票

- `AUTHORITY`: M3 U3a、U3a-2Z、U3a-2P/Q、CU-110PT、CU-201P/R/E。
- `INTERNAL TARGET`: `ProductTimelineProjection`、`TimelineViewport`、`NativeTimelineRenderer`、既存AppKit local event monitor。
- `OWNER`: clip/key identityと時刻はDocument read-only projection、viewportはProject session、pointer deltaとpresentationはHost transient入力/native module。
- `WRITE ROUTE`: scrollはprivate viewport更新とredrawのみ。Document writerへ到達しない。
- `GAP`: 現行rendererは`available height / row_count`でrowを圧縮し、現行product hit-testも全bandをsurface高へ正規化する。水平viewport入力も未接続。
- `RESOLUTION ROUTE`: 既存projection、local monitor、rendererを`REUSE`し、一つのprivate viewport transformへ接続する。
- `DISPOSITION`: `PASS`。

## 4. 合格oracle

1. 25以上のoverlap clip fixtureでもrow高は34 logical pxのままで、下方trackはclipされ、縦scroll後に表示される。
2. 横scrollでruler、bar、key、playheadが同量移動し、Document上のstart / durationは変わらない。
3. scroll後のbar/key hit、move、trim、snapが表示位置と同じLayerId / KeyframeId / RationalTimeへ到達する。
4. scroll前後でDocument JSON、revision、journal、Undo/Redo件数は不変。
5. hidden Timeline、surface外wheel、scroll境界、非有限deltaは変更0である。
6. 通常製品windowで固定行高と上下左右scrollを観測する。

## 5. 非目標とSTOP

- zoom gesture、semantic zoom段階、minimap、scrollbar widget、track reorder、track header機能、永続viewport形式を追加しない。
- React `TimelineCandidate`、`docs/mocks-ui` runtime import、第二Timeline、公開API、Document/serde/journal/plugin契約を追加しない。
- `product_runtime_adapter.rs`のlifecycle-only境界を拡張しない。既存AppKit local monitor以外の入力frameworkを作らない。
- fixed値や期待値を試験通過のために変更しない。既存move / trim / snap / Easing / selectionの意味が変わる場合はSTOPする。

