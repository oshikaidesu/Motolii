# Visual audit: Classic Flutter UI (motolii/ui/lib), 2026-09-25

This audit was read-only. Paths are relative to `motolii/ui/` unless noted. Counts come from grep over `lib/`: 26.6k LOC in about 110 Dart files. They are approximate, because the counts include comments and doc references.

---

## 1. Theme architecture

### What Flutter theming is used
- **Material is present only as a carrier.** `foundation/theme.dart:5` imports exactly `show Theme, ThemeData, ThemeExtension, ColorScheme`. The `material_import` lint (`tool/motolii_lints/lib/src/material_import.dart`) refuses every other Material import. No `TextTheme` is used, and no Material component themes are used.
- `EditorTheme.wrap(child)` (`theme.dart:482-496`) builds a `ThemeData(brightness: dark, colorScheme: ColorScheme.dark(primary: accent, secondary: spatial, surface: panel, onSurface: ink, onPrimary: tabInk, error: error), extensions: [this])`. The ColorScheme is vestigial, and no widget reads it: `Theme.of` appears only at `theme.dart:279`.
- The app shell is `WidgetsApp` (`app/editor_app.dart:38`). Its root style comes from `EditorTheme.text`, it sets `IconTheme` from `EditorTheme.icon`, and it sets `DefaultSelectionStyle(cursorColor: caret, selectionColor: selection)`. `EditorScrollBehavior`, `EditorScale` and `EditorScaledViewport` wrap everything (`editor_app.dart:30-66`).
- Stale doc: `docs/stage5/product-contract.md:55` still says component looks live in `EditorTheme.data` Material component themes (popupMenu, slider, dialog…). That stopped being true once the widgets-only leaves replaced Material. Anyone who reads that line will be misled.

### `foundation/theme.dart` (703 lines)
**`EditorTheme extends ThemeExtension<EditorTheme>`** (`:200`) has four fields: `name`, `colors: Map<String, Color>` (48 keys), `identityColors: List<Color>` (7) and `drawing: EditorInk`. The only built-in instance is `EditorTheme.chromatic` ("Chromatic Workshop", `:213-275`). Every colour has a typed getter (`:281-328`).

| group | tokens (value) | meaning |
|---|---|---|
| surfaces | `app` #252525, `panel` #323232, `raised` #414141, `hover` #4e4e4e, `menu` #252525 | window ground, panel body, raised header/row, hovered fill, menu sheet |
| rules | `line` #1b1b1b, `border` #676767, `menuEdge` #7e7e7e | dark divider, light frame, menu outline |
| ink | `ink` #f0f0f0, `muted` #bdbdbd, `disabledInk` #929292, `inkDisabled` 38% white, `tabInk` #202020, `selectInk` #202020 | text and icon levels; ink on a lit tab or selection |
| state | `accent` #ffbc53 (orange), `animate` #5396ff (blue), `tab` #59c9df (cyan), `select` #b0e3ef (pale cyan), `selection` 40% #59c9df, `caret` #ffbc53, `error` #ff8899 | see §4 |
| property families | `spatial` #819fff, `amount` #f5ad79, `time` #c08ee4, `count` #60cedb, `seed` #ffdf56, `angle` #a0d292 | hue per numeric family |
| kinds (Browser) | `kindText` #ffdf56, `kindShape` #819fff, `kindPath` #60cedb, `kindVideo` #ef87ae, `kindAudio` #a0d292, `kind3d` #cf8eef, `kindHdr` #ffdf78, `kindImage` #819fff, `kindOther` #c08ee4 | shelf kind marks |
| washes / alpha | `scrim` 54% black, `scrimLight` 45% black, `hoverWash` 4% white, `focusWash` 12% white, `washDisabled` 12% white, `scrollThumb`/`Hovered`/`Dragged` 30/65/75% white, `tickActive` 38% black, `tickInactive` 38% white, `tooltip` 90% white, `timelineWash` 30% #606060 | overlays |
| identity | `identityColors` = [#819fff, #ffdf56, #c08ee4, #60cedb, #f5ad79, #a0d292, #ef87ae] | layer affiliation (`layerColor(id) = identity[id % 7]`, `:339`); `timelineColor(id)` = identity with `timelineWash` alpha-blended on top, cached in an `Expando` (`:343`) |

The class also holds statics and derived values:
- `white`, `black`, `clear` (`:329`).
- Press lifts: `hoverLift` .08, `pressedLift` .10, `draggedLift` .16, and `lift` = dragged (`:332`).
- `fontFamily = 'Inter'` (`:336`).
- `text` (`:366`): the base TextStyle. It is `inherit:false`, Inter, `EditorMetrics.font` (10), w500, `ink`.
- `icon` (`:374`): size 14, colour `ink`.
- Menu metrics: `menuPadding`, `menuMinWidth` 112, `menuRowPadding` (`:375-379`).
- **`static final animating = ValueNotifier<bool>`** (`:337`) with `keyAccent => animating ? animate : accent` (`:338`). This is a process-global static, not a theme field. `editor_window.dart:188` sets it.

**`EditorInk`** (`:16-187`) holds the painter colours, with 15 named values plus `collectionColors` (7). The only instance is `EditorInk.dark`.
- Named values: `laneGround` #2c2c2c, `lane` #2b2b2b, `laneAlt` #323232, `grid` #1c1c1c, `gridMinor` #272727, `tick` #bababa, `tickMinor` #aaaaaa, `headerInk` #1d1d1d, `camera` #8ed9e6, `focusRing` #acacac, `easePaper` #d2d2d2, `easeInk` #333333, `easeTime` #854515, `checkerLight` #8c8c8c, `checkerDark` #666666.
- `collectionColors`: red #e05252, orange #e0a052, yellow #e0d452, green #6fd06f, blue #52b9e0, violet #8f7ae0, grey #8a8a8a. This is a fourth palette, independent of the others.
- It is reached through `EditorInk.of(ctx) = EditorTheme.of(ctx).drawing`, which has 6 call sites.

**Other classes in theme.dart:**
- `EditorLook` (InheritedWidget, `:190`) carries only `tooltips` on/off.
- `EditorAppearance` (InheritedNotifier over `ValueNotifier<EditorTheme>`, `:530`) lets the window swap the theme.
- The same file also defines widgets: `EditorTooltip`, `EditorButton`, `EditorSection` (`:599`, which has a raw `fontSize: 10, w600, letterSpacing .5` at `:620-624`), `showEditorMenu`, `EditorMenuItem` and `EditorMenuDivider`.
- `EditorTheme.fromJson/toJson` (`:381-478`) is a public theme-file schema at version 1. It rejects unknown keys, so the set of colour keys is closed, and it overlays the given keys onto `chromatic`. The schema, guide and sample are in `ui/themes/{README.md, theme.schema.json, velvet.json}`.

### `foundation/metrics.dart` (38 lines): `abstract final class EditorMetrics`, static const doubles only
- Rows and bars: `row` 20, `control` 24, `section` 26, `bar` 28, `tall` 30.
- Type sizes: `micro` 8, `dense` 9, `font` 10, `title` 13.
- Boxes: `mark` 28, `field` 52, `cell` 128, `thumb` 128, `sheet` 320, `sheetWide` 420, `canvas` 5000.
- The spacing scale is s2, s3, s4, s6, s8, s12, s16, s32 and s48.
- Off-scale snap candidates (`:23-37`): s5, s7, s10, s11, s14, s15, s17, s18, s19, s22, s23, s34, s36, s44, s60, s64, s70, s76, s78, s85, s90, s96, s155, s160, s200, s244, s280.
- **Radius, border width, font size and spacing all share this one scale. It has no semantic names such as `radius` or `rule`.**
- The lint `raw_dimension` enforces the scale (`tool/motolii_lints/lib/src/raw_dimension.dart`, wired in `analysis_options.yaml`). It allows only 0, .5 and 1 (`:32`). `theme.dart`, `metrics.dart` and `panel_catalog.dart` are exempt. The quick fix `use_metric` swaps in the token that has the same value.
- Metrics are not injectable. They are compile-time consts that no context carries.

### `foundation/glyphs.dart` (395 lines)
`abstract final class Glyph` holds 109 `IconData` consts copied from material/icons.dart, drawn with the `MaterialIcons` font that `uses-material-design: true` bundles. There is no custom icon set. The icon size comes from `EditorTheme.icon` (14) or from explicit sizes.

### `app/theme_settings.dart` (144 lines): runtime switching exists
- The `ThemeSettings` section offers four actions: Load JSON… (native `pickImport`, 128 KB cap, `EditorTheme.fromJson`), Reload (re-reads the saved path), Copy JSON, and Default.
- Persistence: `controller.storeDesk('theme', {path, data})` saves the theme, and `setPaneState` broadcasts it to the other windows.
- `editor_window.dart:190-202` listens to `deskWork['theme']` and sets `EditorAppearance.of(context).value` to either `chromatic` or `fromJson(data)`.
- `editor_app.dart:21,30-34` holds `ValueNotifier(EditorTheme.chromatic)`, rebuilds through `ValueListenableBuilder`, and calls `theme.wrap(...)`.
- **Only colours switch at runtime. Metrics, radius, type and font do not.**

### How panels get tokens
- `EditorTheme.of(context)` is the main route: 382 calls across 57 files. It is an InheritedWidget lookup through Material `Theme`.
- `CustomPainter`s receive a snapshot as a constructor arg (`colors: EditorTheme`, `ink: EditorInk`), and `shouldRepaint` compares it. **12 painters default to the static `EditorTheme.chromatic` or `EditorInk.dark`:**
  - `timeline/paint.dart:17,37`
  - `timeline/overview.dart:14`
  - `stage/overlay.dart:34,54`
  - `depth_desk.dart:319-320`
  - `history_records.dart:208`
  - `browser/color_wheel.dart:123`
  - `ease_desk/painters.dart:7,203,259`
  - `leaves/track.dart:191`
  - `leaves/dialog.dart:152`
  - `panel_controls/dials.dart:127,324`
  - `numeric_paint.dart:76`
  - `scale.dart:185`
- **Some code bypasses the theme entirely:**
  - The ease desk reads `EditorInk.dark.easePaper/easeInk/easeTime` directly at about 25 sites (`ease_desk.dart:118`, `ease_desk/parts.dart:48,54,85,379,415-416`, `ease_desk/painters.dart:48-340`).
  - `browser/filter_library.dart:28` reads the static `EditorInk.dark.collectionColors`.
  - `keyAccent` reads the global static `animating`.
- Metrics are always the static `EditorMetrics.x`.

### Injecting a second token set without touching Classic
- **Colours are already inheritable.** Wrapping the New UI subtree in `newUiTheme.wrap(child)` shadows the root theme for everything under it. The new instance would be `EditorTheme.chromatic.copyWith(name:, colors:{...}, identityColors:, drawing:)`, since `copyWith` is public. The Classic root and the Classic windows keep `chromatic` or the user's JSON.
- **The leaks are the static and default paths listed above.** Any New UI painter must be passed `EditorTheme.of(ctx)`, which the painters already accept. The ease desk and `filter_library` would ignore the New UI colours, which is acceptable if the ease desk stays Classic-looking.
- **Geometry and type have no injection point.** They need a new ThemeExtension that Classic never installs (see §6).
- **Watch the closed colour-key set.** `fromJson` rejects unknown keys, and `lerp` iterates `colors.keys`. New semantic roles should therefore live in a separate extension, not as new keys in `colors`. New keys in `colors` would change the Classic theme-file schema and `Copy JSON` output.

---

## 2. Hardcode census

### Headline
The codebase is already heavily tokenised. Two lints enforce it: `raw_color` (`tool/motolii_lints/lib/src/raw_color.dart`) and `raw_dimension`. **Outside `theme.dart` there are only 3 `Color(...)` constructors, and all three compute colours from data:**
- `color_field.dart:46` parses a hex string.
- `blend_panel.dart:119` builds a blend sample.
- `stage/geometry.dart:60` converts document RGB.

**Only 1 raw fontSize** exists (`theme.dart:621`, in the exempt file). **The raw numbers that remain are 1 and 0.5**, which the lint allows: 35 sites use `width: 1`, `EdgeInsets.all(1)` or `, 1)` in strokes, for example `frames.dart:142 EdgeInsets.all(1)`, `theme.dart:578` and `workspace_view.dart:324`.

What is hardcoded is **the choice of token per widget**. For example, a panel writes `TextStyle(fontSize: EditorMetrics.dense, fontWeight: FontWeight.w600, color: EditorTheme.of(context).muted)` inline. So there are about 123 inline TextStyles, 43 alpha or opacity literals and 31 radii, each choosing its own token.

### Per file

Columns:
- colLit: `Color(0x`/`fromARGB`
- EdgeIns: EdgeInsets
- SizedBox
- Radius: BorderRadius/Radius
- Border: Border/BorderSide
- TStyle: `TextStyle(`
- shadow/grad: BoxShadow/Gradient/blur
- alpha: withValues(alpha)/Opacity

| file | LOC | colLit | EdgeIns | SizedBox | Radius | Border | TStyle | shadow/grad | alpha |
|---|---|---|---|---|---|---|---|---|---|
| foundation/theme.dart | 703 | 80 (the palette) | 4 | 1 | 0 | 2 | 2 (+1 raw fontSize) | 0 | 1 |
| foundation/color_field.dart | 68 | 1 (parse) | 0 | 0 | 1 | 1 | 0 | 0 | 0 |
| leaves/choice.dart | 76 | 0 | 1 | 0 | 0 | 1 | 1 | 0 | 0 |
| leaves/dialog.dart | 177 | 0 | 4 | 1 | 0 | 2 | 4 | 0 | 1 |
| leaves/field.dart | 221 | 0 | 3 | 0 | 1 | 0 | 0 | 0 | 0 |
| leaves/floating.dart | 347 | 0 | 1 | 0 | 1 | 2 | 3 | 0 | 0 |
| leaves/press.dart | 344 | 0 | 2 | 0 | 4 | 2 | 1 | 0 | 2 |
| leaves/track.dart | 288 | 0 | 0 | 1 | 2 | 2 | 1 | 0 | 0 |
| panel_controls/dials.dart | 413 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| panel_controls/fields.dart | 166 | 0 | 2 | 0 | 0 | 1 | 1 | 0 | 0 |
| panel_controls/frames.dart | 291 | 0 | 5 | 6 | 1 | 2 | 4 | 0 | 2 |
| panel_controls/numeric.dart | 617 | 0 | 2 | 2 | 0 | 6 | 4 | 0 | 0 |
| panel_controls/numeric_paint.dart | 160 | 0 | 1 | 0 | 1 | 0 | 2 | 0 | 2 |
| panel_controls/scale.dart | 201 | 0 | 0 | 1 | 0 | 1 | 0 | 0 | 0 |
| panel_controls/toggles.dart | 253 | 0 | 1 | 2 | 0 | 1 | 0 | 0 | 2 |
| app/editor_window.dart | 653 | 0 | 2 | 3 | 0 | 1 | 1 | 0 | 0 |
| app/theme_settings.dart | 144 | 0 | 2 | 1 | 0 | 0 | 1 | 0 | 0 |
| workspace/workspace_view.dart | 394 | 0 | 2 | 3 | 1 | 4 | 4 | 0 | 1 |
| panels/blend_panel.dart | 429 | 1 (computed) | 1 | 2 | 0 | 1 | 1 | 0 | 1 |
| panels/browser.dart | 661 | 0 | 1 | 3 | 0 | 0 | 1 | 0 | 0 |
| browser/color_picker.dart | 395 | 0 | 4 | 5 | 2 | 0 | 2 | 0 | 0 |
| browser/color_wheel.dart | 222 | 0 | 0 | 0 | 1 | 0 | 0 | **5** | 1 |
| browser/colors_shelf.dart | 377 | 0 | 3 | 0 | 0 | 0 | 1 | 0 | 0 |
| browser/create_shelf.dart | 245 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | 0 |
| browser/effects_shelf.dart | 207 | 0 | 1 | 1 | 0 | 2 | 2 | 0 | 0 |
| browser/files_shelf.dart | 337 | 0 | 2 | 1 | 0 | 2 | 2 | 0 | 0 |
| browser/fill_definitions.dart | 153 | 0 | 1 | 2 | 1 | 1 | 1 | 0 | 0 |
| browser/filter_view.dart | 360 | 0 | 7 | 4 | 2 | 3 | 9 | 0 | 0 |
| browser/fonts_shelf.dart | 470 | 0 | 5 | 5 | 1 | 5 | 5 | 0 | 0 |
| browser/frame_bars.dart | 211 | 0 | 6 | 4 | 0 | 1 | 4 | 0 | 0 |
| browser/frame_filters.dart | 142 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | 0 |
| browser/frame_grid.dart | 143 | 0 | 2 | 0 | 0 | 2 | 1 | 0 | 0 |
| browser/media_shelf.dart | 487 | 0 | 2 | 1 | 0 | 2 | 3 | 0 | 3 |
| browser/parts.dart | 303 | 0 | 2 | 1 | 0 | 2 | 5 | 0 | 1 |
| browser/quick_tags.dart | 173 | 0 | 6 | 4 | 1 | 2 | 3 | 0 | 0 |
| browser/rail_collections.dart | 216 | 0 | 5 | 2 | 0 | 0 | 3 | 0 | 1 |
| browser/tile.dart | 335 | 0 | 4 | 4 | 1 | 2 | 4 | 0 | 1 |
| panels/composition_controls.dart | 140 | 0 | 1 | 2 | 0 | 0 | 0 | 0 | 0 |
| panels/depth_desk.dart | 386 | 0 | 0 | 5 | 0 | 1 | 2 | 0 | 0 |
| panels/desk.dart | 238 | 0 | 0 | 5 | 0 | 0 | 2 | 0 | 0 |
| panels/ease_desk.dart | 454 | 0 | 2 | 8 | 1 | 0 | 1 | 0 | 0 |
| ease_desk/painters.dart | 357 | 0 | 0 | 0 | 1 | 0 | 1 | 0 | 4 |
| ease_desk/parts.dart | 434 | 0 | 2 | 13 | 3 | 1 | 8 | 0 | 0 |
| ease_desk/values.dart | 92 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| panels/export_controls.dart | 102 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| panels/gradient_inspector.dart | 335 | 0 | 1 | 2 | 2 | 2 | 0 | 0 | 2 |
| panels/history_records.dart | 277 | 0 | 3 | 2 | 0 | 0 | 3 | 0 | 0 |
| panels/inspector.dart | 383 | 0 | 3 | 2 | 0 | 1 | 3 | 0 | 2 |
| inspector/content_cards.dart | 145 | 0 | 1 | 2 | 0 | 1 | 2 | 0 | 0 |
| inspector/controls.dart | 221 | 0 | 0 | 5 | 0 | 0 | 0 | 0 | 0 |
| inspector/effects_card.dart | 214 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | 0 |
| inspector/layout_card.dart | 352 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| inspector/parts.dart | 191 | 0 | 4 | 2 | 0 | 1 | 2 | 0 | 0 |
| inspector/wells.dart | 383 | 0 | 0 | 11 | 0 | 0 | 2 | 0 | 0 |
| panels/notes_desk.dart | 713 | 0 | 4 | 6 | 0 | 1 | 4 | 0 | 0 |
| panels/panel_settings.dart | 167 | 0 | 2 | 7 | 0 | 0 | 8 | 0 | 0 |
| panels/rich_text_editor.dart | 373 | 0 | 1 | 0 | 0 | 0 | 3 | 0 | 0 |
| stage/chrome.dart | 446 | 0 | 4 | 1 | 0 | 2 | 3 | 0 | 0 |
| stage/geometry.dart | 142 | 1 (doc RGB) | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| stage/overlay.dart | 245 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 2 |
| panels/timeline.dart | 466 | 0 | 2 | 7 | 0 | 0 | 0 | 0 | 0 |
| timeline/ink.dart | 72 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 |
| timeline/overview.dart | 87 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 1 |
| timeline/paint.dart | 605 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8 |
| web_panel.dart / registry / menu / protocol | – | 0 | 2 | 4 | 0 | 0 | 0 | 0 | 0 |
| **total** | 26.6k | **83 (80 in theme)** | **117** | **149** | **31** | **63** | **123** | **6** | **43** |

### Detail by category
- **EdgeInsets / SizedBox (117 / 149).** Every value is an `EditorMetrics` token, except structural 0 and 1. Examples: `frames.dart:21 EdgeInsets.symmetric(horizontal: EditorMetrics.s8)` and `theme_settings.dart:85 EdgeInsets.all(EditorMetrics.s6)`. There is no semantic padding token such as `rowInset` or `cellGap`, so each panel picks its own step.
- **Radius (31 sites).** Values in use: s2 (7× BorderRadius + 2 Radius), s3 (1 + 2 + tile `s3*markScale`), **s4 (5× BorderRadius + 3 `BorderRadius.all` + 2 Radius)** and **s5** (`color_picker.dart:239`).
  - s4 sites: floating menu `floating.dart:26`, numeric well `numeric_paint.dart:28`, colour field `color_field.dart:60`, ease desk `ease_desk.dart:119`, `ease_desk/parts.dart:42,52,425`, `values.dart:49`, colour wheel `:178`, slider thumb `track.dart:270`.
  - The panel frame is s3 (`workspace_view.dart:326`).
  - The slider track is `_track/2` (pill, `track.dart:209`).
  - Circles: `BoxShape.circle` in `toggles.dart:135` (lamps), plus about 39 `drawCircle` calls (markers, keys, handles).
  - **The s4 and s5 sites exceed the brief's 0–3 px range.**
- **Borders (63).** Almost all use the default 1px with `line`, `border` or `accent`.
  - Wider exceptions: s2 for the drop-candidate panel frame (`workspace_view.dart:316-318`, accent), plus two s3-wide borders.
  - Active panel: `workspace_view.dart:320-324`, 1px `focusRing`.
- **TextStyle (123 constructed inline; 9 `copyWith`).** fontSize is always a token:
  - font(10) ×35, dense(9) ×26, micro(8) ×21, title(13) ×8, s12 ×6, plus one each of s22, s23 and `mark`, some computed sizes, and the raw 10 in EditorSection.
  - Weights: w600 ×12 (headers and kickers), w500 ×2, w400 ×1, and the rest inherit w500.
  - letterSpacing appears 8×. Uppercase kickers use 1 (`inspector/parts.dart:33`, `frame_bars.dart:137`, `rail_collections.dart:211`, `frames.dart:276`) or .5 (`theme.dart:623`, `tile.dart:106,297`); `fonts_shelf.dart:466` uses .3.
  - `fontFamily: EditorTheme.fontFamily` is written explicitly in 7 places: painters and `TextPainter`s, which do not inherit `DefaultTextStyle` (`timeline/ink.dart:53`, `track.dart:255`, `ease_desk/painters.dart:166`, `browser/parts.dart:196`, `tile.dart:294`, `workspace_view.dart:183`).
  - There is no named text-style set. Each widget combines size, weight, colour and letterSpacing itself.
- **Shadow / gradient / blur (6).** All are in `browser/color_wheel.dart`: the SweepGradient hue ring (:142), LinearGradient SV (:184,:190), and `MaskFilter.blur` on the handles (:160,:209). These are functional, not decorative. The app has **no BoxShadow and no BackdropFilter**. Brief-compliant.
- **Opacity / alpha literals (43).** Sites that use a token: `press.dart:271-273` uses `hoverLift`/`pressedLift`, and `timeline/paint.dart:464` uses `lift`. **The rest are ad-hoc numbers:**
  - timeline/paint: `muted@.28` (:244), `own@.55` stroke (:248), `own@.9` (:258), `muted@.6` (:322), `muted@.4` markers (:381), `accent@.15` marquee (:404), `white@.025` (:138).
  - `overview.dart:54` white@.10.
  - `stage/overlay.dart:100` app@.55 and `:207` accent@.12.
  - `numeric_paint.dart:92,104` tint@.28/.5.
  - `toggles.dart:96,98` keyAccent@.55/.3 (key lamp states).
  - `browser/parts.dart:65` muted@.45; `tile.dart:210` @.75; `media_shelf.dart:286` panel@.85; `rail_collections.dart:121` spatial@.3.
  - Disabled or frozen states: `Opacity(.4/.45/.35)` at `workspace_view.dart:220`, `blend_panel.dart:255`, `inspector.dart:343`, `layout_card.dart:280`, `frames.dart:210` and `gradient_inspector.dart:303`.
  - **There is no shared "disabled opacity" or "wash" token.**

---

## 3. Shared controls / primitives inventory

Refs are lib-wide word matches outside the defining file, so they approximate call sites.

### `foundation/leaves/*` (widgets-only replacements for Material leaves)
| component | draws | tokens used | hardcodes | refs |
|---|---|---|---|---|
| `EditorPress` (press.dart) | hover/press/focus wash on any child; optional `borderRadius` | hoverWash, focusWash, line, hoverLift, pressedLift | – | 25 |
| `EditorIconButton` | square glyph button | ink, inkDisabled, accent (lit), control/row sizes | – | 14 |
| `EditorTextButton` | text button with `background/foreground/radius` params (default radius zero) | accent default fg, disabledInk | – | 10 (+26 through `EditorButton`) |
| `EditorButton` (theme.dart:563) | 1px-inset, row-2 high text button; selected → accent bg, tabInk | accent, panel, tabInk, ink | `EdgeInsets.symmetric(1,1)` | 26 |
| `EditorChoice` (choice.dart) | dropdown field + menu | app, border, line, ink, muted | – | 9 |
| `EditorTextField` (field.dart) | EditableText with caret and selection | caret, selection, muted, text; cursorRadius s2 | – | 8 |
| `EditorMenuSheet/Row/Anchor/ChoiceRow`, `EditorTooltipSheet` (floating.dart) | context menu (radius s4, menuEdge outline, row hover `select`/`selectInk`), tooltip | menu, menuEdge, select, selectInk, tooltip, disabledInk, menuPadding/RowPadding | radius s4 | 1–2 each; `EditorMenuItem` 36, `showEditorMenu` 6, `EditorTooltip` 60 |
| `EditorDialog`, `EditorSpinner` (dialog.dart) | modal sheet on scrim; spinner painter | scrim, panel, border, accent | spinner default `chromatic` | 1 each |
| `EditorRule`, `EditorScrollbar`, `EditorSlider` (track.dart) | divider; scroll thumb; pill-track slider with s4 thumb and value label | line, scrollThumb*, tickActive/Inactive, accent, tabInk, muted | pill track `_track/2`, thumb s4, painter default `chromatic` | 3 / 3 / 2 |

### `foundation/panel_controls/*`
| component | draws | tokens | hardcodes | refs |
|---|---|---|---|---|
| `EditorNumericField` (numeric.dart + numeric_paint.dart) | **the property well**: s4-rounded well, `TrackStyle` fill/steps/ruler/level wash by family tint, drag-scrub, typed edit (**tabular figures** :418), ladder popup (tabular :551, rung lit `tab`) | app, raised, hover, line, ink, muted, tab, tabInk; tint passed in | tint@.28/.5, radius s4, painter default `chromatic` | 4 (widely used through inspector wells) |
| `TrackStyle` | enum for the above | – | – | 7 |
| `EditorDial`, `EditorPad` (dials.dart) | angle ring; 2D pad (s3 rrect) | accent, app, border, line, ink, muted | painter default `chromatic` | 2 / 2 |
| `EditorSwitch`, `EditorLamp`, `KeyLamp`, `Picked` (toggles.dart) | on/off switch lit `tint ?? accent`; circle lamp; keyframe lamp keyed/draft = keyAccent@.55/.3 | accent, keyAccent, raised, tabInk, muted, disabledInk | alpha literals | 13 / 3 / 15 / 4 |
| `EditorFieldFrame`, `EditorDraftField` (fields.dart) | text box frame, error ink | app, line, accent, error | – | 5 / 5 |
| `EditorBar`, `EditorCard`, `EditorFold`, `EditorAnchorGrid`, `panelButton`, `panelTitle` (frames.dart) | panel header bar; card with kicker (uppercase, letterSpacing 1); fold; 3×3 anchor grid (radius s2) | panel, raised, line, border, ink, muted, accent | `EdgeInsets.all(1)`, Opacity .4 | 2 / 2 / 1 / 1 / 1 / 1 |
| `EditorScale`, `EditorScaledViewport`, `EditorPercentField`, `EditorZoomBar`, `CheckerPainter` (scale.dart) | UI-scale zoom; transparency checker | line, muted, checkerLight/Dark | `CheckerPainter` default `EditorInk.dark` | 2 / 1 / 3 / 5 / 2 |
| `EditorDragSession`, `EditorPreviewQueue` (drag.dart) | behaviour only | – | – | 4 / 3 |
| `EditorColorField`, `Swatch` (color_field.dart) | hex field + swatch (radius s4) | border | parses hex | 2 / 1 |
| `EditorSection` (theme.dart:599) | uppercase section header on `raised` | raised, line, ink | **raw fontSize 10, w600, ls .5** | 2 |

### Panel-local parts (private, `part of`)
- **`panels/inspector/parts.dart`** has the private widgets `_Live`, `_SectionLabel` (kicker: micro, w600, letterSpacing 1, muted, rule `line`), `_HeadGlyph`, `_CellLabel`, `_EffectGrip`, `_AdvancedFold`, `_SeedRoll` and `_SpaceChoice`. They use ink, muted, disabledInk and line. The family tints come from `inspector/property_style.dart:125 _tintOf`, which maps each family to spatial, amount, time, count, seed or angle.
- **`panels/browser/parts.dart`** provides `shelfAction` (2), `shelfButton` (3), `shelfViews` (1), `shelfGrip` (2), `Hover` (1) and `FittedName` (2, which sets `fontFamily` explicitly). They use accent, app, border, line, raised, muted, disabledInk and the literal `muted@.45`.
- **`panels/browser/tile.dart`** has three pieces:
  - `ShelfTile`: a kind mark in a radius `s3*markScale` box coloured by `kindColor`. **Selected draws a 1px `spatial` border** (:175-178), and an accent badge sits at :70.
  - `BrowserDrag`.
  - `dragFeedback`: uppercase, letterSpacing .5.
- **Timeline paint:**
  - `timeline/ink.dart` defines `fillPaint` (22 refs), `linePaint` (15) and `strokePaint` (6), all reused Paint objects, plus a cached `TextPainter` label. The label uses Inter with **no tabular figures** (:47-56).
  - `TimelinePainter` (paint.dart, 3 refs) uses lane, laneAlt, laneGround, grid, gridMinor, tick, tickMinor and headerInk from `EditorInk`. It takes `timelineColor(id)` for rows and clips, `keyAccent` for keys and the playhead (:394-399), `accent` for markers, the marquee and on-toggles (:383,404-406,486,509), `select` for row-drop guides (:545-565), and `white@lift` for the selection region (:464).
  - `ArrangementOverview` (overview.dart, 2 refs) draws the viewport stroke in `tab` (:61).

A New UI would re-skin mainly these components: `EditorPress`, `EditorButton`, `EditorTextButton`, `EditorIconButton`, `EditorMenu*`, `EditorChoice`, `EditorNumericField`, `EditorSwitch`, `KeyLamp`, `EditorBar`, `EditorCard`, `EditorSection`, the inspector `_SectionLabel`, `ShelfTile`, and `TimelinePainter` with `ink.dart`.

---

## 4. Colour semantics in use

| role | token / value | where it is defined | where it is drawn |
|---|---|---|---|
| active panel frame | `EditorInk.focusRing` #acacac, 1px | theme.dart:105 | workspace_view.dart:320-324 (contract: product-contract.md:21) |
| drop-candidate panel | `accent`, 2px | theme.dart:224 | workspace_view.dart:316-318 |
| active tab | `tab` #59c9df with `tabInk` | :226-227 | workspace_view.dart:242,266; files_shelf.dart:88; numeric ladder rung numeric.dart:565; overview viewport overview.dart:61 |
| menu/list hover-select | `select` #b0e3ef with `selectInk` | :230-231 | floating.dart:237; blend_panel.dart:396; rich_text_editor.dart:77; **timeline row-drop guide** paint.dart:545,565 |
| text selection | `selection` = tab @40% | :252 | editor_app.dart:52, field.dart, numeric.dart, frame_keys |
| caret | `caret` = accent #ffbc53 | :262 | editor_app.dart:51, field.dart |
| "on / chosen / primary" | `accent` #ffbc53 | :224 | EditorButton selected, EditorSwitch lit, filter/tag chosen (filter_view, fonts_shelf, fill_definitions, frame_bars, rail_collections, color_picker, notes_desk, desk, blend_panel), timeline markers, marquee, keys-open and M/S/L on |
| keyframes + playhead | `keyAccent` = accent, or `animate` #5396ff while animating | :338 | timeline paint.dart:300,322,349,394-399,535; toggles.dart KeyLamp; inspector.dart; overview |
| browser item selected | **`spatial`** #819fff border | :234 | tile.dart:175-178; rail_collections.dart:121 (drop hover @.3) |
| property families | spatial/amount/time/count/seed/angle | :234-239 | inspector property_style.dart:125; wells, controls (:42 spatial), layout_card :234, transform_card; history_records (time); ease painters (time) |
| layer affiliation (wide area) | `identityColors[id%7]`; Timeline uses `timelineColor` (+30% grey wash) | :265-273, :339-348 | timeline/paint.dart:208,220,419; overview; inspector.dart:180 (header); depth_desk.dart:253 |
| asset kind | `kind*` | :253-261 | tile.dart:46; media_shelf.dart:417 |
| collections | `EditorInk.collectionColors` (7) | :111-119 | browser.dart, rail_collections, frame_grid, filter_library (static) |
| camera line | `EditorInk.camera` #8ed9e6 | :104 | stage/overlay, depth_desk |
| error / warning | `error` #ff8899 | :233 | editor_window, notes_desk, export_controls, effects_shelf, colors_shelf, fields.dart… (no separate warning token) |
| ease desk | easePaper #d2d2d2 (a **light** surface), easeInk, easeTime #854515 | :106-108 | ease_desk/* (static `EditorInk.dark`) |

### Collision check against the 09-20 finding ("6 colours, 3 meanings")
**The finding still holds, and the collision is now wider.** The hex values changed after the Chromatic retune, but `identityColors[0..5]` is still exactly `[spatial, seed, time, count, amount, angle]`.

| hex | family | layer # | kind | other |
|---|---|---|---|---|
| #819fff | spatial | 0 | **Shape and Image** | **Browser selection border** (tile.dart:177), collection drop hover |
| #ffdf56 | seed | 1 | Text | – |
| #c08ee4 | time | 2 | Other | – |
| #60cedb | count | 3 | Path | ≈ `tab` #59c9df (cyan: active tab, selection) |
| #f5ad79 | amount | 4 | – | ≈ `accent` #ffbc53 (orange: on, keys, caret) |
| #a0d292 | angle | 5 | Audio | – |
| #ef87ae | – | 6 | Video | – |

In total, six hues carry up to four meanings (family, layer, kind, selection).

**Other collisions:**
- **Selection has five visual encodings:**
  - `accent` orange: buttons and filters.
  - `tab` cyan: tabs and the ladder.
  - `select` pale cyan: menus and the row-drop guide.
  - `spatial` blue: browser tiles.
  - a white lift: the timeline selection region.
- **`accent` means "on", "keyframe/playhead", "marker", "marquee", "drop target" and "caret" all at once.**
- `animate` #5396ff, which marks keys and the playhead while animating, sits close to the spatial blue.
- `tab` and `select` are near the `count` cyan and `EditorInk.camera` #8ed9e6.
- `collectionColors` is a fourth, independent rainbow that again overlaps the same hue ring.
- product-contract.md:24 says "選択、有効状態、所属を混同しない" (do not confuse selection, enabled state and affiliation). The code still confuses them in the ways listed above.

---

## 5. Fonts

- **Bundled** (`pubspec.yaml`, `assets/fonts/`): only **Inter** Regular (400), Medium (500) and SemiBold (600), with an OFL notice file. MaterialIcons comes through `uses-material-design: true` for `Glyph`.
- **Used:** Inter everywhere through `EditorTheme.text` or an explicit `fontFamily`. Separately, `rich_text_editor.dart:52,75` and `fonts_shelf.dart:243` preview document fonts, which are user content.
- **No mono font** is bundled or referenced anywhere in lib/. On macOS a system mono (SF Mono, Menlo) could be named without bundling, but widget tests would not render it faithfully. Bundling one (for example JetBrains Mono, IBM Plex Mono or Inter's own `tnum`) would add a pubspec asset.
- **Tabular figures are used in only 3 places:** `numeric.dart:418` (typed edit), `numeric.dart:551` (ladder) and `numeric_paint.dart:49` (the ladder's label style). The resting value text inside the numeric well, the timeline ruler and frame labels (`timeline/ink.dart` `label()`), the timecode, the percent and zoom fields, and history do **not** set `FontFeature.tabularFigures()`. Inter supports `tnum`, so adding it is a style change and needs no new asset.
- **The type scale** is 8/9/10/13 (plus 12, 22 and 23 in spots), with a base weight of 500 and 600 for headers. It was retuned by the user on 09-20 (chromatic doc: 本文10・補助9・最小8, i.e. body 10, secondary 9, smallest 8). Kickers are uppercase with letterSpacing .5 or 1.

---

## 6. Recommendation (minimal, no broad refactor)

**Keep Classic untouched.** Classic keeps `EditorTheme.chromatic` and user JSON themes, the static `EditorMetrics`, and every current widget. Do not add keys to `EditorTheme.colors`, because that would change the public theme-file schema and `Copy JSON` output.

**Add one new `ThemeExtension<ShellTokens>`** in a new file, for example `foundation/shell_tokens.dart`. The New UI shell installs it by wrapping its subtree in `Theme(data: Theme.of(ctx).copyWith(extensions: [newUiEditorTheme, ShellTokens.dark]))`.
- `newUiEditorTheme = EditorTheme.chromatic.copyWith(colors: {...}, identityColors: [...], drawing: ...)`.
- Because the extension is re-provided under the shell, every existing leaf and panel control (`EditorTheme.of`) automatically picks up the New UI neutrals, with no code change.
- Classic never installs `ShellTokens`. `ShellTokens.of(ctx)` should fall back to a Classic-equivalent default, or assert, so a Classic widget cannot change look by accident.

**Minimal token names for `ShellTokens`:**
- **Surface:** `ground`, `surface`, `surfaceRaised`, `surfaceSunken`, `rule`, `ruleStrong`, `ink`, `inkMuted`, `inkFaint`.
- **Semantic accents, each with exactly one meaning:**
  - `accentPink`, `accentMint`, `accentSky`, `accentLemon`, `accentPeach`, `accentLavender`.
  - A role map on top of them: `selection`, `focus`/`activePanel`, `keyframe`, `playhead`, `onState`, `warning`, `error`, `dropTarget`.
  - Keep **layer identity** (`trackColors`, a separate deeper/desaturated band, per the 09-20 plan) and **property families** (`familySpatial`…`familyAngle`) as two distinct lists, so the §4 collision is not re-created.
- **Geometry:** `radiusNone` 0, `radius` 2, `radiusMax` 3; `ruleWidth` 1, `ruleWidthStrong` 2; `rowHeight`, `controlHeight`, `gutter`, `cellGap`. The values can reuse `EditorMetrics` consts, but they need semantic names.
- **Type** (a small named set of `TextStyle`s, not a `TextTheme`):
  - `label` (Inter 10/500).
  - `labelStrong` (10/600).
  - `kicker` (8 or 9/600, uppercase, letterSpacing 0.8).
  - `readout` (mono or Inter + `tabularFigures`, 10/500).
  - `readoutSmall` (9).
  - `title` (13/600).
- **State alphas:** `hoverWash`, `pressWash`, `disabledOpacity`, `dimOpacity`, replacing the ad-hoc .28/.4/.45/.55 literals in new code only.

**Leaks to watch.** The New UI must pass `EditorTheme.of(ctx)` explicitly to every painter it builds, because 12 painters default to `EditorTheme.chromatic` or `EditorInk.dark`. The ease desk and `filter_library` read `EditorInk.dark` statically, so they stay Classic-coloured unless they are touched, and `keyAccent` depends on the global `EditorTheme.animating`. None of this needs a Classic change as long as the New UI either reuses those panels as-is or wraps its own painters.

**Mono.** Decide whether to bundle a mono font (a pubspec asset, which affects all builds) or to use Inter with `tabularFigures` for `readout` first. The second option has zero asset cost and satisfies "tabular numbers を優先" (prioritise tabular numbers). Mono for readouts could be trialled later.
