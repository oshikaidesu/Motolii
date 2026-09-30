# UI の Technology Acquisition Audit（2026-09-30）

状態: **観察**（置換はしない。各粒でゲートを通してから）。[Technology Acquisition Gate](../known-implementation-adoption-model.md) と [全体のMap](2026-09-30-technology-acquisition-audit.md) の続き。

問い: **Flutter / Flutter Desktop / 成熟した Flutter ecosystem に既にある一般UI技術を、Motolii はどれだけ再実装しているか。**
順序は Flutter 標準 → Flutter Desktop 公式 → 広く使われ保守されている package → Motolii 自作。

## 0. 先に結論（予想と違った点）

1. **コントロールの密度は、すでにデスクトップのプロツール並み。「UI が大きい」の主因は標準widgetの既定値ではない。**
   実窓（Retina 2750×1642 = 論理 1375×821）の実測で、数値欄の行ピッチ約 20.7 px、Timeline の行ピッチ約 18.3 px、文字 10〜11 px（コードの `UiMetrics`: control 18、行 20 / 18、menu 22、chrome 24、top bar 32）。
   大きく見えるのは、製品固有の大きな要素（Create のタイルは約 48 px、Transform のギズモ、Ease のグラフ）と、全体の余白・情報の並べ方のほうに見える。
   参考（一般に知られた値。今回は未計測）: Blender の標準 widget 高は約 20 px、VS Code の list 行は 22 px。
2. **標準widgetは、Theme だけでデスクトップ密度になる（自作widgetは要らない）。** 実測（`explorer/test/density_compact_theme_test.dart`）:

   | widget | Material 既定 | compact Theme のみ |
   |---|---|---|
   | TextField | 48 | **20** |
   | DropdownMenu | 56 | **20** |
   | FilledButton / TextButton | 48 | **20** |
   | Slider | 48 | **20** |
   | ListTile | 64 | **20** |
   | Checkbox | 48 | **24**（そのwidgetの下限） |
   | SegmentedButton | 56 | **24** |
   | Switch | 48 | 40（下限。小さくするなら Checkbox 系か別の型） |

   落とし穴: `VisualDensity.compact` は `minimumSize` も引く（20 を指定しても 12 になる）。ボタンと入力欄は density を標準のまま**明示の最小高**で、Checkbox や SegmentedButton は `VisualDensity(-4, -4)` で下限へ。
   見た目（角丸のピル型ボタン、紫の色、Material のチェック）は既定のまま出る。Motolii の見た目は Theme（ButtonStyle の shape と ColorScheme）で作る仕事で、widget の自作ではない。
3. **入口の前提が標準widgetを阻んでいる。** 本番を含む全アプリ（`live_hf/main.dart` ほか）が `WidgetsApp` で、`MaterialApp` は1つも無い。Material の TextField・DropdownMenu・Slider は
   `MaterialLocalizations` と `Material` の祖先を要求するので、そのままでは置けない（比較用の fixture でも最初はここで失敗した）。標準へ戻す最初の一歩は、shell の入口に Material の土台を足すこと。
4. **Media Browser の一覧は、Flutter の遅延構築でそのまま解ける。** 同じ条件のヘッドレス計測:

   | 件数 | FluidBoard（今） | `GridView.builder` |
   |---|---|---|
   | 200 | 初回 325 ms、widget 453 | 初回 65 ms、構築 130 |
   | 1,845 | 初回 **1,158 ms**、widget 4,167 | 初回 **21 ms**、構築 130 |

   標準の遅延構築は、構築する数が件数に依存しない。FluidBoard を速くするのではなく、Flutter が解いた問題を Motolii が再び解いていた。
   注意: `GridView.builder` は一様な格子。masonry と view 間の滑らかな移動（glide）には、SDK の `SliverGridLayout`（位置の配列を返す）か `flutter_staggered_grid_view`、見える範囲だけの glide が要る。

## 1. Capability Map（UI全体）

分類: FLUTTER（標準へ委ねられる）／BORROW（成熟した技術へ）／WRAP（既存へMotolii意味を薄く接続）／MOTOLII（本当に所有する）／SUSPECT（一般UI技術を自作している疑い）／REINVENTED（明確な再発明）。
根拠は6件の read-only 調査（repo 読解＋外部検索）。各調査の「未確認」は §7 に引き継ぐ。`確認` は私が独立にコードで確かめた事実。

| Capability | Current Motolii implementation | Flutter built-in | Major Flutter/Desktop technology | Motolii semantics | Class |
|---|---|---|---|---|---|
| App shell | `live_hf/main.dart` は素の `WidgetsApp`（確認: 全アプリが `WidgetsApp`）。Classic / New / proto の shell が並存（`app/` 約1,000行） | `WidgetsApp`、`MaterialApp` | — | どのsessionを載せるか | FLUTTER。shell が複数あるのは SUSPECT（退役ゲート） |
| Workspace / Dock / Split / Tabs | live は `docking` 1.16.2 を包む `dock_workspace.dart` 336行。旧: `workspace/layout.dart` 236 + `workspace_view.dart` 394（確認: 使うのは Classic の `editor_window.dart` だけ） | なし | `docking`（内部で `multi_split_view`・`tabbed_view`） | パネル id と registry | BORROW + WRAP。旧実装は REINVENTED（live からは外れている） |
| Seat（tab 帯を消して自前で描く） | `dock_workspace.dart:77-195`。`docking` の `DraggableData` 内部へ手を入れる | `Draggable` / `DragTarget` | — | 「前面の tab だけ言葉を持つ」という座席の意味 | WRAP。`docking` 内部への結合は SUSPECT |
| Panel placement / persistence | `dock_workspace.dart:278-318`（`docking` の文字列を版と「前面」で包む）、350ms の debounce 保存 | なし | `docking` の layout 文字列 | どのパネルが在り前面か、Reset | WRAP |
| Detached（別窓）パネル | Swift `PanelFlutterWindow` + `ProbeSession` 経由（`MainFlutterWindow.swift` 1,129行）。窓ごとに `FlutterViewController` | Flutter 公式の Desktop Windowing API は実験的（main channel のみ、`@internal`、feature flag 要） | `desktop_multi_window` 0.3.1（macOS可、窓ごとにengine）、`window_manager`（nativeapi-flutter へ移行中） | 窓 id とパネル名、閉じたら元の隣へ戻る、document session は全窓で共通 | WRAP。手作りの仕組みは REINVENTED だが、公式が不安定なので即置換はしない |
| Stage container | `panels/stage.dart` 117 + `stage/window.dart` 182 | `Visibility`、`TickerMode` | — | 絵とは何か、観測者・カメラの view | MOTOLII |
| Timeline（NLE） | `timeline_core/` 1,925行、`hf/shell/timeline*.dart` | なし | なし | clip・grip・trim・snap・scrub-zoom・ruler の時間写像 | MOTOLII |
| Timeline rows / 仮想化 / scroll | 行を手で配置・描画・hit test。wheel handler が3か所（`timeline_core/view.dart:105`、`live_hf/adapters/timeline.dart:220`、`hf/shell/timeline.dart:541`） | `Scrollable`、`ListView.builder`、`SliverFixedExtentList` | `two_dimensional_scrollables` の `TableView`（flutter.dev、遅延構築、固定行・列） | どの行が在り何を編集するか | SUSPECT |
| Timeline geometry（hf版） | 画面座標の定数（確認: `hf/shell/timeline.dart:23-25` の `tlTop = 775`、`tlX0 = 559`、`tlRight = 1521`、`timeline_paint.dart` にも直書き） | `LayoutBuilder`、`Flex` | — | なし | **REINVENTED**（モックを定数に固めた物） |
| Playhead / Ruler | 手描き、手 hit test | `CustomPaint`、`Listener` | — | frame 厳密な scrub、時間→x | MOTOLII |
| Inspector / Property rows | `hf/insp/` 3,274 + `panels/inspector*` 2,762（Classic と hf の二重）+ `app/new/inspector` | `ListView.builder` | — | 各行が何を編集するか | MOTOLII（意味）。3実装の重複は SUSPECT |
| Numeric controls | `panel_controls/numeric.dart` 662 + `hf/insp/toys.dart` 547（ほぼ同じ「ドラッグで値を変える欄」が2つ） | なし（DCC 流儀の部品は標準に無い） | 成熟した package なし | preview → commit → cancel → undo の区切り（`EditorDragSession`） | MOTOLII（ただし2つ在るのは SUSPECT） |
| Sliders | `EditorSlider`（`leaves/track.dart:90-288`） | `Slider` + `SliderTheme` | — | preview/commit の経路 | REINVENTED（Theme で 20 px に出来る） |
| Text fields / Text editing | `EditableText` を直接（15か所）、`EditorTextField`（`EditableText` を包む） | `TextField`、`EditableText`、`ContextMenuController` | — | なし | WRAP（概ね良い）。独自の右クリック menu は SUSPECT |
| Dropdowns | `EditorMenuAnchor`、`choice.dart`、`timeline_core/menu.dart`、`showHfMenu` | `DropdownMenu`、`MenuAnchor`、`RawMenuAnchor` | — | 項目の中身 | REINVENTED（実装が3〜4つ） |
| Buttons / toggles | `EditorButton`、`Hf*`、top bar の `_Key` の3系統 | `TextButton`、`FilledButton`、`SegmentedButton`、`Checkbox`、`Switch` | — | 操作の意味 | REINVENTED（Theme で 20 px に出来る） |
| Main menu | Dart に `PlatformMenuBar` も `MenuBar` も無い（確認）。`MainMenu.xib` に112項目、窓内の top bar は自前 | `PlatformMenuBar`（macOS のネイティブ menu、shortcut もOS） | — | command id | SUSPECT |
| Context menus | `showHfMenu`（`hf/shell/menu.dart` 134行。呼び出し8か所）＝自前の `OverlayEntry`、矢印、hover、端の clamp、focus 復帰。`foundation` にもう1つ（`floating.dart:48-347`、外から使う箇所0＝確認） | `RawMenuAnchor`、`ContextMenuController`、`MenuAnchor` | — | 項目の中身 | REINVENTED（二重） |
| Dialogs | `showHfDialog`（生の `OverlayEntry`）と `showEditorDialog`（`showGeneralDialog` を包む） | `showGeneralDialog` | — | なし | `showEditorDialog` は WRAP、`showHfDialog` は REINVENTED |
| Tooltip | `EditorTooltip` が `RawTooltip` を包む | `RawTooltip`、`Tooltip` | — | 見た目 | WRAP（良い手本） |
| Overlay / popover | `OverlayEntry` 6か所、`EditorMenuAnchor` は `OverlayPortal` | `OverlayPortal`、`CompositedTransformFollower` | — | 見た目 | SUSPECT（2流儀） |
| Scrollbars | `EditorScrollbar` が `RawScrollbar` を包む | `RawScrollbar`、`ScrollbarTheme` | — | 見た目 | WRAP（良い手本） |
| Resize handles | dock は `multi_split_view`（`docking` 経由）。手作りは `workspace_view.dart`（旧）、`timeline.dart:511`、`browser/parts.dart:125` | `MouseRegion`、`Draggable` | `multi_split_view` | Timeline のトラック分割は意味を持つ | dock は BORROW。手作りは SUSPECT |
| Keyboard / Shortcuts | `Focus.onKeyEvent` の if 連鎖が41か所（確認）、`input/editor_shortcuts.dart` 176 + `live_hf/keys.dart` 161 が同じ作り、「入力中」判定も二重。`Shortcuts` / `Actions` / `CallbackShortcuts` は11か所のみ | `Shortcuts`、`Actions`、`Intent`、`CallbackShortcuts`、`DefaultTextEditingShortcuts` | — | キー→操作の写像 | REINVENTED |
| Focus | `Focus(` 約108か所、`FocusTraversalGroup` は0 | `FocusScope`、`FocusTraversalGroup` | — | 前面のパネル | SUSPECT |
| Drag & Drop（アプリ内） | `Draggable` / `DragTarget` | 同左 | — | 落とした時の意味 | FLUTTER |
| File drop / File dialogs | Swift の `NSOpenPanel` / `NSSavePanel` 直接（`MainFlutterWindow.swift:823-942`）。ファイルの drop も Swift 経由 | なし | `file_selector`（flutter.dev、macOS可）、`desktop_drop`、`super_drag_and_drop` | `rrd` / `mp4` の種類、「import は folder を取る」 | BORROW 候補。今は自前 |
| Media Browser（一覧） | `FluidBoard` は全 face を1つの `Stack` の widget に（1,845件で初回 1.16 秒）、`MediaLibraryBody` も全 tile。`things.indexOf(t)` が O(n²) | `CustomScrollView`、`SliverGrid`（+`SliverGridLayout`）、`GridView.builder`、`ListView.builder` | `flutter_staggered_grid_view`、`two_dimensional_scrollables` | face の identity が view 間で続く、選択 ring、置く操作 | **REINVENTED**（実害あり） |
| Media list | `media_list.dart` は `ListView.builder` + `itemExtent` | 同左 | — | 列の内容、並べ替えヘッダ | FLUTTER（良い手本） |
| Search | 検索欄が5つ（`hf/bp/search.dart`、`app/new/primitives.dart`、`catalog_media.dart`、`frame_bars.dart`、`common.dart`）。`/`・Cmd+F の手書き処理 | `SearchAnchor`、`TextField`、`Shortcuts` | — | token AND の絞り込み | WRAP（欄）。5つは REINVENTED |
| Tree / hierarchy | Browser の folder tree と Timeline の layer tree を手で | `TreeSliver`、`two_dimensional_scrollables` の `TreeView` | — | layer / track の意味 | SUSPECT（Timeline 側は未確認） |
| Selection / Multi-selection | `picked` / `_anchor` / Cmd・Shift の規則が3か所（`media_browser.dart`、`shelf_seat.dart`、`frame_keys.dart`）。矢印での格子移動も同じ論理が重複 | 項目選択の標準は無い（`SelectableRegion` は文字のみ） | 成熟した package なし | 「選んだもの」の意味 | REINVENTED（3重） |
| Clipboard | `Clipboard.setData` 3か所 | `Clipboard` | — | 何をコピーするか | FLUTTER |
| Accessibility | `Semantics(` 20か所。`Listener(` 262・`Focus(` 108 に対し少ない。`hf/` には0 | `Semantics` | — | test が semantics label で操作を探す | SUSPECT |
| Design tokens / Theme | `UiMetrics`・`Dn`（`hf/metrics.dart`）と `EditorMetrics`（`foundation/metrics.dart`）と `H` / `N`（静的定数）と、文字列キーの `EditorTheme`（`ThemeExtension`）が並存。行高が18/20/22、control が18と24 | `ThemeData`、`ThemeExtension`、`VisualDensity`、`TextTheme` | — | 作品が言語になる identity（6つの操作族の色など） | MOTOLII（値）。配管は REINVENTED、二重 |
| Icons | `hf/glyphs.dart` 192行（手描き50+）、`foundation/glyphs.dart` 395行（`IconData` 表の複製） | `Icon`、`Icons` | `flutter_svg` | 操作族（scatter / stagger / along / attach 等）の図形は identity | 族の図形は MOTOLII。汎用（矢印・検索・再生・目・鍵）は REINVENTED |
| 座標配置（hf） | `hf/shell/place.dart:59-165`: 1536×1024 の参照座標で置く `RF`・`Rc`・`Ln`・`Pt`・`Wd`・`Tx`。`Tx` は文字を82%へ縮める | `Stack`、`CustomMultiChildLayout`、`LayoutBuilder` | — | なし（スクリーンショットの再現用の定規） | **REINVENTED** |

## 2. hf/ を特に疑う（既存設計だから守らない）

`hf/`（約12k行）と `foundation/`（約5.6k行）に見つかった「UI framework」的な物。それぞれ **なぜ Flutter のものでは足りないのか**:

| 見つかった物 | Flutterで足りない理由はあるか |
|---|---|
| 座標配置エンジン `place.dart`（参照座標へ置く） | **無い**。`Stack` / `LayoutBuilder` で足りる。窓の大きさにも密度にも追従できない、画像再現用の定規 |
| メニュー2系統（`showHfMenu`、`EditorMenuSheet` 系） | **無い**。`RawMenuAnchor` / `ContextMenuController` が overlay 位置・矢印・focus 復帰を持つ。見た目は Theme / skin で出す |
| ダイアログ2系統 | **無い**。`showGeneralDialog` |
| ボタン・スライダ・切替・セグメント・fold・card の3系統 | **無い**。Theme で 20〜24 px（§0-2）。`KeyLamp`（keyed / now / draft の灯）と anchor グリッドだけは Motolii の意味 |
| キー処理の連鎖 | **無い**。`Shortcuts` / `Actions` / `Intent` は中央の keymap・再割当・発見性を持つ。今は誰も keymap を一覧できない |
| 選択の規則×3、矢印格子移動×2 | **無い**。共有の選択モデル（Motolii の「選んだもの」の意味）1つに寄せる |
| Timeline の座標定数（hf版） | **無い**。`LayoutBuilder` |
| 二重の token（`H`/`N` と `EditorTheme`） | **無い**。型付きの `ThemeExtension` 1つ |
| **Motolii が持つ物**: ドラッグで値を変える欄の preview→commit→cancel→undo（`EditorDragSession`）、`KeyLamp`、操作族の図形と色、Timeline の時間意味 | これらは外へ出さない |

## 3. 密度の観察（実窓）

本番 shell を実際に起動して撮影（Retina 2750×1642、論理 1375×821）。`hf/metrics.dart` の値と画面の実測は一致した。

| 領域 | 観察（論理 px） |
|---|---|
| Timeline | 行ピッチ約 18.3、文字 11。`UiMetrics.rowStd = 20`、`rowTight = 18` |
| Inspector の数値欄 | 行ピッチ約 20.7、欄の高さ 18（`control`）、ラベル 10 |
| Browser（Create） | タイル約 48 px（大きい。製品固有の「おもちゃ箱」）。行の文字 10〜11 |
| Menu | 行 22（`menuRow`） |
| Top bar / Chrome | 32 / 24 |

欄ごとの幅・余白の比較対象（一般に知られた値。今回は未計測）: Blender 標準 widget 約 20 px、VS Code の list 行 22 px。Motolii の control 18〜20 px はそれより密。
**数値を小さくすること自体は目的ではない**。今ある余地は、(1) タイルやギズモなど大きい要素の大きさ、(2) 余白（`pad = 9`）、(3) 情報の出し方、のほうにある。Material 既定の大きさが原因ではない（既に使っていない）。

## 4. Phase 4: compact Motolii を Theme で作れるか

**できる（§0-2 の表）。** fixture: `explorer/lib/stories/density.dart`（Motolii 現行 / Material 既定 / compact Theme の3列。`explorer/test/density_compact_theme_test.dart` が「既定は40 px以上、compact は 20〜24 px」を固定）。
見た目の判定は二次。確認できたのは、標準widgetのままで 20 px の行が出ること、文字は 11 px で読めること。
残る課題: Material の見た目（ピル型ボタン、Material のチェック）を Motolii の見た目へ寄せる（Theme の `shape`・`ColorScheme`）、Switch の下限（40 px）、`MaterialApp` の土台（§0-3）。

## 5. Phase 5: Media Browser

§0-4 の計測のとおり。標準の遅延構築（`GridView.builder` / `SliverGrid`）は、1,845件で初回21 ms、構築数130で件数に依存しない。
FluidBoard の責務を分けると、masonry の配置 → `SliverGridLayout` か `flutter_staggered_grid_view`（**BORROW**）、list → `ListView.builder`（**FLUTTER**、既に `media_list.dart` が手本）、
scroll と scrollbar → 標準、drag → 既に `Draggable`、選択 ring と hover scrub → **MOTOLII**、view 間の glide → **MOTOLII**（ただし見える範囲だけ）。
**本物の衝突**: glide（全 face が1つの Stack に在って位置が補間される）と遅延構築（見える物しか作らない）。標準的な折衷は「glide は旧または新の矩形が viewport に入る face だけ、他は snap」。
Explore は pan / zoom の canvas なので sliver では仮想化できず、矩形の viewport culling になる。

## 6. 最終成果物

**A. KEEP MOTOLII（UIの意味）**: Stage container（絵・観測者・カメラ）、Timeline の NLE 意味（grip・trim・snap・scrub-zoom・ruler の時間写像・playhead）、Inspector の「何を編集するか」、
ドラッグで値を変える欄の preview→commit→cancel→undo の区切り、`KeyLamp`、パネルの identity と「隣へ開く」配置の意味、Browser の発見と Place の意味（face identity の連続、選ぶ・置く）、操作族の図形と色（visual identity）、
file の種類と「import は folder を取る」。

**B. RETURN TO FLUTTER（標準へ戻す）**: ボタン・Slider・Checkbox・Switch・SegmentedButton・TextField・DropdownMenu・ListTile（Theme で密度）、メニューと右クリック menu（`RawMenuAnchor` / `ContextMenuController`）、
ダイアログ（`showGeneralDialog`）、Tooltip と Scrollbar（既に手本）、キー処理（`Shortcuts` / `Actions` / `Intent`）、focus（`FocusTraversalGroup`）、座標配置（`Stack` / `LayoutBuilder`）、Timeline の座標定数。

**C. BORROW（成熟した技術）**: Dock（`docking`。既に）、ファイル dialog と drop（`file_selector`・`desktop_drop`）、Media Browser の masonry（`flutter_staggered_grid_view` か `SliverGridLayout`）、
Timeline の行の scroll と仮想化（`two_dimensional_scrollables` の `TableView`）、tree（`TreeSliver`）、別窓（`desktop_multi_window`。公式 Windowing API が安定するまで狭い interface の後ろで待つ）、macOS のメニューバー（`PlatformMenuBar`）。

**D. DELETE / REPLACE**: hf の座標配置 `place.dart`（と、それに乗る画面座標の定数）、メニュー・ダイアログ・ボタン類の重複系統、旧 dock（`workspace/layout.dart`・`workspace_view.dart`。Classic の `editor_window.dart` だけが使う＝Classic の退役ゲートと同時）、
検索欄5つ → 1つ、選択規則3つ → 1つ、数値欄2つ → 1つ、token の二重、汎用アイコンの手描き、Media Browser の全 face 構築。

**E. DENSITY**: 標準widget + Theme だけで、TextField・DropdownMenu・Button・Slider・ListTile は 20 px、Checkbox・SegmentedButton は 24 px（下限）。Switch は 40 px（下限。別の型か Checkbox 系）。
ただし現行の Motolii は既に 18〜20 px で、広げる余地があるのは大きな要素と余白。

**F. OPEN QUESTIONS（実窓または prototype なしでは判断できない）**: 標準widget の Theme が Motolii の見た目（形・色・状態）を出せるか（特に密な行の hover / focus / keyed の表現）、
`MenuAnchor` が暗い高密度の skin に耐えるか、`TableView` が Timeline の clip 描画と hit test を支えるか、`flutter_staggered_grid_view`（v0.7.0、約3年前）の保守と、複数 sliver での scroll jump、
`desktop_multi_window` と `Visibility`・native surface の組み合わせ、`PlatformMenuBar` と既存 `MainMenu.xib` の置換、`MaterialApp` を足した時の副作用、Classic / New / proto shell の退役時期、アクセシビリティの現状（semantics label に頼る test が増えている）。

**今日ゼロから作るなら、Motolii固有コードとして書くのは次の10個だけ**
1. Stage の container（絵・観測者・カメラ・窓の要求）
2. Timeline の clip・grip・trim・snap・scrub-zoom・ruler（時間の意味と描画）
3. Inspector の行が何を編集するか（Selection → property の写像）
4. ドラッグで値を変える欄の preview → commit → cancel → undo の区切り
5. `KeyLamp`（keyed / now / draft）と、操作族の図形と色（Motolii の identity）
6. パネルの identity・「隣へ開く」配置の意味・Reset（dock 自体は `docking`）
7. Browser の発見と Place の意味（face identity の連続、選ぶ・置く・Favorites / Recent）
8. Explore の近さの意味（similarity の重み、地図の記憶、選択で動かない契約）
9. 共有の選択モデル（「選んだもの」の意味を1つ）と、キー→操作の対応表（`Shortcuts` / `Actions` に載せる）
10. Theme 1つ（`ThemeData` + `ThemeExtension`）: 密度・色・形・文字の設定。widget の自作ではない

## 7. 未確認（引き継ぎ）

- 6件の調査の未読部分: `hf/bp/*`・`hf/desk/*`・`hf/insp/*` の本文の大半、`EditorScrollbar` の残り、`panel_catalog.dart`、`EditorScale`、`persist.rs` 相当のUI側、Timeline の layer tree、`MainMenu.xib` の配線。
- 外部: package の最終release日・ライセンス・保守状況の多くは検索の要約のみ（`docking`・`file_selector`・`two_dimensional_scrollables`・`desktop_multi_window` は公式ページで確認、`flutter_staggered_grid_view` は v0.7.0 約3年前）。`macos_ui`・`fluent_ui`・`bitsdojo_window` は未評価。
- Flutter の Desktop Windowing API の現状は二次情報（記事）のみで、Flutter のソースは見ていない。
- 性能はヘッドレス計測（実窓の 1.2 秒フレームと整合）。実窓での再計測は未了。
- 「1画面に表示できる項目数」「AE / Ableton / Resolve の密度」は未計測。

## 出典

[Flutter VisualDensity](https://api.flutter.dev/flutter/material/VisualDensity-class.html)、[MenuAnchor](https://api.flutter.dev/flutter/material/MenuAnchor-class.html)、[PlatformMenuBar](https://api.flutter.dev/flutter/widgets/PlatformMenuBar-class.html)、
[ContextMenuController](https://api.flutter.dev/flutter/widgets/ContextMenuController-class.html)、[docking](https://pub.dev/packages/docking)、[desktop_multi_window](https://pub.dev/packages/desktop_multi_window)、[window_manager](https://pub.dev/packages/window_manager)、
[file_selector](https://pub.dev/packages/file_selector)、[desktop_drop](https://pub.dev/packages/desktop_drop)、[super_drag_and_drop](https://pub.dev/packages/super_drag_and_drop)、
[two_dimensional_scrollables](https://pub.dev/packages/two_dimensional_scrollables)、[flutter_staggered_grid_view](https://pub.dev/packages/flutter_staggered_grid_view)、[SliverMasonryGrid](https://pub.dev/documentation/flutter_staggered_grid_view/latest/flutter_staggered_grid_view/SliverMasonryGrid-class.html)、
[Flutter Desktop windowing（二次情報）](https://flutterwire.com/flutter-desktop-windowing-api/)。
