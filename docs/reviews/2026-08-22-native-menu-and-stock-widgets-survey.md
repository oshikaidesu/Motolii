# ネイティブ/クロスプラットフォームメニューバー + iced標準widget「つけ得」在庫 調査

日付: 2026-08-22 / 発注: 利用者提起(朝)「ヘッダを自前で作ってるのは変。macはOSの、winは自前だが整っているのがある。クロス対応の道具があるのでは」+「標準機能でつけ得なのは?」/ レーン: read-only 調査(コード変更禁止)
先行資料: [メニューバー基盤調査(2026-08-22)](2026-08-22-menubar-foundation-survey.md)(§1 map由来のメニュー構造案・§4 S6併存表・MB-0〜MB-3切片割りは**そのまま正**、本調査はその§2/§3を実機ソース照合で更新するもの)/ [iced fork seam台帳](2026-08-18-iced-fork-seam-ledger.md) / 実装済み: `next/shell/motolii-shell/src/menu.rs`(MB-0+MB-1)

対象 pin: `iced = { git = "https://github.com/oshikaidesu/iced", rev = "73e686ee05efd7d1b61cfea2647186b336d9ab9c" }`(fork branch `motolii/host-seams`、`next/Cargo.toml:78`)。実機チェックアウト `/Users/member_ottoto/.asdf/installs/rust/stable/git/checkouts/iced-1bbb4ed9d90ae4f8/73e686e/`(CARGO_HOMEはasdf配下 — `~/.cargo` ではない、次調査者向け注記)。全ての「実在」判定はこのチェックアウトの grep/read で一次確認した(docs.rs最新版は見ていない)。

---

## 要約(RETURN)

**A. メニューバー — 前回調査の結論を1点だけ訂正・全体は補強**

- 前回調査(§2)は「現行 pin にネイティブ統合は無い→アプリ内メニューバー推奨」と結論したが、**この結論自体は変わらない**。ただし「fork に手が要るなら迂回よりwrapper」という本調査の前提は**外れた** — `iced::window::run(id, |w: &dyn Window| ...)`(`runtime/src/window.rs:432`、stock upstream、host-seams 2 seam台帳に含まれない=無改変)が `raw_window_handle::{HasWindowHandle, HasDisplayHandle}` を実装する `&dyn Window` を返す。これは **fork へ1行も足さずに** macOS の NSView ポインタ・Windows の HWND を取り出せる経路であり、muda 統合はゼロ改変で届く。ネイティブ経路の再検討材料になる。
- **推奨は変えない: v1はアプリ内メニューバー継続**。理由は前回§2.3のとおり(トンマナ柵・保守最低限)に加え、実測で判明した新事実: macOSは `Menu::init_for_nsapp()` で窓ハンドル不要(NSApplication単位)なので技術的難度は低いが、**Linux(muda)はGTKの生の依存とイベントループ相乗り(`gtk::main_iteration()` の外部ポンプ)を要求する**(§2.4実測)。3 OS 一貫の「1実装」を崩さずに済むアプリ内路線の優位性が実測でむしろ強まった。
- **muda はゼロ改変で使える**(§2.2)。ただし呼び出し側(`motolii-shell`)に新規外部依存(`muda` + macOSでは`objc2`系の間接依存)を1本足す判断が要る。iced_aw の `MenuBar`/`Menu` widget と違い、**muda はOSごとのウィジェット実装をラップするだけで iced の型に触れない**ため、Cargo依存衝突(fork互換性リスク)がそもそも発生しない(§2.5、前回§3.2 EVIDENCE_GAP 2 の別解)。
- **iced_aw の fork互換性リスクは前回より深刻と判明**(§2.5): `[patch.crates-io]` は iced_aw 自身の Cargo.toml にしか効かない(Cargo は依存先の`[patch]`を無視する)。Motolii側で改めて `next/Cargo.toml` に `[patch.crates-io]` を書く必要があり、しかも Motolii の iced 依存は crates.io 経由ではなく git 直参照なので**同じ`[patch]`機構では自分自身の依存を差し替えられない**(git依存の patch は `[patch."<git URL>"]` が要る、URLも `iced-rs/iced.git` と `oshikaidesu/iced` で別)。理論上は解決可能だが cargo build 検証なしに軽く見積もれる話ではない。
- MB-2(Layer/Viewメニュー)は**現行の in-window 発注のまま進めてよい**(§3)。muda 移行が採択されても、`menu.rs` の `Item{label, shortcut, message}` 構造(意味を運ぶ最小単位)はそのまま `muda::MenuItemBuilder` へ1:1写せる形になっている — 二重設計にならない。

**B. iced標準widget「つけ得」TOP5(統合コスト昇順)**

| 順位 | 項目 | 効く先 | 統合コスト |
|---|---|---|---|
| 1 | `mouse_area().interaction(mouse::Interaction)` | Inspectorのdrag可能値・timeline-paneのresize境界でOS標準カーソル(resize/grab等)を今すぐ出せる。実装済みinteraction箇所ゼロ(grep実測、§4.4) | 小 |
| 2 | `window::Settings.min_size`/`icon` | 現状 next/ の window 起動設定は既定値のまま(要確認だが `min_size: None` 相当)。ウィンドウ縮小時のUI破綻を1行で防止 | 小 |
| 3 | `pick_list`(combo_box ではなくこちら) | Inspectorのblend mode 13値巡回ボタンの置換候補(§4.1)。ただし**`operate()` 欠落によりq0_fence(`iced_test::Simulator::find`)から項目が見えない**(自前menu.rsが踏んだ穴と同一原因、実機確認済み) — 採用するなら oracle 側を別経路(値の到達確認のみ)にする設計判断が要る | 中 |
| 4 | `widget::operation::focus_next()`/`focus_previous()` | Tab移動。ただし**`Focusable` を実装する標準widgetは`text_input`のみ**(実測、button/pick_list等は非対応) — 呼ぶだけでは効かない、対象widgetに`Focusable`実装を足す作業が伴う | 中(Tab配線)〜大(button等をfocusable化) |
| 5 | `iced::daemon` + `window::open`(multi-window) | Stage島の第2窓分離レーンの下調べ。`wgpu::window::Compositor`は**1個のdeviceで複数windowのSurfaceを作る**構造(`create_surface`はwindow引数を取るだけでdevice非依存)なので、Stage(re_renderer共有device)を第2窓のSurfaceへ繋ぐこと自体は構造的に無理筋ではない(§4.6、cargo build未検証) | 大(未検証・実装は別レーン) |

`tooltip`/`text_editor` は実在確認のみ(§4.2/§4.3、Motolii側の直近ニーズが薄いため順位外)。

---

## §1 A-1: 現状把握

`next/shell/motolii-shell/src/menu.rs`(231行)が MB-0(基盤)+MB-1(File束)を実装済み。要旨:

- トップレベルは `File`/`Edit` の2つ。トリガーは通常の `button`(`chrome::button_style`)、開いている間だけ `Shell::view` の縦積みへ `column` を push する **Q0 型の表示分岐**(絶対配置overlayではない)。
- 実装冒頭のdoc(1〜33行目)に**設計変更の記録**がある: 当初案(前回調査§3.3案A = `overlay::menu::Menu`のwraps)を試みたが、`overlay::menu::Overlay`(pick_list/combo_boxが共有するprimitive)が `iced_core::overlay::Overlay::operate()` を**オーバーライドしない**(`core/src/overlay.rs`の既定空実装のまま)ことが実装時に判明し、これは `widget::Operation` 走査(oracle が唯一使う発見経路)がドロップダウン項目に届かないことを意味する。この事実は本調査でも実機ソースで再確認した(§4.1で詳述、pick_list/combo_boxの「つけ得」評価に直結する)。
- 代替として「開いている間だけ普通の`column`(中身は通常の`button`)を木へpushする」形を採用。`button`は`operate()`で`Container` candidateを自己登録するので、既存の`Simulator`手口(find→bounds→click)でそのまま検分できる。
- 全10項目(Edit 10 + File 4)は既にheaderボタンまたはCmd+キーを持つ動詞(S6併存、前回調査§4のとおり)。メニューは「唯一の入口ではなく第3の入口」という設計保証を保ったまま実配線されている。

fork pin: `next/Cargo.toml:78`。`motolii/host-seams` ブランチは [iced fork seam台帳](2026-08-18-iced-fork-seam-ledger.md) が言うとおり**2 seamのみ**(web-sys完全一致解除・wgpu bind-group床)— メニュー/ウィンドウハンドル関連の改変は一切無い。つまり本調査が§2で見つける経路は全て**upstream iced 0.15.0-devの素の機能**であり、Motolii固有の改変ではない。

---

## §2 A-2/A-3: muda × iced 統合経路の実測

### 2.1 前回調査の再確認: winit shellはwindow handleを外へ渡さない

`iced_winit::window::Window<P,C>`(`winit/src/window.rs:172`)は `pub raw: Arc<winit::window::Window>` フィールドを持つが、この構造体自体が `iced_winit` crateの内部実装(`iced::application`/`iced::daemon`のランタイムが所有)であり、アプリケーションコード(`motolii-shell`)からは到達できない。ここまでは前回調査どおり。

### 2.2 発見: `iced::window::run` が正規の脱出口

`runtime/src/window.rs:432`:

```rust
/// Runs the given callback with a reference to the [`Window`] with the given [`Id`].
pub fn run<T>(id: Id, f: impl FnOnce(&dyn Window) -> T + Send + 'static) -> Task<T>
```

この `Window` トレイト(`core/src/window.rs:37`)は:

```rust
pub trait Window: HasWindowHandle + HasDisplayHandle + Debug {}
```

`raw_window_handle::{HasWindowHandle, HasDisplayHandle}` は業界標準のwindow handle抽象(`core/src/window.rs`は `pub use raw_window_handle;` で再エクスポート済み)。つまり `iced::window::run(id, |w| w.window_handle())` が `Task<Result<raw_window_handle::WindowHandle, HandleError>>` を返し、そこから:

- macOS: `RawWindowHandle::AppKit(handle)` → `handle.ns_view`(`NonNull<c_void>`)
- Windows: `RawWindowHandle::Win32(handle)` → `handle.hwnd`(`NonZeroIsize`)

が取れる。実装先はこの `Task` を `Message` へ変換して `update()` の中で受け取る通常のiced作法(shell.rs の他の非同期処理と同型)。**fork改変ゼロ**でここまで届く — 前回調査の EVIDENCE_GAP 2「winit統合が生のwindow handleをどこまで外へ渡しているか未確認」は本調査で解消し、答えは「`iced::window::run`経由で渡る」。

### 2.3 muda 側API実測(WebFetch、docs.rs/GitHub一次情報)

- macOS: `menu.init_for_nsapp()` — **窓ハンドル不要**。`NSApplication.shared()`単位でメインメニューバーを差し替えるAPIで、メインスレッド呼び出しの制約のみ。§2.2のwindow handle経路すら要らない(NSApp自体はwinitが既に初期化済みなので、muda呼び出しのタイミングだけ気にすればよい)。
- Windows: `unsafe { menu.init_for_hwnd(hwnd) }` — §2.2で取れたHWNDをそのまま渡せる。**ただしaccelerator(ショートカットキー)を効かせるには、Win32メッセージループで`TranslateAcceleratorW`を呼ぶ必要がある** — iced側のwinitイベントループはアプリコードに生メッセージループを渡さないため、これは**現行stackでは満たせない**(accelerator無しのmuda Windowsメニューなら成立するが、Cmd相当のショートカット併記が意味を失う。次点: メニュー項目のacceleratorテキストは表示のみにして実キーバインドはMotolii既存の`resolve_navigation_key`に任せる二重管理を許容するか、要判断)。
- Linux: `menu.init_for_gtk_window(&gtk_window, ...)` — **GTKウィンドウそのものを要求**。iced/winitは素のX11/Wayland surfaceで、GTKウィンドウではない。libgtk-3を新規依存として引き込み、GTKの`main_iteration()`をwinitのイベントループへ外部ポンプする統合コードが要る(muda公式ドキュメントに明記)。3 OS中もっとも重い。
- イベント受信: グローバル `MenuEvent::receiver()`(crossbeamチャンネル)への `try_recv()`ポーリング、または `MenuEvent::set_event_handler(...)` でコールバック登録。**winit の `EventLoopProxy` を要求しない**経路がある(pollingか、あるいはiced側の `iced::stream::channel`(`futures/src/stream.rs:11`、stock API・fork改変なし確認済み)で別スレッドから `MenuEvent::receiver().recv()` をブロッキング待受してiced `Subscription` へ橋渡しする形が最小改変)。**iced_winitの内部`Proxy`(`winit/src/proxy.rs`)へ触れる必要が無い** — 前回調査が懸念した「event loop proxyへの口をどう開けるか」は、実は`Subscription`機構がその口そのものであり、fork seamは不要という結論になる。

### 2.4 muda以外の選択肢の比較

| 選択肢 | 中身 | iced (motolii/host-seams) との相性 |
|---|---|---|
| **muda**(tauri-apps) | winit/tao両対応、OS別ネイティブAPIのラッパー。3 OS共通API | §2.2/2.3のとおり到達可能。GTK依存はLinuxのみの重さ |
| `objc2-app-kit` 直叩き(macOSのみ) | NSMenu/NSMenuItemを自分で組む低レベルbinding | mudaが既にこの上に建っている抽象なので、直叩きは「移植」ではなく「スクラッチ」寄り(保守最低限原則に反する)。macOS単独でよいなら選択肢だが3 OS共通性を捨てる |
| `tao`前提の menu 実装(旧muda 0.11以前や一部crate) | tao(winitフォーク、Tauri専用)のwindow型に結合 | **不成立**。Motoliiのwindow shellはiced内蔵winitフォーク(`oshikaidesu/iced`経由)であり`tao`ではない。tao依存のmenu crateはwindow handleの受け渡し口が合わない |
| iced_aw `MenuBar`/`Menu` widget | iced widget木の中に描くアプリ内メニュー(ネスト対応) | **ネイティブ経路の代替にはならない**(そもそもアプリ内描画)。§2.5でfork互換性リスクを深掘り — MB-2以降でネスト(Blend Mode 12値等)が要る段の検討対象として残す |
| egui系のmenuウィジェット(egui::menu) | — | Motoliiは iced ホストへ移行済み(2026-08-18裁定)でegui新規投資は凍結中。対象外 |

### 2.5 iced_awのfork互換性リスク再検証(前回§3.2 EVIDENCE_GAP 2への回答)

`iced_aw`(GitHub `iced-rs/iced_aw`、現在は iced-rs org 配下)の `Cargo.toml` を一次確認した:

```toml
[dependencies]
iced_core = { version = "0.15.0-dev" }
iced_widget = { version = "0.15.0-dev" }
# ...

[patch.crates-io]
iced = { git = "https://github.com/iced-rs/iced.git", branch = "master" }
iced_core = { git = "https://github.com/iced-rs/iced.git", branch = "master" }
iced_runtime = { git = "https://github.com/iced-rs/iced.git", branch = "master" }
iced_widget = { git = "https://github.com/iced-rs/iced.git", branch = "master" }
iced_test = { git = "https://github.com/iced-rs/iced.git", branch = "master" }
```

**Cargoは依存クレート自身の`[patch]`セクションを無視する**(patchはワークスペースのルートmanifestでのみ有効)。つまりMotolii側が `iced_aw` を依存に足しても、この`[patch.crates-io]`は一切効かない。`iced_aw`の`[dependencies]`は`iced_core`/`iced_widget`をcrates.io版`0.15.0-dev`として要求するが、**crates.ioに`0.15.0-dev`は存在しない**(devプレリリースはgit経由でしか手に入らない設計)ため、Motolii側の`next/Cargo.toml`で改めて`[patch.crates-io]`を書いて`oshikaidesu/iced`フォークへリダイレクトする必要がある。

ここでもう1段ややこしいのは、**Motolii自身の`iced`依存は`[workspace.dependencies]`のgit直参照**(`{ git = "https://github.com/oshikaidesu/iced", rev = "..." }`)であり、`crates.io`ソースではないこと。`[patch.crates-io]`はcrates.ioソースの依存を差し替える機構なので、Motolii自身の直接依存(git直参照)には効かず、`iced_aw`が要求するcrates.io側`iced_core`/`iced_widget`だけを`[patch.crates-io]`でフォークへ向ける形になる。理論上両立可能(Motolii本体はgit直参照のまま、iced_awが要求するcrates.io参照だけpatchで同じフォークrevへ収束させる)だが、**同じ型(`iced_core::Element`等)が2つのソース記述(直参照 vs patch経由)から解決されて実際に同一crateとして畳み込まれるかはcargoの実解決を見るまで確定しない**。前回調査は「`[patch]`未設定、互換性未検証」とだけ書いていたが、実際には**未設定なのではなく、書く先が2段(iced_aw自身のpatchは無効・Motolii側で別途書く必要)ある**というのが本調査の訂正。EVIDENCE_GAPとして維持(§5-2)。

---

## §3 A-4/A-5: 推奨形とMB-2への影響

**推奨: v1(MB-0〜MB-2)はアプリ内メニューバー継続。ネイティブ(muda)化はMB-3以降の独立切片として検討材料に残す。**

理由の更新:
1. 技術的には§2.2の発見で「fork改変が要る」という重さが消えたが、**Linux(muda+GTK)の重さは逆に実測で増した** — 3 OS一貫の「1実装」というアプリ内メニューの強みは変わらず有効。
2. Windowsのaccelerator配線(`TranslateAcceleratorW`)はiced側イベントループへの介入を要し、これは§2.2の`window::run`より一段重い統合(生のWin32メッセージループへのフック)。ここは前回調査になかった新しい技術負債の発見。
3. **意味の一元化は既に達成されている**: `menu.rs`の`Item{label, shortcut, message}`は「動詞名+ショートカット文字列+`Message`」という最小構造で、これは`muda::MenuItemBuilder::new(label).accelerator(...)`へ1:1で写せる形をしている。**muda化を選ぶ日が来ても`menu.rs`のitem定義層は書き直しにならない**(トリガー/dropdownのレンダリング層だけが差し替え対象)。MB-2をこの構造のまま発注してよい理由はここにある。
4. macOSだけ先にネイティブ化する(win/linuxはアプリ内のまま)という前回§2.3案4の**部分ネイティブ化**は、§2.2の発見により技術的難度が下がった(`init_for_nsapp()`は窓ハンドル不要・GTK並みの重さが無い)。ただし「プラットフォームごとにメニューの見た目/挙動が変わる」という一貫性コストは変わらず残るため、**採否は利用者裁定が要る**(本レーンは推奨のみ、決定はしない)。

MB-2(Layer/Viewメニュー)への影響: **いま in-window で発注してよい**。ネスト深度(Blend Mode 12値の2段メニュー等)が実際に要る段になったら、`overlay::menu::Menu`をitem側に入れ子で使うか(前回§3.3案Aの範囲内)、iced_awへ切り替えるか(§2.5のfork互換性検証を先に払う)の二択になるが、どちらも`Item`構造そのものは変えない。

---

## §4 B: iced標準widget「つけ得」棚卸し(pin rev `73e686ee`、実機ソース確認)

### 4.1 pick_list / combo_box(Blend Mode 13値巡回の置換先)

- **実在**: `widget/src/pick_list.rs`(920行超)、`widget/src/combo_box.rs`。両方とも`iced::widget::pick_list`/`combo_box`として`pub use`済み(`src/lib.rs:632`の`pub use iced_widget::*;`経由)。
- **API形**: `pick_list(options, selected, on_select)`ビルダー関数。`.on_select(Fn(T)->Message)`・`.style`・`.menu_style`等のchain。`combo_box`は`State<T>`を外部で保持するステートフル版(検索可能ドロップダウン、blend modeのような固定列挙には過剰)。
- **Motoliiのどこに効くか**: Inspectorのblend mode選択(現状は巡回ボタン、前回メニューバー調査§1で「Inspector巡回ボタンで既に入口あり」と記載)。`pick_list`なら1操作で任意の値へ直接ジャンプできる(巡回13回が要らなくなる)。
- **統合コスト: 中、ただし oracle 側に構造的な穴がある**。実機確認: `widget/src/overlay/menu.rs`の`Overlay`実装は`fn operate`を持たず(`core/src/overlay.rs:41`の既定空実装のまま — grep実測、`impl Overlay for ... { fn operate }`が`overlay/menu.rs`に存在しないことを確認)。Motoliiのoracle(`iced_test::Simulator::find`/`collect_targets`)は`widget::Operation`走査だけが唯一の発見経路(`tests/suite/target_walk.rs`のdoc、`menu.rs`冒頭コメントで既に指摘済みの事実を本調査でも独立に再確認)。つまり**`pick_list`を開いた状態のドロップダウン項目はテストのfind/click手口から見えない** — MB-0が自前column方式へ迂回した理由と同一の穴で、pick_list採用時も同じ壁に当たる。採用するなら「値が変わったこと」を別経路(state読み取り)で検証するoracle設計が要る。

### 4.2 tooltip

- **実在**: `widget/src/tooltip.rs`(372行超)。
- **API形**: `tooltip(content, tooltip_text, Position)`。`.gap()`・`.delay(Duration)`(ホバー遅延、AE/Resolve等の慣習に合う)・`.snap_within_viewport(bool)`。`Position`列挙(`FollowCursor`/`Top`/`Bottom`等)。
- **Motoliiのどこに効くか**: 現状Motoliiにホバーツールチップの入口は無い(grep未実施だが`tooltip`という名のヘルパー呼び出しがmenu.rs/chrome.rs等に見当たらない)。S6台帳のκ調査で挙がった「初回発見用の可視化」不足箇所(アイコンのみのボタン等)へ充てられる。
- **統合コスト: 小**。1関数呼びでラップするだけ、新規state不要。

### 4.3 text_editor

- **実在**: `widget/src/text_editor.rs`。
- **API形**: `text_editor(&Content)`、複数行、`Content::perform(Action)`(カーソル移動・選択・IME等をカプセル化)。`text_input`より重いが複数行対応。
- **Motoliiのどこに効くか**: 現状next/に複数行テキスト入力の需要は確認できなかった(Inspector値欄は単行数値、name欄も単行想定)。優先度は低い。
- **統合コスト**: 未評価(直近ニーズが無いため深掘りしていない)。

### 4.4 mouse_area とカーソル形状(Interaction)

- **実在**: `widget/src/mouse_area.rs`。`.interaction(mouse::Interaction)`(`widget/src/mouse_area.rs:116`)でホバー中のカーソル形状を指定できる。`mouse::Interaction`は`iced::mouse::Interaction`として公開済み(`src/lib.rs:598`)。
- **API形**: `mouse_area(content).interaction(Interaction::ResizingHorizontally)`等。on_press/on_release/on_double_click/on_right_press/on_right_click系(6種)+on_scroll/on_enter/on_move/on_exitも同widgetが持つ(MB-3右クリック基盤の一般化候補としても使える — 前回調査のMB-3切片が「MB-0のoverlay機構を右クリックトリガーへ転用」と書いていたが、実際には`mouse_area`の`on_right_press`が既に標準装備なので**MB-3は新規overlay機構を要らない可能性がある**、独立EVIDENCE_GAPとして§5に記載)。
- **Motoliiのどこに効くか**: timeline-paneのlane境界(resize可能なら`ResizingVertically`)、Inspector数値欄のdrag-scrub(`Grab`/`Grabbing`)。grep実測で`mouse::Interaction`を明示指定している箇所はnext/内に見当たらず(既定の矢印カーソルのまま)、「触れそうで触れない」(ux-authority-order.mdのQ0)状態のまま放置されている操作がある可能性が高い。
- **統合コスト: 小**。既存`mouse_area`使用箇所に`.interaction(...)`を1行足すだけ、新規依存・新規state不要。

### 4.5 window系(最小サイズ・タイトル・アイコン)

- **実在**: `core/src/window/settings.rs`。`Settings{ size, min_size: Option<Size>, max_size: Option<Size>, icon: Option<Icon>, ... }`。
- **タイトルは imperative API ではなく reactive**: `Program`トレイトの`title(&self, state, window_id) -> String`(`src/application.rs:566`)。状態の関数として毎フレーム再評価される形で、`window::set_title`のようなActionは存在しない(意図的な設計 — stateが正でtitleは投影)。
- **macOS固有**: `core/src/window/settings/macos.rs`に`title_hidden`/`titlebar_transparent`/`fullsize_content_view`の3フィールド(`PlatformSpecific`)。カスタムタイトルバー(Electron風)を作る場合の足場が既にある。
- **Motoliiのどこに効くか**: 現状ウィンドウ最小サイズ制約の有無は未確認(要`Settings`呼び出し箇所のgrep、本調査は範囲外としたが次調査で1点確認すべき)。ウィンドウを極端に縮めるとchrome/Inspectorが破綻する既知パターン(`background_rect`のdepth_offset極端値バグと同系統の「極端値を想定しない」設計)への防御線になりうる。
- **統合コスト: 小**(min_size/icon)。

### 4.6 multi-window(daemon API)

- **実在**: `iced::daemon(boot, update, view)`(`src/daemon.rs`)。「窓を1枚も開かずに起動し、`window::open`が返す`Task`で窓を増やす」モデル。`iced::application`(単窓)と地続き — `daemon`は単に窓の存在を強制しないだけの上位モデル。
- **Stage島wgpu presenterの第2窓成立性(論点整理のみ、実装なし)**: `wgpu/src/window/compositor.rs`の`Compositor`は**1個のdevice/queue/adapterインスタンスが複数windowの`Surface`を管理する**構造(`create_surface(&self, window, ...)`はwindow引数を取るだけでcompositor自体は使い回し、`fn present`もsurface単位)。つまりdaemon化して窓を2枚にしても**wgpu deviceは1個のまま**であり、[iced fork seam台帳](2026-08-18-iced-fork-seam-ledger.md)の「bind-group床はprocess全体の1つ」という設計(§3、`request_min_max_bind_groups`)とも整合する — Stageのre_renderer統合が要求する拡張済みdeviceは、daemon化後の2枚目の窓でも**同じdeviceを共有する**ため、2つ目のwgpuコンテキストを別途構築する必要が構造的に無い。**ただしこれはソースの型シグネチャから導いた推論であり、cargo build/実行での検証はしていない**(本レーンはread-only)。分離レーンの下調べとしては「壁は無さそうだが未実証」という評価が正確な言い方になる。
- **統合コスト: 大**(未検証)。実装は別レーンの範囲。

### 4.7 focus移動(tab traversal)

- **実在**: `core/src/widget/operation/focusable.rs`に`focus(id)`/`unfocus()`/`focus_next()`/`focus_previous()`/`find_focused()`/`is_focused(id)`の6関数(全て`impl Operation<T>`を返す、`Task`化して呼ぶ形)。
- **穴**: `Focusable`トレイト(この操作群が辿る対象マーカー)を実装している標準widgetは**`text_input`のみ**(grep実測、`widget/src/*.rs`全体で`impl Focusable`は1箇所)。`button`/`pick_list`/`checkbox`等は非対応。
- **Motoliiのどこに効くか**: Inspector数値欄間のTab移動(text_inputのみなので今すぐ効く可能性がある)。File/Editメニューのキーボード操作(button主体なので今は効かない)。
- **統合コスト**: text_input間の移動だけなら**中**(Tabキー捕捉→`focus_next()`をTaskとして発行、の配線のみ)。button等もfocus対象にするなら**大**(`Focusable`実装を各widgetへ足す独自work、事実上のスクラッチ寄り)。

---

## §5 EVIDENCE_GAP(次調査/発注前に埋めるべき点)

1. **iced_awの`[patch]`二段構造(§2.5)は理論の域を出ない**。実際に`next/Cargo.toml`へ`[patch.crates-io]`を足してcargo metadataが通るかはcargo buildを要し、本レーン(read-only)では検証不能。次にiced_aw採用を検討する回はここから始める。
2. **Windowsのaccelerator配線(§2.3)は未解決のまま残った新発見**。`TranslateAcceleratorW`をiced側イベントループへどう割り込ませるか(生のWin32メッセージフックが要るのか、無くても実用上困らないのか)は本調査の範囲外 — muda採用を具体的に検討する段で個別調査が要る。
3. **mouse_areaの`on_right_press`がMB-3右クリック基盤の要求を満たすか(§4.4)は未検証**。前回メニューバー調査はMB-3を「新規overlay機構が要る高負担切片」と見積もっていたが、標準widgetが既にright-clickイベントを持つなら見積もりが下がる可能性がある。MB-3着手前に確認すべき。
4. **window::Settingsのmin_size/iconが現状next/で使われているかの確認漏れ**(§4.5)。本調査は「実在する」ことのみ確認し、「Motoliiが使っていない」ことのgrep網羅はしていない。
5. **macOSネイティブメニューバーの部分採用(macのみmuda、win/linuxはアプリ内)の要否は利用者裁定が要る**(§3の4点目)。技術的な障壁は下がったが、意匠一貫性とのトレードオフは設計判断であり本レーンの権限外。

---

## 参照

- [メニューバー基盤調査(2026-08-22)](2026-08-22-menubar-foundation-survey.md) — §1構造案・§4 S6併存表は正、本調査はその§2/§3のみ更新
- [iced fork seam台帳](2026-08-18-iced-fork-seam-ledger.md) — host-seams = 2 seamのみ、メニュー/window handle関連の改変ゼロの根拠
- `next/shell/motolii-shell/src/menu.rs` — MB-0/MB-1実装、`Item`構造の実物
- `next/Cargo.toml:71-93` — iced fork pin comment、rfd(裁定176)との並び
- fork実機チェックアウト: `/Users/member_ottoto/.asdf/installs/rust/stable/git/checkouts/iced-1bbb4ed9d90ae4f8/73e686e/`(`runtime/src/window.rs`・`core/src/window.rs`・`core/src/window/settings*.rs`・`core/src/widget/operation/focusable.rs`・`widget/src/{pick_list,combo_box,tooltip,text_editor,mouse_area}.rs`・`wgpu/src/window/compositor.rs`・`futures/src/stream.rs`を読んだ)
- muda: docs.rs `muda`(latest)・GitHub `tauri-apps/muda` README(WebFetch一次確認、2026-08-22)
- iced_aw: GitHub `iced-rs/iced_aw` Cargo.toml(WebFetch一次確認、2026-08-22)
