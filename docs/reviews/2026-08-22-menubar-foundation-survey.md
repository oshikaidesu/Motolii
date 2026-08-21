# メニューバー/右クリック基盤 campaign 設計調査

日付: 2026-08-22 / 発注: 保留玉(S6 併存原則つき campaign 起草)/ レーン: read-only 調査(sonnet、**cargo build/test 禁止**)
背景: [S 空間スコア(ui-spatial-score.md)](../ui-spatial-score.md) S6 非隠蔽公理・S0 慣習段差 / [κ 台帳(2026-08-21-ui-entrance-atlas-survey.md)](2026-08-21-ui-entrance-atlas-survey.md) FINDING 2/3(右クリック基盤・メニューバー基盤の構造的不在)/ 正本= `next/reference/normal-map.tsv`(1,551行、裁定158)

## 要約(RETURN)

- メニュー構造は**発明せず** normal-map の entries(menu 列)分布から導出した。Motolii は AE 型 Layer/Composition モデル(Premiere/Resolve の Clip/Sequence モデルではない)なので、AE の8トップレベル(File/Edit/Composition/Layer/Effect/Animation/View/Window/Help)を構造の一次先例とし、4製品統合後の freq 分布で中身を埋めた(§1)。
- **ネイティブ macOS メニューバー(NSMenu)は現行 iced 0.15 pin(fork `motolii/host-seams`)の winit 統合に存在しない** — `ShowSystemMenu`/`ContextMenu` はタイトルバー系メニューで、アプリ全体のメニューバーではない。ゼロから objc2/muda 級の別依存を足す必要があり、**アプリ内メニューバーを推奨**(§2)。
- iced 標準 widget に「メニューバー」相当はゼロ。ただし **`overlay::menu::Menu`(pick_list/combo_box が内部で使う単段ドロップダウンリスト)は `pub` で公開されており、これをそのまま流用してトップレベルボタン+ドロップダウンを自作するのが最小手数**(wraps)。`widget::pin::Pin` も TL-arch 調査(2026-08-22-timeline-canvas-widget-survey.md)で絶対配置の一級部品と確認済みで併用可能。外部 crate `iced_aw`(0.15.0-dev 対応の `MenuBar`/`Menu` を実装済み)は入れ子サブメニュー込みの完成品だが、**`iced-rs/iced` branch=master への git 依存で、当方 fork `oshikaidesu/iced` rev 固定と衝突する**(`[patch]` 未設定、互換性未検証)ため要 EVIDENCE_GAP(§3)。
- S6 併存表(§4): 現行実装で調査できた範囲では、メニュー候補の主要動詞(Undo/Redo/Copy/Paste/Cut/Duplicate/Select All/Deselect All/Import)は**既に shortcut または D&D の別入口を持つ**ため、メニュー化しても「唯一の入口」にはならない。一方 **New Project/Save As/Save a Copy/Zoom In・Out/Paste Attributes/Find/Label Color 手動指定は現状 next/ に入口がゼロ**で、メニューだけを足すと S6 違反(唯一の入口)になる — この班は shortcut または直接操作の同時追加を伴わせる必要がある。
- 走行中レーン(I-tokens: inspector token 再転写 / TL-P1: rail widget 化)とは write-set が重ならない(header chrome ~ 新規 menubar crate 対 inspector-pane/timeline-pane)。M4(ゼロコピー presenter)は既に検収合格・merge 済みで走行中ではない。**MK2(mask 被覆代数, `engine/src/mask.rs`)・H2・TP という名のレーンは、`docs/reviews/2026-08-21-lane-board.md` の「走行中(未返却)」表および `docs/reviews/2026-08-21-backend-gap-seam-survey.md` に見当たらなかった**(MK2 は backend-gap-seam-survey 上は未発注の設計候補行として存在するのみ)。現存する走行中3レーン(I-tokens/TL-P1、M4 は完了)との衝突は無いと判断するが、MK2/H2/TP の実在と write-set は本レーンの読み取り範囲(委任時に渡された文書群)では確認できなかった — 発注前に呼び出し元セッションでの再確認を推奨(§6 EVIDENCE_GAP)。

---

## §1 map 由来のメニュー構造案

### 1.1 導出方法

`next/reference/normal-map.tsv`(裁定158で採用済92/採用予定1,195/不採用264、未判定0)から `entries(menu:shortcut:panel:pref)` の menu 件数 ≥1 の行を機械抽出した。

```
採用済+採用予定のうち menu≥1: 606行(全1,551行の約39%)
freq≥2 かつ menu≥1(優先キュー): 25行
```

トップレベルのラベルは **4製品の出典生データ自身の menu パス先頭語**を集計して決めた(`docs/reviews/2026-08-21-normal-map-sources/*.md` の `menu\t<path>\t...` 行、`path.split('>')[0]`):

| 製品 | menu行数 | トップレベル頻度(上位) |
|---|---|---|
| AE | 417 | Layer 159 / File 55 / Animation 49 / Edit 42 / View 40 / Composition 21 / (App menu) 20 / Help 16 / Window 12 |
| Premiere | 86(採取分) | File 20 / Sequence 19 / Edit 16 / Markers 16 / Clip 15 |
| Resolve | 357 | Timeline 40 / File 36 / Edit 36 / Mark 34 / Trim 31 / Color 30 / Workspace 29 / Playback 27 / Clip 26 / View 25(+Fairlight/Fusion は scope=out-of-domain?) |
| CapCut | 43(採取分) | Toolbar 中心・出典が薄く階層化されていない(normal-map-README 既知の限界1) |

**Motolii は Layer/Composition モデル**(`docs/concept.md` — DAW Project 相当が `Document`、Timeline 上の子は Layer)であり、Premiere/Resolve の Clip/Sequence モデルではない。よって構造の一次先例は AE の8トップレベルとし、Premiere/Resolve 由来語彙(Clip 系操作)は「Layer に対する動詞」として AE 側の Layer メニューへ意味的に合流させる(操作の中身は4製品統合後の freq、器の名前は Motolii 自身のオブジェクトモデルに従う、という2軸の分離)。

### 1.2 構造案(トップレベル7 + 検討中1)

各項目は `canonical | map行id | freq | entries` の形で引用する。**発明ゼロ — 全て normal-map の実在行**。

#### File
| 項目 | id | freq | entries |
|---|---|---|---|
| New Project | 1221 | 4 | 4:0:0:0 |
| Import (media/file) | 592 | 4 | 4:1:0:0(**採用済**・既存入口=OSドロップのみ、κ FINDING) |
| New Sequence/Timeline/Composition | 1315 | 2 | 2:1:0:0 |
| Save As | 1225 | 3 | 3:1:0:0 |
| Save a Copy | 1227 | 2 | 2:0:0:0 |
| Quit / Exit | 1223 | 3 | 3:3:0:0 |

#### Edit
| 項目 | id | freq | entries |
|---|---|---|---|
| Undo | 437 | 3 | 3:3:0:0(**採用済**・header ボタン+Cmd+Z 配線済み、λ レーン) |
| Redo | 435 | 3 | 3:3:0:0(同上、Cmd+Shift+Z) |
| Cut | 432 | 3 | 3:2:0:0(**採用済**・Cmd+X 配線済み) |
| Copy | 429 | 4 | 4:2:0:0(**採用済**・Cmd+C 配線済み) |
| Paste | 430 | 4 | 4:2:0:0(**採用済**・Cmd+V 配線済み) |
| Duplicate | 434 | 3 | 3:1:0:0(**採用済**・Cmd+D 配線済み) |
| Select All | 436 | 3 | 3:3:0:0(**採用済**・Cmd+A 配線済み) |
| Deselect All | 433 | 3 | 3:3:0:0(**採用済**・Cmd+Shift+A 配線済み) |
| Paste Attributes | 440 | 2 | 2:2:0:0(次項参照) |
| Paste Insert | 441 | 2 | 2:1:0:0 |
| Find | 439 | 2 | 2:1:0:0 |
| Edit Original | 438 | 2 | 2:1:0:0 |
| Keyboard Shortcuts (editor) | 1145 | 2 | 2:2:0:0 |

#### Layer(Motolii の Layer に対する動詞 — Premiere/Resolve の Clip 語彙をここへ合流)
| 項目 | id | freq | entries |
|---|---|---|---|
| New solid layer | 900 | 1 | 0:1:0:0 |
| Split / Razor (clip at playhead) | 163 | 3 | 3:2:0:0 |
| Speed / Duration | 169 | 2 | 3:2:0:0(**S0 済み判定**・Inspector 数値欄+Reset で既に入口あり、ε レーン) |
| Make Subclip | 166 | 2 | 2:2:0:0 |
| Replace (footage/clip) | 167 | 2 | 2:1:0:0 |
| Apply Video/Audio Transition | 164/165 | 2/2 | 2:2:0:0(両方) |
| blend_mode 束(例: Add) | 75 | 1 | 1:0:0:0(**採用済**・Inspector 巡回ボタンで既に入口あり、α レーン) |
| label_color 束(例: Aqua/Blue/Brown) | 624/625/626 | 各1 | 1:0:0:0(**現状入口ゼロ** — `label_color_for_new_layer` は id%12 の自動割当のみで手動変更 Message が無い、ρ レーンの範囲外) |
| mask 束(例: Add/Closed/Create Masks from Text) | 748/750/751 | 各1 | 1:0:0:0(**backend 依存** — MK2/MK3 未着手、γ レーンは MK1 ラスタ配線のみ) |

#### View
| 項目 | id | freq | entries |
|---|---|---|---|
| Zoom In | 1441 | 3 | 2:2:0:0(**現状入口ゼロ** — timeline-pane に wheel/±キーの zoom 実装が grep 上見当たらない) |
| Zoom Out | 1442 | 3 | 2:2:0:0(同上) |

(1/2 プレビュー・市松・観測カメラ状態は μ レーンで既に状態帯へ着地済み・メニュー化不要、S6 違反第1号は解消済み)

#### Window(workspace/panel_window 系 — freq は薄いが AE で12件・Resolve workspace 29件と厚みがある層)
代表 freq1 行(全30行から抜粋、`view!30`カテゴリ全体は本調査の対象外・器具計画で機械抽出予定):
| 項目 | id | freq | entries |
|---|---|---|---|
| Active Panel Selection | 1495 | 1 | 1:0:0:0 |
| All Panels | 1496 | 1 | 1:0:0:0 |

#### Help(20行、ほぼ freq1)
代表: `After Effects Help…`(id 572, freq1, 1:0:0:0)。

#### 検討中: Marker/Mark(独立トップレベルにするか Layer へ畳むか未決)
freq≥2 優先キュー25件のうち **7件(28%)が marker カテゴリ**で、Premiere(`Markers`)/Resolve(`Mark`)は独立トップレベル、AE には無い(Layer 内の Add Marker が近い)。
| 項目 | id | freq | entries |
|---|---|---|---|
| Mark In | 725 | 2 | 2:2:0:0 |
| Mark Out | 726 | 2 | 2:2:0:0 |
| Mark Clip | 724 | 2 | 2:2:0:0 |
| Mark Selection | 727 | 2 | 2:2:0:0 |
| Clear In and Out | 720 | 2 | 2:2:0:0 |
| Clear In | 719 | 2 | 1:2:0:0 |
| Clear Out | 721 | 2 | 1:2:0:0 |

S0(慣習段差、辞書式最優先)はエントリの**種別**分布(menu:shortcut:panel:pref)を裁くもので、トップレベルの**分割**自体は判定材料が無い(AE=無・Premiere/Resolve=有、で先例が割れる)。この1点は **EVIDENCE_GAP**(§6)へ送る。

---

## §2 ネイティブ vs アプリ内メニューバー

### 2.1 実測: 現行 iced pin にネイティブ統合は無い

- `next/Cargo.toml` は `iced = { git = "https://github.com/oshikaidesu/iced", rev = "73e686ee..." }`(fork `motolii/host-seams`。ω 調査で fork=upstream master ドリフト0 と確認済み)。
- fork チェックアウト(`~/.asdf/.../iced-1bbb4ed9d90ae4f8/73e686e/winit/src/lib.rs`)を grep すると、`menu`/`Menu` に一致するのは `window::Action::ShowSystemMenu`(`raw.show_window_menu(...)`)のみ。これは**タイトルバーの右クリックシステムメニュー**(Windows のウィンドウ制御メニューに相当)であり、macOS の NSMenu(画面最上部のアプリメニューバー)ではない。
- `winit/src/conversion.rs` に出てくる `Menu` はキーボードの `ContextMenu` キー(コンテキストメニューキー)のマッピングのみで、無関係。
- iced の `winit` 依存自体(`winit = { git = "https://github.com/iced-rs/winit.git", rev = "05b8ff17..." }`)も、iced 側の shell(`winit/src/lib.rs`)がメニューバー構築 API を公開していない以上、Motolii 側から到達できない。

つまり**現行スタックには NSMenu 経路が存在しない**。ネイティブメニューバーを実現するには、`objc2-app-kit`(macOS 専用)や `muda`(Tauri が使うクロスプラットフォームのネイティブメニュー crate)のような**別依存を新規に足す**必要がある。`muda` は winit と共存前提で設計されてはいるが、iced の `winit` 統合が生の `winit::window::Window` ハンドルをどこまで外へ渡しているかは本調査では未確認(EVIDENCE_GAP)。

### 2.2 先例(AE/Resolve/Ableton)

3製品とも macOS では OS ネイティブのアプリメニューバー(画面最上部)を使う。これは macOS の HIG(メニューは画面最上部に1本、ウィンドウに付随しない)への適合であり、クロスプラットフォームの Premiere/Resolve/Ableton も同様(Windows 版はウィンドウ内メニューバー)。**プラットフォームごとに UI が変わる**のが慣習であり、Motolii がクロスプラットフォーム前提でアプリ内メニューバーに統一しても、少数派側(macOS ネイティブ勢)から見て奇異ではあるが、Electron 系(VS Code 等)のようにアプリ内カスタムタイトルバー+メニューを敷く製品も一般化している。

### 2.3 推奨

**アプリ内メニューバーを推奨**する。理由:
1. 保守最低限(`maintenance-minimal-no-scratch.md`)の "wraps>移植>スクラッチ" — ネイティブ経路は新規外部依存+プラットフォーム分岐コードを増やすが、アプリ内は既存 iced widget の組み合わせで完結する。
2. S0 の判定対象はエントリ**種別**(menu か shortcut か)であり、その menu が OS ネイティブか自前描画かは S0 の管轄外 — アプリ内でも「メニューという入口種別」自体は満たせる。
3. iced の全 UI が tokens 経由の自前描画(トンマナ柵、裁定142)である現状と、ネイティブ NSMenu(OS 標準フォント・OS 標準配色)の混在は視覚的異物になりやすい。アプリ内なら tokens で統一できる。
4. 将来 Windows/Linux 版を出す場合、ネイティブ経路は3系統(NSMenu/Win32 menu/GTK menu)の個別実装が要るが、アプリ内は1実装で足りる。

ただし **macOS の Cmd+Q(強制終了)や Dock メニュー等、OS が期待する最小限のシステム統合は別途確認が必要**(EVIDENCE_GAP)。

---

## §3 iced 側の道具実測と実装経路

### 3.1 標準 widget 実測(pin rev `73e686ee`)

```
next/../widget/src/ に "menu" を含むファイルは無い(button/pick_list/combo_box/pane_grid/pin/...)
唯一の例外: widget/src/overlay/menu.rs(648行)
```

`overlay::menu::Menu<'a,'b,T,Message,...>` は **`pub struct`** で公開されており、`options: &'a [T]` の**単段フラットリスト**を描く overlay。`pick_list.rs`/`combo_box.rs` はこれを内部で呼んで自分のドロップダウンを作っている。ネストしたサブメニュー・キーボードニーモニック・水平配置バーの管理機能は無い ── これは「メニューバー」ではなく「メニューバーの中の1段」を作る部品。

TL-arch 調査(`2026-08-22-timeline-canvas-widget-survey.md`)は同じ pin rev で `widget::pin::Pin` を「絶対配置が標準搭載・自作 layout container 不要」と確認済み(前提転覆の発見)。ドロップダウンパネルをボタン直下へ固定する用途に転用できる。

### 3.2 外部 crate: `iced_aw`

ローカルに `iced_aw-262bd70fcc27b1b7`(rev `924be28`)のチェックアウトがあり、`src/widget/menu/{menu.rs, menu_bar.rs, menu_bar_overlay.rs, menu_tree.rs}` という**完成品の `MenuBar`/`Menu` ウィジェット**を確認した。`MenuBarState`/`GlobalState`/`try_open_menu` 等、ホバーで隣のトップレベルへ切り替わる・クリックで開く、という NLE で期待される挙動一式が実装済み。

**懸念**: `iced_aw` の `Cargo.toml` は `iced = { git = "https://github.com/iced-rs/iced.git", branch = "master" }`(upstream 直参照)であり、当方の `iced = { git = "https://github.com/oshikaidesu/iced", rev = "73e686ee..." }`(fork)とは**別ソース**として Cargo に認識される。ω 調査で「fork は upstream master と commit sha 完全一致」と確認済みなので理論上は互換のはずだが、Cargo の依存解決は git ソースの URL で区別するため、素の `Cargo.toml` 追加だけでは**同じ `iced_core`/`iced_widget` 型が2重に存在するビルドエラー**(M01 移行時に φ が踏んだ「iced_core 2本混線」と同型の罠)になる可能性が高い。回避には `next/Cargo.toml` に `[patch."https://github.com/iced-rs/iced.git"]` で fork へリダイレクトする一手間が要る(M01 で `iced_test` に対して同種の patch を既にやっている先例があるはず、コード未確認・EVIDENCE_GAP)。

### 3.3 三案比較

| 案 | 中身 | 保守負債 | 到達機能 | 判定材料 |
|---|---|---|---|---|
| A: 標準部品の組み合わせ(wraps) | トップレベル= `button` の `row!`(既存 header と同じ文法)。クリックで `overlay::menu::Menu` を1段開く(pick_list と同じ overlay 機構を直接使う) | **最小**。既存 iced のみ、新規 crate 依存ゼロ | サブメニュー無し(v1 は File>Save As のような1段運用に限定。Layer>Blend Mode>Add のような2段は overlay を入れ子にする自作コードが要る) | 保守最低限の原則に最も合致。TL-arch の Pin 発見と合わせ、ドロップダウン位置決めも標準部品で足りる |
| B: `iced_aw::menu`(外部 crate) | 完成品 MenuBar、ネスト対応、ホバー切替 | **中**。fork 互換性が未検証(`[patch]` 要・EVIDENCE_GAP)、外部 crate の維持体制は iced 本体ほど実績が無い(iced_aw は iced-rs org 配下だが更新頻度・0.15 対応の安定度は本調査で未確認) | フル機能(ネスト・ホバー) | 「移植」に近い外部採用。当たれば工数最小だが、fork 互換性の検証コストを先に払う必要がある |
| C: スクラッチの overlay trait 実装 | `iced_core::overlay::Overlay` を自前実装 | **最大**。保守最低限の原則(裁定 — ハック強制時のみスクラッチ許可)に反する | フル機能 | 非推奨。A で足りない機能(ネスト)が出た時点でも、まず A の overlay 入れ子で足りるか検証してから検討する順序が正しい |

**推奨: 案A(標準部品の組み合わせ)を v1(File/Edit の1〜2段構成)で先行させ、ネスト深度が実際に要る段(Layer>Blend Mode の12値巡回等)で案Bの互換性検証を別途行う**。案Bの検証自体は cargo build を要するため本レーン(read-only)の範囲外。

---

## §4 S6 併存原則の適用表

S6(`ui-spatial-score.md` §6)は「唯一の入口を隠し場所にしない」— メニューに載る動詞は必ず shortcut/直接操作/状態表示のいずれかと併存しなければならない。§1 の構造案から代表行を抜き、現状の他入口の有無を照合した。

| 動詞 | map id | メニュー以外の現在の入口 | S6 判定 |
|---|---|---|---|
| Undo/Redo | 437/435 | header ボタン+Cmd+Z/Shift+Z(λ 配線済み) | **併存**(メニューは第3の入口) |
| Copy/Paste/Cut/Duplicate/Select All/Deselect All | 429/430/432/434/436/433 | Cmd+C/V/X/D/A/Shift+A(λ 配線済み) | **併存** |
| Import (media/file) | 592 | OS ドロップ(D&D) | **併存**(κ が「menu優勢なのに D&D のみ」と指摘した違反はメニュー追加で解消) |
| Speed/Duration | 169 | Inspector 数値欄+Reset(ε 配線済み) | **併存** |
| Blend Mode(例: Add) | 75 | Inspector 巡回ボタン(α 配線済み) | **併存** |
| New Project | 1221 | **無し**(rfd 未接続、`next/` に New/Save 系 Message が存在しない — grep 実測) | **メニューのみだと違反**。ショートカット(Cmd+N 相当)を同時配線するか、rfd 裁定(§6)を先に固める必要 |
| Save As / Save a Copy | 1225/1227 | **無し**(同上) | **同上、違反リスク** |
| Zoom In/Out | 1441/1442 | **無し**(timeline-pane に wheel/±キーの実装が grep 上見当たらない) | **メニューのみだと違反**。同一切片で `+`/`-` キーか wheel zoom の追加が必須 |
| Paste Attributes/Paste Insert/Find | 440/441/439 | **無し** | 同上(メニュー追加時は同時に shortcut か直接操作を1つ足す) |
| Label Color(手動指定、例: Aqua) | 624等 | **無し**(現状 id%12 自動割当のみ、手動変更 Message 不在) | メニューを唯一の入口にしないためには、**右クリック基盤(本 campaign の後半切片)か Inspector スウォッチのどちらかを同時に用意**する必要がある |
| Mask(Add 等) | 748等 | **無し**(backend 側 MK2/MK3 未着手 — 動詞自体が engine に無いため UI 単体では成立しない) | 依存待ち。メニュー項目を先に置いても機能しないため本 campaign の対象外に保留 |

**設計保証**: メニュー構造案(§1)のうち、現状「入口なし」の項目(New Project/Save As/Save a Copy/Zoom In・Out/Paste系/Find/Label Color 手動指定)は、**メニュー実装と同じ切片でショートカットまたは直接操作の追加を必須要件として発注書へ明記する**(S6 の「メニューに載る各動詞について唯一の入口がゼロ」という保証をそのまま満たすため)。逆に既に別入口がある項目(Undo/Redo/Copy/Paste 等)は、メニューは純粋な「初回発見用の可視化」(S4 発見依存度が既に低い= 低重み表示で足りる)として追加するだけでよい。

---

## §5 切片割り案(重み均等)

裁定の「行数+判断の重さ+領域数の3軸で均す」「write-set を互いに素に」に従い、4切片へ分ける。

| 切片 | 中身 | write-set | 判断の重さ | 依存 |
|---|---|---|---|---|
| **MB-0: 基盤 widget** | `overlay::menu::Menu` を包む1段ドロップダウン部品(トップレベルボタン row + クリックで開く overlay)。新規 crate または `motolii-shell` 内モジュール1本。位置決めは Pin 転用。tokens 経由の見た目(裁定142 柵) | 新規ファイル(crate 化するかは要判断・shell.rs 非侵襲) | 中(iced overlay API の初採用、TL-arch の Pin 発見を追試する形なので前例あり) | なし(独立着手可) |
| **MB-1: File/Edit 束** | §1 File/Edit の構造案を実配線。**Undo/Redo/Copy/Paste/Cut/Duplicate/Select All/Deselect All/Import は既存 Message へメニュー項目をぶら下げるだけ**(S6 既に併存)。New Project/Save As/Save a Copy は rfd 裁定(§6)待ちのため本切片では「無効化 disabled 表示+placeholder Message」に留める案(Q0 違反回避=効かない chrome を並べない、なら**未実装の間はメニュー項目自体を出さない**のが正) | shell.rs の header 差し替え + MB-0 呼び出し | 中(既存 Message への配線が主、新規ロジックは薄い) | MB-0 |
| **MB-2: Layer/View 束** | Layer メニュー(blend mode/label color/mask は既存入口の有無で出し分け — S6 §4 の判定に従う)+ View メニュー(Zoom In/Out は同時にショートカットも新設必須)。Marker/Mark をここに含めるか独立トップレベルにするかは §1 の EVIDENCE_GAP 次第 | shell.rs + inspector-pane(label color スウォッチを追加するなら) + timeline-pane(zoom 実装) | **高**(Zoom の新規実装・label color 手動指定の新規 Message・S6 判定の個別処理が絡む) | MB-0、MB-1 と write-set 重複なし(触るパネルが別) |
| **MB-3: 右クリック基盤** | κ FINDING 2 の根治(「対象右クリック」= b 種別の中段住所)。汎用 context-menu 部品(MB-0 の overlay 機構を右クリックトリガーへ転用可能)+ Layer 行/Timeline bar への配線。Ableton 先例(クリップ右クリックで quantize・色割当)が意味の型 | timeline-pane(bar/lane_bar への on_press_right 相当)+ MB-0 部品の再利用 | 高(初の一般化 — 現状は cancel 専用2箇所のみで一般 API が無い) | MB-0 完了後(部品を共有するため) |

4切片は合計で「基盤1+機能2+基盤2」の構成になり、書き込み対象(header vs inspector-pane vs timeline-pane)が概ね素になっている。MB-2 のみ inspector-pane と timeline-pane の両方に触れるため、他切片との並走時は inspector-pane を触る他レーン(I-tokens)の完了を待つのが安全(write-set 衝突回避)。

---

## §6 EVIDENCE_GAP

1. **rfd 裁定との絡み(最重要)**: `next/` には rfd 依存も New/Save 系 Message も存在しない(grep 実測ゼロ件)。旧 `crates/` 世界の P06-C1-MAC(rfd 0.17.2 probe 済み)・P12-C1(project lifecycle 決定)は **next/ reset 以前の資産で、next/ へ移植されたか未確認**。File>New Project/Save As/Save a Copy をメニューに実配線する MB-1 切片は、この裁定の再確認(または新規裁定)を待つ必要がある。
2. **iced_aw の fork 互換性未検証**: §3.2 の `[patch]` リダイレクトが実際に効くか、`iced_core`/`iced_widget` の API 面が host-seams fork(= upstream master ドリフト0)と一致するかは cargo build を要し、本レーン(read-only)では検証不能。
3. **macOS システム統合の最小要件未確認**: アプリ内メニューバー採用時でも、Cmd+Q(終了)・Dock メニュー等 macOS が期待する挙動を winit 単体でどこまで満たせるか(現状 `Quit / Exit` の入口自体が無い=OS ウィンドウクローズのみ)は未調査。
4. **トップレベル分割(Marker を独立させるか)の先例が割れている**: AE=無・Premiere/Resolve=有。S0 はエントリ種別の分布を裁く軸であり、トップレベルの数・分割自体を裁く材料が無い。利用者裁定が必要。
5. **走行中レーン MK2/H2/TP の実在未確認**: 発注時の背景情報に名前が挙がっていたが、`docs/reviews/2026-08-21-lane-board.md`(2026-08-21時点の走行中表: I-tokens/TL-P1/M4のみ、M4は既に検収合格・merge済み)にも `docs/reviews/2026-08-21-backend-gap-seam-survey.md`(MK2 は未発注の設計候補行として記載)にも「走行中」の実体が見当たらなかった。呼び出し元セッションでの最新レーンボード確認を推奨(本レーンはこの1点を除き衝突なしと判断)。
6. **Zoom In/Out の現状入口ゼロ**: κ 台帳・現行 grep 双方で timeline-pane に zoom 操作の Message/wheel handling が見当たらない。MB-2 でメニューと同時に最低1つの非メニュー入口(±キー等)を新設しないと S6 違反になる。
7. **Label Color 手動指定の設計未着手**: 現状は `label_color_for_new_layer`(id%12)の自動割当のみで、ユーザーが変更する Message が存在しない。ρ レーン(差し色第一波)は自動割当+継承のみを対象にしており、手動変更は範囲外だった。MB-2 でメニュー項目を出す場合、Inspector スウォッチか右クリック(MB-3)のどちらかを同時に用意する必要がある。
