# 検収の静的化調査 — cargo check / 純関数テスト / フル workspace の3段分離

日付: 2026-08-22 / 状態: **調査**(read-only・書き込みは本ファイルのみ) / 起点: [転写文法セッション引き継ぎ](2026-08-22-session-handoff-transcription-hierarchy.md)新queue「検収の静的化(利用者提起)」

対象: `next/`(正本 workspace、22 member — [Cargo.toml](../../next/Cargo.toml))。旧 `crates/`・`plugins/`・`ui/` workspace(main Cargo.toml)は対象外(2026-08-20裁定で正本を`next/`へ移し、旧workspaceへの新規投資は凍結済み)。実測は本 worktree(`next/target` は本セッション開始時点で空 = cold state)で実施、コード変更ゼロ。

## 0. 結論(先出し)

- **cargo check --workspace は cold でも約50秒、warm では1秒台**。フル `cargo test --workspace --locked --no-fail-fast` は cold で**約10分(607秒)**、target が温まった warm 実行でも**約100秒**。checkとフルの間に**12〜600倍の開き**があり、3段分離の投資対効果は実測上も大きい。
- 過去の検収ログ(lane-board・handoff)から実例を遡及すると、**型層(cargo check)が跨crate意味衝突を2件現に捕まえている**一方、**フルsuiteでしか出ない欠陥(static汚染・並列flake)も2件確認**でき、「checkで十分」ではなく「checkは前段フィルタ、フルは省略できない」が実態に近い。
- `cargo-nextest 0.9.143` は**既に本機にインストール済み**(未導入ではない — `cargo nextest --version` で確認)。filterset(`-E`)による crate/test 単位のtier分割はプロセス毎独立実行のため `-p` サブセット再ビルドより安全で、実測でも `-p` サブセットは**warm フルより遅い(199秒 vs 100秒)**という逆転が起きた(feature統合差分による再コンパイル)。nextestはこの再コンパイル問題を構造的に回避できる導入候補だが、**本調査では導入しない**(NON-GOAL)。
- 既知flake(storm・r2)は3回のフル/準フル実行中 **storm 2回・r2 1回** 発火し、docsの「並列時のみ赤・単独緑」という記述と整合。どの段でも決定的には捕まえられない性質(タイミング依存)であり、3段分離の判断表とは別軸として扱うべき。

## 1. workspace構成と依存グラフ(`cargo metadata --no-deps`実測)

`next/Cargo.toml` の22 member。層は `core/` → `engine/` → `ui/`(pane crate)→ `shell/`(統合)、`probes/` は独立探査用。

| crate | 層 | workspace内依存 |
|---|---|---|
| motolii-core | core | (葉) |
| motolii-eval | core | motolii-core |
| motolii-vector | core | (葉) |
| motolii-testkit | core | (葉、テスト専用) |
| motolii-store | core | motolii-core, motolii-eval, motolii-vector |
| motolii-compositor | engine | motolii-core |
| motolii-media | engine | motolii-core, motolii-store, motolii-testkit |
| motolii-audio | engine | motolii-core, motolii-eval, motolii-store, motolii-testkit |
| motolii-engine | engine | motolii-compositor, motolii-core, motolii-media, motolii-store, motolii-testkit, motolii-vector |
| motolii-export | engine | motolii-compositor, motolii-core, motolii-engine, motolii-media, motolii-store, motolii-testkit |
| motolii-tokens-rs | ui | (葉) |
| motolii-shell-state | ui | motolii-store |
| motolii-browser-pane | ui | motolii-store, motolii-tokens-rs |
| motolii-settings-pane | ui | motolii-store, motolii-tokens-rs |
| motolii-stage-pane | ui | motolii-core, motolii-engine, motolii-store, motolii-tokens-rs |
| motolii-timeline-pane | ui | motolii-shell-state, motolii-store, motolii-tokens-rs |
| motolii-inspector-pane | ui | motolii-core, motolii-settings-pane, motolii-shell-state, motolii-store, motolii-tokens-rs |
| motolii-shell | shell | **全て**(audio, browser-pane, core, engine, inspector-pane, media, settings-pane, shell-state, stage-pane, store, testkit, timeline-pane, tokens-rs) |
| r0〜r4(probes) | probe | store/compositor/engine/testkit を個別に薄く参照(r4のみ孤立) |

`motolii-store` が実質的な「土台」(core3crateを束ね、engine全体とui pane全体から参照される)。`motolii-shell` は依存の合流点で、**shellの変更は他へ波及しないが、他クレートの変更はほぼ必ずshellまで届く**(=shellのビルド/テストが「フル」の代理指標になりやすい)。

## 2. テスト分類(正規表現走査による概算)

`#[test]` / `#[tokio::test]` / `proptest!` の出現数を crate 別に集計し、ファイル内の `wgpu::Device` 系キーワード・`Command::new`/`ffmpeg` 系キーワードの有無で「GPU寄り」「プロセス起動寄り」の目印を付けた(**キーワード一致は誤検出を含む概算** — 例: `motolii-tokens-rs` の gpu_hint 1件は色定数コメントの可能性が高く未検証)。

| crate | test概算 | test付きファイル数 | GPU目印ファイル | プロセス目印ファイル |
|---|---:|---:|---:|---:|
| motolii-core | 59 | 7 | 1 | 1 |
| motolii-eval | 30 | 3 | 0 | 0 |
| motolii-store | 232 | 26 | 0 | 10 |
| motolii-testkit | 0 | 0 | 0 | 0 |
| motolii-vector | 67 | 5 | 0 | 0 |
| motolii-audio | 64 | 12 | 6 | 2 |
| motolii-compositor | 31 | 6 | 3 | 0 |
| motolii-engine | 43 | 12 | 1 | 2 |
| motolii-export | 5 | 3 | 0 | 2 |
| motolii-media | 31 | 7 | 0 | **7(全ファイル)** |
| motolii-browser-pane | 26 | 3 | 0 | 0 |
| motolii-inspector-pane | 50 | 4 | 2 | 0 |
| motolii-settings-pane | 12 | 1 | 1 | 0 |
| motolii-shell-state | 4 | 1 | 0 | 0 |
| motolii-stage-pane | 12 | 2 | 1 | 0 |
| motolii-timeline-pane | 79 | 8 | 1 | 0 |
| motolii-tokens-rs | 27 | 1 | 1 | 0 |
| motolii-shell | 239 | 33 | 10 | 4 |
| **合計** | **約1,011** | 134 | 27 | 28 |

参考値: 過去の検収ログでは「workspace 106スイート 910全緑」(M4検収、lane-board:95)「workspace 99スイート 872全緑」(τ検収、同:108)など、**map採用が進むたび母数が動いている**。本調査の約1,011はその後の増分を含む現時点値で、桁は一致(実測との相互検証OK)。

分類の実体を裏取りした3例:

1. **確実にGPU実体**: `motolii-compositor/src/headless.rs`(`HeadlessGpu::new`)が `wgpu::Instance::enumerate_adapters` + `request_device` で実アダプタを要求する(`next/engine/motolii-compositor/src/headless.rs:20-38`)。`compositor/tests/{compose,zero_copy,with_device}.rs` がこれを使う。macOS本機はMetalアダプタが常在するため実行できるが、**仮想GPUの無いheadless Linux CIでは通らない**構造(KNOWN「GPU実描画はheadless不可視」の裏付け)。
2. **確実にプロセス起動**: `motolii-media` 配下7ファイル全てが `ffmpeg-sidecar`(`Command::new` 相当)を経由する(裁定24でre_video不採用・ffmpegサイドカー採用のため)。`ag1_stream_probe.rs`・`roundtrip.rs`・`framereader_cancel.rs` は実ffmpegバイナリのspawnを要する。
3. **見た目GPUだが実体は薄い**: `motolii-shell/tests/suite/render_pipeline_fence.rs` の `presenter_generation` 系テスト(§4-2で詳述)はGPU描画そのものではなく生成カウンタのロジックを検証するが、**motolii-shellはworkspace全体を推移的に引く**ため、コンパイルコストは実質フルビルドと同等になる。「テストの中身が純関数かどうか」と「そのcrateをビルドするコストが軽いかどうか」は別軸。

## 3. cargo-nextest 調査

- **導入状況**: `cargo nextest --version` → `cargo-nextest 0.9.143`(2026-08-04ビルド、`aarch64-apple-darwin`)。**本機に既に導入済み**(ALLOWLIST上、この確認コマンド以外は未実行 — list/run はNON-GOALのため叩いていない)。
- **partition機能**(公開ドキュメント記載の仕様、本調査では未実行): `--partition count:M/N` でテストバイナリ単位のハッシュ分割、`--partition hash:M/N` も同様。これはCI matrix向けの「同じテスト集合をN台に割る」機能であり、**3段分離(check/pure/full)の意味的な線引きには使えない**(ランダム分割であって「GPU不要」の意味で切ってくれるわけではない)。
- **filterset(`-E`)**: `package(...)`, `binary_id(...)`, `test(...)` 等の式でテスト集合を絞れる。§2の crate分類をそのまま `-E 'not (package(motolii-media) or package(motolii-compositor) or package(motolii-audio) or ...)'` のような式に落とせば、**「pure tier」を意味的に定義できる**。重要なのは、nextestは**先に全テストバイナリを1回ビルドしてから**filterで実行対象を絞る(cargo test自体と同じビルド単位)ため、§4-3で実測した「`-p`サブセットがfeature統合の差でwarmフルより遅くなる」問題が原理的に起きない。
- **`--locked`相当**: nextestは`cargo test`と同じく`--locked`/`--frozen`/`--offline`をそのまま透過できる(cargo標準フラグのpassthrough)。ここは互換。
- **flake対策**: `--retries N`(またはprofile設定の`retries`)でテスト単位の自動再実行ができ、`.config/nextest.toml`の`[[profile.default.overrides]]`で**特定テストだけ**(例: `filter = "test(edit_storm_with_the_real_track_type)"`)に retries を当てられる。**全体に retries をかけて隠すのではなく、既知flake名指しでretries、その他は0のままにする**運用が可能 — これは「既知flake 2件」の運用要求と直接噛み合う。
- **プロセス分離という副産物**: nextestは1テスト=1プロセスで実行する(libtestの1バイナリ内スレッド並列と異なる)。§4-2で見つかった「metrics共有staticの試験間汚染」(M4検収、lane-board:95)のような**プロセス内グローバル状態の汚染由来flake**は、nextest下では別プロセスになるぶん再現しにくくなる可能性がある(ただし、storm/r2がその種の汚染由来かは未検証 — 別コード起因の可能性が高い、§4-4参照)。
- **推奨**: 本調査は導入判断そのものは持ち越すが(NON-GOAL)、機能面では3段分離のtier2(pure)/tier3(full)実行を**同一ビルド成果物から**分けられる点で cargo単体の `-p` フィルタより優れている。次に検収の型を機構化するsupervisor判断の際の一次資料として使える。

## 4. 実測(本worktree、cold state→warm state、`-j 4`・単独実行)

| # | コマンド | 状態 | real | 結果 |
|---|---|---|---:|---|
| 1 | `cargo check --workspace -j 4` | cold(target空) | **49.82s** | 成功 |
| 2 | `cargo test --workspace --locked --no-fail-fast -j 4` | cold(直後、target空から) | **607.41s**(約10.1分) | **2件失敗**: `motolii-store --test document`(storm), `r2-view-projection --test r2` |
| 3 | `cargo test --workspace --locked --no-fail-fast -j 4` | warm(#2直後の再実行、コード無変更) | **99.90s** | 全緑(flake不発) |
| 4 | `cargo check --workspace -j 4` | warm(#3直後) | **1.38s** | 成功 |
| 5 | `cargo test -p motolii-core -p motolii-eval -p motolii-store -p motolii-vector -p motolii-tokens-rs -p motolii-shell-state -p motolii-browser-pane -p motolii-settings-pane -p motolii-testkit --locked --no-fail-fast -j 4` | warm(直前の#4のあとに実行、対象を9crateへ限定) | **199.32s** | **1件失敗**: storm再発。想定より遅い(§4-3) |

### 4-1. checkは常に軽い、フルはcold/warmで一桁以上動く

check(cold 49.82s → warm 1.38s)は**キャッシュ状態に関わらず「速い」**という性質が安定している。一方フル(cold 607.41s → warm 99.90s)は**キャッシュ状態で6倍動く**。supervisorが毎検収でフルを回している現状の負荷は、レーンworktreeがcold(初回・分岐直後)かwarmかで体感が大きく変わっているはずで、「フルが重い」という直感は特にcold worktreeのレーンで顕著だと考えられる。

### 4-2. cold実行でstorm・r2の両flakeが再現

`-j 4`並列でのフル実行(#2)は `motolii-store --test document` と `r2-view-projection --test r2` の2ターゲットを失敗させた。前者は `next/core/motolii-store/tests/document.rs:265` の `edit_storm_with_the_real_track_type`(通称storm)、後者は `next/probes/r2-view-projection` の `tests/r2.rs`。docsが「既知flake2件(storm・r2 — 並列時のみ赤・単独緑)」と記す内容と一致する実測になった(#3のwarm再実行では2件とも不発、#5では storm のみ再発)。**3回中storm 2回・r2 1回**という発火率で、どの段(check/pure/full)を通しても「決定的に捕まえる」ことはできない性質 — flake対策はtier設計と別に、nextest retriesのような機構で扱うのが筋が良い(§3)。

### 4-3. `-p`サブセットは warm フルより遅くなり得る(feature統合の再コンパイル)

pure寄り9crateに絞った#5(199.32s)は、全crateを含むwarmフル#3(99.90s)より**遅かった**。target/はcold実行(#2)の直後で温まっているにも関わらず、`-p`で対象crate集合を変えるとcargoのfeature統合(workspace全体で有効化されるfeatureの合成)が変わり、共有依存の一部が再コンパイルされたと考えられる。**これは「サブセット実行」を素朴に`cargo test -p ...`で実装すると、意図に反してフルより遅くなり得るという実測上の警告**であり、tier2(pure)をcargo単体で運用するなら「常に同じ `-p` 集合を固定する」か、nextestのfilterset(1回のフルビルド後にテスト実行だけ絞る、§3)へ寄せた方が安全という結論を補強する。

## 5. 過去の検収欠陥の遡及検証 — どの段が捕まえたか

`docs/reviews/2026-08-21-lane-board.md` と `docs/reviews/2026-08-22-session-handoff-transcription-hierarchy.md` から実例5件を抽出し、3段のどれで検出可能だったかを判定した。

| # | 欠陥 | 出典 | 検出段 | 根拠 |
|---|---|---|---|---|
| 1 | G1merge時、`Item`をOption化した際の跨crate意味衝突 | handoff:66「G1 merge の意味衝突(Item Option 化)は型が捕捉」 | **check** | 型が捕捉=コンパイルエラー相当。exhaustive match/フィールド型変更はcargo checkだけで再現する典型 |
| 2 | `try_from_frame` の重複再発明(既にcoreにある関数を再実装) | 2026-08-20-session-handoff-reset.md:113「コンパイルエラーで露見」 | **check** | 同名関数の重複定義はコンパイルエラー、テスト実行不要 |
| 3 | `presenter_generation` の0上書きバグ(τレーン施工中の自己検出) | lane-board:108「red で捕捉・修正済み」 | **pure/full境界**(§2-3参照) | ロジック自体は純関数的カウンタだが、`motolii-shell`はworkspace全体を推移的に引くため、tier2を「crateのビルド軽さ」で線引きすると実質フル相当のビルドコストになる。テストの意味では「pure」、ビルドの意味では「full」という2軸のズレの実例 |
| 4 | metrics共有staticの試験間汚染 | lane-board:95(M4検収)「副産物: metrics 共有 static の試験間汚染を発見し METRICS_LOCK 集約」 | **full(並列実行時のみ)** | プロセス内グローバル状態の汚染はテストが同一プロセス・並列で走った時にのみ顕在化する。単発テストやcheckでは原理的に検出不能。nextestのプロセス分離ならこの種の汚染自体が起きにくくなる(§3) |
| 5 | T-canvas検収でtimeline suiteのみ実行しshell suiteを見送り、T-rail検収時にshellでred発覚 | handoff:56「検収の型: 影響 crate の suite は必ず回す」 | **pure/full(スコープの問題)** | tierの「深さ」ではなく「対象crateの広さ」を誤った例。timeline-paneの変更はshellまで波及するため、pure tierであってもtimeline-paneだけでなく依存元(shell)まで対象に含める必要がある教訓 |

**遡及検証の結論**: checkで捕まる欠陥は実在する(#1, #2 — 跨crate型崩れ)。フルでしか捕まらない欠陥も実在する(#4 — 並列プロセス内状態汚染、#5相当のスコープ抜け)。#3は「テストの純度」と「ビルドの重さ」が食い違う実例で、tier設計では**crate依存グラフ上の位置**(§1)を主軸にする方が、テスト内容の純関数性より予測しやすい。

## 6. 判断表(草案)

3段の定義: **① check** = `cargo check --workspace`(型・柵のみ、約50秒cold/1秒台warm)。**② pure** = 変更crateとその依存先・直接の依存元(§1のグラフで1ホップ)に絞った `cargo test`、GPU/プロセス起動テストは対象外(§2の目印表参照)。**③ full** = `cargo test --workspace --locked --no-fail-fast`(現行検収、約10分cold/100秒warm)。

| シナリオ | ① check | ② pure | ③ full | 根拠 |
|---|:---:|:---:|:---:|---|
| **docのみ変更**(*.md) | 不要 | 不要 | 不要 | コード変更なし。`scripts/check-docs.sh`(非cargo)で足りる |
| **pane crate局所**(例: timeline-paneのUI微修正、依存crateの型に触れない) | **必須** | **必須**(該当pane + 直接の依存元、§5-5の教訓どおりshellまで含める) | 省略可(mergeコミットへ集約時にfullを1回) | pane crateはUI層の葉に近く、他paneへの波及は依存グラフ上ない(§1表でpane間の直接依存なし)。ただしshellは必ず引くため、pure tierの対象crateにshellを含めること |
| **store/engine跨り**(motolii-core/motolii-store/motolii-engine等、多数の下流を持つcrateの変更) | **必須** | **必須**(下流全crate — 実質§1の依存グラフでほぼ全crateが対象になる) | **必須** | motolii-storeはcore3crate+engine全体+UI全paneから参照される土台(§1)。局所テストでは下流の意味崩れ(#1の類例)を見落とす。§5-4のstatic汚染のような並列限定欠陥もこの層で起きやすい(motolii-storeやmotolii-shellのテスト数が突出、§2) |
| **fork pin bump**(rerun/iced revの更新) | **必須** | 不十分(GPU/描画経路の実挙動はpure testの対象外) | **必須**(GPU tierを含む全体) | forkのseam(2026-08-18-iced-fork-seam-ledger.md等)はwgpu/re_renderer境界の実行時挙動に依存し、型だけでは捕捉できない。過去のfork更新は必ずGPU実描画を伴うtestで確認されている(§2-3の`headless.rs`系) |

**運用上の注**: pure tierを「軽い」と呼べるのは§4-3の教訓を踏まえ、**crateセットを固定し、nextestのfilterset(§3)かフルビルド後の`-p`固定リストで運用する場合に限る**。素朴に`-p`を毎回変えると、warmフルより遅くなり得る(実測済み)。

## 7. nextest採否の推奨(本調査時点)

- **今は導入しない**(NON-GOAL遵守、本調査でも`run`/`list`は未実行)。ただし機能面の適合性は高い: filtersetがtier2/tier3をcrate依存グラフ(§1)にそのまま対応させられ、`-p`サブセットの再コンパイル問題(§4-3)を構造的に避けられ、`--locked`は透過し、既知flake(storm/r2)へのピンポイントretriesが設定できる。
- 導入を検討する際の一次資料: 本ドキュメント§3(機能面)・§4-3(`-p`の弱点の実測)・§5(遡及検証)。
- 未検証のまま残る点(次調査があれば): (a) nextestのプロセス分離がstorm/r2の発火率を実際に変えるか(§4-2は`cargo test`のみで実測、nextest下は未測定)、(b) filterset式そのものの構文検証(`run`を叩いていないため机上のみ)、(c) `.config/nextest.toml`のプロジェクトへの実導入コスト(CI設定はNON-GOAL)。

## 付記: 実行ログの所在

- 実行#2(cold full、2失敗)の生ログは `/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/aa33e962-4cd6-430f-a034-d932b12321b0/tasks/b9z73vye6.output`(本セッションのbackground task出力、tail 150行のみ保存 — doc-test部と`error: 2 targets failed`行は残るが、失敗テストの詳細panicメッセージ部分はtail範囲外で失われている。再現したい場合は同条件で再実行が必要)。
