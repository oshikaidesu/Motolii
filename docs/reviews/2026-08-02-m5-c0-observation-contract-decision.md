# M5-C0 Observation Contract 意味決定

状態: **決定（意味閉鎖）／schema・runtimeはWAIT**（2026-08-02）

## 1. 目的と範囲

M5 P3のCamera Object／Provider／Observationを、現行Planar互換とM4 resource責任を壊さずに
接続するための意味だけを閉じる。公開Rust型名、wire／serde形、Document version、provider
registryの実装は本書では追加しない。実装はこの決定を根拠にした別のschema粒とruntime粒へ分ける。

現行`CompCameraDoc::PlanarOrthographic`、runtime`CompCamera`、既存CAM-G0 pixel oracleは
互換baselineとして維持する。`CompCamera`をObservation Contractそのものへ昇格しない。

## 2. Observationの初期意味

### 2.1 初期capability閉集合

初期Observationは、representation非依存の**projective observation**だけを閉じる。意味上の
出力は次を含む。

- canonical XYZ worldに対するpose／orientationと、worldからcameraへ写す規約
- projection class（Planar compatibilityまたはPerspective）と、そのclassに必要なparameter
- output `FrameDesc`／aspectとの整合
- clip／depth convention、handedness／axis、finite／near・farの拒否条件
- mesh／point等のstandard projective consumerが同じworld pointを投影できるtyped data

Observationを単一4×4 matrixだけへ縮約しない。projection class、depth convention、output整合を
別の意味として保持する。一方、ray query、projection differential、shutter／motion sample、
volume／splat固有値は初期閉集合へ入れず、複数consumerのfixtureが必要になった時だけ追加decisionへ戻す。

### 2.2 Host／Provider責任

Hostはsingle active camera binding、canonical world、world transform、時刻、FrameDesc、
Quality、bounds／picking参加、depth／visibility／composite、resource／failure／Undoを所有する。
ProviderはCamera Objectのparameterと時刻から上記projective observationを決定的に評価し、scene
object、mesh、point、他providerのIDを走査しない。

provider欠落、version不一致、必要capability不足、非有限値、clip／aspect不整合はtyped refusalとし、
別providerまたはPlanarへ黙ってfallbackしない。失敗時はDocument、journal、selection、Undoを変更しない。

### 2.3 Identity／交換

active bindingには、対象Camera Objectのstable identityと、選択されたproviderのidentity／version
条件が意味として必要である。既存static `PluginId`、文字列名、index、Timeline順をproviderの永続identityへ
流用しない。providerのpackage／entry／version wire形は既存Vism identity decisionと同じ境界で別途閉じる。

Provider換装は全体preflightでparameter mappingとcapabilityを検証し、成功時に1 Undo、失敗時に変更ゼロとする。
既存Planar projectはprovider packageなしで開け、旧fieldを黙ってprovider参照へ置換しない。

## 3. Oracleと実装分割

schema／runtimeへ進む前に、private fixtureで次を固定する。

1. built-in Planarの既存CAM-G0／project pixel不変。
2. 独立provider 2種が同じprojective observationをmesh／point consumerへ供給し、具体provider IDをconsumerが見ない。
3. capability不足、provider欠落、version不一致、非有限値、aspect／clip不整合がtyped refusalとなり、Document／履歴不変。
4. provider換装は全体preflight後1 Undo、失敗時変更ゼロ。
5. Preview／Exportは同じprovider評価とObservationを使い、Qualityだけを差分にする。
6. 宣言boundsからStage選択／Fit／枠外表示が成立し、同期GPU readbackを行わない。
7. Observationへrepresentation固有field、raw JSON、opaque-ID分岐、Host private APIを追加しない。

実装は次の三粒へ分ける。

- **C0-Schema**: semantic decisionと既存Document migrationへ照合し、exact type／wire／versionを独立に仕様化する。
- **C0-Fixture**: schema確定後、Planar保持・provider 2種・拒否・換装Undoのprivate conformanceを追加する。
- **C0-Runtime**: fixture合格後、Host評価→typed Observation→representation非依存consumerを接続する。

## 4. M4 resourceとの接続

Observationのprovider identity／version、parameter、評価時刻、world／FrameDesc／Qualityは、将来の
M4 K1b完全cache keyへ影響する材料として追跡する。ただしM4 K1a ResourceLedger、全owner accounting、
hard budget admissionが未実装の間は、GPU resource、compiled asset、cache storeをM5製品経路へ接続しない。
Observation決定はM4の未実装を代替せず、M4 K1aのAPIやbackend型を発明しない。

## 5. 非目標と停止線

- `CompCamera`へSpatial／Perspective variantを追加すること。
- provider registry、package／entry／artifact identity、wire schema、Document versionを本書から決めること。
- ray／differential／shutter／scene-aware camera、camera-aware scene formatを初期Observationへ焼くこと。
- M4 K1a前のGPU resource owner、budget、cache、compiled asset、同期readbackを製品化すること。
- provider不在をPlanar fallbackで成功扱いすること。

この決定で`M5-C0`の意味gateは完了するが、schema／fixture／runtimeは未完了である。次は`C0-Schema`
を一契約境界で仕様化し、公開API・Document・永続形式へ影響する判断はその粒で再審査する。
