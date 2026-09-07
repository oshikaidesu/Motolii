# 最小コアの定義 — 今日のコードで言い直す

状態: **決定(2026-09-07 利用者裁定「いいよ」。§6 の 4 点をそのまま採用)**。利用者の「Motolii の完成 = Vism の自走」を受け、[小さなコア](../extensible-core-model.md)と[意味の席](2026-07-24-replaceable-semantic-seat-decision.md)の原則を、Stage 5 の実コードと今日足した 3 枚(Glass・Turbulent Displace・Repeater)で具体化する。原則は変えない。名前を「今ある物」に付け直し、自走を止めている継ぎ目を数える。

## 1. 一文で

> **コアは、棚の 1 枚が「宣言」と「写し」だけで載るための、席と口の集合である。表現の式はコアに入らない。**

自走の判定は 1 つ: **fork も render も触らずに、棚に 1 枚足せるか。** 今日は 3 枚とも「宣言 + 写し 1 関数 + fork の受け口」で載った。最後の 1 つがまだコアの外にある。

## 2. コアが所有する物(今の家で言うと)

| 席 / 口 | 今の正本 | 役 |
|---|---|---|
| **Document** identity・時刻・revision・single writer・Undo | `motolii-doc` `store/document.rs`、Intent | 作品の持続性 |
| **効果列の文法**: 順序、enabled、param は property `effect.<id>.param.<name>`、上の効果は素材へ・下は全体へ | `store/effect.rs`、`view/resolve.rs` `resolved_effects` | 棚の 1 枚が「何をいつ受けるか」 |
| **棚と admission**: 並ぶ物・受ける物は同じ表 1 つ | `effects/catalog.rs` `descriptors`、port `applyEffect` | 名前で探し、載せる |
| **kind の宣言**: 欄(label・default・range・choices・section)は data | `placement.rs`・`material.rs`・`field.rs`・ISF manifest(4 か所) | Inspector・property・キーの共通口 |
| **型付き出力(席)**: 効果が返す物の型 | Pass(texture→texture)、Placement(配置の集合)、Surface(`MeshSurface`)、Field(`MeshDisplace`) | consumer が provider を知らずに読む |
| **表現の受け口(consumer)**: 板・網・点群・環境 | `LayerContent` の 4 変種、`SequentialInput` | 同じ席の値を表現ごとに native に受ける |
| **層属性**: environment / flatten / projection / clip | `store/attrs.rs` | 層 1 枚の意味 |
| **評価の同一性**: Preview = Export、線形光、run の組み方 | `compositor/sequential.rs` | 絵の再現性 |
| **資源**: texture の寿命・cache key・VRAM の天井 | `texture_manager`、`environment.rs` の上限 | 抱えない |
| **失敗**: 読めない素材・未知 plugin は層単位で局所化、export は型付き拒否 | `layer_failures`、`failed_*` | 壊れ方を封じる |

## 3. コアに入らない物(棚の 1 枚の中身)

余弦畳み込み・Phong prefilter・split-sum・Schlick・等距円筒の写像・simplex fbm・法線の曲げ・Glass の欄の意味・Repeater の形。これらは**借りた定規**であり、差し替えても席は動かない。今日それらを fork の `environment.rs`・`lighting.wgsl`・`noise.wgsl` に置いたのは「口が無かったから」で、置き場としては暫定。

## 4. 自走を止めている継ぎ目(数えられる分岐)

進み(2026-09-07 同日): **A 済・C 済**。Glass と Turbulent Displace は `vism/glass.wgsl`・`vism/turbulent_displace.wgsl` の 2 file になり、fork の `surface`・`displace` 欄は消えた(`GpuMeshInstance` は `program` と 12 float の `params` だけ)。B は C で「席の読み手 = manifest の順に slot へ並べる 1 関数」に畳まれて実質済。残りは D(点群・2D への配り方。点群は `PointDisplace` の CPU の写しが Turbulent Displace の id を名指ししている)と E。

| 継ぎ目 | 今 | 自走の形 | 大きさ |
|---|---|---|---|
| **A. kind の宣言が 4 つの家に散っている** | placement / material / field / ISF manifest。`builtin_label`・`choices` で束ねているだけ | 1 つの kind 契約(欄・section・choices・出力の型)。ISF manifest の `INPUTS` をそのまま非 shader の kind にも使う | 中(doc 内の整理) |
| **B. 席ごとの読み手が Rust の関数** | `material::surface()`・`field::displace()`・`placement::...`。2 枚目を足すと関数が増える | 席の型は固定、kind → 席の値は data(manifest の `OUTPUT` に席の名前)。読み手は席ごとに 1 つ | 小 |
| **C. fork の受け口が効果ごとに増える** | `GpuMeshInstance::surface`・`::displace`。第三者は fork を触れない | **網の頂点 stage と面 stage に、manifest 付き WGSL を差し込む口**(2D の Pass が `vism/*.wgsl` で既にやっている事の 3D 版)。fork が持つのは「typed uniform と snippet を受ける stage」だけ | 大(fork 1 回、以後は data) |
| **D. 同じ場を 3 表現へ配る所が手書き** | 網は GPU、点群は CPU、2D は未 | Field 席の値を consumer が受ける規約を 1 つ(網=頂点、点群=点、2D=sample 位置)。CPU/GPU の写し 2 つは「同じ program を 2 つの stage へ compile」に畳む | 大 |
| **E. 第三者の置き場** | `crates/motolii-render/vism/` に同梱のみ | 作者の directory を catalog の root に足せる(`catalog_source_roots` は既に複数 root) | 小 |

C と D を閉じると、今日の Glass と Turbulent Displace は「manifest + WGSL」の 2 file になり、fork の `surface`・`displace` 欄は消える(1 つ直せば同族全部)。

## 5. 最小コアの外形(C の口)

```text
vism/glass.wgsl
/*{ "ID": "motolii.glass", "STAGE": "surface",
    "INPUTS": [ {"NAME":"ior","TYPE":"float","DEFAULT":1.5,"MIN":1,"MAX":3}, ... ] }*/
fn surface(albedo: vec3f, n: vec3f, v: vec3f, env: Environment) -> vec3f { ... }

vism/turbulent_displace.wgsl
/*{ "ID": "motolii.turbulent_displace", "STAGE": "field",
    "INPUTS": [ {"NAME":"amount",...}, {"NAME":"along","TYPE":"long","VALUES":[0,1],"LABELS":["Normal","Space"]} ] }*/
fn field(p: vec3f, n: vec3f) -> Displacement { ... }
```

- Host が持つのは stage の契約(引数の型・返り値の型・使える uniform・使える環境の口)と、それを網の頂点 shader・面 shader・点群の compute・2D の sample へ差し込む compile。
- 実装(同日): 作者の file は `fn field(in: FieldIn, p: FieldParams) -> FieldOut` / `fn surface(in: SurfaceIn, p: SurfaceParams) -> vec3f` を書く。欄の struct と wrapper は Motolii の `effects/mesh_program.rs` が manifest から生成し、fork の `MeshProgram` が base(`instanced_mesh_base.wgsl`)に足して compile する。欄は field → surface の順に 12 slot(頂点属性 16 か所の上限から)。選択肢は ISF の `long` + `LABELS`。変種は「field id | surface id | catalog 世代」で覚える。compile の失敗は層単位で `layer_failures` へ。
- 式(noise・BRDF)は file の中。fork の `lighting.wgsl`・`noise.wgsl` は「同梱の 1 枚」へ移り、特権を持たない。
- Placement は shader を持たないので Rust の kind のまま。ただし A の 1 契約に乗る。

## 6. 裁定を求める点

1. **自走の判定を「fork も render も触らずに棚へ 1 枚」に固定してよいか**(配布・OS 対応は完成の定義から外す)
2. **C の口を、既存の ISF 風 manifest の拡張(`STAGE`)で作ってよいか**。別 manifest を発明しない
3. **順序**: A(宣言を 1 つに) → C(surface stage) → D(field を 3 表現へ) → B・E。C を先にする理由は、今日の 2 枚をそのまま反証に使えるから
4. **停止線**: [Vism concept §11](../vism-package-concept.md#11-実装へ進む前の停止線)の「loader・registry・marketplace は作らない」は維持。ここで作るのは file と directory から棚へ載る所まで
