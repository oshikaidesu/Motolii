# G0-9 windowed 100k Timeline spike

実施日: 2026-07-21

状態: **容量・描画基盤 合格**。100,000 keyは製品の常用条件ではなく、十分な余裕を確認するstress fixtureとして採択する。

## 問い

既存のheadless `timeline-bench`とmacOS `g0-9-surface-host`を組み合わせ、次を同時に成立させられるか。

- 1 top-level wgpu Surface
- 100,000 keyを描くnative Timeline viewport
- 左右2枚のopaque child WKWebView
- frame loop内のpipeline / buffer / bind group / texture生成0
- readback 0
- 30秒かつ100反復以上のwindowed計測

製品workspace、公開API、Document、plugin契約は変更しない。

## 結果

Apple M4 / Metal、1440×900、Fifo presentで完走した。
[raw report](g0-9-windowed-timeline-evidence/report.json)を正とする。

| 指標 | 結果 |
|---|---:|
| key / selected key | 100,000 / 10,000 |
| warm-up / measured | 120 / 1,729 frame |
| measured duration | 30.017 s |
| acquire / present | 1,849 / 1,849 |
| readback | 0 |
| hot-loop pipeline / buffer / bind group / texture生成 | 0 / 0 / 0 / 0 |
| acquire-to-present CPU壁時計 median / p95 | 13.741 / 15.352 ms |
| present間隔 median / p95 | 16.664 / 17.006 ms |
| throughput | 57.60 fps |
| max外れ値 | 1,077.70 ms present interval |

100,000 keyは常用規模を大きく超えるstress条件であり、WebView同居の実windowで30秒完走、readback 0、
hot-loop resource生成0、p95が60Hz近傍だったため、**Timelineの容量・描画基盤は合格**とする。
16.667ms超のpresent間隔849件と約1.08秒の外れ値は診断記録として残すが、この過剰条件の不合格理由にはしない。
この採択はtext/icon、入力応答、D2編集契約まで自動的に合格させるものではない。

## CU-0G02 同条件 raw comparison（2026-07-24）

fixed-Mac の同一device/window/fixture/input条件で `direct_vello` と `egui_vello` を逐次実行し、
strict `RawReport` を比較した。正本は [direct raw](g0-9-windowed-timeline-evidence/direct-vello-raw.json)、
[egui raw](g0-9-windowed-timeline-evidence/egui-vello-raw.json)、および typed
[comparison](g0-9-windowed-timeline-evidence/comparison.json) である。比較は絶対閾値や勝者判定を含まない。

- MacBook Air `Mac16,12`、Apple M4 8-core GPU / Metal 3、16 GB unified memory
- macOS 15.5 build 24F74、内蔵 2560×1664 Retina (scale 2.0)、実surface 2880×1708、Fifo
- `Hiragino Sans|normal|normal|300`、font digest `833776a6fd68e2c71e…7e1b3475`、glyph digest `2a6986e5358823c…7abb44ddf`
- 共通 scenario/input/source digest: `089cbd008ee776…b8ed1618` / `56517a580ba7801…d8d42718` / `07d602a48f09e3…a96938e2b`

| mode | measured | frame p95 | input-to-present p95 | RSS |
|---|---:|---:|---:|---:|
| direct_vello | 1,802 frames / 30.049 s | 14.385 ms | 2.127 ms | 218,464,256 B |
| egui_vello | 1,801 frames / 30.041 s | 14.274 ms | 2.254 ms | 160,645,120 B |

両raw reportは acquire=present、readback=0、warmup/measured resource creation=0、input sample数=measured frame数、
RSS available、skip 3回の明示計数、complete を満たす。比較ratioはすべて有限である。これはCU-0G02の evidenceであり、CU-0G03/04/05L、
製品統合、renderer採択を進めない。

再現に用いたコマンド（同時実行しない）:

```bash
cargo run --release --manifest-path spikes/g0-9-windowed-timeline/Cargo.toml --bin g0_9_font_preflight -- 'Hiragino Sans|normal|normal|300'
G0_9_CJK_FACE='Hiragino Sans|normal|normal|300' G0_9_RENDERER_MODE=direct_vello G0_9_TIMELINE_REPORT=/tmp/cu-0g02-direct-raw.json cargo run --release --manifest-path spikes/g0-9-windowed-timeline/Cargo.toml --bin g0-9-windowed-timeline
G0_9_CJK_FACE='Hiragino Sans|normal|normal|300' G0_9_RENDERER_MODE=egui_vello G0_9_TIMELINE_REPORT=/tmp/cu-0g02-egui-raw.json cargo run --release --manifest-path spikes/g0-9-windowed-timeline/Cargo.toml --bin g0-9-windowed-timeline
cargo run --release --manifest-path spikes/g0-9-windowed-timeline/Cargo.toml --bin g0_9_compare -- /tmp/cu-0g02-direct-raw.json /tmp/cu-0g02-egui-raw.json /tmp/cu-0g02-comparison.json
```

## 実装上の確認

- 100,000 instanceは初期化時に1つのstorage bufferへuploadし、pan/zoomはuniformだけ更新する。
- render pipeline、2 buffer、1 bind groupを初期化時に生成し、frame loopでは再生成しない。
- frame loopはsurface acquire、surface texture view、uniform write、command encoder、1 render pass、presentだけを行う。
- surface texture viewはpresentに必要な一時handleとして別計数し、`Device::create_texture`と混同しない。
- source guard testがframe loop内のresource生成、copy/map、`poll(Wait)`呼出しを拒否する。
- WebViewはoffline HTMLをopaque child viewとして配置し、CDNやdev serverを使わない。

## 未証明

- Vello局所pass、text、icon、theme、React visual parity
- density / cluster / individual semantic zoom
- playhead、marquee、snap、hit-test、実pointer、10,000 key drag
- drag中semantic write 0、release時D2 commit 1、Undo 1回
- GPU timestamp query、VRAM
- resize、異DPI monitor、surface/device lost、Windows WebView2

したがって次の縦切りは、同じwindowed harnessへtoolkit非依存layout/hit-testとTransient drag projectionを接続し、
RationalTime/D2の既存境界でrelease 1 commitを証明することになる。Vello/text/parityはその後の独立枝で比較する。

## 再現

```bash
cargo test --manifest-path spikes/g0-9-windowed-timeline/Cargo.toml
G0_9_TIMELINE_REPORT=/tmp/motolii-g0-9-windowed-timeline.json \
  cargo run --release --manifest-path spikes/g0-9-windowed-timeline/Cargo.toml
```
