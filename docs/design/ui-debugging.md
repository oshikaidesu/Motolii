# UI Debugging

Click a thing in the running Motolii window, see where its size comes from, change the number, see the result, undo, keep or discard.
Debug builds only: it uses the Dart VM service that `flutter run` opens and Flutter's own widget Inspector; a release build has none of
it (`kDebugMode` in `lib/main.dart`).

## Design Mode (in the window)

```bash
scripts/motolii-ui.sh design [document.rrd]     # the dev window with Design Mode on; hot reload; Ctrl-C in this terminal ends it
```

Click what looks too big. Flutter's Select Widget overlay marks it, and a small HUD lists what decides its size, for the clicked widget and
the nearest ancestors that set geometry:

| row | meaning |
|---|---|
| `TOKEN 13×` | a canonical `Surface` / `Dn` token (13 uses): changing it changes every place that uses it |
| `EXCEPTION` | `Surface.px(7)`: a registered custom dimension, local to that line |
| `RAW` | a number written in the source |
| `DERIVED` | what Flutter computed (size, constraints, `flex 3 of 3 : 2` in a Row): not editable, change what causes it |
| `NAMED` / `CONST` | a colour or style; a `static const` (needs a hot restart, so read-only here) |

The active value is marked `▸`, an edited one `●`. The orange box in the window is the active row's widget; for a `padding` row the
padding area is tinted.

| key | |
|---|---|
| click / `Tab`, `Shift+Tab` | choose the value (the HUD rows can also be dragged sideways) |
| mouse wheel, `↑` `↓` | ±1 (`Shift` 0.1, `Alt` 5) |
| `A` | BEFORE (the session's start) ↔ CURRENT |
| `Z` or `Cmd+Z`, `Shift+Z` | undo, redo |
| `R` | this value back as it was at the session's start |
| `Esc` | cancel what was just done to this value |
| `C` | the session's changes (`24 → 19`, per line) |
| `[` `]` | UI Scale −1 / +1 (`Shift` 10; not saved) |
| `F2` or `Cmd+Option+D` | Design Mode off (and on again); with changes it asks `KEEP` (Enter) or `DISCARD` (Esc) first |

The HUD's buttons (`A/B`, `UNDO`, `REDO`, `RESET`, `CHANGES`, `DONE`) do the same with the mouse. While Design Mode is on, a click
selects instead of acting and the keys above are the HUD's; off, Motolii is itself again.

A change is written into the source (a token's declaration in `lib/theme/metrics.dart`, or the number in the widget's call) and the
existing hot reload (`SIGUSR1` to the pid `dev` keeps in `~/.local/state/motolii-stage5/`) puts it on screen. KEEP leaves the diff in
the working tree for you to read and commit; DISCARD writes the session's start back. If a reload does not land (a compile error
somewhere), the last change is put back, the HUD says so and Design Mode goes on.

## Command line

The same ground without the window, for scripts and for a DevTools session (`scripts/motolii-ui.sh inspect`; `pick` waits for one
click, `watch` for each, `show` prints what is selected, `tokens [text]`, `set Surface.chromeRow 19`, `edit lib/x.dart:174 8=4`,
`changed`, `reset`, `scale 78`). Flutter DevTools works on the same session (the link `flutter run` prints; Select Widget mode there
selects in the same window).

## Not covered

- Flutter's Property Editor edits constructor arguments from the IDE (VS Code, Android Studio); it is not used here: the HUD does the
  same for the numbers in Motolii's own widgets.
- Dragging the box's edges or the gap between widgets: values are changed by wheel and keys.
- Computed layout is not re-derived: what Flutter reports is shown, the rest is not.
