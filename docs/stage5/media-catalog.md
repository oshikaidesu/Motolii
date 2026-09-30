# Media Catalog and the Browser on top of it

Two lanes, one boundary. **Below**: registered source folders → a resilient index → a Result Set. **Above**: the Browser
draws a Result Set as List / Thumbnail / Explore, with Selection Preview and live faces. The only thing they share is the
presentation-neutral Result Set (`ResultSource` / `BrowserItem` in Dart, `query` in the catalog).

```
SOURCES (registered folders, read-only)
   ↓  recursive index (walkdir, file-id, SQLite WAL + FTS5)      notify = "something changed", never the truth
CATALOG  (asset identity ≠ path; where it is now, where it has been)
   ↓  query: source / folder / type / search / order
RESULT SET  ──►  List │ Thumbnail │ Explore (one fluid board)  ──►  Selection Preview · live faces
```

Where (source / folder), What (type), Which (search), How (view) are independent axes. Folder browsing is one more way to
make the result set, not another browser.

## Mature parts used, and what Motolii owns

| Part | Used for | Licence | Motolii's own glue |
|---|---|---|---|
| `walkdir` | recursive discovery (never follows a folder link, so no loop) | MIT/Unlicense | skip hidden, one media table (`asset_type_for_extension`, the importer's own) |
| `file-id` (notify-rs) | which filesystem object (inode / file index) | MIT/Apache-2.0 | how it is used as evidence (below) |
| `rusqlite` (bundled SQLite, WAL, FTS5 trigram) | the persistent catalog and substring search | MIT / public domain | schema (`catalog/schema.rs`), queries |
| `notify` 8.2 (already a dependency of render) | a change signal | CC0/Artistic | marks a source dirty; the ordinary incremental refresh does the work |
| `SourceFingerprintV1` (already in doc) | the cheap fingerprint (size + first and last MB) | — | none: the catalog stores the same string a work stores |

Motolii-specific and owned: source registration, asset identity, the relocation resolver / relink policy, the browser-facing
query. No custom indexer, watcher framework, search engine or DB abstraction.

## Identity and relocation

* **Path ≠ identity.** An asset keeps its `uid` when it moves or is renamed; a path is only where it is now; old places are
  kept (`past_places`), and a folder seen to move as a whole is one fact (`folder_moves`).
* **Evidence, strongest first** (`catalog/matching.rs`): platform file id with the same size and time; a unique cheap
  fingerprint. Nothing weaker links by itself. Two equal fits stay unlinked (ambiguous). An inode reused by another file
  (different size or time) is not a move. Hard links prove nothing.
* **Content equality ≠ identity.** A copy is a second asset; nothing is merged or deleted.
* **Missing is kept, never purged.** A vanished file stays known as missing (its last place helps the resolver); forgetting
  is an explicit act and touches no file.
* **Source unavailable ≠ asset missing.** An unreadable root, or a readable one that shows nothing where assets were
  indexed (an unmounted volume's empty mount point), changes nothing and is reported as unavailable. `relocate_source`
  reconnects the index to a new root after checking that a sample of the indexed files is there with the same sizes.
* **The resolver** (`catalog/resolve.rs`): exact / strong / ambiguous / missing / source-unavailable. Once some references
  of one folder are found at a new place, that folder's move is applied to the rest of the batch.
* **A work finds its moved media** (`relinkFromCatalog`, `port/relink.rs`): the work's missing media is resolved; only
  exact/strong results are relinked, in one undo step; ambiguous ones are reported with their candidates and left alone.
  The console says what was found. Run when a work opens if any media is missing.
* **Source files are read-only.** Nothing moves, renames, deletes or writes them (tested by snapshotting the trees).

## Measured (debug-optimised test build, small files; `cargo test -p motolii-ui --lib catalog::bench -- --ignored --nocapture`)

| files | index | no-change refresh | one rename | one move | folder move (10 %) | one delete | source relocate | query | search | folder | enrich all |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 100 | 3 ms | 1.3 ms | 1.4 | 1.3 | 2.6 | 1.4 | 2.5 | 0.12 | 0.13 | 0.13 | 4 ms |
| 1,000 | 22 ms | 4.4 ms | 4.1 | 3.9 | 7.4 | 4.0 | 5.4 | 0.31 | 0.33 | 0.48 | 55 ms |
| 10,000 | 255 ms | 37 ms | 36 | 35 | 59 (1,000 moved) | 36 | 41 | 0.7 | 1.2 | 3.0 | 611 ms |

Process RSS about 40 MB at 10k. Enrichment (fingerprints, image sizes) is a separate bounded batch, never a condition of
the index being usable.

## The door to the host

`motolii_catalog_request` (JSON in, JSON out) on its own Swift queue, so a scan never waits on rendering and Flutter only
receives the reply. Ops: `sources addSource removeSource enableSource renameSource relocateSource refresh enrich forgetMissing
query folders resolve faces frame picture watch`. State lives in the state folder (`catalog.sqlite`, `MOTOLII_STATE_DIR`
respected; the explorer keeps its own in `explorer/.state`).

## Browser (Explorer first)

`MediaBrowser` keeps the view, the selection and the open preview; changing view keeps the same asset chosen. List reads
the catalog's own columns and asks the owner for another order (name / type / size / date). Thumbnail is the existing
masonry. Explore is a prototype (chosen asset in the middle, the rest by nearness of measured mean colour): there is no
similarity backend and none is faked in the product. `FluidBoard` moves the same faces between the three (tested: a face is
between its two places part-way through). Selection Preview opens in place (double-tap, Space, tap the chosen one): live
faces are an image zoom/pan, a panorama drag, a turnable 3D model, a clip scrub (one frame at a time from the host), a sound
position. Source / folder / type / search sit together above the views.

## Not done, and the Decision Queue

Not done: production Media shelf still shows the work's own assets (the catalog Browser runs in the explorer); a wake from the
change signal to Flutter (the watcher indexes by itself, but the view asks on interaction); sort by length; collections /
favourites / recent over catalog assets; current-project as a source; the scan holds the catalog lock while it runs (queries
wait behind a large refresh; scanning outside the lock is the next step).

Decision Queue (meaning, for the person to decide):
1. Adopt the catalog Browser as the product's Media (replacing the assets-of-this-work shelf)? What "Place" means for a
   catalog asset (admit into the work, then place) — the catalog id is not the work's `AssetId`.
2. Which actions the Selection Preview offers beyond Reveal in Finder (Place / Import) — only existing actions until then.
3. Relink at open writes to the work (undoable): keep it automatic, or ask first?
4. Folder selection: everything below (library view) or only what stands in it (folder view)? Both queries exist.
5. Explore: keep colour nearness as the first honest projection, or wait for a real similarity source?

## Blind critic gate (Browser vs the North Star image) — status 2026-09-30

Seven rounds with a critic that sees only screenshots and the concept image. Rounds 1–6 each produced real fixes (ring over the face, label collisions, Explore off the seat and unsized, size slider, live-face hints, stagger, lines to the nearest five, every asset in one field, chosen asset named, larger default faces). Round 7 still says FAIL, on points that repeat decided items or read the still filmstrip / shot canvas as defects:

- Explore "small, not filling the pane": the field is width-limited (no face may leave the seat) and centred; the tall blank area is shot canvas. Stretching to an ellipse is possible, not done.
- Filmstrip early frames "bunch in a column": the first ms of a staggered move from the List rows; stills cannot show the path. A recorded motion check would settle it (the widget test asserts identity and no overlap).
- Selection Preview docked below rather than over the result: in the Decision Queue.
- Region labels ("Sky", "City") in Explore: declined, invented meaning (no similarity source).
- Sound play button: no playback capability exists.

The critic gate was NOT passed; the owner's own look is the remaining judge.

## Production readiness (2026-09-30)

- **No lock while reading files.** The walk, the fingerprints, image sizes, thumbnails and clip frames all run outside the catalog lock; the lock is taken to read what is needed and for short writes (new files go in batches of 400 with a breath between, because a std Mutex is not fair). A test asks queries while 6000 files are indexed: the slowest wait was 11 ms (76 ms before the batching, 110 ms before the split).
- **The watcher reaches the window.** The catalog's watcher refreshes the index on its own; when that changed it, the host sends a bare `catalogChanged` (C callback `motolii_catalog_on_change` -> main thread -> channel; the Explorer hears the same through dart:ffi). `CatalogSession` starts the watcher and asks its current query again (signals arriving meanwhile coalesce). Nothing but "look again" is pushed.
- **Proved in the real app:** `integration_test/catalog_follows_test.dart` adds, renames and deletes files under a registered folder from outside the app; the result follows with nothing asked, and the renamed file keeps its asset id.
- **Place is decided by the owner's existing path.** `placeCatalogAsset {id}` (native, `port/catalog_place.rs`): the catalog says what exists; the work owns an asset from the first use. Browsing writes nothing; placing admits the file into the work's asset table (the same admission as import, deduplicated by content hash) and places the layer in the same undo step (`place_layer_with(prelude)`); a second use places again without admitting twice. The preview offers "Place in project" and "Reveal in Finder" (real actions only).
- **The work's own assets are a source of the same Browser** (`This project` chip, `ProjectSource`): the document's assets already carry thumbnails, facts, seconds and peaks, so they map onto `BrowserItem` with no new meaning; type and search apply; Place uses the existing `placeAsset`. The production Media shelf is untouched (nothing deleted). Swapping the production Media tab for this Browser is the remaining step, and now only a wiring decision.

## Old Media shelf -> catalog Browser (parity port, 2026-09-30)

The production Media seat is now the catalog Browser (`MediaSeat`: This project + every Source; List / Thumbnail, Explore stays in the Explorer as two prototypes); the old `LiveBrowserShelf` is removed. Ported with the old meaning: single / Cmd / Shift picking (the last pick is the chosen one), Cmd-A, Escape, Home / End, arrows (list by row, thumbnail by where the faces are), Enter and double-click place, Delete / Backspace removes the work's own unused assets (`removeAsset`, the host keeps used ones), right-click menu (Place, Replace selected layer, Reveal, Open, Extract palette, Copy path, Relink / Locate, Remove: the document's own actions for a work asset; only Place / Reveal / Open / Palette / Copy path for a catalog asset), drag to the Timeline (`{asset}` for a work asset, `{asset, catalog}` for a catalog one), Import button and the drop veil, pick-after-import, used and missing marks, choosing on the press itself (a tap handler beside a double-tap waited ~100-300 ms).

Not yet ported (old shelf had them; no new meaning involved): Favorites / collections 1-7 (menu, digit keys, marks, views), Recents on place, the menu's facts header, the Folders class, Replace selected layer for a catalog asset, and the plain fallback views used above 300 results (no multi-pick there).

Decision Queue (new meaning only): 6. What "Remove" means for a catalog asset (forget it from the index / stop the source) and that no Browser action ever deletes a source file — nothing is wired to Delete for catalog assets today.

Real-app checks run: all integration tests (with the fixture document) plus `catalog_follows_test` and `catalog_place_test`.

## Real-window pass of the production Media seat (2026-09-30)

Looked at in the live app (scratch document, isolated state). Found and fixed: the Sources row scrolled sideways at seat width so Bundled / All were hidden (now wraps); the bundled HDR environments the old library listed had gone (now the `Bundled` Source, placed with `create`); there was no way to register a folder (now `＋ Folder…`, through the existing folder-capable picker). Confirmed by hand: click chooses at once (ring on press), double-click places a layer, the seat's Sources / Types / search read cleanly. Not confirmed by hand (the driving tool cannot send keys to a background window): Enter / arrows / Delete in the window (covered by widget tests) and Place -> undo in the window (covered by the real-app test). One thing seen and not examined: after placing the bundled Sunset sky the Stage showed black (the same `create` the old shelf used; the render, not the Browser).

Direction settled with the owner: the old shelf is a behavioural floor (do not lose basic operations), not a spec to reproduce. Favorites / Recents / Collections 1-7 are not ported; they get re-thought inside the Sources / Places + Library / Types + Collections + Search -> Result Set model. Explore B: hubs for Source / Folder / Project / Type, direct edges only for true pairwise relations (same bytes); filters are visibility over one stable map.

## Explore, Favorites and Recent in the product (2026-09-30)

- **Explore is the sparse nearest-neighbour map** (`explore_graph.dart`), in the production Media seat (no longer a prototype in the Explorer). The hub topology (Source / Folder / Type / Project as nodes) is retired from the product; its screenshots stay as evidence next to the new ones (`docs/stage5/media-explore/`). Each asset is joined to its `k = 3` nearest by a score made only of what the catalog already holds (name, folder distance, time, shape, kind, length/size, held by the work); an asset with nothing near it stands alone; at most `2k` lines end at one asset; the same bytes are joined by a duplicate line. Measured on the 50-asset fixture: k=3 gives 92 lines, 3 islands, max 6 lines at an asset; k=4 joins everything into one piece (125 lines). Type and Folder are an overlay (tint and soft regions), never edges.
- **Layout.** The layout has no wall: repulsion acts within reach only, so a stray asset floats near its island instead of being pushed to the end of the world (the old layout clamped every node to a box, which laid islands along its edges); the settled map is moved inside an even margin. Global keeps one stored layout whatever is chosen, filtered or how big the seat is; Local 1 / Local 2 are the explicit look round the chosen asset (rooted at it; the outer ring is quiet); the camera fits the whole map centred and eases between fits. A new asset settles beside what it is near and the rest moves less than half a face.
- **"Held by the work" is told by content.** The work's asset rows carry `contentHash` (the very text the catalog stores as an asset's fingerprint): a catalog asset is held by the work when the two are equal, whatever path either calls the file (the earlier path comparison never matched). Held assets show the small dot.
- **Place from the map.** Click picks (nothing moves), Enter / double-click / the preview's Place / the menu all place through the existing `placeCatalogAsset`; Explore stays open.
- **Favorites and Recent** use the existing user library (`LiveBrowserUser`, saved with the settings) keyed by the catalog asset id, so they survive a move or rename and an app restart. `F` or the menu keeps / lets go; `★ Favorites` and `Recent` are Sources of the same Browser (the catalog answers `query {ids}`); placing or opening a preview adds to Recent. Only catalog assets are kept: the work's own assets and the bundled ones belong to other owners. Collections are not done.
- The Media seat starts on *All* when the work holds no assets (it used to open on an empty *This project*).
- CURRENT LIMITATION of the current Explore implementation (not a product requirement, not a contract): its force layout is O(n²), so it maps up to 300 assets (`exploreUpTo`) and beyond that asks for a narrower result or List/Thumbnail. List and Thumbnail are lazily built and have no such bound. What Explore must keep, at any size, is asset identity (the same asset across List / Thumbnail / Explore), selection and Place; continuity across views is identity plus perceptual continuity, never "every asset widget alive at once".
