# iced × Rerun `SpatialStage` 埋め込み probe — 測定結果

日付: 2026-08-18
probe: `spikes/iced-rerun-embed-probe`(Motolii workspace 外の隔離 spike)
機材: Apple M4 / Metal / macOS 24.5.0

**これは観測記録である。乗り換えの判断はここでは書かない。**
発端は `docs/reviews/2026-08-18-iced-reentry-survey.md`(調査時点では
branch `claude/ux-cli-gui-integration-002b03` にあり、この branch には未着)の
「壁1: wgpu 版不一致」で、iced master がその壁を越えているかだけを実測した。

## pin

| | |
|---|---|
| iced | `https://github.com/iced-rs/iced.git` rev `3de451447bd28217bb535632867550908e29d5d0`(0.15.0-dev、2026-08-18 時点の master HEAD) |
| Rerun fork | `https://github.com/oshikaidesu/rerun` rev `483b855961c0ccab7ef3f3854a8f1b040422572c`(`motolii/stage-camera-seat`) |
| wgpu | 29.0.4 — グラフ上に**1つだけ**。iced・`re_renderer`・probe 本体が同じ型を共有した |
| egui / egui-wgpu | 0.35 — 同じく1つだけ |
| winit | 0.30.8(iced の git fork `iced-rs/winit` rev `05b8ff17`) |
| 依存グラフ | Cargo.lock で 836 package |

**壁1(wgpu 版不一致)は消えている。** 実際に `iced::widget::shader` の
`Primitive::prepare` が渡してくる `&wgpu::Device` を `re_renderer::RenderContext::new` へ
そのまま渡してコンパイルが通り、動いた。

## (a) 同一 device 成立 — **成立。ただし iced の `max_bind_groups = 2` が効く**

- iced のランタイム device をそのまま `re_renderer` に渡し、**device は1つしか作っていない**。
  `RenderContext`・`egui_wgpu::Renderer`・offscreen texture・blit pipeline のすべてが
  iced の device 上にある(`gui-run.txt` の "borrowed iced's own device (no second device was created)")。
- iced が作る device の実測値: `max_bind_groups = 2`, `max_non_sampler_bindings = 2048`,
  `max_texture_dimension_2d = 16384`, features = `SHADER_F16`, surface format = `Bgra8Unorm`。
- `max_bind_groups: 2` は `iced_wgpu::window::compositor` の**べた書き**で、
  `iced_wgpu::Settings` にも `iced::application(..)` の builder にも上書きの口が無い。
  headless 側(`renderer::Headless for Renderer`)も同じ値をべた書きしている。
- `re_renderer` は3つ使う renderer を持っており、そこで wgpu が止める:

  ```
  In Device::create_pipeline_layout, label = 'LineRenderer::pipeline_layout'
    Bind group layout count 3 exceeds device bind group limit 2
  ```

- **当たるのは `LineRenderer`(3D の線分列)だけ**である。矩形(`GridMap` → `rectangle.wgsl`)も
  world grid(`world_grid.wgsl`)も group 0/1 しか使わないので通る。
- `preflight.txt` の2ケース比較がこれを分離している。iced の device 記述で作った device と、
  `re_renderer` 自身の device 記述で作った device は、**同じ絵・同じ fnv1a
  `4c4186b2e5256325`** を出した。片方だけが上の検証エラーを出す。
  - `preflight-iced-windowed.png` / `preflight-re_renderer.png`(640x480、4象限の四隅 oracle 合格)

**今日の状態**: レイヤー合成は iced の device で動く。線分を引く renderer だけが作れない。
gizmo・outline・bounding box を出す段になるとここが効いてくる。
iced 側で1行(limits の緩和)、あるいは device を差し替える seam が入れば消える。

### 付随: iced は adapter を渡してこない

`shader::Pipeline::new` も `Primitive::prepare` も device と queue しか渡さないが、
`RenderContext::new` は `&wgpu::Adapter` を要求する。probe は instance を作り直して
adapter を選び直している。`DeviceCaps::from_adapter` は adapter を読むだけなので
同じ物理デバイスなら結果は同じだが、「同じ物理デバイスである」ことを型で言えない。
device 自体は1つのままなので (a) の主張は保たれる。

## (b) 絵の到達 — **PASS**

- `iced-b-document-camera.png`(1280x960、scale factor 2)= iced の窓を
  `iced::window::screenshot` で撮ったもの。窓の中身は shader widget だけなので、
  切り出し無しで E0 の oracle をそのまま当てられる。
- E0 の期待色 oracle(四隅、INSET=6):
  - top-left `[230, 30, 30]` = red ✓
  - top-right `[30, 230, 30]` = green ✓
  - bottom-left `[30, 30, 230]` = blue ✓
  - bottom-right `[230, 230, 30]` = yellow ✓
- 4象限の画素数は各 307200(全体 1228800 のちょうど 1/4)。
- **窓の絵の fnv1a `b63a5915482f2325` は、Rerun が描いた offscreen texture の
  fnv1a と一致する**(`iced-b-offscreen-source.png`)。
  つまり iced の窓に出ているのは Rerun の frame **そのもの**で、
  途中で色空間も解像度も変わっていない。
- 画素値が E0 と同じ整数のまま出ているのは、iced の surface が `Bgra8Unorm`
  (sRGB でない)だったため。gamma の持ち上がりすら挟まっていない。

## (c) 入力 → Message — **PASS(配送の最終区間だけ人手)**

- shader widget の `Program::update` に左ボタン押下が入り、
  `Message::StageClicked { x: 160.0, y: 120.0 }` という**型付きの値**が出た。
  座標は widget 相対の論理座標で、renderer の都合は1つも混ざっていない。
- その Message を iced のランタイムが `App::update` へ配送し、
  `App::update` が `SpatialStage::set_camera` を呼んだ。
- 絵が変わった: fnv1a `b63a5915482f2325` → `e10626ece994e3a8`。
  四隅は4象限の色から Rerun の背景(dark)へ変わり、`iced-c-after-click.png` には
  引いたカメラで document の周りに Rerun の world grid が写っている。
- **測っていない区間**: OS のマウス click を winit が拾い、iced の widget tree が
  shader widget まで配る部分。自動走行では `Program::update` を直接叩いて
  Message を取り出し、それを**ランタイム経由で** `App::update` へ流している。
  この最終区間は `cargo run -- gui-interactive <出力先>` で人が押せば確かめられる
  (窓は click まで開いたまま待ち、`interactive-` 接頭辞つきで別に証拠を残す)。
  `iced_test`/Simulator による合成は今回の範囲外。

## 詰まった箇所

1. **iced master と Rerun fork は cargo のグラフ上で同居できない**。
   iced は `web-sys = "=0.3.85"` と完全一致で釘を打っており(wasm32 専用)、
   `re_renderer` の `js-sys ^0.3.94` と衝突する。cargo の解決はターゲットを問わないので
   `cargo fetch` の段階で止まる。`=0.3.85` → `0.3.85` の1文字で消える種類の話だが、
   **今日そのまま書くとまずここで止まる**。probe は `setup.sh` でその1行だけを当てている。
2. **iced の `Pipeline` は `Send + Sync` を要求する**。`SpatialStage` /
   `RenderContext` / `egui::Context` の束は跨げないので thread_local に置いた。
   実害は無いが「GPU 状態はランタイムのスレッドに固定」という設計を強いられる。
3. **`iced::time::every` は `tokio`/`smol` feature 付き**。既定の `thread-pool`
   executor には無い。probe は `iced::window::frames()` を tick に使った。
4. 0.14 → 0.15.0-dev の API 差で詰まった箇所は無い。詰まったのは **wgpu 29 の API 差**
   (`PipelineLayoutDescriptor` の `bind_group_layouts: &[Option<&_>]` / `immediate_size`、
   `RenderPipelineDescriptor::multiview_mask`、`push_error_scope` が guard を返す)。

## ビルドの重さ

- 依存グラフ: 836 package(Cargo.lock)
- `cargo clean` からの `cargo build -j 5`(dev profile): **5分34秒**(Apple M4、
  user 994s / system 128s / 335% CPU)
- target ディレクトリ: 約 6.8 GiB / 11494 files(`cargo clean` の報告)
- 増分ビルド(probe 本体だけ変更): 2〜4 秒

## 測っていないこと

`iced_test` / Simulator の統合、AccessKit、IME、タイムライン widget(発注時の範囲外)。
OS のマウス配送(上記)。長時間安定性、resize 追従、MSAA 有無の差。

## 生成物

| ファイル | 中身 |
|---|---|
| `preflight.txt` | (a) の2ケース比較(窓なし)の全文 |
| `preflight-iced-windowed.png` | iced と同じ device 記述で描いた 640x480 |
| `preflight-re_renderer.png` | `re_renderer` の device 記述で描いた 640x480(上と同一) |
| `gui-run.txt` | GUI 走行の全ログ((a)(b)(c) の判定を含む) |
| `iced-b-document-camera.png` | (b) iced の窓の screenshot(1280x960) |
| `iced-b-offscreen-source.png` | (b) Rerun が描いた offscreen texture(窓の絵と fnv1a 一致) |
| `iced-c-after-click.png` | (c) click → `set_camera` 後の窓 |
