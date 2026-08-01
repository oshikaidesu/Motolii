# M5 3Dインポート／レンダリング境界決定

作成日: 2026-08-01

状態: **決定**

対象: M5-P1／P2。P2Dの既決遮蔽意味、P3 Observation、M4 resource budget、Document／公開plugin契約は変更しない。

## 1. 結論

M5 v1は「3Dファイルを読める」だけで終わらず、通常のglTF素材を2Dコンポジット内で再現可能に描くところまでを
P1／P2の一つの利用者成果とする。ただしfull 3D engineを導入せず、責任を次の三層へ分ける。

1. parser／image decodeは成熟したOSS leaf dependencyへ委ねる。
2. importerは素材意図と診断を失わないprivateな派生assetを作り、renderer capability admission後にGPU向け派生物を作る。
3. Motolii Hostは同じprimitive／material systemを`Layer Order`とshared-depth contributionへ接続し、
   world、camera、遮蔽policy、alpha、scene color、resource budget、diagnosticを所有する。

二つのprivate派生段階はDocument、journal、公開plugin API、永続formatではない。名称、Rust型、field、serde形は本decisionで固定しない。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| AUTHORITY | [M5仕様](../specs/M5-3d-and-post.md)の単一XYZ世界、P1／P2／P2D、絶対規律のVRAM常駐・色変換一元化・Preview／Export同一関数 |
| INTERNAL TARGET | 現行`AssetRef`／Asset metadata、`wgpu` renderer、P2Dの同一object representation参加境界。Importer／GpuAssetCache／3D scene型は未実装・未凍結 |
| OWNER | source asset意図とimport診断はHost importer、renderer-compiled派生物とGPU resourceはHost render／将来GpuAssetCache、world／camera／遮蔽／alpha／budgetはHost |
| WRITE ROUTE | P1／P2はread-only評価派生物。Documentへ書くのは既存Asset参照と将来別decisionで認めた作品意味だけ。private sceneを保存しない |
| GAP | 現行はparser、material capability、lighting、private派生段階、3D描画、P1／P2 fixtureが未成立 |
| RESOLUTION ROUTE | OSS leafをDEPEND候補、Rerun／Bevy／KhronosをPATTERN、Motolii fixtureで採否。full engineと二重rendererを拒否 |
| DISPOSITION | **PASS**: private境界とv1意味を本decisionで閉じる。公開API／schema／GPU具体format／budgetは既存gateへ残す |

## 3. v1 material／lighting

### 3.1 対応する最低線

v1のlit経路はglTF core metallic-roughnessを基準とし、次を一まとまりで扱う。

- base color factor／texture、metallic factor、roughness factor／texture。
- normal textureとscale、occlusion textureとstrength、emissive factor／texture。
- `COLOR_0`、`TEXCOORD_0`／`TEXCOORD_1`、sampler wrap／filter。
- `alphaMode=OPAQUE`／`MASK`、alpha cutoff、double-sided。
- texture semanticごとのsRGB／linear分類を固定し、3D renderer内で出力色変換しない。
- `KHR_materials_unlit`、`KHR_texture_transform`、`KHR_materials_emissive_strength`を最初のextension候補とする。
  crate能力、依存、fixtureをP1実装order前に再確認し、未成立を実装済みと呼ばない。

`alphaMode=BLEND`はv1 3D assetでは対応保証しない。primitive／triangle順やOITを偶然の描画順へ委ねず、
P1 capability admissionで型付き拒否する。P2Dの一般soft-alpha意味や将来OITを撤回する決定ではない。

### 3.2 固定neutral environment

core glTFはmaterialを持つが照明環境を完成させない。bareな固定1灯だけでmetallic-roughnessを描く案は、
metal面と背面を黒くしやすく「読めたが違って見える」ため**棄却**する。v1 lit経路はDocument外・Host所有の
**固定neutral environment**を使う。

- diffuse irradianceとspecular environmentを決定論的な固定assetから供給する。
- environment authoring、HDRI選択、user light、light hierarchy、shadowはv1に入れない。
- environment asset、prefilter方法、露出／tone-mapの具体値はP2 evidence fixtureで固定し、色変換authorityを増やさない。
- 同じ固定bytesと同じrender関数をPreview／Exportで使う。Qualityは明示的なsample／resolution差だけを許す。
- 固定environmentが対象backend／budgetで成立しない場合、ambient、unlit、bare一灯へ自動縮退しない。
  3Dレイヤーだけをplaceholder＋型付きdiagnosticにし、2Dコンポジットは継続する。

`KHR_materials_unlit`はlit経路へ通さず、仕様どおりunlitとして評価する。

## 4. importからGPUまでのprivate責任

### 4.1 faithful imported asset

parser出力を直ちにGPU型へ潰さず、次を保持するprivate派生物を一段置く。

- node／primitive／material／texture／animationのsource意図。
- source pathを外へ漏らさない解決済みresourceとasset fingerprintへの入力。
- extensions used／required、欠落resource、unsupported feature、normal／tangent生成可否の構造化diagnostic。
- 単位、軸、handedness、非有限値、index／accessor範囲、resource sizeの検証結果。

unsupportedな`extensionsRequired`は名前つきで全体を拒否する。任意material extensionにcore fallbackが定義される場合だけ、
core意味を保持して可視diagnosticを出せる。白material、欠落texture、別alpha classへの無言fallbackは禁止する。

### 4.2 renderer-compiled派生物

次段で対象adapter、Quality、material／alpha capability、resource budgetをwhole-assetでpreflightし、GPU upload／pipeline入力へ
compileする。部分admissionでassetを別物にせず、拒否時は元のfaithful assetとdiagnosticを保持する。

- mesh／material／shader systemは一つだけ持つ。
- `Layer Order`と`Group Depth`は別rendererではなくHost所有pass configurationである。
- premultiplyはRender Contributionの合流境界で一度だけ行う。
- group policy、camera provider ID、backend pass ID、raw texture handleをasset modelへ保存しない。
- 両private派生物へ`Serialize`／serde、Document／journal field、公開raw APIを追加しない。

## 5. animationのv1境界

従来文言の「頂点アニメーション付きglTF/Alembic的な焼き込み済みシーケンスがv1唯一の経路」は、
v1でmorph／skinningを持たない範囲と矛盾していたため縮小する。

- v1: static meshとnode TRS animation。STEP／LINEAR／CUBICSPLINEを時刻`t`の純関数として評価する。
- v1の外部DCC導線: rigid／node TRSへベイクしてglTF／GLBで持ち込む。
- v1.x再入場: morph target clip、skinning、baked deformation、Alembic相当cache。
- retargetingは非目標を維持する。custom deformation、simulation stateをrender traitへ隠さない。

## 6. OSS処分

| 対象 | 処分 | 境界／retirement |
|---|---|---|
| `wgpu` | **REUSE** | 既存renderer foundation。3D engineのscene／Document ownerにはしない |
| Rust `gltf` crate | **DEPEND第一候補** | private importer seamの内側。version、license、3 OS、required feature、security／maintenanceをP1 order前に固定 |
| `tobj` | **DEPEND候補** | maintenance／license／MTL coverageを再確認後。OBJ専用rendererを作らず同じfaithful assetへlowerする |
| image decode leaf crates | **DEPEND候補** | color semantic、size budget、malformed inputをHostがpreflight。decoder APIを公開契約へ出さない |
| Rerun importer／CpuModel分離 | **PATTERN** | 固定commitの責任分割だけを参照。code PORTは別裁定まで行わない |
| Rerun `re_renderer`／store／Entity／log protocol | **REJECT** | Motoliiのworld、Document、contribution、resource ownerを逆転させるため |
| Bevy PBR／fixtures | **PATTERN** | BRDF、material、negative fixture比較。ECS／AssetServer／schedule／full rendererはDEPENDしない |
| Khronos glTF spec／Sample Assets／Validator／Sample Viewer | **EXTERNAL ORACLE / PATTERN** | format validation、fixture、PBR比較。Validatorをblocking CI laneにするかは別decision |
| `rend3` | **REJECT** | 2025-06-07 archive／read-only。新規dependencyにしない |
| `renderling` | **比較spikeのみ** | alpha段階かつrust-gpu／SPIR-V build chainを伴う。隔離crate／worktreeで測りworkspace lockfile／通常buildへ入れない |

dependencyのretirement triggerはarchive、license不適合、対象機能のsecurity／malformed-input修復停止、wgpu世代追従不能、
3 OS fixture不合格とする。fixtureはcrate APIでなくglTF／OBJ入力とMotolii出力で固定し、差し替え可能性を保つ。

## 7. Rerun転移票

`MOTOLII AUTHORITY`: M5-P1／P2、同一world／camera、Render Contribution、VRAM常駐、単一色変換、Preview／Export同一関数。

`CODE FACT GAP`: 現行にImporter、GpuAssetCache、glTF／OBJ parser、private scene、3D material rendererはない。

`RERUN EVIDENCE`: fixed commit `954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e` の
`crates/viewer/re_renderer/src/importer/gltf.rs`／`obj.rs`とCPU model→GPU準備の責任分割。
同adapterはRerunの限定supportと内部型を示すだけで、Motoliiのmaterial完成条件、Document、contribution API、store意味を証明しない。

`TRANSFER CLASS`: importer adapter／CPU→GPU分離は`PATTERN`。`re_renderer`、store、Entity、log protocolは`REJECT`。

`TRANSFER LIMIT`: Rerun crate／型／scene意味を依存・vendor・移植しない。将来の小PORTは対象file/API、license、Motolii fixtureを
別decisionで固定する。公開API、Document、plugin契約、永続formatをRerunから逆算しない。

`MOTOLII ORACLE`: §8のP1／P2 fixture。Rerunとの外観／構造類似は合格条件にしない。

## 8. fixtureと完了線

### P1 import

- Khronos Validator適合assetとmalformed corpusを分離し、validator結果をMotolii runtime成功の代用にしない。
- core metallic-roughness、unlit、vertex color、UV1 AO、texture transform、emissive strength。
- `extensionsRequired`拒否名、optional extension diagnostic、欠落texture、truncated buffer、範囲外index、sparse accessor、NaN／Inf。
- oversized texture／meshをOOM前に型付きbudget拒否する。
- glTF／GLB外部URIはimport root内の正規化済み相対pathだけを許し、escape／network URIを拒否する。
- OBJは別render pathを持たず、同じmaterial／bounds／diagnostic入力へ到達する。
- embedded camera／lightはv1ではMotolii authorityへbindせず、存在を可視diagnosticにする。

### P2 render

- no-3D compは3D backend／fixed environmentの有無でpixel bit-identical。
- metallic／dielectric／normal／emissive／unlitの代表assetを固定neutral environmentで比較する。
- normal mapの誤sRGB decode、AOのUV1無視、`COLOR_0`欠落、MASK cutoff境界を負例にする。
- `Layer Order`の2D＋3D、`Group Depth`の2D平面＋同じ3D primitive。scene／shader system二重化を拒否する。
- BLEND assetはv1 capability拒否、Group Depthのsoft alphaは既決どおりtyped refusal。
- node TRS animationをSTEP／LINEAR／CUBICSPLINE、複数`t`、thread順、Preview／Exportで一致させる。
- low-spec adapterのcapability不足は対象3D layerだけplaceholder＋diagnostic、他の2D pixelは不変。
- cold／warm pipeline、upload byte、texture／buffer／environment byte、peak live resource、1080p GPU timeを計測する。
- Khronos／reference viewer画像はcolor／material境界の許容差oracleであり、Motolii全体のacceptance authorityにはしない。

## 9. Fable 5反対側レビューの処分

2026-08-01に`claude-fable-5`をClaude Code CLIのread-only正規経路で呼び、Web一次資料を許可して
`FACTS / INFERENCES / OPTIONS / OPPORTUNITIES / ADVICE / RECOMMENDATION / STOP CONDITIONS`を受けた。
Fable出力はauthorityではなく、Codexが現行spec、decision-index、公式Khronos／Adobe／Apple資料へ再照合した。

| 助言 | 処分 |
|---|---|
| bare一灯PBRを止め、固定neutral environmentかhonest unlitへ | **採用**。fixed environmentを選び、bare一灯を棄却 |
| environment不成立時にambient→unlitへ段階fallback | **棄却**。無言縮退を作らず、対象3D layerをtyped refusal |
| faithful assetとrenderer-compiled sceneの二段にする | **採用**。どちらもprivate／derived／非serde |
| core PBR、UV1、vertex color、unlit／texture transform／emissive strength | **採用**。extension実装成立はP1 evidence待ち |
| morph／skin延期と「vertex animation v1」の矛盾 | **採用**。v1をrigid／node TRSへ縮小 |
| `KHR_mesh_quantization`を即時拒否 | **保留**。parser capability／asset実需をP1で測り、思考だけで拒否しない |
| Blender／reference画像比較を全体acceptanceから外す | **縮小採用**。color／material tolerance oracleとして残す |

一次資料: [glTF extension registry](https://github.com/KhronosGroup/glTF/blob/main/extensions/README.md)、
[KHR_lights_punctual](https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_lights_punctual/README.md)、
[Adobe 3D import FAQ](https://helpx.adobe.com/after-effects/desktop/import-files/import-and-add-3d-models/faq-3d-model-import-in-after-effects.html)、
[Adobe default Environment Light](https://helpx.adobe.com/after-effects/desktop/work-with-3d-composition/extract-and-animate-lights-and-cameras/extract-and-animate-cameras-and-lights-from-3d-models.html)、
[Apple Motion 3D object guidance](https://support.apple.com/guide/motion/guidelines-for-working-with-3d-objects-motn99845383/mac)、
[Khronos Sample Assets](https://github.com/KhronosGroup/glTF-Sample-Assets)、
[Khronos Validator](https://github.com/KhronosGroup/glTF-Validator)、
[renderling](https://github.com/schell/renderling)、[rend3](https://github.com/BVE-Reborn/rend3)。

## 10. STOP

- importerでunsupported material／texture／alphaを白、既定値、別classへ無言変換する。
- fixed environment、露出、tone-map、texture color semanticをDocument／公開plugin契約へ焼く。
- faithful assetまたはrenderer-compiled派生物へserde／Document／journal面を追加する。
- `Layer Order`とshared-depthへ別scene、別material意味、別shader systemを作る。
- glTF内camera／lightをP3前にMotolii active camera／lighting authorityへbindする。
- full Bevy／Rerun renderer、rend3、renderlingを通常dependencyへ入れる。
- Rerun参照を§7の転移票なしで実装orderへ持ち込む。
- GPU具体format、copy／subpass、budgetをevidence前に固定する。
- test golden／許容差を実装へ合わせて変更する。
- public Observation／Render Contribution型をP3前に発明する。
