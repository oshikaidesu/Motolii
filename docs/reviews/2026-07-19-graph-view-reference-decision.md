# Graph View参照・比較決定

日付: 2026-07-19
状態: **Graph View採択／React prototype比較中**

## 決定

Motoliiは、実時間×実値で複数key・複数区間を俯瞰して編集する独立`Graph View`を持つ。既存のFlow / Alight Motion型操作面は`Interval Easing Editor`へ改称し、現在区間の正規化補間を素早く編集するショートカットとして併存させる。

両者を同じ名前、同じ状態所有、同じ表示範囲として扱わない。

| 操作面 | 横軸 / 縦軸 | 対象 | 主用途 |
|---|---|---|---|
| Interval Easing Editor | 区間内の正規化時間`u` / remap進行 | 現在の隣接key 1区間 | preset、区間補間型、Bezier 4値、Overshoot、Copy/Paste |
| Graph View | compositionの実時間 / parameterの実値 | focus中channelの複数key・複数区間。context channelは参照表示 | keyの時刻・値、区間形状、複数選択、全体のリズムと連続性 |

## 参照先と採る範囲

### Apple Motion — 全体構成

一次資料:

- [Display the Keyframe Editor](https://support.apple.com/en-ca/guide/motion/motn14749268/mac)
- [Keyframe Editor controls](https://support.apple.com/en-mide/guide/motion/motn147486cf/mac)

採る候補:

- 左のparameter listと右のgraph area
- Timelineと同じ時刻系、playhead、marker
- `Fit Visible Curves`を明示操作にする
- 編集前curveを薄く重ねるsnapshot
- 選択key群のtransform

採らないもの:

- v1でのSketch Keyframes
- behavior、extrapolation、audio waveformをGraph Viewの必須意味にすること

### Cinema 4D — tangent直接操作

一次資料:

- [F-Curve Keys](https://help.maxon.net/c4d/en-us/Content/html/10616.html)
- [Keys Area / F-Curve Mode](https://help.maxon.net/c4d/2024/en-us/Content/html/10608.html)
- [Add Key At](https://help.maxon.net/c4d/en-us/Content/html/11054.html)

採る候補:

- `Shift`で左右tangentを一時分離
- `Ctrl/Cmd`で角度を保ち長さだけ変更
- `Alt`で長さを保ち角度だけ変更
- 複数keyの同時編集
- curve上へkeyを追加しても形状を変えない

### Maya — tangent状態語彙

一次資料:

- [Graph Editor](https://help.autodesk.com/view/MAYAUL/2023/ENU/?guid=GUID-6D38EAEA-6032-471E-BD0E-54A74D4443C0)
- [Tangents menu](https://help.autodesk.com/cloudhelp/2026/ENU/Maya-Animation/files/GUID-43A4FE2C-4863-4EA6-B6AE-6D2B6757F6C7.htm)

採る候補はIn / Out、Break / Unify、angle / length lock、Auto Clamped相当、buffer curveによる比較。Mayaのmenu量と全tangent種別を初期面へ常設しない。

### Blender — 表示navigation

一次資料:

- [F-Curve Introduction](https://docs.blender.org/manual/en/4.2/editors/graph_editor/fcurves/introduction.html)
- [F-Curve Properties](https://docs.blender.org/manual/en/4.5/editors/graph_editor/fcurves/properties.html)

採る候補はFrame Selected、Normalizeの表示変換、Ghost Curve、channel filter。`Continuous Acceleration`のように離れた区間まで変更が伝播する自動平滑化は採らない。

## Motoliiで固定する差分

1. Graph Viewを開く、pan / zoomする、channelをfilterする、snapshotを表示する操作はDocument・Undo不変。
2. 表示範囲はdrag中に自動変更しない。初回frameと`Frame Selected`だけが明示的に表示範囲を変える。
3. keyまたはhandle dragはTransient preview、releaseで1 gesture / 1 Undo、`Esc`またはcapture lossで変更ゼロ。
4. curve上へのkey追加は既存curveをde Casteljau分割し、追加直前と同じ曲線を保つ。
5. 既定の区間は左右独立のoutgoing interpolationを正本とし、連動tangentは両隣区間を同時変更する明示操作として扱う。UI都合で恒久tangent型を追加しない。
6. Graph Viewのpx、DPI、pan / zoom、display normalization、snapshotはDocument・評価・plugin契約へ入れない。

## Prototypeの停止線

- React fixtureは既存key / interpolationの投影を比較するだけで、製品`GraphViewState`、D2 command、egui型、保存形式を定義しない。
- absolute tangent field、curve snapshot、panel layoutをDocumentへ追加しない。
- 複数点専用補間、MultiEase互換preset、Sketch Keyframesは、単一Graph Viewで解けない制作操作が実証されるまで追加しない。
- Graph Viewから隣接区間を黙って再平滑化しない。

## Prototype審判

1. focus中channelが形・太さ・key表示でcontext curveと区別できる。
2. `Frame Selected`以外ではhandle drag中もview rangeが変わらない。
3. snapshot表示は編集前curveを残し、Document / Undoを変えない。
4. curve上へのkey追加前後で固定sample点の形状差が許容誤差内。
5. `Shift`、`Ctrl/Cmd`、`Alt`のtangent拘束と`Esc`取消を再現できる。
6. Interval Easing EditorとGraph Viewを別の入口・accessible nameで識別できる。

## React比較結果

- `#plugin-browser-candidate`ではGraph ViewをTimelineと同じdock内の排他的なView切替として統合する。切替で外枠寸法と時間文脈を変えず、独立windowや自動resizeを作らない。
- `#graph-view-candidate`はcurve操作の狭い再現fixtureとして残す。製品screen、第二のGraph実装、別状態所有者にはしない。
- Graph固有の黄・青・紫paletteは持たず、既存mockのsurface、border、text、active roleを使う。primary / contextの識別は色相だけに依存せず、太さ、key表示、dash、opacityを併用する。
- dockの実ピクセル寸法とSVG viewBoxを一致させ、時間・値rangeを固定したまま表示座標だけを再投影する。横長dockへ固定viewBoxを非等方scaleしてkey、handle、curveを変形させない。
- `graph-view-model.js`へ時間／値と表示座標の変換、Bezier path、de Casteljau分割、handle拘束を純粋関数として分離する。これは将来のtool／preset／script adapterを検討できる内部seamであり、現時点の公開plugin APIではない。React、SVG、egui型やDocument直接書き換えをこの境界へ持ち込まない。
