# 生の絵を Flutter へ (Filament WebGPU / Dawn、CPU コピーなし) — 作る前の三つの問い

## まとめ (5 行)
1. A: 問題は本物。Filament の WebGPU backend には IOSurface / CVPixelBuffer へ描く道が無い (flag は読まれない、headless は自前の RGBA8 texture、完了 callback は空)。ただし小さい問題。
2. B: 書き手 (Dawn) と読み手 (Flutter) が同じ GPU 面を分け合う。「描き終えた面だけを渡す」「読まれている面に書かない」の二つを守るだけ。
3. C: 部品は全部ディスクにある。IOSurface の取り込み (video.h)、WGPUTexture の import (patch 済み)、driver thread で順に走らせる口 (door::on_driver)、完了待ち (flushAndWait)、Flutter 側 (GlassTexture)。足りないのは「三枚を作ってつなぐ」40 行ほどの door だけ。
4. 未確認だった案は Dawn の規則に合う (IOSurface は RenderAttachment 可)。ただし macOS では毎フレームの BeginAccess / EndAccess は要らない。一度だけでよい。Filament の patch は変えなくてよい。
5. 「終わった」は flushAndWait が返った時。新しい仕組みは不要。残る不明は絵の上下の向きと、Flutter が前の面をまだ読んでいる間の扱い (順送りで避ける)。

読んだもの = file:line つき。推測は「推測」、記憶は「記憶」と書く。何も build / 実行していない。

---

## A. 本当に問題か — はい (ただし小さい)

何も作らずに済む道を探した。無い。

- `SwapChain::CONFIG_APPLE_CVPIXELBUFFER` を読むのは Metal backend だけ。
  `filament/backend/src/metal/MetalDriver.mm:823`。
  WebGPU backend が読む flag は二つだけ: `SWAP_CHAIN_HAS_STENCIL_BUFFER` (`WebGPUSwapChain.cpp:253, 283`) と `SWAP_CHAIN_CONFIG_SRGB_COLORSPACE` (`WebGPUSwapChain.cpp:271`)。
- 窓つきの swap chain は `CAMetalLayer` だけを受ける。flags は捨てている。
  `webgpu/platform/WebGPUPlatformApple.mm:75-88` (`createSurface(void* nativeWindow, uint64_t /*flags*/)`、`wgpu::SurfaceSourceMetalLayer`)。
- headless の swap chain は自分で作った三枚の `RGBA8Unorm` texture (`RenderAttachment | CopySrc`) に描く。外から取り出す口は無い。
  `WebGPUSwapChain.cpp:280-325`、枚数 `WebGPUSwapChain.h:94-97`。present は何もしない (`WebGPUSwapChain.cpp:390-397`)。
- 完了 callback は空。`WebGPUDriver.cpp:178-181` (`setFrameCompletedCallback` の中身なし)。
  Metal backend は `addCompletedHandler` で呼ぶ (`metal/MetalHandles.mm:463-485`)。これが WebGPU に無い差。
- 外部画像の口は全部 PANIC。`WebGPUDriver.cpp:658-678` ("External WebGPU Texture is not supported")。

CAMetalLayer を Flutter に渡す道はどうか。
- Filament 側は今のままで動く (上の `createSurface`)。
- しかし Flutter の `Texture` は `CVPixelBuffer` しか受けない (`skin/book/macos/Runner/MainFlutterWindow.swift:87-90` `copyPixelBuffer`)。CAMetalLayer は platform view (NSView) に置くしかない (記憶)。すると Flutter の部品を上に重ねる・切り抜く・変形するのが不自由になり、Flutter 側を書き直すことになる。採らない。

## B. 抽象的な問題 (道具の名前なしで)

一枚の絵を置く「面」が GPU の上に何枚かある。書き手は一人、読み手は一人、二人は別々の時計で動く。
書き手は「描き終えた」と GPU が言った面だけを読み手に見せ、読み手が見ている面には書かない。
時計は読み手 (画面) のもの。書き手は遅れてよいが、読み手を待たせてはいけない。半分描いた絵が見えてはいけない。

- 書く人: 中身 (inside) の描画 thread。Dawn 経由で GPU が書く。
- 読む人: Flutter の合成 (別の command queue)。
- 「終わった」と言う人: GPU。それを書き手の側で聞く。
- 時計: 画面 (Flutter)。書き手は自分の間隔で描き、できた番号を一つ置くだけ。
- 起きてはいけないこと: 半描きが見える / 破れ (読んでいる面に書く) / 読み手が止まる / CPU へ写す。

## C. 各部品に、もう有るもの

| 部品 | 有るもの | 場所 |
|---|---|---|
| IOSurface つき BGRA の面を三枚 | `CVPixelBufferCreate` + `kCVPixelBufferIOSurfacePropertiesKey` + `kCVPixelBufferMetalCompatibilityKey` | `filament/glass_door_mac.mm:29-33` |
| IOSurface を Dawn の texture にする | `wgpuDeviceImportSharedTextureMemory` → `wgpuSharedTextureMemoryCreateTexture` → `wgpuSharedTextureMemoryBeginAccess` | `filament/video.h:100-117` (読む用。書く用は usage を変えるだけ) |
| その機能を device が持つ | `SharedTextureMemoryIOSurface`, `SharedFenceMTLSharedEvent` を要求済み | `webgpu/platform/WebGPUPlatform.cpp:89-90` |
| WGPUTexture を Filament の Texture にする | patch 済みの `importTextureR`。「format と usage はそのまま」 | `WebGPUDriver.cpp:680-689`, `WebGPUTexture.cpp:285-308`, 公開口 `filament/include/filament/Texture.h:357` |
| Dawn の呼び出しを Filament の順番に入れる | `door::on_driver`, `door::device` | `filament/dawn_probe/gpu_door.h:16-20`, `gpu_door.cpp:16-24` |
| Texture を描き先にする | `RenderTarget::Builder::texture`, `View::setRenderTarget`。stage はもう使っている | `RenderTarget.h:109`, `View.h:152`, `filament/kit.h:141, 658` |
| 「終わった」を知る | `Engine::flushAndWait` = driver の `finish` = submit して GPU 完了まで待つ。stage の `draw(s, nullptr, true)` がもう呼んでいる | `filament/src/details/Engine.cpp:878-879`, `WebGPUQueueManager.cpp:84-97`, `filament/kit.h:745-752` |
| 三枚を回す・できた番号を置く | `g_live.front`, `slot[i].busy`, `glass_front()` | `filament/glass_live.h:14`, `glass_live.mm:52`, `glass_scene.mm:3039-3041` |
| Flutter が面を受け取る | `GlassTexture: FlutterTexture`, `copyPixelBuffer`, `textureFrameAvailable` | `MainFlutterWindow.swift:25-91` (そのまま使える) |

本当に無いもの: 「IOSurface の三枚を Dawn に貸し、Filament の RenderTarget 三つにする」door 一つ。`glass_door_mac.mm` (44 行) の兄弟。あとは描画 loop で `s.view->setRenderTarget(rt[i])` を一行。

### 未確認だった案は Dawn の規則に合うか — 合う

- IOSurface から作る memory が許す usage: `CopySrc | CopyDst | TextureBinding | RenderAttachment`。
  `dawn/src/dawn/native/metal/SharedTextureMemoryMTL.mm:131-135`。Metal 側も `MTLTextureUsageRenderTarget` つきで作る (同 `:95-100`, `:268-271`)。
- `RenderAttachment` が外されるのは format が描けない時だけ (`SharedTextureMemory.cpp:108-112`)。`kCVPixelFormatType_32BGRA` → `BGRA8Unorm` (`SharedTextureMemoryMTL.mm:56-57`) は描ける。
- 作る texture の決まり: 2D、mip 1、sample 1、大きさと format は memory と同じ (`SharedTextureMemory.cpp:176-200`)。
- アクセスなしで submit すると検証エラー (`dawn/src/dawn/native/Texture.cpp:1420-1441`)。だから BeginAccess は要る。
- 書ける texture の BeginAccess は排他 (`SharedResourceMemory.cpp:164-171`)。面ごとに memory が別なので三枚とも同時に持てる。
- 単一平面なら `initialized = false` でよい (多平面だけが必須: `SharedResourceMemory.cpp:142-144`)。
- EndAccess は「最後に使った submit の番号」と fence (MTLSharedEvent) を返す (`SharedTextureMemoryMTL.mm:206-225`, `QueueMTL.mm:247-248`)。待ちはしない。
- Filament 側: 描き先の format は本物の texture から取る (`WebGPUDriver.cpp:2087`)。Filament には `RGBA8` と言い、本物は `BGRA8Unorm` でよい。公開側の検査は usage の `COLOR_ATTACHMENT` だけ (`filament/src/details/RenderTarget.cpp:102`)。

一つ直す所: **macOS では毎フレームの BeginAccess / EndAccess は要らない。**
Metal 版の BeginAccess は fence の種類を確かめるだけで、鍵は掛けない (`SharedTextureMemoryMTL.mm:185-203`)。IOSurface に GPU 用の鍵は無い (記憶)。
video.h も「貸しっぱなし」(`video.h:114-117`)。同じ作法でよい: 作った時に一度 BeginAccess、片づける時に EndAccess。
毎フレーム要るのは Windows (D3D の共有 handle: `webgpu.h:2750` に型はある。中身は未読、推測)。

### 「終わった」をどう知るか — 三つ

1. **`flushAndWait` が返る** (選ぶ)。新しい code なし。描画 thread が GPU の間だけ止まる。CPU と GPU を重ねられないので 1 フレーム = CPU + GPU。今 8-9 ms なら 60 に入る。待ちは 50 µs 刻みの見回り (`WebGPUQueueManager.cpp:91-96`)。
2. `wgpuQueueOnSubmittedWorkDone` (`webgpu.h:4887`) を `on_driver` の中で呼ぶ。止まらない。ただし callback は Filament が `ProcessEvents` を回した時に来る (`WebGPUDriver.cpp:160-165`、Filament 自身の注記 `WebGPUQueueManager.cpp:87-90`)。遅れは見回りの間隔しだい。
3. EndAccess が返す fence から `MTLSharedEvent` を取り (`wgpuSharedFenceExportInfo` `webgpu.h:4989`, `WGPUSharedFenceMTLSharedEventExportInfo` `webgpu.h:2598-2601`)、OS に「この番号で知らせて」と頼む。見回りなし。Chromium がこの形 (記憶)。ObjC が 10 行ほど増える。

まず 1。CPU と GPU を重ねたくなったら 3。

### もっと単純な変種は無いか

| 変種 | 新しい code | Filament patch | 起こりうること |
|---|---|---|---|
| **甲: IOSurface を RenderTarget にする (一度だけ BeginAccess、flushAndWait)** | door 約 40 行 + loop 1 行 | なし | 上下が逆かもしれない (下)。前の面をすぐ使い回すと破れ |
| 乙: headless に描き、Dawn で texture→texture に写す | 写す command を自分で組む | 要る (headless の texture は private: `WebGPUSwapChain.h:96`) | 1 枚ぶんの GPU コピー、patch が増える |
| 丙: CAMetalLayer の swap chain | Filament 側 0 | なし | Flutter 側を platform view に書き直し。`GetCurrentTexture` が画面待ちで止まりうる (記憶) |
| 丁: Metal backend に戻す | 0 | — | ThorVG と動画が Dawn の device を要るので不可 (問いの前提) |

甲が code も patch も一番少ない。

甲で気をつけること:
- **上下**: Filament は texture へ描く時と窓へ描く時で上下が違うことがある (推測)。最初の一枚で分かる。逆なら Flutter 側で裏返す (費用なし)。
- **色の並び**: 本物が BGRA、Filament は RGBA8 のつもり。pipeline は本物の format で作るので合うはず (`WebGPUDriver.cpp:2087`)。最初の一枚で赤と青を見る。
- **破れ**: 今の選び方は「busy でなく front でもない最初の面」(`glass_scene.mm:3040-3041`)。front が替わった直後、Flutter がまだ前の面を合成中でもそこへ書きうる。順送り (0→1→2→0) にすれば 1 フレームの間が空く。
- **半描き**: front を替えるのは flushAndWait の後だけ。Flutter は front しか取らない (`glass_live.mm:52`)。起きない。
- **止まり**: Flutter を待つ所が無い。GPU が混むと flushAndWait が延び、こちらが描く回数が減るだけ。Flutter は前の絵を出し続ける。
- **thread**: Dawn の呼び出しは全部 `on_driver` の中 (video.h, drawn.h と同じ)。Filament の Texture を作るのはその後 (`flushAndWait` を一度: `drawn.h:80` の作法)。
- **最後の view**: `s.view` だけ描き先を替える。headless の swap chain は `beginFrame` 用にそのまま残す (`kit.h:119, 728`)。
- **大きさを変える時**: 三枚とも作り直す。

よそのやり方 (記憶): thermion は Filament の Metal/GL で CVPixelBuffer を Flutter Texture に渡す (WebGPU ではない)。Lumit と wgpu 系は IOSurface を Metal texture にして wgpu に包み、Flutter に CVPixelBuffer で渡す。Chromium は Dawn の SharedTextureMemory + IOSurface に描き、fence を合成側へ渡す。どれも甲と同じ形。

## D. 選んだ変種 (甲) の呼び出し、順に

始める時 (一度):
1. `CVPixelBufferCreate(..., kCVPixelFormatType_32BGRA, {IOSurfaceProperties, MetalCompatibility})` を三枚 — `filament/glass_door_mac.mm:29-31`
2. `CVPixelBufferGetIOSurface(buffer)` — `filament/video.h:84`
3. `door::on_driver(engine, [...]{ ... })` の中で 4-6 — `dawn_probe/gpu_door.h:20`
4. `wgpuDeviceImportSharedTextureMemory(door::device(engine), &desc)`、`WGPUSharedTextureMemoryIOSurfaceDescriptor` を鎖に — `webgpu.h:4842`, `webgpu.h:2780`, 例 `video.h:100-105`
5. `wgpuSharedTextureMemoryCreateTexture(memory, &like)`、`like.format = WGPUTextureFormat_BGRA8Unorm`, `like.usage = WGPUTextureUsage_RenderAttachment` — `webgpu.h:4996`, 許される usage `SharedTextureMemoryMTL.mm:131-135`
6. `wgpuSharedTextureMemoryBeginAccess(memory, texture, &begin)` (`initialized = false`、一度だけ) — `webgpu.h:4995`, 規則 `SharedResourceMemory.cpp:164-171`
7. `engine->flushAndWait()` — `Engine.h:1199`, 作法 `drawn.h:80`
8. `Texture::Builder().width(w).height(h).levels(1).format(RGBA8).usage(COLOR_ATTACHMENT).import((intptr_t)texture).build(*engine)` — `Texture.h:213, 357`, 受け側 `WebGPUDriver.cpp:680-689`
9. `RenderTarget::Builder().texture(AttachmentPoint::COLOR, tex).build(*engine)` を三つ — `RenderTarget.h:109`, 例 `kit.h:658`

毎フレーム:
10. 面 i を選ぶ (順送り、front でないもの) — 今の形 `glass_scene.mm:3039-3041`
11. `s.view->setRenderTarget(target[i])` — `View.h:152`
12. `draw(s, nullptr, true)` = `beginFrame(s.swap)` → `pictures_of` → `endFrame` → `flushAndWait` — `kit.h:745-752`
13. `g_live.front = i; ++g_live.frames` — `glass_live.h:14-15`
14. Flutter: `textureFrameAvailable` → `copyPixelBuffer` → `glass_front()` が面 i を retain して渡す — `MainFlutterWindow.swift:73-90`, `glass_live.mm:52`

片づける時:
15. Filament の RenderTarget と Texture を destroy、`on_driver` の中で `wgpuSharedTextureMemoryEndAccess` → `wgpuSharedTextureMemoryEndAccessStateFreeMembers` → `wgpuTextureRelease` → `wgpuSharedTextureMemoryRelease` — `video.h:64-75`, `webgpu.h:4997, 5005`

最初の一枚で確かめること三つ: 上下、赤と青、順送りで破れが無いこと。
