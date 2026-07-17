# VSM-A3R — 外部表現・Expression・Add-onの責任分類

日付: 2026-07-18

状態: **調査完了／A3候補の推薦まで。設計採用・公開API変更・実装許可ではない**。本書は[レビュー文書の規律](README.md)に従い、外部製品の機能をMotoliiへそのまま移植せず、事実と転移仮説を分ける。採用は独立したVSM-A3Dの処分を要する。

対象: After Effects Expression／Script／Effect SDK、aescripts製品、Blender Driver／Geometry Nodes／Simulation／Add-on。製品の説明と公開manualだけを読み、コード、asset、内部形式は参照・流用していない。

関連正本: [Vism実装計画](2026-07-17-vism-implementation-plan.md)、[UI操作言語 §5.4](../ui-interaction-language.md#54-parameter-panelを表現のホームにする)、[操作単純化モデル S-4](../interaction-simplicity-model.md#s-4-expressionとpluginの位置)、[プラグイン作者向け規約](../plugin-authoring.md)、[Simulationモデル](../simulation-model.md)。

## 1. 調査の問い

VSM-A3は`Clear`より実用的なLayerSourceまたはCompositeを、first-party特権なしで実証する段階である。表現の人気順ではなく、次を問う。

1. 外部製品で「plugin」と呼ばれるものは、実際には何を所有しているか。
2. Expressionの実験性を残しつつ、意味と操作場所を分散させない最小形は何か。
3. Host command、任意script、状態simulationを先取りせず、公開境界を実証できる表現は何か。
4. Parameter Panelだけから発見・調整・変調・診断・修復できるか。

## 2. 確認できた事実

### 2.1 After EffectsはExpression、Script、Effectを同じ責任にしていない

AdobeはExpressionをJavaScriptベースのproperty評価、Scriptをapplicationへ操作を命じるものとして区別する。Expression referenceには`time`、`value`、property／layer参照、補間、path等があり、文字列式がcomposition内の広いobject graphを参照できる。[Expressionの説明](https://helpx.adobe.com/after-effects/using/expression-language.html)、[Expression Language Reference](https://helpx.adobe.com/after-effects/using/expression-language-reference.html)。

AE C++ SDKではEffect pluginは映像・音声と時間変化するparameterを扱う。一方、AEGPはprojectの広い読み書き、menu、panel、internal command、script実行まで扱い、AEIOはmedia形式、Artisanは3D renderを扱う。同じ「plug-in」でも権限と責任が異なる。[After Effects C++ SDK Guide](https://ae-plugins.docsforadobe.dev/intro/what-can-i-do/)。

SmartFXは必要出力を先に受け、pluginが必要入力を宣言してからrenderする。これは表現ではなくHostとexecutorの評価・最適化契約である。[SmartFX](https://ae-plugins.docsforadobe.dev/smartfx/smartfx/)。

### 2.2 aescripts市場では複数責任が一つの棚へ並ぶ

| 製品 | 公開説明で確認できる中心能力 | 実際の責任 |
|---|---|---|
| [Ease and Wizz](https://aescripts.com/ease-and-wizz/) | keyframe間へ補間式を適用 | Expression生成／補間 |
| [Joysticks 'n Sliders](https://aescripts.com/joysticks-n-sliders/) | poseをjoystick／sliderで補間 | property関係／rig／controller生成 |
| [Ouroboros 2](https://aescripts.com/ouroboros-2/) | 一つのpathへ複数stroke、trim、delay、seed等を適用 | 手続き的な反復表現 |
| [Rapid Slideshow](https://aescripts.com/rapid-slideshow-and-presentation-maker/) | image、grid、camera、null、matte、expressionを独自UIから生成 | Project mutation／rig生成 |
| [Trapcode Particular](https://www.maxon.net/en/product-detail/red-giant/particles-and-3d/trapcode-particular) | particle、emitter、physics、preset、visual designer | 生成器／simulation／専用UI |

市場の分類が粗いことは確認できるが、「売れているから一traitへ統合すべき」「独自panelが成功要因」とは結論しない。

### 2.3 Blenderは値、生成、状態、Host拡張を分ける

- Driverはproperty値を別property、組込み関数、数式で駆動する。任意Pythonへ落ちる式は低速化と未知codeのsecurity riskがあるとmanual自身が警告する。[Drivers](https://docs.blender.org/manual/en/latest/animation/drivers/introduction.html)。
- Geometry Nodes Modifierは共有node groupへgeometry input／outputとinstanceごとの公開inputを持たせる。graph編集面と利用者の調整面が分かれる。[Geometry Nodes Modifier](https://docs.blender.org/manual/en/latest/modeling/modifiers/generate/geometry_nodes.html)。
- Simulation Zoneは前frameの結果が次frameへ影響し、cacheとdisk bakeを持つ。非逐次renderにはbakeが必要である。[Simulation Zone](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/simulation/simulation_zone.html)。
- Add-onはBlenderを拡張するsecondary scriptであり、install／enable／disable、独自設定を持つ。[Add-ons](https://docs.blender.org/manual/en/3.1/editors/preferences/addons.html)。
- [Animation Nodes](https://docs.animation-nodes.com/)はmotion graphics向けnode-based visual scriptingである。
- [Geo-Scatter](https://www.geoscatter.com/docs-scattering.html)はUIからscatter systemを作り、sceneへGeometry Nodes modifier付きobjectを追加する。Add-onがHost操作と生成graphを束ねるauthoring shellになり得る例である。

Blender Add-onはA3のexecutor型紙ではない。将来asset、camera、depth／matte、Geometry Nodes結果、bake済みsimulationを交換する明示import／export bridge候補として別に審判する。

## 3. Motoliiへの責任翻訳

| 外部能力 | Motoliiの持ち場 | A3での処分 |
|---|---|---|
| texture加工 | `Filter` | A1で実証済み |
| propertyの時間関数・別値 | `ParamDriver`／型付きLink／Interpolation | A2、B2、UI力学へ |
| parameterから画を生成 | `LayerSource` | **A3本命** |
| N textureのrole付き合成 | `Composite` | 入力role／Document graph実証後 |
| layer、key、camera等を変更 | Authoring Tool／Kit／Host command | atomic batch待ち |
| 前frame状態を使う物理 | `SimulationPlugin + StateTrack`／Bake | A6待ち |
| 外部DCC操作 | 明示import／export bridge | 別spike |

責任分類の狙いは能力を減らすことではなく、保存、評価、Undo、欠落、security、UIの責任寿命を一traitへ潰さないことにある。

## 4. Expressionから採るもの、採らないもの

採るもの:

- parameterを起点に値の決まり方を変更する。
- `time`、補間、loop、noise／wiggle、map、別parameter参照を組み合わせ、即時previewする。
- 固定値やkeyframeを捨てず、試して戻せる。
- 反復需要が安定した表現を型付きToolへ昇格する。

A3では採らないもの:

- layer名、effect名、indexを文字列で走査する参照。
- 任意JavaScript／Pythonを標準project作法にすること。
- Parameter Panelと別の意味正本。
- Scriptによる自由なDocument変更、独自Undo、隠れcontroller。
- project open時の任意code実行。

Expressionの実験性は、Parameter Panel内の固定値／Keyframe／型付きDriver／Linkへ回収する。将来の数式／WASMもparameterの追加value sourceとして隔離する。

## 5. A3候補比較

| 候補 | 現行契約との距離 | UI力学 | 判定 |
|---|---|---|---|
| 決定論的Repeater／Particle Field `LayerSource` | 0-input→texture、`f(t, params)`で成立 | count、spread、size、phase等を集約可能 | **第一候補** |
| Analytic Trail／Multi-stroke `LayerSource` | path inputや時間窓を要求しやすい | 集約可能だが現行input不足 | 縮小候補 |
| role付き`Composite` | N入力role、選択、loweringが未確定 | pickerとrole表示が必要 | 後続候補 |
| 物理particle | state、cache、bakeが必要 | 専用authoring面へ膨張 | A6へ延期 |
| rig生成Add-on | Document mutationとatomic batchが必要 | 独自panelへ分散 | Kit／Authoring Toolへ |
| 汎用Expression runtime | security、dependency、version、determinismを追加 | 第二言語の入口になる | v2まで延期 |

## 6. 第一候補の最小表現

推薦する実証物は、**決定論的な2D Repeater／Particle Field LayerSource**である。物理particle製品の縮小コピーではなく、現在時刻から各instanceを直接求める解析的generatorとする。

候補parameterは`count`、`radius`または`spread`、`size`、`phase`、`speed`、`color`、`seed`。A3Rから`ParamDef`やDocument schemaへ焼かず、A3Dで現行`ValueType`だけに閉じる最小集合へ縮小する。

```text
instance_i(t) = layout(i, count, seed, phase + speed * t)
pixel(t)      = draw(instances(t), color, size, canonical_space)
```

- 同じ`t + params`から同じ画を返す。
- 前frame、再生順、wall clock、内部乱数状態を使わない。
- 正準座標からrender直前に出力sizeへ写像する。
- Draft／Finalは同じrender関数を使う。
- textureはGPUへ直接生成し、CPU frameを製品経路へ出さない。

## 7. 現行コードで見つかったA3S GAP

`LayerSourcePlugin`の公開traitとregistryは存在する。しかし`motolii-doc/src/graph.rs::build_source`は`core.layer_source.clear`だけをplugin render stepへ下ろし、その他を`UnsupportedSourcePlugin`で拒否する。`doc.layer_source.rect`はDocument built-inであり、registry pluginではない。

A3Sで次を仕様化する。

1. prepared recipeのkindをcatalog/runtimeから検査し、ID直書きなしで0-input render stepへ下ろす一般経路。
2. executor欠落、contract-only、kind不一致、未来version時のtyped rejection。
3. raw Documentを変更せずprepared paramsだけを渡すこと。
4. built-in `doc.layer_source.rect`をregistry pluginへ偽装しないこと。
5. LayerSourceが通常Effectと同じParameter Panel modelへ投影できること。

新Vism trait、LayerSource ID allowlist、A3専用graph helper、`clear`分岐コピーで迂回しない。

## 8. Blender Add-onの処分

A3にBlender Add-onは不要である。将来bridgeの実需が出た時だけ、camera、mesh／curve、depth、matte、bake済みsequence等のloss表を持つ独立spikeを起票する。bridgeは明示import／exportで起動し、Project openからinstall、Blender起動、Python実行を行わない。Blender Python型や`.blend`内部構造をVism executor契約へ露出しない。

## 9. 結論と次の単位

A3の第一候補は決定論的Repeater／Particle Field LayerSourceである。Filter、ParamDriverに続く第三の生成kindを実証し、Expressionの実験性をparameter中心へ翻訳でき、state、任意script、Document mutation、multi-input roleを先取りしないためである。

次は**VSM-A3D**で候補を採用／縮小／延期し、parameter意味とA3Sの公開signature改訂範囲を決める。A3D採択前にcrate、trait、Document schema、UI componentを実装しない。

反対側レビューでは、shader demoへの縮小、`count`／`seed`によるDuplicator契約の先焼き、LayerSource contextの不足、Documentへのkind mirror再導入、大面積editorの不当禁止、Composite延期によるGAP隠蔽、proprietary製品からの過剰推論を問う。

