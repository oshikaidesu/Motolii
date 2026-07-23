# 色変換／GPU export lineage価値回収（Unit 5E、2026-07-23）

状態: **採択維持／現行gap訂正**

対象: [色変換先例調査](2026-07-14-color-conversion-prior-art.md) cutoff全1版（13,851 bytes）

関連: [performance model](../performance-model.md)、[M1仕様](../specs/M1-vertical-slice.md)、[backlog](../backlog.md)、[Unit 5C readback回収](2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

歴史版の価値は「GPUなら速い」という一般論ではなく、decodeのYUV→RGBAとexportのRGBA→YUVを同じ色意味の管理下へ置き、encoder内部の暗黙変換を避ける方向を採択した点にある。この方向は維持する。

一方、2026-07-23の現行コードでも逆変換は未実装である。`motolii-gpu`はYUV420p→RGBAだけを持ち、exportはRGBAを同期readbackして`motolii-media::Encoder`へ渡し、ffmpeg `scale`にBT.709 limited変換を任せる。歴史版の「GAP-14」は後にShared Effect lifecycleへ再利用され、backlogに同じIDが二つ存在したため、本件を**GAP-31**へ正規化する。

また、RGB→YUVの色意味とGPU readbackの重畳は別境界である。GAP-31は係数、range、chroma siting、plane layout、encoder入力を閉じる。GAP-29は測定後のstaging数、backpressure、overlapを閉じる。OBSの先例にdouble bufferingがあっても、GAP-31で固定本数や非同期方式まで採択しない。

## 2. 歴史版の主張別処分

| 主張 | 処分 |
|---|---|
| 色不一致の原因をmatrix／tag／range／TRC／丸め／chroma sitingへ分解する | **維持**。GAP-5／31の原因分離表として使う |
| TRCをBT.709かsRGBへ決める | **延期維持**。プレイヤー実測なしに一方へ変えない |
| RGB→YUVをGPU資産へ移す | **採択維持**。IDだけGAP-14→GAP-31へ訂正 |
| ffmpeg swscale精度flagだけを暫定追加する | **棄却維持**。golden変更を伴う小手先の既定化をしない |
| OBS型GPU変換＋staging double bufferを一体で移植する | **縮小**。色変換の先例に限り、転送方式はGAP-29の測定へ戻す |
| HW encoderへ常にYUVを渡せば将来問題が消える | **条件付き候補**。OS／encoder interop、format、zero-copyは未決で、v1 software encodeの完成条件にしない |

## 3. 現行コード事実

- `ColorParams::for_color_space`はBT.709／601とlimited／fullのYUV→RGB係数を一か所に持ち、WGSLとCPU oracleが共有する。
- `YuvToRgba`、`yuv_golden.rs`、`swscale_reference.rs`はdecode方向の意味と外部比較を成立させている。
- 逆方向の`RgbaToYuv`、YUV plane pack、GPU→YUV readback API、CPU inverse oracleは存在しない。
- `Encoder::open`は入力をRGBAに限定し、ffmpeg `scale=out_color_matrix=bt709:out_range=tv`でYUV化する。タグ4種は明示している。
- exportの二経路は`RgbaDownloader`でRGBA `Vec<u8>`を得て`Encoder::write_frame`へ渡す。GAP-31だけを実装しても、readback待ち／encode直列化が自動的に解消するわけではない。
- 現行testは出力tag、black level、decode変換、preview/exportの同一render入口を守るが、GPU inverse変換の係数・siting・plane bytesを審判していない。

よって、decode側GPU変換が完成していること、出力tagが正しいこと、preview/exportが同じRGBAを作ることを、export逆変換完成の証拠にしない。

## 4. GAP-31の再入場順

### 4.1 色意味の閉包

1. v1の入力RGBA意味、出力matrix／range、4:2:0のchroma siting、偶数寸法、plane stride／layoutを既存`FrameDesc`と整合させる。
2. inverse係数をdecode係数と同じauthorityから導出し、独立CPU oracleとswscale外部比較を用意する。
3. 非一様pattern、range端、chroma edge、odd dimension拒否、alpha処分を負例化する。既存golden thresholdは実装へ合わせて変更しない。

### 4.2 export接続

1. `Encoder`へ生YUV入力を追加する時は、RGBA入力を黙って別意味へ変えず、format／frame sizeを型付きにする。
2. ffmpeg `scale`の色変換を削除してもmatrix／primaries／TRC／range tagを保持し、ffprobeとdecode roundtripで確認する。
3. qp0検証の4:4:4と配布用4:2:0を同じbuffer shapeと誤認しない。必要な別formatは実装前に意味を決める。
4. Preview／Finalのrender関数とColorSpace意味を分岐させず、export専用の見た目補正を入れない。

### 4.3 転送はGAP-29へ分離

GAP-31のYUV化後に転送量が減る可能性は測定入力であり、固定ring数の根拠ではない。同期1-frameをまず正しいpixelの基準経路として成立させ、overlap、staging本数、encoder backpressure、cancel／failure cleanupはGAP-29の採択後に扱う。GAP-31とGAP-29を一つの便利な共通化へ束ねない。

## 5. GAP-5との境界

現行出力のTRC tagはBT.709だが、内部8-bit RGBAはsRGB近似として扱う。これは現行既定であり、全プレイヤーで許容済みという証明ではない。GAP-5はBT.709／sRGB tagの出力とQuickTime／Safari／Chrome／VLC／mpv等を、プレビュー一致と入力素材一致の二軸で実測する。

GAP-31は測定前にTRCをsRGBへ変更せず、現行tagを保持してRGB→YUVの係数／range／sitingを閉じる。GAP-5の結果が将来tag policyを変える場合も、色変換shaderとDocument／plugin契約を同時変更しない。

## 6. 復活させないもの

- Shared Effect lifecycleとGPU色変換を同じ`GAP-14`で追跡すること。
- decode GPU化、BT.709 tag、RGBA render一致をexport inverse変換実装済み証拠にすること。
- OBSのdouble buffer本数をMotoliiの測定なしに固定すること。
- `ColorParams`のdecode係数をそのままinverse APIと呼び、独立oracleなしで行列を反転すること。
- swscaleとの一致だけで内部CPU oracle、range端、siting、plane layout試験を省くこと。
- GPU YUV化を理由にHW encoder／native API／zero-copyを同じtaskへ入れること。
- exportだけ見た目を補正し、Preview／Finalの同一render意味を分岐させること。
- GAP-5実測前にTRC tagをBT.709またはsRGBへ「唯一の正解」として再決定すること。
- 既存golden toleranceを実装差に合わせて緩めること。

## 7. 固定証跡とcoverage

処分対象は`3b1cd5ddbbdfa64568ddbce095e682fa0ea184bc`の1 blob、13,851 bytes。receiptは`05e-color-export.tsv`に固定する。

本Unit後のstrict progressは357 / 1,797（19.9%）、未処分1,440である。
