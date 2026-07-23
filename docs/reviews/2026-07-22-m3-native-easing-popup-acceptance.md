# M3 native Easing popup受入契約

作成日: 2026-07-22
状態: **決定**。React起点のnative popupをG0-9の独立受入試験へ追加し、製品U4b接続は既存依存まで停止する。

## 1. 決定

区間Easingを開くGraph iconと現在値の要約だけをReact chromeが所有する。popup内のframe、preset shelf、
user preset library、数値form、説明、value-time curve、Bezier handle、grid、playhead、drag previewは
**一つのnative Rust/wgpu surface**が所有する。popupのwindow生成、anchor、z-order、focus、dismiss、
DPI/layout epochとUser settingsへのpreset永続化はHost coordinatorが所有する。

```text
React Graph icon / current easing summary
  -> OpenCurveEditor { anchor, interval_ref, layout_epoch }
  -> Host coordinator
       -> native popup window + wgpu popup content
       -> revision付きread-only curve / preset projection
       -> User settingsからuser presetを読み、native shelfへ投影
       -> drag中はTransientだけ更新
       -> curve release時だけtyped Document intent
       -> preset save/delete/reorderはtyped User settings intent
  -> D2 single writer / 1 Undo
  -> 同じsnapshotをReact・Preview・Timelineへ再投影
```

Reactモックの`EasingGraphCandidate`はpopup全体を製品DOMとして直接移管するauthorityではない。popupの
幅、枠、余白、情報構成、curve操作のvisual/interaction oracleとして維持する。React/nativeの両側へ
curve、preset thumbnail、selection、Undo、interval identityの正本を置かない。

### 1.1 現行挙動と先例による訂正

固定React source `56c318ed`を実行・監査した結果、`My curves`はbuttonだけでclick後の動作がなく、
basic/advanced thumbnailはcurve値から生成せず固定SVG pathである。Favoriteもcomponent-localな
`useState("Smooth")`だけで、popupの閉じ直しでは残るがpage reload後は`Smooth`へ戻る。表示文言の
`User setting / Undoなし`と実際の永続化は一致していない。

保存可能な製品先例では、[Flow](https://aescripts.com/flow/)がGraph EditorとLibraryを二つの主要componentとして
同じcurve editor製品内に置き、curveのpreset保存、複数library、共有、並べ替えを提供する。
[AccelCurve](https://aescripts.com/accelcurve/)も編集curveをcustom curveとして保存し、group、並べ替え、
rename、group間移動、import/exportを同じ面で扱う。対照的に[Figma](https://help.figma.com/hc/en-us/articles/360051748654-Prototype-easing-and-spring-animations)は
custom Bezierの編集と数値copy/pasteを提供するが、custom easingの保存はできないと明記する。

この差から、user presetを採るならpreset shelfとthumbnailをcurve surfaceから分離しない。thumbnailは保存画像を
curveの別正本にせず、保存した正規curve projectionからnative側で決定的に再生成する。永続値はcurveの型付き値、
名前、順序、group等のUser settingsだけとし、wgpu texture、SVG path、px寸法は保存しない。

## 2. ownership

| 対象 | owner | 禁止 |
|---|---|---|
| Graph icon、現在curveの要約 | React | popup内容、preset thumbnail、curve stateを複製しない |
| popup frame、preset/user library、数値form、説明、curve、grid、handle、stem、playhead、hit-test、drag preview、bounded AX projection | native wgpu + headless interaction | DOM/Canvasを製品popupにせず、独自永続storeを持たない |
| popup anchor、screen clamp、flip、z-order、focus、dismiss、DPI、layout epoch、User settings codec | Host coordinator | Documentへpx/DPI/window座標を保存せず、thumbnail画像を正本として保存しない |
| interval、Interp、revision、Undo | Document/D2 | React/native popupをwriterにしない |

native popupはReact componentではないが、同じsemantic theme token、spacing、radius、stroke、focus語彙を読み、
固定React oracleと同じ情報階層に見える必要がある。「nativeだから別製品に見える」を許容しない。

## 3. isolated spikeの合格条件

G0-9 spikeはDocument、公開API、plugin契約、製品egui shellを変更せず、次を測る。

1. React/WebViewのGraph buttonがlogical anchorと単調`layout_epoch`をHostへ送る
2. Hostがnative popupをanchor近傍へ開き、work area外では上下flipとscreen clampを行う
3. popupがopaque child WebViewより前面に見え、固定React oracleの幅、枠、余白、情報階層をnativeで再現する
4. popup surfaceはCPU pixel readback 0、hot drag中のpipeline/buffer/texture生成0
5. handle drag中のsemantic write 0、releaseでcommit intent 1回
6. `Esc`、popup外click、focus loss、close、capture lossはcommit 0で開始値へ戻す
7. stale revision/layout epoch、重複release、閉じたpopupへの遅延messageはcommit 0
8. keyboard focusと矢印操作を持ち、native curveのbounded accessibility代替をHost treeへ提示できる
9. dark/light、100回open/close、resize、異DPI、第二monitorでanchorとhit targetが破綻しない
10. React oracleとの通常/hover/focus/drag/overshoot/cancel visual matrixを保存する
11. user preset保存で型付きcurve値がUser settingsへ1回だけ書かれ、native shelfのthumbnailが同じcurve projectionから再生成される
12. reopen/restartでuser presetとfavoriteが復元され、rename/reorder/deleteはDocument/Undoを変えない
13. thumbnailのtheme/DPI変更は見た目だけを再生成し、保存値とcurve評価結果を変えない

自動試験はplacement、state machine、exactly-once、Cancel、stale epoch、resource/readback counterを判定する。
z-order、実focus、外click、異DPI、第二monitor、VoiceOver/NVDAは実機審判として分離する。

## 4. spike非目標

- `Interp`、Document schema、D2 command、journal、公開plugin UIの追加
- full U4b、Bounce/Elastic等の高度型、Copy/Pasteの製品接続
- React source assetのproduct ownership移管
- popup windowのplatform抽象を製品公開APIとして確定すること
- native汎用widget toolkit、独自theme system、第二selection/Undo storeの作成

spike内のcommit counterはD2の代替ではなく、release/cancel境界を測るtest doubleである。

## 5. 製品接続の停止線

製品U4bへ接続するには、isolated spike合格に加えて次を必要とする。

- U4aの区間導出と左key outgoing `Interp`の正本
- U2hのselection/focus projection
- curve dragを1 gesture / 1 D2 command / 1 Undoへする個別契約
- React asset直接移管R0と、Easing trigger/native popup oracleのclosure再分類
- macOS/Windowsのz-order、focus、DPI、a11y受入

isolated spikeが動いても、WebView/native製品統合、egui撤去、plugin公開契約の合格には数えない。

## 6. 2026-07-22実施結果

[G0-9 native Easing popup spike](../spikes/g0-9-native-easing-popup.md)でcore縦切りを実施した。
React所有面相当のWebView Graph triggerからHostを経てnative wgpu popupを開き、Bezier handleのrelease 1 commit、preset saveと
favoriteのUser settings test double 2 write、Esc変更ゼロ、再起動後のpreset/favorite復元、readback 0、
hot drag resource生成0を確認した。

固定React oracle `56c318ed`との実機比較では、native popupを論理`510 x 284`へ合わせ、左3列のBezier/
Advanced shelf、中央の縦長graph、右のhandle value cards、semantic color token、通常・選択・drag・保存preset・
favoriteの視覚状態を再現した。Advanced cardsはvisual fixtureに留まり、高度補間の製品意味論は追加していない。

本節は§3のうちcore縦切りの証跡であり、実AX接続、全dismiss/capture経路、異DPI/第二monitor、Windows、
pixel goldenと全visual matrix、正式User Settings/D2接続は未合格である。よって§5の製品接続停止線は維持する。
