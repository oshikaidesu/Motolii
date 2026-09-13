# 場(Field)の取説 — 1 個の効果が、板にも網にも点群にも乗る

作成日: 2026-09-12

状態: **板・網・点群で実装済み。線(制御点)は未対応。場の座標の既定は未決(§6)。**

関連正本: [根底](ideal.md)(10. 変位は画像をずらすのでなく世界を動かす)、[Vism コンセプト](vism-package-concept.md)、[プラグイン作者向け規約](plugin-authoring.md)、[裁定台帳](decision-index.md)

## 1. 一文で

> **場は素材を選ばない。** 作者は「点をどう動かすか」を 1 回書き、板・網・点群のどれに乗せても同じ意図が出る。素材ごとの版は書かない。

これは AE にも、AE 上のどのプラグインにも無い性質である。AE のエフェクトは一生レイヤー平面で焼かれ、本物の mesh には届かない。

## 2. 書き方

manifest に `"STAGE": "field"` を宣言し、`fn field` を 1 つ書く。

```wgsl
/*{ "ID": "example.push", "STAGE": "field",
    "INPUTS": [ { "NAME": "amount", "TYPE": "float", "DEFAULT": 50.0 } ] }*/

fn field(in: FieldIn, p: FieldParams) -> FieldOut {
    return FieldOut(vec3f(p.amount, 0.0, 0.0), in.normal);
}
```

型は fork の re_renderer が持つ 1 本の定義(`shader/utils/field.wgsl`)。板も網も点群も同じ struct を読む。

| 欄 | 意味 |
|---|---|
| `in.frame_position` | 素材の座標での点の位置(§6) |
| `in.normal` | その点の法線 |
| `out.offset` | **足す**変位(世界の単位) |
| `out.normal` | 差し替える法線 |

欄の struct と wrapper は manifest から自動生成される([`effects/surface_program.rs`](../motolii/crates/motolii-render/src/compositor/effects/surface_program.rs))。同梱の実例は [`vism/turbulent_displace.wgsl`](../motolii/crates/motolii-render/vism/turbulent_displace.wgsl)。

## 3. どの素材に、どう乗るか

| 素材 | 場が動かす点 | 状態 |
|---|---|---|
| 網(mesh) | 頂点 | 実装済み |
| 点群 | 点 | 実装済み |
| 板(画像・動画) | 格子の頂点(§4) | 実装済み(2026-09-12) |
| 線(パス・文字の輪郭) | 制御点 | **未対応**。今は `OpKind` の enum に閉じている([`motolii-doc/src/vector.rs`](../motolii/crates/motolii-doc/src/vector.rs)) |

## 4. 板は格子である

板を四角 1 枚(三角形 2 枚)のまま場に通しても、動くのは 4 隅だけで、絵はその四角から出られない。そこで**場を持つ効果が乗っている板だけ**を格子に割り、頂点段で場を当てる。

- 升の数は一辺 `FIELD_GRID = 128`([`effects/surface_program.rs`](../motolii/crates/motolii-render/src/compositor/effects/surface_program.rs))。
- **場を持たない層は割らない。** 既定は 1(＝三角形 2 枚)で、従来と 1 命令も変わらない。
- 描画・選択・picking の 5 位相すべてが同じ格子を使うので、**動いた形のまま選べる**。

同梱の Turbulent Displace は板・網・点群に同じ札で乗る。Direction は Blender の Displace modifier の並び(`Normal / XYZ / X / Y / Z`)に **`XY`(Z を伏せる)** を足した物で、既定は `XYZ`。`XY` が AE の Turbulent Displace そのもの(面内だけ)、既定はそれに浮き沈みが足された物なので、2D の合成では AE と同じに見え、カメラを倒すと立体が現れる(2026-09-13)。2D 専用だった Turbulent Warp は棚から退いた(file は Warp 段の契約として `EXPOSE: false` で残る)。

これにより板でも**シルエットが変わる**。AE で旗を波打たせる時に要る「マスクを描いて Repeat Edge Pixels」という儀式は、シルエットが動かせない事を隠すためのもので、ここでは要らない。

## 5. 乗らないもの

- **点の数が変わる効果**(分割・トリム・刻み)。場は「今ある点を動かす」規則なので、数の増減は別の動詞。Houdini も同じ所で node を分けている。
- **近傍を要る効果**(平滑化)。点ごとに独立して評価されるため。
- **線(制御点)**。§3 の通り未対応。

## 6. 場の座標 — 今は層ローカル

`in.frame_position` は**その層の座標**で渡る。結果として、隣り合う 2 つの層は同じ効果・同じ値でも波が繋がらない。

繋げる手段は既にある。`offset_*` に層の居場所を入れると、式の座標が世界の座標になり、重ねた板と点群が**1 枚の布のように**波打つ(証拠は §7 の `overlap-linked.png` と `overlap-loose.png`)。

ただし今はそれを利用者が手で写す必要がある。既定をどうするかは未決で、[裁定 2026-09-02](decision-index.md)(「変位は画像をずらすのでなく世界を動かす」)の向きと、「札は自動で変えない」の間で選ぶ:

- **A.** 既定で世界座標。隣り合う層が勝手に繋がる。
- **B.** 層ローカルのまま、「世界の場」を利用者が宣言した時だけ繋ぐ。
- **C.** 保留。

## 7. 証拠の撮り方

```bash
cargo run -p motolii-render --example field_everywhere -- <flag.png> <out_dir>
```

8 枚出る。

| 絵 | 示すもの |
|---|---|
| `flag-still` / `flag-xyz` / `flag-normal` | 板のシルエットが四角の外へ出る(マスクなし) |
| `world-still` / `world-moved` | 同じ効果・同じ値が板・点群・網に乗る |
| `world-linked` | 場を世界座標で評価した 3 素材 |
| `overlap-loose` / `overlap-linked` | 層ローカル(ばらける)と世界座標(輪郭が 1 本になる)の差 |
| `pass-everywhere` | 絵の効果(ブラー)が板・点群・網の 3 つに効く(§8) |

機械の審判は 2 つ。[`effects/surface_program.rs`](../motolii/crates/motolii-render/src/compositor/effects/surface_program.rs) の `field_leaves_the_rectangle`(元の四角の外に描かれた画素を数える)と、[`render_effects.rs`](../motolii/crates/motolii-render/src/compositor/render_effects.rs) の `passes_reach_every_material`(網の縁の外へブラーが滲む)。どちらも実 GPU。

## 8. 絵の効果(Pass)も素材を選ばない — ただし画面で効く

`uv → color` の効果(ブラー・グロー・色補正、Shadertoy や ISF から来る物のほぼ全部)は
`STAGE` が `pass`。素材が板なら**層の絵**へ焼く。網・点群・環境には焼く先の絵が無いので、
**その層を単独で画面へ描いた後、その窓の絵へ流す**([`sequential.rs`](../motolii/crates/motolii-render/src/compositor/sequential.rs) の `apply_screen_passes`)。

作者の側に分岐は無い。`fn fs_main` を 1 回書けば、板でも網でも点群でも乗る。

代償は 1 つで、隠さない:

> **画面で効いた層は、その run の中で平らな 1 枚になる。** 網をぼかすと、他の網と深度で刺さり合わなくなる。AE でプリコンポして効果を掛けた時と同じ代償。

避けたい時は、効果を場(§2)か表面(`STAGE: surface`)として書く — そちらは幾何のまま効く。

### Pass が乗らない所

- **層の平面ではなく画面の解像度で効く。** 小さく置いた網のブラーは、層の大きさではなく画面の画素で広がる。
- **余白(padding)と溢れ(SPILL)は板だけ。** 窓は既に画面ぶんあるので、外へ広げる余地が無い。
