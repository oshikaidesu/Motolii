# M3 native Multi-key Graph View受入契約

作成日: 2026-07-22

状態: **決定／isolated fixture実装可**。製品Document、D2、公開API、plugin契約、WebView/native製品統合は変更しない。

## 1. 決定

Multi-key Graph Viewはnative Rust/wgpu surfaceが所有する。UIの操作トポロジーはBlender Graph Editorへ
意図的に寄せ、既存DCC利用者が説明なしにpan、zoom、channel選択、key/handle編集を始められることを優先する。

「Blender-like」は次を意味する。

- 左channel list、上header、右graph canvas、下statusという既知の配置
- 時間X／値Yのgrid、playhead、複数curve、key、左右tangent handle
- neutral dark面、選択の暖色、playheadの寒色という役割語彙
- pan、zoom、marquee、frame selected、additive selection、key/handle drag、Cancelの既知操作

Motoliiの固定React `GraphViewCandidate`は表示channel、情報密度、Motolii固有の見た目を決めるoracleである。
Blenderはnavigation、selection、F-Curve編集の**操作先例**であり、Motoliiの仕様正本ではない。

## 2. sourceとlicense停止線

Blender公式mirrorの`source/blender/editors/space_graph/`には`graph_draw.cc`、`graph_edit.cc`、
`graph_select.cc`、`graph_view.cc`等が公開されている。Blender全体はGPLで、MotoliiはMIT OR Apache-2.0である。

したがって次を禁止する。

- Blender source、shader、icon、定数表、関数構造のcopy、翻訳、port、vendor
- Blender内部型、operator、FCurve storage、Undo、keymapをMotoliiへ持ち込む
- Blenderとpixel一致することをgoldenにする
- 「公開されている」ことをpermissive reuse可能と解釈する

許すのは公式Manual、実機挙動、公開画面から得られる操作語彙と責任分割を先例として使い、Motolii fixture、
headless layout/hit-test、D2 single writerの上へ独自実装することだけである。Blender名・logo・iconは製品UIへ出さない。

## 3. isolated fixtureの範囲

第1fixtureは固定3 channel、9 key前後を使い、次をnative windowへ描く。

1. header: `View / Select / Channel / Key`、pivot、snap、normalizeの席
2. channel list: object/parameter名、color、visibility、lock、active row
3. graph: major/minor grid、time/value label、3 curve、key、selected key、tangent stem/handle、playhead
4. status: active channel、selected count、time/value、操作ヒント
5. pointer: keyまたはhandleのdrag preview、release exactly-once、Esc cancel

初回はBlender全機能、F-Curve modifier、driver、ghost curve、2D cursor、popoverの実装を目標にしない。
席だけを置くcontrolはdisabledまたはfixture表示とし、動くふりをさせない。

## 4. 合格条件

- direct wgpu primitive batch + GPU textで表示し、CPU pixel readback 0
- hot drag中のpipeline/buffer/texture生成0
- React oracleの3 channel/key fixtureを同じstable IDで投影
- Blender利用者がchannel list、graph、selected key/handle、playheadを読む前に識別できる
- drag中semantic commit 0、release 1、重複release 0、Esc 0
- Xは時間順を破壊せず、Yとtangentはfixtureの有限範囲へclamp
- pan/zoomはProject session相当のtest doubleで、Document/Undo不変
- bounded AX projectionを別modelとして提示し、key数比例のOS nodeを常設しない
- 固定React oracle、Blender Manual、native実画面の三者比較結果を記録

## 5. 製品接続前の残件

- U4a/U4eの正式channel/key snapshotとU2h selection/focus
- D2のkey/tangent command、gesture merge、Undo
- Timeline/Easing/Inspectorと同じselection・curve正本
- theme token、IME/AX、異DPI、Windows、surface lost
- Graph Viewのdock/detach lifecycle

isolated fixtureの合格だけで製品統合やegui撤去を許可しない。

## 6. 実施結果

[G0-9 native Multi-key Graph View spike](../spikes/g0-9-native-graph-view.md)でcore fixtureを実施した。
Apple M4 / Metal上のdirect wgpu描画、固定Reactの3 channel / 9 key投影、key/handle drag、release exactly-once、
Esc cancel、時間順clamp、headless pan/zoom/fit、stable ID marquee/additive selection、drag threshold、frame snap、
readback 0、hot drag resource生成0まで合格した。正式AX model、D2/Undo、製品`NormalizedInput`接続、dock接続は
未実装なので、本節は製品統合の停止線を解除しない。
