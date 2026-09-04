# ホットリロードの利益を取る — 置き換え可能な関数群と tip crate の設計

- 日付: 2026-09-03
- 位置づけ: **設計提案**(コードに触らない上流工程)。事実(§0)と、委託先(§1)と、その上に置く形(§2)を分ける
- 発端: 利用者「現在は常にビルドし直して Dioxus の旨味を何も取れていない。置き換え可能な関数群を事前に
  作っておき、内部をノードツールのように組み替えることで共通化とホットリロードの最適化を目指したい。可能か」
- 答えの一行: **可能。ただし今は起動の仕方の時点で何も効いていない。**機構は 4 つ既に手元にあり、
  足りないのは「どの関数を tip crate に置くか」の切り方だけ

## 0. 現在地(2026-09-03、`main` c2c7a89 と手元の依存を読んだ事実)

| 機構 | 在る | 効いているか | 根拠 |
|---|---|---|---|
| **RSX テンプレートのホットリロード** | `dioxus-native` の `hot-reload` feature(probe は有効)。`DevserverMsg::HotReload` → `apply_changes` | **効いていない**。`dioxus_devtools::connect` は devserver(`dx serve`)が居る時だけ繋がる。`cargo run` では無音 | `dioxus-native/src/lib.rs:167-171`、`dioxus_application.rs:66-71` |
| **asset のリロード** | `HotReload` の `assets` を `reload_resource_by_href` で差し替え | **効いていない**。`styles.css` は `include_str!` で焼いている(`app.rs:19`)ので href が無い | `dioxus_application.rs:73-78`、`blitz-dom/src/document.rs:1078` |
| **関数単位のホットパッチ(subsecond)** | Dioxus 0.7.10、`dioxus-devtools` が `apply_patch` を持つ。`dx serve --hotpatch` | **効いていない**(同上、devserver 不在)。効いても **tip crate = `probe` だけ**。`motolii-store`/`engine` の変更は全ビルド | `subsecond-0.7.10/src/lib.rs` 冒頭・「Workspace support」節 |
| **re_renderer シェーダのディスク読み** | `.cargo/config.toml` の `IS_IN_RERUN_WORKSPACE=1` で `load_shaders_from_disk` | **効いている**(motolii/ の中から回した時) | `motolii/.cargo/config.toml` |

subsecond の制約(公開文書に明記、設計に効く物だけ):

- **tip crate のみ**(`main.rs` のある crate)。他 crate の変更は無視される
- **struct のレイアウト変更は不可**(旧レイアウトを参照する新関数が落ちる)。Dioxus は「古い状態を捨てて作り直す」で回避
- static の初期化子の変更は反映されない。thread-local は patch でリセットされ得る(「HUGE WARNING」)
- patch 点は `subsecond::call` を通した所。Dioxus は component と handler を `HotFn` で包んでいる(`dioxus-devtools/src/lib.rs:198`)。
  **custom widget の `paint`/`handle_event` は Blitz が直接呼ぶので包まれていない**

## 1. 委託先(発明しない)

| 借りる物 | 出典 | ここで担う役 |
|---|---|---|
| **Functional core, imperative shell** | Gary Bernhardt(2012、講演) | 純関数(値→値)を核に、副作用(GPU・window・Document 書き込み)を薄い殻へ。**核が置き換え可能な関数群、殻が tip crate の配線** |
| **The Elm Architecture** | Elm guide「The Elm Architecture」 | `update: Msg × Model → Model`、`view: Model → Html`。Motolii では `Intent` が Msg、`Document` が Model、`rsx!` が view。**既にこの形**(裁定「Document への書きは Intent 経由のみ」) |
| **subsecond の「Nesting Calls」** | `subsecond` crate docs | patch 点は入れ子にでき、外側は初期化(副作用)、内側は頻繁に変える関数。**包む場所の設計指針そのもの** |
| **Bevy の hotpatching** | Bevy 0.16 リリースノート「Hotpatching systems」 | ゲームエンジンが system(素の `fn`)を subsecond で差し替える先例。**素の関数 + 安定した引数型**という切り方の実証 |
| **ノードツール** | Blender Geometry Nodes / Houdini(型付きポートの純関数を data で配線) | 「内部をノードのように組み替える」の外部像。ノード = 純関数、配線 = 表(データ) |

利用者の言う「置き換え可能な関数群 + ノードのような組み替え」は、この表では
**Functional core(関数群)+ Elm の update/view(型)+ 配線を表で持つ(ノードツール)**、の 3 つの合成であって、新しい物は無い。

## 2. 設計(§0 の制約の上に §1 を置く)

### 2.1 切り方の規則は 2 本

1. **よく変える物は tip crate(`probe`)の素の関数に置く。struct は下の crate に置く。**
   subsecond が差し替えられるのは tip crate の関数だけで、struct のレイアウト変更は不可。だから
   「引数と戻り値の型は `motolii-store`/`core` の安定した型(`Intent`・`LayerId`・`StoreView`・`RationalTime`)、
   本体は probe の `fn`」にすれば、本体の変更は hot-patch、型の変更だけが全ビルド、と自然に分かれる。
2. **副作用は殻に 1 段だけ。核は値→値。**
   核の関数は `&StoreView` を読み `Vec<Intent>` を返す(書かない)、`&Scene` を受け取って描く(GPU 資源を作らない)、
   `&UiEvent` を受け取って `Option<Msg>` を返す(状態を持たない)。殻(`app.rs`・widget の `impl Widget`)が
   `apply_all`・GPU 資源・window を持つ。**核は state を持たないので「古い状態を捨てる」問題が起きない。**

### 2.2 何を置き換え可能にするか(変える頻度の順)

| 関数群 | 今の場所 | 形(核) | 殻 | 効く機構 |
|---|---|---|---|---|
| **動詞**(Intent を作る) | `dispatch.rs` の `run_intent` の各 arm、`timeline_widget.rs` の drag 確定、`stage_widget.rs` のギズモ確定 | `fn(&StoreView, LayerId, Payload) -> Vec<Intent>`([持ち上げ](2026-09-03-selection-lifting.md)の動詞台帳と同じ物) | `lift` + `apply_all` | hot-patch |
| **献立**(右クリック menu) | `context_menu.rs::entries` | `fn(MenuTarget) -> Vec<Entry>`(既に純関数) | `context_menu` の rsx | hot-patch + RSX |
| **打鍵表** | `keymap.rs::BINDINGS`(static) | **static のままだと変更が反映されない**(§0)。`fn bindings() -> &'static [Binding]` か `fn lookup` 内の `match` へ | `onkeydown` | hot-patch |
| **層行・帯の抽出** | `fixture.rs::layer_rows_from_doc`/`canvas_rows_from_doc` | 既に `fn(&Document) -> Vec<Row>` | `refresh_rows` | hot-patch |
| **Inspector の行の並び・条件付き節** | `inspector.rs` の rsx | rsx literal・attribute・構造 | — | **RSX**(dx serve だけで効く、コード変更不要) |
| **custom widget の描画** | `timeline_widget.rs::paint`(`impl Widget` の中) | `fn paint_timeline(&TimelineState, &mut Scene, w, h, scale)` を切り出し | `impl Widget for TimelineWidget { fn paint { subsecond::call(\|\| paint_timeline(..)) } }` | hot-patch(**殻で包む 1 行が要る**) |
| **当たり判定・ジェスチャの判定** | `hit_test`/`band_hit`/`handle_event` | `fn(&TimelineState, x, y) -> Hit`、`fn(&UiEvent, &State) -> Option<Msg>` | 同上 | hot-patch |
| **CSS** | `styles.css` を `include_str!` | ファイルのまま | `asset!("./styles.css")` を `document::Stylesheet` で head へ(Blitz は `CreateHeadElement` を持つ) | **asset**(dx serve だけで効く) |
| **トークン**(`css_root`) | `tokens.rs` が文字列生成 | 生成を `fn` のまま(static にしない) | `style {}` | hot-patch |
| **シェーダ** | `shader.wgsl`(probe)、re_renderer | — | — | 既に効く(re_renderer 側)。probe の wgsl も同じ経路へ |

「ノードのように組み替える」は、この表の**核関数を配線する表**が data であること。動詞台帳(持ち上げ文書 §3)が
そのまま配線表で、`keymap` と `context_menu::entries` は既にその形をしている。

### 2.3 置かない物(規則 1 の裏)

- **struct 定義**(`TimelineWidget`・`StageWidget`・`IntentCtx`)。変えれば全ビルド。頻度が低いので許容する
- **GPU 資源・`State::Active`・wgpu device を握る物**。殻の持ち物。patch で「捨てて作り直す」と GPU 状態が飛ぶので、
  核に置かない = 飛ばない
- **static/thread-local に置いた設定**(打鍵表・献立・トークン)。§0 の制約でそのまま反映されない。関数にする

### 2.4 起動

`motolii/` の中で `dx serve --hotpatch -p dioxus-native-probe`(dx CLI 0.7 系)。`.cargo/config.toml` の env は
cargo を通るので継承される(**要実測**、§C)。`cargo run` は「ホットリロード無し」の起動として残す。

## 3. 不変量(核が純関数であることの検分)

| 不変量 | 測り方 |
|---|---|
| **核は書かない** | `probe` の核関数の引数に `&mut Document`/`Arc<Mutex<Document>>` が無い(grep 1 本で柵にできる) |
| **同じ入力に同じ出力** | 動詞・献立・抽出・当たり判定は `Document` 水準の test で決定的(既に `context_menu`/`keymap`/`dispatch` の test がこの形) |
| **殻は薄い** | `impl Widget` の各メソッドが「状態を取り出して核を 1 回呼ぶ」以上の分岐を持たない |
| **patch 後も Document は同じ** | hot-patch は Document(store)に触れない。patch 前後で `Document` のバイトが同じ |

## A. 利用者裁定待ち

| 件 | 分岐 |
|---|---|
| 配線を **表(データ)** にするか **素の Rust 呼び出し** にするか | 推し = 表は動詞台帳・打鍵表・献立の 3 つだけ(既にその形)。それ以外は素の呼び出し。表を増やしすぎると Rust の型検査を捨てることになる |
| widget の `paint` を核へ切り出す際、`State` を **読み取り専用の view struct** にするか | 推し = する(`&TimelineState`)。描画中に state を変える経路(`process_messages`)は殻へ |
| `dx serve` を正の起動にするか | 推し = 開発は `dx serve --hotpatch`、検分器具(`MOTOLII_TILT` 等)は `cargo run` のまま |

## B. 順序(器が在る)

1. **`dx serve` で RSX が差し替わることを実測**(コード変更ゼロ。効けば Inspector の並び替えが即座に見える)
2. `styles.css` を `asset!` + `document::Stylesheet` へ(CSS のホットリロード)
3. `dx serve --hotpatch` を実測(tip = probe。`entries`/`layer_rows_from_doc` を変えて反映を見る)
4. 動詞・打鍵表を関数へ(static を外す)。[持ち上げ](2026-09-03-selection-lifting.md)の B-1 と同じ commit で済む
5. `paint`/`hit_test`/`handle_event` の核を切り出し、殻を `subsecond::call` で包む

1〜3 は設計を待たない。4〜5 が「置き換え可能な関数群」の本体で、持ち上げの設計と同じ切り方になる。

## C. 先に測る

- `dx serve` が dioxus-native + custom widget + 共有 wgpu device の構成で動くか(Blitz の example は動く。共有 device は未実測)
- hot-patch 時に Dioxus が vdom を作り直すと `use_hook` の `CustomWidgetAttr::new(stage)` も作り直されるか
  (= GPU 状態の再初期化が毎 patch で起きるか)。起きるなら Stage の `State::Active` を patch をまたいで持つ形が要る
- `.cargo/config.toml` の env が `dx serve` 経由で re_renderer の build.rs に届くか
