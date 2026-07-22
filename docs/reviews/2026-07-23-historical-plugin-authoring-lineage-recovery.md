# Plugin authoring lineageの価値回収（Unit 3B-runtime-B1、2026-07-23）

状態: **決定**（`plugin-authoring.md`歴史41 blobの処分、authoring/distribution境界の訂正）

対象: `plugin-authoring.md`のcutoff全41版。

関連: [全歴史coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)、[plugin作者向け規約](../plugin-authoring.md)、[Vism実装計画](2026-07-17-vism-implementation-plan.md)、[Creator / Developer連続体](2026-07-22-creator-developer-continuum-decision.md)

## 1. 結論

単一path / 41 blobを、初版全文、主lineageの全差分、merge親差分、分岐版、cutoffで到達したcommit非対応2 blobの近接差分まで処分した。履歴は、最初のstatic plugin作法から次の順で境界を育てている。

```text
4種trait + NodeDesc + 純関数 + WGSL
  → Host所有cache・時間依存の予約
  → scaffold・purity・typed params
  → custom UI停止線・hostless配布案との接続
  → 表現を再利用／配布する北極星
  → Authoring Tool / Behavior / Generatorの責任分類
  → Contract Catalogと外部first-party crate実証
  → Vism / Kitとstatic capabilityの分離
  → product surfaceとplugin UIの軸分離
  → creatorとdeveloperを一つの学習曲線へ統合
```

現行へ維持する判断は五つである。

1. ID、kind、parameter、入出力、失敗、純関数等の意味契約と、`'static` trait object、Rust ABI、WASM/native payload、配布形式を分ける。現行static型を将来loaderへ外挿しない。
2. Opacity、Sine、Radial Repeaterは公開`motolii-plugin` façadeだけへ依存する別workspace crateで、bundled first-party無特権のコード実証である。ただし第三者install/load/distributionの実証ではない。
3. `new-plugin.sh`は今も`motolii-plugin`内へ貼る参照実装とHost側testを生成するin-tree toolであり、workspace plugin crate、manifest、composition、Vism packageを作らない。A1で「A2の二例後」へ延期された外部crate scaffoldは、A2/A3完了によりVSM-A4へ再入場できる。
4. `PluginKind::Input / Simulation / ScriptWasm`は予約だけである。特に「native plugin」はstaticに組み立てたRust first-party実装と、OS別binaryを動的loadする将来方式を分ける。WASMもenum名からruntime、sandbox、配布を推測しない。
5. Filter/Composite/LayerSource/ParamDriverは現行実行分類で、Authoring Tool / Behavior / Generatorは責任分類候補である。自由なDocument mutation、timeline走査、custom UI、隠れstateで未実装能力を偽装しない。

## 2. lineage別の処分

| 時期／歴史主張 | 分類 | 判定 | 現在の回収先 |
|---|---|---|---|
| 初版、rebrand、NodeDesc、WGSL、純関数、正準座標 | **現行規範 + 成立理由** | ID/version、GPU/CPU境界、純関数、型付き失敗は維持。旧crate名と初期skeleton signatureはarchiveのみ | [plugin作者向け規約](../plugin-authoring.md) §2〜7 |
| simulation禁止branch → 時間軸自由度のはしご → merge | **訂正の成立理由 + 負例** | 物理自体の全面禁止ではなく、安い`f(t)`、DataTrack、宣言時間窓、Host所有Simulation Bakeへ分けた訂正を維持。`SimulationPlugin`は未実装 | 同§3/§4.5、[simulation model](../simulation-model.md) |
| PipelineCache、AssetRef、lookbehind、cache key | **現行規範 + 予約** | PipelineCache/AssetRefと予約型は成立。GpuAssetCache/Importer/Feedback executorは未実装。詳細は直前Unit 3B-runtime-Aで訂正済み | [resource歴史回収](2026-07-23-historical-plugin-resource-runtime-lineage-recovery.md) |
| INF-7e/f、M2E-7/8/10 | **実装済み作者支援 + 未完の一般化** | in-tree scaffold、purity、RenderCtx、typed accessor、分離testは生存。scaffoldを外部crate作者入口と称さない | 本書§3、[plugin作者向け規約](../plugin-authoring.md) §5 |
| plugin UI初期案、v1停止線、egui/Web再評価 | **現行停止線 + 訂正済み旧案** | NodeDesc fallbackとcustom UI非公開は維持。Slint/egui/Webという製品runtimeからplugin分類を導かず、G0-3/GAP-13へ分離 | [plugin UI歴史回収](2026-07-23-historical-plugin-ui-lineage-recovery.md) |
| plugin ecosystem／tap／GitHub配布への参照 | **再入場候補** | hostless配布、lock、Kit等の価値はUnit 1で回収済み。authoring guideから配布完成を推論しない | [負けた仕様の回収](2026-07-23-losing-specification-value-recovery.md)、Unit 9予定 |
| Visual Module／VST／演奏語彙branches | **成立理由 + 撤回済み比喩** | 表現をproject手順から切り離す目的は生存。「演奏」「楽曲」を製品全体の存在論にしない。Vism名と`.vism`だけ決定済み | [音楽比喩撤回](2026-07-22-ui-music-metaphor-retirement.md)、[Vism concept](../vism-package-concept.md) |
| Authoring Tool / Behavior / Generator分類 | **比較中の価値** | 評価pluginへDocument mutationを足さない分界として維持。trait名、capability、wireは未実装で固定しない | [extensible core](../extensible-core-model.md)、本書§5 |
| A0I→A1/A2/A3外部first-party | **実装済み境界 + 誤読防止** | public façade、Contract Catalog、composition root、allowlistは成立。package/load/install/trustは未成立 | 本書§3〜4、[Vism計画](2026-07-17-vism-implementation-plan.md) |
| creator/developer連続体 | **現行決定 + 未完成の審判** | first-partyを実行可能な手本にする原則は維持。sourceとtestがあるだけで作者入口全体が完成したとはしない | 本書§3、[連続体決定](2026-07-22-creator-developer-continuum-decision.md) |

## 3. 発見したauthoringの空洞

### 3.1 何がコードで成立しているか

- `plugins/motolii-plugin-opacity`
- `plugins/motolii-plugin-sine`
- `plugins/motolii-plugin-radial-repeater`
- 各crateの通常依存は`motolii-plugin`だけで、dev/build dependency、`build.rs`、private Host crate依存を持たない。
- `motolii-plugins-firstparty`がContract Catalog、registry、必須capabilityを組み立てる。
- Host側testkitがGPU golden、purity、catalog/executor parityを検査する。

これは「Host内部APIを使わず別crateで高度な表現を書ける」という重要な実証である。

### 3.2 何がまだ手本から生成できないか

現行`new-plugin.sh`はproduct sourceを`motolii-plugin`内へ貼り、test sourceを`motolii-testkit/tests`へ置くM2E-10形を生成する。次は生成しない。

- `plugins/motolii-plugin-*`のCargo package
- 依存allowlist/panic lintを備えたmanifest
- first-party composition rootへの型付き登録
- Host側conformance/purity/goldenの外部crate向け配置
- standalone build、package、install、署名、trust、権限

A1S §8は外部crate生成対応を「A2で二例揃うまで延期」した。現在はA2に加えてA3も完了しているため、延期条件は解消済みである。本単位はVSM-A4を**再入場可**へ戻す。ただしauthoring scaffoldとVism package/installを一発注へ束ねない。

## 4. native / WASM / staticの語彙を分ける

| 語 | 現在の意味 | 現在ないもの |
|---|---|---|
| native presentation | Stage/Timeline等をRust/wgpuで描く製品surface | pluginであることの証明ではない |
| native first-party implementation | Rust plugin crateをHost binaryへ静的に組み立てる | 動的install/load/unload、OS別artifact配布ではない |
| native payload candidate | 将来VismがOS別binary等を持つ比較候補 | ABI、権限、crash isolation、署名、version併存は未決 |
| `ScriptWasm` | `PluginKind`の予約variant | trait、registry、runtime、memory/time limit、Host import、packageは無い |
| WGSL | 現行Rust pluginがHostのwgpu境界で用いるshader source | WGSL単独Vismのmanifest、validator、resource modelは未決 |

Vismのpayload classはDeclarative / WGSL / source+Host build / WASM / nativeをVSM-B4/C2で比較する。`v2`という時期名やenumの存在を方式採択にしない。

## 5. 現行kindと将来capabilityを混ぜない

現行`ParamDriverPlugin`は外部structured inputを取らない。入力はparams、区間、sample rateで、`DataTrack`を一つ返す。古い表の「構造化データ→DataTrack」をtyped provider/consumer実装済みの証拠にしない。

同様に、位置やscaleを編集したい要求を次へ分解した歴史価値は維持する。

- 一回で通常Documentへ実体化するAuthoring Tool候補
- 時刻や入力変化後も関係が続くBehavior / Driver候補
- 独自recipeを持つGenerator / Structured Recipe候補

ただしこれは責任寿命の分類で、公開traitでもVism manifest fieldでもない。現在の4 traitへ`&mut Document`、名前検索、UI callback、独自Undoを足して先行実装しない。

## 6. 復活させない旧具体とSTOP線

- Rust dylib、WASM、source build、native artifactのいずれも既定Vism payloadと書かない。
- `PluginKind::ScriptWasm`をWASM runtime実装済みまたはv2採択済みと数えない。
- static first-party crateを第三者plugin install/loadと称さない。
- `new-plugin.sh`を外部crate／Vism作者scaffoldと称さない。
- external crate scaffoldへpackage manifest、marketplace、signature、loaderまで束ねない。
- plugin crateから`motolii-testkit`へdev依存して試験を自己完結させない。Hostが反対側から審判する向きを維持する。
- ParamDriverへ入力portを既存表の文言だけで追加しない。
- Visual Module、VST、演奏の旧語彙を製品全体の存在論へ戻さない。
- first-party手本を理由にtrust、sandbox、permissionをfirst/third-partyで同一視しない。

## 7. 固定歴史出典

初版`f9d840f9`を全文で読み、rebrand、simulation/cache分岐とmerge、INF-7/M2E、UI/ecosystem分岐、表現北極星、Vism A0I/A1/A2/A3、runtime再評価、creator連続体まで全親子diffを確認した。cutoff manifestに存在するが`git log --all -- <path>`へ直接対応しない`7ffb3828`と`ad17be61`は、相互差分と主lineage近接版との差分を確認した。前者はA0I/Vism typed input追補なしの枝、後者はA0I追補あり・A1前の枝であり、現在値として戻さない。

41 blobの完全SHAは機械receipt `03d-plugin-authoring.tsv`を正本とする。これらは本書でDISPOSITIONEDとする。Vism package／Kit／payload／hostless distributionの各lineageはUnit 9、native/WASM runtimeの横断残余はUnit 3B-runtime-B2で別に処分する。
