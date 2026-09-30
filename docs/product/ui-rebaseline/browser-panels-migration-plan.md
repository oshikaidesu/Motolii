# Migration plan and morphology (no implementation)

Follows `browser-panels-architecture-review.md`. Two corrections are accepted:

1. No shared responsive rule is derived from Create and Colors alone. Four bodies are treated as representative: Create, Effects, Colors, Fonts. The chassis provides available space. What to fold, hide or rearrange is the shelf's decision. No fold rule is fixed yet.
2. Before any panel polish, the New shell gets back the workspace semantics it lost. No new dock system. The existing `WorkspaceLayout`, `DockNode` and `WorkspaceView` are reused.

## 1. New shell to existing workspace

### What is there

`EditorWindow` (Classic) is three things stacked:

| Part | Where | Owner |
|---|---|---|
| Top row (File, Edit, View, sheet buttons, title) | `editor_window.dart` build | chrome |
| `WorkspaceView(layout, panelBuilder, onMove, onClose, onDetach, onLayoutChanged)` | same | workspace |
| Status line | same | chrome |

Everything that makes the workspace work lives in `_EditorWindowState`: the `WorkspaceLayout`, persistence (`persist`, `restore`), `_placePanel` (show, tab, drawer, hidden, window), `detachPane`, `movePane`, `closePane`, `Reset layout`, the detached-window branch (`c.windowInfo['main'] == false`), and `c.windowClosed`.

`NewShell` keeps the same session but replaces the middle with `_face()`: four fixed regions, `_browserTabs`, `_centerTabs`, an `opened` set and `stack()`. It ignores `hidden` and `window` placements. `routing.md` already lists this as NOT YET ROUTED: SH-19 Reset layout, WS-03 to WS-09 dock operations, SH-29, SH-30 and WS-13 detach windows.

What New adds that is worth keeping: its top bar (menus, transport, mode segment, keys), its status bar, its settings sheet, and its theme tokens.

### Target

```
New top bar  |  existing WorkspaceView  |  New status bar
```

The middle is not redesigned. The dock decides what sits where. New tokens reach it through the theme, and through the look of the tab strip inside `_Leaf`.

### Options for hosting

| Option | What | Cost | Risk |
|---|---|---|---|
| A. Chrome slots on `EditorWindow` | Give the existing window a top-bar and status-line builder plus a small handle (menu, sheet toggles, dirty title). New supplies its own chrome. | Small. One owner of the workspace logic. | The window state must expose a few actions. |
| B. Extract a workspace host | Move the workspace state into a shared host class both shells use. | Larger refactor of a 570-line state class. | Touches Classic, which must not regress. |
| C. Copy the logic into `NewShell` | Duplicate `_placePanel`, persistence, detach. | Small now. | Two copies drift. Not recommended. |

Recommendation: A. It reuses the working code, changes least, and keeps a single owner. B is the cleaner end state but should follow only if A proves awkward.

### Steps (each verifiable, Classic unchanged)

1. Add the chrome slots to the window. Classic passes its current top row and status line. Verify Classic looks and behaves the same.
2. `NewShell` mounts that window with New chrome. Remove `_browserTabs`, `_centerTabs`, `opened`, `stack`, `_face`, `_Region`, the browser-tab listener, and the no-op placement handler.
3. Restore the routed capabilities: dock drag and split, resize, close, reset layout, detach to window, drawers through Desk, layout persistence. Update the NOT YET ROUTED rows.
4. Check the detached-window path renders with the New theme and no full chrome.
5. Only then start panel polish.

### Checks

- Same layout file restores under both shells.
- Drag a tab to each edge and the centre; detach; close; reset.
- A detached window opens without the top bar.
- Narrow leaf shows icon-only tabs with the active name kept.
- Tab-stacked panels keep their state when switched (the dock already keeps them mounted).

### Open points

- Whether `initialDock()` should be the New default as it is (five browser panels in one leaf, Fonts in the catalog but not docked, `.205` and `.68` ratios).
- The New shell's Desk drawers (`c.deskDrawer`) versus the Desk panel host in the dock.
- How the chosen shell reaches a detached window process: the launch flag applies per launch, and a detached window is a second window of the same app.

## 2. Four morphologies

These are the bodies to test. Cells are things to check, not rules. Common baseline that already exists: the dock gives a panel any width and height (docked minimum is small; a detached window uses `minWidth` 260 and `minHeight` 160), and a tab strip that goes icon-only when crowded.

| | Create | Effects | Colors | Fonts |
|---|---|---|---|---|
| Body | Mark grid: symbol then name | Visual snapshot grid: the picture first | Instrument (wheel) plus swatches | Specimen list |
| The face that must survive | The mark | The rendered result | The wheel, or a colour you can pick | The specimen set in the font |
| Caption | Name under the mark; a small mark alone may still be readable | Name matters less than the picture | Not used (bare) | The family name is part of the face |
| Likely first to fold | Caption, then columns | Caption, then columns | Swatch rows, recent, palette names | Metadata (source, weight), then favourites |
| Wide | Many columns | Many columns | Wheel beside swatches | Specimen with metadata |
| Narrow | Fewer columns; icon-only marks | Fewer, smaller pictures | Wheel keeps a floor size; swatches go under it | One specimen per row; name kept |
| Tall | More rows | More rows | Wheel, palette and recent stack | More rows |
| Short | Scroll | Scroll | Wheel and one swatch row; rest scroll | Scroll |
| Tab-stacked | Icon and name tab | Icon and name tab | Icon and name tab | Icon and name tab |
| Detached | Same body at window size | Same | Compact instrument | Same |
| Might break | Losing captions makes similar marks (Rectangle and Rounded Rectangle) unreadable | Dropping the picture loses everything | Removing search does not help; the wheel arrangement itself must change | Dropping the specimen leaves a plain list |
| To test | How small a mark may go | Snapshot size floor | Wheel floor, and its placement change | What the minimum specimen is |

Media is close to Effects and can be added later.

## 3. Chassis responsibility versus shelf responsibility

Principle: the chassis tells the shelf how much space it has. The shelf decides what to fold, hide or rearrange.

| The chassis provides | The shelf decides |
|---|---|
| Available width and height | Whether captions show |
| Placement context: docked, tab-stacked, detached | How many columns, and the tile size |
| The dock tab strip (identity, close, detach menu) | Whether its own header, tools or editor show |
| Search, rail, filter and view controls as capabilities that can be shown or hidden | Which of those the shelf wants at this space |
| Selection, keys, drop hint | Where the editor sits (beside, above, under) |
| Persistence of user choices (rail width, tile size) | What survives when space runs out: its face |

Today the frame already passes width to the shelf through `layout(host, width, tile)`, and the shelf already owns `bare`, `showViews`, `header`, `editor` and `tools`. The change to test is small: let the frame also offer the shelf the available space before it draws its own search bar and rail, so the shelf can ask for a different arrangement. No global fold rule is written; each shelf may answer differently.

## 4. Not decided

- The hosting option (A recommended).
- Any threshold at which search, rail or captions fold.
- Whether the frame's own search bar and rail become shelf-controlled or only overridable.
- Whether in-app floating exists.
- The Relations panel (a feature, planned separately).

## 5. Order

1. Migration steps 1 to 4 (New shell onto the workspace). No visual polish.
2. Choose the shelf-space hand-over as a small change to the frame.
3. Polish Create, Effects, Colors and Fonts one at a time in the real window and judge by looking.
