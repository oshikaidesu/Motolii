# M4 YUV materialization lifetime gap

状態: **CODE FACT GAP / 3動画以上の製品画素を性能証拠にしない**

## 問い

同一frameで複数video sourceを先にYUV→RGBA変換し、その全textureを後段のrender graphへ渡す時、
`YuvToRgba`の2枚ping-pong出力寿命が守られているか。

## 現行code fact

`YuvToRgba::convert`は一つの`SizePool`が持つRGBA output 2枚を交互に返す。返値は
bare `wgpu::Texture` cloneであり、3回目のconvertは1回目と同じsurfaceへ書く。

`export_document_video`は一つの`YuvToRgba`を全video slotで共有し、同一frameのloopで返値を
`backgrounds`へ全件保持した後に`RenderGraphInputs.video_sources`を作る。active video slotが3以上なら、
1番目のslotは3番目の画素へ上書きされ得る。2026-07-11監査LG-2の
「デコード出力はcache側がCOPYまたは専有texture」「`YuvToRgba`はストリーム毎所有」に現行callerが
従っていない。

## 自動審判

`motolii-gpu::yuv::tests::retaining_three_converted_outputs_overwrites_the_first_two_slot_surface`
は実GPUで次を固定する。

1. 異なる3枚のYUV frameを一つのconverterで順に変換する
2. 1枚目を3回目まで保持すると、3回目後にpixelが変わる
3. 変化後の1枚目と3枚目がpixel一致し、同じsurfaceの再利用だと確認する

これは誤った製品pixelを受理するoracleではない。legacy `convert`の保持上限を固定し、
3本以上を同時保持するcallerをこのAPIへ載せないための負例である。

## 修正境界

次の製品修正は、少なくとも同一frameの全active sourceへ互いに異なるlive surfaceを与えなければならない。
候補はstreamごとのmaterializer owner、Host管理lease、またはcacheへの明示copyである。

修正は次を同時に満たす必要がある。

- pipeline/textureを毎frame生成しない
- 必要surface数をResourceLedgerへ割当前に申告する
- 3本、4本、40本のlive sourceでaliasしない
- sourceが同じassetでも異なるsource timeなら独立pixelを保持する
- frame完了または最後の参照handleまでgrantを解放しない
- YUV→RGB係数、色空間、Quality、Document、plugin契約を変えない
- export/previewで同じmaterialization所有規則を使う

## 停止線

- generic wrapperが`&wgpu::Texture`を露出するだけでclone漏れを解決したとしない
- per-convertの新規texture生成とfull-frame copyを恒久高速routeにしない
- 3動画以上の現行export結果を音MAD画素、性能、最低スペックの合格証拠にしない
- budget/alignment供給元と参照handle境界の決定前に製品hard budget完成としない

このgapはM4 cacheで誤画素を再利用する前に閉じる必要がある。cache miss/hitは同じ正しい
`f(t, input)`を返すことが前提であり、materialization aliasをcacheで隠してはならない。
