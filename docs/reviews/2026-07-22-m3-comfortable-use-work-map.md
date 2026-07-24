# M3 快適利用までのワークマップ（2026-07-22）

状態: **利用者成果地図 / Local Alpha・Distribution Ready完成線の正本 / 実行順の正本ではない**

## 1. 目的

M3の残作業を「UI部品を増やす一覧」ではなく、Motoliiを日常的に使って小さな作品を
完成できるまでの一本の制作経路として並べ直す。

現時点ではUIの外観、所有境界、入力、single writer、snapshot配送等の基礎が揃っている。一方で、
product-owned React面、native Stage/Timeline、Document編集、selection、実素材、previewを同じ製品windowで
つないだ制作経路は未完了である。本書はこの残差の**大地図**だけを定める。個別Issue、公開API、schema、
永続形式、具体的な実装ファイルは本書で決めない。現在の実行動線は
[M3縦slice実行方針](2026-07-24-m3-vertical-slice-execution-decision.md)と
[implementation ledger](../implementation-ledger.md)を正とする。

## 2. Authorityと位置づけ

本書は既存仕様の意味や完了条件を置換しない。衝突時の優先順位は次とする。

1. `docs/specs/M*.md`と判定済みdecision文書
2. `docs/ui-concept.md`の「最初の結果」
3. `docs/implementation-ledger.md`の現行依存・着手順
4. [M3縦slice実行方針](2026-07-24-m3-vertical-slice-execution-decision.md)と本書の体験単位
5. 粒度化履歴snapshotとIssue

本書を根拠にDocument、journal、plugin契約、公開API、永続layout形式を追加・変更してはならない。
React製品資産は[直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)、UI状態所有は
[M3 UI境界規律](2026-07-14-m3-ui-boundary-prevention.md)、製品surfaceは
[UI runtime責任境界](../ui-runtime-architecture.md)とG0-9に従う。

### 2.1 M3-A〜Dとの対応

[M3仕様](../specs/M3-ui-integration.md#m3-ad-統合の背骨)のA〜Dは、各縦slice内で使う接続checklistを
`M3-A Presentation Ownership → M3-B Host Projection / Intent → M3-C Product Runtime Integration
→ M3-D Editing Loop`と呼ぶ。本書のW地点との対応は次の通りであり、A〜Dは進捗軸ではない。

| 段階 | 本書の主対応 | 役割 |
|---|---|---|
| M3-A | W0a | 製品資産の所有移管を閉じる |
| M3-B | W0a〜W1のprojection/intent境界 | mock stateをHost境界へ交換する |
| M3-C | W0g/W0b | platform gate後に通常製品windowを結合する |
| M3-D | W1/W2とLocal Alpha fixture | D2/Undoを含む制作ループを閉じる |

M3-A〜DはW0〜W6を置換しない。W0〜W6も強制実行列ではなく、縦sliceが到達する成果の地理である。
W3の実素材、W4の日常操作、W5の応答・復旧、W6の配布品質は独立したM3完成線として残る。

## 3. 二つの完成線

### 3.1 Local Alpha

現在の主開発Macで、教材やdiagnostic routeに頼らず、通常製品windowから次の流れを完走できる状態とする。

```text
起動
  -> 素材またはShapeを追加
  -> Stage / Timeline / Inspectorで同じ対象を選択
  -> 値を変更
  -> keyframeとeasingを設定
  -> seekまたは再生して結果を確認
  -> Undo / Redo
  -> 保存 / 再起動 / 再表示
  -> Export
```

この流れでPreviewとExportは同じ評価関数を通り、UI threadの同期readback、surface別Document clone、
surface別selection、React側Undoを持たない。

### 3.2 Distribution Ready

Local Alpha後に、Windows、異DPI、第二monitor、対象platformのIME/accessibility、offline runtime、
WebView process failure、surface/device lossの対象環境審判を通した状態とする。現在のMac上の成功をWindowsや
未所有hardwareへ外挿しない。

Local AlphaとDistribution Readyを一つのgateに束ねない。まず日常利用を始められる地点を作り、そこで判明した
操作摩擦をM3へ戻す。新しい表現意味はVISM側の探索候補として分離する。

ただし現行[M3仕様 G0-9](../specs/M3-ui-integration.md#m3仕様確定ゲートg0)はWindows実機を含む全審判の合格まで
WebView/native製品統合を停止している。このままでは主開発Mac上のLocal AlphaがDistribution Readyより先に
到達できない。地図自身で既決gateを分割せず、最初に**G0-9段階化仕様改訂**を行う。

- `G0-9L`候補: 主開発Mac上のplatform prerequisite evidenceだけを限定確定するlocal gate
- `G0-9D`候補: Windows・追加hardwareを含むdistribution gate

IDと完成条件は後続仕様改訂で確定する。仕様改訂と反対側レビューが終わるまで、W0b以降の製品統合を発注しない。

## 4. 全体地図

| 地点 | ユーザー成果 | 主な既存境界 | 出口 |
|---|---|---|---|
| W0a 製品資産所有 | React資産が製品の一意なownerへ移る | U0e-2R/2、React R0〜R6 | mockがproduct exportのconsumerになり、二重copyとlegacy runtime importが0 |
| W0g platform gate段階化 | 固定Mac prerequisite evidenceとDistribution Readyの証跡が仕様上分かれる | G0-9、platform分類、M3 spec/ledger | fixed-Mac evidenceとdistributionの審判が別ID・別証跡で閉じ、製品粒を解禁しない |
| W0b 製品window統合 | 通常起動で正しい製品UIが現れる | W0a、G0-6H、U0e-3、H1b、別途確定する製品前提 | Browser、Inspector、KEYS/LAYERS、native Stage/Timelineが正しいownerから表示される |
| W1 対象の連続性 | 一つの対象が三面を貫く | Rectangle D2個別契約、Vector経路、U2h、U3a | 同じrevisionと`LayerId`をStage、Timeline、Inspectorが表示し、Undoで同時に消える |
| W2 制作ループ | 配置した対象へ時間変化を作れる | U3b、U4a、U4b、U4c、U2c-2、U5、M2-D5 | 配置、値変更、key、easing、trim、seek、再生、Undo/Redoが一続きになる |
| W3 実素材・project・Export入口 | 空のprojectから作品を始めて成果物を得る | U6、U1f、U2d、D1c/D1m、既存media/audio/export境界、未定義の製品入口 | 動画/楽曲を追加し、New/Open/Save/reopenとExportを通常製品面から完走できる。SVGはK6合流後 |
| W4 日常操作 | 頻出操作で行き止まらない | U2h、U3e、U1d、keymap、layout、diagnostic | 選択、複製、削除、rename、検索、focus、IME、panel復元が安定する |
| W5a Local Alpha応答・復旧 | 通常反復とsurface再生成で制作の流れが止まらない | U1b、U1c、U1iの成立provider分、local crash/reload recovery | 最新generation、計測、再投影、typed activityをM4 cache chainなしで確認できる |
| W5b 高負荷時縮退 | 重いprojectでも正本時刻を保って最新結果へ追いつく | U1g、U1h、U3f、G0-8、M4 K0〜K1d/K7/K8の該当provider | resource pressureとdeadlineを分離し、縮退理由とreadinessを表示できる |
| W6 配布品質 | 対応環境で同じ制作経路を通せる | distribution platform gate、Windows/追加hardware/AX | Distribution Readyのplatform matrixが証跡付きで閉じる |

依存の大筋は次とする。

```text
W0a 製品資産所有 ─────────────────────────────┐
                                                v
                                      G0-6H -> U0e-3
                                                v
                                      W0b 製品window統合
                                                v
                                      W1 同一対象の三面縦切り
                                                v
                                      W2 制作ループ
                                                v
                                      W3 実素材の入口
                                                v
                                      W4 日常操作のLocal Alpha subset
                                                v
                                      W5a Local Alpha応答・復旧 -> Local Alpha
                                                                       |
                                                       +---------------+---------------+
                                                       v                               v
                                             W5b 高負荷時縮退                  W6 配布品質
                                                                                       v
                                                                              Distribution Ready

W0g G0-9段階化仕様改訂 -> fixed-Mac prerequisite evidence
                                      (W0b非依存・非解禁)
```

これは能力ごとの全タスクを強制的に直列化する図ではない。W4のうち実素材に依存しない選択・診断・keymap等は
W3と並行できる。現行Selected U seriesはW1のRectangle/U2hと矛盾するため、粒度化結果をそのまま発注せず、
U3a後に必要なRectangle D2仕様粒とU2h selection publish粒を置く**spec/ledger順序改訂**を先行させる。
改訂前に「追い越さない」と自己申告して既存順序を迂回しない。

## 5. 各地点の範囲

### W0a 製品資産所有

- 固定React sourceをproduct ownerへ直接移し、mockをproduct exportのconsumerへ反転する
- Browser、Easing trigger、KEYS/LAYERS、InspectorをR0〜R6の順で所有移管する
- Inspectorは固定モック内でlegacy出力を同形React化し、parity後に移管する
- StageとTimelineの座標描画面をnative wgpu ownerに保つ
- 通常routeとdevelopment専用diagnostic routeを分離する

出口では、縮約component、legacy runtime import、mock/product二重copy、surface別semantic stateが0である。

### W0g platform gate段階化

- 現行G0-9の証拠をfixed-Mac platform prerequisite evidenceとdistributionへ仕様上分離する
- 主開発Macのlocal gateにもrenderer同条件比較、IME、VoiceOver、resize/DPI/capture/lost、WebView crashを残す
- Windows実機、第二monitor、異DPI/HDR等をdistribution gateへ移しても免除・synthetic PASSにしない
- M3仕様、implementation ledger、UI runtime責任境界の効力語彙を同じ改訂で同期する
- parent G0-9の「全platform審判完了」とfixed-Mac evidenceの「限定確定」を混同しない

この仕様改訂が反対側レビューを通ってもW0b、H1b、Motolii Studio Preview、window結合を解禁しない。G0-6Hは独立し、U0e-3とW0bの製品前提を停止したままにする。

### W0b 製品window統合

- U0e reference fixtureとG0-6Hの人間審判を経てU0e-3の製品token/component stateを導入する
- product-owned React chromeとnative Stage/Timeline viewportを通常製品windowへ合成する
- codec、offline bundle、focus、IME、AX、resize、capture、lost、WebView crashをlocal gateの実機条件で確認する
- diagnostic routeを製品画面の代替にしない
- React、native surface、Host coordinatorが同じrevision付きsnapshotをread-only投影する

出口は主開発Macの通常製品windowで正しいownerの画面が表示され、reload/crash後もHost正本から再構成できることとする。

### W1 対象の連続性

最初の製品縦切りは既決のRectangle経路を使う。

```text
React Browser Rectangle intent
  -> Host coordinatorのTransient preview
  -> release時だけD2 single writerへ1 macro
  -> Arc<Document> + Transient selectionの整合したsnapshot
     -> native Preview
     -> native Timeline bar
     -> React Inspector
  -> Undo 1回で三面から消える
```

粒度化前に、Rectangleのtarget、start、duration、recipe、正準位置、LayerId発行、journal durability、
selection publish、Redo時selection policyが既存decision/specで決まっているかを個別に監査する。未決を
もっともらしいdefaultで埋めない。

`apply_macro`、journal `commit_edit`、drag terminal化、snapshot/selection publishの順序と失敗時正本を
個別仕様で決める。現行U2bのsnapshot配送だけをdurability成立と数えない。

必須負例はEsc、Stage外drop、capture/focus loss、duplicate/stale message、commit失敗、React reloadである。
drag中のsemantic writeは0、release時の`apply_macro`は1回とする。

### W2 制作ループ

- Timeline barの移動、trim、snapをD2 commandへ接続する
- `NodeDesc`から全保存parameterを編集できるInspectorを成立させる
- parameterの連続変更を最新generation previewへ流し、gesture全体をUndo 1回にする
- keyframeと区間Easingを編集し、Esc/focus loss時は変更0にする
- DirectとAdvancedが同じDocument意味とUndoを持つことを実在入口で検証する
- seek、scrub、再生で古いgenerationを表示せず、Transportの正本時刻をUI都合で変えない
- M2-D5の本番preview loop、GPU timestamp配線、実機E2EをU5の前提として閉じる

出口はRectangle一つでも「置く、変える、動かす、再生する、戻す」を教材なしで完走できることとする。

### W3 実素材の入口

- 同じExplorer内でProjectとFilesを明示的に切り替える
- 動画をpreviewし、source In/OutをDocument外で調整する。SVGはM4-K6合流後の追加経路とする
- 配置確定時だけ既存Clip/TimeMap意味へ変換し、1 Undoにする
- 楽曲を1本設定できるようにするが、Soundtrack無しでも制作経路を成立させる
- Stage View pan/zoom/fitとOutput FrameをDocument/Final不変のviewとして接続する
- 欠落、不正asset、範囲外、不対応codecをtyped diagnosticとして表示する
- UI threadでdecodeせず、PreviewとExportの評価経路を分岐させない
- New/Open/Save/reopenの製品入口を新しいM3 taskとして仕様化し、D1c/D1mのsession所有境界だけを使う
- Exportの製品入口を新しいM3 taskとして仕様化し、Documentと`ExportJob`を混載しない
- export cancel、encoder失敗、disk不足、検証失敗、欠落assetでDocument/historyを変えず、部分fileを成功扱いしない
- 一時file→検証→atomic rename、`Encoder::finish()`、型付き失敗の既存export境界を再利用する

出口は固定fixtureではなく、ユーザーが選んだ動画/楽曲を通常製品面から作品へ加え、保存・再open・Exportまで
完走できることとする。製品入口のtask ID、Unsaved Changes、Save As等の未決UXはslice昇格時にも発明せず、仕様判断へ戻す。

### W4 日常操作

- Stage、Timeline、Inspectorのsingle/additive/range/marquee selectionを同じTransient selectionへ正規化する
- Delete、Duplicate、Renameを登録済み`CommandId`とpreflightへ通す。Copy/Pasteはclipboard意味の仕様判断後だけ追加する
- 対応D2操作が無い対象はsilent disabledにせず、理由と次の一手を示す
- Fit All/Selection、Go to Playhead、前後clip/key移動、検索、filterを同じ時間面へ戻る操作として揃える
- filterや折畳みで隠れたselectionを無言で失わない
- Local Alphaではpanel resize、開閉、dockを正しいWorkspace ownerへ置く。detachはpost-Alpha候補として別粒にする
- version付きJSON keymap fallback、長文日本語IME、focus、keyboard navigation、context helpを実製品windowで確認する。
  keymap設定画面は保存場所と入口の仕様判断後だけ追加する
- panel/WebView再生成後に最新Host snapshotから再構成する

### W5a Local Alpha応答・復旧

- 起動、idle memory、input latency、drop後反映、scrub、parameter更新を計測する
- U1bのgeneration破棄とlatest-frame配送が実製品gestureでも成立することを確認する
- background activityはLocal Alpha時点で実在するproviderだけを型付きsnapshotから投影する
- panel reload、WebView process failure、surface loss後もDocumentとselectionを失わず再投影する
- Exportは全frameを同一評価関数で処理し、Previewのdropや縮退を混入させない

### W5b 高負荷時縮退

- deadline超過時はTransport時刻を変えず、古いpreview要求を捨てて最新時刻へ追いつく
- resource pressureとrender deadlineを別の理由として扱う
- preview scale、実表示fps、予算使用量、縮退理由をDocument外から投影する
- readinessは実在するM4 provider snapshotだけから投影し、未取得をreadyに見せない

具体的な予算値、閾値、hysteresisはG0-8とM4実測前に固定しない。

### W6 配布品質

- Windows 10/11のWebView2、PMv2 DPI、MS-IME、NVDA、offline runtime、`ProcessFailed`復旧
- 異DPI・第二monitor移動、HDR/SDR差、実surface/device loss
- distribution対象Mac構成でlocal gate外のmonitor/HDR等を再確認する
- 対応platformごとの同一Local Alpha制作fixture

未所有hardwareの項目は`WAIT / HARDWARE`として明示し、synthetic testだけでPASSへ上げない。

## 6. Local Alphaの統合fixture

後続の粒度化では、少なくとも次の一作品を共有fixture候補として分解する。

- Rectangle 1件
- 動画 1件
- 任意のSoundtrack 1件
- 保存parameterの変更 1件
- keyframe区間とeasing 1件
- Timeline上のmoveまたはtrim 1件
- DeleteまたはRename 1件
- Undo/Redo
- save/reopen
- Preview確認とExport

同じfixtureから次の負例variantを作る。

- reopen時に素材が欠落している
- export途中cancel
- encoder失敗、disk不足、出力検証失敗
- corrupt/未来版project、corrupt Workspace profile
- React reload、WebView process failure、surface loss

各surfaceは同じrevision、stable ID、selectionを報告する。fixture manifestは試験入力の正本であって、
Document/User settings/Workspace/Project session/Transientの新しい所有層にはしない。

## 7. 状態所有の再確認

| 状態 | owner |
|---|---|
| layer、clip、parameter、keyframe、effect、素材参照 | Document / D2 single writer |
| keymap、resource policy、easing library | User settings |
| panel配置、dock、detach、表示面積 | Workspace profile |
| Timeline scroll/zoom、作業中のview | Project session。具体的な永続codecは既決範囲だけ |
| playhead | runtime ownerはHost coordinator。再open時の永続化は未決であり、仕様判断前にProject sessionへ焼かない |
| selection、hover、drag、popup、診断、preview generation | Transient / Host coordinator |
| focus-visible、未確定IME composition等 | local presentation。ただしHost意味の正本にしない |

## 8. 非目標とVISMへの分離

Local Alphaを閉じるために次を実装しない。

- パーティクルとパス移動の新しい相互関係
- Groupへ表現機能を集約する新しい意味
- Vismのcontainer、loader、保存・配布形式
- 自由plugin UIとcommunity公開契約
- ShapeScript、SVG generator、Feedback Canvasの拡張
- waveform表示とbeat snap（それぞれU3c/U7としてpost-Alpha）
- Duplicator、Depth、Text Motion等の新表現domainの完成
- 新しいDocument field、公開trait、journal variantでUI不足を埋めること

日常利用から「既存操作が不快」と判明したものはM3へ戻す。「どの概念を組み合わせると表現が理解しやすいか」
という問いはVISMの観察・比較候補へ送り、Motolii UIから意味を先回りして固定しない。

## 9. 縦sliceへ昇格する規則

W0〜W6を一度に実装単位へ落とさない。現在sliceへ昇格した成果だけを次の順で具体化する。

1. ユーザーが達成する一操作または一観察
2. 現行spec IDと既存コード事実
3. ownerと読み書き境界
4. 変更許可ファイルと非目標
5. 正例、Cancel/失敗/重複/stale等の負例
6. 自動試験と人間・実機審判
7. STOP条件
8. 1 Issue = 1 commitの依存順

公開API、Document、journal、plugin契約、永続形式へ触れる粒は、通常の仕様改訂と反対側レビューを先行させる。
React所有面は直接移管契約の8ラベルを持つclosed orderになるまで実装へ進めない。

### 9.1 Local Alpha blocking gate

次が全て仕様上閉じるまでLocal Alpha完成を名乗らない。

1. G0-9段階化仕様決定とfixed-Mac platform prerequisite evidence gate合格
2. G0-6Hと製品token導入
3. Rectangle PlaceのD2契約、fresh ID、journal durabilityの製品配線
4. VectorRecipe RectangleのD3/GPU経路
5. U3a Timeline projectionとU2h selection publish/Redo policy
6. M2-D5/U5の段階完了境界、U5接続、最終D5統合審判
7. New/Open/Save/reopen製品入口の仕様とE2E
8. Export製品入口、progress/cancel/失敗、atomic outputの仕様とE2E
9. Local Alpha fixtureの通常経路と負例

## 10. 外部レビューgate

初回Fableレビューは2026-07-22に実施し、`VERDICT: REVISE BEFORE GRANULATION`を受けた。G0-9の到達不能、
D5、Save/Open/Export、Selected U series、M4 chain、playhead ownerの指摘を本改訂へ反映した。

W0a〜W6の項目別分解ができた後、**Claude Fableによるread-only大局再レビュー**を行う。Fableは実装担当や
仕様authorityではなく、地図全体の欠落・循環・過剰scopeを探す反対側助言者とする。

レビューでは少なくとも次を問う。

1. Local Alphaの制作経路に、ユーザーが進めない空白や循環依存がないか
2. 見た目の完成を製品接続の完成と誤認した地点がないか
3. M3へVISMの表現意味、M4のresource意味、M5の表現domainを先取りしていないか
4. Document、User settings、Workspace、Project session、Transientのowner違反がないか
5. Cancel、失敗、stale、reload、Undo/Redo、save/reopen、Exportの負例が各合流点にあるか
6. Local AlphaとDistribution Readyの分離が、platform品質の免除になっていないか
7. 最小の制作fixtureで全地点を審判でき、diagnostic routeだけの成功になっていないか

Fableの出力は未検証の助言として保存し、採否はMotolii仕様と現行コード事実に照らしてCodexが正本へ戻す。
レビューだけで既存STOP、公開契約、タスク順を解除しない。

## 11. 現在の処分

| 対象 | 状態 | 次の行動 |
|---|---|---|
| 本ワークマップ | 利用者成果地図 | Local Alpha / Distribution Readyの完成線を維持し、実行順は縦slice方針へ委ねる |
| Local Alpha制作fixture | 候補 | 既存fixtureと仕様を監査して閉集合を決める |
| G0-9段階化 | 仕様改訂待ち | local/distribution gateをM3 specとledgerへ正本化する |
| Distribution Ready matrix | 計画 | 段階化後のdistribution gateを引き継ぐ |
| Claude Fable初回レビュー | 完了 | P0=1/P1=6を本改訂へ反映 |
| Claude Fable全粒レビュー | 完了 | 2026-07-22時点の候補分解に対する履歴証跡として保持する |
| M3縦slice方針レビュー | 完了 | 2026-07-24 Fable最終`ACCEPT`。VS-1を現在sliceとして運用する |
| 実装発注 | 停止 | 本書の追加だけでは再開しない。ユーザーの明示的な発注依頼を待つ |
