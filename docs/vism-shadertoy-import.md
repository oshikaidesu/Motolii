# Shadertoy の取り込み — 貼れば棚に出る

作成日: 2026-09-12

状態: **Shadertoy(`mainImage`)は実装済み。層の絵を別の時刻で読む口(`TIME_OFFSET`)は実装済み(§8)。Buffer 複数枚・音・`iChannelResolution` / `iChannelTime` は未対応。貼る窓(editor)は作らない(各自の editor で書く)。**

関連正本: [場(Field)の取説](vism-field-model.md)、[Vism コンセプト](vism-package-concept.md)、[プラグイン作者向け規約](plugin-authoring.md)

## 1. 一文で

> **変換器は書いていない。** GLSL → WGSL は wgpu 本体の [naga](https://github.com/gfx-rs/wgpu/tree/trunk/naga)（`glsl-in` / `wgsl-out`）が決定的に行う。Motolii が足したのは**方言の前口上**だけ — Shadertoy の名前を ISF の名前へ結び直す 100 行。

## 2. 使い方

`vism/` に Shadertoy の file を `.frag`(または `.glsl` / `.fs`)で置く。manifest は要らない。

```glsl
void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    fragColor = texture(iChannel0, uv) * vec4(1.0, 0.8, 1.2, 1.0);
}
```

file 名が効果の名前になり、ID は `import.<file 名>`。棚には同梱の効果と並ぶ。

## 3. 名前の結び直し

| Shadertoy | Motolii での姿 |
|---|---|
| `mainImage(out vec4, in vec2)` | ISF の `main()` から呼ばれる |
| `fragCoord` | 画素の座標。**上下を裏返して**渡す(§5) |
| `iResolution` | `vec3(RENDERSIZE, 1.0)` |
| `iTime` | ホストの時計 `TIME`(comp の秒)。欄ではない |
| `iChannel0..3` | **使われている物だけ** image 入力の欄になる |
| `iMouse` | 使われていれば point2D の欄 |
| `iFrame` / `iTimeDelta` / `iFrameRate` | ホストの時計 `FRAMEINDEX` / `TIMEDELTA` / `1/TIMEDELTA`(comp の fps) |
| `iDate` | `DATE` = 0 固定。壁時計は使わない — 同じ時刻は何度描いても同じ絵 |
| `iSampleRate` | 定数 44100 |

写せない名前(`iChannelResolution`・`iChannelTime`)は、黙って壊れずに**名前を挙げて断る**。manifest の無い素の GLSL(`void main()` だけ)も、理由を添えて断る。

実装は [`effects/isf/shadertoy.rs`](../motolii/crates/motolii-render/src/compositor/effects/isf/shadertoy.rs)、入口は [`effects/catalog.rs`](../motolii/crates/motolii-render/src/compositor/effects/catalog.rs) の `prepare`。

## 4. 乗る先

取り込んだ効果は `STAGE: pass` になる。[場の取説 §8](vism-field-model.md) の通り、**Pass は素材を選ばない** — 板は層の絵へ焼かれ、網・点群は描いた後の窓で効く。つまり貼った Shadertoy は 3D にも乗る。

## 5. 上下の向き(落とし穴)

Shadertoy も ISF も**座標は下端が 0**(GL の作法)、wgpu の texture は**上端が 0**。座標をそのまま
サンプリングに使うと、上下逆の位置を読む。**実写に当てて初めて分かる**(文字が裏返る。対称な
効果では絶対に気づけない)。

直し方は「座標を裏返す」ではなく「**読む時に裏返す**」。座標は各方言の作法のまま渡すので、
手続き的な絵の向きも Shadertoy と揃う。

- Shadertoy: `#define texture(smp, coord) texture(smp, vec2((coord).x, 1.0 - (coord).y))`
  (macro は自分自身へ展開し直されないので、中の `texture` は組み込みのまま)
- ISF: `IMG_THIS_PIXEL` / `IMG_NORM_PIXEL` が裏返す

正しさの証明は**恒等**で取った — `fragColor = texture(iChannel0, uv)` の結果が元の絵と
**完全一致**(PSNR ∞)。見張りは `shadertoy.rs` と `isf/mod.rs` の
`the_picture_is_read_right_side_up`。

## 6. 表現の天井(2026-09-12 実測)

実写に当てて、書けるものと書けないものの線を引いた。

| 書きたい物 | 状態 |
|---|---|
| 手続きの絵(`iTime` だけ) | **書ける** |
| 画像フィルタ(`iChannel0`) | **書ける** |
| 近傍を舐める(Sobel・ぼかし) | **書ける** |
| **複数パス**(抽出 → 横 → 縦 → 合成) | **書ける**(ISF の `PASSES`。`PASSINDEX` と中間 buffer が shader から見える) |
| 中間 buffer の寸法を変える(`$WIDTH/2`) | **書ける** |
| **フレーム跨ぎの持ち越し**(`PERSISTENT`・Shadertoy の Buffer 帰還) | **書ける**(2026-09-13、下記。持ち主は host) |

### 持ち越し(feedback)は host が持つ — 2026-09-13

ISF の `PERSISTENT: true` を宣言した target は、前のフレームの中身を保ったまま次のフレームへ渡る。
残像・軌跡・蓄積・反応拡散が、Shadertoy の Buffer の書き方のまま書ける。

```json
"PASSES": [ { "TARGET": "history", "PERSISTENT": true }, { } ]
```

```glsl
if (PASSINDEX == 0) gl_FragColor = mix(IMG_THIS_PIXEL(history), IMG_THIS_PIXEL(inputImage), 0.2);
else                 gl_FragColor = IMG_THIS_PIXEL(history);
```

**効果は覚えない。覚えるのは host。** 状態(前のフレームの texture)は層 × 効果ごとに compositor が持ち、
時刻 t の絵は「**層の入点を初期条件とする漸化式**」で決まる([`plugin-resources.md` §6-3](plugin-resources.md))。

| 場面 | host がすること | 重さ |
|---|---|---|
| 順再生・書き出し | 1 歩進める(前の絵が今の絵になる) | 普通の pass と同じ |
| 同じフレームをもう一度(2 つ目の窓) | 前の絵をもう一度読んで、同じ物を書く | 同上 |
| スクラブ・seek | 直近の checkpoint(30 フレームごと)か入点から、その層だけを t の手前まで順に描く | 最大 29 歩 |
| 書類を編集した | 状態を捨てて入点からやり直す(履歴は書類の関数) | 入点から t まで |

だから**同じ時刻は何度描いても、どの順で描いても同じ絵**(審判は `feedback_is_a_recurrence_from_the_in_point`)。
TouchDesigner / AviUtl の「スクラブすると変わる」型でも、AE の CC Time Blend でもない。
コーデックの GOP(checkpoint + 再生)と同型。

まだ無い物: 板に焼けない層(網・点群・下の合成を読む列)の feedback は画面の道で 1 歩ずつは進むが、
スクラブでは辿り直さず初期条件に戻る。合体後(Group / CompRoot)の別時刻は予約のまま。

datamosh はさらに別トラックで、codec 領域の台帳が
[decision-index.md](decision-index.md)(`M5-DATAMOSH-P0` = `DONE / PRIVATE PROBE`・`BUILD FORBIDDEN`)にある。
層の絵の別時刻だけが繋がっていて、合体後の別時刻と再帰はまだ**」である。

## 7. まだ無い物

- **合体後(Group / CompRoot)の別時刻**(`CompLookbehind` の本来の対象)と、**再帰のフィードバック**。
  §6 の通り設計は 2026-07-10 に済んでいる。層の絵の別時刻(§8)は、その入口の最初の 1 本。
- **貼る窓**は作らない。各自の editor で書き、file を置く。
- **Shadertoy の Buffer A..D をそのまま貼る**(1 file に複数 tab を書く取り決め)。ISF の `PASSES`
  に写せるので、器はもう在る。
- **音**(`iChannel` に音を入れる型)。
- naga の GLSL frontend が読めない書き方。通らなければ理由が出る。

## 8. 別の時刻の絵を読む — `TIME_OFFSET`

image の欄に `TIME_OFFSET` を書くと、**ホストがその時刻の層の絵を作って渡す**。効果は何も覚えない。
値は**数値**(秒。負が過去。作者が固定)か、**float 欄の名前**(その欄が普段の仕組みで Inspector に出て、
利用者が回す)。同梱の実例は [`vism/time_difference.fs`](../motolii/crates/motolii-render/vism/time_difference.fs)。

```glsl
/*{ "ID": "motolii.time_difference", "STAGE": "pass",
    "INPUTS": [
      { "NAME": "inputImage", "TYPE": "image" },
      { "NAME": "past",   "TYPE": "image", "TIME_OFFSET": "offset" },
      { "NAME": "offset", "TYPE": "float", "DEFAULT": -0.2, "MIN": -5.0, "MAX": 5.0 } ] }*/
void main() {
    gl_FragColor = abs(IMG_THIS_PIXEL(inputImage) - IMG_THIS_PIXEL(past));
}
```

### 作法(ホスト側)

- **「時刻 t′ の層の姿」を作るのは Document の resolve 1 箇所だけ。** ホストは `view.resolved_layers(t′)` で
  層を引き直し、その姿で絵を作る(`engine/render.rs` の `sources_at_other_times`)。`source_time` だけを
  手でずらすと、mask やキーフレームが t のままの継ぎ接ぎになる — やらない。
- 別の時刻の読みは**別の流れ**(層の番号を変えて復号器の流れを分ける)。同じ流れで読むと、今の絵まで
  巻き添えで上書きされる。
- 読んだ絵は**写しを取る**。素材の texture は時刻ごとに同じ 1 枚へ上書きされるので、写した物だけが
  「あの時刻の絵」でいられる。写しの命令は**即 submit しない** — 復号したコマの転送は frame 共通の
  encoder に積まれ、流れるのは `before_submit` の中。先に打つと、届いていない texture を写す
  (冷えていれば零、暖まっていれば前のコマ。**同じ時刻の絵が辿り方で変わる** — 2026-09-12 にこれで
  1 度落ちた)。
- 届かなかった時に**今の絵で代用しない**。理由を挙げて層の失敗にする。

### 審判

`engine/render.rs` の `time_reference_is_deterministic` — いきなり飛んだ時と、頭から辿った時で、同じ時刻の
絵が**完全一致**すること(実 GPU、毎コマ変わる素材)。素の動画で同じ事を先に確かめる兄弟の test が並ぶ。

### 限界

- 読めるのは**自分の層の絵**だけ。合体後の別時刻(下の層ごと)は予約のまま。
- 名指した欄が無い(または float でない)場合は、黙って 0 にせず名前を挙げて断る。
- 費用は、ずれ 1 つにつき復号 1 回 + 写し 1 枚。cache はまだ効かない。
- 速度を変えた層(time stretch)は、素材側のずれが comp の秒とは一致しない。

## 9. shader へ届く欄と uniform(2026-09-12 に全部開けた)

| 宣言 | 窓 | shader |
|---|---|---|
| `float` / `long` / `bool` | 数・選択・真偽 | `float` |
| `point2D` | 点(pad + X/Y の枡) | `vec2`。2 成分とも届く |
| `point3D` | 点(X/Y の枡。z は既定のまま — 窓に vec3 の部品が無い) | `vec3` |
| `color` | hex の欄 | `vec4`。4 成分とも届く |
| `image` | 層の絵 / `TIME_OFFSET` の別時刻 / `PASSES` の中間 buffer | `sampler2D` |
| `RENDERSIZE` `PASSINDEX` | — | ホストの uniform |
| **`TIME` `TIMEDELTA` `FRAMEINDEX` `DATE`** | — | ホストの時計(comp の秒・1/fps・コマ番号・0)。読む効果だけ時刻で焼き直す |

多成分の欄は成分ごとに 1 つの f32 で運ぶ(`name`, `name.1`, `name.2`, `name.3` — `effects::component_key` が
唯一の綴り)。`SUBTYPE: TIME` の欄は**利用者がキーフレームを打つただの float**で、時計は流れない(AE の
Evolution と同じ)。時計が要るなら `TIME` を読む。

まだ無い物: 別の層の絵(層を指す欄)、compute shader・storage buffer、音。

## 10. 下の合成を読む — `BACKDROP_INPUT`(pass)

2 枚目の image に `BACKDROP_INPUT` で名指すと、**その層の下に合成された絵**が入る。層を指す欄は無い —
北極星(アライトモーション)と同じで、「真下 1 枚」は `clip_to_below`、「下の全部」はこれ。同梱の
**Background Copy**(`motolii.background_copy`)は自分の絵を下の合成にする 1 行:

```glsl
/*{ "ID": "motolii.background_copy", "STAGE": "pass", "BACKDROP_INPUT": "backdrop",
    "INPUTS": [ { "NAME": "inputImage", "TYPE": "image" }, { "NAME": "backdrop", "TYPE": "image" } ] }*/
void main() { gl_FragColor = IMG_THIS_PIXEL(backdrop) * IMG_THIS_PIXEL(inputImage).a; }
```

写すのは**自分の形(α)の中だけ**。層は形を持つので、形の外は触らない(窓ぶん全部を写すと、続く効果が
画面全体に掛かる)。

### 意図で読む

| 型 | 意図 | 変わる側 | 決める側 | 自分の絵 |
|---|---|---|---|---|
| レンズ | 背後を、自分の形で乱す(Displacement を一番上に) | 下の合成 | 自分 | 出ない |
| 切る／混ぜる | 自分を、相手で加工する(Set Matte・Calculations) | 自分 | 下 | 出る |

レンズは「Background Copy → 歪ませる効果(自分の形は `inputImage` で読む)」の 2 段で書ける。
実態は無く、変化だけが見える。

### 作法と代償

- 下の合成は合成の途中でしか決まらないので、この列は層の絵へ**焼かず、run の窓で効く**(§8 の Pass と同じ
  代償: その層は平らな 1 枚になる)。
- 2 枚目の image は 1 つだけ、`TIME_OFFSET` とは併用しない。
- 下に何も無ければ透明が入る。
- 審判は `render_effects.rs` の `background_copy_then_gain_brightens_what_is_below_and_hides_the_layer`。

## 11. 色の規約 — 効果の列は乗算済み(2026-09-13)

効果(pass・warp)が受け取る絵は**乗算済み線形**。累算器・rerun・Skia・Nuke と同じ、普通の方法。
ぼかし・縮小・段の平均が透明の隣(rgb = 0)を混ぜて縁が黒く沈むのを、効果ごとの当て木ではなく
規約で防ぐ(白の上で白をぼかして縁が沈んだのが発覚の場)。sRGB のまま乗算済みで平均しても足りない
— over が線形でしか成り立たず、縁が 220/255 に沈む(実測)。だから線形。

| 所 | 規約 |
|---|---|
| 層の素材(文字・形・静止画・動画) | 非乗算 sRGB(2026-09-03 の法のまま。置く時に shader が decode → 乗算) |
| **効果の列(pass・warp)の入出力** | **乗算済み線形**(Rgba16Float)。素材は最初の効果の前で 1 度だけ写す |
| 列の出口・置く時 | 乗算済み線形(rerun の `AlreadyPremultiplied`) |
| 下の合成(`BACKDROP_INPUT`) | 累算器そのまま(乗算済み線形) |

写し替えは `vism/material_encoding.wgsl` 1 箇所(入口の素性 = 符号化か・乗算済みか)。
見た目に出る差: **色を掛ける効果は線形で掛かる**(gain 2 倍は sRGB の 100 → 139 であって 200 ではない)。
閾値・輝度で判定する効果は線形の乗算済みの値で判定する。生成する効果(gradient・tri_led)の
出力も線形として置かれる。不透明で色を掛けない効果(ぼかし・歪み)は違いが出ない。

貼る shader(Shadertoy / ISF)も乗算済みで受ける。非乗算が要る演算(色相・レベル補正)は
`rgb / max(a, 1e-5)` で戻してから掛け、`* a` で戻す — Nuke の unpremult / premult と同じ。

審判: `render_effects.rs` の `a_blurred_layer_fades_at_its_edge_without_going_dark_on_the_baked_path` と
`a_blurred_copy_fades_at_its_edge_without_going_dark`(白の上の白の縁が沈まない、実 GPU)。

## 12. 層を指す — `TYPE: layer` と image の `LAYER`

既定は「下」(§10)。それでも相手を名指ししたい時(Set Matte の相手が離れた所にある等)のために、
**層を指す欄**がある。欄の TYPE は `layer`(Motolii の拡張。ISF には無い)、値は LayerId。
image に `"LAYER": "<欄の名前>"` を書くと、その欄が指す層の**同じ時刻の絵**が入る。

```glsl
/*{ "ID": "motolii.set_matte", "STAGE": "pass",
    "INPUTS": [ { "NAME": "inputImage", "TYPE": "image" },
                { "NAME": "matte", "TYPE": "image", "LAYER": "layer" },
                { "NAME": "layer", "TYPE": "layer" } ] }*/
```

- 窓は**カメラの target と同じ部品**(他の層の一覧、先頭に None)で描く。Dart はその部品を共有しただけ。
- ホストは「別の時刻」「下の合成」と同じ道: `view.resolved_layers(t)` から相手を引き、写して 2 枚目に束ねる。
- **自分自身と無い層は断る**(理由を挙げて層の失敗に)。黙って今の絵で代用しない。
- 相手の絵は**相手の効果まで掛かった後**の絵(`clip_to_below` と同じ答え)。
- 2 枚目の image は 1 つだけ。`TIME_OFFSET` / `BACKDROP_INPUT` / `LAYER` は併用しない。
- 同梱: **Set Matte**(`motolii.set_matte` — Alpha / Luminance / 反転、Stretch)。

審判: `render_effects.rs` の `set_matte_cuts_by_the_layer_the_user_picked`。
