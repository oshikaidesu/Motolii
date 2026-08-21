# iced_aw menu widget — fork互換性 probe(使い捨て実証、merge しない)

委任元: `docs/reviews/2026-08-22-menubar-foundation-survey.md` §3.2 `[patch]` EVIDENCE_GAP
（同 §6 EVIDENCE_GAP 2)・§3.3 案B(`iced_aw::menu` 採用案)の cargo build 実測。
read-only 調査だった survey レーンでは検証不能だった箇所を worktree で実測した。

**結論を先に: 案B(`iced_aw::menu` 採用)は現時点で不採用が妥当。fork 互換性は
理論上ではなく実測で破綻している(API 非互換、2箇所)。**

## セットアップ

- `iced_aw` checkout: `/private/tmp/iced_aw-motolii-probe-source`(rev
  `924be285b80339d969311c8dcc1ebff2e56e9dba`、2026-07-31 "fix flake")
- 追加 path 依存(`next/Cargo.toml` workspace.dependencies):
  `iced_aw = { path = "/private/tmp/...", default-features = false, features = ["menu"] }`
  (repo 外の絶対 path — このマシン限定。採用するなら本番 path か git dep へ
  差し替えが要る)
- `next/shell/motolii-shell/Cargo.toml` の `[dev-dependencies]` へ
  `iced_aw.workspace = true` を追加(`iced_test` と同じ dev-only 扱い)
- fork iced pin(参照元): `next/Cargo.toml` の既存
  `iced = { git = "https://github.com/oshikaidesu/iced", rev = "73e686ee05efd7d1b61cfea2647186b336d9ab9c", ... }`
  (裁定170/M01)

## outcome 1: 混線は解けるか

### [patch] の正確な形 — survey の仮説を1点訂正

survey §3.2 は `[patch."https://github.com/iced-rs/iced.git"]` を想定し、
「M01 が `iced_test` に同種の patch をした先例があるはず」としていたが、
**両方とも実測で外れていた**:

- `next/shell/motolii-shell/Cargo.toml` の `iced_test` は `[patch]` を経由していない。
  `iced_test = { git = "https://github.com/oshikaidesu/iced", rev = "73e686ee..." }`
  という**直接 git 依存**で、本体 `iced` と最初から同じソースを指すため patch 自体が
  不要なケースだった。このレーンで足す `[patch]` が、この repo で最初の実例。
- `iced_aw` 自身の `Cargo.toml` には確かに
  `[patch.crates-io] iced_core = { git = "https://github.com/iced-rs/iced.git", branch = "master" }`
  等があるが、**cargo は workspace member / path 依存先のネストした manifest の
  `[patch]` テーブルを一切見ない**(root manifest の `[patch]` だけが効く)。
  つまり `iced_aw` 側のこの記述は、`iced_aw` 単体が root workspace として
  ビルドされる時にしか効かず、本 workspace に組み込んだ瞬間から無視される。
  `iced_aw` の `[dependencies]` 実体は `iced_core = { version = "0.15.0-dev" }`
  という**バージョン指定のみ**(git URL 指定なし)なので、これが実際に解決される
  ソースは **crates-io**。よって効く patch キーは
  `[patch."https://github.com/iced-rs/iced.git"]` ではなく **`[patch.crates-io]`**。

`next/Cargo.toml` へ実際に足した形(採用した形):

```toml
[patch.crates-io]
iced = { git = "https://github.com/oshikaidesu/iced", rev = "73e686ee05efd7d1b61cfea2647186b336d9ab9c" }
iced_core = { git = "https://github.com/oshikaidesu/iced", rev = "73e686ee05efd7d1b61cfea2647186b336d9ab9c" }
iced_widget = { git = "https://github.com/oshikaidesu/iced", rev = "73e686ee05efd7d1b61cfea2647186b336d9ab9c" }
iced_runtime = { git = "https://github.com/oshikaidesu/iced", rev = "73e686ee05efd7d1b61cfea2647186b336d9ab9c" }
```

### 実測1: `cargo check -p motolii-shell`(lib のみ)→ 緑・ただし偽陽性

```
cargo check -p motolii-shell --manifest-path next/Cargo.toml -j 4
```
→ **exit 0**、`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2m 36s`、
error 0件。`Cargo.lock`/build log を照合すると `iced_core` の `Checking` 行は
`iced_core v0.15.0-dev (git+https://github.com/oshikaidesu/iced?rev=73e686ee...)`
の1本のみ — patch は効いていて、直接依存側の混線は実際に解けている。

**ただしこれは偽陽性だった**: `iced_aw` は `motolii-shell` の `[dev-dependencies]`
にしか入れていないため、`--tests` を付けない素の `cargo check -p motolii-shell` は
lib target しか見ず、`iced_aw` 自体を一度もコンパイルしていない
(`Adding iced_aw v0.14.1 (...)` は lockfile 更新のログに出るだけで `Checking iced_aw`
行が無いことで確認)。dev-dependency の実体を検証するには `--tests` が要る。

### 実測2: `cargo check -p motolii-shell --tests` → **exit 101、iced_aw 自体がビルド不能**

```
cargo check -p motolii-shell --tests --manifest-path next/Cargo.toml -j 4
```
→ **exit 101**:

```
error[E0308]: mismatched types
   --> iced_aw/src/widget/menu/menu_bar_overlay.rs:381:50
    |
381 |                 let mut fake_shell = shell.local(&mut fake_messages);
    |                                            ----- ^^^^^^^^^^^^^^^^^^ expected `&mut Bus<_>`, found `&mut Vec<_>`
    = note: expected mutable reference `&mut Bus<_>`
               found mutable reference `&mut Vec<_>`
note: method defined here
   --> .../iced-.../73e686e/core/src/shell.rs:50:12
    |
 50 |     pub fn local<'b, A>(&self, bus: &'b mut Bus<A>) -> Shell<'b, A>

error[E0308]: mismatched types
   --> iced_aw/src/widget/menu/menu_tree.rs:507:54
    |
507 |                     let mut temp_shell = shell.local(&mut temp_messages);
    |                                                ----- ^^^^^^^^^^^^^^^^^^ expected `&mut Bus<_>`, found `&mut Vec<_>`

error: could not compile `iced_aw` (lib) due to 2 previous errors
```

**根本原因**: 本 fork(`oshikaidesu/iced` rev `73e686ee`)の
`iced_core::Shell::local` は `&mut Bus<A>` を取る署名になっているが、`iced_aw`
(2026-07-31 checkout)は upstream 旧 API のまま `&mut Vec<_>` を渡している。
つまり `iced_aw` の checkout 時点と本 fork の pin 時点の間で、upstream
`iced-rs/iced` master 側に `Shell` の `Bus` 化リファクタが入っており、両者が
ズレた。survey が前提にしていた「fork は upstream master と drift 0」は
**fork 作成時点では正しかったが、`iced_aw` 側が同じ速度で追随していない**ため、
2プロジェクト間の相対ズレとして表出した — fork 自体の改造なしにこれを直す
手段は無い(`iced_aw` 側2箇所の呼び出しを書き換えるしかなく、それは NON-GOALS
の「fork 改造」に相当する範囲外作業)。

副次的な発見(致命的ではないが記録に値する): `[patch.crates-io]` は
**semver 互換なリクエスタにしか効かない**。`iced_aw` の間接依存
`iced_fonts`(crates-io、0.3.0)は `iced_core`/`iced_widget` を(0.15.0-dev
という prerelease とは semver 非互換な)`^0.14` 系列で要求しており、patch の
対象外のまま **`iced_core 0.14.0`/`iced_widget 0.14.2`/`iced_graphics 0.14.0`/
`iced_renderer 0.14.0`(+ 付随する `cosmic-text 0.15.0` 旧版一式)という
無関係な旧 iced 0.14 スタックがまるごと二重に依存グラフへ入る**
(`Cargo.lock` 実測、`grep '"iced_core 0.14.0'` で3箇所の依存元
`iced_fonts`/`iced_futures 0.14.0`/`iced_graphics 0.14.0` を確認)。
`motolii-shell` の型が直接この旧スタックに触れることは無いので lib check は
汚染されないが、採用すればビルド時間・バイナリサイズの純増になる — 今回は
本命の Shell/Bus 非互換で止まったため実測できていない。

## outcome 2: oracle 可視の実証

**未達(ブロック)**。`iced_aw` 自体が(menu feature 込みで)ビルド不能なため、
その上に乗る `iced_test::Simulator::find` の実証は原理的に走らせられない。

想定していたテスト本体は
`next/shell/motolii-shell/tests/probe_iced_aw_menu.rs` に**コード化して残した**
(供覧用・commit 対象・merge しない)。内容: `MenuBar` に root "File" → 葉 "New" +
サブメニュー "More" → ネスト2段目 "Nested Item" を組み、
`iced_test::simulator` + `ui.click("File")` → `ui.click("More")` →
`ui.find("Nested Item").is_some()` を assert する形(`iced_test_spike.rs` の
selector 器具をそのまま流用)。**このファイルは一度もコンパイルされていない**
— `iced_aw` が先に落ちるため、このテスト自身のコードが正しいかどうかも
実は未検証。`operate()` が overlay 配下まで正しく候補登録しているかという
本命の問いには、今回のセットアップでは到達できなかった。

## ビルド時間影響

`cargo check -p motolii-shell`(lib のみ、既存 warm target 上への差分ビルド)
は 2m36s で完走(iced_aw 自体は未コンパイルの偽陽性ケース)。`--tests` は
`iced_aw` のコンパイルエラーで数秒〜十数秒で打ち切られるため、フル
`--tests` 通過時の実測時間は取れていない(未着手)。

## iced_aw の API で気づいた癖

- `MenuBar::new(items)` / `Menu::new(items)` は `menu_items!` マクロ +
  `(label_widget, submenu)` タプル記法(`examples/menu_test.rs`・
  `examples/menu.rs` で確認、`submenu_button("SUB")` のようなヘルパーで
  ネストを組む)。API 自体は使いやすい部類。
- `iced_aw` 自身の `Cargo.toml` は `edition = "2024"`・`rust-version = "1.92"`
  を要求(今回のツールチェーンでは edition 起因のビルドエラーは出ていない)。
- `iced_aw` の `[patch.crates-io]`(→ upstream `iced-rs/iced` master)は
  ネストした manifest なので**この workspace には一切効かない**という cargo
  の仕様は、他の外部 crate probe でも踏み得る一般的な罠として記録しておく
  価値がある(root workspace の `[patch]` だけが唯一の介入点)。

## 採否推奨

**現時点で不採用**。理由:

1. `iced_aw`(2026-07-31 checkout)は本 fork(`73e686ee`)に対して
   `Shell::local` の `Bus`/`Vec` 非互換で**実際にビルドできない**
   — 理論上の懸念(survey §3.2)ではなく実測で確定した破綻。
2. 直す手段は `iced_aw` 側コードの改造(2箇所)のみで、それは自前フォーク+
   保守の恒久負債になる([保守最低限・スクラッチ禁止]メモの
   wraps>移植>スクラッチ原則に照らすと、外部 crate をこちらで
   フォーク改造するのは「移植」の皮を被った実質スクラッチに近い)。
3. `[patch.crates-io]` の semver 非互換リクエスタ(`iced_fonts` 経由の
   旧 iced 0.14 スタック二重化)という副作用も、採用するなら追加の
   検証コストになる。

survey §3.3 の推奨(「案A: 標準部品の組み合わせを v1 で先行、ネスト深度が
実際に要る段で案Bの互換性検証を別途行う」)はこの実測でむしろ補強された
— 案Bはコストが「fork 互換性検証」ではなく「fork 互換性の恒久的な
追随コスト」であることが分かったため、ネストが本当に必要になった段でも
再度この非互換が塞ぐ可能性が高い。**MB-2 は引き続き案A(`overlay::menu::Menu`
+ `widget::pin::Pin` の自作)で進めるのが妥当**。

## 変更ファイル(probe、merge しない)

- `next/Cargo.toml`: `iced_aw` path 依存(workspace.dependencies)+
  `[patch.crates-io]` 追加
- `next/Cargo.lock`: 上記に伴う再ロック
- `next/shell/motolii-shell/Cargo.toml`: `[dev-dependencies]` へ
  `iced_aw.workspace = true` 追加(outcome1 の `-p motolii-shell` 検証に
  必須だったため、当初の allowlist 列挙に無かったが追加。指示の「outcome1」を
  満たすための唯一の経路)
- `next/shell/motolii-shell/tests/probe_iced_aw_menu.rs`: 新規、未コンパイル
  (供覧用)
- 本ファイル: `docs/reviews/2026-08-22-iced-aw-menu-probe.md`
