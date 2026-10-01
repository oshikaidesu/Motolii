> **Use of this file (owner, 2026-10-02): COUNTER-EXAMPLE ONLY.** The old Motolii is not a reference for the new UI or UX. This file is a list of what exists (a capability checklist) and of what NOT to import. Nothing here is a spec.

# Inspector seat and shared controls: widget inventory

Source: `/private/tmp/wt/tok/motolii/ui/lib` (branch dev/ui-inspect), read-only. Paths below are relative to that `lib`.
Explorer stories live in `../explorer/lib/stories/panels.dart` (group "Inspector"). Every Inspector story builds one widget, `RightSeat(c)` (panels.dart:`_inspector`), so a part is "shown" only inside a whole-panel story and only when its data is in the scene. Story names: S1 "Transform (shape)", S2 "Transform (text)", S3 "Effects, long parameter list", S4 "Inspector 500x700 (surface oracle)", S5 "Inspector effects 500x700", S6 "Inspector narrow" (260 wide), S7 "Nothing chosen". Also `density.dart` "Density 1-4" (generic Material/EditorButton specimens, not these widgets) and `workspace.dart` "Workspace NN%" (whole window). No story targets a Camera layer, a Layout group or a Stage layer: those scenes are not selected by any `_choose` index that I can verify [uncertain]. Controls under `controls/` have no story of their own.

---

## 1. Seat and subject switching

**Inspector seat (RightSeat)** · `inspector/inspector_seat.dart:15`
- Chooses what the Inspector shows from `InspectorSession.subject`: Empty text, Camera instrument, or a Layer column (Transform, optional Stage rows, optional Layout, effect cards). Scrolls vertically when it has more than Transform (`seat-scroll:<id>` key resets scroll on layer change). Also reacts to `editingFocus` by opening Dock panels (Ease for keyframes, Depth for a camera, Blend for blendMode).
- States: empty, camera, layer-only (Transform alone, no scroll), layer with sections (scroll), frozen/locked layer (passed down).
- Story: S1-S7 (all).

**Empty message** · `inspector/inspector_seat.dart:141`
- One centred muted line (`why` string from session). States: nothing picked / no layers.
- Story: S7.

**Stage section label + margin rows** · `inspector/inspector_seat.dart:94-101` (label "Stage") over `LayerRowsSheet`
- For a Stage layer: a small "Stage" caption then the generic param rows whose id starts `stage.`. Edits via `SessionEffectStore.layerRows`.
- Story: none verified [uncertain].

**Inspector subject model** · `inspector/session.dart:11` (InspectorEmpty / InspectorCamera / InspectorLayer) and `InspectorSession:38`. Non-visual; owns the stores per layer and effect ops (move, step, flip, expand, remove).

---

## 2. Inspector header and tabs

There are no tabs in this Inspector. The nearest things:

**Layer title header (Transform header)** · `inspector/transform/instrument.dart:94` (`_header`)
- Colour bar in the active mode colour, layer name (or "N layers" for multi-selection, key `tf-title`), a "Locked" word when locked, and the Animate toggle (diamond + word).
- States: single / multiple selection; locked; animating on (diamond filled yellow, word yellow) / off (muted). Shown when `showHeader` (the seat passes true).
- Edits: Animate (auto-key) toggle.
- Story: S1-S6.

**Animate toggle** · `inspector/transform/instrument.dart:102` (key `tf-animate`) and camera's `inspector/camera/instrument.dart:84` (key `camera-animate`, text only, shown only when the host supports `animate`).

**ParamStore body header (generic Inspector body)** · `inspector/inspector.dart:72` (`InspectorBody`)
- Title, a "Frozen" outlined badge, row count (mono, key `insp-count`), and a "Filter" search field; list below. Empty result text "No parameter matches". Narrow rule: under 188 px width no two-up cells.
- States: frozen, filtering (flat list, no heroes/fold), narrow, empty.
- Story: none found; `InspectorBody` is not referenced by RightSeat [uncertain whether used elsewhere].

---

## 3. Transform instrument

Main class `TransformInstrument` · `inspector/transform/instrument.dart:15`; host card `NewTransform` · `inspector/transform/card.dart:11` (below 230 px it fixes height 345 px and scrolls inside).
States: wide (gizmo left of Space/Parent column) vs narrow (<172 px: stacked, 0 decimals, no unit text, no end marks); modes move/scale/rotate/anchor; projection 2D / 2.5D / 3D (adds Z fields); locked (`canEdit` false greys chips); scroll when too short.

**Transform gizmo pad** · `inspector/transform/gizmo.dart:17`
- A fixed-size dark pad (default 170x132, instrument gives width minus 96, height 129 or 99 narrow) drawing one abstract body (rectangle at the layer's scale/rotation, clamped), a pivot dot, a dotted background that slides as Position changes, and a corner readout in mono (X/Y px, scale %, rotation degrees; empty in anchor mode).
- Mode behaviours: Move (mint) shows X and Y arrow handles on a ring (a Z handle outside 2D), faint corner/edge scale dots and a faint pink rotate ring; body drag moves, axis drag constrains. Scale (blue): four corner handles and four edge handles (white), "LINKED" label when linked. Rotate (pink): ring with tip handle, optional "X AXIS"/"Y AXIS" label. Anchor (violet): nine dots, hover enlarges, click sets.
- Gestures: drag; Shift = fine (x0.1); Esc aborts a drag; keys 1-4 switch mode, Tab cycles; pointer cancel aborts; disabled when layer locked (greys to N.g33).
- Edits: position px, scale ratio, rotation degrees (unwrapped, turns kept), position.z, anchor 0..1.
- Story: S1/S2/S4/S6 (visible as part of the seat).

**Mode strip** · `inspector/transform/instrument.dart:117`
- Four small square buttons with RoleGlyph icons (move, scale, rotate, anchor); selected one filled with its mode colour (mint, blue, pink, violet). Vertical column when wide, horizontal row when narrow. Keys `mode-move` etc.

**Role glyphs** · `inspector/transform/instrument.dart:368` (RoleGlyph): move cross-arrows, scale box with corner arrow, rotate arc, anchor 3x3 dots. Plus `_OpacityGlyph` (:404, half-filled circle), `_DepthGlyph` (:419, two offset squares), `_DiamondP` (:435, key diamond), `_LinkP` (:448, chain; slashed when off).

**Rotation axis chips (Z / X / Y)** · `inspector/transform/instrument.dart:67`
- Three tiny chips at the gizmo's bottom right, only in rotate mode outside 2D. Selected is pink. Keys `rot-axis-0..2`.

**Space chips (2D / 2.5D / 3D)** · `inspector/transform/instrument.dart:141` `_spaceCol` (wide, labelled "SPACE") and `_spaceRow` (narrow)
- Single-select chips for projection; selected is yellow with dark ink, dimmed accent when not editable. Applies to the selection. Keys `space-2D`...

**Parent stepper** · `inspector/transform/instrument.dart:148`
- "PARENT" caption, a raised bar with prev arrow, name (or "None"), next arrow; wraps around the other layers. Disabled when locked. Keys `parent-prev/next/name`.

**Value rows (Position, Scale, Rotation, Anchor, Opacity, Depth)** · `inspector/transform/instrument.dart:216` (`_rows`), line wrapper `_line` :284
- Each row: mode glyph button (tappable to switch gizmo mode; active gets tinted bg), value wells, a fixed end column, optional relation badge, key diamond + reset arrow (hidden narrow).
- Position: X, Y (+Z outside 2D), px. Scale: X, Y or single "S" when linked and even, percent; Z outside 2D. Rotation: one angle, or X/Y/Z outside 2D, degrees. Opacity: 0..1 shown as percent. Depth (only outside 2D): value + "Depth →" route pill (key `route-depth`, opens Depth desk).
- Three-across rows stack the third value under the pair when narrow.
- Story: S1/S2/S4/S6.

**Scale link chip** · `inspector/transform/instrument.dart:265` (key `link-scale`)
- Chain icon in a bordered box; on = blue tint and joined chain, off = slashed. Keeps ratio when dragging either axis.

**Anchor nine-point pad and name** · `inspector/transform/instrument.dart:330` (`_anchorLine`)
- A 3x3 dot grid; current dot is larger and violet; hovering previews the pivot on the Stage (anchorPreview); click sets. Beside it a name: Top left, Top, ... Bottom right, or "Custom". Locked greys it.

**Mark cluster (key diamond + reset)** · `inspector/transform/instrument.dart:276` (`_marks`)
- Diamond: outline in grey when never animated, tone-coloured outline when animated, filled when keyed on this frame. Tap toggles key. "↺" appears when value differs from default and resets.

**Relation badge** · `inspector/transform/instrument.dart:307`
- Red pill "◉ <relation name>" if the row is driven, or "◉ N" if it drives N rows; tap focuses the relation in the Relations panel. Key `relation-<id>`. Same pill in ParamCell (see 8).

**World block (Blend, flags)** · `inspector/transform/instrument.dart:186` (`_world`)
- "WORLD" caption; a "Blend" row (raised bar with the blend mode name and "›", opens the Blend desk, key `world-blend`); flag chips with dot: "Ghost" (only if ghostable), "Clip to below", "Environment" (Image layers only). Hidden for Camera layers. Chip: on = yellow tint+border, off = raised; disabled = grey text, semantics label gives the meaning.
- Story: S1/S2 (Blend and Clip visible; Ghost/Environment depend on layer kind [uncertain]).

---

## 4. Camera

**Camera instrument** · `inspector/camera/instrument.dart:14`; session wrapper `LiveCamera` · `inspector/camera/card.dart:153` (title = the layer's own name)
- Column: header (colour bar, name, "Locked", Animate word if supported, "Depth →" outlined pill key `route-depth`), the Camera face, then four labelled groups each with a colour tick, small caps title, optional note, key diamond and reset: TARGET (X, Y, Z + a Target choice, note "set by the layer" when locked), ORBIT (Pitch, Yaw), FRAMING (Distance, Zoom, note "one magnification, two tools"; caption "Frames the target at xN.NN", key `magnification`), ROLL (one angle).
- States: narrow (<172 px, 0 decimals, short tags P/Y/D/Z, stacked Z), locked camera (frozen, face at 0.45 alpha), Target layer set (Center X/Y/Z shown but dimmed 0.5 and non-interactive).
- Ranges: Distance and Zoom clamp 0.01..100; angles unbounded degrees; Center and Target Z px.
- Story: none verified (needs a Camera layer selected) [uncertain].

**Camera face** · `inspector/camera/face.dart:55`
- A dark pad (default 286x176, instrument 132 or 111 high) drawing the view ray: target dot with crosshair (mint), the eye on an oblique sphere (violet camera glyph, hollow when behind the target), blue ray with a bar handle for Distance, pink roll ring with 12 ticks and a handle, mono readout "pitch .. yaw .. xD roll ..", "TARGET LAYER" label when the target is a layer.
- Gestures: drag target (disabled when a Target layer decides it), drag eye (orbit), drag distance bar (exponential), drag roll ring; Shift = fine; frozen disables.

**Camera group label** · `inspector/camera/instrument.dart:78` (`_label`): see key/reset cluster; keys `key-target`, `key-orbit`, `key-framing`, `key-roll`, `reset-<name>`.

---

## 5. Layout

**Layout instrument** · `inspector/layout/instrument.dart:14`; card `NewLayout` · `inspector/layout/card.dart:9` (embedded in the seat column, no own scroll).
- Header: colour bar, name ("Child in layout" for a child), "Locked", and a switch: "Grid" (group) or "Ignore layout" (child).
- Group body: layout diagram; Columns/Rows (Col/Row), Gap when on; Padding X/Y; W and H size lines; transition Duration + Easing choice; collapsible ADVANCED fold. Child body: W/H size lines, GRID AREA (Col/Row start, Cols/Rows span) when declared, ADVANCED.
- Gating: Grid off dims everything but Columns and Rows; the size number counts only under Fixed (otherwise dimmed 0.45 and ignores pointer).
- Story: none verified [uncertain].

**Grid / Ignore switch** · `inspector/layout/instrument.dart:60`: word + 22.5x13 track with thumb; on = tone colour. Keys `grid-switch`, `ignore-switch`.

**Layout diagram** · `inspector/layout/diagram.dart:92`
- Dark pad (default 286x190; 147 high, 117 narrow) showing the offered space (dashed), a container (pink outline), padding (dashed blue), nine violet alignment dots, child boxes (mint), and handles: column bar, row bar, gap dot, padding corner and edge squares, size dots (hollow when Hug/Fill, solid when Fixed). Readout in mono "C x R gap pad Justify / Align" or "Grid off".
- Gestures: drag the handles; drag inside sets Justify and Align together; Shift fine; Esc aborts; focusable; frozen disables.
- Values: columns 1..12, rows 0..12 (0 = "auto"), gap px 0..400, padding px, justify (Start, End, Center, Between, Around, Evenly), align (Stretch, Start, End, Center).

**Sizing chips (Hug / Fill / Fixed)** · `inspector/layout/instrument.dart:130` (`_sizeLine`), glyphs `SizingGlyph` :161
- A letter (W/H) and three 19.5 px chips with arrow glyphs; selected is pink. Keys `size-w-0..2`, `size-h-0..2`. Narrow stacks the number under the chips.

**Advanced fold (layout)** · `inspector/layout/instrument.dart:109` (key `layout-advanced`): "▸/▾ ADVANCED n" then ParamCell rows (Min width, Max width with 0 = "none", ...).

---

## 6. Effects / parameters

**Effect card** · `effects/card.dart:18` (`NewEffectCard`)
- A hairline-topped band (raised, work-row high) with list glyph, effect name (muted when disabled), power glyph (accent when on), kebab glyph; the body is the generic param sheet. Whole header is the drag grip (grab cursor) in the seat's reorderable list, and a tap folds the card.
- States: on/bypassed (power glyph, name colour), folded, held (frozen/locked disables power and menu; frozen shows "Frozen — effects are baked. Unfreeze to edit."), reordering.
- Keys `effect-card:`, `effect-head:`, `effect-toggle:`, `effect-menu:`, `effect-grip:`.
- Story: S3, S5 (the "effects" scene).

**Effect menu (kebab menu)** · `effects/card.dart:33`: Apply earlier, Apply later, "Throw every number within its reach" (randomise), "Back to where the numbers rest", "Expand copies into layers" (placement effects only), "Remove effect". Items disabled per capability and position. Rendered by `showHfMenu` (section 10).

**Effect parameters sheet** · `effects/params.dart:9` (`NewEffectParams`) over `ParamSheet` · `inspector/inspector.dart:149`
- The generic params for one effect: heroes, sections, ADVANCED fold, with no header and no scroll of its own. Narrow rule at 188 px. Store: `SessionEffectStore` · `effects/store.dart:11` (edits reach every selected unlocked layer with the same row; drag relative, typed absolute).
- Story: S3, S5.

**Layer rows sheet** · `effects/params.dart:28` (`LayerRowsSheet`): same sheet over a layer's own rows under a prefix (Stage).

**Effect shelf panel (Browser "Effects" tab)** · `effects/shelf.dart:191` (`EffectsPanel`) with `EffectScene` :26 (a procedural dusk picture drawn once) and `FxPainter` :79
- PanelShell with class strip, search "Search effects", count, and three bodies: wide grid (up to 3 columns), narrow grid (1-2), strip (horizontal row of faces, aspect 1.5). Tiles are effect faces with a caption when >= 60 px wide. Empty text "No effect matches ..." Lives in the Browser seat but the code is in my area.
- Story: "Browser Effects" / "Browser Effects narrow" (panels.dart:73-75, tab index 1) [uncertain that LiveBrowser hosts this exact class].

---

## 7. Colour (Browser "Colors" tab, in `colors/`)

**Colors panel** · `colors/shelf.dart:150` (`ColorsPanel`): PanelShell "Colors", search "Search colors", class strip with a "Wheel" and "Gradients" class, count; wide / narrow / strip layouts. Empty texts "No colour matches ...", "No saved gradients."
- Story: "Browser Colors" and "Browser Colors, a real palette" (panels.dart:72,73-75).

**Live colour instrument** · `colors/instrument.dart:20` (`LiveColorInstrument`)
- Hue ring with a square (or triangle, per desk setting `colorShape`) via `WheelPainter` (shelf.dart:505); two vertical `ColorBar`s (Value, Alpha with checker; alpha bar dimmed 0.4 when the target has no alpha); readout: target title (2 lines), editable hex text, eyedropper glyph (lit when armed), R/G/B and H/S/V lines, A% line; hint "Click the Stage to pick · Esc cancels" while armed. Whole control at 0.5 opacity when not enabled.
- Gestures: drag wheel/bars (preview, commit on end, Esc cancels), type hex (3 or 6 digits, `parseHex`), eyedropper toggle.
- Edits: colour as RGBA, HSV.

**Hue ring + SV square / triangle** · `colors/shelf.dart:505` (WheelPainter), geometry helpers `colors/hsv_triangle.dart` (barycentric pick). Without a colour it paints the reference #E8508F.
**Colour bar** · `colors/shelf.dart:444` (`ColorBar`; kind 0 value black to colour, kind 1 alpha over checker; round handle).
**Static reference instrument** · `colors/shelf.dart:358` (`_Instrument`) shows wheel + fixed "#E8508F" text when no live editor is given; `_MiniInstrument` :409: 40 px wheel, swatch and hex while filtering.
**Swatch cards** · `colors/cards.dart:12` (`SwatchCards`): large rounded squares, hex inside bottom-left (ink flips on luminance), edge only when a colour would vanish into the ground; tap uses, right-click menu; capped at 300. Key `hf-color:<name>`.
**Swatch grid** · `colors/shelf.dart:618` (`_Swatches`): small squares (20 px in narrow), "+N" overflow past 600.
**Gradient tiles** · `colors/shelf.dart:654` (`_Gradients`): 58.5 x 11 bars, from stops (r,g,b[,a]); native visual sample if the host supports it; tap / right-click.
**Colour strip** · `colors/shelf.dart` (`_strip`): horizontal narrow swatches with a "+" add mark when no swatch handler.
**Hex parsing** · `colors/color_field.dart` (`parseHex`). Non-visual.

---

## 8. Rows and sections (generic declared-param layout)

**Section label** · `inspector/inspector.dart:129` (PSection): a small coloured tick (tone dealt per group) + UPPERCASE label, key `tone-<label>`.
**ADVANCED fold row** · `inspector/inspector.dart:130` (PFold, key `advanced-fold`): tick, "▸/▾", "ADVANCED", count (mono).
**Cells row (PCells)** · `inspector/inspector.dart:136`: one or two cells side by side (two when both are scalar, bounded, integer or toggle and the width is >= 188).
**Param cell (ParamCell)** · `inspector/inspector.dart:179`
- Label line + control, in two layouts: stacked (label over control) or inline work row (label left, control right; label:value flex 4:5, a lone cell takes the whole value width). Hero cells use a stronger label (g86, w600) and taller well.
- Label line parts: key diamond (shown when animated or keyable; faint when not animated; filled when keyed now), label (ellipsis), off-default dot (key `mod-<id>`), relation pill (red, "◉ name" or "◉ N"), "Link" pill (filled in tone when linked), route accessory pill "<Specialist> →", reset "↺" (hidden when frozen).
- Action chips below: "Reroll" auto-added for a seed; declared `actions` rendered as Action chips.
- States: frozen (tones dimmed, no reset/link/key), hero, modified, animated, keyed, linked, relation, narrow.
- Kind to control map (rows.dart:`kindOf`, :8): scalar/bounded/integer -> Value toy; toggle -> Toggle; choice -> Choice; vec2/vec3 -> Values; pair -> Pair; text -> Text; reference -> Reference; route (colour, font, blend, ease, or a `route` with a non-number) -> Route; anything else -> raw JSON text.
- Heroes: declared heroes, else the first four non-advanced rows (`heroIds`, inspector.dart:16).
- Tones: `inspector/tones.dart:20`; a 6-colour palette (yellow, mint, blue, pink, violet, orange) dealt per section starting at a hash of the thing's id; frozen dims it (`dimTone`).
- Story: S3/S5 (effects), S1/S2 (some rows) [uncertain which kinds appear].

**Row character / Slot** · `inspector/rows.dart:171` (`Character`: angle, seed, count, opacity, position, scale), `inspector/slot.dart:6` (`Slot`): unit (° for angle, % for opacity/percent rows, row `unit`), whole numbers for counts/seeds/ints, `zeroWord` ("auto", "none"), mixed ("—") when selected targets disagree, quantise (whole degrees; tenths with Shift).

---

## 9. Value controls (Inspector "Toys")

**Value well (ValueToy)** · `inspector/value_controls.dart:41`
- A scrubbable number: 18 px (hero 22) raised well with a 1 px tone rule down its left edge, optional axis tag (X, Y, Z, S, Pitch...), the number, and a unit rider (px, %, °, s). A tight declared range (span <= 20 x default, 1 minimum) draws a soft fill + underline as a bar; open values have no bar. Mixed shows "—". Hover shows "‹ ›" when there is no unit. Number auto-drops decimals, then scales down, never cuts a digit; never shows a non-zero value as 0.
- Interactions: drag horizontally scrubs (direction reversed, Shift fine, per-row `speed`), click enters exact text input, double-click resets to default, arrow keys nudge, Enter edit, Delete/Backspace reset, Esc cancels a drag or an edit, wheel sideways or wheel with button held steps (one undo per run), trackpad two-finger sideways pan scrubs, right-click opens the key menu.
- States: rest, hover (raised -> hover bg), focus (1 px ink border), dragging (ink number, hover bg), editing, bad input (pink border + "Number required"), frozen (grey, no input), mixed, tinted (hero 7%, active 9% tone wash), narrow (no unit/decimals).
- Story: S1-S6 (everywhere).

**Values (vec2/vec3)** · `inspector/value_controls.dart:286`: 2 or 3 Value wells, tags X/Y/Z, linked peers (scale ratio).
**Pair** · `inspector/value_controls.dart:307`: two separate declared rows shown as X, Y wells, linkable.
**Toggle** · `inspector/value_controls.dart:327` (key `toy-<id>`): a raised bar with a 22.5x13.5 pill switch and the word "On"/"Off"; hero + on gets a tone wash; frozen greys.
**Choice** · `inspector/value_controls.dart:357`: <= 4 options = a segmented track (selected segment fills in the tone, others text; out-of-range value highlights none); > 4 options = stepper with ‹ › arrows and the name (wraps). Keys `choice-<id>-<i>`.
**Text** · `inspector/value_controls.dart:403`: 22.5 px box with an editable single line (yellow border when focused); applies on blur or Enter, Esc restores; raw JSON variant in mono for unrecognised rows (reverts if it does not parse); read-only when frozen.
**Reference** · `inspector/value_controls.dart:468`: a raised bar with a colour square (hollow outline = none), the target name or "None", "⌄"; a tap cycles through the declared candidates (no menu).
**Route** · `inspector/value_controls.dart:506`: bar with a colour dot (for colour rows), the current value in mono, and a tone pill "<Colors|Fonts|Blend|Ease|name>  →" that opens the specialist desk.
**Action chip** · `inspector/value_controls.dart:535`: rounded pill button in the tone (e.g. "Reroll").
**Value scrub rates** · `SlotScrub` extension `inspector/value_controls.dart:23` (rate/step). Non-visual.

---

## 10. Key / animate menu

**Key menu** · `inspector/key_menu.dart:10` (`keyMenu`): right-click on any Inspector value (`store.menu`); line "Key this frame" or "Remove key" (disabled when the store cannot key or is frozen), then store extras after a divider. Uses `showHfMenu`.
**Transform value menu** · `inspector/transform/store.dart:225`: adds "Relation…", "Show relation" (if in a relation), "Remove relation"; relate/unrelate disabled without capability or when frozen. "Relation…" opens the Relations panel in the Dock.
**Layout / Camera / Effect value menu** · `inspector/layout/store.dart:70`, `inspector/camera/card.dart:117`, `effects/store.dart:130`: key lines only.
**Key diamond** (every row; see 3 and 8): tap toggles key at the playhead; group-level diamonds (Camera groups) key several properties (`toggleKeys`).

---

## 11. Menus, sheets, dialogs, tooltips (`controls/`)

**Hf menu** · `controls/menu.dart:12` (`showHfMenu`)
- A popup list: dark raised box, 22 px rows, an optional "✓" column for the current value, right-aligned shortcut text, dividers, non-selectable "info" lines (first one bold), disabled rows grey. Up/Down walk lines, Enter takes, Esc / outside press closes; opens upward if no room; scrolls if taller than the window; min width 180; hovered line lit.
- Story: none.

**Hf dialog (question)** · `controls/question_dialog.dart:15` (`showHfDialog`)
- Modal over the window with a 55% shade, 285 px box, title, body, answers; destructive answers (red text) on the left, others right, last is primary (Enter); Esc or outside press returns null.
- Story: none.

**Hf popover (sheet)** · `controls/sheet.dart:148` (`showHfPopover`)
- A compact housing anchored under the asking control (right edge aligned), title bar with a close cross, builder body, shadow; no backdrop; Esc closes, Enter runs the primary. Used by the Export, Composition and Interface sheets (`app/sheets.dart`).
**Hf action button** · `controls/sheet.dart:16`: primary (filled mode colour, hover lighter), secondary (raised key), destructive (red text), quiet when no `onTap`, `chosen` state; 18 px visual, >= 24 hit.
**Hf choice (segmented)** · `controls/sheet.dart:70`: segments in a dark track; chosen is lighter; per-option enabled; hover state.
**Hf form row** · `controls/sheet.dart:121`: label column (default 64 px) + child. **Hf fact** `controls/sheet.dart:137`: read-only mono value, no box.

**Tooltip** · `theme/editor_theme.dart:201` (`EditorTooltip`) with sheet `controls/leaves/floating.dart:13` (`EditorTooltipSheet`): near-white (0xE6FFFFFF) rounded pill, black label text, fades in; touch delay 1500 ms.
**Editor menu sheet** · `controls/leaves/floating.dart:44`: the overlay sheet (menu colour, edge, anchor placement below or above), arrow keys walk rows, Esc and outside tap close.
**Editor menu row** · `controls/leaves/floating.dart:187`: 20 px row; lit (select colour) when hovered or focused; disabled greyed.
**Editor menu anchor** · `controls/leaves/floating.dart:257`: a builder widget that opens the sheet; handle `isOpen/open/close`. **Choice row** `:325` closes the menu on press.
**Text-field context menu** · `controls/leaves/field.dart:81`: Cut / Copy / Paste / Select all in an `EditorMenuSheet` (selection handles are off).

---

## 12. Shared controls: leaves (`controls/leaves/`)

**Editor press** · `controls/leaves/press.dart:15`: hover/press/focus wash surface (hover 4% white, press = theme hover, focus wash on keyboard focus only), click cursor, Enter/Space, tap/double/long/secondary callbacks.
**Editor icon button** · `controls/leaves/press.dart:163`: icon square (>= 20 px), hover/press tint, disabled ink, optional tooltip, `isSelected` semantics.
**Editor text button** · `controls/leaves/press.dart:220`: accent-ink text, optional bg/border/radius, hover 8% and pressed/focus 10% wash of the foreground, disabled colours.
**Editor button** · `theme/editor_theme.dart:218`: panel-coloured text button, `selected` = accent fill with dark ink, optional tooltip. (Used in Density story "Apply/Reset".)
**Editor slider** · `controls/leaves/track.dart:15`: flat 2 px track (muted before thumb, line after), 5 px ink thumb with shadow (deeper while pressed), optional `divisions` (ticks, snap, value label bubble in accent while dragging). Disabled state. Used by no Inspector widget I found [uncertain]. Story: none.
**Editor text field** · `controls/leaves/field.dart:16`: unframed text input (caret accent, selection colour), hint, prefix, multi-line/expands, read-only, enabled; own context menu.
**Editor choice** · `controls/leaves/choice.dart:12`: a bordered box with the current label and a drop arrow; opens an anchored menu of choice rows. Disabled state. Story: none.

---

## 13. Shared controls: panel parts (`controls/panel/`)

**Editor numeric field (number well)** · `controls/panel/numeric.dart:18`
- 20 px well, right-aligned number, unit rider, a 3 px family-colour rule on the left edge, optional track behind (`fill`) in four styles `TrackStyle` {fill, level, steps, ruler} with a default-tick; mixed value; `zeroWord`; decimals.
- Interactions: drag scrubs; moving the pointer vertically (above/below 32 px steps) changes the rung ladder x10 / x1 / x0.1 / x0.01 with a floating "x10 x1 x0.1 x0.01" pill while off x1; trackpad pan; wheel with the button held; Shift = x10; double-click or Enter types (field frame turns accent; error "Number required" in the tooltip and error colour); Up/Down nudges; Esc cancels. Tooltip "label · drag to adjust · double-click or Enter to type".
- States: rest, dragging (hover bg, or flooded with the tint when no track), editing, error, disabled, mixed, rung pill. Cancels on window blur is off for this control.
- Story: none (the Inspector uses ValueToy, not this). Used in Stage chrome and sheets.
**Rung pill** · `controls/panel/numeric_paint.dart:8`. **Track painter** · `:69` (`_TrackPainter`, styles per above).
**Drag session mixin** · `controls/panel/drag.dart:14` + `EditorPreviewQueue` :82: drag → preview → commit once; only the latest preview in flight; window blur or app pause cancels. Non-visual.
**Field frame** · `controls/panel/fields.dart:13`: the one input box (20 px, border: line at rest, accent focused, error colour on refusal).
**Editor bar** · `controls/panel/frames.dart:10`: 22 px panel bar; the row slides sideways when too narrow instead of overflowing.
**Editor percent field** · `controls/panel/scale.dart:7`: 64 px wide numeric field in whole percent, label "Scale" by default, cancel restores the start value. Used in the Stage chrome and the Interface sheet. Story: "Stage chrome" (panels.dart:184) [uncertain].
**Checker painter** · `controls/panel/scale.dart:59`: transparency grid, 8 px squares in two greys. Used by Stage chrome.
**Editor switch** · `controls/panel/toggles.dart:65`: 22x12 square-cornered track (accent when on) + glyph that says what it does, tooltip label; `compact` = glyph only (lit when on); disabled ink. Used in Ease desk and Stage chrome.
**Picked** · `controls/panel/toggles.dart:12`: rebuilds one item only when its selection state flips (non-visual helper).

---

## 14. Theme pieces that define the controls' look (`theme/`)

- `editor_theme.dart` (`EditorTheme` :44): colour roles for the standalone controls: app, panel, raised, hover, line, border, ink, muted, accent (0xFFBC53), animate, tab, tabInk, menu, menuEdge, select, selectInk, disabledInk, error, family colours (spatial, amount, time, count, seed, angle), washes (hover, focus), inkDisabled, ticks, tooltip, selection, caret; font Inter; menu padding/min width; `EditorInk` (camera line, checker greys); hover lift 8%, pressed 10%.
- `live_palette.dart`: `liveEditorTheme` re-maps those roles onto the app's neutral ramp (the one the Inspector widgets actually draw with).
- `neutral.dart` (`N`): grey ramp g00..g100 named by lightness (g07 wells, g10 ground, g13 raised, g15 hover, g20 rules/selected, g56 muted, g82 secondary text, g95 primary text), plus shades and glazes (clear, shade40/55/90, glaze, veil, inkSoft).
- `identity.dart` (`H`, `Fam`): semantic hues (scatter pink, stagger blue, along mint, face yellow, follow orange, attach violet), operational colours (relation red 0xFF4D3D, warn, play, record, mode, toggleOn, playhead, guide) and the `H.s`/`H.m` text builders; Inter / Menlo.
- `metrics.dart` (`Surface`, `Dn`, `UiScale`): UI Scale 50-200%; tokens: topBar 32, namedHeader 28, chromeRow 24, workRow 20, control 18, controlHero 22, menuRow 22, hit 24 (floor 18), mark 5, glyph 10, hair/focusStroke snapped, labelRow 11, cellGap 3, labelGap 2, inlineGap 0, sectionGap 6, panelInset 9, radii 3 and 4.5; text roles nameSize 9, labelSize 10, microSize 9.5, numericSize 11 (Inter / Menlo, leading 1.0).
- `glyphs.dart` (`HG`, `HgPainter`): hand-drawn 24-unit glyph set (list, power, kebab, color, composite, cross, pie, ...). `material_icons.dart` (`Glyph`): a copied subset of Material icon codepoints.

---

## Summary

(a) Count of entries: about 112 widget/part entries (approximate; bundled sub-parts counted as one) across sections 1-14 (see the tally below).
Tally: seat/subject 4; header/tabs 4; transform 14 (instrument, gizmo, mode strip, role glyphs, rot-axis chips, space chips, parent stepper, value rows, link chip, anchor pad, marks, relation badge, world block, animate); camera 4; layout 5; effects 8; colour 12; rows/sections 8; value controls 11; key menu 5; menus/sheets/dialogs/tooltips 14; leaves 7; panel parts 10; theme 6. Dart classes counted together where they are one visible part.

(b) Things I could not classify or verify:
- No story selects a Camera layer, a Layout group/child or a Stage layer; I could not confirm which layers the scenes `night-sky.rrd`, `effects.js` index 0/1/2 contain, so any "Story" mark beyond "the seat" is [uncertain].
- `InspectorBody` and `ParamSheet`-level search/filter header (inspector.dart:72) is not referenced by `RightSeat`; I did not search the rest of the app for its caller.
- `EditorSlider`, `EditorChoice`, `EditorIconButton`, `EditorTextButton`, `EditorPress`, `HfFormRow`, `HfFact`, `Picked` have no caller in the Inspector; some callers exist in Stage chrome, app sheets and the Ease desk (`app/sheets.dart`, `stage/chrome.dart`, `desks/ease/face.dart`) which belong to other areas.
- `Effects` / `Colors` shelf classes (`EffectsPanel`, `ColorsPanel`) sit in my directories but render in the Browser seat; I did not verify that `LiveBrowser` instantiates exactly those classes (the `_browser` story helper is in panels.dart:34).
- `HG.preset`, `HG.search`, and most of the glyph enum are used elsewhere; only list/power/kebab/color/composite/cross/pie were traced to this area.
- Blend desk, Ease desk, Depth desk and Relations panel opened from here are out of scope (routes only).
