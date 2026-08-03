# Motolii意味SDKとAviUtlプラグインコミュニティの比較

状態: **縮小採用**。AviUtl 1.x＋拡張編集の成熟コミュニティ、AviUtl2の現行Lua／Plugin SDKと、
MotoliiのVism意味SDK、作者連続性、Host責任境界を比較する。目的は新旧を競うことではなく、AviUtlが実際に
成立させた作者人口、低い配布摩擦、表現到達性を、Motoliiのモダンな境界設計が失っていないかを反証することである。

2026-08-01、ユーザー確認により、§6の八項目を**AviUtl continuity floor**としてMotoliiの現行authorityへ
縮小採用した。Lua／`obj`／file extension／package形式は採用せず、利用者成果と負例だけを移す。

本比較からTypeScript API、runtime、package、Document schema、PluginKindを追加しない。AviUtl互換を目標にせず、
同じ利用者成果へ到達できるかを比較する。

関連:
[Vism意味SDK](2026-08-01-vism-semantic-sdk-cavalry-translation-decision.md)、
[作者連続性と変更カプセル](2026-07-31-authoring-continuity-capsule-goal-contract.md)、
[Vism作者programの言語境界](2026-08-01-vism-authoring-language-boundary-decision.md)、
[Vism Inspector・作者source・Automation責任境界](2026-08-01-vism-inspector-source-automation-boundary-decision.md)、
[Creator / Developer連続体](2026-07-22-creator-developer-continuum-decision.md)、
[AviUtl完成拡張スタックの性能観察](2026-07-29-aviutl-completed-plugin-stack-performance-observation.md)、
[AviUtl／AviUtl2最低スペック移行性能ゲート](2026-07-29-aviutl2-low-spec-migration-performance-gate.md)

## 1. 結論

**現在の製品・コミュニティとしてはAviUtlが勝つ。長期境界はMotolii側に改善の見込みがあるが未実証である。**
この二つを混ぜない。

AviUtlは既に、短いscript、設定項目の自動生成、通常のobject／effectとしての利用、単一file配布、module依存、
native plugin、入出力、編集補助、独立した配布者と解説文化まで一巡している。AviUtl2では単一fileまたはpackageを
PreviewへD&Dしてinstallでき、package identityとuninstallも追加された。一方、Motoliiの意味SDK、TypeScript作者席、
外部IDE、package、第三者install／loadは現時点で決定または未決であり、外部作者の製品一巡は未実装である。

Motoliiが改善しようとしているのは、AviUtlの成功を支えた低摩擦を捨てることではない。AviUtlでglobal `obj`、
layer番号、effect名、共有buffer、pixel mutation、native patchへ集まった責任を、typed semantic value、明示connection、
Host capability、StateTrack、WGSL、Automationへ分けつつ、利用者には一つのVismとして見せることである。

従ってMotoliiの合格条件は「AviUtlより型安全」ではない。

> AviUtlで一fileのscript作者が得ていた速さと表現到達性を維持したまま、依存、作用先、状態、失敗をHostが読めること。

これを実機で証明するまで、Motoliiのモダン設計は**有望な契約**であって、AviUtlコミュニティの代替ではない。

## 2. 比較対象を三つに分ける

### 2.1 AviUtl 1.x＋拡張編集

旧AviUtlは本体だけでなく、拡張編集、`.anm`／`.obj`等のLua script、入力／出力plugin、編集補助、patch、
LuaJIT、preset／alias、解説記事、配布動画を含む完成環境として比較する。script fileを所定folderへ置き、
animation effectまたはcustom objectとして利用できる。これは低摩擦だが、folder配置、暗黙runtime、追加module、
本体／拡張編集／patch versionへの依存を利用者が解決する場合がある。

旧AviUtlの[公式配布ページ](https://spring-fragrance.mints.ne.jp/aviutl/oldver2.php)は本体、拡張編集、
AviUtl2／SDKの系譜を配布している。旧scriptの配置と利用形態は、公開script repositoryの
[導入例](https://github.com/akkadaska/aviutlScripts)でも確認できる。ただし一repositoryを利用者多数派の証拠にしない。

### 2.2 AviUtl2

2026-08-01確認時点で、公式配布ページにはAviUtl2 v2.1.2とPlugin SDKがある。以下の詳細は、同梱文書を加工した
非公式mirrorと、公式SDK zipを自動追従する非公式GitHub mirrorで再確認した。

- Luaは`.anm2`、`.obj2`、`.cam2`、`.scn2`、`.tra2`とscript controlで利用でき、旧fileも一部互換で読める。
- source先頭のcomment declarationからtrack、check、color、file、font、select、text等の設定項目を生成する。
- 通常はLuaJIT、指定によりLuaを選べる。script controlでは`os`、`debug`、`ffi.C`等を外す。
- `.mod2` script moduleにより、native側の関数群をLuaへ追加できる。
- Plugin SDKはinput、output、filter、script module、general pluginを分ける。
- 単一fileまたは`.au2pkg.zip`をPreviewへD&Dしてinstallでき、同一package IDの更新とuninstallを扱う。

出典:
[AviUtl2 Lua同梱文書のmirror](https://docs.aviutl2.jp/lua/)、
[AviUtl2簡易説明のmirror](https://docs.aviutl2.jp/usage)、
[AviUtl2 SDK mirror](https://github.com/aviutl2/aviutl2_sdk_mirror)。
両mirrorは非公式であるため、実装採択時は公式配布zipの固定hashへ再照合する。

### 2.3 Motolii

比較するMotoliiは完成済みruntimeでなく、現在の決定と実装状態を分ける。

- **決定**: TypeScript作者言語、意味SDK、Vism identity、Inspector projection、外部IDE方針、Host所有責任。
- **部分実装**: static first-party plugin façade、Path geometry、PathOp、DataTrack、GPU texture経路等。
- **未実装／未決**: 外部作者SDK surface、live runtime、package、install／load、external IDE transport、第三者一巡、
  Instance／3D／Simulationの主要runtime。

## 3. 作者経路の比較

| 段階 | profile | AviUtlで成立している経路 | Motolii目標 | Motolii現在 |
|---|---|---|---|---|
| Use | 共通 | script／pluginをobject・effectとして使う | Vismを追加しInspectorで調整 | first-party pluginはあるが第三者Vism経路なし |
| Tune | 共通（2で宣言種別を拡張） | source宣言からtrack／check等を自動表示 | typed contractからparameterを投影 | NodeDesc系の部分実装、製品Vism投影未完成 |
| Compose | 共通 | `obj.effect()`、layer参照、buffer、alias、preset | typed port、Kit、Group、Host operation | 仕様・部分実装。第三者一巡未成立 |
| Inspect | 共通 | 設定dialog、source、解説、依存README | 一つのInspectorで作用先、型、space、time、診断 | 方針決定、製品統合未実装 |
| Fork | 共通 | fileを複製・編集 | 同じVismからlocal candidateをFork | 未実装 |
| Author | 共通（2は`.mod2`／現行SDK） | Lua一file、必要ならmodule／native SDK | TypeScript semantic program、WGSL、admitted Rust／Host席 | 言語・意味境界決定、surface／runtime未実装 |
| Publish | 1.x=file／zip、2=packageも可 | file／zip、GitHub、配布動画、AviUtl2 package | 外部商流＋公開catalog＋portable Vism | topology方針、artifact／install未実装 |
| Reuse | 1.x=folder中心、2=D&Dも可 | folder配置／D&D、alias／preset | typed dependency解決、Project Lock、missing診断 | 未実装 |

AviUtlの強さは、UseからAuthorまで同じsoftware内のobject／effectとして連続する点にある。Motoliiの設計も
同じ連続性をVism identityで狙うが、現在は紙上の接続が多い。TypeScriptを採っただけでは追いつかない。

## 4. 表現到達性の比較

AviUtl2 Luaは、現在objectのload／draw、effect名とparameter名によるeffect実行、layer番号からのobject／設定値取得、
framebuffer／共有temp buffer、pixel get／put、camera parameter等へ到達できる。これは短いsourceから非常に広い表現を
作れる理由である。一方、作用先、space、resource、cache freshness、共有stateの境界はAPIの文字列と暗黙contextへ寄る。

| 表現需要 | AviUtl経路 | Motolii対応先 | 判定 |
|---|---|---|---|
| 一fileのanimation effect | `.anm`／`.anm2`＋自動設定項目 | pure Vism＋typed parameter | 同じ低摩擦を実証すべき |
| custom object／generator | `.obj`／`.obj2`、`obj.draw()` | Path／Shape／Instance result | RGBAへ早期に潰さず意味値を返す |
| filter chain再利用 | `obj.effect(name,param,value...)` | typed Vism input／Kit | 文字列名呼出しを型付き接続へ置換 |
| 別layer参照 | layer番号、effect／item名 | explicit typed input／provider | ambient scene traversalは移植しない |
| pixel effect | get／put pixel、shader、native filter | Texture＋WGSL seat | GPU resourceと色変換はHost所有 |
| framebuffer全体効果 | framebuffer load／draw | Group／Composite／explicit surface input候補（Backdrop口は未凍結。[plugin authoring §8](../plugin-authoring.md)） | 暗黙の画面全体取得を移植しない |
| particle／duplicator | Lua object draw loop、module、script群 | Instance value＋L0／L1／L3 | indexをidentityにせず、状態はStateTrack |
| camera effect | `camera_param`等のtable | typed camera observation／Host camera seat | active camera mutationをVismへ渡さない |
| 編集自動化 | general plugin、独自window／project operation | 将来Automation typed proposal | 毎frame評価と別席にする |
| codec／入出力 | input／output plugin | media importer／codec provider（Host capability module） | Vismへ押し込まない |

MotoliiがAviUtlのraw APIを拒否するなら、拒否した各成功例へ代替のtyped routeが必要である。「危険だから不可」で
終わり、別layer参照、複数draw、feedback、pixel effect、module再利用ができなければ、コミュニティにとっては退化になる。

## 5. 比較表

| 評価軸 | profile | AviUtlで成立していること | Motolii設計 | 現時点の優位 |
|---|---|---|---|---|
| 最初の結果まで | 共通（2はD&Dも可） | 一file＋配置／D&Dで短い | Inspect→Fork→external IDE→last-goodを目標 | **AviUtl**。Motolii未実装 |
| parameter UI | 共通 | source commentから自動生成 | typed contractからInspector投影 | AviUtlは実績、Motoliiは型と診断で改善可能 |
| 表現到達性 | 共通、API差あり | global object、buffer、pixel、module、native pluginで広い | semantic family＋WGSL＋Simulation＋Automationへ分割 | AviUtlは実績、Motoliiは未反証 |
| 作用先の理解 | 共通 | object位置とAPI慣習、layer／effect名へ依存 | Vism identityとtyped inputをInspector表示 | Motolii設計。ただしUI未実装 |
| composition | 共通、API差あり | 名前、順序、layer、shared buffer、preset | typed port、Kit、explicit provider | Motolii設計。ただし閉じた実例不足 |
| 配布摩擦 | 1.x=file／zip、2=D&D packageも可 | 小さい配布物から導入できる | 外部商流・catalog方針、artifact未決 | **AviUtl** |
| fork／改変 | 共通 | source fileを直接複製できる | local candidate＋atomic adoptionを目標 | **AviUtl**。Motolii未実装 |
| 依存診断 | 共通、2はpackage identityあり | README、folder、module名、version慣習に分散し得る | typed dependency、preflight、missing診断を目標 | Motolii設計。製品証明なし |
| 互換追従 | 1.xと2を別判定 | 1.x資産、一部旧script互換、patch／runtime差がある | meaning profileとconformanceで分離を目標 | 未判定。Motoliiはversion corpus未実装 |
| failure局所化 | 1.xと2を別判定 | script／native plugin／patchにより障害半径が異なる | Vism identity、Host admission、typed failureを目標 | Motolii設計。process containment未完成 |
| 決定論／再現 | 共通、API差あり | 暗黙context、fps、buffer、moduleに依存し得る | explicit time／seed／input、StateTrack、Project Lock | Motolii設計。第三者保存再読込未実装 |
| performance | 1.x完成stackと2を別測定 | 1.xはJIT、patch、RAM preview等。2は別実装 | GPU resident、WGSL、typed resource budgetを目標 | 実機fixtureまで未判定 |
| cross-platform | 共通 | Windows中心 | wgpu抽象、vendor API非公開 | Motolii設計。Windows製品経路未成立 |
| LLM authoring | 共通、API差あり | Lua sourceと公開例は短いが、暗黙`obj`意味を復元する必要 | typed contract、TS学習資産、診断を目標 | 未判定。作者連続性のchat-only可搬性fixture（六fixtureとは別）未実施 |
| community actuality | 1.x成熟実績、2は継承／形成中を分離 | 作者、script、plugin、解説、配布実績 | 目標 | **AviUtl** |

## 6. AviUtlから必ず残すもの

Motoliiが「モダン化」の名で失ってはならないものを固定する。

1. **一つの小さい成果物から始められる**。最小表現にmanifest、build system、複数言語fileを強制しない。
2. **parameter UIが自動で出る**。作者へcustom GUI実装を要求しない。
3. **通常の表現単位として使える**。code objectを別扱いせず、Vismとして追加、複製、調整できる。
4. **fileを読んでforkできる**。sourceをHost内部databaseや生成物だけへ閉じない。
5. **公開場所を中央storeへ限定しない**。Git、個人site、BOOTH等の外部配布を妨げない。
6. **不足をcommunityが埋められる**。Hostの全更新を待たず、小さい表現やworkflowを独立公開できる。
7. **高度化の階段がある**。semantic programで足りなければWGSL、Simulation、admitted implementationへ進める。
8. **preset／alias相当の再利用がcode不要で成立する**。毎回forkとprogrammingを要求しない。

## 7. AviUtlから移植しないもの

1. layer番号、effect名、setting名を通常の公開接続契約にすること。
2. current object／active scene／framebufferへambient globalから到達すること。
3. 全object共有のtemp bufferや隠れた可変state。
4. preview fpsや評価順の副作用でsimulation速度が変わること。
5. pixel loop、cache破棄、GPU／CPU転送を作者が暗黙に選ぶこと。
6. native binary、patch、runtime DLLの置換を低摩擦scriptと同じtrust levelで扱うこと。
7. folder名、load順、同名file、暗黙moduleだけをdependency解決にすること。
8. pluginがcamera、Document、Undo、selection、export jobの正本を取得すること。

これらを拒否する責任は、機能削除ではなくtyped input、Host operation、resource admission、StateTrack、Automation等の
代替経路を用意する責任と対になる。

## 8. MotoliiがAviUtlへ勝ったと言える反証fixture

言語のsyntax比較ではなく、次の六作品を同じ作者席から作る。

| fixture | AviUtl側 | Motolii側で証明すること |
|---|---|---|
| A: 位置wiggle | 一file animation effect＋track | 一file相当、parameter自動投影、同seed再現 |
| B: Path変形 | animation effect／custom object | SDK-S0 Path2D→Path2D、typed error、Inspector作用先 |
| C: 100 instance音同期 | custom object＋draw loop／module | stable Instance identity、event input、2D／3D同一model |
| D: pixel stylize | pixel／shader／filter plugin | WGSL closure、VRAM常駐、last-good、budget診断 |
| E: 別object連動 | layer番号／effect名参照 | explicit typed connection、rename／reorder不変 |
| F: collision particle | Lua／module／pluginの組合せ | 同じVism identityでL0→L3、Host StateTrack／Bake診断 |

各fixtureで最低限、次を記録する。

- 初見作者が既存成果をforkして最初の変化を表示するまでの時間。目標probeは10分以内。
- edit-to-display p50、再起動回数、compile failure時のlast-good保持。
- source file数、手動配置手順数、必要dependency数、Host外で調べた固有語数。
- sourceを読まずInspectorだけで作用先、input、output、space、time、失敗理由を説明できるか。
- PreviewとExportの結果、seed、Simulation stateが一致するか。
- 第三者package／install一巡成立後に限り、package欠落、version不一致、capability不足を、Rust／WGSL／engine語彙なしで直せるか。
- fixture Dを含む性能値は、[最低スペック移行性能ゲート §4](2026-07-29-aviutl2-low-spec-migration-performance-gate.md#4-二段の合格面)の
  機材、電源、熱、OS、version、素材条件を併記し、開発主機だけでAviUtl比較勝利を判定しない。

六fixtureが反証するのは作者体験と意味到達性である。コミュニティ優位の十分条件ではない。第三者による
作成→conformance→導入→作品保存／再読込、実配布、依存更新、利用者supportの実績は別に判定する。

## 9. 最も危険なMotolii側の失敗

### 9.1 型を増やしすぎて一file文化を殺す

意味SDKのfamily分割が、作者へ大量のimport、wrapper、manifest、generic、boilerplateを要求するとAviUtlより弱い。
Host内部の厳密さを作者sourceの冗長さへ転嫁しない。同じPath意味を保ったまま、最小sourceを短くできるsurfaceが必要である。

### 9.2 安全を理由に表現の逃げ道を閉じる

ambient scene accessを拒否しても、別object連動、feedback、multipass、pixel effect、custom generator、module再利用の
需要は消えない。typed routeが無い能力は「安全に解決済み」ではなく未対応である。

### 9.3 Vism内部artifactを作者へ露出する

一表現がTS、WGSL、Simulation definitionへ分かれる場合も、通常作者へ三projectの同期を要求しない。Hostが一つの
Vism contract、fixture、diagnosticとしてまとめ、必要なsourceだけ段階開示しなければAviUtlの一fileより理解しにくい。

### 9.4 packageを先に重くする

signature、permission、lock、catalog、commerceを最初のwiggleにも要求すると作者人口が生まれない。trustと配布の
必要性は消せないが、local candidate／source forkと、第三者配布／installのgateを分ける。

### 9.5 設計優位を実績と呼ぶ

typed contract、GPU residency、crash containment、Project Lockが文書にあっても、第三者が作成、導入、保存、再読込、
欠落診断まで一巡していなければ、AviUtlコミュニティとの比較では未成立である。

## 10. 現在の処分

| 論点 | 処分 |
|---|---|
| AviUtlの一file＋自動parameter UI | **目標として維持**。TS surface／Inspector fixtureで反証する |
| object／effectとしての通常利用 | **Vism identityへ一般化** |
| file／D&D packageの低摩擦 | **同等成果を要求**。package形式は未決のまま |
| Lua／`obj`互換 | **棄却**。意味到達性だけをfixtureへ移す |
| layer／effect名参照 | **typed connectionへ置換** |
| framebuffer／pixel操作 | **Texture／Composite／WGSLへ分解** |
| module／native pluginへの高度化 | **capabilityとadmissionを分けて維持** |
| general pluginのscene編集 | **将来Automationへ分離** |
| communityがHost不足を直す力 | **必須成果**。Core責任を外へ捨てることとは分ける |
| Motoliiのコミュニティ優位 | **未成立**。第三者一巡と六fixtureまで主張しない |

§6の統合先は次とする。

| AviUtlから残す成果 | Motolii正本 |
|---|---|
| 一つの小さい作者成果、通常effectとしての利用、parameter自動UI、fork、短い反復 | [作者連続性と変更カプセル](2026-07-31-authoring-continuity-capsule-goal-contract.md) ACG-O6〜O10 |
| 意味型と内部artifactを作者へ過剰露出しない | [Vism意味SDK](2026-08-01-vism-semantic-sdk-cavalry-translation-decision.md) §9〜10 |
| local authoringをpackage完成待ちにしない、中央配布へ必須依存しない | [Vism package concept](../vism-package-concept.md) §4 |
| alias／preset／接続済み再利用をcode不要にする | [Vism / Kitモデル](../vism-kit-model.md) §4.1 |
| communityがHost不足を小さい成果で補える | [Vism package concept](../vism-package-concept.md) §4.4の分散配布と第三者生態系 |
| 同じVism identityのまま高度化できる階段 | [作者連続性](2026-07-31-authoring-continuity-capsule-goal-contract.md)の行き止まり禁止、[言語境界決定](2026-08-01-vism-authoring-language-boundary-decision.md) §2 |
| LANG-TS-F0／F1の反証条件 | [implementation ledger](../implementation-ledger.md) AUTHORING-LANGUAGE |

## 11. STOP

本比較から次を行わない。

- AviUtl API、Lua、file extension、package.ini、folder layoutとの互換実装。
- TypeScript surface、Vism package、permission、signature、catalog schemaの決定。
- Document／serde、PluginKind、public Rust trait、engine、loaderの変更。
- AviUtlのlayer／effect名、shared buffer、camera table、pixel cacheをMotolii公開契約へ移植すること。
- community repository数、GitHub star、単一作者の成果を利用者母数または市場規模と呼ぶこと。
- 旧AviUtlとAviUtl2をprofile標記なしに同じ列へ混ぜること。
- Motoliiの設計目標と実装済み能力を同じ状態として混ぜること。

比較から見えた不足は、既存`LANG-TS-F0`、VSM-B／C、P0I、M5、Simulation、Automationの各ownerへ戻す。
新しい恒久APIをこの観察から直接作らない。

## 12. Fable 5反対側レビューとCodex採否

2026-08-01、Claude Code経由のFable 5 (`claude-fable-5`)へ本文書と限定したMotolii正本をread-onlyで渡した。
判定は`ACCEPT（P0=0、P1=0、P2=7）`。重大な正本衝突、実績僭称、公開APIの密輸は無かった。

P2七件はすべて採用した。§1の二義文を修正し、§3／§5へ1.x／2／共通profileを明記した。未定義の
`Host Adapter`を既存のmedia importer／codec providerへ戻し、未凍結Backdrop口、chat-only可搬性fixture、
package成立後だけ測れる診断、低スペック機材条件、六fixtureとコミュニティ実績の判定分離を明示した。
Fableの判定はauthorityではなく、Codexが現行正本へ再照合して採否した。

ユーザーが§6の良い点の統合を決めた後、更新した作者連続性、意味SDK、Package、Kit、ledger、decision-indexを
同じFable 5へ再審査した。判定は`ACCEPT（P0=0、P1=0、P2=2）`。P2二件を採用し、floorの範囲を
`ACG-O6〜O10`へ統一し、community補完と高度化の階段の正本帰属を§10へ追加した。package形式、runtime、
公開API、Document／schemaの黙示決定、実装済み僭称は無いと再確認された。
