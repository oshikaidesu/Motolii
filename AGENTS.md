# AGENTS.md

Repository-specific agent *conduct* rules were archived in [docs/archive/agent-governance/](docs/archive/agent-governance/) and are not active. The only section below is build/test operations, kept here by user decision (2026-08-21) because every lane pays the cost of not knowing it.

## ビルド/テストを回す時(裁定138・実測済み)

- **常設 warm worktree**(役割別・`target/` 温存・使い捨て禁止)+ **レーンごとの `-p` 集合固定**。`-p` の集合が変わると feature unification で再ビルドを踏む(実測: 固定 1.45s / 変動 28.6s)。合格基準線は 16〜33s
- **魔法フラグは無い**: sccache / cranelift / `-Zthreads` / lld / nextest は律速(リンク)に効かないため理由つき却下済み。`debug=line-tables-only` は設定済み
- **stash 禁止**(worktree 間で共有される)/ **`CARGO_TARGET_DIR` 共有禁止**(後勝ち事故の実測あり)/ Edit 直後の stale fingerprint は touch で解消
- レーン開始前に常駐プロセス(fileWatcher 等)を確認する(ビルド計測の汚染源、実測2例)
- cargo を**背景実行にしない**(subagent が自停止する既知事故)

### 検収の3段(2026-08-22 実測 — [静的検収調査](docs/reviews/2026-08-22-static-acceptance-survey.md))

- 実測定数(next/ 22crate・`-j 4`): `cargo check --workspace` = **warm 1.4s / cold 50s**。フル `cargo test --workspace --locked --no-fail-fast` = **warm 100s / cold 607s**。壁時計はキャッシュ状態で6倍動くので**実行時間を合否の物差しにしない** — 段の使い分け(機会)で裁く
- 段の使い分け: doc のみ=check-docs.sh だけ / pane 局所=check+該当pane と shell の suite / store・engine 跨り・fork pin bump・merge 境界=フル必須(判断表は調査 §6)
- `-p` サブセットの素朴運用は warm フルより遅くなる(199s vs 100s — 上記「`-p` 集合固定」の再発見。**この節を読まずにビルド調査を始めるとこの再発見を繰り返す**)
- 時間予算試験(storm・r2)は debug+並列で走らせる事自体が矛盾 — 合否確認は単独 or release で
- **合否の exit code をパイプ越しに取らない**: `cargo … | tail` は合否を殺す(前任の check-docs 事故)。zsh では `$PIPESTATUS` は空(bash 綴り — zsh は `$pipestatus`)で「検証したつもり」になる(2026-08-22 に2回実測)。**リダイレクトで log へ落とし `$?` を直接見る**のが唯一安全

### 頻出コマンド(コピペ用 — 記憶から組み立てない。2026-08-22: cd 位置の誤り5連発の根治)

正本 workspace は `next/`(リポ根の Cargo.toml は旧 workspace で `motolii-shell` を含まない)。**`--manifest-path` で cwd 依存を消す** — 背景実行はシェルの cd 履歴に関わらずリポ根で走るため、cd 前置は事故源:

```bash
# フル関門(merge 前最終)— 合否は $? 直取り
cargo test --manifest-path /Users/member_ottoto/rust_ae/Motolii/next/Cargo.toml --workspace --locked --no-fail-fast -j 4 > /tmp/full.log 2>&1; echo "EXIT=$?"

# release shell(fixture 窓)
cargo build --manifest-path /Users/member_ottoto/rust_ae/Motolii/next/Cargo.toml --release -p motolii-shell -j 4

# fixture 窓の起動(バイナリは絶対パス)
/Users/member_ottoto/rust_ae/Motolii/next/target/release/motolii-shell --fixture

# storm/r2 の無罪確認(負荷 flake — release 単独)
cargo test --manifest-path /Users/member_ottoto/rust_ae/Motolii/next/Cargo.toml --release -p motolii-store --test document edit_storm_with_the_real_track_type
```

### 既知の構造ギャップと改善ルート(未着手 — 変更時はここを更新)

1. **レーン worktree が毎回 cold**: 裁定138 は「常設 warm worktree・使い捨て禁止」を定めるが、subagent の isolation:worktree は毎回新規作成= target 空。フル10分の正体。ルート: 役割別常設 worktree の再利用 or 発注時に main の target/ をコピーして持たせる(共有と違い書き戻らない)
2. **nextest**: リンク律速には効かず却下(裁定138)だが、**別問題**(tier 分割を1回のビルドから filterset で切る・既知 flake 名指し retries)には適合(調査 §3)。0.9.143 導入済み。採否は利用者裁定待ち
3. (完了済みの参考)shell test バイナリ統合 10本→2本はレーン A で着地済み(フルリンク 45.5s→18〜26s、[レーンボード](docs/reviews/2026-08-21-lane-board.md))。`depth_offset` 極端値による外周1px縮みはレーン B と BL3 で**2度**出た同型バグ — 極端値を使わない(`background_rect` doc)

正本: [ビルド速度の調査](docs/reviews/2026-08-19-build-speed-investigation.md)・[静的検収調査](docs/reviews/2026-08-22-static-acceptance-survey.md)・[next/DECISIONS.md](next/DECISIONS.md) 裁定138。レーン運用の実測則は [next/reference/KNOWN.md](next/reference/KNOWN.md) の「レーン運用」節。**ビルド/検収の知見はこのファイルと上記正本にだけ追記する(新文書を増やさない)。ビルド系の調査・発注をする前に必ずこの節を読ませる。**
