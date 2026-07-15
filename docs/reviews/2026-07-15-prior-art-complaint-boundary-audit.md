# 先例収束 / 日曜大工境界監査（2026-07-15）

ステータス: **調査第一陣**。対象はRelative Move、Stage外表示、RoD/RoI、Explicit Shared Effect、Duplicator/Stable ID、Element Domain、Undo境界。目的は「有名製品にある」を採用理由にせず、複数の成熟実装で意味が収束した契約と、利用者が回避工作している可変領域を分けることにある。

## 1. 判定方法

「文句が見つからない」は証明不能なので、次の3条件が揃う部分だけを固定候補とする。

1. 複数の独立した成熟実装または標準で同じ意味が現れる
2. UIを交換しても保存・評価・Undo・identityの意味が残る
3. golden、roundtrip、拒否testで機械判定できる

逆に、公式手順自身がNull、Group、Crop、Grow Bounds、Precompose、Expression、Copy/Pasteを要求する箇所、または公開script/plugin市場で同じ補修が反復される箇所は「日曜大工帯」とする。ここではUIや合成規則を早期凍結しない。

## 2. 結論表

| 領域 | 収束している固定候補 | 日曜大工帯 / 未収束 | Motoliiでの処分 |
|---|---|---|---|
| Relative Move | 選択した複数keyへ同じ値差分を与え、補間・時刻を変えない | 全key選択までの手数、modifier、HUD、motion path表示 | D2 one-shot差分だけ固定。modifier+dragは交換可能UI |
| Undo | Document変更をCommand化し、複数objectの1操作をmacro 1件でUndo | drag中previewの外観、merge時間、物理入力 | 1 gesture=1 history、Cancel=0だけ固定 |
| Stage外表示 | 編集用pasteboard/worldとFinal frameを分離 | 枠外をoutline/full pixel/opacityのどれで見せるか | Final不変と選択可能性だけ固定。scrimは可変 |
| RoD/RoI | 出力可能領域と要求出力に必要な入力領域を分離し、無限をHostでclamp | tight bounds、bbox警告、手動Crop/Grow Bounds、予算 | `Finite/Infinite/Unknown`+保守fallbackを固定。精密化/UIは可変 |
| Shared Effect | 共有parameter definitionを複数の独立したordered stack位置から参照する | 調整layerの「下全部」、precomp、include/exclude式、接続線routing | Definition/Useとstack順を固定候補。線の見せ方は可変 |
| Typed connection | out→in、型不一致拒否、1出力の複数利用 | 1入力制限でkeyを上書き、任意合成規則 | 型付きRefを固定。万能なattribute入力合成は採らない |
| Instance identity | indexとidentityを分離し、IDを乱数・motion blur・overrideへ使う | Distribution編集後に「同じ個体」をどう対応付けるか | `InstanceId != index`を固定。slot-key規則はP0Iまで仮説 |
| Seed | 明示seed+IDで再現可能なvariationを作る | Randomizeボタン、seed UI、アルゴリズム更新 | seed/PRNG version/goldenを固定。UIは可変 |
| Element Domain | point/vertex/primitive/detail等、domainごとに要素寿命と属性を持つ | 全domainを単一index/schema/selector式へ畳む | domain別identity+typed selector protocolまで。単一schema化しない |
| Duplicator | prototype入力、distribution、per-instance channel、context、GPU/packed instance | source transform継承、nested UI、materialize、具体Effectorカタログ | Host境界を固定候補。Cavalry UI/挙動を丸写ししない |

## 3. Relative Move: 演算は枯れている、導線だけ改善対象

After Effects公式は、Position property名を選んで全keyを選択し、Composition上のkeyをdragするとmotion path全体へ同じ差分が入ると説明している。複数keyのgraphical editも旧値との差を全選択keyへ相対適用する。したがってMotoliiのone-shotは新しいanimation意味ではなく、既存の二段操作「全key選択→drag」を一gestureへ圧縮するUIである。

- 固定: 対象snapshot、同じEdit-Space差分、時刻/補間/接線不変、1 Undo、Cancel 0
- 可変: modifierの物理key、HUD、ghost、motion pathの長さ、通常dragとの識別方法
- 禁止: Relative MoveのためのNull、parent、常設offset、Animation Layer生成

先例: [After Effects Motion Paths](https://helpx.adobe.com/ca/after-effects/using/assorted-animation-tools.html)、[After Effects keyframe editing](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-keyframes/editing-moving-copying-keyframes.html)、[Qt Undo Framework](https://doc.qt.io/qt-6/qundo.html)

## 4. StageとBounds: 見えることと速いことを混ぜない

After Effectsにはframe外でlayerを出し入れするpasteboardがあり、Finalはframe内だけである。CavalryもComposition外を独立したViewport canvasとして扱う。よって「編集worldはframeより広い」「Final frameは別」という分離は先例が強い。一方、枠外を実画素で見せるかoutlineだけにするか、scrim濃度をどうするかは収束していない。

OpenFXはRoDをeffectが生成可能な最大領域、RoIを要求出力のために各入力から必要な領域として分離し、infinite RoDをHostがproject extent等へclampする。これは固定候補である。ただしNuke公式自身が巨大/過小bboxにAdjBBox、CopyBBox、BlackOutsideと警告を用意し、AEも古いeffectのedge clippingへGrow Boundsを要求する。精密bboxと手動補修UIは日曜大工帯である。

- 固定: RoD≠RoI≠actual texture bounds、範囲外transparent black、無限clamp、Unknownは空でなく最適化不能
- 可変: tightness、警告閾値、visual bounds、VRAM予算、scrim
- 審判: 最適化経路と全域評価のpixel一致。過小宣言は拒否し、画素欠落で「高速化」しない

先例: [After Effects composition pasteboard](https://helpx.adobe.com/ca/after-effects/using/composition-basics.html)、[Cavalry Viewport](https://cavalry.studio/docs/user-interface/menus/window-menu/viewport/)、[OpenFX processing architecture](https://openfx.readthedocs.io/en/main/Reference/ofxProcessingArch.html)、[Nuke Bounding Box](https://learn.foundry.com/nuke/content/comp_environment/reformatting_elements/adjusting_bbox.html)、[After Effects Grow Bounds](https://helpx.adobe.com/lt/after-effects/using/utility-effects.html)

## 5. Explicit Shared Effect: Definition/Useは先例あり、対象集合式は未収束

DaVinci Resolve Shared Nodesは、groupを作らず同じnode設定を複数clipの個別grade treeへ置き、各tree内で自由な順序を持たせる。Nuke Cloneも同じproperties/control panelを共有しながらrender tree上の位置と接続を別にできる。これはMotoliiの`EffectDefinition`と各layer stack内`EffectUse`に直接対応する。

一方AE Adjustment Layerは「選択対象」ではなく、timeline下方の合成結果全体へ一度適用する。除外にはprecompose、個別effect複製、expression link等が必要になる。したがってAdjustment LayerをExplicit Shared Useの意味へ流用しない。

- 固定候補: definition identityとuse identityの分離、各Useのordered stack位置、共有parameter変更の全Use反映、非隣接可
- 未決: 参照中Definitionの削除を拒否/cascade/materializeのどれにするか、orphan definitionのGC、UI上のunlink/copy-local
- 可変: 常時線、gutter幅、bundle、stub、socket形状
- 禁止: timeline隣接を永続target意味にする、source layerを消費/複製する、万能include/exclude式をv1へ焼く

先例: [DaVinci Resolve Shared Nodes](https://documents.blackmagicdesign.com/SupportNotes/DaVinci_Resolve_15_New_Features_Guide.pdf)、[Nuke clone API](https://learn.foundry.com/nuke/developers/14.0/pythonreference/_autosummary/nuke.clone.html)、[After Effects Adjustment Layers](https://helpx.adobe.com/ca/after-effects/using/creating-layers.html)

## 6. Typed connection: 型だけ取る、Cavalryの単一入力制限は取らない

Cavalryはlayer UIの裏で実connectionを持ち、互換型だけを接続し、1出力を複数属性へ接続できる。MaterialXもtyped data streamをnode graphで接続する。型付きout/inは成熟した契約である。

ただしCavalryは1 attributeに1 inputしか持てず、animation curveもinput扱いなので、別connectionで上書きするとkeyframe dataが失われる。これは「typed connection」と「入力合成規則」を同じものとして固定してはいけない反例である。

- 固定: port type、方向、参照ID、循環/欠落拒否
- 未決: key/Const/DataTrack/ParamDriver/connectionの合成順位
- 禁止: 接続追加が既存keyを黙って削除すること

先例: [Cavalry Connections](https://cavalry.studio/docs/getting-started/key-concepts/connections/)、[MaterialX specification](https://materialx.org/assets/MaterialX.v1.38.Spec.pdf)

## 7. Instance identityとseed: 分離は固定、生成規則はDistribution責務

Blender Geometry Nodesはpoint `id`をstable random identifierとしてindexと分離し、Random ValueのID入力やmotion blurへ使う。OpenUSD PointInstancerも配列indexが別particleに再利用される問題を明記し、time-varying `int64 ids[]`でidentity trackingを行う。ID未指定時だけarray positionへfallbackする。よって`InstanceId != index`は固定できる。

しかし、USDはIDをproducerがauthorする契約であり、全Distribution共通のID導出式を規定しない。Blenderも特定generatorがstable IDを生成するが、Grid/Path/Radial編集を横断する共通slot規則ではない。したがって次だけを先に固定する。

```text
RandomKey = stable_hash(user_seed, instance_id, channel_tag)
value     = pcg32(RandomKey)
```

- 固定: 明示`user_seed:u64`、`InstanceId`とindex分離、PRNG名/version、clock/OS entropy/thread/GPU順禁止
- Distribution契約: 自分のidentity domainと、count/insert/reorder/type-change時に何を同一個体とみなすかを宣言する
- P0I仮説: Linear/Radial/Pathのordinal、Grid座標。これはgoldenと反例試験前に製品schemaへ焼かない
- 代替候補: generator-authored ID列、algorithm-version付きprocedural ID。derived instance全列のDocument保存とは分けて検討する

先例: [Blender stable point ID](https://docs.blender.org/manual/en/3.3/modeling/geometry_nodes/point/distribute_points_on_faces.html)、[Blender Random Value](https://docs.blender.org/manual/de/3.0/modeling/geometry_nodes/utilities/random_value.html)、[OpenUSD PointInstancer](https://openusd.org/24.08/api/class_usd_geom_point_instancer.html)

## 8. DuplicatorとElement Domain: 境界は取る、例外処理は写さない

Cavalry DuplicatorのInput Shapes、Distribution、per-instance Position/Rotation/Scale/Visibility/Opacity/Prototype/Time Offset、nested Index Contextは採用根拠が強い。Houdiniもpoint/vertex/primitive/detailごとにattributeのdomainと寿命を分ける。よってdomain別identityとtyped channel/selector protocolは固定候補である。

一方CavalryはInput Shapeの親transformを無視し、offset/animationを残したい場合は空Groupの子へ入れる公式回避策を示す。これはまさに利用者側の構造工作なので、Motoliiのsource transform意味として模倣しない。またCavalryのIndex Contextは表現順序には使えるが、stable identityの代用にしない。

- 固定候補: typed prototype refs、InstanceContext、domain tag、per-instance channel、nested context、1,000 cloneを1,000 timeline rowへしない
- P0Iで決める: source local transformの扱い、nested identity、materialize、domainごとのSelector適用可否
- 禁止: offsetを残すためだけのGroup、全domainの単一index、index由来random identity

先例: [Cavalry Duplicator](https://cavalry.studio/docs/nodes/shapes/duplicator/)、[Cavalry Index Context](https://cavalry.studio/docs/nodes/utilities/index-context/)、[Houdini Geometry Attributes](https://www.sidefx.com/docs/houdini/model/attributes.html)

## 9. 仕様へ反映する審判

### そのまま維持

- Relative MoveのD2 one-shot差分と1 Undo
- Stage表示とK0最適化の分離
- `Finite/Infinite/Unknown`と保守fallback
- Effect Definition/Use、各layer stack位置での個別評価
- `InstanceId != index`、明示seed、決定論PRNG
- domain別identity、単一schema化の保留

### 実装前に追加決定が必要

1. GAP-14 / D1l: 参照中Definition削除、unlink/copy-local、orphan GC
2. P0I: Distribution別identity continuity表とalgorithm version
3. P0I: Input Shape local transformをDistribution transformとどう合成するか
4. Param pipeline: typed connectionと既存key/Const/DataTrackの合成・拒否規則

### UI試作で壊してよい

- modifier、HUD、ghost
- off-frame scrim/outline/full pixelの見せ方
- Effect connection gutter、routing、bundle/stub
- Duplicator Inspector、Context可視化、seed Randomize導線
