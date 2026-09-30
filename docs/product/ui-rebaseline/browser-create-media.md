# Create / Media bodies — visual redesign (2026-09-28)

A visual redesign, not a restyle. The production UI is the source for capabilities, semantics and contracts; it is
not the visual reference.

## PRESERVE (inherited, never duplicated)

- Capabilities, product semantics, commands/intents (`create`, `placeAsset`, `applyEffect`…), data sources
  (`createKinds`, `assets`, `backgrounds`, asset `thumbnail` / `facts` / `seconds` / `peaks`).
- Picking, drag & drop (Media → Timeline/Stage), search and classes (`SearchCapability`, `ClassifyCapability`),
  Favorites / Recent / collections (`UserViews`), keyboard (the seat's `key`, `shows(order, columns)`), context menus
  (the seat's `more`), Dock / Leaf identity and travel.
- Face-generation mechanisms worth reusing: `MarkPainter` (Create's object marks), the asset thumbnails, the audio
  peaks, `PickedRing`.
- Product Home workspace hierarchy: the Browser is a small tool shelf beside the Stage. Work stays loudest.

## REPLACE (not inherited)

Tile geometry, card housing, paddings, caption placement, section layout and headings, body hierarchy, density,
the Create grid and the Media grid. Shared widgets that exist only because two bodies looked alike are not a reason
to keep looking alike.

## Theses

- **Create = Swiss symbol matrix.** Object marks on a strict grid, no idle housing (a ground only under the pointer
  or when picked), names only on hover / tooltip, a mark kept captioned only where the mark alone cannot tell it.
  Categories are row identifiers of the grid, not heading blocks.
- **Media = Swiss contact shelf.** Each family its own face (still, clip with play mark and length, sound as its own
  peaks, HDR plate, checker behind stills), picture area protected, context not repeated (no "Image" in Images, no
  48 kHz on every sound), one short name line.
- **Chrome = one compact row.** Classes, search and menu share a ~30 px row under the Leaf tab; the tab already
  names the panel, so the body never repeats it.

## Geometry (production starting points, not a licence to grow)

| | target |
|---|---|
| Browser seat | 300–324 px default; checked at ≤ 324 and at the narrow gate 280 (268 px at a 1280 px window) |
| Create key | 40–44 px footprint, row pitch 44–48 px, face 24–30 px, gap 4–6 px, section gap 8–12 px, label 9–10 px |
| Create row | ≥ 6 marks per row at 324, 7 when it fits |
| Media face | still/HDR 52–64 px, clip/sound 58–68 px wide, row 70–82 px, section title 10–11 px, ~4 per section, ≥ 3 at 280 |

## Gate

Before/after at the same window size, Browser width, UI scale and data; the whole screenshot at 25–35 %. Create reads
as a symbol board; its objects are told apart by silhouette and colour; Media's families are told apart at a glance;
the Browser is not louder than the Stage; more choices are visible in the same area.
