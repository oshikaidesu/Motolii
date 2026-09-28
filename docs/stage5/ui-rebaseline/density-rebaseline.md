# UI density rebaseline (2026-09-28)

Authority split: Product Home owns composition and hierarchy; Product Direction owns semantics and character; the Rive
editor is the external benchmark for density, control sizing, spacing and information economy (not its look).

## Rive vs Motolii (logical px)

Rive values are APPROX, read off official docs screenshots (2x captures, the editor itself needs a login, which was
not used). Motolii values are from the code (`hf/**`, `live_hf/**`) before this change.

| Item | Rive (APPROX) | Motolii before | Owner | Motolii now |
|---|---|---|---|---|
| Application bar | ~43 | 62 (1536 px reference face, clipped under ~1300 px) | live_hf/shell.dart, hf/shell/top.dart | 44, folds (hf/shell/top_bar.dart) |
| Tabs / panel header | 29-30 | tabs 30; Browser header 52/40/34; desk header 58/44/40 | hf/bp/common.dart, hf/bp/shell.dart, hf/desk/common.dart | 30 (36 when it names its panel) |
| Inspector row pitch | 31 (label and value inline) | 55 (label 15 + 3 + well 30 + 7), hero 59 | hf/insp/panel.dart, toys.dart | 46, hero 50 (label-above kept) |
| Field / well | 28-29 boxed | 30, hero 34 | hf/insp/toys.dart | 26, hero 30 |
| Button | 30 | 30 (HfKey) | hf/shell/sheet.dart | 26 (HfAction) |
| Menu row | 28 | 28 | hf/shell/menu.dart | 28 (owner: UiMetrics.menuRow) |
| Text | 12-13 | 11-13 | H.s / sans | unchanged |
| Timeline row | 35 (old UI) | 23 | hf/shell/timeline.dart | unchanged (already denser) |
| Transient task | small non-modal floating panel; choices then one primary | Export 420 px centred modal, dims the Stage; Composition 360x380 modal | hf/shell/sheet.dart | popover under its control, no backdrop |

Motolii's text, controls and menus already sat at the benchmark; the excess was the application bar, headers, the
Inspector pitch and the transient sheets. So the new 100 % keeps the geometry base at 1.0 and rebaselines those.

## The density system

- `hf/metrics.dart` — `UiMetrics`: chrome row 30, named header 36, top bar 44, control 26 (hero 30), least hit 24,
  menu row 28, Inspector cell rhythm (13 / 2 / 5), pads. Colours and faces keep their owners.
- `live_hf/ui_scale.dart` — `LiveUiScale`: the user's UI size, 70-130 % in whole-percent steps, kept as `hfScale` in
  the settings; `base` is the new 100 %. Applied at `WidgetsApp.builder` through Classic's `EditorScale` +
  `EditorScaledViewport`, so menus and sheets in the overlay scale too. Cmd+Option+= / - / 0 (Cmd+= / - stay the
  Stage zoom); a short readout says the size.
- Not scaled: the document, the Stage's picture (its native window is asked for at device ratio x UI scale, so one
  texel per device pixel), camera, transforms, time zoom, export.
- "Make all of Motolii 5 % smaller" is `LiveUiScale.base` (for everyone) or the user's own 95 %.

## Visual size vs hit size

Keys stay 24 px or more to the pointer (`UiMetrics.hit`) whatever they draw; menu rows 28; the popover's close is a 24
px target around a 10 px glyph; the Timeline's Split and Marker became real 26 px keys; the Inspector well is the
whole drag target at 26 px.

## Grammar

Shared primitives in `hf/shell/sheet.dart`: `HfAction` (primary / secondary / destructive), `HfChoice` (segmented
selection, never the action's colour), `HfFormRow`, `HfFact` (read-only, no box), `showHfPopover` (under its control,
no backdrop, Esc / Enter). Menus (`showHfMenu`) light the line under the pointer, mark the current value with a check,
and draw titles and facts as information. Dead affordances (strip ✕, header grid key, desk ⋯) removed; Pin drawn
quiet; Fit wired to the Stage's Fit.

## Decision Queue (observation → precedents → options → consequences)

1. **Seat proportions vs UI size.** The Dock splits by the reference frame's weights, so a smaller UI gives more rows,
   not more Stage; the Stage grows only when fixed chrome shrinks. Precedent: Product Home's weights "are the
   reference frame's own rectangles". Options: keep weights; or keep the Timeline seat's px height (291 at 100 %) as
   the UI scales. Consequence: the second gives the Stage the height the UI gives up.
2. **EXPORT as a mode tab.** It is an action that opens a task, drawn as a mode next to EDIT/PLAY (PLAY is really the
   playback toggle). Precedent: Rive puts Export in a file menu and Publish as the one primary. Options: keep; a
   separate Export key; a file menu.
3. **Pointer route to Composition** (only Cmd+Option+K today) — queued earlier (C#2); the popover is ready for any
   route.
4. **Timeline face.** Drawn in the 1536x1024 reference frame at a fixed 1177 px and 8 rows: it does not follow its
   seat's width or height. A responsive face is an architectural step (its geometry is also in timeline_core).
5. **Toggle and choice styles.** The audit found four switch drawings and three "selected" looks across surfaces;
   unifying them is a visual decision on which one is Motolii's.
6. **UI Scale surface.** Keys and a readout exist; a visible control needs the Settings route (queued).
