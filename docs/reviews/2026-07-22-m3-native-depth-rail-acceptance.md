# M3 native Depth Rail受入契約

作成日: 2026-07-22

状態: **決定／isolated fixture実装可**。M5 P2RのDocument意味、D2、Auto Key、Preserve Appearanceは実装しない。

## 1. oracleと責任

固定React `TimelineCandidate.jsx`（commit `56c318ed`）のDepth Railを情報構造・外観・操作oracleとし、native
Timelineの同じ一次元viewport / hit-test / selection文法へ投影する。正本の意味はM5 P2Rであり、React fixture値やpxを
Documentへ焼かない。

初回native fixtureは次を再現する。

- `DEPTH` header、`ROOT` / `ROOT / Group` scope、Edit-Space Z readout
- `-.50 / -.25 / 0 / +.25 / +.50`の線形axis
- 同一Zを扇状展開しない`0 × N` stackとstable-ID focus
- Camera Depthを編集markerと混ぜないread-only gutter marker
- Layer Order Distributeのfar/near range preview、Reverse、Apply、Cancel
- wheel zoom、pan、Fit All / SelectionのDocument外viewport操作

## 2. 状態所有

- Host fixture: stable object ID、parent、authoring order、選択、評価済み`position.z`
- headless Depth kernel: viewport、hit-test、stack projection、distribution preview、gesture token
- native renderer: grid、marker、range band、textのread-only scene
- Document相当fixture: Apply / marker release時だけ1 semantic commit。preview、Cancel、selection、navigationは0

Depth専用channel、暗黙group、expression、null、camera-space XYZ補正、独自Undoを作らない。mixed-parent distributionは
typed rejectionとし、root objectとGroup childを同じ配布集合へ混ぜない。

## 3. 合格条件

- 4 objectがZ=0でもmarker 1個と`0 × 4`で識別でき、hover/selectionで値を散らさない
- focus変更はstable IDだけを変え、Railを自動openせずDocument変更0
- same-parent選択だけをfar/nearへauthoring orderで等間隔previewし、Reverseは割当だけを反転
- preview中semantic write 0、Apply 1、duplicate Apply 0、Cancel 0
- pan/zoom/fitでZ値、selection、commit不変。非有限入力を拒否
- scope切替でGroup自身とchild markerを混在させない
- GPU readback 0、hot gesture resource生成0

正式D2 command、Auto Key、Preserve Appearance、camera診断、遮蔽policy、100 layer、AX、Windowsは後続である。
