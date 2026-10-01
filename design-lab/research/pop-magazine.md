# Pop magazine direction: loud through fine detail

Supersedes `research/editorial.md` (wrong direction). Date 2026-10-02.
Tags: `[seen]` = in a page I opened. `[inferred]` = my reasoning, not in a source. `[proposal]` = a value I chose to test, not a measurement.

Honest limit up front: no source I could open gives measured numbers for a cover (palette size, line px, dot pitch). I read text about covers; I did not get to measure a cover image. Every number in sections 4 and 6 is therefore `[proposal]` or from a tech page, not from a magazine.
A second limit that matters for the owner: the CoroCoro sources describe loudness built from HEAVY outlines ("4 layers of outline"). The owner's condition (fine detail, not weight) is not what those sources describe. Section 3 keeps the mechanisms and drops the weight.

## 1. Sources

Opened and read (via fetch; summaries by the fetch tool, so short):
- corocoro-news.jp, interview with CoroCoro cover designer Sasaki (https://corocoro-news.jp/news/36121/). Read twice with different questions.
- note.com HIGH-FIVE, "what I learned from the CoroCoro cover" (https://note.com/highfive_creek/n/n2c1ff8434a04).
- Togetter, readers compare the usual cover with the one issue (Aug 2025) that a different designer did (https://togetter.com/li/2576944).
- bijutsutecho.com, history of Jump's lettering (https://bijutsutecho.com/magazine/insight/26406). Little about covers; about logo/lettering practice.
- sendenkaigi.com, Sasaki interview (https://www.sendenkaigi.com/marketing/media/sendenkaigi/024358/). Paywalled; only the lead was readable.
- nlab.itmedia.co.jp Guinness article: only "packed densely" confirmed; no specifics.
- Persona Central, Persona 5 UI interview (https://personacentral.com/persona-5-interview-ui-design-sound-music/).
- Splatoon UI, Japanese reports of the Nintendo talk "UI Crunch #13": careerhack.en-japan.com/report/detail/965, mazco.hatenablog.jp/entry/2018/05/03/220000, goodpatch.com/blog/uicrunch-13 (the Goodpatch page gave no numbers).
- geechristine.wordpress.com Splatoon UX: no visual specifics.
- teenage.engineering OP-1 layout guide (https://teenage.engineering/guides/op-1/original/layout).
- Halftone in CSS: blog.master.dev "Pure CSS halftone in 3 declarations" (via redirect from frontendmasters.com).
- W3C Understanding WCAG 1.4.3 (https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html).
- Rive blog "visually accessible designs": about authored content, not the editor's own UI. Little use.
- Search-result snippets only (not opened as pages): Playdate help/SDK pages (Roobert 20/24 Medium, 400x240 1-bit), Ableton manual theme/colour settings, Famitsu cover facts (Necky the fox on odd issues, airbrush illustrator), Rive panel redesign.

Could NOT open (HTTP 403 or certificate error): Behance OP-1 UI case study; medium.com haiiro-io "How Nintendo designed Switch and Splatoon"; iwataasks.nintendo.com Splatoon page (certificate).
Not researched beyond a search that returned nothing useful: Famitsu / Nintendo Dream / CD Journal layout analyses (no design-analysis page found), Weekly Shonen Magazine/Sunday cover analyses, Cavalry, FigJam, Notion illustrations, Pokemon/Digimon status screens, Linear. I make no claims about them.

## 2. What was actually seen (numbers and specifics)

CoroCoro cover (all from Japanese text sources):
- Aim: a cover so "hot" you let go of it; the designer's words: 「一目で熱いとわかる」(corocoro-news). [seen]
- Small, medium and large type each have a role; he calls it a "nice body" balance (ボンッ!キュッ!バンッ!). Rule he states: 「ここを詰めたら、ここを開ける」(pack here, open there). [seen]
- Headline type and visuals are bigger than at launch (1977, Doraemon-centred, with white space) because the information grew. [seen]
- Custom typefaces from a type house (タナカデザイン); layout is hand-specified for the printer; two days per cover from an editor's detailed rough. [seen]
- The magazine's own logo must never be hidden, or children cannot tell what it is. [seen]
- 20+ serial titles/characters on one A5 cover; the designer works so that each part does not overlap (sendenkaigi lead). [seen]
- Title treatment named in the HIGH-FIVE note and in a search summary: outlines stacked in 4 steps, gradients, a shifted copy as a shadow, depth via text placement, "rainbow" text, many colours, no alignment rules. [seen, secondhand, the note author calls priority "unclear"]
- Readers on the Aug 2025 off-style cover: the usual one is "text and colour packed and scattered"; the new one has "information organised, colours calmer"; one child did not recognise it as CoroCoro because the clutter was missing. [seen] This is the strongest evidence that the loudness is the identity (recognition), not noise.
- Not found anywhere: number of colours per cover, outline px, grid description.

Splatoon UI (Nintendo talk, via careerhack and mazco):
- "Raise saturation of what must stand out; keep everything else as low as possible." Icons high chroma, modal/base windows low chroma. [seen]
- Shadows are not black: a yellow shadow uses orange, a pink one uses purple (hue shifted, saturation kept). [seen]
- Checked by converting the screen to greyscale and by squinting. [seen]
- A custom typeface (bold, liquid lines); in Splatoon 2 a thinner, more angular version was made because information density grew. [seen] Relevant: the density answer was a finer face, not a smaller size.
- Icon shapes come from a word first (squid, sport), meant to be recognisable without colour. [seen]
- Background information was removed on the shop screen (posters deleted) to leave room for UI. [seen]

Persona 5 UI (Persona Central):
- One dominant colour (crimson red) with black and white text; the designer says he prefers solid fills over gradations. [seen] This contradicts "use gradients" (section 4); it is a solid-shape source.
- Menu entries contain animation, preloaded to avoid lag. [seen]
- The page's claim about readability trade-offs is the fetch tool's guess; ignored.

Teenage Engineering OP-1 (TE guide, search snippets):
- Four coloured encoders; a coloured graphic or text on the screen says which encoder changes it ("a green graphical element ... green encoder"). [seen]
- Graphics animate in real time with the modulation (snippet). Vector GUI, 60 fps, AMOLED (snippet). [seen]
- Behance case study with colour/line numbers: not opened.

Playdate (snippet only): system UI set in Roobert 20 Medium, headings Roobert 24 Medium, 400x240 1-bit; designer avoids "grids of buttons". Generous size, not fine. [seen in snippet]

Ableton (snippet only): colours are user-themed; "Reduced Automatic Colors" lowers the number of auto-assigned colours; grid opacity and colour intensity are user settings. Shows that "limit the colours" is a built-in knob in a colourful pro tool. [seen in snippet]

Halftone in software (master.dev): radial-gradient dot of two greys (#777 to #fff) multiplied with a lightness map, then `contrast(16)` for small dots and 80+ for large; dots 1em to 9em in the demos; large dots need blur then contrast to smooth edges; dots are not perfectly round when the map moves; Firefox had blending bugs. Moire note (search snippet from other pages): offset multi-colour screens by 30 degrees. [seen]

WCAG 1.4.3 (W3C): 4.5:1 normal text, 3:1 large text (18pt, or 14pt bold); logotypes exempt; decorative text exempt; "text that is part of a picture" with other content is a grey area. 1.4.11 (snippet): 3:1 for UI component boundaries and meaningful graphics. [seen] No guidance on measuring text over patterns (snippet statement).

## 3. Mechanisms that keep a loud layout coherent (ranked)

1. Hierarchy by a few type/size classes with a deliberate size jump (CoroCoro "small, medium, large"; Splatoon's thinner face when density rose). Evidence: corocoro-news interview. Confidence: high that this is how the designer describes it.
2. "Pack here, open there": a dense zone is paired with a looser zone. Evidence: Sasaki quote; Splatoon removing posters on the shop screen. [seen]
3. Saturation budget: only what matters is saturated; containers are low chroma. Evidence: Splatoon talk, explicit rule. For Motolii this is the one rule with a direct procedure (greyscale check). [seen]
4. A fixed identity anchor that is never covered (the CoroCoro logo; Persona's single red; the OP-1 colour-to-encoder map). Evidence: interview, Persona Central, TE guide. [seen]
5. Repeated treatment, not repeated content: every title gets the same family of tricks (stacked outline, gradient, offset shadow) so variety reads as one voice. Evidence: HIGH-FIVE note (secondhand). [seen, weak] Whether each element repeats the SAME tricks is [inferred].
6. Shadows and depth tinted with hue, not black (Splatoon). Keeps colour clean when many colours coexist. [seen]
7. Shapes recognisable without colour (Splatoon icons), so colour can vary freely. [seen]
8. Non-overlap discipline: with 20+ items the designer keeps parts from overlapping while the placement looks random (sendenkaigi lead). Reading order is by size and proximity [inferred]; the HIGH-FIVE author calls the order "unclear" and intended as discovery.
9. Loudness is recognition: readers identify the product by the clutter (Togetter). This argues the chrome may be loud as a brand, as long as the logo/anchor stays visible. [seen]

What I did not find: a hidden grid (no source states one; "no alignment rules" is the only statement) and a palette-per-cover rule. Do not assume them.

## 4. Techniques to be loud through fine detail (parameters are `[proposal]` unless noted)

Starting ranges to put on knobs (a UI of 1x density, panel text 10-11 px):
- Hairline colour frame: 1 px (device pixel snapped), family hue at 60-100% alpha; optional second inner 1 px at white 8-16% (the lab already uses white 16% / 9% hairlines).
- Inner highlight: 1 px top edge, white 10-24% alpha.
- Offset shadow (printed misregistration look): same hue as the shape, hue shifted 20-40 degrees, 1-2 px offset, no blur (Splatoon rule: not black, saturation kept).
- Halftone: dot pitch 3-6 px (2 px is texture noise; 8+ px starts to read as a heavy pattern); dot radius 0.5-1.5 px or ramp 0-40% coverage; angle 45 degrees (single colour) or 15/75 degrees (two colours; the 30 degree separation avoids moire); alpha 6-18% on a header or tile, 0 under editable text. Render as a pre-tiled bitmap or a shader, not a per-frame vector fill [inferred, for performance; the CSS technique itself has dot-roundness and blending caveats].
- Gradient: 2-stop, same hue, lightness delta 6-14 points, over the full height of a header or badge; angled 90 degrees (vertical). Persona 5's designer prefers solid fills, so keep a solid variant of each.
- Badges: 14-18 px tall pill or circle, 8-9.5 px text, 1 px frame, 1 colour from the panel's palette; max 1-3 per tile.
- Micro-type: Latin small caps 8-9.5 px with +0.12 to +0.2 em tracking for labels. Below 10 px conflicts with the lab's own "no text under 10 px" rule (see 5); keep decorative micro-type decorative (WCAG exempts purely decorative text) and keep information at 10+ px [inferred].
- Corner ornaments: 3-5 px L-ticks, cross-marks or a 4-point sparkle at a tile corner, 1 px, 1 per tile; the same ornament everywhere (repetition for unity).
- Texture overlay (paper grain/noise): 2-5% alpha, static, never over text.
- Stacked fine outline for titles (the CoroCoro 4-step idea in a fine form): 3 concentric 0.5-1 px lines in 3 hues, total reach under 3 px, only on display-size headings (14+ px) [inferred adaptation; the source uses thick lines].
- Tinted micro-illustration: 16-24 px line drawing, 1 px stroke, 2 hues, one per panel header or empty state.

Legibility and calming rules:
- Text 4.5:1 against the actual local background (take the worst pixel of any pattern under it); large text 3:1; badge frames and icons 3:1 (WCAG 1.4.3 / 1.4.11, seen).
- Patterns stay under 18% alpha and are kept out of any region that holds text or a field the user edits [inferred].
- Greyscale test: convert the screen to grey; the hierarchy must still read (Splatoon method, seen).
- Per panel: one dominant hue plus at most 2 supporting hues [proposal]; saturation high only on the 1-3 focal items (Splatoon rule, seen).
- Density target by count: about 8-14 visually distinct marks per 100x100 px of chrome [proposal, no source], with at least one empty 24 px band per 150 px of height ("pack here, open there").

## 5. Draft (DESIGN.md + tokens.dart) against the pop direction

| Draft rule | Verdict | Why |
|---|---|---|
| Dark, dense, the work brighter than the chrome (s1) | AGREES (partly) | Pop-density agrees. "Chrome recedes" contradicts "every element loud". Resolve by zone (section 6 and 7). |
| Colour only as relation-family marks, never decoration (s1, s2) | CONTRADICTS | Pop needs colour as personality on chrome, labels, tiles. Splatoon's rule (colour on what matters, low chroma elsewhere) is a middle path. |
| One accent (selected state: 1px accent) | CONTRADICTS | Several coloured elements with their own voice; the OP-1 maps several colours to several controls. |
| Tone "not glossy, not flat-corporate" (s1) | AGREES | Gradients and highlights fit. |
| Greys named by lightness, 6 families, sat/lightness tuned (s2) | AGREES | The 6 family hues are a ready palette to extend. Splatoon-style low-chroma containers = the grey scale. |
| Depth by grey level, no shadow on bars/keys/rows (s6) | CONTRADICTS (soft) | Pop uses hue-tinted offset shadows. Splatoon avoids black shadows, not shadows. |
| No drop shadow, no dark underline on bars (s4) | CONTRADICTS (soft) | Same. It protects timeline legibility; keep it for the work area. |
| Flat fills, 1px faint light edge on bars (s4) | AGREES | A hairline plus a 1px inner highlight is already the fine-detail vocabulary. |
| Hairlines 1 px, snapped to device pixels (s8) | AGREES | The core of "fine". |
| Inter 10-11 px, Menlo values, small caps 9.5 px +0.15 tracking | AGREES | Micro-typography is on the list. A single family is the weak point: the cover sources use many faces (CoroCoro) or a custom one (Splatoon) [seen]. |
| No text below 10 px, dim text floor g56 (tokens.dart `_floor`) | AGREES for information; CONTRADICTS if 8-9 px decorative micro-type is wanted | WCAG exempts decorative text only. |
| "Symbols are few" (15 glyphs), state by position/grey/word before a new mark (s4) | CONTRADICTS | Pop wants tiny ornaments, badges, sparkles, micro-illustration. Possible compromise: ornaments are decorative-only and do not add to the semantic glyph set. |
| Selected state per component class: g20 fill + 2px tick, no hue (s4) | CONTRADICTS (soft) | Pop would colour it; the 1px accent underline is already small and fine. |
| Don't add a new colour for a new feature; pick a family or stay grey (s7) | CONTRADICTS | Mirrors the "one voice per element" aim. |
| "Prefer fewer kinds of line, fewer greys, fewer emphases" (s7) | CONTRADICTS | Pop wants more kinds of fine marks. The unity then has to come from repetition (section 3, 5). |
| One style per line type, take one rule and apply everywhere (s7) | AGREES | This is the "repeated treatment" mechanism. |
| Whitespace only between rows, none inside a bar (s5) | AGREES for the work area | Matches "pack here, open there": the timeline is the pack zone. |
| Draft's `quiet` Look as default; selected underline dropped in quiet (s4) | CONTRADICTS | Default should be the loud Look, with quiet as the opt-down. [inferred] |
| Danger colour one warm red (tokens `C.danger`) | AGREES | A fixed anchor colour for meaning. |
| Row 23 px, bar 0.87, key 0.33 (s9) | AGREES | Work-area geometry should not change. |

## 6. Three distinct options (no recommendation)

All three keep the work (bars, keys, numeric fields during editing) quiet. All values `[proposal]`.

### Option A: "Sticker sheet" (Splatoon saturation rule + CoroCoro stacked labels, in fine line)
- Sources: Splatoon (saturation budget, hue-shifted shadow, greyscale test); CoroCoro (type size classes, offset shadow, stacked edges, logo-always-visible).
- Palette: the 6 family hues plus 2 more (cyan, lime) = 8 hues; chrome greys unchanged. Focal items at saturation 70-90%, lightness 55-70; containers at saturation 0-8%. Shadows: the hue shifted 25 degrees at the same saturation.
- Lines: 1 px hairline frames in hue at 70-100% alpha; stacked label edge of 2-3 hairlines (each 0.75-1 px) on 14 px+ headings only.
- Halftone: none, or 4 px pitch at 8% on the panel header strip only.
- Badges/callouts: 14-16 px pill, 9 px caps, solid hue fill with dark ink (check 4.5:1), tilted 0-3 degrees for a sticker feel (tilt on badges only, never on type that is edited). 1-3 per tile. Callout: a 1 px frame bubble with a 4 px tail.
- Energy goes: panel headers, tab labels, Browser tiles, state badges, empty states.
- Quiet: Stage, Timeline rows/bars/keys, numeric fields (no tilt, no pattern, no hue other than the family).
- Density target: 10-14 marks per 100x100 px in Browser/headers; the timeline stays at the draft's density.
- Unity by: the same pill, the same shadow offset, one saturation rule, greyscale check.

### Option B: "Screentone page" (manga print: halftone, two-colour, hairline panels)
- Sources: manga/magazine print conventions as discussed in the CoroCoro/Jump sources only in general terms (pack/open balance, custom lettering); halftone parameters from master.dev; offset-angle moire note; Persona 5 solid-fill preference for the shapes themselves.
- Palette: per-panel 2-colour print: one family hue plus a paper-ink grey, a different hue per panel (6 panels = 6 hues, only one per panel). Hue at saturation 55-80%, lightness 50-65; the second colour is a darker tone of the same hue (shifted 20 degrees).
- Lines: 1 px frames, double hairline (1 px, 2 px gap, 1 px) on panel borders; 0.75 px rule lines inside.
- Halftone: pitch 3-5 px, angle 45 degrees (15/75 degrees when two screens overlap), dot radius ramp 0-40% coverage as a vignette from each panel corner and as a gradient on headers; alpha 6-14%; texture grain 3%.
- Badges/callouts: small numbered circles 14 px with a 1 px frame, speech-style callouts for hints with a 1 px tail, speed-line ticks (1 px, 6-10 px long) at corners.
- Energy: panel backgrounds in header zones and empty regions, panel borders, section numbers, tooltips.
- Quiet: every region holding bars, keys, text fields has zero pattern.
- Density target: 8-12 marks per 100x100 px outside editing areas; one 24 px clear band per 150 px.
- Unity by: one screen angle and pitch family, one hue per panel, frame style.

### Option C: "Toy instrument" (OP-1 colour-coded controls + Playdate friendliness + Ableton colour knobs)
- Sources: Teenage Engineering (colour of on-screen graphic matches the encoder), Playdate (friendly, generous type, avoids button grids), Ableton (reduced-colours switch, colour intensity as a setting).
- Palette: 4-6 control hues only, each bound to a function class (for example Time, Value, Relation, Output); the same hue appears on the knob-like control, its label, its value badge and its timeline mark. Saturation 75-95% on the control face, 40-60% on its label tint, containers grey. Include a "reduced colours" switch (all hues to one) as a setting (Ableton precedent).
- Lines: 1 px frame; 1 px inner highlight (white 14-22%); 1 px inner shade (black 20-30%) to give the control a tiny physical edge. No thicker.
- Halftone: none on controls; a 5-6 px dot-grid at 5% on panel backgrounds as "hardware panel print".
- Badges/callouts: tiny silk-screen style labels at 8.5-9 px caps +0.18 em, stencil-like numbers at control corners (1 px frame), small arrow ticks; micro-illustrations (16-20 px line icons in hue) beside section titles.
- Energy: controls' faces, their labels, value badges, section headers.
- Quiet: the timeline bars and keys keep the family colours as already drafted; numeric fields while editing go to a plain high-contrast field (ink on grey, no tint).
- Density target: 12-16 small marks per 100x100 px on the control surface (a hardware-panel feel); the timeline unchanged.
- Unity by: the colour-to-function map (the 4th mechanism in section 3), repeated bevel edge, repeated label style.

## 7. What to build first: a study of 8 widgets with knobs

One window, each widget shown in A / B / C side by side or switched by a global selector, rendered in the real Flutter window (the lab's existing approach), not mocks. Global knobs: hue count (1-8), saturation of focal items (30-100%), container chroma (0-15%), hairline alpha, pattern alpha (0-20%), pattern pitch (2-8 px), density (number of ornaments per tile), a greyscale toggle, a contrast readout (worst-pixel ratio for every text).

1. Panel header strip: title, tab, 2 badges, corner ornament. Knobs: stacked edge count (0-3), header halftone alpha/pitch/angle, gradient delta.
2. Browser tile: thumbnail frame, name, state badge, count badge, sticker tilt (0-3 degrees). Knobs: frame colour alpha, offset-shadow hue shift (0-40 degrees) and offset (0-2 px).
3. Inspector row: label, value field, unit badge, relation mark. Must stay quiet while editing: knob = how much tint appears when idle vs focused (0 when focused).
4. Control cluster (OP-1 style): 4 knob-like controls bound to hues with labels and value badges; "reduced colours" switch.
5. Timeline slice: 4 rows of bars and keys with the draft's geometry and a loud header/ruler; verifies that the energy stops at the work.
6. Callout/tooltip and empty state with a micro-illustration (16-24 px, 1-2 hues).
7. Button and toggle set in 5 states (rest, hover, selected, disabled, danger); knobs: 1px inner highlight alpha, shadow, accent count.
8. Type specimen: the same label in 8.5/9/9.5/10/11 px, caps tracking 0.1-0.2 em, with and without a 3-line stacked fine edge; shows contrast pass/fail against the local background.

Acceptance checks the study should print (derived from sections 3 and 4): greyscale view still ranks focal items first; every information text at 4.5:1 or better against its worst local pixel; no pattern under a text field; marks per 100x100 px reported; hue count per panel reported.
