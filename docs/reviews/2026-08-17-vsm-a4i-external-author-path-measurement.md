# VSM-A4I 外部作者経路の実測 — LayerSource 専用である

作成日: 2026-08-17

状態: **観察**

対象: `scripts/new_plugin_crate.py`（`VSM-A4I` の外部作者 crate scaffold と Host 検査）。

関連: [Vism実装計画 VSM-A4I](2026-07-17-vism-implementation-plan.md)、[プラグイン作成](../plugin-authoring.md)、[Vismプラグインカタログ](../vism-plugin-catalog.md)

## 0. なぜ測ったか

「既知解に沿って各種 Vism を作れるか」を確かめる検証ターンとして、scaffold で 1 本、手書きで 1 本作った。結論は**LayerSource なら作れる、Filter は書けるが認定できない**である。

## 1. scaffold は主張どおり動く（LayerSource）

```
python3 scripts/new_plugin_crate.py --from core.layer_source.radial_repeater \
  --name rgbsplit --vendor example --out-dir <外部>
→ generated external plugin crate

python3 scripts/new_plugin_crate.py --check <外部>
→ purity ... ok
→ golden ... ok
→ check passed: plugin=example.layer_source.rgbsplit
```

生成された crate の通常依存は `motolii-plugin` **一本だけ**。`[workspace]` で Host workspace から隔離され、`unwrap` / `expect` / `panic` / `todo` / `unimplemented` が deny。dev/build 依存も `build.rs` も無い。**A4I の主張はこの kind については実測で成立している。**

## 2. Filter は公開境界だけで書ける

`core.filter.opacity` の形を写し、外部 crate として RGB Split Filter を手で書いた（約190行）。`motolii_plugin` の公開項目だけを使い、`get_or_create_tex_sample_uniform4` / `require_f64` / `PipelineCacheKey` で閉じる。**ビルドは通る。** 公開 API は Filter を受けている。

## 3. しかし認定経路は LayerSource 専用である

`--check` は 2 段階で撥ねる。

**第一の関門: 識別子** — `scripts/new_plugin_crate.py:257` が `[a-z][a-z0-9_]*\.layer_source\.[a-z][a-z0-9_]*` を要求する。

```
check[hygiene] plugin=example.filter.rgbsplit:
  plugin identity must be vendor.layer_source.name
```

`layer_source` は同ファイルの 6 箇所（`SOURCE_ID`、plugin_id 構築 2 箇所、hygiene 正規表現、testkit import、`register_layer_source` 呼び出し）に焼き込まれている。

**第二の関門: シンボル名** — 生成される Host ハーネスは fixture の Rust シンボルを直接 import する。

```rust
use candidate::{radial_repeater_contract, RADIAL_REPEATER_LAYER_SOURCE};
```

scaffold は `PLUGIN_ID` と `display_name` を差し替えるが**シンボル名は fixture のまま残す**。つまり `--check` が通るのは `--from` で生成した crate だけであり、**手書きの外部 crate は識別子を直しても import 行で落ちる**。

## 4. 汎用化に必要なもの（未実施）

土台は揃っている。

- `PluginRegistry` の `register_filter` / `register_layer_source` / `register_param_driver` / `register_composite` — **4種すべてある**
- `motolii-testkit` の `assert_filter_pure` / `assert_layer_source_pure` / `assert_param_driver_pure` / `assert_composite_pure` — **4種すべてある**
- fixture となる first-party plugin — Filter は `core.filter.opacity`、ParamDriver は `core.param.sine` が repo にある

足りないものは 2 つである。

**(1) `PluginRegistry` の kind 別列挙口。** 汎用ハーネスは `register_plugins(&mut registry)` 方式で実体を受け取る形になるが、purity は `&dyn FilterPlugin` 等の**実体**を要求する。registry から kind 別に取り出す公開 API が無いため、汎用 purity が書けない。`conformance` は contract だけで済むので generic にできる。**詰まるのは purity だけである。**

**(2) golden を作者の opt-in にすること。** 現在のハーネスには radial_repeater 専用の CPU オラクル（`radial_oracle`、約60行）が焼き込まれている。Filter に相当物は無く、**作れない**。期待画像は「その効果が何をするか」という作者の主張であり、Host が生成できるものではない。Host が用意できるのは purity（同じ入力で同じ出力か）までである。crate がオラクルを持たないなら `golden: なし` と明示し、`check passed` と言わないこと。

## 5. 入口規約についての非目標

汎用化では「外部 crate が何を export するか」を決めることになる。これは**Host 内検査の規約であり、公開 ABI ではない**。

現在の plugin は Rust crate を Host へコンパイルで取り込む形で、動的ロードは実装されていない（生成される `AUTHORING.md` が「install / package / load / register しない。composition root へ明示的に足して Host を再ビルド・再起動する」と書く）。したがって入口名を決めても外からは見えず、後で改名できる。

第三者配布の入口をどの規格で喋るかは `VSM-B4`（payload class）と `.vism` container の領域であり、**未決のまま維持する**。OFX / frei0r / ISF の採択可能性を狭めない。

なお `register_reference_plugins` / `register_reference_contracts`（`crates/motolii-plugin/src/reference.rs:14,22`）が first-party 向けに同じ形を既に持っている。外部 crate へ適用するなら新しい書き方の発明ではなく、この形の再利用になる。複数 entry を最初から許し、kind は `PluginContract` が運ぶため、entry 数にも kind にも上限を作らない。

## 6. カタログへの含み

[Vismプラグインカタログ](../vism-plugin-catalog.md) の SINGLE lane（新しい共通能力を要求しない表現）は12件ある。今日の実測により、外部作者として着手できるかが分かれる。

- **今日から可能（LayerSource 3件）**: Fractal Field、Gradient Ramp、Deterministic Particle Field
- **公開 API では書けるが認定できない（Filter 9件）**: RGB Split、Grain、Dither、Halftone、Pixelate、Scanline、Tile、Kaleidoscope、Warp

「A4I でサンドボックスの面は決定済み」は、正確には**1 kind 分だけ決まっている**状態である。

## 7. 成果物

- scaffold 生成物: `/private/tmp/vism-scaffold-probe`（`--check` 通過）
- 手書き Filter: `/private/tmp/vism-rgbsplit-filter`（ビルド通過、`--check` 不可）

どちらも `/private/tmp` にあり、再起動で消える。再現手順は §1〜§3 に書いたコマンドで足りる。
