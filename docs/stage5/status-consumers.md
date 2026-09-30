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
