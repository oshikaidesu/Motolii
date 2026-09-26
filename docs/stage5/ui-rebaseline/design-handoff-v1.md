# Motolii New UI: Design Handoff v1

Source of truth: `concept/north-star-dark.png` (1536 x 1024). Capability source: Classic Motolii (not a visual source).

This document measures the reference. It does not redesign it.

## 0. How to read this

- **Measurement space.** All pixel values are in the reference image space, 1536 x 1024, origin top-left. Coordinates are inclusive pixel indices.
- **Tags.**
  - `MEASURED`: read directly from pixels. Tolerance is +-1 px or +-2 levels per colour channel.
  - `ESTIMATED`: inferred from pixels (font size from cap height, radius from edge softness). A tolerance range is given.
  - `NOT SPECIFIED BY REFERENCE`: a still image cannot show it.
  - `DERIVED - REQUIRES VALIDATION`: a conservative proposal for something the image does not show.
  - `ARTIFACT`: an image-generation error. Do not reproduce it.
- **Panel widths are not product constants.** The reference widths are recorded so that anatomy can be located. Rules that must hold at every width are in each panel's "Responsive" section.
- **Annotated crops** are in `handoff/`. See section 10.
- **Colour.** Values are sampled from raster pixels. The image is a generation, so the same object varies by a few levels across its surface. Where a variation looks intentional it is recorded as a range.

Rules that apply everywhere:

1. Do not unify components. Unify identity (section 8).
2. One loud thing, held by quiet things. The Stage artwork is the only rich, bright, high-frequency surface.
3. Colour identifies a thing. Neutral surfaces carry structure. White carries information.

## 1. Global visual system

### 1.1 Panel map (reference layout, not requirements)

| Panel | x range | y range | Notes |
|---|---|---|---|
| Top / Transport | 0-1535 | 0-61 | 1 px rule at y=61-62 |
| Browser | 11-333 | 62-~1012 | right rule x=333; full height; footer text at y=999-1007 |
| Stage | 344-1122 | 62-692 | left rule x=344, right rule x=1122 |
| Inspector | 1135-1521 | 62-692 | left rule x=1135, right rule x=1521 |
| Timeline | 345-1521 | 703-993 | spans under Stage AND Inspector |

Gutters between panels, `MEASURED`:

- Browser to Stage: 10 px (x 334-343).
- Stage to Inspector: 12 px (x 1123-1134).
- Stage and Inspector to Timeline: 10 px (y 693-702).
- Left window margin 11 px. Right window margin 13 px. Space below the Timeline is ~30 px (y 994-1023); its purpose is not determinable (`ARTIFACT` suspected).

The Timeline's horizontal extent (under the Inspector) is a layout fact of the reference. It is not derived from panel widths.

### 1.2 Neutral tokens

The window background and the panel background are the same colour. Panels are separated by a 1 px rule and a gutter, not by a fill change.

| Token | Value | Where | Tag |
|---|---|---|---|
| window / panel | `#191919` (gutters `#171717`-`#1A1A1A`) | everywhere | MEASURED |
| raised (neutral tile, input, key) | `#1F1F1F`-`#222222` | Browser tiles, inputs, icon keys | MEASURED |
| stripe (timeline alt row, group row) | `#272727`-`#2C2C2C` | Timeline zebra | MEASURED |
| selected surface, Browser tab / preset | `#2D2D2E`, preset `#292929` | | MEASURED |
| selected surface, Inspector tab / subtab | `#333539` / `#3E4043` (cool grey) | | MEASURED |
| selected surface, Timeline tab | `#2C3344` (blue-tinted) | | MEASURED |
| selected surface, Composition tab | `#2A2A2C` | | MEASURED |
| primary rule (panel outline) | `#343434`-`#3C3C3C` | 1 px | MEASURED |
| secondary rule (inside panel) | `#262626`-`#2E2E2E` | 1 px | MEASURED |
| row separator (Timeline) | `#161616`-`#1C1C1C` | 1 px, darker than fill | MEASURED |
| text primary | `#F0F0F0`-`#FFFFFF` | active labels, values | MEASURED |
| text secondary | `#C0C0C0`-`#DEDEDE` | inactive tabs, row names | MEASURED |
| text tertiary | `#A8A8AC`-`#BBBABC` | annotation, version, ruler | MEASURED |
| text disabled | `NOT SPECIFIED BY REFERENCE` | | |

The four "selected surface" values differ by a cool tint of 0 to +18 in the blue channel. Whether this is one token with a tint or four tokens is undecidable from the image. See section 9.

### 1.3 Semantic identity: colour families

Do not use one RGB per identity. The reference uses different lightness and saturation depending on the job the colour does.

| Identity | Browser surface | Inspector accent (dot, fill) | Timeline body | Timeline name icon | Off / disabled |
|---|---|---|---|---|---|
| Scatter | `#F27AB6` | dot `#F277B4`, slider fill `#F478B6` | `#E276AC` (range `#D977AB`-`#E477AD`) | `#C76295` | NOT SPECIFIED |
| Stagger | `#5596E9` | dot `#3088ED` | `#4F7CB3` (range `#4B78AD`-`#537DAE`) | `#477FBD` | NOT SPECIFIED |
| Along Path | `#7BCBA3` | dot `#72C99E` | `#7BB98C` (range `#6FB386`-`#7BB98C`) | `#517F68` | NOT SPECIFIED |
| Face (Rotation in Timeline) | `#F0D455` | dot `#F6D112` | `#D2BE56` (range `#CEBA52`-`#D4C257`) | `#E3C748` | Inspector row dot stays yellow; row toggle is off |
| Follow | `#F69260` | not shown | not shown | not shown | |
| Attach (Scale Variation in Timeline) | `#A282E8` | not shown | `#A988E9` | `#A086E2` | |

Reading the table:

- The Browser surface is the most saturated and brightest. It is a big area, so it must be readable as a swatch.
- The Timeline body is darker and less saturated than the Browser surface for blue and green (blue `#5596E9` to `#4F7CB3`, green `#7BCBA3` to `#7BB98C`). Pink, yellow and purple change by less than 12 levels. Do not assume a fixed ratio.
- The Inspector dot is the most saturated of all because it is small.
- Names in the Timeline differ from Browser names for two rows: the reference labels the yellow row "Rotation" and the purple row "Scale Variation". Section 9 lists this as an open question.
- Inspector slider fill has two visible values: Density `#F478B6` (full) and Spread `#9B4A6F` (about 62% of full). See 5.6.

### 1.4 Operational colours

| State | Value | Where | Tag |
|---|---|---|---|
| Play key | fill `#7BCC9E`, glyph `#040709` | Top | MEASURED |
| Stop key | fill `#1C1C1C`, border `#363636`, glyph `#969697` | Top | MEASURED |
| Record key | fill `#1F1F1F`, dot `#F03C8A` | Top | MEASURED |
| Active mode (EDIT) | fill `#7A87E3`, label near-white | Top | MEASURED |
| Toggle on | track `#6982D1`, knob `#FCFCFE` | Inspector | MEASURED |
| Toggle off | track `#999BA0`, knob `#FDFBFD` | Inspector | MEASURED |
| Playhead | line `#6BA7D7`-`#7FB3F4`, handle `#7FB3F4` | Timeline | MEASURED |
| Selection | see selected surface tokens | | |
| Focus ring | NOT SPECIFIED BY REFERENCE | | |

Observations:

- Record is `#F03C8A`, close to the Scatter family `#F27AB6` (same hue region, more saturated, darker). A reader can confuse them. This is recorded as an open question, not corrected.
- The playhead blue (`#6EA6DB` family) is not the same as the EDIT violet-blue (`#7A87E3`) or Stagger blue (`#5596E9`).
- Play is mint green (`#7BCC9E`), close to the Along Path surface (`#7BCBA3`). Same observation.

### 1.5 Typography roles

Font families in the reference: a geometric-grotesque sans for names and labels, and a monospace for readouts and annotations. The reference is raster, so exact families are `ESTIMATED`. Sizes come from cap height (sans cap ratio about 0.72, mono about 0.70).

| Role | Family | Size | Weight | Case | Tracking | Colour | Measured cap height |
|---|---|---|---|---|---|---|---|
| Brand (Motolii) | sans | 28-30 px | 600 | Title | about -0.02 em | `#EDEEEE` | ~21 |
| Tagline (top left/right) | mono | 11 px | 400 | Sentence | 0 | `#DBDCDC` / `#BEC0C0` | ~7-8, 2 lines in 24 px |
| Panel navigation (tabs) | sans | 12-12.5 px | 600 | UPPER | +0.05 to +0.08 em | active `#FDFDFD`, inactive `#DDE0E0` | 9 |
| Section heading (PRIMITIVES...) | sans | 12-12.5 px | 600 | UPPER | +0.06 em | `#EAEBEC` | 10 |
| Item name (tiles, presets, rows) | sans | 13 px | 400 | Title | 0 | `#D6D7D8`-`#DADCDD` | 9-10 |
| Object name (Inspector field) | sans | 15 px | 500 | Title | 0 | `#FCFCFD` | 11 |
| Object name (Timeline group) | sans | 13 px | 500-600 | Title | 0 | `#F8FAFA` | 9 |
| Relation title (open, Inspector) | sans | 19-20 px | 600 | Title | about -0.01 em | `#FDFDFD` | 14 |
| Relation row name (folded) | sans | 16 px | 500 | Title | 0 | `#F7F8F8` | ~11.5 |
| Secondary metadata | sans | 11-12 px | 400 | Sentence | 0 | `#B5B6B5` | 11 (kana) |
| Parameter label (Density) | sans | 12-12.5 px | 500 | Title | 0 | `#EEEFEE` | 8.5-9 |
| Numeric readout (0.72) | mono | 12 px | 400 | | 0 | `#F0EFEF` | 9 (digit) |
| Top readouts (120.00, 00:02:13) | mono | 14 px | 400 | | 0 | `#F0F1F0` | 10 (digit) |
| Timeline ruler | mono | 12 px | 400 | | 0 | `#BDBEC0` | 9 (digit) |
| Timeline row name | sans | 13 px | 400 | Title | 0 | `#DDDEDE` | 9 |
| Annotation on Stage ("Jewel Field", "ARRANGE...") | mono | 13-14 px | 400 | mixed | +0.05 em on the upper-case list | `#E0E1E4` / `#A8A9AC` | ~10 |
| Version (v0.5.0) | mono | 12 px | 400 | | 0 | `#BBBABC` | 9 |

Notes:

- Tabs, section headings and mode labels (EDIT, PLAY, EXPORT) are upper-case sans with positive tracking. They are not monospace in the reference.
- Monospace appears for: top readouts, taglines, the Timeline ruler, numeric values inside the Inspector, the version string, and text that is part of the artwork.
- Line height for two-line blocks: the tagline is 24 px for two lines (pitch about 13-14 px, `ESTIMATED`).
- A single CJK sub-label ("点を散らして、広がりをつくる") appears under the Scatter title at 11-12 px. Window text is English by project rule; this line is a generation of the reference and is recorded as an `ARTIFACT` (section 8).

### 1.6 Geometry primitives

| Primitive | Radius | Border | Tag |
|---|---|---|---|
| Neutral tile, key, icon button | 3-4 px | 1 px `#2F2F2F`-`#363737` | ESTIMATED radius (range 2-5) |
| Relation tile (coloured) | 3-4 px | 1 px, 15-25 levels darker than fill, no light rim | ESTIMATED |
| Input field | 3 px | 1 px `#252525` | ESTIMATED |
| Slider track | full (height 4-5 px) | none | ESTIMATED |
| Toggle | full pill, 37 x 20, knob 16 | none | MEASURED size |
| Tabs | 0-2 px | 1 px | ESTIMATED |
| Scatter card | 3-4 px | 1 px `#2F2F2F` bottom, `#1D1D1D` top | ESTIMATED |
| Bars in Timeline | 2-3 px | none, 1 px dark outline | ESTIMATED |

Shadows: none visible. Gradients on flat fills: none. Depth is by fill difference and 1 px rules.

### 1.7 States policy

The reference is a still. For each control the tables below use `idle` (visible), `selected` (visible), `active` (visible only where noted), and mark the rest.

- Hover: NOT SPECIFIED BY REFERENCE for every control. A conservative derived state is "surface +6 to +8 luminance levels". `DERIVED - REQUIRES VALIDATION`.
- Focus: NOT SPECIFIED BY REFERENCE for every control.
- Disabled: only one is visible (Face Target toggle off, dot still yellow). For everything else NOT SPECIFIED BY REFERENCE.

### 1.8 Responsive policy (applies to every panel)

Panels are user-resizable. The rules are intrinsic:

- Icons keep their visual size. They do not shrink.
- Text is truncated with a clip, not ellipsis, before it wraps. Names are not wrapped.
- Reflow before shrink: put precision controls below their visualizer before making the visualizer smaller than its useful size (the visualizer floor is in 5.7).
- Fold before hide: collapse secondary metadata to nothing before collapsing primary controls.
- Scroll when a component would otherwise be crushed.
- Each panel section states the verb for each element: `SCALE`, `REFLOW`, `FOLD`, `HIDE`, `SCROLL`.

## 2. Top / Transport

### A. Anatomy (left to right, reference positions)

1. Wordmark "Motolii" (x 19-115, y 21-42).
2. Tagline left, two lines (x 152-253, y 21-44).
3. Transport keys: play, stop, record (x 363-496).
4. Readouts: `120.00`, `4 / 4`, `00:02:13`, plus a small plus glyph (x 533-770).
5. Mode switch: EDIT | PLAY | EXPORT (x 830-1093).
6. Three icon keys (x 1148-1292): fullscreen-like, a pin/marker-like glyph, folder-like.
7. Tagline right, two lines, right-aligned (x 1406-1506).
8. Bottom rule (y 61-62).

### B. Dimensions

| Element | Value | Tag |
|---|---|---|
| Bar height | 61 px + 1 px rule | MEASURED |
| Key height (transport, mode, icon keys) | 37 px (y 14-50) | MEASURED |
| Top and bottom margin of keys | 14 px above, 10 px below (to y=61) | MEASURED |
| Play key | x 363-403, width 41 | MEASURED |
| Stop key | x 410-449, width 40 | MEASURED |
| Record key | x 456-496, width 41 | MEASURED |
| Gap between transport keys | ~6 px (range 5-7) | ESTIMATED |
| Mode switch | x 830-1093, width 264, three segments | MEASURED |
| EDIT segment | x 830-918, width 89 | MEASURED |
| PLAY and EXPORT | share the remaining 175 px | MEASURED |
| Icon keys | x 1148-1187, 1201-1240, 1252-1292; width 40; gaps 12 and 11 | MEASURED |
| Wordmark | 97 x 22 px (cap ~21) | MEASURED |
| Readout digits | height 10 | MEASURED |

### C. Alignment

- Keys share a vertical span y=14-50 and a single centre line y=32.
- Readout text and tagline are centred on the same y=32 line (readout digits y 27-36, centre 31.5).
- Left group is anchored to the left margin (x=19). Right group is right-aligned to x=1506 (right margin 30). The centre group (transport, readouts, mode) is not centred on the window; it starts at x=363 and ends at x=1292 with the mode switch and icon keys after a 55 px gap.
- Separators between readouts are 1 px vertical marks with about 12 px each side.

### D. Typography

Brand, tagline, top readouts as in 1.5. Mode labels: upper-case sans, 12 px, +0.06 em; the active label is near-white on `#7A87E3`, inactive labels are `#EAECEB`-`#DDE0E0` on `#1B1B1B`.

### E. Colour

- Transport play: `#7BCC9E`. Stop and record: neutral key with neutral or record-coloured glyph.
- Mode: active `#7A87E3`; inactive `#1B1B1B` with a 1 px `#343434` outline around all three.
- Icon keys: fill `#1E1E1E`, border `#343636`, glyph `#CCCECE`.

### F. States

| Control | idle | hover | selected/active | disabled | focus |
|---|---|---|---|---|---|
| Play key | mint fill | NOT SPECIFIED | (playing state) NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED |
| Stop key | neutral key | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED |
| Record key | neutral key, pink dot | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED |
| Mode segment | dark | NOT SPECIFIED | violet-blue fill (EDIT visible) | NOT SPECIFIED | NOT SPECIFIED |
| Icon key | neutral key | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED |

Derived (validation required): a playing state where the play key becomes a stop-like toggle; a recording state where the record dot fills the key.

### G. Responsive

- The three groups have different bodies on purpose. Keep them different at every width.
- FOLD order as width decreases: right tagline (HIDE first), left tagline (HIDE), `4 / 4` (HIDE), `120.00` (HIDE), the icon keys (FOLD into one overflow key), then the wordmark shortens to a glyph.
- Never hide: transport keys, the time readout, the mode switch.
- Keys keep their 37 px height (no SCALE).

## 3. Browser

### A. Anatomy

1. Tab strip: OBJECTS (active), RELATIONS, EFFECTS, MEDIA, and a search key (y 71-107).
2. Section PRIMITIVES with a 4-column tile row (Text, Shape, Image, Camera).
3. Section GENERATORS with a 4-column tile row (Repeater, Grid, Circle, Spiral).
4. Section RELATIONS: a 3-column, 2-row block of coloured tiles.
5. Section EFFECTS: a 3-column, 2-row block of neutral tiles.
6. Section PRESETS: a list of five rows, first selected.
7. Footer: a "New Preset" row and a version string.

The five item types are **not** one component. See the item table below.

### B. Dimensions

Tab strip, `MEASURED`:

| Element | Value |
|---|---|
| Top padding above tabs | 8 px (y 63-70) |
| Active tab | x 11-88, width 78, height 37 (y 71-107) |
| Search key | x 296-331, width 36, height 37 (border at x=296-297) |
| Rule under tabs | y=108 |

Item types, `MEASURED` unless tagged:

| Item | Tile size | Columns, pitch | Icon box | Label | Alignment | Surface |
|---|---|---|---|---|---|---|
| Primitive | 76 x 85 (x 14-89, y 145-229) | 4, pitch 80, gap ~5 | 21 x 21, centred, top offset 22 px | cap 9, baseline 16 px above bottom | centred | neutral `#202020`, border `#313131` |
| Generator | 76 x 85 (y 275-360) | 4, pitch 80, gap ~5 | 21-25, centred | same | centred | neutral |
| Relation | 98 x 80 (x 15-112, y 402-482) | 3, pitch ~105.5, gap ~7; row gap 6 | ~21, left-aligned, left inset ~17, top offset 18 | cap 10, left inset 14, baseline 15 px above bottom | left | family colour, border 15-25 levels darker |
| Effect | ~97 x ~79 (x 15-111, y 597-676) | 3, pitch ~105.5; row gap ~3-4 | 23-24, left-aligned, left inset ~17 | cap 9, left | left | neutral `#202020` |
| Preset row | full width (x 12-327), 30 tall selected, pitch ~29 | list | 14 x 14 rounded square at x 14-26 | cap 9, text starts x=54 | left | selected `#292929`, border `#404041`; others no fill |

Vertical rhythm:

| From | To | Distance | Tag |
|---|---|---|---|
| Tab rule (y=108) | section label cap top (y=122) | 14 | MEASURED |
| Section label cap top | tile top | 23 (label y 122-131, tile y=145) | MEASURED |
| Primitive tile bottom (y=229) | Generator tile top (y=275) | 46, includes the next section label | MEASURED |
| Generators tile bottom (y=360) | Relations tile top (y=402) | 42 | MEASURED |
| Relations block bottom (y=566) | Effects tile top (y=597) | 31 | MEASURED |
| Effects block bottom (y=757) | Presets selected row top (y=794) | 37 | MEASURED |
| Preset rows bottom | New Preset rule (y=943) | ~4 | MEASURED |
| New Preset row | y 943-986, 44 tall | | MEASURED |
| Footer text | y 999-1007 | | MEASURED |

The label spacing above EFFECTS is irregular in the reference (label y 575-589, only ~8 px above the tile row). The other sections have 13 px between label bottom and tile top. See section 8.

### C. Alignment

- Section labels share the left edge x=20.
- Primitive and Generator tile columns: left edges at x = 14, 95, 175, 254 (pitch 80).
- Relation and Effect tile columns: left edges at x = 15, 121, 226 (pitch ~105.5).
- Relation and Effect glyphs and labels share one left edge inside each tile (tile left + ~15-17).
- Preset icons start at x=14, preset text starts at x=54.
- "New Preset" plus glyph at x~24-28, text at x=54 (same text edge as presets).

### D. Typography

- Tabs: upper-case sans 12-12.5 px, 600. Active `#FDFDFD`, inactive `#DDE0E0`.
- Section labels: upper-case sans 12-12.5 px, 600, +0.06 em, `#EAEBEC`.
- Tile labels: sentence-case sans 13 px, 400, `#D6D7D8` on neutral tiles; near-black `#111` (dark, ~600 weight looks slightly heavier) on coloured tiles, `ESTIMATED`.
- Preset rows: sans 13 px. Selected `#C8CACB` on `#292929`; others `#DADCDD`.
- Version: mono 12 px, `#BBBABC`.

### E. Colour

- Primitive, Generator and Effect tiles carry **no** colour.
- Relation tiles are the only large colour areas in the Browser. Fill values are in 1.3.
- Glyph colour on relation tiles: near-black (`#0C1219`-`#171516`).
- Glyph colour on neutral tiles: `#B3B2B3`-`#CED0D0`.
- The selected preset uses a neutral surface. No accent colour.

### F. States

| Item | idle | hover | selected | active/pressed | disabled |
|---|---|---|---|---|---|
| Tab | no fill | NOT SPECIFIED | `#2D2D2E` fill, brighter label | NOT SPECIFIED | NOT SPECIFIED |
| Primitive / Generator / Effect tile | neutral, 1 px border | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED |
| Relation tile | family fill | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED |
| Preset row | no fill | NOT SPECIFIED | `#292929` + 1 px `#404041` | NOT SPECIFIED | NOT SPECIFIED |

Derived (validation required): tile hover = neutral fill +6 levels, relation tile hover = fill +4% lightness. Selected tile derived = 1 px `#F0F0F0`-at-40% inner outline.

### G. Responsive

| Element | Behaviour as width decreases |
|---|---|
| Tab strip | HIDE labels after the 2nd tab into an overflow key; the search key stays. |
| Primitive / Generator | REFLOW from 4 columns to 3, then 2 columns; tile width SCALES within 60-90 px (`DERIVED - REQUIRES VALIDATION`). |
| Relation | REFLOW from 3 to 2 columns. Tile height stays ~80 px (glyph and label fixed size). |
| Effect | REFLOW from 3 to 2 columns, then FOLD into a one-column list of glyph + label (rows ~28 px). |
| Preset | Row layout does not change. Text is clipped. |
| Section labels | Never hidden. |
| Footer | HIDE the version string first. |
| Whole panel | SCROLL vertically when content exceeds height. The tab strip is pinned. |

Never lost at any width: the Relation colour surfaces and the search key.

## 4. Stage chrome

The Stage is a work surface. Chrome is thin.

### A. Anatomy

1. Composition tab strip: open tab "Composition 1 x", a plus tab (y 71-100).
2. Work surface (x 345-1121, y 101-637).
3. Tool column at the left edge of the work surface (x 354-395, y 111-~414), seven items.
4. Bottom bar (y 638-692): 100% dropdown, two fit keys, a plus mark, a group of colour dots, and on the right a Camera View dropdown and a fullscreen key.
5. Stage-owned annotations inside the artwork ("Jewel Field / ARRANGE ANIMATE REPEAT BELONG", the headline and caption at the bottom-left) belong to the artwork, not to the UI.

### B. Dimensions

| Element | Value | Tag |
|---|---|---|
| Panel outline | x 344-1122, rule 1 px `#3B3C3C` right, `#323436` left | MEASURED |
| Tab strip | y 63-100; top padding 8; tab top y=71 | MEASURED |
| Open tab | x 345-~484, y 71-100, fill `#2A2A2C`, top border `#454546` | MEASURED |
| Plus tab | x 486-525, width 40, fill `#222222` | MEASURED |
| Work surface | x 345-1121 (777) x y 101-637 (537) | MEASURED |
| Tool column | x 354-395 (42 wide), items ~43-44 tall, 7 items | MEASURED |
| First tool item (selected) | y 111-154 (44), fill `#383A40`, glyph `#CED3DB` | MEASURED |
| Other tool items | fill `#212122`-`#242424`, divider `#2C2C2D` | MEASURED |
| Bottom bar | y 638-692 (54 tall), top rule y=637-638 (`#323235`), bottom rule y=692 (`#323232`) | MEASURED |
| 100% dropdown | x 354-470 (117), y 649-681 (33), fill `#1D1D1D` | MEASURED |
| Fit keys | x 489-526 and 534-570 (~38 wide), y 649-681 | MEASURED |
| Colour-dot group | x 610-742 (133), y 649-681, fill `#242424`; dots ~12 px, pitch ~30 | MEASURED / ESTIMATED |
| Camera View dropdown | x 894-1037 (143), y 649-681 | MEASURED |
| Fullscreen key | x 1057-1093 (36) | MEASURED |

Bottom bar internal margins: 9-10 px left of the first control, 11 px above and below the controls (y 638 to 649, 681 to 692).

### C. Alignment

- Tool column left edge (x=354) aligns with the 100% dropdown left edge (x=354).
- Every bottom-bar control has the same top (y=649) and bottom (y=681).
- Tab strip left edge and work surface left edge coincide (x=345).

### D. Typography

- Tab text: sans 14 px, 500, `#FBFCFB` (cap 10).
- Dropdown text ("100%", "Camera View"): sans 12.5-13 px, 400, `#F7F7F7`.
- No other UI text on the work surface.

### E. Colour

- All chrome is neutral. Colour appears only in the colour-dot group in the bottom bar: `#76B0F0` (blue), `#66996A` (green), `#E472AA` (pink), then two empty dots with a `#282828`-`#262628` ring.
- The three coloured dots resemble Stagger, Along Path and Scatter. What they control is `NOT SPECIFIED BY REFERENCE`.

### F. States

| Control | idle | hover | selected | disabled |
|---|---|---|---|---|
| Composition tab | `#2A2A2C` (the open tab) | NOT SPECIFIED | open = selected | NOT SPECIFIED |
| Tool item | neutral | NOT SPECIFIED | `#383A40` fill, lighter glyph | NOT SPECIFIED |
| Dropdown / key | `#1D1D1D` | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED |

### G. Responsive

| Element | Behaviour |
|---|---|
| Work surface | SCALE to fit; it takes all remaining area. |
| Tool column | Stays at 42 px. If height is short, SCROLL inside the column. |
| Bottom bar | HIDE the colour-dot group, then the fit keys, then the "Camera View" dropdown text (key only). The 100% dropdown and fullscreen key stay. |
| Composition tabs | FOLD extra tabs into a chevron menu. |

No UI text may be placed over the work surface (the reference has none; the tool column is the only overlay).

## 5. Inspector

### A. Anatomy

1. Panel tab strip: INSPECTOR (active), PROJECT, LOOK, and a more key (y 71-107).
2. Selected-object header: label "Group", pink swatch, name field "Jewel Field", lock key (y ~120-168).
3. Mode tabs: Transform, Relations (active), Effects, Material (y 182-218).
4. Active relation: **Scatter**.
   - Relation header: identity dot, title, sub-label, toggle, more key.
   - Visualizer (left) and precision controls (right) separated by a vertical rule.
5. Folded relation rows: Stagger, Along Path, Face Target.
6. Add Relation row.

### B. Dimensions

| Element | Value | Tag |
|---|---|---|
| Panel | x 1135-1521 (387 outer) | MEASURED |
| Tab strip top padding | 8 (y 63-70) | MEASURED |
| Active tab | x 1135-1241 (107) x y 71-107 (37), fill `#333539` | MEASURED |
| More key | x 1483-1520, y 72-106 | MEASURED |
| Object header | swatch x 1148-1175 (28 x 27, y 134-160); field x 1190-1449 (~260 wide), y 140-168; lock key x 1467-1505 (39 x 36), y 132-167 | MEASURED |
| Label "Group" | cap 10-12, y 122-133, x 1192 | MEASURED |
| Mode tab bar | x 1134-1520, y 182-218 (~36 tall); active cell x 1237-1338 (102), fill `#3E4043` | MEASURED |
| Scatter card | x 1136-1520 (385), y 227-514 (288); fill `#1F201F`; bottom edge `#2F2F2F` | MEASURED |
| Card header | ~56 (y 227-~283), contents: dot 17 x 17 at x 1147-1163, y 240-256; title x 1181 (cap 14); sub-label y 263-273; toggle 37 x 20 at x 1440-1476, y 242-262; more key dots at x ~1498 | MEASURED / ESTIMATED |
| Visualizer | x 1136-1342 (206) x y ~284-514 (~230). The circle: centre ~(1240, 402), diameter ~186, dashed 1 px `#5F5F5F` (estimated). Drag handle: ring 15 x 16 at x 1288-1302, y 325-340 | MEASURED / ESTIMATED |
| Vertical rule | x=1342 | MEASURED |
| Precision column | x 1343-1520 (177); content inset left 11 (x=1354) and right 18 (value right edge x=1498) | MEASURED |
| Density | label y 309-319; track centre y ~332; value box right side | MEASURED |
| Spread | label ~y 357-367; track centre ~y 381 | ESTIMATED |
| Falloff | label y 401-409; curve region y ~415-450 | ESTIMATED |
| Shape | label y 459-470; selected key x 1354-1380 (27 x 26), y 476-501; five options, pitch ~31 | MEASURED / ESTIMATED |
| Slider | track x 1354-~1501 (147); track height 4-5; thumb diameter ~9-10. Thumb x is ~1437 for Density and ~1418 for Spread. These do not equal the readouts 0.72 and 0.48 (`ARTIFACT`; geometry is measured, value mapping is not) | MEASURED / ESTIMATED |
| Folded row | x 1136-1520 (385), height 38 (fill y 529-566, 569-607, 610-647), pitch 40, gap 2, fill `#212120`, border `#333333` top / `#292929` bottom | MEASURED |
| Folded row dot | 18 x 18 at x 1149-1166 (row 1: y 539-556) | MEASURED |
| Folded row toggle | x ~1440-1476 (37 x 20), kebab at x ~1498 | MEASURED |
| Gap card to first folded row | 13 (y 515-528) | MEASURED |
| Add Relation row | y 655-691 (37), gap above 4 (y 650-654 vs previous row) | MEASURED |
| Panel bottom | y ~692 | MEASURED |

Visualizer to precision ratio: 206 : 177, that is 54% : 46% of the card width (`MEASURED`).

Card is 288 tall. The header is about 19% of the card height and the visualizer + precision block is about 80%.

### C. Alignment

- Dot (x 1147-1163 in the card, 1149-1166 in folded rows) and identity swatch column: card dot and folded-row dots have the same centre x (~1156-1157).
- Title and folded row names share the left edge x=1181.
- Toggles right edge x=1476 in the card header and in folded rows: same column.
- Kebab (more) keys share x~1498.
- Precision column: labels left at x=1354; numeric values right-aligned at x=1498; sliders span x=1354-1501; Shape buttons start x=1354.
- Visualizer circle centred in the visualizer cell horizontally (cell centre 1239, circle centre 1240).

### D. Typography

See 1.5. Summary: title 19-20 px sans 600; folded names 16 px sans 500; parameter labels 12 px sans 500; values 12 px mono; sub-label 11-12 px sans `#B5B6B5`.

### E. Colour

- Card fill `#1F201F` is only about 6-8 levels above the panel (`#191919`). The card is not a loud box.
- Identity colour is used for: the dot, the visualizer marks and handle, the slider fill and thumb, the curve, the Shape selected key. Text is never coloured.
- Folded rows: dot only in colour. Face Target keeps a yellow dot while the toggle is off (grey track).
- Toggle on: `#6982D1` (violet-blue, not the relation colour). This is a state colour, not identity.

### F. States

| Control | idle | hover | selected/active | disabled |
|---|---|---|---|---|
| Panel tab | no fill | NOT SPECIFIED | fill `#333539` | NOT SPECIFIED |
| Mode tab | fill `#222222` | NOT SPECIFIED | `#3E4043`, label `#FFFFFF` | NOT SPECIFIED |
| Relation card | open | | open (Scatter) | NOT SPECIFIED |
| Folded row | closed | NOT SPECIFIED | NOT SPECIFIED | toggle off (grey), dot still coloured |
| Slider | Spread: fill `#9B4A6F`; Falloff: not measurable | NOT SPECIFIED | Density: fill `#F478B6`, value in a lighter box | NOT SPECIFIED |
| Shape option | glyph only | NOT SPECIFIED | pink filled key | NOT SPECIFIED |

The Density / Spread slider difference is visible in the still. It may be an active-vs-idle state or a generation inconsistency. It is recorded as `REFERENCE VARIANCE`. Derived (validation required): idle slider fill = identity colour at 62%, active/hover = 100%.

### G. Responsive

The Inspector is the panel where reflow matters most.

| Element | Width decreases | Notes |
|---|---|---|
| Panel tabs | HIDE PROJECT and LOOK into the more key | INSPECTOR stays |
| Object header | swatch and name stay; the "Group" label and lock key HIDE | |
| Mode tabs | SCALE label text down to 11 px, then FOLD to icons or the active label only | |
| Scatter card: visualizer + precision | **REFLOW**: below ~340 px card width, precision moves under the visualizer as full-width rows (label left, slider centre, value right, row ~22 px) | ESTIMATED breakpoint; DERIVED - REQUIRES VALIDATION |
| Visualizer | SCALE to fit its cell, but not below ~100 px diameter; below that, HIDE the marks and keep the ring and handle | |
| Card header | sub-label HIDES first, then title clips | toggle and more key never hide |
| Folded rows | HIDE any inline mini-visual first (none in the reference), then clip the name; dot, name and toggle remain | |
| Add Relation | label shortens to a plus key | |
| Whole panel | SCROLL vertically. Panel tab strip and object header are pinned. | |

At any height, if more than one relation is open, the extra ones fold. Only one instrument is open at once (`DERIVED - REQUIRES VALIDATION`; the reference shows one).

### Special: expanded Scatter instrument

The instrument is one surface. Regions, top to bottom:

1. Header (56): dot 17, title, sub-label, toggle, more.
2. Divider: 1 px `#2F2F2F`.
3. Body (~230): visualizer (206) | 1 px rule | precision (177).
4. Precision, top to bottom: Density, Spread, Falloff (label + value + curve), Shape.
   - Pitch Density label to Spread label: ~49 px. Spread label to Falloff label: ~44 px.
   - Each parameter is a label row (label left, value right) plus a slider track below. The value has a slightly lighter box behind it for Density.
   - Falloff replaces the slider with a curve: two control dots on a curve of width ~145 px, height ~35 px.
   - Shape: label, then five options at pitch ~31. Only the selected one has a fill.
5. There is no second card inside the card.

The collapsed Relation rows are separate bordered rows (each has its own 1 px border), not lines inside one container. This differs from the open instrument.

## 6. Timeline

Treat the Timeline as independent layers.

### A. Anatomy

1. Panel top rule (y=703).
2. Tab strip (y 704-741): Timeline (open), Graph, Console.
3. Hierarchy column: x 345-557, 213 wide.
4. Ruler: y 743-773, 31 tall, bottom edge y=774 (brighter `#484847`).
5. Row area: y 775-993.
6. Rows: group (Jewel Field), property (Transform), relation rows (Scatter, Stagger, Along Path, Rotation, Scale Variation), object rows (Camera), audio row.
7. Bars, keyframes, waveform, playhead.

### B. Dimensions

| Element | Value | Tag |
|---|---|---|
| Panel | x 345-1521, y 703-993 (290 tall) | MEASURED |
| Tab strip | y 704-741 (38 tall); open tab x 345-447, fill `#2C3344` | MEASURED |
| Hierarchy / time divider | x=558 | MEASURED |
| Ruler | y 743-773 (31); tick label digit height 9, centred y~757 | MEASURED |
| Group row | y 775-797 (23) | MEASURED |
| Child rows | y 799-819, 821-841, 844-864, 866-886, 889-909, 911-932 (21-22 tall) | MEASURED |
| Row pitch | ~22.7 (includes a 1-2 px darker separator) | MEASURED |
| Camera row | y 934-954 | MEASURED |
| Audio row | y 966-989 (waveform 966-991, 26 tall) | MEASURED |
| Time scale | ~90.9 px per labelled step (x 560, 651, 742, ... ) | ESTIMATED |
| Relation bar height | 17-19 (Scatter y 823-840 = 18) | MEASURED |
| Bar inset in row | top 2, bottom 1-2, so bar / row = ~0.85 | MEASURED |
| Property bar (Transform) | y 805-819 (~14), narrower and translucent | MEASURED |
| Bar horizontal extents (reference) | Scatter x 621-1317; Stagger x 607-1201; Along Path x <558 (clipped) to 1281; Rotation x 608-928; Scale Variation x 682-1143; Transform x 625-856 | MEASURED |
| Keyframe | diamond ~9 x 8 (Transform key x 628-636, y 805-812); light outline | MEASURED |
| Playhead | line at x=736-738 (1-2 px), handle y 748-~760 (pentagon ~13 wide) | MEASURED |
| Names column columns | disclosure triangle x 358-368; group label x 386; child icon x 385-400 (16 x 16); child label x 412; row icons for camera/audio at x 388-398 | MEASURED |

### C. Alignment (the columns visible in the reference)

Hierarchy column has three vertical alignment columns at x = 358, 385, 412 (pitch 27):

- x=358: disclosure triangles (Jewel Field down triangle, Camera right triangle) and the two lock glyphs of the Audio row.
- x=385: the group label "Jewel Field" starts here; property and relation icons (16 px squares) also start here.
- x=412: child labels start here (Transform, Scatter, Stagger, Along Path, Rotation, Scale Variation, Camera, Audio).

Time area:

- Vertical grid lines align to ruler labels. Labels are centred on their line in the intended design.
- Bars start and end on time positions; their left edges are square with the row's inner edge, not with the row separator.
- The centre rail inside each bar (a thin dark line) is at the bar's vertical centre and runs between its first and last key.

### D. Typography

| Text | Style |
|---|---|
| Group name | sans 13 px, 500-600, `#F8FAFA` |
| Child names | sans 13 px, 400, `#DDDEDE` |
| Ruler | mono 12 px, `#BDBEC0` |
| Tab labels | sans 13-14 px; open tab 500, `#FEFFFE`; others `#C5C5C5` |

### E. Colour

Layers, as the reference draws them:

1. Panel background `#191919`.
2. Hierarchy column fill `#1F1F1F`-`#202020`.
3. Row fills: group row `#272727`-`#292929`; children alternate `#212121` and `#2A2A2A`-`#2C2C2C` (zebra: Transform `#212121`, Scatter `#2B2B2B`, Stagger `#212120`, Along Path `#2C2C2C`, Rotation `#212121`, Scale Variation `#2A2A2A`, Camera `#212120`). The hierarchy column repeats the zebra in a slightly lower contrast.
4. Ruler `#1F1F1F`, bottom edge `#484847`.
5. Major time grid: 1 px vertical `#2A2A2A`-`#343434`. Minor grid: fainter; ruler minor ticks 3-6 px long.
6. Bars: identity colour at the "Timeline body" values in 1.3. Property bar (Transform) is a muted pink `#967187`-`#88667C`.
7. Keyframes: light outline `#E0E0E0`-ish on the bar; some filled dark. The exact fill variety is `ESTIMATED`.
8. Selection: NOT SPECIFIED BY REFERENCE. The Scatter row's stripe colour equals the alternate stripe. Do not read it as selection.
9. Playhead: `#6BA7D7`-`#7FB3F4`.
10. Waveform: band `#2E4740`-`#2F4742`, wave `#3B6D5F`-`#3B7060`.

Row-icon rules:

- Relation rows: a 16 x 16 rounded square in the "Timeline name icon" colour with a light diamond inside.
- Property row (Transform): an outlined diamond in pink `#967187`-family, no filled square.
- Group and object rows: a triangle disclosure. The Camera row also has a round camera icon.

### F. States

| Element | idle | hover | selected | disabled |
|---|---|---|---|---|
| Tab | no fill | NOT SPECIFIED | fill `#2C3344` | NOT SPECIFIED |
| Row | zebra | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED |
| Bar | family body | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED |
| Keyframe | outline diamond | NOT SPECIFIED | NOT SPECIFIED | NOT SPECIFIED |
| Playhead | line + handle | dragging NOT SPECIFIED | | |

Derived (validation required): selected row = fill +10 levels and a 3 px identity-colour left edge on the name; selected bar = 1 px `#F0F0F0`-at-70% outline; selected key = filled with the bar's darker shade.

Do not add trim handles, curves or extra interior detail to bars beyond what the data has. The reference shows a centre rail and keys only.

### G. Responsive

| Element | Behaviour |
|---|---|
| Time area | SCALE horizontally with the zoom. It takes all remaining width. |
| Hierarchy column | fixed 213 at the reference width. Below ~150 px, HIDE the disclosure triangle column; below ~90, HIDE label text and keep icons. |
| Row height | fixed 21-23. No SCALE. |
| Ruler labels | HIDE alternate labels (decimate) as the zoom-out reduces the pitch below ~70 px. Labels always land on a major line. |
| Tab strip | FOLD Graph and Console into the open tab's menu if width < ~300. |
| Rows | SCROLL vertically when the row count exceeds the height. The ruler and tab strip are pinned. |

The Timeline extends across the Stage and Inspector columns in the reference. This is a layout fact. The component itself must work at any width.

## 7. Global states summary

See each panel. Every hover, focus and pressed state is `NOT SPECIFIED BY REFERENCE`.

## 8. Cross-panel identity: Scatter

The rule: DO NOT UNIFY COMPONENTS. UNIFY IDENTITY.

| Panel | Shape | Colour treatment | Colour area | Label | Selected | Disabled |
|---|---|---|---|---|---|---|
| Browser | rounded rectangle tile, 98 x 80 | full fill `#F27AB6`; glyph and label near-black | ~7,840 px squared (the largest colour area in the UI) | "Scatter", left aligned, 13 px sans | NOT SPECIFIED | NOT SPECIFIED |
| Stage | dashed ring and a small circle handle in the artwork (drawn by the work overlay, not by chrome) | pink (`#F277B4`-family) on the work | small | none | handle shown = it is active | NOT SPECIFIED |
| Inspector | dot 17 px; the distribution visualizer; slider fills; curve; shape key | dot `#F277B4`; marks `#F277B4`; slider fill `#F478B6` | dot ~230 px squared; visualizer marks ~90 dots of 4-10 px | "Scatter" 19-20 px sans 600, sub-label | open card | toggle off (`#999BA0`) with the dot kept coloured (as Face Target shows) |
| Timeline | 16 x 16 icon square + a horizontal bar 18 px high | icon `#C76295`, body `#E276AC` | body: a 696 x 18 bar | "Scatter", 13 px sans, at x=412 | NOT SPECIFIED | NOT SPECIFIED |

What stays the same across panels:

1. The hue family (pink, hue ~330 deg).
2. The name "Scatter".
3. The presence of a small colour element next to the name.

What differs on purpose: the body (tile, ring, dot + distribution, bar), the size of the colour area (large in Browser, tiny in Inspector, medium in Timeline), and the lightness (Browser brightest, Timeline darker).

The area ratio of colour is not measured further because the Stage overlay is inside the artwork.

## 9. Reference artifacts (do not reproduce)

Each entry: REFERENCE ARTIFACT to intended interpretation.

| Where | Reference artifact | Intended interpretation |
|---|---|---|
| Timeline ruler | Labels repeat and skip: `00:04` twice, `00:10` twice; between `00:00` and `00:03` there are two tiny overlapped labels (`0:05`, `0:02`) at x ~624-710. The tab reads "TImeline". | Evenly spaced labels at one step per ~90.9 px, one label per major line, no duplicates. Tab text "Timeline". |
| Timeline | Along Path bar continues past the left edge of the time area and tints the hierarchy column boundary. | The bar is clipped at the time area's left edge. |
| Timeline | Scatter bar has a darker inner span (x ~866-974) and Stagger has a black rail with unusual joins. | An interior detail exists only if the data has a curve or key range. Do not draw it otherwise (see 6-F). |
| Timeline | The Rotation and Scale Variation rows are named differently from Browser names (Face, Attach). | Names come from the data. Colour comes from the relation family. Do not map by label. |
| Browser | A short vertical tick under the Text tile (x ~55, y ~232-248). | Nothing. |
| Browser | EFFECTS section label sits ~8 px above its tiles; other sections have 13 px. | Use the common label-to-tile distance of 13 px (or the section spacing chosen once for all). |
| Browser | Preset row icons are unreadable smudges. | Any small neutral glyph, 14 x 14 rounded square. |
| Inspector | The Stagger row shows an extra stray mark next to the more key (x ~1495, y ~538). | No extra mark. |
| Inspector | Shape glyphs (cross and star) are inconsistent in size. | Five glyphs in equal visual boxes. |
| Inspector | Sub-label under the title is Japanese. | Window text is English; the sub-label is secondary metadata. |
| Top | Right-hand icons (fullscreen-like, pin-like, folder-like) have no clear product meaning; the tagline blocks are decorative. | Keep as three neutral icon keys and two tagline blocks only if the product wants them; otherwise their space folds first (see 2-G). |
| Stage | The Composition label and the tool column overlap the artwork edge; the artwork contains its own text ("Jewel Field", the Japanese headline, English caption). | The artwork text is part of the artwork. UI adds no text over the work. |
| Global | The window's bottom ~30 px and the Browser bottom edge are inconsistent (Browser to ~1012, Timeline to 993). | Panels bottoms should be consistent in the implementation; the reference is not authoritative here. |

## 10. Unknowns requiring validation

1. Whether the four "selected surface" values (`#2D2D2E`, `#333539`/`#3E4043`, `#2C3344`, `#2A2A2C`) are one token or four. They differ in tint only.
2. Record key colour `#F03C8A` versus Scatter pink `#F27AB6`, and play key `#7BCC9E` versus Along Path mint `#7BCBA3`. The reference uses close colours for operational and identity roles.
3. The meaning of the three coloured dots in the Stage bottom bar.
4. Whether the Timeline shows a selected row. The Scatter row's stripe colour equals the alternate stripe.
5. Whether the Density / Spread slider difference (`#F478B6` vs `#9B4A6F`) is an active state or generation variance.
6. Whether only one relation may be open in the Inspector, or several.
7. The hover, focus, pressed and disabled states for every control.
8. Tile radius (2-5 px range), text families (raster only), and the exact letter-spacing values.
9. Whether Relations and Effects tiles should stay left-aligned while Primitive and Generator tiles are centred (the reference does this; no rationale is recorded).
10. Naming of the yellow and purple Timeline rows ("Rotation", "Scale Variation") against the Browser's "Face" and "Attach".
11. The exact vertical rhythm between Browser sections (the reference has one irregular gap).

## 11. Annotated crops

All in `docs/stage5/ui-rebaseline/handoff/`. Guides: cyan = boundaries, magenta = dimensions, yellow = text/glyph boxes, green = selected/active, orange = gutters or open questions.

- `crop-1-topbar.png`
- `crop-2-browser.png`
- `crop-3-inspector.png`
- `crop-4-timeline.png`
- `crop-5-stage-chrome.png`
- `overview-panels.png` (panel map, section 1.1)

## 12. Capability source

Classic Motolii is the capability oracle only. This document contains no Classic visual information. The capability list to be validated against this layout lives in `inventory/`. It has not been re-read for this handoff.

## 13. Rulings on the open questions (adopted before implementation)

Handoff v1 is the implementation baseline. The nine open items in section 10 are ruled as follows. These are construction decisions, not design changes.

1. **Selection surfaces.** One selection family. Local luminance and tint correction per place is allowed. Four semantic tokens are not created.
2. **Operational vs identity colour.** The measured values stay. Colour namespaces are not separated. Meaning is also carried by position and shape. Record must be strongly distinct in its real recording state.
3. **Stage bottom colour dots.** Fictional. Not implemented. Meaning is assigned only if Classic capability shows a match.
4. **Timeline selected row.** Not invented from the Concept. The minimal representation is designed when Classic selection semantics are connected.
5. **Density / Spread colour.** Same Scatter family. No active-state assumption. One accent token.
6. **Inspector multi-open.** Undecided until capability is checked. No accordion is specified.
7. **Hover, focus, pressed, disabled.** Derived conservatively later. A visual-state sheet is made at implementation time and validated.
8. **Timeline row names vs Browser names.** Not mapped. Timeline rows take names and colours from their real semantic identity.
9. **Radius and font.** Radius is rounded to an integer inside the measured range (tile 4, key 3, input 3, tab 2, bar 3, card 2). Font is a product sans (Inter) plus a numeric mono (Menlo). No search for a lookalike face.

Notes from the first construction pass:

- The left tagline is set in Inter, not mono. The reference width (about 5.4 px per character at cap height 8) is narrower than any mono at that height.
- Text is width-fitted to the measured reference boxes: the font is shrunk by up to 18% first, then tracking is adjusted within -0.7 to +2.2 px. Inter and Menlo are wider than the reference faces.
- The Scatter card outline is at x=1134-1521, y=225-515 (1 px `#363636`), and folded rows and Add Relation span x=1134-1521. The earlier figure of x=1136-1520 in section 5-B is superseded.

## 14. Construction pass 2: icons, type weight, spacing, state sheet

Code: `motolii/ui/lib/proto_hf/` (throwaway prototype, not wired). Evidence: `handoff/build-compare-v1.png` (top: reference, bottom: build), `handoff/build-candidate-v1.png`, `handoff/build-states-v1.png`.

**Icons.** `glyphs.dart` draws every glyph to the reference forms in a 24-unit box. Weight is deliberately unequal: outline glyphs 1.4-1.7, filled masses (shape, glow, image, camera, star, lock), dots (scatter, stagger, repeater). Along Path is an open ring with two end dots, Face is a broken ring, Follow is two arrows plus a faint third, Attach is two interlocked links. Icon visual sizes: tiles 21-25, tools 19-22, small keys 15-20.

**Type weight.** Important words are fuller and brighter, technical text is lighter and dimmer.

| Tier | Style |
|---|---|
| Brand, relation title, object name, row names, group name | Inter 600, `#FDFDFD`-`#FFFFFF` |
| Item names, tab labels | 400-500, `#C9C9C9`-`#DDE0E0` |
| Parameter labels, secondary text | 400, `#9C9D9D`-`#DADBDB` |
| Numeric readouts | Menlo 12, `#D4D4D4` |

**Spacing check.** Text bounding boxes were compared to the reference for 23 items. Baselines match within 1 px for all of them. Left and right edges match within 1 px for 18 of 23. The rest differ by 2 px, caused by the wider Inter and Menlo faces.

**State sheet** (`handoff/build-states-v1.png`). Everything except idle is `DERIVED - REQUIRES VALIDATION`. Rules:

| Control | hover | pressed | focus | disabled | selected |
|---|---|---|---|---|---|
| Neutral tile, key, dropdown | surface +7, border +8 | surface -5 | 1.5 px `#E9E9EC` at 70%, 3 px outside | content 38% | not specified |
| Relation tile | lighten 7% | darken 10% | same ring | 30% of family colour over neutral, content 40% | not specified |
| Tab | surface `#232323` | selected -3 | same ring | label 38% | measured fill |
| Toggle | knob +2 | knob width 20 | same ring | 38% | on / off measured |
| Slider | thumb 12 | thumb 12 plus 22% halo | ring on thumb | 35% | none |
| Folded row | `#262626` | `#1D1D1D` | ring | content 40%, dot stays coloured | none |
| Preset row | `#202020` | `#242424` + border | ring | 38% | measured |
| Timeline row and bar | row +6, bar +8% | not specified | ring | bar 35% | row +10 with a 3 px identity edge, bar 1 px outline `#F0F0F0` at 80% (no selection exists in the reference) |
| Keyframe | 7 px | filled, 8 px | ring | 35% | filled white |

The focus ring is neutral on purpose so it never competes with identity colours.

## 15. Normalisation pass (visual golden candidate)

The reference is a generation. Where its pixels disagree, a shared rule was inferred from several measurements and applied to every element of the same role. Nothing here adds a feature, layout or interaction. Code: `motolii/ui/lib/proto_hf/` (`ref.dart` holds the tokens). Evidence: `handoff/polish-concept-before-after.png` (top to bottom: reference, before, after), `handoff/polish-timeline-concept-before-after.png`, `handoff/audit-squint.png`, `handoff/audit-grayscale.png`, `handoff/build-candidate-v2.png`, `handoff/build-states-v2.png`.

### 15.1 Neutral tokens (replace section 1.2 values)

| Token | Value | Replaces (measured, local variation) |
|---|---|---|
| ground | `#191919` | window, panel, gutters (`#171717`-`#1A1A1A`) |
| raised | `#202020` | tiles, keys, inputs, dropdowns, folded rows, card, Timeline row ground A, ruler (`#1B1B1B`-`#222222`) |
| raisedHi | `#262626` | hover, value wells, Timeline row ground B (`#262626`-`#2C2C2C`) |
| sel | `#2F3034` | selected tab, open tab, selected preset (`#2A2A2C`, `#2C3344`, `#2D2D2E`, `#333539`, `#292929`) |
| selHi | `#3B3D42` | selected cell inside a bar: subtab, tool (`#3E4043`, `#383A40`) |
| rule | `#343434` | every outline: panel edges and control borders (`#303132`-`#3E3E3D`) |
| rule2 | `#2A2A2A` | every inner divider |
| track | `#2C2C2C` | slider track |
| text / text2 / text3 | `#F5F5F5` / `#D2D2D2` / `#9E9E9F` | primary, secondary, tertiary. Disabled is 38% alpha, not a fourth grey. |

The local blue tint of the four selected surfaces is dropped. Selection reads as a slightly lighter neutral.

### 15.2 Semantic families

Each identity is one hue. The Browser and Timeline variants are sampled from the reference. The Timeline variant is NOT a blanket desaturation: the Timeline is as vivid as the reference. The Inspector accent is the only computed variant.

| Variant | Rule (HSL) | Used for |
|---|---|---|
| b (Browser) | the measured surface | large tiles |
| i (Inspector) | saturation x1.16, lightness x0.96 | dots, marks, slider fill, curve |
| t (Timeline) | sampled medians of the reference bars: Scatter `#E274AA`, Stagger `#4C7AAC`, Along Path `#77B68C`, Rotation (yellow) `#CEBA54`, Scale Variation (lavender) `#A889E9`. Transform is neutral `#8E8892`. | long bodies |
| n (name icon) | sampled: Scatter `#C76295`, Stagger `#477FBD`, Along Path `#517F68`, yellow `#E3C748`, lavender `#A086E2`; Transform neutral `#77717C` | Timeline name squares |
| dim | 30% of b over raised | off and disabled surfaces |

Base colours: Scatter `#F27AB6`, Stagger `#5596E9`, Along Path `#7BCBA3`, Face `#F0D455`, Follow `#F69260`, Attach `#A282E8`.

### 15.3 Typography roles

| Role | Style |
|---|---|
| brand | Inter 28 / 600, fitted to 97 px |
| navigation (tabs) | Inter 12.5 / 500, caps, +0.7 tracking |
| section | Inter 12.5 / 600, caps, +0.7 tracking |
| object | Inter 15-21 / 600 (field 15, group name 12.5, folded row 16, relation title 21) |
| item | Inter 12.5 / 400, text2 (tile labels, presets, Timeline names) |
| control label (sentence case) | Inter 13 / 400-500 (composition tab, dropdowns, subtabs, Timeline tabs, Add Relation) |
| parameter | Inter 12.5 / 400, text2 |
| numeric | Menlo 12, text2 (Inspector values, ruler, version). Top readouts are Menlo 13. |
| annotation | 12, text3 (sub-label, taglines) |

### 15.4 Geometry grammar

Repeated grammar is shared. Different meanings stay different components.

| Grammar | Value |
|---|---|
| Neutral and relation tiles | radius 4, 1 px `rule` |
| Keys, dropdowns, inputs, search | radius 3, 1 px `rule` |
| Tabs | radius 2 |
| Surfaces with an outline (Scatter card, folded rows, Add Relation) | radius 2, 1 px `rule`; folded rows and Add Relation are both 38 high |
| Timeline bars | radius 4 (square on the clipped side), 1 px outline 10% lighter than the body (the reference edge is lighter, not darker) |
| Toggle | 37 x 20 capsule, knob 16 |
| Slider | track 5, thumb 10 |
| Icon visual boxes | tile glyphs 21-25, tools 19-22, small keys 15-20 |

### 15.5 Optical alignment

Triangles and arrows are placed by visual centre, not bounding box: the play triangle is shifted right by 1.4 px inside its key; the outline triangle, the stylize triangle and the pin arrow are drawn 0.8 px higher in their 24-unit box; the timeline disclosure triangles are centred on the row. Text baselines are placed at row centre plus 4.5.

### 15.6 Timeline layers (independent contrast)

| Layer | Rule |
|---|---|
| 1 panel ground | `ground` |
| 2 row ground | alternates `raised` / `raisedHi`, continuous across both columns (a row is one strip, not two cells). Group row `raisedHi`. Audio row `raised`. |
| 3 hierarchy | three fixed columns: c0 x=363 (disclosure or state), c1 x=393 (glyph), c2 x=412 (label). The group label sits on the c1 edge (x=386). Baseline = row centre + 4.5. |
| 4 ruler | 31 high, numerals Menlo 12 in text3, major tick 8, minor tick 4 |
| 5 time grid | major line white 9%, minor line white 4%, row gap (ground) about 3%. Time reads strongest. |
| 6 temporal body | Flat sampled identity colour (no gradient). Rows 1-6 are siblings with one vertical metric. Inside the body: one thin connector line through the keys, with small diamond keyframes on it. A body with a single key has no connector. |
| 7 keyframe | One idle appearance for every key. The reference shows small diamonds on the connector. Its per-key differences (fill, brightness, first key, overlap with the playhead) are generation noise and are not adopted. Selected and focused exist only in the state sheet (`DERIVED - REQUIRES VALIDATION`). |
| 8 playhead | ruler marker + 1.3 px line in `#6EA6DB`, above bodies and keys |
| 9 waveform | a floor: dark green band `#1E2925`, wave `#3B6D5F` at 69%. Lower contrast than any relation body. |

Audits: the Timeline keeps the reference's vivid colour. The Stage wins by information structure, not by saturation: photographic, luminous, irregular against flat, geometric, repetitive (`audit-squint.png` was taken with the earlier desaturated variant and is superseded). In greyscale the panel hierarchy, the tab strips, the row structure and the Stage dominance all hold (`audit-grayscale.png`).

### 15.7 Text placement check

For 15 measured text items the candidate's bounding box is within 2 px of the reference on every edge. Remaining differences come from the wider Inter and Menlo faces.

### 15.8 Timeline semantics (corrects earlier readings)

- The vertical structure is not a layer-with-properties tree. `Jewel Field` is the only parent row. Transform, Scatter, Stagger, Along Path, Rotation and Scale Variation are parallel operations acting on the group. Camera and Audio are separate top-level items and may use a different representation.
- **Structure = geometry. Identity = colour. Content = marks inside the body.** Colour presence or absence does not express hierarchy. Transform is neutral because its identity is neutral, not because it is subordinate.
- The six sibling rows share: row height, bar height, top and bottom inset, indent, glyph box, glyph centre, label baseline and keyframe centre. Verification: `handoff/polish2-timeline-row-guides.png` (left: the reference with evenly spaced guides, showing its vertical jitter; right: the build, where every guide passes through the icon, label, bar and keys of its row).
- The idle visual golden contains no inferred interaction states.
- Row names "Rotation" and "Scale Variation" are placeholder data taken from the reference listing, coloured yellow and lavender. They are not a mapping to Browser relations.

### 15.9 Reading rule for the reference (retraction)

The reference is a generation. Local shading, breaks in lines, per-key fill differences and overlapping colours are noise. Only structure that repeats across several places is adopted. Withdrawn readings: a connector that darkens toward the right, a connector that starts at the second key, a connector only on part of a bar, a special first key, a key that turns blue under the playhead, meaningful interior gradients. Gradients are not a design element here.

Timeline grammar adopted (high confidence):

1. neutral row ground
2. coloured temporal body (flat)
3. a thin connector line inside the body
4. small diamond keyframes on the connector
5. playhead
6. time grid

The connector has one rule for every relation: constant colour, constant thickness, first key to last key, derived from the identity colour by one rule. No connector for a single key.

### 15.10 Proportion audit (Timeline, measured with one procedure on both images)

Procedure: `handoff/measure-timeline-ratios.py`. Medians over several rows or samples. Row pitch is the regression slope of the six sibling row centres.

| Quantity | Reference | Build |
|---|---|---|
| row pitch | 22.4 px | 23.0 px |
| bar height | 18.5 px | 16.0 px |
| key outline extent | 10 px (range 7-11, 10 of 17 keys detectable) | 8 px (design 8-10) |
| connector thickness | 2 px | 3 px (design 2.2-2.6, anti-aliased) |
| identity glyph box | 16 px | 16 px |
| label cap height | 9 px | 9 px |
| ruler digit height | 9 px | 8 px |

| Ratio | Reference | Build |
|---|---|---|
| bar height / row pitch | 0.82 | 0.70 |
| key diameter / bar height | 0.54 | 0.50 |
| connector thickness / bar height | 0.11 | 0.19 |
| identity glyph / row pitch | 0.71 | 0.70 |
| label cap height / row pitch | 0.40 | 0.39 |
| ruler text height / row pitch | 0.40 | 0.35 |

Reading: the identity glyph, label and key proportions already match. The bar is thinner than the reference, the connector is thicker, and the ruler digits are one pixel smaller. The build's keys are not larger than the reference by this measure.

### 15.11 Applied after the proportion audit

Timeline only: bar height 18 (bar / pitch 0.78 by the audit procedure; the reference is 0.82, and the procedure quantises bar edges in 2 px steps), connector 1.5 px in one constant colour for every relation (connector / bar 0.11), ruler digits Menlo 13 (ruler / pitch 0.39), and one keyframe appearance (a 6.2 px diamond in a pale tint of its body). Nothing else in the Timeline changed. Evidence: `handoff/polish4-timeline-zoom-concept-before-after.png`, `handoff/build-candidate-v5.png`.
