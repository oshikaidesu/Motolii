# M5: 3D合成とポストプロセス

ステータス: **ドラフト**(凍結ゲートで確定)

## 目的(退治する落とし穴)

C-4(2.5Dとブレンドモードの衝突)、F-1(glTFとの軸整合)、F-6(テキスト基盤の分界)。

## 方針([concept.md](../concept.md)の2.5D/3D合成スコープをC-4の割り切りで具体化)

- **空間は1つだけ**(2026-07-14ユーザー決定)。2D画像、動画、テキスト、図形、glTF、点群を含む全オブジェクトが常に同じ正準XYZ世界、同じ`CompCamera`、同じworld transformを持つ。簡易表示と実depthで座標・アニメーション・親子関係を解釈し直さない
- グループは空間モードではなく、最終的なvisibility resolveだけを選ぶ**拡張可能な遮蔽ポリシー**を持つ。初期の組み込みポリシーは次の3つとする
  - **`Layer Order`(既定・後方互換)**: Zはカメラ投影・視差・見かけスケールに常に効く。遮蔽だけはレイヤー(タイムライン)順で決まり、子同士がZを跨いでも前後反転しない。現行C-4はこのポリシーとして維持する
  - **`Group Depth`**: 座標・camera・transformは`Layer Order`時と同一のまま、グループ内の子を共有depth passへ描き、depth bufferでZ交差・遮蔽を解決する
  - **`AE-style Bins`(Advanced)**: 各子の明示`Depth Participant`を使い、連続する参加レイヤーを同じdepth binとして解決する。不参加レイヤーはauthoring orderで合成され、bin境界を作る。全レイヤーは参加状態によらず同じXYZ世界とZを持つ
- 通常UIの**`Z Occlusion` OFF / ON**は`Layer Order` / `Group Depth`への簡単な入口とする。Advancedを開いた場合だけ3ポリシーと`Depth Participant`を直接選べる。Advanced controlsの表示状態はDocument外、出力を変える遮蔽ポリシーと参加フラグはDocument+D2 commandの対象とする
- 遮蔽ポリシーは**Documentの子順を並べ替えず、visibility resolveだけを切り替える**。Z・カメラがアニメしてもタイムライン行、選択、Undo履歴は動かない
- `AE-style Bins`はAEの隠れた副作用を互換再現しない。effect・mask・layer typeは無言でbinを分断せず、明示参加フラグだけが境界を作る。Timelineはbin境界を常時表示し、Advanced controlsを閉じても適用中のポリシーをbadgeで隠さない
- 初期3方式を閉じた最終enumとはみなさない。将来のDepth-ordered Cards、OIT、素材別queue等は、同じobject/world/camera入力と明示的な能力・診断を使う追加ポリシーとして増やす。未知ポリシーを`Layer Order`へ無言fallbackせず、既存ポリシーの意味も再解釈しない
- Z遮蔽の通常範囲は選択したグループの箱で明示し、子が親の遮蔽境界を無視するescapeは採らない
- `Group Depth / AE-style Bins`は「代表ZでRGBAレイヤーを並べるだけ」ではなく、同じ世界を実在の共有depth passで解決する。soft alphaは単一depth bufferでは完全解にならないため、opaque/cutout/soft alphaの対応意味論を分離し、未対応を無言で近似しない
- v1は依然として汎用3Dシーンエディタではない。ユーザーが触るのは素材配置、M2/M3から常在する共有`CompCamera`、遮蔽ポリシーであり、ライト階層・collection・constraint・複数camera等は持ち込まない
- **軸整合(F-1)**: 世界座標は正準座標系と同じ**Y-up・原点中央**(glTFのY-upと一致し、変換なしで同居)。2Dレイヤーは「Z=0のXY平面に置かれた高さ1.0のクワッド」として世界に置く。正準空間がY-upなので2D↔3Dで軸反転が発生しない(テクスチャのV原点等、ラスター側の上下はmotolii-gpu内部で吸収し、パラメータ空間には漏らさない)
- カメラ文脈はコンポ全体で共有する。M2の`PlanarOrthographic`を既存variantとして保持し、M5でSpatial/Perspectiveを**新variant**として追加する。位置+注視点+暗黙world-upをpose保存へ使わず、orientation補間・clip・target constraint特異点をdecision PRで先に固定する。レイヤーごとのカメラ、カメラ切替、グループ内カメラ、複数ビューは作らない
- ユーザーが直接触る3Dレイヤー側の値は「素材の配置」に限定する。位置/スケール/回転/奥行き/点サイズ等のモーショングラフィック的なパラメータに留め、オブジェクト階層・ライト・コレクション・制約・レンダーレイヤー等のBlender/Nuke的な概念はv1に入れない
- ライティングはunlit(必要になったら固定1灯)
- OBJはglTF変換パスで受ける(内部はglTFのみ)
- 被写界深度はZ距離ベースのポストブラーで代用
- ポストプロセスは通常のRenderNodeとして実装(専用機構を作らない)

## 見かけサイズを変える直接操作: Scale / Depth Move

perspective cameraでは、XY scaleの増減とZ移動のどちらでも画面上の見かけサイズが変わる。AEのように数値欄やscriptへ逃がさず、キャンバスのtransform toolを次の2操作へ明示分離する(2026-07-14ユーザー決定)。

- **Scale**: `scale.x/scale.y`だけを編集し、`position.z`は固定する。キャンバスはbounding boxのcorner/edge handleとScale iconを使う
- **Depth Move**: `position.z`だけを編集し、scaleは固定する。キャンバスはanchorから伸びるZ rail/axis arrow、`Z` icon、tabular数字の現在値/差分を使う
- active toolは形・icon・labelで区別し、色だけに依存しない。timeline/Inspectorでも`Position Z`と`Scale`を別行・別iconで常時識別可能にし、操作中の対象channelを示す
- orthographic cameraではZ移動で見かけサイズは変わらないが、Z座標と遮蔽結果は変わる。Scaleへ自動変換せず、Depth Move toolのままZ railと数値変化を示す
- `Scale / Depth Move`のtool選択はTransient interaction(または保存寿命決定前のWorkspace/session候補)であり、Document・ジャーナルへ保存しない。確定した`scale`または`position.z`だけをD2 commandで書く
- 1 dragは1 macro/Undoとし、Scale gestureからZ command、Depth Move gestureからScale commandを発生させない。pointer軌跡やlogical pxをDocumentへ流さない
- mode切替やdragのために式・script・expressionを要求しない。キーフレームは既存のScale / Position Zパラメータへ直接打つ

## Depth Rail / 奥行き展開

複数レイヤーのZ配置は数値欄、script、制御nullへ追い出さず、CanvasとTimelineに隣接する**Depth Rail**で直接編集する。これは初期セットアップ専用dialogではなく、現在時刻の評価済みdepthを再生中も表示する常設可能なtool viewである。選択レイヤーを「奥行き展開」して簡易parallaxを作る入口と、激しいZ animationを監視・編集する場所を同じ視覚言語へ統一する。

### 表示座標とカメラ

- 編集railの既定軸は**Edit-Space Z**とする。root layerではWorld Z、同じgroup/parentを持つ子ではその共通parent空間のZであり、markerは評価済み`position.z`を示す。dragは同じchannelだけを変更するため、cameraやparentが回転しても別のXYZ channelへ値を密輸しない
- 複数選択のExpand/Distributeは共通parentを持つ場合に限る。mixed-parent選択はworld位置とCamera Depth rankを読めるが、v1では一括Z編集を構造化診断付きで無効にする。world transformを保つためにlocal XYZを裏で書き換えたり、自動reparent/group化したりしない
- 現在cameraから見た前後関係は別の**Camera Depth rank**としてmarkerのgutter/badgeへ表示する。Advancedの`Camera Depth Monitor`はview-space depthの読み取り専用railを追加できるが、Edit-Space Z dragへ偽装しない
- camera前方とEdit-Space Zがほぼ直交し、Z移動が画面上の前後移動にならない場合は診断を出す。camera-space方向へ動かしたい操作は将来の別toolであり、`Depth Move(Z)`の意味を変えない
- camera plane、near/far、cameraより後方、`Group Depth / AE-style Bins`での実効depth交差を形とlabelで示し、色だけへ依存しない

### 編集操作

- 単一marker drag: そのレイヤーのZを編集。Shiftで精密操作、double clickで正準値を直接入力
- 複数選択range handle: 選択集合の中心を保ったまま既存Z間隔を`Expand / Compress`する。選択数が増えても全体spanが勝手に増えない
- `Distribute`: 選択範囲のnear/far端を保って等間隔化。`Reverse`: Z値の集合を保ったまま割当順だけ反転。`Flatten`: 選択中心の同一Zへ戻す
- `Randomize / Explode`はAdvanced。Randomizeは明示seedでpreviewを再現し、確定時は通常Z値だけをDocumentへ書く。Explodeは選択中心から符号付き距離を拡大し、無限値・camera near plane越えを診断する
- groupを選択した場合はgroup自身を1 markerとして扱う。子を編集するにはTimelineでgroupへ入るか`Edit Children`を明示し、自動group化・暗黙の子展開を行わない
- pointer downからupまでをlive previewし、確定はD2 macro 1回、Cancelは変更ゼロとする。Auto Key有効時は現在時刻へ通常のZ keyframeを作り、無効時は既存transform編集規則に従う。Depth Rail独自のanimation channelを作らない

### Preserve Appearance

parallaxの初期配置では、Zを配った瞬間に元の2D構図が崩れないことが重要である。Depth Railには`Preserve Appearance`を置き、perspective cameraでdrag/Distributeした現在時刻の**screen-space anchor位置と見かけサイズ**を維持する補正を明示的に選べるようにする。

- OFFでは`position.z`だけを変更する。ONでは必要な`position.x/y`と`scale.x/y`補正も同じD2 macroへ含め、HUDとInspectorで補正channelを表示する
- 補正は現在時刻の`CompCamera`と変形から解析的に求め、pixel readbackや前フレーム状態を使わない。orthographic cameraでは補正不要で、Zだけを変更する
- Alt押下中だけ一時的にON/OFFを反転できる。確定後にexpression、controller null、隠れたlink、再登録情報を残さず、通常のtransform値/keyframeとして編集可能にする
- cameraやlayerに既存animationがある場合も、現在時刻の見た目を基準に補正する。将来時刻まで見た目を固定するexpressionは作らず、その後のcamera移動でparallaxが生じる

### 再生中の動的表示と範囲

- marker、Camera Depth rank、現在値、遮蔽診断は評価snapshotの時刻に追従して更新する。GPU readbackを行わず、M3の非blocking最新値mailbox+generation破棄を使ってUI threadを待たせない
- **動くのは値、物差しは安定**を原則とし、再生中にrailのzoom/panを自動fitしない。範囲外markerは端の方向indicatorとして残す
- wheel/pinch=`zoom`、pan gesture=`pan`、`Fit All`、`Fit Selection`、任意の`Follow Selection`を提供する。Follow中もscaleを連続変更せず、選択がsafe marginを越えたときだけpanする
- v1の編集scaleは線形だけとする。極端なdepthを圧縮する非線形表示はdrag量の意味を曖昧にするため、必要性をfixtureで確認するまで追加しない
- tool選択、railのzoom/pan、Follow、表示filterはTransientまたはWorkspace/session候補でDocument外。確定したtransform/keyframeだけをDocumentへ保存する

Depth Railは配置を担当し、遮蔽規則を変更しない。Zを交差させても`Layer Order`なら重なりはauthoring orderのまま、`Group Depth / AE-style Bins`なら各ポリシーの規則で遮蔽する。

## カメラ文脈とLayerSourcePluginの分界

object/material/generatorの追加はpluginへ開くが、world/camera/depth参加境界はHostが所有する。コアを汎用3Dアプリ化せず、2Dモーショングラフィックの操作感を保つため、v1では次の分界を守る。

- コア(Document/Render): `CompCamera`、単一の正準XYZ世界、全オブジェクトのworld transform、グループの遮蔽ポリシー、レイヤー順、グループ/マスク/エフェクトの評価順、Quality、出力`FrameDesc`を所有する
- `Layer Order`: LayerSourcePluginは`t`、`Quality`、`CompCamera`、同じworld transform、出力`FrameDesc`を受け取り、GPU上でRGBAテクスチャへ描く。他objectとの遮蔽だけを通常Compositeで決める。これは既存RGBA契約を保つ経路であり、全ポリシーの唯一のobject表現ではない
- `Group Depth` / `AE-style Bins`: flatな`LayerSourcePlugin → RGBA`へdepthを密輸せず、P2Dで同じオブジェクト表現を共有depth passへ参加させる境界を定義する。2D平面・glTF・点群が、`Layer Order`時と同じ座標のまま参加できることを最小要件とする
- プラグインは決定論的であること。`render_frame(t, Quality)`の入力から同じ出力を返し、**レンダ系traitは**前フレーム状態に依存するシミュレーションを持たない(逐次シミュレーションはレンダ経路の外のベイク境界=SimulationPlugin+StateTrackで扱う。[simulation-model.md](../simulation-model.md)、2026-07-10改訂)
- 全遮蔽ポリシーは同じ`render_frame(t, Quality)`から評価し、preview/exportで別経路を作らない。選択値も隠れたruntime stateではなくDocument上の明示値として扱う

## 動的シミュレーション(物理演算)について — M5固有の補足(2026-07-07 決定 → **2026-07-10 改訂: 「レンダ経路ではやらない」に縮小**)

> **改訂(2026-07-10)**: 本節の決定は「決定論」と「フレーム独立性」を一括りにしていた。固定シード+固定タイムステップの逐次シミュレーションは完全に決定論的であり、失われるのはランダムアクセス性のみ — それは区間キャッシュ+チェックポイント(ベイク)で回収できる。よって決定を次の通り縮小改訂する: **「コアのレンダ経路(`render_frame(t)`と全render系trait)は物理シミュレーションを持たない」は不変**。一方、**逐次シミュレーション自体は、レンダ経路の外のホスト管理ベイク境界(`SimulationPlugin`+StateTrack)を通じて一級のプラグイン種別として設計に含める**。全体設計・成立性試算・段階導入は[simulation-model.md](../simulation-model.md)。以下の本文は改訂前の記録として残す(下記「決定」の1と3の読み替えは改訂注を参照)。

**正本は[concept.md](../concept.md)の根本コンセプト「馬鹿正直にシミュレートしない」(第一選択は常にf(t)の安い力)と、横断決定「逐次依存シミュレーションをレンダ経路に持ち込まない」**(いずれも2026-07-10)。[B-5](../pitfalls-and-roadmap.md)・[M4](M4-cache-and-analysis.md)と同じ2D合成パイプライン上の制約である。ここでは3Dレイヤー文脈だけ補足する。

- **3Dは「別エンジン」ではない**: `Layer Order`は既存LayerSource→RGBAを使い、共有遮蔽は同じworld/cameraからHostのobject参加境界へ接続し、最終的にRGBAへ合流する。Blenderの布/液体/パーティクル/ソフトボディの例は「外部ツールが重い理由」の**たとえ**であり、「3D物理だけ特別に禁じる」境界線ではない。2Dパーティクル・2D剛体・オプティカルフローも同じく逐次依存として扱う(要るならレンダ外のベイク境界へ)
- Blender等が重いのは、液体・布・パーティクル・ソフトボディといった**フレーム間の状態に依存する物理シミュレーション**を毎フレーム解いているため。これは本質的に「前フレームの結果を入力にする逐次計算」で、並列化もキャッシュも効きにくく、GPUの得意分野から外れる
- 一方、MVで使う3Dの大半は**メッシュを配置してカメラを動かす/決定論的なアニメーション(キーフレーム変形・回転)**であり、これは「時刻t → 頂点位置」の純関数で表せる。前フレーム依存がない = 任意フレームを独立に・並列に・キャッシュ可能に計算できる

### 決定

1. **コアは物理シミュレーションを持たない**。3Dは「時刻t → シーン状態」が純関数で決まるもの(静的メッシュ、キーフレーム/手続き変形、剛体的トランスフォーム)に限定する。これはレンダラ全体の「`render_frame(t)`は決定論的な純関数」という大原則(B-4)と完全に一致する
   - **改訂注(2026-07-10)**: 「コア=レンダ経路」と読み替える。シミュレーションはレンダ経路の外(ベイク相)で走り、`render_frame(t)`はベイク結果(StateTrack)を読む純関数のまま — 原則B-4は不変([simulation-model.md](../simulation-model.md)§3)
2. **動的シミュレーションが欲しい場合は"ベイク済み"で持ち込む**。他ツール(Blender等)でシミュレーションし、頂点アニメーション付きglTF/Alembic的な**焼き込み済みシーケンス**として読み込む。うちはそれを再生するだけ(逐次計算しない)。「シミュレーションはインポート前に終わっている」を原則にする
   - **改訂注(2026-07-10)**: このルートは映画級の重いシミュレーション(高解像度流体・破壊等)の推奨経路として存続する。v1の唯一の経路でもある(SimulationPluginの実装はv1.x)
3. ~~仮に将来プロシージャルな動きを足すとしても、前フレーム非依存(時刻の純関数)なものだけを許可する。前フレーム状態を積む本物のシミュレーションはプラグインにも入れない~~ → **改訂(2026-07-10)**: 前フレーム状態を積むシミュレーションを**ホスト管理のベイク境界(`SimulationPlugin`+StateTrack、キャッシュキーはM4-K1と同一の枠)として設計に含める**。キャッシュ設計を壊すのは「隠れ状態」であって「状態」そのものではない — 状態をホストが所有・チェックポイント保存すれば、キャッシュ/スクラブ/並列書き出し/タイムリマップと全部両立する。tの純関数で書けるもの(レベル0/1)を優先する原則は残る([simulation-model.md](../simulation-model.md)§2)

この判断により、3DはM4のキャッシュ機構(ノードID×時間区間×パラメータハッシュ)とグループ仮出力(ベイク)に**そのまま乗る**。「プレビューでは多少カクついてよい、キャッシュで滑らかにする」という要望は、3DレイヤーをDraft品質(低ポリ/低解像度)で回し、確定したら区間ベイクする、という既存の仕組みで実現できる(3D専用の特別扱いが不要)。

## タスク分割(粗案)

### 操作単純化モデルへの割当

M5は[操作単純化モデル](../interaction-simplicity-model.md)の最初の大規模実地審判である。P2Uは`Scale / Depth Move`を別channelのDirect操作へ、P2Rは大量Z編集を通常transformのToolへ、P2DはSimple `Z Occlusion`とAdvanced policyを同じDocument意味へ正規化する。いずれも隠れgroup/null/expression/controller/Bakeを生成しない。P5では完成画だけでなく「3D背景へ2Dキャラを配置→前後関係を調整→Advancedで意味を確認→Undoで復元」の操作記録を残し、入口開閉でDocumentと画が変わらないことを確認する。

| ID | 内容 | 依存 | 完了条件(概要) |
|---|---|---|---|
| P0I | [#170](https://github.com/oshikaidesu/Motolii/issues/170) **Cavalry型Instance/Behaviour境界spike+意味凍結**: Input Shapes、Distribution、per-instance channels、nested Contextを最小fixtureで再現し、`InstanceId != index`とdomain別Selectorを固定する。製品schema/APIはまだ追加しない | 凍結ゲート, [2026-07-15決定](../reviews/2026-07-15-relative-scope-duplicator-decision.md) | (1)Linear/Grid/Radial/Pathのslot key表 (2)count増減/並べ替え後も残存InstanceId由来seedとmotion sampleが同identityへ追従 (3)nested親子ID/context depth (4)TextCluster/Word/Line、ShapePath、CloneInstanceの寿命差 (5)Position/Rotation/Scale/Visibility/Opacity/Prototype/TimeOffset channel型 (6)cache依存完全列挙 (7)PCG32実装名/versionと`hash(user_seed,id,channel)`golden (8)OS entropy/時計/thread/GPU順を乱数入力にしない |
| P1 | glTF読み込み(メッシュ・マテリアル最小限)+ OBJ→glTF変換パス | 凍結ゲート | サンプルアセットの読み込みゴールデンテスト |
| P2 | 3D系LayerSourcePlugin境界: 既存`PlanarOrthographic`を参照し、メッシュ/点群/動画テクスチャ平面をpremultiplied RGBAへレンダ(レイヤー単位に1パス) | P1, M2-D1j/D1k/D3f | 正投影ではZ差だけで視差が生じないこと、plugin内部Z解決後のRGBAが通常Compositeで2D layerと合成されること、3D未使用compのpixel不変をgolden固定 |
| P3 | `CompCameraDoc::Spatial`追加variantのdecision/schema/runtime統合。orientation補間・handedness/軸・projection/clip・target constraint特異点・Planar切替を先に固定 | P2 | Spatial/PerspectiveでZを持つ平面がcamera移動により視差するgolden、camera animation E2E、既存Planar project/pixel不変、preview/export同一、レイヤーごとのcameraが構文不能 |
| P2D | 拡張可能な遮蔽ポリシー境界+組み込み`Layer Order / Group Depth / AE-style Bins`。通常UIは`Z Occlusion` OFF/ON、Advancedは全ポリシーと明示`Depth Participant`を表示 | P1, P3, M2-D1j/D1k/D3f, M3-U2c, M3-U4c, M4-K0 | (1)同じ座標の2平面が`Layer Order`ではレイヤー順、`Group Depth`ではZ交差で前後反転 (2)3D-2D-3D fixtureで明示参加フラグだけがbinを分断 (3)effect/mask/typeは無言でbinを変えない (4)切替前後で座標/見かけ投影/Document子順/Undo/選択不変 (5)group外はピクセル不変 (6)opaque/cutout/soft alphaの対応・拒否が明示 (7)未知/非対応ポリシーを無言fallbackしない (8)Advanced非表示でも適用ポリシーとbin境界を識別可能 (9)Unknown boundsをsort/cull根拠にせずFinalを切らない |
| P2U | Stage transform toolの`Scale / Depth Move`分離。Scale handleとZ rail/axisをM3-U2dのCamera/Object操作、既存toolbar/Inspector/timeline語彙へ統合 | P3, M2-D2, M3-U2d | (1)Scale dragはscaleだけ、Depth dragはposition.zだけをD2 command化 (2)各gestureがUndo 1回 (3)perspective/orthographic fixture (4)active toolを文字なし・grayscaleでも形/iconから識別 (5)DPI差で同じ正規化gestureが同じdomain値 (6)script不要で両channelへkeyframe可能 |
| P2R | Depth Rail / 奥行き展開: 複数レイヤーのlive Z表示・直接編集・Expand/Compress/Distribute/Reverse/Flatten・Preserve Appearance | P2U, P3, M2-D2, M3 | (1)再生/seekで評価済みEdit-Space Z markerとCamera Depth rankが追従 (2)rail viewportは再生中不変、範囲外indicator+Fit操作 (3)single/range dragと各actionがD2 macro 1回、Cancel変更ゼロ (4)Auto Key ON/OFF fixture (5)Preserve OFFはZのみ、ONはscreen anchor/sizeを維持し補正channelを可視化 (6)perspective/orthographic/rotated camera/common-parent/mixed-parent拒否fixture (7)100 layer/極端ZでUI非blocking・readbackなし (8)group/null/expression/専用channelを生成しない (9)occlusion policy別の同一Z配置fixture (10)Randomizeの同seed同結果・Explodeの有限値/near-plane診断 |
| P4 | ポストプロセスNode群: ブラー(+Z距離マスク)、色調整(リフト/ガンマ/ゲイン)、グレイン | 凍結ゲート, M4-K0 | 各ノードのゴールデンテスト。Blurのinput regionが半径分拡張し、Unknown fallback/全域評価とpixel一致 |
| P5 | 統合検証: 動画平面 + glTFメッシュ + Duplicator/Stagger/Random/Falloff + DataTrack/ParamDriver駆動オーバーレイ + ポストを1シーンで書き出し。実動画トラッキングを使う場合はM4の解析プラグイン完了後に接続する | P2R, P3, P4, P7c, P7U | E2Eゴールデンテスト + 実素材でのショット制作。同seedのpreview/export/再起動がinstance単位で一致 |
| P6 | motolii-text: 最小テキスト基盤(F-6)。スタック=fontique+harfrust+Vello `draw_glyphs`。**APIは一発`draw_text`ではなくラン単位の純関数3点**(下記)。組版(縦書き・ルビ・行組・歌詞タイミング)はプラグイン側 | 凍結ゲート | (1)かな漢字混在の shape→draw ゴールデン (2)フォールバック解決テスト (3)variations を変えると送り幅が変わるテスト (4)クラスタ対応表で「N文字目のグリフ範囲」が取れるテスト |
| P7a | **Duplicator Document schema+追加migration**: Input Shape参照列、Distribution、明示`user_seed:u64`、Instance channel recipe、Behaviour順序をP0Iの決定型で保存。生成instance列やderived boundsは保存しない | P0I, M2-D1e, GR-PV | (1)旧project追加migration冪等 (2)typed source ref/循環/欠落/非有限拒否 (3)同seed roundtrip (4)unknown Behaviour保持 (5)instanceや乱数結果のJSON焼込みなし (6)旧reader拒否 (7)意味論goldenを書換えず新variantで拡張 |
| P7b | **Host Duplicator評価+GPU instance**: Distributionからstable slot key/InstanceId/Contextを生成し、Input Shapesをinstance列へ評価。Timeline rowを増やさない | P7a, P2, M4-K1 | (1)1,000 instanceで1,000 layer/textureを生成しない (2)同Document/t/input/seedでbit同一instance metadata (3)count増減で残存ID/乱数不変 (4)Grid列数変更で座標slot identity規約一致 (5)nested context depth (6)2D/3D/Depth policyでCloner再生成なし (7)UI thread readbackなし (8)preview/export同一関数 |
| P7c | **first-party Behaviour**: Stagger、Random、Falloffを純関数`Behaviour(InstanceContext,t,params)->typed channel value/weight`として実装 | P7b | (1)Stagger=index/count順序、Random=`pcg32(hash(user_seed,InstanceId,channel_tag))`、Falloff=正準world距離 (2)Behaviour順序/enableで結果が決定 (3)Randomizeはseedを書換えるD2 commandだけ (4)再生/seek/thread順で乱数不変 (5)型不一致をtyped error (6)effect未使用時既存pixel同一 |
| P7U | **Duplicator/Behaviour UI**: Input Shapes接続、Distribution、seed、per-instance channel、Stagger/Random/FalloffをTimeline/Inspector/Stageへ接続 | P7c, M3-U2c, M3-U2g | (1)source/Behaviour接続をfrom/inで可視化 (2)seed数値編集+明示Randomize、再評価で勝手に変化しない (3)instance選択はderived UI状態でDocumentへ1,000行を焼かない (4)Direct/Tool/Advancedが同Document意味 (5)1 gesture=1 Undo (6)1,000 instanceでUI非blocking |

並列レーン: M2-D1j/D1k/D3とM3-U1f/U2dで2D world/camera/Stageを先に成立させ、K0は透過Stageと独立して進める。P0IとP1 importも互いおよびK0から独立し、P1合流後にP2。P0I→P7a、P2+P7a+K1→P7b→P7c→P7U。P2後はP2D/P2U/P3を依存に従って進め、P2U+P3後にP2R。P4はK0後、P6は独立、P5が合流点。world/cameraの成立をP1へ依存させない。

### P6 API契約(2026-07-10。プラグインが組版できるための口)

一発 `draw_text(文字列, フォント, サイズ)` で切ると variations とクラスタ対応が外に出ず、歌詞プラグインがシェーピング自作に戻る(F-6境界事故)。コアは次の**ラン単位・純関数**だけを公開する:

1. **itemize + fallback**: `text → [(範囲, フォント)]`  
   フォント混在・欠字フォールバック・スクリプト分割(itemization)の口。
2. **shape**: `shape(ラン, フォント, 方向, 言語, 軸座標, フィーチャ) → グリフ列 + 送り(主軸advance+交差軸offset) + クラスタ対応表`  
   - **軸座標(variations)は入力必須** — wght等が変わると幅・カーニングが変わるため、描画時だけ渡しても足りない。毎フレーム再シェーピングが正規。  
   - **クラスタ対応表**(何文字目→何グリフ目)は出力必須 — 「指定文字だけ大きく」「文字ごとに踊らせる」でランを割る位置の生命線。  
   - シェーピングはフォント単位系で**サイズ非依存**。サイズ差はプラグインがラン分割＋`font_size`違いで描く。  
   - ユーザー空間の軸値(例: wght 400..700)→正規化座標(-1..1、avar込み)の変換は**コア責務**(skrifa)。
   - **追記(2026-07-12、縦書き先例調査の判定反映 — 未着工の今なら費用≈0の文言3点)**:
     (1) 出力の送りは**方向中立の2D語彙**(主軸advance+交差軸offset)で定義し、出力型は`#[non_exhaustive]`【C-1: 採用・文言のみ。縦ゴールデンは足さない】
     (2) 入力の**OpenTypeフィーチャリストは既定空・コアは意味を解釈せず透過のみ** — 縦中横用`hwid`/`twid`/`qwid`や`vpal`/`vkrn`/`vrtr`等は呼び出し側指定【C-2: 弱い採用。唯一プラグイン側で吸収不能な口】
     (3) **itemize結果の部分範囲を、呼び出し側が任意に再分割してshapeしてよい**(スタイルスパン境界・UAX#50向き境界等での再分割を契約として保証)【C-3: 採用・文言のみ】
     判定の根拠は[調査メモ](../reviews/2026-07-12-vertical-text-prior-art.md)と[反対側レビュー](../reviews/2026-07-12-vertical-text-prior-art-counter-review.md)を必ず併読([規律6](../reviews/README.md))。
3. **draw**: `グリフラン + サイズ + 軸座標 → Vello draw_glyphs`  
   `normalized_coords` / `font_size` / `glyph_transform` を貫通。軸アニメは既存キー/イージング/DataTrackの f32 パラメータとして載る。

**キャッシュ**: シェーピング結果のキーに軸値を入れると連続アニメ中は毎フレームミスする。**軸アニメ経路ではシェーピング結果をキャッシュしない(毎回計算)**を正規とする(1行数十文字は µs 級。`ShaperInstance`/`ShapePlan` の再利用で軽くする)。M4のキャッシュキー完全性原則と矛盾させないこと。

プラグイン側で組める例: 複数行(UAX#14)、フォント混在(itemize結果ごと shape)、文字ごとサイズ(ラン分割)、軸パラメータ推移(毎フレーム shape→draw)、文字ごと変形(`glyph_transform`)。

## 実装ガード(先行ツールの失敗・ユーザー不満クロスチェック 2026-07-11)

AEの3世代の3D(Ray-traced CS6 / Cineware / Advanced 3D)の死因、glTF実装の実害、RustのCJKテキストスタックの現在地、ポスト系の定番苦情を調査し、既存方針(C-4割り切り・F-6分界・P6契約)に無いガードを抽出した。マクロな教訓: **AE三世代の3Dの死因は絵作り能力の不足ではなく「2Dパイプラインとの統合品質」**(プレビュー速度・既存エフェクト互換・GPU検出・プロジェクト互換)。本プロジェクトでは世界を分けず、Z遮蔽OFFを安全な既定に保ち、ONの影響を明示グループへ閉じることがこの死因を構造的に避ける生命線になる。

1. **「3D不使用コンポは3Dバックエンド有無でピクセル同一」ゴールデン**: AE Advanced 3Dはレンダラー切替がコンポ全体のセマンティクスを変え、既存3D系エフェクトとの互換を壊して「Classic 3Dに戻せ」が定番回避策になった。CS6 Ray-tracedは廃止時に旧プロジェクトを道連れにした(NVIDIA固有実装で移行パスなし — F-9のベンダーAPI排除の傍証)。3D系LayerSourcePluginの導入が2D経路の出力に一切影響しないことをゴールデンで固定する → P2完了条件に追加
2. **GPU能力不足は機能単位のフォールバック**: AEの「Advanced 3D is not supported by the current hardware」全体エラー・Intel iGPU誤選択の苦情に対し、能力不足時は3Dレイヤーのみプレースホルダ化して他は動かす(コンポ全体をエラーにしない)。アダプタ選択は明示ログ+設定可能に → P2
3. **ベクター/テキストのラスタライズ解像度は射影後のスクリーンフットプリントから決める**: AEの「3D化した途端テキストがボケる」(continuously rasterizeでも直らないケースあり)は、ラスタライズ解像度がレイヤー座標系で決まり射影後の画素密度と一致しないことが根因。2.5D(Zは射影のみ)はこの罠の隣にいる。テスト: 「Z≠0でも見かけサイズが同じならZ=0と同一シャープネス(SSIM閾値)」 → P2
4. **メッシュ描画はリニア、コンポ空間への合流点は単一の明示変換**: three.js/A-Frameの「GLBが暗い/色褪せる」最頻出苦情はモデルでなく出力エンコーディング(linear→sRGB)とトーンマップの契約不一致が原因。色変換一元化(絶対規律2)の3D版として「PBR系計算はリニア、合流点の変換は1箇所」を明文化し、Khronos glTF-Sample-Assetsを基準レンダラ(Blender)参照画像とのゴールデン比較に組み込む → P1/P2。未決事項「3Dパスの中間フォーマット」はこの調査を踏まえ**リニアFP16を推奨案**とする
5. **インポート診断の可視化(無言のフォールバック禁止)**: glTFの苦情は「読めない」より「読めたのに違って見える」が支配的(テクスチャが黙って落ちる/全部プラスチックに見える)。一方`extensionsRequired`未対応は原因不明の即死になる。未対応拡張は「どの拡張が原因か」を示す構造化エラーで拒否し、欠落テクスチャ・未対応マテリアル機能はインポート診断として一覧可視化する(F-9の未知プラグインID警告+パススルーと同じ思想のアセット版) → P1
6. **単位・軸・スケールは頂点へ焼き込む**: エクスポータ方言(Blenderの「+Y Up」トグルでの軸反転、巨大/極小スケール流入でタンジェント破壊)への対策として、インポート正規化で単位・軸を頂点データへ焼き、シーングラフ最上位に補正トランスフォームを残さない(Godotがufbx移行で採った方式)。テスト: 同一アセットを異なるup-axis/単位設定でエクスポートした複数ファイルが同一レンダになる → P1
7. **スキンメッシュ/リターゲットのスコープ宣言**: Godotはリターゲット(レストポーズ規約がバラバラの外部資産)でサポート地獄に入った。v1は静的メッシュ+ノードTRSアニメのみ。将来スキニングを足す場合も「オーサリング済みクリップの再生のみ・リターゲット永久非対応」を明文化し、「Blenderでベイクして持ち込む」を公式導線とする(方針「ベイク済みで持ち込む」と整合) → P1のスコープ外に明記
8. **テキスト: 同梱フォントの下限保証+シェーピング差し替え可能性**: RustのCJKスタックには実害が現存する(fontique経由でmacOSのCJKフォントが列挙されず日本語が真っ白 = linebender/xilem#1358、cosmic-text系のCJK重なり・フォールバック不発、rustybuzz系のAAT相違)。(a) Noto Sans CJKを同梱し「同梱フォントだけで日本語が完全描画できる」を下限保証、(b) シェーピングをtrait境界の裏に置きharfbuzz FFIへ差し替え可能な形を保つ、(c) 3OSの実フォント環境でCJK混植スナップショット+**豆腐発生時は該当コードポイントを診断出力**(Resolveの豆腐相談の大半はフォント起因 — 診断が無いとユーザーは原因に到達できない) → P6完了条件に追加
9. **縦書きは「回転」で実装しない**: AEの縦書きは約物(「」、。ー)が逆さ・縦中横は手動という品質で、プロはIllustratorで組んで画像で持ち込むのが定番運用になっている。やるならUAX#50の向き分類+`vert`適用を組版として実装、v1でやらないなら「横書きのみ」を明示スコープ宣言する。**中途半端な回転実装を出すのが最悪**(未決事項に追加)
10. **カラオケワイプ・ルビ・文字別タイミングは歌詞プラグイン第1号の一級要件**: AviUtl圏の実需は「素の機能では1曲1日」(音節ごとのオブジェクト複製+クリッピング手調整+ルビ用別オブジェクト)で、個人スクリプトへの全面依存が現状(保守は作者の善意頼み)。AE/Resolveも両方この領域を落としており差別化点。P6のクラスタ対応表はこのための生命線 — 第1号プラグインの要件定義に「ワイプ(クラスタ単位の時刻オフセット)・ルビ(ベーステキストとの対応区間を持つ注釈ラン)・フルコーラス分の性能予算」を含め、P6契約の実地検証を兼ねる
11. **8bit量子化はエンコード直前の単一ステージ+既定ディザON**: AEは深度変換の瞬間しかディザせず、「16bpcにしたら逆にバンディングが見える」という直感に反する挙動で民間療法(Add Grain 0.3)が10年流通した。量子化点を1箇所に固定し既定でディザ(blue-noise等)。テスト: 黒→暗灰のグローランプを8bit出力してヒストグラムに階調プラトーが無いこと → P4
12. **モーションブラーのpreview==final**: AE Pixel Motion Blurの最悪の苦情は「プレビューでは正常、最終レンダだけブロックノイズ」(推定ベースのMBがプレビューと最終でパス分岐)。`Quality.effect_samples`でサンプル数を変える場合も**既定はpreview==final(同一サンプル数)**とし、品質を落とす時は明示表示(B-4の約束のプラグイン版)。プラグインへ供給する解析的モーションベクトル(変換由来=正確)のAPIを先に用意し、オプティカルフロー推定型は最後 — 「モーションブラーはプラグイン領域」の決定と整合
13. **世界は1つ、遮蔽ポリシーだけを明示切替**: 全オブジェクトは常に同じXYZ世界へ属する。通常UIは`Z Occlusion` OFF=`Layer Order` / ON=`Group Depth`、Advancedは`AE-style Bins`も選べる。AE式でも明示`Depth Participant`だけが隣接binを作り、切替で座標解釈・Document子順・Undo履歴・選択行を変えない。非対応のfilter/mask/object/policyは構造化診断し、別ポリシーへ無言fallbackしない → P2D
14. **見かけサイズ変化の原因をScale / Zで識別可能にする**: perspective上で同じ拡大に見えても、Scale toolはscaleだけ、Depth Move toolはposition.zだけを変更する。bounding handleとZ rail/axisを形/icon/labelで分け、色だけ・数値欄だけ・scriptだけへ意味を隠さない。tool選択はDocument外、確定transformだけをD2 commandへ流し、1 gesture=1履歴を守る → P2U
15. **選択肢を増やせるが、意味を上書きしない**: 初期3方式は組み込みプリセットであり最終的な閉集合ではない。新方式は同じworld/object/camera入力を使う追加ポリシーとして登録し、安定ID・version・能力・alpha保証・fallback可否を宣言する。公開trait/永続schemaの形はP2D実機spike前に発明しない。Advanced controlsの開閉はDocument外、出力を変える選択だけをDocumentへ置く → P2D
16. **Depth Railをsetup scriptにしない**: AE圏では同じ需要がexpression、controller null、Refresh、Bake、precomp修復として何度も再実装された。Motoliiは評価済みEdit-Space Zをlive表示し、通常transformをD2 macroで編集する。自動group化、mixed-parentの裏XYZ補正、隠れlink、専用animation channel、再生中のauto-fit、GPU readbackを禁止し、Preserve Appearanceの補正channelも可視化する → P2R

出典: community.adobe.com(Ray-traced廃止/Cineware激遅/Advanced 3Dハードウェアエラー/Pixel Motion Blur) / creativecow.net(3Dテキストぼけ/16bitバンディング) / discourse.threejs.org(GLBが暗い) / projects.blender.org #118319(+Y Up軸反転) / godotengine.org(ufbx頂点焼き込み)・godotengine/godot#89244(リターゲット) / linebender/xilem#1358 / pop-os/cosmic-term#325 / crft.jetsets.jp(AE縦組みの約物) / aketama.work・note.com(AviUtlカラオケ字幕の実態)

## 未決事項

- 3Dパスの中間フォーマット(HDRリニアで持つか、レイヤーごとに8bit確定か)。**実装ガード4の調査によりリニアFP16が推奨案**
- ~~縦書き対応の時期とスコープ(v1は横書きのみか)~~ → **判定(2026-07-12): v1は横書きのみ(縦書き延期)+ガード9据え置き**(回転ベースの反面事例2件=resvg・Flutter公式回答を調査で追加確認)。P6契約へは費用≈0の文言3点のみ先行反映(C-1/C-2/C-3、上記追記)。**C-4**(縦行送りメトリクス口)/**C-5**(縦メトリクス欠落診断)は延期=縦書き着手時の完了条件へ、**C-6**(sideways描画材料)は棄却=`glyph_transform`貫通で現契約に既にある、**C-7**(fallback縦適性)は論点記録のみ=実質の緩和策はガード8(a)同梱フォント下限保証。併読: [調査メモ](../reviews/2026-07-12-vertical-text-prior-art.md)・[反対側レビュー](../reviews/2026-07-12-vertical-text-prior-art-counter-review.md)
- ~~モーションブラーはスコープ外のままでよいか~~ → 決定(2026-07-07): プラグイン領域。Quality型にサンプル数の口だけ確保
- ~~テキストP6の一発API vs ラン単位~~ → **決定(2026-07-10)**: ラン単位3点セット(上記)
