# Browser rebuild — Phase 4

User request: reproduce AEViewer as a working desktop media browser; replace the prior incremental card restyling. Official reference: https://aescripts.com/aeviewer-pro/ and its preview-modes.mp4, tabs-nav.mp4, instant-search.mp4, collections.mp4 demos. Keep compact image-first tile silhouettes; do not invent placeholders for unavailable capabilities.

Execution order:
1. Checkpoint current browser source without staging unrelated work.
2. Replace Media presentation with a dedicated media-library owner: document assets, filesystem navigation, back/forward/up, search, favorites/collections, compact grid and detailed list, consistent footer.
3. Async file metadata and preview through installed ffprobe/ffmpeg; bounded work, cancellation/disposal and explicit failure. Render with Flutter image/widgets; no fabricated waveform or media metadata.
4. Import and place through EditorSession/Document only. Library organization is workspace state. Keep existing Create/Effects/Colors and their concurrent edits intact.
5. Meaningful filesystem/model tests plus Flutter analysis and real-window navigation, mode switch, selection, import/place and Undo. Record unsupported AE formats/authoring separately from implemented browsing.

The implementation uses public behavior/screens as its specification, not AEViewer binaries or proprietary source. Existing shared theme/metrics and Flutter controls remain the UI mechanism. FFmpeg showwavespic/scale and ffprobe JSON are the external metadata/preview reference.

Status: REJECTED by user before acceptance. This phase expanded behavior instead of reproducing the requested appearance. Its implementation and dedicated tests were removed; Phase 3.1 presentation restored through a narrow inverse edit. No other work was rolled back.
