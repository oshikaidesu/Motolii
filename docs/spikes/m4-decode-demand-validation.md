# M4 decode demand validation

状態: **SOFTWARE + HARDWARE-DOWNLOAD HARNESS / zero-copy route未検証**

## 問い

MP4の一枚あたりdecode速度だけでなく、Motoliiの主要操作が作る要求形ごとに詰まり方を
分離できるか。

## 現在のmatrix

`motolii-media/tests/decode_demand_bench.rs`は同じH.264入力に対して次を記録する。

| route | 要求形 | 主に見える費用 |
|---|---|---|
| sequential | 一つの`FrameReader`で先頭から120 frame | process warmup後の連続software decode |
| seek | 12個の離散frameへreaderを開き直す | process生成、keyframe seek、読み捨て |
| parallel | 8個の短clip先頭を同時要求 | subprocess競合、CPU oversubscription、tail latency |
| command software | 一つのFFmpeg commandで同じ要求列をraw YUV pipeへ流す | hardware routeとの同形比較 |
| command hardware-download | hardware surfaceを明示生成し、CPU YUVへdownloadして同じpipeへ流す | hardware decode、surface download、format変換の合算 |

各要求のelapsed timeを生配列で残し、平均だけに畳み込まない。parallelは個別時間に加えて
wall timeも残す。hardware routeを指定した場合はsoftwareとhardwareのframe 0をbyte単位で比較する。
performance pass/fail閾値は持たない。

## 実行

```sh
MOTOLII_DECODE_DEMAND_OUT=/tmp/motolii-decode-demand.json \
  cargo test -p motolii-media --test decode_demand_bench \
  record_decode_demand_matrix_without_thresholds -- --ignored --nocapture
```

`MOTOLII_DECODE_FIXTURE=/path/to/input.mp4`を指定すると実素材を使う。未指定時の生成fixtureは
1280x720、30fps、10秒、H.264、GOP 60であり、スマホVFR、4K、10bit、長GOPの代替ではない。

hardware-download比較は環境変数でFFmpegのOS別surfaceを明示する。macOS VideoToolboxの例:

```sh
MOTOLII_DECODE_HWACCEL=videotoolbox \
MOTOLII_DECODE_HW_OUTPUT_FORMAT=videotoolbox_vld \
MOTOLII_DECODE_HW_SURFACE_FORMAT=nv12 \
MOTOLII_DECODE_DEMAND_OUT=/tmp/motolii-decode-hardware-comparison.json \
  cargo test -p motolii-media --test decode_demand_bench \
  record_decode_demand_matrix_without_thresholds -- --ignored --nocapture
```

`MOTOLII_DECODE_HWACCEL`を設定した時は`MOTOLII_DECODE_HW_OUTPUT_FORMAT`も必須とする。
surface formatは未指定なら`nv12`。現段階のcommand比較はrotation 0だけを受理し、回転素材を
黙って別画素として比較しない。

## 非証明

- 現行`FrameReader`はFFmpeg subprocessからraw YUV420pを読むsoftware baselineである
- `ffmpeg -hwaccels`に方式が載るだけではhardware decode経路の成立を証明しない
- hardware-download routeはhardware surfaceを明示するが、`hwdownload`後にCPU raw YUV pipeへ戻す
- decode surfaceからwgpu textureへのzero-copy、GPU color conversion、ResourceLedger計上は未接続
- synthetic fixtureの結果を最低スペック保証、AviUtl2比較、スマホ比較へ使わない

次は同じ要求列と実素材に対し、native decoder surface、GPU import、wgpu lowering、
copy/uploadを別列として測る。現行stdout pipeはnative handleとsurface lifetimeを運べないため、
codec flagの追加ではこの経路にならない。責任と合格gateは
[GPU surface import境界](m4-gpu-surface-import-boundary.md)で分離する。
画素一致または明示した色差審判を通るまで高速routeを採択しない。

## 2026-07-29 開発Mac観測

Apple M4 / 16GB、FFmpeg 7.1.1、生成720p30 fixture、debug test binaryで最初の基線を取った。
以下は製品保証でなく、route分解の観測値である。

| metric | 観測 |
|---|---:|
| sequential first frame | 42.47 ms |
| sequential frame 1〜119 median | 0.43 ms |
| sequential frame 1〜119 p95 | 0.74 ms |
| discrete seek median | 52.43 ms |
| discrete seek p95 | 59.02 ms |
| 8 parallel requests wall | 297.60 ms |
| 8 parallel request median | 215.84 ms |
| 8 parallel request max | 234.69 ms |

定常software decodeはこのfixtureでは30fps deadlineより十分短い。一方、離散要求と短clip同時要求は
一桁以上遅く、現行の「要求ごとにFFmpeg subprocessを開く」費用とseek読み捨てが支配する。
したがって次の候補は単純なcodec交換だけではなく、長寿命decoder session、要求集約、最新要求cancel、
clip間のdecoder上限、hardware routeを同じmatrixで比較することになる。

## 2026-07-29 VideoToolbox download観測

同じApple M4、同じ生成fixture、同じFFmpeg 7.1.1で、software command routeと
`videotoolbox_vld → hwdownload → nv12 → yuv420p → pipe`を比較した。

| metric | software command | VideoToolbox download |
|---|---:|---:|
| sequential 120 frame total | 91.19 ms | 271.71 ms |
| discrete seek median | 50.37 ms | 172.96 ms |
| 8 parallel requests wall | 148.87 ms | 394.24 ms |
| frame 0 byte diff | 0 / 1,382,400 | 0 / 1,382,400 |

このfixtureではhardware-download routeは全要求形でsoftwareより遅い。hardware decoderの能力不足を
意味せず、短い要求ごとのVideoToolbox session生成、hardware surface生成、CPU download、format変換、
pipe転送を合算した結果である。逆に「hardware decodeを有効にすれば最低スペック機で軽くなる」という
前提は成立しない。採択候補は長寿命sessionとGPU surface importを一体で測る必要があり、CPUへ戻す経路を
製品の高速routeと呼ばない。
