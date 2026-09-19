# 拡張性の棚 — 外の code の池と、Motolii へ入る道(2026-09-18)

棚(vism/)は WGSL(WESL の module + import)。外の code は「排除せず、ホストが写す」(TL に無い in は口にする)。
ここは**どこから取れるか**と**どの道で入るか**の一覧。道が出来た物は ✅、道が決まって未実装は ◻、人(LLM)が写す物は ✎。

## 1 段で通る(naga が読む)

| 池 | 言語 | 場所 | 道 | 状態 |
|---|---|---|---|---|
| Shadertoy | GLSL | https://www.shadertoy.com | 貼る → ISF に写す → naga glsl-in(`isf::shadertoy`) | ✅ |
| ISF の倉庫 | GLSL(ISF) | https://github.com/Vidvox/ISF-Files 、 https://editor.isf.video | .fs をそのまま棚へ | ✅ |
| lygia(GLSL / HLSL / WGSL の 3 言語、animation・easing の束) | GLSL / WGSL | https://lygia.xyz 、 https://github.com/patriciogonzalezvivo/lygia | WGSL 版は module として import、GLSL 版は naga | ◻ |
| glsl-easings | GLSL | https://github.com/glslify/glsl-easings | 関数を `cavalry` module に写す(名前は元のまま) | ◻ |
| The Book of Shaders(shaping functions) | GLSL | https://thebookofshaders.com/05/ | 同上 | ◻ |
| GLSL Sandbox | GLSL | https://glslsandbox.com | Shadertoy と同じ口(uniform 名だけ違う) | ◻ |
| twigl(GLSL ゴルフ) | GLSL | https://twigl.app | 同上 | ◻ |
| Vertex Shader Art(頂点だけの動き) | GLSL | https://www.vertexshaderart.com | 頂点段の hook | ◻ |
| Hydra(ライブコーディング) | JS → GLSL | https://hydra.ojack.xyz | 吐いた GLSL を naga | ◻ |
| SPIR-V(下の 2 段の中継点) | SPIR-V | — | naga spirv-in | ◻ |

## WGSL(そのまま)

| 池 | 場所 | 道 | 状態 |
|---|---|---|---|
| compute.toys | https://compute.toys | toy の口(compute.toys の prelude をそのままの名前で: time / custom / screen / pass_in・out / #storage / #workgroup_count / #include)。mouse・key は欄に、種は Seed 欄に、記憶は in 点から回し直し | ◻(着手待ち) |
| compute.toys 公開倉庫(1000 本、.wgsl + .json + .jpeg) | https://github.com/compute-toys/public-shaders | 同上 | ◻ |
| compute.toys include(std / iq / Dave_Hoskins / nikat / davidar) | https://github.com/compute-toys/include | WESL の module に写す(`#include` → `import`) | ◻ |
| Bevy の shader package | https://github.com/bevyengine/bevy (WESL 移行中) | WESL の package として import | ◻ |

## 2 段で通る(先に WGSL か SPIR-V に落とす)

| 池 | 言語 | 場所 | 道 | 状態 |
|---|---|---|---|---|
| Slang playground / Slang の作品 | Slang | https://shader-slang.org/slang-playground/ | slangc → WGSL(外部道具、`media::tool_command`) | ◻ |
| Unity / Unreal Custom / Notch .fx | HLSL | — | dxc → SPIR-V → naga、または slangc(HLSL 読み)→ WGSL | ◻ |
| rust-gpu | Rust | https://github.com/Rust-GPU/rust-gpu | SPIR-V → naga | ◻ |
| Cavalry の shader node / Flutter の fragment shader | SkSL | https://skia.org/docs/user/sksl/ 、 https://cavalry.studio/docs/ | Skia の SkSL compiler の WGSL backend(外部道具) | ◻ |

通らない: Metal(MSL)は読む側が無い。Houdini VEX・TouchDesigner の Python は言語が違う(✎)。

## shader ではないが写せる(✎ 人 = LLM が写す。機械の変換ではない)

| 池 | 場所 | 写し方 |
|---|---|---|
| Processing / p5.js の sketch | https://openprocessing.org 、 https://p5js.org/examples/ 、 https://processing.org/examples/ | 棚の `processing` module が同じ名前(map / norm / random / noise / TWO_PI)。for 文は番号 k に、draw() は時刻に |
| Cavalry の JavaScript Layer(`ctx.index` / `ctx.count`) | https://cavalry.studio/docs/nodes/general/javascript-layers/ | block(k, time) と 1 対 1。ノードは `cavalry` module の関数 |
| AE の expression(`index` / `time` / `wiggle`) | — | 同上。wiggle = cv_noise、loopOut = fract |
| GSAP / CSS の easing の名前 | https://gsap.com/docs/v3/Eases/ | `cv_ease` に既に入っている |

## 規則

- 名前は出典のまま(Cavalry の語、Processing の語、compute.toys の prelude)。Motolii の名前を発明しない。
- 外部の compiler(slangc・dxc・Skia)は同梱せず `media::tool_command` で呼ぶ。他の OS の枝も同時に。
- TL に無い in(mouse・key・壁時計・種)は排除せず口にする(記憶: convert-never-exclude)。
- 調査: [gpu-mograph-survey](reviews/2026-09-18-gpu-mograph-survey.md)、[motion-code-survey](reviews/2026-09-18-motion-code-survey.md)(進行中)。
