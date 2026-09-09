# Visual selection — gradient and typography

2026-09-09 user approval: replace the gradient/text control presentation with previews that predict the authored result. Color selection opens the existing Colors panel, never a popup.

1. Reuse `engine::text::rasterize_text_document` and `doc::vector::render` for small read-only samples. Follow the existing `easeModel` read-only request route and thumbnail PNG encoding. Generate visible font rows only; bound caches and reject stale row results. Font samples use selected content, a shared specimen size, and the same shaper; do not synthesize a font with Flutter's fallback.
2. Close the missing saved-gradient application path. Model all stops, their positions, linear/radial form, and direction in native `SetShapes`; expose a single fill projection to the Inspector. Preserve single-Undo edits and locked-layer rejection. Invalidate a color target when stop order changes.
3. Replace Solid/Gradient text toggles with rendered fill samples; place stop controls below their ramp and open Colors on the selected stop. Keep the existing palette as the selection destination.
4. Use font specimen rows, a clearer text content area, alignment icons, and grouped size/line-height/tracking controls. Retain Document ownership and current preview/commit behavior.
5. Run focused native/widget regression, native build once changes settle, then real-window checks for font comparison/application/Undo and gradient selection/stops/direction/Undo. Preserve pre-existing dirty files and the user's live document. No claim of real-window completion from compile results alone.

Reference search: current color.rs/ColorSlot/read.rs; existing saved palette `_stops` and `_gradientBox`; font_browser.dart; engine/text.rs and vector/raster.rs; Flutter ListView.builder and current thumbnail/easeModel request implementations. Current gaps: saved multi-color swatches only modify a local list; font shelf displays names only; Inspector exposes two endpoint colors and no gradient direction.
