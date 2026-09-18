# GPU 先行のモーショングラフィックス道具の一覧 — 「鍵と層の中で、物ごとの法を text で書き、GPU で走るか」(2026-09-18 取得)

課題: 利用者「wgsl 上で既に Cavalry みたいな気持ちいいモーグラをやろうとしてるプロジェクトは結構ありそう」。5 軸で並べる: (1) 鍵とタイムライン (2) 層(素材・文字・動画) (3) 物ごとの法を text で書けるか・言語 (4) GPU(compute / 頂点 / 画素) (5) 開いているか。一次資料のみ(公式 docs・README・changelog)。子調査 7 本の結果を集約(各行の出典は末尾)。

## 0. 要約(5 行)

1. **5 軸を全部持つ物は無い。** 鍵と層を持つ道具(Rive・Cavalry・Notch・AE・Fable・Jitter)は法の text が CPU 側(Luau・JS・SkSL)か、GPU でも閉じている(Notch の HLSL、AE の GPUDog)。GPU で法を書ける物(compute.toys・TouchDesigner・three.js TSL・use.gpu・Bevy・nannou)は鍵と層が無いか編集器が無い。
2. **形が一番近いのは Rive**: タイムライン + 層 + Luau の物ごとの script(2026-01 GA)+ **WGSL の頂点/画素シェーダ(2026-09-04 GA)**+ 自社 GPU renderer。ただし編集器は非公開 SaaS、.riv は再生用、compute 不可。
3. **仕組みが一番近いのは Notch**: 2026.2 で Clone Effector / Particle Effector / Deformer に HLSL を書ける(= 物ごとの法を GPU で)。ただし Windows・閉じた .dfx・年額 $279〜。
4. **AE 側も動いている**: GPUDog(2026、AE の中で WGSL を live 編集、Metal/Vulkan/D3D12)、VKO Shader(GLSL、2026-08)、ISF4AE(MIT)。ただし画素の効果で、物ごとの法ではない。
5. **Rust/wgpu の OSS 編集器は 3 本が生きている**: Lumit v0.5.0(2026-09-16、custom shader nodes は開発中、GPLv3)、Graphite(keyframe は late 2026 予定)、Friction(ECMAScript expression + shader effects)。「物ごとの法が text で棚にあり、書類が開いている」形は無い。

## 1. 表(5 軸)

| 道具 | 何か | (1) 鍵/TL | (2) 層 | (3) 物ごとの法・言語 | (4) GPU | (5) 開放性 | 最新版 | URL |
|---|---|---|---|---|---|---|---|---|
| Rive | 2D/UI の編集器 + runtime | あり | 画像・文字・音声(動画なし) | Luau(Node/Converter/Layout/PathEffect の protocol、2026-01 GA)+ WGSL の vertex/fragment(2026-09-04 GA、compute 不可、bind group 4) | 自社 renderer(Metal/Vulkan/D3D/GL/WebGL/WebGPU) | runtime MIT、.riv 仕様公開(format 7)、編集器は非公開 SaaS、書き出し有料 | changelog 連続配信 | https://rive.app/docs/scripting/wgsl-shaders |
| Cavalry | 手続き型 2D | あり | 画像・文字・動画 | JavaScript Layer/Utility(ctx.index/ctx.count、ctx.time 無し)+ SkSL(GLSL 系、uniform time/resolution) | Skia、OpenGL 4.1 | .cv = JSON、非公開、個人無料(2026-02 Canva) | 2.7.2(2026-04) | https://cavalry.studio/docs/nodes/general/javascript-layers/ |
| Notch | リアルタイム VFX/モーション | あり(bezier/linear/step) | Precomp・Render Layer・Video Loader・Text | HLSL(.fx)を Post FX / **Clone Effector / Particle Effector / Deformer**(2026.2)に、JS は欄の操作だけ | GPU 全段 | .dfx/.dfxdll、closed、Windows、Indie $279/yr | 2026.2(2026-06) | https://manual.notch.one/2026.2/en/docs/workflows/working-with-custom-shaders/ |
| After Effects 26.5 + GPUDog / VKO / ISF4AE | 層の道具 + シェーダ plugin | あり | あり | Expression = JS(ES2018)。GPUDog = **WGSL live editor**(GLSL/ISF/Shadertoy 取り込み)、VKO = GLSL、ISF4AE = ISF/GLSL(MIT) | GPUDog: Metal/Vulkan/D3D12 | AE 非公開、plugin は有料(ISF4AE は MIT) | AE 26.5(2026-09)、VKO 1.0(2026-08) | https://gpudog.dev/ https://github.com/baku89/ISF4AE |
| TouchDesigner | ノード型 | あり(Animation COMP) | operator(Text TOP・Movie File In TOP) | GLSL TOP(pixel/compute)、GLSL MAT(TDInstanceID())、Python expression、Script CHOP | Vulkan、compute あり | .toe binary、closed、非商用無料 | 2025.33230(2026-09) | https://docs.derivative.ca/GLSL_TOP |
| Cables.gl | ノード型 WebGL | あり(keyframe editor) | op(TextTexture・VideoTexture) | CustomShader_v2(GLSL)、custom op(JS) | WebGL、WebGPU は拡張 op、compute 未 | MIT、patch JSON | standalone 0.11.1(2026-08) | https://cables.gl |
| Unreal 5.8 Motion Design | 3D の モーション mode | あり(Sequencer) | Text・SVG・Shape・Media Plate・Cloner/Effector | Effector 自体は欄のみ。text は Material Custom(HLSL)と Niagara CustomHLSL / Scratch Pad(data channel で effector へ) | Niagara GPU/CPU、Simulation Stages は GPU | .uasset、ソース公開だが EULA | 5.8(2026-06、UE5 最後の major) | https://dev.epicgames.com/documentation/en-us/unreal-engine/motion-design-cloners-and-effectors-in-unreal-engine |
| Unity 6.6 VFX Graph | 粒子 | あり(Timeline) | あり | Custom HLSL Block(inout VFXAttributes、particleId/age/seed) | compute | プロプライエタリ | 6.6(2026-08) | https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@17.4/manual/Block-CustomHLSL.html |
| three.js r186 TSL + WebGPURenderer | ライブラリ | API のみ(AnimationMixer/KeyframeTrack)、編集器なし | なし | TSL(JS → WGSL/GLSL)、instanceIndex・time・deltaTime、Fn().compute(count) | compute + 頂点 + 画素 | MIT | r186(2026-09-08) | https://github.com/mrdoob/three.js/wiki/Three.js-Shading-Language |
| Babylon.js 9.27 | ライブラリ | Animation + ACE(curve editor) | なし | NME はノード、ShaderMaterial(WGSL/GLSL)、ComputeShader(WGSL) | compute(WebGPU) | Apache-2.0 | 9.27.0(2026-09-17) | https://doc.babylonjs.com/features/featuresDeepDive/materials/shaders/computeShader |
| use.gpu 0.20 | 宣言的 WebGPU | API(<Animate> keyframes)、編集器なし | data 駆動の layer | WGSL/GLSL を text で、linker が結合(@link fn) | compute + render | MIT、alpha | 0.20 / 0.21-dev | https://gitlab.com/unconed/use.gpu |
| compute.toys | WebGPU compute の遊び場 | なし | なし | WGSL(Slang→WGSL)、compute = スレッド id × time | compute | MIT | 2026-05 push | https://compute.toys |
| Shadertoy / Hydra / KodeLife / Bonzomatic / Shader Park / WGSL playground 12 本 | 画素の遊び場・live coding | なし | なし | GLSL / JS→GLSL / HLSL・Metal / WGSL | 主に画素、KodeLife と一部は compute | 各種(CC BY-NC-SA、AGPL、closed、MIT/BSD) | 2023〜2026 | 末尾 |
| Jitter / Fable / Linearity Move / Spline | Web/デザイン系の編集器 | あり(Jitter はイベント型) | あり | なし(Jitter は既製 shader を click + AI prompt、Spline は JS Code API) | Spline は WebGPU 既定(V2 2026-08) | 非公開クラウド | 2026 | https://jitter.video https://www.fable.app https://spline.design |
| Motion Canvas / Theatre.js / Remotion / Manim | コードの動画 | generator / sequence / useCurrentFrame / play() | Img/Video/Txt(Motion Canvas)、React(Remotion) | TS/JS/Python、Motion Canvas は GLSL fragment(実験) | Canvas 2D / three / Chromium(Remotion 5 は ANGLE) | MIT / Apache+AGPL / 独自 / MIT | 3.11 / 0.7.0(2023) / 4.0.526 / 0.21 | 末尾 |
| Processing 4 / p5.js 2.x / nannou / Bevy(+motiongfx) / Vello | 創作コーディングと Rust の土台 | なし(Bevy は AnimationClip、motiongfx は Rust の timeline) | なし | Java / JS(p5.strands→GLSL、WebGPU 実験)/ Rust + WGSL / Rust + WGSL(WESL へ)/ 口なし | GL / WebGL(WebGPU 実験)/ wgpu / wgpu / cpu・gpu・compute の 3 実装 | LGPL / LGPL / MIT・Apache / MIT・Apache / Apache・MIT | 4.5.6 / 2.3.3 / 0.20 / 0.19 / 0.10 | 末尾 |
| Blender 5.2 GN / Houdini 22 | DCC | あり | あり | ノード(Index・Scene Time)/ VEX wrangle(@ptnum・@Time、CPU)+ OpenCL SOP(GPU) | GN は CPU / OpenCL SOP は GPU | GPL / 商用 | 5.2 LTS / 22.0 | 末尾 |
| Rust/wgpu の OSS 編集器: Lumit / Graphite / Friction / Glaxnimate | 2D モーション編集器 | あり / late 2026 予定 / あり / あり | あり | custom shader nodes 開発中 / code editor for custom nodes は Beta 1 予定 / ECMAScript expression + shader effects / Python | wgpu / Vello / Skia+GL / Qt | GPLv3 / MIT・Apache / GPL-3 / GPL | 0.5.0(2026-09-16)/ Alpha / 1.0.0-rc.3 / 0.6.0 | https://github.com/luminalmvm/Lumit https://graphite.art https://friction.graphics |

## 2. 形が一番近い 3 つ(理由)

1. **Rive** — 5 軸のうち 4 つ(鍵・層・物ごとの script・GPU の text)。2026-09-04 に WGSL が編集器で GA。欠けるのは (5): 編集器が閉じていて書類が再生用。compute も無い。
2. **Notch** — 「Effector に HLSL を書く」= 物ごとの法を GPU で、が 2026.2 で製品に入った。欠けるのは (5) 全部(Windows・closed・年額)と、法が製品の Effector の枠内。
3. **TouchDesigner** — GLSL compute + instancing(TDInstanceID)+ タイムライン。欠けるのは層(operator であって書類でない)と (5)(.toe binary、有料)。

## 3. 見つからなかった物

- 「物ごとの法が text で棚にあり、鍵と層の書類の中で GPU で走り、書類も編集器も開いている」物: 無し。
- Rive の compute、Cavalry の ctx.time、Fable の script(検索要約のみ)、Jitter の shader の中身、Cables の WebGPU 本体化、Unreal Effector の text 入口、Epic の custom-hlsl-in-niagara 個別ページ(JS 描画)、helpx/aescripts/fable.app(403/DNS)。
- 「Processing 5」は存在しない(libprocessing = Rust/Bevy の R&D)。

## Sources(子調査 7 本の出典を統合)

- https://athenodoros.github.io/wgsl-playground/
- https://bevy.org
- https://cables.gl
- https://compute.toys
- https://derivative.ca
- https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/simulation/simulation_zone.html
- https://docs.derivative.ca
- https://docs.unity3d.com/Packages/com.unity.visualeffectgraph@17.4/manual/Block-CustomHLSL.html
- https://friction.graphics
- https://github.com/Gargaj/Bonzomatic
- https://github.com/compute-toys/compute.toys
- https://github.com/hydra-synth/hydra
- https://github.com/linebender/vello
- https://github.com/luminalmvm/Lumit
- https://github.com/voxell-tech/motiongfx
- https://graphite.art
- https://hexler.net/kodelife
- https://hydra.ojack.xyz
- https://nannou.cc
- https://p5js.org
- https://processing.org
- https://shaderpark.com
- https://www.shadertoy.com
- https://www.sidefx.com/docs/houdini/vex/snippets.html
