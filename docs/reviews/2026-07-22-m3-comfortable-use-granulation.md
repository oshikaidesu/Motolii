# M3 快適利用ワークマップ粒度化（2026-07-22）

状態: **Fable全粒レビュー合格 / Codex採否済み / 実装発注ではない**

## 1. 目的

[快適利用ワークマップ](2026-07-22-m3-comfortable-use-work-map.md)のW0a〜W6を、依存、owner、正例、負例、
STOPを一つずつ検証できる粒へ分ける。本書は粒の母集団と合流順を固定するが、公開API、Document、journal、
plugin契約、永続形式、未決UXを採択しない。

既存M2/M3/M4 task IDが既に正しい粒を持つ場合は再定義せず、そのtaskとLocal Alpha経路の**合流粒**だけを置く。
各粒を実装発注へ上げる時は最新mainの型とpathを再監査し、closed orderの変更許可ファイルを確定する。

## 2. 状態語と粒の種類

| 語 | 意味 |
|---|---|
| `DECIDE` | 意味、gate、失敗時正本、owner、公開/永続境界の仕様改訂が先 |
| `DO` | 既決仕様と依存が揃い、closed order化できる |
| `WAIT` | 依存未到達。実装発注しない |
| `HUMAN` | 人間による目視・聴感・操作審判 |
| `MEASURE` | 固定環境とraw結果を保存する計測。値の採択とは分ける |
| `HARDWARE` | 現在所有していない実機が必要。synthetic PASS禁止 |
| `DONE` | main到達と証跡が正本に記録済み |

粒の種類は`SPEC`、`ASSET`、`CORE`、`PRODUCT`、`E2E`、`HUMAN`、`MEASURE`、`HARDWARE`とする。
`SPEC`粒の完了は実装許可ではなく、直接後続だけを`DO`へ上げる。

各表の「依存」は、その成果が意味上成立するための**論理依存**を記す。現行Selected U seriesで同時点の`DO`を
1件に保つ**運用順**はCU-G02とimplementation ledgerが別に決める。運用上先に実行するだけの粒を論理依存へ
混ぜず、逆に論理依存は運用順の都合で省略しない。

## 3. 全粒を拘束する不変条件

1. Document変更はD2 single writerだけ。UI、React、native surface、fixtureに別writerを作らない
2. 全surfaceは同じrevision付きHost snapshotをread-only投影し、selection/playheadを同期コピーしない
3. React製品資産は固定sourceを直接移管し、mockをconsumerへ反転する
4. Stage/Timelineの座標描画面はnative wgpu owner。Reactはchrome、form、tool panelだけ
5. UI threadでdecode、GPU readback、blocking send、unbounded scanを行わない
6. PreviewとExportは同じ評価関数を通り、差は`Quality`だけ
7. Cancel、失敗、duplicate、stale、reloadでDocument/history/revision/counterを部分変更しない
8. UI都合でDocument、journal、plugin/community公開契約、永続layout形式を増やさない
9. golden、threshold、期待値を実装都合で変更しない
10. 未決に遭遇した粒は隣接機能を実装せず`ORDER: STOP`で該当SPEC粒へ戻る

## 4. 先行する仕様・順序粒

### CU-G01 G0-9段階化仕様改訂

| 項目 | 内容 |
|---|---|
| 種類 / 状態 | `SPEC / DECIDE` |
| 一成果 | fixed-Mac platform prerequisite evidenceとdistribution gateを別ID・別証跡にする |
| authority | M3 G0-9、product mock recovery §6、快適利用map §3.2/W0g |
| 変更面 | `docs/specs/M3-ui-integration.md`、`docs/implementation-ledger.md`、`docs/ui-runtime-architecture.md`、G0-9現行decisionの追補、decision index |
| 正例 | local gate合格で固定Macのplatform prerequisite evidenceだけを限定確定し、W0b、H1b、Motolii Studio Preview、window結合、parent G0-9とdistribution gateは未完了のまま残る |
| 負例 | WindowsをMac結果でPASS、IME/AX/crashをdistributionへ追放、R0〜R6のvisual合格だけでlocal統合解禁 |
| STOP | 対応platform範囲、解禁対象、parent gate状態が一意に書けない |

### CU-G02 Local Alpha critical path順序改訂

| 項目 | 内容 |
|---|---|
| 種類 / 状態 | `SPEC / WAIT`（CU-G01） |
| 一成果 | Selected U seriesへRectangle D2、Vector、U3a、U2h subset、D5、project lifecycle、Exportの合流順を入れる |
| authority | implementation ledger、M3直列順、Rectangle contract §10、work map §9.1 |
| 変更面 | `docs/implementation-ledger.md`、`docs/specs/M3-ui-integration.md`、既存U枝番表、decision index |
| 正例 | 同時点で`DO`は1粒だけ。既存U4a/U4c/U2c-2の依存を壊さずW1/W2へ合流する |
| 負例 | U2h全体を一commit化、RectangleをDelete intentへ偽装、U3a前に三面E2Eを完成扱い |
| STOP | W1と既存Selected U seriesを同時に満たす順が作れない |

### CU-G03 edit durability順序契約

| 項目 | 内容 |
|---|---|
| 種類 / 状態 | `SPEC / DECIDE` |
| 一成果 | `apply_macro`、journal commit、drag terminal、snapshot/selection publishの順序と失敗時正本を決める |
| authority | M2 D1d/D1m/D2、U2b、Rectangle contract §8/§11 |
| 変更面 | M2仕様のD2/journal追補、Rectangle D2 decision、M3 U2b依存記述、decision index |
| 正例 | 成功時だけdurable editと整合snapshotが一度見え、reopenで同じDocument意味を復元する |
| 負例 | journal失敗後に成功snapshotをpublish、UI retryで二重適用、selectionをjournalへ保存 |
| STOP | crash境界の正本または既存journal互換を一意に決められない |

### CU-G04 project lifecycle製品入口仕様

| 項目 | 内容 |
|---|---|
| 種類 / 状態 | `SPEC / DECIDE` |
| 一成果 | New/Open/Save/reopenの最小製品入口、typed failure、session所有をM3 task化する |
| authority | M2 D1c/D1c-FU/D1m、VSM-A0S catalog必須open、work map W3 |
| 変更面 | M3仕様、UI concept-to-tickets後続表、implementation ledger、decision index |
| 正例 | catalog必須sessionからopenし、save/reopenでDocument意味とunknown保持が不変 |
| 負例 | raw低水準openを製品UIから使用、lock steal、未来版を黙って書換え、unsaved UXを暗黙決定 |
| STOP | Unsaved Changes、Save As、read-only newer等の必要範囲が未統一 |

### CU-G05 Export製品入口仕様

| 項目 | 内容 |
|---|---|
| 種類 / 状態 | `SPEC / DECIDE` |
| 一成果 | Export開始、設定、progress、cancel、成功検証、失敗表示をM3 task化する |
| authority | M1 T9/G2〜G7、M2 Document≠ExportJob/D6、U1i、work map W3/W5a |
| 変更面 | M3仕様、必要なexport UI decision、implementation ledger、decision index |
| 正例 | Document外`ExportJob`から実行し、検証済み成果物だけを成功表示する |
| 負例 | cancel後に成功表示、encoder/disk/ffprobe失敗の文字列潰し、部分fileを最終pathへ残す |
| STOP | ExportJob既存型、provider snapshot、cancel可能点、atomic renameの責任が不明 |

### CU-G06 Local Alpha fixture manifest決定

| 項目 | 内容 |
|---|---|
| 種類 / 状態 | `SPEC / WAIT`（CU-G03〜G05） |
| 一成果 | Rectangle、動画、任意Soundtrack、parameter、key/easing、move/trim、Delete/Rename、save/reopen、Exportの一作品を固定する |
| authority | work map §6、U0e fixture manifest規律、M1 real-material oracle |
| 変更面 | testkit/fixture ownership文書とmanifestのみ。Document schemaではない |
| 正例 | 全surfaceが同じrevision/ID/selectionを報告し、Preview/Exportが同じ意味を示す |
| 負例 | missing asset reopen、export cancel/failure、corrupt/future project、corrupt workspace、WebView/surface復旧 |
| STOP | fixtureを成立させるため製品special-caseまたは新しい所有層が必要 |

### CU-G07 D5/U5段階完了境界

| 項目 | 内容 |
|---|---|
| 種類 / 状態 | `SPEC / DECIDE` |
| 一成果 | D5のpre-U5計測配線、U5製品接続、post-U5最終E2Eを別粒としてM2/M3仕様へ固定する |
| authority | M2 D5行、M3 U5行、implementation ledgerのD5統合pending |
| 変更面 | `docs/specs/M2-document-model.md`、`docs/specs/M3-ui-integration.md`、`docs/implementation-ledger.md`、decision index |
| 正例 | CU-209はD5完了を名乗らず、CU-210接続後のCU-212だけがD5最終審判を閉じる |
| 負例 | D5未完のままU5を完了、U5無しの10分fixtureでD5完了、TransportをUI側で再実装 |
| STOP | pre/post U5の責任と証跡を分離できない |

### CU-G08 keymap製品入口判断

| 項目 | 内容 |
|---|---|
| 種類 / 状態 | `SPEC / DECIDE` |
| 一成果 | Local Alphaはversion付きJSON fallbackで閉じるか、設定画面・保存場所をM3 task化するか決める |
| authority | U0d-1/2非目標、M3実装ガード5、G0-2 |
| 変更面 | M3仕様、U枝番表、implementation ledger、decision index |
| 正例 | fallback採用時はCU-403をpost-Alphaへ送り、UI採用時はUser settings ownerとcodecを明記 |
| 負例 | 保存場所、GUI import/export、OS予約処分をPRODUCT粒で暗黙決定 |
| STOP | keymap UIなしではLocal Alpha不能という新要件を仕様化できない |

### CU-G09 Browser catalog projection契約

| 項目 | 内容 |
|---|---|
| 種類 / 状態 | `SPEC / WAIT`（CU-0A03） |
| 一成果 | Browser itemごとのsubtype/availability/motion/impact/provider/tagをDocument外Host read modelへ閉じる |
| authority | React直接移管契約 §7、G0-3分離、Browser既決表示 |
| 変更面 | Host read-model decision、M3仕様/枝番表、decision index。Document/plugin公開契約は変更しない |
| 正例 | unknown/dangling/non-finite/oversizedをtyped拒否し、fixtureと製品が同じdecoderを使う |
| 負例 | catalog ID/label/thumbnail token/indexから意味を推測、`??`でもっともらしく表示 |
| STOP | 必要fieldがplugin/community公開契約またはDocument schema変更を要求 |

### CU-G10 clipboard意味判断

| 項目 | 内容 |
|---|---|
| 種類 / 状態 | `SPEC / WAIT`（CU-106） |
| 一成果 | Copy/Pasteのpayload、同一document範囲、ID remint、Shared Effect処分を決めるかpost-Alphaへ送る |
| authority | U2h非目標、A8 Independent duplicate、D1l Shared Effect |
| 変更面 | M2/M3該当spec、decision文書、implementation ledger、decision index |
| 正例 | 採用範囲が1 Undo、subtree内参照だけ再写像、cross-document非対応を明示可能 |
| 負例 | OS clipboard JSONをraw Document mutation口にする、Shared Effect意味をUIで発明 |
| STOP | payload/再写像/欠落plugin処分が未統一 |

## 5. W0a 製品資産所有

| ID | 種類 / 状態 | 一成果 | 依存・再利用 | 合格と必須負例 | STOP |
|---|---|---|---|---|---|
| CU-GR01 | `GUARD / DONE` | Fable原因レビューのdispatch・scope・検収証跡guardをclean integration branchへ固定 | U0e-2却下証跡、発注ガード正本、既存delegate test harness | stale BASE/authority、WAIT粒、React label欠落、許可外path、検収write/timeoutを実装担当起動前後で機械拒否 | bypass、公開契約変更、既存差分の巻込みが必要 |
| CU-0A01 | `ASSET / DONE` | U0e-2R固定React baselineを最新mainへ再結合 | U0e-1、固定`eb16d06` | React build/既存43 tests、後発decision保持。新UI判断・archive逆実装0 | 固定treeと一致しないsourceしかない |
| CU-0A02 | `ASSET / DONE` | U0e-2の5 reference screen入力を固定 | CU-0A01、U0e-2契約 | 同一fixtureのnormal/lightness/grayscale/CVDを30 PNGとprovenance/atomic generationへ固定。人間判断をpixel testへ置換しない | 現行codecで三層fixtureを閉じられない、または既存React source closureが不足 |
| CU-0A03 | `SPEC / WAIT` | R0 source inventory/provenance manifest | CU-0A01、固定SHA`56c318ed` | path/export/CSS/model/test closure全列挙、legacyとnative面を誤分類しない | 固定sourceまたはclosure不在 |
| CU-0A04 | `ASSET / WAIT` | R1 Browserをproduct ownerへ直接移管 | CU-0A03 | 17-state対象visual 1%以下、DOM/class/ARIA/interaction維持、mock側copy 0 | 別leaf/CSS後追い/opaque ID分岐が必要 |
| CU-0A05 | `ASSET / WAIT` | R2 Easing trigger/要約だけを移管 | CU-0A04 | popup全体をnative oracleに残し、Reactにcurve/Undo state 0 | popup全体をReact製品面へ持込む必要がある |
| CU-0A06 | `ASSET / WAIT` | R3 KEYS/LAYERS tool panelをnative Timelineから分離移管 | CU-0A05 | tool panel DOM/CSS維持、ruler/bar/key/playhead移管0 | TimelineCandidate全体のimportが必要 |
| CU-0A07 | `ASSET / WAIT` | R4 Inspectorを固定mock内で同形React化して移管 | CU-0A06 | legacy parity後に単一owner、skeleton代用0、legacy runtime import 0 | 正しい同形componentを作れず意味発明が必要 |
| CU-0A08B | `CORE / WAIT` | Browser fixture stateをrevision付きprojection/typed intentへ交換 | CU-0A04、CU-G09 | unknown/non-finite/oversized/dangling拒否、React semantic write 0 | catalog field不足をID/labelから推測したくなる |
| CU-0A08E | `CORE / WAIT` | Easing trigger stateをHost projection/typed intentへ交換 | CU-0A05、U4b契約 | key上/区間外disabled理由、curve state二重所有0 | Interp/区間意味の新契約が必要 |
| CU-0A08K | `CORE / WAIT` | KEYS/LAYERS stateをHost projection/typed intentへ交換 | CU-0A06、既存U3a/U4a/U4b projection契約 | selection/packingをReactが所有しない、reloadでHostから復元 | native/React間に双方向storeが必要 |
| CU-0A08I | `CORE / WAIT` | Inspectorのfixture stateを既決fieldだけのread-only Host projection/typed intent境界へ交換 | CU-0A07、現行NodeDesc/DocParam | Document clone/history 0、unknown field typed拒否。実編集接続はU4a/U4cへ残す | 未決fieldを補わないと正しい製品面を表示できない |
| CU-0A09 | `PRODUCT / WAIT` | R6 diagnostic routeをproduction navigationから分離 | CU-0A08B/E/K/I | 通常routeは正しい製品面、diagnosticはdevelopment限定 | diagnostic画面しか成立しない |

React粒のclosed orderは直接移管契約の`REACT AUTHORITY`から`STOP`まで8ラベルを順番どおり持つ。

## 6. W0g fixed-Mac platform prerequisite evidence

| ID | 種類 / 状態 | 一成果 | 依存 | 合格と必須負例 | STOP |
|---|---|---|---|---|---|
| CU-0G02 | `MEASURE / DONE` | direct wgpu(+Vello局所)対eguiを同条件比較 | CU-G01、既存G0-9 fixtures | 同一scenario/input/source digestでraw frame/input/RSS、present/acquire、resource creation/readback、skipを保存。絶対閾値・勝者判定なし。Terra実装、Grok P0/P1/P2=0、workspace試験全緑 | fixture・device・window条件が非同一 |
| CU-0G03H | `CORE / DONE` | L2人間審判用local acceptance harness surface | CU-G01、CU-0G02、既存G0-9 surface host | bounded AccessKit tree、実first-responderを伴うnative→Web→native focus ring、ready/layout epoch、実DOM composition/shortcut sink、fullscreen/minimize別観測。Sol利用不能のためClaude Sonnet fallback実装を採用し、Grok/FableともP0/P1=0でACCEPT（P2は非blocking観測）、20 lib + 3 main tests/build/docs全緑。製品意味・人間PASS・L3 failure injection 0 | 製品windowへ昇格、synthetic IMEで代用、公開UI/Document契約が必要 |
| CU-0G03H2 | `CORE / DONE` | native/Web focus往復のmacOS機械E2E追補 | CU-0G03H | 実NSWindowの正逆8遷移、実first-responder class、native NSEvent monitor / Web DOM relay到達元、修飾Tab非奪取を固定。20 lib + 7 main tests、実window E2E、workspace/docs全緑、Grok/FableともP0/P1=0でACCEPT | synthetic IME/VoiceOverの代用、製品window昇格、topology変更、製品keyboard routingへ一般化が必要 |
| CU-0G03 | `HUMAN / DONE` | macOS IME/VoiceOver/focus復帰受入 | CU-0G03H2 | ユーザーがG0-9L local acceptance harnessの実windowでpreedit、候補位置、確定/取消、Enter/Esc/Space漏出0、bounded AX、機械記録済みfocus経路へのVoiceOver追従、fullscreen/minimize後のfocus/IME復帰を人間審判して合格。synthetic focus結果から外挿せず、[人間証跡](../spikes/g0-9-surface-host.md#cu-0g03-人間審判2026-07-24)を保存 | local acceptance harnessの実windowで試せない、機械focus試験を人間PASSへ読み替えたくなる |
| CU-0G04 | `MEASURE / DONE` | 100回resize/DPI/capture/lostとWebView crash復旧 | CU-G01、CU-0G03 | 同じ実windowで102 resize、実DPI event、合成capture、注入Lost再present、実WebContent終了とwry callback/backoff/reload/再Ready、offline既定拒否、stale layout/WebView epoch拒否を[単一manifest](../spikes/g0-9-surface-host.md#cu-0g04-lifecycle--failure実機審判2026-07-24)へ保存。sentinel/resource/semantic write不変 | synthetic resizeだけで実機合格にしたくなる |
| CU-0G05L | `SPEC / DO` | local gate判定とfixed-Mac platform prerequisite evidenceの限定確定を記録 | CU-0G02〜04 | P0/P1=0、対象Mac構成と未合格platformを明記。W0b、H1b、Motolii Studio Preview、window結合を解禁しない | Windows/追加hardwareまで合格、または製品粒を解禁と書きたくなる |

## 7. W0b 製品window統合

| ID | 種類 / 状態 | 一成果 | 依存 | 合格と必須負例 | STOP |
|---|---|---|---|---|---|
| CU-0B01 | `HUMAN / WAIT` | G0-6Hで5画面の階層・識別・馴染みを判定 | CU-0A02 | `docs/mocks-ui/reference-handoff.md`へ判定者/条件/採否理由を保存。agentが目視代行しない。G0-9LをPASS・省略・迂回しない | reference画面欠落 |
| CU-0B02 | `PRODUCT / WAIT` | U0e-3 token/component state/iconを製品導入 | CU-0B01 | contrast/focus/意味色+形、raw color/spacing拒否 | 新画面固有componentが必要 |
| CU-0B03 | `PRODUCT / WAIT` | H1b codec/offline bundle/mountをHostへ接続 | CU-0A09、別途確定する製品前提 | CDN/HMR/fixture script依存0、invalid/stale codec拒否。G0-9L evidenceを依存充足にしない | WebView transportを公開plugin APIへ一般化したくなる |
| CU-0B04N | `PRODUCT / WAIT` | native Stage/Timeline viewportを1 top-level Surfaceへ接続 | CU-0B02、別途確定する製品前提 | 同一device/queue、CPU readback 0、viewport resizeでsemantic不変。G0-9L evidenceを依存充足にしない | native viewportごとに別正本Surfaceが必要 |
| CU-0B04R | `PRODUCT / WAIT` | opaque child WebView islandsをHost layoutへ接続 | CU-0B03、CU-0B04N | transparent overlay 0、focus/geometry epoch、React semantic state 0。G0-9L evidenceを依存充足にしない | DOM/pxをnative/Document identityへ使いたくなる |
| CU-0B05 | `E2E / WAIT` | reload/crash/focus/resize後にHost snapshotから再投影 | CU-0B04N/R | 同じrevision/selection、old epoch拒否、Document/history不変 | surface間state同期が必要 |

## 8. W1 対象の連続性

| ID | 種類 / 状態 | 一成果 | 依存 | 合格と必須負例 | STOP |
|---|---|---|---|---|---|
| CU-101 | `SPEC / WAIT` | Rectangle Placeのtarget/start/duration/recipe/position/nameを決定 | CU-G02 | 正準Y-up/RationalTime、UI px/DPI/DOM型0 | いずれかを暗黙defaultで埋める必要 |
| CU-102 | `SPEC / WAIT` | fresh LayerId + AddTrackItem原子性を採択 | CU-101、CU-G03 | live-next検査、失敗時counter/history/revision不変、journal互換 | 公開raw ID mint/汎用transactionが必要 |
| CU-103 | `CORE / WAIT` | `VectorRecipe::StandardShape::Rect`をD3/GPUへlower | M2 Vector意味、CU-101 | preview/export pixel同一、fixture rectで代用しない | 新Vector意味・golden更新が必要 |
| CU-104 | `SPEC / WAIT` | U2h selection publish/Undo/Redo policyを決定 | CU-G02/03 | Document snapshotと整合したTransient selection envelope | selectionをDocument/journal/Undoへ保存したくなる |
| CU-105 | `CORE / WAIT` | U3a layout/hit-test/dense Timeline projectionを閉じる | 既存U3a | 1000 clip/100k key、zoom境界でselection/playhead/range不変。G0-9L evidenceを依存充足にしない | React TimelineまたはDOM identityが必要 |
| CU-106 | `CORE / WAIT` | U2h selection kernelとessential focusを実装 | CU-104/105 | Stage/Timeline/Inspector同じstable ID、hidden selection件数+戻る | surface別selection storeが必要 |
| CU-107 | `CORE / WAIT` | drag epoch/sequence/dedupe coordinatorを製品Hostへ接続 | 既存D&D spike、CU-0B05 | preview/terminal配送、Esc/outside/capture loss、stale/duplicateをD2未接続で検証 | transport IDをDocumentへ保存したくなる |
| CU-109 | `CORE / WAIT` | journal commitとsnapshot publishをCU-G03順序で製品edit runtimeへ配線 | CU-G03、U2b/D1m | journal失敗時publish 0、再open同値、retry二重適用0 | UI側journal writerまたは新永続payloadが必要 |
| CU-110 | `CORE / WAIT` | Place intent/requestからfresh ID plannerと1 macro commitを接続 | CU-102/107/109 | preview中D2 0、valid dropでAddTrackItem/apply_macro各1、失敗/cancel 0 | 公開planner/汎用transactionまたはraw ID mintが必要 |
| CU-111 | `PRODUCT / WAIT` | Undo/Redo製品CommandIdとsingle-writer配送を接続 | CU-109、U0c/U2b | 成功時だけsnapshot publish、失敗でDocument/history不変、UI history 0 | Undo/Redoをsurface別local stateにしたくなる |
| CU-108 | `E2E / WAIT` | Rectangleを三面へ投影しUndo/Redoする | CU-103/106/110/111、CU-0B05 | 同じrevision/LayerId、Undoで三面から消えRedoで同ID復帰 | diagnostic/fixture-only rectしか表示できない |

## 9. W2 制作ループ

| ID | 種類 / 状態 | 一成果 | 依存 | 合格と必須負例 | STOP |
|---|---|---|---|---|---|
| CU-201 | `CORE / WAIT` | U3b move/trim/snapをD2へ接続 | CU-105/106 | random操作列、相対位置、Undo全巻戻し、Cancel 0 | marker/beat未決を同時実装したくなる |
| CU-202 | `CORE / WAIT` | U4a-1 ValueType→control→command対応model | U2b-1、既存U4a-1契約 | 全保存param対応またはtyped拒否、新ValueType 0 | plugin独自UIが必要 |
| CU-203 | `PRODUCT / WAIT` | U2c-3共通feedback component | CU-0B02、U2c-1 | state matrix、理由+回復、色/文字単独依存拒否 | 個別picker/popup state machineが必要 |
| CU-204 | `PRODUCT / WAIT` | U2c-5 Brief/Context/Inspect/Assistive投影 | CU-203、U2c-4 | reason/subject/facts一致、recovery通常Intent経由 | diagnosticからDocument直接mutationが必要 |
| CU-205 | `PRODUCT / WAIT` | U4a-2自動Inspectorとnonblocking preview | CU-202/204、U1b | 100 slider updates、latest preview、1 gesture=1 Undo、stale拒否 | custom plugin panelへ逸脱 |
| CU-206 | `PRODUCT / WAIT` | U4b keyframe/区間Easingを製品接続 | CU-205、native Easing core | drag write 0/release 1/Esc 0、非対象curve不変 | key構造やthreshold変更が必要 |
| CU-207 | `PRODUCT / WAIT` | U4c Advanced意味検査とroundtrip | CU-205/204、D1l | open/close serialize不変、非既定意味badge、未実装pipeline偽装0 | 新公開Param APIが必要 |
| CU-208 | `E2E / WAIT` | U2c-2 Direct/Advanced同値conformance | CU-205/207 | 同じDocument意味/Undo。Tool未実装を明記 | hidden helper/空harnessでしか通らない |
| CU-209 | `MEASURE / WAIT` | D5のpre-U5 GPU timestamp/preview-loop計測配線を閉じる | CU-G07、D3/D4/D4-FU | D5完了を名乗らず、audio主clock、GPU正本、計測可能な最小loopを証明 | U5側でTransportを再実装したくなる |
| CU-210 | `PRODUCT / WAIT` | U5 seek/scrub/playback UI | CU-209、CU-201（U3b）、CU-0B04N/R | vsync/repaint暴走でもclock不変、latest seek、停止後idle | UI repaintを主clockにしたくなる |
| CU-212 | `MEASURE / WAIT` | U5接続後のD5 10分実機E2Eと最終完了判定 | CU-210、CU-G07 | drift、frame drop追従、GPU計測配線、M2/M3証跡同期 | pre-U5骨格だけでD5 DONE |
| CU-211 | `E2E / WAIT` | Rectangle制作ループを完走 | CU-201/206/208/212 | 置く→変える→key/easing→trim→再生→Undo/Redo | 保存/Exportまで完成と過大申告 |

## 10. W3 実素材・project・Export入口

CU-301/302はU6のLocal Alpha動画subsetであり、これだけでU6 parent完了を名乗らない。SVGはM4-K6後、
waveformはU3c、beat snapはU7としてpost-Alphaへ残し、この経路の暗黙依存にしない。

| ID | 種類 / 状態 | 一成果 | 依存 | 合格と必須負例 | STOP |
|---|---|---|---|---|---|
| CU-301 | `PRODUCT / WAIT` | U6 Files/Project read-only探索と動画preview | CU-0B04N/R、U0e | UI thread decode 0、range変更でDocument/Undo不変、欠落/codec診断 | filesystem stateをDocumentへ保存したくなる |
| CU-302 | `PRODUCT / WAIT` | U6 Inbox受取と動画配置 | CU-301、CU-G03、U2b | 確定時だけClip/TimeMapへ1 Undo、range負例、duplicate/stale拒否 | Inboxを第二asset owner/履歴にしたくなる |
| CU-303 | `PRODUCT / WAIT` | Soundtrack 1本を設定 | CU-302、M2 audio境界 | Soundtrack無しでも同じ制作経路、設定1 Undo。欠落/corrupt/unsupported codecはtyped拒否 | 音楽中心別mode/timelineが必要 |
| CU-304 | `PRODUCT / WAIT` | U1f Stage View/Output Frame/off-frame表示 | CU-0B02、D1k/D3f | pan/zoom/fitでDocument/Final不変、枠外選択可、readback 0 | K0最適化を完成条件にしたくなる |
| CU-305 | `PRODUCT / WAIT` | U2d Camera/Output Frame直接操作 | CU-304、U2c | Camera 1 gesture=1 Undo、Hand/Fit workspaceのみ、DPI不変 | camera/object操作を混同 |
| CU-306O | `PRODUCT / WAIT` | New/Open製品入口 | CU-G04 | session/catalog必須、lock/未来版/corrupt typed拒否 | raw open、lock steal、未決New defaultが必要 |
| CU-306S | `PRODUCT / WAIT` | Save/reopen製品入口 | CU-G04、CU-G03、CU-306O | durable save/reopen同値、unknown保持、失敗時原本不変 | 未決Save As/Unsaved UXが必要 |
| CU-307 | `PRODUCT / WAIT` | Export設定・開始・progress/cancel UI | CU-G05、既存export runtimeの型付きprovider snapshot（U1i原則） | ExportJobはDocument外、cancel可否を偽装せず、UI closeで結果不変 | UI所有export queueが必要 |
| CU-308 | `E2E / WAIT` | Export atomic outputと失敗復旧 | CU-307、既存export | cancel/encoder/disk/probe/missing assetでtyped failure、partial final file 0、finish保証 | golden/期待値変更または文字列error化が必要 |
| CU-309 | `E2E / WAIT` | 実素材をsave/reopenしてExport | CU-302/303/305/306S/308 | 同じDocument意味、Preview/Export同一評価、成果物probe合格 | fixture専用sourceやCLIだけでしか通らない |

## 11. W4 日常操作

W4のうちCU-401A/401B/402〜406は依存が揃えばW3と並行できる。CU-401Bはclipboard仕様採択時だけ、
CU-403はkeymap UI採択時だけ行い、CU-407 detachはLocal Alpha後へ送る。

| ID | 種類 / 状態 | 一成果 | 依存 | 合格と必須負例 | STOP |
|---|---|---|---|---|---|
| CU-401A | `PRODUCT / WAIT` | Delete/Duplicate/Rename essential command surface | CU-106、対応D2操作 | CommandId+preflight、Independent remint、1操作=1 Undo、unsupported reason | Linked copy/Shared Effect意味をUIで発明 |
| CU-401B | `PRODUCT / WAIT` | 採択範囲だけCopy/Pasteを接続 | CU-G10、CU-401A | typed payload/preflight、ID remint、1 Paste=1 Undo、unsupported明示 | cross-doc/Shared Effect/欠落plugin意味を拡張 |
| CU-402 | `PRODUCT / WAIT` | U3e Timeline navigation/search/filter | CU-105/106 | 1000/100kでnonblocking、filtered selectionを保持/表示 | display名をidentityへ使用 |
| CU-403 | `PRODUCT / WAIT` | CU-G08で採択した場合だけkeymap設定入口と競合診断 | CU-G08、U0d、CU-204 | base不変、user delta、未知CommandId保持、Document不変 | OS予約や保存場所をPRODUCT粒で決定 |
| CU-404 | `HUMAN / WAIT` | 長文日本語IME/focus/keyboard navigation | CU-0B04N/R、U0c | preedit下線/候補位置/Enter非奪取、focus order、shortcut抑止 | synthetic key eventだけでPASS |
| CU-405 | `PRODUCT / WAIT` | panel resize/open/close/dockのWorkspace保存 | CU-0B04N/R、U1a-3 layout authority | corrupt profile全reset、Document/Undo/Final不変 | toolkit tree/pxをDocumentへ保存 |
| CU-406 | `E2E / WAIT` | disabled/error/context helpを共通投影 | CU-204、CU-401A/402 | Brief/Context/Inspect/Assistive facts一致、silent disabled 0 | 外部検索必須または直接mutation |
| CU-407 | `PRODUCT / WAIT` | detach/re-dock/別window接続 | Local Alpha、detachable contract | 同じsnapshot/selection/playhead、window/DPIはDocument外 | surface別state cloneまたは未所有hardware PASS |

## 12. W5a Local Alpha応答・復旧

| ID | 種類 / 状態 | 一成果 | 依存 | 合格と必須負例 | STOP |
|---|---|---|---|---|---|
| CU-5A01 | `MEASURE / WAIT` | U1c起動/idle/input/drop/scrub/parameter raw計測 | CU-211/309 | 環境とraw値保存、開発HUD、閾値を同時採択しない | GPU/ffmpeg未実行を緑扱い |
| CU-5A02 | `PRODUCT / WAIT` | Local Alphaで実在するExport/素材配置providerだけU1i activity投影 | CU-204、CU-302/307 | queued/running/completed/failed/cancelled、unknown progress、cancel可否正直。M4-K4 import-proxyは要求しない | UI所有queue/偽cancel |
| CU-5A03 | `E2E / WAIT` | reload/crash/surface lossから再構成 | CU-0B05、CU-106、CU-309 | Document/selection不変、latest snapshot、old epoch拒否 | local cacheからsemantic復元 |
| CU-5A04 | `E2E / WAIT` | Local Alpha統合fixture通常経路 | CU-G06、CU-309、CU-401A/402/404〜406、CU-5A01〜03 | 通常製品起動からExportまで完走、JSON keymap fallback可、diagnostic route不使用 | 一部をCLI/fixture-onlyで代用 |
| CU-5A05 | `E2E / WAIT` | Local Alpha統合fixture負例 | CU-5A04 | missing asset、cancel/failure、corrupt input、reload/crashで正本不変と回復表示 | failureを成功fixtureから除外 |
| CU-5A06 | `HUMAN / WAIT` | Local Alpha日常操作審判 | CU-5A04/05 | 教材なし完走、操作摩擦をM3/Vism/保留へ分類 | 新表現要求をその場でDocumentへ追加 |

## 13. W5b 高負荷時縮退

M4のK0/K1a/K1b/K1c/K4/K1d/K7/K8は各M4仕様の既存粒を正とし、本書で再定義しない。

| ID | 種類 / 状態 | 一成果 | 依存 | 合格と必須負例 | STOP |
|---|---|---|---|---|---|
| CU-5B01 | `SPEC / WAIT` | Local Alpha実測から必要M4 provider coverageだけを列挙 | CU-5A06、M4 provider inventory/spec | provider未実装を明示し、全K-chainを暗黙必須にしない | cache/resource意味の変更が必要 |
| CU-5B02 | `PRODUCT / WAIT` | U1g latest-time preview縮退 | CU-5B01、CU-5A01（U1c計測）、CU-210（U5）、K1d | audio/time不変、表示frameだけdrop、capacityだけではdropしない | preview専用評価関数が必要 |
| CU-5B03F | `PRODUCT / WAIT` | U0f resource policy User settings | CU-5B01、G0-8/K1a | codec roundtrip、Document/Undo不変、hard cap反映 | backend free VRAMを正本化 |
| CU-5B03H | `PRODUCT / WAIT` | U1h performance/memory HUD | CU-5B02/03F | 理由を文字+icon、100 telemetry nonblocking、HUD非表示でも制御同一 | HUDが制御正本になる |
| CU-5B04 | `PRODUCT / WAIT` | U3f time-local readiness投影 | CU-5B01、CU-5B03H（U1h）、K1b/K7/K8、CU-105 | provider snapshot一致、1000区間nonblocking、未取得≠ready | readinessをDocument/cache policyへ逆流 |
| CU-5B05 | `E2E / WAIT` | 高負荷fixtureで縮退と回復を確認 | CU-5B01/02/03F/03H/04 | deadline/pressure分離、latest追従、Final全frame | 閾値/golden緩和で合格 |

## 14. W6 Distribution Ready

| ID | 種類 / 状態 | 一成果 | 依存 | 合格と必須負例 | STOP |
|---|---|---|---|---|---|
| CU-601 | `HARDWARE / WAIT` | Windows WebView2/PMv2/MS-IME/NVDA/offline受入 | Local Alpha、distribution gate | Local Alpha同一fixture、ProcessFailed復旧、z-order/capture/DPI | macOS/synthetic結果の外挿 |
| CU-602 | `HARDWARE / WAIT` | 異DPI/第二monitor/HDR-SDR/pen受入 | Local Alpha、追加hardware | move/resize/detachでDocument/評価不変、実device evidence | 所有しないhardwareをPASS |
| CU-603 | `HUMAN / WAIT` | 対応Mac構成のdistribution再審判 | Local Alpha | monitor/fullscreen/AX/IME/process termination matrix | local gate結果の無検査転載 |
| CU-604 | `E2E / WAIT` | platform別同一制作fixture結果を統合 | CU-601〜603 | 対応platform全て証跡、未対応は明示、preview/export意味同一 | 一platform失敗を既知制約へ格下げ |
| CU-605 | `SPEC / WAIT` | distribution gateとparent G0-9を完了判定 | CU-604 | spec/ledger/decision index同期、P0/P1=0 | 未完hardwareをDONE扱い |

## 15. 合流順

```text
CU-G01 G0-9段階化 ─> CU-G02 order ───────────────┐
                                                  |
CU-0A01..09 assets ───────────────────────────────┴─> CU-0B01..05 window ─┤
                                              |                |
                                              +─> CU-107/108 ──┤
CU-0G02..05L fixed-Mac evidence（W0b/CU-105非依存・非解禁）
CU-G03 durability ──────────────────────────────> CU-101..111 W1
                                                               v
CU-G07 D5/U5境界 ─> CU-209 ─┐
CU-201 U3b ──────────────────┴─> CU-210 U5 ─> CU-212
CU-201..208/211/212 W2 ────────────────────────────────────── 制作loop
       |
       +─> CU-G04 lifecycle ─> CU-306O/S ─┐  （CU-G04/G05の仕様判断自体はW2と並行可）
       +─> CU-G05 export ─────> CU-307/308 ─┤
       +─> CU-301..305 media/stage ─────────┤
                                           v
                                      CU-309 実素材E2E
CU-401A/402/404..406 daily ops ─────────────┤
CU-G06 fixture ─────────────────────────────┤
                                           v
                                    CU-5A01..06 Local Alpha
                                      /                 \
                             CU-5B01..05                CU-601..605
                             高負荷縮退                  Distribution Ready
```

同じ行に見える粒も、現行Selected U seriesの`DO` 1件運用を自動解除しない。CU-G02が正本化した順だけを使う。

## 16. 全粒Fableレビューgate

実装発注前にClaude Codeの`claude-fable-5`へ本書と全authorityをread-onlyで渡し、次を監査する。

1. Local Alpha通常経路に未所有の入口、循環、到達不能gateが残っていないか
2. 各`SPEC`粒が未決を本体実装から隔離できているか
3. 既存M2/M3/M4 taskの重複planner/helper/stateを要求していないか
4. React直接移管、native座標面、single writer、snapshot所有を全粒で維持しているか
5. 正例だけで合格する粒、Cancel/失敗/stale/reload/Undo/save/reopen/Export負例が欠ける粒がないか
6. W5aがM4 chainを暗黙に取り込み、W5bがLocal Alphaを再び塞いでいないか
7. W6を延期箱にせず、対応platformの終了条件を機械的に追跡できるか
8. 1 Issue = 1 commitへさらに分割すべき粒、逆に分けすぎて単独検証不能な粒がないか

判定は`ACCEPT FOR ORDER DRAFTING`または`REVISE GRANULATION`とする。レビュー助言だけで`DECIDE/WAIT`を
`DO`へ上げず、Codexが既存正本とコード事実へ照合して採否を本書へ戻す。

## 17. Fable全粒レビューとCodex採否

Claude Codeの`claude-fable-5`をread-onlyで使い、地図、全粒、M2/M3/M4仕様、implementation ledger、
React直接移管契約、Rectangle D2契約、現行コード事実を横断監査した。

| 回 | 判定 | 指摘 | Codex採否 |
|---|---|---|---|
| 初回 | `REVISE GRANULATION` | P0=0 / P1=5。G0-9重複、D5/U5境界、journal/Place/Undo owner、SVG、keymap等 | 全件採用し、SPEC/CORE/PRODUCT粒を分離 |
| 再回 | `REVISE GRANULATION` | P0=0 / P1=2。CU-210→U3b、CU-5B04→U1hの論理依存漏れ。P2=5 | P1/P2全件採用し、依存・負例・図を修正 |
| 最終 | `ACCEPT FOR ORDER DRAFTING` | P0=0 / P1=0。表記上のP2=3 | 3件とも採用。循環0、Local Alpha到達可能、隠れたM4 chain 0を確認 |

Fableの判定は助言であり、Codexが各指摘を正本とコード事実へ照合して上表のとおり採択した。これにより本書は
closed orderを**作成できる母集団**になったが、各粒の`DECIDE/WAIT`、現行ledger、発注時precheckを解除しない。

## 18. 現在の停止線

- CU-0G03はユーザーによる実windowの実IME／VoiceOver／復帰審判で合格し、CU-0G04も責任最小化短票どおり既存winit/wgpu/wry/macOS標準経路を使う単一manifestで合格した。直近の実行候補は`CU-0G05L`だけであり、反対側review後も固定Mac prerequisite evidenceの限定確定以外を解禁しない
- CU-G01〜CU-G10は仕様判断粒で、コード実装と同時に行わない
- closed orderはユーザーの明示的な発注時だけ作り、各粒の論理依存、現行ledger、仕様判断を再確認する
- FableがP0/P1相当を出した場合は粒の修正へ戻り、レビューを再実行する
