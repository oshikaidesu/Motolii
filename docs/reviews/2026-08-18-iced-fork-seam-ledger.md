# iced fork seam 台帳 — 上流とどこで乖離しているか

作成日: 2026-08-18

状態: **台帳**(seam 2件とも本レーンで当てた実測。差分は全文を読んで書いている)

対象: iced の Motolii fork。「上流を追いかけたくなったとき、何を再適用すればよいか」を1枚にする。
型は [Rerun fork seam 台帳](2026-08-18-rerun-fork-seam-ledger.md)を踏襲した。

関連: [ホスト移行裁定](2026-08-18-iced-host-migration-decision.md)(M-0 の受け皿)、
`spikes/iced-rerun-embed-probe/README.md`(seam 2件の必要性の実測)。

## 0. fork の位置

| | commit | 一行 |
|---|---|---|
| 上流の pin | `3de451447bd28217bb535632867550908e29d5d0` | Remove `From<u8>` requirement for `slider` widgets(0.15.0-dev / 2026-08-18 の master HEAD) |
| seam 1 | `c9457a15` | seam(deps): loosen the exact web-sys pin so a native embedder can coexist |
| seam 2 | `73e686ee` | seam(wgpu): let an embedded renderer raise the bind group ceiling |

branch = `motolii/host-seams`。**まだ push していない**(検収側が push して rev を固定する)。

`git diff --stat 3de45144 73e686ee` = 4 files, +104 / -3。
うち `wgpu/src/device_limits.rs`(92行)は**丸ごと追加ファイル**なので、
上流 file への実質的な改変は **+12 / -3** しかない。

置き場: `~/rust_ae/iced-motolii-20260818`。上流 pin は
`spikes/iced-rerun-embed-probe/setup.sh` の `ICED_REV` と**同じ**である
(spike で測った実測が、そのままこの fork の実測でもある)。

## 1. seam 一覧

「追加」= 上流に無いものを足しただけ(rebase で conflict しにくい)。
「改変」= 上流の既存行に手が入っている(rebase で読み直しが要る)。

| 場所 | 種類 | 何のため |
|---|---|---|
| `Cargo.toml:244-248` | 改変(+5 / -1) | **seam 1。** `web-sys` の完全一致釘打ちを解除(§2) |
| `wgpu/src/device_limits.rs` | 追加(92行, file 丸ごと) | **seam 2。** bind group 上限の床(§3) |
| `wgpu/src/lib.rs:25-27` | 改変(+3 / -0) | 上の module 宣言 |
| `wgpu/src/lib.rs:930-931` | 改変(+2 / -1) | headless renderer の device 要求が床を読む |
| `wgpu/src/window/compositor.rs:161-162` | 改変(+2 / -1) | 窓の compositor の device 要求が床を読む |

**上流 rebase で最初に見る順**: `wgpu/src/window/compositor.rs`(device 要求の形は
上流でもよく動く)→ `wgpu/src/lib.rs` の `Headless` 実装 → `Cargo.toml`。
追加ファイル1枚は基本そのまま乗る。

## 2. seam 1 — `web-sys` の完全一致釘打ち解除

**なぜ要るか。** iced master の workspace は `web-sys = "=0.3.85"` と完全一致で釘を打っている。
web-sys 0.3.85 は `js-sys = "=0.3.85"` を要求し、Rerun fork の `re_renderer` は
`js-sys ^0.3.94` を要求する。どちらも wasm32 でしか使わない依存だが、**cargo の解決は
ターゲットを問わず全部を1つのグラフに載せる**ので、iced と Rerun は同じ workspace に
入れない:

```
error: failed to select a version for `js-sys`.
    ... required by package `re_renderer`
  previously selected package `js-sys v0.3.85`
    ... which satisfies dependency `js-sys = "=0.3.85"` of package `web-sys v0.3.85`
    ... which satisfies dependency `web-sys = "=0.3.85"` of package `iced_winit`
```

**直し方**は `=0.3.85` → `0.3.85` の1文字。native ビルドには一切影響しない。
設計上の壁ではなく、上流に投げれば済む種類の話である。

**再適用手順**: `Cargo.toml` の `[workspace.dependencies]` で `web-sys` の行を探し、
`=` を落とす。上流が版を上げていたらその版のまま `=` だけ落とす。

**効いていることの確認**: `cargo metadata` が通る(通らなければ上の error で止まる)。
Motolii workspace では `Cargo.lock` に `wgpu` が**1つだけ**であることも同時に見る。

## 3. seam 2 — bind group 上限の床

**なぜ要るか。** `iced_wgpu` は device を要求するとき `max_bind_groups: 2` を**べた書き**する。
`iced_wgpu::Settings` にも `iced::application(..)` の builder にも上書きの口が無く、
`Compositor` の `adapter` / `engine` フィールドも private である。

Motolii は shader widget の中で Rerun の `re_renderer` を**同じ device の上で**回す
(2026-08-18 裁定「Rerun はホスト非拘束」)。`re_renderer` の `LineRenderer` は
bind group layout を3つ使うので、iced の device では作れない:

```
In Device::create_pipeline_layout, label = 'LineRenderer::pipeline_layout'
  Bind group layout count 3 exceeds device bind group limit 2
```

しかも「gizmo が出ないだけ」では済まない。Rerun の 3D view は**操作が始まった瞬間に**
orbit 中心の目印を線分で描くので、パイプラインが無効なままコマンドバッファごと捨てられ、
**ドラッグしている間だけ絵が止まる**(spike §8 の対照群: 同じ台本で
`re_renderer` の device は18フレーム全部違う絵、iced の device は2フレーム目で固まる)。

**どこに何があるか**(`73e686ee` 時点の行番号):

| file:line | 中身 |
|---|---|
| `wgpu/src/device_limits.rs`(新規) | `DEFAULT_MAX_BIND_GROUPS` / `request_min_max_bind_groups` / `min_max_bind_groups` と単体テスト |
| `wgpu/src/lib.rs:25-27` | module 宣言(Motolii seam のコメント付き) |
| `wgpu/src/lib.rs:930-931` | `impl renderer::Headless for Renderer` の device 要求 |
| `wgpu/src/window/compositor.rs:161-162` | `Compositor::new` の device 要求 |

**なぜ `Settings` へ通さなかったか。** 通すのが上流 PR の形であることは分かっている。
だが `iced_wgpu::Settings` → `iced_winit` → `iced::application` の builder まで
値を通すと十数 file に触れることになり、**master 追随のたびに全部が再 conflict する**。
rerun fork の camera seat と同じ「純追加・明示ブロック・上流 file への差分は最小」流儀を
優先し、床は process 全体の1つに置いた。上流が `Settings` に欄を持った日に、この module は
消えて2つの呼び出し箇所がその欄を読む形になる。

**なぜ「床」で「上書き」ではないか。** 値は上がる一方(`fetch_max`)なので、
同じ process に2人の埋め込み側が居ても互いの天井を黙って下げられない。
**既定は上流と同じ `2`** で、誰も呼ばなければ挙動は1ビットも変わらない。

**呼び方**(埋め込み側):

```rust
// 窓を建てる前。既に在る device には効かない。
iced_wgpu::device_limits::request_min_max_bind_groups(4);
```

**再適用手順。**

1. `wgpu/src/device_limits.rs` はそのまま置く(上流と衝突しない)
2. `wgpu/src/lib.rs` — module 宣言を戻し、`impl renderer::Headless for Renderer` の
   `required_limits` の `max_bind_groups` を `crate::device_limits::min_max_bind_groups()` にする
3. `wgpu/src/window/compositor.rs` — `limits.into_iter().map(..)` の中の
   `max_bind_groups` を同じ関数呼びにする。上流が limits の作り方を変えていたら、
   **`max_bind_groups` を書いている場所**を探し直す(`rg 'max_bind_groups' -g '*.rs'` で2箇所)
4. `cargo test -p iced_wgpu device_limits` を回す(§4)

## 4. 恒久 oracle — rev を上げたら落ちて教えてくれるもの

**今日ある物は2つで、どちらも弱い。** 正直に書く。

- fork 内 `device_limits.rs` の単体テスト1件 — 既定値が上流の `2` であることと、
  床が下がらないこと。**seam が生きていることは言うが、seam が効いていることは言わない**

  ```sh
  cargo test -j 5 --manifest-path ~/rust_ae/iced-motolii-20260818/wgpu/Cargo.toml --lib device_limits
  ```

  実測(2026-08-18, macOS): `test device_limits::tests::the_floor_starts_at_the_upstream_value_and_only_goes_up ... ok`

- Motolii 側 `cargo test -p motolii-shell-iced` — `cargo metadata` が通らなければ
  そもそもビルドできないので、**seam 1 は「Motolii が建つこと」自体が oracle である**

**seam 2 の効きを見る oracle はまだ無い。** M-2(Stage 島)で
`spikes/iced-rerun-embed-probe` の bridge 台本を製品 adapter のテストとして
持ち込んだ時点で、rerun fork の `rerun-e0-composition-probe` と同じ位置の
常設 oracle になる。それまでは「rev を上げたら spike を手で回す」しかない。
この穴は M-2 の受け入れ条件に含める。

## 5. 上流 PR 候補としての位置

両 seam とも**上流に投げれば消える**種類のもので、fork を永続させる理由ではない。

- seam 1: `=` を落とすだけ。iced の wasm ビルドにも影響が無いことは上流側で確認が要る
- seam 2: 上流の形は `iced_wgpu::Settings` の欄。この fork の形(process 全体の床)は
  **PR の形ではない**ので、投げるときは書き直す

投げるまでは fork 2本体制(rerun + iced)を引き受ける、というのは
[ホスト移行裁定](2026-08-18-iced-host-migration-decision.md)「引き受けるコスト」の
とおりである。

## 6. 測っていないこと

- **3 OS 未検証。** macOS / Metal 1台のみ
- **`request_min_max_bind_groups` の実効。** M-0 の殻は Rerun を持たないので、
  床を上げた device が実際に `LineRenderer` を通すことは**この fork では未確認**である
  (spike は fork ではなく vendored copy の上で `re_renderer` 側の device 記述を使って
  対照群を取った)。実証は M-2
- **iced の他の limits。** `max_non_sampler_bindings: 2048` も同じくべた書きだが、
  当たっていないので触っていない
- **上流 rebase の実演。** pin を上げて再適用する手順は書いたが、まだ1度もやっていない

## 7. Motolii 側の参照点

- `Cargo.toml` の `[workspace.dependencies]` に `iced` / `iced_test` が**絶対パス**で
  居る。**検収で push 済み rev pin へ差し替える**(同 file のコメントに差し替え後の形が
  書いてある)。相対パスにしないのは、この repo が `.claude/worktrees/*` からも建つためで、
  worktree と本チェックアウトでは `../` の深さが違う
- iced を引ける crate は `motolii-shell-iced` だけ。柵は
  `crates/motolii-testkit/src/ui_toolkit_dep_policy.rs` の `ICED_CRATE_ALLOWLIST` で、
  **同じ crate は egui 系の allowlist に載っていない**(新しい殻へ古い toolkit が
  滲み戻らない向きの柵)
