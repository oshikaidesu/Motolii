# AGENTS.md

## まず1つだけ選ぶ(全部やらない)

**この repo は生成器を持っているが、毎回全部を撃たない。** 頼まれた事に合う入口を1つだけ選ぶ:

| 頼まれた事 | やること |
|---|---|
| **雑談・質問・「これ何?」** | **何も実行しない。** 答えるだけ。コマンドを撃たない |
| **「今どうなってる?」** | `python3 scripts/plan_steps.py "$(git rev-parse --show-toplevel)"` **1本だけ** |
| **実装する / 直す** | 同上で**段階を1つ選び**、その段階の障害だけを塞ぐ。**他は触らない** |
| **調べる** | **先に結論を書く場所を決める**(`docs/reviews/<日付>-<主題>.md`)。書きながら調べる |
| **壊れている** | 赤の出力をそのまま読む。**推測で直さない** |

**手を動かす前に、それが今の段階に要るかを確かめる。** 要らないなら**やらない**のが正解
(`plan_backlog.py` に983件あるが、**段階が呼ぶまで着手しない** — 先回りは速度を下げる)。

### 急ぎたくなったら、行き先を変える

**「実行したい」を止めようとすると戻ってくる**(コミュニティの実測)。**そのまま書く物へ流す**:

- 直したい → **どの段階のどの手順か**を先に1行書く。書けないなら、まだ直す番ではない
- 大きく直したい → `docs/reviews/` に**結論から**書く。設計文書が先、実装は後
- 全部直したい → **write-set を計算する**(`scripts/plan_waves.py`)。交わるなら1本ずつ

### 一度立てた計画に固執しない

**新しい指示が来たら、前の計画は捨てる。** 「元の実装を続ける」は既知の失敗型
(コミュニティで繰り返し報告されている)。**指示が変わった = 前提が変わった**ので、
段階の選び直しから始める。

## 誰を信じるか(2026-08-23 利用者裁定)

| | 権威 | 権威でない |
|---|---|---|
| **利用者** | **窓を開けた UX の合否だけ**(触って気持ち悪い/使えない) | **事実・意味・設計。利用者は一ユーザーであり、前提が外れていることに気づかない立場にいる** |
| **外部資料** | **意味の正本**。4製品(AE/Premiere/Resolve/CapCut)・Lottie 地図・Rive・各社公式ドキュメント | — |
| **機械** | **現在地**(上の4コマンド)と**赤/緑**(柵・型) | — |
| **この repo の文書** | **仮説**。機械と食い違ったら**機械が正しい** | 権威ではない |

**利用者の指示は「目的」であって「事実」ではない。** 前提が外れていると思ったら、
**確かめてから進む**(止まらない)。確かめ方は上の4コマンドか `grep` で足りる。

**壁に当たっても上へ聞かない**(裁定222)。**外部資料を引いて自分で決め、出典を書く。**
先例が割れていても止まらず、人口の多い作法を採って**採らなかった理由も残す**。
出典が見つからなければ「**出典なし**」と正直に書く(捏造しない)。

## 意味負債を作らない(後から見て即分かる形で置く)

実装1つにつき、**この3つが揃っていないなら未完成**:

1. **出典** — なぜその作法なのか。製品名・URL・版。無いなら「出典なし」と明記
2. **証拠** — `file:line`。台帳に書いたら `check_evidence.py` が実在を検査する
3. **検収条件** — 「何が起きたら通ったのか」を1つ。**テストはこれだけでよい**(裁定218)

**書いてはいけない物**: 測っていない数字を数の列に置く(裁定223)。
判断は「判断(測っていない)」と明示した**別の節**へ出す。**注釈では足りない** —
表と数字は末尾に何を書いても測ったように読めるから。

**生成物(`next/reference/generated/`)は手で編集しない。** 触ったら再生成する。

## 爆速で回す(2026-08-23 実測で効いた物だけ)

- **既定の関門は `cargo check --tests`。`cargo test` は検収条件1点だけ**(裁定220)
- **cargo は前景で・`timeout 600000`。背景実行(`&` / `run_in_background` / 完了通知待ち)を使わない** —
  待つとそこで止まる(1日15件、通知から計数)
- **並列は write-set が交わらないことを計算する**(`scripts/plan_waves.py`)。
  その場の判断で並列を作ると、近いファイルに責任が積まれる(`shell/lib.rs` が 6,228行になった実績)
- **先回りで作らない** — `plan_steps.py` が出す「まだ静通していない最も早い段階」だけをやる。
  残作業983件の割り振りは `plan_backlog.py` が持っているが、**段階が要求するまで着手しない**
- **自前で書く前に外を探す**(裁定215)。自前の構文解析17,488行 → rustdoc JSON へ置換して
  取りこぼし2,078項目が判明した実績がある。**探索範囲を必ず書く**

## 「通る」は2種類

- **静通** = 穴(入口が無い/意味が無い)がゼロ。**あなたが到達できる上限**
- **実通** = `【未確認】` もゼロ。**窓を開けないと付かない = 利用者の席**

静通を先に全段階そろえ、**実機確認は最後にまとめて1回**にする。

## この repo の規約はここが正本(隠し場所へ書かない)

**エージェント向けの規律は、このファイルと `next/reference/` 配下(git で追える見える物)が正本。**
`~/.codex/` や `~/.claude/` のようなホーム配下の隠し場所へ**プロジェクト固有のことを書かない** —
目に見えない二重管理になり、**なぜその規則が在るのかを誰も辿れなくなる**。

2026-08-23 に実際そうなっていた物を撤去した:

| 隠し場所 | 何が積まれていたか |
|---|---|
| `~/.codex/rules/default.rules` | 一回限りの承認16件。`PR 176`・`issue 51`・**存在しない crate 名 `motolii-ui`** が焼き付き |
| `~/.codex/skills/hatch-pet` | 作業と無関係(85KB) |
| `~/.codex/skills/{ponytail,reuse-before-scratch}` | 空ディレクトリだけ |
| `.claude/settings.local.json` | 承認の allowlist **336件・うち172件が80字超の一回限り**(未整理) |

各隠し場所には「**ここには書かない・正本は repo 側**」という誘導を置いた。
**消しただけだと善意で復活する**ため。

## 生成器(手で編集しない。触ったら再生成)

| 出す物 | コマンド | 中身 |
|---|---|---|
| **次にやる段階** | `scripts/plan_steps.py` | 手順書(`next/reference/procedures/P*.md`)の節をそのまま段階にした階段。**段階を発明していない**。28段階 |
| 残作業と割り振り | `scripts/plan_backlog.py` | `normal-map.tsv` の `採用予定`/`結線待ち` × `bundle` → `intent-bundles.tsv` の `home` → crate。983件/9 crate |
| 並列可能性 | `scripts/plan_waves.py` | 責任ファイルを union-find で束ねる。**成分どうしは同時に走れる** |
| 入口が在るか | `scripts/derive_entries.py` | `Intent` の枝が `next/ui`・`next/shell` の非テストから参照されているか。**台帳と食い違うと柵が落ちる** |
| 証拠が実在するか | `scripts/check_evidence.py` | 台帳の `file:line` が今も在るか。**腐りの多い軸が次に機械化すべき軸** |
| 何を持っているか | `bash scripts/gen-inventory.sh` | rustdoc JSON から。5,134項目。**構文解析を自前でしない**(nightly が要る) |

**呼び手ゼロの検出はコンパイラの担当** — `[workspace.lints.rust] unreachable_pub = "warn"` で公開面を
絞ると、既定の `dead_code` が未使用を自動で出す。台帳へ書き写す工程は無い。

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

