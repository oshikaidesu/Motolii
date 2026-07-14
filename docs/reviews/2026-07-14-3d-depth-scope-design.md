# 2Dレイヤー順と3D深度合成の境界設計（2026-07-14）

ステータス: **採択済み設計レビュー**（2026-07-14ユーザー決定をM5へ反映）

対象: 「映像が主役なら、Z位置に合わせてレイヤーを自動整理すべきではないか」という疑問と、将来のレイヤー間深度遮蔽の置き場。

## 結論

**世界は1つだけとし、全オブジェクトが常にXYZと同じカメラを持つ。グループで切り替えるのは空間ではなく、Z遮蔽の解決ポリシーだけである。通常UIはOFF/ONへ簡略化し、AdvancedではAE式を含む複数方式を選べる。タイムライン項目そのものは自動並べ替えしない。**

- タイムライン順は、所有・選択・時間編集・親子関係を追うための安定した authoring order とする
- 全オブジェクトは種類によらず同じ正準XYZ世界・world transform・`CompCamera`を持ち、Zは常に投影・視差・見かけスケールへ効く
- 通常UIの`Z Occlusion=OFF / ON`は`Layer Order / Group Depth`へ対応する
- Advancedの`AE-style Bins`は、明示`Depth Participant`が連続する範囲だけをdepth binとして解決する
- ボタンはグループヘッダー/Inspectorに置き、遮蔽の影響範囲を箱として明示する
- Z遮蔽はグループ境界を越えず、どのポリシーもグループ出力をpremultiplied RGBAへ平坦化して上位へ渡す
- 共有depth参加境界は、現行`LayerSourcePlugin → RGBA`へdepthを密輸せずM5-P2Dで定義する
- 初期3方式は組み込みプリセットであり閉じた最終形ではない。将来方式も同じobject/world/cameraを入力とする追加ポリシーとして増やし、既存方式の意味を変えない

この分離により「映像の見た目をZに従わせる」ことと「編集対象が再生中に飛び回らない」ことを両立できる。

## 先例の比較

### After Effects: 既定にはせず、Advancedで明示化して採る

After Effectsは、連続する3Dレイヤーを同じ3D binで処理する。2Dレイヤー、3D adjustment layer、layer style付き3Dレイヤー等がbinを分断し、別bin同士は交差・相互シャドウを行わない。現在はTimelineへbin境界を表示するが、スコープの意味が隣接項目の種類から創発する点は変わらない。

出典: [Adobe — 3D animation settings / binning indicators](https://helpx.adobe.com/after-effects/desktop/work-with-3d-composition/work-with-3d-animation-settings/3d-animation-settings.html)

Motoliiではこれを通常UIの既定にはしないが、表現上の選択肢としてAdvancedの`AE-style Bins`へ取り込む。ただし互換再現ではなく、次の差を設ける。

- 全レイヤーは参加状態にかかわらず同じXYZ世界とZを持つ
- effect・mask・layer typeは無言でbin breakerにならず、明示`Depth Participant`だけが境界を作る
- bin境界はTimelineへ常時表示し、Advanced controlsを閉じても適用中のポリシーをbadgeで示す
- AEプロジェクト取込時は、AE固有のbreakerをimport adapterが明示参加フラグへ変換し、runtimeの隠れ規則にしない

これによりAE式の表現力を残しながら、「隣のレイヤーを触ったら意味が変わった」という遠隔作用を可視化する。

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

代わりに、同じworld transformからvisibility resolveを評価時の派生値とする。

```text
authoring_order: Documentに永続化する安定順
render_order(t): CompCamera(t)とworld transform(t)から毎フレーム導出
```

ソートキーはworld-spaceの`position.z`ではなくcamera-space depthである。カメラが回転・移動すればworld Zと画面上の前後は一致しない。同距離のtie-breakはauthoring orderとし、決定論を固定する。

## 段階案

### v1: 単一世界+選択可能な遮蔽ポリシーを実装する

- 2D texture平面、動画、テキスト、図形、glTF、点群のすべてに同じZ/world transform/共有cameraを通す
- `Layer Order`: 同じ投影結果をレイヤー順で遮蔽する。通常UIでは`Z Occlusion=OFF`
- `Group Depth`: 同じobject/world/camera表現をグループ全体の共有depth passで遮蔽する。通常UIでは`Z Occlusion=ON`
- `AE-style Bins`: Advancedで明示参加レイヤーの連続範囲ごとに共有depth passを作る
- グループの明示controlで切り替え、座標解釈とDocumentの子順は変更しない
- 既存プロジェクトと新規グループの既定は`Layer Order`にし、従来C-4のピクセル同一を守る

決定論的パーティクルを同一seed/同一パラメータの2出力へ分ける作法はOFF時の軽量な制作イディオムとして残すが、ONの代用品とは呼ばない。

### 軽量経路: Depth-ordered Cards

ON時の内部最適化またはDraft近似として、非交差平面を代表深度で並べる軽量経路を検討できる。

- 子はcamera-spaceの代表深度でback-to-frontに並べる
- authoring orderは変更しない
- 平面同士の交差、循環する前後関係、複数半透明面の完全性を保証しない
- `Group Depth`なのにこの近似へ無言fallbackしない。使う場合はDraft近似または追加ポリシーとして明示する
- alpha overlapと交差の反例fixtureを置き、近似であることをゴールデン化する

単純なオブジェクト/レイヤー単位ソートは、透明面が交差する場合には完全順を作れず、ポリゴン分割が必要になる場合がある。

出典: [Khronos — Transparency Sorting](https://wikis.khronos.org/opengl/Transparency_Sorting)

### 採択: 共通オブジェクトのdepth参加境界

`Group Depth`と`AE-style Bins`では、RGBA化済みレイヤーを並べるのではなく、`Layer Order`時と同じオブジェクト表現を共有depth passへ描く能力をM5-P2Dで定義する。不透明・cutoutは実depthで交差させる。半透明の完全性まで必要なら、その先にOIT/deep samplesがある。

これは現行`LayerSourcePlugin`へdepth textureを足すだけの変更ではない。別trait/別出力能力として意味論、対応alpha、projection、depth range、flatten境界を同時に決める必要がある。

### 遮蔽方式は固定3択にしない

3D表現では、単純depth buffer以外にも非交差カードの距離ソート、透明queue、weighted blended OIT、per-pixel linked-list OIT、deep samples等が必要になり得る。したがって初期3方式を最終enumとして閉じず、レンダ側には「同じobject/world/camera入力からvisibilityを解決するポリシー」という追加境界を置く。

ただし「拡張可能」は、未決の公開traitや自由文字列を今すぐDocumentへ焼く意味ではない。P2Dの実機spikeで次を証明してから、安定ID・version・能力宣言・alpha保証・構造化診断を持つ追加形式を仕様化する。

- 新ポリシーを追加しても既存ポリシーの座標・遮蔽結果を再解釈しない
- 未知/利用不能ポリシーを`Layer Order`へ無言fallbackしない
- preview/exportが同じポリシー実装を使い、差は`Quality`だけ
- backend固有APIを公開契約へ出さず、wgpu/WGSL抽象に留める
- ポリシーごとの対応object・alpha・depth・近似範囲を宣言し、UIが事前診断できる

通常UIはOFF/ONの2択を保つ。選択肢の多さはAdvancedへ段階的に開示し、表現力の拡張と日常操作の単純さを両立する。

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

Motoliiはこの複雑さを`Z Occlusion`ボタン1つで解決済みとはみなさない。ONを採用しても、soft alpha、正確な交差、DOF/motion blur用depth、透明物のdepth writeは別の意味論として扱う。

### Unreal: 自動距離ソートにはmanual overrideが必要になる

Unrealはtranslucency sort policyとしてcamera centerからbounds centerまでの距離、post-projection Z、固定軸へのprojectionを区別する。公式資料は、近接する透明物で煙が球の前後へ突然popする例を示し、`Translucency Sort Priority`による手動上書きを案内する一方、それが別のsorting issueを生む可能性も明記する。

出典: [Unreal Engine — Translucency sort policies](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/Engine/ETranslucentSortPolicy__Type)、[Unreal Engine — Using Transparency / Sort Priority](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-transparency-in-unreal-engine-materials)

さらにUnrealのMotion Design機能は、透明priorityの算出をOutliner Top First / Bottom First、Camera Distance、Manualから選べる。これはMotoliiで検討中の「authoring order」「camera-derived order」「manual override」という3択とほぼ同じ先例である。

出典: [Unreal Engine Motion Design — Translucent Priority modifier](https://dev.epicgames.com/documentation/en-us/unreal-engine/modifiers-in-unreal-engine#translucentpriority)

ただし、Motoliiでは1レイヤーのmanual priorityを最初から公開しない。数値priorityが大量に残るとタイムライン順とは別の見えない順序台帳になる。必要性がfixtureで証明された場合も、対象遮蔽ポリシー内だけの`sort bias`等に狭め、UIに実効順を常時表示する。

## ゲームエンジン先例から増えた懸念点

### G1. 代表点の選択で結果が変わる

UnityはSpriteのcenter/pivot、Unrealはbounding sphere centerやprojected Zを候補に持つ。大きいカード、回転したカード、エフェクトで見た目のboundsが広がるレイヤーは、どの代表点を使っても一部ピクセルの前後と一致しない。

スパイクでは少なくとも次を比較し、名称と保証範囲を固定する必要がある。

- anchorのview-space Z
- transform後bounds centerのview-space Z
- cameraからbounds centerまでのEuclidean distance
- view directionへのprojected distance

Depth-ordered Cards経路はcamera-space projected depthを第一案とするが、実装前に横へ大きく離れた同一Zカード、回転カード、camera orbitのfixtureで判定する。

### G2. opaque / cutout / soft alphaを分類できるか

ゲームエンジンはGeometry、AlphaTest、Transparentを別queueにできる。Motoliiのpremultiplied RGBAレイヤーは、1枚の中にopaque pixel、soft edge、完全透明pixelを同時に持てるため、レイヤー単位分類が難しい。

共有depth passへ進む前に、少なくとも次を意味論として分離する必要がある。

- opaque: depth test/write可能
- cutout: alpha thresholdでfragment discard後にdepth write可能
- soft alpha: back-to-frontまたはOITが必要。単純depth writeは透明edgeを誤遮蔽する

分類をプラグイン推測やフレーム内容の走査で決めず、将来能力の明示契約として扱う。

### G3. nested groupの代表深度とflatten境界

Unity Sorting Groupは外部に対してroot位置由来の単一距離を持ち、内部順を維持する。この方式は安定するが、子が奥行き方向へ大きく広がるgroupでは、外部オブジェクトが本来その子の間へ入る表現を捨てる。

Motoliiは「Z遮蔽はgroup境界を越えない」を優先するため、この損失を仕様として受け入れる。depth系ポリシーのグループを入れ子にする場合も、内側を先に平坦化した原子的カードとして扱う。親を無視してrootへ参加するUnity式`Sort At Root`は作らない。

### G4. 自動ソートには必ずoverride要求が出る

Unity/Unrealはいずれもdistance sortに加えてSorting Layer / Order / Priorityを持つ。代表点ソートの限界により、制作現場では「この煙だけ常に手前」の指定が必要になるためである。

ただしmanual整数priorityは局所修正が別の不具合を生む。導入判断時は次の順で狭くする。

1. 同距離だけauthoring orderでtie-break
2. Canvas上で診断し、カード分割やgroup分割を案内
3. それでも実需が残る場合だけZ Occlusion対象グループ内の明示`sort bias`

`sort bias`を持つなら時刻tで評価可能にし、hidden stateや再生履歴依存を入れない。

### G5. tie-breakをGPU/エンジン内部へ任せない

Unity公式は、複数Rendererが同じsorting priorityの場合の最終tie-breakをユーザーが制御できない内部処理とし、明確なpriorityを与えるよう勧める。Motoliiはpreview/export、OS、GPU backendを跨いだ決定論が必要なので、同値時の順序をauthoring order、さらに必要なら`LayerId`で完全順序化する。

### G6. sort flipの時間的pop

Unreal公式例のとおり、代表深度が交差すると透明物が突然前後へpopする。hysteresisで隠すと過去フレーム依存が入り、ランダムアクセス性を壊す。

したがって:

- render orderは各時刻で純粋に計算する
- popを状態で隠さない
- 深度差が閾値近傍を横切る区間を診断可能にする
- 必要ならユーザーがgroup分割、sort bias、Z Occlusion ONへ移れる導線を作る

### G7. motion blur / temporal samplingではサブフレームごとに順序が変わる

カメラやカードが高速移動する場合、シャッター区間内で前後が反転し得る。フレーム中央のrender orderを全サンプルへ使い回すとpreview/exportの見た目が破綻する。

モーションブラーが複数時刻サンプルを評価する場合、各sample timeでtransform、camera、render orderを同じ関数から再評価する。キャッシュキーにもCompCameraと参加レイヤーtransformの依存を含める。

### G8. effect / mask後のboundsとsort point

effectやmaskの結果から毎フレームboundsをreadbackしてsort pointを決めると、VRAM常駐と性能を壊す。sort keyはDocument上のtransformと宣言的boundsからCPUで決定可能にし、pixel結果を読まない。

boundsを拡張するeffectはNodeDesc等で静的/解析的footprintを宣言できる場合だけ反映し、未宣言effectのpixel alphaから推測しない。これは時間窓を静的`TemporalFootprint`で宣言する既存思想と同型である。

### G9. 実depth passにはnear/far・比較方向・精度が要る

現行`CompCamera`はnear/farを公開契約に持たない。Depth-ordered Cardsの代表値ソートには不要だが、共有depth textureへ進むとprojection matrix、near/far、reversed-Z、depth format、clear/compare規則が恒久意味論になる。

したがってRGBA+D、OIT、deep samplesの口を先に予約しない。M5-P2Dで同じオブジェクト表現を共有depth passへ参加させる実機spikeとprecision fixtureを先に通し、公開traitの形はその結果を使う仕様改訂で確定する。

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

## M5-P2Dスパイクの合否条件

1. `Layer Order`の既存コンポは導入前後でピクセル同一
2. Z・カメラをアニメしてもDocument順、Undo履歴、選択行は変わらない
3. `Group Depth`は明示グループ内部だけに閉じ、`AE-style Bins`は明示`Depth Participant`だけからbinを作る
4. camera-space depthを使い、同距離はauthoring orderで決定論的に解決する
5. マスク・フィルタ・flattenが深度参加を止める場合、無言で切り替えない
6. 非交差カード、交差カード、soft alpha、循環前後関係を別fixtureにし、保証範囲を機械判定する
7. preview/exportは同一の評価関数を通り、差は`Quality`だけ
8. 3D-2D-3D fixtureでparticipantを切り替え、bin境界と出力が仕様どおり変わる
9. Advanced controlsを閉じても、適用中の`AE-style Bins`と境界をTimeline上で識別できる
10. 未知/非対応ポリシーを別方式へ無言fallbackせず、構造化診断する

## UIへの含意

タイムラインの安定順を守りつつ映像中心で操作するため、Z遮蔽UIはprogressive disclosureを使う。

- 通常UIはグループヘッダー/Inspectorに`Z Occlusion`のON/OFFボタンだけを置き、`Layer Order / Group Depth`へ対応させる
- Advancedは`Occlusion Policy`として`Layer Order / Group Depth / AE-style Bins`を選べ、AE式のときだけ各子の`Depth Participant`を露出する
- Advanced controlsの表示/非表示はUser settingsまたはWorkspace/session候補であり、Document/ジャーナルへ焼かない。出力を変えるポリシーと参加フラグはDocument+D2 commandへ置く
- グループ境界、現在の遮蔽規則、AE式のbin境界をicon・形・ラベルで識別可能にする
- `AE-style Bins`を含むDocumentを開いたとき、Advancedが非表示でもheader badgeとbin outlineは消さず、編集controlを開く導線を出す
- 現在フレームのdepth rankを派生badge/gutterで示す
- 「現在深度順で表示」は一時的なview projectionとし、Document/Undoへ書かない
- キャンバスから前後の対象を選べるようにする
- 深度交差や近似alphaが保証外になる箇所を診断表示する

これは「読む前に識別できる」GR-UI 9と、UI状態をDocument意味論へ逆流させないGR-UI 1/5の両方に従う。

### 先例追加: 「奥行き展開」はAE圏で反復再発明されている

複数レイヤーをZへ並べ、元の2D構図を保ったままcamera parallaxへ変える操作は単発商品の偶然ではない。少なくとも次の別作者・別世代の製品が同じ穴を埋めている。

| 先例 | 中核操作 | AE由来の後処理 |
|---|---|---|
| [pt_Multiplane](https://aescripts.com/pt_multiplane/) | layered Photoshop/Illustratorをmultiplane化し、Z移動時の見た目を維持 | scale expression、controller null、Bake/Remove |
| [AnimateParallax](https://aescripts.com/animateparallax/) | visual depth view、Near/Far、個別marker、Even/Random/Reverse、Apply/Reset | Refresh/再登録、expression、Bake、precomp+Collapse Transformations注意 |
| [Parallaxer 3](https://aescripts.com/parallaxer/) | one-click scene、Autoscale、scene scale preset | Regroup/Flatten/Expandでscene構造を修復 |
| [DistributeLayers](https://aescripts.com/distributelayers/) | 選択layerを3Dへ配布し、position/rotation/scale/opacityへ非線形offset+random | script panelから一括適用 |
| [Align3D](https://aescripts.com/after-effects/3d/align-3d/) | Zを含む3D align/distributeとrange指定 | script panelから一括適用 |
| [Match Position](https://aescripts.com/match-position/) | first/last間の3D distribution、割合指定 | parent構造差をtool側で補修 |
| [Camera 3D Toolkit Pro](https://aescripts.com/camera-3d-toolkit-pro/) | X/Y/Z spacing、parallax、camera rig/focus | camera/null/3D mode一括生成 |

さらにAdobe Animate本体は[Layer Depth panel](https://helpx.adobe.com/animate/desktop/using/layer-depth.html)として、layerごとの色付きdepth line、drag編集、camera parallax、`Maintain Size`、Alt押下中の一時補正を標準搭載している。よって需要だけでなく、**depthを数値欄ではなく別の視覚軸で編集するUI方向**も複数製品で実証済みである。

共通して現れる機能は、(1)複数layerのZ配布、(2)個別depthの視覚編集、(3)見かけサイズ/構図維持、(4)even/random/reverse、(5)camera setupである。一方、Refresh・expression・null・Bake・precomp修復は表現要件ではなくAEのデータモデルを迂回する費用である。Motoliiは前者を標準toolへ吸収し、後者を生成しないことを優位性とする。

### 採択: Depth Rail / 奥行き展開

Depth Railは一度きりのsetup wizardではなく、現在playheadの評価済みZを表示・編集するlive tool viewとする。

```text
Edit Z:    奥  ←──● BG────● FX──● Character──→  手前
Camera:          3          2          1       (derived rank)
```

- 主railは`position.z`の編集意味を守るためEdit-Space Z。rootではWorld Z、同一parentの子では共通parent Zとし、camera-space depthはderived rank/任意の読み取り専用monitorとして分離する
- mixed-parent選択は一括編集を拒否し、world位置を合わせるためのlocal XYZ書換えや自動reparent/group化を行わない
- 再生・seek・camera animation中もmarker/rank/診断を評価snapshotへ追従させる
- marker drag、range Expand/Compress、Distribute、Reverse、Flattenを通常Z値/keyframeへ直接書く
- Randomize/ExplodeはAdvancedかつ明示seed/有限値診断。確定後は結果値だけを保存する
- groupは1 markerで、子は明示`Edit Children`時だけ表示する。自動group化しない
- Railのviewportは再生中にauto-fitせず、zoom/pan、Fit All/Selection、範囲外indicatorを使う。「動く値、安定した物差し」を守る
- tool/viewport状態はDocument外、transform/keyframeだけをD2 macro 1回で確定する

Edit-Space Zとcamera-space depthを同じrailへ偽装しない理由は、cameraやparentが回転するとcamera前方と編集Z軸が一致しないためである。camera-space railを直接dragしてXYZを同時変更すると、`Depth Move`が`position.z`だけを編集するという契約が壊れる。v1ではEdit-Space Zを編集可能、Camera Depthを派生表示とし、camera前方移動が実需なら別toolとして追加する。

`Preserve Appearance`は現在時刻のscreen-space anchorと見かけサイズを保つ。OFFはZだけ、ONは解析的なXY/Scale補正を同一macroへ明示的に含める。補正値はInspector/HUDへ出し、Altで一時反転できる。pixel readback、expression、controller、将来時刻まで固定するlinkは使わない。これにより現在の2D構図を崩さず奥行きを与え、その後のcamera移動でparallaxを発生させられる。

激しいZ animationではmarkerがrail内を交差・範囲外移動してよい。前後keyframe ghost/trailは派生表示として追加可能だが、過去frameをUI stateとして意味論へ入れない。camera plane/near plane越え、非有限値、soft-alpha遮蔽保証外は警告し、動きを自動clampしない。

### 見かけサイズ変化をScale / Depth Moveへ分ける

perspective cameraでは、XY scaleを増やしても、オブジェクトをcamera側へZ移動しても画面上では大きく見える。この2操作が同じpointer gestureや数値欄に隠れると、ユーザーは「形を大きくした」のか「空間内を手前へ動かした」のかを一目で判別できない。

Canvas transform toolを次の明示モードへ分ける。

| Tool | 永続的に変える値 | Canvas表現 | 変えない値 |
|---|---|---|---|
| `Scale` | `scale.x / scale.y` | bounding boxのcorner/edge handle、Scale icon、tabular差分 | `position.z` |
| `Depth Move` | `position.z` | anchorから伸びるZ rail/axis arrow、`Z` icon、tabular差分 | scale |

追加規律:

- active toolはshape + icon + labelで示し、色だけに依存しない
- Inspector/timelineの`Position Z`と`Scale`は別行・別iconにし、操作中のchannelを示す
- orthographic cameraでもDepth MoveをScaleへ化けさせない。見かけサイズが不変でもZ値と遮蔽結果の変化を表示する
- tool選択はTransient interaction(保存するなら帰属決定前のWorkspace/session候補)で、Document/ジャーナルへ入れない
- 確定したscaleまたはposition.zだけをD2 commandへ渡し、1 drag=1 macro/Undoとする
- Scale gestureからZ command、Depth Move gestureからScale commandを出さない
- direct manipulationと通常keyframeだけで完結し、script/expressionを必須導線にしない

これにより「全オブジェクトは常にZを持つ」という単一世界モデルを、ユーザーが見た目だけでなく操作意味からも追跡できる。

## 判定

- **採用**: 全オブジェクトが常に属する単一XYZ世界、authoring orderの固定、拡張可能なグループ遮蔽ポリシー
- **組み込み**: `Layer Order / Group Depth / AE-style Bins`。通常UIは前2つを`Z Occlusion` OFF/ONとして簡略表示し、Advancedで全方式と明示参加フラグを選べる
- **採用**: Canvas transform toolの`Scale / Depth Move`分離。tool stateはDocument外、確定transformだけを保存
- **採用**: live `Depth Rail / 奥行き展開`。共通parentのEdit-Space Zを編集しCamera Depthは派生表示、通常transform/keyframeへ直接確定し、group/expression/null/Bakeを生成しない
- **棄却**: Z変化に応じたDocument/タイムライン行の自動並べ替え、effect・mask・layer typeから創発する不可視のbin境界
- **縮小**: レイヤー単位Zソートは非交差カード用の明示近似/Draft経路としてのみ扱う
- **M5-P2Dで実装**: `Layer Order`時と同じobject/world/camera表現を使う共有depth参加境界と追加可能なpolicy境界。現行`FrameDesc`へdepth fieldを足す方式は採らない
- **延期**: soft alphaの完全OIT/deep samples。単純depth bufferで解決したことにしない
- **仕様への影響**: C-4を空間モデルではなく`Layer Order`の遮蔽規則へ縮小し、同じ世界へ`Group Depth / AE-style Bins`を追加する。将来方式は既存意味論を変えない追加ポリシーとして扱う
