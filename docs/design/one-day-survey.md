# 外部技術調査（2026-10-05）— 繋ぐだけで一日、AE を越える絵のために

[one-day.md](one-day.md) の次に書く計画であり、外部技術の調査報告である。実装の記録ではない。今のソースの説明でもない。調査は 2026-10-05 に、コードを読まない状態で、責務ごとに一人ずつの調べ手（12 責務 + 補助 6 + 欠け 3）と、候補ごとの反証者（22 + 5）で行い、一次資料（crates.io / docs.rs / GitHub の release と source / 論文 / ベンダ文書）だけを出所にした。欠け 3 件の反証と最終の統合は session limit で落ちたので、その 3 件は未反証と明記する。

物差しは一つ。機能数ではなく、**利用者が窓から、めちゃくちゃかっこいい画を実時間で作れる**こと。この文書が答える問いは二つ。(1) 今の選択より最適な既存品はあるか。(2) 「光を書かない」— 照明・ガラス・look・動き・時間・式・保存のどれが、既存の物を繋ぐだけで済み、どれが Motolii の持ち物として残るか。

数値は、その条件の外へ広げない。

## 結論

1. **繋ぐだけで AE 級に達した製品の先例は無い。** 軽いのは部品（ThorVG 核 ~170 KB、dotlottie-web の WebGPU wasm 1.40 MB）で、出荷された編集器は全部エンジンを書いた（Olive 50.4 MB、Natron 174 MB、Shotcut 210 MB、Rerun 205 MB、Motion 3.2 GB）。「超軽量」と「拡張性」は部品と file 形式に先例がある（ThorVG、ISF、FxPlug 4 の IOSurface）。「超リッチ」はベクトル・文字・Lottie・75% の式まで ThorVG の Lottie に先例がある。カメラ・3D・モーションブラー・pixel motion は、どのリッチな製品も自分で書いた。
2. **今の選択で置き換える物は無い。** wgpu 29.0.4、ThorVG 1.1.2、VideoToolbox、wgpu-3dgs-viewer 0.7.0、salsa 0.28.5、wesl 0.4.2、h264-reader 0.9.0 は全部 KEEP。延長が 11 責務、据え置きが 3。
3. **光は書かない。** 照明・ガラス・look・AO・影・歪み・モーションブラーは、既存のシェーダ file を棚に置く形でだけ入る（Bevy の `environment_filter.wesl` と `ssao.wesl`、three.js の transmission、Bevy の `pbr_transmission.wgsl`、Khronos PBR Neutral、Tony McMapface の LUT、AMD CAS、KinoDatamosh）。どれも EXTRACT（file を写す）であって、Motolii が式を書く責務ではない。どれを使うかは利用者の美的判断。
4. **Motolii が最低限持つ物は、先例が示す通り 10 行で尽きる**（下の「Motolii が持つ物」）。文書モデルと undo と保存形式は、どの先例も既製品を採らなかった。
5. **wgpu は 29 のまま。** wgpu-native（ThorVG が乗る）に v30 は無く、wgpu 31 が 2026-10-09 予定。29.0.4 は wgpu-native 29.0.1.1・ThorVG 1.1.2・wgpu-external-frame 0.1.1・wgpu-3dgs-viewer 0.7.0 が同じ MTLDevice で会う唯一の列。
6. **ThorVG との継ぎ目は今の形では未完**。prebuilt の wgpu-native 29.0.1.1 は `wgpuQueueGetNativeMetalCommandQueue` が NULL を返す（PR #565）ので、Motolii の `wgpu::Device` を wgpu-native の MTLDevice + MTLCommandQueue の上に作る（`device_from_raw` → `queue_from_raw`）には source から 29.0.4 で再 build（3 行の un-stub）が要る。それまでは `wgpuQueueOnSubmittedWorkDone` で門をかけた 1 コマ先行の輪だけが正しい継ぎ目。（未反証）
7. **Gaussian の門は静的な点数ではない。** 公表値（M4 Air 5.8M 点、sort 11.5 ms / 63.7 ms；M4 Max 3.5M 点 ~350 FPS）に対し、1080p 10 万点/sort の規則は一桁保守的。門は `N_alive ≤ N_max（実測）∧ Σ投影面積 ≤ k·A_frame ∧ max over t` の三部にし、点数の削減は `splat-transform --decimate`（3.9.0、MIT）か Spark の Rust `build-lod` で入口に置く。
8. **式のエンジンは QuickJS-ng（rquickjs 0.14.0）一つ**、AE API は棚の `.js` prelude（data）。ThorVG 内の JerryScript は Lottie file の式にだけ使い、共有はしない（注入口が無い、float32、毎コマ再 eval）。
9. **音は `beat-this` 1.1.0（MIT、rten）を繋ぐ**。デコードは既に選んだ AVAssetReader。1 コマごとの特徴表（RMS・帯域・onset）だけが自前。
10. 文字は ThorVG の text layout を迂回し、`harfrust + skrifa`（必要なら parley 0.11.1）で字形を出し ThorVG の shape path に流す。ThorVG の text は GPOS/GSUB・可変フォント・字ごとの口が無い。

## 置き換え・延長・据え置き

| 責務 | 今の選択 | 判定 | より最適な候補（繋ぐ物） | 理由（一行） |
|---|---|---|---|---|
| Gaussian の予算 | wgpu-3dgs-viewer 0.7.0、1080p 10 万点/sort の静的門 | EXTEND | 入口に splat-transform 3.9.0 `--decimate`（ADOPT→反証で EXTRACT）、Spark 2.3.1 `build-lod`（EXTRACT）、SPZ v4(zstd) の薄い reader、timestamp で測る三部の門 | 静的門が一桁保守的、viewer は KEEP |
| スプラットの時間 | 無し | EXTEND | Spark Dyno の per-splat 式（EXTRACT、MIT）、FreeTimeGS の式（式だけ; 実装は AGPL）、Spacetime Gaussians の PLY 属性（MIT） | preprocess 前の compute 1 本で全部同じ形 |
| ガラス | 1 pass の screen-space 屈折 | EXTEND | three.js r186 `transmission_pars_fragment`（EXTRACT）、Bevy `pbr_transmission.wgsl`（EXTRACT）、HDRP の proxy 屈折（式だけ） | 無い物: 厚み（front/back depth）、KHR volume、3 tap 分散、roughness→mip、split-sum env、縁の fade |
| ベクトル・文字・Lottie | ThorVG 1.1.2 C API | KEEP（文字だけ EXTEND） | harfrust + skrifa 0.48.0（ADOPT）→ ThorVG shape path | ThorVG の text layout の穴 |
| デコードと書き出し | AVAssetReader + CVMetalTextureCache | KEEP（EXTEND） | VTDecompressionSession + AVSampleCursor（ADOPT、任意位置）、VTCompressionSession（ADOPT、書き出し）、Linux/Windows は FFmpeg 9 hwcontext（ADOPT） | wgpu 30 は macOS では要らない |
| datamosh と motion | h264-reader 0.9.0 + VideoToolbox | KEEP（EXTEND） | KinoDatamosh（EXTRACT、MIT 頭）を解析的 motion vector で | VideoToolbox は参照欠落を drop する（FrameDropped）ので DPB を自己整合に |
| post の look | 棚の glow、gamma 空間 Bgra8 | EXTEND | Tony McMapface 48³ LUT（EXTRACT）、Khronos PBR Neutral（EXTRACT）、AMD CAS/RCAS（EXTRACT）、三角 dither（KEEP） | look は 1 module、EDR は REJECT |
| 照明 | 無し | EXTEND | Bevy `environment_filter.wesl`（EXTRACT、SPD は含まない）、Bevy `ssao.wesl`＝VBAO（EXTRACT）、LTC 面光源の表（EXTRACT、BSD-3+引用条項）、PCF 影 + contact shadow（EXTRACT）、image crate `hdr`（ADOPT）、Poly Haven CC0 | cmgen は REJECT、splat の再照明は REJECT |
| 文書・増分・保存 | salsa 0.28.5 + serde | KEEP（EXTEND） | undo 0.52.0（ADOPT、slab 依存あり）、ron 0.12.2、notify 8.2.0 + debouncer-full 0.7.0（反証: macOS で Remove を飲む） | 時刻を property 粒度の memo key にしない |
| upscale と pacing | IOSurface 輪 3 枚、`device.poll(wait)` | EXTEND | `MTLCommandBuffer.addCompletedHandler` + GPUEndTime を時計に（ADOPT）、重い層だけ半解像度の板（KEEP）、FSR 1.0 WGSL（EXTRACT、`let`→`const` の手直し） | MetalFX spatial は KEEP（閉じた枠、platform で式が割れる） |
| 音→キー | 無し | EXTEND | beat-this 1.1.0（ADOPT）、AVAssetReader の PCM（ADOPT）、特徴表は自前（KEEP） | aubio 系は GPL |
| 版 | wgpu 29.0.4 | KEEP | — | wgpu-native に v30 無し、31 は 2026-10-09 |
| 棚の拡張 | WGSL + ISF 風 JSON 頭 + wesl | EXTEND | ISF 2.0 を頭の契約に（EXTRACT）、GLSL は取り込み時に naga で WGSL へ変換し本番から `glsl` を落とす、wesl 0.6.0 は wgpu と独立に更新可 | 「Rust 無しの WESL package」は反証（cargo の package は crate） |
| 式・script | 未決 | EXTEND | QuickJS-ng 0.17.0 / rquickjs 0.14.0（ADOPT）、lottie-web ExpressionManager を API 表に（EXTRACT） | エンジンは 2 つ（ThorVG 内 + 自前）、合計 ~1.3 MB 以下 |
| ホットリロード | notify + naga + 自前 swap | EXTEND | 自前のまま（KEEP）。worker thread での compile と世代 swap、wgpu の `get_compilation_info` で file:line、debouncer は rename 対応のため | 置き換える crate は無い（cuneus / wgcore / naga_oil / pilka / rend3 は REJECT） |

## Gaussian の予算

| 名前 | 版・日付 | ライセンス | 種類 | 判定 | 解く物 | 数値（条件） |
|---|---|---|---|---|---|---|
| PlayCanvas splat-transform | 3.9.0（2026-10-02） | MIT | CLI | ADOPT → 反証で EXTRACT | `--decimate` / `--decimate-adaptive`、LOD 札、SOG / SPZ v4 / PLY の入出力 | NanoGS 表 1: ρ=0.1 で LightGaussian 比 +2.59 dB（Mip-NeRF360）。「+4.62 dB」は論文に無い |
| Spark `build-lod` + `spark-lib` | 2.3.1（2026-10-01）、rust/ workspace wgpu "29" | MIT | crate | EXTRACT | LoD 木の構築と予算内の slice | 「plain library」は反証: 使えるのは builder と slice の算法だけ |
| NanoGS | arXiv 2603.16103（2026-03-17） | 論文（code の license 未記載） | 論文 | EXTRACT | 学習無しの簡約（質量保存の moment matching） | 上の dB |
| wgpu-3dgs-viewer | 0.7.0（2026-05-15、wgpu ^29）/ 0.8.0（2026-08-23、wgpu ^30） | MIT OR Apache-2.0 | crate | KEEP | GPU radix sort + raster | 0.8 は wgpu 30 が要る |
| brush（ArthurBrussee） | main 1.0.0、wgpu 30、burn git | Apache-2.0 | crate | REJECT | — | crates.io に無い |
| sort-free 系（StochasticSplats / WSR / Mobile-GS / Duplex-GS） | ICCV 2025 / ICLR 2025 / ICLR 2026 / 2025-08 | 各種 | 論文 | REJECT | sort を消す | 再学習が要る（標準 PLY に効かない） |
| 3DGUT outside CUDA（vkSplatting） | 2026.2.9（2026-09） | Apache-2.0 | 技術 | REJECT | 歪んだカメラ | Vulkan |

調べ手の判定: 10 万点/sort at 1080p の規則は、M4 Air 5.8M 点で sort 11.5 ms / frame 63.7 ms、M4 Max 3.5M 点 ~350 FPS という公表値に対して一桁保守的。raster は被覆（coverage）に、10 万点の sort は dispatch の overhead に縛られ、画素比で scale する静的な数ではない。欠け 3「whole-frame-ledger」の案: 門は `N_alive ≤ N_max（その機で timestamp で測った sort+raster）∧ Σ投影面積 ≤ k·A_frame（composition のカメラ）∧ t の最大で評価`。この三部の門は未反証。強い画: Niantic の SPZ を `--decimate` で予算に落とし、一つのカメラに立方体・動画・ガラスと同居。

## スプラットの時間

| 名前 | 版・日付 | ライセンス | 判定 | 解く物 |
|---|---|---|---|---|
| Spark Dyno の per-splat 式（reveal / dissolve / unroll / twister / rain / explosion / flow） | v0.1.9（2025-09-22）で reveal と transition、dissolve は 2025-11-23（PR #218）、最新 2.3.1 | MIT | EXTRACT（反証で保持） | 再構成データ無しの動き |
| FreeTimeGS の式 μx + v(t−μt)、σ(t)=exp(−½((t−μt)/s)²) と .ftgs.ply v1 / .tsog | CVPR 2025; FreeTimeGsVanilla（2026-09-29、AGPL-3.0）、TSOG 1.0.1（2026-09-28） | 式は論文、実装は AGPL / 非商用 | EXTRACT（式だけ） | 速度付きの点 |
| Spacetime Gaussians の PLY 属性（trbf_center / trbf_scale / motion_0..8 / omega_0..3）、splaTV | CVPR 2024、repo 2024-06-16 | MIT（rasterizer は Inria） | EXTRACT | 時間付き file |
| V³ / VideoGS | SIGGRAPH Asia 2024 | MIT（training） | EXTRACT | 属性を 2D 画像にして H.264 で運ぶ |
| Irrealix / KIRI の AE plugin の parameter 集合 | KIRI v1.2.0（2026-08-31） | — | 仕様として参照 | opacity ramp、crop、colorize、splat-scale |
| Ex4DGS、4D-Rotor / 4DGS 系 | 2024–2026 | 非商用の系譜あり | REJECT | — |

一つの compute「time kernel」を viewer の Preprocessor の前に置き、(splat, t) → pos / rot-scale / alpha / color と生存の圧縮を行う。Dyno も FreeTimeGS も Spacetime も同じ pass で棚の関数が違うだけ。sort と raster は変えない。費用は点ごとの O(N)。

## ガラス

| 名前 | 版・日付 | ライセンス | 判定 | 何を写すか |
|---|---|---|---|---|
| three.js MeshPhysicalMaterial transmission（`transmission_pars_fragment` + transmission pass）、KHR_materials_volume / _dispersion | r186 / npm 0.186.1（2026-09-24）、dispersion は r164（2024-04-26） | MIT | EXTRACT | 厚み、Beer–Lambert、3 tap 分散（halfSpread）、roughness→mip LOD。反証: 「1 bicubic tap（分散で 3）」は過小、実際はより多い |
| Wyman 2005 → Mayer & Assarsson HPG 2025 → JCGT 2026 の dual-depth 屈折 | JCGT vol.15 no.1（2026-04-19） | 論文 | EXTRACT → 反証で KEEP | code が無い |
| Bevy `pbr_transmission.wgsl` | 0.12.0（2023-11-04）以来、0.19.1 / 0.20.0-rc.2 | MIT OR Apache-2.0 | EXTRACT | WGSL そのまま |
| Unity HDRP の proxy 屈折 + Gaussian pyramid + 吸収 + 縁 fade、Filament の solid/thin | HDRP 17.2.0 | Unity Companion（式だけ読む）/ Apache-2.0 | EXTRACT | 式 |
| Liquid Glass の 2D SDF lensing（WWDC25 と再現群） | 2025-06 以降 | 各種 | 参照 | UI のガラス |
| drei MeshTransmissionMaterial | 10.7.9（~2026-09-25） | MIT | EXTRACT | backside pass、chromatic、samples |
| SDF ray-traced glass（Shadertoy） | — | CC BY-NC-SA 3.0 | KEEP | code を写さない |

どのエンジンも「仕上がった板の上の 1 pass」に収束している。今のガラスに無い物: front/back depth の厚み、KHR volume の吸収、3 tap 分散、roughness→mip、split-sum env + BRDF LUT、縁の fade。全部 three.js か Bevy の file から写せる。1080p で 1 pass ≈1 ms の静的見積りは、M4 の 120 GB/s のピーク値を achievable として使っており楽観（批評者）。

## ベクトル・文字・Lottie

| 名前 | 版・日付 | ライセンス | 判定 |
|---|---|---|---|
| ThorVG WebGPU engine（`tvg_wgcanvas_set_target` に WGPUTexture） | v1.1.2（2026-09-18、最新）、wgpu-native v29.0.1.1 が要る | MIT | KEEP |
| harfrust + skrifa（+ parley 0.11.1） | skrifa 0.48.0（2026-10-03）、parley 0.11.1（2026-08-16、HarfRust は 0.8.0 から） | MIT OR Apache-2.0（harfrust の表示は要確認） | ADOPT |
| cosmic-text | 0.19.0（2026-04-22） | MIT OR Apache-2.0 | REJECT |
| Vello GPU（sparse strips）+ glifo + velato | vello_gpu 0.3.0（2026-10-02、wgpu ^30）；wgpu 29 の最後は vello_hybrid 0.2.0（2026-08-07） | Apache-2.0 OR MIT | REJECT（wgpu 30 が要る） |
| Skia Skottie の text animator | main | BSD-3 | EXTRACT → 反証で KEEP（JSON 14 鍵だけ、「AE 完全」は偽） |
| dotlottie-rs / lottie-rs | v0.1.58（2026-06-22、ThorVG 1.0.6）/ 0.1.0（2024） | MIT | REJECT |
| Rive Renderer / rive-rs | main / 2025-07-04 | MIT | REJECT |

ThorVG の text は GPOS/GSUB・可変フォント・hinting・bidi が無く、文字列が一つの shape で、字ごとの口と字ごとの blur が無い。shaping を Rust 側（harfrust + skrifa）で行い、字形の輪郭を ThorVG の shape path として渡す。字ごとの blur は棚の WGSL。Apple Silicon での WebGPU 経路の退行（#4675）は 2560 幅で測ってから。ThorVG の WG texture の alpha が premultiplied 固定か（ABGR8888S を受けつつ）は矛盾のまま（下記）。

## デコードと書き出し

| 名前 | 版・日付 | ライセンス | 判定 |
|---|---|---|---|
| VideoToolbox 直: VTDecompressionSession + AVSampleCursor + AVSampleBufferGenerator、書き出しは VTCompressionSession の pool | macOS 10.8+/10.10+、`outputProviderWithRandomAccess` は macOS 26+ | Apple SDK | ADOPT（反証で保持） |
| wgpu 30.0.1（Metal `texture_from_raw` の DropCallback、vulkan dmabuf、dx12 plane slice） | 30.0.1（2026-08-22）、31 は無し | MIT OR Apache-2.0 | ADOPT → 反証で KEEP（major 更新そのもの、naga 30 を連れる） |
| FFmpeg hwcontext（d3d12va / vaapi / videotoolbox）を rsmpeg 0.18.0+ffmpeg.8.0 か ffmpeg-next 9.0.0 で | FFmpeg 9.0.2（2026-09-18） | LGPL-2.1+ / MIT / WTFPL | ADOPT（Linux / Windows だけ） |
| D3D12 Video Encode + MF の D3D12 対応 MFT、Flutter の DXGI shared handle | Windows 11 | Windows SDK | ADOPT |
| libva PRIME_2 + EGL_EXT_image_dma_buf_import → Flutter FlTextureGL | libva 1.1.0+、Flutter 3.47 | MIT / BSD-3 | ADOPT |
| wgpu-external-frame | 0.1.1（2026-08-29、wgpu =29.0.4） | Apache-2.0 OR MIT | EXTRACT |
| re_video、wgpu-native-texture-interop 0.2.0 | 0.38.1 / 0.2.0 | MIT OR Apache-2.0 / MPL-2.0 | REJECT |

macOS では今の経路が読み戻し 0 の最適。足すのは任意位置（スクラブ）の VTDecompressionSession と、書き出しの VTCompressionSession（IOSurface 裏の CVPixelBuffer を直接受ける）。`create_texture_from_hal(initial_state)` は Metal では no-op（wgpu-hal 29.0.4 の `transition_textures` は空）。

## datamosh と motion

| 名前 | 版・日付 | ライセンス | 判定 |
|---|---|---|---|
| KinoDatamosh の算法（motion vector + feedback、block 4/8/16） | master（2021-10-03） | file は MIT 頭（repo は Unlicense） | EXTRACT |
| h264-reader の bitstream datamosh を VideoToolbox で | 0.9.0（2026-09-14） | MIT OR Apache-2.0 | KEEP |
| VTFrameProcessor（VTOpticalFlowConfiguration / VTMotionBlurConfiguration） | macOS 15.4+ | Apple SDK | ADOPT → 反証で KEEP（60 fps の公表値が無い） |
| 解析的 velocity + McGuire 2012 / Jimenez 2014 の再構成ブラー → MetalFX temporal への道 | I3D 2012 / SIGGRAPH 2014 | 論文 | EXTRACT |
| FFmpeg `export_mvs` の並列 CPU decode | FFmpeg 2014-08 以来 | LGPL | REJECT |
| Vision の optical flow | macOS 11+/14+ | Apple SDK | REJECT |
| FFglitch | 0.10.2（2024-10-30） | GPL | REJECT（H.264 無し） |

VideoToolbox は参照を欠いた picture を隠蔽せず drop する（FrameDropped）。だから datamosh は DPB を自己整合に保つ編集（今の IDR 1 保持 + P 反復）が正しく、frame_num の gap や POC の不連続は未検証。

## post の look

| 名前 | 版・日付 | ライセンス | 判定 | 数値（条件） |
|---|---|---|---|---|
| Tony McMapface（48³ LUT + 6 行の sampler） | main（Bevy 0.16 に ktx2 同梱） | MIT OR Apache-2.0 | ADOPT → 反証で EXTRACT | 配布 .dds は 442,516 B（「884,736 B RGBA16F」は誤り） |
| 三角 PDF dither + 決定的 grain（Gjøl & Svendsen 2016、Lottes GDC 2016、Peters の blue noise CC0） | 2016 | 技法 / CC0 | KEEP | — |
| Khronos PBR Neutral | 2024-05-16、three.js の 'Neutral' | Apache-2.0 | ADOPT → 反証で EXTRACT | 0.08..0.8 で output = input − 0.04 の算術は正しいが結論が違う（反証） |
| AgX（Blender 4.0、three.js r161、Godot 4.4、Bevy 32³ LUT） | 2024–2025 | MIT 等 | EXTRACT | — |
| AMD CAS / RCAS | FidelityFX SDK 1.1.4（2025-05） | MIT | EXTRACT | — |
| lut-cube 0.2.0 | 2025-02-22 | MIT（非標準表示） | KEEP（50 行の自前） | — |
| EDR 出力（Flutter の外部テクスチャで Rgba16Float IOSurface） | engine PR #51748 は 2024-04-15 に close | — | REJECT | — |

look は 1 module（decode → fp16 の加算 + exposure → tone map → OETF → CA + vignette → grain → dither）と texture_3d の LUT 枠。gamma 空間の pipeline は変えない。全画面 1 pass 0.26 ms の静的見積りはピーク帯域から（楽観）。Flutter 3.47（2026-08-12）が macOS の surface を wide gamut にした件と、Bgra8Unorm non-sRGB の輪がどう通るかは未検証（矛盾の表）。

## 照明

| 名前 | 版・日付 | ライセンス | 判定 |
|---|---|---|---|
| Bevy `environment_filter.wesl` + `generate.rs`（実時間の環境 map 事前 filter） | Bevy 0.17（2025-09-30）、0.19.1 / 0.20.0-rc.2 | MIT OR Apache-2.0 | EXTRACT（反証: SPD の mip 鎖は別 file、「~140 行」は過小） |
| XeGTAO → Bevy `ssao.wesl`（0.15 以降は VBAO、XeGTAO v1.30 由来） | XeGTAO 2024-04 archive、Bevy PR #13454 | MIT | EXTRACT |
| LTC の面光源（Heitz 2016）の表 | SIGGRAPH 2016、three.js に FP16/FP32 | BSD-3 + 引用条項 | EXTRACT |
| key light の shadow map（PCF / PCSS）+ contact shadow（Bend、bgfx 44-sss、three.js SSSNode） | 2019–2023 | BSD-2 / MIT | EXTRACT |
| image crate `hdr`（Radiance → Rgb32F）、exr 1.74.2、Poly Haven CC0（~997 HDRI、studio 61） | image 0.25.10（2026-03-10） | MIT OR Apache-2.0 / BSD-3 / CC0 | ADOPT |
| Filament cmgen | 1.77.2 | Apache-2.0 | REJECT |
| splat の再照明（Ref-Gaussian、MetalGaussianSplatRelighting 等） | 2025–2026 | 各種 | REJECT |

Rust が持つのは HDRI の decode と一度きりの prefilter の dispatch だけ。照明の式は全部 Bevy の WGSL/WESL を棚に置く。splat は再照明しない（露出合わせ、AO、depth 経由の contact shadow だけ受ける）。「XeGTAO Medium ≈ High の 2/3」の数には出所が無い。

## 文書・増分・保存

| 名前 | 版・日付 | ライセンス | 判定 |
|---|---|---|---|
| salsa | 0.28.5（2026-09-24）、0.29 は無い | Apache-2.0 OR MIT | KEEP |
| undo | 0.52.0（2025-03-08） | MIT OR Apache-2.0 | ADOPT（反証: 「依存無し」は偽、slab ^0.4） |
| serde + RON（serde_json 代替）、cache は postcard / rkyv | ron 0.12.2（2026-06-22）、rkyv 0.8.18 | MIT OR Apache-2.0 | KEEP |
| notify + notify-debouncer-full | 8.2.0 + 0.7.0（2026-01-23）；9.0.0-rc.5 + 0.8.0-rc.2 | CC0 / MIT OR Apache-2.0 | ADOPT（別の反証: 0.7.0 は macOS で Remove を飲む `push_remove_event`） |
| comemo | 0.5.1（2026-01-29） | MIT OR Apache-2.0 | EXTRACT |
| loro / automerge | 1.16.2 / 0.12.0 | MIT | REJECT |
| incremental / adapton | 0.2.8 / 0.3.31（2019） | MIT / MPL-2.0 | REJECT |

時刻を property 粒度の memo key にしない。t に依存しない compiled な property 表を memo し、t での sample は salsa の外（CPU か GPU）。t が key に残る所は frame index に量子化し、lru と `trigger_lru_eviction()` を毎コマ。「1000 層 × 50 property」は調べ手の仮定で、Motolii の要件ではない。

## upscale と pacing

| 名前 | 版・日付 | ライセンス | 判定 |
|---|---|---|---|
| 重い層だけ半解像度の板 + nearest-depth / bilinear の合成 | GPU Gems 3 ch.23、NVIDIA 2011 | 技法 | KEEP |
| FSR 1.0 EASU + RCAS の WGSL | AMD 2021、port 2022-10-25 | MIT | EXTRACT（反証: port の `let` は wgpu 0.15 以降 `const` に直す） |
| MetalFX spatial（objc2-metal-fx 0.3.2 + `as_hal`） | macOS 13+ | Apple SDK | ADOPT → 反証で KEEP |
| `MTLCommandBuffer.addCompletedHandler` + GPUEndTime を時計に（`device.poll(Wait)` の代わり） | macOS 10.15+、wgpu-hal 29.0.4 `raw_command_buffer()` は public | Apple SDK | ADOPT |
| CADisplayLink（NSView.displayLink、macOS 14+） | — | Apple SDK | KEEP（Flutter の拍で足りる） |
| MetalFX temporal / frame interpolator | macOS 13+ / 26+ | Apple SDK | REJECT |
| checkerboard | GDC 2016/2017 | 技法 | REJECT |

全画面の upscale は既定では載せない。`device.poll(wait)` の毎コマの CPU 同期点は、完了 handler の時計に替える。

## 音→キー

| 名前 | 版・日付 | ライセンス | 判定 |
|---|---|---|---|
| beat-this（Beat This! の rten 移植） | 1.1.0（2026-10-02）、rten 0.24、rubato 3.0 | MIT | ADOPT（反証: 「~1× realtime」は誤り、13:48 の曲を 12.1 s = ~68× 速い。数は README の自己申告） |
| AVAssetReader の Linear PCM（objc2-av-foundation 0.3.2） | macOS 10.7+ | Apple SDK | ADOPT |
| 自前の特徴表（RMS、log 帯域、SuperFlux onset、beat/bar 位相）on realfft 3.5.0 / rustfft 6.4.1 | 2025 | MIT | KEEP |
| symphonia | 0.6.1（2026-08-13） | MPL-2.0 | REJECT |
| stratum-dsp | 1.0.0（2025-12-19） | MIT OR Apache-2.0 | EXTRACT |
| aubio 系、BTrack、Essentia、madmom | — | GPL / AGPL / CC BY-NC-SA | REJECT |

取り込み時に 1 コマごとの特徴表を一度作り、t のクエリは O(1)、スクラブは決定的。

## 棚の拡張（調査 2）

| 名前 | 版・日付 | ライセンス | 判定 |
|---|---|---|---|
| WESL + wesl.toml package | 0.4.2 固定（2026-08-05）、最新 0.6.0（2026-10-04）、naga / wgpu 依存無し | MIT OR Apache-2.0 | ADOPT（反証: cargo の WESL package は Rust crate、「Rust 無し」は偽。棚フォルダに置く運用が実体） |
| ISF 2.0 を頭の契約に + Vidvox ISF-Files（~200、MIT） | ISF 2.0 | MIT | ADOPT → 反証で EXTRACT（既存の ISF→WGSL importer は無い、通る割合は未測） |
| naga glsl-in を取り込み時だけ（`#version 450` への前置き書き換え → wgsl-out） | naga 30.0.1 / naga-cli 29.0.4、tweak_shader 0.6.1 と wgpu-shadertoy が先例 | MIT OR Apache-2.0 | KEEP |
| Shadertoy importer（利用者の app key、CC BY-NC-SA 3.0 を頭に残す） | API v1 | — | EXTRACT |
| dotLottie（zip）、LottieFiles 素材（Lottie Simple License） | spec v2 | — | KEEP（unzip は自前が小さい） |
| SPZ / PLY（wgpu-3dgs-viewer） | — | MIT | ADOPT |
| Rive .riv | — | MIT | REJECT（第 2 のベクトル renderer） |

Rust 無しで置ける file は 5 種: (1) `.wesl` / `.wgsl` + ISF 風 JSON 頭、(2) 取り込み時に (1) へ変換した ISF `.fs` と Shadertoy JSON、(3) Lottie JSON（+ dotLottie の unzip）、(4) 画像・動画、(5) `.ply` / `.spz`。表せない物: mesh（glTF）と Rive、Substance / Cavalry 固有、層の変形・他層の時刻差・3D のような host の意味。

## 式と script（調査 2）

| 名前 | 版・日付 | ライセンス | 判定 |
|---|---|---|---|
| QuickJS-ng via rquickjs | v0.17.0 / 0.14.0（2026-09-18） | MIT | ADOPT（反証: 「エンジン一つ」は process としては偽、ThorVG 内に JerryScript が残る） |
| ThorVG 同梱 JerryScript（`-Dextra=lottie_exp`） | 1.1.2 の tag は既定 on、main は 2026-09-01 から既定 off；float32 のみ（2026-03-31）；毎コマ `jerry_eval` | Apache-2.0 | ADOPT（Lottie file の式だけ） |
| 式を data に（compile 一度 / 毎コマ call、lottie-web の形） | — | MIT | KEEP |
| QuickJS（Bellard）/ Boa 0.22.0 / JerryScript 単体 / Luau | 2026 | MIT 等 | REJECT |

大きさ（条件付き）: qjs CLI 1.2 M（macOS ARM64、js-engine-benchmark 2026-10-04 の自前 build）、JerryScript 258 K（ARM Thumb-2、README）、Boa CLI 28.6 M（同 benchmark）、deno 83.6 M。AE の式は ES2018（AE 16.0 以来、V8 — helpx は取得不可で snippet のみ）。seedRandom は層 + property + 時刻の種（スクラブが安定する契約）。

## ホットリロード（調査 3）

| 名前 | 版・日付 | ライセンス | 判定 |
|---|---|---|---|
| wgpu 29 の error scope + `get_compilation_info` + worker thread での pipeline build と世代 swap（自前） | 29.0.4 | MIT OR Apache-2.0 | KEEP |
| notify + notify-debouncer-full | 8.2.0 + 0.7.0 | CC0 / MIT OR Apache-2.0 | ADOPT → 反証で KEEP（macOS で Remove を飲む） |
| wesl `compile_sourcemap` + Diagnostic（annotate-snippets） | 0.4.2 | MIT OR Apache-2.0 | ADOPT（反証: wesl の validate は「あまり検証しない」と docs にあり、span は名前ベースで行番号は残らない） |
| Bevy PipelineCache（Queued / Creating / Ok / Err、毎コマ check_ready） | bevy_render 0.19.1 | MIT OR Apache-2.0 | EXTRACT |
| TouchDesigner GLSL TOP の Error Behavior（Show Previous Shader）、Bonzomatic、KodeLife | — | 先例 | EXTRACT |
| cuneus / wgcore / naga_oil / pilka / rend3 | 2026-09-30 / wgpu ^23 / 0.23.0 / 2024 / archive | MIT 等 | REJECT |
| Rust 側: cdylib の世代 swap（dlclose しない、RTLD_LOCAL、IOSurfaceRef / MTLDevice を渡す） | wgpu-hal `device_from_raw` / `texture_from_raw`、libloading 0.9.0 | — | 開発者だけ（利用者の道ではない） |
| rustc_codegen_cranelift | 2026 | — | ADOPT → 反証で REJECT（arm64 macOS の ABI 非互換、作者の 2025-06-30 報告） |
| Dioxus subsecond | 0.7.10（2026-07-30） | MIT OR Apache-2.0 | EXTRACT（手順だけ） |

Metal の MSL compile と PSO 生成は `create_render_pipeline` の中で同期（wgpu 29 に async は無く、PipelineCache は Vulkan のみ）。WWDC20 の値: 1,700 PSO で cold 1 分 26 秒、binary archive で 3 秒（2020 年の Intel 6 core Mac mini、その game の shader）≈ 1 本 50 ms。M4 の値は公表無し。60 fps を保ったまま reload するには worker thread で build して拍の境で swap する。wgpu 30 から error の文に shader source と compiler message が含まれなくなる（29 では含まれる）。

## Motolii が持つ物（繋ぐ物が無い所）

| 責務 | 既存品 | Motolii が持つ物 |
|---|---|---|
| 文書モデル（層、順、親子、キー、時刻） | ベクトル部分木だけ Lottie JSON | 多種層の文書と salsa のクエリ |
| Undo | どの先例も既製品を採らず | 自前の command log（undo 0.52.0 を使うなら slab が付く） |
| 保存形式 | ベクトルは Lottie（Kdenlive は engine の XML を再利用した先例） | 自前の file（RON） |
| 合成と環 | ThorVG がベクトル内、棚の WGSL が blend と効果 | 毎コマの編成、current / other、4x4 + カメラ、matte / blend の分配 |
| ベクトル・文字・Lottie | ThorVG 1.1.2 | JSON と slot を渡すだけ（文字は shaping を渡す） |
| 動画の decode / encode | VideoToolbox、AVAssetWriter | t→frame の写像、書き出しの待ち行列 |
| splat | wgpu-3dgs-viewer 0.7.0 | カメラの共有、time kernel の dispatch |
| 式 / script | QuickJS-ng + lottie-web の API 表 | host API の束縛 |
| 音 | beat-this、AVAssetReader | RMS と onset の表 |
| cache | salsa 0.28.5 | GPU texture の寿命 |
| UI | Flutter（3.47 で Impeller が macOS 既定） | timeline、inspector |

AE 機能 → 既存品の対応（調査 2）: 層と precomp → ThorVG の precomp（ベクトル）+ 自前の多種層；track matte と blend → ThorVG の MaskMethod（10）と BlendMethod（18）、種類をまたぐ matte は棚の 1 file；3D 層とカメラ → 既存品無し（ThorVG は 2D 3x3、Lottie export も camera 未対応）；shape modifier → ThorVG（Trim / Repeater / Offset / Round / Pucker-Bloat / ZigZag）、Wiggle / Merge / Twist は未記載；text animator → ThorVG（range selector、text on path）；式 → ThorVG lottie_exp が AE API の 75%、残りは QuickJS-ng；time remap → Lottie の sr/st/tm（precomp）、動画は t→frame；frame blend → 2 枚の decode 済み frame に棚の 1 file、pixel motion は optical flow（読み戻し無しの道は未検証）；motion blur → 既存品無し（t を引数に N 副時刻を累積する棚の pass）；効果 → ISF 2.0 + wesl + ThorVG SceneEffect；Lottie export → lottie-spec 1.0.1 を書く自前（Rust の writer 無し）；audio keyframe → beat-this + 自前の表；render queue → VTCompressionSession + AVAssetWriter。

## 版の表

| 部品 | 版（日付） | ライセンス | 繋がる版 |
|---|---|---|---|
| wgpu / wgpu-hal / naga | 29.0.4（2026-07-02）；30.0.1（2026-08-22）；31 は 2026-10-09 予定 | MIT OR Apache-2.0 | 本線は 29 |
| wgpu-native | v29.0.1.1（2026-06-23；crate-type cdylib+staticlib、rlib 無し、wgpu-core 29.0.1 固定、lock は 29.0.3）；v30 無し | MIT OR Apache-2.0 | ThorVG が乗る；`wgpuQueueGetNativeMetalCommandQueue` は NULL（PR #565） |
| ThorVG | 1.1.2（2026-09-18、最新） | MIT（JerryScript Apache-2.0、rapidjson MIT） | wgpu-native 29.0.1.1 |
| wgpu-3dgs-viewer / -core | 0.7.0（2026-05-15、wgpu ^29、glam ^0.32、wesl ^0.3）；0.8.0（2026-08-23、wgpu ^30） | MIT OR Apache-2.0 | 0.7.0 |
| wgpu-external-frame | 0.1.1（2026-08-29、wgpu =29.0.4） | Apache-2.0 OR MIT | Linux だけ |
| wesl | 0.4.2 固定；0.4.3 / 0.4.4 / 0.5.0 / 0.6.0（2026-10-04）；naga / wgpu 依存無し | MIT OR Apache-2.0 | 独立に更新可（0.5/0.6 が今の棚を全部通すかは未検証） |
| salsa | 0.28.5（2026-09-24）；0.28.3 yank；0.29 無し | Apache-2.0 OR MIT | — |
| h264-reader | 0.9.0（2026-09-14） | MIT OR Apache-2.0 | — |
| notify / notify-debouncer-full | 8.2.0（2025-08-03）/ 0.7.0（2026-01-23）；9.0.0-rc.5 / 0.8.0-rc.2 | CC0 / MIT OR Apache-2.0 | 0.7.0 は macOS で Remove を飲む |
| rquickjs / quickjs-ng | 0.14.0 / v0.17.0（2026-09-18） | MIT | — |
| skrifa / parley / harfrust | 0.48.0（2026-10-03）/ 0.11.1（2026-08-16）/ 0.10 | MIT OR Apache-2.0 | — |
| beat-this | 1.1.0（2026-10-02、rten 0.24、rubato 3.0） | MIT | — |
| splat-transform / Spark | 3.9.0（2026-10-02）/ 2.3.1（2026-10-01、rust workspace wgpu "29"） | MIT | — |
| image / exr | 0.25.10（2026-03-10）/ 1.74.2（2026-07-10） | MIT OR Apache-2.0 / BSD-3 | — |
| objc2 / objc2-* frameworks / objc2-metal-fx / objc2-video-toolbox | 0.6.4（2026-02-26）/ 0.3.2（2025-10-04） | MIT / Zlib OR Apache-2.0 OR MIT | — |
| FFmpeg / rsmpeg / ffmpeg-next | 9.0.2（2026-09-18）/ 0.18.0+ffmpeg.8.0 / 9.0.0（2026-08-05） | LGPL / MIT / WTFPL | Linux / Windows |
| Flutter | 3.47.0（2026-08-12、Impeller が macOS 既定、wide gamut 既定） | BSD-3 | 外部テクスチャは 32BGRA / NV12 |
| three.js / Bevy / Vello | r186（2026-09-24）/ 0.19.1 + 0.20.0-rc.2 / 0.11.0（wgpu 30） | MIT / MIT OR Apache-2.0 / Apache-2.0 OR MIT | file を写す元 |
| undo / ron / rkyv | 0.52.0 / 0.12.2 / 0.8.18 | MIT OR Apache-2.0 / MIT | — |

共存できない組: wgpu 30 系（wgpu-3dgs-viewer 0.8.0、vello 0.11 / vello_gpu 0.3、brush）と wgpu-native 29.0.1.1（ThorVG）。wgpu 30 へ動くと naga ^30 が付き、`VertexState.buffers` の Option 化（#9351）を連れる（bind_group_layouts の Option 化は 29.0.0 #9034 で、30 の費用ではない）。

## 数値の条件

| 数値 | 条件 |
|---|---|
| 5.8M 点、sort 11.5 ms / frame 63.7 ms | M4 Air、公表値（調べ手の引用、viewer は未記載） |
| 3.5M 点 ~350 FPS | M4 Max、公表値 |
| 26,367 点 | one-day.md の静的門: 1080p 10 万点/sort × 画素比 ÷ 2 sort、2560×1536 |
| ~170 KB / 1,398,289 B | ThorVG 核の自称最小 / dotlottie-web 0.80.0 の WebGPU wasm |
| 50.4 MB / 174 MB / 210 MB / 205 MB / 3.2 GB | Olive 0.2.0-nightly dmg / Natron 2.5.0 dmg / Shotcut 26.9.27 dmg / Rerun 0.38.1 CLI / Motion 6.4 |
| 13,733,288 B | wgpu-native v29.0.1.1 の macOS aarch64 release zip（static + dynamic + headers、file 別の大きさは未公表） |
| 1.2 M / 258 K / 28.6 M / 83.6 M | qjs（macOS ARM64、2026-10-04 の benchmark build）/ JerryScript（ARM Thumb-2 README）/ Boa CLI / deno |
| 643 KiB | salsa の release 増分（one-day.md のスパイク） |
| 442,516 B | Tony McMapface の配布 .dds |
| +2.59 dB / +5.00 dB | NanoGS 対 LightGaussian、Mip-NeRF360、ρ=0.1 / 0.01（表 1） |
| 13:48 の曲を 12.1 s | beat-this README の M4 自己申告（≈68× 実時間） |
| 1 分 26 秒 / 3 秒 | 1,700 PSO、cold / binary archive、2020 年の 6 core Mac mini（WWDC20） |
| 1.0 ms = 120 MB、0.26 ms/全画面 pass | M4 base の 120 GB/s ピークからの静的見積り（楽観） |
| 450 FPS 1080p | FreeTimeGS、RTX 4090、論文 |
| 1.23×、−37% memory | WSR（sort-free）、Snapdragon、再学習込み |
| 60 fps / 2560×1536 / M4 16 GB | この文書の門（計測値ではない） |

## 利用者が決めること

1. ガラスの見た目: three.js 流（厚み + 吸収 + 3 tap 分散 + roughness→mip）か、Liquid Glass 流（2D SDF の lensing + 縁の specular）か、今の 1 pass のままか。
2. look の基準: Tony McMapface の LUT か、Khronos PBR Neutral か、AgX か。彩度の高い motion graphics 色でどれが「かっこいい」か。
3. 照明の最小集合: IBL だけ / IBL + LTC の面光源 / + VBAO / + PCF 影 + contact shadow のどこまでか（1 コマの予算で割る）。
4. splat の予算の形: 静的な点数の門を捨て、実測の三部の門にするか。
5. 文字: ThorVG の text のまま（速い、穴あり）か、harfrust + skrifa で shaping を迂回するか。
6. ThorVG の継ぎ目: wgpu-native を source から再 build して 1 queue にするか、1 コマ先行の輪で待つか。

## まだ開いているもの

未検証（批評者）:
- wgpu-3dgs-viewer 0.7.0 の depth: `Renderer::new(.., depth_stencil, ..)` と `render_with_pass` はあるが、frag_depth の書き、blend（premultiplied か straight か）、MSAA の前提、RadixSorter の算法（look-back か reduce-then-scan か）は source を読んでいない。
- 時刻のエコーの板（t−6、t−12）が履歴の輪か seek で再描画かを、どの調べ手も前提にしていない。60 fps の言明は全部「出力 1 コマに描画 1 回」のもの。門の「2 sort/frame」の意味も説明されていない。
- M4 の 120 GB/s を pass あたりの達成帯域として使った見積り（ガラス、post）は楽観。
- 「XeGTAO Medium ≈ High の 2/3」に出所無し。
- ThorVG の WG texture 出力が premultiplied 固定か（thorvg.h の ABGR8888S は「un-premultiplied」、v1.0.7 の「straight-alpha surface output」）。
- 「全部 1 device で会う」は Cargo の水準で偽: wgpu-native は別の wgpu-core と device と MTLCommandQueue。MTLDevice の水準でだけ同じ。2 queue の順序は未解決（欠け「thorvg-seam-one-queue」の反証は未実行）。
- `MTLCommandQueue.enqueue()` の順序は確認したが、`commit()` が enqueue を含むかは commit() の頁で未確認。
- `Features::EXTERNAL_TEXTURE` + `ExternalTextureFormat::Nv12`（wgpu 29.0.4、Metal）は反証されていない。`AVAssetReader.supportsRandomAccess` の macOS 27 deprecation、`outputProviderWithRandomAccess` macOS 26+ は Apple の引用無し。
- Flutter 3.47 の wide gamut 既定と、Bgra8Unorm non-sRGB premultiplied の外部テクスチャの通り方（変換か pass-through か、alpha）、2560×1536 の Texture widget の Impeller での毎コマ費用。
- IOSurface 裏の texture の storage mode（Private か否か）は両方向とも未検証。
- VideoToolbox の frame_num gap / POC 不連続での挙動（drop か drift か）。
- 欠け 3 件（mixed-kind-depth-contract、thorvg-seam-one-queue、whole-frame-ledger）の第 1 候補は反証前。

矛盾（二つの調べ手が食い違う）:
- wgpu-hal 29.0.4 Metal の `raw_command_buffer()`: public（v29.0.4 の command.rs で確認）。datamosh の反証者が private と言ったのは mod.rs だけを読んだため。
- ThorVG の alpha mode（上）。
- splat の門: 画素比の規則（upscale-pacing: 2× で ≈105,000、1.5× で ≈59,000）対「規則は測る物が違う」（gaussian-budget）対「max_t alive(t) で判定」（splat-time）。三つは両立しない。
- Flutter の最終 surface（post-look: 32BGRA/sRGB か BGRA10_XR の wide gamut；versions: 3.47 で wide gamut 既定）。両方正しければ、Bgra8Unorm の輪が 8 bit sRGB に sample されるという前提は古い。
- 29→30 の移行費用の表に bind_group_layouts の Option 化が混ざっている（それは 29.0.0 #9034）。
