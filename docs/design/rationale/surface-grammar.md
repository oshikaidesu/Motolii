# Surface Grammar: one canon for how the product window looks

**Canon: `motolii/ui/lib/theme/metrics.dart`.** Every size, gap, radius, text size and grey of the product window is named there; a panel names a token and never picks a number. Retune in that file, not in panels. (Flutter does layout, input, text, focus, scroll and rendering; Motolii owns this grammar.)

## Tokens and what they mean
Every dimension token is a **derived value**: `f(base, UI Scale, policy)` read at build time. The numbers below are the base at UI Scale 100 %.
- `Surface` density: `topBar 32`, `namedHeader 28`, `chromeRow 24`, `workRow 20`, `control 18`, `controlHero 22`, `ruler 20`, `menuRow 22`, `mark 5`, `glyph 10`, and the stacked cell's `labelRow/cellGap`. A row token is a height: who sits in which row is the token's name.
- `Surface` spacing and shape: `labelGap 2`, `inlineGap 3`, `sectionGap 6`, `panelInset 9`; `controlRadius 3`, `faceRadius 4.5`.
- `Surface` face density: `faceRow 32`, `faceTile 40` (a face is sized by its owner and kept readable).
- `Surface` hairlines: `hair 1`, `focusStroke 1.5`; `hit 24` (the pointer target, never below `hitFloor 18`).
- `Surface` levels (colours by role): `base`, `raised`, `hover`, `selected`, `divider`, `dividerFine`, `well`, `ink`, `muted`, `disabled`. They are steps of the grey ramp `N` (`theme/neutral.dart`, `g10` = 10 % lightness = the ground).
- `Dn` type roles: `nameSize 11` (body), `labelSize 10`, `microSize 9.5` (tiny), `numericSize 11`; the line box is the font size times `Dn.leading` (1.0) or `Dn.leadingText` (1.1, Browser helpers): a ratio, so baseline and paddings keep their relation at any scale.
- Semantic colour is not a surface level: `H` (`theme/identity.dart`: a property's hue, play, record, relation, guide), `desks/parts.dart` and `inspector/tones.dart` palettes.
- `EditorTheme` (`theme/editor_theme.dart`) is the `ThemeExtension` the shared controls in `lib/controls` read. `Step` is gone.
- `Surface.px(n)` is a custom dimension (SCALE policy): the registered escape hatch. Reach in this order: Surface role, Dn role, N step or Surface level, `Surface.px(n)` with `// surface: <reason>`, a new token in `metrics.dart` after asking the owner.

## UI Scale
A user setting: an integer percent (provisional range 50..200 in `UiScale`), 1 % steps, default 100, an **application** preference (`uiScalePercent` in the window's saved settings; the old `hfScale` fraction is read once), never part of a project, the same for every project. Set it from the top bar's `UI` key (or Cmd+,), Cmd+Option +/-/0, or the percent field; it takes effect at once.
- Policies: SCALE (base x scale; fractional logical pixels are fine, layout is not rounded), SNAP (hairline, border, divider: scaled then snapped to whole physical pixels of the view), MINIMUM (hit bounds never below their floor), CLAMP (a face stays between 3/4 and 1.5x of its base), FIXED (the work: Stage composition coordinates, artwork, document, export, time zoom; outside the scale).
- Mechanism: tokens are read when a widget builds; on a change the root (`UiScaleScope`, `app/ui_scale.dart`) marks every element dirty like a hot reload does, so each widget builds again with its State kept (scroll, focus, text input) and render objects lay out and paint again. Nothing is transformed: there is no `Transform.scale`, no re-keyed tree. The Stage's native view is asked for at its logical size times the device's pixel ratio.

## Work density vs face density
Work surfaces (Timeline, Inspector property rows, toolbars, menus, lists, numeric and relation editing) are maximised: `workRow 20`, `control 18`, the label inline to the left of its control, sections separated by a line or a level change, never a card. Face surfaces (Media Browser, Effects, Create, Colors, Fonts, presets) get the area they need to be read, sized by their own owner (Media's plates, Create's marks) and CLAMPed; `workRow` does not apply to them.

## CLI police (`motolii/ui/tool/motolii_lints`, run by `scripts/motolii-ui.sh test`)
`dart run bin/check.dart ../../lib --ratchet=baseline.txt` prints "Motolii UI Grammar": a line per area (typography, colours, spacing, geometry, radius, scaling, housing), each violation as `path:line` with the rule and the canonical family to use, totals, the exception count and the canonical coverage (tokens / all UI dimension occurrences, painters and fixed excluded). Exit 1 on a new violation or exception.
- Violations: a raw size, gap, radius, font size, line extent or named dimension constant (`raw_dimension`); a literal colour outside the palette files (`raw_color`); a Material import; `Card(` or a box with both `border` and `borderRadius` (`card_housing`); arbitrary visual scaling (`Transform.scale`, `ScaleTransition`, a text scaler, a scaled viewport).
- Exceptions: `Surface.px(...)`. Counted per file in `baseline.txt` (`px <path> <total> <without a reason>`) and ratcheted down; a new file holds none; each new one says `// surface: <reason of at least eight characters>`. Geometry that belongs to the work is excused the same way; a painter's or canvas function's own pixels by `// surface-block: <reason>` above the class or function.
- `check.dart --fix` is the codemod: a literal of a token's value and role becomes that token, anything else `Surface.px(n)`.
- Baseline: lower a line by fixing the file, then `dart run bin/check.dart ../../lib --write-baseline=baseline.txt`; it refuses when any count would rise.

## Status: the values are provisional
The base values above are not yet derived from measurement. They were set by impression against the real window and Ableton Live as a density oracle (a 2026-09-29 pass at a MacBook's physical size). The Ableton / Motolii measurement pass has not been made in this repository (its working measurements were kept outside it and are not part of the tree). Until that pass lands, **do not retune values by impression**; change a value only with a measurement, and only in `metrics.dart`. What is firm is the structure: one canon, named roles, the ratchet, the two densities.
