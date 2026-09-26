# Flutter semantic-ownership audit (2026-09-26, working checklist)

Question: if Flutter were replaced tomorrow, which product semantics would have to be reverse-engineered from Dart?
Stays in Flutter: dock and window state, focus, hover, peek, gesture interpretation, coalescing of previews, painting, scale, keyboard maps.
Belongs behind the host: what a value, a layer, a key, a curve or a user's collection means.

| Where | Semantics that live in Dart | Disposition |
|---|---|---|
| Ease, Sequence | how a curve becomes a delay per layer (layer i of N through the curve, the whole spread, six frames a step) | **moved**: `sequence` and `previewSequence` take the layers and the curve; `editor::sequence_delays` decides. `ghosts` is still accepted. Rust test `a_sequence_given_only_a_curve_spreads_the_delay_itself` |
| Ease | which key intervals a selection means (`_deriveSegments`), when the desk may apply (`_canApply`) | host status field `easeIntervals` needed |
| Transform, Layout, effects | with several layers selected a preview is turned into one absolute value per layer (each keeps its offset; a typed value is absolute per axis; opacity is clamped; Keep the shape links the axes) | host op that takes the gesture (`previewProperties` with a relative form) needed |
| Blend | which layers a mode applies to (selected, unlocked, not a camera) | host status field `blendTargets` needed |
| Depth | how a drag on the plan becomes a camera orbit and distance (yaw from the pointer, distance over cos pitch) | host op `depthDrag` needed. The layer side already uses the basis the host sends |
| Desk | which desk follows which selection (keys to Ease, a camera to Depth, several layers to Ease as a Sequence, a blend change to Blend) | host status field `deskFollows` needed |
| Notes | block ids minted in Dart, card defaults and clamps, the label and range of a reference, the carry-over of earlier notes | host op `notes` should mint ids and defaults; `legacy` import belongs to the host |
| Browser | the list of what Create can make (kind, name, detail, rail), the meaning of favorites (collection 1) and of the digit keys, the user's tag, range, recent and search rows | host status field `createKinds` needed; the user rows are workspace settings, documented in `new-shell-audit.md` and shared with Classic |
| Shell | dirty state, the document name, confirm on close | shared session functions; host should say dirty |

Rule for the next edit: a new product rule goes to the host first; Dart gets a field to show and an op to call.
