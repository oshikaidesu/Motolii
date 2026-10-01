# Ease desk: intent cards (2026-10-02)

Scope: the OLD Motolii Ease desk, read as a counter-example. Only the intent and the interaction contract are extracted. Nothing about how it looked or was built is a spec.
Old code root: `/private/tmp/wt/tok/motolii/ui/lib/desks/ease/` (`face.dart` 786 lines = panel + painter, `desk.dart` 306 = session host, `meaning.dart` 13 = one-line meanings). Explorer story: `ui/explorer/lib/stories/panels.dart:187` ("Ease desk", 420x420, `many_keys.js`). Inventory: `research/inventory/timeline-stage-desks.md` items 57-63. Product note: `Motolii/docs/stage5/ease.md`.
The owner has NOT said which behaviours they liked. Every candidate is listed; "Candidate" is my recommendation to tick or not, not a decision. [inferred] = my reading, not stated anywhere.

---

## A. Choosing and shaping a curve

### A1. Pick a named curve in one click
- **Intent**: Make a change ease the way I mean without drawing anything.
- **Contract**:
  - One click on a named curve applies it to the current target as one undoable step.
  - Hover or arrow-key focus shows the curve ("peek") without writing anything; leaving the row or pressing Esc puts the real curve back.
  - Arrow/Home/End move the choice, Enter applies.
- **Where it came from**: `face.dart:400-437` (`_presetRow`, peek, keyboard), `face.dart:480-483` (`_apply`). Doc: `docs/stage5/ease.md` "hover/keyboard comparison writes nothing".
- **Old app: mixed.** WELL: peek-before-commit is a real, useful contract. POORLY: the tiles are tiny curve icons with no names in the row (name only appears in the plot corner while peeking); colours cycle through 5 hues per tile, which encodes nothing (decoration); the row scrolls horizontally, so presets hide at narrow widths.
- **Candidate?** Yes. Peek-without-writing is the valuable part. Drop the per-tile colour cycle.

### A2. The nine curve families and what each one means
- **Intent**: Understand what a curve does before I pick it.
- **Contract**:
  - Each family has a plain-language one-liner shown with its name (Hold: wait then jump; Linear: constant pace; Bezier: shape start and finish; Bounce: reach the end then rebound; Elastic: pass the end and spring back; Cyclic: repeat a wave; Random: vary the pace irregularly; Steps: distinct levels; ElasticSteps: spring into each level).
  - Families come from the engine's list, not from the UI (UI does not own interpolation maths).
- **Where it came from**: `meaning.dart:1-13`; kinds from host `easeKinds` (`desk.dart:19,51-63`). Flat-at-zero kinds are filtered out (`desk.dart:65`); Linear is injected if absent (`desk.dart:57`).
- **Old app: WELL for the wording, POORLY for placement.** The sentence was painted inside the plot canvas and only when the plot was tall and wide enough (`labels: w>=230 && plotH>=150`), so in a dock it usually vanished. [inferred]
- **Candidate?** Yes. Keep the one-liners; give them a place that is not in the graph and does not disappear when small.

### A3. Drag handles to shape a custom curve
- **Intent**: Bend the ease by hand and see the result immediately.
- **Contract**:
  - Dragging a handle changes the curve live and the preview/Stage moves with it.
  - Release commits exactly one undo step; Esc or focus loss during a drag restores the previous curve.
  - Bezier has two handles; other families expose the points the engine names (none for Hold/Linear).
  - A grab target stays finger-sized even when the drawn handle is small.
- **Where it came from**: `face.dart:581-650` (pick/move/pointer-up/cancel, 28px pick radius, Esc revert at `:625-640`), `desk.dart:232-243` (preview, cancel), `desk.dart:272-277` (one `ease` command per release).
- **Old app: WELL on commit/cancel semantics, POORLY on drawing.** Handles were big yellow discs with a halo and a thick yellow stem line. The vertical mapping is a fixed -0.15..1.15 window but handles may be dragged to -0.3..1.3 (`face.dart:600` vs `easePlotBox`), so an overshoot handle can leave the drawn box. [inferred from the two constants]
- **Candidate?** Yes. This is the core job. Keep the contract; redraw everything.

### A4. Allow overshoot (values outside 0-1)
- **Intent**: Let a curve go past the end value for anticipation or overshoot, or forbid it.
- **Contract**:
  - One visible switch decides whether Y may leave 0-1; X always stays 0-1.
  - Turning it off clamps handles and typed values to 0-1.
- **Where it came from**: `face.dart:196-209` header switch "Over OK", `:218-221` clamps, `:600`.
- **Old app: POORLY.** Defaulted ON, lived in the header trailing slot (not near the graph it governs), and the plot window did not grow to match. Product doc lists "Overshoot" at the bottom row in the newer layout, so the position was already unsettled.
- **Candidate?** Yes, but only as a small plain toggle next to the graph. Ask whether the default should be on.

### A5. Family parameters (elastic, steps, bounce and friends)
- **Intent**: Tune how springy / how many steps without leaving the panel.
- **Contract**:
  - Each family shows only its own named parameters (Bezier: X1 Y1 X2 Y2).
  - Changing a parameter redraws the curve from the engine's answer, not from UI maths; the latest answer wins when edits arrive fast.
  - Hold and Linear show "no numbers".
- **Where it came from**: `face.dart:240-262` (`_params`: every numeric key of the engine's description becomes a field, name split from camelCase), `desk.dart:207-223` (`easeModel` round trip). Exact parameter names (amplitude, period, steps, bounces...) are engine-defined and not visible in the UI code. [inferred]
- **Old app: mixed.** WELL: no UI-side curve maths for non-Bezier kinds (matches the "engine owns meaning" rule). POORLY: field labels are auto-generated uppercase strings, there is no unit or range hint, and an engine refusal surfaces only as a console error.
- **Candidate?** Yes. Needs the owner to list which parameters matter per family.

---

## B. Numbers

### B1. Numeric readout and exact entry
- **Intent**: See and type the exact values when the eye is not precise enough.
- **Contract**:
  - Click a value to type it; Enter commits (one undo step), Esc abandons.
  - Horizontal drag on a value scrubs it (old rate 0.005 per pixel) and commits on release.
  - With several different intervals selected the values show "-" and are not editable.
  - Scroll wheel over a value: NOT implemented in the old desk. If wanted, the project-wide rule applies: one notch = one unit, Shift = ten (see `old-motolii-complaints.md` #1). [inferred suggestion]
- **Where it came from**: `face.dart:264-362` (`_values`), scrub at `:289-293`, typing `:296-322`, mixed dash `:236`.
- **Old app: mixed.** WELL: click-to-type + drag-scrub + mixed state are the right set. POORLY: values were big (large numeric size) blocks with a loud yellow fill, two per row, taking more room than the graph in a narrow dock; scrub rate had no modifier for fine/coarse; no keyboard stepping.
- **Candidate?** Yes. The number is secondary to the shape (matches the owner's "number is 4th" ruling).

### B2. Mixed selection
- **Intent**: Edit several different intervals at once without losing what differs.
- **Contract**:
  - When intervals differ the readout says "Mixed" and shows all curves; choosing a preset sets all of them in one step.
- **Where it came from**: `desk.dart:124-131` ("Mixed", "N intervals"), `face.dart:178-195` (`sel=-1`), `:473-476` (`_apply` loops).
- **Old app: POORLY.** Mixed curves were drawn as three hard-coded colours capped at 3 [inferred from `cols=[kViolet,kMint,kPink]`, `i` would overflow beyond 3 curves], and the label said "3 intervals" as a literal string for the unhosted case.
- **Candidate?** Yes (lower priority). The behaviour is right; the representation was a stub.

---

## C. Saving, naming, applying

### C1. Save a curve as my own preset
- **Intent**: Keep a curve I made so I can reuse it later.
- **Contract**:
  - "Save" adds the current curve (kind + parameters) to a user list that survives tool switches.
  - Clicking a saved curve applies it to the target.
  - "Clear" removes saved presets (the copied curve is not one of them).
- **Where it came from**: `desk.dart:168-190` (`curveClip`, `easePresets`, `newKeyShape` in desk settings), `face.dart:492-506`.
- **Old app: POORLY.** Saved curves have NO NAME (only a thumbnail). "Clear" wipes all at once, no per-item delete, no rename, no undo. [inferred: no code path for name/delete one]
- **Candidate?** Yes. User explicitly asked about naming. Add name + delete one + undo for clear.

### C2. Copy a curve to another interval
- **Intent**: Paste the feel of one move onto another.
- **Contract**: "Copy curve" puts the current curve first in the saved row; clicking it applies it.
- **Where it came from**: `desk.dart:178-179`, `face.dart:495`.
- **Old app: POORLY.** Copy and Save were two near-identical buttons; the copied curve sat in the same thumbnail row as saved ones (distinguishable only by position). [inferred]
- **Candidate?** Maybe. One of Copy/Save could be dropped if "saved" is a clipboard of one.

### C3. Apply to the selected keys / interval
- **Intent**: Change the ease of the keys I selected.
- **Contract**:
  - Picking a curve or releasing a handle writes it to the interval(s) starting at the selected keys; the key selection is left untouched.
  - Locked layers or an engine that does not support ease make the panel read-only and say so.
- **Where it came from**: `desk.dart:256-277` (`apply` names the key, one command, one undo), `:124-132` (Read only).
- **Old app: WELL** for write semantics (one command, selection untouched). The original had no "Apply" button; apply is immediate.
- **Candidate?** Yes.

### C4. "Use for new keys"
- **Intent**: Every key I add from now on should ease this way.
- **Contract**: Sets the default curve for newly created keys; if Animate/record is armed it re-arms with the new curve.
- **Where it came from**: `desk.dart:187-191`. Same idea in Rive "Set as default" (https://rive.app/docs/editor/animate-mode/interpolation-easing).
- **Old app: POORLY.** A text chip among four chips; no indicator of which curve is the current default, no way to see or reset it. [inferred]
- **Candidate?** Yes. Needs a visible "current default" state.

---

## D. Preview and target

### D1. Preview of motion
- **Intent**: See the move play with this ease, to judge feel, not shape.
- **Contract**:
  - A play control runs one 0 to 1 sweep over the current curve, local to the panel (no document or Stage change); pressing again stops it.
  - The graph shows the progress marker; a value indicator at the edge reads the eased value.
  - Reduce Motion: skip the animation.
- **Where it came from**: `face.dart:439-459` (`_playButton`, 1.4 s), `face.dart:735-745` (nine-dot motion strip and moving dot). Doc `ease.md`: "一回再生、書類を変更しない".
- **Old app: mixed.** WELL: local, non-destructive, one-shot. POORLY: the play button was a preset-sized tile in the preset row (mixed up with presets); the 1.4 s is fixed with no duration control (cubic-bezier.com offers one); the "motion strip" of nine dots plus mint dot added a second visualisation of the same thing.
- **Candidate?** Yes. Duration control is optional.

### D2. Which interval am I editing? (target + playhead)
- **Intent**: Know what the curve I am touching belongs to, and where the playhead is in it.
- **Contract**:
  - The panel states its target in words: "Layer · property · frames", "N intervals", "Mixed", "Read only", or "No interval · Workspace".
  - The shown interval follows the playhead when it enters another interval; a manual pick holds until the playhead moves to a different one.
  - A line in the graph marks where the playhead is within the interval; nothing is drawn when the playhead is outside it.
- **Where it came from**: `desk.dart:121-132` (target text), `:100-119` (`current`, `playhead`), `face.dart:166-176`, 712-718.
- **Old app: mixed.** WELL: following the playhead and naming the target in words. POORLY: one dense dot-separated string in the header; the playhead line had a big flag and 1.6px white stroke competing with the curve.
- **Candidate?** Yes.

### D3. "No interval · Workspace" (a curve with nothing selected)
- **Intent**: Design a curve first, then use it, without needing keys selected.
- **Contract**:
  - With no interval, edits go to a kept "working curve" (not applied anywhere) and the label says "No interval · Workspace".
  - That curve stays after switching tools; selecting an interval later can apply it.
- **Where it came from**: `desk.dart:258-262` (`apply` stores `ease` in desk settings, Classic DK-030), `:125`, caption `:135`.
- **Old app: WELL** conceptually (the panel is never empty/dead). POORLY: the word "Workspace" is jargon for "scratch"; nothing says the edit is not being applied. [inferred]
- **Candidate?** Yes. Reword; make "nothing is applied" explicit.

### D4. Sequence ("ghost") mode
- **Intent**: Spread an ease over several selected layers as staggered delays.
- **Contract**:
  - No keys selected and 2+ layers selected: the curve defines each layer's delay in pick order; a dot per layer sits on the curve.
  - Drag previews live; release is one undo step; locked layers make it read-only.
- **Where it came from**: `desk.dart:68-80,121-125,230-243,263-269`; options switch `face.dart:508-521`; ghost curves `:658-663` (unhosted toy only).
- **Old app: POORLY.** The "Sequence ghosts" switch was display-only when hosted (mode comes from selection, the switch could not be flipped), so it looked like a control but was a status light. Ghost offset curves were a fake demo. [inferred]
- **Candidate?** Maybe. Powerful but a different job from "ease a key interval"; owner should decide if it belongs in Ease at all.

---

## E. Behaviour rules

### E1. Undo behaviour
- **Intent**: One gesture = one undo; cancelling leaves no trace.
- **Contract**:
  - Click preset / type value / release handle / apply sequence = exactly one undo step.
  - Drags use a draft preview; Esc or lost focus reverts without an undo entry.
  - Peek and Play write nothing.
- **Where it came from**: `desk.dart:232-243,257-277`; `docs/stage5/ease.md` ("ドラッグ中の下書き、Escape／focus lossの取消、Undoの単位を保持"). Tests named in the doc: `ease_interaction_test.dart`.
- **Old app: WELL.** This is the strongest part.
- **Candidate?** Yes, keep as a contract.

### E2. Narrow and short panel: what shrinks first
- **Intent**: The panel stays usable at dock size without hiding the main loop (see curve, pick, drag).
- **Contract (proposed, from old behaviour + product doc)**:
  1. Graph and presets always visible; they never scroll away.
  2. Shrink first: decorative labels, the one-liner meaning, corner captions, frame labels.
  3. Then numbers wrap from 2 columns to 1.
  4. Secondary groups (saved, options) scroll below, never the graph.
  5. Never shrink the graph/handles into a thin strip or stretch it wide to fill.
- **Where it came from**: `face.dart:131-165` (`_body`: strip if h<200, `plotH = 46% of h` clamped 110-300, labels only if w>=230 and plotH>=150, 1-column below 230); `docs/stage5/ease.md` ("小窓への収容を理由にカーブを縮小・横伸ばししない", graph square, only the list scrolls).
- **Old app: POORLY in fact.** Its own rule was the first one dropped: in the short strip the plot height was `h-20` and everything else scrolled below, and labels were the first to go, taking the meaning sentence with them. The adopted `ease.md` and the old code disagree (the doc says square, code says 46% of height). [inferred]
- **Candidate?** Yes. Contract E2 is the one the owner can most easily judge.

### E3. Keyboard
- **Intent**: Work without the mouse.
- **Contract**: Preset row: arrows/Home/End peek, Enter applies, Esc leaves; plot: Esc cancels drag; value: Enter/Esc. Global F9 = Easy Ease (from the window keys inventory, `timeline-stage-desks.md` item 90).
- **Where it came from**: `face.dart:400-420,625-640`.
- **Old app: WELL** for the preset row, **absent** for handles (no arrow-nudge).
- **Candidate?** Yes.

### E4. Easy Ease shortcut on keys
- **Intent**: One keystroke gives a sensible in/out ease on selected keys.
- **Contract**: F9 applies Easy Ease to selected keys; Shift/Cmd+Shift variants (in/out only) exist. [inferred from the inventory line; not in desk code]
- **Where it came from**: inventory item 90 (window keys). Precedent: AE Easy Ease (F9), https://helpx.adobe.com/after-effects/using/keyframe-interpolation.html (page blocked, 403; confirmed via search snippet https://www.premiumbeat.com/blog/understanding-keyframe-interpolation-after-effects/).
- **Old app**: lives outside the Ease desk; unverified. 
- **Candidate?** Maybe (only if the owner wants AE-style muscle memory).

---

## What the old Ease did that a docked panel must NOT do

Evidence is the old code; the owner's complaint is `old-motolii-complaints.md` #3 ("Ease lines too thick; too loud for one panel in a dock").
1. **Heavy, many strokes on the plot.** Main curve 2px, rails 1.2px, playhead 1.6px, handle stems 2px, mint motion line 1.6px, plus a 12/7px handle disc with halo, 7px endpoint discs, 6px current-value disc, 5.4px edge disc, nine edge dots. (`face.dart:700-745`) Everything is the same loud weight, so nothing is primary.
2. **Loud colour as the default.** A pink-violet-blue gradient fill under the curve, a solid yellow value block, 5-colour preset tiles, mint accents, blue play state (`face.dart:700-745`, `:317`, `:36`, `:439`). Colour carries no information (the product doc even says colour must not be the only signal).
3. **Large fixed-size furniture.** Large numeric type in filled blocks (`face.dart:317-319`), a plot at 46% of the panel height (110-300px), preset tiles at a fixed tile size in a horizontally scrolling row, and a header row with a switch, so at dock size most of the panel is chrome. [inferred from sizes]
4. **Information that vanishes when small.** Name, meaning, frame range and caption are drawn inside the plot only when `w>=230 && plotH>=150` (`face.dart:154`), so exactly in the docked case the explanation disappears.
5. **Handles that cannot be seen in range.** Fixed -0.15..1.15 window with drag to -0.3..1.3 (`face.dart:600`).
6. **Two visualisations of one thing.** The curve plus a nine-dot motion strip plus a mint dot, plus the playhead flag.
7. **Controls that look interactive but are not.** The hosted "Sequence ghosts" switch; ghost offset curves (unhosted toy only); unhosted preset list (Ease, Spring) differs from the hosted one.
8. **Unnamed saved items; one-button clear-all.**
9. **A literal "3 intervals" and a 3-colour cap** in mixed mode.
10. **Placement of Overshoot in the header** away from the graph.

---

## Precedent comparison (sources actually opened)

| Product | How it serves "choose / shape an ease" | Good for a small docked panel | Source |
|---|---|---|---|
| **After Effects** | Graph Editor with a Value graph and a Speed graph; Easy Ease (F9) as one command; Bezier direction handles; separate incoming/outgoing interpolation per key; influence/speed as numbers in a dialog | One keystroke covers the 80% case; the speed graph shows pace, not position, so an ease reads at a glance. Weak: handles live in a full-size editor, not a small panel | Search summary of https://helpx.adobe.com/after-effects/using/speed.html (direct fetch 403) and https://www.premiumbeat.com/blog/understanding-keyframe-interpolation-after-effects/ (opened) |
| **Cavalry** | Graph Editor with Linear, Bezier, Stepped, Auto-Bezier, Plateau, Spline, Clamped; named "Magic Easing" presets (Slow In/Slow Out, Spring, Anticipate, Bounce...) applied from the key's context menu; joined vs broken handles via Alt-drag; ghosting of previous curve states; grid snapping | Named presets in plain words with no editor needed; ghost of the previous curve is a cheap compare. Handle lock options are heavy for a dock | https://cavalry.studio/docs/user-interface/menus/window-menu/scene-window/graph-editor/ (opened); preset names from search summary (https://nstudio.uk/easing/ listed, not opened) |
| **Rive** | Interpolation panel for the selected key: Linear / Cubic / Hold / Elastic (Amplitude, Period); a small preview graph with two handles; a single text field of four numbers; "Set as default" for new keys; full Graph Editor toggle replaces the timeline | Closest to this panel's size: tiny graph + one numeric string, families with their own two parameters, a clear "default for new keys". Graph auto-adjusts range when handles leave view | https://rive.app/docs/editor/animate-mode/interpolation-easing (opened) |
| **Blender F-curve** | Interpolation mode (constant, linear, Bezier, easings, Back, Bounce, Elastic...), easing direction, handle types, F-Curve sidebar tab with numeric fields | Interpolation as a dropdown plus a few numeric fields is compact; separating "which family" from "which direction (in/out/in-out)" | https://docs.blender.org/manual/en/latest/editors/graph_editor/fcurves/introduction.html (opened; page returned only an outline, details from my own knowledge and not re-verified) |
| **cubic-bezier.com** | Four-number readout `cubic-bezier(0,0,.25,1)` with a copy button; preview and compare with a duration slider; Library of saved curves; click a library curve to compare against the current one; permalink; import/export | Compare-against-current is the best small-panel idea; copy-as-text; adjustable preview duration | https://cubic-bezier.com (opened) |
| **easings.net** | Families Sine, Quad...Back, Elastic, Bounce, each with In / Out / InOut; selecting one shows a graph plus animated demos (size, position, opacity) beside a linear reference | Name families by plain words; show the motion applied to something, with linear alongside for contrast | https://easings.net (opened) |
| **Ableton envelopes** | Click a segment to add a breakpoint; Alt/Option-drag a segment to curve it; double-click with Alt resets to straight; right-click Edit Value for an exact number; Shift-drag for fine vertical resolution; snap to grid with a modifier to bypass; Draw mode (B) | Direct manipulation with modifier-based fine control instead of extra widgets; reset-by-gesture; exact entry via context menu rather than permanent fields | https://www.ableton.com/en/manual/automation-and-editing-envelopes/ (opened) |

Note: the AE help page (helpx.adobe.com/after-effects/using/speed.html and keyframe-interpolation.html) returned 403 to the fetcher, so AE specifics above rely on a search summary and one tutorial; verify before quoting. Blender's manual page returned only a navigation outline.

### What the precedents suggest [inferred]
- Presets with names, not only icons (Cavalry, easings.net).
- Compare current vs candidate/previous (cubic-bezier.com library, Cavalry ghosting) — a stronger form of the old "peek".
- A numeric field as a single compact string or small fields, not big blocks (Rive, cubic-bezier.com).
- Modifier keys for fine control instead of more on-screen controls (Ableton).
- A visible "default for new keys" (Rive).

---

## Tick list for the owner
A1 pick named curve (with peek) / A2 one-liner meanings / A3 drag handles / A4 overshoot toggle / A5 family parameters / B1 numeric readout + typing + scrub / B2 mixed selection / C1 save + name curves / C2 copy curve / C3 apply to selected keys / C4 use for new keys / D1 preview play / D2 target + playhead following / D3 "No interval · Workspace" / D4 sequence mode / E1 undo contract / E2 narrow-panel priority / E3 keyboard / E4 Easy Ease key.
