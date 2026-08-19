# iced Browser 視覚再現 — round 3(2026-08-19)証拠

## 前提

利用者指示: 「css 抽出器具が実用にできるならブラウザとインスペクターパネルの
改定をお願いします」。前夜〜当日朝に **Blitz を使った CSS 計算値の抽出器具
(`motolii-css-metrics`)が完成し main に着地**した
(`docs/reviews/2026-08-19-css-computed-metrics-extraction.md`)。以後、寸法・色は
人が css を読んで写すのではなく、器具が吐いた値を根拠にする。

作業途中で正本の方針が訂正された: **このパネルの egui 実装
(`motolii_ui::browser_panel`)は「html から egui への変換がうまくいかなかった
部分」であり、構造・階層・視覚の手本にしない**。見た目の正本は
`docs/mocks-ui/public/browser-library.html` + `browser-library.css`
そのもの。egui を参照してよいのは**振る舞い**(検索・kind filter の意味関数)
だけで、見た目・構造・階層は html から読んだ(詳細は
`crates/motolii-shell-iced/src/browser_pane.rs` の module doc)。

## 並び絵

| 面 | ファイル |
|---|---|
| 基準(mocks-ui) | `mock-browser-reference.png`(利用者支給) |
| iced 実物(`--screenshot` 実測、座席あり・starter media 4本) | `iced-browser-seated.png`(1960×1300 = 980×650 論理 ×2) |
| 同、Browser 部分を切り出し(0,0)-(632,650) | `iced-browser-crop.png` |

撮り方:
```
cargo run -p motolii-shell-iced -j 5 -- \
  --project <fresh project.json> --screenshot out.png 120
```
project は `ScriptedPrompts{ new_project_path: Some(path) }` +
`Message::NewProjectPressed` で作った素の新規 project(starter media 4本は
`BrowserPane::default_shell()` の既定 registered folder から出るので、
project 自体は空のままで良い)。

## ファイル分離

`view.rs` の `browser_panel()`(旧実装、ヘッダ〜selection tray〜OS drop zone〜
card 描画一式、約450行)を `crates/motolii-shell-iced/src/browser_pane.rs`
(新規、`inspector_pane.rs` と対称)へ丸ごと切り出した。`view.rs` 側には
`crate::browser_pane::browser_pane(shell)` の呼び出し1行だけが残る。
Browser の名乗り定数(`BROWSER_RAIL_ALL` 等)も `browser_pane` 側の
`RAIL_ALL` 等へ移動 — `view::` prefix を落として重複を無くした
(`tests/drive_browser.rs` を `browser_pane::` 参照へ書き換え)。

## 器具の値を使った箇所

`browser_pane.rs` の `pub mod dims` / `pub mod colors`(全定数に
`browser-library.css:行番号` の doc コメント付き)。主要な実測(viewport
900×600、`motolii-css-metrics browser`):

| dims 定数 | 値(px) | css 出所 |
|---|---|---|
| `HEADER_H` / `HEADER_PAD_X` | 26.0 / 8.0 | `.browserHeader{height:26px;padding:0 8px}` |
| `TOOLBAR_H` / `TOOLBAR_PAD_X` / `_PAD_Y` | 30.0 / 5.0 / 3.0 | `.browserToolbar{height:30px;padding:3px 5px}` |
| `CONTROL_H` | 21.0 | 共通 control(history/toolbar/検索欄/viewModes button) |
| `CATALOG_HEADER_H` / `_PAD_X` | 31.0 / 6.0 | `.catalogHeader{height:31px;padding:0 6px}` |
| `VIEW_BUTTON_W` / `_GAP` | 21.0 / 2.0 | `.viewModes button{width:21px}` / `.viewModes{gap:2px}` |
| `SUMMARY_H` / `_PAD_X` | 21.0 / 6.0 | `.resultSummary{height:21px;padding:0 6px}` |
| `GRID_PAD_X` / `_BOTTOM` | 1.0 / 3.0 | `.thumbnailGrid{padding:0 1px 3px}` |
| `CARD_PAD` / `THUMB_PAD` | 3.0 / 3.0 | `.libraryCard{padding:3px}` / `.libraryThumb{padding:3px}` |
| `TRAY_H` / `_PAD_X` / `TRAY_DOT` | 27.0 / 6.0 / 5.0 | `.selectionTray{height:27px;padding:0 6px}` / `.selectionDot{5x5}` |
| `SIDEBAR_H2_H` / `ROW_H` / `ROW_PAD_X` | 16.0 / 19.0 / 7.0 | `.librarySidebar h2{height:16px}` / `.locationRow{height:19px;padding:3px 7px 0}` |
| `ROW_INDENT` | 13.0 | `.locationRow.indent{padding-left:13px}` |
| `SHELF_MIN_H` / `_PAD_X` / `_PAD_Y` | 24.0 / 5.0 / 3.0 | `.filterShelf{min-height:24px;padding:3px 5px}` |
| `CHIP_MIN_H` / `_PAD_X` / `_PAD_Y` | 17.0 / 5.0 / 2.0 | `.filterShelf button{min-height:17px;padding:2px 5px}` |

`SIDEBAR_W`(112.0)/`SIDEBAR_W_NARROW`(92.0)は css の宣言値
(`@media (max-width:420px)`)を直接使う関数 `dims::sidebar_width(pane_w)` —
box が pane 幅に依存する値は絶対座標を転記しない(前レーンの観察と同じ理由)。
`THUMB_ASPECT`(16/9)も同様に**比率だけ**を css から借り、実 px は
`dims::thumb_height(pane_w, columns)` が既知の pane 幅から解く(iced は
egui の `ui.available_width()` に相当する実行時の利用可能幅を widget 構築前に
読めないため)。色は `pub mod colors` に同じ流儀で 30 個(`PANEL_BG` から
`TRAY_META_FG` まで、すべて css:行 引用)。

sidebar / filterShelf の中身は html が `data-tab`/`data-view` を JS で設定する
ため、素の html を抽出すると 0×0 になる(既知の罠)。
`tests/css_metrics_oracle.rs::extract_browser_with_initial_media_tab()` が
html の `<script>` 冒頭の決定的な初期代入をそのまま `data-tab="media"
data-view="grid"` として `<main>` へ静的に差し込んだ一時 copy を作り、そちら
を抽出することでこの2枚も実測できた(前レーンが「次に自動化できそうな
こと」として書き残していた手を実行 — `docs/mocks-ui` 自体は書き換えない)。

## 移植した機能(理由つき)

| 機能 | html | 移植元 | 実装 |
|---|---|---|---|
| 検索欄 | `#library-search`(html:25) | egui `BrowserPanel::set_query`(trim+小文字化) | `BrowserPane::set_query` — 既存 `cards()` の末尾に絞り込みを追加(既定=空文字は常に真、既存呼び出しの挙動を変えない) |
| kind filter chip(Video/Image/Audio) | `.filterShelf button`(html:103) | egui `BrowserPanel::set_kind_filter`(完全一致) | `BrowserPane::set_kind_filter` |
| Clear | `.clearFilter`(html:103) | egui の Clear 相当 | kind filter と検索語を両方リセット |
| 表示モード(Thumbnails/Grid/List) | `.viewModes button`(html:94-97) | egui `BrowserPanel::set_view` | `BrowserViewMode` + `dims::columns()`(4/2/1、html と同じ) |
| filter shelf の開閉 | `#filter-toggle`(html:26) | egui `BrowserPanel::shelf_open` | `docs/ui-interaction-language.md`:122 の要求(常設panelでは検索→結果の視線を遮る filter 行を常設しない)を、常設パネルのまま開閉可能にする形で満たす |

html の B-roll/Brand のような派生 tag chip は動的に出さない — この product
の実データ(starter media は平坦な1 folder)には kind 以外に意味のある tag
が無く、指示は Video/Image/Audio + Clear の3+1個を名指ししていた。

## 置かなかった chrome(理由つき)

| html にある物 | 理由 |
|---|---|
| history ‹›(html:23-24) | html 自身が無条件 `disabled`。ナビゲーション履歴という feature が product に無い(Undo/Redo のような「空だから disabled」ではなく機能そのものが存在しない) |
| Tags toggle / tag editor / "Edit tags" | `media_library` の tag は kind・拡張子・folder 名からの derive-only。追加/削除する API が無い |
| Media/Effects/Create/Panels タブ | egui 版でも Effects/Create/Panels は `visible_cards()` が常に空(項目 model が無い)。1枚しか働かない tab strip は tab ではない |
| COLLECTIONS(Favorite/B-roll/Brand) | 人が能動的に印を付ける「収集」概念が product に無い。`docs/ui-interaction-language.md`:77 も将来枠として名指ししているだけ |
| PLACES(複数登録 folder) | `MediaLibrary` は `project()` が `roots.first()` しか見ない(実質1 root)。複数 location 切替は機能が無い |
| 右クリック context menu(2枚とも) | html 自身、ほぼ全項目が `disabled`(Preview/Copy source path/Reveal in folder/Place in composition)。残る Favorite 登録・tag 編集も上記の理由で機能が無い |

これらは次の発注の材料として本 README に列挙してある。

## 実装中に見つけて直した不具合(自分の目で見た差分)

### 1. `container::Style.border` は四辺一括 — 単辺の罫線に使うと箱になる

`iced::Border` に per-side width が無い。header/toolbar/catalogHeader/
filterShelf/tray の `border-bottom`/`border-top`(css は単辺)を最初
`container::Style.border` で実装したところ、四辺とも塗られて「帯の下に
1px 線」ではなく「帯が箱で囲まれる」絵になっていた。この製品の既存流儀
(`theme/style.rs::separator` — `rule::horizontal(1)` を同胞 widget として
挿す)に揃え、`hairline_h`/`hairline_v` ヘルパへ差し替えた。

同じ理由で `.locationRow` の選択時 `border-left: 2px solid` (html:132/144)
も最初は `button::Style.border` で実装し、**選ばれた rail が四辺とも accent
色の箱に見える**不具合になっていた(スクリーンショットで実際に確認 —
下記 iced-browser-crop.png は修正後)。`sidebar()` を、accent bar(2px 幅の
別要素)+ button(面と字だけ)の2要素構成へ直した。button 単体では表現
できない単辺装飾は、**別要素として置く**のがこの iced fork での正しい形。

### 2. thumb の高さが `Fill` in `Shrink` で 0 になっていた

`.libraryThumb`(css:241 `aspect-ratio:16/9`)を最初
`container(...).width(Fill).height(Fill)` で実装し、それを高さ未指定の
`column![thumb, card_copy]` へ入れていた。iced の `Fill` は親が確定した
高さを持つときだけ効くので、親が `Shrink`(既定)だと `Fill` の子は 0 高さに
潰れる(egui の immediate-mode と違い、`ui.available_width()` を実行時に
読めない — module doc「iced の flex 相当」節の逆側)。`dims::thumb_height
(pane_w, columns)` を新設し、既知の pane 幅・列数から css の
`aspect-ratio:16/9` を解いた実 px を先に計算して渡す形に直した。

なお、抽出結果(`out/browser.json`)で `.libraryThumb` の box を直接見ると
w=15/h=18 のような極小値が返っていた — `<span>` + `display:flex` +
`aspect-ratio` のみで明示 width が無い箱を、Blitz の flex 解決が内容量
ベースの極小値へ寄せてしまう既知のずれ(review 文書「対象は個別レイアウトの
絶対座標ではない」と同じ理由)。この値は転記せず、比率(`THUMB_ASPECT`)
だけを css から借りて上記の通り自前で解いた。

## oracle の強化

`crates/motolii-shell-iced/src/browser_pane.rs` の `dims`/`colors` を
`pub mod` にして、`tests/css_metrics_oracle.rs` に Browser 側の**両方向
チェック**を3本追加した(Timeline 側と同じ「実物を `use` する」形 —
前レーンが Inspector について「`pub(crate)` へ上げられれば両側化できる」と
書き残していた提案を実行):

- `browser_dims_match_css_computed_values` — header/toolbar/catalogHeader/
  viewModes/resultSummary/grid/card/thumb/tray(素の html で足りる箇所)
- `browser_sidebar_and_filter_shelf_dims_match_css_computed_values` —
  sidebar h2/locationRow/indent/filterShelf/chip(`data-tab` 静的注入が要る箇所)
- `browser_sidebar_width_and_thumb_height_follow_the_css_declarations` —
  pane 幅依存の2関数(`sidebar_width`/`thumb_height`)を css の宣言
  (breakpoint・aspect-ratio)と独立に組み直して突き合わせ

## テスト集計(red → green)

1. **red 先行**: `browser_pane.rs` 新設直後、`cargo check -p
   motolii-shell-iced` は2件の compile error(`Padding: From<[u16;4]>` 未実装、
   `toolbar()` の借用が `'static` を満たさない)。修正後 green
2. `css_metrics_oracle.rs` に Browser の assert を足した直後、28件が
   `f64`/`f32` 型不一致で compile error。`box_h`/`pad_left` 等の呼び出し側へ
   `as f64` を足して green
3. `intent_gateway_fence.rs::every_product_source_is_scanned` が
   `browser_pane.rs` の追加を検出して red(走査表に無い file は柵を素通り
   する、という柵の意図どおりの検出)。表へ1行足して green
4. `cargo test -p motolii-shell-iced -j 5 --lib --tests --no-fail-fast` —
   **24 test binary(lib 含む)、142 tests、全 green**(既存分含め1件も
   落としていない。`drive_browser.rs` の既存9テスト — rail 切替・click
   選択・double-click 配置・project rail dedupe・recent 順・空状態・drop
   hover — は無改変で green のまま、新規5テスト — 検索・検索0件・kind
   chip トグル・Clear・filter shelf 開閉・表示モード切替 — を追加)
5. `motolii-ui --lib` の `timeline_editor::audio_seat::
   a_real_device_session_starts_and_reseeks_at_the_playhead` が全体テスト
   実行時に1回だけ落ちた(実 device の callback が 150ms 待ちの間に
   進まなかった)。**この browser レーンとは無関係**(motolii-ui は未変更 —
   `git diff --stat` で確認)。単独再実行(`cargo test -p motolii-ui --lib
   timeline_editor::audio_seat`)では green — 並走中の他レーンによる CPU
   競合が原因の環境依存 flake と判断した

## 残った差(自分の目で見比べて、正直な列挙)

- **header 右の文言**: 基準は `LOCAL LIBRARY`(固定文言)。この iced 実装は
  登録 folder の実名(`pane.library_root_name()`、スクリーンショットでは
  `media`)を出す — この判断は本レーンより前から在った(`view.rs` の元実装を
  そのまま `browser_pane.rs` へ移しただけで、今回変えていない)。実 folder 名
  を出す方が「今どの folder を見ているか」の情報量が増えるので、直さず残した
- **Filters toggle の押下状態の色**: html/css には `#filter-toggle` 専用の
  pressed 状態スタイルが無い(素の `.toolbarButton` のまま)。この実装は
  filter shelf が開いている間、view-mode ボタンの pressed 語彙(accent 縁+地)
  を流用して視覚フィードバックを足した — Q3(全操作に即時視覚応答)を満たす
  ための能動的な追加で、html には無い状態を1つ発明している。次回 html 側に
  対応する視覚が定義されたら、そちらに合わせ直す
- **card の box-shadow(css:250 `inset 0 0 0 1px …55`)**: 選択 thumb の
  二重枠(境界線の内側にもう1本薄い線)は iced の `container::Style` に
  box-shadow 相当が無いため未実装。単一の accent 枠だけになっている
- **`.locationRow` の縦 padding(css:130 `padding: 3px 7px 0`)**: 横方向
  (7px)は再現したが、上下は button の中央揃えに任せている(css は
  padding-top 3px・bottom 0 で厳密には中央より僅かに上寄りになる)。19px の
  行高では視認できる差ではないため直していない

## token へ追加した値

**0件の hex 発明。** `dims`/`colors` は全部 `browser-library.css:行番号` の
写し(値は `motolii-css-metrics browser` で実測検証済み)。`theme::Tokens`
の21 semantic role には対応が無い raw hex を使っている箇所があるが、これは
egui 側 `browser_panel/theme.rs` と同じ既存の事情(この製品の色は raw hex
で token 対応が無い)であって、新しい hex を発明したわけではない。

## 変更ファイル

- `crates/motolii-shell-iced/src/browser_pane.rs`(新規、約1,300行) —
  Browser pane の絵ぜんぶ
- `crates/motolii-shell-iced/src/browser.rs`(加法) — `BrowserViewMode`、
  `query`/`kind_filter`/`view`/`shelf_open` state + getter/setter、
  `cards()` への絞り込みフィルタ
- `crates/motolii-shell-iced/src/message.rs`(加法) — `BrowserQueryChanged`/
  `BrowserKindFilterChosen`/`BrowserFiltersCleared`/
  `BrowserFilterShelfToggled`/`BrowserViewChosen`
- `crates/motolii-shell-iced/src/shell.rs`(加法、既存 arm 無改変) — 上記
  Message の5新規 arm
- `crates/motolii-shell-iced/src/view.rs`(大幅削減、約450行 → 呼び出し1行)
- `crates/motolii-shell-iced/src/lib.rs`(module doc 更新 + `pub mod
  browser_pane` 追加)
- `crates/motolii-shell-iced/tests/drive_browser.rs`(既存9テストの
  `view::BROWSER_*` → `browser_pane::*` 改名 + 新規5テスト)
- `crates/motolii-shell-iced/tests/common/mod.rs`(加法) — `type_into`
  ヘルパ(text_input への打鍵)
- `crates/motolii-shell-iced/tests/css_metrics_oracle.rs`(加法) — Browser
  oracle 3本
- `crates/motolii-shell-iced/tests/intent_gateway_fence.rs`(加法1行) —
  `browser_pane.rs` を走査表へ
