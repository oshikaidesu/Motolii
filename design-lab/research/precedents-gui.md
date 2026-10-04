# Precedents: graphic instrument vs number field (P1-P3)

Scope: only P1, P2, P3 from the brief. Inputs read: research/critique-gui-dose-1.md (critic: fields as prominent as pad/dial/handles; pad, dial, scale box have different sizes and top edges; Shot 6) and research/intents/inspector.md (owner decision 2026-10-02: spatial GUI primary, numeric fields secondary readout; B3 pad was "No by default" until overruled).

Tags: [seen] = I read the page body and it states it. [snippet] = only a search-result snippet or a fetch-tool summary of a page (not the raw page), so wording is second-hand. [inferred] = from my own memory of the product or my reading, NOT verified here.

## Honest limits (read first)
- NO pixel measurements were obtained for any tool. No screenshot of any shipped tool was opened. Every "number size/weight" below is therefore [inferred] or absent. Do not treat as measured.
- Fetches that returned only navigation menu / no body: Blender manual `editors/3dview/sidebar.html` and `.../display/gizmo.html` (both returned nav only), Blender manual GitHub raw paths (404), Ableton `live-device-view` (404), Adobe `viewing-animating-properties` (403), Apple Motion `motna5015809` (nav only, no control details).
- Not opened at all: Cavalry pad / angle dial / pivot widget pages (only mentioned in passing), Logic Pro, Ableton XY/Macro screenshots, Figma on-canvas padding handle screenshots, AE Composition-panel handles page.

## P1. Hierarchy of number vs graphic instrument

Pattern 1A: Graphic is the control, number is a type-in on the same row (Apple Motion Inspector).
- Opened: https://support.apple.com/guide/motion/properties-inspector-controls-motna5015809/mac (nav only), plus search snippet from the same guide.
- [snippet] Controls are "sliders, dials, pop-up menus, checkboxes". Position is value sliders X/Y/Z; a disclosure triangle on Position/Rotation reveals dials for all three axes. Rotation is a dial for a 1-D degree value.
- [snippet] Adjusting is also possible via on-screen canvas controls, so the dial is the panel's graphic, the number is the slider's own field.
- Number size/weight/placement: not seen. When shown: dial hidden until disclosure, so graphic is on demand, numbers always [snippet]. Editable: [inferred] yes (type into the slider field).

Pattern 1B: Number is always visible and editable; graphic is a separate, smaller key on the same row/widget (Figma constraints widget + dropdowns; Figma alignment box).
- Opened: https://help.figma.com/hc/en-us/articles/360039957734-Apply-constraints-to-define-how-layers-resize (fetch summary), https://help.figma.com/hc/en-us/articles/360040451373-Explore-auto-layout-properties (fetch summary).
- [snippet] Constraints: an interactive diagram with clickable lines, AND dropdown menus doing the same job; both always present. Auto layout: a visual alignment box; padding/gap are numeric fields; both in the panel.
- [snippet] X/Y/W/H/Rotation are fields only in the panel (rotation -180..180); canvas shows a blue dimension label, corner handles, rotation affordance only when hovering just outside the bounds. i.e. the instrument for spatial values lives on the Stage, not in the panel.
- [inferred] The diagram is small (about the height of two field rows); the fields are the same weight as other fields, the diagram is not larger than them. Not measured.

Pattern 1C: Readout appears with the knob, tied to hover/info rather than fixed in the layout (Ableton device/rack knobs).
- Opened: https://www.ableton.com/en/live-manual/12/instrument-drum-and-effect-racks/ (fetch summary); search snippet on Info View.
- [snippet] Macro Controls are a bank of knobs, default 8, up to 16, with name and unit taken from the mapped parameter; unit changes to 0-127 when several params with different units map to one macro.
- [snippet] Info View (toggle `?`) names what is hovered. [inferred] The numeric value shows under/over the knob on hover or drag and a double-click resets (the inspector.md card C7/C10 cites Ableton for that; I did not verify it from the manual here).
- Number size: not seen. Editable: [inferred] yes (type after click) but not verified.

Pattern 1D: Number and graphic coexist but number editing is multi-field (Cavalry Attribute Editor).
- Opened: https://cavalry.studio/docs/user-interface/menus/window-menu/attribute-editor/control-rows/control-rows-interaction/
- [snippet] A Control Row holds several Controls ([x,y], [r,g,b]); hover-click-type, scrub left/right, Alt applies to all fields; Tab to next. Page names "pad, angle dial, pivot" as graphic controls but gives no detail on them.

P1 finding: in none of the opened sources could I confirm a case where the number is demoted below the graphic. In Figma the panel's fields are the full-weight element and the graphic (constraints box) is the smaller helper; in Motion the dial is hidden by default. The owner's "numbers secondary" is therefore not backed by a precedent I opened; the closest are Ableton knobs (the graphic is the control; the value appears with it) [inferred].

## P2. Several instruments in a narrow column

Pattern 2A: One widget per row, shared row grid (label | field(s) | unit/key); instruments are small keys in that grid (AE, Figma, Cavalry Control Rows).
- [snippet] Cavalry: "attribute = Control Row + Controls" (cavalry attribute-editor page). Figma panel: X/Y/W/H/Rotation in a regular field grid [snippet]. inspector.md B1 states the same rule for our own Transform ("every row has the same columns").
- Size/alignment numbers: not seen. [inferred] widths come from a 2-column or 3-column field grid; the graphic does not break the grid.

Pattern 2B: Instrument revealed inside the row on demand (Motion).
- [snippet] Dials appear under the Position/Rotation row after a disclosure triangle; one instrument visible at a time, in the same column width as the sliders.

Pattern 2C: One bank of identical knobs on a fixed grid (Ableton Macro Controls).
- [snippet] 8 knobs by default, up to 16, uniform bank; the selector buttons change the count. [inferred] each knob has the same diameter and label slot; I did not measure.

Pattern 2D (inferred only): One canvas holds several handles (3D viewport gizmo / Stage). Not opened (Blender gizmo page returned nav only). Marked [inferred].

P2 finding: precedents I could open share one cell size (bank/row grid); none I opened mix a pad, a dial, a scale box and an anchor of different sizes. That mismatch is exactly the critic's Shot 6 finding.

## P3. Fields and instruments for the same value: both at once, or one on demand?

Pattern 3A: Both always, different places (Figma, Motion).
- [snippet] Figma: panel has fields; canvas has handles/labels; both always available and edit the same value. Motion: canvas on-screen controls and Inspector are both available, "simultaneous" [snippet].
- For our case this corresponds to Stage + Inspector, with the Inspector fields only. Our column contains both, so it is not a match.

Pattern 3B: Both in the panel, same value, graphic and dropdown (Figma constraints diagram + dropdowns).
- [snippet] The diagram and the two dropdowns are always both visible and both write the same constraint.

Pattern 3C: Graphic on demand (Motion dials behind a disclosure triangle).
- [snippet] The number slider is always shown; the dial per axis appears only when the triangle is open.

Pattern 3D (inferred only): Blender N-panel vs gizmo: N-panel fields always (when the sidebar is open) and gizmo shown by header toggle. NOT verified: both Blender pages returned nav only. [inferred]

P3 finding: the opened sources show "both at once" (3A/3B) and "graphic on demand" (3C). I found no precedent where the number is on demand and the graphic is always present, which is the shape of the owner's decision.

## Candidate arrangements for Widgetbook (layout only, 280 px column)

P1 candidates (same value set: position X/Y/Z, rotation, scale, anchor):
- P1-a "Field-per-instrument": instrument on top, its field strip directly below at the same width (current state).
- P1-b "Readout inside": instrument fills the cell; the value is printed as small text on/inside the instrument frame, editable on click, no separate strip.
- P1-c "Readout on demand": instrument only; value shown while hovering/dragging in a pill at the instrument edge; a single "Values" toggle in the section header shows the field strips for all.
- P1-d "Motion style": numeric rows always; instrument unfolds under its row via a disclosure triangle.

P2 candidates:
- P2-a "One cell size": every instrument gets a square cell of the same side, 2 per row (or 3 across), shared top edge and baseline for labels.
- P2-b "One canvas": pad, dial, scale box, anchor drawn as handles inside a single square canvas, one set of fields below it.
- P2-c "Bank row": a single row of equal small instruments (dial, scale, anchor) under one larger pad, widths from one 4 or 8 px grid.
- P2-d "Row grid": instruments become small inline keys of identical height inside the field-row grid (Figma-like), no big pad.

P3 candidates:
- P3-a "Both always" (instrument + fields stacked), as now.
- P3-b "Instrument default, fields on toggle" (section header switch).
- P3-c "Fields default, instrument on disclosure" (Motion).
- P3-d "Fields in a separate Relations/Values tab" while the Inspector shows only instruments plus readout text.

## Next step (for the owner or a follow-up search)
Open real screenshots for: Blender N-panel + gizmo, Ableton Macro/XY with value readout, Figma constraints widget, AE Transform group. Without them, no pixel values (cell size, font size, gaps) can be quoted.
