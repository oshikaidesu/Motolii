# Camera Object / Provider決定 — 観測実装を換装可能にする

作成日: 2026-07-24

一般分類の正本: [換装可能な意味の席／Provider／Effect分類決定](2026-07-24-replaceable-semantic-seat-decision.md)。本書はそのCamera固有化であり、Cameraだけの例外規則を作らない。

状態: **決定**。既存M2 `CompCameraDoc::PlanarOrthographic`の意味、pixel、migration、単一active camera、単一world、Stage View分離は維持する。一方、将来のSpatial／PerspectiveをHostのcamera enumへ追加し続ける方針と、具体`CompCamera`型を全空間rendererへ恒久的に渡す方針は本決定で撤回する。Fable 5の反対側レビューで検出したP1（旧正本との矛盾、索引未登録、bounds／picking境界欠落）を同じ変更で修正する。

関連正本: [M2 CompCamera決定](2026-07-16-m2-comp-camera-decision.md)、[統一Stage／Camera設計](2026-07-14-unified-stage-camera-design.md)、[M5](../specs/M5-3d-and-post.md)、[Vism package concept](../vism-package-concept.md)

## 1. 問題

タイムライン上でidentity、時間範囲、parameter、keyframeを持つ実体は、Hostの固定実装へ閉じず換装可能であるべきである。Cameraも例外ではない。

現在の`CompCamera`はplanar orthographicの具体実装であり、`world point → NDC → pixel`を評価する。これを将来も空間rendererへ直接渡し続けると、点群、Gaussian splat、volume、radiance field等の新しいscene representationが必要とする観測情報を、Camera enumやcamera schemaへ個別追加する逆依存が生じる。

Cameraが点群へ対応するのではない。Cameraはscene representationに依存しない**観測**を供給し、点群等のrendererがその観測契約を消費する。

恒久原則は次の一文に集約する。

> **Hostはcameraの実装ではなく観測の席を所有する。** Camera実装とscene representationは互いを知らず、representation非依存のversioned typed Observation Contractでだけ出会う。

## 2. 決定

### 2.1 Cameraはタイムライン上の交換可能なObject／Providerになる

- 将来の製品正規形は、stable object identity、時間範囲、通常parameter／keyframeを持つ`Camera Object`とする。
- Camera Objectの具体的な観測評価はversioned `Camera Provider`が行う。first-party cameraも第三者cameraも同じ公開境界とconformanceを使う。
- Compositionは**単一のactive camera binding**だけをHost意味として所有する。v1でcamera cut、複数active view、layer／group固有cameraを同時に導入しない。
- Provider換装は、対象Camera Objectのidentityを保った明示操作とする。parameter変換可能性を事前検証し、成功時1 Undo、失敗時変更ゼロとする。名前一致、index、黙示defaultで別providerへ変換しない。
- Camera Object、active binding、provider parameterの永続schema／公開Rust型／package wire形式は未決である。この決定から実装形を発明せず、M5 P3のdecision／schema改訂で固定する。

この形は[Vism / Kitモデル §9](../vism-kit-model.md#9-現行bpmからbpm-rhythm-vismへどう移るか)のBPM Rhythm providerと同じ文法である。Hostが意味の席と投影を所有し、provider IDを知らないconsumerがtyped出力を読む。Cameraだけの新しいpackage／resolution／failure体系を作らない。

### 2.2 Hostが所有し続けるもの

換装可能性は、空間と作品再現性までpluginへ委譲する意味ではない。Hostは次を所有する。

- 単一の正準XYZ world、world transform、時刻`t`
- Output Frameの`FrameDesc`、aspect、Quality
- active camera binding、object identity、選択、D2単一writer、Undo、journal
- Stage Viewと書き出しcameraの分離
- Preview／Export同一評価、cache／invalidation、resource budget
- depth／visibility／compositeのHost参加境界
- spatial participantが宣言するbounds／pickingのHost参加境界。選択、Fit、枠外表示、snapに使い、GPU同期readbackを禁止して宣言boundsまたは非同期derived cacheだけを使う
- provider解決、version／capability検査、欠落／不一致の型付き診断
- 評価済み観測を各scene rendererへ配るHost-owned route

第三者Camera ProviderがDocumentを直接変更し、独自Undo、独自world、独自clock、独自depth pass、別Export経路を持つことは許可しない。

### 2.3 Camera Providerが所有するもの

Camera Providerは、保存parameterと時刻`t`から観測を決定的に評価する。

- pose／orientationとprojection／intrinsics
- clip／depth convention
- standard projective mapping
- 必要性を実証した場合のscreen sampleからのray query等、型付き観測capability
- provider固有parameterの意味とvalidation
- 任意の専用authoring UI。ただしHostの選択、Undo、active bindingを所有しない

Providerはscene object、point record、mesh、splat、volume、他provider IDを走査しない。入力scene representationに応じて異なる結果を返さない。

## 3. Observation Contract

Camera Providerの出力とrendererの入力の間に、scene representation非依存の`Observation Contract`を置く。これは意味上の固定名であり、公開Rust型／wire名はM5 P3で決めるため、本書から実装名を逆算しない。

最低限の意味:

- 評価時刻とcanonical worldに対する観測pose
- output aspect／FrameDescとの整合
- projection classと明示capability
- clip／depthの規約
- standard projective consumerが使うtyped data
- capability不足を無言近似せず拒否できること

単一の4×4 matrixを全観測の閉集合とみなさない。一方、将来を想像してray、differential、shutter、volume sampling等を最初の恒久型へ全部焼かない。最初は既存Planarと最小Perspective／point fixtureに必要なcapabilityだけを閉じ、追加capabilityはversioned・追加的に増やす。

rendererは必要capabilityを宣言する。Camera Providerが満たさない場合は構造化診断を返し、別cameraやPlanarへ黙ってfallbackしない。

`Camera Depth rank`等のHost UIが必要とするview-space depthを、最初のObservation capabilityに含めるか導出値にするかもM5 P3で判定する。局所UI helperで具体camera型へ戻さない。

## 4. 点群とその先

点群、mesh、Gaussian splat、将来のvolume／radiance fieldはCameraのvariantではなく、worldへ参加する別renderer／providerである。

- Point rendererは位置、属性、point／splat表現を所有し、Cameraへpoint sizeやclassificationを追加しない。
- Gaussian splat rendererはsplat covariance等を所有する。projection differentialが共通観測capabilityとして必要かは、複数consumerのfixtureで証明してから追加する。
- Ray-based rendererが必要になった場合は、Camera Providerのscene非依存ray capabilityを消費する。radiance field固有samplingをCameraへ入れない。
- renderer固有最適化のために具体Camera Provider ID、parameter JSON、内部型を検査しない。

これにより、新しいscene representationはCamera実装の変更を要求せず、新しいCamera Providerも各scene data形式を知る必要がない。両者を公開Observation Contractだけに依存させ、並列実装可能にする。

## 5. 魚眼等の扱い

完成した2D像を作風として歪ませる魚眼、barrel、lens warpはFilter Vismを第一選択とする。これはcamera pose、visibility、scene samplingを変えない。

画面外geometryのvisibilityやsampling密度まで変える本当の非線形撮影が必要になった場合だけ、対応するCamera Provider／観測capabilityを別途判断する。Filterで十分な需要を理由にCamera契約を肥大化せず、逆にpost-filterでは成立しない撮影を黙って同一視しない。

## 6. 既存Planar projectの扱い

M2で実装済みの`CompCameraDoc::PlanarOrthographic`、Document v5 migration、runtime `CompCamera`、CAM-G0 pixel oracleは変更しない。

- 既存Planar projectは引き続き同じpixelを出す。
- built-in Planar cameraはCamera Provider正規形の永続compatibility baselineとする。
- Camera Object schema導入時に既存fieldを黙って再解釈、削除、provider参照へ置換しない。
- migrationが必要なら追加versionの独立decisionで、旧project不変、roundtrip、欠落provider非依存、Preview／Export同一を固定する。
- Camera Provider不在時に別cameraへfallbackしない。built-in Planarは既存projectが外部packageなしで開けるため常在する。

既存runtime `CompCamera`はPlanar compatibility実装であり、将来Observation Contractそのものではない。

## 7. 並列実装・第三者接続

Camera Provider、point renderer、mesh renderer、Gaussian splat rendererを同時に実装しても、互いのprivate型や実装順へ依存してはならない。

M5 P3は少なくとも次のconformance fixtureを先に固定する。

1. built-in PlanarがCAM-G0と既存project pixelを維持する。
2. 二つの独立Camera Providerが同じObservation Contractへ評価できる。
3. meshとpointの独立rendererが具体provider IDを知らず同じ観測を消費する。
4. capability不足が型付き拒否になり、黙示fallbackしない。
5. provider欠落／version不一致でDocument・履歴不変、Finalを成功扱いしない。
6. provider換装が全体preflight後の1 Undoで、失敗時変更ゼロ。
7. Preview／Exportが同じprovider評価とObservation Contractを使う。
8. Camera／rendererのどちらにも生JSON走査、opaque ID分岐、Host private APIがない。
9. renderer crateが具体Camera Providerのprivate型／crateへ依存せず、Camera Providerへpoint／mesh／splat／volume型を渡す口もない。
10. initial Observation Contractを4×4 matrixだけの閉集合にせず、逆に未実証のray／differential／shutter fieldも焼かない。
11. spatial participantの宣言boundsからStage選択／Fit／枠外表示が成立し、同期GPU readbackを行わない。
12. built-in／first-party providerが第三者と同じ公開境界、capability、resource、failure conformanceだけを使う。

## 8. 撤回・維持・未決

### 撤回

- M5でSpatial／Perspectiveを`CompCameraDoc`のHost-owned追加variantとして増やし続ける。
- 具体`CompCamera`型を全将来scene rendererの恒久入力契約とする。
- Camera transform authoring UIがHostにあることを、Camera model実装もHost固定である根拠にする。

### 維持

- 単一canonical world
- 単一active camera
- Output Frameと書き出しcameraの一致
- Stage View pan／zoom／fitはDocument外
- Preview／Export同一評価
- 既存Planar schema、migration、pixel
- depth／visibility／composite、選択、Undo、resource lifecycleはHost所有

### 未決

- Camera Objectとactive bindingのDocument形
- Provider identity／version pin／parameter payloadのwire形
- Observation Contractの公開型と最初のcapability閉集合
- provider換装時のparameter mapping形式
- provider identity／versionとcache key／invalidationの関係
- Camera ProviderをVism provider類として表す際の成果物kind。Vismと同じpackage／resolution／conformance機構を再利用し、別wire体系は作らないが、具体kindは未決
- view-space depth／bounds／pickingを最初のObservation capabilityへ含める範囲
- camera cut／複数camera objectの製品時期
- 専用Camera UIのsandbox／公開runtime

未決をM5 P2、M3 Camera Tool、point rendererの局所helperへ焼いた時点でSTOPし、P3のdecisionへ戻す。

加えて、次のいずれかが必要に見えた時点でSTOPする。

- renderer都合でcamera schema／Observation Contractへpoint size、splat covariance、radiance-field sampling等のrepresentation固有fieldを足す
- Observation Contractをmatrix単独へ縮約する、または未実証の将来fieldをまとめて恒久化する
- providerがscene走査、独自depth pass、独自Export、Document直接変更を要求する
- capability不足を近似、別provider、Planarへの黙示fallbackで成功扱いする
- first-party専用API、具体provider ID分岐、生JSON／opaque ID走査を追加する

M5 P3は、旧[M2 CompCamera決定 §4](2026-07-16-m2-comp-camera-decision.md#4-spatialperspective-is-an-additive-future-variant)が列挙したorientation補間、handedness／local axis、projection／clip、target constraint特異点、Planar切替、semantic goldenの問いを引き継ぐ。Host enum追加だけを撤回し、未解決の意味論まで捨てない。
