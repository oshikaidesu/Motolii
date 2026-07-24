# Vismプラグインカタログ — 将来表現を一覧から計画する

作成日: 2026-07-24

状態: **計画台帳**。将来実装を見通すための一覧正本であり、個々の採択順、公開API、parameter、Vism package／entry identity、`.vism`形式の実装許可ではない。

関連正本: [Vismコンセプト](vism-package-concept.md)、[Vism / Kitモデル](vism-kit-model.md)、[Vism実装計画](reviews/2026-07-17-vism-implementation-plan.md)、[first-party表現需要調査](reviews/2026-07-23-first-party-vism-expression-demand-survey.md)、[プラグイン作者向け規約](plugin-authoring.md)、[時間軸の自由度モデル](simulation-model.md)

## 1. この一覧の役割

利用者が将来Plugin Browserで見る粒度に近い形で、Motoliiが持つ／持ちたい表現を一か所から探せるようにする。

- Text animation、pixel effect、particle、data-driven motionを別々の一覧へ分断しない。
- 利用者が探す「Glow」「Random Entrance」「Pixel Sort」等を主語にし、`Filter`や`LayerSource`等の内部実行分類は実装欄へ置く。
- 実装済み参照pluginと将来候補を同じ一覧で見せ、状態を混同しない。
- 候補ごとに、現在の公開境界だけで作れるか、どの共通能力を先に閉じるかを示す。
- 外部製品名や過去資料の表現名は候補発見にだけ使い、Motoliiの公開ID、parameter、永続意味を逆算しない。

この文書のカテゴリは閲覧用である。製品Browserの`Effects / Create`等のタブ構成や、1 package内のentry数を確定しない。

## 2. 表示記号

### 状態

| 状態 | 意味 |
|---|---|
| **同梱済み** | first-party catalog／runtimeへ登録された参照plugin。`.vism`動的配布済みという意味ではない |
| **計画決定** | 将来のfirst-party表現として置くことは決定済み。実装開始には依存gateと個別仕様が必要 |
| **候補** | 需要調査から一覧へ置いた比較対象。採択順、parameter、entry粒度は未決 |
| **Gate待ち** | 表現需要はあるが、共通能力または意味審判が閉じるまで実装しない |
| **履歴照合待ち** | 現行正本にも名前はあるが、過去Gitの候補群との照合が未完了 |

### 実装lane

| lane | 実装見通し | 主な停止線 |
|---|---|---|
| **READY** | 現行の公開plugin境界だけで実装・試験済み | first-party特権を追加しない |
| **SINGLE** | 現在frameだけを読む単一pass Filter／LayerSource／ParamDriver | VRAM常駐、決定的seed、正準座標、色変換を混ぜない |
| **PORTS** | mask、field、structured data等の型付き入力を受ける | 具体plugin ID検索、生JSON、opaque ID分岐で代替しない |
| **MULTIPASS** | Host所有の一時textureと複数passを使う | VSM-A8G0〜G1前に専用API、自前pool、loop内resource生成をしない |
| **BAKE** | 重い解析／伝播結果を編集時jobで決定的artifactへ確定し、renderでは読むだけにする | 同期CPU処理、plugin内cache、前frame出力への隠れ依存で代替しない |
| **TEXT** | 書記素identityとText Sequence／Effector評価へ投影する | 専用`TextAnimatorPlugin`、二重identity、Text内の隠れstateを作らない |
| **TEMPORAL** | 宣言した前後frame窓を読むL2表現 | Filterへ前frameを隠さず、`TemporalFootprint`解凍前に実装しない |
| **SIM** | Host所有StateTrackへベイクするL3表現 | render traitの`&self`へ状態を隠さず、固定step／seed／無効化をHostが所有 |
| **KIT** | 複数Vismとproviderを型付き接続してProjectへmaterializeする | 具体provider直参照、失敗時の部分commit、linked updateを先行実装しない |

`SINGLE`で成立する表現を`SIM`へ上げない。反対に、時間窓や逐次状態が本質の表現を`SINGLE`へ偽装しない。

## 3. 現在同梱されている参照plugin

| プラグイン | カテゴリ | 何ができるか | 実装形／時間 | lane | 状態・証拠 |
|---|---|---|---|---|---|
| **Opacity** | Color | premultiplied RGBAの不透明度を調整 | Filter／L0 | READY | **同梱済み** `core.filter.opacity` v1。[VSM-A1](reviews/2026-07-17-vism-implementation-plan.md) |
| **Sine** | Generate / Modulation | amplitude、frequency、offsetから値列を作る | ParamDriver／L1区間一括生成 | READY | **同梱済み** `core.param.sine` v2。[VSM-A2](reviews/2026-07-17-vism-implementation-plan.md) |
| **Radial Repeater** | Generate / Repeat | 点群を正準半径上へ反復配置し、位相と角速度で動かす | LayerSource／L0 | READY | **同梱済み** `core.layer_source.radial_repeater` v1。[A3D](reviews/2026-07-18-vism-a3d-radial-repeater-decision.md) |

これらはVism package loaderの完成例ではなく、first-partyが第三者と同じ公開plugin境界だけで成立するpre-Vism参照実装である。

## 4. Light / Color

| プラグイン候補 | 何が見えるか | 実装形／時間 | lane | 状態・実装見通し |
|---|---|---|---|---|
| **Perceptual Glow** | 明部から知覚的に自然な発光を作る | Filter／L0・複数pass | MULTIPASS + PORTS | **Gate待ち**。linear/HDR中間、Host所有transient texture、typed maskをVSM-A8G0〜G1で共通化してから実装 |
| **Bloom** | 広い半径の光のにじみを重ねる | Filter／L0・複数scale | MULTIPASS | **候補**。Glowと同じ共通resource経路を使い、別の専用backdoorを作らない |
| **Light Rays** | mask／明部から方向性のある光条を伸ばす | Filter／L0 | PORTS + MULTIPASS | **候補**。source maskと方向を型付き入力／parameterに分ける |
| **Directional / Radial / Spin Blur** | 一方向、中心放射、中心回転に沿って像を流す | Filter／L0・複数sample／pass | MULTIPASS | **候補**。三方式を共通sampling能力で反証し、色収差やtransitionは小Vism／Kitとして後段へ分ける |
| **Lens Flare** | 光源位置からflare、glint、shimmerを生成する | LayerSource／Filter・L0 | PORTS + MULTIPASS | **候補**。2D位置、luminance／mask、将来の3D light projectionをtyped providerとして分離。vendor固有element treeや専用designerを写さない |
| **Energy Stroke** | path、mask、text輪郭をneon／energy lineとして描く | LayerSource／Filter・L0 | PORTS、必要時MULTIPASS | **候補**。stroke出力とGlowを小Vismとして接続し、Saber／3D Stroke互換の巨大entryにしない |
| **Ground / Long Shadow** | objectから平面へ落ちる影、または方向へ伸びる影を作る | Filter／LayerSource・L0 | PORTS + MULTIPASS | **候補**。object alpha／pathと投影面・光方向をtyped input／parameterへ分け、3D sceneやlayer走査を所有しない |
| **RGB Split** | 色channelを空間的にずらす | Filter／L0 | SINGLE | **候補**。creative effectであり、出力色変換やdisplay transformを含めない |
| **Grain** | 再現可能な粒状感を加える | Filter／L0 | SINGLE | **候補**。Document由来seedと正準grain scaleを使い、frameごとの未seed乱数を禁止 |
| **VHS / NTSC Material** | scan、bleed、noise、揺れを組み合わせた素材感 | 小Filter群 + Kit／L0、一部L2候補 | SINGLE + KIT | **候補**。巨大な一体pluginへせず、Grain／Scanline／色ずれ等を接続する。時間窓が必要な成分だけTEMPORALへ分離 |

## 5. Pixel / Stylize

| プラグイン候補 | 何が見えるか | 実装形／時間 | lane | 状態・実装見通し |
|---|---|---|---|---|
| **Dither** | 限られたpalette／patternで階調を表す | Filter／L0 | SINGLE | **候補**。palette、pattern、正準cell scaleを個別意味として定める |
| **Halftone** | 網点で明暗や色を表す | Filter／L0 | SINGLE | **候補**。px固定parameterを保存せず、出力解像度への投影をHost側で行う |
| **Pixelate** | 画を大きなcellへ量子化する | Filter／L0 | SINGLE | **候補**。正準cell sizeとsampling／edgeの意味をfixtureで固定 |
| **Pixel Sort** | 明度等の区間に沿ってpixelを並べ替える | Filter／L0 | PORTS | **候補／signature**。GPU内sort、direction、threshold、mask、Draft/Final負荷を実証し、CPU readbackを使わない |
| **Scanline** | 規則的な走査線を重ねる | Filter／L0 | SINGLE | **候補**。表示DPIでなく映像の正準空間に置く |
| **ASCII Raster** | 画素の明暗をglyph密度へ置き換え、文字で像を再構成する | Filter／LayerSource・L0 | PORTS、Text gate後 | **候補**。glyph atlas／font assetはtyped inputとし、font UIや組版正本をeffectへ隠さない |
| **Shape Fill / Write-on** | 種点からshape／image領域を伝播して埋める | Bake provider + Filter／renderはL0 | BAKE + PORTS | **候補**。arrival-time fieldをHost artifactへ確定し、毎frame flood fillやplugin内cacheを行わない |

## 6. Distort / Repeat

| プラグイン候補 | 何が見えるか | 実装形／時間 | lane | 状態・実装見通し |
|---|---|---|---|---|
| **Displace** | field／textureに従って画素位置をずらす | Filter／L0 | PORTS | **候補**。fieldを具体provider名で探さずtyped texture inputで受け、枠外samplingを明示 |
| **Warp** | 規則またはfieldで画面を変形する | Filter／L0 | SINGLEまたはPORTS | **候補**。単純な閉形式warpと外部field版を同じ曖昧なentryにしない |
| **Heat Haze** | noise fieldで局所的な熱揺らぎ／屈折を作る | field provider + Displace／L0 | SINGLE + PORTS | **候補**。決定的fieldとsamplingを分離し、Heat Distortion専用の重複warp実装を作らない |
| **Tile** | 入力textureを反復して面を埋める | Filter／L0 | SINGLE | **候補**。Object複製identityを持たないtexture effectとしてRadial Repeater／Clonerと分離 |
| **Kaleidoscope** | 回転対称の反射反復を作る | Filter／L0 | SINGLE | **候補**。texture samplingの閉形式として始め、汎用Clonerへ拡張しない |
| **Fractal Field** | noise／fractal fieldを背景、mask、displace入力へ供給する | LayerSource／L0 | SINGLE、後にPORTS | **候補**。決定的seedと正準座標を先に固定し、consumerを内蔵しない |

## 7. Text / Typography

Text animationも一般プラグイン一覧の一カテゴリであり、別の製品体系にしない。Text固有のidentity、組版、Sequenceは実装前提としてリンクし、専用の裏口APIへしない。

| プラグイン候補 | 何が見えるか | 実装形／時間 | lane | 状態・実装見通し |
|---|---|---|---|---|
| **Random Entrance** | 書記素ごとに決定的な順序／差で登場する | Text Sequence／L0 | TEXT | **計画決定／Gate待ち**。Text 1 Object、Character Score、選択同期、read-only timing投影を最初の縦切りにする。[TM翻訳](reviews/2026-07-19-m3-text-motion-task-translation.md) |
| **Type Pulse** | 文字ごとにpulseしながら登場／強調する | Text SequenceまたはEffector候補／L0 | TEXT | **候補／履歴照合待ち**。現行モックには発見入口だけがあり、適用handoffと評価形は未決。[UI参照地図](ui-reference-map.md) |
| **Typewriter / Sequential Reveal** | 文字列を順番に表示する | Text Sequence／L0 | TEXT | **候補**。既存text-model animatorを使える範囲と、個別介入を要する範囲を分ける |
| **Kinetic Word Layout** | word／lineを自動配置し、順番にcamera／transform motionへ展開する | Text小Vism + Kit + Authoring Tool候補／L0 | TEXT + KIT | **候補**。TypeMonkey型の需要を受けるが、大量layer生成、独自camera、marker走査を正本にしない |
| **Kinetic Lyrics** | 歌詞の登場、配置、timing、強調を組み合わせる | Text plugin + 小Vism + Kit／L0 | TEXT + KIT | **Gate待ち**。一つの巨大Lyrics pluginへ閉じず、Text identity／Sequenceと交換可能な表現を接続する |
| **Character Collision** | 文字同士またはshapeと衝突して動く | Text投影 + Simulation／L3 | TEXT + SIM | **Gate待ち／履歴照合待ち**。衝突なしの動きはL0、衝突だけをStateTrackへ上げる |

Text実装の責任分離:

- Host: 書記素identity、選択、Undo、投影、合成順。
- first-party Text plugin: 組版、domain定義、標準Sequence。
- Vism: Random Entrance、Type Pulse等の具体表現。
- Simulation: 衝突等、本当に逐次状態が必要な部分だけ。

正本は[Textモデル](text-model.md)と[リリックモーション比較台帳](reviews/2026-07-19-lyric-motion-text-sequence-comparison.md)であり、この一覧から恒久schemaを追加しない。

## 8. Generate / Particle

| プラグイン候補 | 何が見えるか | 実装形／時間 | lane | 状態・実装見通し |
|---|---|---|---|---|
| **Deterministic Particle Field** | seed、誕生時刻、tから粒子群を直接描く | LayerSource／L0 | SINGLE | **計画決定**。標準first-partyの既定経路。軌道を閉形式で計算し、任意時刻へ直接seek可能にする |
| **Particle Collision** | 粒子が床、shape、文字へ衝突する | Simulation + LayerSource／L3 | SIM | **Gate待ち**。StateTrack、固定step、checkpoint、SDF colliderをHostが所有してから実装 |
| **Particle Force / Flow** | fieldに沿って粒子を流す | L0 field samplingまたはL3蓄積 | SINGLE／SIM | **候補**。蓄積しないcurl noise等はL0、速度場の逐次移流だけL3へ分ける |
| **Object / Path Repeater** | path、grid、source poolへinstanceを配置する | LayerSource／Effector候補／L0 | PORTS | **Gate待ち**。stable InstanceId、typed source、P0Iの評価形を先に閉じ、万能Cloner pluginを作らない |
| **Connected Points / Facets** | point間の距離／近傍からline、triangle、facetを生成する | LayerSource／L0 | PORTS | **候補**。typed Point Setとstable identityを受け、AE light／null走査やPlexus型node graphを持ち込まない |
| **Gradient Field / Ramp** | 複数stopから色またはscalar fieldを生成し、背景、map、mask入力へ供給する | LayerSource／provider・L0 | SINGLE、後にPORTS | **候補**。stop編集UIはHost parameter editor、生成結果だけをVismが所有し、display色変換を混ぜない |

## 9. Data / Reactive / Kit

| プラグイン／Kit候補 | 何が見えるか | 実装形／時間 | lane | 状態・実装見通し |
|---|---|---|---|---|
| **BPM Rhythm** | 手入力BPM／tempo mapから拍、位相、bar等のリズムdataを供給する | provider Vism／L0またはL1 | PORTS | **計画決定**。BPM／拍リズムの製品所有者はVism。現行`Document.bpm`は互換入力として投影し、Core固有機能の根拠にしない。出力型はVSM-B2で決める |
| **Beat Pulse** | BPM Rhythm等のbeat位置からScale、Opacity等をpulseさせる | ParamDriver／L0またはL1 | PORTS | **候補**。provider IDを検索せずtyped rhythm inputを受けるconsumerとして分離する |
| **Audio Pulse Kit** | BPM Rhythm／解析provider、motion、Glow、Repeater等を一度に接続する | materialize Kit | KIT | **Gate待ち**。provider／consumer identity、全体preflight、1 Undo、失敗時変更ゼロの後に実装 |
| **Audio Analysis Provider** | 音声からbeat／envelope等のdataを作る | Analysis/Bake provider | BAKE + PORTS + KIT | **後続候補**。v1コア完成後。render中に音声解析せず、結果をtyped dataへ確定する |

## 10. Temporal

| プラグイン候補 | 何が見えるか | 実装形／時間 | lane | 状態・実装見通し |
|---|---|---|---|---|
| **Echo** | 過去／未来の複数時刻を残像として合成する | Filter／L2 | TEMPORAL | **Gate待ち**。宣言時間窓とcache keyが成立してから実装 |
| **Slit Scan** | 空間位置ごとに異なる時刻の像を並べる | Filter／L2 | TEMPORAL | **Gate待ち**。任意seekや自己出力feedbackへ逃げず、有限時間窓を宣言 |
| **Time Displacement** | fieldに従い入力時刻を局所的にずらす | Filter／L2 | TEMPORAL + PORTS | **Gate待ち**。fieldの値域からfootprint上限を求め、無制限時間参照を許可しない |

## 11. Transitions

Transition Vismは表現計算だけを所有する。clip overlap、duration、trim、selection、UndoはHost Timelineが所有し、AEのtransition pluginが両方を一体化した形を写さない。

| プラグイン候補 | 何が見えるか | 実装形／時間 | lane | 状態・実装見通し |
|---|---|---|---|---|
| **Shape Wipe** | shape／maskの進行でfromからtoへ切り替える | 2 texture + mask + progress／L0 | PORTS | **Gate待ち**。2入力typed portとTimeline transition placementの意味決定後 |
| **Glitch Transition** | pixel sort、RGB split、noise等を切替区間へ組み合わせる | 小Filter群 + transition Kit／L0、一部L2候補 | PORTS + KIT | **候補**。一体pluginより既存小Vismの構成を優先し、必要な時間窓だけTEMPORALへ |
| **Light / Bokeh Transition** | flare、blur、bokehで二つの像を光学的に切り替える | 2 texture Filter／L0・複数pass | PORTS + MULTIPASS | **Gate待ち**。Glow／Blurと同じHost所有resourceを使い、transition専用poolを作らない |
| **Page Roll** | 紙面を丸めるように一方の像をめくり、背面の像へ切り替える | 2 texture + deformation + progress／L0 | PORTS + MULTIPASS | **Gate待ち**。最小2D／2.5D変形をfixture化し、Timeline配置、camera、汎用3D meshをeffectへ抱え込まない |

## 12. Vismに入れないもの

外部製品でpluginとして配られていても、次はHostの基礎責任または別成果物である。

| 需要 | 正しい置き場所 |
|---|---|
| Effect検索、適用shortcut、anchor操作、selection操作、easing editor | Host input／command／UI |
| Gradient stop editor、camera transform tool、text animation authoring assistant | Host parameter／authoring UI。camera transform toolは選択／D2／Undoを所有する操作面であり、camera観測model自体は換装可能なCamera Providerとする。[Camera Object / Provider決定](reviews/2026-07-24-camera-object-provider-decision.md) |
| Auto clipping、resize、layer ordering、波形表示 | Host render／layout／Timeline／diagnostic基礎能力 |
| Vector transfer、Figma／Illustrator bridge | External adapter → typed Asset／VectorRecipe → Authoring Tool |
| 通常のTimeMap、trim、baseline motion blur | Hostの時間／render基礎能力 |
| patch、script runtime高速化、plugin manager | Host infrastructure／distribution。表現Vismへ移植しない |
| media input、codec、encoder output、PSD等の外部形式読込 | Input／Delivery／Asset adapter。Vism catalogへ混ぜない |
| Tracking、depth推定、重い背景除去 | Analysis／Bake provider。同期Filterへ偽装しない |
| Lottie、animated SVG等の外向き変換 | 将来のDelivery Adapter。v1の映像Exportへ混ぜない |

## 13. 実装順の見通し

暦順ではなく、公開境界を再利用できる順に進める。

1. **READYを製品Browserへ投影する**
   Opacity、Sine、Radial Repeaterを、catalog metadataから検索・一覧・Inspectorへ同じidentityで投影する。
2. **SINGLEを増やす**
   Dither、Halftone、Pixelate、RGB Split、Scanline、Fractal Field、Heat Haze等から、同じ公開境界で意味の異なる小表現を選ぶ。各pluginは個別の意味fixtureと負例を先に持つ。
3. **PORTSを一用途でなく共通能力として閉じる**
   Displace、Pixel Sort、Connected Points、BPM Rhythm→Beat Pulse等の二つ以上でtyped texture／data inputを反証する。
4. **MULTIPASSをGlowだけの専用口にせず閉じる**
   Blur系fixture、Perceptual Glow、Lens Flareが同じHost所有resource／pass経路を使うことを確認する。
5. **BAKEを解析専用でなく再生成可能artifactとして閉じる**
   Shape Fillのarrival-time fieldとAudio Analysis等が同じjob／取消／stale／cache規律を使うことを確認する。
6. **TEXTを一般カタログへ接続する**
   Random Entranceの縦切りを先行し、Type Pulse、Kinetic Lyricsへ広げる。Text専用plugin一覧を別に作らない。
7. **Transitionの計算とTimeline配置を分けて閉じる**
   Shape Wipeを最小fixtureに、2入力texture評価とclip overlap／Undoを別所有者にする。
8. **TEMPORALとSIMを正規の高コスト経路で解凍する**
   Echo／Slit ScanはL2、衝突Particle／Character CollisionはL3へ置き、隠れ状態による近道を拒否する。
9. **KITで完成用途を束ねる**
   小Vismの交換可能性を保ったままAudio Pulse、Lyrics Starter、VHS Material等をmaterializeする。

各段でfirst-partyと第三者のconformanceを分けない。次段のために現段のpluginへ暫定raw API、文字列走査、Host private依存を追加しない。

## 14. 将来展望 — 全Vismを安全に並列実装できる構造

本カタログに置くVismは、将来それぞれを同時発注・同時実装しても、契約、コード、test、登録、artifactが相互干渉しない構造を完成条件にする。並列化の単位は**一つのVism実装境界**であり、共有公開API、Document、永続形式、plugin kind、Host resource契約を各実装が同時に変更する形ではない。

安全な並列実装の前提:

1. **共有境界を先に直列で閉じる**
   `SINGLE / PORTS / MULTIPASS / BAKE / TEXT / TEMPORAL / SIM / KIT`の各laneは、代表fixtureと負例で公開契約を凍結してからVism実装を横へ展開する。
2. **一Vism一所有単位**
   原則として独立plugin crate、固有ID／version、固有parameter意味、固有oracleを持つ。別Vismのsource、private helper、test期待値を変更しない。
3. **共有primitiveはHost側の正規境界だけを使う**
   texture pool、pass scheduling、typed port、seed、time、Bake、StateTrack等をVismごとに再実装しない。新しい共通能力が必要なら当該Vism実装を止め、共有境界の独立タスクへ戻す。
4. **登録衝突を機械拒否する**
   duplicate ID、kind／version／contract不一致、reserved ID、migration競合、欠落executorをcomposition rootとconformanceで検出する。
5. **fixtureとartifactを分離する**
   各Vismは正例、負例、purity、GPU、Preview／Export一致、予算、欠落／未来版を自分のfixtureで判定する。共通goldenや他Vismの期待値を書き換えて通さない。
6. **catalog統合は生成または検証可能にする**
   手編集の巨大登録表を複数実装が奪い合わない。manifest／contractから決定的に列挙するか、独立entryの追加だけで済むcomposition方式を採る。
7. **Kitは依存を隠さない**
   複数Vismの完成用途はtyped connectionとpreflightで束ね、consumerが具体provider IDや他Vismの内部を検索しない。
8. **並列不能を正常なSTOPとする**
   公開API、Document、永続形式、plugin contract、共通resource、未決domainの変更が必要になったVismは、その場で便利な共通化をせず停止する。共有境界を正本化・実装・検収した後に再開する。

したがって「全Vismを並列実装できる」とは、無制限の同時編集を許すことではない。**共有laneを一度閉じれば、そのlane上のVism追加は互いを知らず、同じconformanceを通して独立に完成できる**ことを意味する。Vism数が増えてもCore改造、登録表競合、helper複製、test期待値変更が増えないことを、将来のcatalog量産gateで機械判定する。

### 14.1 第三者生態系との接続も同じ並列モデルにする

内部の並列量産は、将来の第三者作者生態系を小さく先取りする。第三者作者は同じrepository、計画、release周期、組織、Host実装を共有しない。したがって公開境界は、中央の担当者が全Vismを調整し続ける前提ではなく、**多数の作者が互いを知らずに同時開発・公開・更新しても、競合が局所化し診断可能であること**を完成条件にする。

第三者接続で追加される条件:

1. **名前空間とidentityを作者間で衝突させない**
   package、entry、Kit、Project instance、artifact identityを分離し、表示名や登録順からidentityを導出しない。具体的な命名規則、registry、version併存方式はPhase B／Cで比較する。
2. **Host内部ではなくcapabilityへ依存する**
   VismはMotoliiのprivate crate、UI tree、Document走査、特定first-party Vismへ依存せず、version付き公開capabilityとtyped inputだけを要求する。
3. **依存解決を公開前に検査できる**
   Kitとpackageは必要capability、Vism constraint、asset、permissionを宣言し、欠落、循環、非互換、予算超過をinstall／materialize／実行前にtyped diagnosticで返す。
4. **conformanceを作者側でも同じように実行できる**
   first-party専用CIや秘密fixtureを合格条件にしない。第三者が公開前にpurity、GPU、resource、migration、missing、Preview／Export一致を同じbundleで検査できるようにする。
5. **由来、build、permissionを機能互換と分ける**
   作者、source／artifact hash、署名、build環境、filesystem／network等の権限、review状態を保持する。動くこと、安全であること、信頼することを一つの判定に潰さない。
6. **未知／欠落／未来版を作品全体の破壊へ広げない**
   該当Vismだけをunavailableとして原本を保持し、無関係な編集を続けられる。strict exportは必要Vismをtyped拒否し、黙った置換やpass-throughで完成扱いしない。
7. **同時更新を中央の一斉更新にしない**
   Vism、Kit、Project instanceのlifecycleを分け、第三者の新versionが既存Projectを無断変更しない。互換範囲、migration、pin／解決方式は明示し、自動追従の詳細は後続審判とする。
8. **成功した第三者能力を追加的に昇格できる**
   fork／第三者capabilityは名前空間付きで実験し、複数Vismとfixtureで需要が安定してからBase capabilityへ提案する。Core enumや特権APIへ直接昇格しない。

内部Vismと第三者Vismは別の実行規律を持たない。first-party catalogで並列安全を証明できない境界は、第三者へ公開しない。反対に、第三者が同じconformanceだけで成立しないfirst-party実装は参照実装として不合格とする。

## 15. 追加・更新規則

新しい候補を思い出した／発見した時は、最低限次を埋めてこの一覧へ追加する。

1. 利用者が探す表現名と、見える結果。
2. `Filter / LayerSource / ParamDriver / Simulation / Kit`等の実装候補。
3. L0／L1／L2／L3の時間性。
4. 必要な共通能力と、既存laneで再利用できる箇所。
5. 状態: 同梱済み／計画決定／候補／Gate待ち／履歴照合待ち。
6. Motolii fixtureと恒久的な負例。
7. 現行正本、需要調査、または検証済み履歴パケットへのリンク。

過去Git由来の候補は、固定historical-recovery契約が現行worktreeへ復旧した後、repo外のcandidate packetから別単位で裁定する。履歴の類似だけで状態を**計画決定**へ上げない。
