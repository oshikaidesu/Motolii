# M5 Render Contribution証拠Wave親task

作成日: 2026-07-29

状態: **決定**（2026-07-29共通基盤改訂。旧`P2D-RCA/B/C`直発注は停止）

親task ID: `P2D-RC0`

対象: M5 `P2D`の実装前契約調査。既存の3D契約を置換せず、固定機能enumを増やす代わりに、
未知の空間表現が型付き要求とrender contributionを追加できる最小境界を、三つの独立証拠から検討する。

## 1. この親taskが決めること

この親taskが決めるのは証拠の分割、authority、変更path、停止線だけである。Render Contributionの
最終意味、公開trait、永続schema、Vism/package形式は決めない。

- 共通のHost境界、用語、引用anchor、非目標、成立済みコード事実は主担当Codexが一度だけ固定する。
- 外部証拠の取得／転記と、Motoliiへの比較／裁定を同じ発注へ束ねない。
- 旧`P2D-RCA/B/C`の直接発注は、RCAがGrok `REJECT`、RCB/RCCがSpark context枯渇となったため停止する。
- 後続grainのID、capsule path、allowlistを別変更で登録するまで再発注しない。
- 証拠統合、採否、最終契約決定は後続`P2D-RCI`へ残す。`P2D-RCI`は本Waveではdispatchしない。

## 2. Wave共通基盤

この節は後続発注のnavigationであり、Render Contributionの正本や公開契約ではない。記述が引用先と
衝突する場合は引用先が勝ち、leafはローカル解釈で解消せず`ORDER: STOP`する。`P2D-RCI`はこの節を
意味根拠として引用せず、下記の元authorityを直接引用する。この節は`P2D-RCI`処分時に失効する。

優先順位は次のとおり。後続grainは下位資料から上位の意味を逆算しない。

1. [M5仕様](../specs/M5-3d-and-post.md)の「方針」「Camera Provider／Observationと空間rendererの分界」、
   task `P2/P3/P2D`、実装ガード13/15。
2. [換装可能な意味の席／Provider決定](2026-07-24-replaceable-semantic-seat-decision.md)の
   「2.3 Hostが所有するもの」「2.4 Providerが所有するもの」「2.5 Provider換装」「6. 停止線」。
3. [Controlled Microkernel決定](2026-07-25-controlled-microkernel-host-module-parallelism-decision.md)の
   「3. Coreに残す最小責任」「5. 権限と多重度」「6. pluginという語と信頼境界の分離」。
4. 現行コード事実。
5. Rerunまたはゲームエンジンの先例。

引用は見出し名、実装ガード番号、task IDを使う。番号付き節を持たない文書へ`§N`を書かず、
行番号はコード事実だけに使う。後続leafのauthorityは本書§2〜§3と、そのleafへ事前固定した証拠capsuleだけに
狭める。ここに無い資料や語が必要なら、leafが探索せず主担当Codexへ戻す。

既存M5契約のうち、次は再審議しない固定事実である。

- 全objectは同じ正準XYZ world、同じactive camera／Observation、同じworld transformを使う。
- `Layer Order / Group Depth / AE-style Bins`は初期組み込みpolicyであり、既存意味を後から上書きしない。
- Hostはworld、active camera seat、Observation配布、layer/group順、visibility resolve、
  bounds/picking参加、Quality、FrameDesc、failure/admissionを所有する。
- `Layer Order`のPlanar／RGBA互換経路と、3D不使用compositionのpixel不変を維持する。
- 未知または非対応の能力を黙示fallbackせず、Preview／Exportは同じ評価経路を使う。

Wave内の固定語彙は次だけである。Rust名、trait、schema、wire、enumを意味しない。

- **型付き要求**: 表現がadmissionより前に「何を必要とするか」を宣言する側。具体的な型形状は未決。
- **render contribution**: admission後にHostが収集し、phase resolve、resource budget、failureへ従わせる
  「描画へ何を提供するか」の側。万能callbackや公開traitを意味しない。
- **admission**: Hostが要求を受理または型付き拒否する境界。黙示fallbackを含まない。
- **phase resolve**: 受理済みcontributionの順序、参加、資源、失敗をHostが解決する責任。phase enumは未決。
- **evidence capsule**: 判断を持たない固定出典の抜粋と非証明範囲。製品へimportしない
  `FROZEN / DELETE-LATER`証拠カプセルであり、採用裁定を次粒へ継承しない。
- **First Vism**: first-party専用口を持たず同じ境界を通る最初のconformance fixture上の役割だけ。

未決のままleafが埋めてはならないものは、Rust名／trait／signature／serde／wire／phase enum、
registry多重度、capability交渉形式、進化規則の実装形、First Vismの具体表現、FP16中間形式、
provider identity/versionの永続形である。opaque、cutout、soft alpha、scene-color/refractionは
比較軸とfixture候補であり、閉じた機能enumではない。

<!-- P2D-RC COMMON NON-GOALS BEGIN -->
- 公開API、Document schema、plugin契約、wire形式、Vism/package/schema、実装コード、fixtureコードを変更しない。
- `RenderContribution`等のRust名、trait signature、serde形、registry多重度、phase enumを決定しない。
- P2Dの初期3 policy、P3 Observation、Host authorityを別設計へ置換しない。
- Rerun／ゲームエンジンの型、状態所有、render graph、package名、material／phase enumをMotolii authorityにしない。
- Host enum、具体provider ID、raw JSON／文字列走査、opaque ID／private type走査、公開raw mutation、
  invented serde default、重複planner/helper、lint抑制、期待値／golden変更で境界を迂回しない。
- leaf単独で採用決定、実装解禁、P2D完了を宣言しない。
<!-- P2D-RC COMMON NON-GOALS END -->

後続leafはこのblockを空白差以外そのまま転記する。固定語彙外の新しい規範語、anchorの無い規範文、
共通基盤と元authorityの矛盾を検出した場合は`ORDER: STOP`とする。

## 3. 現行コード事実

次は主担当Codexがcommit `4a5669febe47b959723d5eaa7ccf288f83b9f87c`で確認した成立事実である。
後続leafは同じ三crateを再読解せず、この表とdispatch時のhash照合を使う。hashが変わった場合だけ
主担当Codexが再確認し、leafはRerunとの差だけをgapと呼ばない。

| path / API | 成立している事実 | まだ成立していないこと |
|---|---|---|
| `crates/motolii-plugin/src/lib.rs` `LayerSourceContext` / `LayerSourcePlugin::render` | 具体`CompCamera`と0-input sourceを受け、`TextureRef`のRGBA outputへ描く | provider非依存Observation要求、複数draw phase、depth/transparent/refraction contribution |
| `crates/motolii-plugin/src/lib.rs` `PluginKind` / `PluginRegistry` | 固定kindごとの登録と`NodeDesc`検証がある | 未知の空間表現が要求能力とcontributionを追加するseat |
| `crates/motolii-doc/src/graph.rs` `build_source` | prepared `LayerSourcePlugin`を`RenderStep::Plugin`の0-input RGBA nodeへloweringする | shared depth passへのtyped admissionやphase contribution |
| `crates/motolii-render/src/lib.rs` `dispatch_plugin` | Filter／Composite／LayerSourceを固定分岐し、LayerSourceへcameraとRGBA outputを渡す | capability交渉、opaque/cutout/soft-alpha/refractionのphase resolve |
| `crates/motolii-render/src/lib.rs` `RenderSession` / `LinearRenderGraph` | texture node列と既存compositeを実行する | P2Dの共有depth object参加境界 |

code fact hash:

- `crates/motolii-plugin/src/lib.rs`: `5129a4983c5edbf0f29cbae4596f71a6e6593996c36f114d4598a59feded2ca7`
- `crates/motolii-doc/src/graph.rs`: `436aa51781669b90a51b9a9a27ef87604bff84c73924501e71e4bbbc7a915bcf`
- `crates/motolii-render/src/lib.rs`: `0068a75720f6642db13ab9457ec615a5acee112085dedb4d5982a5aa2eacb0f0`

この表は公開API変更の許可ではない。

## 4. 旧grainの処分と責任分離

| 旧grain | 実行結果 | 処分 |
|---|---|---|
| `P2D-RCA` | Spark差分後、Grok `REJECT`（authority誤引用、概念境界／非目標不足） | 差分を採用せず停止。登録templateも後続authorityにしない |
| `P2D-RCB` | 共通authority／inventory読解中にSpark context枯渇、差分なし | 未裁定のまま停止。直接再発注しない |
| `P2D-RCC` | 複数engine一次資料取得中にSpark context枯渇、差分なし | 未比較のまま停止。直接再発注しない |

三つの旧template内に残る`状態: 比較中`と`変更許可`は本節により失効し、dispatch権限やauthorityを
与えない。template本文を使う必要がある場合も、別ID／allowlist登録時に新しい出力へ移し替える。

| 主担当Codexが所有すること | 後続leafへ許すこと |
|---|---|
| 本書§2のanchor／固定語彙、§3 code fact再確認、比較軸、証拠取得、Rerun転移裁定、採否、統合 | 事前固定されたcapsule一つの転記、または固定済みcapsuleだけを読む単一比較質問 |
| Rerun固定commit取得、license／dependency closure、`DEPEND / VENDOR / PORT / PATTERN / REJECT`裁定 | 未裁定Rerun assetの取得／分類、network、repo横断探索を行わない |
| engine一次資料をprovider family別capsuleへ固定 | 一つのprovider capsuleから指定観測項目だけを記録する |

一つのleafは`取得／転記／比較／裁定`のうち一動詞だけを持つ。判断leafはnetworkへ出ず、
repo archaeologyをせず、本書§2〜§3と事前固定capsuleだけを読む。capsuleは一grain固有path、約200行以下、
source commit/URL、versionまたは取得日、license、証明／非証明範囲、`FROZEN / DELETE-LATER`、
製品非import、削除条件をheaderへ持つ。page dump、Motolii要件、採用／推奨／裁定を書かない。

後続grainはID、capsule path、単一動詞、入力anchor、allowlist、削除条件を別変更で登録してからdispatchする。
旧`P2D-RCA/B/C`を再利用せず、`P2D-RCI`の依存も新grain登録時に更新する。

## 5. 後続のMotolii比較境界

主担当Codexが§2の固定語彙と比較軸を所有する。後続の比較leafは§2／§3だけを入力にし、
authority mapping、概念定義、比較を同時に行わない。旧RCA差分を出発仕様として引用しない。

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

後続登録時の`ORDER: STOP`:

- Vism/package/schema、公開trait、Document field、wire形式、永続IDを決めないとdraft不能に見える。
- P2/P3/P2D、Host authority、既存LayerSource互換の意味変更が必要になる。
- First Vismの具体表現、販売／配布、UI、package内容を決め始める。
- 現行コードに無い能力を実装済み事実として扱う。

## 6. Rerun強制動線

将来orderは[学習・転移計画 §9](2026-07-20-rerun-learning-transfer-plan.md#9-rerun参照を発注へ入れる強制動線)
とAGENTS.mdに従い、次のラベルを**この順序で**持つ。通常必須項目も省略しない。

1. `MOTOLII AUTHORITY`: M5 P2/P3/P2D、本書§2、semantic seat決定、§3の完成／非完成事実。
2. `CODE FACT GAP`: `LayerSourcePlugin::render`、`build_source`、`dispatch_plugin`のcall pathと、
   shared depth/phase contributionが未成立である再現可能なコード証跡。
3. `RERUN EVIDENCE`: 固定commit `954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`と下記対象file/API。
   package inventory済み範囲と、全関数・LFS snapshot・性能・Motolii適合を証明しない範囲を分ける。
4. `TRANSFER CLASS`: 主担当Codexが証拠capsule取得後にfile/API asset単位で裁定済みの
   `DEPEND / VENDOR / PORT / PATTERN / REJECT`を記す。実装担当へ分類を委任しない。
5. `TRANSFER LIMIT`: 上表の単一docs fileだけ。Rerun型、Entity、ViewClass、Blueprint、store、
   draw-phase enum、serde、shader、dependency、license未監査assetを製品へ持ち込まない。
6. `MOTOLII ORACLE`: M5 fixture案で、2D pixel不変、同一world/camera、opaque/cutout/soft alpha、
   unknown capability拒否、Preview/Export同一を判定する。Rerun類似を合格条件にしない。

主担当Codexが一度だけ取得し、個別capsuleへ分ける固定監査対象:

- `examples/rust/custom_visualizer/src/main.rs`のbuilt-in `Spatial3DView`拡張登録。
- `examples/rust/custom_visualizer/src/height_field_visualizer.rs`の`VisualizerSystem`実装とdraw data生成。
- `examples/rust/custom_visualizer/src/height_field_renderer.rs`の`Renderer`実装、draw data、phase参加。
- `examples/rust/custom_view/src/main.rs`の`App::add_view_class`登録。
- `crates/viewer/re_renderer/src/draw_phases/draw_phase_manager.rs`のphase登録・実行責任。
- `crates/viewer/re_renderer/src/draw_phases/mod.rs`のphase語彙。

必要なfile/APIがこの集合外へ広がる場合は、主担当Codexが追加理由、license、監査範囲を示すだけに留め、
そのassetは未裁定STOPとする。inventoryの候補分類を採用裁定へ転載しない。未裁定assetを含むorderは
`TRANSFER CLASS`を満たせないためdispatchしない。

`ORDER: STOP`:

- Motoliiの目的より先にRerunのView／Visualizer／Renderer構造を要件化する。
- package名だけで範囲を決める、未裁定assetを持ち込む、公開API／Document／plugin契約変更を要する。
- Rerun内部構造が無いとMotolii契約を作れないとして、旧`P2D-RCA`相当のMotolii比較draftまたは
  統合decisionを代行する。
- Rerunのsnapshot、見た目、phase名へ合わせるためMotolii fixture期待値を変更する。

## 7. ゲームエンジン一次資料の翻訳境界

Wave全体で少なくとも三つの独立した公式一次資料群を使い、各主張へ固定versionまたは取得日とURLを付ける。
候補はUnity SRP、Unreal Engine Mesh Pass／Translucency、Godot screen-reading shader／back-buffer、
Bevy render phaseの公式manual・API docs・公式sourceである。community記事や二次解説を根拠にしない。

主担当Codexがprovider familyごとに一つのcapsuleを取得する。公式page dumpや長い引用を保存せず、
下記観測項目に対応する短い抜粋、URL、version／取得日、license／利用条件、非証明範囲だけを残す。
三family以上の比較とMotolii fixture翻訳は、capsule取得後のnetwork禁止leafへ分ける。

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

`P2D-RCI`は新しい後続grainが登録・完了し、各capsule／比較文書の事実、推論、裁定が分離された後だけ
別grainとして登録する。旧`P2D-RCA/B/C`の完了を依存条件にしない。
統合時も、証拠多数決で公開APIを決めない。Motolii authorityを正として反例を突き合わせ、
必要ならcontract decision、private spike、schema decision、First Vism fixtureを別ticketへ再分割する。

次のどれかが残る場合、`P2D-RCI`は`ORDER: STOP`とする。

- Host authority、要求とcontribution、alpha/refraction、failure、進化規則の意味が未統一。
- Rerunまたはゲームエンジンの内部責任を採らないと成立しない。
- 公開trait、Document schema、plugin契約、Vism/package形式を一つのdecisionで同時に決める必要がある。
- First Vismのconformance役割と製品／配布意味が分離できていない。

## 9. Opus 5／Fable 5助言の処分

2026-07-29、依頼元Codexセッション`019faae0-2508-7812-88cf-d6ad25973d38`から
`claude-opus-5`と`claude-fable-5`をread-onlyで呼び、旧三loopの生証跡と本改訂diffを監査した。
編集、再委任、Spark起動、仕様決定は許していない。

- Opus 5の「共通負債は複数動詞の同梱だが、RCAの意味／引用失敗とRCB/RCCの取得context枯渇は
  別failure」という助言を採用した。
- Fable 5の「親taskだけでなくM5、ledger、decision index、reviews indexを同時更新する」
  「固定語彙、元見出しanchor、共通非目標、失効条件を置く」という助言を採用した。
- 未裁定Rerun取得をSpark grainへする案は、AGENTS.mdの裁定済み`TRANSFER CLASS`必須規律と衝突するため
  採用せず、主担当Codexが証拠取得後に裁定する§6の形へ縮小した。
- Fable 5の具体diff再審査`REVISE`で検出したP1（見出しanchor 5件の誤記）とP2（旧`A`参照、
  `本節`の曖昧さ、旧template内許可の失効不足）を訂正した。

外部modelの出力はauthorityにせず、上記は主担当Codexが現行docs、code fact hash、生runner証跡へ
再照合して採否した処分記録である。
