# M5 Render Contribution証拠Wave親task

作成日: 2026-07-29

状態: **決定**

親task ID: `P2D-RC0`

対象: M5 `P2D`の実装前契約調査。既存の3D契約を置換せず、固定機能enumを増やす代わりに、
未知の空間表現が型付き要求とrender contributionを追加できる最小境界を、三つの独立証拠から検討する。

## 1. この親taskが決めること

この親taskが決めるのは証拠の分割、authority、変更path、停止線だけである。Render Contributionの
最終意味、公開trait、永続schema、Vism/package形式は決めない。

- `P2D-RCA`: Motolii authorityと現行コード事実だけから契約draftを作る。
- `P2D-RCB`: Rerun固定commitの対象file/APIを監査し、assetごとに転移分類を裁定する。
- `P2D-RCC`: ゲームエンジン一次資料のrender phase、transparent、refraction方式をMotolii fixtureへ翻訳する。
- 三粒の証拠統合、採否、最終契約決定は後続`P2D-RCI`へ残す。`P2D-RCI`は本Waveではdispatchしない。

A/B/Cは互いを依存にせず、同時に開始できる。各grainの変更許可は事前登録済みの固有review
template一つだけであり、積集合は空である。

## 2. Motolii authority

優先順位は次のとおり。後続grainは下位資料から上位の意味を逆算しない。

1. [M5仕様](../specs/M5-3d-and-post.md)の単一world、P2/P3/P2D、実装ガード13/15。
2. [換装可能な意味の席／Provider決定](2026-07-24-replaceable-semantic-seat-decision.md)のHost authority、
   provider非依存typed output、first-party専用口禁止、万能trait先行禁止。
3. [Controlled Microkernel決定](2026-07-25-controlled-microkernel-host-module-parallelism-decision.md)の
   typed capability、admission、failure、conformance規律。
4. 現行コード事実。
5. Rerunまたはゲームエンジンの先例。

既存M5契約のうち、次は再審議しない。

- 全objectは同じ正準XYZ world、同じactive camera／Observation、同じworld transformを使う。
- `Layer Order / Group Depth / AE-style Bins`は初期組み込みpolicyであり、既存意味を後から上書きしない。
- Hostはworld、active camera seat、Observation配布、layer/group順、visibility resolve、
  bounds/picking参加、Quality、FrameDesc、failure/admissionを所有する。
- `Layer Order`のPlanar／RGBA互換経路と、3D不使用compositionのpixel不変を維持する。
- 未知または非対応の能力を黙示fallbackせず、Preview／Exportは同じ評価経路を使う。

## 3. 現行コード事実

2026-07-29の親task登録時点では、次が成立している。後続A/Bは同じpathとtestを再確認し、
Rerunとの差だけをgapと呼ばない。

| path / API | 成立している事実 | まだ成立していないこと |
|---|---|---|
| `crates/motolii-plugin/src/lib.rs` `LayerSourceContext` / `LayerSourcePlugin::render` | 具体`CompCamera`と0-input sourceを受け、`TextureRef`のRGBA outputへ描く | provider非依存Observation要求、複数draw phase、depth/transparent/refraction contribution |
| `crates/motolii-plugin/src/lib.rs` `PluginKind` / `PluginRegistry` | 固定kindごとの登録と`NodeDesc`検証がある | 未知の空間表現が要求能力とcontributionを追加するseat |
| `crates/motolii-doc/src/graph.rs` `build_source` | prepared `LayerSourcePlugin`を`RenderStep::Plugin`の0-input RGBA nodeへloweringする | shared depth passへのtyped admissionやphase contribution |
| `crates/motolii-render/src/lib.rs` `dispatch_plugin` | Filter／Composite／LayerSourceを固定分岐し、LayerSourceへcameraとRGBA outputを渡す | capability交渉、opaque/cutout/soft-alpha/refractionのphase resolve |
| `crates/motolii-render/src/lib.rs` `RenderSession` / `LinearRenderGraph` | texture node列と既存compositeを実行する | P2Dの共有depth object参加境界 |

この表は公開API変更の許可ではない。Aは最小概念契約のdraftまで、B/Cは証拠までに留める。

## 4. 三つのclosed grain

| GRAIN | 目的 | `ALLOWED_FILE` | 依存 | 完了の出口 |
|---|---|---|---|---|
| `P2D-RCA` | Motolii authority/current-code-fact起点のRender Contribution契約draft | `docs/reviews/2026-07-29-m5-render-contribution-contract-draft.md` | `P2D-RC0` | 要求、contribution、Host admission/resolve、能力進化、failure、First Vism fixtureの意味を比較中draftとして記録 |
| `P2D-RCB` | Rerun固定commit `954bf95a`の対象file/API監査と転移裁定 | `docs/reviews/2026-07-29-m5-rerun-render-contribution-evidence.md` | `P2D-RC0` | asset単位の`DEPEND / VENDOR / PORT / PATTERN / REJECT`と非証明範囲を記録 |
| `P2D-RCC` | ゲームエンジン一次資料からrender phase/transparent/refractionをMotolii fixtureへ翻訳 | `docs/reviews/2026-07-29-m5-render-phase-primary-source-evidence.md` | `P2D-RC0` | engine固有責任を採用せず、Motoliiで必要な正例・負例・観測項目を記録 |

各orderでhashを固定するauthority path:

| GRAIN | 必須authority |
|---|---|
| `P2D-RCA` | `AGENTS.md`、`docs/specs/M5-3d-and-post.md`、本書、`docs/reviews/2026-07-24-replaceable-semantic-seat-decision.md`、`docs/implementation-ledger.md`、`crates/motolii-plugin/src/lib.rs`、`crates/motolii-doc/src/graph.rs`、`crates/motolii-render/src/lib.rs` |
| `P2D-RCB` | Aのauthority一式に加え、`docs/reviews/2026-07-20-rerun-learning-transfer-plan.md`、`docs/reviews/2026-07-20-rerun-source-asset-inventory.md` |
| `P2D-RCC` | `AGENTS.md`、`docs/specs/M5-3d-and-post.md`、本書、`docs/reviews/2026-07-24-replaceable-semantic-seat-decision.md`、`docs/reviews/README.md`、`docs/implementation-ledger.md` |

order作成時は対象branchのclean HEADで各pathをSHA-256化し、上表の記述をhashの代用にしない。

三grain共通の非目標:

- 公開API、Document schema、plugin契約、wire形式、Vism/package/schema、実装コード、fixtureコードを変更しない。
- `RenderContribution`等のRust名、trait signature、serde形、registry多重度、phase enumを決定しない。
- P2Dの初期3 policy、P3 Observation、Host authorityを別設計へ置換しない。
- A/B/C単独で採用決定、実装解禁、P2D完了を宣言しない。

## 5. `P2D-RCA` order境界

Aは既存M5契約と§3のコード事実から、最小概念を比較中draftへ落とす。

必須論点:

1. 空間表現が宣言する**型付き要求**と、Hostが受理後に集める**render contribution**を分離する。
2. contributionはworld/camera/Observation/transform/layer order/Quality/FrameDescを所有せず、
   Hostのadmission、phase resolve、resource budget、failureへ従う。
3. opaque、cutout、soft alpha、scene-color/refraction等を一つの万能draw callbackへ潰さず、
   能力、順序、alpha保証、fallback可否、診断として比較する。
4. 新能力は追加的で、既存contributionの意味を再解釈せず、未知能力を黙示fallbackしない進化規則を置く。
5. First Vismは製品機能やpackage形式を決めるものではなく、first-party専用口なしで同じ境界を通る
   **conformance fixture上の最初の表現**という役割だけを持つ。
6. 第二の未知表現を想定してもHost enum、具体provider ID、raw JSON、private type走査を要求しない負例を書く。

`ORDER: STOP`:

- Vism/package/schema、公開trait、Document field、wire形式、永続IDを決めないとdraft不能に見える。
- P2/P3/P2D、Host authority、既存LayerSource互換の意味変更が必要になる。
- First Vismの具体表現、販売／配布、UI、package内容を決め始める。
- 現行コードに無い能力を実装済み事実として扱う。

## 6. `P2D-RCB` Rerun強制動線

Bの将来orderは[学習・転移計画 §9](2026-07-20-rerun-learning-transfer-plan.md#9-rerun参照を発注へ入れる強制動線)
とAGENTS.mdに従い、次のラベルを**この順序で**持つ。通常必須項目も省略しない。

1. `MOTOLII AUTHORITY`: M5 P2/P3/P2D、本書§2、semantic seat決定、§3の完成／非完成事実。
2. `CODE FACT GAP`: `LayerSourcePlugin::render`、`build_source`、`dispatch_plugin`のcall pathと、
   shared depth/phase contributionが未成立である再現可能なコード証跡。
3. `RERUN EVIDENCE`: 固定commit `954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`と下記対象file/API。
   package inventory済み範囲と、全関数・LFS snapshot・性能・Motolii適合を証明しない範囲を分ける。
4. `TRANSFER CLASS`: file/API asset単位で一つの`DEPEND / VENDOR / PORT / PATTERN / REJECT`を裁定する。
5. `TRANSFER LIMIT`: 上表の単一docs fileだけ。Rerun型、Entity、ViewClass、Blueprint、store、
   draw-phase enum、serde、shader、dependency、license未監査assetを製品へ持ち込まない。
6. `MOTOLII ORACLE`: M5 fixture案で、2D pixel不変、同一world/camera、opaque/cutout/soft alpha、
   unknown capability拒否、Preview/Export同一を判定する。Rerun類似を合格条件にしない。

固定監査対象:

- `examples/rust/custom_visualizer/src/main.rs`のbuilt-in `Spatial3DView`拡張登録。
- `examples/rust/custom_visualizer/src/height_field_visualizer.rs`の`VisualizerSystem`実装とdraw data生成。
- `examples/rust/custom_visualizer/src/height_field_renderer.rs`の`Renderer`実装、draw data、phase参加。
- `examples/rust/custom_view/src/main.rs`の`App::add_view_class`登録。
- `crates/viewer/re_renderer/src/draw_phases/draw_phase_manager.rs`のphase登録・実行責任。
- `crates/viewer/re_renderer/src/draw_phases/mod.rs`のphase語彙。

必要なfile/APIがこの集合外へ広がる場合は、Bの文書内で追加理由、license、監査範囲を示すだけに留め、
そのassetの転移裁定は`REJECT`または未裁定STOPとする。inventoryの候補分類を採用裁定へ転載しない。

`ORDER: STOP`:

- Motoliiの目的より先にRerunのView／Visualizer／Renderer構造を要件化する。
- package名だけで範囲を決める、未裁定assetを持ち込む、公開API／Document／plugin契約変更を要する。
- Rerun内部構造が無いとMotolii契約を作れないとして、Aまたは統合decisionを代行する。
- Rerunのsnapshot、見た目、phase名へ合わせるためMotolii fixture期待値を変更する。

## 7. `P2D-RCC` 一次資料の翻訳境界

Cは少なくとも三つの独立した公式一次資料群を使い、各主張へ固定versionまたは取得日とURLを付ける。
候補はUnity SRP、Unreal Engine Mesh Pass／Translucency、Godot screen-reading shader／back-buffer、
Bevy render phaseの公式manual・API docs・公式sourceである。community記事や二次解説を根拠にしない。

各資料から記録する観測項目:

- phase admission、ordering key、depth test/write、opaque/cutout/soft alphaの分離。
- transparent同士、opaqueとの交差、sorting限界、OIT等を追加する位置。
- refraction／scene-color readが要求するcopy、subpass、resource lifetime、同期、画面外sampleの限界。
- capability不足、未対応material、循環read、同一phase read/writeのfailure。

Motoliiへは方式や内部型でなく、次のfixture候補へ翻訳する。

1. 同じworld/cameraのopaque 2面がZ交差で反転する。
2. cutoutがdepth参加し、soft alphaをopaque depth writeへ黙示格上げしない。
3. soft alphaの順序依存と非対応診断が明示される。
4. scene-color/refraction要求が入力snapshot、範囲、順序、failureを宣言し、隠れcopyや別Export経路を作らない。
5. 未知contribution／capability不足が既存2D compositionを壊さず型付き拒否になる。
6. contribution未使用時の既存pixel不変とPreview／Export同一。

`ORDER: STOP`:

- engineのrender graph、material、scene、camera、queue enumをMotoliiの公開契約へ転載する。
- engineにある機能をMotoliiの新要件にする、またはP2D既決要件を削る。
- 二次資料だけで結論する、versionの違うengine挙動を同じ事実として混ぜる。
- 数値threshold、phase名、copy方式をfixture前に固定する。

## 8. 後続統合

`P2D-RCI`はA/B/Cが個別に完了し、各文書の事実／推論／裁定が分離された後だけ別grainとして登録する。
統合時も、証拠多数決で公開APIを決めない。Motolii authorityを正として反例を突き合わせ、
必要ならcontract decision、private spike、schema decision、First Vism fixtureを別ticketへ再分割する。

次のどれかが残る場合、`P2D-RCI`は`ORDER: STOP`とする。

- Host authority、要求とcontribution、alpha/refraction、failure、進化規則の意味が未統一。
- Rerunまたはゲームエンジンの内部責任を採らないと成立しない。
- 公開trait、Document schema、plugin契約、Vism/package形式を一つのdecisionで同時に決める必要がある。
- First Vismのconformance役割と製品／配布意味が分離できていない。
