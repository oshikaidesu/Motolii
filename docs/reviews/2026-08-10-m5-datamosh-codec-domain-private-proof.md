# M5 Datamosh codec-domain private proof

日付: 2026-08-10
状態: **決定／M5-DATAMOSH-P0 `DONE / PRIVATE PROBE`、製品runtime未接続**

## 1. Outcome

DatamoshをRGBA後段filterと誤分類せず、圧縮映像の参照関係を扱うcodec-domain候補として最小確認する。
固定2秒、10 fps、2 GOPのMPEG-4 Part 2 / MP4 fixtureから二つ目のkey packetだけを除去し、後続P-frameを残す。
出力は通常FFmpeg decodeを完走し、参照欠落前とは異なるframeを生む。

## 2. 既知実装preflight

| 項目 | 裁定 |
|---|---|
| MECHANISM CLASS | 圧縮映像packetのkey／inter-frame参照関係を変えるcodec-domain transform |
| KNOWN IMPLEMENTATION SEARCH | 現行`motolii-media` FFmpeg sidecar、FFmpeg 7.1.1 `noise` bitstream filter、repoのMP4 encode／probe／decode fixture |
| CANDIDATES | FFmpeg packet drop、独自H.264／MPEG-4 parser、任意byte noise、decode後motion推定 |
| ADOPTION ROUTE | FFmpeg sidecarを`REUSE / WRAP`し、`noise=drop='eq(n,10)*key'`で固定key packetだけをdrop |
| REJECTED CANDIDATES | 独自parserは一般codec機構の新設、byte noiseは意味を指定できず、RGBA motion推定はcodec-domain proofでない |
| THIN MOTOLII SEAM | fixed input + packet selection recipe + sidecar invocation + probe/decode oracle |
| THIN MOTOLII RESIDUAL | 将来のcodec profile、typed I/O、Host owner、Preview／Export、tool/version identity、第三者conformance |
| RETIREMENT | private shell fixtureを製品Adapter／effect APIへ昇格しない |
| BUILD JUSTIFICATION | NONE |
| BUILD | FORBIDDEN |

[`noise` bitstream filterのFFmpeg公式資料](https://ffmpeg.org/ffmpeg-bitstream-filters.html#noise)は
containerを壊さずpacket内容を損傷またはpacket dropするfilterとし、
`drop`式へpacket index `n`とkeyframe flag `key`を公開する。本proofはbyte損傷`amount`を使わず、
既知の二つ目のkey packetだけを除去する。

## 3. 自動oracle

[`probe.sh`](../../spikes/m5-known-implementation/M5-DATAMOSH-P0/probe.sh)は次を一回で確認する。

1. 入力は20 packet、key packetはindex 0と10だけである。
2. transform後も元assetはbyte不変である。
3. 同じinput／recipe／tool versionで二回作ったMP4はbyte-identicalである。
4. 出力はtarget key packetだけを欠く19 packetで、FFmpeg decodeが19 frameを完走する。
5. PTS 1.1のdecoded frame hashが元入力と異なり、参照欠落が画へ影響したことを示す。
6. FFmpeg／ffprobe不在は`DATAMOSH_TOOL_MISSING`として停止する。

## 4. 成立範囲と停止線

成立したのは、MP4 containerを壊さず既存FFmpegでkey packet除去を再現できることだけである。
motion vectorの読出し／交換／倍率変更、residual編集、H.264／HEVCのprofile差、hardware decoder、音声保持、
長尺seek、Preview／Export一致、製品codec Adapter、Vism API、Document schema、配布形式は未検証である。

特にH.264 IDR除去はdecoderが後続frameを捨てる場合があり、本fixtureのMPEG-4 Part 2成功を全codecへ外挿しない。
製品化はprofileとfailure contractを別仕様で閉じ、FFmpeg versionをrecipe identityへ含めるexact targetが現れるまで
`OBSERVATION / BUILD FORBIDDEN`を維持する。
