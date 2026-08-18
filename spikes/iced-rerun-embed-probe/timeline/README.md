# iced で書いた仮タイムライン(DX 実測用)

同じ workspace の `../probe` が「iced に Rerun を埋められるか」を測ったのに対し、
こちらは **製品の本丸=密な編集面を iced の Elm 構造で書くと何が起きるか** を測る。

採用の判断はしない。測ったことだけを置く。判断は検収側が survey へ折り返す。

## 走らせ方

```sh
cd spikes/iced-rerun-embed-probe
./setup.sh                                        # 初回だけ。iced を vendor/ に取ってくる

cargo run -j 5 --release -p iced-timeline-probe               # 24 clips / 12 track
cargo run -j 5 --release -p iced-timeline-probe -- --clips 500  # 500 clips(全部入る zoom で開く)

cargo test -j 5 -p iced-timeline-probe            # 4ジェスチャの判断部分だけ、窓なしで 20 本
```

frame 時間は**窓のタイトルと画面下端の両方**に出る。
`--release` を付けないと draw 時間が 10 倍ほど遅く出るので、数字を見るときは付ける。

## 触れるもの

| | 操作 | 備考 |
|---|---|---|
| 移動 | clip 本体をドラッグ | フレームグリッドへスナップ。複数選択は相対配置を保ったまま動く |
| トリム | clip の端 8px をドラッグ | カーソルが `ResizeHorizontal` に変わる。1フレーム未満に潰れない |
| スクラブ | ruler(上端 24px)をクリック/ドラッグ | playhead。押しただけでも動く |
| 横 zoom | **Cmd+ホイール** | カーソル下のフレームが動かない |
| 横パン | **Shift+ホイール** / trackpad の横スワイプ | |
| 縦スクロール | 素のホイール | |
| 選択 | クリック / **Cmd+クリック**で追加・解除 | 選択済みを素で押しても選択は潰れない |
| 送り | **←→** = 1フレーム、**Shift+←→** = 10フレーム | 選択があれば clip、無ければ playhead |

範囲外(意図的): 分割・キーフレーム・イージング・undo・ラバーバンド選択・
snap 候補(clip 端や playhead への吸着)・ドラッグ中の端パン。

行高 20px と端 hit 幅は製品と揃えてある。**端 hit 幅は 8px** で、
これは発注書の「7px」ではなく製品の実値である
(`crates/motolii-ui/src/timeline_editor/mod.rs:939` `const TRIM_EDGE: f32 = 8.0;`。
7px はモックの `.trimHandle{width:7px}` = **見た目の帯**の幅で、掴める幅ではない)。

## 実測(2026-08-18 / M シリーズ Mac / release ビルド / 1280×520 の窓)

| clips | 画面に描いた数 | `draw()` 幾何組み立て | フレーム間隔 |
|---|---|---|---|
| 24 | 24 | 0.04 ms | 16.5 ms |
| 500 | **500**(culling 無しで全部) | 0.27 – 0.37 ms | 16.5 ms |
| 5000 | **5000** | 1.57 ms | 16.6 ms |

フレーム間隔はどれも 16.5 ms、つまり **60Hz の vsync に張り付いたまま**で、
5000 clips でも描画が律速になっていない。連続再描画は
`program.rs` の `RedrawRequested → Action::request_redraw()` で強制している
(計測のためであって、製品でやることではない)。

canvas の `Cache` は**使っていない**。egui の即時モードと同じ「毎フレーム作り直し」に
揃えないと比較にならないため。`Cache` を入れれば静止時の 0.27 ms は 0 になる。

証拠 PNG: `docs/reviews/evidence/iced-timeline-probe/`

## 行数

| 役割 | ファイル | 総行 | コメント・空行を除く |
|---|---|---|---|
| データモデル + 射影 + hit test | `model.rs` | 274 | 175 |
| Message(意図の一覧) | `message.rs` | 67 | 22 |
| `update()` + ジェスチャ状態機械 | `app.rs` | 305 | 229 |
| イベント → 意図の翻訳 + カーソル | `program.rs` | 205 | 132 |
| 描画 | `view.rs` | 252 | 201 |
| 窓の配線 + 引数 | `main.rs` | 107 | 62 |
| テスト | `tests.rs` | 404 | 342 |
| **合計** | | **1,614** | **1,163** |
| **テストを除く** | | **1,210** | **821** |

ジェスチャ別の内訳(コメント込み。行域は下の対応表を参照):

| ジェスチャ | 固有の行 |
|---|---|
| 移動 | 33 |
| トリム | 24 |
| スクラブ | 8 |
| zoom / パン / 縦スクロール | 62 |
| 矢印送り | 35 |
| 5つで共有している配線(hit test・押す/引く/離すの中継・`Drag` 型・カーソル・選択) | 180 |
| **計** | **342** |

## 詰まった箇所

### 1. ホイールイベントが修飾キーを運んでこない

`iced::mouse::Event::WheelScrolled { delta }` には `modifiers` が無い
(`core/src/mouse/event.rs`)。`KeyPressed` / `KeyReleased` は持っているのに、
ホイールだけ落ちている。Cmd+ホイール = 横 zoom を書くには、
`Event::Keyboard(ModifiersChanged)` を別に購読して現在値を持ち回るしかない。

この spike では `main.rs` の `subscription` で拾って `App::modifiers` に入れ、
`view()` 経由で canvas へ渡している(`program.rs:104-136`)。
egui は `i.modifiers.command` を欲しい場所で1行読めば済む
(`timeline_editor/mod.rs:4634-4643`)。**iced 側が明確に不利な唯一の点**。

### 2. canvas にフォーカスの概念が無い

`Program::update` はこの窓に届いたキーイベントを**全部**見る。canvas が2枚あれば
2枚とも矢印キーを拾う。誰が持っているかを決める仕組みは iced 側に無いので、
複数面になった時点で自前の調停が要る。
(`../probe` の「12. フォーカスの奪い合いは起きない — 調停が存在しないから」と同じ穴。)

### 3. `iced::application(state, update, view)` にクロージャを渡すと通らない

```rust
|app: &App| -> Element<'_, TimelineMsg> { ... }
// error: implementation of `ViewFn` is not general enough
```

クロージャの推論が `for<'a> Fn(&'a App) -> Element<'a, _>` を作れない。
`fn` 項目にすれば HRTB が付いて通る(`main.rs` の `fn view`)。
API がクロージャを受け取る顔をしているので、素直に書くと必ず1回踏む。
Rust 側の制限であって iced の設計ではないが、踏むのは iced を書く人である。

### 4. カーソル変更は素直だった

`Program::mouse_interaction` が `mouse::Interaction` を返すだけ。
`update` と**同じ `hit_test` 純関数**を通せるので、
「trim のカーソルが出ているのに掴んだら移動した」という種類のズレが
構造的に起きない。egui 側は hover の書き込みと hold の上書きを
描画順で調停している(`mod.rs:4326-4337` と `mod.rs:4853-4855`)。

### 5. `Action` は1イベントにつき Message を1つしか出せない

`Program::update` の戻りは `Option<Action<Message>>` で、`Action::publish` は
Message 1つ。「選択を変えて、かつドラッグを開始する」のように2つの意味を
同時に起こしたい場合は、複合 Message を1つ作るしかない
(この spike の `ClipGrabbed` が `additive` を抱えているのはそれ)。
`Task` を返せる `update()` 側と違って、canvas 側には分割の口が無い。

## 状態をどこに置いたか(比較の前提)

iced の canvas は `Program::State` という **widget ローカルな可変状態**を用意しており、
進行中ジェスチャはそこに置ける(上流の `examples/bezier_tool` がそうしている)。

この spike は **`type State = ();` にして使わなかった**。
「全部 Message にしたら何行になるか」を測るのが目的なので、
使うと測りたい物が測れなくなるため。代償は 2 つ:

1. `App::Drag::Move` が押した瞬間の clip 位置(`origins`)を**モデルの複製として**持つ。
2. ドラッグ中は毎マウス移動で Message が1本飛ぶ。

得たものは 1 つで、これが大きい:

> **`draw()` は `&self` でモデルを借りるので、描画中にモデルを書く道が型として無い。**

`view.rs` にモデルへの `&mut` は1つも無い(計測用の `AtomicU64` 2本だけが例外で、
これはモデルの意味に触らない)。モデルを書き換える文は
`app.rs` の `update()` / `apply_selection()` と `model.rs` の `Viewport::zoom_at()` に
**16 文**あり、それが全部である。
