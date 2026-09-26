# Visual golden observations

Status: what the Concept and its Flutter build look like today. This is a record of one surface, not the language. Rules such as "neutral things stay neutral", "no shadows" or "radius 4" describe this golden only and must not be used to derive new panels. The language is in `visual-language.md`.

One sentence: **a lightbox with the wiring exposed.** The work is lit in the middle. Everything else is a dark housing that holds it, plus a few coloured parts that are the relations acting on it.

This file holds principles, not measurements. Measurements live in `design-handoff-v1.md`.

## 1. Mass hierarchy (what the eye sees, in order)

1. The Stage artwork. The only photographic, luminous, irregular thing on screen.
2. Identity colour: Relation tiles, the open Scatter instrument, timeline bodies.
3. The playhead and the mode/state accents (play, EDIT, toggles).
4. Names and titles (white, fuller).
5. Everything technical (numbers, ruler, annotations): grey, small, subordinate.

Stage wins by information structure (photographic vs flat and geometric), not by lowering everyone else's saturation.

## 2. Colour

- Housing is one neutral ground with a few steps of grey. Panels are separated by a 1 px rule and a gap, not by fill changes. No blue tint in greys.
- Colour identifies a thing. Pink is Scatter everywhere: big tile, open instrument, small dot, timeline body. Never decoration.
- Same identity, different body per place: large flat surface in the Browser, marks and handles in the Inspector, flat bar in the Timeline. Area encodes distance: discovery large, editing medium, identification small.
- Operational colour (play, active mode, toggle, playhead) is separate from identity colour. Position and shape carry meaning too.
- Neutral things stay neutral: primitives, generators, effects, stage controls.
- Off keeps its identity dot. Disabled is 38 percent alpha, not a new grey.

## 3. Shape

- Sharp, flat, graphical. Low radius (tiles 4, keys and inputs 3, tabs and cards 2). 1 px rules. No shadows, no glass, no gradients, no floating cards.
- Toggles are capsules. Sliders are thin tracks with a small round thumb. Tiles are small boxes. Each kind of control keeps its own body.
- Glyphs are geometric but not CAD. Uneven stroke weight: thin outlines, solid masses, dots. Relation glyphs are the relation's face.

## 4. Typography

- Sans for names and labels. Mono only for machine readout: numbers, ruler, version, annotation.
- Upper-case, tracked, medium weight for navigation and section headings. Sentence case for items.
- Weight follows importance, not size alone: brand, relation title, object name and row names are fuller; parameters, sub-labels and readouts are lighter and greyer.
- A readout must never win against the button beside it.

## 5. Structure rules

- **Do not unify components. Unify identity.** One meaning may have a different body in every panel.
- **Structure = geometry. Identity = colour. Content = marks inside the body.** Colour presence never implies hierarchy.
- Timeline: Jewel Field is the only parent. Transform and the relations are parallel siblings with identical vertical metrics.
- **Density by folding, not shrinking.** The relation being edited opens into a phenomenon first (distribution, curve) with numbers second. The others fold to one line and keep their colour dot.
- Stage chrome must nearly disappear when squinting. No text on the artwork.
- Panels are resizable. Panel identity is fixed, panel size is not. Fold before shrink, scroll before crushing.

## 6. What breaks it

- Everything the same visual weight.
- Colour used as accent or ornament.
- One generic button for every job.
- Glyphs or keys that read as white blobs beside the Timeline's small diamonds.
- Numbers before the phenomenon.
- Cards inside cards, large whitespace, rounded SaaS look, Material defaults.
- Reading noise in the generated reference (local shading, broken lines, per-item fills) as intent.

## 7. How to judge

Look first, same scale, side by side. Squint and greyscale. Compare silhouette and visual mass before pixels. Measure only when the eye cannot explain the difference. A number that matches while the picture differs means the number is wrong.

Desirability test: would you rather own this than the reference, and can a professional work in it for hours?
