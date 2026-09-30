# Motolii design brief (short)

Goal: make Motolii a production UI you want to touch, want to own, and can use for hours.

**Density from Ableton. Typesetting from Swiss. Fun from a toybox.**

## Housing: Swiss and Ableton

A dense production tool, kept in order by grid, alignment, type and rules. The dark casing itself is quiet.

## Contents: toybox

Open Create and you want to place something. Open Relations and you want to try one. Colour makes you want to touch colour. Effects make you want to touch a phenomenon.

Think: what if Ableton's browser were a toybox? A strict category list and search on the left. On the right, Text, Shape, Cube, Scatter, Along Path, Blur, Color, each a small toy you want to pick up.

Not a SaaS card gallery: no wide empty space, no giant cards, no rounding on everything, no piled-up gradients.

## Controls have faces, but not all of them

Save, search, number fields and toggles stay ordinary. Only creative concepts show their phenomenon.

## Timeline: Ableton-like

It is where you read the structure of the work, so it can be colourful. Colour is an aid for telling neighbours apart, not meaning.

- Each layer or relation gets a colour from a palette, automatically.
- It only has to be stable inside that timeline.
- Scatter is not always pink. Stagger is not always blue.
- The Browser's colour does not have to match the Timeline's colour.

## Stage: the work itself

The Stage does not win by greying the UI. It wins by kind: photographic, irregular, high-frequency against graphic, geometric, organised.

## Judgement

Make it and look. Does "I want to use this" appear? Compare side by side, squint, and use numbers only when the eye cannot explain a difference.

## Workspace is kept (verified in the old code)

The old UI already has the AE-style workspace. Do not redesign it.

- `initialDock()` (`workspace/layout.dart:146`) stacks Create, Media, Effects, Colors and Files as five independent panels in one dock leaf. They are tabs only because they share a place today. The Stage leaf stacks Stage and Camera.
- A tab is a `Draggable<String>` and can be moved to another dock.
- Right-click on a tab offers Detach and Close. `openPanelWindow` (`app/editor_window.dart:336`) detaches a panel into its own OS window.
- When tabs do not fit, only the active tab keeps its word and the rest become their icon, named on hover (`workspace_view.dart:200`). This already fits the dense direction.

Consequence for the Concept: its horizontal tab strip in the Browser (Objects, Relations, Effects, Media) was read as in-panel navigation. It is the visible result of panels sharing a dock, not a page switcher. A panel must stand on its own when detached, and a tab strip only appears when panels are stacked.

What to polish: each panel's inside, and the tab and panel chrome. Not the workspace structure.
