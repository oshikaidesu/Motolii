# Capability inventory — Browser (Create / Media / Effects / Fonts / Colors / Files)

Source: full read of `motolii/ui/lib/panels/browser.dart` and every file in `motolii/ui/lib/panels/browser/` (23 files, 6074 lines), plus call sites in `panels/registry.dart`, `session/session_commands.dart`, `session/session_files.dart`, `session/session_native.dart`, `panels/panel_settings.dart`, `panels/inspector/content_cards.dart`, `panels/gradient_inspector.dart`, `panels/stage/touch.dart`, `app/editor_window.dart`. Intent from `docs/stage5/product-contract.md` (§ Browser row, l.9/14/16/92) and `docs/stage5/browser-rebuild.md` (Phase 4 = REJECTED; AEViewer look, no invented capabilities).

Path prefixes used in the Code column: `B/` = `motolii/ui/lib/panels/browser/`, `BR` = `motolii/ui/lib/panels/browser.dart`.

**Important structural fact:** in production every shelf is its **own dock panel** (`motolii/ui/lib/panels/registry.dart:42-48` builds `BrowserPanel(fixedTab: name, showTabs: false)` for Create/Media/Effects/Fonts/Colors/Files; catalog group "Browse", `motolii/ui/lib/foundation/panel_catalog.dart:82-87`). The in-panel tab strip and the `browserTab` listener are therefore **unreachable in the shipped app** (only tests construct `BrowserPanel` without `fixedTab`). Rows for them are kept (marked "dormant") so the capability is not lost.

"Effect path" names: `command:<op>` = `EditorSession.command(op)` → Rust `DocumentOperation`; `native:<m>` = method-channel call (Swift `macos/Runner/MainFlutterWindow.swift` for `reveal`/`openFile`/`pickImport`; Rust `motolii/ui/native/src/lib.rs` for `request{op:fontFacts}`); `desk:<key>` = `controller.storeDesk(key)` (persisted user/workspace settings, not document).

---

## Table

### Frame — panel shell, search bar, view switch (all shelves)

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| BR-001 | Switch shelf via in-panel tab strip (Create/Media/Effects/Fonts/Colors/Files) | click tab button | dormant: only when `showTabs && fixedTab==null` (no production caller) | BR:360-375, BR:301-308 | UI-local (`changeTab`) | PRESERVE + MOVE | Shelf switching is today done by dock tabs; new shell must still let every shelf be reached |
| BR-002 | Each shelf opens as its own dockable panel (tab / window / hidden per Settings) | Settings panel placement / dock | always | `motolii/ui/lib/panels/registry.dart:42-48` | panel placement | PRESERVE + MOVE | Reachability of all 6 shelves is the capability; location may change |
| BR-003 | Search current shelf by name (substring, case-insensitive) | type in "Search <Shelf>" field | always | B/frame_bars.dart:40-68; BR:242-254 | UI-local | PRESERVE | Core filtering |
| BR-004 | Search text remembered per shelf when switching shelves | switch tab | dormant with tab strip (per-panel instance otherwise) | BR:303-305, BR:70 | UI-local | PRESERVE | Keeps each shelf's context |
| BR-005 | Focus + select-all search text | Cmd/Ctrl+F while panel focused | panel focused, not typing in a field | B/frame_keys.dart:17-24 | UI-local | PRESERVE | Standard find shortcut |
| BR-006 | Clear search; if empty, clear selection | Esc while panel focused | not typing in a field | B/frame_keys.dart:25-35 | UI-local | PRESERVE | Two-stage escape |
| BR-007 | Show/hide Filter View band | click filter icon beside search (tooltip "Show filters"); icon accent when shown or any filter active | shelf has filter groups (Media/Effects/Fonts/Colors always; Create/Files only once user tags exist on that shelf) | B/frame_bars.dart:74-93; BR:381-389 | UI-local (`filtersShown[tab]`) | PRESERVE + CONTEXTUALIZE | Filter band is large; can be on-demand but toggle must remain |
| BR-008 | Switch view Grid / List / Thumbnails | click one of three icon buttons | shelf `showViews` (Create, Media, Effects, Files; not Fonts/Colors) | B/parts.dart:77-113; BR:383-385 | desk:`browserView` (one value shared by all shelves) | PRESERVE | View modes are a real browsing capability |
| BR-009 | Shelf-specific tool buttons beside search (Import / From image) | click | per shelf (see BR-060, BR-120) | B/frame_bars.dart:70-73 | per shelf | PRESERVE + MOVE | Slot for shelf actions |
| BR-010 | Empty state "No matches" | automatic | visible list empty | BR:541-542; B/frame_bars.dart:197-211 | UI-local | PRESERVE | Feedback |
| BR-011 | Grid scrolls vertically | wheel / trackpad | list overflows | BR:547-575 | UI-local (`scroll`) | PRESERVE | — |
| BR-012 | Auto-reveal newly imported assets: Media shelf clears search & category, selects the new ids | automatic after any import (menu Import, drop, Files double-click, Import button) | Media panel exists (panel with fixedTab Media or tabbed browser) | BR:260-275; `motolii/ui/lib/session/session_files.dart:94-120` | UI-local (listens `importedAssets`) | PRESERVE | Import feedback loop |
| BR-013 | Be pointed at Fonts / Colors from Inspector (font row, colour swatch, gradient stop) | click in Inspector | Inspector action; shows Fonts/Colors panel via `placePanel(...,'show')` (tab-strip path `_revealTab` dormant) | BR:295-299; `motolii/ui/lib/session/session_commands.dart:23-41` | `browserTab`, `placePanel`, command:`focusColor` | PRESERVE + CONTEXTUALIZE | Only route from Inspector to font/colour choosing; keep cross-link |

### Frame — category rail, rail grip, collections, labels

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| BR-020 | Narrow list to a category (rail entry; first = whole shelf) | click rail entry | rail width ≥ 48 | B/frame_bars.dart:142-143; B/shelf.dart:115; BR:152-155 | UI-local (`classifications[tab]`) | PRESERVE | Contract l.14: classification must remain |
| BR-021 | Resize category rail | drag rail grip (vertical line) | always | BR:436-448 | desk:`browserRail` (0–200 px; <48 stores 0) | PRESERVE | Layout pref |
| BR-022 | Collapse / expand rail | double-click rail grip | always | BR:449-452 | desk:`browserRail` (0 ↔ 96) | PRESERVE | — |
| BR-023 | Folded rail shows turned label (current category or shelf) ; click reopens at 96 px | click folded rail tab ("Show … categories") | rail < 48 | B/frame_bars.dart:151-194; BR:427-435 | desk:`browserRail` | PRESERVE | Recovery path from collapsed rail |
| BR-024 | Filter shelf to one of 7 collections (toggle; click again clears) | click collection row in rail | rail open | B/rail_collections.dart:106-108; BR:406-411 | UI-local (`filter.collection`) | PRESERVE | Live-style collections |
| BR-025 | Rename a collection in place | double-click collection row → text field; Enter keeps, focus loss discards | rail open | B/rail_collections.dart:62-92, 109-116, 50-52 | desk:`collectionNames` | PRESERVE | User naming |
| BR-026 | Add dragged items to a collection | drop a tile (BrowserDrag = picked ids, or Media asset drag = that one asset) on a collection row; row highlights while hovering | rail open | B/rail_collections.dart:94-102, 120-124; BR:414-415 | desk:`collections` | PRESERVE | Drag-to-collect |
| BR-027 | Saved filter labels listed in rail; click restores filter + rail category + search text and opens Filter View | click label row | at least one label on this shelf | B/rail_collections.dart:163-172; B/frame_filters.dart:91-99 | UI-local (+ reads desk:`labels`) | PRESERVE | Saved searches |
| BR-028 | Forget a saved label | click × beside label ("Forget this label") | label exists | B/rail_collections.dart:173-187; BR:413 | desk:`labels` | PRESERVE | — |
| BR-029 | Collection tooltip teaches drop / number key / rename | hover collection row | — | B/rail_collections.dart:103-105 | UI-local | PRESERVE | Discoverability |

### Frame — Filter View (tag groups)

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| BR-030 | Filter by a tag (exclusive within group; click again clears) | click tag chip | Filter View shown | B/filter_view.dart:121-136, 232-235; B/frame_filters.dart:66-78 | UI-local (`ShelfFilter`) | PRESERVE | Groups AND, tags OR |
| BR-031 | Add/remove a tag to the group's selection (multi within group) | Cmd/Ctrl+click tag chip | Filter View shown | B/filter_view.dart:232-235; B/frame_filters.dart:68-69 | UI-local | PRESERVE | Modifier semantics |
| BR-032 | Fold / unfold a filter group (folded header shows active tags in accent) | click group name/arrow | Filter View shown | B/filter_view.dart:76-115; BR:492-498 | desk:`folds` | PRESERVE | — |
| BR-033 | "Values" groups auto-populated from what items actually carry (Resolution, Frame rate, Styles, Parameters, Blend, Stops) | automatic | shelf declares `FilterKind.actual` | B/frame_filters.dart:16-23, 133-142 | UI-local | PRESERVE | Data-driven facets |
| BR-034 | Add a custom numeric range chip to a range group (min–max, either end blank) | click + → two fields (autofocus min) → Enter | range group (Media Duration) | B/filter_view.dart:137-142, 283-360; BR:505-518 | desk:`ranges` | PRESERVE + VISUALIZE | Candidate for a range gadget; keep numeric entry |
| BR-035 | Remove a range chip | click × on range chip | range group | B/filter_view.dart:133-135, 261-274; BR:519-536 | desk:`ranges` | PRESERVE | — |
| BR-036 | User-tag group ("Tags") appears as a filter group | automatic | shelf has any user tag | B/frame_filters.dart:31-32; B/filter_library.dart:220 | UI-local | PRESERVE | Own tags are filterable |
| BR-037 | See result count + number of active filters | read Results bar | Filter View shown | B/filter_view.dart:152-178 | UI-local | PRESERVE | Feedback |
| BR-038 | Clear all filters (tags + collection) | click Clear | any filter active | B/filter_view.dart:180-196; BR:500-503 | UI-local | PRESERVE | — |
| BR-039 | Save current filter (+ category + search) as a rail label | click "Add label" icon | any filter active | B/filter_view.dart:197-210; B/frame_filters.dart:80-89 | desk:`labels` | PRESERVE | Saved searches |

### Frame — tile / grid interaction (all shelves unless stated)

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| BR-040 | Select a tile (also focuses panel for keys) | primary pointer-down on tile | always | B/tile.dart:158-161; BR:311-337 | UI-local (`selected[tab]`) | PRESERVE | — |
| BR-041 | Range-select tiles | Shift+click (Cmd/Ctrl+Shift adds range to existing) | multiSelect shelves only (Media, Effects) and an active tile exists | BR:317-326 | UI-local | PRESERVE | Finder semantics |
| BR-042 | Toggle a tile in selection | Cmd/Ctrl+click | Media, Effects | BR:327-329 | UI-local | PRESERVE | — |
| BR-043 | Select all visible | Cmd/Ctrl+A | Media, Effects; panel focused | B/frame_keys.dart:48-52 | UI-local | PRESERVE | — |
| BR-044 | Move selection by keyboard (←→ by 1, ↑↓ by row of columns, Home/End) ; Shift+arrow extends range | arrow keys / Home / End | panel focused, list non-empty; Shift-range only Media/Effects | B/frame_keys.dart:59-69; BR:311-337 | UI-local | PRESERVE | Keyboard browsing |
| BR-045 | Apply the active (or first) item | Enter / numpad Enter | panel focused, list non-empty | B/frame_keys.dart:55-58 | shelf apply (see shelves) | PRESERVE | Keyboard apply |
| BR-046 | Apply item by double-click | double-click tile | non-bare shelves (Create, Media, Effects, Files); Fonts when no text layer is active | B/tile.dart:169-171; B/shelf.dart:128 | shelf apply | PRESERVE | Stray single click must not add layers |
| BR-047 | Apply item by single click | click tile | bare shelves (Colors; Fonts with a text layer) and item supported | B/tile.dart:164-166 | shelf apply | PRESERVE | Dressing is cheap, one click |
| BR-048 | Delete/Backspace on active item | Delete / Backspace | panel focused; only Media implements (others no-op) | B/frame_keys.dart:70-73; B/shelf.dart:205 | Media: command:`removeAsset` | PRESERVE | See BR-079 |
| BR-049 | Put picked tiles into collection N / remove from collection | press 1–7 / press 0 | selection non-empty, no modifier | B/frame_keys.dart:40-47; B/filter_library.dart:81-91 | desk:`collections` | PRESERVE | Note: 0 removes (digit ≤ 7 includes 0) — undocumented in tooltip |
| BR-050 | Focus quick-tag "Add…" field | Cmd/Ctrl+E | selection non-empty | B/frame_keys.dart:36-39 | UI-local | PRESERVE | — |
| BR-051 | Context menu: title row + facts rows (disabled, informational) | right-click tile | always; right-click also selects | BR:596-617; B/tile.dart:167-168 | UI-local | PRESERVE | Metadata is only visible here |
| BR-052 | Context menu: Apply (shelf label: Place / Open / Import to Media / Apply) | right-click → first action | always enabled (apply guards internally) | BR:618-621, 658 | shelf apply | PRESERVE | — |
| BR-053 | Context menu: add picked + clicked items to collection 1–7 | right-click → coloured collection row | always | BR:626-643, 651-656 | desk:`collections` | PRESERVE | — |
| BR-054 | Context menu: Remove from collection | right-click → "Remove from collection" | clicked item is in a collection | BR:644-648 | desk:`collections` | PRESERVE | — |
| BR-055 | Drag tile(s) inside the Browser (to rail collection) | drag tile; feedback shows name or "N rows" | shelves without their own drag (Create, Effects, Fonts, Colors, Files; Media only when item unsupported) | B/tile.dart:255-269 | UI-local drag `BrowserDrag` | PRESERVE | — |
| BR-056 | Hover tooltip: name, Missing file, In use by a layer, detail, how to apply / "Apply unavailable" | hover tile | always | B/tile.dart:142-154 | UI-local | PRESERVE | Carries the apply-gesture hint |
| BR-057 | Long names slide to reveal end on hover | hover tile caption | name overflows | B/parts.dart:171-303; B/tile.dart:125-130 | UI-local | PRESERVE | — |
| BR-058 | Thumbnails mode shows name band on hover/selection | hover or select | view = Thumbnails | B/tile.dart:186-226 | UI-local | PRESERVE | — |
| BR-059 | Status marks on tile: collection dot (colour), "used" dot, missing-file warning, format badge | automatic | per item state | B/frame_grid.dart:109-137; B/tile.dart:58-110 | UI-local | PRESERVE + VISUALIZE | State at a glance |
| BR-059a | Selected-tile frame | automatic | selected | B/tile.dart:172-180 | UI-local | PRESERVE | — |

### Frame — bottom bars (quick tags, size bar, count)

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| BR-060a | Quick-tags band: see picked item name / "N rows", its built-in tags (read-only, intersection across picked), user tags | automatic | selection non-empty | B/frame_filters.dart:102-130; B/quick_tags.dart:50-65, 94-128 | UI-local | PRESERVE + CONTEXTUALIZE | Already contextual |
| BR-061 | Add a user tag to all picked items | type in "Add…" → Enter | selection non-empty | B/quick_tags.dart:67-92; B/frame_filters.dart:127 | desk:`tags` | PRESERVE | — |
| BR-062 | Remove a user tag from all picked items | click × on user tag chip | selection has user tags | B/quick_tags.dart:56-61, 156-167 | desk:`tags` | PRESERVE | — |
| BR-063 | Tile size − / + (10 % steps) | click − or + | always | `motolii/ui/lib/foundation/panel_controls/scale.dart:131-142,160,174`; B/frame_grid.dart:81-90 | desk:`browserTile` (72–240 px) | PRESERVE | — |
| BR-064 | Tile size slider | drag slider | bar wide enough (`sliderRoom`) | scale.dart:161-171 | desk:`browserTile` | PRESERVE | — |
| BR-065 | Tile size exact percent | type/scrub in percent field | always | scale.dart:150-157 | desk:`browserTile` | PRESERVE | Exact value |
| BR-066 | Count label: "N of M selected" / "N of M shown" / "M items"; tooltip with all three numbers; compact to bare number when narrow | read / hover | always | B/frame_grid.dart:47-73, 94-105 | UI-local | PRESERVE | — |

### Create shelf

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| BR-070 | Categories All / Text / Shapes / Paths / 3D / Helpers | rail click | — | B/create_shelf.dart:17-24, 75-87 | UI-local | PRESERVE | (primitives not matching a shape word classify "Other" and appear only under All) |
| BR-071 | Add Text layer | double-click / Enter / menu Apply on "Text" | has('create') | B/create_shelf.dart:28, 122-125 | command:`create{kind:text}` (+visibleFrames) | PRESERVE + MOVE | Also a toolbar candidate |
| BR-072 | Add shape layer: Rectangle, Rounded Rectangle, Ellipse, Star, Polygon | same | has('create') | B/create_shelf.dart:30-37 | command:`create{kind:<id>}` | PRESERVE | — |
| BR-073 | Add Null | same | has('create') | B/create_shelf.dart:38-43 | command:`create{kind:null}` | PRESERVE | — |
| BR-074 | Add Camera | same | has('create') | B/create_shelf.dart:44 | command:`create{kind:camera}` | PRESERVE | — |
| BR-075 | Add Particles emitter | same | has('create') | B/create_shelf.dart:45-50 | command:`create{kind:particles}` | PRESERVE | — |
| BR-076 | Add Stage layer (widen working area) | same | has('create') | B/create_shelf.dart:51-56 | command:`create{kind:stage}` | PRESERVE | Contract l.88 |
| BR-077 | Add Line / Bezier path layer | same | has('create') | B/create_shelf.dart:57-68 | command:`create{kind:line|bezier}` | PRESERVE | — |
| BR-078 | Add bundled 3D primitive (list from native `primitives`) | same | has('create') | B/create_shelf.dart:69-71 | command:`create{kind:<primitive id>}` | PRESERVE | — |

### Media shelf

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| BR-080 | Categories All / Video / Images / HDR / Audio / 3D | rail click | — | B/media_shelf.dart:29-36, 53-56 | UI-local | PRESERVE | — |
| BR-081 | Opens in Thumbnails view by default | automatic | no stored `browserView` | B/media_shelf.dart:26 | UI-local | PRESERVE | (overridden by global view pref) |
| BR-082 | Filter groups Kind / Resolution / Frame rate / Duration (ranges) / Source (Imported, Bundled, Reference, Material) | Filter View | — | B/media_shelf.dart:63-107 | UI-local | PRESERVE | — |
| BR-083 | Import files (OS picker limited to importExtensions) | click "Import" tool | has('import') (else disabled "· unavailable") | B/media_shelf.dart:141-146; session_files.dart:85-90 | native:`pickImport` → command:`import{paths}` | PRESERVE + MERGE | Same as File ▸ Import menu (`app/editor_window.dart:394-396`) |
| BR-084 | "Drop to import" hint while OS files are dragged over the window | OS drag hover | Media shelf visible; `dragging` from native `dragHover` | B/media_shelf.dart:276-303; session_native.dart:188-195 | UI-local (drop itself handled window-level `filesDropped`) | PRESERVE + MOVE | Hint only; drop is global |
| BR-085 | Place asset as layer | double-click / Enter / menu "Place" | has('placeAsset') and not missing | B/media_shelf.dart:127-135 | command:`placeAsset{id}` (+visibleFrames) | PRESERVE | — |
| BR-086 | Place bundled HDRI (environment layer) | same on HDR builtin tile | has('create') | B/media_shelf.dart:41-49, 129-131 | command:`create{kind:background:<id>}` | PRESERVE | Contract l.92 |
| BR-087 | Drag asset onto Timeline to place at drop position | drag tile to Timeline row | item supported | B/media_shelf.dart:246-271 | drag data `{asset,name}` → Timeline `placeAsset` | PRESERVE | Only drag-to-time path |
| BR-088 | Replace selected layer's source with this asset | menu "Replace selected layer" | not builtin; has('replaceAsset'), not missing, a layer selected (else disabled) | B/media_shelf.dart:174-181, 221-222 | command:`replaceAsset{id}` | PRESERVE + CONTEXTUALIZE | Only UI route for replaceAsset |
| BR-089 | Reveal in Finder / Explorer | menu | not builtin, has path, not missing | B/media_shelf.dart:182-183, 223-224 | native:`reveal{path}` | PRESERVE | — |
| BR-090 | Open with default app | menu | not builtin, has path, not missing | B/media_shelf.dart:184-187, 225-226 | native:`openFile{path}` | PRESERVE | — |
| BR-091 | Extract palette from image into saved swatches (switches Colors rail to Saved) | menu "Extract palette" | not builtin, image MIME, path, not missing | B/media_shelf.dart:188-192, 231-232; B/colors_shelf.dart:279-293 | desk:`swatches` (UI k-means `paletteOf`) | PRESERVE + MERGE | Same as Colors "From image" (BR-121) |
| BR-092 | Copy file path | menu "Copy path" | not builtin, has path | B/media_shelf.dart:194-198, 227-228 | clipboard | PRESERVE | — |
| BR-093 | Relink / Locate missing file (picker limited to same family extensions) | menu "Locate file…" (missing) / "Relink to another file…" | not builtin, has('relinkAsset') | B/media_shelf.dart:199-203, 307-332 | native:`pickImport` → command:`relinkAsset{id,path}` | PRESERVE + CONTEXTUALIZE | Only UI route for relink |
| BR-094 | Remove asset from library | menu "Remove from library" / Delete key | not builtin, has('removeAsset'), not used by a layer (else disabled / no-op) | B/media_shelf.dart:204-208, 233-234, 239-242 | command:`removeAsset{id}` | PRESERVE | Only UI route for removeAsset |
| BR-095 | Facts in menu: format·MIME, W×H · fps · kHz · ch · duration, file size, parent folder, Missing, In use | right-click | per item | B/media_shelf.dart:149-163, 335-365 | UI-local | PRESERVE + MOVE | Only place asset metadata is shown |
| BR-096 | Missing / used marks and 3D shape line-art thumbnail by filename | automatic | per item | B/tile.dart:58-83; B/media_shelf.dart:402-456 | UI-local | PRESERVE | — |

### Effects shelf

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| BR-100 | Categories All / Blur / Light / Color / Stylize / Distort / 3D / Path / Place / Other | rail click | — | B/effects_shelf.dart:21-32, 87-105 | UI-local | PRESERVE | — |
| BR-101 | Filter groups Seat / Applies to / Time / Inputs / Parameters / Origin | Filter View | — | B/effects_shelf.dart:111-163 | UI-local | PRESERVE | — |
| BR-102 | Apply one or all picked effects to the selected layer(s) | double-click / Enter / menu Apply | has('applyEffect'), a layer selected; Path-stage effects need a Shape layer selected | B/effects_shelf.dart:166-175, 197-206 | command:`applyEffect{pluginIds}` | PRESERVE + CONTEXTUALIZE | Only UI route for adding effects |
| BR-103 | Reload effect shelf (re-scan user shaders) | click refresh icon in header | has('reloadEffects') | B/effects_shelf.dart:45-68 | command:`reloadEffects` | PRESERVE | ISF-Editor style reload |
| BR-104 | See why an effect was refused (one line in error colour, full text in tooltip) | read / hover header | has('reloadEffects') and a notice exists | B/effects_shelf.dart:70-80 | UI-local (`effectsNotice(state)`) | PRESERVE + MOVE | Only place shader errors surface |
| BR-105 | Effect snapshot preview rendered natively (glyph fallback) | automatic | — | B/effects_shelf.dart:178-194 | native visual sample `{kind:effect}` | PRESERVE | — |

### Fonts shelf

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| BR-110 | Categories All / Used here (families used by any text layer; used dot) | rail click | — | B/fonts_shelf.dart:80, 127-155 | UI-local | PRESERVE | — |
| BR-111 | Font facts loaded once (scripts, styles, axes, mono, colour) and shown as chips on each row | automatic on first show | native reply | B/fonts_shelf.dart:28-48, 235, 256-277 | native:`request{op:fontFacts}` (Rust) | PRESERVE | — |
| BR-112 | Filter groups Script / Styles / Axes / Kind (Mono, Color) | Filter View | facts loaded | B/fonts_shelf.dart:94-124 | UI-local | PRESERVE | — |
| BR-113 | Specimen: each family drawn with the active text layer's first line (or its own name); current family marked with accent bar | automatic | — | B/fonts_shelf.dart:194-252 | UI-local | PRESERVE | — |
| BR-114 | Set font family of the chosen character scope of active text layer | single click / Enter / menu Apply | active layer is Text, not locked, has('setFont') | B/fonts_shelf.dart:173-176, 289-303 | command:`setFont{layer,scope,…,family}` | PRESERVE + MOVE + CONTEXTUALIZE | Brief example: Fonts near Text selection |
| BR-115 | Create a new text layer in that face | double-click / Enter | no active text layer; has('create') | B/fonts_shelf.dart:182-187, 292-295 | command:`create{kind:text,family}` | PRESERVE | Shortcut path |
| BR-116 | Hint "Double-click a face to add a text layer" | automatic | no active text layer | B/fonts_shelf.dart:319-326 | UI-local | PRESERVE | — |
| BR-117 | Header shows "<layer name> · <content>" of the layer being dressed | automatic | active text layer | B/fonts_shelf.dart:342-347 | UI-local | PRESERVE | — |
| BR-118 | Choose character scope: All text / Hiragana / Katakana / Kanji / Latin / Uppercase / Lowercase | dropdown `EditorChoice` | text layer, enabled, supports('styleText') | B/fonts_shelf.dart:54-62, 352-363 | UI-local → `textStyleTarget` (shared with Inspector rich text) | PRESERVE + MERGE | Scope selector also relevant to rich-text editor |
| BR-119 | Set character size (px) for the scope, with live preview, commit, cancel | type / scrub numeric field | same as BR-118 | B/fonts_shelf.dart:368-380 | command:`styleText{size,preview}` → `commitPreview` / `cancelPreview` | PRESERVE + MOVE | Inspector deliberately hides `.size` (content_cards.dart:18) — only route |
| BR-119a | Set justification Left / Center / Right | click one of 3 icon buttons | layer has `text_justify` property; setProperty available | B/fonts_shelf.dart:382-393, 406-446 | command:`setProperty{property:text_justify}` | PRESERVE + MOVE | Inspector hides text_justify (content_cards.dart:17) — only route |

### Colors shelf

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| BR-120 | Categories All / Saved / Used here / Starter | rail click | — | B/colors_shelf.dart:55-75 | UI-local | PRESERVE | — |
| BR-121 | Extract palette from an image file into Saved swatches | click "From image" tool → OS picker (multi) | — | B/colors_shelf.dart:155-157, 268-293 | native:`pickImport` → desk:`swatches`; error "No colours found…" | PRESERVE + MERGE | Duplicates BR-091 |
| BR-122 | Filter groups Kind (Solid/Gradient) / Blend / Stops / Source | Filter View | — | B/colors_shelf.dart:80-111 | UI-local | PRESERVE | — |
| BR-123 | See what the wheel edits ("Layer · Fill / Stroke / Fill · stop", "Composition · Background") | read title | a colour target exists | B/colors_shelf.dart:167-182, 339-350 | UI-local | PRESERVE | — |
| BR-124 | Pick hue on ring | drag / click ring | has('setColor') | B/color_picker.dart:97-128, 204-216; B/color_wheel.dart:35-37 | command:`previewColor` (queued) → `commitPreview` or `setColor` | PRESERVE + VISUALIZE | Only UI route for editing any colour value (Inspector hides colour rows) |
| BR-125 | Pick saturation/value in inner square or triangle (hue kept for greys) | drag / click inner area | has('setColor') | B/color_picker.dart:97-128; B/color_wheel.dart:79-96 | same as BR-124 | PRESERVE + VISUALIZE | — |
| BR-126 | Switch wheel inner shape Square ↔ Triangle | click "Square/Triangle" label | — | B/color_picker.dart:304-329 | desk:`colorShape` | PRESERVE | — |
| BR-127 | Enter hex colour (3 or 6 digits, validated) | type in hex field, commit | has('setColor') | B/color_picker.dart:246-270 | command:`setColor` (or local `picked` if no target) | PRESERVE | Exact numeric entry |
| BR-128 | Eyedropper: pick colour from Stage | click pipette toggle, then click Stage; Esc cancels | — | B/color_picker.dart:273-303, 333-350; `motolii/ui/lib/panels/stage/touch.dart:217-229` | `eyedropper` flag → Stage → command:`applyPalette` | PRESERVE | — |
| BR-129 | Alpha slider | drag slider | target carries alpha (text, params) | B/color_picker.dart:352-387 | command:`previewColor` → commit | PRESERVE + VISUALIZE | — |
| BR-130 | Cancel an in-progress wheel drag (reverts preview) | Esc (picker focused), app backgrounded, window loses focus, target changes | drag in progress | B/color_picker.dart:58-65, 80-86, 161-180, 190-195 | command:`cancelPreview` | PRESERVE | Safety |
| BR-131 | Unbound wheel (no target): wheel/hex just set a scratch colour | wheel / hex | no colour target | B/color_picker.dart:118-119, 259-261; B/colors_shelf.dart:196-197 | UI-local (`picked`) | PRESERVE | Scratch colour (not persisted) |
| BR-132 | Change fill type Solid / Linear / Radial / Angular / Diamond (tiles preview current fill each way) | click tile | active layer has a `fill` map and has('setGradient') | B/fill_definitions.dart:97-126; B/colors_shelf.dart:202-206 | command:`setFillMode{gradient:false}` / `setGradient{kind}` | PRESERVE + MERGE | setFillMode only reachable here; gradient inspector edits stops |
| BR-133 | Change gradient blend RGB / Linear / Oklab / Oklch / Oklch long / Steps | click tile | as BR-132 and kind ≠ solid | B/fill_definitions.dart:127-149 | command:`setGradient{blend}` | PRESERVE + MERGE | — |
| BR-134 | Save current colour / gradient as a swatch (switches rail to Saved) | click "Save current color" | colour target exists | B/colors_shelf.dart:207-219, 298-318 | desk:`swatches` | PRESERVE | — |
| BR-135 | Resize wheel | drag horizontal grip under picker | — | B/colors_shelf.dart:220-229, 33-37 | desk:`browserWheel` (clamped 96–200 by width) | PRESERVE | — |
| BR-136 | Apply solid swatch to selection | single click / Enter / menu Apply | has('applyPalette'), a layer selected | B/colors_shelf.dart:124-151 | command:`applyPalette{rgba}` | PRESERVE | — |
| BR-137 | Apply gradient swatch to current fill slot | same | has('setGradient'), layer selected, a slot resolved | B/colors_shelf.dart:138-148 | command:`setGradient{slot,stops,blend}` | PRESERVE | — |
| BR-138 | Forget a saved swatch | menu "Forget swatch" | item is saved | B/colors_shelf.dart:235-256 | desk:`swatches` | PRESERVE | — |
| BR-139 | Swatch previews (solid or natively rendered gradient strip) | automatic | — | B/color_values.dart:22-37 | native visual sample `{kind:gradient}` | PRESERVE | — |

### Files shelf

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| BR-140 | Jump to place: Home / Desktop / Downloads / Pictures / Movies / Music (highlighted when current) | rail click | — | B/files_shelf.dart:32-57 | UI-local (`dart:io` listSync) | PRESERVE + MOVE | Brief: Files need not be a big always-on Finder |
| BR-141 | Back / Forward folder history | click ← / → in path bar | history non-empty (else disabled) | B/files_shelf.dart:193-202, 260-270 | UI-local | PRESERVE | — |
| BR-142 | Up to parent folder | click ↑ | not at root | B/files_shelf.dart:203, 272-275 | UI-local | PRESERVE | — |
| BR-143 | Breadcrumb navigation (click any ancestor crumb; path scrolls horizontally, anchored right) | click crumb | not the last crumb | B/files_shelf.dart:205-241 | UI-local | PRESERVE | — |
| BR-144 | Open a folder | double-click / Enter / menu "Open" | folder tile | B/files_shelf.dart:94-96 | UI-local | PRESERVE | — |
| BR-145 | Import a file to Media (then Media reveals it) | double-click / Enter / menu "Import to Media" | file tile, has('import') | B/files_shelf.dart:97-99 | command:`import{paths}` via `importPaths` | PRESERVE | — |
| BR-146 | Reveal in Finder / Open with default app (files only) / Copy path | right-click menu | has path | B/files_shelf.dart:124-157 | native:`reveal`, native:`openFile`, clipboard | PRESERVE | — |
| BR-147 | Listing shows folders first then only importable files; hidden entries skipped; image thumbnails from file | automatic | — | B/files_shelf.dart:281-325 | UI-local | PRESERVE | — |
| BR-148 | Facts in menu: Folder / format·MIME · size · parent folder | right-click | — | B/files_shelf.dart:107-121 | UI-local | PRESERVE | — |

### Adjacent (outside browser/ but controls Browser state)

| ID | Capability | Trigger | Condition | Code | Effect path | Proposed disposition | Reason |
|---|---|---|---|---|---|---|---|
| BR-150 | Browser tile size slider in Settings | Settings panel | — | `motolii/ui/lib/panels/panel_settings.dart:36-41` | desk:`browserTile` | PRESERVE + MERGE | Same value as BR-063–065 |

---

## UI-local state to preserve

Persisted in `deskWork` via `EditorSession.storeDesk` (user/workspace settings, survives restart, **not** in the document):

| Key | What | Written by |
|---|---|---|
| `browserView` | Grid(0)/List(1)/Thumbnails(2) — **one global value for all shelves** (shelf `defaultView` only applies while unset; Media's default 2 is lost once any view is chosen) | B/parts.dart:111 |
| `browserTile` | tile size px 72–240 (default 120) | B/frame_grid.dart:89, panel_settings.dart:41 |
| `browserRail` | rail width 0–200 (0 = collapsed; default 96) — shared by all shelves | BR:431-452 |
| `browserWheel` | colour wheel size | B/colors_shelf.dart:227 |
| `colorShape` | 'square' / 'triangle' | B/color_picker.dart:310 |
| `swatches` | saved colours / gradients `{stops, blend}` | B/colors_shelf.dart:255, 285, 310 |
| `tags` | user tags keyed `"<shelf>/<id>"` | B/filter_library.dart:54-74 |
| `collections` | collection 1–7 per `"<shelf>/<id>"` | B/filter_library.dart:81-91 |
| `collectionNames` | renamed collections `{"1": name}` | B/filter_library.dart:117-121 |
| `labels` | saved filters per shelf `{name, filter, rail, query}` | B/filter_library.dart:140-153 |
| `ranges` | user duration ranges per `"<shelf>/<group>"` | B/filter_library.dart:101-105 |
| `folds` | folded filter groups per shelf | B/filter_library.dart:130-134 |

In-memory only (lost on restart / panel rebuild), per panel instance:
- Search text (`search`, `queries[tab]`) — BR:67-70
- Rail category per shelf (`classifications`) — BR:66
- Selection and active (anchor) item per shelf (`selected`, `active`, `picked`) — BR:71-72, 87
- Filter state per shelf (`filters`: tag sets + collection) and Filter View shown (`filtersShown`) — BR:79-80
- Grid scroll offset (`scroll`) — BR:73
- Rail drag / wheel drag in-progress widths (`railDrag`, `wheelDrag`)
- Files: current folder, back/forward stacks, listing (`FilesShelf.folder/folderBack/folderForward`) — always starts at `$HOME` in production (`initialFolder` only set by tests)
- Fonts: `fontFacts` cache (fetched once per shelf instance)
- Colors: unbound scratch colour `picked`; picker draft/drag part/remembered hue
- Filter range-adder open state; collection rename field
- Session-level shared notifiers the Browser reads/writes: `textStyleTarget` (Fonts scope, shared with Inspector rich text), `eyedropper`, `importedAssets`, `browserTab`, `dragging` (`motolii/ui/lib/session/session_core.dart:69,78`)

## Suspected duplicates / merge candidates (flag only)

1. **Import**: Media "Import" button (BR-083) = File ▸ Import menu (`app/editor_window.dart:394-396`) = window file drop = Files double-click (BR-145). All end in `importPaths`. Merging changes nothing behaviourally.
2. **Palette from image**: Media menu "Extract palette" (BR-091) and Colors "From image" (BR-121) both call `ColorsShelf.savePalette`. Merge is behaviour-neutral (the Colors one accepts several files at once; Media one uses the asset's file).
3. **Tile size**: Browser zoom bar (BR-063–065) and Settings slider (BR-150) write the same `browserTile`. Neutral.
4. **Fonts vs Inspector text card**: Inspector shows the font name only as a button that calls `focusFont` → opens Fonts shelf (`panels/inspector/content_cards.dart:31-36`) and explicitly **hides** `text_justify`, `*.size` and colour rows (content_cards.dart:12-20). So Fonts shelf is the *only* UI for `setFont`, `styleText` size, scope and justify — merging into an Inspector text section would be a MOVE, not a duplicate removal; behaviour unchanged only if the `textStyleTarget` scope and preview/commit/cancel routing move with it.
5. **Colour wheel vs Inspector colour**: Inspector colour swatches and gradient stops only call `focusColor` (`panels/inspector/controls.dart`, `panels/gradient_inspector.dart:99-108`, `panels/composition_controls.dart`); the wheel is the *only* editor of colour values (`setColor`/`previewColor`). Contract l.16 explicitly keeps the wheel permanently in Browser ▸ Colors following the Inspector target. Merging would contradict that adopted decision → needs user ruling.
6. **Fill type / blend** (BR-132/133) vs gradient inspector (`panels/gradient_inspector.dart`, edits stops/angle via `setGradient`): same command, different fields; `setFillMode` (solid) is only reachable from Browser. Merging into the gradient inspector would not change behaviour if both fields keep `slot` routing.
7. **Eyedropper** (BR-128): toggle lives in Browser, pick happens in Stage (`panels/stage/touch.dart:217-229`) and calls `applyPalette` (not `setColor` on the target) — note: this means eyedropper applies to the selection, not strictly the wheel's target. Worth checking before merging with any Inspector picker.
8. **Solid swatch apply** (`applyPalette`, BR-136) vs wheel `setColor` on the target: two different write paths for "give this layer this colour" (selection-based vs slot-based). Merging would change behaviour (which slot receives it).
9. **Place asset**: Media double-click (BR-085) vs drag to Timeline (BR-087) — same command, drag adds position. Not duplicates.
10. **Create text**: Create ▸ Text (BR-071) vs Fonts double-click with no text layer (BR-115) — same `create{kind:text}`, Fonts variant adds `family`. Neutral to merge if family stays optional.
11. **Collections assignment**: four routes (drag to rail, digits 0–7, context menu rows, context "Remove from collection"). Keep all; they are cheap.
12. **Dormant tab strip / `browserTab` reveal** (BR-001, BR-013): production uses separate dock panels + `placePanel(…,'show')`. A new shell may re-introduce a tabbed Browser; if so, the `_revealTab` path becomes live again.

## Notes / oddities found while reading (not fixed)
- `FilesShelf.listingError` is set ("Cannot read this folder") but never displayed; the grid just shows "No matches" (B/files_shelf.dart:29, 316-324).
- Digit key `0` removes picked rows from collections (B/frame_keys.dart:40-47) — not mentioned in any tooltip.
- Media drag (when supported) replaces the generic BrowserDrag, so dragging a multi-selection of Media tiles onto a collection collects only the dragged asset, not the whole pick (B/tile.dart:255-258 vs B/rail_collections.dart:98-101).
- Context-menu "Apply" is always enabled even when the tile tooltip says "Apply unavailable" (BR:618-621).
- `browserView` is global, so Media's thumbnail default is overridden as soon as any shelf's view is changed.
