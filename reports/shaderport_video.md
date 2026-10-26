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

## 動画に作者の効果を掛けて、毎コマ動かした（10/26）

- 子を VideoTexture にした組み合わせを、作者の composer（`composeNodeTree`）自身に作らせた。道具は shaderport の書き出し（`tools/export-wgsl/export.test.ts`、Apache-2.0）をそのまま使い、変える所だけ直しのファイルにした（`filament/tsl/shaderport_frames.patch`：子を `SP_BASE` で選ぶ、`SP_PICK` の効果だけ書き出す、`SP_FRAMES` コマ分のつまみの値を作者の値の置き場から TypeGPU 自身に詰めさせる）。毎コマの `_animTime` は作者のレンダラーと同じく「dt × 速さのつまみ」で進める（`gpu/index.ts` の updateAnimatedTime）。動かし方も shaderport の `run.mjs` と同じ（作者の試験の場所に置いて vitest）。`filament/tsl/shaderport_export.sh`。
- 借りた 2 つは版を固定して取り寄せる（`filament/tsl/sources.tsv`：shader-effects-inc/shaders 81fd4ec MIT、quizzyjoy/shaderport ab89086 Apache-2.0）。
- 作者の composer が作った形：Glitch・VHS・Halftone・CRTScreen・PixelSort・DataMosh・Glass・Pixelate・ChromaticAberration は 2 段（動画を `rtt_0` に描き、`rtt_0` に効果）、Kaleidoscope は 1 段（ゆがみなので動画を直接ゆがめて読む）。
- 再生役：`filament/dawn_probe/video_external.sh <動画> <書き出し> <効果> <コマ数> <出力.mp4>`。毎コマ、GStreamer の次のコマ → `texture_external` → そのコマのつまみの値 → 作者の段を作者の順に描く。入力の繋ぎ方は WGSL の宣言の名前のまま（uniforms、video_N、rtt_N、作者の 4 つのサンプラー）。子の描き先は作者と同じ `rgba16float`。
- 結果（`reports/splat_orbit.mp4`、1280×720、60fps、360 コマ）：11 本とも装置の誤り 0。

| 効果 | 最初に作る分 | 1 コマ（命令を作り始めて GPU が終わるまで）平均 | 一番遅いコマ |
|---|---|---|---|
| VideoTexture | 9.8 ms | 1.84 ms | 19.63 ms |
| Glitch | 343.9 ms | 2.34 ms | 29.63 ms |
| VHS | 167.9 ms | 2.30 ms | 22.06 ms |
| Halftone | 274.2 ms | 1.55 ms | 10.98 ms |
| ChromaticAberration | 135.6 ms | 1.73 ms | 8.64 ms |
| Kaleidoscope | 191.7 ms | 1.51 ms | 9.21 ms |
| PixelSort | 131.4 ms | 1.76 ms | 30.77 ms |
| CRTScreen | 281.4 ms | 1.80 ms | 21.19 ms |
| DataMosh | 7.6 ms | 1.40 ms | 8.76 ms |
| Glass | 312.5 ms | 2.29 ms | 8.16 ms |
| Pixelate | 291.1 ms | 1.56 ms | 22.39 ms |

  - 平均は 60fps の枠（16.7 ms）の 1/7〜1/12。一番遅いコマが平均を大きく超える原因（最初のコマの組み立てか、解読器を待つ間か）は、まだ分けていない。読み戻し（動画にするため）は時間に入れていない。
  - 絵：`reports/shaderport_on_video.mp4`（4×3、6 秒）と `reports/shaderport_on_video.jpg`。左上から 元の動画、Glitch、VHS、CRTScreen、Halftone、ChromaticAberration、PixelSort、DataMosh、Glass、Pixelate、Kaleidoscope、VideoTexture。
- 向き：2 段の物は上下が正しい（子の段と最後の段で 1 回ずつ uv をひっくり返して戻る：作者の説明 `composer.ts:241-247` の「書いて読む行き来が合う」）。1 段の物（VideoTexture、Kaleidoscope）は上下が逆になる。ブラウザで作者がこれをどう扱っているかは、まだ確かめていない。
- まだの事：Motolii の画面（段の絵、Filament の世界）に入れること。今は確かめの道具が自分で描き先を作り、絵を読み戻して ffmpeg に渡している。

## 石・彫り・3D の 16 本（10/26）

- 同じ道（`SP_BASE=VideoTexture`、360 コマ）で 16 本を描いた。16 本とも装置の誤り 0、1 コマ平均 0.83〜9.64 ms（Form3D 9.64、Crystal 5.08、Stone 4.75、ほかは 1.3〜3.6）。
- 絵：`reports/shaderport_stone_mask.mp4`（4×4、6 秒）、`reports/shaderport_stone_mask.jpg`。左上から Stone、Emboss、Engraving、Crystal／Form3D、Surface3D、Shatter、ObjectTracker／TimeTrail、Watercolor、Chalkboard、ContourLines／Sparkle、Glow、LightLeak、ReflectivePlane。
- 目で見た分け方（作者の絵とはまだ比べていない）：
  - 効いている：Engraving、ObjectTracker（追跡の照準）、Chalkboard、Sparkle、ReflectivePlane、ContourLines と LightLeak（1 段なので上下逆）。
  - このコマでは元とほぼ同じ：Stone、Watercolor、Glow、Shatter（マウスで割る作り）、TimeTrail（前のコマを使う作りだが、再生役が毎コマ描き先を消している）。
  - 形の中だけ：Emboss は作者の初期の形（弧）の中、Crystal は白い円。どちらも「Shape Effects」の仲間。
  - 崩れている：Surface3D は真っ黒、Form3D は帯状。原因はまだ調べていない。
- 装置の誤りが 0 でも、作者どおりの絵とは限らない。

## 金属系は動画を受け取れるか（10/26）

- LiquidMetal・Chrome・Obsidian・BrushedMetal は `role: 'shapeEffect'`。子の絵は取らず、形の距離の絵（SDF）の上に金属を流す。形は作者の数式の 14 種（`gpu/kit/sdf.ts:692-705`）か、前もって作った SDF の `.bin`（`shapeSdfUrl`）。
- `.bin` の形式は分かっている：512×512、距離を [-1, 1] に Uint16 で詰めた物（`sdf.ts:447-471`）。読んだ距離は `r16float` の絵になる（`sdf.ts:474-530`）。ただし作者は `.bin` を自分のサーバー（`/api/plugin/sdf/generate`、`packages/partner/src/index.ts:161-220`）で SVG から作っていて、作り方は公開されていない。
- 作者のライブラリは、絵から距離の絵を作る部品を別の効果の中に持っている：Irradiance のシルエットの種まきと JFA の飛び石（`std/paint/radiance.ts:417-441`、`silhouetteSeedKernel`・`silhouetteStepKernel`）。
- なので、動画を金属にする道は「動画の切り抜き（どこが形の内側か）→ 作者の JFA で毎コマ距離の絵 → LiquidMetal の SDF の絵として差し込む」。切り抜きは、Motolii の video.h のキー（グリーンバックなど）か、別の既存の切り抜きが要る。まだ作っていない。

## 上下の向きを直した（10/26）

- 原因：作者の本番のレンダラーは composer を `flipY: false`・`premultiplyAlpha: true` で呼ぶ（`packages/core/src/gpu/index.ts` の composeOptions、1268-1275）。shaderport の書き出しは何も渡さないので、composer の初期値 `flipY: true`（`composer.ts:241-247、349`）で組まれ、uv が 1 回ひっくり返っていた。2 段の効果は子の段と最後の段で 2 回ひっくり返って元に戻るが、1 段の効果は逆さまのままだった。
- 直し：`filament/tsl/shaderport_frames.patch` で、書き出しも作者の本番と同じ設定で composer を呼ぶ（トーンマップは両方とも初期値の `'linear'`、`gpu/index.ts:463`）。書き出した WGSL から `1.0 - in.uv.y` は無くなった。
- 確かめ：27 本を書き出し直して描き直した（装置の誤り 0、1 コマ平均 0.79〜7.06 ms）。3 秒目のコマを元の動画と比べた相関で、24 本が「そのまま」の向き（+0.42〜+1.00、上下逆にすると -0.30〜-0.73）。Form3D・Kaleidoscope・Surface3D は絵が大きく変わる（または真っ黒）ので数字では判定できない。Kaleidoscope は目で見て向きが合っている。
  - 上の「向き」の節の、1 段の効果が上下逆という結果は、この直しで無くなった。
- 並べた動画も描き直した（`reports/shaderport_on_video.mp4`、`reports/shaderport_stone_mask.mp4`）。
- まだの事：
  - Surface3D は真っ黒、Crystal は白い円、Form3D は波の帯。原因はまだ調べていない。
  - 相関が 1.00 の物（Glitch、PixelSort、DataMosh、Stone、Watercolor、Glow、Shatter、Sparkle、TimeTrail など）は、作者の初期設定のままだと 3 秒目では元とほとんど同じ。効き方がまばらな物（Glitch など）と、効いていない物の見分けは、作者の絵と比べるまで付けない。TimeTrail は前のコマを使う作りなのに、再生役が毎コマ描き先を消している。

## 作者の段を毎コマ回す既存のネイティブの実行役を探した（10/26）：無い

- `filament/dawn_probe/video_external.cpp`（約 330 行）は、段を順に回し、WGSL の宣言の名前で入力を繋ぎ、描き先を管理する再生役で、自作だった。前の節で「並べただけ」と書いたのは誤り。書く前に既存の実行役を探していなかった。
- 後から探した（手下の調べ、URL は開いて確かめた物）：
  - shaderport 自身の HarmonyOS の描き役（`harmony/entry/src/main/cpp/renderer.cpp`、806 行、`SetEffect` で段を順に描く）：OpenGL ES 3.0 と HarmonyOS の窓の上。Dawn では動かない。
  - Wisp（https://github.com/nannou-org/wisp、nannou_isf の後継、MIT/Apache-2.0）：WGSL の段・`@pass(target, persistent, float)`・つまみを毎コマ書き換える、と形は一番近い。Bevy と wgpu の中だけ、C の口なし。
  - wgpu-compute-toy（MIT、compute のみ）、cuneus（MIT、段の組み立ては Rust のコード）、pygfx/shadertoy（BSD-2、Python、Shadertoy の決まった uniform）：どれも wgpu の上で、他人の Dawn の装置は使えない。
  - librashader（MPL-2.0、C の口あり）：受け取るのは slang（GLSL）で WGSL ではない。
  - shaders.com の効果をブラウザの外で動かす物は fframes（MIT）だけ。手で SkSL に書き直した物で、WebGPU ではない。
- 分かった事：Motolii の Dawn の装置の上で作者の段の一式を回してくれる既存の物は無い。形として一番近いのは Wisp の段の書き方で、Motolii の ISF の段（`filament/isf.cpp`、`screen.h` の `// --pass@`）とも同じ考え方。
- Motolii の段の仕組み（screen.h → screen_door）が作者の WGSL を受け取れないのは、screen_door が Filament の描く係を通るため。入力の並びを group 0 に決めうち（vgpu の形）にしていて、入口の名前も `main` に決めうちしている。作者の並び（group 0・1・2）を受け取るには、文字の書き換え（翻訳）か、screen_door をシェーダー自身の並び（layout auto、gpu_door の道）に替えるかになる。

## screen_door を「シェーダー自身の並び」に替えられるか（10/26）：Filament の描く係の中ではできない

- screen_door は Filament の描く係（Driver の Program と DescriptorSetLayout）を通る。そこで作者の WGSL をそのまま描けない理由は Filament の側に 3 つある：
  1. 入口の名前が `"main"` に決めうち（`filament/backend/src/webgpu/WebGPUPipelineCache.cpp:287、358`）。作者は `final_1`、`rtt_rtt_0`。
  2. 1 つの descriptor の番号から、テクスチャを ×2、サンプラーを ×2+1 として同じ set に作る（`WebGPUDescriptorSetLayout.cpp:94-127`）。作者の「テクスチャは group 1、サンプラーは group 2」は書けない。
  3. `texture_external` が無い（`WebGPUDriver.cpp:657-678`、`WebGPUDescriptorSetLayout.cpp:160-162`）。
- なので、作者の WGSL をそのまま描けるのは Filament の描く係の外（gpu_door、ThorVG と同じ生の Dawn の道）だけ。そこで段を回す既存の実行役は無い（前の節）。
- さらに、作者の効果のいくつかは WGSL の段だけでは動かない：Surface3D は compute を使い（定義の中に compute が 30 か所）、TimeTrail は前のコマを残し、Shatter はマウスの動きを読む。書き出し（shaderport）も自作の再生役もこれを持たない。持っているのは作者のレンダラー（`packages/core/src/gpu/index.ts`）だけ。
- 切り直し：作者の効果を作者どおりに毎コマ回す既存の実装は、作者のレンダラーそのもの。前に測った通り、作者の JS が毎コマ命令を出す時間は 1 ms 未満（Shader Lab で）。残る問題は、作者のレンダラーを Motolii と同じ Dawn の装置で動かすこと（dawn.node は自分で装置を作る：まだ渡す口が無い）と、JS を動かす土台。

## 作者のレンダラーを丸ごと取り寄せた（10/26）

- 作者は、他の道具と組むための口を用意していた：`createRendererFromJSON(preset, { gpu: { device, adapter } })` → `initialize(canvas)` → `renderFrame({ deltaSeconds })`（`packages/core/src/presetRenderer.ts:103-201`）。`gpu` で持ち主の装置を使い（説明：「同じ装置を使う別の WebGPU のレンダラーと一緒に」、`packages/js/src/createSharedDevice.ts`）、`deltaSeconds` で時計を持ち主に渡して 1 コマずつ描く（作者の言葉で「コマに合わせた書き出し」、`gpu/index.ts:3276-3300`）。
- 作者の組み立て済みの配布物 `shaders@4.0.5`（npm、MIT）を丸ごと取り寄せた（`filament/tsl/package.json`）。中身の `'use gpu'` 関数は作者の組み立て（unplugin-typegpu）を通さないと空になるので、自分で組み立てない。
- 確かめ：`filament/tsl/check_node/shaders_check.mjs`（Node と Motolii の dawn.node、確かめの台本で Motolii の一部ではない）。canvas は作者が求める物だけの代役（大きさと、テクスチャがこちらの物の WebGPU の context）、`window` は作者が読む物だけ（大きさ、画素比、イベントの口、`matchMedia`）。
- 結果（640×360、120 コマ、子は Plasma）：
  - Glitch：1 コマ 1.77 ms（作者自身の GPU 待ちを含む）、色の帯の乱れまで出る。
  - 自作の再生役で壊れていた物が全部描けた：Crystal（屈折する水晶玉、前は白い円）5.92 ms、Surface3D（前は真っ黒）7.60 ms、Form3D（絵を 3D のリボンにねじる作りで、帯になるのが正しかった）4.07 ms、TimeTrail 4.62 ms。
  - 金属：LiquidMetal 3.14 ms、Chrome 2.52 ms（作者の初期の形の円に金属が出る）。
  - 絵：`reports/shaders_author_renderer.jpg`（上：Surface3D、TimeTrail、Crystal／下：Form3D、LiquidMetal、Chrome）。
- まだの事：
  - 動画を入れること（作者の VideoTexture は `<video>` と `device.importExternalTexture` を使う。dawn.node には無い）。
  - Motolii の中で動かすこと：JS の土台と、dawn.node に Filament の装置を渡す口。今は Node の確かめだけ。
