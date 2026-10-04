# Precedents W6 (asset drag & drop) / W9 (easing editing) / W10 (keyboard only)

Date 2026-10-02. Tags: [seen] = read in vendor docs fetched this session; [snippet] = from a search-result summary only; [inferred] = my reading, not stated. No app was run.
Could not open: Adobe helpx (AE shortcuts, AE speed/graph editor, Premiere timeline) = HTTP 403; frame.io AE shortcuts = 504; Blender 5.2 manual pages (asset browser, list view, F-curve properties) fetched as nav-only shell with no body; Figma "Smart Animate" and prototype-easing pages = 404 or off-topic. So AE/Blender/Premiere facts below are [snippet] unless noted. Nothing here about pixel look (ghost opacity, stroke weight): no vendor doc states it; needs screenshots/running the apps.

## W6 Drag & drop of assets

| Question | Ableton Live | Blender | After Effects / Premiere |
|---|---|---|---|
| Drag ghost look | not documented [seen: manual silent] | not documented (docs unreadable) | not documented |
| Where drop accepted / target shown | Tracks in Session/Arrangement, or Device View; empty area right of Session tracks / below Arrangement tracks = "creates a new track" [seen] | 3D viewport or Outliner [snippet]; Outliner: drop on collection name or contents moves object into collection [snippet] | Project panel -> Timeline (AE layer stack / Premiere sequence) [snippet]; highlight style not documented |
| Drop onto existing object | Dropping a preset over an existing preset in the device chain replaces it [snippet] | Link vs Append is a header setting chosen before the drag, not at drop time [snippet] | AE: Alt+drop on a layer = replace source, keeps effects and keyframes [snippet]. Premiere: plain drop on clip = overwrite; Alt+drop = replace, keeps effects [snippet] |
| Modifier for alternate | Ctrl/Cmd only for folder opening in browser [seen]; nothing found for drag | Ctrl on Outliner drop = link to collection instead of move [snippet]; a commit "Add option to link assets on drag & drop" exists (could not open) | Premiere: Ctrl/Cmd = insert (default is overwrite) [snippet]. AE: Alt = replace [snippet] |
| Cancel with Esc | "Press Esc to cancel operations during drag procedures" [seen, vague] | Esc/RMB cancels modal ops [snippet]; for drags not confirmed | not found |
| Auto-scroll near edges | not found in any source | not found | not found |
| What appears on drop | New clip/device on selected track; double-click or Enter loads onto selected track as a non-drag path [seen]. Preview toggle + Shift+Enter / Right arrow auditions a sample before adding [seen] | Object in viewport at drop point; Outliner entry [snippet] | AE: new layer at top of stack / at playhead [inferred, not stated]; Premiere: clip at drop position, overwriting [snippet] |

Consensus: (1) the drop target is the real container (track, collection, layer stack), and dropping in "empty space past the last item" creates a new container [seen Ableton]. (2) Plain drop onto an existing thing is NOT additive for Premiere (overwrite) and Ableton (replace preset); a modifier changes it. (3) Esc cancels (weakly sourced). (4) A non-drag path exists (double-click/Enter) [seen Ableton].
Split: which modifier means what. Alt = replace-keep-properties in AE and Premiere; Ctrl/Cmd = insert in Premiere; Ctrl = link in Blender Outliner. Blender decides link/append up front in a header setting, not by modifier.
Unknown (needs hands-on): ghost opacity/shape, insertion line vs outline vs tint, auto-scroll speed.
Candidate defaults (options only):
- A: plain drop onto a layer = replace source keeping its effects/keys (AE Alt behaviour made the default); Alt+drop = add as new layer. Esc cancels, no write.
- B: plain drop onto layer = add new layer above it; Alt+drop = replace (AE/Premiere convention as is).
- C: drop only accepted in empty lane space (creates new), never on an existing item; replace is a separate Inspector action (avoids modifier entirely).

## W9 Easing editing

| Question | After Effects graph editor | Figma (prototype/Motion) | Blender graph editor |
|---|---|---|---|
| Preset vs handles | Both: Easy Ease (F9 or Keyframe Assistant) preset, then drag direction handles; handle drag = "influence" (distance) and speed (height) [snippet]. Numeric "Keyframe Velocity" dialog also [snippet] | Easing menu: Linear, Ease in, Ease out, Ease in and out, Ease in/out back, Hold; springs Gentle/Quick/Bouncy/Slow [seen]. "Custom Bezier": drag handles or type cubic-bezier numbers [seen]. Custom curves/springs can be saved as variables [seen] | T = interpolation mode, Ctrl+E = easing mode (menu) [snippet]; handles dragged freely; handle types (Auto Clamped etc.) [snippet] |
| Hover preset = peek without writing | not found | Only "hover over the preview window in Interaction details to see a preview" [snippet]: the preview pane, not preset rows. Hover-on-preset peek not documented | not found; Ease operator is a modal drag with a live Blend value [snippet] (live preview while dragging, Esc cancels) |
| Esc revert or commit | not found | not found | Esc (or RMB) cancels the running operation and reverts [snippet] |
| Handle snapping / constraint | Shift-drag snaps handle to horizontal/vertical; Alt/Option splits in/out handles; Ctrl/Cmd locks length [snippet] | y may leave 0..1: overshoot/"back" curves [snippet]. x clamp to 0..1 not stated (inferred: CSS cubic-bezier requires x in 0..1, UI likely clamps) | Ctrl snaps to increments, Shift slows (fine) during drag [snippet]; Auto Clamped handles stop overshoot propagation [snippet] |
| Tiny curve panel stroke weight | not documented | not documented | not documented |

Consensus: preset list AND free handles coexist; presets are a starting point, handles refine; y outside 0..1 means overshoot, allowed; a modifier gives axis-constrain/fine; Esc cancels any in-progress drag (Blender confirmed [snippet]).
Split: AE splits in/out handles (Alt); Figma has one curve per segment. AE exposes influence/speed numbers; Figma exposes cubic-bezier 4 numbers. Peek-on-hover for presets: no tool documented doing it; treat as our own invention (note memory rule: no invented meaning; ask user's "normal" first).
Candidate defaults:
- A: preset row click = write; hover = nothing; handles drag with Shift = axis lock; Esc during drag reverts to value at mousedown. (Figma/AE/Blender safest.)
- B: A plus hover on a preset shows the curve in the tiny panel only (no value write, no motion). Cheap middle ground.
- C: A plus hover plays the motion on the Stage as a temporary overlay that reverts on leave (true peek; no precedent found, needs user ruling).
Constraint option: x of both control points clamped 0..1, y free (-1..2 range shown), numeric 4-field entry as a fallback.

## W10 Keyboard-only

Sources: Ableton v12 manual [seen]; Figma via search lists [snippet]; AE via search lists [snippet]; Blender via search lists [snippet].

| Action | Ableton | After Effects | Blender | Figma |
|---|---|---|---|---|
| Play/pause | Space; Shift+Space continue [seen] | Space [snippet] | Space [snippet] | n/a (prototype only) |
| Step frame/nudge time | Left/Right arrows nudge/navigate [seen] | Page Down/Up, Ctrl+Left/Right [snippet]; Shift+PgUp/Dn = 10 frames [snippet] | Left/Right arrows = frame [snippet] | n/a |
| Prev/next keyframe | not found | J / K [snippet] | Down/Up arrow [snippet] | n/a |
| Jump start/end | Home / End [seen] | Home / End [snippet] | Shift+Left (first frame) [snippet]; end = Shift+Right [snippet, from earlier summary] | n/a |
| Set key | n/a (clip-based) | not verified (Alt+Shift+P is the one I know; unconfirmed) | I [snippet] | n/a |
| Delete | Delete [seen] | Delete [inferred] | X or Delete [snippet] | Delete/Backspace [inferred] |
| Undo/Redo | Cmd+Z / Cmd+Shift+Z (Win Ctrl+Y) [seen] | Cmd+Z / Cmd+Shift+Z [inferred] | Ctrl+Z / Ctrl+Shift+Z [inferred] | Cmd+Z [snippet] |
| Zoom | Cmd +/- [seen] | , and . (not verified) | Home = view all [inferred] | Shift+1 fit [snippet] |
| Arrow nudge object | arrows nudge [seen] | arrows nudge layer (not verified) | n/a | 1px; Shift = 10px [snippet] |
| Select next/prev | arrows between clips/slots [seen] | not found | not found | Tab = next sibling; Enter = children [snippet] |
| Shift meaning | Shift+drag = fine [seen] | Shift+PgUp/Dn = x10 [snippet] | Shift = slow/fine during transform [snippet] | Shift+arrow = x10 [snippet] |
| Alt/Cmd | Alt(Win)/Cmd(Mac)+drag bypass snap [seen] | Alt = replace/split handles [snippet] | Ctrl = snap to increments [snippet] | n/a |

Conflicts (shared key, different meaning):
- Shift: Ableton fine (small) vs Figma/AE large (x10) vs Blender slow-in-drag. Fine vs large is the real split; Blender and Ableton agree on fine-by-Shift for drags, Figma/AE agree on x10 for steps.
- Left/Right: Blender = frame step; Ableton = nudge/navigate; AE = layer nudge, frame is Page Up/Down or Ctrl+arrow.
- Up/Down: Blender = keyframe jump; AE uses J/K for that.
- Redo: Ctrl+Y (Ableton Win) vs Ctrl+Shift+Z.
- Home/End: Ableton/AE = project start/end; Blender Home = view all, Shift+Left = jump start.
- Esc: Ableton closes dialogs / cancels drag; Blender cancels modal op.
- Alt: replace (AE/Premiere) vs bypass snap (Ableton Win) vs split handles (AE graph).

Minimal shared set (3+ tools agree, [snippet]-grade): Space play/pause; Delete; Cmd/Ctrl+Z undo; Cmd+Shift+Z redo; Home/End jump ends (Ableton+AE); Esc cancel; arrows step/nudge one unit; Shift = modifier for step size.
Split: step-frame key, keyframe jump key, set-key key, zoom key.
Candidate defaults:
- A (AE-like): Space, Left/Right = 1 frame, Shift+Left/Right = 10 frames, J/K prev/next key, Home/End.
- B (Blender-like): Space, Left/Right = 1 frame, Up/Down = prev/next key, I = set key, Shift = x10 only on steps, Ctrl = snap on drag.
- C (Figma/Ableton-like, object-first): arrows nudge selected 1 unit, Shift+arrow x10 (Figma) and Shift+drag = fine (Ableton); time stepped with , and . ; Tab/Shift+Tab select next/prev; Esc cancels drag, else deselects.
