# グループのtransform boundsをどこから採るか（第2案・未採択）

日付: 2026-08-10
状態: **起草（第2案）/ 未採択。反対側レビュー前**

## 0. この文書の扱い

**決定ではない。** 反対側レビューを通すまで採択せず、`decision-index` へも登録しない。
**本書を根拠に実装を発注しない。**

**第1案は同日に `REJECT` された**（[第1案と却下記録](2026-08-10-group-transform-bounds-draft.md)）。
本書はその却下で判明した構造的事実を前提に書き直したものである。

第1案の敗因は「docsを読んで妥当なモデルを立て、**実装コードを読まずに起草した**」ことだった。
本書はコードを先に読んでいる。読んだ箇所は§5に列挙する。

## 1. 問い

[グループrootを掴んで動かす鎖](2026-08-10-group-drag-call-site-sketch.md)の `???` #7。

**グループを掴むとき、handleが囲む矩形をどう導出するか。**

第1案は「content由来かcanvas由来か」という2択で問いを立てたが、**それが誤りだった**。
グループには評価モデル上「自分の画像」が無く、`local_rect` に当たるものを持たない。
正しくは**導出規則の設計**である。

## 2. 第1案の却下から引き継ぐ制約

- **RoD / RoI を Stage操作boundsへ流用しない。**
  `specs/M4-cache-and-analysis.md:43` が「実texture bounds、Document永続値、
  GPU alpha readbackと混同しない」と明示的に禁じている
- **`Unknown` を composition全域へ倒す規則を作らない。**
  原因が既定かfallbackかに関わらず、操作上の結果が全域handleになる
- **「transformはフラット1枚への操作」を前提にしない。** 変形は子へ継承される（§5.1）

## 3. 一次資料 — W3C SVG2 の bounding box

SVGの `<g>` は**変形を子へ継承する入れ子コンテナ**であり、Motoliiのグループと同じ構造を持つ。
そして bounding box の規則が仕様として書かれている。

> For each descendant graphics element child of parent...
> **set box to be the union of box** and the result of invoking the algorithm to compute a bounding box with child

> the values of the **opacity, visibility**, fill, fill-opacity, fill-rule,
> stroke-dasharray and stroke-dashoffset properties on an element
> **have no effect on the bounding box**

- **stroke は除外**（別に stroke bounding box がある）
- **filter / clipping path / mask は除外**
- 幾何が零でも bounding box を持つ

`display` と `visibility` は別概念であり、**その差がまさに bounds に出る**。

> `display: none` — the element (and all its descendents) **not becoming part of the rendering tree**
> `visibility: hidden` — the element not being painted. It is, however, **still part of the rendering tree**。
> such elements **contributes to bounding box calculations** and clipping paths, and does affect text layout

## 4. 先例 — エフェクトはhandleを広げない

利用者の観察（2026-08-10）:

> パーティクルエフェクト適用時ももとのシェイプにgizmoがあります

出力が画面いっぱいに広がっても、handleは発生源のシェイプに残る。
これはAfter Effects／Alight Motionで運用されている挙動であり、
**§3のSVG仕様（filter除外）と同じ規則**である。

**したがってこの点にMotolii独自の考え方を入れない。**

## 5. Motoliiコードの実測（supervisorが本日読んだ箇所）

### 5.1 変形は子へ継承される

`crates/motolii-doc/src/graph.rs` の `build_group`:

```text
let child_xform = self.world_affine(layer)?;   // graph.rs:351
...
// F-3: 子合成 → グループ effect stack → clipping mask。変形は継承済み。   // graph.rs:377
```

グループの変形は各子の world affine へ継承され、変形済みの子を合成してから
グループのeffect stackを適用する。

### 5.2 幾何解決は可視性に依存しない

`crates/motolii-doc/src/spatial_resolve.rs` の `resolve_document_spaces` は
`visible` / `solo` / `should_draw` を**一度も参照しない**。`collect_layers` が全layerを集め、
world affine を全て埋める。

**非表示の子の幾何は既に計算されている。** 外枠へ含める追加コストは無い。

### 5.3 非表示でも依存されていれば評価する例外が既存

`build_group`:

```text
if !draw && !next_needs_mask { continue; }
```

非表示の子でも、次の子が clipping mask として必要とするなら build する。
2026-08-08から未決の「非表示だが依存先として評価される参照元」は、
**片方が既に実装されている**。

### 5.4 leaf の幾何は存在するが、対象が限られる

`crates/motolii-ui/src/stage_geometry_projection.rs` は
`StageLayerGeometry { local_rect: StageLocalRect { center, size }, world, camera_view }` を返す。
Rectangleはplugin paramの `center` / `size` から `local_rect` を作る（`:172`）。

一方 `Group` / `VideoSource` / `VectorSource` / `PluginSource` は
`StageGeometryUnavailable` を返す（`:101` / `:132`）。**leafでも幾何を持たないものがある。**

## 6. 案

### 6.1 導出規則

**グループのtransform boundsは、子孫の幾何を canonical 空間で union したものとする。**

各 leaf について `local_rect` の4隅を、継承済み `world` で canonical へ写し、その全体の外接矩形を取る。
入れ子グループは再帰する（SVG §3 の container 規則と同じ）。

composition寸法は使わない。**Motoliiにcanvasを持つグループは存在しない。**

### 6.2 エフェクトは union を広げない

グループ自身および子の effect stack、clipping mask、blend、opacity は
**boundsへ影響しない**（§3 SVG仕様、§4 先例）。

blurやパーティクルで出力が外へ広がっても、掴む枠は動かない。
逆にすると、エフェクトを足すたびにhandleが動き操作対象が変わって見える。

### 6.3 非表示の子は union に寄与する

Motoliiの `visible: false` は **`visibility: hidden` 側の意味論**として扱う。
非表示の子も boundsへ寄与し、**表示を切り替えてもhandleは動かない**。

根拠は3つある。

1. SVG仕様が `visibility` は bounding box に影響しないと明記する（§3）
2. Motoliiの幾何解決は既に可視性非依存で、**全layerのworld affineが埋まっている**（§5.2）
3. 「非表示だが依存先として評価される」例外が既に実装されている（§5.3）

**これは `display: none` 相当の意味論をMotoliiが持たない、という主張ではない。**
`visible` フラグ1つをどちらへ写すかの選択であり、本書は `visibility` 側を採る。

### 6.4 幾何を持たない子

`Video` / `Vector` / `Plugin` / 入れ子`Group` が `Unavailable` である間、
**それらを含むグループは完全な bounds を出せない。**

このとき **composition全域へ倒さない**（第1案の却下理由 §2）。
**未確定として扱い、handleを出さない。**

これは本書が解く問題ではない。**leafへ幾何を与えることが別契約**であり、
それが閉じれば同じ union 規則がそのまま適用できる。

全ての子が幾何を持つグループは、**本規則だけで今すぐ bounds を得る。**

### 6.5 空グループ

子が0のグループは union が空である。**handleを出さない。**
全域へ倒さない。

## 7. 決めていないこと

- **picking領域**（見えている画素でグループを選べるか）。`???` #8 と同じ層
- `Video` / `Vector` / `Plugin` の `local_rect` 契約（§6.4）
- 評価時刻の指定。子のtransformが時間変化する場合、どの時刻の union を使うか
- pivot / anchor を union のどこに置くか
- snap の基準点

## 8. 非目標

- 本書を根拠に実装を発注すること
- gizmo採択の裁定（`BUILD JUSTIFICATION` は `NOT NONE` のまま）
- `visible` の意味論を bounds 以外の面（評価、export、a11y）へ拡張して決めること
- RoD / RoI の契約へ触れること

## 9. 反証してほしい点

1. §6.1 の union 規則は、`StageLayerGeometry` と `world` から**実際に計算できるか**。
   canonical空間と camera_view の適用順を取り違えていないか
2. §6.3 は Motolii の `visible` の既存意味論と**衝突しないか**。
   `visible_layers_at` は非表示を除外するが、それを bounds で無視してよいか
3. §6.2「エフェクトは広げない」は、Motoliiの clipping mask 意味論
   （下レイヤーへクリップ）と**整合するか**。マスクで切られた側の bounds はどうなるか
4. §6.4「handleを出さない」は、利用者から見て**故障と区別できるか**
5. SVG仕様の引用は正確か。`getBBox` と `getStrokeBBox` の差、
   および container 要素での `display:none` の扱いを取り違えていないか
