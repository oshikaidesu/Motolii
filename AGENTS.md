# AGENTS.md

## 読む前に — 繰り返し踏んだ罠(2026-08-23)

**ここを飛ばすと同じ穴を踏みます。** 経緯は [教訓](docs/reviews/2026-08-23-lessons.md)。

**「数」の列は、その場で数え直せる物だけを置く。** 数え直せない物は下の「判断」節へ。

| 罠 | 対処 | 数(再現できる物だけ) |
|---|---|---|
| **背景で cargo を投げて待つとレーンが止まる** | **前景で・`timeout 600000`。`&`/`run_in_background`/`Monitor` を使わない。監督役も「待つ」を選択肢から外す** | 停止 **15件**(1日・通知から計数) |
| **台帳が実装より古くなる** | **機械が出せる列を人が書き写さない**(生成+柵)。触ったら再生成 | 1日25件。**全部この型**だった |
| **「確認する」を成果物にする** | **OUTCOME は状態の変化で書く**。可能なら機械で判定できる形に(裁定221) | 次の一手が調査になり調査で終わった |
| **「実装が在る」を「利用者に届く」と読む** | `済` は**実機で確認した物だけ**(裁定219) | 4件が「済」なのに動かなかった |
| **自前で書く** | **先に外を探す**(裁定215)。自前解析 17,488行 → rustdoc JSON へ置換で取りこぼし2,078項目が判明 | 自前版は doc コメントを呼び手に数えていた |
| **その場の判断で並列を作る** | **write-set が交わらないことを計算する**(`scripts/plan_waves.py`) | `shell/lib.rs` が 6,228行・責任指名102回 |
| **勘で優先順位を決める** | **導出する**(`scripts/check_evidence.py`)。「効きそう」は人の手が残っている証拠 | 選んだ軸が実際には最下位だった |
| **壁を上へ投げる** | **外部資料から決めて出典を書く**(裁定222)。利用者へ上げるのは実通の判定だけ | — |

### 判断(測っていない。根拠は経験のみ)

- **文章で禁止しても止まらない。復帰の定型文を送るほうが効く** — 停止15件はすべて定型文で復帰したが、
  「定型文だから復帰した」は**対照を取っていない**。効いた印象があるだけ
- **副監督が価値を出したのは「発注前の実測」であって、レーン管理ではない** — 比較していない
- **階層を1段深くすると停止点が倍になる** — 2層で監督役も止まったのは事実だが、**倍という数は言葉の綾**

**発注書を書く前に [教訓](docs/reviews/2026-08-23-lessons.md) を読むこと。**


Repository-specific agent *conduct* rules were archived in [docs/archive/agent-governance/](docs/archive/agent-governance/) and are not active. The only section below is build/test operations, kept here by user decision (2026-08-21) because every lane pays the cost of not knowing it.

## 段階(`next/reference/generated/steps.md`)

**「普通のモーショングラフィックが出来る」までの階段。** 手順書(`procedures/P*.md`)の
節をそのまま段階とし、各手順の判定を数えたもの。**段階を発明していない。**

```bash
python3 scripts/plan_steps.py "$(git rev-parse --show-toplevel)"
```

**次にやる仕事は「まだ静通していない最も早い段階」から選ぶ。**
これが「先回りで無い機能を作らない」の機械的な担保(利用者裁定 2026-08-23)。

「通る」は2種類:
- **静通** = 穴(入口が無い/意味が無い)がゼロ。**レーンが到達できる上限**
- **実通** = `【未確認】` もゼロ。**窓を開けないと出ない = 利用者の検分でしか付かない**

静通を先に全段階そろえ、**実機確認は最後にまとめて1回**にする。

**静通できない壁に当たっても利用者へ上げない**(裁定222)。**外部資料から決める** —
4製品(AE/Premiere/Resolve/CapCut)と Lottie 地図等の一次資料を当たり、
**採った作法と出典を書く**。先例が割れていても止まらず、裁定151(人口の多数決)で
決めて根拠を残す。出典が無ければ「出典なし」と正直に書く。
**利用者へ上げてよいのは実通の判定(窓を開けた UX 合否)だけ。**

## 入口の判定は導出する(`scripts/derive_entries.py`)

`Intent` の各枝に**入口が在るか**は実コードから導ける — `next/ui`・`next/shell` の
非テストコードが `Intent::X` を参照していれば入口あり。手書きに残すのは
**「なぜ穴か」「どう直すか」「責任」**だけ。

```bash
python3 scripts/derive_entries.py "$(git rev-parse --show-toplevel)"
```

台帳(`next/reference/axis/A01-entry.tsv`)と食い違うと
`next/core/motolii-testkit/tests/entries_fence.rs` が落ちる。**実コードが正**なので
台帳側を直す。導入時に3件の腐りが出た(`SetPropertySlot`/`SetPropertyModulators`/
`SetSlots` が「穴」のままだった)。

## 在庫表(`next/reference/generated/inventory.tsv`)

Motolii が今持っている物の全在庫(5,134項目)。**手で編集しない。**

```bash
bash scripts/gen-inventory.sh
```

**rustdoc が公式に吐く JSON を読み替えるだけ**で、構文解析を自前でしない
(2026-08-23 利用者裁定「外部に答えはあります」— 自前の `syn` 解析 17,488行は
撤去済み。テキスト一致で呼び手を数えて doc コメント内の言及まで拾う誤りを踏んだ)。
nightly が要る(rustdoc JSON は nightly 限定。active である必要はない)。

**`callers` 列は持たない。** 呼び手ゼロの検出はコンパイラの担当:
`[workspace.lints.rust] unreachable_pub = "warn"` で公開面を絞ると、既定の
`dead_code` が未使用を自動で出す。台帳へ書き写す工程は無い。

## ビルド/テストを回す時(裁定138・実測済み)

- **常設 warm worktree**(役割別・`target/` 温存・使い捨て禁止)+ **レーンごとの `-p` 集合固定**。`-p` の集合が変わると feature unification で再ビルドを踏む(実測: 固定 1.45s / 変動 28.6s)。合格基準線は 16〜33s
- **魔法フラグは無い**: sccache / cranelift / `-Zthreads` / lld / nextest は律速(リンク)に効かないため理由つき却下済み。`debug=line-tables-only` は設定済み
- **stash 禁止**(worktree 間で共有される)/ **`CARGO_TARGET_DIR` 共有禁止**(後勝ち事故の実測あり)/ Edit 直後の stale fingerprint は touch で解消
- レーン開始前に常駐プロセス(fileWatcher 等)を確認する(ビルド計測の汚染源、実測2例)
- cargo を**背景実行にしない**(subagent が自停止する既知事故)

### 検収の3段(2026-08-22 実測 — [静的検収調査](docs/reviews/2026-08-22-static-acceptance-survey.md))

- 実測定数(next/ 22crate・`-j 4`): `cargo check --workspace` = **warm 1.4s / cold 50s**。フル `cargo test --workspace --locked --no-fail-fast` = **warm 100s / cold 607s**。壁時計はキャッシュ状態で6倍動くので**実行時間を合否の物差しにしない** — 段の使い分け(機会)で裁く
- 段の使い分け: doc のみ=check-docs.sh だけ / pane 局所=check+該当pane と shell の suite / store・engine 跨り・fork pin bump・merge 境界=フル必須(判断表は調査 §6)
- **追いつきターンの波運転(裁定189、2026-08-22)**: 意味既決の消化フェーズでは、レーンの検収線を `cargo check --tests -p <crate>` まで(テストは書くが実行は後送 — 落ちるテスト先行の「書く」は維持)。supervisor が波単位で merge を束ね、一括 cargo test の**必然の関門は2つだけ**: (1) **消費点の手前**(実窓・push・引き継ぎ — ここは必ず緑) (2) **前提結合のある発注の手前だけ該当 suite**(結線レーン・同領域の続編など「前波の実行時挙動」の上に積む場合のみ。map 起点の独立束は該当しない — スコープは台帳由来で実行時の赤に汚されない)。それ以外のタイミングは正しさでなく**コスト最適化**(CPU の空きで安く回す)— エラーは決定論的に再現するので放置で消えも腐りもせず、bisect は worktree 消滅後も機械的に動く。赤は fix-forward(log₂N 有界)。レーン毎の cold テスト税(〜10分)が波1回に集約される。新しい意味論・store/engine 跨りの束は従来の即検収に戻す
- `-p` サブセットの素朴運用は warm フルより遅くなる(199s vs 100s — 上記「`-p` 集合固定」の再発見。**この節を読まずにビルド調査を始めるとこの再発見を繰り返す**)
- 時間予算試験(storm・r2)は debug+並列で走らせる事自体が矛盾 — 合否確認は単独 or release で
- **合否の exit code をパイプ越しに取らない**: `cargo … | tail` は合否を殺す(前任の check-docs 事故)。zsh では `$PIPESTATUS` は空(bash 綴り — zsh は `$pipestatus`)で「検証したつもり」になる(2026-08-22 に2回実測)。**リダイレクトで log へ落とし `$?` を直接見る**のが唯一安全

### 頻出コマンド(コピペ用 — 記憶から組み立てない。2026-08-22: cd 位置の誤り5連発の根治)

正本 workspace は `next/`(リポ根の Cargo.toml は旧 workspace で `motolii-shell` を含まない)。**`--manifest-path` で cwd 依存を消す** — 背景実行はシェルの cd 履歴に関わらずリポ根で走るため、cd 前置は事故源。パスは `$(git rev-parse --show-toplevel)` で**自分のいるツリーの根**に解決させる — レーンは自分の worktree・supervisor は main checkout と自動で正しく分かれ(絶対パス直書きだと worktree から main を誤ビルドする)、リポ移動にも生き残る。ハードコードは `next/` の1語のみ(構成が変われば本節を更新):

```bash
# フル関門(merge 前最終)— 合否は $? 直取り
cargo test --manifest-path "$(git rev-parse --show-toplevel)/next/Cargo.toml" --workspace --locked --no-fail-fast -j 4 > /tmp/full.log 2>&1; echo "EXIT=$?"

# fixture 窓のビルド(preview profile = release同等opt+incremental。初回のみ遅い)
cargo build --manifest-path "$(git rev-parse --show-toplevel)/next/Cargo.toml" --profile preview -p motolii-shell -j 4

# fixture 窓の起動(supervisor が main checkout から)
"$(git rev-parse --show-toplevel)/next/target/preview/motolii-shell" --fixture

# storm/r2 の無罪確認(負荷 flake — release 単独)
cargo test --manifest-path "$(git rev-parse --show-toplevel)/next/Cargo.toml" --release -p motolii-store --test document edit_storm_with_the_real_track_type
```

### 既知の構造ギャップと改善ルート(未着手 — 変更時はここを更新)

1. **レーン worktree が毎回 cold**: 裁定138 は「常設 warm worktree・使い捨て禁止」を定めるが、subagent の isolation:worktree は毎回新規作成= target 空。フル10分の正体。ルート: 役割別常設 worktree の再利用 or 発注時に main の target/ をコピーして持たせる(共有と違い書き戻らない)
2. **nextest**: リンク律速には効かず却下(裁定138)だが、**別問題**(tier 分割を1回のビルドから filterset で切る・既知 flake 名指し retries)には適合(調査 §3)。0.9.143 導入済み。採否は利用者裁定待ち
3. (完了済みの参考)shell test バイナリ統合 10本→2本はレーン A で着地済み(フルリンク 45.5s→18〜26s、[レーンボード](docs/reviews/2026-08-21-lane-board.md))。`depth_offset` 極端値による外周1px縮みはレーン B と BL3 で**2度**出た同型バグ — 極端値を使わない(`background_rect` doc)

正本: [ビルド速度の調査](docs/reviews/2026-08-19-build-speed-investigation.md)・[静的検収調査](docs/reviews/2026-08-22-static-acceptance-survey.md)・[next/DECISIONS.md](next/DECISIONS.md) 裁定138。レーン運用の実測則は [next/reference/KNOWN.md](next/reference/KNOWN.md) の「レーン運用」節。**ビルド/検収の知見はこのファイルと上記正本にだけ追記する(新文書を増やさない)。ビルド系の調査・発注をする前に必ずこの節を読ませる。**

### glam の `inverse()` は自己アサートする — 呼ぶ前に `determinant()` を見る(2026-08-22 実測)

ワークスペースのどこかの依存が glam の `debug-glam-assert`/`glam-assert` feature を
有効化しており、feature unification で**全体に効いている**。その結果:

`Mat2::inverse()`(`glam-0.30.10`)は**結果を返す前に** `glam_assert!(...is_finite())`
で自己アサートする。つまり「`inverse()` を呼んでから `is_finite()` で後始末する」形の
ガードは、**そのガードへ到達する前に panic する**。実害の例:
Scale X = 0 のレイヤーで Stage の Anchor Point ハンドルを掴むと debug ビルドで落ちた
(release では `is_finite()` ガードが機能して「偶然」動いていた — 潜在的な地雷)。

**正しい形**: `determinant()` を先に見て、非有限または 0 なら `inverse()` を**呼ばない**。

```rust
let det = m.determinant();
if !det.is_finite() || det == 0.0 { return fallback; }
let inv = m.inverse();
```

`Mat2` から回転/せん断だけを取り出した行列(det=1 が保証される物)は例外だが、
**保証の根拠をその場に書くこと**。

