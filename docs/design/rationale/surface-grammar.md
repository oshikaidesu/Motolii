# Surface Grammar: one canon for how the product window looks

**Canon: `motolii/ui/lib/theme/metrics.dart`.** Every size, gap, radius, text size and grey of the product window is named there; a panel names a token and never picks a number. Retune in that file, not in panels. (Flutter does layout, input, text, focus, scroll and rendering; Motolii owns this grammar.)

## Tokens and what they mean
- `Surface` geometry: `topBar 32`, `namedHeader 28`, `chromeRow 24`, `workRow 20`, `control 18`, `controlHero 22`, `hit 24`, `menuRow 22`, `mark 5`, `glyph 10`, `hair 1`, `focusStroke 1.5`, and the stacked cell's `labelRow/labelGap/cellGap`. A row token is a height: who sits in which row is the token's name.
- `Surface` spacing and shape: `inlineGap 3`, `sectionGap 6`, `panelInset 9`; `controlRadius 3`, `faceRadius 4.5`.
- `Surface` levels (colours by role): `base`, `raised`, `hover`, `selected`, `divider`, `dividerFine`, `well`, `ink`, `muted`, `disabled`. They are steps of the grey ramp `N` (`theme/neutral.dart`, `g10` = 10 % lightness = the ground).
- `Dn` type roles: `nameSize 11`, `labelSize 10`, `microSize 9.5` (sans and mono share them).
- Semantic colour is not a surface level: `H` (`theme/identity.dart`: a property's hue, play, record, relation, guide), `desks/parts.dart` and `inspector/tones.dart` palettes. Colour means identity, state or selection.
- `EditorTheme` (`theme/editor_theme.dart`) is kept as the `ThemeExtension` the shared controls in `lib/controls` read; `Step` (metrics.dart) names the rungs those controls and the Stage chrome stand on. Do not add to `Step`.
- Reach in this order: Surface role, Dn role, N step or Surface level, Step (shared controls only), a new token in `metrics.dart` after asking the owner.

## Work density vs face density
Work surfaces (Timeline, Inspector property rows, toolbars, menus, lists, numeric and relation editing) are maximised: `workRow 20`, `control 18`, the label inline to the left of its control, sections separated by a line or a level change, never a card. Face surfaces (Media Browser, Effects, Create, Colors, Fonts, presets) get the area they need to be read, sized by their own owner (Media's plates, Create's marks); `workRow` does not apply to them. There is no global scale; `LiveUiScale` is the user's own choice on top.

## Lint ratchet (`motolii/ui/tool/motolii_lints`, run by `scripts/motolii-ui.sh check`)
- `raw_dimension`: a numeric size, gap, radius or text size in a widget is a finding; the quick fix reads `Surface`/`Dn`. `raw_color`: a literal colour outside the palette files. `material_import`: a stock Material surface. `card_housing`: `Card(` or a `BoxDecoration` carrying both `border` and `borderRadius` (a section is a line or a level, not a box).
- Excuse: `// surface: <reason>` on the line or the line above, a reason of at least eight characters, says why a token cannot express it (geometry that belongs to the work: Stage, frames, thumbnails, painter and timeline reference coordinates, animation). A bare `// surface:` excuses nothing.
- Baseline: `tool/motolii_lints/baseline.txt` lists, per file, the raw styling that predates the grammar. A file may not exceed its line and a file not listed must be clean. Lower a line by fixing the file, then `dart run bin/check.dart ../../lib --write-baseline=baseline.txt`; never raise one.

## Status: the values are provisional
The numbers above are not yet derived from measurement. They were set by impression against the real window and Ableton Live as a density oracle (a 2026-09-29 pass at a MacBook's physical size). The Ableton / Motolii measurement pass has not been made in this repository (its working measurements were kept outside it and are not part of the tree). Until that pass lands, **do not retune values by impression**; change a value only with a measurement, and only in `metrics.dart`. What is firm is the structure: one canon, named roles, the ratchet, the two densities.
