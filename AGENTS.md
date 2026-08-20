# AGENTS.md

Repository-specific agent *conduct* rules were archived in [docs/archive/agent-governance/](docs/archive/agent-governance/) and are not active. The only section below is build/test operations, kept here by user decision (2026-08-21) because every lane pays the cost of not knowing it.

## ビルド/テストを回す時(裁定138・実測済み)

- **常設 warm worktree**(役割別・`target/` 温存・使い捨て禁止)+ **レーンごとの `-p` 集合固定**。`-p` の集合が変わると feature unification で再ビルドを踏む(実測: 固定 1.45s / 変動 28.6s)。合格基準線は 16〜33s
- **魔法フラグは無い**: sccache / cranelift / `-Zthreads` / lld / nextest は律速(リンク)に効かないため理由つき却下済み。`debug=line-tables-only` は設定済み
- **stash 禁止**(worktree 間で共有される)/ **`CARGO_TARGET_DIR` 共有禁止**(後勝ち事故の実測あり)/ Edit 直後の stale fingerprint は touch で解消
- レーン開始前に常駐プロセス(fileWatcher 等)を確認する(ビルド計測の汚染源、実測2例)
- cargo を**背景実行にしない**(subagent が自停止する既知事故)

正本: [ビルド速度の調査](docs/reviews/2026-08-19-build-speed-investigation.md)・[next/DECISIONS.md](next/DECISIONS.md) 裁定138。レーン運用の実測則は [next/reference/KNOWN.md](next/reference/KNOWN.md) の「レーン運用」節。
