# 作者連続性と変更カプセルのゴール契約（2026-07-31）

状態: **利用者成果・不変条件・停止線は決定／program作者言語は後続でTypeScript採択／chat LLM・payload・runtime・package・公開型は未決／実装許可ではない**

対象: [Creator / Developer連続体](2026-07-22-creator-developer-continuum-decision.md)、
[小さなコアと探索可能な拡張](../extensible-core-model.md)、
[ジェネラティブユーザー境界](../generative-user-boundary.md)、
[Vism作者journey](2026-07-27-vism-authoring-journey-decision.md)、
[AviUtlコミュニティ比較](2026-08-01-motolii-semantic-sdk-aviutl-community-comparison.md)、
[External Authoring Bridge](2026-07-29-external-authoring-bridge-seat-decision.md)

## 0. この文書が決めること

既決の`Use → Tune → Compose → Inspect → Fork → Author → Publish → Reuse`を再決定しない。
本書は、その階段を将来の作者実装へ接続する際に後戻り不能な負債を作らないため、次の五点だけを
追加のゴール契約として固定する。

1. **行き止まり禁止**: 現在の表現と型付き意味を捨てて別製品モデルへ移り直させない。
2. **一回一変更カプセル**: 一回の作者操作はHostが列挙・検査できる一つの変更境界へ閉じる。
3. **oracleはHost所有**: 応答側が合否条件を書き換えられない。
4. **依存能力は宣言する**: 作者programへambientなHost操作面を渡さず、必要能力を型付き要求として表へ出す。
5. **初心者には一つの作者面**: 一つの表現を作るために、複数file、複数言語、seat、runtimeの構造理解を入場条件にしない。

2026-08-01追補: この五不変条件をAviUtl作者成果へ適用する**AviUtl continuity floor**として、一つの小さい
作者成果、自動parameter面、通常表現としての利用、source fork、短い反復、code不要の再利用、外部配布、
高度化の階段を、Lua／ambient APIを移植せず維持する。新しい第六原則ではなく、五原則の具体的な合否面である。

これはTS、WGSL、Rust、WASM、LLM、editor、package形式、公開ABIの採択ではない。

## 1. ゴール契約

> Motoliiは、既決の作者経路の各一段を、現在の作品、対象identity、型付き入力、Preview、診断、
> versionを失わずに進められる、Host生成・Host検査の一つの変更カプセルとして接続する。
> 公開作者境界へ出したtyped valueと能力は、first-party専用の隠れた中継を要求せず、同じ公開境界上で
> 宣言・接続・検査できなければならない。その内部接続が複数のartifactへ分かれても、通常の作者には
> その表現にとって一つの入口、一つの編集・Preview・採用単位として提示する。具体的な型、席、payload、runtimeは
> 個別fixtureと反対側reviewで締結する。

「自由」は言語、seat、payload、runtimeを一覧から同時選択できることではない。利用者の現在の意図から
次の一段が見え、より深い表現へ進んでも既存の意味と検査可能性を捨てないことである。

## 2. 合否を測る十の成果

| ID | 合格条件 | 失敗条件 |
|---|---|---|
| ACG-O1 Inspect到達 | 公開表現instanceから1操作で実装identity、version、typed入力／出力、要求能力、由来、欠落診断へ到達できる | 公開表現なのに作者境界または由来へ到達できない |
| ACG-O2 atomic adoption | Fork後の候補は開始revisionを照合し、Hostの全体preflight後に一回だけ採用される。Document変更なら1 macro、失敗／Cancel／staleなら変更0 | 部分適用、黙示merge、適用後だけ判明する未検査依存 |
| ACG-O3 Host oracle | fixture、negative oracle、resource上限、expected／actual診断はHostまたは独立conformanceが所有する | 作者応答と同じ書込範囲からoracleや期待値を変更できる |
| ACG-O4 一本道 | 通常制作UIは目的と現在対象から次の操作を示す | 言語名、semantic seat、payload、runtimeの選択を通常利用者へ要求する |
| ACG-O5 公開境界の非特権性 | 公開typed value／capabilityを使うfirst-party成果が、第三者と同じ宣言、admission、fixtureで成立する | first-partyだけが内部producer、consumer、変換、scene queryへ到達する |
| ACG-O6 一つの作者面 | 最初の独自表現を、一つの可視source／recipeと一つのPreviewから作り始められる。内部artifactはHostが結合し、必要時だけ段階開示する | 最初の有意味な変更に複数file、複数言語、seat間配線、build graphの理解を要求する |
| ACG-O7 通常Vism＋自動control | 作者成果はcode objectでなく通常Vismとして追加、複製、調整でき、宣言parameterはHostがInspectorへ自動投影する | sourceを開かないと利用できない、または作者へcustom UI実装を要求する |
| ACG-O8 local authoring独立 | local candidateの作成、fork、検査、Previewは最終package、install store、signature、catalog成立を前提にしない | 最初のwiggleにも配布用manifest、署名、catalog登録、複数artifact手動配置を要求する |
| ACG-O9 code不要の再利用 | 単一Vismの設定はPreset、複数Vismのtyped接続はKitとして保存・再利用できる | 再利用のたびにsource fork、opaque project blob、名前／layer番号参照を要求する |
| ACG-O10 反復継続 | forkから最初の変化までの時間、edit-to-Preview、再起動回数、last-goodを測り、言語境界決定の10分probe／F1 feedback gateを満たす | compile失敗でPreviewを失う、再起動を通常反復にする、反復時間を測らず「簡単」と称する |

未締結の型や席はACG-O5違反ではない。比較前の空席を「永久に第三者へ開かない」と決めること、または
first-partyだけの裏口で先に埋めることが違反である。

## 3. 変更カプセルの意味

変更カプセルは一ファイル、archive、manifest、wire schema、LLM promptの名称ではない。
次を一つの論理的な変更境界としてHostが列挙できることを指す。

- 対象identityと開始revision。
- 利用者が達成したい一つの目的。
- 変更してよい作者面と、変更してはいけないclosure。
- 読んでよいsource、型、宣言済み依存能力、fixture、診断。
- 期待する出力種別と、適用前の負例。
- resource、deadline、決定論、秘密情報の境界。
- 全体候補、preflight結果、採用または変更0。

一つのカプセルが複数の内部sourceや生成artifactを参照することは禁じない。ただし、それを通常作者へ複数の
独立した編集面として露出しない。一つの利用者目的から必要な内部artifactが複数生じる場合、Hostが一つの
作者面から決定的に導出し、一つのPreviewと一つのatomic adoptionへ閉じる。高度な作者が内部構造を開く場合も、
同じidentityと診断を保った明示的な段階開示とし、初心者経路の前提にしない。

`一カプセル = 一物理file`、`一Vism = 一言語`、複数seatを一つの新しい言語へ融合すること、のいずれも
本書では決めない。一回の応答が複数の独立した利用者目的、採用単位、公開契約変更を同時に含むことは認めない。

カプセルのclosureはHostやscaffoldが保持してよいが、不可視にしてはならない。依存、型、fixture、由来、
loss、許可read setはInspect可能でなければならない。

## 4. 表現programをparameter制御へ縮めない

本書は将来の作者programを`scalar → parameter`へ限定しない。Path、Text、Instance、Field、Texture、
Data、Geometry等のtyped identityを、必要なrasterize／materialize境界まで保持して生成、変換、分解、反復、
合成、解析できる余地を閉じない。

ただし、[横断stress test](2026-07-29-vism-cross-culture-expression-stress-test-observation.md)の
Path→Path、補助typed output、Declared Feedback、Data→Data、Surface／Material／Geometryは**観察中の空席**である。
本書から新しいkind、trait、Document field、serde面、payload classを逆算しない。

作者programが外部のoperationを必要とする場合、次を守る。

- ambientな`app`、`document`、scene全体、layer名、property path、identity採番を渡さない。
- 巨大な`motolii.std.*`名前空間を自由に呼ばせない。
- 必要能力は型付きinput／capability要求として宣言し、Host／Kitがproviderを解決する。
- providerの実装identity、install path、実装言語を作品意味にしない。
- program内部の純utilityと、外部providerを要求する能力を混ぜない。
- 未知能力は推測や文字列lookupで補わず、typed failureと正規の解決候補を返す。

これにより、小さいVismとKitの型付き接続を維持しながら、一つの表現が複数の専門能力を組み合わせる余地を
残す。具体的なmodule、import、dependency、version形式は未決である。

## 5. 再帰・反復・Feedbackを同じ語で扱わない

| 分類 | 意味 | 処分 |
|---|---|---|
| R1 有限構造再帰 | 深度または要素上限のあるfractal、L-system、Path細分 | MaterializeまたはPure Live候補。Hostが深度、要素数、CPU、memory、deadlineを強制する |
| R2 authoring時展開 | 一回実行し、有限の通常Shape／typed recipe／command候補へする | 第一選択。全体preflight後に1 macroまたは変更0 |
| R3 同一時刻の有限DAG | Path→Path→typed mask等の非循環chain | Host／Kitが型とcycleを検査する候補。公開席は個別締結 |
| R4 時間Feedback | 前回出力が次stepの意味入力になる | 通常DAGやFilterの隠れstateへ入れず、Host所有Simulation／Feedback／Bakeへ送る |
| R5 無上限再帰・固定点待ち | `while true`、無上限展開、収束まで未定回数 | admissionで拒否。deadline超過を成功や近似へ丸めない |

R1とR2を許すためにR5を許さない。R4を表現可能にするために`&self`、global、static、前frame textureへ
状態を隠さない。

## 6. 製品が所有し、editorへ投棄しない作者UX

Motoliiは独自の汎用code editor、IDE、formatter、debuggerを製品責任にしない。特定の外部editorも
必須にしない。一方、editorを持たないことを理由に作者経路を不可視にしてはならない。製品は少なくとも
次を所有する。

1. 公開表現から1操作で開くInspect面。
2. 現在の対象、入力、Preview、versionを保ったFork。
3. 独立identityと上流由来の表示。
4. 変更カプセルの生成、受理、完全性検査、preflight差分。
5. expected／actual、resource、欠落能力、回復候補を含む製品内診断。
6. last-good Previewと、採用／Cancel／stale拒否の明示。
7. 初心者には一つの作者面だけを見せ、依存graph、生成物、seat、runtimeを必要に応じて段階開示する導線。

ここでいう一つの作者面は、巨大な単一sourceや独自統合言語を意味しない。利用者が編集する正本と現在の目的が
一つに見え、内部の分割・生成・接続を理解しなくても最初の有意味な表現を完成できるという製品責任である。
内部をInspectしたい作者には隠さず開くが、理解を入場券にしない。

外部editor、CLI、将来のchat LLMは、いずれも非信頼なproposal供給源である。どの供給源もHost oracle、
single writer、permission、resource、conformanceを迂回しない。

## 7. chat-only LLMについて公約する範囲

chat-only LLMを現時点の製品機能または全payload共通の入口と表示しない。現行には外部scaffold、local loader、
live runtime、package、一般Materialize adapterがなく、成立を証明していない。

一方、次を将来の**可搬性制約**として保持する。

- 一turnごとに自己完結し、過去会話をauthorityにしない。
- repo、package、複数source全体の探索をLLMへ要求しない。
- Hostがtarget、read set、型、依存能力、fixture、diagnosticを閉じる。
- 応答の成功宣言を無視し、Hostの実検査だけを証拠にする。
- 修復時は前turnへの暗黙依存でなく、新しい診断込みカプセルを生成する。

最初に成立し得るchat-only利用者経路は、runtime常駐を要しないMaterialize候補である。live script、WASM、
WGSL、Simulationへ同じclaimを一般化しない。

## 8. 変更カプセルの必須負例

| ID | 入力／故障 | 必須結果 |
|---|---|---|
| ACG-N1 | 外部応答の部分diff、`残りは同じ`、複数fence、複数変更単位 | 全体reject。部分適用0。§3のHost導出内部artifactは本負例の対象外 |
| ACG-N2 | 応答がfixture、expected値、oracle、resource上限を変更 | reject。応答側の書込範囲外で判定 |
| ACG-N3 | 未宣言API、import、provider、capabilityの使用 | typed reject。文字列lookupや暗黙installを行わない |
| ACG-N4 | copy-paste切断、encoding破壊、完全性hash不一致 | reject。欠落を推測補完しない |
| ACG-N5 | 許可read set外の内容、secret、token、絶対path、個人情報がカプセルへ混入 | 生成時reject。外部送信後の回収を安全策にしない |
| ACG-N6 | 開始revisionがstale | reject。黙示merge、別対象への適用0 |
| ACG-N7 | `COMPLETE`、`tests passed`等の宣言だけでpayloadなし、空変更、未実行 | 宣言を無視し、Host検査だけで判定 |
| ACG-N8 | 一応答が複数の独立した利用者目的、採用単位、公開契約変更を含む | reject。親成果を一粒へ縮小して再生成 |
| ACG-N9 | wall clock、未宣言random、network、filesystem、process、environment、入力eventへ依存 | 既存の経路分類／capability拒否へ接続し、成功扱いしない |
| ACG-N10 | 最初の独自表現に複数file／言語／seat間配線、生成物の手編集、build graph理解を要求 | 初心者経路としてreject。Host導出か単一作者面へ戻す |

## 9. 理想を現実へ接続する証拠順

| Phase | 対象 | 証明すること | 証明しないこと |
|---|---|---|---|
| ACG-P0 | 本書と負例表 | ゴール、claim boundary、STOP | API、schema、runtime |
| ACG-P1 | VSM-A4S（仕様着手可・未実装）が対象とするsource fork journeyの一粒 | 一つの可視source／Previewから、Hostが閉じたfixture→外部変更→既存conformance→候補採用へ進む骨格 | 一般利用者、local Vism、chat-only、live runtime、複数内部artifactの一般解 |
| ACG-P2 | Materialize一粒 | parameter mappingでないtyped domain生成、全体preflight、1 Undo、stale拒否 | Path→Path一般席、Simulation、任意script runtime |
| ACG-P3 | stress test §7の個別fixture | Path→Path、typed mask、Data→Data等を一席ずつ比較 | 空席の一括採択、万能program API |
| ACG-P4 | PP-Gate／VSM-B4／containment runtime粒の後続 | 採択されたfrontend／payload／runtimeの作者経路 | 本書だけからのTS、WASM、WGSL採択 |

2D collisionとPath→Path→analysis→Filterを一粒へ束ねない。前者はSimulation／Bakeの意味、後者は
typed identityと複数能力の構成を反証する別fixtureである。

## 10. 既存契約接続票

| 粒 | AUTHORITY | INTERNAL TARGET | OWNER | WRITE ROUTE | GAP | RESOLUTION ROUTE | DISPOSITION |
|---|---|---|---|---|---|---|---|
| 作者経路 | Creator / Developer連続体 | 既存6審判 | 連続体正本 | 変更なし | なし | 再掲だけ | PASS |
| 一回一カプセル | D2 1 gesture=1 macro、Materialize、External Bridge | `DocumentWriter`と将来batch preflight | Host coordinator | 本書→個別fixture | 論理形式と製品往復未実装 | ACG-P1/P2 | RESOLVE |
| 非parameter表現program | authoring journey比較、cross-culture stress test観察 | 未決空席 | 未確定 | 個別loss table／fixture | 型、席、寿命、provider未決 | ACG-P3 | REDUCE |
| program authoring／TS | [言語境界決定](2026-08-01-vism-authoring-language-boundary-decision.md) | MTS-1候補profile | Host authoring toolchain | LANG-TS-F0/F1 | 言語方針採択済み、profile／engine未成立 | F0意味診断→C2後F1 | RESOLVE |
| Formula／inline式 | PP-Gate | `ParamSource`凍結契約 | PP-Gate裁定 | PP-1〜6 | TS採択からinline式は決まらない | 個別比較 | RESOLVE |
| WGSL／Rust／WASM | Microkernel、malware containment | static registryと予約variant | 将来runtime粒 | VSM-B4/C2/D系 | ABI、loader、package未決 | 既存STOP順序 | REDUCE |
| editor非所有と作者UX | 連続体、dev-experience | Inspect／Fork製品面は未実装 | Host product | ACG-P1以後 | 最小製品面未締結 | §6の7責任を個別粒化 | RESOLVE |
| 一つの作者面 | Creator / Developer連続体、作者journey | 未決。複数artifactを束ねる公開形式はない | Host product | ACG-P1/P2のfixture | 初心者表示、段階開示、内部導出の境界未締結 | 一つのsource／Previewから始めるfixtureを先行 | RESOLVE |
| chat-onlyカプセル | External Bridge同型境界、INF-7gの限定証拠 | なし | 未確定 | ACG-P2候補 | 全経路未成立 | 負例先行 | RESOLVE |

## 11. P0レビュー拒否条件

次のいずれかを含む設計、mock、spec、実装は即REJECTする。

1. 宣言なしに呼べるambient operation namespace、Host DOM、scene query、名前検索、identity採番権。
2. base revision照合なし、部分commit可能、失敗後に一部だけ残るカプセル適用。
3. payloadと同じ書込範囲にあるfixture、oracle、expected値、resource上限。
4. 観察中の空席を本書だけから公開kind、API、schemaへ昇格すること。
5. R4／R5をFilter、static、global、`&self`、前frame textureの隠れstateで偽装すること。
6. TSをNode、npm、browser、特定engineとの互換で定義すること。
7. 許可read set外のsource、secret、個人情報を外部カプセルへ含めること。
8. 通常制作UIへ言語、seat、payload、runtimeの選択を出すこと。
9. first-partyだけが内部operation、raw Document、内部texture、内部analysis結果へ到達すること。
10. 特定editor、外部service、network到達を作者経路の成立条件にすること。
11. 初心者の最初の独自表現に、複数file、複数言語、seat間配線、生成artifactの手編集を要求すること。

## 12. 決定・比較中・未実装

### 本書で決定

- 行き止まり禁止、一回一変更カプセル、Host所有oracle、宣言能力、初心者には一つの作者面という五不変条件。
- AviUtl continuity floorをACG-O6〜O10へ翻訳し（O6は第五不変条件由来）、Lua互換でなく作者成果として維持すること。
- 製品がInspect、Fork、カプセル往復、preflight、診断、atomic adoptionを所有すること。
- 内部artifactが複数でも、通常作者には一つの表現、入口、編集、Preview、採用単位として見せること。
- 表現programをparameter mappingへ縮約しない一方、空席を一括採択しないこと。
- 再帰／反復／FeedbackのR1〜R5分類と停止線。
- chat-onlyを現行機能でなく可搬性制約として扱うこと。

### 比較中のまま維持

- [後続決定](2026-08-01-vism-authoring-language-boundary-decision.md)で採択したTypeScript authoring sourceのMTS-1 profile、engine、module、dependency、payloadへのlowering。
- Formula／inline式。
- Path→Path、補助typed output、Declared Feedback、Data→Data、Surface／Material／Geometry。
- source closure、module、dependency、payload class。
- 一つの作者面を実現するsource／recipe形式と、内部artifactの導出・段階開示方式。

### 未実装・本書から着手不可

- local Vism authoring、out-of-tree第三者scaffold、package、loader、install store。
- live JS／expression／WASM runtime、runtime所有WGSL、GPU deadline／worker isolation。
- chat-only製品導線、カプセルschema、公開ABI、Document field。

## 13. 非目標とSTOP

- 本書から`PureValue`、`SimulationStep`、`RenderKernel`等の第二の分類体系を作らない。
- `PathPlugin`、`FeedbackPlugin`、`DataOperatorPlugin`等のkindを作らない。
- **本書だけから**TypeScript、QuickJS、Luau、WASM、WGSL、Rustを採択しない。TypeScript program authoring sourceは[独立した言語境界決定](2026-08-01-vism-authoring-language-boundary-decision.md)で採択済みだが、engine、inline式、payload、packageの採択を意味しない。
- Motolii内に汎用code editorやIDEを実装しない。
- chat LLMを信頼境界、reviewer、test runner、oracleにしない。
- カプセルをDocument／Vism packageの恒久schemaとして先に保存しない。
- ACG-P1とACG-P2、2D collisionとPath構成を一つの実装粒へ束ねない。

新しい公開API、Document意味、plugin契約、永続形式、runtime、package、permissionが必要になった時点で、
該当粒だけSTOPし、既存の解凍手続きと独立仕様へ戻す。親ゴールや別の証拠粒まで停止しない。

## 14. 反対側レビュー

2026-07-31にFable 5へ、候補ゴール、現行正本、コード事実、Path／解析／Filter／Simulationの要求、
chat-only利用者、独自editor非所有をread-onlyで渡した。

初回判定は**REVISE**。未決の空席、TS、chat、複数runtimeを一文の後戻り不能な公約へ混ぜる案はREJECT。
一方、行き止まり禁止、一回一カプセル、Host所有oracle、宣言typed capabilityは既存契約と同型であり、
新runtimeを発明せず決定可能とした。本書はその訂正を反映した。

その後、利用者から「一本のcodeでなくなる時点で初心者を排除し、構造を理解しにくくするのではないか」
という反例が出た。これを好みでなく入場条件の欠陥として扱い、第五不変条件、ACG-O6、ACG-N10を追加した。
この追加後のFable実diff再審査は**REVISE（P0=0、P1=3）**。五不変条件の再設計は不要とした一方、
見出し数量、逆引き台帳、反例由来の記録漏れを拒否した。本書はその三件と状態表現のP2を修正し、
最終再審査へ送った。最終判定は**ACCEPT（P0=0、P1=0）**。一つの作者面と内部typed seat分離、
一カプセル一目的と複数のHost導出内部artifactは別軸として両立し、未決事項の黙示採択、first-party特権、
初心者排除、状態粉飾はないと判定された。最終指摘P2二件も、ACG-N1の対象限定と本判定記録へ反映した。
