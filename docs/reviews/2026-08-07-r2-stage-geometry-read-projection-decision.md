# R2-STAGE-GEOMETRY-READ — Stage幾何read projectionの縮小採用

日付: 2026-08-07
状態: **縮小採用 / 実装済み（2026-08-11 Vector Rect追補）**

## 1. この決定が閉じる一問

「表示中のobjectを選び、Stage gizmoで動かす」背骨の直上流にある、次の一問だけを閉じる。

> published revisionとcameraから、`LayerId`と幾何を型で言えるread projectionを、
> 第二writer・第二GPU owner・Document schema変更なしにcurrent codeへ接続できるか。

selection producer、pointer route、gizmo、Position key書き込みは本決定に含まない。

## 2. 前提の訂正

[M3 RN runtime実行地図](../m3-rn-runtime-execution-map.md)と
[supervisor handoff](2026-08-07-m3-supervisor-handoff-stage-to-gizmo.md)は、幾何boundsを
`TARGET_MISSING`（必要なtyped targetがcurrent mainにない）として扱っていた。

local main `9b2deac4`のread-only再照合で、これは`REMAP`であると確認した。**幾何は既に評価済みで、
LayerIdとの対応も実体化されており、publishされていないだけである。**

| 必要な材料 | current mainの実在 | 可視性 |
|---|---|---|
| 正準座標の点・サイズ | `CanonicalPoint` / `CanonicalSize`（`motolii-core/src/canonical.rs`） | `pub` |
| LayerId→world変換 | `GraphBuilder::world_affines: HashMap<u64, Affine2D>`（`motolii-doc/src/graph.rs:204,284`） | private |
| world変換の再構成 | `resolve_transform`（親子継承、anchor、scale、rotation、LookAt解決込み） | `pub`（lib.rs:49で再export） |
| camera view変換 | `camera_view_affine` = `scale(1/h) * rotation(-roll) * translation(-c)`（`affine.rs:170-173`） | `pub(crate)`。ただし構成要素は全て`pub` |
| camera値 | `eval_comp_camera_doc(doc, eval, tracks)`（`camera_eval.rs:20`） | `pub` |
| 画面→局所の逆写像 | `Affine2D::try_invert`（`affine.rs:81`） | `pub` |

`compose_camera_world`は`pub(crate)`だが、中身は`camera_view_affine(camera) * world`であり、
`Affine2D::{scale, rotation, translation}`と`eval_comp_camera_doc`の公開組み合わせで再構成できる。
**motolii-docへの公開API追加なしに成立する。**

## 3. 決定

### 3.1 投影する値 — AABBではなく (局所rect + 変換)

flattenした軸並行bounds（AABB）を投影**しない**。次を投影する。

```text
LayerId -> { canonical local rect, world Affine2D, camera view Affine2D }
```

理由: `resolve_local_only`（`affine.rs`）はrotationとLookAtを合成しており、world変換は真に回転を含む。
AABBへ潰すとrotation下でgizmo handleの位置が実際の辺からずれ、hit-testとhandle描画が食い違う。
消費側は`Affine2D`を自分で適用し、screen→局所の換算に既存`try_invert`を使う。

これはprojectionを小さくもする。新しい幾何の意味を発明せず、**既に評価済みの2値を並べるだけ**になる。

### 3.2 v1の適用範囲 — Rect source限定

`build_source`（`graph.rs`）はClipSourceごとに分岐し、幾何の入手可能性が一様でない。

| ClipSource | 正準サイズの入手 | v1処分 |
|---|---|---|
| `Plugin(RECT_LAYER_SOURCE)` | `size` paramを`eval_vec2`で正準単位のまま取得（`graph.rs:564,584`） | **投影する** |
| `Asset { video: Some }` | source解像度→正準への換算が未決 | typed unavailable |
| `Vector { recipe: StandardShape::Rect }` | M2 D3で局所中心`[0, 0]`と`width`／`height`が確定済み | modifierなしだけ**投影する** |
| `Vector { recipe: other }` | Rect以外またはmodifier適用後のextent契約は未決 | typed unavailable |
| `Plugin(other)` | prepared recipeにextent契約なし | typed unavailable |
| `Asset { video: None }` | visual graphへ参加しない（AG-1） | 投影しない（正しい不在） |
| `Group` | 固有sizeを持たず、子の合併の意味が未決 | typed unavailable |

v1をRectへ縮小する根拠は、R1／VS-1の利用者出口がRectangleの背骨であり、
video／Rect以外のvector／pluginのextent正準化とGroup合併は、それぞれ独立した意味決定だからである。
**入手できない幾何をfakeせず、typed unavailableで返す。** これは
[handoff §9](2026-08-07-m3-supervisor-handoff-stage-to-gizmo.md)の非目標「fake geometry」と一致する。

2026-08-11追補: M2 D3で永続意味が確定した`VectorRecipe / StandardShape::Rect`について、
既存のparameter評価とworld／camera変換を再利用するread projectionを追加した。schema、公開API、
shape別frameは追加せず、modifier付きRectと他のVectorは従来どおりtyped unavailableとする。

### 3.3 時刻gate — 今見えているものだけ

`clip_active(clip, t)`が偽のClip、`should_draw(env)`が偽のitemはその時刻に描画されない。
投影もそれに従い、**現在時刻で実際に可視なlayerだけ**へ幾何を出す。

`clip_active` / `should_draw` / `item_layer_id` / `item_envelope`は`graph.rs`のprivateである。
可視判定をmotolii-ui側で再実装すると「見えているか」の第二ownerが生まれるため、これは行わない。
狭い公開述語をmotolii-docへ足すか、既存公開経路で再利用できるかを実装粒の最初のstepで確定する。
**ここが本決定で唯一残る公開API判断であり、実装担当に発明させない。**

### 3.4 置き場所 — motolii-ui、`project_timeline`と同型

`timeline_projection.rs`の`pub fn project_timeline(document: &Document, ...)`が確立済みの型である。
本projectionも同じくmotolii-uiに置き、Documentをread-onlyで読む。
motolii-doc、Document schema、journal、公開plugin契約、GPU ownerを変更しない。

## 4. positive / negative oracle

positive:

- 既定camera・非回転のRectで、投影rectが`build_rect_overlay`へ渡る`center`／`size`とbit一致する
- 親子付きlayerで、投影変換が`resolve_transform`の結果と一致する
- rotationとLookAtを持つlayerで、変換適用後の4隅が`compose_camera_world`相当と一致する
- camera（center／roll／height）変更が投影へ反映される

negative:

- audio-only Clip（`video: None`）に幾何が出ない
- `clip_active`が偽の時刻で幾何が出ない
- Group／video／Rect以外またはmodifier付きvector／plugin sourceが**typed unavailable**であり、0や既定値を返さない
- 投影実行でDocument write 0、revision不変、journal追記0
- 特異な変換（`try_invert`が`None`）でpanicせず、typed失敗を返す

## 5. 非目標

- selection producer、pointer route、gizmo、drag、snap
- Position key書き込み、Timeline投影、Easing接続
- video／Rect以外またはmodifier付きvector／plugin source extentの正準化
- Group bounds合併の意味決定
- AABB／bounding volumeという語をDocumentや公開契約へ入れること
- motolii-docへの一般的な幾何API新設（3.3の狭い可視述語を除く）
- 第二GPU device、rust-skia overlay backend、CPU readback
- RN Browser、RN Timeline、RN Stage pointerの実装
