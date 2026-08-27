# AGENTS.md

## この repo が持っている物(使うかどうかは判断してよい)

**手順は指定しない。** 事実を安く手に入れる道具だけ置いてある:

```bash
python3 scripts/plan_steps.py     "$(git rev-parse --show-toplevel)"   # 次にやる段階と、その障害
python3 scripts/plan_backlog.py   "$(git rev-parse --show-toplevel)"   # 残作業983件と割り振り
python3 scripts/check_foundation_phase.py "$(git rev-parse --show-toplevel)" # 基盤段階と並列解禁状態
python3 scripts/plan_waves.py     "$(git rev-parse --show-toplevel)"   # write-set が交わらない組
python3 scripts/rehearse_parallel.py "$(git rev-parse --show-toplevel)" # 意味レーンを同時実行して隔離を検査
python3 scripts/derive_entries.py "$(git rev-parse --show-toplevel)"   # 入口が在るか(実コードから)
python3 scripts/check_evidence.py "$(git rev-parse --show-toplevel)"   # 台帳の証拠が実在するか
bash    scripts/gen-inventory.sh                                       # 何を持っているか(5,150項目・7列目に署名)
python3 scripts/check_coherence.py   "$(git rev-parse --show-toplevel)" # 台帳どうしが食い違っていないか
python3 scripts/rank_load_bearing.py "$(git rev-parse --show-toplevel)" # 荷重(壊すと巻き添えが多い所)
python3 scripts/derive_components.py "$(git rev-parse --show-toplevel)" # コンポーネント契約から意味の粒と赤/緑を導出
python3 scripts/check_responsibility.py "$(git rev-parse --show-toplevel)" # WIREが意味を書き込んでいないか
python3 scripts/derive_technical_delegation.py "$(git rev-parse --show-toplevel)" # 技術の委託先とスクラッチ境界を導出
python3 scripts/check_technical_delegation.py "$(git rev-parse --show-toplevel)" # 技術委託台帳のjoin・証拠・語彙を検査
python3 scripts/check_icebook_panel_drafts.py "$(git rev-parse --show-toplevel)" # Icebook向けパネル草案の件数・必須欄を検査
python3 scripts/derive_icebook_panel_stories.py "$(git rev-parse --show-toplevel)" # パネル草案をIcebook story索引へ導出
```

**現在地は上が出す。引き継ぎ文書に依存しない。** 段階状態は
`next/reference/foundation/phase.json`、作業割りは生成台帳、現在の意味はコードと証拠が出す。
**会話の文脈が無くても、ここから構造を辿って再開できる。**

## 完了の定義(Done when)

**これが揃っていない実装は未完成。** 手段は問わない:

1. **段階が1つ進む** — `plan_steps.py` の「静通」が増える。増えないなら、それは今やる仕事ではない
2. **出典** — なぜその作法か。製品名・URL・版。無いなら「**出典なし**」と明記(捏造しない)
3. **証拠** — `file:line`。`check_evidence.py` が実在を検査する
4. **検収条件1つ** — 「何が起きたら通ったか」。**テストはこれだけでよい**
5. **柵が緑** — `cargo test -p motolii-testkit`(台帳の柵5本)

## 速さについて分かっていること(規則ではない。実測)

- `cargo check --tests` はフルテストの数分の1。**型で足りる変更に `cargo test` を使うと遅くなる**
- **背景実行の完了待ちでターンを終えると、そこで止まる**(1日15件、通知から計数)。前景が速い
- **write-set が交わる作業を同時に動かすと、後で1本ずつ直す羽目になる**
  (`shell/lib.rs` が 6,228行・責任指名102回になった)
- **段階が要求しない機能を先に作ると、どこに必要かが誰にも分からなくなる**(983件がその状態だった)

## 誰を信じるか(2026-08-23 利用者裁定)

| | 権威 | 権威でない |
|---|---|---|
| **利用者** | **窓を開けた UX の合否だけ**(触って気持ち悪い/使えない) | **事実・意味・設計。利用者は一ユーザーであり、前提が外れていることに気づかない立場にいる** |
| **外部資料** | **意味の正本**。4製品(AE/Premiere/Resolve/CapCut)・Lottie 地図・Rive・各社公式ドキュメント | — |
| **機械** | **現在地**(上の4コマンド)と**赤/緑**(柵・型) | — |
| **この repo の文書** | **仮説**。機械と食い違ったら**機械が正しい** | 権威ではない |
| **製品 front** | **Makepad**(`next/probes/r7-makepad-panel`)。裁定251/252 | **`motolii-shell` crate**（凍結 iced アセンブラ）。view/update とも製品 interface ではない |

**利用者の指示は「目的」であって「事実」ではない。** 前提が外れていると思ったら、
**確かめてから進む**(止まらない)。確かめ方は上の4コマンドか `grep` で足りる。

意味の正本は `motolii-store` / `motolii-shell-state` / `motolii-engine`。
`motolii-shell` は iced 窓のアセンブラであり、製品核ではない(裁定253/254)。
製品 front はこれを引かない。`next/README.md` の「front は iced のみ」は裁定251が覆した。再導出しない。

**壁に当たっても上へ聞かない**(裁定222)。**外部資料を引いて自分で決め、出典を書く。**
先例が割れていても止まらず、人口の多い作法を採って**採らなかった理由も残す**。
出典が見つからなければ「**出典なし**」と正直に書く(捏造しない)。

## 報告と対話の作法

**Codex の system prompt は「利用者のスタイルを鏡映せよ」と指示している。** よってここに
書いた作法は、禁止ではなく**鏡に映す像**として働く。以下は 2026-08-23 の長い実作業で
テンポが良かった側の書き方を、そのまま移したもの(**効果は未検証**。この会話が唯一の根拠)。

- **訂正は簡潔に。** 謝罪や自己批判を重ねない。反省を長く書かない。直して次へ進む
- **懸念は1〜2文述べて、そのまま作業を続ける。** 止まらない
- **利用者が再確認したら、それは決定。** 議論を続けず、全部やる
- **確認するのは「解釈が違うと作業が実質的に変わる」時だけ。** それ以外は決めて進む
- **検証済みなら断定する。** ヘッジしない。**測っていないなら、そう書く**
- **既に確立した事実を再導出しない。決着した決定を蒸し返さない**(テンポの正体はほぼこれ)
- **失敗は出力つきで正直に報告する。** 赤は隠さない。**自分のミスも同じ粒度で書く**

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

**生成物(`next/reference/generated/`)は手で編集しない。** 触ったら再生成する。
生成器が何を出すかは冒頭のコマンド一覧を参照。

## cargo は何のために回すか

**「何が在るか・どう呼ぶか・どう組むか」を cargo に聞かない** — 在庫表の7列目に型がある
(**関数2,479・構造体フィールド1,165・enum バリアント922**)。適当に書いて `cargo` で
確かめて繋ぎ直すのは**cargo を検索に使っている**。

```bash
# 構造体リテラルを書く前に、フィールドと型を引く
awk -F'\t' '$1=="struct_field" && $4 ~ /asset\.rs/ {print $2, $7}' next/reference/generated/inventory.tsv
# enum の腕を書く前に、バリアントの形(unit/tuple(n)/struct{n})を引く
awk -F'\t' '$1=="variant" && $4 ~ /document\.rs/ {print $2, $7}' next/reference/generated/inventory.tsv
```

**今日の取り残し2件はどちらも構造体フィールドと enum バリアントだった**
(`Asset` に `status` が増えて構造体リテラルが壊れた・`ClipContextMessages` に `split` が
増えて試験が壊れた)。**関数の署名だけでは足りない。**

cargo でしか出ないのは **(a) 網羅性**(`match` の腕・enum にバリアントを足した時。
今日の取り残し検出はこれが担った)と **(b) 借用・生存期間** の2つだけ。
よって **`cargo check --tests` は「書き終わってから1回」**(裁定228)。

## ビルド/テストを回す時(裁定138・実測済み)

- **常設 warm worktree**(役割別・`target/` 温存・使い捨て禁止)+ **レーンごとの `-p` 集合固定**。`-p` の集合が変わると feature unification で再ビルドを踏む(実測: 固定 1.45s / 変動 28.6s)。合格基準線は 16〜33s
- **魔法フラグは無い**: sccache / cranelift / `-Zthreads` / lld / nextest は律速(リンク)に効かないため理由つき却下済み。`debug=line-tables-only` は設定済み
- **stash 禁止**(worktree 間で共有される)/ **`CARGO_TARGET_DIR` 共有禁止**(後勝ち事故の実測あり)/ Edit 直後の stale fingerprint は touch で解消
- レーン開始前に常駐プロセス(fileWatcher 等)を確認する(ビルド計測の汚染源、実測2例)
- cargo を**背景実行にしない**(subagent が自停止する既知事故)

### 検収の3段(2026-08-22 実測 — [静的検収調査](docs/reviews/2026-08-22-static-acceptance-survey.md))

- 実測定数(next/ 22crate・`-j 4`): `cargo check --workspace` = **warm 1.4s / cold 50s**。フル `cargo test --workspace --locked --no-fail-fast` = **warm 100s / cold 607s**。壁時計はキャッシュ状態で6倍動くので**実行時間を合否の物差しにしない** — 段の使い分け(機会)で裁く
- 段の使い分け: doc のみ=check-docs.sh だけ / pane 局所=check+該当pane と shell の suite / store・engine 跨り・fork pin bump・merge 境界=フル必須(判断表は調査 §6)
- **追いつきターンの波運転(裁定189・233)**: レーンの検収線は静的検査で、`inventory` / `derive_components` / `derive_entries` / `check_coherence` / `plan_steps` / `plan_waves` / `diff --check` で赤を閉じる。テストは書くが、各エージェントは通常 cargo を回さない。supervisor が波単位で merge を束ね、cargo は波末または消費点で影響範囲をまとめて1回だけ回す。必然の関門は (1) **消費点の手前**(実窓・push・引き継ぎ) (2) **前提結合のある発注の手前だけ該当 suite**(結線レーン・同領域の続編など前波の実行時挙動に依存する場合)。例外は **enum/match の網羅性・公開型境界・借用/生存期間・消費点の実行時挙動**で、該当時も各レーンが個別に回さず supervisor の門へ送る。これで cargo の lock 待ちを並列の上限にしない
- **責任境界(裁定234)**: `//! responsibility: wire` を持つファイルは意味レーンから除外し、WIRE結線へ送る。`plan_waves.py` は意味write-setだけで連結を作り、外部依存と責任未記入を別々に報告する。WIREへ意味の書き込みを足さないことは `check_responsibility.py` が検査する
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

### UI修正はホットリロード運転(2026-08-27 利用者指示 — 「毎回ビルド」の根治)

r7 の UI 調整で**変更ごとに `cargo build`→再起動をしない**。窓はセッションで1回だけ起動:

```bash
cargo run --locked --manifest-path next/probes/r7-makepad-panel/Cargo.toml -- --hot --remote > /tmp/r7.log 2>&1 &
```

- `--hot` は makepad 本体の live reload: `script_mod!` を持つ `src/*.rs` の保存が
  再ビルド無しで窓に届く(`*_surface.rs` / `chrome/*.rs` / `main.rs` すべて)。
  以後のループは「保存 → `--remote` の `/g` `/snap` で確認」だけ
- `--hot` と `--remote` は独立の引数チェック(platform/live_reload.rs)なので併用可。
  リポ根から起動する(リソースパス解決)
- `--hot` は Rust の `const` に届かない。繰り返し詰める視覚定数は `#[live]` フィールド+
  宣言に置く(r7 README「調整する値は script_mod! に置く(裁定269)」)
- 変更前に makepad skills を読む: makepad-2.0-design-judgment → 該当 compliance skill
- `--remote` 運転の正本は `~/rust_ae/makepad-motolii/AGENTS.md`。利用者の窓には触らない・
  自分が開けた窓は終わったら落とす

### 実窓を見る時(2026-08-25 実測 — computer-use 迂回の根治)

- **Makepad の窓は macOS のアクセシビリティに応答しない**。computer-use の `get_app_state` はタイムアウトする。`.app` ラッパーへ包み直しても変わらない(2026-08-25 に両方実測)。**AX が返らないのは実装の欠陥ではなく Makepad の性質**なので、ここから原因調査を始めない
- **全画面 `screencapture -x` を証拠にしない** — 窓が画面のどこにあるか読めず、利用者からは「見えない」。窓単体を撮る:

```bash
# 窓IDを引く(owner 名で絞る。プロセス名の一部を渡す)
swift -e 'import CoreGraphics
let info = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as! [[String: Any]]
for w in info { let o = w[kCGWindowOwnerName as String] as? String ?? ""
  if o.contains("r7-makepad") { print(o, w[kCGWindowNumber as String] ?? "") } }'

# その窓だけを撮る
screencapture -x -l <窓ID> /tmp/window.png
```

- **GUI の起動プロセスを待たない**: 窓は終了しないので `yield_time_ms` 付きの待ちは丸ごと空振りする(2026-08-25 に30秒×6回=3分を捨てた)。起動は投げっぱなしにし、存在確認は `pgrep -af`、結果確認は上のキャプチャで取る
- **実窓で時間を測るなら debug バイナリを使わない**(2026-08-25 実測の失敗): `./target/debug/` のまま計測すると支配項が偽装される。実例 — `ImageBuffer::new`(1920×1080 RGBA = 8.3MB のゼロ埋め)が **46.578ms** と出て「CPU 画像経路が主因」と結論されたが、これは約180MB/s = 最適化なしの1バイトずつゼロ埋めの速度で、release では `memset` になり1ms未満。57.654ms の中央値のうち**81%が debug artifact の疑い**だった。「検収の3段」の『合否確認は単独 or release で』は実窓の計測にもそのまま効く。**窓の計測は `--profile preview` で取る**(release同等opt+incremental、この用途のために既に用意してある)
- **計測値を根拠に構造判断へ進む前に、桁が物理的に妥当か1度問う**: 8MB のメモリ操作が数十msなら、それは処理コストではなくビルド設定を測っている

- computer-use を触るのは**操作そのもの(ドラッグ・hover・押し分け)が検査対象の時だけ**。見た目の確認は上の2コマンドで足りるので、`computer-use/SKILL.md`(211行)を読む必要はない(2026-08-25 は1セッションで9回読み直し、圧縮5回の主因になった)

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
