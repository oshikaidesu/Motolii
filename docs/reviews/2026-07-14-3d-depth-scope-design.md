# 2Dレイヤー順と3D深度合成の境界設計（2026-07-14）

ステータス: **設計レビュー**（仕様採択前。M5/C-4を変更しない）

対象: 「映像が主役なら、Z位置に合わせてレイヤーを自動整理すべきではないか」という疑問と、将来のレイヤー間深度遮蔽の置き場。

## 結論

**Zから描画順を導出することは有効だが、タイムライン項目そのものを自動並べ替えしてはならない。**

- タイムライン順は、所有・選択・時間編集・親子関係を追うための安定した authoring order とする
- 空間上の前後関係はキャンバスの責務とし、将来の明示的な `Depth Group` 内だけで camera-space depth から render order を導出する
- トップレベルと通常グループは現行C-4どおりレイヤー順を正とする
- 深度はグループ境界を越えず、グループ出力はpremultiplied RGBAへ平坦化して上位へ渡す
- `Depth Group` はv1へ入れない。M5の現行`LayerSourcePlugin → RGBA`境界とは別能力として、必要性を実作例で確認した後に判断する

この分離により「映像の見た目をZに従わせる」ことと「編集対象が再生中に飛び回らない」ことを両立できる。

## 先例の比較

### After Effects: 暗黙binは採らない

After Effectsは、連続する3Dレイヤーを同じ3D binで処理する。2Dレイヤー、3D adjustment layer、layer style付き3Dレイヤー等がbinを分断し、別bin同士は交差・相互シャドウを行わない。現在はTimelineへbin境界を表示するが、スコープの意味が隣接項目の種類から創発する点は変わらない。

出典: [Adobe — 3D animation settings / binning indicators](https://helpx.adobe.com/after-effects/desktop/work-with-3d-composition/work-with-3d-animation-settings/3d-animation-settings.html)

Motoliiでは「ある項目の種類やトグルが隣接レイヤーの意味を変える」方式を採らない。深度集合は明示コンテナだけで作る。

### Apple Motion: 明示2D/3Dグループは強い先例

Motionは、2Dグループ内をレイヤー順、3Dグループ内を深度順で合成する。さらに3Dグループには、深度順を無効にしてLayers list順へ戻す`Layer Order`設定と、平坦化する`Flatten`設定がある。

出典: [Apple — 2D and 3D group properties in Motion](https://support.apple.com/guide/motion/motn2fb59f53/mac)、[Apple — Create 3D intersection](https://support.apple.com/guide/motion/motn369b0783/mac)

「明示グループの中だけ意味を切り替える」点はMotoliiのグループ境界・ベイク・単一評価モデルと整合する。ただしMotionでも、フィルタやマスクによるラスタライズが交差を止める場合がある。Motoliiはこれを無言の意味変更にせず、非対応操作を拒否するか境界状態を可視化する必要がある。

### Fusion: 単一Zによるピクセル比較は限定解

FusionのZ Mergeは、foreground/backgroundの固定順ではなく、各ピクセルのZ値を比較して前後を決める。これは各ピクセルに代表面が1つある不透明・cutoutに近い素材では有効だが、複数の半透明サンプルが同じピクセルへ重なる場合の完全解ではない。

出典: [Blackmagic Design — Fusion Tool Reference](https://documents.blackmagicdesign.com/UserManuals/Fusion7_Tool_Reference.pdf)

### Nuke Deep / GPU OIT: 半透明の完全解はRGBA+Dより広い

Nukeのdeep imageは、1ピクセルに複数の色・不透明度・camera-relative depthサンプルを保持する。KhronosのOIT例も、透明ジオメトリの色と深度をピクセルごとのリストへ収集し、ピクセル単位でソートしてからalpha blendする。

出典: [Foundry — Deep Compositing](https://learn.foundry.com/nuke/current/content/comp_environment/deep/deep_compositing.html)、[Khronos — OIT with per-pixel ordered linked lists](https://docs.vulkan.org/samples/latest/samples/api/oit_linked_lists/README.html)

したがって、単一の`RGBA+D`だけでは「同じパーティクルレイヤーの一部がキャラクターの前、一部が後ろ」を一般には保持できない。1ピクセル内で前後両側の粒子が重なれば、平坦化時点で情報が失われる。完全性が必要ならdeep samples、OIT、または元ジオメトリを共有レンダーパスへ参加させる能力が必要になる。

## 自動整理の正しい意味

Documentのレイヤー列をZ順へ書き換えてはいけない。Z・親変形・カメラはいずれも時刻で変化するため、永続順まで同期すると次の問題が起きる。

- 再生中に行と展開済みプロパティが上下へ移動する
- 選択、キーフレーム編集、マスク、親子関係の追跡が不安定になる
- Z交差のたびに履歴・ジャーナル・キャッシュ無効化が発生しかねない
- カメラを動かしただけでDocument構造が変わる
- 同距離付近で順序が振動し、状態fulなhysteresisを入れると`f(t)`の純関数性を損なう

代わりに、明示Depth Groupのrender orderを評価時の派生値とする。

```text
authoring_order: Documentに永続化する安定順
render_order(t): CompCamera(t)とworld transform(t)から毎フレーム導出
```

ソートキーはworld-spaceの`position.z`ではなくcamera-space depthである。カメラが回転・移動すればworld Zと画面上の前後は一致しない。同距離のtie-breakはauthoring orderとし、決定論を固定する。

## 段階案

### v1: C-4を維持する

- トップレベル/グループ内部ともレイヤー順
- Zはカメラ投影・視差・見かけスケールだけに使う
- 点群/メッシュ内部のZ解決は各LayerSourcePlugin内で完結
- 前後へ分けたい決定論的パーティクル等は、同一seed/同一パラメータの2出力を分割Z平面で手前・奥へ分けるプラグイン作法を使う

最後の方法は特定表現の実用イディオムであり、一般深度合成とは呼ばない。

### v1.x候補: Depth-ordered Cards

実作例で必要性が出た場合だけ、非交差平面を対象に明示Depth Groupをスパイクする。

- 子はcamera-spaceの代表深度でback-to-frontに並べる
- authoring orderは変更しない
- 平面同士の交差、循環する前後関係、複数半透明面の完全性を保証しない
- UIと仕様では「Depth compositing」ではなく「Depth-ordered Cards」等、能力を過大表示しない
- alpha overlapと交差の反例fixtureを置き、近似であることをゴールデン化する

単純なオブジェクト/レイヤー単位ソートは、透明面が交差する場合には完全順を作れず、ポリゴン分割が必要になる場合がある。

出典: [Khronos — Transparency Sorting](https://wikis.khronos.org/opengl/Transparency_Sorting)

### 将来候補: Scene Participant

不透明・cutout素材の正確な交差が実需になった場合は、RGBA化済みレイヤーを並べるのではなく、参加ソースを共有depth passへ描く別能力を検討する。半透明の完全性まで必要なら、その先にOIT/deep samplesがある。

これは現行`LayerSourcePlugin`へdepth textureを足すだけの変更ではない。別trait/別出力能力として意味論、対応alpha、projection、depth range、flatten境界を同時に決める必要がある。

## ゲームエンジン追補（Unity / Unreal、2026-07-14）

### 判定: アルゴリズムは新しくない。編集モデルへの接合部がMotolii固有

ゲームエンジンはすでに「Scene/Hierarchyの所有順を動かさず、カメラと明示priorityからrender orderだけを派生する」設計を広く採っている。したがって、Zに応じた自動描画順そのものは新発明ではない。

Motolii固有の設計課題は、これを次の既存契約へどう接合するかにある。

- タイムライン順を正とする2Dコンポジット
- グループを内部合成した1枚として上位へ渡す再帰評価
- `LayerSourcePlugin → premultiplied RGBA`のflat境界
- 時刻ランダムアクセス、フレーム並列、preview/export同一関数
- ブレンドモード、マスク、エフェクト、ベイクを含むauthoring semantics

つまり、探索すべき新規性はソートアルゴリズムではなく、**ゲームエンジンのderived render orderを映像編集の安定したauthoring orderへ逆流させずに統合する境界**である。

### Unity: queue → 明示priority → 距離 → groupという多段解

Unity 6のBuilt-in Render Pipelineは、まずBackground / Geometry / AlphaTest / Transparent / Overlayのrender queueへ分ける。opaque側は既定front-to-back、transparent側は別の透明ソート規則を使う。2D Rendererの透明queueでは、Sorting Layer / Order in Layer、render queue、camera distance、Sorting Group等の優先順位を重ねる。

出典: [Unity 6 — レンダーキューとソート](https://docs.unity3d.com/ja/current/Manual/built-in-rendering-order.html)、[Unity 6 — 2D Rendererのソート](https://docs.unity3d.com/ja/current/Manual/2d-renderer-sorting.html)

重要な先例は次の4点。

1. **Hierarchyを並べ替えない**: sort modeは描画時の派生規則であり、GameObjectの所有階層を書き換えない
2. **カメラ相対/軸投影を選ぶ**: Perspective、Orthographic、Custom Axisを分ける。Orthographicはview direction沿いの距離を使う
3. **Sorting Groupを外側には原子的に扱う**: group全体へroot位置由来の単一camera distanceを与え、外部とソートしても内部順を維持する
4. **内部は別規則**: Sorting Group内では子ごとのcamera distanceを無視し、各RendererのSorting Layer / Order in Layerを使う

出典: [Unity 6 — Sorting Group内部のソート](https://docs.unity3d.com/ja/current/Manual/sprite/sorting-group/sort-renderers-within-sorting-group.html)

これはMotoliiの「通常グループは内部レイヤー順を保ち、平坦化した1枚を上位へ渡す」と同型である。一方、Unityの`Sort At Root`は親Sorting Groupを無視してroot levelとソートできる脱出口を持つ。Motoliiでは**深度が明示グループ境界を越えない**規律と衝突するため、この脱出口を採らない。

### Unity HDRP: 透明は単一機構で解けず、救済策が分裂する

HDRPは透明surfaceに対して、manualなSorting Priority、背面→前面の2 draw、Transparent Depth Prepass、Depth Write、Transparent Depth Postpass、motion vector出力を個別に持つ。これは「depth bufferを足せば透明が一意に解ける」のではなく、目的ごとに別の近似・補助パスが要る傍証である。

出典: [Unity HDRP — Transparent Surface Type](https://docs.unity3d.com/ja/Packages/com.unity.render-pipelines.high-definition%4010.5/manual/Surface-Type.html)

Motoliiはこの複雑さをv1.xのDepth-ordered Cardsへ持ち込まない。soft alpha、正確な交差、DOF/motion blur用depth、透明物のdepth writeを1つの`depth=true`へ畳まない。

### Unreal: 自動距離ソートにはmanual overrideが必要になる

Unrealはtranslucency sort policyとしてcamera centerからbounds centerまでの距離、post-projection Z、固定軸へのprojectionを区別する。公式資料は、近接する透明物で煙が球の前後へ突然popする例を示し、`Translucency Sort Priority`による手動上書きを案内する一方、それが別のsorting issueを生む可能性も明記する。

出典: [Unreal Engine — Translucency sort policies](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/Engine/ETranslucentSortPolicy__Type)、[Unreal Engine — Using Transparency / Sort Priority](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-transparency-in-unreal-engine-materials)

さらにUnrealのMotion Design機能は、透明priorityの算出をOutliner Top First / Bottom First、Camera Distance、Manualから選べる。これはMotoliiで検討中の「authoring order」「camera-derived order」「manual override」という3択とほぼ同じ先例である。

出典: [Unreal Engine Motion Design — Translucent Priority modifier](https://dev.epicgames.com/documentation/en-us/unreal-engine/modifiers-in-unreal-engine#translucentpriority)

ただし、Motoliiでは1レイヤーのmanual priorityを最初から公開しない。数値priorityが大量に残るとタイムライン順とは別の見えない順序台帳になる。必要性がfixtureで証明された場合も、Depth Group内だけの`sort bias`等に狭め、UIに実効順を常時表示する。

## ゲームエンジン先例から増えた懸念点

### G1. 代表点の選択で結果が変わる

UnityはSpriteのcenter/pivot、Unrealはbounding sphere centerやprojected Zを候補に持つ。大きいカード、回転したカード、エフェクトで見た目のboundsが広がるレイヤーは、どの代表点を使っても一部ピクセルの前後と一致しない。

スパイクでは少なくとも次を比較し、名称と保証範囲を固定する必要がある。

- anchorのview-space Z
- transform後bounds centerのview-space Z
- cameraからbounds centerまでのEuclidean distance
- view directionへのprojected distance

v1.x候補はcamera-space projected depthを第一案とするが、実装前に横へ大きく離れた同一Zカード、回転カード、camera orbitのfixtureで判定する。

### G2. opaque / cutout / soft alphaを分類できるか

ゲームエンジンはGeometry、AlphaTest、Transparentを別queueにできる。Motoliiのpremultiplied RGBAレイヤーは、1枚の中にopaque pixel、soft edge、完全透明pixelを同時に持てるため、レイヤー単位分類が難しい。

共有depth passへ進む前に、少なくとも次を意味論として分離する必要がある。

- opaque: depth test/write可能
- cutout: alpha thresholdでfragment discard後にdepth write可能
- soft alpha: back-to-frontまたはOITが必要。単純depth writeは透明edgeを誤遮蔽する

分類をプラグイン推測やフレーム内容の走査で決めず、将来能力の明示契約として扱う。

### G3. nested groupの代表深度とflatten境界

Unity Sorting Groupは外部に対してroot位置由来の単一距離を持ち、内部順を維持する。この方式は安定するが、子が奥行き方向へ大きく広がるgroupでは、外部オブジェクトが本来その子の間へ入る表現を捨てる。

Motoliiは「depthはgroup境界を越えない」を優先するため、この損失を仕様として受け入れる。Depth Groupの入れ子を許す場合も、内側を先に平坦化した原子的カードとして扱う。親を無視してrootへ参加するUnity式`Sort At Root`は作らない。

### G4. 自動ソートには必ずoverride要求が出る

Unity/Unrealはいずれもdistance sortに加えてSorting Layer / Order / Priorityを持つ。代表点ソートの限界により、制作現場では「この煙だけ常に手前」の指定が必要になるためである。

ただしmanual整数priorityは局所修正が別の不具合を生む。導入判断時は次の順で狭くする。

1. 同距離だけauthoring orderでtie-break
2. Canvas上で診断し、カード分割やgroup分割を案内
3. それでも実需が残る場合だけDepth Group内の明示`sort bias`

`sort bias`を持つなら時刻tで評価可能にし、hidden stateや再生履歴依存を入れない。

### G5. tie-breakをGPU/エンジン内部へ任せない

Unity公式は、複数Rendererが同じsorting priorityの場合の最終tie-breakをユーザーが制御できない内部処理とし、明確なpriorityを与えるよう勧める。Motoliiはpreview/export、OS、GPU backendを跨いだ決定論が必要なので、同値時の順序をauthoring order、さらに必要なら`LayerId`で完全順序化する。

### G6. sort flipの時間的pop

Unreal公式例のとおり、代表深度が交差すると透明物が突然前後へpopする。hysteresisで隠すと過去フレーム依存が入り、ランダムアクセス性を壊す。

したがって:

- render orderは各時刻で純粋に計算する
- popを状態で隠さない
- 深度差が閾値近傍を横切る区間を診断可能にする
- 必要ならユーザーがgroup分割、sort bias、真のScene Participantへ移れる導線を作る

### G7. motion blur / temporal samplingではサブフレームごとに順序が変わる

カメラやカードが高速移動する場合、シャッター区間内で前後が反転し得る。フレーム中央のrender orderを全サンプルへ使い回すとpreview/exportの見た目が破綻する。

モーションブラーが複数時刻サンプルを評価する場合、各sample timeでtransform、camera、render orderを同じ関数から再評価する。キャッシュキーにもCompCameraと参加レイヤーtransformの依存を含める。

### G8. effect / mask後のboundsとsort point

effectやmaskの結果から毎フレームboundsをreadbackしてsort pointを決めると、VRAM常駐と性能を壊す。sort keyはDocument上のtransformと宣言的boundsからCPUで決定可能にし、pixel結果を読まない。

boundsを拡張するeffectはNodeDesc等で静的/解析的footprintを宣言できる場合だけ反映し、未宣言effectのpixel alphaから推測しない。これは時間窓を静的`TemporalFootprint`で宣言する既存思想と同型である。

### G9. 実depth passにはnear/far・比較方向・精度が要る

現行`CompCamera`はnear/farを公開契約に持たない。Depth-ordered Cardsの代表値ソートには不要だが、共有depth textureへ進むとprojection matrix、near/far、reversed-Z、depth format、clear/compare規則が恒久意味論になる。

したがってRGBA+DやScene Participantの口を先に予約せず、実depth spikeでprecision fixtureを含めてから別仕様改訂する判断を補強する。

## ゲームエンジン追補後のスパイク追加条件

既存7条件に加えて、次を要求する。

8. perspective / orthographic / camera orbitでsort keyの意味が一貫する
9. center・pivot・bounds centerで結果が割れるfixtureを置き、採用代表点を明記する
10. opaque / cutout / soft alphaを別fixtureにし、同一機構で扱ったことにしない
11. nested groupは内側flattenを先に行い、親境界を越えるsort escapeを持たない
12. 同値tieはauthoring order + `LayerId`で全順序化し、GPU/backend差を許さない
13. depth crossingのpopをhidden stateで隠さず、任意時刻の結果が同一になる
14. motion blurはsample timeごとにcamera/transform/render orderを再評価する
15. sort key決定のためのGPU→CPU readbackを行わない
16. 真のdepth passへ進む場合はnear/far・depth format・比較規則・soft alpha拒否を先に仕様化する

## `FrameDesc`へdepthを予約しない

現行`FrameDesc`は色フレームのサイズ・stride・pixel format・color space・premultiplied alphaを記述する。depth attachmentは色フレームの属性ではなく、別texture format、比較規則、projection、sample semanticsを持つレンダ成果物である。

さらにM5仕様は、厳密なレイヤー間遮蔽を現行`LayerSourcePlugin`境界とは別拡張として扱うと既に宣言している。意味が未決のまま`FrameDesc`や公開traitへ口を焼くことはGR-PVの「意味が先」「恒久面は狭く」に反する。

したがって、今はコードへ`#[non_exhaustive]`やoptional depth fieldを足さない。将来の判断席だけを本レビューに残す。

## 将来スパイクの合否条件

1. Depth Group外の既存コンポは導入前後でピクセル同一
2. Z・カメラをアニメしてもDocument順、Undo履歴、選択行は変わらない
3. 深度意味論は明示グループ内部だけに閉じ、隣接レイヤーから創発しない
4. camera-space depthを使い、同距離はauthoring orderで決定論的に解決する
5. マスク・フィルタ・flattenが深度参加を止める場合、無言で切り替えない
6. 非交差カード、交差カード、soft alpha、循環前後関係を別fixtureにし、保証範囲を機械判定する
7. preview/exportは同一の評価関数を通り、差は`Quality`だけ

## UIへの含意

タイムラインの安定順を守りつつ映像中心で操作するため、将来のDepth Groupでは次を候補とする。

- グループ境界と合成モードをicon・形・ラベルで識別可能にする
- 現在フレームのdepth rankを派生badge/gutterで示す
- 「現在深度順で表示」は一時的なview projectionとし、Document/Undoへ書かない
- キャンバスから前後の対象を選べるようにする
- 深度交差や近似alphaが保証外になる箇所を診断表示する

これは「読む前に識別できる」GR-UI 9と、UI状態をDocument意味論へ逆流させないGR-UI 1/5の両方に従う。

## 判定

- **採用候補**: 明示Depth Groupというスコープ、render orderのcamera-space depth派生、authoring orderの固定
- **棄却**: Z変化に応じたDocument/タイムライン行の自動並べ替え、AE式の暗黙隣接bin
- **縮小**: レイヤー単位Zソートは非交差カード用近似としてのみ扱う
- **延期**: RGBA+D、共有depth pass、OIT、deep samples。実需と意味論が揃うまで公開契約へ予約しない
- **現仕様への影響**: なし。M5/C-4を維持する
