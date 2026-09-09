# History — local records and checkpoints

2026-09-09 user acceptance: lightweight logs and named checkpoints first; branches later; no merge in this increment.

Authority and precedents:
- [VS Code local history](https://code.visualstudio.com/updates/v1_66#_local-history): named local snapshots and restore.
- [Flutter error handling](https://docs.flutter.dev/testing/errors): FlutterError.onError and PlatformDispatcher.onError.
- [Apple crash reports](https://developer.apple.com/documentation/xcode/acquiring-crash-reports-and-diagnostic-logs): show OS reports, not inferred crashes.
- Document::save/load in crates/motolii-doc/src/store/persist.rs: existing atomic RRD snapshot format. Checkpoints reference external media, like normal project saves.

Implementation contract:
- History keeps Undo separate from Records and Checkpoints. Logs do not become undo steps.
- One native session owns disk records across windows. Records are capped at 500, details at 32 KiB; checkpoint files are retained until explicitly managed outside the app.
- A checkpoint saves authored content without changing project path, dirty flag, or Undo cursor. Flush pending editors first; reject active previews.
- Restore validates the target before replacement and saves a recovery checkpoint before restoring. Restore changes the project to the checkpoint source, marks it unsaved, and starts a new Undo history. Recovery remains selectable.
- Failed persistence must not be reported as successful checkpoint creation or restoration.
- OS crash reports are read on demand; interrupted-session markers are warnings, not proof of a crash. Reports remain local, can be copied, and are never automatically uploaded.
- Future branch identity belongs to saved checkpoint metadata, never widget state. Automatic merge is outside this increment.

Acceptance: save/reload metadata; restore after editing and after restart; recovery before restore; missing/corrupt snapshot leaves current document intact; failed write has no success row; logs are bounded and survive restart; narrow panel layout and real-window save/restore.

## Verification — 2026-09-09

- Native build completed. `python3 motolii/ui/test/checkpoint_native_test.py` passed on the actual dylib: copy preserves path/dirty/undo; restore restores two layers and resets Undo; missing/corrupt RRD leaves the document intact; explicit save clears dirty.
- `swiftc motolii/ui/macos/Runner/HistoryStore.swift motolii/ui/test/history_store_test.swift -o /tmp/motolii-history-store-test` and the resulting executable passed: recovery retains later work, backup failure blocks restoration, metadata survives restart, 500-record cap, report allowlist, interruption marker.
- Flutter `history_panel_test.dart` and `history_records_test.dart` passed: Undo/Redo, rejection, details, flush-before-checkpoint, cancel/restore, narrow 240px panel. Scoped analyzer and raw-dimension lint passed.
- Real window: History tabs, OS report list, complete report detail and Copy/Close controls observed. Subsequent crash-list folding is source/test verified only.
- Remaining real-window gate: checkpoint create/restore. A later app build collided with a concurrent Xcode build (`build.db` locked); reconnecting to the running app by path, bundle ID, and resetting CUA still returned `AXError.failure` on click. Existing project content was not edited for verification. Do not claim end-to-end window acceptance until this gate passes.
