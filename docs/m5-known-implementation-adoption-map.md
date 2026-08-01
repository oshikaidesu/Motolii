# M5 既知実装調査・暫定採択地図

状態: **調査完了／暫定採択地図／製品runtime実装は未許可**（2026-08-02）

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

### 3.2 decision recoveryが必要

- `416aa2c2`: faithful import assetとrenderer-compiled assetの分離、core PBR＋neutral environment、
  `gltf`／`tobj` private leaf、bare一灯／自動unlit縮退の拒否。
- `33e957df`: requirementとcontributionの分離、whole-request admission、Host resource owner、
  linear-premultiplied scene color、soft alpha typed unsupported、format／copy／budget evidence gate。

この2系列は内容を再発明せず、現行M5仕様との矛盾、失効ID、main未収載の理由を確認し、
採用／縮小採用／棄却を一度だけ処分する。

### 3.3 未決

- spatial Observationの具体公開形、camera capability閉集合、provider pinningのschema。
- faithful importの初期入力をGLBだけにするか、外部URIを持つ`.gltf`まで同時に許すか。
- 具体scene-color GPU format、copy／alias method、hard budget。
- 3D objectのdense picking方式。gizmo CPU hit-testとは別問題とする。
- P6でParley全体をprivate layout leafとして採るか、現行Fontique／HarfRustの薄いrun経路を保つか。
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
| Khronos Validator | `434283be08a668a8fb4e437145630ddbf93b0686` | schema、accessor、animation、extension検査 | Host resource policy、renderer capability |
| Khronos Sample Assets | `2bac6f8c57bf471df0d2a1e8a8ec023c7801dddf` | core／testing assetと個別license | Motoliiのgolden許容差 |
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
- [Rerun architecture](https://github.com/rerun-io/rerun/blob/main/ARCHITECTURE.md)
- [Bevy examples](https://github.com/bevyengine/bevy/blob/main/examples/README.md)
- [renderling](https://github.com/schell/renderling)
- [Vello](https://github.com/linebender/vello)
- [Parley](https://github.com/linebender/parley)
- [HarfRust](https://github.com/harfbuzz/harfrust)

## 6. 機構class別の暫定裁定

| 親 | route | 接続target | Motoliiに残す薄い責任 | 拒否／保留 |
|---|---|---|---|---|
| A. glTF／OBJ import | `ADOPT`候補: `gltf`／`tobj` private leaf。`EXTERNAL`: Khronos Validatorをfixture／CI oracle。Rerunは`PATTERN` | `Asset` metadata→Host importer→private faithful asset | admission、URI／size policy、axis／unit normalization、typed diagnostic、content identity | parser成功を製品成功にしない。Assimp／USD sceneを初期導入しない |
| B. scene／object representation | `REUSE`: 単一world、既存transform、AssetRef、LayerSource意味。Rerun／Bevy sceneは`PATTERN`のみ | faithful asset＋Document projection→private evaluated object list | stable semantic identity、world transform、source intent、derived bounds | engine scene graph／ECS entity／serdeを第二のDocumentにしない |
| C. spatial renderer | `REUSE`: wgpu／現行RenderSession。Rerun／Bevyは`PATTERN`。renderlingは隔離`SPIKE`候補 | private compiled asset→既存LayerSource／Render Contribution | single world、Observation consumption、premul linear output、Quality、typed refusal | Bevy ECS、Rerun store、renderling slab／rust-gpuを無裁定輸入しない。rend3は`REJECT` |
| D. camera／Observation | `REUSE`: Planar baselineとCamera Provider決定。engine cameraは`PATTERN`のみ | Host評価→representation非依存typed Observation | active binding、provider pin、capability preflight、bounds／picking参加 | P3前のpublic API、具体provider ID分岐を禁止 |
| E. depth／Render Contribution | `REUSE`: M5 depth decisions、現行composite。Rerun／Bevyは`PATTERN` | Host policy→admitted contributions→shared attachments | whole-request admission、alpha class、ordering、resource owner | historical private seamの公開昇格、soft alphaのopaque化を禁止 |
| F. text | `REUSE/WRAP`: 現行Fontique＋HarfRust＋Vello。Parleyはprivate比較候補 | `motolii-text` run API→Vello `draw_glyphs` | 同梱CJK下限、fallback診断、cluster／variation、純関数run | layout／editing API全体を公開しない。Vello alphaを品質保証に読み替えない |
| G. Vello局所pass | `REUSE`: 現行Vello dependencyとrenderer setup | text／shapeの局所2D scene→既存GPU合流 | adapter、premul、resource reuse、capability refusal | blur／postの万能backendにしない。frame内resource生成を増やさない |
| H. post | `REUSE`: wgpu／pipeline cache。Bevy等はalgorithm／fixtureの`PATTERN` | existing filter/render graph→linear GPU pass | RoI padding、Quality ladder、same preview/export、golden | Vello blurは未完成なので採用しない。scene engine post stackを持ち込まない |
| I. picking／gizmo／bounds | `REUSE`: native Stage overlayとCPU gizmo hit-test。Rerunはdense picking／outlineの`PATTERN` | read-only projection→Transient selection→既存D2 commit | semantic ID mapping、async generation、Unknown bounds、no canonical pixels | gizmoへGPU ID readbackを使わない。dense pickingを同期readbackにしない |
| J. Duplicator | `REUSE`: P0I/P7決定、PCG32、GPU instance。Cavalry／USD／Niagaraは`PATTERN` | stable slot key→InstanceId→typed channels→GPU instance | domain別identity、nested context、seed、Behaviour純関数 | ECS entity／array indexをidentityにしない。1 instance=1 layerを禁止 |

## 7. 実装可能な子地図

ここでいう子は将来の一契約境界であり、この文書だけではdispatchしない。

| 子 | 結果 | 既知route | exact接続target | 正例／負例oracle | 依存／cutover |
|---|---|---|---|---|---|
| M5-A0 | 2系列のdecisionをmain正本へ処分 | `REUSE` `416aa2c2`／`33e957df` | M5 spec、decision-index、ledger | 現行決定と矛盾0、失効IDをDOへ戻さない | 最初。docs-only |
| M5-A1 | GLB全体preflightとdiagnostic | `ADOPT` `gltf`、`EXTERNAL` Validator | Host importer→private faithful asset | positive／malformed／required ext／oversize／escape URI | A0後。Document／renderer変更なし |
| M5-A2 | OBJを同じprivate assetへlower | `ADOPT` `tobj` | A1 faithful asset | triangle／normal／UV／MTL欠落を明示。無言PBR化なし | A1後。OBJは変換入口のみ |
| M5-R0 | core PBR／unlit headless比較 | wgpu `REUSE`、Rerun／Bevy／renderling比較 | private compiled asset→offscreen target | Khronos metal／dielectric／normal／emissive、cold／warm、low-spec refusal | A0/A1。route採択前spike |
| M5-R1 | Layer Orderへ3D contribution接続 | R0採択route | existing LayerSource／RenderSession | 3D未使用pixel不変、premul、Preview／Export一致 | R0＋M4 resource owner |
| M5-R2 | Group Depth opaque／cutout | Render Contribution決定を`REUSE` | Host depth policy→same material system | Z交差、cutout、soft alpha typed refusal、group外不変 | P3 Observation＋resource gates |
| M5-C0 | PlanarとSpatialのObservation decision | Camera Provider決定を`REUSE` | Host camera eval→typed Observation | provider 2種、capability拒否、Planar pixel不変 | A0後、public API前 |
| M5-T0 | P6 run APIとParley採否を比較 | Fontique／HarfRust／Vello `REUSE`、Parley比較 | new private `motolii-text` leaf→Vello | CJK、fallback、cluster、variation、missing glyph diagnostic | 公開契約は比較後 |
| M5-P0 | Blur/LGG/grain algorithm survey＋fixture | wgpu `REUSE`、既知shader `PATTERN` | filter graph／pipeline cache | RoI radius、Unknown全域、linear、Draft/Final | M4 K0/K1。Vello非依存 |
| M5-I0 | dense object picking比較 | CPU BVH/rayとRerun async GPUを比較 | Stage projection→Transient selection | moving camera、stale generation、readback stall 0、same semantic ID | C0＋bounds contract |
| M5-D0 | stable instance evaluator | P0I/P7 decisionを`REUSE` | input shape→slot key→InstanceId→channels | count増減／reorder／nested／thread順でidentity不変 | schema前にtest-only meaning fixture |

## 8. 推奨順序

1. **Decision recovery**: `M5-A0`で`416aa2c2`と`33e957df`を現行mainへ処分する。
2. **並列調査**: `M5-A1`のparser capability matrix、`M5-R0`のrenderer headless matrix、
   `M5-T0`のtext run comparison、`M5-P0`のpost algorithm surveyを独立に閉じる。
3. **意味decision**: A1/R0の証拠からP1/P2境界、C0 Observation、scene-color format／resource gateを閉じる。
4. **薄い接続**: Layer Orderを先に通常製品routeへ接続し、Group Depth、picking、Duplicatorを後続にする。
5. **cutover**: test-only adapterや旧局所copyを、同一oracleが成立した子ごとに`FROZEN → RETIRE`する。

3D import、renderer、camera、depth、Document schema、UIを一つの発注へ束ねない。

## 9. STOPと再調査条件

- engine／scene／ECS型をDocument、公開API、plugin契約、serdeへ出さないと成立しない。
- parser、renderer、text stackのdefaultからMotoliiの未決意味を逆算する。
- `416aa2c2`／`33e957df`をmain統合済みまたは製品実装済みと報告する。
- Rerun inventoryの候補分類だけで依存、vendoring、portを決める。
- Vello局所成功をpost／text layout／低スペック完成へ外挿する。
- renderlingの機能一覧だけでrust-gpu、GPU allocator、scene ownerを採用する。
- soft alpha、unsupported extension、欠落texture、budget不足を別表現へ無言縮退する。
- test goldenやthresholdを既知実装の出力へ合わせて変更する。

再調査は、固定fixtureでoracle不成立、license／maintenance／3 OS不適合、低スペックhard floor超過、
security上の入力拒否不足、または既知routeが公開境界を侵食する反証が出た場合だけ行う。
