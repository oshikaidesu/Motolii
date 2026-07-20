# Motion基盤候補の既知技術による処分決定(2026-07-14)

ステータス: **【決定】**。対象はRelative Move、Bounds/ROI、Effect Scope、Cloner/Effector、Element Domain。「反対側レビュー未実施」を一括保留の理由にせず、既知技術で意味が安定している最小契約だけを先に固定する。**2026-07-15のUI・Explicit共有・Cavalry採用・seed規約は[後続決定](2026-07-15-relative-scope-duplicator-decision.md)を優先する。**

本書の意味決定は着手許可ではない。[M2基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)は解除済み。U0a入場後、製品実装への割当は個別M3チケット依存に従う。

## 1. 判定

| 項目 | 今固定する意味 | 今は固定しないもの | 割当 |
|---|---|---|---|
| Relative Move | 選択keyへ同じ型付き差分を適用する一回のD2 macro。Documentに補正channelを残さない | 常設offset、Animation Layer、汎用Modifier列 | M3-U2f |
| Bounds / ROI | 論理的な出力範囲と要求領域を分離し、`Finite / Infinite / Unknown`をfail-safeに扱う | tight bounds保証、alpha readback、距離別LOD値、VRAM予算値 | M4-K0/K1、M3-U1f、M5-P2D/P4 |
| Effect Scope | `OwnedGroup / ExplicitSet / Backdrop`を異なる意味として分類。タイムライン隣接を永続意味にしない | 万能Scope式、pattern query、暗黙の「下全部」 | M3-G0-7/U4c、具体schemaは別仕様改訂 |
| Cloner / Effector | `InstanceId != index`、Host所有の`InstanceContext`、Effector純関数というspike仮説 | Cloner製品機能、公開plugin trait、Materialize詳細 | M5-P0I spike |
| Element Domain | domainを分けたまま共通Selector評価を比較する。indexを永続IDにしない | 全domainを畳む単一Element schema | M5-P0I spike |

## 2. Relative Move

v1のRelative Moveは非破壊Modifierではない。

```text
RelativeMove(selection, delta)
  -> D2 macro {
       MoveKey(key_1, typed_delta)
       MoveKey(key_2, typed_delta)
       ...
     }
```

- gesture全体がUndo 1回、Cancelは変更ゼロ
- 対象keyが混合型、削除済み、編集不可なら適用前に型付き拒否し、部分適用しない
- 同じ時間範囲を移す機能と、時間位置をずらす機能を混同しない。ここでいう差分は値空間の差分
- UIは「キーを書き換えた」ことを正直に示し、後段offsetが残ったように見せない

将来の常設補正はMaya Animation LayersのAdditive/Overrideが先例だが、translation、scale、rotation、Boolean/enumで合成則が異なる。したがって万能`Add`を導入せずPP-Gateで型別に判定する。

先例: [Autodesk Maya Animation Layer modes](https://help.autodesk.com/cloudhelp/2025/ENU/Maya-Animation/files/GUID-BBCA0BC3-7608-4E86-8E9F-B4099C316156.htm)

## 3. Bounds / ROI

OpenFXのRegion of Definition / Region of Interest / Render Window分離を、名称をMotoliiの正準空間へ合わせて採る。

```text
SpatialExtent = Finite(Aabb) | Infinite | Unknown

output_extent(t, input_extents, params) -> SpatialExtent
input_regions(t, requested_output, params) -> [RequestedInputRegion]
```

- `output_extent`: ノードが意味上生成し得る範囲。実際に確保したtexture矩形ではない
- `requested_output`: 今回Stage/Finalが必要とする範囲
- `input_regions`: その出力に必要な各入力範囲。Blurなら要求出力を半径分拡張する
- `Infinite`: 任意位置に出力可能。Hostが今回の要求範囲へclampする
- `Unknown`: 空扱いしない。ROI最適化を無効化し、必要入力の全RoDまたはHostの安全上限へ保守的にfallbackする
- 範囲外を要求された有限sourceはtransparent black。`Unknown`宣言ではFinal画素を切らない
- extent/regionは正準座標で、px/DPIを公開契約へ入れない
- GPU alpha同期readbackでextentを求めない

プラグインが未対応でも正しさを保てるよう、最初は`Unknown`を既定にして全入力評価へfallbackできる。誤った`Finite`宣言を実行時に常時検出することは、全域評価との比較なしには不可能である。最適化は固定fixtureで「宣言領域評価と全域評価がpixel一致」するconformanceを通った組み込みノードだけで有効にし、未検証pluginは`Unknown`のまま扱う。conformance不一致は宣言を拒否し、無言で有限範囲を信用しない。

先例: [OpenFX Image Processing Architectures](https://openfx.readthedocs.io/en/latest/Reference/ofxProcessingArch.html)、[OpenFX Rendering](https://openfx.readthedocs.io/en/latest/Reference/ofxRendering.html)

## 4. Effect Scope

次の三つを同じ型の曖昧な範囲指定にしない。

| 意味 | 入力集合 | 所有 |
|---|---|---|
| `OwnedGroup` | Groupが所有する子を合成した結果 | Groupが子を所有 |
| `ExplicitSet` | 型付きIDで明示した集合 | Scopeは対象を所有しない |
| `Backdrop` | 評価グラフ上の指定地点までの合成結果 | 対象の列挙ではなく評価地点参照 |

v1で既に確定しているGroup effectは`OwnedGroup`である。`ExplicitSet`と`Backdrop`は意味分類のみを確定し、Document形状は別の仕様改訂まで追加しない。AE Adjustment Layer型の「タイムライン上で下にあるもの全部」を永続意味にしない。UIで隣接操作を入口にしても、保存時は型付き入力または評価地点へ正規化する。

先例: [OpenFX Image Effect Contexts](https://openfx.readthedocs.io/en/main/Reference/ofxImageEffectContexts.html)、[OpenUSD Collections and Patterns](https://openusd.org/release/user_guides/collections_and_patterns.html)、反例として[Adobe Adjustment Layers](https://helpx.adobe.com/ca/after-effects/using/creating-layers.html)

## 5. Instance / Element spike

Cloner/EffectorとElement Domainは製品採用をまだ決めない。ただし既存world/plugin境界を誤って凍結しないため、同じspikeで次を反証する。

```text
InstanceContext {
  stable_id,
  index,
  count,
  base_transform,
  local_time,
  seed,
}

Effector(instance, t, params) -> InstanceDelta
```

- `stable_id`と配列`index`を分離する。増減・並べ替え後もseed、motion blur、overrideが意図した要素へ追従するfixtureを置く
- Effectorはinstance一覧を所有せず、Hostがcontextと評価順を渡す。隠れ状態・前フレーム依存を持たない
- TextCluster/TextWord/TextLine/ShapePath/CloneInstanceは別domainのまま比較する
- 共通化候補は`Selector(element_context, t) -> weight`という評価形だけ。ID寿命、並べ替え、要素数変化、group意味が一致しなければ単一schemaへ畳まない
- Textのword/line等、編集で再生成される要素のindexを永続参照にしない

先例: [OpenUSD PointInstancer](https://openusd.org/24.08/api/class_usd_geom_point_instancer.html)、[Houdini Geometry Attributes](https://www.sidefx.com/docs/houdini/model/attributes.html)、[Blender Attribute Domains](https://docs.blender.org/manual/en/3.6/modeling/geometry_nodes/attributes_reference.html)、[Adobe Text Animators](https://helpx.adobe.com/after-effects/desktop/animating-text/text-animation/animating-text.html)、[Cinema 4D MoData](https://developers.maxon.net/docs/py/2023_2/modules/c4d.modules/mograph/MoData/index.html)

## 6. 停止条件

- K0の契約確定前にROI最適化、枠外culling、effect paddingを個別実装しない
- P0Iのfixture前に公開`Element`/`EffectorPlugin`/`InstanceContext` traitを追加しない
- `ExplicitSet`/`Backdrop`のmigration、循環、欠落参照、評価順が決まる前にDocument fieldを追加しない
- PP-Gate前にRelative Moveを常設offsetへ拡張しない
- いずれもキャッシュキー寄与とpreview/export同一意味の審判を伴わなければ採用しない
