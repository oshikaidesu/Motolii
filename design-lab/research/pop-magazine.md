# Pop magazine / game-UI direction as a seasoning (Dose 0-3)

Supersedes `research/editorial.md`. Date 2026-10-02. Rewritten after three owner messages the same day:
1. Persona 5 is the reference; the tool should feel like a game UI.
2. THE GUARD (overrides the first framing): KEEP the current tone (calm greys, 1 px hairlines, Inter 10-11 px, colour only as relation-family marks plus one accent, strength from fine detail not weight). The pop/game energy is a restrained SEASONING added on top. The owner's fear is an over-the-top UI.
3. Persona matters as UX: the act of choosing is itself graphic.

Tags: `[seen]` = in a page I opened. `[snippet]` = only in a search-result summary, page not opened. `[inferred]` = my reasoning. `[proposal]` = a value I chose to test; not a measurement.

Honest limits:
- No source I opened gives measured numbers for a magazine cover or for Persona menus (palette size, line px, skew degrees, timing in ms). Every number in sections 5-9 is `[proposal]` unless tagged otherwise.
- The CoroCoro and Persona sources describe loudness built from HEAVY shapes (stacked thick outlines, big slabs, thick black text outlines). The owner's condition is the opposite. So sources give MECHANISMS; the fine-detail rendering is my adaptation, and it is untested.

## 1. Sources

Opened and read (via fetch; the fetch tool summarises, so excerpts are short):
- CoroCoro designer Sasaki interview: https://corocoro-news.jp/news/36121/ (read three times with different questions)
- HIGH-FIVE note on the CoroCoro cover: https://note.com/highfive_creek/n/n2c1ff8434a04
- Togetter, readers on the one issue with a different cover designer: https://togetter.com/li/2576944
- Jump lettering history: https://bijutsutecho.com/magazine/insight/26406 (little on covers)
- Sendenkaigi Sasaki interview (paywalled; lead only), ITmedia Guinness article (no design detail)
- Persona 5, Atlus CEDEC+KYUSHU 2017 talk report (Famitsu): https://www.famitsu.com/news/201711/13145540.html
- Persona 5 interview summary: https://personacentral.com/persona-5-interview-ui-design-sound-music/
- Persona 5 dev-talk thread (summary of the same talk): https://www.resetera.com/threads/atlus-talks-about-persona-5s-ui-design-dev-talk.5865/
- Siliconera on the same talk: https://www.siliconera.com/atlus-reveals-design-secrets-behind-persona-5s-distinctive-ui/
- Jiaxin Wen, "The UI Design of Persona 5": https://jiaxinwen.wordpress.com/2017/04/27/the-ui-design-of-persona-5/ (the best interaction detail I found)
- findmanblog (Japanese breakdown): https://www.findmanblog.net/entry/2020/12/30/174818
- GameUI Lab on Persona 5 The Phantom X: https://uidesign.chodoiilife.com/p5x/
- ameblo, "P5 game design and UI together": https://ameblo.jp/amamiyasouta1211/entry-12766728068.html
- ResetEra, complaints about Persona 5 UI: https://www.resetera.com/threads/im-enjoying-persona-5-but-i-hate-the-ui.105249/
- Splatoon, talk reports: https://careerhack.en-japan.com/report/detail/965, https://mazco.hatenablog.jp/entry/2018/05/03/220000, https://goodpatch.com/blog/uicrunch-13 (no numbers), https://geechristine.wordpress.com/2015/07/08/ux-design-in-splatoon/ (no visual specifics)
- Teenage Engineering OP-1 layout guide: https://teenage.engineering/guides/op-1/original/layout
- Halftone in CSS: https://blog.master.dev/pure-css-halftone-effect-in-3-declarations/
- W3C WCAG 1.4.3: https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html
- Rive blog on accessible designs: https://rive.app/blog/making-rive-designs-more-visually-accessible (about authored content, not Rive's own UI)
- Pokemon stat hexagon, a THIRD-PARTY web recreation of the Gen 5 screen, not the game: https://pokemoncn.dev/components/stat-hexagon

Search snippets only (page not opened or blocked): Medium Persona 5 analyses (skew angles; fruitcupkun, marktan, ousiadroid, design-bootcamp, kinga.olszewska, game-design-fundamentals: all returned HTTP 403 or no output); Blender manual gizmo page (fetched but content empty; axis colours and gizmo parts come from search summaries); TV Tropes Ring Menu; Game UI Database (403); Playdate help/SDK; Ableton manual; Kaoss Pad; Baldur's Gate 3 character creator articles; Famitsu cover facts.

Could NOT open: Behance OP-1 UI case study (403), medium.com haiiro-io on Splatoon (403), iwataasks.nintendo.com (certificate), ridwankhan.com Persona UX (fetch failed), codepen Persona menu clone (403), gameuidatabase (403).
Not found at all: any talk or article by Lisa Sato or Shigenori Soejima on the UI; any GDC/Game Developer piece on Persona menu timing; cursor-movement timings in ms; confirm/cancel feel analysis; a game-styled X/Y/Z pad, rotation dial or scale handle shipped in a tool or game; analyses of Famitsu, Nintendo Dream, CD Journal layouts; anything on Cavalry, FigJam, Notion, Linear, Zelda/Metroid menus, rhythm-game HUD design (only judgement-line mechanics from search snippets). I make no claims about those.

## 2. What was actually seen

CoroCoro cover:
- Aim: so "hot" you let go of it (designer's words, Japanese). [seen]
- "Pack here, open there"; small, medium, large type each in their own role. [seen]
- Headline type and visuals got bigger since 1977 because the information grew. [seen]
- Custom typefaces from a type house; hand-specified layout; 2 days per cover from an editor's rough; the magazine logo must never be hidden. [seen]
- 20+ serial titles on one A5 cover, parts kept from overlapping. [seen, lead only]
- Title tricks reported by a note author and a search summary: outlines stacked in 4 steps, gradient, offset shadow, depth by placement, rainbow text, no alignment rules. [seen, secondhand]
- Readers of the off-style issue: usual one "packed and scattered", new one "organised, colours calmer"; a child did not recognise it as CoroCoro. [seen] Loudness is the identity.
- Not found: colours per cover, outline px, any grid.

Persona 5 (Atlus talk reports, consistent across 4 pages):
- Concept "pop punk". Main colour red; sub-colours kept out "as much as possible" (the only sub-colour exception reported: HP/MP). [seen]
- Readability tricks named by the designer: (a) a white line drawn through the menu to lead the eye ("people follow lines"); (b) priority information sits in higher-lightness space, the rest lower; (c) when moving down a hierarchy, layout and angle change so level is recognisable. [seen]
- Menu = split screen, one half black/white, one half red, lines pointing to the centre where the text is. [seen, secondhand]
- Reported skew values (blocks -10 degrees, slabs -16 degrees) come from a Medium analysis summary I could not open. [snippet]
- Solid fills, no gradients, but "subtle colour changes" inside the solid fills; every screen looks independently designed yet one consistent style; generic UI is not reused (a shop with 2 choices still has its own UI). [seen]
- Text: cut-out newspaper (ransom note) look; figure/ground swapped and colour changed on part of a name. [seen]
- Menu entries carry animation; data preloaded to avoid lag; UI vector-converted for PS3/PS4; designers hand positions to programmers and adjust "1 dot, 1 frame". [seen]
- Business context: the UI was a low-cost marketing tool to change the series' brand. [seen]
- Complaints from players (one forum thread): inconsistent character size inside a word; black-and-white pause-menu elements blend; victory splash too long; constant movement, "headaches", "strains my eyes". [seen] These are the failure modes to design against.

Splatoon (Nintendo talk): saturation budget (icons high chroma, containers low); hue-shifted shadows (yellow shadow orange, pink shadow purple); greyscale and squint tests; custom face made thinner and more angular when information density rose; shapes recognisable without colour. [seen]

OP-1: coloured encoders tied to same-coloured on-screen graphics. [seen]
Pokemon stat screen (third-party recreation, parameters of the recreation not the game): 4 rings at 25/50/75/100% of radius, 6 spokes, outer ring 1.5 px at 55% alpha, inner rings and spokes 1 px at 18% alpha, polygon fill 50% alpha, 2.5 px vertex dots, 10 px mono labels at 70% alpha. [seen]
Blender transform gizmo (search summary): X red, Y green, Z blue; small coloured squares for two-axis moves; white centre for free move. [snippet]
Halftone CSS: dot/map multiply then `contrast(16)` for small dots, 80+ for large; blur then contrast for jagged edges; offset multi-colour screens 30 degrees against moire. [seen]
WCAG: 4.5:1 text, 3:1 large text and component boundaries; decorative text and logotypes exempt; no method for text over patterns. [seen / snippet]
Rive: buttons at least 24x24 px "though I usually go bigger". [seen]

## 3. Mechanisms that keep a loud layout coherent (ranked)

1. Few size/type classes with big jumps (CoroCoro). [seen]
2. One dominant colour and no sub-colours; plus lightness for priority (Persona, saturation for priority in Splatoon). Both rank by a single channel. [seen]
3. A lead line: a white line through the screen that the eye follows (Persona). [seen]
4. Pack here, open there (CoroCoro, Splatoon shop removing posters). [seen]
5. Repeated treatment, varied content: each screen unique, one family of tricks (Persona, CoroCoro). [seen]
6. Level cue: layout/angle changes when going down a level (Persona). [seen]
7. Shapes readable without colour; greyscale test (Splatoon). [seen]
8. A fixed anchor never covered: the logo (CoroCoro), the red (Persona). [seen]
9. Loudness is recognition (Togetter). [seen]
No source states a hidden grid. Do not assume one.

## 4. Persona as UX: choosing is itself graphic

What the sources show about how Persona menus work as interaction:
- The whole screen is the menu. Battle commands are placed around the acting character rather than in a box; buttons are long triangles that point at the character whose turn it is. [seen: ameblo, Jiaxin Wen]
- Each choice has its own place and form, and each context its own layout (even a 2-choice shop). [seen]
- The selected choice is the loudest element: the selected tab is the red one; the menu list is brought in with faster animation than the background so attention goes to the list. [seen: chodoiilife, Jiaxin Wen]
- Face buttons map to categories (triangle = skill, square = item), so the common choice is one press, no cursor travel. [seen via search summary of an ameblo-type page; the ameblo page I opened confirmed commands around the character]
- Next step and consequence: a sliding shape bridges the camera change from fight to menu; the character stays left and the monster right so the scene is not hidden; the battle result shows 4 facts at once (win, money, experience, item). [seen]
- Sub-menus as spatial continuation: hierarchy change is shown by changing layout and angle. [seen: Famitsu talk report]
- Layered motion speeds: foreground menu fast, background slow. [seen]
- Fast for repeat use: the result screen is built to need no button mashing; but the community lists long victory and shop animations as the cost. [seen]
- NOT found: timing of the highlight in ms, how cancel/confirm feel, any analysis of repeat-use skipping.

Principle, stated for a tool [inferred]: a choice is shown as a small set of placed, distinct, previewable things (not a text list); the current one is the loudest thing on screen; hovering or moving previews the consequence; leaving is always cheap; the whole sequence is as fast as a list.

### Choice points of a pro editing tool, graphic form at the restrained dose
All keep: selection state is the only place energy goes; geometry never moves on hover (hover previews, commit moves); nothing animates longer than the dose limit (section 7); Esc reverts a preview.

| Choice point | Graphic form (fine detail) | Keyboard / mouse contract | Dose it earns | Plain or graphic |
|---|---|---|---|---|
| Relation type (6 families) | A row of 6 chips, each 24 x 20 with a 14 px line pictogram and its family hue as 1 px frame; selected chip: frame at 100%, tiny kicker above it with the name, 2 px cut corner at dose 2+ | Arrow keys move, 1-6 jump, Enter commit, Esc revert; hover shows the relation on the Stage as a hairline preview, no commit | 1-3 | Graphic: only 6 items, high meaning |
| Blend mode (about 25-30 modes) | Plain list grouped like After Effects (separators); at dose 2+ a 16 px live swatch at the row's end (checker over gradient) | Type-to-filter, Up/Down, Enter; hover previews on the Stage after 250 ms | 0-1 | PLAIN: long list, scanning speed and alphabet memory matter; a ring of 30 items is slower |
| 2D / 2.5D / 3D | 3 small plates (28 x 22) drawn in hairline: flat square, tilted square, cube; the selected plate gets hue hairline; the choice switches the Stage preview on commit | Left/Right, 2/3 keys, Enter | 1-3 | Graphic: 3 items, spatial meaning |
| Easing preset (8-12 curves) | Row/grid of 28 x 20 hairline curve thumbnails (1 px curve, 0.18 alpha frame); selected: hue hairline, corner cut at dose 2+, curve drawn at 1.5 px | Arrows, Enter, hover animates a 12 px dot along the curve inside the thumbnail only | 1-3 | Graphic |
| Effect choice (many) | Browser tiles with small live thumbnail; selected tile: 1 px accent outline (as draft), kicker with category, tiny badge for state | Type-to-filter, arrows, Enter; double-click applies | 1-2 | Tiles graphic, but the search field and long results list stay plain |
| Tool / mode | Segmented control; selected: g20 pill, 1 px accent underline (draft), at dose 2+ a 3 px slash cue and a key hint badge | Single-letter keys (as in AE), no animation beyond 90 ms | 1-2 | Mostly plain: used hundreds of times a day |
| Tabs | Draft underline; dose 2+ adds a numbered kicker (01, 02) and a lead hairline across the panel top | Ctrl+1..9, arrows | 1-2 | Plain at dose 0 |
| Numeric field (editing) | none | Type, Tab, arrows, scrub | 0 always | PLAIN: speed and precision while typing; any tint competes with the digit |
| Long property or layer lists | none (row bands and 1 px lines) | Standard | 0 always | PLAIN: hundreds of rows; Persona's own forum complaint is noise in the busiest screens |

Why those stay plain [inferred, supported by the Persona complaints]: repeated, high-frequency choices punish decoration with time (animation) and attention (noise); the Persona designers solved this by keeping a single colour and moving the energy to the entrance and the selected item, not to the whole list.

## 5. Fine-detail techniques (all `[proposal]` ranges)

- Hairline accent: 1 px (device-pixel snapped), hue at 50-100% alpha; companion inner highlight 1 px at white 10-24%.
- Corner cut instead of a slab: a 2-6 px chamfer (45 degrees) on one or two corners of a badge, chip or selected tile; drawn as a path with a 1 px stroke. (Persona's angular cut-outs, rendered at 1/10 the scale.)
- Skew: only on small non-editable labels (kicker, badge): -6 to -10 degrees; never on text the user edits. Persona-reported -10/-16 degrees [snippet].
- Lead line: one 1 px line at 14-30% white across a panel, from the kicker to the first control (Persona's white lead line, fine).
- Kicker: a 9-10 px caps label +0.15 to +0.22 em tracking above a heading; decorative or redundant, informational text stays at 10+ px per the lab rule.
- Figure/ground swap on one word of a heading (Persona text trick), at 13 px only.
- Hue-shifted 1 px offset shadow for a selected badge (Splatoon rule): hue +20 to +35 degrees, no blur, no black.
- Halftone: pitch 3-6 px, dot radius 0.5-1.5 px, alpha 6-12%, 45 degrees (two screens 30 degrees apart), pre-rendered tile or shader; never under editable text.
- Tiny badge: 14-16 px tall, 8.5-9.5 px caps, 1 px frame, hue = a family or the accent; 0-3 per tile.
- Colour: no new hue beyond the six families and the accent (the Guard); extra energy comes from the SAME hues at different alpha/saturation.
- Motion: ease-out, starts the frame after input; see dose table.

## 6. Spatial hands-on GUI (pad for X/Y/Z, rotation dial, scale handles)

What shipped examples I found:
- Blender-style gizmo: axis colours X red, Y green, Z blue; small coloured squares for two-axis moves; a white centre for free move. [snippet]
- Kaoss Pad (1999): an XY pad as the main performance surface. [snippet]
- Pokemon Gen 5 stat hexagon, as recreated by a third party: nested rings at 25/50/75/100%, 1 px at 18% alpha, outer ring 1.5 px at 55%, translucent filled polygon, 2.5 px vertex dots. [seen] The only concrete "game stat pad" numbers I have, and they are already a hairline language.
- Radial / ring menus: Persona 3 battle ring and the 3 Portable revolver look; ring menu as a known pattern. [snippet]
- Character creation: Baldur's Gate 3 chose presets plus attachments over sliders; the lead says sliders take effort and results look alike. [snippet] A counter-precedent: pick-from-shapes can beat free sliders.
- OP-1: colour of the on-screen graphic tells which physical control owns it. [seen]
- Not found: a game-styled X/Y/Z pad, rotation dial or scale handle in any shipped game or editor.

A hairline game-language sketch [proposal, untested]:
- X/Y pad: 112-160 px square, 1 px frame, 1 px grid at 25/50/75% with 18% alpha (the hexagon recipe), crosshair at 30% alpha, a 6 px ring cursor with 1 px hue stroke; 4 corner L-ticks (4 px). Z: a 1 px vertical rail beside it with 5 ticks and a 5 px diamond; or a depth ring. Axis hues: collision risk with family hues (pink, green, blue); at dose 1 axes are grey (X/Y/Z letters in 8.5 px caps), at dose 2-3 each axis gets a 1 px hue tick only, not a fill.
- Rotation dial: 1 px circle 64-96 px; ticks every 5 degrees (3 px), every 15 (5 px), every 90 (8 px with 8.5 px label); 6 px handle; detent at 15 degrees with a 40-60 ms tick flash.
- Scale handles: 5 px squares at corners and edge midpoints with a 24 x 24 hit area (Rive's minimum target) [inferred]; a 1 px hairline box; ratio-lock shown as a diagonal 1 px line.
- Quiet while dragging: the numeric readout in the draft mono 11 px; no pattern, no skew, no tint on the pad while a value is being typed.

## 7. Dose scale (0..3): same tone, more seasoning

Dose 0 = today's draft. Each step only adds; nothing at a higher dose removes the calm tone.

| Parameter | Dose 0 | Dose 1 | Dose 2 | Dose 3 |
|---|---|---|---|---|
| Loud elements per screen (max) | 0 | 3 | 6 | 10 |
| Element classes allowed to carry energy | none | selection state, kickers | + badges, spatial pad | + headers, empty states |
| Hairline accent alpha (selection frame) | accent 1 px underline | hue 50% | hue 75% | hue 100% + 1 px inner highlight 20% |
| Corner cut (chamfer) | 0 | 0 | 3 px on selected/badge | 3-6 px on header, badge, tile |
| Skew on non-editable labels | 0 | 0 | -6 degrees badges only | -8 to -10 degrees kicker and badge |
| Kicker (caps label, +0.2 em) | none | 9.5 px on panel headers | + chips and tiles | + selected state |
| Badges per tile | 0-1 | 1 | 2 | 3 |
| Halftone alpha / pitch | 0 | 0 | 6% / 4 px, header strips only | 6-12% / 3-5 px, headers and empty areas |
| Hue-shifted offset shadow | none | none | none | 1 px on selected badge |
| Selection animation | instant | 90 ms ease-out | 120 ms | 160 ms |
| Looping ambient animation (count) | 0 | 0 | 0 | at most 1, 8% alpha, paused when pointer is in the work area |
| Saturated non-family pixels (% of window) | 0 | at most 0.5% | at most 2% | at most 4% |
| Selected label size step | none | none | 11 to 12 px | 11 to 13 px, reserved cell |
| Spatial pad grid / ticks | grey | grey | + 1 px axis hue ticks | + hue cursor ring, cut corners |

## 8. Three options (each an ADDITION to the calm tone; no recommendation)

### Option 1: "Selection and kicker" (Dose 1)
(a) Added: kicker labels (9.5 px caps) on panel headers; the selected state in choice controls gets a 1 px hue hairline plus 90 ms ease-out; the 6 relation-family chips and the 3 mode plates (section 4) in graphic form; the spatial pad in grey with axis letters.
(b) Budget: max 3 loud elements per screen; classes: selection state, kickers. Must stay quiet: work area, numeric fields, long lists, timeline body (all at dose 0 forever).
(c) Rendering: 1 px hairlines, 9.5 px kickers, 14 px pictograms; no fills, no shadows, no pattern.
(d) Dose values: column "Dose 1" in section 7.
Sources: Persona (selected item loudest; white lead line; level cue), Splatoon (one channel ranks), OP-1 (mapping of mark to control).

### Option 2: "Cut corners and numbered kickers" (Dose 2)
(a) Added to Option 1: 3 px chamfers on selected chip/tile/badge; badges up to 2 per tile (state, count) with -6 degree skew; numbered kickers (01, 02) and a lead hairline across panel tops; selected label 12 px; halftone strip (6% at 4 px) on panel headers; spatial pad gets hue ticks on its axes and the 4 corner L-ticks; easing and effect tiles get kicker plus badge.
(b) Budget: max 6 loud elements per screen; classes: selection, kickers, badges, spatial pad. Quiet as above, plus no halftone below header strips.
(c) Rendering: chamfer as a 1 px stroked path, badge frame 1 px, halftone pre-rendered tile, skew via a text-bearing container only for non-editable labels.
(d) Dose values: column "Dose 2".
Sources: Persona (angular cut, hierarchy cue by angle, per-screen identity), CoroCoro (pack here / open there; small badges and kickers as the small type class), Splatoon (saturation budget), halftone parameters (master.dev).

### Option 3: "Header energy and ambient motion" (Dose 3)
(a) Added to Option 2: chamfers 3-6 px on headers; kicker with figure/ground swap on one word; hue-shifted 1 px offset shadow on the selected badge; halftone 6-12% / 3-5 px on empty areas and headers; 3 badges per tile; a single looping ambient mark at 8% alpha that stops when the pointer is in the work area; selected label 13 px in a reserved cell; the pad gets a hue cursor ring and cut corners; rotation-dial detent flash 40-60 ms.
(b) Budget: max 10 loud elements per screen; classes add headers and empty states. Quiet list unchanged. At most 4% of window pixels in non-family saturation.
(c) Rendering: all of the above at 1 px; no thicker strokes; hue only from six families plus accent.
(d) Dose values: column "Dose 3".
Sources: Persona (cut-out type tricks, layered motion speeds, one colour dominance), CoroCoro (stacked edges reduced to 1 px, offset shadow), Splatoon (hue-shifted shadows), Pokemon hexagon recreation (grid alpha recipe).

## 9. How to tell it has gone too far (checkable)

Counts and areas
- More than the dose's loud-element budget on any one screen (3 / 6 / 10).
- More than one distinct kind of ornament within 100 x 100 px of the work area (the work area should have zero).
- Saturated non-family pixels above 0.5 / 2 / 4% of the window (measure on a screenshot).
- A badge count above 1 / 2 / 3 per tile, or more than 3 hues inside one panel.
- Any chamfer, skew, halftone or kicker inside the Stage, timeline body, numeric fields or long lists.
Contrast and legibility
- Any information text below 4.5:1 against its worst local pixel (3:1 for large text and component frames, WCAG 1.4.3 / 1.4.11); any text under 10 px that carries information (the lab's rule).
- A pattern or tint of any alpha under text a user reads or edits.
- Greyscale test fails: the selected item is not the first thing seen when the screen is turned to grey (Splatoon test).
- Elements that blend into each other because of similar lightness (the Persona pause-menu complaint).
Motion
- Any selection animation above 160 ms, or any entrance animation that delays an action.
- More than one thing moving at once; any loop that keeps running while the pointer is in the work area (Persona complaints: constant movement, eye strain).
- Hover moves geometry or changes size (hover may only preview).
- Animation not skippable under key repeat or with a reduced-motion setting.
Taste signals [inferred]
- You can no longer name the one focal item of a panel in a second.
- Two panels differ in decoration style (loses the "repeated treatment" unity).
- A new hue appears that is not a family or the accent.
- Anyone asks to turn it off for long sessions: Dose must have an off switch (Dose 0) and a per-panel override.

## 10. What to build first (study of 8 widgets, one knob: Dose 0..3)

In the real Flutter window, one global Dose slider (0-3) plus the raw parameters of section 7 as overrides. Each widget is shown at all four doses side by side:
1. Choice chips row: relation family (6) with kicker and selected state; keyboard contract live (arrows, 1-6, Esc revert).
2. Mode plates: 2D / 2.5D / 3D hairline plates.
3. Easing thumbnails: 10 curves, hover dot, selection.
4. Browser tile: thumbnail, name, badges 0-3, selected state.
5. Panel header strip: kicker, number, lead line, halftone strip.
6. Spatial pad: X/Y pad plus Z rail, rotation dial, scale box; drag with a readout; grey vs hue ticks.
7. Plain controls control group: numeric field, long list (30 rows), blend-mode list: must look identical at every dose (the quiet control).
8. Too-far meter: prints the section 9 counts for the current screen: loud elements, saturated-pixel %, worst text contrast, kinds of motion running.
Acceptance: widget 7 is pixel-identical across doses; widget 8 reports no sign above its dose limit at the dose chosen.
