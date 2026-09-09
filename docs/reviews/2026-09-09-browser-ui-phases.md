# Browser UI phases

This numbering belongs only to the Browser UI redesign, not Stage 5 or other panels.

## Phase 1 — baseline

User named the existing UI Phase 1 on 2026-09-09. Exact source is retained by local Git tag `codex/browser-ui-phase-1`, commit `f41b6a8928593c42259675f9f7af9ec1f4434f1d`.

Rollback allowlist: `motolii/ui/lib/panels/browser.dart` only. That file was clean when tagged. The tag does not capture the pre-existing uncommitted work in Inspector, Ease, rendering, or workspace files; never restore the whole tree from this tag. Inspect intervening changes before restoring the allowed file.

## Phase 2 — fixed information in the existing grid

User direction: preserve the current gaze path and category/grid arrangement. Choose useful information in advance and show it persistently. No expanding cards or separate details panel. Increase useful information density through aligned fields and quieter decoration.

Current implementation: remove the repeated color stripe and unselected borders; keep the selection border and shared theme. Show Create descriptions, Effects categories, Media kind/format/usage or missing state, and color labels/gradient stop counts in fixed positions. Existing selection, application, drag, import reveal, sizing, and categories keep their routes.

Scope limitation: the current native asset snapshot supplies no dimensions or duration, and the effect catalog supplies only ID/name. Those fields and effect result previews are not implemented by this pass. Do not fabricate metadata or describe category labels as effect previews.

References: existing `BrowserPanel` data and interactions; shared `EditorTheme`/`EditorMetrics`; [Final Cut Pro browser views](https://support.apple.com/guide/final-cut-pro/customize-browser-views-ver7ff5e14b9/mac) for persistent visual and metadata browsing. Expansion was explicitly rejected by the user.

Validation: targeted Flutter analysis passed; existing `browser_size_test.dart` and `browser_import_reveal_test.dart` passed (3 tests). Hot reload was signalled, but the observed native window still showed Phase 1. Two same-bundle application processes were running, and computer use reported an intervening user change. Phase 2 real-window acceptance remains pending; no restart or process termination was performed.


## Phase 3 — AEViewer 2 Pro browser presentation

The user explicitly chose AEViewer 2 Pro and requested its UI reproduced, superseding the exploratory ecommerce/4:3 suggestions. Phase 2 is retained by local tag `codex/browser-ui-phase-2` (`9b2629893b44bccc77871bf45253af2167f52d04`). Restore only `motolii/ui/lib/panels/browser.dart`, after inspecting intervening edits. Other dirty work is outside this checkpoint.

Reference: [official Pro page](https://aescripts.com/aeviewer-pro/), [preview modes video](https://aescripts.com/pub/media/author/author_media/motionland/aeviewer/preview-modes.mp4), [folder screenshot](https://aescripts.com/pub/media/author/author_media/motionland/aeviewer/folder-preview-2.png). Observed directly in browser: dark compact grid; preview over a one-line caption; format badge at caption right; blue selected outline; bottom view icons, count and size slider. Folder tiles have a different preview treatment. Do not substitute a marketing illustration for actual file-grid geometry.

Implementation scope and sequence: preserve current category/grid gaze path and existing tile-count sizing; replace Phase 2 stacked metadata/action rows with the observed preview/caption/format anatomy; provide Grid/List/Thumbnails presentation switches and count at the bottom; move existing asset actions to a context menu; retain selection, multi-selection, drag-to-timeline and Document/Intent routes; analyze and run existing browser regression tests, then inspect the real window and use the new modes/menu.

This is an independent Flutter implementation of the browser presentation, not an imported AEViewer binary or source. AEViewer-only filesystem navigation, packages, scripts, font import, animated previews and audio audition are not implemented by this UI pass. Existing Motolii category tabs and Colors editor stay functional; image fallback does not establish 3D preview support. Exact full-product parity is not claimed.

Validation: targeted analysis, diff whitespace checks and 3 existing browser tests passed. macOS development build completed. A copied build with bundle ID `dev.motolii.browser-phase3` isolated observation from the old same-bundle window. Real window: four-column Media grid, format badges, blue selection, Grid/List/Thumbnails switches, context-menu Place, resulting Timeline layer and Stage image, and Edit > Undo removing the layer/image all verified. Cmd+Z did not visibly undo in this copied-build session; menu Undo passed. Existing user window was not restarted. The verification copy remains open as `Motolii Browser Phase 3`.


## Phase 3.1 — keep silhouette, soften boundaries

User accepted Phase 3 silhouette and requested relief from cramped styling. Baseline immediately before this refinement is `codex/browser-ui-phase-3` (`f7c2aabff98375776b78bdbe8201e547ab147c95`); browser.dart-only rollback scope applies, and intervening color-editor changes must be preserved when reverting this small refinement. Grid geometry, columns, preview dimensions and caption height stay unchanged. Unselected borders become transparent; the caption/tile surface joins the surrounding panel; format becomes quiet unboxed secondary text; name horizontal inset increases from 4 to 6 logical pixels. The blue selection outline stays. Existing AEViewer preview-modes reference and shared theme/metrics are the implementation references. Hot reload and real-window Media inspection verified the visual changes; no operation changes or new tests.

## Phase 3.2 — the AEViewer anatomy, made legible

Phase 4 (a working media library) was rejected: the request was the look, not new
behaviour. Phase 3.2 keeps Phase 3.1's silhouette, four columns, gaze path and
every existing route, and only changes what the tile shows.

One caption line replaces the two of Phase 2/3. The name owns the whole tile
width — no field shares the line — and sits vertically centred in a line of
`EditorMetrics.control`, so it has air above and below. The extension leaves the
name and becomes the format badge, a `micro` label on a translucent ground in the
picture's bottom-right corner; name and format therefore never compete for width.
Status marks moved onto the picture's top-left: a pale dot for "in use by a
layer", an orange warning for a missing file. The frame around a tile is now only
ever the selection (`EditorTheme.spatial`), so selection and use cannot be read
for each other. Unselected tiles keep no frame at all, and the gutter stays the
regular `s6`.

A mesh with no thumbnail used to be one `view_in_ar` icon for every file, so
`torus.obj` and `sphere.obj` were indistinguishable. `_ShapeMark` now draws the
body the file name says (sphere, torus, cube, cylinder, cone, pyramid, plane);
an unrecognised name keeps the generic icon. The name is the only evidence
available — the snapshot carries no mesh geometry — and no other metadata is
invented.

The bottom-right count now says what it counts: `40 items`, `12 of 40 shown`,
`3 of 40 selected`, with the three numbers together on hover.

Not done: folders, collections, history, waveforms, audition, favourites — all
out of scope by the user's instruction. Names longer than the tile still
ellipsize, with the full name on hover.

Validation: `flutter analyze` clean for `lib/` (one pre-existing unused import in
`test/snapshot_contract_test.dart`). `browser_tile_test.dart` (new, 4 tests) plus
`browser_size_test.dart` pass; `browser_import_reveal_test.dart` fails on
`importedAssets` before it reaches any Browser assertion — an EditorSession
regression carried in from the 2026-09-09 WIP snapshot, outside this lane. Real
window not inspected by this lane.
