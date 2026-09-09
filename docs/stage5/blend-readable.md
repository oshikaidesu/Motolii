# Blend — choose the visible result

2026-09-09 user-approved direction: the goal is mixing the selected material, not searching terminology. The user rejected rendering the composition once per candidate because of cost.

Desk shows four fixed blend-behavior specimens at a time, computed once from the existing W3C blend equations and cached. They are illustrative, not scene thumbnails. No media decoding, engine invocation, GPU readback, document revision invalidation or frame-driven sample generation. The existing Stage remains the only actual composition result. Hover requests are coalesced; leaving cancels; clicking applies once. Opacity adjusts strength and commits once on release. Names are secondary captions/tooltips.

Use previewBlend / previewProperties / commitPreview for gestures, with interaction ownership to avoid cancelling another editor. Validate zero sample regeneration on document/frame changes, latest-only hover requests, interruption, selection changes, single-commit apply/opacity, and small-panel layout. The user is inspecting the app themselves: do not take over their window for acceptance.
