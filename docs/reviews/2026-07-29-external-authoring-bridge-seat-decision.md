# External Authoring Bridgeの将来席予約

作成日: 2026-07-29

状態: **決定**。Illustrator／Figma／Blender等の外部制作toolから、選択した意味を
Motoliiへ直接Push／Pullし、通常編集可能なtyped成果へmaterializeする
`External Authoring Bridge`の席を将来に残す。v1ではbridge runtimeを実装しない。
IPC、wire format、schema、manifest、discovery、permission UI、linked update、個別tool対応は
未統一であり、本書は公開API、Document field、plugin kind、実装taskの追加を許可しない。

関連正本:
[小さなコアと探索可能な拡張](../extensible-core-model.md)、
[Vism作者journeyとshader依存closure](2026-07-27-vism-authoring-journey-decision.md)、
[Vismプラグインカタログ](../vism-plugin-catalog.md)、
[Vismコンセプト](../vism-package-concept.md)、
[M5 3D合成とポストプロセス](../specs/M5-3d-and-post.md)。

## 1. 決定

ファイルを介したAsset importとは別に、外部toolの現在選択をMotoliiの現在文脈へ渡す
authoring時bridgeの席を閉じない。Bridgeは外部tool固有の意味をHostへ直接実行させるruntimeではなく、
受け取った候補をMotoliiのtyped Asset、VectorRecipe／PathOp、text、mesh、materialized data、
将来締結されるshader／material source等へ変換し、Authoring Toolと同じpreflight／commit境界へ渡す。

```text
Illustrator / Figma / Blender / Houdini / shader tool / 任意の作者tool
                         ↓ Push / Pull
               External Authoring Bridge
                         ↓
     typed transfer proposal + source provenance + loss
                         ↓
              Host admission / target resolve
                         ↓
       preview: Create / Update / Fork / Reject
                         ↓
           Authoring Tool typed command batch
                         ↓ preflight
              1 macro commit or no change
                         ↓
       通常のDocument / Asset / Vism要求として再現
```

Bridgeの価値は対応file形式の数ではない。外部toolを、MotoliiのPath Editor、Mesh／Material Editor、
Geometry Generator、Shader Editor、DataTrack Author等として選択単位で利用し、file分割、layer準備、
中間render、手作業の再構築を減らすことにある。

これは[ジェネラティブユーザー境界 §4](../generative-user-boundary.md#4-表現を受ける5つの正規経路)の
`E. External Material`を選択単位・明示再送へ開くauthoring入口であり、第六の保存経路ではない。
外部toolから受けた後の責任寿命は、materialize、Asset、Vism要求、Bake等の既存分類へ必ず帰着させ、
`Live Link`という新しいDocument意味を暗黙に作らない。

## 2. 席として先に保存する意味

具体形式より先に、将来の設計が次を表現できる余地を保存する。

1. **選択単位**: document全体のimportだけでなく、外部toolまたはMotoliiで選んだ対象だけを送れる。
2. **方向**: 外部toolからのPush、Motolii側からのPull、Motoliiで選んだ対応可能な意味の外向き投影を
   同じ責任境界で比較できる。双方向対応を全bridgeへ義務づけない。
3. **明示操作**: Create、既存対象の内容更新、style更新、独立Fork、外向き投影を区別できる。
   これは将来の意味分類であり、現行enumやcommand名ではない。
4. **更新対象の同定**: 同名layer、配列index、現在の選択順へ依存せず、どのMotolii対象を更新するか
   preflightで一意に示せる。外部toolのopaque IDをMotoliiのObjectIdへ流用しない。
5. **保持範囲**: geometry更新時に既存animation、effect、transform、placement、selection等の何を保持し、
   何を置換するかを操作前に示せる。更新の意味をbridgeごとの隠れ挙動にしない。
6. **typed materialization**: rasterへ落とす前のpath、text、mesh、material、instance等のidentityを、
   受け手の締結済み型が表現できる範囲で保つ。全形式を一つのopaque JSONへ潰さない。
7. **lossと由来**: 未対応要素、近似、rasterize／bake、単位・軸・色・font・時間変換、作者・license・
   source toolをCommit前に診断できる。proposalが運ぶsource tool、作者、license等の由来は
   Bridge側の申告として扱い、検証済み事実や確認済みbadgeへ自動変換しない。
8. **保存後の独立性**: Commit済みProjectのopen、Preview、Exportは、外部tool、bridge process、
   network、作者端末の絶対pathを必要としない。継続依存が必要な表現は、別途締結されたVism／Asset要求として
   明示し、bridge sessionを作品意味にしない。
9. **Host編集規律**: BridgeはDocumentを直接変更しない。Bridge processへ渡すのは、利用者が明示した
   対象のtyped selection／target文脈と、当該操作に必要な公開意味の投影だけであり、Document全体の
   read-only snapshotを既定で渡さない。snapshotを読むのはHost内のadmission／Authoring Tool境界である。
   外向きに渡すtyped subsetは将来のbridge固有authority contractで締結する。Hostがsingle writerで
   一つのmacroとしてCommitし、Cancel、拒否、途中失敗は変更ゼロとする。

これらはfieldを今から予約する要求ではない。将来の席を「毎回新規Assetを作るだけ」「外部file pathを
覚えるだけ」「名前で置換するだけ」の恒久契約で塞がないための審判である。

## 3. 制作動線

### 3.1 外部toolからPushする

1. 利用者がMotoliiの受け先Compositionと、必要なら更新対象を明示する。
2. 外部toolでpath、text、mesh、material等を選び、BridgeへPushする。
3. Bridgeは候補payload、source／selection由来、要求能力、変換lossをtransfer proposalとして渡す。
4. Hostはpayload型、座標／色／時間、asset closure、permission、target一意性、上限をpreflightする。
5. UIはCreate／Update／Fork／Reject、保持される意味、失われる意味、生成されるObject／Assetを示す。
6. Acceptで一つのD2 macroへCommitする。失敗またはCancelではDocument、Asset store、selectionを変更しない。
7. Commit後の成果はMotoliiの通常UI、Undo、journal、Preview／Export、欠落診断を使う。

### 3.2 MotoliiからPullまたは外向き投影する

1. 利用者がMotoliiで対象と外向きに渡す意味を選ぶ。
2. Hostは外向き投影可能なtyped subsetだけをBridgeへ渡し、外部toolの内部Documentへ直接書かない。
3. Bridgeは外部tool側でCreate／Updateできる候補とlossを示す。
4. 外部tool側の確定、Undo、保存は外部toolが所有する。Motolii側のDocument正本やUndoと一体化しない。

Pullは汎用Project exporterではない。Pathを外部で精密編集する、Mesh／Materialを上流toolへ戻す等の
限定用途から反証し、Motoliiの全意味を他toolで再現できるとは称さない。

### 3.3 後日の明示Update

再Push時は自動同期せず、候補sourceと対象、前回由来、現在のMotolii編集、保持／置換範囲を比較してから
UpdateまたはForkを選ぶ。source identityや更新関係の論理形式と保存ownerは未統一であり、
この動線からDocument field、sidecar、daemon、background watcherを逆算しない。

## 4. 責任境界

| 責任 | External Bridge | Motolii Host | Projectへ残るもの |
|---|---|---|---|
| 外部tool API、外部選択、外部側permission | 所有 | 所有しない | 残さない |
| source固有意味から候補payloadへの変換 | 所有 | 形式と上限を検査 | 変換済み意味と必要な由来だけ |
| target解決、Motolii ID、single writer | 提案まで | 所有 | Host正準identity |
| preflight、loss表示、Accept／Cancel | 診断材料を供給 | 所有 | Commit receipt候補。形式未統一 |
| Undo、journal、atomic commit | 所有しない | 所有 | 通常の編集履歴 |
| Preview／Export、GPU resource、cache | 所有しない | 所有 | 通常の評価意味 |
| 外部tool側のUndo／保存 | 外部toolと分担 | 所有しない | 残さない |

BridgeをVism、Input Adapter、Delivery Adapterのいずれかへ無理に畳まない。

- **Input／Asset Adapter**はfileやstreamをMotoliiが読めるAssetへする。
- **External Authoring Bridge**は外部toolの選択文脈から、編集候補と更新対象を明示操作で渡す。
- **Authoring Tool**は候補をHost標準のtyped command結果へmaterializeする。
- **Vism**は保存後も評価される再利用可能な映像表現と要求能力を所有する。
- **Delivery Adapter**は作品の外向き成果物を作る。Pullを理由にv1の完成映像以外のDeliveryを解凍しない。

一つの第三者製品が複数役割を同梱する可能性はあるが、authority、permission、lifecycle、失敗範囲は
この分類どおり分けて審判する。

## 5. 第三者への開放、正本化、商流

### 5.1 席は第三者へ開き、対応appをCoreへ列挙しない

本席はfirst-party専用にしない。CoreとHost公開契約に対応外部appのallowlist、app名による
capability分岐、特定vendor SDK型を置かず、第三者は同じtyped proposal境界とbridge固有authority
contractだけで到達できる。first-party BridgeにもHost private API、raw Document、審査省略、
広い既定permissionを与えない。

これは接続機会の平等であり、無審査の接続許可ではない。接続はHost所有のconsent／permissionを通り、
既定でlistenする常駐daemon、自動接続、無操作でのpayload受理を製品へ常設しない。
Overlord v1型のapp名一致broadcastを作らず、将来のdiscovery／pairingは受信先Host instanceと
利用者操作を一意にできることをfixtureで審判する。具体protocol、UI、process構成は未統一である。

### 5.2 authorityは移動せず、受けた意味だけを正本化する

Documentのauthorityは接続前、proposal中、Commit後の全段階でMotoliiのsingle writerが所有し、
Bridgeまたは外部toolへ一度も移動しない。Acceptで受けた意味はCommit時点で通常の
Document／Asset／Vism要求として正本化し、以後のsource側編集は作品へ自動反映されない。

再Pushは常に新しいtransfer proposalとして同じadmission／preview／Acceptを通る。前回のAcceptは
次回のpermission、target、保持範囲、lossの承認を意味しない。外部toolとMotoliiが一つの作品stateを
共有する共同正本、last-write-wins、暗黙mergeをどの将来形でも作らない。

### 5.3 配布と商流を開き、権限とは分ける

Bridgeの配布・商流には
[第三者Vismの持続可能な経済圏](2026-07-26-third-party-sustainable-economy-decision.md)と
同じ憲法を適用する。BridgeはVism packageである必要がなく、外部tool extension＋companion app等の
独立製品形態、無料／有料、OSS／proprietary、買い切り／subscription等を同格に許す。
MotoliiはBridgeのmarketplace、決済、購入者管理を所有しない。

価格、license、source公開、first-party性、署名、catalog掲載をtrust、互換、接続許可へ変換しない。
権限は商流上の同格性から導出せず、将来のbridge固有authority contractだけが接続時permission、
外向きtyped subset、payload上限、filesystem／network／process／secret分離を決める。
Bridgeのcatalog掲載とKitからの要求表現は未統一であり、Vism catalog／Kitの現行決定から逆算しない。

## 6. payloadは一つに統一しない

最初から万能な`BridgePayload`や外部scene graphを正本にしない。候補は受け手の意味ごとに分ける。

| 外部意味 | 正規化先の候補 | 保つもの | materialize／bake候補 |
|---|---|---|---|
| vector／shape | VectorRecipe／PathOp／typed Shape | open／closed、vertex、first vertex、fill／stroke、grouping | 未対応appearance、gradient mesh等 |
| text | text source＋締結済みstyle／font要求 | cluster化前の文字、方向、style由来 | outline／rasterizeは明示loss |
| raster | Asset | pixel、色解釈、alpha、source由来 | 外部effect適用済み画像 |
| mesh／scene subset | glTF等のtyped Asset | mesh、material値、TRS、単位・軸 | simulation、volume、未対応animation |
| shader／material source | 将来締結されるsource closure／recipe | 宣言parameter、binding、依存、由来 | 非対応node、texture bake |
| data／motion | DataTrack／typed parameter／Bake | 時間基準、単位、stable identity | 外部runtime依存の計算結果 |

表は採用済みschemaではない。各行は対象specとfixtureで独立に締結する。Bridge都合で
GAP-15、M5、VSM-B5、DataTrack、Freezeの公開契約を同時に決めない。

## 7. v1と将来を分ける

### v1

- Bridge runtime、companion app、常駐service、公開bridge SDKを実装しない。
- glTF／SVG等の通常importと、Authoring Toolのatomic commitをそれぞれの既存gateで閉じる。
- Bridge用のDocument field、外部object ID、URL、socket、linked sourceを予約しない。
- 現行importがCreate-onlyでも、将来Updateを永久に拒む一般原則へ昇格しない。

### 将来の再入場条件

将来主張する能力ごとに、対応する独立fixtureを先に作る。

1. **Vector Push**: 選択pathをCreateし、grouping／名前／座標変換とlossを確認する。
2. **Update in place**: geometryだけを更新し、既存animation／effect／placementを保持する。
3. **Fork**: 同じsource候補から別Objectを作り、既存対象を変更しない。
4. **Missing companion**: 外部toolとBridgeが無い端末でもProjectをopen／Preview／Exportできる。
5. **Ambiguous target**: 重複、削除、copy、source identity衝突時に無言で別対象を上書きしない。
6. **Atomic failure**: unsupported payload、未知／未来のprotocol／payload版、permission拒否、取消、
   途中切断、開始revision不一致でDocument／Asset変更ゼロ。Preview済みproposalと異なる内容を
   Commitせず、Accept後にBridgeからpayloadを再取得しない。
7. **3D／shader loss**: 未対応material、node、simulationを一覧化し、近似、Bake、Rejectを区別する。
8. **Round-trip limit**: Pull可能なtyped subsetと戻せない意味を表示し、完全往復を偽称しない。

8 fixtureは席全体の一括入場条件ではなく、能力claimごとの再入場条件である。最初の一粒は
Vector Push (1)、Missing companion (4)、Atomic failure (6)だけを合格条件とし、
`source selection → typed proposal → Host macro → 再open`を閉じる。

Update系claim (2／3／5)はsource identityの論理形式と保存ownerの独立決定後、
3D／shader (7)は対応payload正本の締結後、Round-trip (8)はPull可能なtyped subsetの締結後に
個別へ入場する。wire formatやSDKを先に作らず、vector契約をmesh、shader、dataへ自動一般化しない。

## 8. 先例の読み方

[Overlord公式Illustrator workflow](https://battleaxe.co/overlord/docs/illustrator)は、
Illustrator／Figmaの選択をAfter EffectsへPush／Pullし、既存shapeの更新も扱う。
一方、Appearance、gradient mesh、tapered stroke等の未対応要素があり、Host間の意味差と
element identity不足が更新能力を制約する。MotoliiはUI、転送形式、対応表を模倣せず、
選択単位、明示Update、typed materialization、loss診断、stable targetという問題設定だけを先例にする。

Overlordの成立から、Motoliiのschema、双方向義務、Adobe型layer階層、外部ID保存を逆算しない。

## 9. 停止線

- Bridgeまたは外部toolへ自由な`&mut Document`、raw journal、Host private型を渡す。
- 外部tool名、layer名、property path、配列index、現在選択順で更新対象を決める。
- 外部toolのopaque IDをMotoliiのObjectId／AssetIdとして保存する。
- Project open、Preview、Exportがcompanion process、network、絶対path、外部tool installへ到達する。
- background watcherや自動同期が、利用者のAcceptなしにDocumentまたはAssetを変更する。
- source側変更とMotolii側変更の競合をlast-write-winsや黙示上書きで処理する。
- 一部だけCommitした後に変換失敗し、Undo一回で元へ戻らない。
- 未対応要素を黙って削除、近似、rasterize、bakeする。
- 全外部toolをopaque JSON／汎用scene graph／万能bridge plugin kindへ畳む。
- Illustrator、Blender、Figma等の具体SDK型、OS IPC、vendor APIをDocument、Core公開API、
  Vism expression contractへ焼く。
- 対応appのallowlist、app名一致routing、first-party Bridge専用capabilityをCore／Host公開契約へ焼く。
- 価格、license、source公開、署名、first-party性、catalog掲載をpermission、trust、互換の代用にする。
- Pullを理由にProject exporter、Delivery形式、双方向完全互換を同時に実装する。
- Bridgeのpermission、secret、filesystem、network、process authorityを
  Vismのambient authority 0や通常Asset importの検査だけで合格扱いする。

## 10. 動線と反映先

入口は[docs README](../README.md)と[決定逆引き台帳](../decision-index.md)。
表現需要からは[Vismプラグインカタログ §12](../vism-plugin-catalog.md#12-vismに入れないもの)、
作者入口からは[Vism作者journey](2026-07-27-vism-authoring-journey-decision.md)、
編集権限からは[extensible core §4.1](../extensible-core-model.md#41-authoring-toolは自由なmut-documentを受け取らない)、
表現の受入分類からは[ジェネラティブユーザー境界 §4](../generative-user-boundary.md#4-表現を受ける5つの正規経路)、
3D素材からは[M5](../specs/M5-3d-and-post.md)へ接続する。

実装を検討する時は、対象payloadの正本を先に閉じる。

- Vector／Path: `GAP-15`、PathOp、SVG import。
- Mesh／Material: `M5-P1`以降。
- Shader source: `VSM-B5`とshader closure。
- Data／motion: DataTrack、Analysis／Bake、Simulation／StateTrack。
- Commit／Undo: D2とAuthoring Toolのtyped command batch。
- permission／process／network: bridge固有のauthority contract。Vism安全契約から自動継承しない。

本書はこれらのtaskを直列化せず、Bridge実装を完了条件へ追加しない。各能力が締結された後も、
External Authoring Bridgeの一粒は`source selection → typed proposal → one macro or no change`へ限定する。

## 11. Fable 5 read-only助言の処分

2026-07-29、Claude Fable 5 (`claude-fable-5`、max effort)へ本書と関連正本、Codexが一次資料で
確認したOverlord構成を渡し、read-onlyで共有公開境界を監査した。初回判定は
`VERDICT: REVISE`、P0=0／P1=3、P2=7であり、固定SHAに対するrelease検収ではない。

採用したP1修正:

1. BridgeがDocument全体のread-only snapshotを受けるように読めた主語を分離し、
   Host内snapshotと外向きtyped projectionを分けた。
2. 8 fixtureを一括gateにせず、最初のVector PushとUpdate／3D／Pullの能力claim別gateへ分けた。
3. 「authorityがMotoliiへ移る」という候補表現を棄却し、authorityは常にsingle writerが所有、
   Acceptされた意味だけを正本化すると固定した。

同じ助言から、第三者開放とapp非列挙、既定off／明示consent、Bridge商流の同格性を採用した。
一方、IPC、wire format、source identity、permission UI、protocol互換方式、catalog掲載、
Kitとの関係、性能上限は未統一のまま維持した。Fable出力は助言であり、本書への採否と既存正本への
再照合はCodexが行った。
