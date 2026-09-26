# visual-language.md against the old Flutter UI

`visual-language.md` was fixed first. This file only gathers evidence. It does not change the language, and no principle was adjusted to fit the old code. Contradictions are reported as found.

Verdicts: SUPPORTED (the old code already does this), PARTIAL, CONTRADICTED (the old code does the opposite or lacks it), NEW (only the Concept has it).

Coverage: read in full or in large part: Browser shelves for Create, Effects, Colors, Media thumbnails, Fonts preview, Files header, the shelf and tile abstractions, the Theme colour functions, Inspector effect card, wells and dials, timeline row painting, stage overlay and camera doc comments. Not read closely: Files, most of Timeline interaction, Stage chrome bars. Nothing here was checked visually against a running old build.

## Summary

| # | Principle | Verdict | One line |
|---|---|---|---|
| 1 | Housing | SUPPORTED | A single metric scale, no shadows, an Ableton lineage, and a code split between the frame and what it holds. |
| 2 | Things have faces | SUPPORTED, uneven | Effects, Colors, 3D bodies, dial, pad, gradient bar have faces. Several Create items are text glyphs. |
| 3 | Toybox | SUPPORTED | Colour is used for kinds and layers across the UI, not for one category. |
| 4 | Identity travels | PARTIAL, one CONTRADICTION | Layer colour travels Inspector to Timeline. Browser identity (kind) does not reach the Timeline (layer id). |
| 5 | Place changes the body | SUPPORTED | Every place already draws the same thing differently. |
| 6 | Phenomenon before parameters | PARTIAL | Effects tiles show the rendered effect. The Inspector is parameter-first. |
| 7 | Fold, do not miniaturise | SUPPORTED | Heroes in front, the rest folded. |
| 8 | Work loudest | UNVERIFIED | Code is consistent with it. The old Timeline colour is deliberately washed. |

## Evidence by principle

### 1 Housing: SUPPORTED

- `foundation/metrics.dart:1`: every size, gap, radius and font size is one metric scale.
- No shadows or backdrop blur anywhere in the old UI (the only shadow in the tree is in the new prototype).
- Ableton and Live 12 are cited as the model in 9 files (browser, filter view, playback, toggles, timeline menu). The word Swiss does not appear. The lineage is Ableton, not named Swiss.
- The split is in the code: `BrowserHost` is the housing that owns tabs, search, rail, grid and selection. A `BrowserShelf` is what it holds and speaks to the frame only through that interface (`panels/browser/shelf.dart`).

### 2 Things have faces: SUPPORTED, uneven

By area:

| Area | Evidence |
|---|---|
| Create | Cube, sphere, torus, cylinder, cone, pyramid and plane are drawn as line bodies (`ShapeMark`, `create_shelf.dart`). Text, shapes, Null, Particles, Stage, Line and Bezier are single text glyphs in a kind colour. Camera is a stock video icon. So: faces for 3D bodies, weak faces for the rest. |
| Media | The item's own thumbnail; otherwise a family icon; a mesh draws the body its name says. |
| Effects | The tile is the effect's own rendered snapshot (`NativeVisualSample`), with a glyph only when none ships. This is the strongest face in the old UI. |
| Fonts | The tile sets words as text; no picture. |
| Colors | The swatch or gradient is the tile. The shelf is bare, with no caption ground. A colour wheel edits the slot. |
| Inspector | An angle is a needle in a ring (`EditorDial`); a pair is a point on a square (`EditorPad`); a fill is a bar with its stops as handles (`GradientInspector`). Most other parameters are numeric wells. |
| Stage | A three-axis gizmo, a Blender-style camera pyramid, and a Boxcam. |
| Timeline | A clip in a layer colour. No face beyond that. |

Nothing contradicts the revised sentence. Generic controls (menus, tooltips, plain wells) exist, and the language allows them.

### 3 Toybox: SUPPORTED

- Nine kind colours (text yellow, shape blue, path cyan, video pink, audio green, 3D violet, HDR, image, other) plus an identity palette per layer in `foundation/theme.dart`.
- Colour is used across Create, Media, Files, Inspector header and Timeline. It is not reserved for one category.
- Toy-like actions: "Throw every number within its reach" and "Back to where the numbers rest" (`inspector/effects_card.dart`), and a dice for random values (`inspector/writing.dart`).
- A colour wheel with saved swatches and gradient stops with blend modes.

This contradicts the reading that only Relations carry colour. The old UI never worked that way.

### 4 Identity travels: PARTIAL, one CONTRADICTION

Supported:

- The Inspector head fills with the layer's colour, and a code comment states why: it is "the layer's own colour as a flat block, as its bar wears it in the Timeline" (`panels/inspector.dart:180`).
- The Timeline uses the same palette with a place-specific variant: `timelineColor` alpha-blends a wash over the identity colour (`foundation/theme.dart:404`). This is a colour family with a per-place body, which the language describes.

Contradicted:

- Browser identity is by kind (`kindColor`: text is yellow, shape is blue). Inspector and Timeline identity is by layer id, modulo a palette index (`layerColor`, `timelineColor`). A Text tile is yellow in the Browser, and the resulting text layer gets a colour that depends on its id. Identity does not travel from the Browser to the layers it creates.

### 5 Place changes the body: SUPPORTED

The same layer is a tile with a picture in the Browser, a head plus wells, dials and pads in the Inspector, a cage or 3D gizmo on the Stage, and a clip bar in the Timeline. Group children also collapse to stripes when a group is closed (`timeline/paint.dart`).

### 6 Phenomenon before parameters: PARTIAL

Supported:

- Effects show their rendered result before any label (Browser).
- The ease desk draws the curve and a band showing which key interval it acts on (`ease_desk/painters.dart`).
- Angle, point and fill have visual controls.

Not supported:

- Effect cards are built from the effect's declaration into numeric wells. The rule is "four knobs in front, the rest behind shift" (the OP-1 comment in `effects_card.dart`). No per-effect visualiser of the phenomenon was found. The Concept's Scatter instrument (distribution first, numbers second) has no counterpart.

### 7 Fold, do not miniaturise: SUPPORTED

- Effect cards fold the advanced rows behind a fold; heroes stay in front (`inspector/effects_card.dart`, `inspector/folds.dart`).
- Note: the Browser scales tiles by a factor with a square-root rule for marks (`tile.dart`). That is view zoom by the user, not a way to pack complexity.

### 8 Work loudest: UNVERIFIED

- Code is consistent: the Stage draws the picture and greys the world around it, and chrome is native.
- Measured in Theme: the Timeline wash is 30 percent grey (`timelineWash 0x4d606060`) over identity colours. The old Timeline is deliberately quieter than the Browser. The current golden keeps the Timeline vivid. This is a difference between old UI and golden, not a contradiction of the principle.
- Not checked against an old screenshot.

## What only the Concept has (NEW)

- **Relation as a first-class thing.** The words relation, scatter, stagger and along path do not appear anywhere in the old UI code. The nearest engine idea is a placement effect (`motolii.repeat`, "Place") and the Particles item.
- **Group as parent of parallel operations.** The old Timeline rows are layers, with property rows. A group's children draw as stripes. No group of parallel relations exists.
- **An instrument-first Inspector** where the phenomenon is the primary surface and numbers sit second.
- **Identity by what a thing does** (a relation) rather than what it is (a kind) or where it sits (a layer id).

## Which principles survive all three sources

Counting the old UI, the Concept and the visual golden together:

| Principle | Old UI | Concept | Golden |
|---|---|---|---|
| 1 Housing | yes | yes | yes |
| 3 Toybox | yes | yes | yes |
| 5 Place changes the body | yes | yes | yes |
| 7 Fold, do not miniaturise | yes | yes | yes |
| 2 Things have faces | uneven | yes | yes |
| 4 Identity travels | partial | yes | yes |
| 6 Phenomenon before parameters | partial | yes | yes |
| 8 Work loudest | unverified | yes | yes |

Principles 1, 3, 5 and 7 are already present in all three. Principles 4 and 6 are where the old UI is weakest and where the Concept adds the most.

## Open questions this raises

1. Should Browser kind identity and layer identity become one system? Today they are two, and the language says identity travels.
2. Where should a Relation's identity come from when the old UI identifies by kind and by layer id?
3. Does the old Timeline's deliberate wash (30 percent grey) reflect a reason worth keeping, given the golden is vivid?

## Resolution of the open questions (decided)

Colour carries no semantic duty. Layers and relations get colours from a palette automatically, stable within one Timeline, chosen to tell neighbours apart. Browser colour and Timeline colour need not match. So questions 1 to 3 above are closed: no identity system is needed to unify kind and layer colour. In the current golden, Scatter pink and Stagger blue in the Timeline are sample data, not a rule. The short brief is `design-brief.md`.
