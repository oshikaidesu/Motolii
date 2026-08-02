# M5 既知実装採択・検証地図

状態: **技術採択・意味decision recovery済み／検証済み／製品runtimeはObservation・resource gate待ち**（2026-08-02）

## 1. 目的と入力

M5を既存`P0I`〜`P7`の順に独自実装せず、利用者成果を成立させる機構class、現行codeの接続先、
既知実装、Motoliiだけに残す責任、oracle、cutoverを先に固定する。

調査の入力は次の順とする。

1. 現行`main`の[M5仕様](specs/M5-3d-and-post.md)、[決定逆引き台帳](decision-index.md)、
   [実装台帳](implementation-ledger.md)
2. 現行codeの`CompCamera`、`Asset`、`LayerSourcePlugin`、`RenderSession`、wgpu／Vello経路
3. 未統合branch `codex/m5-3d-import-design-20260801` のcommit `416aa2c2`
4. `main`外の歴史commit `33e957df` にあるRender Contribution締結地図とdecision群
5. Rerun固定commit `954bf95a`の監査済みassetと、2026-08-02に再確認した公式一次資料

`416aa2c2`と`33e957df`は有用な設計入力だが、現行`main`へ統合済みのauthorityとは扱わない。
同じ内容を再相談・再発注せず、main正本との差をdecision recovery粒で処分してから採用する。

## 2. 利用者成果

M5の最初の通常製品成果を、次の4本へ分ける。

1. Blender／Cinema 4D等から書き出した通常のGLBを1素材として読み込み、対応範囲を忠実に表示し、
   非対応要素を無言で捨てず診断できる。
2. 2D、動画平面、text、mesh、pointを単一world／単一active cameraで扱い、Layer Orderと
   Group Depthを切り替えても座標、選択、Undo、Preview／Exportの意味が変わらない。
3. 日本語を含むtextを、font fallback、shaping、cluster対応、variationを失わずGPUへ描ける。
4. Duplicator／Behaviour／postを組み合わせても、同じDocument、時刻、入力、seedから同じ結果を得る。

「3D engineを作る」「scene frameworkを採る」「task IDを消化する」は利用者成果ではない。

この地図は技術routeを確定する。性能、3 OS適合、fixture忠実性は未証明であり、
`codex/m5-known-implementation-plan-20260802`上の子検証を通過するまで製品依存やruntime完成へ読み替えない。

## 3. 決定済み・未決・未実装

### 3.1 現行mainで決定済み

- 単一Y-up正準XYZ world、単一active camera、2D／3Dを別世界にしない。
- 現行`PlanarOrthographic`はpixel互換baselineであり、P3前にSpatial Observationへ偽装しない。
- Preview／Exportは同じrendererを使い、差は`Quality`だけにする。
- 色変換は合流点の一箇所だけ。3D／text／postはlinear-lightで処理する。
- Layer Order／Group Depth／AE-style Binsの意味、soft alphaの無言depth格上げ禁止。
- Stage gizmoはcanonical出力外のnative wgpu overlay、少数の既知形状はCPU解析hit-test。
- Duplicatorは`InstanceId != index`、明示seed、PCG32、hidden stateなし。
- P6はrun単位のitemize／shape／drawを要求し、縦書きはv1非目標。

### 3.2 技術採択決定とdecision recoveryの分離

10機構classの技術routeは本書で決定した。これは`M5-A0T`の完了であり、既知実装の再選定を
各runtime粒で繰り返さない。一方、次の作品意味・公開境界は`M5-A0S`として別に回収する。

- `416aa2c2`: faithful import assetとrenderer-compiled assetの分離、core PBR＋neutral environment、
  `gltf`／`tobj` private leaf、bare一灯／自動unlit縮退の拒否。
- `33e957df`: requirementとcontributionの分離、whole-request admission、Host resource owner、
  linear-premultiplied scene color、soft alpha typed unsupported、format／copy／budget evidence gate。

この2系列は内容を再発明せず、現行M5仕様との矛盾、失効ID、main未収載の理由を確認した。
`M5-A0S`で`縮小採用／観察／棄却（archive-only negative）`へ一度だけ処分済みであり、
詳細は[M5-A0S決定回収](reviews/2026-08-02-m5-a0s-decision-recovery.md)とdelta receiptに固定する。

### 3.3 未決

- spatial Observationの具体公開形、camera capability閉集合、provider pinningのschema。
- faithful importの初期入力をGLBだけにするか、外部URIを持つ`.gltf`まで同時に許すか。
- 具体scene-color GPU format、copy／alias method、hard budget。
- 3D objectのdense picking方式。gizmo CPU hit-testとは別問題とする。
- P6の公開run意味。実装は現行Fontique／HarfRust／Velloを基準にし、BiDi／script itemizeやfallbackを
  手書きする必要が出た場合だけParleyのprivate leafを比較する。
- post blur、grain、LGGの具体algorithm、RoI padding、低スペックquality ladder。

### 3.4 未実装

- glTF／OBJ importer、faithful private asset、GPU compiled asset、3D asset cache。
- spatial renderer、typed Observation、camera provider、製品Group Depth経路。
- 汎用`motolii-text`、fallback診断、cluster／variation公開口。
- 製品post node、dense object picking、bounds derived cache、3D gizmo。
- Duplicator schema／runtime／Behaviour／製品UI。

## 4. 現行code fact

| 機構class | 成立済み | gap |
|---|---|---|
| Asset identity | `motolii-doc::Asset`に安定ID、type、content hash、path、size | payload importerとGPU asset cacheなし |
| Camera | `motolii-core::CompCamera`と`PlanarOrthographic`評価／拒否試験 | perspective／spatial Observationなし |
| Render | `LayerSourcePlugin`→wgpu texture→`RenderSession` composite | mesh／material／shared depthの製品経路なし |
| GPU lifecycle | wgpu 29、pipeline cache、dynamic target再利用 | M4 hard budget／owned lifetime未成立 |
| Text drawing | UI内にFontique 0.10、HarfRust 0.7、Vello 0.9の局所経路 | fallback／cluster／variationを持つ汎用P6なし |
| Vello | native Timelineのshape／glyph／overlay | M5 text／vector effect／post ownerではない |
| Picking／gizmo | ownership decision、CPU解析hit-test方針 | dense scene object pickingと3D overlayなし |
| Duplicator | 意味decisionとtest条件 | schema／runtime／GPU instanceなし |

UI内の局所text描画やprivate depth fixtureを、M5製品runtimeの完成証拠へ昇格しない。

## 5. 公式sourceの固定点

2026-08-02に各公式repositoryの`HEAD`を再確認した。採用時はrelease tag、Cargo.lock、license、3 OS、
fixtureを別途固定する。HEADは調査再現点であり、そのまま製品versionではない。

| 候補 | 調査固定点 | 証明するもの | 証明しないもの |
|---|---|---|---|
| `gltf-rs/gltf` | `50d65229477fe5f785c2c90df21eb59c93ea2261` | glTF 2.0 parser、feature別extension、MIT/Apache | Motoliiの受理範囲、URI安全、faithful renderer |
| `tobj` | `076344c2f74d546956ed82a3ea458309b6df5269` | 軽量OBJ／MTL parser、triangulation制約 | OBJ→glTF忠実変換、PBR意味 |
| `mikktspace` | `6275cc4f15cff8be29819fb34ae8be3b9129dae1` | MikkTSpace tangent生成のRust実装 | mesh admission、normal-mapの製品忠実性 |
| Khronos Validator | `434283be08a668a8fb4e437145630ddbf93b0686` | schema、accessor、animation、extension検査 | Host resource policy、renderer capability |
| Khronos Asset Generator | `3d99767e9a67fbfe109f0d298c1e8d909bcac9db` | importer負例を含む生成fixture | Motoliiの受理policy、全extension対応 |
| Khronos Sample Assets | `2bac6f8c57bf471df0d2a1e8a8ec023c7801dddf` | core／testing assetと個別license | Motoliiのgolden許容差 |
| Khronos Sample Renderer | `7ca60cc82aae21d4c88a0d3541d7fbde9b253b54` | glTF PBRの参照renderer／shader pattern | Motoliiのwgpu実装、性能、resource owner |
| `glam` | `bd172a701971499191b8af85ed4d299e04057b08` | graphics math、SIMD対応vector／matrix／quaternion | Document座標意味、serde／公開型 |
| `rand_pcg` | `7592cf749b7f5158a37e74533e3428c3341edbac` | portable PCG generatorと参照vector | seed mixing、InstanceId意味 |
| `obvhs` | `5cb74827ab33c0ab76e3379380344e955ecce1d3` | CPU BVH build／traversal候補 | Motolii dense pickingの勝者、更新費用 |
| Rerun | 現行監査anchor `954bf95a`、再確認HEAD `4d4333cf3e2c1d97f2b0e26f18b5c87f79d57b99` | importer分割、wgpu renderer、picking／outline、resource poolの実例 | Motoliiのscene、Document、plugin契約 |
| Bevy | `25368b78ce5e9b15dc770cdf2af4595602cc8a7b` | PBR、depth、picking、gizmo、post、glTFの比較実例 | ECS／schedule／asset ownerの採用理由 |
| renderling | `a7b44f796a38cb2c734d69354fa35f1288aa02a1` | headless wgpu、glTF、PBR／IBL、image testの比較候補 | 成熟度、rust-gpu供給責任、Motolii統合適合 |
| rend3 | `d088a841b0469d07d5a7ff3f4d784e97b4a194d5` | 過去のwgpu renderer分割例 | 新規製品依存の保守性 |
| Vello | `b377de1be0f93ba2d1c651e3d654b66f1107a720` | wgpu 2D scene／glyph描画 | 完成したblur／filter、低スペック保証 |
| Parley | `78de830e4ef1ab6d3558f92d815ca40f2ab98eaf` | Fontique＋HarfRust＋Skrifa＋ICU4Xのlayout統合 | Motolii P6公開API、縦書き採用 |
| HarfRust | `bd0a9d22a54257b34be879ae394476d52dbc0917` | pure Rust shaping、HarfBuzz追従 | 全HarfBuzz conformance、system font integration |

一次資料:

- [`gltf-rs/gltf`](https://github.com/gltf-rs/gltf)
- [Khronos glTF Validator](https://github.com/KhronosGroup/glTF-Validator)
- [Khronos glTF Sample Assets](https://github.com/KhronosGroup/glTF-Sample-Assets)
- [`tobj`](https://github.com/Twinklebear/tobj)
- [`mikktspace`](https://github.com/gltf-rs/mikktspace)
- [Khronos glTF Asset Generator](https://github.com/KhronosGroup/glTF-Asset-Generator)
- [Khronos glTF Sample Renderer](https://github.com/KhronosGroup/glTF-Sample-Renderer)
- [`glam`](https://github.com/bitshifter/glam-rs)
- [`rand_pcg`](https://github.com/rust-random/rngs/tree/master/rand_pcg)
- [`obvhs`](https://github.com/DGriffin91/obvhs)
- [Rerun architecture](https://github.com/rerun-io/rerun/blob/main/ARCHITECTURE.md)
- [Bevy examples](https://github.com/bevyengine/bevy/blob/main/examples/README.md)
- [renderling](https://github.com/schell/renderling)
- [Vello](https://github.com/linebender/vello)
- [Parley](https://github.com/linebender/parley)
- [HarfRust](https://github.com/harfbuzz/harfrust)

### 5.1 Fable 5 read-only助言の処分

2026-08-02にFable 5へ、10 classの代替候補、過大依存、見落としたfixture、private/public境界を
read-onlyで相談した。出力は採択authorityや検収判定ではなく、Codexが上記一次資料と現行code factへ
再照合する反対側助言として扱った。処分は次のとおり。

- 10 classのrouteは全て`KEEP`し、class自体の置換は行わない。
- importerへ`mikktspace`、camera／transform／boundsへ`glam`、Duplicatorへ`rand_pcg`を
  private leafとして追加する。
- Khronos Asset Generatorをimport負例、Sample RendererをPBR patternへ追加する。
- dense pickingは`obvhs`／owned flat BVH／async GPUを比較し、狭い用途へ`parry3d`を採らない。
- Parleyは全面採用でなく、手書きBiDi／script itemizeを避ける条件付きprivate比較に縮小する。
- renderlingは必須spikeから任意・非gateの外部比較へ降格する。
- PCG32のseed mixingに暗号hashを流用せず、小さいowned mixerと固定golden vectorを持つ。

## 6. 機構class別の採択裁定

| 親 | route | 接続target | Motoliiに残す薄い責任 | 拒否／保留 |
|---|---|---|---|---|
| A. glTF／OBJ import | `ADOPT`: `gltf`／`tobj`／`mikktspace` private leaf。`EXTERNAL`: Khronos Validator／Asset Generator／Sample Assets。Rerunは`PATTERN` | `Asset` metadata→Host importer→private faithful asset | admission、URI／size policy、axis／unit／tangent normalization、typed diagnostic、content identity | parser成功を製品成功にしない。Assimp／USD sceneを初期導入しない |
| B. scene／object representation | `REUSE`: 単一world、既存transform、AssetRef、LayerSource意味。Rerun／Bevy sceneは`PATTERN`のみ | faithful asset＋Document projection→private evaluated object list | stable semantic identity、world transform、source intent、derived bounds | engine scene graph／ECS entity／serdeを第二のDocumentにしない |
| C. spatial renderer | `REUSE`: wgpu／現行RenderSession。Khronos Sample Renderer／Blender、Rerun／Bevyは`PATTERN`。renderlingは任意の隔離比較 | private compiled asset→既存LayerSource／Render Contribution | single world、Observation consumption、premul linear output、Quality、typed refusal | renderlingをoracleやgateにしない。Bevy ECS、Rerun store、renderling ownerを輸入しない。rend3は`REJECT` |
| D. camera／Observation | `REUSE`: Planar baselineとCamera Provider決定。`ADOPT`: `glam` private math leaf。engine cameraは`PATTERN`のみ | Host評価→representation非依存typed Observation | active binding、provider pin、capability preflight、bounds／picking参加 | `glam`型をDocument／serde／公開APIへ出さない。P3前のpublic API、具体provider ID分岐を禁止 |
| E. depth／Render Contribution | `REUSE`: M5 depth decisions、現行composite。Rerun／Bevyは`PATTERN` | Host policy→admitted contributions→shared attachments | whole-request admission、alpha class、ordering、resource owner | historical private seamの公開昇格、soft alphaのopaque化を禁止 |
| F. text | `REUSE/WRAP`: 現行Fontique＋HarfRust＋Vello。ParleyはBiDi／script itemize／fallbackの条件付きprivate比較 | `motolii-text` run API→Vello `draw_glyphs` | 同梱CJK下限、fallback診断、cluster／variation、純関数run | BiDi／script itemizeを自作しない。layout／editing API全体を公開しない |
| G. Vello局所pass | `REUSE`: 現行Vello dependencyとrenderer setup | text／shapeの局所2D scene→既存GPU合流 | adapter、premul、resource reuse、capability refusal | blur／postの万能backendにしない。frame内resource生成を増やさない |
| H. post | `REUSE`: wgpu／pipeline cache。Bevy等はalgorithm／fixtureの`PATTERN` | existing filter/render graph→linear GPU pass | RoI padding、Quality ladder、same preview/export、golden | Vello blurは未完成なので採用しない。scene engine post stackを持ち込まない |
| I. picking／gizmo／bounds | `REUSE`: native Stage overlayとCPU gizmo hit-test。dense pickingは`obvhs`／owned flat BVH／async GPUを比較、Rerunは`PATTERN` | read-only projection→Transient selection→既存D2 commit | semantic ID mapping、async generation、Unknown bounds、no canonical pixels | `parry3d`をpickingだけに採らない。gizmoへGPU ID readback、dense pickingの同期readbackを禁止 |
| J. Duplicator | `REUSE`: P0I/P7決定、GPU instance。`ADOPT`: `rand_pcg` private PCG32。seed mixingは固定したowned SplitMix64-style mixer | stable slot key→InstanceId→typed channels→GPU instance | domain別identity、nested context、seed、Behaviour純関数 | `sha2`をseed mixingに使わない。ECS entity／array indexをidentityにしない |

## 7. 実装可能な子地図

ここでいう子は将来の一契約境界であり、この文書だけではdispatchしない。

| 子 | 結果 | 既知route | exact接続target | 正例／負例oracle | 依存／cutover |
|---|---|---|---|---|---|
| M5-A0T | 10 classの技術routeを決定 | 本書の`ADOPT/REUSE/WRAP/PATTERN/EXTERNAL/REJECT` | M5 spec、decision-index、ledger | 全classにtarget／oracle／拒否／retirementあり | **DONE**。本書のcommit |
| M5-A0S | 2系列の作品意味decisionをmain正本へ処分 | `REUSE` `416aa2c2`／`33e957df` | M5 spec、decision-index、ledger | 7 blobを縮小採用／観察／棄却へ処分、失効IDをDOへ戻さない | **DONE（docs-only）**。runtime前 |
| M5-A1 | GLB全体preflightとdiagnostic | `ADOPT` `gltf`／`mikktspace`、`EXTERNAL` Validator／Asset Generator | Host importer→private faithful asset | positive／malformed／required ext／oversize／escape URI／tangentなしnormal map | **DONE / KEEP**。[receipt](reviews/evidence/m5-known-implementation/M5-A1/README.md)。製品依存は未追加 |
| M5-A2 | OBJを同じprivate assetへlower | `ADOPT` `tobj` | A1 faithful asset | triangle／normal／UV／MTL欠落を明示。無言PBR化なし | **DONE / KEEP（private leaf）**。[receipt](reviews/evidence/m5-known-implementation/M5-A2/README.md)。製品入力・依存は未追加 |
| M5-R0 | core PBR／unlit headless検証 | wgpu `REUSE`、Khronos Sample Renderer／Blender `PATTERN` | private compiled asset→offscreen target | Khronos metal／dielectric／normal／emissive、cold／warm、low-spec refusal | **DONE / KEEP**。[receipt](reviews/evidence/m5-known-implementation/M5-R0/README.md)。製品material／renderer未接続 |
| M5-R1 | Layer Orderへ3D contribution接続 | R0採択route | existing LayerSource／RenderSession | 3D未使用pixel不変、premul、Preview／Export一致 | R0＋M4 resource owner |
| M5-R2 | Group Depth opaque／cutout | Render Contribution決定を`REUSE` | Host depth policy→same material system | Z交差、cutout、soft alpha typed refusal、group外不変 | P3 Observation＋resource gates |
| M5-C0 | PlanarとSpatialのObservation decision | Camera Provider決定を`REUSE`、`glam` private leaf | Host camera eval→typed Observation | provider 2種、3 OS golden、capability拒否、Planar pixel不変 | **WAIT / SPECIFY前のpreflight完了**。[preflight](reviews/2026-08-02-m5-c0-observation-preflight.md)。public API・M4 K1a前 |
| M5-T0 | P6 run APIと条件付きParley採否を比較 | Fontique／HarfRust／Vello `REUSE`、Parley itemize比較 | new private `motolii-text` leaf→Vello | CJK＋Latin＋emoji＋RTL、fallback、cluster、variation、missing glyph diagnostic | **DONE / KEEP + REDUCE**。[receipt](reviews/evidence/m5-known-implementation/M5-T0/README.md)。手書きitemize禁止。variationは固定variable font待ち。公開契約は比較後 |
| M5-P0 | Blur/LGG/grain algorithm survey＋fixture | wgpu `REUSE`、既知shader `PATTERN` | filter graph／pipeline cache | RoI radius、Unknown全域、linear、Draft/Final | **DONE / KEEP（algorithm contract）**。[receipt](reviews/evidence/m5-known-implementation/M5-P0/README.md)。GPU pass／M4 ownerは未接続 |
| M5-I0 | dense object picking比較 | `obvhs`／owned flat BVH／Rerun-style async GPU | Stage projection→Transient selection | 10k object moving camera、stale generation、readback stall 0、same semantic ID | **DONE / KEEP + REDUCE**。[receipt](reviews/evidence/m5-known-implementation/M5-I0/README.md)。CPU semantic一致とstale拒否まで。GPU readback／Stage接続は未完了 |
| M5-D0 | stable instance evaluator | P0I/P7 decision、`rand_pcg`、owned stable mixer | input shape→slot key→InstanceId→channels | **DONE / KEEP（test-only meaning fixture）**。[receipt](reviews/evidence/m5-known-implementation/M5-D0/README.md)。count増減／reorder／nested／thread順／golden vectorを確認。schema／3 OSは未接続 |

## 8. 推奨順序

1. **技術採択**: `M5-A0T`として本地図を確定する。**DONE**。
2. **独立検証**: `M5-A1`、`M5-R0`、`M5-T0`、`M5-P0`、`M5-I0`、`M5-D0`は**DONE**。
  このbranch上で一粒一commitとして閉じた。依存しない粒は並行可能だがdiffを束ねない。
3. **意味decision recovery**: `M5-A0S`で`416aa2c2`と`33e957df`を処分した。**DONE（docs-only）**。
4. **意味decision**: 検証証拠からP1/P2境界、`M5-C0` Observation、scene-color format／resource gateを閉じる。ここが次のgate。
5. **薄い接続**: Layer Orderを先に通常製品routeへ接続し、Group Depth、picking、Duplicatorを後続にする。
6. **cutover**: test-only adapterや旧局所copyを、同一oracleが成立した子ごとに`FROZEN → RETIRE`する。

3D import、renderer、camera、depth、Document schema、UIを一つの発注へ束ねない。

## 9. このbranchでの検証契約

- 検証branchは`codex/m5-known-implementation-plan-20260802`とする。mainのdirty worktreeへ混ぜない。
- 一つの子検証は一つのcommitとし、code／fixtureは`spikes/m5-known-implementation/<child>/`、
  receiptは`docs/reviews/evidence/m5-known-implementation/<child>/`へ置く。
- 各receiptはsourceのrelease／commit、license、実行command、正例／負例、計測条件、結果、
  `KEEP / REMAP / REDUCE / REJECT`、製品採用時のretirementを記録する。
- 比較だけの依存は通常workspaceと`Cargo.lock`へ入れない。private leafの製品依存追加は、検証合格後の
  独立採択commitで行う。renderlingはworkspace外の任意比較であり、未実行でも`M5-R0`を止めない。
- 検証中はDocument／serde／公開API／plugin契約、golden／thresholdを変更しない。
- `M5-A0S`はdocs-onlyの歴史意味処分であり、P3の意味decision、M4 resource gateを通るまで、
  検証成功を製品runtime完成と報告しない。

## 10. STOPと再調査条件

- engine／scene／ECS型をDocument、公開API、plugin契約、serdeへ出さないと成立しない。
- parser、renderer、text stackのdefaultからMotoliiの未決意味を逆算する。
- `416aa2c2`／`33e957df`をmain統合済みまたは製品実装済みと報告する。
- Rerun inventoryの候補分類だけで依存、vendoring、portを決める。
- Vello局所成功をpost／text layout／低スペック完成へ外挿する。
- renderlingの機能一覧だけでrust-gpu、GPU allocator、scene ownerを採用する。
- renderlingをPBR oracleまたはM5-R0の必須gateにする。
- BiDi／script itemizeを手書きする、`sha2`をseed mixingへ使う、pickingだけのため`parry3d`を採る。
- `glam`、`gltf`、`tobj`、`rand_pcg`等のprivate型をDocument／serde／公開APIへ漏らす。
- 固定fixture、3 OS、low-spec計測前に高性能または性能勝者と報告する。
- soft alpha、unsupported extension、欠落texture、budget不足を別表現へ無言縮退する。
- test goldenやthresholdを既知実装の出力へ合わせて変更する。

再調査は、固定fixtureでoracle不成立、license／maintenance／3 OS不適合、低スペックhard floor超過、
security上の入力拒否不足、または既知routeが公開境界を侵食する反証が出た場合だけ行う。
