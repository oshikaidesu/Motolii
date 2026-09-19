# Flutter desktop の widget kit — Motolii が借りられる物と縫い目(2026-09-19)

調査のみ。実装は無し。対象は `motolii/ui/lib/foundation/{theme,metrics,panel_controls,color_field}.dart` と、panels 21 file が呼ぶ Material の跡(InkWell 25 / Tooltip 58 / IconButton 16 / TextButton 14 / Switch 10 / TextField 9 / MenuAnchor 5 / Slider 4 / Scrollbar 3 / showDialog 1 / DropdownButton 1、`grep` 2026-09-19)。手元の Flutter は 3.47.2(`.tools/flutter`、2026-08-26)。

## 要約(5 行)

1. 2026 年に生きている desktop 向け kit は 4 つ — shadcn_flutter 0.0.54、forui 0.26.0、fluent_ui 4.16.1、macos_ui 2.2.x。yaru は Linux/GNOME 専用、flutter_platform_widgets は公式に打ち切り。
2. どの kit も Motolii の芯(drag→preview→commit の契約、`DocumentSlice` の購読、CustomPainter 32 箇所、数値の文法)は持っていない。AE 型の「数字を掴んで擦る」field は fluent_ui の NumberBox も矢印・ホイールまでで、擦りは無い。
3. 先例 2 つはどちらも kit を使っていない。Lumit は「Material の chrome は避ける、葉は自前、`WidgetsApp` の土台だけ借りる」と文書で宣言し、Rive も自前(公開されているのは「custom pieces を足すのが楽」という証言だけ)。
4. 借りて得をする層は「見た目の葉」— 押す・切り替える・覗く(button / switch / tooltip / popover / context menu / resizable)。ここを kit に置き換えると触る file は 20 前後、得るのは focus ring・キー操作・Material 消しの手間の削減。
5. 推し: **kit は入れず、Flutter 3.47 の `material_ui` 分離に乗って `flutter/widgets` + 自前の葉へ寄せる(Lumit と同じ型)。** 1 つだけ借りるなら shadcn_flutter(widgets.dart のみ依存、Resizable/Tree/Menubar/ContextMenu/Scrollbar/ColorPicker が揃う、BSD-3)。ただし 0.0.x で毎版 breaking、Motolii の `EditorMetrics` 24/20/11 px 密度に合わせる縫い目は theme 1 file では済まない。

## kit の表

一次資料は pub.dev の package 頁・changelog・repo(取得日 2026-09-19)。「活動」は GitHub の pushedAt。

| kit | 版 / 日付 | license | 依存の芯 | desktop | 活動 | 部品の有無(Motolii の欲しい物) | token / 密度 |
|---|---|---|---|---|---|---|---|
| **shadcn_flutter** | 0.0.54(約 22 日前 = 2026-08 末)| BSD-3 | `flutter/widgets` のみ(0.0.54 で Material/Cupertino を切り離し、`shadcn_flutter_material` で相互運用)。dep は data_widget・animation_kit | 6 platform 全部 | push 2026-09-06、★944、likes 469、160 pt、20.7k/週 | 数値 field: **無し**(NumberInput は 0.0.45 で deprecated → TextField + InputSpinnerFeature)。Slider ○ Switch ○ Tabs ○ Menubar ○ ContextMenu ○ Popover ○ Tooltip ○ Scrollbar ○ **Resizable ○** **Tree ○** ColorPicker ○ | `ThemeData(colorScheme, typography, radius, scaling, iconTheme, surfaceOpacity/Blur)`。`scaling` は 1 本の倍率で、行高 20 / control 24 / font 11 のような個別の値は component ごとの theme を書く。Flutter 3.47 必須 |
| **forui** | 0.26.0(約 25 日前)| MIT + OFL | **material_ui ^1.0 と cupertino_ui ^1.0 に依存**(0.26 で移行)| 6 platform 全部 | push 2026-09-18、★2350、likes 436、160 pt、27.6k/週 | 数値 field 無し。Slider ○ Switch ○ Tabs ○ ContextMenu ○(0.23〜) PopoverMenu ○ Tooltip ○ **Resizable ○** TextField ○ Select ○。**Tree 無し・Scrollbar 無し・ColorPicker 無し** | `FThemeData(colors, typography, style, icons, breakpoints)` + widget ごとの `FXxxStyle.inherit(...)` と `copyWith(delta)`。`ThemeExtension` も受ける。CLI で style を生成して手で書き換える型(`dart run forui style create`)。触覚・motion 低減の OS 追従あり |
| **fluent_ui** | 4.16.1(約 46 日前)| BSD-3 | material_ui ^1.2(FluentApp 内で Material 併用のため)、intl、math_expressions(NumberBox)| 6 platform 全部、見た目は WinUI 3 | push 2026-09-14、★3470、likes 3.2k、150 pt、23.9k/週 | **NumberBox ○**(矢印 ↑↓ / PageUp/Down / ホイール / 式評価 / min-max clamp / `parser`。**擦り無し**、`onChanged` は focus loss か button)。Slider ○ ToggleSwitch ○ TabView ○ MenuBar/MenuFlyout/ContextMenu ○ Tooltip ○ Scrollbar ○ **TreeView ○**(4.15 で controller) ColorPicker ○。**split pane 無し**(NavigationView は sidebar) | `FluentThemeData` + `VisualDensity`(4.15 で compact 追加)。accent は system_theme 連携。28 言語 |
| **macos_ui** | 2.2.2(約 11 か月前、dev 枝は 2.2.3)| MIT | Material 不要。macos_window_utils、appkit_ui_element_colors | macOS 専用(他 OS は「保証しない」)| push 2026-08-22、★2137、likes 1.07k、150 pt、32.9k/週 | 数値 field 無し。MacosSlider ○ MacosSwitch ○ MacosTabView ○ Pulldown/PopupButton ○ Tooltip ○ MacosScrollbar ○ **ResizablePane ○** Sidebar ○ **MacosColorWell ○**(AppKit の panel) TreeView 無し | `MacosThemeData` 光/闇。密度の口は無い(AppKit の寸法固定)。Flutter 3.35+ |
| yaru | 10.2.0(2026-06-03)| MPL-2.0(icon は CC-BY-SA)| Material の theme(dbus/gsettings/gtk 依存)| **Linux のみ**表記 | push 2026-07-22、★395、likes 288、140 pt | GNOME の見た目。Motolii の欲しい物は無い | Material の ThemeData を Yaru 色で作る |
| flutter_platform_widgets | 10.0.1(約 8 か月前)| MIT | Material/Cupertino の adaptive wrapper | 6 platform | **打ち切り**(「Material/Cupertino の分離で今後の保守無し」と pub.dev に明記)| — | — |
| desktop_ui_kit | 0.8.0(約 55 日前)| MIT | Flutter SDK のみ | Win/mac/Linux | likes 0、53/週、未検証 publisher | Button・TreeView・**SplitPanel・DockPanel**・MenuBar・CommandPalette。数値 field 無し | 光/闇/高コントラスト。若すぎて借り先にならない(参考のみ)|
| multi_split_view | 3.6.2(約 3 か月前、2026-05-24)| MIT | flutter + meta | 6 platform | ★191、likes 355、160 pt、38.7k/週 | 分割だけ(flex / size、divider の push)| 自前 |
| docking | 1.16.2(6 か月前、1.17 rc)| MIT | multi_split_view 2.4 + tabbed_view 1.18 に pin | 6 platform | likes 75、2.09k/週 | split + tab の dock 木 | 自前(依存の pin が古い)|
| dock_panel | 0.1.1(2 か月前)| BSD-3 | **flutter_riverpod 必須** | 6 platform | likes 3、29/週 | IDE 型 dock、layout 永続化 | riverpod を抱えるので不採用 |

補足:

- Flutter 3.47(2026-08-12)で `material_ui` / `cupertino_ui` が SDK 外の独立 package になった。旧 `package:flutter/material.dart` は今動くが、秋(11 月想定)の stable で正式 deprecation。Motolii の `foundation/theme.dart` は `flutter/material.dart` を import しているので、どの道 1 度は触る。kit を入れないなら `dart fix --apply --code=migrate_design_widgets` で済む。
- Flokk(gskinner)は 2023-07 で止まっている。Superlist は Flutter の showcase 頁に技術の記述が無く、widget 層は非公開。

## 先例(Lumit・Rive)の作り

### Lumit(github.com/luminalmvm/Lumit、GPLv3、push 2026-09-16、★29)

`flutter_ui/pubspec.yaml`(v0.5.0+1、SDK ≥3.6):UI の kit は **0 個**。dependencies は flutter_localizations / intl / crypto / cryptography / clock / ffi / flutter_svg / file_selector / desktop_drop / flutter_rust_bridge 2.12.0 / provider / freezed_annotation / uuid / syntax_highlight。font は Hanken Grotesk + Geist Mono(+ Desk 様式用に IBM Plex)。

`lib/` の構成(file 数):panels 111 / src 44 / state 40 / shell 39 / widgets 29 / l10n 14 / theme 6 / icons 5。`widgets/controls/` に base・buttons・dropdowns・indicators・menus・modal_window・popups・slider・text_field・value_arithmetic・value_field と、`widgets/angle_dial.dart`、`shell/dock_widget.dart`(`flutter/widgets` のみ import、自前の dock 木)。

`docs/archive/flutter-port/04-WIDGET-MAP.md` が方針を 1 行で言っている:「Material の chrome は避ける — 独自の metrics・splash・motion が house design(12 px 本文、16 px の操作高、枠無しの idle、ripple 無し)と喧嘩する。`WidgetsApp` 級の土台(focus・overlay・navigation)だけ使い、葉の widget は自前」。Tooltip も「Flutter の Tooltip は全体 off にできない」と自前。`MenuAnchor` だけは「見た目と喧嘩しない所で下に使う」。

`widgets/controls/value_field.dart` は Motolii の `EditorNumericField` と同じ役:横 drag で擦る、click で入力、範囲 clamp、speed。加えて **修飾子の梯子** `[10, 1, 0.1, 0.01]`(Shift ×10、Ctrl ×0.1、Alt ×0.01)を drag 中に毎 update 読み、擦っている間だけ pill(8 px mono、`surface_4`)で 4 段を見せる。theme は `theme_tokens.dart` の `ThemeToken{key,label,description,group,read,write}` の一覧 = 名前付き token のみ、hex literal は lint で落とす(`docs/15-DESIGN.md` §4)。

Motolii との差:同じ「自前の葉」型だが、Lumit は Material の `ThemeData` すら使わず `InheritedWidget` の `LumitThemeScope`。Motolii は `ThemeData` + `ThemeExtension<EditorInk>` で Material の既定を黙らせる型(`visualDensity.compact`、`shrinkWrap`、`InputDecorationTheme` 等、theme.dart:300-400)。

### Rive(editor は Flutter、非公開)

公開されている一次資料は flutter.dev/showcase/rive と rive-app の GitHub(runtime `rive-flutter` MIT、`flutter-engine` fork)だけ。showcase の証言:CanvasKit で試作して描画が改善、「custom pieces を足すのが楽」、build ごとに数千 test、macOS desktop 版は「抽象化が済んでいるので殆ど手直し不要」。widget 層の package や Material 使用の有無は公開されていない。読める事実は「editor = 自前 widget + 自前 renderer」までで、kit を借りた形跡は無い。

## Motolii に借りられる物と縫い目

### どの kit でも自前のまま残る物(縫い目の内側)

| 残す物 | 場所 | 理由 |
|---|---|---|
| drag→preview→commit の契約 | `panel_controls.dart:1097` `EditorDragSession`、`:1165` `EditorPreviewQueue`、`onPreview/onCommit/onFinish/onCancel` の 4 口 | kit の slider/field は `onChanged`/`onChangeEnd` の 2 口。窓の focus 喪失で cancel、commit 中は次の drag を待つ、queue の間引き — どれも kit に無い |
| 数値の文法 | `EditorNumericField`(`:383`)、`EditorPercentField`、`TrackStyle`(fill/threshold/steps/ruler の `_TrackPainter`)、`mixed`、`defaultValue` の tick、`unit`、`owner` による draft の所有 | fluent NumberBox は入力+矢印+ホイール。擦り・track の塗り・mixed・owner は無い |
| `DocumentSlice` の購読 | `session/editor_session.dart:13` | kit と無関係の層 |
| painter 32 箇所 | `_DialPainter`、`_PadPainter`、`_TrackPainter`、`CheckerPainter`、timeline/stage/ease の painter | `EditorInk` を painter に渡す型は kit の外 |
| dial / pad / anchor grid / lamp | `EditorDial`、`EditorPad`、`EditorAnchorGrid`、`EditorLamp`(`KeyLamp`)| AE 型の部品。どの kit にも無い |
| `EditorMetrics` の 24/20/11 px 密度 | `metrics.dart` | kit の既定は 32〜40 px。数値を注ぐ口(shadcn `scaling`、fluent `VisualDensity`)は 1 本の倍率で、行 20 と control 24 の比までは決められない |

### 置き換えられる物(縫い目の外側)と、kit ごとの相性

| 今の Material 呼び出し | 件数 / file | shadcn_flutter | forui | fluent_ui | macos_ui | 自前(kit 無し)|
|---|---|---|---|---|---|---|
| `InkWell`(hover/press の面) | 25 / 12 file | Button の variants(ghost/outline)+ `Clickable` | `FButton`、`FTappable` | `HoverButton`(`utils/hover_button.dart`)| PushButton は AppKit 寸 | `MouseRegion`+`Listener` の `EditorButton`(theme.dart:413 に既にある)|
| `IconButton` 16 / `TextButton` 14 | 主に panels | ○ | ○ | ○ | ○ | `EditorButton` を広げる |
| `Tooltip` 58(`EditorTooltip` 経由が大半) | 全 panel | Tooltip ○ | FTooltip ○ | Tooltip ○(4.16 で tap dismiss)| MacosTooltip ○ | `EditorTooltip` は Material Tooltip の薄い皮(theme.dart:403)。kit の tooltip に差し替えても 1 file |
| `TextField` 9 | inspector・browser・rich_text・notes | TextField(`InputFeature`)| FTextField | TextBox | MacosTextField | `EditableText` 直(Lumit と同じ)|
| `MenuAnchor` 5(`showEditorMenu`、`EditorChoice`)| theme.dart・panel_controls | Menubar/ContextMenu/DropdownMenu ○ | FPopoverMenu/FContextMenu ○ | MenuFlyout ○ | MacosPulldownButton ○ | `MenuAnchor` は `flutter/widgets` ではなく material_ui 側 → kit 無しなら `OverlayPortal` で自前 |
| `Switch` 10(`EditorSwitch`)| inspector 等 | Switch ○ | FSwitch ○ | ToggleSwitch ○ | MacosSwitch ○ | `EditorSwitch` は既に自前 painter |
| `Slider` 4 | zoom bar・colors_shelf | Slider ○ | FSlider ○ | Slider ○ | MacosSlider ○ | `EditorZoomBar` は既に自前 |
| `Scrollbar` 3 | timeline | Scrollbar ○ | 無し | Scrollbar ○ | MacosScrollbar ○ | `RawScrollbar`(widgets.dart にある)|
| `showDialog` 1 / `DropdownButton` 1 | editor_window・panel_controls | Dialog ○ / Select ○ | FDialog ○ / FSelect ○ | ContentDialog ○ / ComboBox ○ | MacosAlertDialog ○ / MacosPopupButton ○ | `showGeneralDialog`(widgets)|
| split pane / dock(`workspace_view.dart` の自前)| 1 file | Resizable ○ | FResizable ○ | 無し | ResizablePane ○ | multi_split_view か今の自前 |
| tree / list(browser・timeline の層)| 自前 | Tree ○ | 無し | TreeView ○ | 無し | 自前 |
| color picker(`color_field.dart`、colors_shelf)| 自前 | ColorPicker ○(HSV) | 無し | ColorPicker ○(WinUI)| MacosColorWell(AppKit panel、mac 限定)| 自前 |

### 縫い目の代価(file 数の目安)

- **kit 導入の共通費**:`main.dart`(App の差し替え:`ShadcnApp` / `FTheme` / `FluentApp` / `MacosApp`)、`foundation/theme.dart`(token の橋:`EditorInk` は残す、Material の黙らせ 100 行を kit の theme に写す)、`foundation/panel_controls.dart` の `EditorChoice`・`EditorBar`・`EditorFieldFrame`・`EditorCard`(枠と menu の皮)、`foundation/color_field.dart`。= 4 file、ただし theme.dart と panel_controls.dart は大改修。
- **panels 側**:Material を直接呼ぶ 21 file のうち、`EditorTooltip`/`EditorSwitch`/`EditorButton` 経由に既に寄っている物は foundation 側で吸収できる。直接 `InkWell`/`IconButton`/`TextButton`/`TextField` を書いている panel(editor_window、blend_panel、browser + 6 shelf、desk、ease_desk、history_records、inspector、notes_desk、panel_settings、rich_text_editor、web_panel、workspace_view)= **17 file** は 1 行ずつ差し替え。
- **合計 ≈ 21 file、うち大改修 2**。数値 field・dial・pad・painter・DocumentSlice は無傷。
- **kit 固有の代価**:
  - shadcn_flutter:0.0.x、0.0.50 と 0.0.53 と 0.0.54 が連続 breaking(Navigation の key 化、overlay の anchor 化、Material 切り離し)。Motolii の `EditorMetrics` へ合わせるには `scaling` では足りず component theme を 10 前後書く。Radix icon font を抱える。
  - forui:material_ui/cupertino_ui に依存する(Material を消す目的には逆)。Tree・Scrollbar・ColorPicker が無いので 3 つは自前のまま。style は CLI で生成した Dart を手で持つ型 = 版上げごとに再生成。
  - fluent_ui:見た目が WinUI(macOS first の Motolii では違和感、Mica/Acrylic は不要)。split pane 無し。NumberBox は擦れないので `EditorNumericField` は残る。material_ui 依存。
  - macos_ui:mac 専用で他 OS は「保証しない」→ cross-platform の記憶(cross-platform-basis)に反する。11 か月 release 無し(dev 枝は動いている)。密度の口が無い。
- **kit 無し(Lumit 型)の代価**:`MenuAnchor` 5 と `Scrollbar` 3 と `showDialog` 1 と `TextField` 9 を `flutter/widgets` の素材(`OverlayPortal`、`RawScrollbar`、`showGeneralDialog`、`EditableText`)で自前にする。= 上と同じ 21 file を触るが、新しい依存と breaking の追跡は 0。`material_ui` の deprecation(秋)にも独立で乗れる。split は multi_split_view(MIT、依存 meta のみ)を借りてよい。

## 推し(1 つ)

**kit は入れない。`flutter/widgets` + 自前の葉に寄せ、分割だけ multi_split_view を借りる(Lumit と同じ型)。**

理由:

1. Motolii の価値がある部品(擦る数値・dial・pad・track の塗り・lamp・painter)はどの kit にも無く、契約(preview/commit の 4 口、`owner`、`mixed`)も kit の 2 口に収まらない。借りて減るのは button/tooltip/menu/dialog の「皮」だけで、それは `EditorButton`・`EditorTooltip`・`EditorSwitch`・`EditorZoomBar` として既に半分自前になっている。
2. 先例 2 つ(Lumit の文書、Rive の証言)がどちらも「土台は Flutter、葉は自前」で、kit を挟んでいない。Lumit は理由まで書いている(kit の metrics・splash・motion が house design と喧嘩する)— Motolii の theme.dart が Material を黙らせる 100 行を書いている事実がその再演。
3. 3.47 の material_ui 分離で「Material を抜く」費用が下がった。kit を入れると material_ui を抜けない(forui・fluent は依存)か、0.0.x の breaking を追う(shadcn)かの二択になる。
4. 密度。`EditorMetrics` の row 20 / control 24 / font 11 は kit の既定(32〜40 px、14 px)から 2 段小さく、kit の倍率 1 本(`scaling`、`VisualDensity`)では比を保てない。自前なら token を直接読む。

代償:

- menu(`MenuAnchor` 5)・scrollbar 3・dialog 1・text field 9 を `OverlayPortal` / `RawScrollbar` / `showGeneralDialog` / `EditableText` で書く手間。焦点・矢印キー・Esc・外側 click で閉じる、の 4 つは自分で持つ(Lumit の menus.dart / popups.dart が写せる先例、GPLv3 なので値と型だけ写す — lumit-reference の記憶どおり)。
- Tree・color picker は今も自前なので増えない。
- 「借りる物が減る」のは patchwork の原則から見て後退に見えるが、ここで借りるべき先例は kit ではなく **Lumit の value_field(修飾子の梯子 4 段 + 擦り中の pill)** と **shadcn_flutter の Resizable/ContextMenu の作り(widgets.dart だけで overlay と focus をどう組んだか)** で、それは読んで写せる。
- もし 1 つだけ kit を入れるなら shadcn_flutter(widgets.dart のみ、部品が全部揃う、BSD-3)。条件は「0.1.0 まで待つ」と「`ShadcnApp` を root にせず `Theme` だけ差して部品単位で拾う」。今は時期尚早。

## Sources

- shadcn_flutter: https://pub.dev/packages/shadcn_flutter 、changelog https://pub.dev/packages/shadcn_flutter/changelog 、repo https://github.com/sunarya-thito/shadcn_flutter (`packages/shadcn_flutter/pubspec.yaml`、`lib/src/theme/theme.dart`、`lib/src/components/{layout/resizable,layout/tree,menu/menubar,menu/context_menu,control/scrollbar,form/slider,form/color/solid/color_picker}.dart`)
- forui: https://pub.dev/packages/forui 、changelog https://pub.dev/packages/forui/changelog 、repo https://github.com/forus-labs/forui (`forui/pubspec.yaml`、`forui/lib/src/theme/theme_data.dart`、`forui/lib/src/widgets/`)
- fluent_ui: https://pub.dev/packages/fluent_ui 、changelog https://pub.dev/packages/fluent_ui/changelog 、repo https://github.com/bdlukaa/fluent_ui (`pubspec.yaml`、`lib/src/controls/form/number_box.dart`、`lib/src/controls/` 一覧)
- macos_ui: https://pub.dev/packages/macos_ui 、changelog https://pub.dev/packages/macos_ui/changelog 、repo https://github.com/macosui/macos_ui (`pubspec.yaml`、README)
- yaru: https://pub.dev/packages/yaru 、changelog https://pub.dev/packages/yaru/changelog 、repo https://github.com/ubuntu/yaru.dart
- flutter_platform_widgets(打ち切り): https://pub.dev/packages/flutter_platform_widgets
- desktop_ui_kit: https://pub.dev/packages/desktop_ui_kit 、multi_split_view: https://pub.dev/packages/multi_split_view 、docking: https://pub.dev/packages/docking 、dock_panel: https://pub.dev/packages/dock_panel 、tabbed_view / panes / flutter_web_split_view: pub.dev 検索(2026-09-19)
- Flutter 3.47 と material_ui / cupertino_ui: https://flutter.dev/blog/whats-new-in-flutter-3-47 、https://docs.flutter.dev/release/breaking-changes/material-ui-and-cupertino-ui
- Lumit: https://github.com/luminalmvm/Lumit (`flutter_ui/pubspec.yaml`、`flutter_ui/lib/widgets/controls/value_field.dart`、`flutter_ui/lib/shell/dock_widget.dart`、`flutter_ui/lib/theme/theme_tokens.dart`、`docs/archive/flutter-port/04-WIDGET-MAP.md`、`docs/15-DESIGN.md`)
- Rive: https://flutter.dev/showcase/rive 、https://github.com/rive-app/rive-flutter 、https://github.com/rive-app/flutter-engine
- Superlist: https://flutter.dev/showcase/superlist (技術記述なし)、Flokk: https://github.com/gskinnerTeam/flokk (最終 push 2023-07-13)
- Motolii 側: `motolii/ui/lib/foundation/theme.dart`、`metrics.dart`、`panel_controls.dart`、`color_field.dart`、`motolii/ui/lib/session/editor_session.dart`、`motolii/ui/pubspec.yaml`(依存は flutter のみ)
