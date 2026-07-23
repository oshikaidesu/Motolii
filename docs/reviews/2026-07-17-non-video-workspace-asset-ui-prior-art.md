# 動画ソフト外から引き直すWorkspace・素材探索・視線設計

日付: 2026-07-17
状態: **先例調査・Motoliiへの翻訳仮説**。M3製品実装、公開UI API、Document schema、Vism package形式の許可・決定ではない。
## 1. 調査の転換点

これまでのメインUI検討は、AE、AviUtl、NLE、DAW等の既存制作UIを起点に、固定Track/Lane、Inspector、Graph Editor等をMotoliiへどう縮約するかを主に扱ってきた。しかし現在の論点は、既存動画UIの部品配置では解けない。

- 外部素材の探索は例外的なImport操作ではなく、制作中に繰り返す主作業である
- Import済みAssetと、未Importの外部候補と、Timeline上の配置Instanceは意味が異なる
- 自由なwindow配置だけでは「最初にどこを見て、操作後どこを見るか」を設計できない
- Timeline、Inspector、Architectに既に家がある情報を、空いたpanelへ重複表示してはならない
- ドキュメントを読まないと視線の入口が分からないUIは、Motoliiの目的に反する

したがって本調査は、写真管理、3D／ゲーム制作、CAD、IDEから、次の三点を検索する。

1. 素材探索を制作の主画面へどう接続しているか
2. task別Workspaceとユーザー自由配置をどう分けているか
3. layoutではなく視線の主役と操作後のhandoffをどう作っているか

## 2. 先例比較

| 先例 | 観察できる仕組み | Motoliiへ移せる点 | そのまま移さない点 |
|---|---|---|---|
| Lightroom Classic | `Library`はimport・整理・比較・選択、`Develop`は編集というtask別module。Libraryで選んだ集合はFilmstripとして全moduleに残る | 素材探索を独立した主taskに昇格しつつ、選択中素材の小さなcontinuity surfaceだけをAnimateへ持ち越す | module切替で制作文脈を全面置換し、StageとTimelineをdrop先から消す構成 |
| Godot | `AssetLib`が`2D / 3D / Script`と同格のmain screenにあり、外部資源の探索をEditor内の主作業として扱う | 外部素材探索をFile dialogより上位の制作面として扱う | AssetLibは主にaddon/package探索であり、media AssetとTimeline Instanceの所有契約は別途必要 |
| Unreal Engine | 同じContent Browser系UIを、focusを失うと畳まれる`Content Drawer`、Dockした常設Browser、floating windowとして出せる | 一つのSources componentを一時Drawer／Dock／floatへ投影する。狭い別実装を作らない | 最大4個のContent Browserを別filterで開く設計。Motolii v1で複数selection正本・複数preview stateを許さない |
| Logic Pro | Project Audio BrowserとAll Files Browserを分離し、Project内素材と外部filesystemの意味を同じmain window内で区別する。active window／areaにはkey focus表示がある | 一つのMedia BrowserからProjectと登録folderを選び、結果上ではProject登録状態を明示する | Screenset番号と任意window集合だけを初心者の主要導線にすること |
| Blender | WorkspaceはArea/Editorの組合せで、Modeling、Animating、Scripting等のtaskに向けたpresetとして切り替える | Workspace名をpanel配置ではなく制作意図で定義する | predefined layoutだけで操作開始点・commit後の視線移動まで解決したと見なすこと |
| Fusion | capabilityをpurpose-driven Workspaceへ分け、commandを起動した文脈でだけ現れるcontextual tabを持つ | 低頻度toolを常設せず、明示command後に必要な操作だけを昇格する | tool分類をDocument型やplugin kindの閉集合へ直結すること |
| Maya | workflow別のfactory Workspaceとcustom Workspaceを持ち、factory defaultへresetできる。workspaceはuser directoryに保存されscene fileへ入らない | layoutはUser／Workspace state、作品はDocumentという境界、`Reset Workspace`、factory presetを採る | どのpanelも自由にdockできること自体を視線設計と呼ぶこと |
| IntelliJ IDEA / VS Code | Distraction-free／Zen modeは主Editor以外を明示的に隠す。IntelliJのFind tool windowは結果がない時は現れない | 主役を一つにし、結果や作業が存在する時だけ補助面を昇格する | 重要な制作入口をhover、shortcut、完全非表示へ追いやること |

## 3. 強く収束したパターン

### 3.1 Workspaceはwindow座標ではなく制作意図で名付ける

Blender、Fusion、Lightroomはいずれも、ユーザーへ最初に提示する切替単位を`左panelが広いlayout`のような幾何ではなく、`Library / Develop`、`Modeling / Animation`、`Design`等の作業目的で表す。

Motoliiでも、製品が提供する標準Workspace候補は次のような意図名にする。

- `Animate`: StageとTimelineが主役
- `Source`: 外部候補とProject Assetの探索が主役
- `Color`: 色面と対象parameterが主役
- `Architect`: 接続・依存・scopeが主役
- `Inspect`: Advancedな由来・identity・異常検査が主役

任意dockはこれらの代わりではなく、完成した標準Workspaceを個人環境へ調整する拡張である。

### 3.2 一つのSources componentを複数の密度へ投影する

UnrealのContent Drawer／Content Browserは、狭い素材欄と広い管理画面を別製品として作る必要がないことを示す。Motoliiでは一つのSources stateを次へ投影する仮説が最も強い。

1. **Tray**: 最近使用、Pin、Import済み未配置、missing、`Browse`入口だけ
2. **Drawer**: 明示操作で一時展開し、StageとTimelineをdrop先として残す
3. **Docked / Floating**: 素材探索を長く続けるユーザー向け

selection、query、Folder、Label、thumbnail size、preview、In/Outは同じstateを使う。形態ごとに別のAsset Browserを作らない。

### 3.3 Media Browserへ統合し、所有状態は混同しない

Logic ProのProject Audio Browser／All Files Browserは、同じ「素材を探す」操作でも所有意味が違うことをUIで分ける。

一方、AEViewer系の需要は、Import dialogやProject panelを往復せず、外部file、過去project、音声、画像をImport前にpreviewし、そのまま再利用することへ収束している。したがってMotoliiでは、`Assets`と`Explorer`を別のtop-level tabへ置かず、一つの`Media` Browserへ統合する。

Media Browserのsource railは、`All Media`、`Project`、複数の登録folder、`Collections`、`Recent`を同じ場所に置く。検索、grid/list、thumbnail寸法、preview、selectionは共通にする。ただし表示を統合しても、少なくとも次の意味は維持する。

| Source / 状態 | 意味 | Document変更 |
|---|---|---|
| 登録folder | filesystem上の外部候補。preview、検索、In/Out | なし |
| `PROJECT` | Import済みAsset。未配置も存在できる | Import／relink等の正規commandだけ |
| Timeline | Assetを参照する配置Object／Clip | 配置command |

`All Media`は検索結果を連合表示するが、同じfileが既にImport済みなら一件へdedupeし、`IN PROJECT`、`UNPLACED`、`MISSING`等の関係を形と短い状態表示で示す。再dragでは重複Importせず同じAsset identityから新しい配置を作る。CollectionsとLabelは参照を束ねるだけで、fileやProject Assetを第二の所有物として複製しない。

### 3.4 主taskを切り替えてもcontinuity surfaceを残す

LightroomはLibraryで選んだ素材集合をFilmstripとしてDevelop等へ持ち越す。これは、主画面を切り替えても「いま何を素材として扱っているか」を失わない先例である。

MotoliiではSource Workspaceを開いてもStageとTimelineを完全に消さず、少なくともpreviewとdrop先を残す。Animateへ戻った時は、直前のSources selection／queryを失わない。ただしFilmstripをそのまま模倣して新しい常設横帯を増やすのではなく、Tray、Drawer、Stage previewのどれが最小continuity surfaceかをモックで比較する。

### 3.5 自由配置にFocus Contractを追加する

Logic Proはactive window／areaを視覚的に区別し、IDEのfocus modeは主Editor以外を明示的に退ける。一方、Blender/Maya型Workspaceは主にlayoutを保存する仕組みであり、操作後の視線handoffまでは宣言しない。

Motoliiの標準Workspaceと、将来Vism／creatorが提案する専用Workspaceは、幾何情報だけでなく次のFocus Contractを必要とする。

1. Entry: 何を押すと入り、最初にどの面へfocusするか
2. Primary: 現在の主役面は一つか
3. Preview: 候補結果をどこで見るか
4. Commit target: drag／Enter／double clickの結果がどこへ現れるか
5. Handoff: commit後にどの面・Object・barを強調するか
6. Exit / Restore: Cancel、Close、Resetでどこへ戻るか

これはDocumentやVism packageへwindow座標を焼く提案ではない。Focus Contractの公開形式、provider権限、plugin由来Workspaceは未決であり、M3製品実装前に仕様化しない。

### 3.6 Progressive disclosureは「隠す」だけでは成立しない

Microsoftのguidanceは、progressive disclosureがbaselineを単純化する一方、discoverability低下とunexpectedな出現／消失による不安定さを持つと明記する。また、critical functionをHelpなしで発見できなければ、Helpの品質では第一印象を救えないとしている。

Motoliiでは次を条件にする。

- `Browse`、Graph、Depth等の入口は、閉じていても形と位置から存在を認識できる
- 開閉状態と元へ戻す方法を読む前に識別できる
- selection変更だけで大面積panelを自動展開しない
- 追加情報が高度・独立taskなら、狭いpanelへ圧縮せず十分な面へ昇格する
- よく使う入口を`More`、hover、shortcutだけへ隠さない

## 4. Motolii向けの暫定採否

### 採用仮説

1. 外部素材探索をImport dialogではなく`Source`という第一級taskにする
2. Sourcesは`Media`という一つのcomponentとし、`All Media / Project / 登録folder / Collections / Recent`をsource railで切り替える
3. Tray／Drawer／Dock／floatは同じstateの表示密度違いとする
4. 標準Workspaceは制作意図で名付け、主役面を一つにする
5. layout customizationはUser／Workspace stateで、Document、Journal、Undoへ入れない
6. factory presetと`Reset Workspace`を必須にする
7. Workspace switching／panel展開は明示操作に限り、selectionでlayoutを跳ねさせない
8. Sourcesからのcommit後は、作成されたStage Object／Timeline barへ視線handoffする

### 拒否仮説

1. Signal Path跡地の約200pxへ完全なAsset Browserを押し込む
2. `Assets / Files / Explorer`を別top-level tab・別実装・別selectionにする
3. 同じ外部FileをProject Assetとして重複Importする
4. ユーザーがwindowを並べ替えるまで主要workflowが成立しない
5. creatorへ生のwindow座標だけを渡させ、開始点・commit先・restoreを定義しない
6. 主役面を色だけで示す、または複数面を同じ強さで常時主張させる
7. `Source`へ入るとStage／Timelineが消え、previewとdrop先を見失う

## 5. 次のモック比較

製品コードへ入る前に、同じ素材fixtureで次の三案を比較する。

| 案 | 通常時 | 探索時 | 審判 |
|---|---|---|---|
| A: Tray → Drawer | 小さなSources Tray | 同じ面がStage側へ展開、Timelineは残る | 最短導線、狭い識別、layout安定 |
| B: Source Workspace | Animateでは入口だけ | Sourcesが主役、Stage previewとTimeline drop先を残す | 視線の主役、連続探索、戻り先 |
| C: Docked Sources | Userが幅を保持 | 常設の広いSources | 小画面、Timeline面積、別monitor |

共通fixture:

1. 外部videoを検索、hover preview、In/Out設定、Timelineへdropする
2. 同じfileを再度探し、`IN PROJECT`から重複Importなしで第二配置を作る
3. Import済み未配置Assetを探して配置する
4. missing Assetをrelinkし、既存配置が同じstable Assetへ戻る
5. SourceからAnimateへ戻り、直前selection／queryと作成barを見失わない
6. panel開閉・resize・float・Workspace切替でDocument snapshot／Undoが不変
7. 1280×720、1440×900、別monitor消失後のrestoreで操作不能領域を作らない

人間確認では、説明文を読まずに5秒以内で「素材を探し始める場所」「preview場所」「配置先」を指せるかを別々に記録する。自動試験はlayout state、selection identity、Document不変、重複Import拒否を担当し、人間の視線判定を偽装しない。

## 6. 調査からまだ決められないこと

- Animate既定でTrayを常設するか、入口だけにするか
- Source WorkspaceとDrawerのどちらを主経路にするか
- Dock／floatをv1へ含めるか、factory Workspace＋resizeまでに留めるか
- Sources selectionをStage selectionと常に同期するか、candidate selectionとして分けるか
- creator／VismがWorkspaceを提案できる時期と権限
- Focus Contractを内部component規約に留めるか、将来provider境界へ出すか

これらは先例だけで埋めず、上記モック比較とG0-6人間審判を先に行う。

## 7. 一次資料

- [Lightroom Classic — Workspace basics](https://helpx.adobe.com/uk/lightroom-classic/help/workspace-basics.html): task別moduleと全moduleに残るFilmstrip
- [Lightroom Classic — Library workflow](https://helpx.adobe.com/lightroom-classic/desktop/help/library-module-basic-workflow.html): import／整理／比較／選択の主画面
- [Godot — About the Asset Library](https://docs.godotengine.org/en/latest/community/asset_library/what_is_assetlib.html): AssetLibを2D／3D／Scriptと同格に置くmain screen
- [Godot — Using the Asset Library](https://docs.godotengine.org/en/stable/community/asset_library/using_assetlib.html): Editor内探索、検索、installの分離
- [Unreal Engine — Content Browser](https://dev.epicgames.com/documentation/en-us/unreal-engine/content-browser-in-unreal-engine): Content Drawer、Dock、float、sidebar
- [Logic Pro — Main window interface](https://support.apple.com/guide/logicpro/main-window-interface-lgcpe9cc403a/10.7/mac/11.0): Project Audio BrowserとAll Files Browser
- [Logic Pro — Open and close windows](https://support.apple.com/guide/logicpro/open-and-close-windows-lgcp5cbf18ca/10.7/mac/11.0): active window／areaのkey focus
- [Blender Manual — Workspaces](https://docs.blender.org/manual/en/4.0/interface/window_system/workspaces.html): task別のpredefined layout
- [Fusion Help — Workspaces](https://help.autodesk.com/cloudhelp/ENU/Fusion-GetStarted/files/GS-WORKSPACES.htm): purpose-driven Workspaceとcontextual tab
- [Maya Help — Workspaces](https://help.autodesk.com/cloudhelp/2024/ENU/Maya-Basics/files/GUID-0384C282-3CA1-4587-9775-F7164D3F6980.htm): factory／custom、reset、user directory保存
- [IntelliJ IDEA — Viewing modes](https://www.jetbrains.com/help/idea/ide-viewing-modes.html): Distraction-free／Zenによる主Editorの昇格
- [VS Code — Custom Layout](https://code.visualstudio.com/docs/configure/custom-layout): layout customizationとZen mode
- [Microsoft — Progressive disclosure controls](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-progressive-disclosure-controls): discoverability／stabilityリスク
- [Microsoft — Guidelines for app help](https://learn.microsoft.com/en-us/windows/apps/design/in-app-help/guidelines-for-app-help): critical functionをHelpへ依存させない原則
- [AEViewer公式](https://www.aeviewer.com/): media形式横断preview、subfolder検索、Collections、favorite folders、複数import方法
- [Creative Dojo — AEViewer review](https://creativedojo.net/aeviewer/): AE Import dialogよりthumbnail／動画／波形探索が速いという観察と、Favorites等の分類が分かりにくいUX指摘
- [After Effects利用者の再利用library開発背景](https://www.reddit.com/r/AfterEffects/comments/1t8f758/made_an_extension_that_saves_layers_comps_with/): 旧projectを開く、comp名だけで探す、missingをrelinkする、project全体をimportする負担
- [大量音声素材の探索需要](https://www.reddit.com/r/editors/comments/xwewc2/media_asset_management_especially_for_large/): 大量import前の試聴、Bridge不足、team／cloud MAMは過大という利用者の声
