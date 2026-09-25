# Renderer invariants: Scene → Prepared Frame → View State → View Frame → Present

Adopted by the user on 2026-09-23. This document fixes where each piece of state belongs, so that later work cannot quietly move it. It adds no abstraction.

The split was checked against the source of four renderers:
- **Rerun.** One `ViewBuilder` per view per frame. The rect, `resolution_in_pixel`, projection and target are fixed together and handed to the egui callback. Pan/zoom is written to the blueprint, never to the store.
- **Godot.** The projection is built from the viewport size inside `render_camera`. Probes are rendered once per frame. `viewport_set_size` never touches the scenario.
- **Blender.** `view3d_winmatrix_set` recomputes the projection from `winx/winy` on every redraw. The camera border decouples the render resolution from the region's shape. One depsgraph is shared by every view.
- **three.js.** It is the counter-example. `camera.aspect` is left to the caller, which is why its picture stays stretched until `updateProjectionMatrix` is called.

## The layers

| Layer | Owns | Invalidated by |
|---|---|---|
| Scene | the work: composition size, timeline, layers, geometry, materials, groups and plates as meaning | Intents only |
| Prepared Frame | the shared result for (revision, time, quality, explicit density request): pictures, plates' opaque content, environment, world reflection and light | those four inputs |
| View State | a view's observer/camera, zoom, pan, ROI | the user acting on that view |
| View Frame | one tick's {target size, ROI, projection, target} for one view, plus what reads the view's picture (transmission, screen-space effects, outline, feedback) | rebuilt every tick |
| Present / Surface | physical pixels, DPI, the presentable texture, and which picture the UI shows with which window | a size change or a finished frame |

## Invariants

1. The Scene changes only through Intents. View, Surface and Present never write to it.
2. A Prepared Frame never reads a view's window, ROI, observer, target or any Surface state. It is a function of (revision, time, quality, explicit density request).
3. World preparation and world captures (reflection, light) happen once per tick, however many views or plates there are.
4. Anything that reads a view's picture belongs to the View, including inside a plate. Standard Glass in a plate refracts the view's picture below the plate.
5. Stage and Camera are two views of the same Prepared Frame. They differ only in camera, ROI, target and outline.
6. The composition's aspect never follows the Surface. The ROI maps the composition into the view through `viewport_transformation`, and the projection's aspect is the composition's.
7. A View Frame's {size, ROI, projection, target} are fixed together in one tick. A target of another size is refused.
8. Present publishes a picture together with the window and ROI it was drawn for. The UI never places picture N by the metadata of frame N−1.
9. Resize and zoom requests are latest-wins. A window superseded before its turn is neither sent nor drawn.
10. Density/LOD changes only precision. It never changes geometry or placement.
