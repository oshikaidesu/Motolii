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

ISF の `isf_FragNormCoord` は**上端が 1**、texture の `v` は**上端が 0**。素直に `fragCoord = isf_FragNormCoord * RENDERSIZE` と渡すと、`texture(iChannel0, uv)` が上下逆の位置を読む。実写に当てて初めて分かる(文字が裏返る)。

そこで `fragCoord` は `y` を裏返して渡す。正しさの証明は**恒等**で取った — `fragColor = texture(iChannel0, uv)` の結果が元の絵と**完全一致**(PSNR ∞)すること。見張りは `shadertoy.rs` の `the_picture_is_read_right_side_up`。

## 6. まだ無い物

- **貼る窓**。今は file を置く。窓から貼って即見えるのは次。
- **Buffer A..D**(複数パスの Shadertoy)。ISF は複数パスを持てるので、器はある。
- **音**(`iChannel` に音を入れる型)。
- naga の GLSL frontend が読めない書き方。通らなければ理由が出る。
