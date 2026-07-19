# Alight Motionキーフレームグラフ観察台帳

日付: 2026-07-19
状態: **公式事実確認済み／既決の区間補間をReact fixtureへ反映**

## 目的と証拠

Alight Motion（AM）をキーフレームUXの参考にする時、AMの事実、Motoliiへの採否、現行React fixtureとの差分を分離する。AM画面の一括模倣や、旧HTMLへの機能追加を許可する資料ではない。

一次資料:

- [Alight Motion Help Center — Animation Easing Curves](https://support.alightmotion.com/hc/en-us/articles/10536934703889-Animation-Easing-Curves)（2026-07-19再確認）
- [concept.md](../concept.md)「AM式の高度イージング型を採用」（2026-07-10決定、commit `97d934e`）。Bounce / Elastic / Steps / Elastic Stepsを式やParamDriverでなく区間補間として持つ
- ユーザー撮影スクリーンショット（撮影済みとの申告あり、リポジトリ未取込）。取込規約は[evidence README](evidence/am-keyframe-graph/README.md)
- 現行比較画面: `http://127.0.0.1:5173/#plugin-browser-candidate`

出典等級:

- `公式`: AM公式ヘルプ本文・添付画像で直接確認
- `実機`: 版、OS、操作列を記録したユーザースクリーンショットで確認
- `Motolii判断`: AMを現行Document、Undo、UI境界へ翻訳した採否

未取込スクリーンショットの記憶だけで`実機`を埋めない。

## 観察、採否、現行差分

| ID | AMで確認した操作面 | 出典 | Motolii判断 | `#plugin-browser-candidate`の現状 |
|---|---|---|---|---|
| AM-KG-01 | Curve Editorは専用iconから開く | 公式 Step 2 | 採用。Preview直下のGraph icon | bridge fixtureに存在 |
| AM-KG-02 | playheadを隣接key間へ置くと、その区間のcurveが見える | 公式 Step 3 / Multiple Keyframes | 採用。key単体や任意選択集合でなく1区間を対象 | bridge fixtureに存在 |
| AM-KG-03 | 現在propertyのkeyは白いdiamond＋dark border、他propertyのkeyは薄くborderなし | 公式 Step 1 | 採用。fill、stroke、opacityでcurrent/contextを区別 | **React候補へ反映・操作試験あり** |
| AM-KG-04 | shape presetと2本のhandleでcurveを編集する | 公式 Step 4 | 採用。形状thumbnail＋Bezier handle | bridge fixtureに存在 |
| AM-KG-05 | Xは元時間、Yはremapped time、傾きが速度 | 公式 Step 4 | 意味だけ採用。常設説明を増やさずaccessible descriptionへ | **React候補のaccessible nameへ反映** |
| AM-KG-06 | 隣接key pairごとに別curveを持つ | 公式 Multiple Keyframes | 採用。左keyのoutgoing interpolation | UI試験が不足 |
| AM-KG-07 | Overshootはoverflowで明示ON/OFF | 公式 Overshoot | 採用。導出表示だけで操作入口を隠さない | **React候補へ反映**。範囲外curveのOFFは理由つき拒否 |
| AM-KG-08 | CurveをCopyし別区間へPasteできる | 公式 Copying and Pasting Curves | 採用。CopyはTransient、Paste currentは1 Undo | **React候補へ反映・操作試験あり** |
| AM-KG-09 | 選択propertyの全keyframe pairへPasteでき、他property／layerは変えない | 公式 Copying and Pasting Curves | 対象件数つき`Paste all in current channel`、1 macro／1 Undoとして採用 | **React候補へ反映・操作試験あり** |
| AM-KG-10 | Bounce、Elastic、Cyclic、Random、Steps、Elastic Stepsの高度補間型 | 公式 Advanced Easing Types／Motolii 2026-07-10決定 | 採用済み。すべて既存key pairの**区間補間**であり、適用してもkeyの個数・時刻・値を変えない。CyclicはSine波として識別可能にする | **React候補へ反映・非破壊試験あり** |
| AM-KG-11 | Overshoot OFF時の既存範囲外curve処分 | 公式では未確認 | 黙ったclampは禁止。意味決定までは理由つき拒否を候補にする | 未決 |
| AM-KG-12 | 数値文字列copy、favorite即適用、User curve library | AM当該記事では未確認 | Flow／Motolii側の判断として分離 | AM由来と表記しない |

## 実装境界

`#plugin-browser-candidate`のEasing Graph viewは`src/candidates/EasingGraphCandidate.jsx`へ置換した。旧HTMLは変更せず、`#all-surfaces`等のparity sourceとして維持する。区間導出、Bezier handle更新、Undo表示のfixture adapterはまだlegacy scriptに依存するため、製品状態modelの完了扱いにはしない。

修正順は次とする。

1. `component-map.json`でReact viewとlegacy state adapterの所有を分けて追跡する。
2. current/context key、Overshoot、Copy/Paste/Paste allはReact componentとPlaywright操作試験を同時に維持する。
3. 区間導出、curve編集state、Undo adapterをReact candidate stateへ移した後、同領域のlegacy selector／script依存を削除する。
4. 高度補間型の製品実装は`concept.md`の追加的`Interp` variantへ接続する。React候補は既決の意味を比較するモックであり、未実装の永続schemaやparameter既定値を発明しない。
5. Stepを含む高度補間の適用前後で、keyframeの個数・時刻・値が完全一致し、変更されるのは選択区間の左keyが持つoutgoing interpolationだけであることを試験する。

## U4b受け入れへの追補候補

- current channel keyとcontext-only keyを通常／grayscaleで区別できる。
- Overshoot状態と操作入口を読め、OFFで既存範囲外curveを黙ってclampしない。
- CopyはDocument／Undo不変。Paste currentは1区間だけ、Paste allは現在channelの全区間だけを1 macro／1 Undoで変更する。
- Copy前Paste、対象0、別channel／別layer混入、bulk途中失敗を負例にする。
- 高度補間型の適用でkeyframeを追加・削除・移動せず、key値も変更しない。1区間の補間変更を1 Undoにする。
- Randomのseed、Cyclicの周期等の詳細parameterは、既決の恒久形式が確認できるまでモックから保存形式へ昇格させない。

## スクリーンショット取込後

ユーザー撮影資料を受領したら、evidence manifestへ版、OS、撮影日、操作状態を追記する。公式記事と現行アプリ版が異なる場合は、React fixtureを先に直さず本表の観察と採否を再審査する。
