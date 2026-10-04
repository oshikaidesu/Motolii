# Ideas mined from the OLD Motolii: Browser and Inspector (problem awareness + mechanics)

Owner rule (2026-10-02): the old look is a counter-example, never imported. This file keeps only NEEDS (abstract), MECHANISMS, and what the owner said. No taste judgement. Old code root `/private/tmp/wt/tok/motolii/ui/lib` (= `L/`); product docs `Motolii/docs/stage5/` (= `D/`); lab cards `design-sense-lab/research/` (= `R/`). Items marked [inferred] are the card authors' reading, not stated by the owner.

## 0. Owner statements that are actually on record
- Browser filters were "modelled on Ableton" (R/intents/browser-filters.md:138). Liked in the old app: the Ableton-like filters; Transform as X/Y/Z GUI (R/intents/inspector.md:365: pad + dial + handles, numbers secondary, "only the INTENT is taken"); timeline expansion, eye, lock, "each parameter's display" (R/intents/timeline-expansion.md:297). Owner wants FEW symbols per row (timeline-expansion.md:264).
- Complaints (R/old-motolii-complaints.md): (1) scroll step 1% behaved as 10%; (2) docking made all fonts diverge; (3) Ease lines too thick/loud for a docked panel. Suggested contracts: one-line scroll/key/drag step rule; check "in the dock" for type, stroke, voice.
- The owner has NOT said which Ease behaviours they liked (R/intents/ease.md:5). The owner REJECTED Phase 4 "AEViewer-style media browser" because it "expanded behavior instead of reproducing the requested appearance" (D/browser-rebuild.md:14).
- D/ui-rebaseline/design-brief.md: Browser = "Ableton's browser as a toybox" (strict category list + search; each item a small toy); controls get faces only for creative concepts; save/search/number/toggle stay ordinary.

## BROWSER side

### B1. Need: say which pool I am looking in, and get back to it
- How: sources row (This project, Bundled, All, Favorites, Recent, folder chips, "+ Folder", "+ Import"), starts on "This project" if it has assets; offline folders stay visible as "·off"; breadcrumb + sub-folder counts. `L/browser/media/catalog_controls.dart:168,377`; R/intents/browser-filters.md:15-40.
- Worked: nothing silently disappears (offline chip); starting where the user's stuff is. Failed: a place, a saved view and a verb sit as equal chips; 3-4 control rows stacked above content in a narrow panel (:286).
- Not solved: no sidebar/content keyboard split (Left/Right), no per-place vs global search rule (browser-filters.md:114-120).

### B2. Need: narrow by describing the thing, not by knowing where it lives (filters)
- How: classification is DERIVED, not typed: a descriptor per thing (id, kind, family, tags, capabilities, source, searchTerms, face) + one closed registry with an `x-` extension lane + a validator; class column = top-level families present in current results; search keeps families holding matches. `D/ui-rebaseline/thing-metadata-proposal.md:§3`; `L/browser/things.dart:203`, `L/browser/classify.dart:132`, data `L/browser/data/things/registry.json`. Evidence: +100 descriptors as data only changed 0 Dart files (proposal §4).
- Capability model (not a shared widget): each shelf declares items/rails/classes/filters/selection/actions/availability/deps; presentation and morphology stay per panel; "share capabilities, not bodies" (`D/ui-rebaseline/browser-capability-audit.md:§3-4`).
- Worked: one-axis family strip; AND tokens in search; Esc clears then leaves, `/` or Cmd+F focuses (`L/browser/search.dart:30-69`).
- Failed: combination impossible (Blur AND Favorites): Favorites/Recent/colours/classes are alternatives in ONE strip; no "N filters on" count and no single Clear; no-result text is a dead end; kind filter single-choice though model takes a set (`catalog_controls.dart:200`); `kind:/tag:/cap:` query syntax is invisible (no chips); search hidden behind a magnifier in 4 shelves but always open in Media.
- Not solved: tag editing by the user, saved-search UI (class exists, `user_state.dart:7`, no creator), auto-tags; Colors and Fonts still not on descriptors (proposal §5); label/collection scope across panels undecided (capability-audit §5.2).

### B3. Need: mark favourites fast and get back; resume what I just used
- How: seven colour collections mirrored from Ableton + Recent (cap 12, `L/browser/user_state.dart:15,75,99`), Create board "Recent" row (max 8, hidden while searching).
- Failed: assigned only by context menu / `F`; Ableton's 1-7/0 keys not copied (the fast part).

### B4. Need: see what an effect/preset will do before applying
- How: tiles are the effect painted on a sample picture (`L/effects/shelf.dart:79` FxPainter) or a host snapshot (`L/browser/browser.dart:138`); colours show swatches, fonts set words in the font, curves draw the curve (`browser/create/faces.dart:16`, face types mark/fx/curve).
- Not solved: fixed sample, not the user's frame; NO Presets shelf and no hot-swap (try alternatives in place) existed (R/inventory/browser.md:210; browser-filters.md:231-238). Closest: Ease desk preset peek (see I5) and Colors "Saved" gradients with tap-to-apply, right-click Forget (inventory browser.md:133).

### B5. Need: use an item (place / apply) without ambiguity
- How: drag with ghost 85% + original 40% (`media/fluid.dart:165`); double-click/Enter places media; fonts: click dresses active layer, double-click makes a text layer (`browser/browser.dart:90-91`).
- Failed: single click = apply on effect tiles but = select on media (same gesture, two meanings, `browser.dart:102`); tiles with no host binding drawn at 48% and still clickable, 8 of 25 Create tiles, no reason shown (`:122`).

### B6. Need: browse large visual libraries; see inside a clip; find by resemblance
- How: List<->Thumbnail share the same faces (200 ms ease-out, selection kept, size slider 44-140, `media/media_browser.dart:383`); columns drop by width (NAME/TYPE/LENGTH/SIZE>=300/DATE>=420, `media/list.dart:7`); preview panel in place (`media/preview.dart:25`); Explore = pannable map of results, 1-2 hops, refuses >300 items (`media/explore/graph.dart:199`); missing-file badge + Locate, "Drop to import" veil (`catalog_controls.dart:215-227`).
- Failed: scrub only while PRESSED on a 2-4 px strip (`media/fluid.dart:603-694`), not hover; no auto-preview switch / own volume; Explore relation lines likely placeholder (spokes empty, inventory), cap makes it useless on big libraries.

### B7. Need: one browser that stays usable when docked small
- How: shelves are independent dock panels (`workspace/seats.dart:41`); morphs keyed to width (<260 chip) / height (<170 strip) (`browser/panel_chrome.dart:143`); tab strip only when stacked; detach to OS window (`D/ui-rebaseline/browser-panels-architecture-review.md:§1`). "Fold before shrink, scroll before crushing, keep the face."
- Failed: docking made fonts diverge (complaint 2); the 240 px cut copied 3x; every panel constructs all 6 shelves; the new shell listed browser panels as tabs of one region (review §4).

## INSPECTOR side

### I1. Need: one place to edit what I picked, honest about many layers
- How: subject = Empty / Camera / Layer (`L/inspector/session.dart:11`, `inspector_seat.dart:15`); "N layers" title; mixed value shows "—" (`inspector/slot.dart`); drag is RELATIVE to each layer, typed value ABSOLUTE; edit reaches every selected unlocked layer having the same row (`L/effects/store.dart:8-10`, `inspector/value_controls.dart:117`). Preview/commit/cancel: values go through a latest-pending queue, one commit, Esc/blur/pointer-cancel drops unsent value (`D/inspector.md`).
- Worked: relative-vs-absolute rule; "Locked"/"Frozen" word with disabled inputs.
- Not solved: search over parameters may not be wired to the live panel (inventory inspector.md:42; card H3).

### I2. Need: edit space (position/scale/rotation) by feel, numbers second
- How: Transform instrument with a direct-manipulation pad (move/scale/rotate/anchor modes, Shift fine, Esc aborts, `L/inspector/transform/gizmo.dart:17`); 2D/2.5D/3D switch reveals Z and X/Y rotation (`transform/instrument.dart:141,251`); nine-point anchor with Stage hover preview (`:330`); linked scale keeps ratio (2:3 -> X=4 gives 4:6; toggling the link changes nothing, `D/inspector.md`); rotation unwrapped (720 stays 720).
- Owner: the X/Y/Z GUI was the good idea (inspector.md:365). Failed: four modes on keys 1-4 plus per-mode colours made a tool-within-a-tool; duplicates the Stage (card B3).
- Not solved: parent chooser cycles with prev/next through all layers (`instrument.dart:148`); no hover-less way to see "off-default".

### I3. Need: change a number with predictable speed (the contract complaint #1)
- How: scrub / click-to-type / double-click default / arrows / wheel notch / precision "ladder" while dragging (`L/controls/panel/numeric.dart:210-339`); number never lies (decimals drop, digits never cut, `value_controls.dart:246-267`); unit riders drop when narrow; bounded values show a range fill, open values none; special zero words (`slot.dart` zeroWord).
- Failed (root of complaint #1, [inferred]): two number controls with OPPOSITE scrub direction and OPPOSITE Shift (x0.1 in `value_controls.dart:122,222`, x10 in `numeric.dart:256,278`); step relative to current magnitude (`value_controls.dart:24,35`); wheel steps only with a button held (`numeric.dart:273`); first click opens editor so double-click reset flashes it (`:203-214`).

### I4. Need: say "this changes over time", at the value
- How: key diamond per row, 3 states (never / animated / keyed here) by tone; tap keys at playhead; "Animate" auto-key switch in header (`transform/instrument.dart:102,276`); right-click a value: Key this frame / Remove key / relation extras, disabled with reason (`inspector/key_menu.dart:10`); group-level diamonds (camera groups `toggleKeys`).
- Failed: 7.5 px diamond; state by tone only; off-default dot + reset arrow + diamond + relation pill crowd one label line.
- Not solved: no value readout on timeline lanes (R/intents/timeline-expansion.md:192); keys on locked layers unguarded [inferred].

### I5. Need: reshape the easing between keys without a graph editor (Ease desk)
- How: band = which interval is running; square graph + family one-liners + preview; 9 curve families from the ENGINE (`easeModel` samples/handles, UI owns no interpolation maths); hover/arrow-key "peek" writes nothing, Esc restores, click/Apply = one `ease` command; drag handle = live preview, release = one undo (`D/ease.md`; `L/desks/ease/face.dart:399-437,581-650`, `desk.dart:232-277`). Reduce-motion respected.
- Failed: lines too thick for a dock (complaint 3); meanings drawn inside the plot only when `w>=230 && plotH>=150` so vanish docked (`face.dart:154`); overshoot switch in header away from the graph; preset tiles unnamed, 5-colour cycle encoding nothing; handle can leave the drawn window (-0.3 vs -0.15).
- Not solved: default for new keys, named saved curves, compare-with-previous (card list C1-C4).

### I6. Need: hand a value to its proper tool, and Fill/Font/Blend/Ease from one source
- How: "route" rows show current value + pill that opens the specialist (Colors/Fonts/Blend/Ease) AIMED at that layer's slot; no popup (`value_controls.dart:506`, `D/inspector.md`: swatch passes target slot to Colors; font name opens Fonts, applied via one SetTextDocument). Stage select <-> Inspector focus; P/S/R/T reveal the row (focusProperty/ensureVisible).
- Failed: "Colors ->" pill repeats the known word; Fonts shelf only picks a family name (no sample text).

### I7. Need: tame long parameter lists
- How: declared "heroes" (else first four) in front, rest behind an "Advanced n" fold; two small controls pair when >=188 px; sections foldable; effect = card (on/off, fold, menu, drag reorder) (`L/inspector/inspector.dart:16-26,37,129-130`, `effects/card.dart:18`).
- Failed: header both fold-on-tap and drag grip; per-section colour tick dealt by id hash carried no meaning; menu copy "Throw every number within its reach" unclear.

## Unusual mechanisms worth abstracting

### U1. Relations as an operation, not a place (D/relations-v0.md, 32 lines)
- A relation is NOT stored. It is the set of existing `PropertyLink`s (`motolii.link.remap`: in_min/in_max/out_min/out_max/clamp, + `in_component`, `out_components`) that happen to share a source and input range, regrouped for display (`L/desks/relations/model.dart:46-62` `relationsOf`). The link REPLACES the destination's own value (existing ownership rule), so no replace-vs-blend choice; cycles refused.
- Flow: right-click a value in Transform -> "Relation..." (value + vector axis become the source) -> dots = things at the current frame, laid out where they are on the Stage (`desk.dart:66-90`) -> click / Shift-click / LASSO (point-in-polygon; Shift adds, Esc cancels) picks the member set (`:130-170`) -> choose destination chips (Scale, Rotation, Opacity, Position) -> two scrubbable ends per range + "set current as min/max" arrows (`:309-325`; range seeded +-300 around the source's current value, `session.dart:38-45`) -> one `relate` command = one undo step, one link per member (1:N), source pinned to its current value first. `unrelate` keeps each thing's displayed value as its own constant. Deleting the source leaves members at default; deleting a member removes its link.
- Preview while scrubbing the range, one commit on release; units shown as percent/deg/px against stored ratios.
- Badges on the Inspector line: "◉ Null 1" (driven) / "◉ 8" (drives 8); click focuses the relation. v0 acceptance proved it in the live renderer, save/reopen, export (relations-v0.md:Acceptance).
- Not built: DistanceSignal (needs destination position in `translate_link`), Scatter/Along-path/Stagger "gadgets" (the concept `relation-gadgets.png`: visual representation as control surface, numbers second; `D/ui-rebaseline/brief.md:§7`). Legacy split `position.x/.y` vs vec2 forced "component" addressing. Relation badge lives in two places (ParamCell and `_line`).

### U2. Drivers / expression replacement as typed links
- AE pain: expressions are hated yet retain users (D/ae-pain-points.md:C-addendum). Answer: keep the capability, drop the grammar: ParamDriver (wiggle/LFO/time*n), keyframe loop modes, typed links LookAt/Follow/ParentRef + target picker, instance index, audio->attribute; long tail by WASM plugins. Coverage table lists loopOut as the one "undecided" gap. Mostly designed, little shipped (Relations v0 is the first shipped link kind).

### U3. UI owns no meaning; engine returns it
- Interpolation samples/handles, effect param declarations, `status.link`, descriptors all come from the host; Flutter renders and routes (`D/ease.md`; `D/ui-rebaseline/flutter-semantic-ownership.md`). Plugins declare params and the property panel is auto-generated (ae-pain-points.md:C).

### U4. Gesture grammar as a single contract
- Header of `L/inspector/value_controls.dart`: "drag = manipulate, Shift = fine, click = exact, double click = default, arrows nudge, Delete resets, hard min/max clamp silently, never draw a finite bar" - the intent was one grammar; the implementation forked it (I3).

### U5. Cancel as a first-class phase
- Esc, window blur, pointer-cancel abort an in-flight gesture and send no commit (D/inspector.md; friction ledger F17 showed the old timeline lacked it, `D/ui-friction-ledger.md`).

## Cross-cutting problems the old design did NOT solve (or hacked)
- Merging stacked states: source/kind/search/favourite/class as separate single-choice states; no combined filter indicator.
- Hot-swap of a preset/effect on the layer already in place; hover-scrub; auto-preview toggle.
- Hidden power (query syntax, ladder drag, wheel-with-button, hover-only "< >" grab hint, Alt-drag) with no cue.
- Presets: no shelf; only Ease presets (named later via Desk settings) and saved gradients.
- Per-panel breakpoints hard-coded (172/188/230 px, 345 px fixed card).
- Colour or tone as the only state signal (animated, locked, hero).
- Duplicate affordances for one thing (pad vs Stage; Explore vs search).
