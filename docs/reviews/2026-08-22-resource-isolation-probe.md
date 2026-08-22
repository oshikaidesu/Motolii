# 並列レーンの共有可変資源分離 probe — cargo ロック実測+分離手段比較

日付: 2026-08-22 / 状態: **probe**(実測・製品コード非変更) / 起点: 裁定203(並列の障害は「共有される可変資源」に一般化)

対象: cargo が触る共有可変資源(`.package-cache`・registry index/cache/src・git checkout)。このマシンの実 `CARGO_HOME` は `~/.asdf/installs/rust/stable/`(`~/.cargo` は存在しない)。

## 0. 結論(先出し) — 当初仮説は途中で覆った

**最重要の発見(このprobeの本体)**: このマシンでは `cargo`/`rustc` は **asdf の shim**(`~/.asdf/shims/cargo`)経由で起動され、shimは内部で `asdf exec` を呼ぶ。asdf の rust プラグインの `exec-env` フック(`~/.asdf/plugins/rust/bin/exec-env`)が

```bash
export CARGO_HOME=$ASDF_INSTALL_PATH
export RUSTUP_HOME=$ASDF_INSTALL_PATH
```

を**無条件に実行し、呼び出し元が設定した `CARGO_HOME` を毎回黙って上書きする**(実測で確認)。つまり**「レーンごとに `CARGO_HOME` を環境変数で分ける」という最有力候補が、このマシンでは字面通りには機能しない** — `export CARGO_HOME=/lane/foo; cargo check` を実行しても、shim が `CARGO_HOME` を共有パスへ強制的に戻すため、実際に触れるのは相変わらず共有の `~/.asdf/installs/rust/stable` である。これは実測で二重に確認した(§3後半・§4)。**発注テンプレートに「CARGO_HOME を分ければ直る」とだけ書くと、このマシンでは効かない指示になる**。

分離を実際に効かせるには、shimを迂回してツールチェーンの実体バイナリを直接呼ぶ必要がある(`~/.asdf/installs/rust/stable/bin/cargo`)。これを行うと `CARGO_HOME` の分離は**実測でロック待ちを完全にゼロへ落とす**(§4)。

以下、実測で確定した事実:

1. **共有CARGO_HOME(shim経由の通常運用)で2レーン同時 `cargo check --workspace` を回すと、`Blocking waiting for file lock on package cache` が両ログに3〜4回出る**(再現性あり、複数回実測)。
2. **`--offline` を付けてもロック待ちは消えない**(3回のまま) — ネットワーク遮断とキャッシュ排他は無関係。
3. **`cargo vendor` で source replacement しても、CARGO_HOME を共有したままなら2レーン同時実行でロック待ちは消えない**(3回、実測)。当初「directory sourceはグローバル共有キャッシュを経由しないので原理的にロック不要」という仮説を立てたが、**この仮説は実測で否定された** — ロックは registry アクセスの有無ではなく **`$CARGO_HOME` という物理パスそのものに対する排他制御**であり、vendor化してもそのプロセスは同じ `$CARGO_HOME/.package-cache` を触りに行く。
4. **shimを迂回して実体バイナリを直接呼び、`CARGO_HOME` を本当に分離すると、ロック待ちは実測で完全にゼロになる**(vendorの有無に関わらず — 素の registry 参照でも、vendor構成でも、両方0件)。**ロックを消す唯一の構造的な手段は `$CARGO_HOME` の物理的分離であり、vendorはそれ単体では代替にならない**。
5. ロック待ち自体の時間コストは短い(ビルド起動直後の数行のみ)。§2で観測した全体所要647秒(単独cold実測50秒の13倍)の主犯はロックではなく**マシン全体のCPU競合**(実測時 load average 8.5〜15.6、10コア機に対し22本超のcargo/rustcプロセスが他レーンから同時に走行中)。ロック分離は「順番待ち」を消すが「CPU予算の奪い合い」は別問題として残る。
6. `cargo vendor` はこのワークスペースの実依存を **901MB・847crate・実測166秒**でローカルへ展開する(git依存4本=iced/winit/cryoglyph/rerun のforkも含む)。CARGO_HOME全体(15G)やregistry部分クローン(4.25G・実測8〜9分)より軽量。ただし**git依存のsource replacementが本ワークスペースの `rerun` forkで実際にはマッチせず、`--offline`ビルドが失敗する不具合を実測で確認**(§5)。crates-io側の置換は機能していたため、git source replacementのURL正規化に何らかの不一致がある — 採用前に要修正。

## 1. 環境確認

```
$ echo $CARGO_HOME   # 未設定(呼び出し元シェルでは)
$ ls ~/.cargo         # No such file or directory
$ which cargo
/Users/member_ottoto/.asdf/shims/cargo
$ cat ~/.asdf/shims/cargo
#!/usr/bin/env bash
exec /Users/member_ottoto/.asdf/bin/asdf exec "cargo" "$@"
$ cat ~/.asdf/plugins/rust/bin/exec-env
export CARGO_HOME=$ASDF_INSTALL_PATH
export RUSTUP_HOME=$ASDF_INSTALL_PATH
```

registry構成とサイズ(`du -sh`実測。他レーン競合下で計測自体に約10分かかった=このコマンド自体がこの環境の負荷の傍証):

| パス | サイズ |
|---|---|
| `registry/cache` | 445M |
| `registry/src` | 3.7G |
| `registry/index` | 106M |
| `git`(git依存チェックアウト) | 5.5G |
| CARGO_HOME全体 | 15G |
| ファイル数(cache+index+src) | 176,145 |

## 2. 実測1: 共有CARGO_HOMEでの2レーン同時 `cargo check --workspace`(cold・shim経由=通常運用)

worktree 2本(`wt-a`・`wt-b`、`ff71ba76` からの新規チェックアウト=target空)で `cargo check --manifest-path next/Cargo.toml --workspace -j 4` を同時起動(通常の `cargo` = shim経由、CARGO_HOME指定なし=共有)。

```
$ grep -c "Blocking waiting for file lock on package cache" log-a-shared.log log-b-shared.log
log-a-shared.log:3
log-b-shared.log:4
```

該当行は `Checking` が1件も出る前(起動直後)にまとまっている。`lsof` で該当時刻に確認した保持プロセス(このprobeの2本以外にも他レーンが同時アクセス):

```
$ lsof | grep package-cache
cargo ... .package-cache-mutate   (4プロセス)
cargo ... .package-cache          (3プロセス)
```

所要時間(両者とも同時完了、`date +%s.%N` 差分): **wt-a 647.87s / wt-b 647.87s**。

参考: AGENTS.mdの単独cold実測は約50秒(裁定138)。647秒との開き(約13倍)を**ロックのせいと即断しない**こと。Blocking行自体は起動直後の数回のみで、以降は両レーンとも並行してCompiling/Checkingが進んでいた。実測時の `uptime` は `load averages: 9.67 9.41 15.58`(10コア機)、`ps aux | grep -c "cargo\|rustc"` は22本(他レーン走行中)。**主因はCPU予算の奪い合い**、ロックは秒〜十数秒の「順番待ち」を作るのみ。

## 3. 実測2: `--offline` はロックを消すか / vendorはロックを消すか(いずれもCARGO_HOME共有のまま)

同一条件で `--offline` を追加(`wt-c`・`wt-d`、共有CARGO_HOME・shim経由):

```
$ grep -c "Blocking waiting for file lock on package cache" log-c-offline.log log-d-offline.log
log-c-offline.log:3
log-d-offline.log:3
```

**`--offline` を付けてもBlocking行の回数は変わらない**。ネットワークを切ってもキャッシュ排他ロックは残る。

次に `cargo vendor` で source replacement した状態(`wt-a`・`wt-b` に `.cargo/config.toml` + `vendor/` を配置、CARGO_HOMEは共有のまま・shim経由)で2レーン同時実行:

```
$ grep -c "Blocking waiting for file lock" log-a-vendor2.log log-b-vendor2.log
log-a-vendor2.log:3
log-b-vendor2.log:3
```

**vendor化してもCARGO_HOMEを共有していればロック待ちは消えない**。単独プロセスでvendor構成を実行した際は待ちがゼロだったため(後述§5参照、単独では「待つ」相手がいないので当然)、当初「vendorはロック不要」と誤認したが、**2並列で初めて可視化された**。ロックは registry ソースの有無ではなく `$CARGO_HOME` という物理パスへの排他制御である。

## 4. 実測3: shimを迂回して `CARGO_HOME` を本当に分離するとどうなるか

`export CARGO_HOME=<lane専用パス>` をしても、shim経由の `cargo`(`asdf exec` 越し)は `exec-env` によって共有パスへ戻される。これを実測で確認: 空の分離ディレクトリを用意して shim経由で実行 → **実行後もディレクトリは空のまま**(何も書き込まれていない=分離が効いていない証拠)、かつBlocking行は共有時と同じ3回。

ツールチェーンの実体バイナリを直接呼ぶ(`~/.asdf/installs/rust/stable/bin/cargo`、shimを経由しない)と、`CARGO_HOME` の指定が実際に反映される:

```
$ CARGO_HOME=$LANE/cargo-home-empty-a ~/.asdf/installs/rust/stable/bin/cargo check --offline ...
$ find $LANE/cargo-home-empty-a -maxdepth 2
cargo-home-empty-a/.package-cache   ← 生成された(shim経由では生成されなかった)
cargo-home-empty-a/registry/CACHEDIR.TAG
cargo-home-empty-a/git/CACHEDIR.TAG
```

この状態で2レーン同時実行した結果、**Blocking行は両方とも0件**(実測)。ビルド自体は空のCARGO_HOMEに何もキャッシュがなく`--offline`のため `re_build_info`(rerun fork)のgit取得で失敗(EXIT=101)したが、これは分離の成否とは無関係な別事象(§5)——**ロック待ちがゼロになったという構造的事実は失敗の有無に左右されない**(ロックの取得自体はビルド開始直後、依存解決の前段で発生するため)。

## 5. `cargo vendor` の実測とgit依存の未解決事項

```
$ cargo vendor --manifest-path next/Cargo.toml next/vendor
(847 crate、901MB、実測166秒)
```

`cargo vendor` はcrates-io依存だけでなく、workspaceが使うgit依存4本(`iced-rs/cryoglyph`・`iced-rs/winit`・`oshikaidesu/iced`・`oshikaidesu/rerun` — いずれもfork)も併せてvendor化し、以下の設定を標準出力へ出す:

```toml
[source.crates-io]
replace-with = "vendored-sources"

[source."git+https://github.com/oshikaidesu/rerun?rev=<rev>"]
git = "https://github.com/oshikaidesu/rerun"
rev = "<rev>"
replace-with = "vendored-sources"
# (iced/winit/cryoglyphも同型で3件)

[source.vendored-sources]
directory = "vendor"
```

**crates-io側の置換は機能を確認**(vendor構成でのcargo checkがcrates-io依存では正常に完了、警告のみでビルド成功=§3の単独実行実測で確認済み)。しかし**shim迂回+完全に空のCARGO_HOMEで`--offline`実行すると、`oshikaidesu/rerun`のgit依存だけが置換にマッチせず実際にgit fetchを試みて失敗した**(crates-io本体は素通り)。Cargo.lockの `source = "git+https://github.com/oshikaidesu/rerun?rev=<rev>#<full-hash>"` と config.tomlの `[source."git+https://github.com/oshikaidesu/rerun?rev=<rev>"]`(vendorが出力した形そのまま)を突き合わせたが文字列上は一致しており、**原因は未特定**(cargoの内部的なSourceId正規化の差の可能性)。**この不具合は本probeの時間内に解決できず、vendor+git依存の組み合わせを本採用する前に要修正**として持ち越す。

## 6. 分離手段の比較(実測ベースに更新)

| 手段 | ロックを消すか(2レーン同時・実測) | 初回コスト | ディスク/レーン | 備考 |
|---|---|---|---|---|
| (a) `CARGO_HOME`分離(env varのみ、shim経由) | **消えない**(shimが上書き・実測) | ゼロ(だが無効) | ゼロ(だが無効) | このマシンでは**動作しない** |
| (a') `CARGO_HOME`分離+shim迂回(実体バイナリ直接呼び出し) | **消える(0件、実測)** | registryのみclonefile実測8〜9分/4.25G、gitチェックアウト込みなら+5.5G | +4.25G(registry)〜+9.75G(git込み) | 推奨の技術的前提。呼び出し規約の変更が必要 |
| (b) `cargo vendor` + source replacement(CARGO_HOME共有のまま) | **消えない**(実測3回) | 実測166秒・901MB | 901MB(全レーン共通のvendorディレクトリを指すなら実質ゼロ増) | 単体では不十分。(a')と併用が必要 |
| (b)+(a') 併用 | **消える(0件、実測)** | vendor 166秒(初回のみ・共有可)+空CARGO_HOMEはほぼゼロ | 901MB(共有)+ほぼゼロ(レーンごとのCARGO_HOMEはvendor構成なら中身が要らない) | **git依存のsource replacementに未解決バグあり(§5)。修正後が本命** |
| (c) `--offline` | 消えない(実測) | ほぼゼロ | ゼロ | ロック分離策としては不採用(ネットワーク遮断のみの効果) |
| (d) `CARGO_TARGET_DIR`分離 | 対象外(別ロック種別) | ゼロ(デフォルト動作) | 既存どおり | worktree相対で自然に分離される。明示指定で共有しない限り無問題 |

## 7. 発注テンプレートへの推奨

**技術的前提(最優先で書くべき事項)**: このマシンで `CARGO_HOME` をレーンごとに分離するなら、**asdfのshimを経由する `cargo`/`rustc` ではなく、ツールチェーンの実体バイナリを直接呼ぶ**(例: `~/.asdf/installs/rust/stable/bin/cargo`)。`export CARGO_HOME=...; cargo ...` は**このマシンでは効かない**(exec-envが上書きする)。これに気づかず「CARGO_HOMEを分ければ直るはず」と発注すると、実際には何も変わらず「直らなかった」という誤った再発報告を生む。

推奨構成(暫定・vendor+git依存バグ修正待ちのため、まずは(a')単体から):

```bash
# レーン起動時に1回(shim迂回が必須)
export CARGO_HOME="$LANE_DIR/.cargo-home"
mkdir -p "$CARGO_HOME"
REAL_CARGO=~/.asdf/installs/rust/stable/bin/cargo
REAL_RUSTC=~/.asdf/installs/rust/stable/bin/rustc
# レーン内のビルド/検収コマンドはすべて $REAL_CARGO 経由で呼ぶ(PATHのcargoではなく)
cp -Rc ~/.asdf/installs/rust/stable/registry "$CARGO_HOME/registry"   # 初回のみ・実測8〜9分/4.25G
```

`cargo vendor` は §5 のgit依存バグを修正できれば、初回コストを166秒・901MBまで縮められ、かつネットワーク非依存にできる(**次レーンへの持ち越し課題**)。修正が済むまでは (a') 単体(registry部分クローン)を暫定運用とする。

正式な仕組み化(shim迂回をラッパースクリプト化する等)は**保守最低限の原則によりこのprobeでは行わない** — supervisor裁定後に決める。

## 8. 逸脱・持ち越し

- **vendor+git依存のsource replacement不一致(§5)は原因未特定のまま持ち越し**。次レーンでの追試を推奨(`cargo -v vendor`のverboseログや`cargo tree --offline`でSourceId正規化を直接確認する方向)。
- **CPU競合とロック競合の切り分けが不完全**(§2)。このマシンは実測時点で他レーン(probe外)が多数(22本規模)走行中で、意図的な無負荷環境を作れなかった。647秒という数字はロック単体のコストではなくマシン全体の負荷を含む。
- **(a')の完全な成功ビルドは未実測**: shim迂回+分離CARGO_HOMEでの「ロック待ちゼロ」は構造的に確認できたが、実際に**正常完了する**フルビルドの所要時間(registryクローン済みCARGO_HOMEを使った場合)は時間内に測定できなかった(§4のテストは意図的に空のCARGO_HOMEで`--offline`により早期に失敗させ、ロック行動そのものだけを観測する設計にした)。
- registry全体サイズ計測(`du -sh`)自体が競合下で約10分かかった。diskI/O律速の一般論であり、cargo固有のロックとは別現象。
