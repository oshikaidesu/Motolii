# Inspector: intent cards (owner ticks the ones to keep)

Scope: intent only. The old Motolii is a counter-example; nothing of its look or implementation is a spec. Old paths are under `/private/tmp/wt/tok/motolii/ui/lib/` (`inspector/` = I, `controls/panel/` = C). Precedent URLs are at the end. Marks: [inferred] = my reading, not stated in code or docs. "Old: WELL / POORLY / MIXED" is my critical verdict on the old app's behaviour.

Card index (tick list): A Selection kinds (A1-A7) / B Transform (B1-B8) / C Value editing (C1-C10) / D Keys (D1-D4) / E Linking, bounds, mixed (E1-E4) / F Effects and groups (F1-F5) / G Toggles and parent (G1-G3) / H Panel behaviour (H1-H3) / I Hand-over to other tools (I1-I3) / J Do-not-repeat list.

---

## A. What the Inspector shows

### A1 One subject at a time, chosen by selection
- **Intent**: The panel is always about the thing I just picked; I never ask "what is this showing?".
- **Contract**: Selecting a layer replaces the content with that layer's properties; scroll resets to top on a different layer; the panel title/name states the subject.
- **Came from**: I/inspector_seat.dart:15 (scroll key resets per layer), I/session.dart:11 (Empty / Camera / Layer subjects). AE, Figma, Blender all follow selection. Old: WELL.
- **Candidate?** Yes. Basic, nobody would object; owner decides only the ordering of sections.

### A2 Shape / generic layer
- **Intent**: A visible object offers Transform first, then its own look (fill, stroke, effects).
- **Contract**: Transform group is always first and always present; layer-specific groups follow; effects come last as a stack.
- **Came from**: I/inspector_seat.dart:15 (Transform, optional Stage rows, optional Layout, effect cards). AE order (Transform then effects). Old: WELL on order [inferred].
- **Candidate?** Yes.

### A3 Text layer
- **Intent**: Text is edited as text (content, font, alignment, fill/stroke) with the same Transform on top.
- **Contract**: Font name opens a font chooser; alignment is a choice; fill/stroke open the colour tool; all of it respects a locked layer.
- **Came from**: docs/stage5/inspector.md (Text card, font entry, Alignment, Fill/Stroke); I/value_controls.dart:403 (TextToy: apply on Enter/blur, Esc restores). Old: MIXED (the intent is right; the font chooser only picks a family name, no preview).
- **Candidate?** Yes.

### A4 Camera
- **Intent**: A camera is edited as a camera (where it looks, from which side, how far, how rolled), not as a flat layer.
- **Contract**: Groups are Target (x,y,z or a target layer), Orbit (pitch, yaw), Framing (distance, zoom), Roll; when a target layer drives the target, those fields are visibly disabled and say why.
- **Came from**: I/camera/instrument.dart (groups TARGET/ORBIT/FRAMING/ROLL; "set by the layer" note); I/camera/face.dart:55 (direct-manipulation face). Blender N-panel View/Camera; AE camera Point of Interest/Position. Old: MIXED (grouping good; "one magnification, two tools" Distance+Zoom is confusing [inferred]).
- **Candidate?** Yes, if the new tool has cameras at all.

### A5 Group / layout container and its children
- **Intent**: A group that arranges its children shows arrangement controls (columns, rows, gap, padding, size mode); a child shows its own size and grid placement, or "ignore layout".
- **Contract**: Layout off dims all but its on/off; size number only counts when size mode is Fixed; child can opt out of layout.
- **Came from**: I/layout/instrument.dart:14,60,130 (Grid/Ignore switch, Hug/Fill/Fixed). Figma auto-layout panel (precedent). Old: WELL on gating; the diagram drawn as a mini-pad is a look, not carried.
- **Candidate?** Maybe: depends on whether the new tool has auto-layout. Ask the owner.

### A6 Nothing selected
- **Intent**: When nothing is picked, the panel says so in one plain line and offers nothing editable.
- **Contract**: One short muted sentence stating why (nothing picked / no layers); no controls, no stale values from the last selection.
- **Came from**: I/inspector_seat.dart:141. Old: WELL. [inferred] Figma instead shows page/canvas settings with nothing selected, which is a different, useful intent (document-level properties); owner may want that.
- **Candidate?** Yes (simple one-liner); optionally "show composition settings instead".

### A7 Several selected
- **Intent**: Selecting several layers lets me edit what they share in one go, and tells me honestly where they differ.
- **Contract**: Title says "N layers"; a value that differs shows a neutral "mixed" mark, not one layer's number; a scrub moves each by the same amount; a typed value sets all to that absolute value.
- **Came from**: I/transform/instrument.dart:94 ("N layers"), I/slot.dart (mixed "—"), I/value_controls.dart:117 / effects/store.dart ("drag relative, typed absolute"). Blender Alt-edit-all (precedent), Cavalry Alt-scrub-all. Old: WELL on relative-vs-absolute rule.
- **Candidate?** Yes.

---

## B. Transform

### B1 One Transform group: position, scale, rotation, anchor, opacity
- **Intent**: The five things everybody changes first are in one place at the top, in a fixed order, so muscle memory works.
- **Contract**: Order never changes between layers; every row has the same columns (label, values, unit, key, reset) so X/Y/Z line up vertically.
- **Came from**: I/transform/instrument.dart:216-253 (`_rows`), "every line keeps the same end column" at :298. AE Transform group (P/S/R/T/A order, precedent). Old: WELL.
- **Candidate?** Yes.

### B2 2D / 2.5D / 3D as a single switch that reveals Z
- **Intent**: I can say "this layer lives in 3D" once; Z fields, extra rotation axes and depth appear only then.
- **Contract**: In 2D, only X/Y and one rotation angle show; switching to 2.5D/3D adds Z and X/Y rotation without changing existing values.
- **Came from**: I/transform/instrument.dart:141 (Space chips 2D/2.5D/3D), :251 (rotation X/Y/Z outside 2D). AE 3D-layer switch (precedent: one toggle reveals Z and orientation). Old: MIXED; three named spaces is clearer than AE's boolean but the distinction 2.5D vs 3D is not self-explanatory [inferred].
- **Candidate?** Yes. Owner decides naming and whether three levels are wanted.

### B3 Direct-manipulation pad
- **Intent**: Besides numbers, I can grab a small stand-in for the layer and push/scale/rotate it, so I feel the change without hunting on the Stage.
- **Contract**: Pad drag edits the same values as the number fields live; Esc during drag puts the value back; Shift = finer; disabled when locked.
- **Came from**: I/transform/gizmo.dart:17 (move/scale/rotate/anchor modes); Esc abort and Shift fine per inventory. Precedent: none of AE/Figma have a pad in the panel; Blender has none either. Ableton has XY pads in devices. Old: POORLY [inferred]: four modes keyed 1-4, colour per mode, rotation-axis chips and a corner readout made a tool-within-a-tool; and the Stage already offers the same manipulation, so it duplicates.
- **Candidate?** No by default (duplicates the Stage). Keep only if the owner says "the panel must be usable without the Stage".

### B4 Anchor as nine points
- **Intent**: Setting the pivot to a corner/edge/centre is one click; hover previews where it will land.
- **Contract**: Click one of nine positions sets the pivot; hover previews on the Stage; a custom pivot is named "Custom" and typed values still work; changing the pivot does not move the layer visually [inferred].
- **Came from**: I/transform/instrument.dart:330 (`_anchorLine`, anchorPreview). Figma's 9-point alignment / Cavalry pivot presets [inferred]. Old: WELL on the intent (very quick), the drawing is not carried.
- **Candidate?** Yes.

### B5 Rotation survives turns
- **Intent**: Rotating past 360 keeps the turns (720 stays 720) so animating spins works.
- **Contract**: Rotation is stored/shown as unwrapped degrees; typed 450 stays 450.
- **Came from**: gizmo notes "unwrapped, turns kept" (inventory 3). AE shows "1x +90". Old: WELL.
- **Candidate?** Yes.

### B6 Opacity as percent, depth as an optional extra
- **Intent**: Opacity reads 0-100%; Depth appears only outside 2D and hands over to the depth tool.
- **Contract**: Percent is display only; 0.5 stored shows 50%; depth row hidden in 2D.
- **Came from**: I/slot.dart (displayScale 100), I/transform/instrument.dart:216. Old: WELL.
- **Candidate?** Yes.

### B7 Mode strip tied to value rows
- **Intent**: [inferred] Clicking a row's icon focuses that kind of manipulation.
- **Contract**: Clicking a row label focuses/selects that property for the Stage and for a shortcut (P/S/R/T show that row).
- **Came from**: I/transform/instrument.dart:216 (glyph button switches gizmo mode); docs/stage5/inspector.md (P/S/R/T focusProperty). AE P/S/R/T shortcuts (precedent). Old: MIXED.
- **Candidate?** Yes for the P/S/R/T "reveal this property" part; No for the mode strip.

### B8 Properties show unit riders
- **Intent**: Every number says what it measures (px, %, deg, s) without opening a tooltip.
- **Contract**: Unit shown next to the value when there is room; dropped, never the digits, when narrow.
- **Came from**: I/value_controls.dart:150 (`showUnit`), C/numeric.dart:52. Figma shows labels inside the field. Old: WELL.
- **Candidate?** Yes.

---

## C. Editing a value

### C1 Scrub
- **Intent**: I change a number by dragging it, and see the result live.
- **Contract**: Dragging right increases the value (and left decreases); the Stage updates during the drag; release commits as exactly one undo step.
- **Came from**: I/value_controls.dart:218-225; C/numeric.dart:334 (drag right = increase). AE (drag the value or the label), Figma (drag the label), Blender (LMB drag). Old: POORLY in one place: ValueToy subtracts the drag (`_v -= d.delta.dx * rate`, :222) so dragging right LOWERS the value, while EditorNumericField raises it. Two number controls with opposite directions. Do not repeat.
- **Candidate?** Yes. Owner confirms direction (every precedent: right = up).

### C2 Click to type exactly
- **Intent**: When I know the number I type it.
- **Contract**: Click (no movement beyond ~4 px) or Enter opens text input with all digits selected; Enter or leaving applies; invalid text keeps the field open and says "Number required"; Esc cancels and restores the old value.
- **Came from**: I/value_controls.dart:84-114,201-214; C/numeric.dart:210. AE (click value), Blender (click/Enter, Esc cancels, Tab = next field). Old: WELL on contract; a typed value is absolute for every target in a multi-selection (I/slot.dart typed).
- **Candidate?** Yes. Consider also Tab to the next field (Blender) [not in old].

### C3 Esc cancels
- **Intent**: If I change my mind mid-drag or mid-typing, Esc undoes only that gesture and leaves no history.
- **Contract**: Esc during a drag restores the pre-drag value and ignores the rest of the drag; Esc during typing discards; window blur / pointer cancel aborts without committing.
- **Came from**: I/value_controls.dart:162-164; docs/stage5/inspector.md (cancel on blur/pointer cancel). Ableton/Blender same. Old: WELL.
- **Candidate?** Yes.

### C4 Fine and coarse modifiers
- **Intent**: Same gesture, three speeds: normal, fast, fine; the user can predict each.
- **Contract**: Pick ONE rule for the whole panel, e.g. normal = 1 unit, Shift = x10, Alt/Ctrl = x0.1; the same rule for scrub, scroll and arrow keys.
- **Came from**: AE: Shift x10, Ctrl x0.1 (Adobe help). Figma: Shift = big nudge. Blender: Shift = precision, Ctrl = snap. Ableton: Ctrl/Cmd drag = fine. Old: POORLY: ValueToy uses Shift = x0.1 (I/value_controls.dart:122,222,137), EditorNumericField uses Shift = x10 (C/numeric.dart:256,278,390). Same key, opposite meaning in two controls of the same app. This is very likely the root of the "1% became 10%" complaint [inferred].
- **Candidate?** Yes, the single most important contract to pin. Owner picks AE's convention (Shift x10, Alt fine) or the other.

### C5 One scroll notch = one unit
- **Intent**: Wheel over a number steps it by a predictable amount.
- **Contract**: One notch = one displayed unit (1 px, 1 %, 1 deg); Shift = x10; one undo step per run of notches.
- **Came from**: C/numeric.dart:246-284 (notch = `speed * by`), comments: "each notch steps the value by one unit (Shift x10)". I/value_controls.dart:35 (step = magnitude * 5% for open values: the unit varies with the current value). Blender Ctrl-wheel. Old: POORLY: the step in ValueToy is relative to the current value's size, so the same notch moves 1 on a small number and 50 on 1000; in the numeric field it also requires the primary button to be held for a vertical wheel (:273,281). Not discoverable. Note also the Shift mismatch above.
- **Candidate?** Yes, with the fixed-unit contract.

### C6 Ladder of precision while dragging
- **Intent**: While dragging I can move the pointer up/down to switch between coarse and fine without keys.
- **Contract**: Level = x1, up = x10, down = x0.1, further down = x0.01; a small pill shows the active rung; changing rung keeps the value.
- **Came from**: C/numeric.dart:303-339 (rungs at +-32 px and 70 px; "the Figma ladder"). Closest real precedent: Photoshop/Blender-style slider fine control; I did not confirm that Figma has this [inferred, the code comment claims Figma]. Old: MIXED: elegant for mouse, invisible, not available on trackpad pan and not reachable by keyboard.
- **Candidate?** Maybe. Nice for pointers; owner decides.

### C7 Double-click resets to default; Delete/Backspace too
- **Intent**: Any value can be returned to its default in one gesture; I never need to remember the default.
- **Contract**: Double-click (or Delete/Backspace when focused) sets the default; a reset mark appears only when the value differs; reset of a keyed value writes a key, not wipes the track [inferred].
- **Came from**: I/value_controls.dart:169,206 (double-click within 320 ms), I/transform/instrument.dart:280 (reset arrow). Ableton double-click/Delete, Blender Backspace (precedents). Old: POORLY on the double-click: the first click already opens the text editor and the second one closes and resets it, so a text field flashes open (:210-212).
- **Candidate?** Yes. Owner picks double-click vs per-row reset arrow vs both.

### C8 Arrow-key nudge
- **Intent**: With a number focused, arrows step it (Shift = coarse).
- **Contract**: Up/Right = +1 step, Down/Left = -1 step; Shift per C4; one undo each.
- **Came from**: I/value_controls.dart:135,166; C/numeric.dart:383. Ableton Shift+arrows fine. Old: MIXED (step size again value-relative, see C5).
- **Candidate?** Yes.

### C9 Number never lies
- **Intent**: A narrow field may shorten digits but must never show a non-zero value as 0, or cut a digit ("10" for 100).
- **Contract**: When space is short: drop decimals first, then shrink, never truncate digits; a value that is not zero never reads 0.
- **Came from**: I/value_controls.dart:246-267. Old: WELL (a good rule).
- **Candidate?** Yes.

### C10 Value readout while dragging
- **Intent**: [inferred] I can read the number at the moment I drag, not only after.
- **Contract**: The number stays visible and updates live during drag; no tooltip is needed to know the value.
- **Came from**: Ableton readout during drag (precedent). Old: WELL (number is in the well itself).
- **Candidate?** Yes.

---

## D. Keys and animation

### D1 Per-value key toggle
- **Intent**: I can say "this value changes over time" on exactly one property.
- **Contract**: Key mark has three clearly different states: never animated / animated but no key here / keyed at the current time; click toggles a key at the playhead.
- **Came from**: I/transform/instrument.dart:276-279 (`_marks`), I/key_menu.dart:10. AE stopwatch (precedent). Old: MIXED: three states by tone only [inferred]; the mark is 7.5 px.
- **Candidate?** Yes.

### D2 "Animate all" / auto-key switch
- **Intent**: One switch turns on "every change I make now becomes a key".
- **Contract**: Visible in the header; state obvious (on/off); while on, editing an unkeyed value creates its first key at the playhead.
- **Came from**: I/transform/instrument.dart:102 (`tf-animate`). AE auto-key (Cavalry has a similar record mode [inferred]). Old: MIXED: the word "Animate" plus a diamond is clear but the effect on a given edit is not previewed.
- **Candidate?** Yes.

### D3 Right-click a value for key actions
- **Intent**: Less common actions (key this frame, remove key, relate) live in a context menu on the value.
- **Contract**: Right-click on any value offers Key this frame / Remove key and property extras; unavailable ones are greyed with reason.
- **Came from**: I/key_menu.dart:10, I/transform/store.dart:225. AE right-click. Old: WELL.
- **Candidate?** Yes.

### D4 Group key
- **Intent**: [inferred] Keying a group (e.g. all of Orbit) in one click.
- **Contract**: A group-level mark keys every member at once.
- **Came from**: I/camera/instrument.dart (`toggleKeys`, keys `key-orbit`...). Old: MIXED.
- **Candidate?** Maybe.

---

## E. Linking, bounds, mixed

### E1 Linked proportions on Scale
- **Intent**: I can lock width/height so changing one scales both without distortion, and unlock it without side effects.
- **Contract**: With the link on, changing X by a factor changes Y by the same factor (2:3, set X=4, gives 4:6); toggling the link itself changes nothing in the artwork; unlinked edits write only the touched axis; if one axis is 0, typing a value sets both.
- **Came from**: docs/stage5/inspector.md "Scaleのリンク"; I/value_controls.dart:67-70 (ratio via peers); I/transform/instrument.dart:265. Figma lock aspect ratio (precedent; note their history: constrain only applied to number entry, then "True Aspect Ratio Lock"). Old: WELL on contract; the link is a small chip at the row end that is easy to miss [inferred].
- **Candidate?** Yes.

### E2 Bounded values read as a bar; open values do not
- **Intent**: A value with a real min/max shows where it sits in its range; a value with no limit shows no fake range.
- **Contract**: Declared range -> a soft fill behind the number; no range -> none; values outside the hard limit clamp silently.
- **Came from**: I/value_controls.dart:1-5 (comment), :146-147,237. Ableton knob/slider position (precedent). Old: WELL on intent; [inferred] the fill is a look, only the idea is carried.
- **Candidate?** Yes.

### E3 Special words for special zeros
- **Intent**: Where 0 means "auto" or "none", the field says that word.
- **Contract**: 0 shows as the declared word; typing still takes numbers.
- **Came from**: C/numeric.dart:61, I/slot.dart (zeroWord). Old: WELL.
- **Candidate?** Yes.

### E4 Off-default marker
- **Intent**: At a glance I see which values I have changed from default.
- **Contract**: A small mark beside the label when value differs from default; a one-gesture reset (C7).
- **Came from**: I/inspector.dart:186 (off-default dot `mod-<id>`). Blender colours changed fields; AE has none. Old: MIXED (dot + arrow + diamond crowd the label line).
- **Candidate?** Yes, but choose one place for the mark.

---

## F. Effects and parameter groups

### F1 Effect = a card with on/off, fold, menu, drag to reorder
- **Intent**: An effect is one removable, reorderable unit; order matters and I can see it.
- **Contract**: Header has name, on/off, menu; clicking header folds; dragging header reorders; bypassed effect is visibly dimmed.
- **Came from**: effects/card.dart:18 (inventory 6); AE Effect Controls (fx toggle, collapse triangle). Old: MIXED: using the whole header as a drag grip AND fold-on-tap is easy to mis-trigger [inferred].
- **Candidate?** Yes.

### F2 Heroes first, the rest behind "Advanced"
- **Intent**: Show the few parameters that matter at the front; keep the long tail one click away.
- **Contract**: Declared heroes (else the first four) show; others under an Advanced fold with a count; a fold's open/closed state persists across layers [inferred]; search flattens everything.
- **Came from**: I/inspector.dart:16-26 (`heroIds`), :130 (PFold), :72 (filter). Cavalry/Blender collapse panels (precedent). Old: WELL on intent; "first four = heroes" is a fallback heuristic.
- **Candidate?** Yes.

### F3 Sections group related parameters
- **Intent**: Parameters are grouped by meaning with a small heading.
- **Contract**: Group label above related rows; groups foldable; one group not hiding an edited parameter.
- **Came from**: I/inspector.dart:129. Old: MIXED: colour tick per section dealt by hash [inferred] (no meaning); not carried.
- **Candidate?** Yes for grouping; No for per-section colours.

### F4 Two small controls side by side
- **Intent**: Short controls (toggles, small numbers) pair up in one row to save height; longer ones stay full width.
- **Contract**: Two-up only when both are small and the width allows; otherwise stacked.
- **Came from**: I/inspector.dart:37 (`isHalf`, width >= 188). Old: WELL.
- **Candidate?** Yes.

### F5 Effect menu operations
- **Intent**: Effect-level actions (move earlier/later, reset to rest, randomise, remove) live in one menu.
- **Contract**: Items disabled by position/capability; destructive item last.
- **Came from**: effects/card.dart:33. Old: MIXED: phrases such as "Throw every number within its reach" are poetic and unclear.
- **Candidate?** Yes for the list; No for the wording.

---

## G. Toggles, blend, parent

### G1 Blend, Ghost, Clip-to-below as a "world" block
- **Intent**: How this layer composites with the layers beneath it is one small area.
- **Contract**: Blend shows the current mode and opens the blend chooser; Ghost and Clip-to-below are on/off switches with a plain-word tooltip; Environment only for image layers; hidden for cameras.
- **Came from**: I/transform/instrument.dart:186-214. AE Transfer Controls (Blend / TrkMat / Track Matte; Blender has none like it). Old: MIXED: flag chips with tinted fill give no hint of what "Ghost" does until hover [inferred].
- **Candidate?** Yes.

### G2 Toggle and segmented choice
- **Intent**: On/off is a switch that says its state; a short list (up to 4) is a segmented control; a longer one is a stepper or menu.
- **Contract**: Switch shows On/Off as a word (not colour alone); out-of-range value highlights none; long lists open a menu rather than cycling blindly.
- **Came from**: I/value_controls.dart:327,357. Old: MIXED: the stepper for > 4 options cycles one by one (and Reference cycles on tap, :478) which is slow for long lists.
- **Candidate?** Yes.

### G3 Parent chooser
- **Intent**: Setting a layer's parent is one control in the Transform area.
- **Contract**: Shows the parent's name or "None"; changing it does not move the layer visually [inferred]; disabled when locked.
- **Came from**: I/transform/instrument.dart:148 (prev/next stepper, wraps). AE has a pick-whip + dropdown in the layer switches (precedent: dropdown list of layer names, plus a drag-to-pick). Old: POORLY: prev/next arrows cycle through all layers; with many layers it is unusable. A list/search or pick-from-Stage is the intent.
- **Candidate?** Yes. Intent only; control form is open.

---

## H. Panel behaviour

### H1 Narrow degrades by dropping, not by breaking
- **Intent**: A narrow panel still works; it drops units and decimals, stacks rows, and then scrolls, but never overlaps or clips.
- **Contract**: Below defined widths (old: 172/188 px): no units, 0 decimals, stacked third value; below a height it scrolls rather than overflows; digits never cut.
- **Came from**: I/transform/instrument.dart:60, I/inspector.dart:94, I/transform/card.dart (below 230 px fixed height and scroll). Old: MIXED: the breakpoints were hard-coded per instrument and the 345 px fixed height is arbitrary [inferred].
- **Candidate?** Yes.

### H2 Locked and frozen layers
- **Intent**: A locked layer is visibly read-only everywhere in the panel, with one reason shown.
- **Contract**: All inputs disabled, one reason line ("Locked" / "Frozen: effects are baked. Unfreeze to edit."); values still readable.
- **Came from**: I/transform/instrument.dart:94, effects/card.dart ("Frozen" message). Old: MIXED: dimming relied on tone alone.
- **Candidate?** Yes.

### H3 Search / filter for long lists
- **Intent**: When a layer has many parameters I can find one by typing.
- **Contract**: Search flattens groups and hides folds; empty result says "No parameter matches".
- **Came from**: I/inspector.dart:72 (body filter). Old: UNCERTAIN: the inventory could not confirm the filter is wired into the live panel (`RightSeat` does not reference `InspectorBody`).
- **Candidate?** Maybe.

---

## I. Hand-over between Stage, Inspector and other tools

### I1 Value opens its specialist tool
- **Intent**: Colour, font, blend mode, ease are not edited in a tiny control; the field shows the current value and a button hands over to the proper tool, which edits THAT slot of THAT layer.
- **Contract**: Clicking a colour/font/blend/ease value opens the matching tool already aimed at that property; the result applies to the selected unlocked layers; no popup; the tool can be closed with Esc.
- **Came from**: I/value_controls.dart:506 (`RouteToy` -> `store.route`), docs/stage5/inspector.md (swatch passes target slot to Colors; font name opens Fonts). Old: MIXED: intent is good (single source tool); the "Colors ->" pill label repeats the word the user already knows [inferred].
- **Candidate?** Yes. Biggest consistency win; owner picks how the tool opens (side panel vs bottom desk).

### I2 Stage selection <-> Inspector focus
- **Intent**: Picking a thing on the Stage makes the Inspector show it, and pressing a property key on the Stage (P/S/R/T) scrolls the Inspector to that property.
- **Contract**: Stage select updates the Inspector in the same frame; property shortcut reveals and focuses the matching row, unfolding if needed.
- **Came from**: docs/stage5/inspector.md (focusProperty/ensureVisible), I/session.dart:42,95. AE (P, S, R, T). Old: WELL on intent.
- **Candidate?** Yes.

### I3 Hover to preview on the Stage
- **Intent**: [inferred] Hovering a pivot or choice previews its effect on the Stage before I click.
- **Contract**: Hover shows the result on the Stage without committing; leaving removes it.
- **Came from**: I/transform/instrument.dart:330 (anchorPreview). Old: WELL for the anchor only.
- **Candidate?** Maybe.

### I4 Relations
- **Intent**: When a value is driven by another (or drives others), the row says so and one click shows the relationship.
- **Contract**: A mark on the row names the driver or counts the driven; clicking opens the relations view focused on it; a driven value's editability is stated.
- **Came from**: I/transform/instrument.dart:307 (relation badge), I/transform/store.dart:225. Cavalry connections (alt-drag attribute to attribute). Old: MIXED: red pill is loud and cramped [inferred].
- **Candidate?** Maybe, if the new tool has relations.

---

## J. Rough spots in the old app: DO NOT REPEAT

1. Two number controls with opposite scrub direction (ValueToy lowers on drag-right; EditorNumericField raises): I/value_controls.dart:222 vs C/numeric.dart:335.
2. Shift means x0.1 in one control and x10 in another (I/value_controls.dart:122,137,222 vs C/numeric.dart:256,278,390). Likely cause of the "1% steps behaved like 10%" complaint (complaints #1) [inferred, not reproduced].
3. Step size relative to the current value (`magnitude * .05`) instead of a fixed unit: I/value_controls.dart:24,35.
4. A wheel notch only steps a vertical scroll if the primary mouse button is held (C/numeric.dart:273,281; I/value_controls.dart:183): invisible, undiscoverable.
5. Click opens text edit immediately, so double-click-to-reset flashes the editor: I/value_controls.dart:203-214.
6. Parent chooser cycles through all layers with prev/next arrows: I/transform/instrument.dart:148.
7. Reference and long Choice controls cycle blindly one-by-one (I/value_controls.dart:357,468).
8. Section colours dealt by hash of the id: no meaning, noise (I/tones.dart:20).
9. Panel look diverged when docked ("fonts diverged", complaints #2) and Ease lines too heavy for a panel (complaint #3): panel voice must be checked in dock size, not alone.
10. Hover-only affordances (the "< >" grab hint, I/value_controls.dart:272) and 7.5 px key diamonds: hard to find and hit.
11. Animated-state shown by tone only (D1): needs shape/word, not just colour.
12. Duplicate manipulation (pad vs Stage) and a four-mode strip with keys 1-4: a second tool inside the panel.
13. Hard-coded breakpoints per instrument (172 / 188 / 230 px) and a fixed 345 px card height: use one rule.
14. Unclear poetic labels in menus ("Throw every number within its reach").
15. Several stories (Camera, Layout, Stage) have no test scene; I could not confirm those states were ever checked visually (inventory 1, uncertain).

---

## Precedents (what each does well)

- **After Effects** (Adobe help: properties and keyboard shortcuts, https://helpx.adobe.com/after-effects/using/keyboard-shortcuts-reference.html; scrubby slider summary https://planetphotoshop.com/scrubby-slider-shift-click-trick.html; Proportional Scrubbing https://helpx.adobe.com/after-effects/using/proportional-scrubbing-in-timeline.html): label-scrub, Shift = x10, Ctrl = x0.1, stopwatch per property, P/S/R/T reveal shortcuts, one fixed Transform order, proportional scrubbing across several selected layers. Does well: predictability and one global modifier rule. Does poorly [inferred]: tiny stopwatch states, dense Effect Controls.
- **Cavalry** (Attribute Editor https://cavalry.studio/docs/user-interface/menus/window-menu/attribute-editor/ ; Control Rows interaction https://cavalry.studio/docs/user-interface/menus/window-menu/attribute-editor/control-rows/control-rows-interaction/ ; Connections https://cavalry.studio/docs/getting-started/key-concepts/connections/): controls are scrubbed; Alt applies to all fields at once; Alt-drag makes a connection from an attribute to many layers; "Reset all Attributes" on the layer header without removing keys or connections. Does well: connections as first-class and reset-all that is safe.
- **Figma** (https://help.figma.com/hc/en-us/articles/360039956914-Adjust-the-position-or-dimensions-of-layers ; https://forum.figma.com/suggest-a-feature-11/launched-true-aspect-ratio-lock-36138/index2.html): hover the label to scrub, Alt on field also scrubs, lock aspect ratio beside W/H, the label sits inside the field. Does well: labels inside fields (saves width), linked proportions with a true ratio lock.
- **Blender** (https://docs.blender.org/manual/en/latest/interface/controls/buttons/fields.html): drag, Ctrl = snap to steps, Shift = precision, Ctrl-wheel, Enter/click types, Esc cancels, Tab = next field, Backspace = reset, Ctrl-C/V on a field, Alt = apply to all selected, drag vertically across fields to edit several at once. Does well: complete, discoverable-by-manual keyboard grammar and multi-selection editing.
- **Ableton** (https://www.ableton.com/en/manual/live-keyboard-shortcuts/ ; https://forum.ableton.com/viewtopic.php?f=3&t=46013): double-click or Delete resets to default; Ctrl/Cmd-drag = fine; Shift+arrows = fine; the value is read out while dragging/hovering. Does well: reset-to-default as a universal gesture and always-readable value.
- I could not verify Ableton's Shift vs Ctrl meaning on every platform; the sources above disagree slightly (one says Shift for fine drag, one says Ctrl/Cmd).


---
## Owner decision (2026-10-02): the spatial pad is a YES
The card about the direct-manipulation pad (B3) was marked 'No by default' because it duplicates what the Stage does. The owner says the good idea of the old app was exactly this: X/Y/Z as a GUI, not a table. So: **Transform is edited through a hands-on spatial GUI (pad for X/Y with Z depth, rotation dial, scale handles); the numeric fields are the secondary readout.** Only the INTENT is taken; the look is new (game-UI, Persona 5 as the named example).
