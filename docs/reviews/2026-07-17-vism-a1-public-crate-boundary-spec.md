# VSM-A1S — Opacity外部crate化の公開境界仕様

作成日: 2026-07-17

状態: **仕様決定／A1本体は未着手**。A0I-1〜3でContract Catalog、prepared resolution、製品実行入口を実装したが、現行`FilterPlugin`を別crateから実装するための公開依存、first-party組み立て点、検査器、移動前pixel基線が未確定だった。本書でそれらを固定し、A1実装者が公開APIや例外を発明しない状態にする。

関連文書: [VSM-A0S](2026-07-17-vism-a0s-contract-catalog-spec.md)、[A0D](2026-07-17-vism-a0d-contract-migration-ownership-decision.md)、[Vism実装計画](2026-07-17-vism-implementation-plan.md)、[plugin作者規約](../plugin-authoring.md)

## 1. 入場条件

A1の基線は次の3コミットである。

| ID | commit | 意味 |
|---|---|---|
| VSM-A0I-1 | `cb2c9a7` | contract／catalog／runtime |
| VSM-A0I-2 | `e4f42c6` | Document prepared resolution／Writer |
| VSM-A0I-3 | `057e2e9` | graph／export／製品open |

各コミット単独の対象testと、A0I-3時点の`cargo test --workspace`が全緑であることを確認済み。A1はこの基線より前のworking treeや、A0Iと同居した未コミット差分へ積まない。

上記hashとA1S／A1Gを含む基線をoriginへpushするまで、local `main`をrebaseしてhashを書き換えない。push前にA1を隔離branchへ発注する場合も、このlocal `main`のhash列を明示的な起点とし、別のmainや未コミット差分へ積まない。

## 2. 公開crate境界

### 2.1 façadeを採用する

外部参照plugin crateの通常依存は`motolii-plugin`一つだけとする。`motolii-plugin`はtrait署名と参照実装に必要な型を同じversionで再exportする。

最小再export集合:

- `motolii_eval::{DataTrack, Value}`
- `motolii_gpu::{GpuCtx, PipelineCache, PipelineCacheKey}`
- `motolii_core::{CompCamera, Fps, FrameDesc, Quality, RationalTime}`
- crate façadeとしての`wgpu`
- uniform byte化に必要なcrate façadeとしての`bytemuck`

A1のOpacityが実際に使わない再exportを便宜で増やさない。A2のSineで追加型が必要なら、A2着手前の公開面レビューで追加する。

再exportは内部型を隠すものではない。上記型と、そのOpacity実装が呼ぶ`PipelineCache` APIはpre-Vism静的作者面のsemver責任になる。wgpuをfaçade経由に固定する目的は、作者crateが別versionのwgpuを直接選び、同名の別型でtrait実装不能になる事故を防ぐことにある。

### 2.2 外部参照crateの置き場所

first-party無特権を検査可能にするため、外部参照pluginは`plugins/motolii-plugin-*`に置く。A1は`plugins/motolii-plugin-opacity`を追加する。

このpath配下では次を要求する。

- `[dependencies]`は`motolii-plugin`だけ。
- `[dev-dependencies]`と`[build-dependencies]`は空。
- `build.rs`、独自proc-macro、workspace内部crateへのalias依存を持たない。
- `src/**/*.rs`で`motolii_`から始まるpathは`motolii_plugin::`だけ。
- `tests/**/*.rs`では`motolii_plugin::`に加え、そのpackage自身のcanonical crate名だけを許す。A1では`motolii_plugin_opacity::`であり、任意名の例外allowlistにはしない。
- Slint、OS／vendor API、CUDA／Metal／D3D等を参照しない。
- `motolii-plugin`と同じpanic禁止lintをcrate manifestへ持つ。

crate内testは追加依存を要しない自己完結unit testだけを許す。GPU golden、purity、catalog parity、Host組み立て検査は審判側からplugin crateを読む方向に置き、plugin crateから`motolii-testkit`へdev依存しない。

## 3. 検査器

A1の依存検査はdenylistではなくallowlistである。既存`motolii-plugin/tests/conformance.rs`のCargo table走査とsource token走査を再利用し、次を自動化する。

1. `plugins/motolii-plugin-*`をpathで列挙し、1件以上あることを主張する。
2. normal／dev／build／target別依存をすべて読む。
3. renameされたpackage名も実package名で判定する。
4. `motolii-plugin`以外の直接依存、`build.rs`、proc-macro設定を拒否する。
5. sourceの`motolii_*` pathとpanic経路を走査する。
6. 違反fixtureが赤になることと、実ツリーの違反0件を別testで証明する。

検査の例外allowlistは作らない。必要に見えた時点でA1を停止し、本書へ戻す。

既知の検査限界として、workspace内の同一Cargo buildではdependency featureがunifyされるため、別crateが有効化したfeatureへの偶発的依存を完全には証明できない。A1は直接依存allowlist、source閉集合、実ツリー検査を審判とし、standalone package buildの導入は完了したものとして偽装しない。この限界がOpacityの成立性へ影響した場合は検査例外を足さず停止する。

A1は検査の副産物として、Opacity crateが実際に名指しした公開APIをfixture内の閉集合として固定する。最低限、trait／contract型、`Value`、GPU context、pipeline cache、wgpu型、uniform byte化APIを列挙し、未知の公開pathが増えたら赤にする。これは「現在の契約で書けた」と「この契約が将来も最小である」を混同しないための観測値である。

## 4. first-party composition root

`motolii-plugin`は外部Opacity crateへ依存できない。循環を避け、Hostごとの登録漏れを一箇所で検査するため、A1で内部組み立てcrate`crates/motolii-plugins-firstparty`を追加する。

このcrateだけが次を所有する。

- first-party既定`PluginCatalog`の組み立て。
- first-party既定`PluginRegistry`の組み立て。
- 検証済み`PluginRuntime`の生成。
- Host必須capability ID集合。
- A1前後のcatalog／executor ID parity。

CLI、Document export、製品test helperは個別に`reference_catalog()`と`register_reference_plugins()`を組み合わせず、この組み立てcrateを使う。`motolii-plugin::reference`はA2等の未移動参照実装を一時的に保持してよいが、既定セットのcomposition rootではなくなる。

v1のHost必須capabilityには`core.filter.opacity`を含める。Document graphがenvelope opacityをこのIDへlowerするためである。組み立て後runtimeに必須IDが無い場合はstartup時の型付きエラーとし、graph評価時の偶発的missingまで遅延させない。

## 5. 移動前pixel基線

Opacity本体を動かす前に、独立コミットVSM-A1Gで`motolii-testkit/tests/opacity_filter.rs`を強化する。

必須ケース:

1. `amount=0.0`で全成分0。
2. `amount=1.0`で入力と一致。
3. `amount=0.5`で、alphaと色が異なる非単色・非一様のpremultiplied RGBA入力を成分ごとに半減。
4. 既存のRgba8Unorm経路を維持し、期待値をA1本体で変更しない。

GPU無しでpixel gateが空洞化しない審判コマンドは既存のskip policyを使う。

```sh
MOTOLII_REQUIRE_GPU=1 cargo test -p motolii-testkit --test opacity_filter
```

`gpu_or_skip`はこの環境変数下でskipせず失敗するため、新しいskip機構は追加しない。通常のローカル実行では従来どおりGPU欠落skipを許す。

## 6. A1本体の完了条件

- `plugins/motolii-plugin-opacity`が`core.filter.opacity` version 1のcontractとexecutorを所有する。
- `motolii-plugin::reference`からOpacity contract、executor、WGSL、ID断言を削除し、同等物を重複保持しない。
- `motolii-plugins-firstparty`が移動前と同じcatalog／executor ID集合を組み立てる。
- Host必須ID検査が`core.filter.opacity`欠落fixtureを拒否する。
- A1Gの期待値を変更せず全pixel caseが通る。
- assembled runtimeに対するpurityが通り、Opacity離脱で列挙件数が静かに減らない。
- external crate allowlist、panic scan、vendor denylistが通る。
- 公開API利用閉集合fixtureが通る。
- `cargo test -p motolii-plugin`
- `MOTOLII_REQUIRE_GPU=1 cargo test -p motolii-testkit --test opacity_filter`
- `cargo test -p motolii-testkit --test purity`
- `cargo test --workspace`
- `git diff --check`

## 7. A1本体の採択単位

A1本体は次の3コミットを直列に作り、各段を独立してreview可能にする。

1. **VSM-A1-1 — façade公開面**: A1Sで列挙した最小再exportと、外部pathからその公開面が存在することだけを証明するcompile test。composition rootやOpacity実装は動かさない。
2. **VSM-A1-2 — first-party composition root**: `motolii-plugins-firstparty`を導入し、CLI、Document export、製品test helperを一箇所の組み立てへ移す。この段ではOpacityを`motolii-plugin::reference`内に残し、catalog／executor ID集合と挙動を変えない。
3. **VSM-A1-3 — Opacity外部crate化**: Opacity contract／executor／WGSLを外部crateへ移し、旧実装を削除する。依存検査、必須capability負例、公開API利用閉集合、列挙parity、assembled purityをこの段で完成させる。

後段の都合を前段へ混ぜない。各段は対象testと`cargo test --workspace`を通してからmainへ採択し、3段をsquashしない。

## 8. 非目標と停止条件

- `.vism` package、manifest、loaderを作らない。
- OpacityのID、version、parameter、pixel、clamp意味を変えない。
- `FilterPlugin`署名や`PipelineCache`を再設計しない。A1は現契約が外部から成立するかの実証である。
- private crate依存の例外、lint抑制、testkitへのdev依存を足さない。
- golden期待値を書き換えて移動差を正当化しない。
- `new_plugin.py`の外部crate生成対応はA2で二例揃うまで延期する。

façade再exportだけでOpacityを書けない場合、実装者は代替helperや新traitを追加せず停止し、必要だった型・操作・最小fixtureをGAPとして返す。
