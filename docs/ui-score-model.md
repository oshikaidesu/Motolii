# 譜面UI構成モデル — Laneを所有者にしない時間面

ステータス: **設計決定**(2026-07-17)。M3製品実装の許可、Document schema、公開UI API、具体的なegui componentを確定する文書ではない。

## 1. 目的

譜面はDocumentの時間構造を読む投影であり、DAW/AE型の固定Track/Lane一覧ではない。モックを更新するたびに「1項目1横行」「固定名列」「横行固有control」が戻る出戻りを防ぐため、変えてよい見た目と変えてはいけない構成を分離する。

固定React baselineの帯アクションrail、Automation展開、Key Tools、Group展開、
multi-key Graph Viewは比較証拠として
[M3 UI参照地図](ui-reference-map.md)とprototype台帳から参照する。本書の現行score、
状態所有、受け入れ条件へは昇格させず、採択時は仕様・本書を先に改訂する。

## 2. 正準構成

譜面は上から次の4層で構成する。

1. **楽曲基準線**: 最上段に固定する時間の基準。一般ObjectのLaneではない
2. **Depth Rail**: 現在時刻の`Edit-Space Z`を`Depth`として比較・編集する数直線。rootとGroup childの表示段はparent空間を読み違えないための区分であり、設定所有者ではない
3. **Inbox**: 左端に1個だけ置く。外部から受け取ったが未配置の素材、未解決のreview note、未確認のbackground job結果など、「まだ正規の持ち場へ片付いていないもの」への参照を一時表示する。選択やhoverへ追従しない
4. **時間面**: 右側の一枚の面へObject/Group/Clipのbarを置く。barの縦位置は重なり回避のpacking結果にすぎない

bar、Z marker、Stage、Inspectorは同じ安定IDを選択として投影する。どの入口から選んでも別の選択状態やUndoを作らない。

## 3. DepthはPosition X/Yから独立した操作面にする

Documentの正準座標は引き続きXYZであり、Depthの保存値も既存の`position.z`である。一方、UIではX/YとZを対称な3入力として扱わない。

- `Position X/Y`はStage平面上の配置として直接操作する
- `Depth`は前後関係、Cameraとの位置関係、Group内の相対奥行き、遮蔽へつながる独立した操作面としてDepth Railで扱う
- Inspectorも`Position: X / Y / Z`の一列ではなく、`Position: X / Y`と`Depth: Z`を別groupへ投影する。一般表示は`Depth`を主名とし、数値・Developer infoで`Z`を併記してよい
- Depthを表示・編集しただけで、別の「3D mode」、3D専用Object、別座標系へ切り替えない。高度な空間機能の存在をZ値の変更へ暗黙に結び付けない
- `Depth Z`はZ方向の**平行移動**であり、Z軸まわりの**回転**ではない。`Rotation Z`、Camera roll、平面内回転と同じcontrol・label・automation channelへまとめない
- UI上の分離を理由に、第二のDepth field、Depth専用Document所有者、Depth固有の保存形式を追加しない

Depth Railは現在時刻`t`の評価結果へ追従する。再生・seek・keyframe評価でmarkerが動き、固定された初期配置表として扱わない。railの開閉は明示操作とし、選択変更だけで自動展開してlayoutを跳ねさせない。開閉はWorkspace-sessionまたはTransientであり、Document・Journal・Undoへ入れない。

2D Objectの既定`z=0`では同一parentのmarkerが同一点へ重なる。このときhoverや選択だけでmarkerを扇状展開せず、同値集合を`0 × N`のような件数付きstackとして投影する。Timeline barを選択すると同じ安定IDをstack内のfocusとして識別し、Railが開いている時だけ該当位置を表示範囲へ寄せる。通常のbar選択だけでRailを開かず、bar内または譜面headerの明示Depth iconからRailを開いて対象へfocusする。

同一Zを実際の奥行きへ展開する頻出操作はcontext menuへ隠さず、Depth Rail上の**Layer Order Distribute** iconから行う。対象は同じparentを持つ選択Objectに限り、ユーザーが指定した奥端・手前端の区間へauthoring orderで等間隔配置する。既定方向は現在の`Layer Order`で手前に合成されるObjectをcamera側へ割り当て、`Reverse`は区間を変えず割当順だけを反転する。区間drag中は通常`position.z`をlive previewし、確定は選択集合へのD2 macro 1回、Cancelは変更ゼロとする。表示上のstack解消だけを目的にDocument値を自動変更しない。

rail上の直接操作はDepthの現在値を編集できる。Depth automationが有効な対象では、現在時刻のkeyを更新し、keyが無い時刻で確定したdragはその時刻のkeyを作る。automationが無効な静的Depthをdragしただけでは、暗黙にautomationを開始しない。keyの時間配置とEasingは時間面を正本とし、railを第二の小型Timelineにしない。

Cameraはroot worldと同じ比較文脈に専用形状のmarkerとして投影できるが、world Zの大小をcamera-space depthと偽装しない。Cameraの向き・targetから導く前後順位は別の導出表示とする。生成系はParticle個体をmarker化せず、Emitter／生成元の安定IDだけを選択・keying対象にする。必要なら粒子群の評価済みDepth範囲をread-onlyの帯として表示し、個体編集はMaterialize後の通常Objectへ限定する。

Groupは親側ではGroup自身を1 markerとして扱い、子を同時表示しない。TimelineでGroupを展開してもDepth Railの編集scopeは自動で混在させず、`Edit Children`、scope icon、またはRailを開いた状態での明示的なchild bar選択により、同じGroupの子だけへ切り替える。child選択によるscope追従はRailの高さや開閉を変えず、root markerを同じrailへ残さない。Group内部の子はparent-local Zへ配布し、root Object、Group自身、別Groupの子を同じ区間操作へ混ぜない。Group内部は選択した遮蔽ポリシーで合成し、Group境界ではpremultiplied RGBAへ平坦化して親側の1 Objectとして扱う。

## 4. 不変条件

- 1項目1横行、固定名列、永続する縦位置を持たない
- 横行固有のSolo/Mute、設定、enable、値編集を置かない
- `Track`や`Lane`をDocument上の所有者、評価順、保存形式として追加しない
- 再packingでDocument上の所有者、Group関係、評価順、Z、時間区間を変えない
- 見失った時の項目一覧は一時検索から開き、Inboxを第二の恒久treeや選択一覧にしない
- Inboxはasset、note、jobを一つの保存形式へ統合しない。それぞれの正規状態への参照だけを表示し、配置・解決・確認・dismiss後はInboxから外す
- Inboxへ通常操作のhistory、選択追従情報、設定、command launcherを自動蓄積しない
- Inboxが空の時だけ、既読管理可能なTipを一件表示してよい。TipはUser settingでdismissし、Document・Journal・Undoへ入れない
- 通常時はInspectorで操作中のchannelのkeyだけを対応barへ重ねる。全parameter行を常設しない
- Easingの対象はkey単体ではなく、現在時刻を挟む同一channelの`左key → 右key`区間である。Preview直下のEasing Graph iconはplayheadが区間の**内部**にある時だけ点灯・操作可能になり、key上、最初のkeyより前、最後のkeyより後では消灯する。key clickをGraph Viewの入口へ兼用しない
- iconから開くGraph Viewには補間種別、value-time graph、Bezier handle、raw 4値、preset、overshootを収め、簡略presetだけを「Easing編集」と呼ばない。対象表示はObject・channelまでとし、区間番号、key数、時刻範囲、key stripを重ねない。Graph左右の余白へcurve形状thumbnailとhandle値を置き、curve名はhover / focusのInfoへ下げる。curve/preset適用は現在区間への1 command / 1 Undoとし、補間値は区間の左keyに属するoutgoing interpolationとして扱う
- Graph iconのsingle clickはGraph Viewを開き、double clickは◎で示したお気に入りcurveを現在区間へ即適用する。お気に入りは1個だけのUser settingでDocument・Undoへ入れず、最後に使ったcurveへ自動追従しない。double click適用だけが1 command / 1 Undoであり、key上・区間外では実行しない
- Effectは対象bar上で`IN → Effect → OUT`として読み、readinessはbar下辺の区間patternとして読む。どちらも独立Laneを作らない

## 5. Group名はbar自身に表示する

packingだけではGroup化の結果が読みにくいため、短い所有関係ラベルをbar内へ置く。

- Group本体のbarは`GROUP · <名前>`を表示する
- Group childのbarは`↳ <親Group名>`と自身の名前を表示する
- ラベルはDocumentのGroup関係から導出する投影であり、UI専用の名前や保存状態を持たない
- ラベルのために左の固定名列、専用横行、縦の囲い、Group専用のS/M・設定入口を追加しない
- barが狭い時は種類icon、項目名、親Group名を優先し、値はInspector、接続はArchitect、操作説明は下端Statusへ逃がす
- 離れた時間区間やpacking位置を縦の囲いで束ね、実在しない継続所有を示さない

この反復表示は「どのLaneにいるか」ではなく、「このbarは何で、どのGroupに属するか」を読むための局所的な手掛かりである。

## 6. 状態と操作の持ち場

| 対象 | 持ち場 | Undo |
|---|---|---|
| Group関係、時間区間、Edit-Space Z | Document。既存D2 commandと単一writerを通す | あり |
| Depth markerのpointer down〜up | live preview後にD2 macro 1回。automation中は現在時刻のZ keyを更新または追加し、Cancelは変更ゼロ | 1 gesture = 1 |
| Layer Order Distributeの奥端・手前端、方向、対象 | 区間と方向のpreviewはTransient。同じparentの選択stable ID、authoring order、現在のZ値から通常`position.z` command列へ正規化し、専用Depth保存形式を作らない | 1確定 = 1 macro / 1 Undo |
| 選択、scroll、bar packing、Depth Rail・Easing Graph Viewの開閉、一時検索 | Workspace-sessionまたはTransient。Easing対象区間はplayheadと両端keyから導出 | なし |
| Inboxへの未配置file参照、未確認job、dismiss済みTip | 各正規状態を所有せず、Workspace-session / Transient / User settingから未整理状態だけを投影する。review noteの共有・永続意味は本モックで決めない | なし |
| readiness、provider状態 | read-only snapshotの投影 | なし |
| Camera-space depth、Particle群のDepth範囲 | 評価結果からのread-only導出。Documentの第二のDepth値にしない | なし |

packingのpx、DPI、ウィンドウ座標をDocument、評価、公開plugin契約へ流さない。

## 7. 受け入れる構成 / 拒否する構成

| 受け入れる | 拒否する |
|---|---|
| 一枚の時間面へbarをpackingする | 項目ごとの固定Track/Lane |
| 左端に未整理物への参照だけを示すInboxを1個置く | 左端へ全項目名、選択接続、Inspectorのparameterを並べる固定列 |
| bar内の`GROUP · 名前`、`↳ 親Group名` | Groupごとの恒久的な横行・縦帯 |
| bar/Z/Stageで同じ選択IDを共有する | 表示面ごとに別の選択正本を持つ |
| UIではPosition X/YとDepth Zを別groupへ投影する | 保存用Depth fieldや暗黙の3D modeを追加する |
| 同一Zを件数付きstackで示し、明示iconから選択集合を指定区間へauthoring orderで配布する | hover／選択だけの扇状展開、表示衝突回避のための自動Z変更、context menuだけの配布入口 |
| Timeline選択を開いているRailの同一markerへ同期し、明示Depth iconでRailを開く | 通常bar clickのたびにRailを自動開閉してlayoutを跳ねさせる |
| 親側ではGroupを1 marker、Edit Children中は同じGroupの子だけをparent-local Zで表示する | root、Group自身、複数parentの子を同じ編集rail／区間操作へ混在させる |
| Emitterをmarker、Particle群をread-only範囲として示す | Particle個体を無制限にmarker・Document項目化する |
| 現在操作中のkeyだけをbarへ重ねる | automation可能な全parameter行を展開する |
| Preview直下のEasing iconをkey間だけ点灯し、その区間をGraph Viewで編集する | key clickでEasingを開く、key単体や任意key集合を補間の所有者にする |
| readinessをread-only patternで示す | readiness表示からcache/bake policyを変更する |

## 8. 回帰審判

モックまたは製品UIを変更する時は、最低限次を負例fixtureまたは構造検査で固定する。

1. 固定名列、1項目1横行、`Track`/`Lane`所有者が追加されていない
2. barを別の縦位置へpackingしても同じDocument意味snapshotになる
3. Group化・解除・親変更でbar内ラベルがDocumentのGroup関係から更新される
4. bar選択とZ marker選択が同じ安定IDをStage・譜面・Inspectorへ投影する
5. channel変更時に同じbar上のkey集合だけが切り替わり、空parameter行が増えない
6. readiness表示を操作してもDocument、cache、bake policyが変わらない
7. 左Inboxが選択・hoverで入れ替わらず、未配置素材・未解決note・未確認jobだけを参照し、処理後に消える。asset、note、jobの保存意味を一形式へ統合しない
8. playheadが隣接keyの間にある時だけEasing iconが点灯し、key上・区間外では消灯する。key clickだけではGraph Viewが開かない
9. Easing Graph Viewでcurve、handle、raw値、補間種別を同じ区間正本から検査でき、適用が1区間への1 Undo、handle dragの`Esc`が変更ゼロになる。区間番号・key数・時刻範囲・key stripを重複表示しない
10. お気に入りcurveは形状thumbnail上の単一◎markで識別でき、mark変更はDocument・Undo不変。点灯中Graph iconのdouble clickはそのcurveを現在区間へ1 Undoで適用し、single click popupを残さない。最後に使用したcurveやHistory順でお気に入りが変わらない
11. Depth Railがseek・再生時の現在評価値へ追従し、静的Depthのdragだけではautomationを暗黙に開始しない
12. Position X/YとDepth ZをUI上で分けても、同じ`position`のDocument意味を読み書きし、第二のDepth fieldや3D modeを生成しない
13. Depth Zの平行移動とRotation Zを異なるlabel・control・automation channelとして識別できる
14. Camera markerがworld Zとcamera-space depthを混同せず、Particle個体数に比例してmarker数やDocument項目数が増えない
15. 同一parentの2D Object 20〜100個が全て`z=0`でもmarkerをhover／選択で扇状展開せず、`0 × N`の件数とTimelineでfocus中のstable IDを識別できる
16. Layer Order Distributeが明示した奥端・手前端を保ち、authoring orderで等間隔配置する。ReverseはZ集合を保って割当だけを反転し、操作前後でDocument子順、選択、Group関係を変えない
17. Timeline barの通常clickは選択だけを同期し、閉じたRailを開かない。bar内またはheaderのDepth iconはRailを開いて同じIDへfocusし、Document、Journal、Undoを変えない
18. Group自身と子を同時scopeへ混ぜず、Edit Childrenでは同じparentの子だけをparent-local Zへ配布する。mixed-parent選択は変更ゼロのまま理由を表示する

次のいずれかが必要に見えた時は、UI実装を止めて本書とM3仕様を先に改訂する。

- 固定Track/Laneを公開型またはDocumentへ追加する
- packing位置へ意味を持たせる
- Groupラベル専用の永続状態を追加する
- Timeline側へInspectorと同じ設定編集面を複製する
- Inboxへ全履歴、全asset、全note、全jobを恒久保存する

## 9. 関連文書

- [UIコンセプト](ui-concept.md)
- [UI操作言語](ui-interaction-language.md)
- [UI視覚言語](ui-visual-language.md)
- [M3高密度メインUIモック](mocks/README.md)
- [M3 UI境界汚染の予防](reviews/2026-07-14-m3-ui-boundary-prevention.md)
- [M3 UI統合仕様](specs/M3-ui-integration.md)
