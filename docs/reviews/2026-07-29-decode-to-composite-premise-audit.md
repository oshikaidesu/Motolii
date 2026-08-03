# Decode→Composite表現境界の前提監査（2026-07-29）

状態: **比較中**

## 1. 目的

40本前後の動画を含む初回Previewについて、「MP4 decodeが重い」「cacheが必要」という現象名だけで
方式を選ばず、compressed packetから最終compositionまでに置いた**表現変換、copy、materialize、
重複評価、待機の境界**を監査する。

この文書は、現行S2のffmpeg子process、K4のproxy、M4 K1／K7／K8、公開plugin契約、Document schemaを
変更する採択文書ではない。既存の[VJ多数動画同時再生の観察](2026-07-28-vj-multistream-video-prior-art-observation.md)
を、decode library比較より手前の「前提を疑う問い」へ分解する。

## 2. 現行コード事実

現行`FrameReader`はffmpeg子processへ`-f rawvideo -pix_fmt yuv420p`を指定し、stdoutから1 frameぶんを
新しい`Vec<u8>`へ読み、`CpuFrame`を返す。`YuvToRgba`はCPU上のY／U／V planeを
`Queue::write_texture`で3 textureへuploadし、別passで`Rgba8Unorm` textureへmaterializeする。
YUV変換係数、range、GPU変換、リソース再利用は既存goldenで守られている。

追加の現行事実:

- `read_frame_at`は1枚ごとに`FrameReader::open`するため、このhelperをscrub要求ごとに使えばprocess起動、
  container parse、seek／GOP pre-rollを毎回払う。製品schedulerがこの呼び方をしないことをfixtureで固定する。
- `-pix_fmt yuv420p`は10-bit、4:2:2等を8-bit 4:2:0へ変換してからpipeへ出す。したがってFFmpeg内部の
  format変換が、現時点ですでに入力側の暗黙の精度／chroma authorityになり得る。
- rotationは`-vf transpose`／flipでdecode出力へbakeされる。surface passthrough時は別処理へ移るため、
  「rotation適用済みか」がframe identityになる。
- `ColorParams`はmatrix／rangeだけを持ち、chroma siting、TRC、10-bitを表さない。現行chroma samplerは
  offsetなしのbilinear、materialize先は`Rgba8Unorm`である。
- PTSはcontainer timestampではなく`frame_index / CFR fps`から再構成される。VFR、edit list、
  negative CTSを含む素材で同一frameを指す正本にはまだなっていない。

```text
MP4
  → ffmpeg subprocess decode
  → raw YUV420p stdout
  → Vec<u8> / CpuFrame
  → Y/U/V texture upload
  → YuvToRgba pass
  → RGBA texture
  → transform / effect / composite
```

したがって、現行子processへhardware decode optionを足すだけでは、stdoutへrawvideoを出す前に
hardware surfaceをCPU-readable YUVへ移す境界が残る。hardware decoderの利用と、decoder surfaceを
GPU処理へcopy-minimizedで渡すことは別の審判である。

## 3. MP4という語を四つへ分ける

| 責任 | 主な変数 | 別々に測る理由 |
|---|---|---|
| container／demux | MP4 index、sample table、I/O、packet供給 | codec演算が速くてもseek index／disk待ちで止まり得る |
| codec decode | H.264／HEVC／AV1、profile、bit depth、参照frame | hardware対応、software fallback、同時stream天井が異なる |
| decoded surface | DPB、output pool、NV12／P010／planar YUV、同期 | CPU download、GPU copy、surface lifetimeが決まる |
| timeline demand | source time、seek順、同一frame再利用、deadline | 不要なdecode、seek storm、古いgenerationを生み得る |

「最高効率のMP4 library」を一問で決めず、この四責任のどこを委譲し、どこをHostが所有するかで比較する。

## 4. 前提監査表

| 暗黙の前提 | 比較する別解 | 正しさの停止線 |
|---|---|---|
| decoded frameはCPUを経由する | native hardware surfaceをimport、またはGPU内1 copyでwgpu入力へ渡す | CPU fallbackを残し、OS型をDocument／plugin APIへ出さない |
| 各動画は先にRGBA textureへなる | plain動画だけYUV sample＋transform＋blendを同じpassへ融合 | effect／mask／色意味が必要な地点では明示materializeし、別色authorityを作らない |
| layerごとにdecoderを持つ | asset＋source frame identity単位のdecode共有 | 異なるinterpretation、proxy、色tag、rotation、time mapを誤共有しない |
| output frameごとにdecodeする | 同じsource frameを指す複数output時刻でsurfaceを再利用 | VFR、retime、frame selection規則を近似しない |
| source解像度で常にdecodeする | Quality／投影footprintに合うproxy、対応decoderのreduced-resolution出力 | proxyへLUT／作品解釈を焼かず、Finalは原本意味へ戻る |
| request順にseekする | 最新playhead中心の要求coalesce、昇順batch、古いgeneration破棄 | audio／Transport時刻を変更せず、表示frameだけを選ぶ |
| activeなら常にdecodeする | 時間外、opacity zero、確実な不可視、同一frameを安全に除外 | Backdrop／temporal／Unknown依存を過小評価しない |
| cacheは完成RGBAだけ | packet、decoded surface、effect入力、Group、Compositionを別階層で比較 | cache warmthでpixel意味を変えず、各階層に完全keyとhard budget |
| zero-copyが常に最速 | import同期、format変換、GPU間copyを含む総frame timeで比較 | copy数という代理指標だけで採択しない |
| hardware decodeなら40本通る | codec／GPU／driver別のengine throughputとsession／surface上限を測る | 無言software fallback、OOM、editor blockを合格にしない |
| decoder surfaceを長く保持できる | decoder poolから借りるsurfaceとMotolii所有textureへのcopyを比較 | cache保持で空きsurfaceを枯らしdecodeを停止させない |
| importはhandleを渡せば終わる | device identity、queue ownership、fence／semaphore、layout、lifetimeをOS別に測る | full queue wait、暗黙race、device lost後のstale surfaceを許さない |
| decode deviceとrender deviceは同じ | iGPU decode／dGPU render、D3D11／D3D12等の不一致を検出する | cross-device CPU roundtripをzero-copyと報告しない |
| coded sizeとdisplay sizeは同じ | crop／clean aperture、stride、1088→1080等をidentityへ含める | padding画素を表示・cache共有しない |
| decoder出力はasset＋frameだけで決まる | open GOPのdecode開始点、decoder／version／option、grain／concealmentをkey候補にする | seek起点で違う画素を同一keyへ入れない |
| 外部surfaceはVRAM台帳外でよい | DPB、decoder output pool、import済み外部memoryを別entryで概算／計測 | K1aが見えないVRAMを導入しない |
| fused pathは単なる最適化 | graph shapeだけで決まる決定的path selectionと共通色authorityを試す | Quality、cache warmth、deadlineでpixel経路を切り替えない |

## 5. library／API候補の責任処分

### 5.1 FFmpeg libavcodec＋AVHWFramesContext

FFmpegのhardware contextはVideoToolbox、D3D11VA、D3D12VA、VAAPI、QSV、CUDA、Vulkan等の
device／frame poolを表現する。Motoliiの既存demux／seek／codec母数を保ちやすい本命比較候補だが、
現行子processからin-process native dependencyへ変わるため、license、配布、FFI安全性、crash隔離、
FFmpeg version pinを別審判する。

- [FFmpeg `hwcontext.h`](https://www.ffmpeg.org/doxygen/8.0/hwcontext_8h.html)
- [FFmpeg VideoToolbox hardware context](https://ffmpeg.org/doxygen/trunk/hwcontext__videotoolbox_8h.html)

`ffmpeg-next`は2026-07時点でmaintenance中心かつdocumented API範囲が狭い。crate名だけでhardware
surface所有を委譲できたとみなさず、採る場合もsys bindingを覆う狭いHost adapter候補として比較する。
[rsmpeg](https://docs.rs/crate/rsmpeg/latest) 0.18.0はFFmpeg 8 bindingsを公開し、MITで継続更新されているため、
in-process比較の第一候補を`ffmpeg-next`へ固定せず、`rsmpeg`とraw sys adapterを同じfixtureで比較する。
ただしbindingのlicenseと、linkするFFmpeg build／codecのlicense・配布条件は別審判である。

### 5.2 GStreamer

GStreamerはdecoderだけでなくmemory negotiationとbuffer poolを持ち、Windowsでは
`memory:D3D11Memory`、Linuxでは`DMABuf`／`VAMemory`、macOSではVideoToolbox decoderを提供する。
copy-minimized pipelineの完成先例として強いが、framework規模、plugin配布、version／driver matrix、
Motoliiの時刻駆動・決定的seek・wgpu所有deviceとの接続費用を測る。

- [GStreamer D3D11 H.264 decoder](https://gstreamer.freedesktop.org/documentation/d3d11/d3d11h264dec.html)
- [GStreamer VA memory](https://gstreamer.freedesktop.org/documentation/valib/index.html)
- [GStreamer hardware decode tutorial](https://gstreamer.freedesktop.org/documentation/tutorials/playback/hardware-accelerated-video-decoding.html)
- [GStreamer Apple media](https://gstreamer.freedesktop.org/documentation/applemedia/)

### 5.3 OS native decoder

VideoToolbox、Media Foundation／D3D11 Video、VAAPIは最短経路候補だが、三OSのlifecycle、同期、
format、device選択、fallbackをHostが直接所有する。最高効率の上限比較には使えるが、初期実装量が
短いことを理由に採らない。

- [Apple VideoToolbox](https://developer.apple.com/documentation/videotoolbox)
- [VideoToolbox decompression properties](https://developer.apple.com/documentation/videotoolbox/decompression-properties)
- [Microsoft Media Foundation](https://learn.microsoft.com/en-us/windows/win32/api/_mf/)

### 5.4 Vulkan Video

Vulkan VideoはH.264／H.265／VP9／AV1 decode operationを標準化するが、Metal／VideoToolboxを置換する
三OS共通解ではなく、wgpuのportable公開APIからそのまま使えることも証明しない。Windows／Linuxの
上限比較または将来backend候補に限定する。

- [Vulkan Video coding](https://docs.vulkan.org/spec/latest/chapters/videocoding.html)

#### 現存するRust／wgpu実例: `gpu-video`

[Software Mansion `gpu-video`](https://docs.rs/gpu-video/latest/gpu_video/) 0.4.0はMITで、Vulkan Videoの
H.264 decode結果をNV12の`wgpu::Texture`として返し、GPUから出さない経路とNV12→RGBA helperを公開する。
Motoliiの「decoder surfaceからwgpuへ」という仮説が実装可能である直接証拠であり、最初の隔離spike候補に
昇格する。

ただし現行公開範囲はWindows／LinuxのVulkanとH.264 decodeで、HEVC／AV1 decode、macOS／VideoToolbox、
MP4 demuxは覆わない。exampleもMP4からAnnex B H.264を事前抽出して与える。さらにrelease 0.3で
wgpu 29へ更新された一方、masterの未release変更はwgpu 30と初期化API再変更を含む。従って
`DEPEND`を先に決めず、Motoliiの既存wgpu 29 device、demux／seek、色tag、fallbackへ接続できる固定versionを
spikeで判定する。

- [`gpu-video` repository／MIT license](https://github.com/software-mansion/smelter/tree/master/gpu-video)
- [`gpu-video` changelog](https://github.com/software-mansion/smelter/blob/master/gpu-video/CHANGELOG.md)

### 5.5 pure Rust software decoder

pure Rustであることと、一般MP4の最高効率hardware decodeは別である。dav1d等は特定codecの強い
software fallbackになり得るが、H.264／HEVCを含む素材母数、hardware surface、container／seek、
multi-stream全体の答えにはならない。

### 5.6 「zero-copy」を名乗る候補の仕分け

| 候補 | 現時点の処分 | 理由 |
|---|---|---|
| `gpu-video` 0.4.0 | **SPIKE / 未採択** | permissive、H.264→wgpu textureの直接実例。Windows／Linux限定、codec／demux不足 |
| [`grafting`](https://github.com/mark-ik/wgpu-graft) | **SPIKE / 未採択** | MPL-2.0、wgpu 28／29対応。DMABUF／Vulkan、IOSurface／Metal、D3D11／DX12 shared handleをHost所有`wgpu::Texture`へ正規化する三OS実例。ただし現行のdirect DMABUFはsingle-plane中心で、decoderのNV12／P010 multi-plane接続は未証明 |
| Reco video-stitcher | **設計証拠のみ** | 公開性能文書はD3D11、IOSurface／Metal、CUDA外部memoryからwgpuへ渡す三OS経路を説明するがAGPL。sourceを読まず、独立benchmarkの代用にしない |
| `mediadecode-ffmpeg` 0.3.3 | **CPU fallback比較** | permissiveでhardware probe／software fallbackは有用だが、hardware frameを`av_hwframe_transfer_data`でCPUへdownloadするため本仮説のGPU bridgeではない |
| Geyser 0.1 | **WAIT** | MIT／Apache-2.0のVulkan／Metal texture共有候補だが、公開roadmap上wgpu external-memory importは未完成 |
| Chromium型decode service | **PATTERN / 後段比較** | 別processのcrash isolationと共有GPU surfaceを両立する先例。旧AviUtlの[InputPipePlugin観察](2026-07-29-aviutl-completed-plugin-stack-performance-observation.md#2-本体外で獲得された責任)も低スペック環境でdecode process隔離、handle reuse、共有memoryを組み合わせた直接先例だが、GPU surface共有や安全境界の完成は証明しない。IPC、同期、三OS adapterの費用が最大なので初手にはしない |

この仕分けは依存追加の許可ではない。特に「AVBufferRefをcopyせずCPU frameとして参照すること」と
「hardware surfaceをCPUへdownloadせずGPU textureとして参照すること」を同じzero-copyと呼ばない。

wgpu 30には`ExternalTexture`と`Device::create_external_texture`が入り、複数の**既存`TextureView` plane**を
shaderへ一つのexternal textureとして束ねられる。一方、IOSurface／DMABUF／shared NT handle等から
最初の`wgpu::Texture`を作るplatform importは引き続き`wgpu-hal`責任であり、
[`ExternalTexture` RFC](https://github.com/gfx-rs/wgpu/issues/3145)もnative platform handle生成を公開APIの
scope外としている。Motoliiは現行wgpu 29なので、まず`grafting`のHost adapterを固定versionで比較し、
将来wgpu 30へ上げる時は「planeの束ね」と「native handle import」を別責任として再評価する。

## 6. 最初に比較する五仮説

1. **persistent reader＋demand scheduler**
   1 frameごとのprocess起動を除き、最新playhead中心の要求coalesce、優先度、古いgeneration破棄を測る。
2. **source-frame共有**
   30fps sourceを60fps compositionで読む場合や、同一assetを複数layerが同じsource timeで読む場合のdecode回数を測る。
3. **hardware surface→GPU経路**
   現行CPU YUV pipe、hardware decode＋CPU download、hardware surface＋GPU内copy／importを分ける。
4. **YUV materialize遅延／pass fusion**
   plain動画だけYUV→linear変換、transform、opacity、normal blendを一passへ畳み、effect付きはRGBAへ実体化する。
5. **footprint-aware proxy**
   非重複40 tileと全画面40 overlapを分け、source decode pixel、disk read、upload、composite coverageを測る。

順番を変える理由は、scheduler／共有がdecode回数を乗算的に減らし、後続のどのdecoder方式にも残るからである。
zero-copyは1 frame当たりのcopy定数を減らすが、不要frameのdecode自体は消さない。

## 7. 原因分離fixture

同一内容のH.264／HEVC 10-bit、可能ならAV1を、1080p／4K、long-GOP／all-intraまたは編集proxyで準備する。
最小のfalsificationは`N=1/8/40`についてcold first frame、連続再生、scrub burst、同一asset 8複製、
30→60fps再利用を分ける。候補は安い順に、現行、現行＋persistent reader／scheduler／共有、
in-process software decode、hardware decode＋CPU download、hardware surface importとする。
各素材のcodec、profile、bit depth、GOP、fps／VFR、bitrate、duration、audio stream、file hashを
fixture manifestへ記録し、[最低スペック移行性能ゲート](2026-07-29-aviutl2-low-spec-migration-performance-gate.md)
のL0-V〜L3もこの母集団を参照する。製品既定codecや恒久asset formatを決めるmanifestではない。

記録する値:

- packet read bytes、demux／seek時間
- decoded frame数、discard frame数、software fallback理由
- CPU bytes、GPU upload bytes、GPU内copy bytes
- decoder／DPB／output surface数とresident bytes
- surface返却待ち、decoder pool starvation、import同期wait
- YUV→RGBA／fusion／effect／composite各GPU時間
- first-frame、steady p50／p95、deadline miss、cancel latency
- source frame identityごとのconsumer数と共有hit
- cacheなし初回、decoded-surface hit、K7 hit、K8 hit

採択は単一のfpsでなく、**正しさ、初回応答、steady throughput、memory、seek、fallback、配布費用**を
別々に比較する。

## 8. 非目標とSTOP

- 現行S2をこの比較だけで撤回しない。子processはcorrectness／crash isolation fallbackとして残す。
- hardware decode、FFmpeg、GStreamer、Vulkan、HAPをDocument、Final形式、plugin公開契約へ焼かない。
- wgpu-hal／native handleを使う案は、Host内部のunsafe boundary、device identity、同期、device lost、
  lifetimeをfixtureで閉じる前に製品経路へ入れない。
- importした外部memory／DPB／decoder poolをK1aのhard budgetで数えられない間は製品経路へ入れない。
- open GOP、AV1 film grain、error concealment等でdecode開始点／decoder optionが画素を変え得る時は、
  decoder identityとdecode-start policyなしにsurfaceを共有・永続cacheしない。
- YUV直接合成を、任意effect／blend／mask／色変換へ一般化しない。
- fused／materializedの選択をQuality、cache hit、負荷へ依存させない。共通の色係数、chroma座標、
  premultiply順、clamp点、goldenを持てないならfusionを止める。
- 不可視判定、source-frame共有、proxy選択でUnknown依存やVFRを近似しない。
- 「zero-copy」「hardware」の語を、実測したcopy数、surface residency、fallback理由なしで性能証拠にしない。
- 実測前にdecoder本数、surface pool、proxy閾値、先読みframe数を固定しない。

## 9. 現時点の位置づけ

M4のcache設計を置き換える計画ではない。初回のdecode／upload／materialize費用を減らすK4前段と、
二周目以降をK7／K8で一系列へ置換する後段を接続する。最も有力な見落とし候補は、library選択そのもの
ではなく、**hardware surfaceをCPUへ戻す境界、全動画を先にRGBAへする境界、layer単位で同じsource
frameを重複decodeする境界**である。

さらに、Fable 5のread-only助言を現行コードと一次資料へ再照合した結果、実装順の第一候補は
**既存subprocessのままpersistent reader／要求coalesce／source-frame共有を先に測ること**へ更新した。
これで目標を満たせばin-process化もzero-copyも採らない。満たさない場合は、まず`grafting`で三OSの
native texture→Host `wgpu::Texture`境界を再利用できるかを測り、Windows／Linuxのdecode上限は
`gpu-video`、三OSとcodec母数は`rsmpeg`＋FFmpeg hardware contextを隔離比較する。`grafting`で未成立の
NV12／P010 multi-plane、decoder pool lifetime、色、同期だけをMotolii adapterの残余責任とする。
別processとGPU共有は両立可能なので、将来crash isolationが必須ならChromium型decode serviceも比較表から
落とさない。
