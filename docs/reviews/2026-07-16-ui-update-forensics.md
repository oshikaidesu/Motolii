# UIアップデート考古学 — 改善履歴から潜在的な失敗を読む

日付: 2026-07-16
状態: **調査と採用審判**。個別製品の画面を模倣せず、[UI操作言語](../ui-interaction-language.md)と[実装チケット分解](2026-07-16-m3-ui-concept-to-tickets.md)の拒否条件を補強する

## 1. 調査方法

プロ用ソフトのUI失敗は、通常「失敗」として公表されない。そこで完成画面の印象やユーザー投稿だけでなく、公式リリースノート、公式設計記事、公式マニュアルに残る次の変更を証拠として読む。

- beta後の撤回、既定値の復元、旧操作への退避経路
- 後から追加された説明、label、focus、検索、履歴
- 同じ意味を複数Viewで統一する修正
- no-opなのにUndoが増える等、操作と履歴の不一致修正
- 複雑な階層を迂回するcontextual panel
- keymap preset、全割当編集、base+delta export

証拠と推論を混ぜないため、以下の3段階で記録する。

| 等級 | 何が分かるか | 扱い |
|---|---|---|
| A: 公式が理由つきで撤回 | 変更が実作業を遅くした等の因果まで確認できる | 強い設計反証 |
| B: 公式がworkflow改善として再設計 | 旧UIに摩擦があったことは読めるが、失敗原因は断定できない | 設計仮説 |
| C: 公式bugfix | 具体的に壊れた状態遷移やUndo/focus不整合が分かる | 受入試験へ変換 |
| D: 公式fork / 互換系統 | 別実装を継続保守する費用を払ってでも残した操作型が分かる | 強い需要証拠。ただし技術・政治・配布要因を分離 |

## 2. 事例

### 2.1 Figma UI3: 浮遊パネルを固定へ戻した

**公式事実(A)**: FigmaはUI3 betaでnavigation/properties panelを浮遊させたが、長時間利用者を遅くし、小さい画面でcanvasを圧迫し、rulerも使いにくくしたというfeedbackを受け、全面展開時に固定panelへ戻した。beta中は旧UIへ戻れる選択肢も残した。公式記事は、抽象iconと微妙なaffordanceが機能増加に伴って読みにくくなったこと、Auto Layoutの配置変更がmuscle memoryを崩したため戻したことも説明している。

出典: [Figma UI3設計振り返り](https://www.figma.com/blog/our-approach-to-designing-ui3/)、[UI3初期設計](https://www.figma.com/blog/behind-our-redesign-ui3/)、[UI3移行前の再修正](https://www.figma.com/blog/making-the-move-to-ui3-a-guide-to-figmas-next-chapter/)

**安全な推論**:

- canvas面積を増やす変更でも、高頻度controlの空間的な定位置を壊すと総操作時間は悪化しうる。
- 洗練された最小iconでも、意味の種類が増えると説明力が不足する。
- 熟練者のmuscle memoryは古さではなく、毎日の操作コストを圧縮した資産である。

**Motoliiへの採用**:

- v1 shellは固定分割から始め、dockingや浮遊panelを革新点にしない。
- iconだけへ寄せず、label/Info/semantic badgeを同じcomponent契約に含める。
- 大きな配置変更は既定を一度に置換せず、reference screenと人間審判を通す。

### 2.2 Ableton Live 12: Browser再設計と小さな不整合修正

**公式事実(B/C)**: Live 12はBrowserへfilter、tag、search historyを導入し、その後もFilter Viewの再設計、Quick Tags、表示columnの選択、専用shortcut、typeとarchitectureを区別するicon更新を加えた。release notesには、Learn Viewへfocusが入るとglobal navigation shortcutが効かない、空のtime selection削除で不要なUndo stepが増える、Session/Arrangementで同じtrack color操作の結果が一致しない、といった修正も残っている。

出典: [Ableton Live 12 release notes](https://www.ableton.com/en/release-notes/live-12/)、[Live 12 Browser](https://help.ableton.com/hc/en-us/articles/12927340213660-The-Live-12-Browser)、[Accessibility and Keyboard Navigation](https://www.ableton.com/en/live-manual/12/accessibility-and-keyboard-navigation/)

**安全な推論**:

- 機能を追加しただけでは発見可能性は完成せず、表示/非表示、履歴、focus移動、shortcutまで一つのworkflowとして必要になる。
- 同じ意味を複数Viewへ投影する場合、片側だけ更新される状態はUI bugではなく意味の分裂である。
- Undoは入力event数ではなく、意味のある変更単位でなければならない。

**Motoliiへの採用**:

- focus中の局所componentがglobal navigationやIMEを奪わないことをU0c-2/U1dで検査する。
- no-op/CancelはDocumentとUndo履歴を変えない。
- Stage、Timeline、Inspectorは同じDocument snapshotの投影とし、View別の編集正本を持たない。
- Browser/Inspectorの高度化は新panel追加より、検索、filter、履歴、説明の共通component拡張を優先する。

### 2.3 After Effects: Properties panelは階層探索の迂回路

**公式事実(B)**: AdobeはProperties panelを、選択layerの重要controlへcontextualにアクセスし、Timelineの複数階層を開閉したりpanel間を移動したりする量を減らすworkflowとして説明している。同じpropertyをTimelineへRevealする経路も残している。

出典: [After Effects Properties panel](https://helpx.adobe.com/after-effects/using/properties-panel.html)

**安全な推論**:

- 階層構造そのものが必要でも、主要操作まで毎回その階層を辿らせる必要はない。
- contextual Inspectorは第2の状態正本ではなく、同じpropertyへの近道である時に安全である。

**Motoliiへの採用**:

- Effect Inspectorを最初の縦切りにし、選択→詳細の定位置を作る。
- InspectorとTimelineは同じparameter ID/commandを使い、入口ごとに値やUndo単位を分けない。
- Advancedは同じ意味の由来・評価順・scopeをRevealする場所であり、別の隠れ設定を持たない。

### 2.4 Blender: 独自操作からの移行をpresetと全keymap編集で受け止める

**公式事実(B/C)**: Blenderは初回設定で現行Blender、2.7x互換、Industry Compatibleのkeymapを選べ、select mouse buttonやSpacebar動作も変更できる。keymap editorは各editorのbinding追加/削除/変更、preset import/export、変更分だけのdelta exportを持つ。公式manualは、個別customizationがversion更新時に衝突しうる制約も明記している。

出典: [Blender defaults](https://docs.blender.org/manual/en/3.0/getting_started/configuration/defaults.html)、[Blender keymap editor](https://docs.blender.org/manual/en/2.83/editors/preferences/keymap.html)、[Industry Compatible keymap](https://docs.blender.org/manual/es/2.83/interface/keymap/industry_compatible.html)

**安全な推論**:

- 強い独自操作を持つソフトでも、新規利用者と他ソフト併用者には既知の操作型が必要になる。
- shortcut customizationは設定画面だけでなく、stable action ID、base、user delta、migrationの問題である。
- 互換presetは全員を一つの既定へ強制するより安全だが、文脈ごとに意味が変わるshortcutを無秩序に増やす免許ではない。

**Motoliiへの採用**:

- 全shortcutをstable `CommandId`へ結び、機能側のraw key/modifier判定を禁止する。
- builtin baseを不変にし、user deltaをversion付きJSONとして保持する。
- 将来presetを追加できる構造は持つが、v1でAE/Blender/Ableton互換を名乗らない。

### 2.5 GNOME 3 → MATE / Cinnamon / GNOME Classic: 操作型の分岐を保守する

**公式事実(D)**: Linux Mintの開発者guideは、GNOME 3をGNOME 2と根本的に異なる設計・操作paradigmと説明している。MintはまずMGSE extensionでpanel、system tray、application menu、window list等を戻し、その後MGSEをGNOME Shell forkのCinnamonへ発展させた。並行してMATEはGNOME 2をrename/repackageして継続した。GNOME側も3.8で、伝統的desktopを好む利用者向けにGNOME Classicを追加し、community feedbackへの応答と説明した。

出典: [Linux Mint Cinnamon開発史](https://linuxmint-developer-guide.readthedocs.io/en/latest/cinnamon.html)、[GNOME 3.8 Classic mode](https://blogs.gnome.org/foundation/2013/03/27/gnome-community-releases-gnome-3-8/)、[GNOME Classic manual](https://help.gnome.org/gnome-help/gnome-classic.html)

**安全な推論**:

- application menu、window list、system tray等の外殻は、見た目以上に学習済みworkflowを保持している。
- extensionで旧機能を足す段階からshell forkへ進んだ事実は、局所patchだけでは一貫した操作型を維持できない場合があることを示す。
- upstreamの革新と従来workflowの需要は同時に成立しうる。どちらか一方を無知・保守的と片付けない。

**Motoliiへの採用**:

- 選択→Inspector、中央Stage、下Timeline等の既知の外殻を、装飾上の刷新で移動しない。
- pluginやscript panelを、Hostが欠いた基本workflowをユーザーが復元する場所にしない。
- Advancedや拡張は共通componentを組み合わせ、別shell相当の操作体系をアプリ内へ増殖させない。

### 2.6 Linux Mint XApps: shell固有化したアプリを共通部品へ戻す

**公式事実(D)**: Linux Mintは、特定desktop以外で正しく統合できないGNOME applicationが増えたことを理由に、2016年からXAppsを開始した。modern toolkitを使いながらtraditional titlebar/menubarを維持し、desktop/distro非依存で、MintのCinnamon/MATE/Xfce各editionへ同じcore applicationと改善を届ける方針を明記している。

出典: [Linux Mint XApps developer guide](https://linuxmint-developer-guide.readthedocs.io/en/latest/xapps.html)

**安全な推論**:

- toolkitが共通でも、特定shellのnavigationやwindow conventionを暗黙に要求すると再利用可能性は失われる。
- editionごとに似たappを作るより、共通componentと意味を一箇所で直す方が、改善とbugfixを全利用者へ届けやすい。
- 「modern」と「既存workflowを維持する」は対立しない。

**Motoliiへの採用**:

- Slintは`motolii-ui`へ隔離し、domain intent、Document、commandをtoolkit非依存に保つ。
- pluginごとにtarget picker、parameter control、Undo、error表示を作らせず、Host componentを一箇所で改善する。
- 共通component不足をplugin固有UIの増殖で埋めず、Host語彙の昇格候補として記録する。

### 2.7 KDE Plasma: 自由度を明示的なmodeと既定値へ閉じ込める

**公式事実(B)**: Plasmaは高いcustomizationを維持しつつ、desktop変更を明示的なEdit Modeへ集約し、6.1では全体を縮小表示して編集対象を見渡せるよう再設計した。同じreleaseでshutdown dialogの選択肢を減らしている。6.7ではdesktop typingの挙動をtype-aheadとKRunnerから選択可能にし、新theming systemはtech previewとして既定offで導入した。

出典: [KDE Plasma 6.1](https://kde.org/announcements/plasma/6/6.1.0/)、[KDE Plasma 6.7](https://kde.org/announcements/plasma/6/6.7.0/)

**安全な推論**:

- 高い自由度そのものがガラパゴス化を生むのではなく、通常操作と構成変更の境界が曖昧な時に予測可能性が壊れる。
- 選択肢を持たせても、頻度の低い分岐を明示的なmode/settingsへ置けば通常workflowは狭く保てる。
- 大規模な新基盤を既定offのpreviewとして検証することで、利用者の作業をmigration testにしなくて済む。

**Motoliiへの採用**:

- layout編集、keymap編集、Advanced検査は通常の制作gestureと区別できる明示状態に置く。
- Simpleで頻出選択を絞り、Advancedは同じ意味の詳細を扱う。意味の異なる別製品modeにはしない。
- 新しいUI基盤や自由plugin UIは、成立性spikeをそのまま製品既定へ昇格させない。

### 2.8 Godot: 独自のScene/Node意味を保ったままeditor表面を減らす

**公式事実(B)**: Godot 2.0は、primitiveなresource treeを依存関係のrename/move/deleteまで扱うFilesystem dockへ置き換え、選択Node/Resourceに応じてtoolをcontextualに出す構成を強化した。後のFeature Profileでは、教育、職種、2D専用等の用途に合わせて3D editor、dock、Node、Resource、property、contextual editorを非表示にできるが、既存sceneのNode自体を別形式へ変えない。

出典: [Godot 2.0 editor再設計](https://godotengine.org/article/godot-engine-reaches-2-0-stable/)、[Godot Feature Profiles](https://godotengine.org/article/godot-32-will-allow-disabling-editor-features/)、[Godot Inspector](https://docs.godotengine.org/en/stable/tutorials/editor/inspector_dock.html)

**安全な推論**:

- 独自の価値は新奇なwidgetではなく、Scene/Node/Resourceという一貫した意味modelから生まれている。
- 初心者向け簡略化や職種別UIは、Document能力を削った別形式ではなく、同じ意味の表示profileとして実現できる。
- contextual toolは、選択対象と操作対象が同じIDで結ばれている時にpanel量を減らせる。

**Motoliiへの採用**:

- Simple/Advancedは別Document意味にせず、同じparameter、target、commandの投影量を変える。
- UIを隠しても既存Documentを破損・Bake・型変換せず、非表示能力の存在と結果はbadge/Inspectで確認できるようにする。
- Asset/Plugin Browserはfile一覧で終わらず、rename/move/delete時の参照関係と拒否理由をHostが扱う。

### 2.9 Home Assistant: user拡張の成功例を標準componentへ昇格する

**公式事実(B)**: Home AssistantのProject Graceは、既存dashboard基盤Lovelaceの柔軟性と拡張性を維持しつつ、急な学習曲線、拡張時のscale不足、responsive layoutの弱さを改善対象とした。新Sections viewはsectionをbase unitにし、当初はexperimentalかつ既存dashboard migrationなしと明示した。後続では、利用実例で純正badgeより多用されていたcommunity製Mushroom Chip cardを参考にbadgeを再設計し、旧/new layoutの両方で動かした。

出典: [Project Grace chapter 1](https://www.home-assistant.io/blog/2024/03/04/dashboard-chapter-1)、[Project Grace chapter 2](https://www.home-assistant.io/blog/2024/07/26/dashboard-chapter-2)、[新dashboardの段階採用](https://www.home-assistant.io/blog/2026/02/04/release-20262/)

**安全な推論**:

- user拡張は例外の無秩序な容認だけでなく、Host語彙に不足する需要を観測するprobeになりうる。
- 広く使われたcustom componentの操作型を標準へ昇格すると、Host側でresponsive、copy/paste、visibility、互換を一括提供できる。
- 新layoutをexperimentalにし、旧layoutと既存customizationを即時破棄しないことで、実データから設計を修正できる。

**Motoliiへの採用**:

- plugin固有UIの要望は、まず不足するHost component能力として記録し、複数実例が収束した時だけ共通語彙へ昇格する。
- common componentへ昇格した後は、旧入口と新入口を同じIntent/Document意味へ正規化する。
- reference fixtureなしにcustom UIの自由度だけを先に公開契約へしない。

## 3. Motoliiの更新考古学ゲート(AF)

新しいUI componentまたはshell変更は、次をIssueの拒否条件へ必要な分だけ割り当てる。

| ID | 過去の改善から得た審判 | 最初の適用先 |
|---|---|---|
| AF-1 | 高頻度Viewの役割と位置を安定させる。面積削減だけを理由に浮遊/自動移動させない | U1a-1 |
| AF-2 | iconだけ、色だけ、hoverだけを唯一の説明にしない | U0e-3, U2c-3 |
| AF-3 | 同じ意味の複数Viewは同じID、Intent、Document snapshotを投影する | U2b-1, U2c-2 |
| AF-4 | 局所focusがglobal navigation、Undo/Redo、IMEを不当に奪わない | U0c-2, U1d |
| AF-5 | no-op、Cancel、拒否操作はUndo stepを生成しない | U2a-1 |
| AF-6 | contextual Inspectorは正準propertyへの近道とし、別状態を所有しない | U4a-1, U4a-2 |
| AF-7 | shortcutはstable action ID + immutable base + user deltaで管理する | U0c-1, U0d-1〜3 |
| AF-8 | 大きなUI変更は人間の実作業審判と退避可能な段階導入を持つ | G0-6H, U3d |
| AF-9 | 新機能でpanelを増やす前に、既存componentのcontext、検索、filter、説明を拡張できないか審査する | U2c, U4a, U6 |
| AF-10 | Hostが欠いた基本workflowをplugin/extensionで復元させない | U1a, U2c, U4a |
| AF-11 | toolkit/shell固有型をdomain意味へ出さず、共通修正を全入口へ反映する | U0a, U0b-2, U2c-2 |
| AF-12 | customizationは明示的なmode/settingsへ置き、通常gestureと状態を混同しない | U0d, U2c, U4c |
| AF-13 | 新UI基盤・大規模外観変更はspike/previewから始め、既定採用を別審判にする | U0e, G0-6H, U1e |
| AF-14 | UI profileが能力を隠してもDocument意味を削除・変換せず、同じID/commandを使う | U2c, U4a, U4c |
| AF-15 | user拡張を需要probeとして観測し、複数実例が収束してからHost componentへ昇格する | U2c, U4a, plugin UI follow-up |
| AF-16 | 不安定なplatform/backendを抽象境界とfallbackで隔離し、Document意味や操作文法へ漏らさない | U0a, U1a, U1b, S1 |
| AF-17 | browser/OS/toolkitが入力・focus・accessibilityを自動解決すると仮定せず、共通componentとconformanceで補う | U0c, U1d, U2c |

AFは「他製品が後から追加したのでMotoliiも全部初日に入れる」という一覧ではない。各更新が示す壊れ方を、自動試験、人間審判、STOP条件へ変換するための監査表である。

## 4. 今回は採らない推論

- 更新された機能はすべて旧版の失敗だった、とは断定しない。
- Figmaが固定panelへ戻したため、全contextで浮遊UIが悪いとはしない。
- AbletonのBrowser機能をMotoliiへ同じ配置・名称で移植しない。
- AEのProperties panel追加を、AE全体の散在が解消した証拠とは扱わない。
- Blenderのpreset数を、入口ごとに異なる意味を許す根拠にしない。
- GNOMEの分岐を、GNOME 3全体の失敗やtraditional desktopの普遍的優位の証明にしない。
- Cinnamon/MATE/XAppsのfork理由を、技術移行、namespace、distribution方針から切り離してUIだけの因果にしない。
- KDEのcustomization量を、Motoliiへ自由dockingや無制限theme APIを入れる根拠にしない。
- GodotのFeature Profileを、作品ごとに別schemaや別command体系を持つ根拠にしない。
- Home Assistantのcustom card文化を、未制限のplugin UI公開契約を先に焼く根拠にしない。
- 公式記事の宣伝文句を性能・学習容易性の測定結果として扱わない。

## 5. 今後の調査対象の選び方

「OSSだから」「長寿だから」ではなく、次の条件で優先する。

1. 既存製品の機能copyではなく、新しい正準objectまたはworkflowを作った
2. 初期設計、変更理由、撤回、互換経路の公式資料が残る
3. user拡張や複数職種を受け入れた後も意味modelが分裂していない
4. 見た目の刷新でなく、選択、履歴、参照、Inspector、拡張境界を検証できる

| 優先 | 対象 | 調べる価値 | 扱い |
|---|---|---|---|
| 主対象 | Ableton / Figma / Godot / Home Assistant | Session、browser共同編集、Scene/Node、smart-home dashboard等の独自workflowを成熟させた | 個別の価値modelとUIの対応を読む |
| 混成・反面教師 | Blender | Maya等の既存3D制作語彙へのcounterであり、独自体系の歪みと後の互換keymapを同時に観察できる | 新規価値modelの純粋例とせず、独自操作が蓄積する費用を読む |
| 構造反証 | GNOME/Cinnamon/MATE/XApps/KDE | fork、互換mode、shell密結合、customization境界の維持費が公開されている | UI以外の分岐要因を必ず併記 |
| 対照群 | GIMP | multi-windowからoptional single-window、さらにdefault化した履歴はshell topologyの反証になる | 画像編集の価値model創造例としては使わない |
| 低優先 | 既存商用ソフトの直接clone、見た目だけのfork | 元製品の既知語彙を再実装した影響が大きい | 固有のbugfix以外は一般化しない |

GIMPにも公式に、頻出complaintへ対応したUI改修、optional single-window導入、後のdefault化という有用な履歴はある。ただし読み取れるのは「window topologyをどう移行したか」が中心であり、Motoliiの新しい制作意味やcomponent境界を発明する主資料にはしない。

出典: [GIMP 2.6 UI変更](https://www.gimp.org/release-notes/gimp-2.6.html)、[GIMP 2.7 single-window](https://www.gimp.org/release-notes/gimp-2.7.html)、[GIMP 3.0 manual](https://docs.gimp.org/3.0/en_GB/gimp-image-window.html)

結論は限定的である。**安定した場所、同じ意味の複数投影、説明可能なcomponent、意味単位のUndo、変更可能な入力、段階的な人間審判、toolkit非依存の共通意味**は、成熟ソフトが後から費用を払い、時にはforkを継続保守してまで補強してきた領域である。Motoliiでは初期の共通境界として先に作る価値がある。

## 6. 不安定な土台が安定した製品境界を要求する仮説

Figmaの安定感を「WebはI/Oが制限されるから」とだけ説明するのは難しい。Figmaの公式性能記事では、editorの制約はCPUまたはGPUであることが多く、I/Oが支配的なのは稀としている。一方、Webという土台はnetwork切断、browser/OS shortcut競合、GPU backend差、context/device loss、非同期readback、canvas accessibility欠落を持つ。

公式技術資料に見える対処は次の通り。

- clientはDocument copyを持ち、変更をobject property単位で同期する。offline復帰時は最新Documentへ未送信editを再適用する。
- Document同期とcomments/users/projects等を、異なる性能・offline・security要件の別systemへ分ける。
- renderer上位codeとWebGL/WebGPUの間にinterfaceを置き、同じC++ rendererをWasm/nativeへ出す。
- WebGPU移行は一括置換せず、device/driver別計測、段階rollout、mid-sessionのWebGL fallbackを持つ。
- browser内canvasは標準accessibilityを失うため、keyboard/screen reader対応をreusable UI componentと検査toolで再構築する。

出典: [Figma multiplayer architecture](https://www.figma.com/blog/how-figmas-multiplayer-technology-works/)、[WebGPU renderer移行](https://www.figma.com/blog/figma-rendering-powered-by-webgpu/)、[Figma performance](https://www.figma.com/blog/keeping-figma-fast/)、[canvas accessibility](https://www.figma.com/blog/building-accessibility-into-a-canvas-based-product/)、[international shortcut conflict](https://www.figma.com/blog/behind-the-scenes-international-keyboard-shortcuts/)

したがってWebの寄与は、機能を自然に単純化したことより、**壊れうるplatformを正準意味から切り離さないと製品が成立しなかったこと**にあると推論する。制約は自動的に良いUIを作らないが、正しく応答すれば境界を鍛える。

Motoliiでも同じ構造を採る。

- Document/CommandをSlint、OS event、GPU backendから独立させる。
- wgpu textureと非同期workerを正規経路にし、UI共有deviceの同期readbackを禁止する。
- physical keyではなく`CommandId`と正規input eventを正本にする。
- Slintやplugin固有UIの欠落を、製品全体へ漏れるspecial-caseで補わない。
- backend更新、外観刷新、IME対応は、同一意味を保ったspike、conformance、fallback、段階採用として扱う。

この意味で、Motoliiが学ぶべきなのは「Web化」ではなく、**不安定な実装面の上に、狭く安定した意味境界を置く設計圧**である。
