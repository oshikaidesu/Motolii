# M3 Graph View headless interaction依存裁定

作成日: 2026-07-22

状態: **決定／isolated fixtureへ限定して依存可**。製品U3a/U2h/U4b、Document、D2、公開API、plugin契約、
WebView/native統合は変更しない。

## 1. 探した穴

native Multi-key Graph View core fixture後に残ったpan、zoom、fit、marquee、additive selection、snap、
pointer cancel、focus loss、AXのうち、Motoliiが独自実装せずheadless部品へ委ねられる範囲を検索した。

[UI runtime責任境界](../ui-runtime-architecture.md)に従い、window、renderer、scene graph、Document、selection正本、
history、Undoを所有する候補は除外した。

## 2. 裁定

| 候補 | 裁定 | 理由 |
|---|---|---|
| `understory_view2d 0.1.0` | **DEPEND / isolated fixture** | MIT OR Apache-2.0。`Viewport1D/2D`だけを提供し、pan/zoom/fit/座標変換、有限値hardeningを所有する。window、input、renderer、scene、selectionを持たない |
| `ui-events 0.3.0` / `ui-events-winit 0.3.0` | **PATTERN / 今回は非依存** | pointer cancel、mouse/touch/pen、winit/Web adapterの先例として適合するが、製品には既に`NormalizedInput`とsafety interruptがある。並行する入力正本を追加しない |
| `AccessKit` | **後続DEPEND候補** | OS AX adapterの正規基盤。ただしbounded semantic projectionと製品window接続はU2h/U3aの正式範囲であり、isolated描画fixtureへ見せかけのAX treeを足さない |
| `blinc_canvas_kit` / `UZOR` / renderer込みviewport | **REJECT** | layout、widget、render context、persistent context、selection等まで所有し、MotoliiのReact/native境界とselection単一正本を侵食する |

`understory_view2d`は二つの`Viewport1D`として使い、time軸とvalue軸の独立zoomを作る。crate型はisolated fixture内に
閉じ、Document、domain公開API、plugin契約へ出さない。

## 3. Motoliiが所有する部分

- stable channel/key IDとactive/selected集合
- single/additive/marqueeの選択意味
- key/tangent hit-test、drag threshold、Transient preview
- frame/key snap候補と優先度
- release exactly-once、Cancel/focus loss時の復元
- D2 command、single writer、Undo（今回のfixtureではtyped test doubleのみ）
- bounded AX projectionの意味とnode予算

この部分を汎用canvas libraryへ委ねると二重selection、独自Undo、surface別stateを生むため、薄いdeterministic
state machineとしてMotolii側に残す。

## 4. 今回埋める穴

isolated Graph View fixtureで次を実装・検証する。

1. headless time/value viewportによるcursor-anchor zoom、pan、Fit All、Fit Selection
2. stable IDのsingle/additive/marquee selection
3. drag thresholdとframe snap test double
4. pointer cancel、focus loss、Escでviewport以外の編集previewを完全復元
5. window/GPUなしの固定入力列test

正式D2/Undo、製品`NormalizedInput` adapter、AccessKit OS tree、edge scroll、100,000 key virtualizationは非目標とする。

## 5. 合格条件

- pan/zoom/fitがDocument相当fixture、semantic commit、Undo countを変えない
- cursor-anchor zoomでanchor下のworld pointが不変
- marquee中semantic commit 0、selection確定はDocument commit 0
- key drag中commit 0、snapを含むrelease 1、Cancel 0
- additive selectionが既存選択を保持し、single selectionは置換する
- non-finite入力でviewport、key、selectionを壊さない
- direct wgpu側は同じheadless projectionだけを読み、GPU readback 0、hot resource生成0を維持

## 6. 実施結果

[G0-9 native Multi-key Graph View spike](../spikes/g0-9-native-graph-view.md)へ実装し、headless 10試験で
cursor-anchor zoom、Fit All / Selection、stable ID marquee / additive selection、4 logical px drag threshold、
0.1 frame snap test double、Cancel復元、non-finite拒否を固定した。

macOS実機ではwheel zoom、Fit All、marquee 4 key選択、Fit Selectionを順に行い、navigation change 3、
selection change 2、semantic commit 0だった。middle panはComputer Use APIがmiddle-button dragを表現できないため
実マウス未確認で、独立headless pan試験とwinit adapter実装までを証跡とする。製品`NormalizedInput`接続、pointer capture lost、
AccessKit、D2/Undoは引き続き停止線の外側である。
