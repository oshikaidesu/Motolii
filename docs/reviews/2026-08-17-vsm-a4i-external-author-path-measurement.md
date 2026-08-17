# VSM-A4I 外部作者経路の実測と汎用化

作成日: 2026-08-17

状態: **観察**（§1〜§3）＋ **実装**（§4）

対象: `scripts/new_plugin_crate.py`（`VSM-A4I` の外部作者 crate scaffold と Host 検査）。

関連: [Vism実装計画 VSM-A4I](2026-07-17-vism-implementation-plan.md)、[プラグイン作成](../plugin-authoring.md)、[Vismプラグインカタログ](../vism-plugin-catalog.md)

## 0. なぜ測ったか

「既知解に沿って各種 Vism を作れるか」を確かめる検証ターンとして、scaffold で 1 本、手書きで 1 本作った。

出発点の結論は**LayerSource なら作れる、Filter は書けるが認定できない**だった（§1〜§3）。汎用入口が正しいかを検証する過程で、詰まりとした前提が誤りだと分かり、そのまま汎用化まで通した（§4）。

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

## 4. 汎用化した（実測で通した）

土台は揃っていた。

- `PluginRegistry` の `register_*` — 4種すべてある
- `motolii-testkit` の `assert_*_pure` — 4種すべてある
- **`PluginRegistry::iter(kind) -> Iterator<(&PluginId, DynPlugin)>`（`crates/motolii-plugin/src/registry.rs:138`）と `DynPlugin`（同 `:256`、4 kindのenumで `desc()` / `kind()` を持つ）** — これで汎用ハーネスが実体を取り出せる

初版の本節は「registryにkind別の取り出し口が無いため汎用purityが書けない」と書いていたが**誤りである**。`pub fn register_` だけを検索して結論した推論だった。列挙口は最初から存在する。

汎用化は次の形で成立した。

**入口**: 外部crateが2本の関数を出す。`reference.rs:14,22` の first-party 向けの形をそのまま使う。

```rust
pub fn register_contracts(catalog: &mut PluginCatalogBuilder) -> Result<(), PluginContractError>;
pub fn register_plugins(registry: &mut PluginRegistry) -> Result<(), PluginError>;
```

複数entryを最初から許し、kind は `PluginContract` が運ぶ。**entry数にもkindにも上限を作らない。**

**ハーネス**: `register_*` を呼び、`registry.iter(kind)` で列挙し、`DynPlugin` で dispatch する。paramsは `NodeDesc` が宣言した既定値だけで組み、fixture固有の値を持ち込まない。未対応kind（ParamDriver / Composite）は**黙って通さず失敗させる**。

**golden**: 作者のopt-inにした。現行ハーネスにあった radial_repeater 専用CPUオラクル（約60行）は汎用化できない。期待画像は「その効果が何をするか」という作者の主張であり、Hostが生成できるものではない。`--check` は golden を回さず、回していないことを明示して出力する。

### 検証結果

同じハーネス・同じ入口2本で、kindの違う2 crateが通った。

```
LayerSource（scaffold生成 example.layer_source.rgbsplit）
  conformance ... ok / purity ... ok / check passed
Filter（手書き example.filter.rgbsplit）
  conformance ... ok / purity ... ok / check passed
新規生成（example.layer_source.freshprobe）
  conformance ... ok / purity ... ok / check passed
```

**手書きcrateが通ったことが要点である。** 以前は `--from` 生成物しか通らなかった。

## 5. 入口規約についての非目標

汎用化では「外部 crate が何を export するか」を決めることになる。これは**Host 内検査の規約であり、公開 ABI ではない**。

現在の plugin は Rust crate を Host へコンパイルで取り込む形で、動的ロードは実装されていない（生成される `AUTHORING.md` が「install / package / load / register しない。composition root へ明示的に足して Host を再ビルド・再起動する」と書く）。したがって入口名を決めても外からは見えず、後で改名できる。

第三者配布の入口をどの規格で喋るかは `VSM-B4`（payload class）と `.vism` container の領域であり、**未決のまま維持する**。OFX / frei0r / ISF の採択可能性を狭めない。

なお `register_reference_plugins` / `register_reference_contracts`（`crates/motolii-plugin/src/reference.rs:14,22`）が first-party 向けに同じ形を既に持っている。外部 crate へ適用するなら新しい書き方の発明ではなく、この形の再利用になる。複数 entry を最初から許し、kind は `PluginContract` が運ぶため、entry 数にも kind にも上限を作らない。

## 6. カタログへの含み

[Vismプラグインカタログ](../vism-plugin-catalog.md) の SINGLE lane は12件ある。§3 の時点では LayerSource 3件だけが着手可で、Filter 9件は認定不可だった。§4 の汎用化により、**12件すべてが同じ経路で認定できる**。

ただし golden は作者のopt-inになったため、各表現は自分のオラクルを持ち込む必要がある。Hostが用意するのは conformance と purity までである。

ParamDriver と Composite は purity ハーネスが未対応で、`--check` は黙って通さず失敗する。fixture（`core.param.sine`）は repo にあるので、対応は同じ形の追加で済む。

## 7. 成果物

- scaffold 生成物: `/private/tmp/vism-scaffold-probe`
- 手書き Filter: `/private/tmp/vism-rgbsplit-filter`
- 新規生成: `/private/tmp/vism-fresh-probe`

3本とも `--check` 通過。`/private/tmp` にあるため再起動で消えるが、再現手順は §1 と §4 のコマンドで足りる。手書き Filter の全文は本文書に無く、`core.filter.opacity` を写して `register_contracts` / `register_plugins` を足したものである。
