# 換装可能な意味の席／Provider／Effect分類決定

作成日: 2026-07-24

状態: **決定**

対象: HVR-D04 Unit 8A、Camera、Timeline Object、Vism Provider、Filter／Effect、将来の第三者実装

## 1. 問題

将来のCamera、点群、生成Object、解析provider等をHostの具体実装へ閉じると、新しい表現が増えるたびにCore enum、Document、consumerを同時変更することになる。一方、何でもpluginへ出すと、作品identity、選択、Undo、参照、world、Preview／Export等の共有意味まで第三者実装へ分裂する。

また「タイムライン上に見えるものは換装可能」という直観だけでは足りない。Effect parameterもタイムラインに見えるが、それ自体は独立Objectではない。逆にBPM Rhythmのように独立行を持たなくても、複数Host surfaceが読む型付きprovider seatは存在する。

必要なのは、UI上の見え方でなく、**Hostが守る意味の席**、**換装可能な具体評価**、**既に得られた値／像への処理**を分ける規則である。

## 2. 決定

### 2.1 一般則

**意味の席はHostに固定し、具体評価はProviderとして換装可能にする。既に得られた値または像だけを変え、上流のscene参加・sampling・identityを変えない処理はEffect／Filterとする。**

この規則は、Coreへ具体機能を焼き込むか、すべてをpluginへ追い出すかの二択ではない。

- Hostは小さな型付きsemantic seatとconsumerへの投影を所有する。
- first-party／third-partyの具体実装は同じversioned Provider契約とconformanceを使う。
- Effect／Filterは、明示inputへ作用する通常のVismとして同じ純関数・VRAM・Preview／Export契約に従う。
- presentation、provider、semantic seatを別軸にする。専用UIやnative overlayを持つことはDocument ownerになる理由ではない。

### 2.2 semantic seatの判定

次の条件を、現在コードに存在するかではなく**作品が要求する意味**として判定する。少なくとも1の独立seat identity／bindingと4のprovider非依存typed outputを両方満たすことを必須とし、残る条件でHost所有範囲を具体化する。

1. stable seat identityまたはstable bindingと、独立した寿命／時間範囲を持つ。
2. 選択、参照、親子、順序、active binding等の作品意味へ参加する。
3. 通常parameter／keyframe、Undo／Redo、journal、save／openの対象になる。
4. 複数consumerがprovider固有型を知らず、同じ型付き出力を読む。
5. 欠落やversion不一致が作品再現性へ影響し、構造化診断が必要になる。

Timeline上の独立Object／rowとして見えることは強い徴候だが、単独の必要十分条件ではない。Effect内parameter、derived particle一粒、gizmo、selection outlineを、それだけでDocument Objectへ昇格させない。大量derived instanceは明示Materialize時だけDocument実体になる。

### 2.3 Hostが所有するもの

- semantic seatのstable identity、lifetime、参照、順序、active binding
- Document保存、D2 single writer、Undo／Redo、journal、migration
- provider identity／versionの解決、capability検査、欠落／不一致診断
- provider非依存のtyped output contractとconsumerへの配布
- canonical time／world／FrameDesc／Quality、Preview／Export同一評価
- cache key／invalidation、resource budget、trust／permission
- selection、bounds、picking、Timeline／Inspector等のHost projection

Hostは具体provider IDやparameter JSONで処理を分岐せず、first-party専用raw APIを作らない。

### 2.4 Providerが所有するもの

- 保存parameterの意味、validation、versioned migration capability
- `t`と明示inputからtyped outputを決定的に評価する具体アルゴリズム
- Provider固有の任意authoring UI。ただしHostのselection、Undo、identityを所有しない。現行公開契約にはcustom plugin UIがなく、G0-3／GAP-13で解凍されるまでは自動生成panelだけを使う（[plugin authoring §1.5](../plugin-authoring.md#15-uiは書かないv1)）
- 宣言したcapability、resource need、failure

ProviderはDocumentを直接変更せず、独自world、独自clock、独自Undo、別Export経路、consumerのprivate型走査を持たない。

### 2.5 Provider換装

換装は同じsemantic seatのidentityを保つ明示操作とする。

1. 対象provider、version、capability、parameter mappingを全体preflightする。
2. 成功時だけprovider bindingとparameterを**1 D2 macro＝1 Undo**で置換する。新しいtransaction APIを発明しない。
3. mapping不能、capability不足、provider欠落は変更ゼロの型付き拒否にする。
4. 名前一致、配列index、近いparameter、黙示default、built-in fallbackで成功扱いしない。
5. 換装前後で参照先identity、Timeline上の寿命、selection、Host bindingを維持する。

具体的な永続schema、公開Rust型、wire形式、mapping形式は各seatのdecision taskで固定する。本書から共通万能Provider traitや生JSON payloadを発明しない。

## 3. Effect／Filterとの分界

Effect／Filterを第一選択にするのは、処理が次を満たす場合である。

- 入力が既に評価済みの値、geometry、texture、RGBA像である。
- 処理後も上流Objectのidentity、active binding、world参加、visibility集合を変えない。
- sceneを再走査せず、別Object／Providerのprivate型を知らない。
- 同じ明示input、parameter、`t`、Qualityから出力が決まる。
- 適用順を通常のeffect stack／render graphで表現できる。

Effectの出力がboundsを変え得る場合は、宣言的bounds／input-region契約を追加する。boundsが変わることだけを理由にObject Providerへ昇格させず、逆にHost selection／layoutへ必要なboundsをGPU readbackや見た目推測へ逃がさない。

### 3.1 Camera／魚眼

- active Cameraはsemantic seatを持ち、具体観測評価はCamera Providerとして換装可能にする。
- 完成した2D像を歪ませる魚眼、barrel、lens warpはFilter Vismを第一選択にする。
- 画面外geometryのvisibility、ray、sampling密度まで変える撮影だけCamera Provider capabilityとして別判断する。

同じ「魚眼」という名称でも、post-image stylizationとscene observationを同じ実装種別へ固定しない。

### 3.2 Resize／Content-Aware Scale

- 通常のObject transform、composition output size、import decode size、export resolutionは、それぞれ既存のHost意味／境界へ置く。「resize」という名前だけで一つのCore機能やEffectへ統合しない。
- Photoshop型Content-Aware Scaleのように、評価済み画像の内容を解析して非線形に変形する処理は、現時点では**Filter Vism候補**とする。将来追加されてもCore resize enumを変更せず導入できる。
- 入力画像内の保護領域、seam、saliency等はFilter parameter／内部評価であり、元Objectのidentityやworld参加を変えない。
- 将来、編集可能geometry、layout constraint、picking、他Objectとの関係を一次的に変更する別機能が必要になれば、同名でもAuthoring Tool、Geometry Provider、Host command等へ再分類する。Content-Awareという商品名から恒久APIを逆算しない。

この判断はContent-Aware Scaleの実装採択、品質、アルゴリズム、GPU実現性を証明しない。

## 4. 並列実装と第三者接続

異なるProvider、Effect、consumerは、共有するtyped contractだけへ依存し、互いのprivate crate、opaque ID、実装順を知らない。同一seatへ複数Providerを並列実装し、別teamがconsumerを実装しても、conformance fixtureだけで結合できる形にする。

最低限の一般conformance:

1. first-party二実装が同じ公開Provider境界だけで評価できる。
2. consumerが具体provider ID／private型／parameter JSONを知らない。
3. provider欠落／version不一致／capability不足が型付き拒否になる。
4. 換装が全体preflight後の1 Undo、失敗時変更ゼロになる。
5. Preview／Exportが同じprovider／effect評価を使う。
6. provider UI、native overlay、headless評価が第二のDocument stateを作らない。
7. first-party専用APIなしで第三者実装も同じfixtureへ合格できる。
8. Provider A、Provider B、consumerを同時実装してもprivate依存が生じない。

## 5. 歴史候補5件の裁定（HVR-D04 Unit 8A）

候補packetはHVR-D03に従うrepo外一時成果物として検収し、永続pathを正本化しない。

固定projection hash: `7b590239d8a4900c52d57cba81c24a963874f06ab649fbd13486dd3c9316e27a`

| Blob／根拠行 | 裁定 | 価値分類 | 回収する価値 | 復活させないもの |
|---|---|---|---|---|
| `4d642e871bcea3affa61d65bf416c1e04c91ef66` L85 | **縮小採用** | 現行規範／負例 | Hostがworld／camera／depthの意味の席を所有し、pluginがobject実装を供給する分界 | 具体`CompCamera`実装まで恒久Core所有とする解釈 |
| `811122a1eedf164d3d66d8abdc3fd29cb3cb47ce` L85, L124 | **一部棄却** | 成立理由／負例 | Planar pixel不変、Preview／Export同一、layer固有camera禁止、黙示fallback禁止 | `CompCameraDoc::Spatial`をHost enumへ直接追加する設計 |
| `848dbe1cae2fea0c63aeaf201a524e123ad3d0f8` L48, L129 | **採用** | 現行規範／負例 | UI上の意味分離がDocument／transform／ownerの分裂を生まない規律 | Depth専用field、第二owner、UI都合のObject |
| `09dba1ae92a8771a73ab5849e906123339850050` L62, L67 | **採用** | 現行規範／負例 | 高度操作も通常propertyへのD2 macroとして確定し、隠れObject／channelを作らない規律 | setup script、controller、独自Undo、暗黙Materialize |
| `c0344e67cfbf4f5e83c7e430d696328f50afd645` L43, L68 | **採用** | 現行規範／負例 | presentation overlay、semantic write、canonical outputを分離する規律 | overlayによるcanonical pixel変更、GPU readback、第二state |

5件は別々の一般原則ではなく、同じM5文書lineageの世代差である。よって本書の一規則へ統合し、古いCamera enum案やUI具体値を現行仕様として復活させない。

## 6. 停止線

- 「タイムラインに表示される」だけで独立Document Objectを追加する。
- 換装可能性を理由にHostのidentity、world、Undo、provider解決をpluginへ委譲する。
- first-party providerだけがHost private API、生JSON、opaque ID分岐を使う。
- 具体Providerの追加ごとにCore enum、consumer、永続schemaを同時変更する。
- Effectでscene再走査や別Objectのprivate stateを読み、実質的なProviderを偽装する。
- 本書から万能Provider trait、共通parameter JSON、共通wire payloadを先行実装する。
- Content-Aware Scale候補を理由に通常transform、output resize、decode resizeを同じ契約へ統合する。
- provider換装でidentityを作り直す、参照を名前検索で張り直す、失敗後に部分変更を残す。

## 7. 未決

- semantic seat／Provider種別をVism package kindへどう表すか。
- 共通conformance harnessとseat固有fixtureの分担。
- Provider parameter mappingの宣言形式。
- bounds／picking／input-regionの共通語彙の最小閉集合。
- Content-Aware Scaleの需要、品質oracle、アルゴリズム、VRAM常駐実装、解析cache。
- 通常Object Provider、Camera Provider、Data Provider、Analysis Provider間で共有できる公開型の範囲。

これらは各具体taskで意味と負例を先に固定し、未決を共通APIへ焼かない。

## 8. Fable検収

2026-07-24、Claude Code Fable 5 (`claude-fable-5`、max effort)で関連正本、現行camera／plugin code、5 receiptをread-only監査した。

- `VERDICT: ACCEPT`
- `P0: 0`
- `P1: 0`
- 一般則: §3の結合条件と停止線を含めれば安全
- Content-Aware Scale: 非線形な評価済み画像変形という狭い候補分類で安全
- 5 receipt: 同一M5 lineage、corpus membership、重複なし、現行正本への回収根拠が成立
- 未決: §7およびCamera固有未決を維持

P2 8件は同じ変更で反映した。semantic seat必須判定、要約のeffect-stack条件、custom UI凍結参照、`1 D2 macro＝1 Undo`、blob根拠行／価値分類、HVR-D04状態と動的count、`docs/README.md`入口、本節の検収記録である。
