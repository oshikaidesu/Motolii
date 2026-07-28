# M4 GPU surface import boundary

状態: **縮小採用 — loweringとnative importを分離 / 製品経路未採択**

## 問い

hardware decoderが作ったframeをCPU raw YUVへ戻さず、Motoliiのwgpu renderへ渡せるか。
また、その経路を「ExternalTexture対応」の一語で完了扱いせず、どの責任と審判へ分けるか。

## 現行コード事実

- `motolii-media::FrameReader`はFFmpeg subprocessのstdoutからraw YUV420pを読む
- hardware-download benchも`hwdownload`後のCPU YUVを同じpipeへ戻す
- stdoutは`AVHWFrame`、`CVPixelBuffer`、D3D11 texture、DMA-BUF等のnative handleと
  decoder frame lifetimeを運ばない
- pinned `wgpu 29.0.4`の`Device::create_external_texture`は、既にwgpuへ存在する
  `TextureView` planeを受け取る。OS decoder surfaceをimportするAPIではない
- pinned wgpuの`EXTERNAL_TEXTURE` featureは実装途上で、対応backendも限定される。
  このfeatureを製品`GpuCtx`で先に要求しない

したがって、現在のsidecar commandへcodec flagを足すだけではGPU surface importにならない。
実験にはin-process decoderまたはOS decoder bridgeが必要であり、surfaceのdevice identity、
参照寿命、同期をHost adapterが所有する。

## 分解した責任

| 段 | owner | 合格証拠 |
|---|---|---|
| native decoder surface | OS別Host adapter | decoderがCPU raw frameでなくnative surfaceを返す |
| native import | backend別Host adapter | 同じGPU device由来のsurfaceをwgpu texture/viewへ安全に結ぶ |
| YUV lowering | wgpu render | plane viewと明示color descriptorからRGBAを生成する |
| lifetime / fence | Host adapter | GPU command完了前にdecoder poolへsurfaceを返さない |
| resource accounting | ResourceLedger | pool count、推定resident bytes、device identity、grantを追跡する |
| correctness | testkit | 同じsource、時刻、rotation、color descriptorでpixel oracleを通す |
| demand | testkit | sequential、seek、parallelの同じ要求列で各段の時間とqueue深度を記録する |

`wgpu::ExternalTexture`はYUV loweringの再利用候補であり、native importの代替ではない。
利用可能backend、format、色変換、shader bindingがfixtureへ合う場合だけ`REUSE`する。
custom WGSL loweringとの比較を残し、ExternalTextureの採択を公開plugin契約へ出さない。

## platform別の候補

### macOS

候補はVideoToolbox/CoreVideoの`CVPixelBuffer` planeを
`CVMetalTextureCacheCreateTextureFromImage`でMetal textureへ写像し、同一deviceのwgpu HALへ
結ぶ経路である。CoreVideoのtextureと元image bufferはGPU command完了まで強参照する。

Appleの公式APIは既存`CVImageBuffer` planeからMetal textureを作る境界と、利用中の
lifetime責任を示すが、wgpuへの接続、ResourceLedger会計、Motoliiの画素一致を証明しない。

### Windows

候補はD3D11VA decoder outputのTexture2D／shared handleを、互換device/backendを確認した上で
wgpu HALへ結ぶ経路である。D3D11 decoder output viewがTexture2D resourceを指すことは確認できるが、
共有可否、plane view、同期、DX12 interop、driver別成立は実機spikeで判定する。

### Linux

DMA-BUF／Vulkan等は独立の将来経路とする。wgpu ExternalTexture対応からLinux native import成立を
推論しない。

## dependency処分

| 候補 | 処分 | 理由 |
|---|---|---|
| wgpu ExternalTexture | `REUSE`候補 | 既存wgpu plane viewのloweringだけを担当させる |
| OS/backend native interop | `WRAP`候補 | handle、device、fenceをHost内へ閉じる |
| 現行FFmpeg sidecar | `KEEP` | correctness fallbackとsoftware基線。zero-copy routeには使わない |
| 汎用cross-platform importer自作 | `REJECT` | backend差とlifetimeを一つの公開抽象へ早期固定する |
| in-process libav binding | `ADOPT / WRAP`比較中 | `AVHWFramesContext`を保持できるが依存・thread・lifetime gateが必要 |

## 外部validation gate

一つだった`gpu_surface_import`を次へ分ける。

1. `native_decoder_surface_import`
   - CPU raw pipeを経由せず、native surfaceとdevice identityを取得する
2. `wgpu_external_texture_lowering`
   - import済みplane viewから明示したcolor descriptorでRGBAを得る
3. `surface_lifetime_fence`
   - GPU完了前のpool再利用、grant解放、stale generation表示を負例で拒否する
4. `gpu_surface_pixel_oracle`
   - software基線との一致規則を明記し、同じ要求列で合格する

全gateが揃うまでCPU raw YUV経路をfallbackとして残し、高速routeとは呼ばない。
macOS一台の成立をWindows最低スペック合格へ外挿しない。

## STOP条件

- native handle、OS型、HAL型をDocument、plugin API、永続形式へ出す必要がある
- decoder deviceとwgpu deviceの同一性を検証できない
- GPU完了前にsurfaceがdecoder poolへ再利用され得る
- resident bytesまたはpool上限をResourceLedgerへ計上できない
- pixel oracleを緩めないと性能routeが通らない
- wgpu feature有効化だけをimport成立として扱う

## 一次資料

- Apple:
  [CVMetalTextureCacheCreateTextureFromImage](https://developer.apple.com/documentation/corevideo/cvmetaltexturecachecreatetexturefromimage%28_%3A_%3A_%3A_%3A_%3A_%3A_%3A_%3A_%3A%29?changes=_3_2&language=objc)
- Apple:
  [CVMetalTextureCache](https://developer.apple.com/documentation/CoreVideo/cvmetaltexturecache-q3j)
- FFmpeg:
  [AVHWFramesContext](https://ffmpeg.org/doxygen/8.0/structAVHWFramesContext.html)
- FFmpeg:
  [hwcontext.h](https://www.ffmpeg.org/doxygen/8.0/hwcontext_8h.html)
- Microsoft:
  [D3D11_VIDEO_DECODER_OUTPUT_VIEW_DESC](https://learn.microsoft.com/en-us/windows/win32/api/d3d11/ns-d3d11-d3d11_video_decoder_output_view_desc)

