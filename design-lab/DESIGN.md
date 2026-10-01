# DESIGN.md — Motolii Timeline (draft v0)

A design-system document for a coding agent, in the nine-section format of
[VoltAgent/awesome-design-md](https://github.com/voltagent/awesome-design-md).
Scope is **one panel: the Timeline** of Motolii (a dark, dense motion-graphics editor, Flutter, desktop).

How to read the values below. Each is tagged:
- `[code]` read from Motolii's source (`motolii/ui/lib/theme/metrics.dart`, `neutral.dart`, `identity.dart`, `timeline/timeline.dart`)
- `[concept]` measured on Motolii's concept art (`north-star-dark.png`, 1536 px wide; 1 art px = 1 logical px) and kept because it holds across the whole picture
- `[decided]` chosen by the owner in review on 2026-10-01/02
- `[open]` not decided. Do not invent a value: show 2-3 options and ask.

Concept art has generation noise (a line that is dark here and light there, a key that is blurred). **Never copy a detail that is inconsistent within the art.** Copy only what holds everywhere.

---

## 1. Visual Theme & Atmosphere

- Dark, quiet, dense. The work (the bars, the keys) is brighter than the chrome; the chrome recedes.
- The reference is a professional motion tool (After Effects, Ableton), not a consumer app: rows are packed, nothing is decorative.
- One idea per surface: **bars carry colour, everything else is grey.** Colour means a relation family (Scatter, Stagger, Along Path, Face, Follow, Attach); it is never used for decoration. `[code]`
- Tone words: precise, calm, a little playful in colour. Not glossy, not flat-corporate.
- **Direction (owner, 2026-10-02, corrected the same day) `[decided]`: like an entertainment magazine cover (e.g. CoroCoro Comic), NOT a calm editorial layout.** Each element has a loud personality of its own, there is a lot on the screen, it is colourful and playful and energetic. The owner's condition: that energy comes from fine detail, not from weight or thickness, and the density must feel good. (An earlier reading as Swiss/Tufte-style restraint was wrong; `research/editorial.md` followed that wrong reading and is superseded by `research/pop-magazine.md`.) The colour-discipline and 'quiet' rules written earlier in this file came from that wrong reading and are under review: see `research/pop-magazine.md`.
- **Guard (owner, 2026-10-02) `[decided]`: KEEP THE CURRENT TONE.** The pop-magazine / game-UI direction below is a restrained seasoning on top of the existing calm tone, not a redesign. The risk the owner named is ending up with an over-the-top UI; that is the failure to avoid. So: the current greys, hairlines, type scale, row density and the rule 'strength from fine detail, not weight' stay as they are; the new energy is added in small, bounded doses (a limit on how many loud elements one screen may have, and a list of where energy is allowed: headers, kickers, badges, selection highlights, the spatial pad) and never in the work area or the numeric fields. The current widgets (126 + the Inspector) are NOT to be rebuilt for this; they get touches. A 'Dose' knob (0 = today's tone, 1-3 = more seasoning) is how the owner judges it in the real window.
- **UX principle from Persona (owner, 2026-10-02) `[decided]`: turn choosing into graphics.** Persona took a choose-from-a-menu RPG and made the choosing itself graphic: the menu is not a plain list; each choice has a form of its own, the selected one is the loudest thing on screen, moving between choices has a clear animated language, and you always see where you are and what the choice will do. Motolii uses the same idea wherever the user picks something (a relation type, a blend mode, 2D/2.5D/3D, an easing, an effect, a mode): the choice is shown as a graphic you can see and aim at, not as a dropdown of words. Bounded by the Guard above: it applies to CHOICE POINTS, not to every control, and the energy stays in the selection state, not in the chrome around it.
- **Game UI (owner, 2026-10-02) `[decided]`: it should feel like a game's UI; Persona 5's menus are the named example** (sharp, high-contrast, characterful menus where every element has attitude). The owner's words: the old Motolii's good idea was putting X/Y/Z into a GUI instead of a table; its look was not stylish. So the Transform editing is a hands-on spatial GUI (a pad you drag in X/Y with Z depth, a rotation dial, scale handles) with the numbers as a secondary readout, not a table of fields.

## 2. Color Palette & Roles

Greys (named by lightness, `N`) `[code]`

| role | token | hex |
|---|---|---|
| well (darker than ground) | `N.g07` | `#131313` |
| ground | `N.g10` | `#191919` |
| row band, even | — | `#1E1E1E` `[decided]` |
| row band, odd | — | `#2A2A2A` `[decided]` |
| row line | — | `#161616` `[decided]` |
| hover / selected row | `N.g15` / `N.g20` | `#262626` / `#343434` |
| primary text | `N.g95` | `#F2F2F2` |
| muted text | `N.g56` | `#8E8E8E` |

Relation families (bar colour = family `.t`, the Timeline tone) `[code]`

| family | hex |
|---|---|
| Scatter (pink) | `#E974AB` |
| Stagger (blue) | `#4781E5` |
| Along Path (green) | `#7DD5B1` |
| Face (yellow) | `#EFCB4E` |
| Follow (orange) | `#F69260` |
| Attach (purple) | `#A889E9` |

- A bar at rest is the family colour with saturation x0.85 and lightness x0.93; hovered, saturation x0.95; selected, the colour as is. `[decided]`
- Playhead and the key under it: `#6EA6DB` (`H.playhead`). `[code]`
- Accent for a selected key: the panel's accent. `[code]`

## 3. Typography Rules

- Names: Inter 500, 11 px (`Dn.name`). Values and time: Menlo 11 px, tabular (`Dn.value`). Labels: Inter 10 px. Small caps / marks: Inter 9.5 px, +0.15 tracking. `[code]`
- Ruler labels are values: Menlo, `N.g63`, 9.5 px, placed 3 px right of their tick. `[code]`
- The line box is 1.0 x the size. Text never scales by a transform, only by the UI Scale. `[code]`

## 4. Component Stylings

**Row** `[concept]` `[decided]`
- Height 23 px. Bands alternate `#1E1E1E` / `#2A2A2A`. A 1 px `#161616` line closes each row.

**Bar** `[concept]` `[decided]`
- Height 0.87 of the row (about 20 px of 23), so neighbouring bars nearly touch (about 3 px apart). Corner radius about 2 px (0.09 of the row).
- Flat fill. A faint light line on the top and the bottom edge (white 16%). **No drop shadow, no dark underline.**
- A bar is at least 3 px wide so it can always be grabbed.

**Key (keyframe)** `[concept]` `[decided]`
- A diamond, 0.33 of the row across (about 7.6 px). Near-white fill (`#F2F2F2` at 90%), a 0.75 px edge at white 60%. **No dot in the middle.**
- The key under the playhead: filled `#6EA6DB`, 1.2 x larger, a 1.5 px white edge.
- A picked key: filled with the accent, a white edge.
- Keys on one row are joined by a **1 px white line** (70% at rest, 90% when the layer is selected). Not dark, not dashed on layer rows. `[decided]`

**Time grid** `[decided]`
- One style: a 1 px light line at every labelled tick (white 16%) and a fainter 1 px line between ticks (white 9%), about every 30 px. No dark lines, no special line at 00:00.

**Ruler** `[code]`
- Height 20 px, `N.g07`, ticks 6 px (major) and 3 px (minor), labels as in section 3.

**Selected state (decided by component class, 2026-10-02)**
- Choice controls (tabs, segmented, mode switch): a g20 pill with g95 text and a 1px accent underline. The `quiet` Look drops the underline.
- Rows and cells (list rows, layer labels, tree rows, menu rows): a g20 fill with g95 text and a 2px g95 tick on the left. No accent hue.
- Tiles and thumbnails: a 1px accent outline.
- Text never goes below 10px; dim text on a dark ground is lifted to g56 at the least, but dark ink on a light fill (the playhead head, a flag) is left alone.

**Symbols are few (owner, 2026-10-02) `[decided]`.** More symbols make the screen cluttered. The lab keeps one small, fixed set of line glyphs (`book/lib/parts/glyphs.dart`, 15 now). A new glyph needs a reason that no existing glyph, a word, or a position/colour change can carry. State is shown by position, grey level or a word before it is shown by a new mark. Motolii's 49 glyphs are NOT the target: they are folded into this small set (several into one), and some become plain words.

**The old Motolii is a counter-example, not a source (owner, 2026-10-02) `[decided]`.** Nothing of the old Motolii's UI is imported, and its UX is not copied either. It is studied only to learn what NOT to do (`research/old-motolii-complaints.md`, `research/anti-patterns.md`). This design (the lab) is the only reference for how Motolii looks and feels. What the product must be able to do (capabilities) comes from the product's purpose and from shipped precedents (After Effects, Ableton), never from how the old UI happened to do it.

## 5. Layout Principles

- Density first: the row is 23 px, the label column 180 px, the ruler 20 px. `[code]` `[decided]`
- Bars and keys sit on one baseline: the row's vertical centre.
- Whitespace is **between rows** (3 px) and nowhere else; no padding inside a bar.
- Spacing and size come from tokens (`Surface`, `Dn`), never from a number typed in the panel. A size no token names is written `Surface.px(n)` with a `// surface: <reason>`.

## 6. Depth & Elevation

- There is almost none. Depth is shown by **level of grey**, not by shadow or border: ground `#191919` < raised `#202020` < hover `#262626` < selected `#343434`.
- Only floating things (menus, tooltips) have a shadow. A bar, a key and a row never do.

## 7. Do's and Don'ts

Do
- Take one rule from a reference and apply it **everywhere**. A line style is one style.
- Show 2-3 options for an `[open]` value and let the owner choose in the real window.
- Measure in the same unit as the reference (ratio to the row height) before comparing.
- Prefer fewer kinds of line, fewer greys, fewer emphases.

Don't
- Don't copy a detail that the reference itself does not repeat (a line that is dark in one place and light in another is noise).
- Don't paint a key with a dot, a halo or a shadow to "make it readable"; make it larger or lighter instead.
- Don't add a new colour for a new feature; pick a family or stay grey.
- Don't build a tool (a gallery, a lint, a catalog) before searching the repository and the web for the one that exists.

## 8. Responsive Behavior

- Desktop only. The window can be narrow: the label column keeps 180 px, the time area scrolls and zooms.
- Zoom changes the grid step: the labelled step is the finest one with labels at least 70 px apart; the faint step is the one nearest 30 px. `[code]`
- UI Scale (50-200%) multiplies every token; hairlines snap to whole device pixels. `[code]`

## 9. Agent Prompt Guide

Quick reference
- Row 23, bar 0.87 of it, key 0.33 of it, radius 0.09 of it.
- Greys: bands `#1E1E1E` / `#2A2A2A`, line `#161616`. Bars: family colour, sat x0.85, light x0.93.
- Keys: near-white diamonds, 1 px white link line. Grid: white 16% / 9%, 1 px.

Prompts
- "Draw the Timeline rows for a 4-layer composition using this document. Use only the values tagged `[code]` or `[decided]`; for anything `[open]`, render three options side by side."
- "Compare this screenshot with `preview-dark.html` at the same scale (ratio to the row height). List only differences that appear in the whole picture, not in one spot."
- "Before adding a component, search the repository and `awesome-design-md` for an existing one. Report what you found, then propose."

Open questions (decide in the real window, not here)
- Bar colour: one factor for every hue, or per-hue (the concept's blue is less saturated, its purple more).
- Whether layer-row link lines are dashed for non-linear interpolation.
- Whether the odd/even bands apply to property rows as well as layer rows.
