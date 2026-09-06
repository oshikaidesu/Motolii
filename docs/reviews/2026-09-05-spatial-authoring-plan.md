# Spatial authoring: structural repair

User intent: preserve the useful layer/group/keyframe model, make compositing direct and organized, and solve the structures that force workarounds. This is not a request to gather shortcuts into another panel. Existing code is replaceable; Dioxus/Blitz authoring, native Rust/wgpu sharing, Vism and the single re_renderer 3D world remain.

## Confirmed constraints in current implementation

- `store/view/resolve/transform.rs::world_affine` inherits `Affine2`; resolved z/rx/ry are read only from the child in `resolve.rs`. Parent depth/orbit does not participate in the inherited pose.
- `engine/texture.rs::selected_layer_size_in` assigns groups the entire composition rectangle, unrelated to descendant geometry.
- `functions/placement.rs` and Stage companions can transform both a selected ancestor and its descendant, and apply world-looking deltas as local coordinates.
- `Mask` resolves only to a 2D `Path`. `engine/render.rs::apply_masks_to_layer` needs texture content and returns native spatial content unmasked with a diagnostic. Groups have no content there and skip mask processing.
- No shared mask/path edit target exists. A mask button without target identity would leave Delete, Stage and Inspector referring to different things.

## Required chain

1. **Shared spatial pose.** Establish one full affine world pose (`parent_world * local`) reused by rendering, camera projection, bounds and inverse edit mapping. Reuse glam/Rerun affine types; do not reconstruct an independent UI pose. Keep authored values distinct from evaluated pose. Resolve source-bound/pivot convention explicitly before changing rendering: current texture/mesh routines tilt around inferred content centers, while authored XY has an anchor. Saved artwork compatibility must be tested rather than assumed.
2. **Truthful geometry.** Decoded local bounds belong to the existing render/media owner. Transform planar corners or native volume corners by the same world pose. Derive groups from descendants and multi-selection from selected roots; empty groups have no invented composition-sized content. Group pose cannot depend on its own transformed descendant bounds.
3. **Selection edit plan.** Reuse `document/group.rs::outermost_present` as a shared query. Freeze editable selection roots, parent/world poses, time and manipulation frame at gesture start. Derive local changes through the parent inverse. Preserve full affine results; do not silently approximate shear/nonuniform parent transforms into insufficient parameters. Preview and commit consume the same plan in one Undo transaction.
4. **Editable attachments.** Selection identifies layer content versus its mask/path attachment. Vertex/tangent selection is subordinate to that identity. Shape paths and planar mask paths share one authoring mechanism, checked property writes and owned previews. Add/select/edit/invert/enable/delete/Undo/save-open must all connect; the current arbitrary inset rectangle is not a requirement.
5. **Coverage on contributions.** Separate mask source/coordinate domain from coverage combination. Keep existing planar path as a supported source, not the universal model. Group coverage acts on the correct composed contribution rather than duplicating a mask onto each child. Native mesh/point-cloud relationships remain native. No implicit flattening is an acceptable completion path.

## Decisions that implementation must settle explicitly

- Stable pivot/source-coordinate convention and compatibility with saved placements.
- Exact writable representation for affine results that authored TRS/XY-skew cannot represent; no silent lossy decomposition.
- Manipulation frame for camera-plane/world/local operations, using the same evaluator.
- Mask domain distinctions: texture-local planar boundary, projected boundary, surface relationship and world-space region are different meanings. Do not invent a catch-all enum that hides those distinctions.
- Mask disabled state must be an identity operation in the coverage fold, not opacity zero. New IDs must not reuse orphaned mask-property namespaces after attachment deletion.

These are implementation responsibilities within the user's authorization, not an approval checklist. Choose using existing semantics, upstream mechanisms and concrete creation/edit examples; record the chosen contract before parallel writers touch its consumers.

## Execution and evidence

- First vertical delivery: parent depth/orbit affects actual descendants consistently in Stage/export, with truthful group bounds. This is a milestone, not completion of all spatial authoring.
- Then shared selection transform and editable attachment lifecycle; source/selection contracts must be fixed before parallel implementation.
- Exclusive ownership: doc pose/query; render geometry/consumers; UI selection/input; attachment controls. Shared interface files are integrated serially, never assigned to overlapping lanes.
- Regressions: parent/child and parent+child selection, rotated/scaled parents, anchored poses, unchanged saved placements, preview==commit, Undo/Redo, cancellation and source rejection. Use glam composition/inverse laws and current render pixel fixtures as rulers.
- Mask acceptance includes actual editable boundaries and visible coverage, save/open/export, and separately tracked group/native spatial cases. A numeric mask list or vector-only editor cannot claim Photoshop/CLIP STUDIO brush-mask completeness.
- Batch type/dependency changes into a recorded baseline; then verify real-window editing with warm patches. Preserve current user documents and report any state that cannot survive baseline replacement.

## Status

Investigation and write-set planning complete. No spatial representation migration or new mask lifecycle has been implemented by this plan yet. Existing passing tests do not cover these missing structures.

## Evaluator responsibility decision — corrected after Lottie clarification

Lottie is the semantic authority. The `re_tf` experiment was withdrawn after re-reading the existing Rerun parts decision and checking Lottie reference evaluation. No projected EntityDb, frame registry, `re_tf` dependency or duplicate authored graph remains. The existing hierarchy traversal is shared between XY and 3D affine types; matrix arithmetic uses glam. Rerun receives evaluated geometry for display/rendering.

`local_transform3d` now follows pinned lottie-web v5.13.0 for the supported authored subset. Six oracle cases (30 points) were generated from the reference Matrix/TransformProperty operation order, with explicit Z-basis conversion. This corrected X/Y rotation order and pre-existing skew sign; group decomposition was updated to invert the same sign. Anchor Z, scale Z and Orientation remain missing authored dimensions, not intentional restrictions.

## World-pose consumer batch

`LayerPlacement.world_transform: Option<Affine3A>` carries one evaluated world pose. Document layers supply Some from a batched hierarchy evaluation. None is the compatibility route for explicit low-level compositor fixtures. Textured corners, mesh/point-cloud geometry, camera compensation, accumulator centers and Stage plane mapping consume the full pose. Baked/reprojected output clears the pose together with legacy placement values.

Renderer owns `projected_placement_corners(comp,camera,projection,placement,local_min,local_size)`; Stage uses that same function. Native source normalization retains existing XY bounds origin and Z center. Authored anchor governs X/Y tilt; old center-based tilt is not applied twice. Compatibility of tilted saved scenes is still a real-window check, not assumed from the XY cases.

Type-change baseline: saved `Documents/motolii-ux-current-2026-09-05.rrd`, closed app11800 and server10781, then applied the coherent consumer batch. The dependency graph and public retained placement layout changed, so one new baseline is required. Undo/ephemeral selection do not survive it. The new baseline has been requested after the checks below; normal warm patching resumes afterward.

## Verified checks and remaining work

- Document library: 12 passed, including Lottie reference cases, parent transforms and camera affine accuracy.
- UI binary: 230 passed; render library: 16 passed.
- Integration: group commands6, mask coverage3, spatial import2 passed. Spatial save/reopen pixels match.
- These checks do not establish finished 3D authoring. Real-window consumer verification, descendant-derived group bounds, parent-aware inverse edits, shared selection pivot/root reduction, affine write-back and the full mask lifecycle remain open.

Native milestone: baseline built in 26.18 s; server20778/app21702 reopened the saved example. Text+Rectangle were grouped, group Z scrubbed to -34.45, then group X rotation scrubbed to approximately34.45 degrees. Both descendants visibly followed the depth/rotation, using the same Stage. Three Undo operations restored the original two artwork layers and their original visible placement. The group selection rectangle is still the composition bounds and remains wrong; this observation does not close group bounds, ungrouping a 3D affine pose, or parent-aware inverse editing.

## Content bounds batch

`Engine::selected_layer_bounds_in` now returns local SpatialBounds. Text alpha bounds are derived once from the existing CPU raster and stored with the texture; no GPU readback or per-frame raster scan. Shape origins align to raster normalization, native bounds retain depth, and groups union descendant leaf bounds using evaluated poses. Empty/singular groups have no fabricated box. Bounds describe editable source content, not post-effect/mask extents.

Stage and Session carry typed selected bounds. All eight corners drive volume hit/marquee coverage and twelve-edge outlines; handles use an explicit midpoint manipulation plane. Inspector anchor fractions include bounds origin. Anchor relocation compensates XYZ using the same local transform, preserving tilted artwork instead of moving it accidentally.

Checks: UI233 and renderer18 passed. New regressions cover cached alpha bounds, nested/offset/empty group bounds, volume picking, offset resize and one-Undo anchor preservation. Real-window bounds acceptance pending. Still open: conservative box-vs-mesh picking, edge-on manipulation planes, inherited group visibility/compositing, shared multi-selection pivot, ancestor reduction and parent-aware inverse transform edits.

Baseline record: saved the restored example, closed app21702 and server20778, then changed text-cache/selection-bounds retained types together. One same-profile baseline is requested for the completed batch; afterward resume warm patching. Undo/temporary selection are reset by this type transition, while the saved project is reopened.

Native content-bounds receipt: new baseline29.09 s, server22973/app23895. Saved example reopened. Selecting Text produced a tight glyph-content box instead of the composition rectangle. Grouping Text+Rectangle produced a bounded volume enclosing both, with depth edges. The anchor-center control moved the anchor indicator to50%/50% while artwork and its box stayed fixed. Two Undo operations restored the ungrouped saved layout. This closes those observed examples only; parent-aware inverse gestures and ancestor deduplication remain next.
