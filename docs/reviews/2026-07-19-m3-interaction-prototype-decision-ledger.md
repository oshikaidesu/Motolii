# M3操作prototype未決パラメータ台帳（2026-07-19）

ステータス: **比較仮説台帳**。本書は製品仕様、公開API、永続形式、既定値の決定ではない。採否は各M3タスクの仕様改訂と再現可能なprototype審判で行う。

## 履歴からの回収

2026-07-19のCodexタスク`M3を試す`では、`#plugin-browser-candidate`を見ながらBrowser分類、Pack、複数actionを持つPreset、poster／motion previewを逐次検討した。会話そのものは正本にせず、再利用できる比較仮説をP41〜P46へ回収した。

- `Media / Create / Effects`はaction viewの比較仮説であり、package kindや永続分類の決定ではない。
- Packは管理identityと利用時のaction投影を分ける仮説であり、Pack形式、install API、Marketplace契約を決めない。
- AE型Presetのkeyframe／effect／expression保持は先例上の問題提起であり、現行Motoliiの保存recipe意味を決めない。
- poster必須＋任意motion previewはBrowser resourceの比較仮説であり、GIFや動画を正準保存形式にしない。
- motion previewの区間選択とTimeline Loop／Clip Trimの共通化はP47の比較対象であり、会話時点では採択しない。
- 会話の途中案、旧HTMLへの同時変更、React画面に表示されたという事実だけでは採択済みにしない。

React移行の所有境界は[M3 UI参照地図](../ui-reference-map.md)を正本とする。

## 目的と境界

静的goldenに写らない操作パラメータを、既に決定済みのUI文法や登録済みgapと混同せず、操作prototypeの比較単位へする。推奨欄は先例から得た出発仮説であり、実装根拠そのものではない。

前提:

- UI状態、Document、User settings、Workspace/session候補、Transientの所有境界は[UI境界汚染予防](2026-07-14-m3-ui-boundary-prevention.md)に従う。
- 選択、D&D、Brief / Context、拒否理由、Undo / Redo等の決定済み文法は[UI操作言語](../ui-interaction-language.md)を再審理しない。
- Timelineの縦位置は意味ではなくpacking結果であり、Inboxは第二のasset所有者ではない（[譜面UI構成モデル](../ui-score-model.md)）。
- 本台帳からpx、duration、modifier、transaction型をDocument、plugin契約、製品公開APIへ焼かない。

## 現候補モックの自己矛盾と処置

| # | 食い違い | 候補モックでの処置 | 製品決定 |
|---|---|---|---|
| C1 | Plugin Stage dropがカーソル下targetでなく選択中objectへ適用 | object上だけvalid outline、空白dropはCancel | U2c / U4dでpreflightとhit-testを決定 |
| C2 | folderのStage dropが無言拒否 | カーソル近傍へ「folderは配置不能・中のfileを選ぶ」 | typed reasonの型はU2c |
| C3 | drag Contextが下端Briefだけ | カーソル近傍Contextを追加し、下端はBriefに限定 | 時間定数は本書P39 |
| C4 | Timelineがdrop先として見えない | asset ghostとplugin bar targetの比較fixtureを追加 | 時間写像・packingはP2 / P3 |
| C5 | commit後handoffが無い | Stage bounds / Timeline barへ一時outline。reduced-motionは静的outline | durationはP40 |
| C6 | Missing / Unavailableのdrag試行が無反応 | pointer開始位置へ理由とrecovery入口を表示 | install契約は発明しない |
| C7 | Redoの席が無い | candidate chromeへUndo / Redoと対象labelを表示 | command意味は既存D2を投影 |
| C8 | 標準shape / layerを`Create`固定棚へ置くと第三者拡張が分類外になる | `Elements`をtype / providerで投影する登録カタログとして比較。Built-inも一provider扱い | registry形状、plugin参加契約、標準項目のdomain型は本モックから発明しない |

これらは`#plugin-browser-candidate`の比較fixture修正であり、製品egui実装許可や採択を意味しない。

## Prototype決定台帳

`出発仮説`は比較の最初の候補であり、採択値ではない。

### D&D（U6 / U2g / U2e）

| ID | 決めること | 出発仮説 / 審判 |
|---|---|---|
| P1 | drop先scopeの全列挙 | Stage=配置、Timeline=時間指定、Tag box=整理、Browser Project=import。HoverValid / HoverInvalidを再利用 |
| P2 | Timeline dropの時間写像 | in点=cursor時刻、Snap有効時はgrid吸着、drag中ghost bar |
| P3 | dropとpackingの整合 | drag中から確定packing位置へghostを置き、release時に跳ばない |
| P4 | Stage dropのXYと時刻 | drop XY、現在playhead時刻を第一候補 |
| P5 | OSからのdrop scopeと進捗 | Stage / Timeline / Browserを比較。進捗はU1i Activity、未確認物だけInbox投影 |
| P6 | 複数選択drag | 件数badge付きghost。型不一致混在は部分適用せず全拒否 |
| P7 | edge auto-scroll / spring-load | panel端scroll、folder hover-openを約600ms候補として実測 |
| P8 | drag modifier | keymap経由。複製等の意味を物理keyへ直書きしない |
| P9 | drag開始閾値 | clickとの弁別を約3px候補からG0-6で実測。grab cursor併用 |
| P10 | Cancel保証 | Esc / capture loss / target外drop=変更ゼロをconformance fixture化 |

### Timeline直接操作（U3b / U3e / U7 / U2h）

| ID | 決めること | 出発仮説 / 審判 |
|---|---|---|
| P11 | trim handle hit幅と優先順位 | key > trim > bar本体。trim中はin / out HUD |
| P12 | ripple既定 | 自動close寄りとgap保持を比較し、User setting候補。強制しない |
| P13 | bar移動中packing | drag中は他barを凍結、commit時だけrepackを第一候補 |
| P14 | snap視覚と距離 | snap線flash、距離は実測、temporary disableはkeymap |
| P15 | Timeline空白操作 | click=選択解除、drag=marquee |
| P16 | Group bar double click | **棄却**。通常Groupを別Composition／別Previewへ一段入る入口にせず、同じ時間面でfold / unfoldする。戻るための`Esc`とnavigation stackを作らない |
| P17 | key hit / move / add / delete | diamond hit拡張、直接時間drag。追加gestureはautomation文脈と同時決定 |
| P18 | playhead hit / scrub / follow | ruler hit高を広げ、click jump / drag scrub。followはpage / smooth / off候補 |
| P19 | zoom / scroll | cursor中心zoom、pinch、Fit All / Selection。wheel mappingは比較 |
| P20 | Inbox増減とclick予告 | 増加1回強調、除去fade、hoverでseek等の結果を予告 |

### 数値scrub（U4a）

| ID | 決めること | 出発仮説 / 審判 |
|---|---|---|
| P21 | click / drag弁別 | 閾値未満click=全選択type、Enter確定 / Esc破棄 / Tab移動 |
| P22 | 精度modifier | 粗い / 細かい操作をkeymap intent化し、dial流速と同期 |
| P23 | 画面端 | pointer lockとedge wrapをmulti-monitor / trackpadで比較 |
| P24 | reset | context action「既定値へ」を候補にautomation時意味も同時決定 |
| P25 | 有界 / 無界widget写像 | C-2解凍後、型からfinite bar / dialを機械選択 |
| P26 | automation mark hit領域 | 行高全体へ広げ、Fitts審判へ追加 |

### Discovery Browser（U4d / U6 / U2h）

| ID | 決めること | 出発仮説 / 審判 |
|---|---|---|
| P27 | filterの暗黙reset | sourceを黙ってAllへ戻さず、Results headerへ条件chipと個別clear |
| P28 | 検索0件 | 現scopeの0件と、All / 別typeへ広げる回復候補 |
| P29 | plugin hover preview | loop preview、reduced-motionは静止+明示Play |
| P30 | 複数選択入口 | Select mode + Cmd / Shift。Escでmode解除 |
| P31 | view切替 | tooltip、選択 / scroll保持。Ctrl+wheelは同じthumbnail User settingへの入口候補 |
| P32 | card内taxonomy誤爆 | hover / focus時だけlink外観を昇格してcard選択と弁別 |
| P33 | Tags primary配置 | 配置Commitより弱く、常設Tag shelfと選択後menuの役割を比較 |

### Stage / Inspector（U1f / U2d / U2h / U2e）

| ID | 決めること | 出発仮説 / 審判 |
|---|---|---|
| P34 | Stage hover輪郭 | 選択前のDiscoverとしてhit輪郭を予告 |
| P35 | object drag snap / guide | frame中心・端のsmart guideと吸着距離を実測 |
| P36 | view入力 | cursor中心wheel zoom、Space+drag Hand、pinch、pasteboard click解除 |
| P37 | enum / Input affordance | enumはselect形状、Inputは接続picker入口として表示 |
| P38 | Easing double click遅延 | 遅延、modifier、楽観openの3案を最頻single click latency込みで比較 |

### Feedback横断（U2c / U1i）

| ID | 決めること | 出発仮説 / 審判 |
|---|---|---|
| P39 | Brief / Context優先順位とhover遅延 | focus即時、hover約400ms、隣接controlはwarm no-delay候補 |
| P40 | handoff / 消滅の時間定数 | commit outline、Inbox fadeをmotion token候補で比較。reduced-motionは静的outline |

### Action view / Pack / Context command（U2h / U4d）

| ID | 決めること | 出発仮説 / 審判 |
|---|---|---|
| P41 | Browserのaction view | Media / Create / Effectsを比較。Create=新規Object、Effects=既存targetへの適用。providerやpackage種別を一次navigationにしない |
| P42 | 右クリックの責務 | 独自編集経路を持たず、同じCommit Intentの文脈入口に限定。選択済み項目のmenuとStage空白の`Add`短縮menuを比較し、Browse allはCreateへ戻す |
| P43 | Packの一体管理 | Packは一商品identityのまま管理し、選択中Pack scopeを保持してMedia / Create / Effects収録数と各viewへ切り替える。Pack形式やinstall APIは本モックから定義しない |
| P44 | dual-action Item | 同じstable identityが新規作成と既存target適用を提供できる比較fixtureを置く。投影はaction別、Favorites・履歴・Pack所属はidentity共有。自己申告型や公開manifestは未決 |
| P45 | animation presetの構造影響 | keyframe / expression / effectを含む適用候補は、語の分類ではなく「何を作る／変更するか」をiconで予告する。時間基準、merge / replace、1 Undo境界は仕様決定まで未決 |
| P46 | 保存候補のthumbnail | posterは必須で自動生成する。motion previewは任意で、Auto 2秒／Record／GIF・Video読込／posterのみを比較する。再生はhover／focus時だけ、reduced-motionは静止。previewはBrowser resource / cache候補でDocumentとUndoを変えず、GIF等を正準保存形式へしない |
| P47 | Preview Rangeと既存range操作の共有境界 | Browser保存sheetにIn／Out、playhead、選択範囲loop確認を持つ小型fixtureを置き、Timeline LoopとClip Trimに同じ視覚・pointer／keyboard文法を適用できるか比較する。共有候補はrangeとplayheadの入出力まで。Browser Preview Range=保存前Transient、Timeline Loop=再生session、Clip Trim=Document＋Undoという所有とCommit意味は統合しない |
| P53 | Timeline横時間軸とpacking帯ごとの間隔 | **比較中**。`#plugin-browser-candidate`をReact-native Timelineへ置き換え、一枚のpacking時間面の上端へ高contrastなBeat ruler、major / minor tick、横scrollを置く。bar背面には所有者ではない水平packing guideを常時表示する。設定toolbarと全帯共通controlは追加しない。Inbox右隣の細い無名帯アクションrailへ各guide境界の小さな二本線gripと広いhit領域を統合する。rail右端と時間面左端、rail header下端とBeat ruler下端、各rail行下端と対応guide下端を同じ座標にし、独立した行高を持たせない。hover / focus時は対応する横線全体を強調し、上下dragした帯だけの間隔とbar高を変更する（各30〜46のprototype範囲）。境界以外のguide面はdragを受けない。値と保存寿命はWorkspace-session候補であり、Document、Undo、製品token、egui公開型へ焼かない |
| P54 | Object barと帯アクションrailのSolo / Mute | **比較中**。各Object bar自身へ形と文字を併用したS/M controlを置く。加えて無名帯アクションrailのS/MはLane状態を持たず、押下時に同じpacking帯へ載る全Objectの既存S/Mへ一括適用する。表示はObject集合の全ON / 全OFF / 混在から導出し、再packingだけでは状態を別Objectへ移さない。Solo中は非対象barをdim、Muteは対象barをdim＋破線で示す。音声だけのTrack契約へ限定せずObject投影として比較するが、評価意味、親子伝播、Group、複数Solo、再生／書き出し一致、commandと保存先は本モックで決めない |
| P55 | Inboxを説明文でaction queueとして読ませる | **棄却**。`NEEDS ACTION`、`OPEN`、空状態の説明文は、用途を読んで理解させるだけで操作の因果を改善しなかった。「ここで意図を説明してようやく分かるUXは失敗」という実機評価により比較を終了する |
| P56 | Automation channelの発見・展開とKey Tools | **比較中（focus-only popover案は棄却）**。AEの良い一覧性を残し、Object bar内の`◆ n`から実際にAutomationを持つchannel行だけをbar直下へ縦展開する。複数channelのkeyを同時に表示・選択し、Keystone 3型のmodular操作面へ渡す。操作面はTimeline右端へ既定dockする独立`Key Tools` Viewとし、panel systemで移動・split・tab化可能にする。内部は`KEYS / LAYERS`を排他的に切り替え、KEYSでは`Object別 / Channel別 / 全選択`を明示してAlign / Stagger / Stretch等を適用し、既存Easingを保持する。LAYERSでは選択Object向け操作だけを表示し、両modeの全sectionを同時表示しない。BrowserのPreset棚とInspectorのparameter面は占有しない。未使用channelは末尾の`＋ Automationを追加…`から検索して追加し、0件Objectも`◇＋`入口から到達できる。Object本体clickは選択のまま保ち、全parameter行、固定名列、shortcut暗記を要求しない。prototypeの追加はReact stateだけで比較し、Document schema、command、default key生成、削除意味を決めない |
| P57 | Easing Graphの縦表示範囲とOvershoot識別 | **採択**。curveやhandleの値から表示範囲を動的fitしない。Overshoot OFFは標準固定範囲、ONはmanual handleとElastic limitの最大可動域を最初から収める固定範囲へ一度に切り替える。同じOvershoot状態のparameter変更では0..1 guide座標を不変にする。モードは説明文やON/OFF文字ではなく、枠内curve／上限越えcurveの専用ピクトグラムとpressed状態をグラフ上へ常設して示す。Document、補間parameter、Undoを表示範囲から変更しない |
| P58 | Inboxの直接utility化 | **棄却**。直接actionへ変えてもasset、review note、background jobを一つの場所へ混ぜる根拠は生まれず、一次ユーザーがTimeline横で行いたい仕事へ一致しなかった。未配置assetはSources、jobはstatus / diagnostic、review noteは用途未採択のままとし、Inboxを別名で残さない |
| P59 | Groupの表示と編集文脈 | **採択**。通常Groupは同じStageとTimelineで、畳めばGroup bar 1本、開けばchildをその場へ展開する。親Groupの背景色は展開帯だけへ継承し、child barは自身の種類・identity色を保つ。種類icon、indent、親label、接続guideも残して色だけに依存しない。Group barには名前、child数、通常Objectと同じS/MとAutomation要約を置く。S/MはGroup出力の試聴・隔離、展開はchild選択・並べ替え・Automation編集に使い、Group操作からchildのS/M保存値を複製しない。Group Soloでは配下をaudibleに投影し、child Soloでは描画に必要な親Groupを通す。double clickまたはfold iconは開閉に使い、別Composition／別Previewへの遷移、戻る`Esc`、navigation stackを作らない。将来のGroup Definition/UseやCompositionClipは通常Groupと別の再利用source機能として判断する |
| P60 | Key ToolsのTimeline左dock | **採択（prototype配置）**。Inboxを削除し、右端の`KEYS / LAYERS`を同じ左端幅へ移す。選択したkey / Objectに対するAlign / Stagger / Stretch / Shiftを操作面、その直右の一枚の時間面を結果面として読む。右端を時間面へ返し、Timelineの有効横幅を増やす。これは組み込みprototypeの配置判断であり、plugin custom panel API、固定window座標、Document保存形式を追加しない |
| P61 | 3点以上のイージングを作るGraph View | **停止**。MultiEaseは単一の3点Bezierではなく、複数カーブと点編集から適用時に中間keyframeも生成するが、その需要はAE Graph Editorの扱いにくさを回避した結果かもしれず、3点以上が本質要件だとはまだ言えない。まずInterval Easing EditorとP62のmulti-key Graph Viewで制作操作を十分に行えるか比較し、それでも解けない具体例が残るまで複数点専用モードを実装しない。必要性が実証された場合だけ、previewはTransient、明示Applyで通常keyframeへmaterializeし1 macro／1 Undoとする案を再比較する。1区間内の恒久knot、新しい`Interp`制御点配列、runtime script、公開preset形式は本モックで定義しない |
| P62 | Interval Easing Editorと別のmulti-key Graph View | **採択（React prototype比較中）**。`#graph-view-candidate`へ、Apple Motion型のparameter list＋実時間×実値graph、Cinema 4D型のtangent拘束、Maya型のIn/Out状態、Blender型のFrame Selected／snapshotを比較実装する。focus channelは太い実線とkey、context channelは薄い破線で投影する。drag中auto-fit禁止、Frame Selectedだけが明示的にviewを変え、curve上のkey追加はde Casteljau分割でshapeを維持する。Interval Easing Editor、空間Motion Path、複数点専用補間とは別surface。React state、SVG座標、snapshot、view range、absolute tangentをDocument・公開APIへ焼かない |

P47は**比較中**、P58は**棄却**、P60は**採択（prototype配置）**。非目標:

- Timeline Loopの既定値、保存範囲、Document意味を本モックから変更しない。
- Clip Trimのcommand、Undo、素材参照契約を作らない。
- preview render／transcode API、worker構成、cache寿命を決めない。
- GIF／動画を正準保存形式にせず、具体duration、frame rate、解像度を製品tokenへしない。
- Inboxをasset tree、通知履歴、command launcher、別名の汎用queueとして復活させない。

採否条件は、Browser保存sheetとTimeline Loopの2 fixtureで、同じrange操作を説明なしに使えること、Escで各ownerが不変へ戻ること、片方のCommitが他方の状態を変更しないことを再現可能に確認すること。Clip Trimは第三の意味を混ぜず、共有primitiveの適用可能性だけを後続比較する。TimelineではInboxが存在せず、左`KEYS / LAYERS`の右隣に帯アクションrailと時間面が連続し、右端まで時間面として使えることを確認する。

P53 / P54 / P56の状態は**比較中**、P59は**採択**。非目標:

- 固定Track / Lane、左Object rail、固定名列、1項目1横行を復活させない。左端は選択中のkey / Objectへ作用する`KEYS / LAYERS`と、名前・所有状態を持たない細い帯アクションrailだけにする。
- packing帯ごとの間隔とbar高の具体値、段階数、保存形式、project / global帰属を決めない。各境界gripは無名帯アクションrailへ統合し、対応する帯だけを変え、packing位置以外の意味を変えず、時間面全体や全帯をresize対象にしない。gripの見た目よりhit領域を広くし、名称labelや常設設定欄を追加しない。
- S/Mの評価順、Group親子伝播、音声mix、preview / export、Undo、journalをReact stateから定義しない。
- 通常Groupのfold / unfoldから別Composition、別Preview、子専用Timeline、navigation stackを作らない。Group Definition/UseやCompositionClipの再利用source意味を先取りしない。
- Automation可能な全parameterをTimelineへ事前展開しない。Object本体clickへ選択とAutomation展開を重複割当せず、shortcutを唯一の入口にしない。prototypeの追加操作からDocument field、default key、Undo意味を発明しない。
- 旧HTML archiveへ同じ操作を逆実装せず、React候補と操作試験だけを更新する。

採否条件は、1440×900で固定名列なしにObject名・bar上S/M・時間位置を同時に読めること、水平guideをbarが無い区間でも追えること、無名帯アクションrail各下辺のgripが見た目より広いhit領域とresize cursorを持ち、hover / focusした線だけを強調し、1本を最小／最大へ変えても他の帯高を変えずbar同士が重ならないこと、railのM/Sが同じ帯の全Objectへ一括適用され全ON / 全OFF / 混在を集合から表示すること、Solo / Muteの対象と非対象を色だけでなくpressed状態・明度・線種で識別できること、横scroll後もBeat rulerとbarの時刻対応を追えること。`◆ n`の件数と一覧がAutomation済みchannelだけから導出され、channel選択で1 channelのkeyだけがfocus表示され、0件Objectからも検索追加へ到達できること。Group開閉の前後でStage、playhead、選択IDが不変で、childが同じ時間面へ展開されること。展開帯だけが親背景を継承し、child barは固有色と非色手掛かりを残すこと。Group Soloでchildが暗転せず、child Soloで親出力が遮断されず、いずれもchildのS/M保存値を書き換えないこと。構造検査で`OBJECTS`列、設定toolbar、全帯共通control、名称label、Lane所有状態、全parameter行、通常Group用の別Composition／Preview navigationを拒否する。製品採択時は状態所有と評価意味をM3仕様で別途決定する。

## 判定方法

各P項目はprototypeごとに次を記録する。

1. 対象fixture、操作列、viewport / scale、入力device。
2. Document / Undoを変える時点、Cancel後の不変条件。
3. valid / invalid / partial-invalid、keyboard / pointer、reduced-motionの負例。
4. 採択、延期、棄却と根拠。先例だけで採択しない。
5. 公開契約、永続形式、px / duration tokenが必要なら停止し、対応仕様改訂へ戻す。
