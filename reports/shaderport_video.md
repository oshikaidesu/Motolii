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

## 動かした：作者の VideoTexture に本物の動画を食わせた（10/26）

- 並びの壁は、Filament の描く係を通さずに越えた。Filament の描く係は入口の名前を `"main"` に決めうちしている（`WebGPUPipelineCache.cpp:287、358`）ので、作者の `final_1` を通すには文字の書き換えが要る。代わりに、Motolii が ThorVG で使う道（`gpu_door`：Filament の Dawn の装置を借りる）で、作者の WGSL を Dawn に直接渡した。group の並び（作者の composer が決める uniforms 0・textures 1・samplers 2、`composer.ts:61-62`）も入口の名前も、作者のまま。
- 道具：`filament/dawn_probe/video_external.sh <動画> <効果.wgsl> <出力先>`（`video_external.cpp`）。並べただけで、中身は全部借り物。
  - 動画のコマ：GStreamer（`filesrc ! decodebin3 ! appsink NV12`）、IOSurface を video.h と同じく SharedTextureMemory（`R8BG8Biplanar420Unorm`）で Filament の装置に共有。
  - `texture_external`：Dawn の ExternalTexture。明るさ（Plane0Only）と色（Plane1Only）の 2 面、変換の数は `ComputeExternalTextureParams`（BT.709）。作り方は Dawn の試験 `ExternalTextureTests.cpp:108-136` のとおり。`doYuvToRgbConversionOnly` で色は動画のまま（GStreamer の videoconvert と同じ考え方）。
  - 頂点：作者が描くときの TypeGPU 0.12.3 の `fullScreenTriangle`（MIT）。パイプラインは作者の WGSL 自身の並び（layout auto）。
- 結果（`reports/splat_orbit.mp4` の最初のコマ、1280×720）：
  - 装置の誤り 0。作者の VideoTexture の WGSL は 1 文字も変えていない。
  - GStreamer 自身の `videoconvert` で RGB にした同じコマと比べて、上下を逆にすると平均の差 2.0、最大 20（0〜255）。少しだけ明るい（平均 90.6 対 88.7）。差は YUV→RGB の計算の違い（Dawn と GStreamer、色の半分の解像度の補い方）と見ているが、分けて確かめてはいない。
  - 絵：`reports/shaderport_video_external.jpg`（左から GStreamer の答え、作者の WGSL で描いたそのまま、それを上下逆にした物）。
- まだ分からない事：上下の向き。書き出された WGSL は `1.0 - in.uv.y` で uv を 1 回ひっくり返す（作者の composer の flipY、初期値 true、`composer.ts:241-247`）。作者の頂点で描くと上下逆になった。一方、作者のメモ（`gpu/kit/media.ts:13-15`）は「composer の uv なら `1 - y` なしで自然な向き」と書く。ブラウザでの作者の向きの合わせ方は、まだ確かめていない。
- まだの事：VideoTexture の上に効果（Blur など）を重ねること。作者の効果は子の絵（rtt）を読むので、重ねるには作者の組み合わせ（composition）の形で描く必要がある。
