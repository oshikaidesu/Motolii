# Production UI Oracle — 2026-10-04

Entry: `scripts/motolii-ui.sh dev` (human) / `scripts/motolii-ui.sh it-oracle` (automated). Not Design Mode, not design-lab.

Verdict: **PASS** = gesture → feedback → document/stage change; silent no-op = **FAIL**.

Bootstrap: integration tests seed `motolii/ui/integration_test/fixtures/oracle_seed.js` via `runScript` when the work is empty (`oracle_fixture.dart`).

## Phase 1 — multi-window

| ID | Surface | Operation | Expected | Result | Notes |
|----|---------|-----------|----------|--------|-------|
| P1-A | Browser | Create shape/text | Layer + Stage | PASS | `create_places_test.dart` |
| P1-B | Inspector | Change property | Stage updates | PASS | `first_preview_test.dart`, `inspector_reset_test.dart` |
| P1-C | Stage | Select / pan / zoom | Feedback | PASS | `stage_move_test.dart`, `stage_trackpad_test.dart`, `stage_boxcam_test.dart` |
| P1-D | Detached Timeline | Session + seek | Same document | PASS | `detached_panel_test.dart` (detach API + seek; child-window UI not driven in same test process) |
| P1-E | Detached Inspector | Property change | — | PARTIAL | No automated detach+inspector test yet |
| P1-F | Main | Undo / edit | Undo works | PASS | `history_test.dart`, desk tests |
| P1-G | Detach close | Redock panel | — | PARTIAL | `windowClosed` redock not in integration_test |
| P1-H | Restart | Open saved `.rrd` | — | PARTIAL | Not in integration_test (manual oracle) |

## Phase 2 — integration_test (2026-10-04 run)

| Test file | Focus | Result |
|-----------|-------|--------|
| create_places_test.dart | Browser create | PASS |
| catalog_place_test.dart | Media place | PASS |
| catalog_follows_test.dart | Catalog | PASS |
| first_preview_test.dart | Inspector preview | PASS |
| timeline_test.dart | Timeline | PASS |
| stage_move_test.dart | Stage move | PASS |
| stage_trackpad_test.dart | Stage trackpad | PASS |
| stage_boxcam_test.dart | Box camera | PASS |
| inspector_reset_test.dart | Inspector | PASS |
| nudge_test.dart | Nudge keys | PASS |
| history_test.dart | Undo | PASS |
| ease_test.dart | Ease | PASS |
| depth_test.dart | Depth desk | PASS |
| relations_test.dart | Relations | PASS |
| notes_test.dart | Notes | PASS |
| media_explore_test.dart | Media explore | PASS |
| latency_test.dart | Latency | PASS |
| detached_panel_test.dart | Multi-window | PASS |
| heavy_folder_test.dart | Browser folder | SKIP (env) |
| surface_audit_test.dart | Surface tokens | SKIP (env) |

## Gaps / follow-ups

| Item | Status |
|------|--------|
| Graph seat content | PARTIAL — placeholder surface only (`seats.dart`) |
| Pin top-bar key | PARTIAL — drawn, no operation |
| Save → restart → open | Manual oracle |
| Detached Inspector + redock on close | Add integration_test when needed |
| Export end-to-end | Not in this suite |

## Convenience added this pass

- `oracle_seed.js` + `bootOracleApp()` for integration tests on empty documents
- `EditorSession.detachPanelRequested` (wired from LiveShell) for detach without menu scraping
- `scripts/motolii-ui.sh it-oracle` alias; `it` fixed for empty `it_defines` under `set -u`
