# MotoliiRnProbe(2026-08-10回収)

RN製品UI再現probe。`App.tsx` 660行 — Browser 3タブ(`MEDIA`/`EFFECTS`/`CREATE`)、
Inspector/Extensions、Timeline 3モード、effect一覧、panel registry、
native `MotoliiGpuComponentView.mm`、Fabric spec `MotoliiGpuView`/`MotoliiTimelineView`。

元所在は `~/Documents/Codex/2026-08-06/ui-rust-ui-c-react/work/MotoliiRnProbe/` で、
[2026-08-10回収監査](../../docs/reviews/2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md)により
リポジトリへ移管した(node_modules/Pods/build/target/vendor/.yarn除外。`yarn install`で復元可)。
以後の正本はこのディレクトリ。RN標準のREADMEは`README.upstream.md`へ退避。

**状態境界**: 製品RN shellの正本は`ui/motolii-rn`(移管済みはBrowserPanel/Inspector initial read/
StageComponentViewのみ)。本probeは**未移管部分のconcept oracle**であり、そのまま製品コードとして
importしない。移管は[RN runtime実行地図](../../docs/m3-rn-runtime-execution-map.md)のR1/R2粒で行う。
