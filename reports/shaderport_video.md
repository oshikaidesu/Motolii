# shaderport の VideoTexture と、動画を入れる道（10/26）

shaderport（quizzyjoy/shaderport、Apache-2.0）は、shader-effects-inc/shaders（v4.0.4、TypeGPU で書かれた 200 の効果）の WGSL を書き出す道具を持つ。Codex での試し（`~/Documents/Codex/2026-10-26/new-chat-2/work/shaderport-trial`）で 193 本が書き出せ、Blur（2 段）と Plasma が Motolii の Filament の Dawn の装置で描けた。VideoTexture は `texture_external` を求めるので、screen_door（入力は `texture_2d`）に渡すと Dawn の検証で止まる。

## 確かめた事

- VideoTexture の入力は `@group(1) @binding(0) var video_0: texture_external;`、読み方は `textureSampleBaseClampToEdge`（書き出した `VideoTexture__final.wgsl` の 46・78 行）。
- **Filament の WebGPU の層に外部テクスチャの口は無い**：`createTextureExternalImage*` も `SAMPLER_EXTERNAL` も「External WebGPU Texture is not supported」で止まる（`filament/backend/src/webgpu/WebGPUDriver.cpp:657-678`、`WebGPUDescriptorSetLayout.cpp:160-162`）。
- **Motolii には動画をテクスチャにする口が既にある**（`filament/video.h`）：GStreamer がハードの解読器で NV12 を作り、Mac の IOSurface のまま、Dawn の SharedTextureMemory で Filament と同じ装置に共有。明るさ（R8）と色（RG8）の 2 枚を Filament に `import` し、flat.mat で RGB にする。CPU を通らない。
- **Dawn にも口がある**：`ExternalTextureDescriptor` が plane0・plane1 の 2 枚と YUV→RGB の数から `texture_external` を作る。見本は Dawn の試験 `ExternalTextureTests.cpp:108-136`（`InitExternalTextureDescriptor`）、数は `src/dawn/common/ExternalTextureParams.h:50` の `ComputeExternalTextureParams`（BT.709 などの名前から）。
- **ただし、そもそも `texture_external` は要らない**：upstream 自身が絵を連れてくる入口を 2 つ持つ（`packages/core/src/std/paint/media.ts`）。
  - `imageMedia`（ImageTexture）：普通のテクスチャを `texture_2d<f32>` で `textureSample`。sRGB の解き方はシェーダーの中（作者のメモ：「外部テクスチャの解き方の media 版」）。
  - `videoMedia`（VideoTexture）：ブラウザの `<video>` を毎コマ `importExternalTexture` でコピーせずに読むための形。
  - upstream 自身の試験の見本（`__snapshots__/shader-ImageTexture.test.ts.snap` と `shader-VideoTexture.test.ts.snap`）を比べると、違いは入口の型（`texture_external` ↔ `texture_2d<f32>`）、読み方（`textureSampleBaseClampToEdge` ↔ `textureSample`）、つまみの `loop_` だけ。
  - Motolii では動画のフレームは既に Filament の普通のテクスチャになるので、upstream の ImageTexture の形で足りる。
  - ImageTexture は Codex の書き出しでは失敗した（7 本のうち 1 本、理由はログに無い）が、WGSL は upstream の試験の見本にある。

## 次の壁（まだ動かしていない）

- 入力の並びが違う：upstream の WGSL は、つまみ・絵・サンプラーを group 0・1・2 に分ける。screen_door は Filament の描く係の口を通るので、group 0 に「絵 0・サンプラー 1・つまみ 2」と決まっている（`filament/dawn_probe/screen_door.h:8-10`、`screen_door.cpp:96-113`）。
- Codex の試しは、ここを WGSL の文字の書き換えで合わせていた（`blur.cpp` の `rebind`）。それは翻訳の層なので、既存のやり方を探すのが先。
