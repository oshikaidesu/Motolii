# G0-9 windowed 100k Timeline spike

製品workspaceから隔離した、direct wgpu Timelineのwindowed検証ハーネス。
`g0-9-surface-host`のlogical/physical layoutを再利用し、公開API、Document、plugin契約を変更しない。

確認範囲:

- top-level `wgpu::Surface` 1枚
- native Timeline viewport 1つ
- 左右のopaque child WebView 2枚
- 100,000 key instance（うち10,000 selected）を毎frame direct wgpu描画
- pipeline / buffer / bind group / textureは初期化時だけ生成
- frame loopのreadback、`device.poll(Wait)`なし
- 120 frame warm-up後、30秒かつ最低100 measured frame
- Fifo acquire-to-present CPU壁時計とpresent間隔のp50 / p95 / max

## 実行

```bash
cargo test --manifest-path spikes/g0-9-windowed-timeline/Cargo.toml
G0_9_TIMELINE_REPORT=/tmp/motolii-g0-9-windowed-timeline.json \
  cargo run --release --manifest-path spikes/g0-9-windowed-timeline/Cargo.toml
```

調整用環境変数:

- `G0_9_TIMELINE_WARMUP`（既定120）
- `G0_9_TIMELINE_FRAMES`（既定100）
- `G0_9_TIMELINE_SECONDS`（既定30）
- `G0_9_TIMELINE_REPORT`（既定`/tmp/motolii-g0-9-windowed-timeline.json`）

完了にはframe数と秒数の両方を要求する。`pass`はsurface/resource/readbackを含む構造合格であり、
製品fps目標の採択ではない。

## 非目標

- Vello、text、icon、theme、React visual parity
- Timeline layout/hit-test、RationalTime、snap、marquee、D2、Undo
- GPU timestamp queryとinput-to-present latency
- Windows WebView2、異DPI monitor、surface/device lost

surface textureから毎frame作る`TextureView`はpresentに必要な一時handleなので別計数する。
`Device::create_texture`によるtexture生成には数えない。
