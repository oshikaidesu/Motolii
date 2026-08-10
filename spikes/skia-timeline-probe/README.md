# skia-timeline-probe(2026-08-10回収)

`skia-safe 0.99.0` + `wgpu 29` + `winit 0.30.9` の隔離probe。
元所在は `~/Documents/Codex/2026-08-06/motolii-ui-hybrid-research-handoff/work/skia-timeline-probe/`
で、[2026-08-10回収監査](../../docs/reviews/2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md)によりリポジトリへ移管した(`target/`除外)。以後の正本はこのディレクトリ。

- `src/bin/motolii_depth*.rs` + `motolii-depth-rail-v*.png`:
  [Depth Rail選択フォーカス設計](../../docs/reviews/2026-08-08-depth-rail-selection-focus-decision.md)の
  変遷(v4〜v14)。`motolii_depth6.rs`がv14本決定の静止画、`motolii_depth_interactive.rs`が本決定の対話demo。
- `src/bin/timeline_*.rs` / `curve_editor_interactive.rs` / `stage_present_interactive.rs` / `stage_overlay_bench.rs`:
  [Timeline設計決定とskia fixtures](../../docs/reviews/2026-08-08-timeline-design-decisions-and-skia-fixtures.md)の実証bin群。

**状態境界**: 隔離probe合格であり、製品接続・製品完成の証拠ではない
(`N-OVERLAY`は[統合地図](../../docs/outcome-driven-integration-map.md)で`PROBE_ONLY`のまま)。
