# UI vertical slice: use Flutter as the UI framework

Follows `2026-09-30-ui-technology-acquisition-audit.md`. A minimal production slice, not a refactor.

## Acquisition gate (re-run)

| Need | Flutter standard | Package | Decision |
|---|---|---|---|
| Lazy list of assets | `ListView.builder` (`itemExtent`, `cacheExtent`) | none needed | Reuse |
| Lazy thumbnail grid with mixed aspect | `SliverGrid` + custom `SliverGridDelegate` / `SliverGridLayout` (the layout *positions* are Motolii's; the laziness is Flutter's) | `flutter_staggered_grid_view` would replace ~90 lines but the positions must stay bit-identical to `FluidBoard.thumbnail` for List↔Thumbnail identity; not adopted | Extend (standard sliver API) |
| Keys / shortcuts | `Shortcuts` → `Actions` → `Intent`, `DismissIntent` | none | Reuse |
| Widget skin | `ThemeData` + `ThemeExtension` + `WidgetStatesController` | none | Reuse |

## Returned to Flutter

- Media Browser List and Thumbnail: one widget per visible asset (plus `cacheExtent`), not one per asset. `MediaListView`, `MediaLibraryBody`, `_Board`, `MaterialCard` and the list `_Face` were deleted (net code removed in `media_library.dart`, `media_list.dart`).
- Media Browser keys (`MediaBrowserState._key`, a ~60-line if-chain) → `Shortcuts`/`Actions`/`Intent`. Esc is the framework's `DismissIntent`; an Action that is not enabled lets the key through (text fields keep their typing), which replaces the hand-written `_typing` guard and the modifier tests (`SingleActivator` already means "no other modifier").
- Compact density: standard Material widgets reach 20 px through Theme alone (Density 3 story, `density_compact_theme_test`).

## Still Motolii

Asset identity across List / Thumbnail / Explore (`ValueKey('face-id')`, carried faces), selection model, Place, Source/Favorites/Recent, the view-switch continuity rule (identity + perceptual continuity), the input-immediate motion rule, Explore neighbourhood layout.

**Explore's 300 is `CURRENT LIMITATION` of the current implementation** (force layout O(n²)), not a product requirement. It is not in Document, public contract or UI constraints; List/Thumbnail do not depend on it. Continuity across views is *not* "every widget alive at once": it is implemented with visible items + small overscan + an overlay of only in-sight faces.

## Thin wrappers (and why)

- `_PlacedGrid` / `_PlacedLayout` (media_fluid.dart): `SliverGrid` has no built-in layout for Motolii's precomputed rectangles; the standard delegate API is the extension point (min/max child index by scroll offset from running extrema).
- `_Do<T extends Intent>`: `CallbackAction` has no `isEnabled`; 12 lines.
- `SegmentedButton(showSelectedIcon: false)` per widget: `SegmentedButtonThemeData` has no such field in this SDK, so the Theme cannot remove the check icon that overflows at 20 px.

## Failed to borrow (measured)

- `ListTileThemeData` has no hover colour; hover comes from `ThemeData.hoverColor` (global). A list row's hover state cannot be forced in a still (no `WidgetStatesController` on `ListTile`).
- `VisualDensity` subtracts from `minimumSize`: buttons went 20 → 12 px until density was left standard with explicit `minimumSize`. `Switch` cannot go below 40 px.
- The shell is a bare `WidgetsApp`; Material widgets need `MaterialApp` (`MaterialLocalizations`) around them.
- Shortcuts fire on `KeyRepeatEvent`; the old code handled `KeyDownEvent` only. Holding Enter now repeats Place (arrow repeat is wanted; the one-shot ops are a behaviour difference, see report).
- Explore ↔ List/Thumbnail cannot glide (Explore is an unbounded canvas); it snaps.

## Skin by Theme (MotoliiTheme spike)

`explorer/lib/stories/density.dart`, story "Density 4": normal / hover / pressed / focus / disabled button, text focus / disabled, checkbox on / off / disabled, segmented selected, row normal / selected, keyed lamp + identity colour (`MotoliiMarks` `ThemeExtension`), section boundary (`DividerTheme`). No own Button/TextField/Dropdown. Sizes unchanged (20 px rows).

## hf/ classification (no deletion)

| Part | Lines | Class |
|---|---|---|
| `hf/neutral.dart`, `hf/metrics.dart` | 42, 62 | visual skin (palette, rhythm) → ThemeData values |
| `hf/glyphs.dart` | 192 | visual skin / icon set |
| `hf/dock/theme.dart` | 21 | visual skin |
| `hf/bp/*` (Create/Browser faces, color cards, fonts, effects, shelf) | 3,787 | Motolii semantics (what a shelf/asset/effect *is*) mixed with skin; search/list widgets inside are returnable candidates |
| `hf/insp/*` (transform/camera/layout models, gizmo, toys, rows) | 3,274 | `*_model.dart`, gizmo, layout diagram = Motolii semantics; `rows.dart`/`slot.dart` row chrome = returnable to ListTile/Theme |
| `hf/desk/*` (Ease, Blend, Depth, Notes, History) | 2,917 | Motolii semantics (ease curve editor, blend) + skin (`*_skin.dart`) |
| `hf/shell/place.dart` | 165 | fixed-coordinate ruler of the 1536×1024 reference: replacement candidate; used by 12 files. **Not** a statement that Stage/Timeline/Inspector placement is arbitrary: the workspace arrangement (Browser\|Stage\|Inspector over Timeline\|Desk) is Motolii meaning and stays |
| `hf/shell/timeline.dart` (`tlTop = 775`, `tlPitch`…) | 562 | Timeline geometry in reference coordinates; replacement candidate for layout constants, the Timeline semantics stay |
| `hf/shell/menu.dart`, `dialog.dart`, `sheet.dart` | | general UI tech (menu/dialog/sheet) → `MenuAnchor`/`showDialog` candidates |
| 40 `onKeyEvent` sites, 5 `Shortcuts` | | general UI tech; Media Browser is converted, 39 remain |
| obsolete duplication | | `proto_hf` copies of the above once live_hf replaces them |

## Real window (production `live_hf`, `it_doc.rrd`)

Normal workspace at 1375×821 and resized to 1000×640: Stage stays the main area; Browser/Inspector keep proportion; Inspector is clipped below "Glow" and scrolls; Timeline ruler loses 00:07+ and the Ease header loses its subtitle at the small size. No overlap or broken fixed coordinates observed at that size. Below roughly this size the reference-coordinate constants (`tlTop`, `place.dart`) are the first to show. Captures: `/tmp/live_w1.png`, `/tmp/live_w2.png`.

## Before / After (same 1,845-item fixture)

| | Before | After |
|---|---|---|
| Widgets built for Thumbnail (headless, n = 1,845) | ≈ 4,167 | ≈ 183 (List ≈ 19), independent of n (n = 10,000: same) |
| Headless first frame | 1,158 ms | ≈ 536 ms (n = 10,000: 391 ms) |
| Real app, worst frame, Thumbnail while indexing | 1,238 ms | 759 ms |
| Real app, worst frame, Thumbnail settled | 785 ms | 560 ms |
| Real app, worst frame, List | 110 ms | 104 ms |
| Real app, worst frame, Explore | 100 ms | 33 ms |
| Frame ≥ 1 s | once | none |

Interaction semantics kept (tests): selection survives scroll out/back; Place unchanged; one face per asset across view change; hover changes no geometry; `media_browser_test` (8), `media_fluid_test` (10), `explore_graph_test` pass.
