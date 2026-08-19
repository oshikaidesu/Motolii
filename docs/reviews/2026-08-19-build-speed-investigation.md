# ビルド律速の調査と対処(2026-08-19)

日付: 2026-08-19
状態: **決定**(利用者裁定「入れるべき」。効果の実測は本文書に追記していく)
経緯: UIトンマナ campaign の運転中、レーン1本あたり10〜20分のビルド待ちが常態化し、
利用者が「cargoビルドはもっと爆速にならないのか」と調査を指示。

## この機械の実測(2026-08-19)

| 項目 | 実測 | 含意 |
|---|---|---|
| CPU / RAM | 10コア / **16 GB** | Rust の重量ワークスペースには RAM が少ない |
| swap | **2.3 GB 使用 / pageout 223万回** | 激しくスワップしている |
| `.cargo/config.toml` | **存在しない** | リンカもキャッシュも未設定だった |
| `[profile.dev]` / `[profile.test]` | **未定義** | `cargo test` が stock(完全 debug 情報)で走っていた |
| sccache | インストール済み・**リクエスト0件** | 繋がっていない(かつ下記の理由で繋いでも効かない) |
| リンカ | ld64-530(2022年ビルド) | 旧 classic ld。ただし下記のとおり交換は非推奨 |
| skia-safe | `binary-cache` feature 有効 | **prebuilt を使用済み**(ここは既に手当て済み) |

**自己申告**: 16GB の機械に対して supervisor が並列3〜4レーン × `-j 4〜6` を掛けていた。
CPU が余っていてもメモリが枯渇してスワップし、全体が遅くなる領域に自分で入れていた。
同日の「共有 target 汚染」4件も、この過負荷が背景にある可能性が高い。

## 外部調査の結論(2026年8月時点)

出典と各項目の詳細は調査返却(本文書作成時のセッション記録)に基づく。要点:

- **macOS で代替リンカへ投資しない**: mold 本家は macOS 非対応(商用 sold のみ)、
  wild の Mach-O は「単純なプログラムのみ」段階、**Bevy は実測で lld をコメントアウトし
  「default ld64 linker is faster」と明記**。Rerun も macOS 向け特別設定を置いていない
- **sccache は worktree 跨ぎで効かない**: パスがキャッシュキーに入るため別 worktree ではミスする
  (実測記事で明言)。加えて incremental と非互換、リンクを伴う crate は対象外。
  ベンチマークでは条件次第で**悪化**(初回 +23〜153%)する報告もある
- **dev プロファイルの debug 情報削減が最も費用対効果が高い**: Rust Performance Book の目安で
  **20〜40%**。David Lattimore の実測では debug 情報削減とリンカ改善の合わせ技で 20.2秒→1.5秒
- **worktree 間共有はハードリンク/CoW が正解**: 実測でディレクトリ丸ごとコピー 2分19秒に対し
  ハードリンク**1秒未満・追加ストレージ0**。cargo 公式のクロスワークスペースキャッシュは
  2026年時点でまだ nightly 実験段階
- **cargo-hakari**(workspace-hack)は `-p <crate>` 単位でテストを回す運用の
  feature unification ズレによる重複ビルドに効く可能性がある(未検証・候補)
- `-Zthreads`(並列フロントエンド)と cranelift backend はどちらも**本採用は時期尚早**
  (診断の非決定性/ICE が残る、デバッガ対応が未完成)

## 採った対処

### 1. `[profile.dev]` の調整(本文書と同 commit で適用)

```toml
[profile.dev]
debug = "line-tables-only"   # backtrace の行番号は残す
split-debuginfo = "unpacked" # macOS 既定の明示

[profile.dbg]                # 完全な debug 情報が要る時の待避
inherits = "dev"
debug = "full"
```

`cargo test` は test プロファイル(dev 継承)を使うため、テスト実行にも効く。

### 2. レーンの target は「共有」でなく「APFS CoW クローン」

**`motolii-dispatch` skill に既に書かれていた手**(実測: クローン 22.7秒、以後のビルド 55秒、
network 不要、CoW なので実ディスク消費は増えない):

```bash
cp -Rc <warm>/target <lane-worktree>/target
```

同日の運用では `CARGO_TARGET_DIR=/private/tmp/motolii-lane-target` を**全レーンで共有**しており、
これが (a) 別 worktree のバイナリが混ざる汚染、(b) stale fingerprint、(c) cargo lock 待ち、
を同時に起こしていた(4回実測)。**CoW クローンは隔離を保ったまま依存の再ビルドを避ける**ので、
共有 target の上位互換である。以後のレーン発注はこちらを既定にする。

### 3. 並列度を機械に合わせる

16GB では**同時 cargo は2本まで・`-j 4`**(cargo にはリンカ呼び出しだけ絞る機構が無く、
リンクはメモリ律速になるため。cargo 側の未実装 issue #9157/#12912 を確認済み)。

## 採らなかったもの(理由つき)

- リンカ交換(lld/mold/wild): macOS では効果が無いか悪化(上記)
- sccache: worktree 跨ぎで効かず、incremental と非互換
- cranelift / `-Zthreads`: 2026年8月時点で本採用は時期尚早
- `[profile.dev.package."*"] opt-level`: 依存の再ビルドコストが大きく、
  テスト**実行**時間が現状の律速でないため今回は見送り(将来の候補)

## レーン運用の落とし穴(実測)

- **worktree は `.git` を共有するので stash スタックも共有される**。無名の `git stash pop` は
  **他レーンの stash を掴む**(2026-08-19 に2件実測: レーンI-c が他レーンの stash を掴みかけて
  中断、supervisor も自分の profile 変更を stash して見失った)。
  レーン内では **`stash@{N}` を明示**し、`apply` → 確認 → `drop` の順で扱う。
  そもそも成果を stash に置いたまま turn を終えない
- **Edit 直後の cargo は mtime 同期の遅れで stale fingerprint を掴む**ことがある。
  結果が編集内容と食い違う時は `touch` してから回し、**出力のテスト名が手元のソースと一致するか**を見る
- subagent は背景 cargo を待って中間停止する。発注文に「**cargo は前景・timeout 600000**」を最初から書く

## 未検証の候補(次に効きそうな順)

1. **cargo-hakari**(workspace-hack)— `-p <crate>` 運用の feature 分裂による重複ビルドの解消
2. `cargo build --timings` でボトルネック crate を特定してから追加手当て
3. RAM 増設(機構ではなく物理。16GB がこのワークスペースの規模に対して律速)

## 効果の実測

(適用直後の初回は profile 変更により全再ビルドになるため、2回目以降で比較する)
