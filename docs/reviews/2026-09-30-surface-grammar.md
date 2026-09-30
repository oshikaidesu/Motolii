# Surface Grammar: one token source for the product window's look

Product window = `motolii/ui/lib/live_hf` over `lib/hf`. Its scale is `lib/hf/metrics.dart` (`Surface` geometry/spacing/shape/levels, `Dn` type roles) and `hf/neutral.dart` (`N` ramp). `foundation/metrics.dart` `EditorMetrics` belongs to Classic/New (to be retired); it is not the product's scale, so no third scale was added: `UiMetrics` became `Surface`.

## Gate
Flutter `ThemeData`/`ThemeExtension` (already the decision of 2026-09-08 in decision-index), Style Dictionary-style token files, Mix / flutter_design_tokens / dimensions_theme (rejected 2026-09-08: stagnant or shelf replacement), existing `tool/motolii_lints` (analyzer plugin + `bin/check.dart`). Decision: reuse the lint; tokens stay Dart constants. No CSS engine, no parser.

## Scattered constants found (hf + live_hf)
- text sizes: `sans/mono/caps(size)` 203 calls: 11 x99, 9.5 x53, 10 x40, outliers 13.5 x5, 16.5 x3, 12, 13, 10.5; `H.s` 30 calls. -> `Dn.nameSize/labelSize/microSize` (value-preserving).
- spacing literals: SizedBox 242 uses over 63 distinct values (2, 6, 4.5, 7.5, 9, 3, 10.5 dominate); EdgeInsets 271 uses over ~150 forms; radii 160 uses over 14 values (2, 3, 4, 4.5, 1.5).
- one colour under many names: `N.g13` = `H.raised` = `kRaised` = `kTile` (x2 definitions); `N.g15` = `H.raisedHi`/`H.rule2`/`H.track`/`kRule2`/`kRaisedHi`/`kTileHi`; `N.g20` = `H.sel`/`H.rule`/`kRule`/`kSel`; `N.g10` = `H.window`/`kGround`; `N.g56` = `kMuted`/`kMutedTone`; `N.g95` = `H.text`/`kInk`; `N.g07` = `kWell`.

## Surface tokens (few)
Geometry: `topBar 32, namedHeader 28, chromeRow 24, workRow 20, control 18, controlHero 22, hit 24, menuRow 22, mark 5, glyph 10, hair 1, focusStroke 1.5` (+ the stacked cell's `labelRow/labelGap/cellGap`, being retired). Spacing: `inlineGap 3, sectionGap 6, panelInset 9`. Shape: `controlRadius 3, faceRadius 4.5` (no panel/section radius: no cards). Levels: `base, raised, hover, selected, divider, dividerFine, well, disabled, muted, ink`. Type: `Dn` 11/10/9.5. Identity/keyed/selection colours stay with their owners (`desk/common`, `insp/tones`) until they converge.
Deleted duplicate names (45 files): `kRule kRule2 kGround kRaised kRaisedHi kSel kMuted` (bp/common), `kTile` x2, `kTileHi`, `kInk kWell`, `kMutedTone`, `rowStd/rowTight/rowHead`; `UiMetrics.{pad,gap,tight}` renamed `panelInset/sectionGap/inlineGap`.

## Lint (extends `tool/motolii_lints`; no second system)
- `raw_dimension`: also `sans/mono/caps(<number>)`; `hf/metrics.dart`, `hf/neutral.dart` are scale files; quick fix and `--fix` read `Surface` for `hf/` and `live_hf/`, `EditorMetrics` elsewhere.
- `raw_color`: palette files (`hf/shell/place.dart`, `hf/desk/common.dart`, `hf/insp/tones.dart`) may name colours.
- `card_housing` (new): `Card(` and a `BoxDecoration` with both `border` and `borderRadius`.
- Exceptions: `// surface: <reason, >= 8 chars>` on the line or the line above; `// surface-file: <reason>` in the first lines. A bare comment excuses nothing.
- Ratchet: `tool/motolii_lints/baseline.txt` (2,097 raw stylings in 101 files today). A file may not exceed its line; a file not listed must be clean; fixing lowers the line. `motolii-ui.sh test` runs `check.dart lib --ratchet=baseline.txt`.

## Vertical slice: the Glow effect (effect card -> work rows)
Files: `hf/insp/panel.dart` (`ParamCell(inline)`, `ParamSheet`), `live_hf/adapters/effects_card.dart`, `right_seat.dart`. No new widget; existing Toys + tokens. A card (margin, radius, border, padding, label-over-gap-over-control) became a band on the raised level over a body on the base, a hairline between, label left / control right at `workRow`; a single row keeps the two-up label column so values align.
| 500x700 story, same scene | before | after |
|---|---|---|
| Glow (5 params) | 166 px | 5 rows in 3 lines: 106 px in the real-app audit (174 -> 106) |
| Rounded Corners | 77 px | 46 px |
| Effects fully inside 700 px (Hue/Sat, Glow, Blur) | Hue/Sat + Glow + part of Blur | all three, ending at y = 605 |
Real-app audit (`integration_test/surface_audit_test.dart`, `SURFACE_AUDIT=1`; region 381x513 of the Inspector seat): controls fully visible 12 -> 16; param cells 3 -> 6; empty (housing, gaps, padding) area 49.0% -> 44.0%; full-width empty rows 158 -> 137 of 513 px.
Screens: `/tmp/shots_before`, `/tmp/shots_after`, `/tmp/live_w1.png` (before), `/tmp/live_after.png` (after).

## Housing tax (where the Inspector's area goes, real app, before)
control 17.6%, text outside controls 15.6%, the transform gizmo 17.7% (267x129), empty 49.0%. A stacked parameter cell is 11 (label) + 2 + 22 (hero control) + 3 = 35 px for one number, where the control itself is 18-22 px: label/control separation is ~40% of each cell. A card costs a 6 px margin, a 22.5 px header row, 6 px padding twice, and a border on top of that. Controls are already 18-22 px; the gizmo and the cards and the stacked form layout are why it still feels larger than Ableton. The Transform block and its gizmo are untouched (Face density: the gizmo is a face).

## Work density vs Face density
Work: Timeline, Inspector property rows, toolbars, menus, lists, numeric/relation editing: `workRow 20`, `control 18`, inline label. Face: Media Browser, Effects shelf, Create, Colors, Fonts, presets: sized by their own owner (plates, marks) to be read; `workRow` does not apply. No global scale change.

## Not done (next)
Convert the remaining ~1,250 raw values in hf/live_hf file by file (the ratchet only stops growth); Transform block rows and the gizmo; timeline lane constants (`tlPitch 23, tlRowH 22, tlBarH 18`, reference coordinates); identity/keyed/selection colour convergence; `H.*` palette aliases in `place.dart`.
