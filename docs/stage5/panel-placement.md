# Panel placement

User correction (2026-09-06): the shared panel/placement abstraction remains useful, but configuration belongs in Settings. Desk holds tools between a popup and a permanent panel: Depth, Ease, Blend and History. Making those tools optionally permanent does not make Desk the owner or catalog of all other panels.

- The common catalog owns identities, icons, usable sizes and drawer eligibility; native windows know panel identities only.
- Settings presents the full panel list and configures tab, separate window or hidden. Auxiliary tools additionally support Desk.
- Desk owns both contextual presentation and its idle default. Settings and placement operations must not select an active drawer tool or change that default. Desk reads shared selection/focus and presents available auxiliary tools. It does not manage panel registration, window creation or general layout. Stage, Timeline, Inspector, Notes and Web are not drawer choices.
- The main workspace owns one placement per panel and persists it. Separate windows use the same route. Moving a tool preserves Document and workspace data; editors flush before moving/closing.
- Text and Reference are unified in Notes pages, stored through Document intents. Notes and Web remain independent viewing surfaces. Web opens the regular browser.

Acceptance: Settings can promote Depth/Ease from Desk to a permanent tab/window and return them; fixed panels use the same tab/window/hidden infrastructure; Desk contains no full-panel placement catalog; selection changes cannot unmount a Depth interaction; Notes saves/reopens and survives hosting changes.

Verified: 17 Flutter tests pass, including Desk scope and shared placement; real-window Settings moved Depth to a tab and back to Desk.
