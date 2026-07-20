# 譜面UI構成モデル — Laneを所有者にしない時間面

ステータス: **設計決定**(2026-07-17)。M3製品実装の許可、Document schema、公開UI API、具体的なegui componentを確定する文書ではない。

## 1. 目的

譜面はDocumentの時間構造を読む投影であり、DAW/AE型の固定Track/Lane一覧ではない。モックを更新するたびに「1項目1横行」「固定名列」「横行固有control」が戻る出戻りを防ぐため、変えてよい見た目と変えてはいけない構成を分離する。

## 2. 正準構成

譜面は上から次の5層で構成する。

1. **楽曲基準線**: 最上段に固定する時間の基準。一般ObjectのLaneではない
2. **Depth Rail**: 現在時刻の`Edit-Space Z`を`Depth`として比較・編集する数直線。rootとGroup childの表示段はparent空間を読み違えないための区分であり、設定所有者ではない
3. **Inbox**: 左端に1個だけ置く。外部から受け取ったが未配置の素材、未解決のreview note、未確認のbackground job結果など、「まだ正規の持ち場へ片付いていないもの」への参照を一時表示する。選択やhoverへ追従しない
4. **帯アクションrail**: Inbox右隣に置く、従来の名前欄相当の細い無名領域。時間面の各packing帯と上下境界を共有し、帯上Objectへの一括M/Sと、その帯だけの表示高調整を受ける。名前、Lane ID、Lane所有状態は持たない
5. **時間面**: rail右端から始まる一枚の面へObject/Group/Clipのbarを置く。barの縦位置は重なり回避のpacking結果にすぎない

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

rail上の直接操作はDepthの現在値を編集できる。Depth automationが有効な対象では、現在時刻のkeyを更新し、keyが無い時刻で確定したdragはその時刻のkeyを作る。automationが無効な静的Depthをdragしただけでは、暗黙にautomationを開始しない。keyの時間配置とEasingは時間面を正本とし、railを第二の小型Timelineにしない。

Cameraはroot worldと同じ比較文脈に専用形状のmarkerとして投影できるが、world Zの大小をcamera-space depthと偽装しない。Cameraの向き・targetから導く前後順位は別の導出表示とする。生成系はParticle個体をmarker化せず、Emitter／生成元の安定IDだけを選択・keying対象にする。必要なら粒子群の評価済みDepth範囲をread-onlyの帯として表示し、個体編集はMaterialize後の通常Objectへ限定する。

## 4. 不変条件

- 1項目1横行、固定名列、永続する縦位置を持たない
- 横行そのものが所有するSolo/Mute、設定、enable、値編集を置かない。例外として、現在そのpacking帯に載るObject集合へ既存Object操作を一括適用する**帯アクションrail**を置いてよい。railは状態を所有せず、押下時のObject集合へ操作をmaterializeする
- 帯アクションrailの右端と時間面の左端、rail header下端と時間ruler下端、各rail行下端と対応packing guide下端を同じ座標にする。独立した行高やずれたhit領域を作らない
- 帯高のresize hit領域はrail各行の下辺へ統合する。drag中は対応guideを強調し、触った帯だけを変更する。時間面全体、全帯共通control、guide面全域をresize対象にしない
- `Track`や`Lane`をDocument上の所有者、評価順、保存形式として追加しない
- 再packingでDocument上の所有者、Group関係、評価順、Z、時間区間を変えない
- 見失った時の項目一覧は一時検索から開き、Inboxを第二の恒久treeや選択一覧にしない
- Inboxはasset、note、jobを一つの保存形式へ統合しない。それぞれの正規状態への参照だけを表示し、配置・解決・確認・dismiss後はInboxから外す
- Inboxへ通常操作のhistory、選択追従情報、設定、command launcherを自動蓄積しない
- Inboxが空の時だけ、既読管理可能なTipを一件表示してよい。TipはUser settingでdismissし、Document・Journal・Undoへ入れない
- Object bar内の`◆ n`は、そのObjectで実際にAutomationを持つchannel数を示す明示入口とする。押すとAE型の縦一覧としてAutomation済みchannel行をbar直下へ展開し、複数channelのkeyを同時に読んで選択できる
- 未使用channelはAutomation一覧へ混ぜず、末尾の`＋ Automationを追加…`から検索して追加する。物理key shortcutの記憶、全parameterの事前展開、Object本体clickへの展開兼用を必須にしない
- 選択keyへ作用するKeystone型の操作面は必要sectionだけを開閉し、`Object別 / Channel別 / 全選択`の適用単位を明示する。Align / Stagger / Stretch等は既存Easingを保持し、曖昧な一括対象やmodifierだけのmodeにしない
- Keystone型操作面はTimeline右端の独立した**Key Tools View**を既定位置とする。Timelineへの浮遊popover、BrowserのPreset棚、InspectorのEffect parameter面へ埋め込まず、共通panel systemでsplit / tab / resize / 別dockへ移動できる
- Key Toolsは`KEYS / LAYERS`の排他的modeに分ける。KEYSは選択key、LAYERSは選択Objectだけを入力とし、片方のmodeで他方のsectionを同時表示しない。mode切替はViewの動的投影でありDocument意味を変えない
- Easingの対象はkey単体ではなく、現在時刻を挟む同一channelの`左key → 右key`区間である。Preview直下のEasing iconはplayheadが区間の**内部**にある時だけ点灯・操作可能になり、key上、最初のkeyより前、最後のkeyより後では消灯する。key clickをInterval Easing Editorの入口へ兼用しない
- iconから開くInterval Easing Editorには補間種別、正規化time-remap curve、Bezier handle、raw 4値、preset、overshootを収める。対象表示はObject・channelまでとし、区間番号、key数、時刻範囲、key stripを重ねない。Editor左右の余白へcurve形状thumbnailとhandle値を置き、curve名はhover / focusのInfoへ下げる。curve/preset適用は現在区間への1 command / 1 Undoとし、補間値は区間の左keyに属するoutgoing interpolationとして扱う
- Easing iconのsingle clickはInterval Easing Editorを開き、double clickは◎で示したお気に入りcurveを現在区間へ即適用する。お気に入りは1個だけのUser settingでDocument・Undoへ入れず、最後に使ったcurveへ自動追従しない。double click適用だけが1 command / 1 Undoであり、key上・区間外では実行しない
- Graph ViewはTimelineと同じ実時間上でfocus中channelの実値curveを表示する独立dock面であり、Interval Easing Editorとは別入口にする。parameter listとgraph areaを持ち、複数key・複数区間の時刻、値、導出tangentを編集する。context channelは参照表示に留め、focus切替、pan/zoom、Frame Selected、snapshotではDocument・Undoを変えない
- Effectは対象bar上で`IN → Effect → OUT`として読み、readinessはbar下辺の区間patternとして読む。どちらも独立Laneを作らない

## 5. Groupは同じ時間面で畳む／開く

通常のGroupをAEのプリコンポのような別Composition、別Preview、別Timelineへの入口にしない。Groupは同じStageと同じ時間軸に残り、畳めばGroup bar 1本、開けばchildをその場へ展開する。Stageの画とplayhead、選択IDは開閉で変えず、戻るためのnavigation stackや`Esc`を要求しない。

- Group本体のbarはGroup icon、`<名前>`、child数を表示し、通常のObject barと同じS/M、Automation要約、選択入口を持つ
- Group barのS/MはGroupという項目エンベロープの出力を試聴・隔離する入口であり、childのS/M状態を一括書き換えない。GroupをSoloした時は配下をGroup出力の一部としてaudibleに投影し、childだけをSoloした時は描画に必要な親Groupを通す。複数Soloの詳細な評価順はM/S意味論の仕様決定へ残す
- 開いた時はchild barをGroup bar直下へ局所的に展開し、親Groupの背景色は展開帯だけへ継承する。child bar自身の色は種類・identityを示す自身の色から変えない。indent、`↳ <親Group名>`、短い接続guideも併用し、色だけを所属の唯一の手掛かりにしない。child自身のS/MとAutomationは各barで操作する
- double clickまたはfold iconは同じ場所で開閉する。通常Groupのdouble clickを別Previewへの遷移に割り当てない
- ラベルと開閉状態のうち、親子関係と名前はDocumentから導出し、開閉はWorkspace-sessionまたはTransientに置く。UI専用のGroup名や別Compositionを作らない
- ラベルのために左の固定名列、Group専用横行、時間全域を囲う縦帯、Group専用設定panelを追加しない
- barが狭い時は種類icon、項目名、親Group名を優先し、値はInspector、接続はArchitect、操作説明は下端Statusへ逃がす
- 離れた時間区間やpacking位置を縦の囲いで束ね、実在しない継続所有を示さない

M/Sがあるため、Group全体の結果を聞く／見るためだけに別Previewへ移る必要はない。ただしM/Sはchildを選択・並べ替え・Automation編集する動線の代わりではないため、その編集には同じ時間面での展開を使う。将来Group Definition/UseやCompositionClipのような再利用sourceを採択する場合だけsource編集面を別途検討し、通常Groupへ遷移意味を混ぜない。

## 6. 状態と操作の持ち場

| 対象 | 持ち場 | Undo |
|---|---|---|
| Group関係、時間区間、Edit-Space Z | Document。既存D2 commandと単一writerを通す | あり |
| Depth markerのpointer down〜up | live preview後にD2 macro 1回。automation中は現在時刻のZ keyを更新または追加し、Cancelは変更ゼロ | 1 gesture = 1 |
| 選択、scroll、bar packing、Depth Rail・Interval Easing Editor・Graph Viewの開閉、一時検索 | Workspace-sessionまたはTransient。Easing対象区間はplayheadと両端keyから導出。Graphのview rangeとsnapshotもDocument外 | なし |
| 帯アクションrailのM/S | 現在そのpacking帯に交差する安定Object ID集合を押下時にsnapshotし、Object単位のMute / Solo intentへ展開する。rail自身は状態を保存せず、表示は集合の全ON / 全OFF / 混在から導出する。再packingだけでは既適用Objectの状態を変えない | 製品実装では展開したObject操作を1 macro / 1 Undo。prototypeはDocument意味を決めない |
| `◆ n`の開閉、Automation channel検索、key選択、Key Toolsの`KEYS / LAYERS`、sectionと適用単位 | 開閉・検索・選択・mode・sectionはTransient。dock位置・split・tab・幅はWorkspace-session。`n`と一覧はDocument上でAutomationを持つchannel集合から導出 | なし |
| Groupの開閉、展開中の局所配置 | Workspace-sessionまたはTransient。Group関係、名前、child順、Group自身とchild自身のS/MはDocumentの項目状態から投影し、開閉で書き換えない | なし |
| `＋ Automationを追加…`からのchannel追加 | 既存D2 automation commandへ正規化する製品候補。prototypeはReact stateで操作動線だけを比較し、Document形式を決めない | 製品実装では1追加 = 1 Undo |
| Inboxへの未配置file参照、未確認job、dismiss済みTip | 各正規状態を所有せず、Workspace-session / Transient / User settingから未整理状態だけを投影する。review noteの共有・永続意味は本モックで決めない | なし |
| readiness、provider状態 | read-only snapshotの投影 | なし |
| Camera-space depth、Particle群のDepth範囲 | 評価結果からのread-only導出。Documentの第二のDepth値にしない | なし |

packingのpx、DPI、ウィンドウ座標をDocument、評価、公開plugin契約へ流さない。

## 7. 受け入れる構成 / 拒否する構成

| 受け入れる | 拒否する |
|---|---|
| 一枚の時間面へbarをpackingする | 項目ごとの固定Track/Lane |
| 左端に未整理物への参照だけを示すInboxを1個置く | 左端へ全項目名、選択接続、Inspectorのparameterを並べる固定列 |
| Inbox右隣の無名帯アクションrailから、押下時の帯上Object集合へM/Sを一括適用する | packing帯自体へ永続M/Sを保存し、再packing後の別Objectへ状態を自動継承する |
| rail行と時間面のpacking帯が同じ上下境界を共有し、下辺だけをresize入口にする | railと時間面で別の行高を使う、境界から離れた浮遊handle、どこでもresizeできるguide面 |
| Groupを同じ時間面で1本に畳む／childをその場で展開する | Group double clickで別Composition、別Preview、別Timelineへ移動する |
| Group barの名前・child数・S/M、親背景を継承する展開帯、固有色を保つchild barと`↳ 親Group名` | Groupごとの恒久的な横行・縦帯、親S/Mをchild状態へ複製する、child bar自身の色まで親色へ上書きする |
| bar/Z/Stageで同じ選択IDを共有する | 表示面ごとに別の選択正本を持つ |
| UIではPosition X/YとDepth Zを別groupへ投影する | 保存用Depth fieldや暗黙の3D modeを追加する |
| Emitterをmarker、Particle群をread-only範囲として示す | Particle個体を無制限にmarker・Document項目化する |
| 現在操作中のkeyだけをbarへ重ねる | automation可能な全parameter行を展開する |
| bar内の`◆ n`からAutomation済みchannel行を縦展開し、複数行のkeyを選択して明示scopeの操作面へ渡す | 1 channelだけのpopoverへ置換する、Object本体clickへ選択と展開を重複割当する、全parameterを一覧へ常設する、shortcutを唯一の入口にする |
| Timeline右端の独立Key Toolsで`KEYS / LAYERS`を切り替え、必要sectionだけを表示する | Browser、Inspector、Timeline overlayのいずれかへ固定埋込みする、Key操作とLayer操作を同時に全表示する |
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
8. playheadが隣接keyの間にある時だけEasing iconが点灯し、key上・区間外では消灯する。key clickだけではInterval Easing Editorが開かない
9. Interval Easing Editorでcurve、handle、raw値、補間種別を同じ区間正本から検査でき、適用が1区間への1 Undo、handle dragの`Esc`が変更ゼロになる。区間番号・key数・時刻範囲・key stripを重複表示しない
10. お気に入りcurveは形状thumbnail上の単一◎markで識別でき、mark変更はDocument・Undo不変。点灯中Easing iconのdouble clickはそのcurveを現在区間へ1 Undoで適用し、single click popupを残さない。最後に使用したcurveやHistory順でお気に入りが変わらない
11. Graph Viewは複数key・複数区間を実時間×実値で表示し、Frame Selected以外ではdrag中もview rangeが変わらない。snapshotとcontext curveを主curveから識別でき、curve上へのkey追加前後で形状が不変になる
11. Depth Railがseek・再生時の現在評価値へ追従し、静的Depthのdragだけではautomationを暗黙に開始しない
12. Position X/YとDepth ZをUI上で分けても、同じ`position`のDocument意味を読み書きし、第二のDepth fieldや3D modeを生成しない
13. Depth Zの平行移動とRotation Zを異なるlabel・control・automation channelとして識別できる
14. Camera markerがworld Zとcamera-space depthを混同せず、Particle個体数に比例してmarker数やDocument項目数が増えない
15. 帯アクションrailのM/Sは押下時のObject ID集合へだけ展開され、全ON / 全OFF / 混在を集合から導出する。適用後の再packingで別ObjectへM/Sが移らず、rail名、Lane ID、Lane保存状態を生成しない
16. rail右端と時間面左端、rail header下端と時間ruler下端、全rail行下端と対応guide下端が一致する。1帯をresizeした後もその帯以降の境界が双方で一致し、他帯の高さは変わらない
17. `◆ n`の件数と展開行がAutomation済みchannel集合だけから導出され、未使用parameterを混ぜない。複数channelのkeyを同時に表示・選択でき、Object本体clickの選択意味を変えない
18. Automationを持たないObjectにも`＋ Automationを追加…`へ到達できる明示入口があり、検索確定前はDocument変更ゼロ、確定時だけ対象Object / channelへ1 command / 1 Undoとなる。shortcutなしで同じ動線を完了できる
19. Keystone型操作面は必要sectionだけを開閉でき、`Object別 / Channel別 / 全選択`を画面上で選べる。選択key以外を変更せず、Align / Stagger / Stretch後も各keyのEasingを保持し、1操作を1 Undoにする
20. Key ToolsはTimeline右端dockを既定にしつつ、panel systemで移動・split・tab化できる。KEYSではLayer操作、LAYERSではKey操作を同時表示せず、mode切替だけでDocument、選択、Undoが変わらない
21. Groupを畳むと同じ時間面のGroup bar 1本になり、開くと同じStage、playhead、選択IDを保ってchildがその場へ展開される。double clickで別Composition、別Preview、別Timelineを生成せず、戻るための`Esc`を要求しない
22. Group barのS/M操作はGroup出力を対象にし、childのS/M保存値を一括変更しない。Group Solo中も配下はaudibleで、child Solo中は親Group出力を通す。展開後は各child自身のS/MとAutomationへ到達できる
23. 展開childの帯背景だけが親Groupの背景色を継承し、child barは自身の色を保つ。種類icon、indent、親label、接続guideも残り、色覚や低彩度表示でも親子関係を失わない

次のいずれかが必要に見えた時は、UI実装を止めて本書とM3仕様を先に改訂する。

- 固定Track/Laneを公開型またはDocumentへ追加する
- packing位置へ意味を持たせる
- Groupラベル専用の永続状態を追加する
- 通常Groupのために別Composition、Preview navigation stack、子専用Timelineを追加する
- Timeline側へInspectorと同じ設定編集面を複製する
- Inboxへ全履歴、全asset、全note、全jobを恒久保存する

## 9. 関連文書

- [UIコンセプト](ui-concept.md)
- [UI操作言語](ui-interaction-language.md)
- [UI視覚言語](ui-visual-language.md)
- [M3高密度メインUIモック](mocks/README.md)
- [M3 UI境界汚染の予防](reviews/2026-07-14-m3-ui-boundary-prevention.md)
- [M3 UI統合仕様](specs/M3-ui-integration.md)
