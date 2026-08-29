# 並列レーンがビルドしてよい仕組みの現在地(2026-08-29)

状態: **観察**。stage4のビルド不変量(1 target同時共有禁止・レーンごとの冷ビルド禁止、
`motolii/AGENTS.md`)を守ったままレーンにビルドさせる、その時点の道具の台帳。

利用者の指摘「並列レーンがビルドしてもいい仕組みはおそらく生まれているはず——
並列実装はLLM時代では普通だから」を受けた調査。当たっていた。

| 層 | 仕組み | 現在地 |
|---|---|---|
| 家内実測 | warm targetのAPFS CoWクローン(レーンは複製の上で差分ビルド、捨てる) | [ビルド律速調査](2026-08-19-build-speed-investigation.md)で採用済み。差分コストのみ |
| コミュニティ | [worktrunk](https://github.com/max-sixty/worktrunk)(Rust製worktreeマネージャ、2026年初出・7月時点5.8k stars、エージェント並列用の最普及) | hookでworktree間のbuild cacheコピーを自動化=CoWパターンの製品化。並列4〜8 worktree/人が普通の運用として成立している時代の傍証 |
| 上流Cargo | [cross-workspace cache](https://rust-lang.github.io/rust-project-goals/2026/cargo-cross-workspace-cache.html)(path非依存・内容ハッシュのuser-wide共有キャッシュ) | **2026年project goal、年内にnightly実験予定**。前提の`build-dir`分離は安定化済み([Layout v2 call for testing](https://blog.rust-lang.org/2026/03/13/call-for-testing-build-dir-layout-v2)、1.96で常時有効)。これが着けば複製すら不要になる本命 |

## 実測(同日、wt v0.75.0導入+スモークテスト)

- `wt switch smoke-lane --create`+`pre-start = "wt step copy-ignored"`
  (除外: `app/target/`38GB・`.claude/`)で、motolii/target 6.4GBのreflink複製が
  67秒・**ディスク消費ゼロ**(空き1.2TiB不変)
- ただしレーン初回の`cargo build --release`は**21分**——cargoのfingerprintが
  絶対パスを含むため、パスの変わった複製targetは大部分が無効化される
  (sccacheがworktree跨ぎで効かないのと同じ物理)
- **結論: CoW複製はディスクを救うが初回時間を救わない。時間のwarmさはパスの
  安定性から来る**——[ビルド律速調査](2026-08-19-build-speed-investigation.md)の
  「常設warm worktree(役割別・同一パス・target温存・使い捨て禁止)」が引き続き正
- 運転の型: レーンは常設(`~/rust_ae/Motolii.smoke-lane`が投資済みの第1号)、
  worktrunkは管理(`wt switch`/`list`/`merge`)とhookを担い、copy-ignoredは
  新レーン新設時の1回きりの初期費用のディスク保護

再入場トリガー: cargo cross-workspace cache(path非依存)がnightlyに載ったら試す。
安定化したら初回21分も消え、常設制約を緩められる。
