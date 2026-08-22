# iced(fork)標準機能の未使用在庫 — 全網羅の棚卸し

日付: 2026-08-22 / 発注: 利用者「私の目の届かないもの — iced ベースの標準機能でまだ使っていないものはないか。iced_aw 以外に使えそうな部品はあるか、それとももう大丈夫か」/ レーン: read-only 調査(コード変更禁止・成果物は本文書1本)

対象 pin: `iced = { git = "https://github.com/oshikaidesu/iced", rev = "73e686ee05efd7d1b61cfea2647186b336d9ab9c", features = ["wgpu", "advanced", "image", "canvas"] }`(`next/Cargo.toml:78`、fork branch `motolii/host-seams`、裁定170)。
在庫列挙の一次資料: cargo checkout 実体 `/Users/member_ottoto/.asdf/installs/rust/stable/git/checkouts/iced-1bbb4ed9d90ae4f8/73e686e/`(CARGO_HOME は asdf 配下 — `~/.cargo` は存在しない)。**全行がこの checkout の mod 木・ソース read 由来**(docs.rs・記憶からの転記なし)。使用判定の grep は §2 冒頭に再現コマンドを併記。

先行資料: [native-menu+stock widgets survey(同日)](2026-08-22-native-menu-and-stock-widgets-survey.md)(TOP5 止まり — 本調査はその全網羅化)/ [iced_aw menu probe(同日)](2026-08-22-iced-aw-menu-probe.md)(E0308 実測)/ [iced エコシステム採掘(08-19)](2026-08-19-iced-ecosystem-mining.md)。

---

## 冒頭の答え: 「もう大丈夫か?」 → **概ね YES、ただし3点の NO**

**YES の根拠**: fork の widget 在庫 52 部品を機械列挙して照合した結果、Motolii が未使用の 37 部品のうち 30 は「現行の設計方針上、使う場面がない/自前実装が既にある/feature 未点灯で存在すらしない」であり、見落としによる取り逃しではない(§3 で1行ずつ判定)。canvas/shader/Task/Subscription/操作 walk という重い部分は既に深く使い込んでいる。

**NO(=目の届いていなかった実利)は3点**:

1. **Settings のモーダル overlay 化は `stack` + `opaque` + `center` で今すぐ実装可能**(§5)。fork ソースで確認: `stack` は `operate()` を実装し(`widget/src/stack.rs:183`)、`opaque` も `operate` を透過する(`widget/src/helpers.rs:640`)ため、**pick_list/overlay::menu が踏んだ「oracle から見えない」穴をこの経路は踏まない**。しかも `stack`/`shader` は shell が既に import 済み(`lib.rs:26`、Stage overlay の `stack!` で使用中)— 新規概念ゼロで届く。擬似コードを §5 に置いた(発注可能精度)。
2. **feature flag の向こうで眠っている開発器具**: `debug`(F12 devtools・メトリクス)/ `tester`(**e2e テストの記録・編集・再生**、`tester/src/lib.rs:1` "Record, edit, and run end-to-end tests")/ `time-travel`。Cargo.toml の feature 1行で点く fork 同梱品であり、外部 crate ではない。「落ちるテストで発注する」規律の器具として tester は検分価値が高い(§3.5)。
3. **`tooltip` と `toggler` が未使用のまま**: tooltip はショートカット表示(メニュー/headerボタンへの併記)に1関数ラップで効く。toggler は Settings の bool 項目(現 checkbox)の代替候補。どちらも統合コスト小(§3.2)。

エコシステム(iced_aw 以外)の結論は **「現時点で全て不要」**(§4)。iced_aw probe が実測した壁(fork の `Shell::local` Bus 化 E0308 + crates.io `0.15.0-dev` 不在)は iced_aw 固有ではなく **crates.io の iced 0.13/0.14 系に建つ外部 widget crate 全部に共通する壁**であり、しかも候補 4 crate が提供する機能(table/animation/drag&drop)は**この fork が既に標準で同梱している**(`widget/src/table.rs`・`core/src/animation.rs`+`widget/src/transition.rs`)。

---

## §1 在庫台帳(fork ソース mod 木由来)

出典: `widget/src/lib.rs:12-150`(mod 宣言と feature gate)+ `widget/src/helpers.rs`(公開コンストラクタ 53 関数)+ `widget/src/lazy/helpers.rs`。feature 列は fork ルート `Cargo.toml [features]` 実物。Motolii の点灯状態: `wgpu`/`advanced`/`image`/`canvas` を明示、`default-features = false` を**書いていない**ため default(`tiny-skia`/`crisp`/`web-colors`/`thread-pool`/`x11`/`wayland` 等)も点灯。`svg`/`qr_code`/`markdown`/`lazy`/`highlighter`/`debug`/`tester`/`selector`/`sysinfo` は**消灯**。

在庫総数の数え方: helpers のコンストラクタから container の整列 alias 8 本(`center_x`〜`bottom_right`)と小物 3 本(`value`/`void`/`iced` ロゴ)を畳み、**widget 52 部品**+ランタイム API 群(window 42 関数・operation・animation ほか、§1.2)。

### 1.1 widget 52 部品(mod 木順・グループ再構成)

| グループ | 部品 | feature |
|---|---|---|
| layout / 合成 | container(+center 等 alias 8), column, keyed_column, row, grid, stack, opaque, hover, pin, space, responsive, float, scrollable, rule, pane_grid, table(+table::column), themer | — |
| 遅延 | lazy, component | lazy(消灯) |
| interaction | button, mouse_area, sensor, tooltip | — |
| input | checkbox, radio, toggler, slider, vertical_slider, text_input, text_editor, pick_list, combo_box, progress_bar | — |
| text | text, rich_text(+span), markdown | markdown(消灯) |
| 描画/media | canvas, shader, image, svg, qr_code | canvas/image(点灯)、svg/qr_code(消灯) |
| animation | transition(`widget/src/transition.rs`、`core::animation::Animation` を widget 化した公式部品) | — |

### 1.2 ランタイム/コア API 在庫(widget 以外)

- **window**(`runtime/src/window.rs`、**pub fn 42 本**): 購読 6(`frames`/`events`/`open_events`/`close_events`/`resize_events`/`close_requests`)、開閉 4(`open`/`close`/`oldest`/`latest`)、形状 11(`drag`/`drag_resize`/`resize`/`set_resizable`/`set_min_size`/`set_max_size`/`set_resize_increments`/`size`/`maximize`/`minimize`/`toggle_maximize`)、位置 4(`position`/`move_to`/`monitor_size`/`scale_factor`)、モード/装飾 6(`mode`/`set_mode`/`toggle_decorations`/`set_level`/`show_system_menu`/`allow_automatic_tabbing`)、その他 11(`gain_focus`/`request_user_attention`/`set_icon`/`raw_id`/**`run`(&dyn Window → raw handle)**/**`screenshot`**/`enable_mouse_passthrough`/`disable_mouse_passthrough` 等)
- **application ビルダー**(`src/application.rs`): `.window(window::Settings)`(**min_size/max_size/icon/macOS titlebar 3点セットへの唯一の入口**)・`.antialiasing`・`.settings`・`.centered`・`.scale_factor`・`.presets` 等
- **daemon**(`src/daemon.rs`): multi-window ホスト
- **operation**(`core/src/widget/operation/`): focusable(`focus_next`/`focus_previous` 等6関数)・scrollable(`snap_to`/`scroll_to`)・text_input
- **animation**(`core/src/animation.rs`): `Animation<T>` 状態機械(easing/duration/`interpolate`)、`transition` widget と対
- **その他モジュール**(`src/lib.rs:516-651`): `time`, `task`, `clipboard`(OS 側 read/write), `font::load`, `system`(情報取得、sysinfo 消灯), `event::listen*`, `keyboard::on_key_press/release/listen`, `stream`, `exit`
- **開発器具 crate**(fork 同梱・feature 消灯): `devtools`(F12 comet/time_machine)・`tester`(e2e 記録再生、`.ice` 形式)・`selector`(`Selector` trait — `iced_test` の find の下部構造)・`beacon`(計測ストリーム)

## §2 使用中/未使用の照合台帳

再現コマンド(worktree root で):

```
# 部品ごと(例: pick_list)。コンストラクタ・型名・マクロの3形を見る
grep -rn --include="*.rs" -E "\bpick_list\(|PickList" next/
grep -rn --include="*.rs" -F "stack!" next/          # マクロ形(stack/column/row)
grep -rn --include="*.rs" -F "window::set_min_size" next/   # API 系は FQN で
```

判定規約: 日本語コメント内の一致(例: `opaque(αs=αb=1)`)・probes/ 配下のみの使用は「未使用(probe実証のみ)」へ落とした。マクロ形(`stack![...]`)と型名形(`Space::new()`)は関数 grep に出ないため全部品で3形を見た(§冒頭の census を1回、疑義行は個別に開いて確認)。

### 使用中(15 部品+API 群)

| 部品/API | 使用箇所(代表) |
|---|---|
| container / center | 全 pane(33/8 hit) |
| column / row(+マクロ) | 全 pane |
| stack(`stack!`) | `shell/src/lib.rs:3812` Stage overlay 重ね |
| space(`Space::new`) | `timeline-pane/src/rail.rs` ほか |
| scrollable | 2箇所 |
| button | 19箇所(header/menu/inspector) |
| mouse_area | `timeline-pane/src/rail.rs:293`・`inspector-pane/src/lib.rs:1444` |
| `.interaction()` | `inspector-pane/src/lib.rs:1477`(ResizingHorizontally — 前回調査 TOP1 は着手済み) |
| checkbox | 1箇所(settings) |
| slider | 1箇所 |
| text_input | 6箇所 |
| text | 107箇所 |
| canvas | 39 hit(timeline 本体・stage overlay) |
| shader | `shell/src/lib.rs` Stage presenter(裁定166/171) |
| pin / Stack(直 API) | probes/r4-widget-timeline のみ(製品コード未使用) |
| API: Task(19)・Subscription(12)・`window::frames`(transport tick)・`keyboard::Event` 生 listen(`lib.rs:3549-`)・`widget::Operation` walk(iced_test/q0_fence 85 hit)・`iced::application` ビルダー(title/subscription/theme のみ) | |

### 未使用(37 部品+API 群)

widget: keyed_column, grid, opaque, hover, pin(製品), responsive, float, rule, pane_grid, table, themer, lazy, component, sensor, tooltip, radio, toggler, vertical_slider, text_editor, pick_list, combo_box, progress_bar, rich_text/span, markdown, image(widget として — 裁定166 で意図的に撤去済み), svg, qr_code, transition。
API: `window` 42 関数中 41(frames 以外全部 — set_min_size/icon/screenshot/run/open 含む)、`.window(Settings)` ビルダー(main.rs:91-102 に無し=min_size 未設定の実測確定、前回調査 EVIDENCE_GAP 4 の解消)、daemon、`operation::focusable`(focus_next 等)、`core::animation::Animation`、`iced::clipboard`(app 内 clipboard は設計上 OS 非接続 — `clipboard.rs:1`)、`system`、`font::load`、`highlighter`、devtools/tester/selector(feature 消灯)。

## §3 未使用 37 部品の「つけ得」評価(1行ずつ)

発注書指定の `next/reference/intent-bundles.tsv` は**存在しない**(実在は `normal-map.tsv`/`lottie-coverage.tsv` — supervisor へ差し戻し事項)。効き先は既知課題(枠の文法=裁定179 / Settings モーダル / Tab フォーカス / blend 巡回置換 / ショートカット表示)+ `normal-map.tsv` の動詞群で書く。

### 3.1 つけ得(採る価値あり — 優先順)

| # | 部品 | 効く先 | コスト |
|---|---|---|---|
| 1 | **stack + opaque + center** | Settings モーダル overlay(既知課題)。§5 で実装口確認済み・oracle 可視 | 小 |
| 2 | **tooltip** | ショートカット表示(既知課題)。header ボタン・menu トリガーへ `.delay()` 付き1ラップ。`snap_within_viewport` で端も安全 | 小 |
| 3 | **`.window(window::Settings{min_size, icon})`** | 窓縮小での chrome 破綻防止+配布時のアプリアイコン。main.rs へ1 chain | 小 |
| 4 | **toggler** | Settings の bool 項目(現 checkbox 1箇所)。「即時反映のスイッチ」意味論は intent-first(裁定174)と整合 | 小 |
| 5 | **tester / debug feature**(fork 同梱) | 「落ちるテストで発注」規律の録画器具・F12 実測メトリクス。dev-only feature 点灯なので製品バイナリ非汚染(要 build 実測 — 本レーン外) | 小〜中 |
| 6 | **transition + core::animation** | 枠の文法(裁定179)の状態遷移(hover/選択の減衰)・panel 開閉のスライド。自前 tick より宣言的、`window::frames` 購読と併存可 | 中 |
| 7 | **pick_list** | blend 巡回置換(既知課題)。前回調査 §4.1 のとおり overlay 項目が oracle 不可視 — 採るなら検証は state 到達で。**対抗案**: §5 の stack モーダルを「blend 選択パネル」に流用すれば oracle 可視のまま一覧選択にできる | 中 |
| 8 | **operation::focusable(focus_next/previous)** | Tab フォーカス移動(既知課題)。Inspector の text_input 間は今すぐ効く(Focusable 実装は text_input のみ — 前回調査 §4.7) | 中 |
| 9 | **pane_grid** | timeline/inspector/stage の可変分割。**発注走行中のレーンあり**(セッション引き継ぎ 6b59661d「pane_grid走行中」)— 本調査からは重複発注禁止の注意のみ | (走行中) |
| 10 | **window::screenshot** | 検収スクリーンショットの窓実体取得(現 `screenshot.rs` は自前合成)。較正(Browser 比率台帳)で実窓と oracle 画の突合に使える | 中 |
| 11 | **sensor** | Browser カード grid の仮想化(見えたカードだけ thumbnail decode)。素材が増えるまで不要、増えたら第一候補 | 中 |
| 12 | **keyed_column** | 同上の並べ替え時 state 保持(rail の layer 並べ替えにも)。diff 事故が実測されるまで待ち | 小 |

### 3.2 不要(理由つき)

- **grid / responsive / float / hover / themer / rich_text / markdown / qr_code / svg / image(widget) / progress_bar / vertical_slider / text_editor / combo_box / radio / rule / table / lazy / component**: 順に — カード grid は自前 row 折返しで実装済み・レイアウトは dims 一元で responsive 不要・float(拡大浮遊)は文法に無い・hover(ホバー時のみ表示)は Q0「隠れていないから読める」原理に反する・theme は tokens 一本で局所差し替え不要・rich_text/markdown は表示対象が無い・QR 無縁・svg は resvg 経路が別にある・image widget は裁定166 で shader へ置換済み(復活は退行)・進捗バーは export 実装時に再訪・縦 slider の需要無し・複数行入力の需要無し・combo_box は検索付き pick_list で blend 13 値には過剰・rule は区切り線を token 色 container で描く現行で足りる・table は Inspector が「ラベル+コントロール」行形式で表形データが無い・lazy/component は view 再計算が実測ボトルネックになってから(component は upstream 非推奨方向)。
- **daemon / multi-window**: Stage 第2窓レーンの下調べ済み(前回調査 §4.6)。UX 要求が立つまで寝かせる。
- **window API 残り 30+**(drag/mode/level/passthrough 等): 通常のデスクトップアプリに不要な特殊窓操作。`show_system_menu` だけは自前 titlebar 化(macos.rs 3点セット)を採る日に再訪。
- **iced::clipboard**: app 内 clipboard は NeoUtl 同型の設計判断(`clipboard.rs:1`)で意図的非接続。将来「テキストを OS へコピー」動詞が map から立ったら1 Task で届く、それまで不要。
- **system / highlighter / selector(直接依存)**: 情報表示の場が無い・コード表示が無い・selector は iced_test 経由で間接使用済み。

## §4 エコシステム部品(iced_aw 以外)— 結論: 全て不要

前提の壁(実測済み): fork `73e686ee` は upstream の `Shell::local(&mut Bus<A>)` リファクタ後 API。crates.io の iced は 0.13/0.14 系まで(0.15.0-dev は git 限定)なので、**crates.io に建つ外部 widget crate は例外なく (a) 旧 iced スタックの二重取り込みか (b) `[patch.crates-io]` + Bus 非互換 E0308 のどちらかに落ちる**([iced_aw probe](2026-08-22-iced-aw-menu-probe.md) の一般化)。以下は実在を crates.io API で確認(2026-08-22 WebFetch)。個別 probe は未実施 — 下の判定は「fork 同梱機能との重複」が主因で、互換性は主因ではない(憶測の互換性断定はしない)。

| crate | 実在(ver / 更新) | vendoring 移植の価値 |
|---|---|---|
| iced_anim | 0.3.1 / 2026-01 | **不要** — fork が `core::animation::Animation` + `transition` widget を標準同梱(§1.1)。同機能の外部版を移植する理由が無い |
| iced_table | 0.14.0 / 2026-05 | **不要** — fork が `widget/src/table.rs`(`table()`+`table::column()`)を標準同梱。そもそも表形データが無い(§3.2) |
| iced_drop | 0.2.42 / 2026-08 | **今は不要** — 中身は「custom widget + operation で drag&drop」の小品。Browser→timeline のドロップ動詞が立つ日は、これを**移植せず同じ作り(mouse_area + operation)を自前 1 ファイルで書く**方が fork 追随負債が無い(wrapper-over-hack / 保守最低限)。設計参考としての READ だけ価値あり(MIT) |
| iced_term | 0.8.0 / 2026-03 | **不要** — ターミナル emulator。Motolii に端末面は無い |
| iced_audio(08-19 調査済) | — | **不要(当面)** — knob/xy_pad は将来のオーディオ UI で再訪。upstream 0.14 依存で同じ壁 |
| iced_video_player(08-19 調査済) | — | **移植不要・READ SET 維持** — 「YUV を wgpu へ直接上げる custom primitive」という作りは裁定166 で既に自前実装済みの方向と同じ。preview の GStreamer デコード検討時に再READ |

総括: **fork(upstream master 追随)が 0.13/0.14 時代にエコシステムが埋めていた穴(table/animation/操作 walk/テスト器具)をほぼ standard へ取り込み済み**で、外部 crate に頼る動機が構造的に薄くなっている。iced_aw を含め「エコシステムはもう大丈夫」が現時点の答え。再入場トリガーは toolkit-reentry と同型で「UX 要求が立ち、fork 同梱に該当機能が無い」時のみ。

## §5 Settings モーダル overlay の実装口(fork ソース確認済み)

**可能。しかも oracle 可視。** 根拠(全て checkout 実体の read):

- `stack`(`widget/src/stack.rs`): 子を同一 bounds に重ねる。`fn operate`(`:183`)を実装 — 全 layer の子が `widget::Operation` walk に乗る。イベントは**上の layer から**配られる(`update` が `children.rev()`)。
- `opaque`(`widget/src/helpers.rs:577-712`): マウス press を `shell.capture_event()` で吸って下層への貫通を止める + `operate` は内容へ**素通し**(`:640-649`)。つまり「モーダルの後ろを触れなくする」と「oracle から見える」が両立する。
- `center`(`helpers.rs:259`): Fill container で中央寄せ。背景 style を半透明 scrim にすれば背景暗転も同じ部品で済む。
- upstream 公式の同型実例が repo 内にある: `examples/modal/src/main.rs:189-216` の `fn modal(base, content, on_blur)` — `stack![base, opaque(mouse_area(center(opaque(content)).style(scrim)).on_press(on_blur))]`。同 example は付随作法も揃えて見せている: モーダルを開く update 腕で `operation::focus_next()` を返して初期フォーカスを入れる(`:47`)・Esc で閉じる+Tab/Shift-Tab を `focus_next/previous` へ配る event listen(`:71-91`)。§3.1 #8(Tab フォーカス)の最小実装例もこのファイルがそのまま手本になる。

Motolii への当てはめ(発注に使える精度の擬似コード — `shell/src/lib.rs::view` の `if self.settings_panel_open` 分岐を置換):

```rust
// lib.rs は既に `use iced::widget::{ ..., stack, ... }` 済み(:26)。opaque/center/mouse_area を追加 import。
let base: Element<_> = layout.push(/* 既存の Stage/Timeline/Transport 積み */).into();

if self.settings_panel_open {
    let card = container(settings_pane::view(...).map(Message::Settings))
        .width(Length::Fixed(dims.settings_modal_width))   // 内容幅で止める(Q0)
        .style(chrome::panel_style)                        // 既存 chrome トーン
        .padding(dims.spacing_m);

    stack![
        base,
        opaque(                                            // scrim: 下層を触れなくする
            mouse_area(
                center(opaque(card))                       // card 自身も opaque(card クリックで閉じない)
                    .style(|_| container::Style {          // 半透明 scrim(token 色 + α)
                        background: Some(colors.scrim.into()), ..Default::default()
                    })
            )
            .on_press(Message::Settings(settings_pane::Message::ToggleSettingsPanel)) // 外側クリックで閉じる
        )
    ]
    .into()
} else {
    base
}
```

検収面の含意: card 内の button/checkbox/text_input は `operate` 素通しにより既存 `Simulator::find` 手口がそのまま効く(menu.rs が column 方式へ迂回した穴は**この経路には無い**)。Esc で閉じる配線は既存の生 keyboard listen(`lib.rs:3549-`)へ1腕追加。scrim トークン(`colors.scrim`)は tokens 側に未定義なら新設が要る — そこだけが本体差分以外の作業。

## EVIDENCE_GAP

1. tester/debug feature 点灯時のビルド時間・依存増は未実測(read-only レーン)。採用検討時に `cargo check` 実測から。
2. iced_drop の「operation で drop 対象を探す」内部構造は crates.io メタのみ確認、ソース未読。自前 drag&drop 設計時に READ。
3. §5 は擬似コードまで(cargo check 未通し)。`container::Style` の scrim 当て方・`mouse_area` と `opaque` の重ね順は `examples/modal` に忠実だが、fork rev での example 自体のビルド確認はしていない。
4. 発注書指定の `next/reference/intent-bundles.tsv` が不存在(実在: `normal-map.tsv`/`lottie-coverage.tsv`)。束 id 参照は supervisor 側で読み替えが要る。

## 参照(一次資料パス)

- fork checkout: `/Users/member_ottoto/.asdf/installs/rust/stable/git/checkouts/iced-1bbb4ed9d90ae4f8/73e686e/` — `widget/src/lib.rs`(mod 木)・`widget/src/helpers.rs`(コンストラクタ)・`widget/src/{stack,table,transition,sensor,pin,float,tooltip}.rs`・`core/src/animation.rs`・`core/src/widget/operation/`・`runtime/src/window.rs`・`src/application.rs`・`src/lib.rs`・ルート `Cargo.toml [features]`・`tester/src/lib.rs`・`selector/src/lib.rs`・`examples/modal/src/main.rs`
- Motolii 側: `next/Cargo.toml:78`・`next/shell/motolii-shell/src/{lib,main,menu,clipboard,transport}.rs`・`next/ui/motolii-{inspector,timeline}-pane/src/`
- crates.io API(実在確認のみ): iced_anim 0.3.1 / iced_drop 0.2.42 / iced_table 0.14.0 / iced_term 0.8.0(2026-08-22 取得)
