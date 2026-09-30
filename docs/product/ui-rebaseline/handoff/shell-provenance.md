# Assembled shell: seat provenance

Fixture: `motolii/ui/lib/proto_hf/main_shell.dart` (fixed 1536 x 1024, fake data, no Rust). Capture: `handoff/shell-6.png`.

Rule: each seat uses the latest accepted component. No fallback to the Phase B `proto/` set.

| Seat | Source component | Latest accepted evidence | In the shell |
|---|---|---|---|
| Top bar | `hf.dart` `top()` | `build-candidate-v5.png` (hf.dart last edited 10:25) | yes, at its reference rectangle (0, 0, 1536, 62) |
| Stage | `hf.dart` `stage()`, art `proto/stage_hf.png` | `build-candidate-v5.png`, `crop-5-stage-chrome.png` | yes, at (344, 62, 779, 632) |
| Timeline | `hf.dart` `timeline()` with `_TlPaint`, `_TlMarks` | `polish4-timeline-zoom-concept-before-after.png`, `build-candidate-v5.png`, handoff 15.11 | yes, at (344, 703, 1178, 291) |
| Browser | `bp/*` via `main_browser.dart` `leaf()` | `browser-panels-things.png` | yes, at (10, 62, 324, 953) |
| Inspector | `insp/*`: generic `panel.dart`, `transform.dart`, `layout.dart`, `camera.dart` | `insp-7.png`, `insp-p2-1.png`, `tf-3.png`, `lay-3.png`, `cam-4.png` | yes, right seat (1135, 62, 387, 631) |
| Desk | `desk/*` | `desk-final4.png` | yes, as an alternate in the right seat |

## Superseded, not used

- `proto/stage.dart`, `proto/timeline.dart`, `proto/top_bar.dart`, `proto/tokens.dart` (Phase B scaffold, edited 07:28 to 07:30, art `stage_ref.png`). The first assembled capture (`shell-2` to `shell-5`) used these by mistake and is not a valid whole-window comparison.
- `hf.dart` `browser()` and `inspector()` (early painted candidates). Replaced by `bp/*` and `insp/*`.
- Production `lib/panels/*` (Classic panels restyled in Phase C). Separate lane, not part of this fixture.

## Open, not decided here

- The reference has the Timeline spanning under both Stage and Inspector, and it has no Desk. The fixture keeps the reference rectangles and shows a Desk in the Inspector seat as a placeholder. Where the Desk lives is undecided.
- `hf.dart` was edited (stage art swap) after the `build-candidate-v5` capture. The shell uses the file as it is now.
- Geometry against the concept image has not been compared yet.
