# Shadertoy の取り込み — 貼れば棚に出る

作成日: 2026-09-12

状態: **Shadertoy(`mainImage`)は実装済み。Buffer 複数枚・音・`iChannelResolution` / `iChannelTime` は未対応。貼る窓(editor)はまだ無い。**

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
| `iTime` | `SUBTYPE: TIME` の欄。host が時刻を流す |
| `iChannel0..3` | **使われている物だけ** image 入力の欄になる |
| `iMouse` | 使われていれば point2D の欄 |
| `iFrame` / `iTimeDelta` / `iFrameRate` / `iDate` / `iSampleRate` | `iTime` と 60fps から組む定数 |

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
| **フレーム跨ぎの持ち越し**(`PERSISTENT`・Shadertoy の Buffer 帰還) | **書けない。法で断る** |

`PERSISTENT` を採らないのは能力不足ではなく裁定である — 任意の時刻へ飛べる道具なので、
持ち越すと**絵が操作の履歴に依存する**(同じ時刻を 2 回描くと違う絵になる)。断り文は
[`isf/mod.rs`](../motolii/crates/motolii-render/src/compositor/effects/isf/mod.rs) に書いてある。

そのため、軌跡・流体・反応拡散といった**自分の前フレームを読む**表現はこの口からは書けない。
時間を使う表現は「自分の出力」ではなく「**入力の別の時刻**」を読む形なら決定的に書ける
(AE の Echo / Time Difference と同じ型)。その口はまだ無い。

## 7. まだ無い物

- **入力の別の時刻を読む口**(§6 の代わりの道)。
- **貼る窓**。今は file を置く。
- **Shadertoy の Buffer A..D をそのまま貼る**(1 file に複数 tab を書く取り決め)。ISF の `PASSES`
  に写せるので、器はもう在る。
- **音**(`iChannel` に音を入れる型)。
- naga の GLSL frontend が読めない書き方。通らなければ理由が出る。
