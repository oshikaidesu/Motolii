# Browser-family panels: architecture review

Review only. Nothing was implemented. The attached concept (six panels, common anatomy, panel states) was used for its idea, not for pixels. All facts below come from reading the current Flutter code.

## 1. What the code already does

- **Every browser shelf is already an independent panel.** `foundation/panel_catalog.dart` lists Create, Media, Effects, Fonts, Colors and Files as separate `PanelSpec`s (category `Browse`). `panels/registry.dart` builds each one as `BrowserPanel(fixedTab: name, showTabs: false)`. So the six-panel idea is the existing model; the "Browser as a page switcher" reading was wrong.
- **Tabs are shared seats.** `initialDock()` (`workspace/layout.dart:146`) stacks five of them in one dock leaf. A tab is a `Draggable<String>`. Dropping on a leaf edge splits it: left and right at 18 percent, top and bottom at 22 percent, otherwise centre; the top 20 px of a leaf means join the tabs (`workspace_view.dart:285`).
- **Detach is an OS window.** Right-click on a tab gives Detach and Close. Detach calls `openPanelWindow` (`app/editor_window.dart:336`). There is no in-app floating layer.
- **Narrow tabs.** When labelled tabs do not fit, only the active tab keeps its word; the rest are icon-only, named on hover (`workspace_view.dart:200`).
- **Panel header already exists.** The dock's tab strip is the panel header: icon, name, a close key, and a right-click menu. A single-panel leaf shows one tab.
- **One frame, many shelves.** `BrowserPanel` owns search, tools, view switch, filter toggle, category rail, a rail grip, grid, selection and keys. A `BrowserShelf` supplies the panel-specific parts (`panels/browser/shelf.dart`): `preview`, `layout`, `editor`, `header`, `tools`, `bare`, `identity`, `rails`, `groups`, `apply`. Effects show the rendered effect (`NativeVisualSample`). Colors are a bare shelf with a wheel editor above swatches. Fonts set words in the font. Media shows thumbnails. Create draws line bodies for 3D and glyphs for the rest.

## 2. What the panels really share, and what is specific

Shared today (the frame):

- Search bar with tools and view switch (always shown).
- Category rail with a drag grip; double-click toggles it; below 48 px it becomes a small tab.
- Grid, list and thumbnail views, tile scale, selection, keyboard, drop hint.
- Filter view, user tags, collections, labels.
- Header row slot, editor slot, overlay slot.

Specific (the shelf):

- The face of an item: mark, rendered result, wheel, specimen, thumbnail.
- Tile layout (Colors uses a small custom cell; others use the standard grid).
- Whether tiles carry captions (`bare` for Colors and Fonts).
- The editor (Colors wheel and fill editor, Fonts style editor, Files path header, Effects reload header).
- What apply and double-click mean.

Minimum chassis inferred: **identity and title** (the dock tab), **panel actions** (close, detach, tab menu), **optional discovery controls** (search, rail, filter, views), **body**. Discovery controls are capabilities of the frame, not required parts. Today search is always drawn and the rail is user-collapsible but does not fold by itself.

## 3. How far the concept fits the current architecture

| Concept panel | Fit | Note |
|---|---|---|
| Create | exists | Polish faces. Many items are single glyphs (Null, Particles, Stage, Line, Bezier); 3D bodies already have line faces. |
| Relations | no data | The words relation, scatter, stagger and along path do not exist in the UI code. It would be a new shelf, and needs an engine-side capability to list and apply relations. This is a feature, not a visual change. |
| Effects | exists | Rendered snapshots already do what the concept shows. |
| Colors | exists | Wheel plus swatches already are the instrument. |
| Fonts | exists | Words set in the font. In the catalog but not in `initialDock()`. |
| Media | exists | Thumbnails, and a mesh draws its body. Audio waveform is not drawn (icon only) in the code read. |
| Files | exists | Path header plus grid. |

Adding a panel today touches three places: a `PanelSpec` in the catalog, the shelf in the list at `panels/browser.dart:65`, and a hard-coded name list in `panels/registry.dart`. The New shell keeps a second copy of the shelf list (`app/new/browser.dart:32`). The name list in the registry could read the `Browse` category instead. That is a small tidy, not a new abstraction.

Each `BrowserPanel` constructs all six shelves although `fixedTab` uses one. It is not a visual issue; noted only.

## 4. Two conflicts found

1. **The New shell (Phase B, `app/new_shell.dart`) fixes four regions and lists the browser panels as tabs of one region** (`_browserTabs`). This is the "navigation" reading, and it drops dock, detach and drag. It should host the panels through the existing workspace instead.
2. **The concept's Floating state does not exist.** The code detaches to an OS window only. In-app floating would be new behaviour and a decision, not a visual polish.

## 5. Morphology matrix

Current behaviour is what the code does now. Proposal is what the panel should do (design proposal, not yet validated). Rule for all: fold before shrink, scroll before crushing, keep the face.

| Panel | Wide | Narrow | Tall | Short | Tab-stacked | Detached (OS window) |
|---|---|---|---|---|---|---|
| Create | Grid, rail, caption. Now: same. | Fewer columns, then list of mark and name. Rail folds to a tab (exists, manual). Search folds to an icon (proposal). | More rows, no change. | Scroll. | Tab shows icon and name, or icon only when crowded (exists). | Same body at window size; minimum 260 x 160 (exists). |
| Relations | New. Proposal: phenomenon faces in a grid. | Two columns, then a list with a small living face. | As Create. | Scroll. | As Create. | As Create. |
| Effects | Rendered tiles, rail. Now: same. | Fewer columns; keep the picture, drop caption first. | More rows. | Scroll. | As Create. | As Create. |
| Colors | Wheel with swatches beside or under (exists, wheel height is draggable). | Wheel shrinks to a floor, swatches wrap below (proposal). | Wheel plus swatches plus recent. | Wheel first, swatches scroll (proposal). | As Create. | Compact instrument: wheel and one swatch row. |
| Fonts | List of specimens with metadata. | Drop metadata, keep the specimen and the name. | More rows. | Scroll. | As Create. | Same. |
| Media | Thumbnail grid. | Fewer columns, then list. | More rows. | Scroll. | As Create. | Same. |
| Files | Path header plus grid. | Path collapses to the last folder, then an icon. | As Create. | Scroll. | As Create. | Same. |

Not decided by the code: at what width the search field becomes an icon and the rail auto-folds. Today both are user actions. A single automatic fold rule in the frame would give every browser panel the same behaviour.

## 6. Conclusion

### KEEP

- The whole workspace: dock, split, resize, tab stack, drag and drop, detach to window, close, narrow compact tabs, layout persistence.
- Independent panels for Create, Media, Effects, Fonts, Colors, Files. No Browser container and no global page switcher.
- `BrowserPanel` and `BrowserShelf` as the frame and the shelf contract. A new panel is a new shelf.
- Per-shelf faces already present: Effects snapshots, Colors wheel, Fonts specimen, Media thumbnails, Create 3D bodies.
- The dock tab strip as the panel header (identity, close, detach menu).

### CHANGE

- Faces that are weak: Create's single-glyph items; Media audio shows an icon, not a waveform.
- Chrome: the panel and tab visual polish inside the existing tab strip and frame.
- Frame folding: one automatic rule for narrow widths (search to icon, rail to tab, captions dropped) so every browser panel morphs the same way.
- The registry's hard-coded name list, to read the catalog category.
- The New shell: host panels through the workspace, not a fixed `_browserTabs` region.
- Decide whether in-app floating is wanted at all; today only an OS window exists.
- Relations: a real feature (engine capability and a new shelf), scheduled apart from the visual work.

### DO NOT COPY FROM THE CONCEPT

- A Browser container or any switcher between panels.
- The full header, search, view controls and category filter as mandatory parts of every panel.
- All six panels having the same width, height and header anatomy.
- Identical cell structure for Create and Relations.
- Coloured icon chips per panel as a semantic system (colour carries no meaning here).
- The Floating panel state as drawn, and the pin and minimise keys, until they are decided as behaviour.
- The Japanese and marketing text, version line, "Concept Design" labels.
- Any pixel value from the generated image.

## 7. Suggested next step

Implement one or two panels only, inside the existing `BrowserShelf`, and judge by looking. The best candidates are Create (faces) and Colors (instrument). Fix the frame's narrow-fold rule at the same time, since it is shared.
