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
| (d) | iced の入力を egui の `RawInput` へ翻訳すると、`SpatialStage` **自身の** orbit / zoom が動く | **到達は PASS。絵の追随は iced の device で FAIL。姿勢の保持は device に関係なく FAIL** |
| (e) | egui が要求する cursor icon を `Program::mouse_interaction` へ写せる | **PASS(1フレーム遅れ。実際の OS カーソル変化は未確認)** |
| (f) | repaint 要求 / IME / キーボード / フォーカスの経路 | **repaint と IME に口が無い。キーボードは素通し、フォーカス調停は存在しない** |

証拠(PNG と実行ログ)は `docs/reviews/evidence/iced-rerun-embed-probe/`。
(d)(e)(f) の分は `interactive-bridge-` 接頭辞。

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

# (d)(e)(f)。窓を開き、合成した drag / wheel の列を台本どおり流して自分で終了する。
cargo run -j 5 -- bridge <出力ディレクトリ>

# 同じ (d)(e)(f) を人のドラッグで。窓は開いたまま待つ(未実施の口)。
cargo run -j 5 -- bridge-interactive <出力ディレクトリ>

# (d) の対照群。窓を開かずに**同じ台本**を、指定した device 記述の上で回す。
cargo run -j 5 -- bridge-offscreen re_renderer <出力ディレクトリ>
cargo run -j 5 -- bridge-offscreen iced-windowed <出力ディレクトリ>
```

`cargo test` は oracle・素材・**翻訳と台本**の単体テストを回す(GPU 不要)。

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

### 7. 入力の翻訳そのものは薄い(壁ではない)

`probe/src/bridge.rs`(442 行、うち約半分は説明とテスト)が iced の
`Program::update` が受けた `iced::Event` を `egui::Event` へ変換して1フレーム分溜め、
`Primitive::prepare` が `egui::RawInput.events` へ載せる。egui-winit がやっていることの
iced 版で、差は2つだけだった。

- **原点**。egui へ渡す座標は「stage が描かれている領域」の左上原点。iced は
  `Program::update` に widget の `bounds`(論理座標)をくれるので引ける。
- **pixels_per_point**。`Offscreen` は widget の**物理**画素数で作られ、
  `egui_ctx.set_pixels_per_point(1.0)` にしてあるので、論理座標 × scale factor が
  egui の座標になる。scale factor は `Viewport` にしか無いので、キューには論理座標のまま
  積んで prepare 側で掛ける。`ScrollDelta::Pixels` は論理 point のままで
  `MouseWheelUnit::Point` に載せる(ここで scale を掛けると2倍速く回る)。

翻訳の結果は数値で確かめてある。左ドラッグ12手で `SpatialStage::last_eye()` が動き、
ホイール6ノッチで軌道半径が変わる。**`set_camera` は一度も呼んでいない** — 動かしたのは
Rerun 自身の `EyeController` である。

### 8. `LineRenderer` は「作れない」だけでなく「触った瞬間に毎フレーム落ちる」

(a) の但し書きは「gizmo が出ないだけ」ではなかった。Rerun の 3D view は
**操作が始まった瞬間に orbit 中心の目印を線分で描く**(`ui_3d.rs` の
"center orbit orientation help"、最後の操作から 0.35 秒 + fade 0.1 秒だけ出る)。
その1バッチが `LineRenderer` を使うので、iced の device では

```
In a CommandEncoder, label = '/'
  In a set_pipeline command
    RenderPipeline with 'LineRenderer::render_pipeline_color_opaque' label is invalid
```

となり、**コマンドバッファごと捨てられて offscreen texture が更新されない**。
つまり「ドラッグしている間だけ絵が止まり、手を離して 0.45 秒ほど経つと動き出す」。

同じ台本を2つの device 記述で回した対照群がそれを1行で言い切っている。

| device 記述 | ドラッグ18フレームの絵 | 検証エラー |
|---|---|---|
| `re_renderer`(`max_bind_groups = 4`) | **18通り**(毎フレーム変わる) | 0 |
| `iced-windowed`(`max_bind_groups = 2`) | **2通り**(2フレーム目で固まる) | 16 |

`last_eye()` の数列は**両者で完全に一致する**(どちらも 0.07682 動いた)。
入力は同じだけ届いていて、違うのは絵が出るかどうかだけである。
証拠: `interactive-bridge-offscreen-{re_renderer,iced-windowed}-run.txt`。

### 9. orbit は残らない — `SpatialStage` がブループリント書き戻しを握り潰す

これは iced とは無関係で、**対照群でも同じように失敗する**。

Rerun の 3D カメラの姿勢はブループリント (`EyeControls3D`) が正本で、ドラッグの結果は
`EyeController::save_to_blueprint` → `SystemCommand::AppendToStore` で書き戻される。
`SpatialStage::process_system_commands` は `SetSelection` と `SetFocus` しか見ておらず、
`AppendToStore` は `_ => {}` で落ちる。ブループリントに何も無いままなので
`EyeController::from_blueprint` は毎フレーム fallback(シーンの自動フレーミング)を返し、
さらに fallback を使っている間は `start_interpolation()` が呼ばれ続ける。

結果として:

- そのフレームの drag_delta は効く(絵も eye も動く)
- 次のフレームには既定の姿勢へ向かって補間で戻り始める
- 64 フレーム後にはほぼ元の位置に戻っている(0.15286 → 0.00319)

「入力は通る。姿勢が残らない」。埋め込み側から Rerun のカメラを**持続的に**動かす道は、
今のところ `set_camera`(こちらは sticky)しか無い。

### 10. egui の再描画要求に答える口が `Primitive::prepare` に無い

egui は毎フレーム `ViewportOutput::repaint_delay` で「次はいつ描いてほしいか」を言う。
この走行では `0`(すぐ)と「無限」の2値が観測された。iced 側で答えられるのは
`Program::update` が返す `Action::request_redraw_at` だけで、**`Primitive::prepare` の
戻り値は `()`** である。つまり「入力が無いのに egui が再描画したい」場合
(補間・アニメーション・非同期の読み込み)を伝える経路が無い。

probe は `iced::window::frames()` で毎フレーム描いて回避している。実運用では
「常時再描画」か「`prepare` から `window::RedrawRequest` を送れる seam を iced に足す」の
どちらかになる。

### 11. shader widget は IME を要求できない

iced の IME は `Widget::update` の中で `Shell::request_input_method` を呼んで有効化する。
`shader::Program::update` は `Shell` を受け取らないので、**呼ぶ手段が無い**。
`shader::Event` は `iced::Event` そのものなので `InputMethod(Preedit/Commit)` は型としては
届きうるが、要求していない以上イベントは来ない。egui 側 (`platform_output.ime`) の
出力も同じ理由で行き先が無い。今回は実装せず、経路の有無だけ確かめた。

### 12. フォーカスの奪い合いは「起きない」— 調停が存在しないから

iced は**すべてのイベントをすべての widget に配る**
(`user_interface.rs`: overlay が capture しない限り `root.update` を必ず呼ぶ)。
キーボードイベントも focus に関係なく shader widget に届く。iced の focus は
`operation::focusable` を実装した widget が自分で名乗る規約で、shader widget は
それを実装していない。したがって

- egui が `wants_pointer_input = true` と言っていても、iced はそれを見ない
- 捕まえたければ埋め込み側が `Action::and_capture()` を返して**自分で決める**
- 逆に何もしなければ他の widget にも同じイベントが流れる

本 probe は `and_capture()` を返していない(窓いっぱいで競合が無く、
捕まえると (f) の観測そのものを潰すため)。

### 13. cursor icon は写せる。ただし1フレーム遅れる

`platform_output.cursor_icon` は `SpatialStage::show` を回した**後**にしか読めず、
`Program::mouse_interaction` はその次のフレームに呼ばれる。写像自体は
egui の 26 種のうち `VerticalText` を除く全部が `iced::mouse::Interaction` に対応した
(8方向のリサイズは iced の2軸+2斜めへ丸める)。走行中に Rerun が要求したのは
`Default` と `Crosshair` の2つだけで、どちらも写せた。

なお **ホバー中は `Crosshair`、ドラッグ中は `Default`** だった。Rerun は orbit 中に
`Grab`/`Grabbing` を要求しない。「掴んでいる感」を出したいなら埋め込み側が足す話になる。

自動走行は `Program::mouse_interaction` の戻り値までしか見ていない。
**OS のカーソルが実際に変わるか**は `bridge-interactive` で人が確かめる必要がある(未実施)。

## ビルドの重さ

- 依存グラフ: 836 package(Cargo.lock)。wgpu も egui もグラフ上に1つずつしかない。
- `cargo clean` からの `cargo build -j 5`(dev profile): **5分34秒**(Apple M4)
- target: 約 6.8 GiB / 11494 files
- probe 本体だけの増分ビルド: 2〜4 秒

## 測っていないこと

- `iced_test` / Simulator の統合、AccessKit、タイムライン widget(発注時の範囲外)。
- **IME とキーボードの実装**。経路の有無だけ調べた(上の 11 番)。`bridge.rs` が翻訳するのは
  mouse move / press / release / wheel と `ModifiersChanged` だけで、`Key` / `Text` /
  `Ime` は意図的に落としている。
- **OS のマウス入力の配送**。自動走行の (c)(d)(e)(f) はいずれも widget の
  `Program::update` を直接叩いている。翻訳から先(egui・Rerun・GPU)は全部本物だが、
  winit → widget tree の配送区間だけは通っていない。そこは `gui-interactive` /
  `bridge-interactive` で人が押して確かめる必要がある。**どちらも未実施**。
- **OS カーソルが実際に変わるか**(上の 13 番)。写像と `Program::mouse_interaction` の
  戻り値までしか見ていない。
- 長時間動かしたときの安定性、resize の追従(size 変更時は stage を作り直す実装になっている)、
  MSAA の有無による差。
- Rerun 側の picking(クリックでエンティティを選ぶ)。入力は届くようになったので
  `take_selected_entity_path()` が動くはずだが、今回は測っていない。
