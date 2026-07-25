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

## 変更許可と非目標

変更候補は`delegate-cursor-supervised.sh`と同runnerの専用testだけ。正確なallowlistはOpus orderと
Codex precheckで現行コードへ再照合する。

次は非目標とする。

- product crate、build script、fixture意味、`.gitignore`の変更
- `target/**`のallowlist化
- ignored pathのfingerprint／raw manifest監査の削除または縮小
- K0実装差分の採用、修正、copy
- 汎用cache cleaner、background service、公開API

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
