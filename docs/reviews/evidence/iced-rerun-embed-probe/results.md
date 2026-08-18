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

---

# 追記 2026-08-18 — 入力ブリッジ (d)(e)(f)

上の (a)(b)(c) では egui を headless(`RawInput.events` が空)で回していたので、
`SpatialStage` 自身の対話は動いていなかった。カメラを動かしていたのは
iced の `Message` を受けた `set_camera` の直呼びである。
この追記は**その欠けている半分**を1本のブリッジで埋めて測った記録である。

引き続き**観測記録であって判断ではない**。

## ブリッジ

`spikes/iced-rerun-embed-probe/probe/src/bridge.rs`(442 行)。
iced の `shader::Program::update` が受けた `iced::Event` を `egui::Event` へ変換して
1フレーム分溜め、`Primitive::prepare` が `egui::RawInput.events` に載せる。
egui-winit がやっていることの iced 版で、実質的な違いは2点だけだった。

- **原点**: egui の座標は「stage が描かれている領域」の左上原点。iced は
  `Program::update` に widget の `bounds`(論理座標)をくれるので引ける。
- **pixels_per_point**: `Offscreen` は widget の物理画素数で作られ
  `set_pixels_per_point(1.0)` にしてあるので、egui 座標 = 論理座標 × scale factor。
  scale factor は `Viewport` にしかないので、キューには論理座標で積んで prepare で掛ける。
  `ScrollDelta::Pixels` は論理 point のまま `MouseWheelUnit::Point` へ(掛けると2倍速く回る)。

`set_camera` はこの経路では一度も呼んでいない。動かしているのは Rerun 自身の
`EyeController` である。

## (d) orbit / zoom — **到達 PASS / 絵の追随 FAIL(iced の device 限定)/ 姿勢の保持 FAIL(device 非依存)**

台本(`bridge_app.rs::script`)は 1 step = 1 egui フレームで、
`clear_camera()` → 補間が落ち着くまで待つ → 左ドラッグ12手 → 離す →
目印が消えるまで待つ → ホイール6ノッチ、と進む。

### 到達

- 左ドラッグ12手で `SpatialStage::last_eye()` が動いた。
  窓ありの走行で 0.15286、窓なしの対照群で 0.07682(初期姿勢の落ち着き具合の差)。
- ホイール6ノッチで軌道半径が変わり、eye が 0.457 動いた。
- **`set_camera` は呼んでいない**。iced → bridge → `egui::RawInput` →
  `SpatialStage::show` → `EyeController::handle_input` まで一続きに通っている。

### 絵の追随

Rerun の 3D view は**操作が始まった瞬間に orbit 中心の目印を線分で描く**
(`ui_3d.rs` の "center orbit orientation help"。最後の操作から 0.35 秒 + fade 0.1 秒)。
その1バッチが `LineRenderer` を使うので、iced の device では毎フレーム

```
In a CommandEncoder, label = '/'
  In a set_pipeline command
    RenderPipeline with 'LineRenderer::render_pipeline_color_opaque' label is invalid
```

となり、コマンドバッファごと捨てられて offscreen texture が更新されない。

**同じ台本を2つの device 記述で回した対照群**:

| device 記述 | ドラッグ18フレームの絵 | そのうち検証層に蹴られたフレーム |
|---|---|---|
| `re_renderer`(`max_bind_groups = 4`) | **18通り** — 毎フレーム変わる | **0** |
| `iced-windowed`(`max_bind_groups = 2`) | **2通り** — 2フレーム目で固まる | **16** |

`last_eye()` の数列は**両者で完全に一致する**(どちらも 0.07682)。
入力は同じだけ届いていて、違うのは絵が出るかどうかだけである。

窓なし `iced-windowed` の per-tick 表がいちばん見やすい:
tick 73 まで検証エラーは通算1(warmup の `create_pipeline_layout` 1件)のまま
digest が毎行変わり、tick 74 から digest が凍って検証エラーが**1フレームにつき1件**
増え、tick 111 で増加が止まると同時に digest がまた動き出す。
目印が消えると絵が戻る、という筋がそのまま数字に出ている。

### 姿勢の保持 — iced とは無関係の壁

`EyeController::save_to_blueprint` はドラッグの結果を
`SystemCommand::AppendToStore` でブループリントへ書き戻す。
`SpatialStage::process_system_commands` は `SetSelection` と `SetFocus` しか見ておらず
`AppendToStore` は `_ => {}` で落ちる。ブループリントが空のままなので
`EyeController::from_blueprint` は毎フレーム fallback(シーンの自動フレーミング)を返し、
fallback を使っている間は `start_interpolation()` が呼ばれ続ける。

- そのフレームの drag_delta は効く
- 次のフレームには既定の姿勢へ向かって補間で戻り始める
  (ボタンを押したまま1フレーム止めるだけで 0.059 戻った)
- 64 フレーム後にはほぼ元の位置(0.15286 → 0.00319)

**対照群でも同じように戻る**ので、これは iced の話ではなく fork の seam の話である。

## (e) cursor icon — **写せる。1フレーム遅れる。OS カーソルの実変化は未確認**

- 写像は `bridge::to_iced_interaction`。egui の 26 種のうち `VerticalText` を除く全部が
  `iced::mouse::Interaction` に対応した(8方向のリサイズは iced の2軸+2斜めへ丸める)。
- 走行中に Rerun が要求したのは `Default` と `Crosshair` の2つだけで、どちらも写せた。
- **ホバー中は `Crosshair`、ドラッグ中は `Default`**。Rerun は orbit 中に
  `Grab`/`Grabbing` を要求しない。
- `platform_output.cursor_icon` は `show` を回した後にしか読めず、
  `Program::mouse_interaction` はその次のフレームに呼ばれる。**構造的に1フレーム遅れる**。
- 自動走行は `Program::mouse_interaction` の戻り値までしか見ていない。
  OS のカーソルが実際に変わるかは `bridge-interactive`(未実施)の担当。

## (f) 制約 — repaint と IME に口が無い / キーボードは素通し / フォーカス調停は存在しない

- **repaint**: egui は毎フレーム `ViewportOutput::repaint_delay` を出す
  (この走行では `0` と「無限」の2値)。iced 側で答えられるのは `Program::update` が返す
  `Action::request_redraw_at` だけで、**`Primitive::prepare` の戻り値は `()`**。
  「入力が無いのに egui が再描画したい」(補間・アニメーション・非同期読み込み)を
  伝える経路が無い。probe は `iced::window::frames()` で毎フレーム描いて回避している。
- **IME**: iced の IME は `Widget::update` の中で `Shell::request_input_method` を呼んで
  有効化する。`shader::Program::update` は `Shell` を受け取らないので**呼ぶ手段が無い**。
  `shader::Event` は `iced::Event` そのものなので `InputMethod` は型としては届きうるが、
  要求していない以上イベントは来ない。egui の `platform_output.ime` も行き先が無い。
- **キーボード**: iced は**すべてのイベントをすべての widget に配る**
  (`user_interface.rs`: overlay が capture しない限り `root.update` を必ず呼ぶ)。
  focus による絞り込みは無いので、`Key`/`Text` を翻訳すれば届く。今回は実装していない。
- **フォーカスの奪い合い**: 起きない。調停が存在しないからである。iced の focus は
  `operation::focusable` を実装した widget が自分で名乗る規約で、shader widget は
  実装していない。egui が `wants_pointer_input = true` と言っても iced は見ない。
  捕まえたければ埋め込み側が `Action::and_capture()` を返して自分で決める。
  本 probe は返していない(窓いっぱいで競合が無く、捕まえると観測自体を潰すため)。

## 生成物(追記分)

| ファイル | 中身 |
|---|---|
| `interactive-bridge-run.txt` | 窓ありの走行の全ログ(verdict・per-tick 表・step ログ) |
| `interactive-bridge-00..09-*.png` | 窓ありの走行の各節目(offscreen texture。`09-window` だけ窓の screenshot) |
| `interactive-bridge-offscreen-re_renderer-run.txt` | 対照群: `re_renderer` 記述の device で同じ台本 |
| `interactive-bridge-offscreen-re_renderer-*.png` | 同上の各節目(640x480) |
| `interactive-bridge-offscreen-iced-windowed-run.txt` | 対照群: iced 記述の device で同じ台本 |
| `interactive-bridge-offscreen-iced-windowed-*.png` | 同上の各節目(640x480) |

窓ありの `interactive-bridge-02-during-drag.png` と対照群の
`interactive-bridge-offscreen-re_renderer-02-during-drag.png` を並べると、
「同じカメラで、片方だけがドラッグ中の絵を更新できている」ことが目で見える。
