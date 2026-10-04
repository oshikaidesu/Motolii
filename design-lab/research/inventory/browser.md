> **Use of this file (owner, 2026-10-02): COUNTER-EXAMPLE ONLY.** The old Motolii is not a reference for the new UI or UX. This file is a list of what exists (a capability checklist) and of what NOT to import. Nothing here is a spec.

# Inventory: Browser seat (Motolii, worktree `/private/tmp/wt/tok`, branch dev/ui-inspect)

Source root: `/private/tmp/wt/tok/motolii/ui/lib` (paths below are relative to it). Read in full: `browser/**` (all 28 files), plus the shelves that live outside `browser/` but are the Browser's contents: `effects/shelf.dart`, `colors/shelf.dart`, `colors/cards.dart`, `colors/instrument.dart`, `fonts/shelf.dart`. Stories: `/private/tmp/wt/tok/motolii/ui/explorer/lib/stories/panels.dart` (browserStories l.69-77, catalog stories l.161-180) and `paper.dart`.

Structure facts that matter for design:
- In the shipped dock each shelf is its own panel: `workspace/seats.dart:41` builds `LiveBrowser(fixedTab: n)` for Create(0) Effects(1) Colors(2) Fonts(3) Media(4). With `fixedTab` set the 5-tab strip is not drawn (`browser/face.dart:69`).
- There is NO "Presets" shelf and NO separate "Files" shelf in this code. "Things" (`browser/things.dart`) is the descriptor catalogue (data model, not a widget) that feeds Create and Effects.
- Create + Effects + Colors + Fonts share `PanelShell` (chrome). Media does NOT use `PanelShell`; it has its own header/controls (`browser/media/*`).
- Story column: "Browser <Shelf>" = `panels.dart:73` (288x800), "Browser <Shelf> narrow" = `panels.dart:74` (240x600). Media-with-real-library = `panels.dart:70-71`. Colors real palette = `panels.dart:72`. Catalog stories = `panels.dart:161-180`. "none" = no story found. Menus/popovers have no story (stories are static captures; no story opens a menu).

---

## 1. Panel chrome / tabs (shared by Create, Effects, Colors, Fonts)

- **Browser host (one shelf per panel)** · `browser/browser.dart:20` `LiveBrowser`
  Builds the model for one shelf and wraps it in a `BrowserSeatScope`. Tab index 0-4 = Create, Effects, Colors, Fonts, Media (l.55). Media gets `MediaSeat`, the rest get shelf panels (face.dart:71-100).
  States: fixedTab (single panel) vs 5-tab (dormant in product). Inputs: scene (sample picture for effect faces), shelf data. Story: "Browser <Shelf>" for all five.
- **Seat behaviour wrapper (tile click/menu/dim)** · `browser/browser.dart:102` `_LiveSeat`, interface `browser/seat.dart:7`
  Wraps each tile: click = apply ("use"), right-click = picks it and opens the seat menu (l.121-135). Tile with no host binding is drawn at 48% opacity (l.122). Supplies host snapshot faces (l.138), font-dressing header (l.164), header kebab menu (l.188). Story: none for the menu; dimmed tiles appear in "Browser Create/Effects".
- **Tab strip (Leaf)** · `browser/parts.dart:66` `Leaf` + `_Tab` l.163 + `HfTabGlyph` l.203
  Row of tabs with glyph (Create=plus, Effects=glow, Colors=color, Fonts=text, Media=image; l.48-52) + name; selected tab raised background and brighter ink; crowded => only active tab keeps its word (compact: glyph only); optional `trailing` slot and `tabWrap`. Height = chromeRow. Only drawn when more than one tab. Story: none (product always passes one tab) [uncertain whether any test draws it].
- **Panel shell (responsive layout)** · `browser/panel_chrome.dart:143` `PanelShell`
  Places header, optional sub-header, and body in 3 morphologies: wide, narrow (width < columnMinWidth 260 shows a class chip), strip (height < 170: one-row horizontal tile strip). All four shelves pass `classStrip: true`, so class names sit in the header row (HeadMode.full if width >= 260 else compact; stacked when sharing a tab strip). Handles keyboard focus (click gives focus; Esc clears search first, then seat key). Story: "Browser <Shelf>" (wide) and "... narrow" show the two layouts; strip mode: none.
- **Panel header row** · `browser/panel_chrome.dart:39` `PanelHeader`
  Row with class strip (lead), tools slot, result count (mono, only mode full), magnifier key, kebab key. States: folded (magnifier) vs open (search field filling row with kebab and x); open while query non-empty or focused; docked/stacked variants hide title and icon (title/icon are only drawn when no `lead`, not docked, not stacked). Height: `Surface.namedHeader` (full) or `chromeRow`. Story: all shelf stories.
- **Header icon key** · `browser/panel_chrome.dart:23` `HeaderKey`
  22.5px tappable glyph (magnifier, kebab); `on` brightens glyph. Used for search + more. Story: shelf stories.
- **Glyph box** · `browser/panel_chrome.dart:14` `GlyphBox`
  Draws a theme glyph (`HG.*`) at a size/colour; used for shelf icons, kebab, cross, QuietFace, media fallbacks. Story: shelf stories.
- **Result count (header)** · `browser/panel_chrome.dart:125` and `:99`
  Right-aligned mono count of shown items (e.g. "1,234" formatting for >=1000 in Create/Effects/Fonts; plain in Colors). When search is open it appears as the field's trailing text (only while query active). Story: shelf stories.
- **Section heading with count pill** · `browser/shelf_sections.dart:16` `SwissHeading` (also `SectionLabel` `browser/panel_chrome.dart:226`)
  Semibold section name plus optional small rounded count pill (`count` param). Used by Create board sections and Colors sections (SectionLabel passes no count). Story: Create/Colors shelf stories.
- **Fonts "dressing" sub-header** · `browser/browser.dart:164` (key `fonts-header`)
  One muted line under the header on Fonts only: "Double-click a face to add a text layer" when no text layer is active, else "<layer name> · <text content>". Story: Browser Fonts (state depends on selection).
- **Empty-body message** · `browser/panel_chrome.dart:233` `emptyBody`
  Muted one-line message. Texts in code: `No mark matches "<q>".` / `No mark matches.` (Create), `No effect matches "<q>".` / `No effect matches.` (Effects), `No colour matches "<q>".` / `No colour matches.` / `No saved gradients.` (Colors), `No typeface matches "<q>".` / `No typeface matches.` (Fonts). Story: none (empty states not staged).
- **Grid unit helpers** · `browser/shelf_sections.dart:11-13,38` `kShelfUnit/Pad/Gap`, `shelfColumns`
  Layout constants (4px unit) and column maths; no visual. [uncertain whether used by current panels; defined for Create/Media housing]

## 2. Search and filters (classes)

- **Search field** · `browser/search.dart:85` `SearchField`
  Rounded raised field: magnifier glyph, hint text (hidden once typing), editable text, clear "x" when active, optional trailing widget (the count). States: idle border vs focused border (brighter). Hints: "Search create", "Search effects", "Search colors", "Search fonts". Story: not shown open in any story (folded by default).
- **Search behaviour (capability)** · `browser/search.dart:10` `SearchCapability`
  Tokenised AND match (case-insensitive). "/" or Cmd/Ctrl+F opens + focuses; Esc clears query, then leaves the field. Create and Effects use `ThingQuery` (`browser/things.dart:203`) which also parses `kind:`, `family:`, `tag:`, `cap:`, `source:` and numeric ranges like `duration:5-30`. Fonts: a query starting with "." reveals system-private faces (`fonts/shelf.dart:116`).
- **Search magnifier key (alternative face)** · `browser/search.dart:153` `SearchKeyFace`
  Small 19.5px magnifier key that opens into the field. [uncertain: not referenced by the four current panels; PanelHeader has its own folded search]
- **Class strip (horizontal categories)** · `browser/classify.dart:132` `ClassStrip`
  Scrollable one-row list of class names; chosen class has a raised pill; groups separated by space; edge fade shows more to scroll; `bare` = inside header row. States: selected / unselected, scroll-edge fades (before/after). Story: all shelf stories (the class names across the header).
- **Class row/pill** · `browser/classify.dart:107` `_Row`
  A single class label (selected = darker raised ground + bright ink). Story: shelf stories.
- **Class column (vertical categories)** · `browser/classify.dart:41` `ClassColumn`
  Scrolling left rail with the chosen class scrolled into view, trailing "+" glyph. [uncertain: unused by current panels because all pass `classStrip: true`; kept in PanelShell wide mode]
- **Class chip (narrow)** · `browser/classify.dart:217` `ClassChip`
  Small chip "<class>  x" shown in header when a class is chosen and the layout is narrow/strip (PanelShell l.190); click clears to All. Hidden when not filtering. Story: "Browser <Shelf> narrow" only if a class chosen (none staged).
- **Class groups per shelf (content)** · `browser/things.dart:272` `ThingViews.groups`, `browser/browser.dart` / `session.dart:266`, `colors/shelf.dart:137`
  Create/Effects: [All + top-level families from registry] then [Favorites, Recent, user collections (Orange..Gray), saved searches]. Create families: Text, Shapes, Paths, Tools, 3D, Helpers; Effects families present in data: Blur, Light, Color, Distort, Stylize, Noise (plus "Host" when host-only effects exist). Colors: [All, each palette class, Gradients] / [Wheel]. Fonts (live): [All, Sans, Serif, Display, Mono, Hand, JP, KR (only those present)] / [Used Here, Favorites, Installed]. Story: shelf stories.
- **Media: Sources row** · `browser/media/catalog_controls.dart:168`
  Labelled row "SOURCES" with chips: This project, Bundled, All, ★ Favorites (dim when none), Recent (dim when none), one chip per registered folder source (name + " ·off" when unavailable; dim when disabled/unavailable), "＋ Folder…", "＋ Import…" (only if host supports import). Chosen = raised chip, semibold. Story: "Browser catalog, real sources", "this project", "folders".
- **Media: Types row** · `browser/media/catalog_controls.dart:131,199`
  Chips: All, ▣ (image), ▶ (video), ♪ (audio), 3D (model), 360° (environment). Single-choice. Story: catalog stories.
- **Media: Folder breadcrumb row** · `browser/media/catalog_controls.dart:377` `_Folders`
  Row "FOLDER": "/" root chip, "/ part" crumbs (last = chosen), then sub-folder chips "<name> <count>" (dim). Only when one source is in view. Story: "Browser catalog, folders".
- **Media: search box** · `browser/media/catalog_controls.dart:203,342`
  Dark rounded box, placeholder "Search assets", plain editable text; queries the catalog owner on every change. Story: catalog stories (empty).
- **Media: chip / row label** · `browser/media/catalog_controls.dart:324` `_Chip`, `:310` `_Row`
  Chip with on / dim / off states; row label is 54px uppercase micro text. Story: catalog stories.
- **Media: failure line** · `browser/media/catalog_controls.dart:211`
  Text of `session.failure` (catalog error) under the controls. Story: none.
- **Media: list column header (sort)** · `browser/media/list.dart:7` `MediaListHeader`
  Columns NAME, TYPE, LENGTH, SIZE (shown width >= 300), DATE (width >= 420); sortable ones (name, type, size, date) show ↑/↓ for the active sort; LENGTH not sortable. Appears only in List view (animated size in/out, media_browser.dart:366). Story: "Browser catalog, list", "folders".
- **Media: explore controls bar** · `browser/media/explore/graph.dart:364` `ExploreBar`
  Chips Global / Local 1 / Local 2, spacer, "Type · Folder" toggle (tint + blobs). Shown only in Explore view. Story: "Explore 01-08".

## 3. Create shelf

- **Create panel** · `browser/create/shelf.dart:18` `CreatePanel`
  Searchable/classifiable shelf of marks over the descriptor catalogue (25 things in `browser/data/things/builtin/create.json`: Text; Rectangle, Rounded, Ellipse, Star, Polygon; Line, Arrow, Path, Blob; Pen, Pencil, Spray, Eraser; Cube, Sphere, Torus, Cylinder, Cone, Pyramid, Plane; Null, Camera, Light, Particles, Stage). States: wide, narrow, strip, empty. Story: "Browser Create", "Browser Create narrow".
- **Create board** · `browser/create/board.dart:16` `CreateTiles`
  Sections (heading per section = family section: Primitives, Tools, 3D, Helpers) each a wrapped grid of tiles; final "Recent" section (up to one row, max 8, hidden while searching). Columns = floor by 44px min; tiles share row width. Story: "Browser Create".
- **Create tile** · `browser/create/board.dart:67` `_Tile`
  Mark (16px) over name (micro, ellipsised) in a fixed-height box. States: rest (raised ground), hover (hover ground). Semantics: button with name. Behaviour from seat: click uses; right-click menu; dimmed 48% when no host binding (arrow, blob, pen, pencil, spray, eraser, pyramid, light have no mapping in `browser/session.dart:21-41`). Story: "Browser Create".
- **Create strip tile** · `browser/create/shelf.dart:81` `_Tile` (used in `_strip` l.71)
  Mark only (no caption) up to 40px with hover ground; used in strip mode (height < 170). Story: none.
- **Mark icons (26)** · `browser/create/shelf.dart:113` `MarkPainter` (enum `Mk` l.16)
  Hand-painted icons: text, rect, rounded, ellipse, star, polygon, line, arrow, path, blob, pen, pencil, spray, eraser, nul, camera, light, particles, stage, cube, sphere, torus, cylinder, cone, pyramid, plane. Special visuals: shaded 3D bodies (cube, sphere, torus, cylinder, cone, pyramid, plane), dashed stage frame. Colour per thing from descriptor. Story: "Browser Create".
- **Thing face (dispatcher)** · `browser/create/faces.dart:16` `ThingFace`
  Draws seat's own picture if given, else by face type: `mark` (MarkPainter), `fx` (FxPainter, optional hue shift), `curve` (CurvePainter), else QuietFace. Story: Create/Effects stories.
- **Quiet face (placeholder tile)** · `browser/create/faces.dart:44` `QuietFace`
  Dark rounded square with a faint pie glyph; fallback for things with no picture / while a snapshot loads. Story: Effects stories (host effects with no snapshot).
- **Curve face** · `browser/create/faces.dart:54` `CurvePainter`
  Stroked curve for functions sine, spring, decay, noise, orbit, bounce, coloured by hue. [uncertain: present in registry faces but no builtin thing uses it; only the stress set may] Story: none.

## 4. Effects shelf

- **Effects panel** · `effects/shelf.dart:191` `EffectsPanel`
  Grid of effect tiles over descriptors (15 in `effects.json`: Blur, Bokeh, Glow, Vignette, Color Shift, Duotone, Invert, Chromatic, Displace, Pixelate, Halftone, Threshold, Sharpen, Noise, Film Grain) plus any host effects (family "Host", `browser/session.dart:99-111`). States: wide (3 columns max), narrow (2 or 1), strip (horizontal 1.5:1 tiles), empty. Story: "Browser Effects", "Browser Effects narrow".
- **Effect tile** · `effects/shelf.dart:253`
  Face (aspect 1:0.84) over name caption (label size, ellipsised); caption hidden if tile < 60px wide; click applies to selected layer(s), right-click menu. Seat dims it 48% if host has no binding. Story: "Browser Effects".
- **Effect face (live preview of the effect)** · `effects/shelf.dart:79` `FxPainter` + sample scene `EffectScene` l.26
  Each tile shows a dusk-over-hills sample picture with that effect applied (blur, bokeh, glow, vignette, hue shift, duotone, invert, chromatic split, displace, pixelate, halftone, threshold, sharpen, noise, film grain). Hue-tinted variants through `hueFilter` (l.69). Special visual: before/after-style sample render. Story: "Browser Effects".
- **Host effect snapshot** · `browser/browser.dart:138` `face()` via `BrowserSession.picture` (`session.dart:132`)
  Native-rendered picture of the effect; while loading: FxPainter fallback if reference has a face, else QuietFace; over QuietFace so an empty snapshot still reads as a tile. States: loading, loaded, no snapshot. Story: "Browser Effects" (depends on host).
- **Effects strip tile** · `effects/shelf.dart:229`
  Horizontal strip of faces at 1.5:1, no captions. Story: none.
- **Effect card (not in the Browser)** · `effects/card.dart:18` `NewEffectCard`
  This is the Inspector's applied-effect card (grip, power toggle, kebab menu, params); listed so it is not mistaken for a browser tile. Out of scope for this inventory.

## 5. Colors shelf

- **Colors panel** · `colors/shelf.dart:150` `ColorsPanel`
  Wheel instrument on top, then palette sections, then Gradients. Classes: All / palette classes / Gradients, and Wheel (wheel only). States: wide, narrow (wheel + compact small swatches), strip (horizontal swatch chips), filtering (wheel collapses to mini instrument), wheel-only, gradients-only, empty. Story: "Browser Colors", "... narrow", "Browser Colors, a real palette".
- **Live colour instrument** · `colors/instrument.dart:20` `LiveColorInstrument`
  Interactive colour editor for the colour target: hue ring + square (or triangle) picker, value bar, alpha bar (dimmed 40% when target has no alpha), hex text field (Enter / blur applies, bad hex reverts), eyedropper glyph (armed state brightens; hint "Click the Stage to pick · Esc cancels"), target title (max 2 lines), numeric readouts R/G/B and H/S/V (+A %). Disabled (50% opacity) when the host cannot set colour. Special visuals: hue ring, SV square, colour triangle (desk setting `colorShape`), checkerboard behind alpha. Story: "Browser Colors" (shows reference or live depending on session).
- **Reference instrument (static)** · `colors/shelf.dart:358` `_Instrument`
  Non-interactive picture of wheel + hex "#E8508F" + eyedropper glyph + two bars, used when no live editor is given. Story: possible in fixtures only [uncertain].
- **Mini instrument (while filtering)** · `colors/shelf.dart:409` `_MiniInstrument`
  40px wheel, a colour square and hex of the current colour. Story: none staged.
- **Colour wheel painter** · `colors/shelf.dart:505` `WheelPainter` (triangle maths `colors/hsv_triangle.dart`)
  Ring + square or triangle with handles at the current HSV. Story: Browser Colors.
- **Colour bar (value/alpha)** · `colors/shelf.dart:444` `ColorBar`
  Vertical gradient bar with a round handle; alpha variant over a checker. Story: Browser Colors.
- **Swatch card** · `colors/cards.dart:12` `SwatchCards`
  Large rounded colour block (>= 2 columns, ~60px cells) with its hex set inside bottom-left in a contrasting ink; hairline border only where the colour matches the ground; capped at 300 per section. Tap applies; right-click opens menu. Story: "Browser Colors".
- **Small swatch** · `colors/shelf.dart:618` `_Swatches`
  20px rounded squares, up to 600 and then "+N" text; used in the narrow layout. Story: "Browser Colors narrow".
- **Section label per palette class** · `colors/shelf.dart:272` (`SectionLabel`)
  Heading per class: fixture classes Used Here, Saved, Starter, Seasonal, Nature, Material, Neon, Pastel, Monochrome; live data: Used Here (palette entries in use), Starter, Saved. Story: "Browser Colors".
- **Gradient swatch** · `colors/shelf.dart:654` `_Gradients`
  58.5x11 rounded bars filled with the saved gradient (native sample render when available, Flutter LinearGradient placeholder otherwise); tap applies to the colour slot; right-click Apply/Forget. Empty text "No saved gradients.". Fixture gradients: Candy, Sky, Sunset, Ocean, Meadow, Dusk. Story: "Browser Colors, a real palette" (if gradients exist).
- **Colour strip chip** · `colors/shelf.dart:330` `_strip`
  Horizontal 38px colour chips; a "+" mark when no tap handler. Story: none.

## 6. Fonts shelf

- **Fonts panel** · `fonts/shelf.dart:61` `FontsPanel`
  One row per family, list view. Rows 44px (wide) / 34px (narrow). Classes: All/Sans/Serif/Display/Mono/Hand/JP/KR, Used Here, Favorites, Installed. System-private faces (name starts ".") hidden unless the search starts with ".". Story: "Browser Fonts", "Browser Fonts narrow".
- **Font row** · `fonts/shelf.dart:215` `_Row`
  Sample glyph box (38 or 30px), family name typeset in its own face, meta line (only when row >= 40: "<class>  ·  <N> styles  ·  <script/axes/Mono/Color facts>"), favourite star. States: chosen (hover ground + left accent bar), unchosen, narrow < 110px wide (sample only). Click = dress the active text layer; double-click = make new text layer with that face (`browser/browser.dart:90-91`). Shown input: sample "Aa" / あ / 漢 / 가 / ع / Я (`browser/session.dart:278`), name, class, style count, facts. Special visual: font specimen. Story: "Browser Fonts".
- **Font specimen** · `fonts/shelf.dart:180` `_sample`; `browser/visual_sample.dart:93` `NativeVisualSample`
  Placeholder: the sample glyph typeset in the face. When a Text layer is active and native samples are on, the host draws that layer's own text in the face; queued and throttled (`_SampleShelf`). States: placeholder, drawn. Story: "Browser Fonts" (placeholder unless text layer active).
- **Favourite star** · `fonts/shelf.dart:329` `_Star`
  15px star, outline off / filled on; only on rows >= 40px; toggles collection 1 for the font. Story: "Browser Fonts".
- **Fonts strip tile** · `fonts/shelf.dart:142`
  Raised square with 24px sample; width min(72, h*1.3). Story: none.

## 7. Media shelf (catalog browser)

- **Media seat** · `browser/media/seat.dart:9` `MediaSeat`
  Owns the catalog session; first look starts on "This project" when the work already holds assets, else on everything. Story: "Browser Media, a real library" (+ wide).
- **Catalog media composition** · `browser/media/catalog_controls.dart:21` `CatalogMedia`
  Stacks MediaBrowser with the Sources / Types / Folder / Search controls and a full-panel "Drop to import" overlay. Story: "Browser catalog, real sources" family.
- **Media browser frame** · `browser/media/media_browser.dart:26` `MediaBrowser`
  Header, controls, body, optional preview (flex 5 : 4). States: list / thumbnail / explore, selection (single, multi, range), preview open/closed. Keyboard: Esc (close preview, then clear pick), Cmd/Ctrl+A, Space (toggle preview), F (favourite), Enter (place), Delete/Backspace (remove from library), Home/End, arrows. Story: catalog stories.
- **View switch header** · `browser/media/media_browser.dart:383` `_Header`
  Tabs List | Thumbnail | Explore (Explore only offered when a layout is supplied; always supplied by CatalogMedia), count at right, thumbnail size slider (only in Thumbnail). Selected tab raised ground + semibold. Story: catalog stories.
- **Thumbnail size slider** · `browser/media/media_browser.dart:426` `_SizeSlider`
  74px track with round handle; maps to column min width 44..140 px. Story: "Browser catalog, big faces" (a bigger column start).
- **Fluid board (the three projections)** · `browser/media/fluid.dart:29` `FluidBoard`
  Same faces shown as List rows (46x26 face + facts), Thumbnail masonry (kinds interleaved, shortest column next, name under each) or Explore map. Moving List<->Thumbnail animates faces 200 ms ease-out; dragging the size slider is 1:1. Selected face scrolls to centre on preview toggle. Story: "Browser catalog, fluid filmstrip" (`explorer/lib/paper/filmstrip.dart`), "live tiles" (`paper/live_tiles.dart`).
- **Media face (tile)** · `browser/media/fluid.dart:604` `_Face`
  Rounded picture with overlays: top-left type badge (▶ m:ss, ♪ m:ss, "3D ↻", "360° ↔"), top-left ★ if favourite, top-right dot if used in the work, top-right yellow "!" if file missing, hover = 1px light outline, selected = double ring (light + dark), optional coloured tint outline (Explore overlay), dim 28% (Explore, not near focus). Video: faint 2px scrub strip at bottom that grows to 4px with a white fill on press and swaps in the frame at that time. Drag carries asset to the Timeline (ghost 85% opacity, original 40%). Press selects; double-click places; right-click menu. Story: catalog stories.
- **Media face picture** · `browser/media/library.dart:15` `materialFace`
  By kind: Audio = waveform on dark ground (min/max peak bars in a lifted wave hue, centre line; headphones glyph when no peaks), Video = still/thumbnail, 3D = live model, 2D = picture over a checkerboard (transparent pixels visible), fallback = raised ground with image glyph. Special visuals: waveform, checker, 3D. Story: catalog stories ("preview sound wide" shows the waveform).
- **3D model face** · `browser/media/model_face.dart:14` `ModelFace`
  Real glTF/GLB drawn by flutter_scene, auto-framed from bounds, camera yaw/pitch; `turnable` (drag to turn, grab cursor; used in the preview), `hoverYaw` (pointer sweep on tile) [uncertain: hoverYaw not wired in the files read]. States: loading (dark ground + fallback), failed (fallback), loaded. Story: "Browser catalog, preview model" (+ paper "Browser 3D faces 140/68" prototypes).
- **List row label** · `browser/media/fluid.dart:451` `_label` (list)
  Name, type word, length (m:ss), size (>=300 wide), date YYYY/MM/DD (>=420 wide). Selected name brighter. Story: "Browser catalog, list".
- **Thumbnail / explore caption** · `browser/media/fluid.dart:464`
  Name under the face (left aligned in Thumbnail; centred under chosen face in Explore, hidden during motion). Story: catalog stories.
- **Selection preview panel** · `browser/media/preview.dart:25` `MediaPreview`
  Opens in place under the board. Side-by-side when >= 520 wide (face 3 : info 220px), else face over a 118px info band. Story: "Browser catalog, preview clip / model / sound wide".
- **Preview info block** · `browser/media/preview.dart:47` `_Info`
  Name (13.5px, close "x" key `preview-close`), chips ("Image · JPEG", size, length, W x H), facts rows (Audio "48 kHz · stereo", Source, Path, Date), links "Place in project" (underlined, only if can place) and "Reveal in Finder" (only if can). Story: preview stories.
- **Preview live face** · `browser/media/preview.dart:103` `_LiveFace`
  Instrument per kind: image = zoom/pan (1x-8x), environment = panorama dragged round (0.5x-3x), model = turnable, clip = frame scrub, sound = waveform with a moving position line. Story: preview stories.
- **Scrubber bar** · `browser/media/preview.dart:201` `_Scrubber`, painter `_Bar` l.230, `_Position` l.191
  Thin track with round handle and a "m:ss / m:ss" label; disabled (dim) when owner cannot supply frames. Story: "Browser catalog, preview clip", "preview sound wide".
- **Drop-to-import hint** · `browser/media/catalog_controls.dart:215` (key `media-drop-hint`)
  Full-panel veil with light outline and the text "Drop to import"; fades in (120 ms) while OS files are dragged over the window; takes no pointer. Story: none (needs a drag).
- **Explore graph canvas** · `browser/media/fluid.dart:313` (InteractiveViewer, scale .2-3.5) with `GraphEdges` `explore/overlay.dart:62`, `GraphBlobs` l.109, layout `explore/graph.dart:199` `exploreLayout`
  Pan/zoom map of faces; lines between near assets (colours by relation: similar grey-blue, folder blue, source amber, type violet, project green, duplicate red); focus + neighbours stay bright, others dim; Global (all) or Local 1/2 hops around the chosen asset; overlay adds a coloured outline by media kind and soft folder blobs. Camera flies 260 ms to a new fit. Story: "Explore 01-08" (global wide/narrow/overlay/selected image/selected model/local 1/local 2/local 2 overlay).
- **Hub pill (graph node)** · `browser/media/explore/overlay.dart:89` `HubPill`
  Small pill in a relation colour for Source/Folder/Project/Type nodes. [uncertain: `exploreLayout` currently returns `hubs: const []`, so none are drawn]
- **Explore limit message** · `browser/media/media_browser.dart:361`
  Text: "Explore cannot map this many assets yet (N). Narrow the result ..." when items > 300. Story: none.
- **Explore spokes** · `browser/media/fluid.dart:724` `_Spokes`
  Lines from chosen face to a list of nearest. [uncertain: `links` is always empty in the current layout]
- **Media empty state** · `browser/media/media_browser.dart:354`
  Text "Nothing matches." in muted label. Story: none.

## 8. Context menus and popovers inside the browser

All use the shared `showHfMenu` (`controls/menu.dart:14`: rows, optional info row, dividers, disabled rows, shortcuts). No story opens any of them.

- **Shelf tile menu** · `browser/browser.dart:188` `_LiveSeat.more`
  Opens on tile right-click or the header kebab key. Rows: tile name (info, divider), Apply (only if the tile has an action, divider), "Add to Favorites / Orange / Yellow / Green / Blue / Purple / Gray" (7 collection lines, `browser/user_state.dart:75`), "Remove from <collection>" (if in one), Effects: "Reload effects" (if host supports), Colors: "Save current color" (disabled when no colour target), "Palette from image…", "Square wheel" / "Triangle wheel" toggle.
- **Colour swatch menu** · `browser/browser.dart:70`
  "Apply" and, for saved swatches, "Forget swatch".
- **Gradient menu** · `browser/browser.dart:80`
  "Apply", "Forget swatch".
- **Media asset menu** · `browser/media/media_browser.dart:297`, rows built in `browser/media/catalog_controls.dart:262` and `session/media_actions.dart:28`
  Title row (asset name) then, per owner: bundled = "Place"; work asset = Place, "Replace selected layer" (disabled with nothing selected), Reveal in Finder / Show in Explorer / Show in file manager, "Open with default app", "Extract palette" (images), "Copy path", "Locate file…" / "Relink to another file…", "Remove from library" (disabled while a layer uses it); catalog asset = Place, "Add to Favorites"/"Remove from Favorites", Reveal, Open, Extract palette, Copy path.

## 9. Presets / Things / user collections

- **Presets shelf**: not found. No code in `browser/**` draws a presets panel. See unclassified list.
- **Thing descriptor catalogue (data)** · `browser/things.dart:8` `Thing`, `Registry`, `ThingQuery`, `ThingViews`, `Catalog`; loaded from `browser/data/things/*.json` by `browser/catalog_io.dart`.
  Not visible itself; defines tile names, families, tags, faces, and the closed face types (mark, fx, curve). Story: n/a.
- **User views (Favorites / Recent / colour collections)** · `browser/user_state.dart:7` `LiveBrowserUser`
  Seven named collections (Favorites, Orange, Yellow, Green, Blue, Purple, Gray; renameable via `collectionNames`), Recent (12), saved searches. Surface in the class strip and the tile menu. Story: shelf stories (if state present).

## 10. Empty / error / loading states (collected)

- Empty bodies: Create, Effects, Colors, Fonts (texts in section 1); Media "Nothing matches."; Colors gradients "No saved gradients."; Media Explore too-many message.
- Error: Media catalog `failure` text; missing file badge "!" on face and "Locate file…" menu; "·off" on unavailable source chip.
- Loading: effect snapshot (FxPainter/QuietFace), font specimen placeholder glyph, 3D model dark ground with fallback, native visual sample placeholders, gradient placeholder.
- Dimmed: tiles without a host binding (48%), dim chips (Favorites/Recent empty, disabled source), Explore non-neighbours (28%), alpha bar without alpha (40%), colour editor disabled (50%).

---

## (a) Count of entries

Entries by section: 1 Panel chrome 12, 2 Search and filters 16, 3 Create 8, 4 Effects 6, 5 Colors 11, 6 Fonts 5, 7 Media 21, 8 Menus 4, 9 Presets/Things 3 (one of them is the "Presets shelf: not found" line). Section 10 is a summary of states, not counted.
Total: **86** entries (incl. 1 not-found line, 1 out-of-scope pointer in Effects).

## (b) Things I could not classify

- **Presets**: the brief lists "Presets/Things"; no presets shelf or preset widget exists in `browser/**` or the four shelves. Only the descriptor catalogue ("Things") exists as data.
- **Files shelf / Browser tabs "Files"**: mentioned in the older capability inventory (`docs/stage5/ui-rebaseline/inventory/browser.md`) but not present in this worktree's code (tabs are Create, Effects, Colors, Fonts, Media only).
- `SearchKeyFace` (search.dart:153), `ClassColumn` (classify.dart:41), `shelfColumns` (shelf_sections.dart:38), `HubPill`, `_Spokes`, `CurvePainter` faces: defined but no live use found in the current panels; kept as entries marked [uncertain].
- `ModelFace.hoverYaw`: parameter exists; no caller seen in files read.
- `DockedPanel` / `PanelSeat` (parts.dart:34,55): invisible inherited widgets that switch header variants; classed as layout flags, not entries.
- `visual_sample.dart` `_SampleShelf`: a throttled request queue, not a widget (its widget is `NativeVisualSample`).
- `effects/store.dart`, `effects/params.dart`, `effects/card.dart`, `colors/color_field.dart`: Inspector-side or non-visual helpers; not Browser UI.
- Paper prototype stories (`paper.dart`: "Browser library ...", "Browser broken Media A2..L", "Browser paper Media 1-6", "Browser 3D faces", "Density ..."): freehand explorations in `explorer/lib/paper/*`, not product widgets; not inventoried.
- Whether hover states exist on shelf tiles beyond Create tiles (hover ground) and Media faces (outline): Effects tiles and Colors swatches have no hover code in the files read.
- Whether the in-panel 5-tab strip is ever shown in the product: code says no (fixedTab always passed in `workspace/seats.dart:41`); not verified at runtime.
