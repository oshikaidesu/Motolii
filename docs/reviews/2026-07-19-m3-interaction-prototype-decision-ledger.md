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
| P16 | Group bar double click | 一段入る、Escで出る転移仮説と比較 |
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
| P48 | Browser階層railの表示と横幅 | **採択**。Sources / Registered folders / Collections / Packsを含む左階層は境界dragで横幅を変更し、左端までdragすると閉じる。閉じた後は現行の縦型`HIERARCHY`入口から再表示する。幅と開閉はWorkspace-session候補で、Results、検索、選択、Document、Undoを変えない。開いている間は常設popを重ねず、狭いBrowser幅ではResultsを優先する |
| P49 | Browser / Inspector / Timelineの可変サイズ | **採択**。3面はpointerとkeyboardで独立resizeでき、Stageは残余領域へ追従する。最小Stage幅を保ち、double click / Homeで組み込み初期値へ戻す。寸法はWorkspace-session候補でDocument・Undo・plugin契約へ入れず、Motolii所有layout modelから`egui_tiles`へ投影する |
| P50 | Browserのユーザーtag分類 | **採択**。Media / Effects / Createのtag表示はEffects型の左階層UIへ統一し、itemをtagへdropして分類、tagからResultsを絞り込む。共通化するのはUIと操作文法だけで、tag名と割当はitem種別ごとに分ける。Host taxonomyのType / Providerやplugin manifestの分類とは別で、DocumentとUndoを変えない。Workspace metadata候補だが恒久保存形式はこのモックで決めない |
| P51 | Browser thumbnail寸法の共通化 | **採択**。既存Settingsの`Plugin thumbnail size`を重複controlを足さず`Browser thumbnail size`へ一般化し、Media / Effects / Createの結果gridへ同じ値を投影する。User settings候補でDocument・Undo不変。React固有stateやpx値をDocument・plugin契約へ焼かない |
| P52 | UI labelの折り返し | **採択**。見出し、階層label、tag、card名、状態文は原則1行固定で折り返さず、幅を超えた部分をelideする。全文や詳細は既存の下部Info / tooltip / focusから読む。文字列長で行高やpanel配置を変えない |
| P53 | Browser result cardとView modeの共通化 | **採択**。Media / Effects / Createは`thumbnail-only / thumbnail+name / list`の3表示を共通View toggleから選べる。card内はitem名へ先に幅を割り当て、tagは残余幅だけでelideする。Effects cardの高さはBrowser共通thumbnail寸法へ一致させ、旧Plugin専用寸法の空白を残さない。選択表示は1 itemだけにする |

P47の状態は**比較中**。非目標:

- Timeline Loopの既定値、保存範囲、Document意味を本モックから変更しない。
- Clip Trimのcommand、Undo、素材参照契約を作らない。
- preview render／transcode API、worker構成、cache寿命を決めない。
- GIF／動画を正準保存形式にせず、具体duration、frame rate、解像度を製品tokenへしない。
採否条件は、Browser保存sheetとTimeline Loopの2 fixtureで、同じrange操作を説明なしに使えること、Escで各ownerが不変へ戻ること、片方のCommitが他方の状態を変更しないことを再現可能に確認すること。Clip Trimは第三の意味を混ぜず、共有primitiveの適用可能性だけを後続比較する。

### 固定React baselineの補助観察（U0e-2R）

固定commit `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0`には、帯アクションrail、
Object barのS/M、Automation channel展開、Key Tools、同一時間面のGroup展開、
Interval Easing Editorとmulti-key Graph Viewの比較実装がある。branch側台帳は
これらへP53〜P62を再利用していたため、mainのP53〜P61と衝突するIDを現行P項目へ
統合しない。実装は比較証拠として保持するが、状態は**観察**であり、mainの
P53採択とP54〜P61未決を上書きしない。製品採択には一意な新ID、状態所有、
command/Undo、M3仕様と譜面UI構成モデルの先行改訂が必要である。

### 移行者の安心（AE / AviUtl。U4d / U2c / U1i / U0d / U1c系、2026-07-19追補）

出所は[AE痛点カタログ](../ae-pain-points.md)、[AviUtl2一次声台帳](2026-07-17-aviutl2-comment-voices.md)、[受容先例調査](2026-07-16-m3-ui-rapid-acceptance-prior-art.md)。一次声・先例は**問いの出所であり採択根拠ではない**（レビュー規律どおり反対側審判を経る）。P54〜P61は全て**未決**。

| ID | 決めること | 出発仮説 / 審判 |
|---|---|---|
| P54 | 欠落pluginのproject単位診断 | C6のpointer位置拒否を超え、「このprojectは欠落pluginがあっても開ける・どのclipが影響を受ける・入手/差し替え/復元の入口」をBrowser `Used`絞り込み、Inbox、U1i Activityのどの既存面へ投影するか比較。install契約・Marketplace・自動取得は発明しない。審判: 欠落を含むprojectを開いた直後、影響clipへの到達と「開けるが該当効果は出ない」の理解が説明なしに成立すること |
| P55 | 文脈ヘルプの席 | [ui-concept](../ui-concept.md)決定済みの文脈ヘルプ（名前・できること・shortcut・いまできない理由）を候補モックの下端Brief帯へ投影する。新規面は足さず、P39のBrief / Context優先順位と同じ席・同じ遅延定数を使う。審判: 既存Brief表示と衝突せず、disabled対象の「いまできない理由」がhover / focus両方で読めること |
| P56 | 再生可能区間の表示 | cache済み / 未計算 / realtime可の区間をTimeline rulerへ表示する比較。先例: AE RAM previewバー、AviUtl2 cache再生（一次声V1）。cacheの実装・寿命・粒度はperformance-model側の正本で、本モックは視覚文法（位置・色相当・reduced-motion時の静的表現）だけ比較する |
| P57 | 検索の他ツール語彙エイリアス | Browser検索がitem正式名だけでなく他ツール通称・日英表記（例: Glow / グロー / 発光、AE・AviUtl・AM系の呼び名）で該当itemへ届く比較fixture。alias辞書の持ち場（Host taxonomy / plugin manifest / Workspace）は未決で、本モックからmanifest形式へ焼かない。審判: 移行者語彙の代表setで0件にならず、P28の0件回復と整合すること |
| P58 | keymap presetの同梱 | U0d（base + user delta、全shortcut再割当）の上へAE風 / AviUtl風 / Premiere風等のpresetを同梱するかの未決。選択入口（初回起動 / Settings）、全presetが同一command集合を発行できるconformance、presetをuser delta形式のまま表現できるかを比較。物理keyをDocumentへ入れない既決は不変 |
| P59 | transport / rulerの時刻表記切替 | BAR/BEAT・timecode・frame番号の切替または併記をUser settings候補として比較。Documentの時間意味、SNAP BEAT等のsnap既定は変えない。出所: AviUtl圏のframe単位への信頼（frame単位audio scrubbingが移行決定打になった一次声V5）とAEのtimecode筋肉記憶。音楽座標を主役から降ろす決定ではない |
| P60 | 素材drop時のcodec即時診断 | OS dropとMedia importで「そのまま使える / 変換が要る / 非対応」をtyped即時回答し、回復入口（変換・音声のみ抽出等）を提示する比較。P5のOS drop scope、U1i Activity、M1 media / exportガード群へ接続。技術判定はmedia側正本で、本モックは文言と表示席（Brief / Inbox）だけ比較。出所: AviUtl2初期MP4非対応の一次声V3 |
| P61 | 保存・復元状態の見える化 | 最終保存時刻・自動保存の有無・復元入口の表示。前提: 保存方針とworkspace復元UX（GR-UI-1で保留中）の決定が先行し、本モックは表示席（chrome / Activity）だけ比較する。実装保証より先に「復元できます」表示を作らない。出所: AE痛点A群（安定性・作業消失） |

**新規panelの扱い**: P54〜P61は全て既存面（Browser、Inspector、Timeline、下端Brief / Info、Inbox、U1i Activity、Settings）への投影を第一候補とし、独立panelを既定で足さない。panel形状への圧力が最も強いのはP54（project診断）だが、まずInbox / Activityへの投影で審判し、投影では成立しない再現可能な証拠が出た場合だけ独立panelを比較対象へ昇格する。P49でlayoutは可変・拡張可能なので、v1既定でpanelを増やさないことは将来の追加を妨げない。

移行の非目標も同時に固定する: 旧project（.aup / .aep等）の完全変換は約束しない。救済対象の比較は素材・timing・easing・preset・plugin設定の粒度で行い（一次声台帳の問い4）、「単純cut・実写多素材・字幕主体はPremiere / Resolve / YMM4等が適する」という用途境界の説明自体を安心材料として扱う（同・問い9）。

## 判定方法

各P項目はprototypeごとに次を記録する。

1. 対象fixture、操作列、viewport / scale、入力device。
2. Document / Undoを変える時点、Cancel後の不変条件。
3. valid / invalid / partial-invalid、keyboard / pointer、reduced-motionの負例。
4. 採択、延期、棄却と根拠。先例だけで採択しない。
5. 公開契約、永続形式、px / duration tokenが必要なら停止し、対応仕様改訂へ戻す。
