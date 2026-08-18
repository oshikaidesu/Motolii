# iced × Rerun `SpatialStage` 埋め込み probe(隔離)

2026-08-18 の iced 再評価調査(`docs/reviews/2026-08-18-iced-reentry-survey.md`。
調査時点では branch `claude/ux-cli-gui-integration-002b03` にあり、この branch には未着)が挙げた
「壁1: wgpu 版不一致」— iced 0.14 は wgpu 27、Motolii と Rerun fork は wgpu 29 — が
**iced master では消えている**(master の `workspace.dependencies` は `wgpu = "29"`)。
それを受けて、壁が本当に無くなったのかを1本のアプリで実測する。

判断はしない。測ったことだけを置く。

## 何を測ったか

| | 主張 | 結果 |
|---|---|---|
| (a) | iced のランタイム device 1つで `re_renderer` も動く(device を2つ作らない) | **成立。ただし iced の `max_bind_groups = 2` に `LineRenderer` が当たる** |
| (b) | E0 と同じ4象限シーンが iced の窓に出て、E0 の期待色 oracle を通る | **PASS** |
| (c) | shader widget の click が型付き Message になり、`set_camera` で絵が変わる | **PASS(配送の最終区間だけ人手)** |

証拠(PNG と実行ログ)は `docs/reviews/evidence/iced-rerun-embed-probe/`。

## pin

| | |
|---|---|
| iced | `https://github.com/iced-rs/iced.git` rev `3de451447bd28217bb535632867550908e29d5d0`(0.15.0-dev / 2026-08-18 の master HEAD) |
| Rerun fork | `https://github.com/oshikaidesu/rerun` rev `483b855961c0ccab7ef3f3854a8f1b040422572c`(`motolii/stage-camera-seat`) |
| wgpu | 29.0.4(グラフ上に1つだけ。iced・Rerun・probe 本体が同じ物を共有している) |
| egui / egui-wgpu | 0.35(同上、1つだけ) |
| winit | 0.30.8(**iced の git fork**。Motolii 本体 workspace の winit と揃わないので、この spike は workspace 外にある) |

## 走らせ方

```sh
cd spikes/iced-rerun-embed-probe
./setup.sh                       # iced を vendor/ に取ってくる(後述の1行だけ直す)
cargo build -j 5

# (a) の切り分け。窓を開かない。
cargo run -j 5 -- preflight <出力ディレクトリ>

# (b)(c)。窓を開き、自分で撮って自分で終了する。
cargo run -j 5 -- gui <出力ディレクトリ>

# (c) を人の click で確かめたいとき。窓は click まで開いたまま待つ。
cargo run -j 5 -- gui-interactive <出力ディレクトリ>
```

`cargo test` は oracle と素材の単体テストだけを回す(GPU 不要)。

## 詰まった箇所(そのまま壁の一覧でもある)

### 1. iced と Rerun fork は cargo のグラフ上で同居できない(要 1行修正)

iced master の workspace は `web-sys = "=0.3.85"` と**完全一致**で釘を打っている。
web-sys 0.3.85 は `js-sys = "=0.3.85"` を要求し、Rerun fork の `re_renderer` は
`js-sys ^0.3.94` を要求する。どちらも wasm32 でしか使わない依存だが、cargo の解決は
ターゲットを問わないので衝突する:

```
error: failed to select a version for `js-sys`.
    ... required by package `re_renderer`
  previously selected package `js-sys v0.3.85`
    ... which satisfies dependency `js-sys = "=0.3.85"` of package `web-sys v0.3.85`
    ... which satisfies dependency `web-sys = "=0.3.85"` of package `iced_winit`
```

`=0.3.85` → `0.3.85` の1文字で消える。上流に投げれば済む種類の話で、設計上の壁ではない。
だが**今日 iced と Rerun を同じ Cargo.toml に書くと、まずここで止まる**。
`setup.sh` はこの1行だけを当てる(`git diff --stat` が 1 file / 1 insertion であることが証拠)。

### 2. iced は device の limits に手を入れる seam を持たない

`iced_wgpu::window::compositor` は device をこう作る(`wgpu/src/window/compositor.rs`)。

```rust
let limits = limits.into_iter().map(|limits| wgpu::Limits {
    max_bind_groups: 2,
    max_non_sampler_bindings: 2048,
    ..limits
});
```

`max_bind_groups: 2` は**べた書き**で、`iced_wgpu::Settings` にも
`iced::application(..)` の builder にも上書きの口が無い(`Compositor` の
`adapter` / `engine` フィールドも private)。headless 側(`renderer::Headless for Renderer`)も
同じ値をべた書きしている。

`re_renderer` は自分の device を `downlevel_webgl2_defaults` 由来(`max_bind_groups = 4`)で
要求する。実際に3つ使う renderer が居るので、iced の device では落ちる:

```
In Device::create_pipeline_layout, label = 'LineRenderer::pipeline_layout'
  Bind group layout count 3 exceeds device bind group limit 2
```

当たるのは `LineRenderer`(3D の線分列)だけである。矩形(`GridMap` → `rectangle.wgsl`)も
world grid(`world_grid.wgsl`)も group 0/1 しか使わないので通る — カメラを引いた
`iced-c-after-click.png` には Rerun の world grid が実際に写っている。
preflight の2ケース(iced の device 記述 / `re_renderer` の device 記述)は
**同じ絵・同じ fnv1a `4c4186b2e5256325`** を出した。
つまり今日の状態は「レイヤー合成は動く。線分を引く renderer だけが作れない」。
gizmo・outline・bounding box を出す段になると、ここが黙って効いてくる。

### 3. iced は adapter を渡してこない

`shader::Pipeline::new` も `Primitive::prepare` も `&wgpu::Device` と `&wgpu::Queue` しか
渡さない。`re_renderer::RenderContext::new` は `&wgpu::Adapter` を要求する。
probe は instance を作り直して adapter を1つ選び直している(`Gpu::borrow_iced_device`)。
`DeviceCaps::from_adapter` は adapter を読むだけなので同じ物理デバイスなら結果は同じだが、
**「同じ物理デバイスである」ことを型で言えない**。device は1つのままなので (a) の主張は保てる。

### 4. iced の `Pipeline` は `Send + Sync` を要求する

`SpatialStage` / `RenderContext` / `egui::Context` の束はその境界を跨げない。
`unsafe impl Send` を書く代わりに thread_local に置いた(`embed.rs`)。
iced の native runtime は `prepare`/`draw` を同じスレッドから呼ぶので実害は無いが、
「GPU 状態はランタイムのスレッドに固定される」という設計を強いられる。

### 5. `iced::time::every` は executor feature 付き

既定の `thread-pool` executor には `every` が無く、`tokio` か `smol` の feature が要る。
probe は依存を増やさずに済ませたいので `iced::window::frames()` を tick にした。

### 6. 0.14 → 0.15.0-dev の API 差(この probe が触った範囲)

- `Program::update` が返すのは `iced::widget::Action<Message>`(`publish` / `and_capture` /
  `request_redraw_at`)。`shader::Event` は `iced::Event` そのもの。
- `Primitive` の関連型は `type Pipeline`(かつては `Renderer` 相当の別名だった)。
  `Pipeline::new(device, queue, format)` が device を受け取る唯一の場所。
- `Primitive::draw` が `true` を返すと iced の render pass をそのまま使える。
  viewport と scissor は iced が widget の bounds に合わせてくれている。
- `window::latest() -> Task<Option<Id>>` + `Task::and_then` + `window::screenshot(id)`。
  `Screenshot` は `rgba`(sRGB, padding 無し)と `size` と `scale_factor` を持つ。
- 全体として「詰まったのは iced の API ではなく wgpu 29 の API 差」だった
  (`PipelineLayoutDescriptor` の `bind_group_layouts: &[Option<&_>]` と `immediate_size`、
  `RenderPipelineDescriptor::multiview_mask`、`push_error_scope` が guard を返すようになった点)。

## ビルドの重さ

- 依存グラフ: 836 package(Cargo.lock)。wgpu も egui もグラフ上に1つずつしかない。
- `cargo clean` からの `cargo build -j 5`(dev profile): **5分34秒**(Apple M4)
- target: 約 6.8 GiB / 11494 files
- probe 本体だけの増分ビルド: 2〜4 秒

## 測っていないこと

- `iced_test` / Simulator の統合、AccessKit、IME、タイムライン widget(発注時の範囲外)。
- **OS のマウス click の配送**。自動走行の (c) は widget の `Program::update` を直接叩いて
  型付き Message を取り出し、それを**ランタイム経由で** `App::update` へ流している。
  winit → widget tree の配送区間だけは `gui-interactive` で人が押して確かめる必要がある。
- 長時間動かしたときの安定性、resize の追従(size 変更時は stage を作り直す実装になっている)、
  MSAA の有無による差。
