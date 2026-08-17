# Rerun E0 composition probe

利用者裁定「Rerunは合成のメイン基盤」の E0 節が挙げる3点を実測するための、
Rerun だけを触る隔離 probe である。実測結果は
[E0 probe 実測](../../docs/reviews/2026-08-18-rerun-e0-composition-probe.md)にある。

```text
premultiplied RGBA wgpu::Texture
  -> SpatialStage::copy_gpu_image
  -> GridMap visualizer / RectangleRenderer
  -> SpatialStage::show(&mut egui::Ui, &mut RenderContext)
  -> egui::Context::run_ui (窓なし) -> egui_wgpu::Renderer -> offscreen wgpu::Texture
  -> copy_texture_to_buffer -> PNG + 画素 oracle
```

測るのは以下の3つ。

- **(a) offscreen** — OS 窓を開かずに `SpatialStage` のシーンを texture へ描き、
  pixel を読み出せるか。決定性は同じ binary を2回走らせて PNG の sha256 を比べる。
- **(b) camera** — 定義済みカメラをプログラムから注入でき、既知配置のレイヤー矩形が
  期待座標へ写るか。投影の一致は `SpatialStage::last_eye()` が返すカメラで
  各象限を格子状に投影し、落ちた pixel の色を照合して見る。
- **(c) occlusion** — 画素 alpha を持つ2枚を前後に置いて、前が後を遮蔽し、
  alpha=0 の部分は後ろが透けるか。

## 拘束

2026-08-11裁定(workspace `Cargo.toml:104-111`)により、Motolii は Spatial Viewer の
wrapper であって direct な `re_renderer` scene も第二 runtime も作らない。この probe も
同じ拘束下にある。`src/harness.rs` が作るのは wgpu device と egui の driver だけで、
シーン構築・カメラ・描画は `SpatialStage::show` → `SpatialView3D` → `ViewBuilder` と
いう製品と同じ経路を通る。`ViewBuilder` を自分で組むことはしない。

窓が要らないのは、Rerun が `ViewBuilder` を `egui_wgpu::Callback` に入れて
`ui.painter()` へ積むだけだからである(`re_viewer_context/src/gpu_bridge/re_renderer_callback.rs`)。
実際の `draw()`/`composite()` を走らせるのは `egui_wgpu::Renderer` で、
その相手は surface でなくてよい。

## 走らせ方

```bash
cargo run -j 5 -p rerun-e0-composition-probe -- /tmp/e0-run1
cargo run -j 5 -p rerun-e0-composition-probe -- /tmp/e0-run2
shasum -a 256 /tmp/e0-run1/*.png /tmp/e0-run2/*.png
```

3点すべてが通れば exit 0、1つでも落ちれば exit 1。落ちた項目は理由を本文へ出す。

## 読み方

- `e0-a-offscreen.png` — 4象限に塗り分けた1枚のレイヤー。4色すべてが十分な画素数で
  出ていれば「窓なしで本当に描かれた」。clear color だけの絵はここで落ちる。
- `e0-b-default-camera.png` — 既定カメラ。斜め俯瞰で、レイヤーの手前側の角が画面外へ出る。
  oracle は画面内へ落ちた点だけを数え、そのすべてが自分の象限の色であることを要求する。
- `e0-b-after-*.png` — `focus_entity` / `reset_view` を呼んだ後。**既定と1 byte も
  変わらない**ことが、カメラ注入経路が塞がっている証拠になる。
- `e0-c-disc-in-front.png` / `e0-c-disc-behind.png` — 不透明な赤い矩形と、中央だけ
  不透明で外側 alpha=0 の緑の円板。円板が手前なら緑が乗り、奥なら緑は1画素も出ない。
  円板には**わざと大きい `draw_order`** を与えてある。draw order で前後が決まって
  しまう実装なら、奥に置いた円板が漏れて見える。

これは Rerun の空間合成能力だけを確認する fixture であり、Vism runtime、Filter、
Preview/Export、`motolii-ui` の製品コードへ接続しない。
