# Who reads the full status (map for a possible later delta)

Status: a map, nothing here is implemented as a delta. 2026-09-30.

## Path of one preview step

Dart `command`/`commandDirect` → Swift `request` (main, `renderQueue.sync`) → Rust `motolii_probe_request`
(`status_text`: the cached snapshot is serialized where it lies) → Swift `JSONSerialization` into `[String: Any]`
→ Swift `render` (GPU finish) → Rust `status` again for the render reply → Dart `_accept` (`EditorSession.map`,
`take` per key) → slices → Stage / Inspector / Timeline / panels. Swift `broadcast` merges the same dictionary into
`session.state` and sends `documentChanged` to every other attached host.

## Consumers

| consumer | reads | delta-sufficient? |
|---|---|---|
| Dart `_accept` (`session_native.dart`) | whole envelope → `rendered` | yes, if the delta carries changed rows + the small fields |
| `DocumentSlice` subscribers (Stage, Inspector session, Timeline session, depth, relations, blend, ease, desk, browser) | named keys, `sameValue` | yes — they already wake per key |
| Inspector | selected layer row + properties | yes (one row) |
| Stage | every layer's `bounds/x/y` (hit test), `selectedBounds`, `stageWindow`, gizmos | geometry of all rows only |
| Timeline | rows (name, in/out, keys), markers, waveforms | changed rows + key list |
| Swift `broadcast` | `width/height/frame`; re-parses the whole JSON to get them | yes; forwarding to other hosts needs the same payload |
| Detached/panel windows (`documentChanged`) | same envelope as main | same as Dart |
| Desk / Browser / Relations / Depth hosts | `depthLayout`, `assets`, `catalog`, relations | yes; `depthLayout`, gizmos, assets are by-demand candidates |

## O(document) per preview, by measurement (94 rectangles, release)

- `layer_keys` 1.1 ms, `authored_signature` (cached per revision) 1.3 ms — not worth a store-event subscriber.
- `build_status` 9.6 → 4.7 ms after moving the rows into the reply instead of `json!`-copying them.
- serialize ~2 ms for ~1 MB; Swift and Dart each parse it again (the ~7 ms decode). A delta would remove this.
- GPU wait ~33 ms is render work for 94 layers, not bookkeeping.

## Consumers by what they need (2026-09-30)

| class | consumers | source today |
|---|---|---|
| full snapshot | Timeline rows/markers/waveforms, Stage hit test (every layer's bounds), Desk/Relations/Depth when open, first attach of any window | `rendered` value |
| structural state only (layer order, kinds, selection) | Browser "This project", Inspector tree, Desk faces | `layerShape()` slice, not the rows |
| selected layer only | Inspector rows, Ease/Blend hosts, colour target | `inspectorSession` slice |
| frame / render metadata only | playhead, renderCount, `needsRender`, `stagePublishedFrames` | `frame`, small fields; `deferSnapshot` already answers `{"needsRender":true}` for it |

What was removed on that basis (no new protocol):
- the Camera view is no longer drawn when its tab is hidden (`cameraShown`, same shape as `stageWindow`; nothing draws → Camera is drawn as before);
- reference fields (assets + thumbnails, catalog rows, fonts, backgrounds…) are built only when their key (document revision, shelf generation, asset files present, playing) moves — not on every preview step; the cache no longer clones/compares them when nothing moved.

## GPU wait (ledger row 11) — settled from the code

- The live path has **no per-submit wait**: `flush_pending` submits a batch and calls `begin_frame` (upstream's reclaim); a frame is many batches, never one wait per layer. `wait_for_gpu`/`poll(Wait)` remain only in export/readback and tests.
- A render ends in **one** completion wait (`frames.finish` for coherence; `selection_bounds.take` waits on the same submission, so they do not add up).
- Headless, 1582×890 Stage window: 4 layers 0.3 ms CPU + 4.6 ms GPU; 94 layers 4.3 ms CPU + 9.1 ms GPU. In the app the same wait reads ~17–33 ms because Flutter's own raster shares the GPU — not Motolii's to remove.
- Rerun's `GpuReadbackBelt` reads textures only (`read_buffer` is commented out in the fork), so `selection_bounds` (GPU reduce, then a 4-word staging copy) has no upstream replacement; kept.

## UNRESOLVED (left for the owner)

1. `EffectScratch` → upstream `GpuTexturePool` (ledger row 6): ~40 call sites hand raw `wgpu::Texture`s back and forth and rely on same-frame reuse; upstream reclaims at frame boundaries. Ownership change, no measured gain.
2. `RecordCache`/`TrackCache` in `motolii-doc/src/store/read.rs` vs `re_query::QueryCache` with a `ChunkStoreSubscriber`: invalidation today is by whole store generation. Measured cost of the per-status fingerprinting is ~1 ms at 94 layers, so the subscriber is not worth its weight yet.
3. Status is parsed in Swift (`JSONSerialization`, ~6 ms at 94 layers), re-encoded by the channel codec and decoded in Dart. Removing that needs either a text passthrough or a delta — a wire change.
4. Camera Framing / renderer ownership: untouched.
