# 監督runnerの派生`target/`閉包

状態: **実行決定**。Issue [#329](https://github.com/oshikaidesu/Motolii/issues/329)を`GR-D3`として先行し、
完了まで`M4-K0 / #167`の再発注を止める。

## 観測

K0の第二施工は、許可されたtest/docだけを作り、`fmt`、`clippy`、15 contract tests、
`cargo test --locked --workspace`、docs、golden policyを通過した。しかしGrok起動前に
`delegate-cursor-supervised.sh`がexit 7で停止した。

原因はK0差分ではない。既存のplugin/testkit build scriptとworkspace testが、
`CARGO_TARGET_DIR`に関係なくworktree rootの次を生成する。

- `target/scaffold-plugin-fixture`
- `target/new-plugin-scaffold-test`
- `target/d1i4-empty-classification.tsv`

runnerはignored directoryを子孫までraw manifestへ含め、許可外ignored pathが残れば
scope closureを拒否する。この防護自体は正しい。`target/**`のallowlist化、ignored pathの
fingerprint除外、`.gitignore`変更では解決しない。

## 決定

`GR-D3`はrunnerだけの独立粒とし、scope closureより前に、runnerが所有を証明できる既知の
worktree-root派生物だけをfail-closedで清掃する。

1. 許可されたentryの完全一致を確認する。未知entryが一つでもあれば削除せずSTOPする。
2. symlink、path escape、worktree root不一致を拒否する。
3. 清掃失敗を無視せず、Grokとcheckpointへ進まない。
4. 清掃後もignored pathの列挙、raw manifest、fingerprint、allowlist closureを従来どおり通す。
5. cleanupを一般的な`git clean`や任意directory削除APIへ広げない。

これにより保証は「ignored出力を見ない」ではなく「監督試行が残した既知の派生物は検収前に
存在せず、それ以外のignored mutationは引き続き拒否する」となる。

## runner自己更新のbootstrap

runner自身を変更する粒では、親processはSpark起動前に旧scriptを読み込んでいるため、同じ試行中に
新しい清掃処理を利用できない。これは通常ループを迂回する理由にはしない。

1. Opus orderのin-loop commandは、worktree-root派生物を作らない専用runner testと構文／静的検査に限る。
2. 旧runnerが実diffをscope closureし、checkpointを発行し、Grokが同じdiffをread-only検収する。
3. `VERDICT: ACCEPT`かつP0/P1=0の後、commit前にCodexが外部`CARGO_TARGET_DIR`を指定して
   `cargo test --locked --workspace`とdocs gateを実行し、証跡へ添付する。
4. 新runner自身を使ったK0停止形の再現はdemonstration evidenceであり、Grok検収の代替にしない。

直接Grokだけを呼ぶ、未検収commitを置く、temporary wrapperを信頼側へ置く、checkpointを手で作る、
`target/**`を一時allowlistする方法は採らない。

通常の発注でもcargoを実行する場合はworktree外の`CARGO_TARGET_DIR`を必須とする。GR-D3が清掃するのは
既知のfixture三entryだけであり、Cargo本体の`CACHEDIR.TAG`、`debug/`、`tmp/`を清掃対象へ広げない。

## v4/v5失敗の共通分類
v4とv5は症状が異なるが分類は同一で、authoring channelとfixtureのGit addressingであった。実装対象契約の欠陥ではない。

## 既存閉包が止めなかった範囲
既存のraw manifest、fingerprint、allowlist closureはignored mutationやscope逸脱を止めるが、この2件は止められなかった。防護を弱めた結果ではなく、防護の外側で起きた。

## v4: linked worktreeの.git pointer
v4で失敗した理由は、linked worktreeの`.git`がdirectoryではなくpointer fileであることを前提にしていなかったためである。branchは`codex/gr-d3-runner-target-closure-v4-20260725`、commitは`c7963144`、`.git`の扱いが未対応だった。

## v5: 外側EOF衝突とambient shell
v5で失敗した理由は、外側EOF衝突とambient shell環境の持ち込みによる。対象は`codex/gr-d3-runner-target-closure-v5-20260725`、`db90c6cc`である。

## v4/v5差分の隔離処分
v4とv5の差分は、どちらもGrok検収前、main到達前に隔離した。完了証拠とはせず、比較材料としてのみ扱う。

## v6施工protocol
v6は次を必須とする:
- 隔離worktreeへの`apply_patch`のみを用いる。
- `TMP_ROOT`は絶対pathで指定する。
- `--git-dir`および`--work-tree`を明示する。
- ambientな`GIT_*`はunsetする。
- `HEAD`と全refのsnapshotを取得する。
- `PASS`を採用根拠にしない。
- 試行は一回に限る。
- 失敗時にだけtest authorship撤回を留保する。

## ref bootstrapのcanonical証跡

v6 protocolの`HEAD`と全refのsnapshotは、記録した値を後から照合できて初めて証跡になる。runner自己更新の
制約はこの照合を弱める理由にならず、通常ループを迂回する理由にもしない。

1. Codex precheckの時点で、隔離worktreeの`git rev-parse HEAD`一行と`git show-ref`全行を`LC_ALL=C`で
   整列した本文のSHA-256を一つだけ求め、これを`REF_DIGEST`としてorder証跡へ封じる。
2. 算出は`--git-dir`と`--work-tree`を明示し、ambientな`GIT_*`をunsetした状態で行う。
3. 既存のraw manifest、fingerprint、allowlist closure、ignored path監査、Grok検収は一つも変更しない。
4. `VERDICT: ACCEPT`かつP0/P1=0の後、main採用より前に同じ手順で`REF_DIGEST`を再算出し、封じた値と比較する。
5. 不一致なら差分を隔離処分し、test authorship撤回を留保してSTOPする。採用も再試行も自動化しない。
6. 新runnerが同じ`REF_DIGEST`を自力で出せるようになった後は、手順1〜5の値を比較材料として使い、
   人手の手順をそのまま恒久機構へ昇格させない。

この節はdocsの訂正であり、v6の一回だけの施工試行を消費しない。

次は採らない。

- runnerを包むtemporary wrapperやgit hookで`REF_DIGEST`を作る方法
- checkpointを手で作る方法、Grokを直接呼ぶ方法、未検収commitを置く方法
- 実在しない前提を満たしたことにして先へ進む方法
- 手動のref記録だけを証跡とする方法

## 変更許可と非目標

変更候補は`delegate-cursor-supervised.sh`と同runnerの専用testだけ。正確なallowlistはOpus orderと
Codex precheckで現行コードへ再照合する。

次は非目標とする。

- product crate、build script、fixture意味、`.gitignore`の変更
- `target/**`のallowlist化
- ignored pathのfingerprint／raw manifest監査の削除または縮小
- K0実装差分の採用、修正、copy
- 汎用cache cleaner、background service、公開API
- Cargo本体のbuild artifact清掃。発注側が外部`CARGO_TARGET_DIR`へ隔離する

## 負例と完了条件

- 既知entryだけなら清掃後にscope closureへ進む
- 未知entry、symlink、path escape、清掃失敗は内容を残してfail closedする
- 許可外ignored mutationは従来どおり拒否する
- 専用runner test、`cargo test --locked --workspace`、`scripts/check-docs.sh`が通る
- K0の実際の停止経路でGrok起動まで到達できることを再現する
- Grokが`VERDICT: ACCEPT`かつP0/P1=0を返す

## 再入場

`GR-D3`をmainへ統合した後、K0は新しいmainからfresh worktreeと新しいclosed orderを作る。
未検収の旧worktree差分を自動採用せず、比較材料としてのみ扱う。
