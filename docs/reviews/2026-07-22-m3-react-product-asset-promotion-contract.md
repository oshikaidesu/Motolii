# M3 Reactモック製品資産の直接移管契約

作成日: 2026-07-22
状態: **決定 / 発注停止線**

## 1. 決定

Browser、Inspector、Timeline左側の`KEYS / LAYERS` tool panel、Easing Panelなど
React所有面は、現行Reactモックを見た目だけの参考にして製品用へ作り直さない。
固定したReact source assetを製品packageへ**直接所有移管**し、モック固有の状態と
legacy bridgeだけをHostのprojection / intent境界へ交換する。

```text
固定React asset
  ├─ component tree / DOM / CSS / stable ID / ARIA / interaction testを維持
  └─ mock state / fixture adapter / legacy scriptを交換
                      ↓
             product-owned React package
                      ↓
        Host projection → render → typed intent
```

ここで「DOM/CSSをDocument・公開API・plugin契約へ焼かない」と、
「既にあるDOM/CSS実装を製品資産として再利用しない」を混同しない。前者は従来どおり禁止、
後者は本決定により棄却する。製品所有へ移したcomponentのDOM/CSSは交換可能な内部実装であり、
それ自体を永続意味や公開互換契約にはしない。

本決定はユーザーが発注停止中に行うdocs正本化である。本文書の追加だけでは発注を再開しない。
ユーザーが依頼動詞として改めて「発注」を明示するまで、外部モデルによる発注書作成、実装、検収を
起動しない。

## 2. この決定が必要になった事実

既存文書はReactモックのcomponent、fixture、Storybook、Playwright、stable IDを
「製品候補資産」「比較oracle」と記録した一方、製品化の方法を固定していなかった。
そのため次の誤読が成立した。

1. `docs/mocks-ui`はoracleとしてだけ残す
2. `ui/motolii-web`へ縮約した別componentを新規作成する
3. screenshot差をCSSの追加修理で徐々に近づける
4. projectionに無い表示をopaque catalog IDの分岐で補う

隔離worktreeでこの誤読が実際に発生し、初期状態のpixel mismatchはBrowser 7.693%、
Inspector 5.827%、`KEYS / LAYERS` 2.697%まで残った。寸法は一致していたため、問題は
surface topologyではなく、元React assetを移管せず縮約再実装したことにある。

該当差分は未採用・未commitでmainへ統合されていない。縮約componentはpresentation sourceとして
**棄却**する。ただし、正本sourceと独立して成立するfixture decoder、CSP、offline bundle、mount、
process cleanup、visual harness等の試験資産は、別途コードレビューを通したものだけ再利用候補に残す。

## 3. 用語とauthority

| 用語 | 意味 |
|---|---|
| source asset | 製品へ移すReact component、CSS、model、stable ID、ARIA、interaction test |
| product owner | runtimeでsource assetを一意に所有する製品package。現候補は`ui/motolii-web` |
| mock consumer | product-owned componentをfixtureで組み立て、Storybook/Playwright/oracleとして使う開発入口 |
| projection | Host snapshotから導出したrevision付きread-only表示入力 |
| intent | React操作をHostへ返す型付き要求。Document変更の成否やIDをReactが確定しない |
| diagnostic route | codec、負例、D&D lifecycle等を狭い画面で観察する開発専用route |
| visual oracle | 固定fixture、viewport、font、操作列でsource assetの外観・操作を比較する審判 |

authorityの優先順は次とする。

1. Document、D2、Undo、selection、状態寿命: M2/M3仕様とUI境界規律
2. React/native所有: [UI runtime責任境界](../ui-runtime-architecture.md)
3. React sourceの移管方法: **本文書**
4. 見た目・操作の固定入力: 固定React source assetとそのPlaywright/Storybook fixture
5. diagnostic route: 契約の観察用であり、製品画面や意味のauthorityではない

## 4. 固定sourceと現行inventory

移管元はmerge commit
`56c318edcddab7cf95d263cc2f7dd2b4e6791134`の`docs/mocks-ui/**`とする。
このtreeはU0e-2Rで固定した
`eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0`の`docs/mocks-ui/**`と一致し、
後発mainとReact baselineを再結合した版である。発注時はbranch名や作業中worktreeでなく、このSHAと
対象pathを両方書く。

現行inventoryは次のとおりである。

| 製品面 | 固定source | 処分 |
|---|---|---|
| Browser | `src/candidates/DiscoveryBrowserCandidate.jsx`、`discovery-browser-candidate.css` | componentとCSSを直接移管。legacy shellへのreplace hookだけ外す |
| Easing Panel | `src/candidates/EasingGraphCandidate.jsx`、`easing-graph-candidate.css`、`easing-graph-model.js` | view/modelを直接移管し、区間とcurveをHost projectionへ接続 |
| `KEYS / LAYERS` | `src/candidates/TimelineCandidate.jsx`内の`.candidate-key-tools` subtree、`timeline-candidate.css`の対応規則 | native Timeline本体からReact tool panelだけを同じDOM/CSSのまま抽出して移管 |
| Inspector | `src/legacy/LegacyHostBoundaryScreen.jsx`から`LegacyInspector`へ流れるlegacy DOM/script。`src/surfaces/InspectorSurface.jsx`は縮約skeleton | 正しい独立React componentがまだ無い。先にモック側でlegacy出力を同形Reactへ抽出し、parity後に移管する。skeletonを製品版として使わない |
| 可変panel | `src/layout/ResizablePanelLayout.jsx`、`resizable-panel-layout.css` | Web所有panelのlayout componentだけを移管候補にする。native viewport境界はHost topologyに従う |
| primitives/token | `src/primitives/**`、`src/tokens/**` | 実際にsource assetが参照する閉包を移管。未使用assetを一括portしない |
| Stage / time surface | legacy Stage、`TimelineCandidate`のruler/rail/bar/key/playhead/graph | 製品Reactへ移管しない。native wgpu所有のoracleとしてのみ残す |

「Vite上でReactが描画している」だけではReact-native assetと判定しない。
`html-react-parser`、`legacyBody`、`legacyStyle`、`legacyScript`、`Function(...)`、raw HTML由来の
DOMまたはglobal query/listenerに依存する面はlegacy bridgeである。正しいReact sourceが無い面では、
製品packageに別の縮約版を作らず、固定モック内で同形React componentへ抽出する工程を先に置く。

## 5. 所有移管の形

source assetはmockとproductへcopyして二重所有しない。次の順で単一ownerへ移す。

1. 固定source path、export、CSS closure、Storybook story、Playwright操作をmanifest化する
2. legacy bridge依存がある面は、固定モック内で同形Reactへ抽出してvisual/interaction parityを通す
3. component、model、CSSをproduct packageへ履歴を追える形で移す
4. `docs/mocks-ui`をproduct packageのconsumerへ反転し、fixture/story/oracleからproduct exportをimportする
5. product packageから`docs/mocks-ui`、`src/legacy`、raw HTML、fixture scriptへの依存が0であることを検査する
6. mock側に旧component copyが残っていないことをclosure testで検査する

移管の第一commitはpresentation ownershipだけを変え、Document操作やHost transportを同時実装しない。
第二commit以降でmock stateをprojection / intentへ一境界ずつ交換する。

## 6. 維持するものと交換するもの

### 6.1 原則として維持する

- componentの責任分解と入れ子
- DOM順、class、stable component ID、既存の`data-*` test hook
- CSS selector、layout、theme token参照、icon/textの表示
- role、label、ARIA state、keyboard/focus order
- panel、tab、search、card、tool、presetの既存interaction
- Storybook story、Playwright操作列、visual fixture
- mockで決定済みの表示状態。未決意味は表示fixtureのままにしてDocumentへ逆算しない

維持対象は内部実装の移行baselineであり、永久不変の公開DOM APIではない。製品移管後のUI改善は、
製品component、fixture、visual/interaction test、決定台帳を同じ変更で更新する通常手続きへ移る。

### 6.2 交換する

- Document、keyframe、curve、selection、Undoを模したcomponent内`useState`
- legacy scriptが行うDOM query、global listener、直接mutation
- raw HTML parserと`Function(legacyScript)`による初期化
- 仮JSONを暗黙に信頼する入力
- surface間で値を同期するeffectまたは双方向store
- fixture専用のID採番、history、semantic reducer

交換後の正規経路は一つだけとする。

```text
Host Document / Transient coordinator
  → revision付きprojection
  → product-owned React component
  → typed intent
  → Host coordinator / D2 single writer
  → 新しいsnapshot
```

React内に残せるstateはhover、popover開閉、未確定input composition、focus-visible等、
再mount時にHost意味を失わないlocal presentation stateだけである。selection、drag terminal、keyframe、
easing、Undo、Document revision、stable object IDはReactの正本にしない。Workspace profileやProject sessionへ
属するstateも、対応するHost projection / intentが決まるまでlocal既定値で恒久化しない。

## 7. Browser metadataの境界

Browser cardのsubtype、availability、motion preview、impact badge、provider/source、tag count、icon等は、
見た目を合わせるためopaque `catalogId`、label、配列index、thumbnail tokenから推測しない。

- componentの役割に固定された装飾はcomponent-owned presentation metadataにできる
- itemごとに異なる意味はHost catalog projectionの型付きfieldから表示する
- projectionにfieldが無ければ、fixtureとdecoderを含む契約改訂を先に行う
- unknown field、unknown enum、dangling category/provider/tag参照は診断して拒否する
- `?? "Effect"`等で壊れた参照をもっともらしい表示へ縮退しない

これはReact source assetを維持するためにDocument schemaへUI metadataを足す許可ではない。
catalog projectionはDocument・plugin manifest・公開community contractと別のHost read modelとして審判する。

## 8. diagnostic routeの分離

契約確認用の縮約画面は削除しなくてよいが、正しい製品画面の代替にしない。

- 通常route: product-owned Browser / Inspector / `KEYS / LAYERS` / Easing Panelを表示
- diagnostic route: codec reject、mount、D&D lifecycle、stale message、CSP、broker等を表示
- diagnostic routeは明示的なdevelopment buildまたは`#diagnostics/*`に限定する
- production navigation、community panel catalog、通常tabへ既定表示しない
- diagnostic componentを通常製品panelとしてexportしない
- 同じprojection/intent codecを使っても、diagnostic fixtureをDocument意味の正本にしない

## 9. 禁止事項とSTOP

次の一つでも発生したら`ORDER: STOP`とし、実装を続けない。

1. source assetがあるのに別の縮約componentを新規作成した
2. visual mismatchをleaf CSSの追加だけで追いかけ、source移管を避けた
3. Inspectorのlegacy依存を理由に`InspectorSurface`等のskeletonを製品版へ昇格した
4. `TimelineCandidate`全体をReact製品Timelineとして持ち込み、native所有面と二重化した
5. product packageが`docs/mocks-ui`、legacy HTML/script、archive routeをruntime importした
6. mockとproductに同じcomponent/CSSの独立copyを残した
7. catalog ID、label、thumbnail token、配列位置のspecial-caseで欠落fieldを捏造した
8. ReactへDocument clone、selection store、Undo/history、stable ID counterを追加した
9. visual threshold、golden、期待値を実装都合で緩和・更新した
10. 正しい画面を出さず、diagnostic routeだけを合格成果にした
11. 公開API、Document、journal、plugin/community契約、永続layout形式の変更が必要になった
12. WebView Host、sandbox、Windows/macOS受入をsource移管のvisual合格で証明済みにした

## 10. 発注書の強制ラベル

React source assetを扱う発注書は、通常項目に加え次を順番どおり持つ。

1. `REACT AUTHORITY`: 対象面、本文書、UI runtime境界、対応spec ID
2. `SOURCE ASSET`: 固定SHA、旧path、export、CSS/model/test closure
3. `PRESERVE`: DOM、class、stable ID、ARIA、interaction、visual stateの維持範囲
4. `REPLACE`: legacy/mock stateのうちprojection / intentへ交換する範囲
5. `STATE OWNER`: Document / User settings / Workspace / Project session / Transient / local presentationの分類
6. `DIAGNOSTIC ROUTE`: 製品routeと契約確認routeの分離
7. `NEGATIVE ORACLE`: 二重copy、legacy import、opaque-ID分岐、二重state、threshold変更の拒否試験
8. `STOP`: 本文書§9と、公開契約・意味の未決に遭遇した場合の停止

一つでも欠落、順序逆転、対象path不一致があればCodex事前審査は承認せず、受注者を起動しない。
発注は一面・一所有境界ずつ行い、Browser、Inspector、KEYS/LAYERS、Easing、Host codec、WebView統合、
D2 commitを一枚の変更許可へ束ねない。

## 11. 検収条件

presentation ownership移管は次を全て満たして初めて合格とする。

1. old path → product path → product export → mock consumerを列挙するprovenance manifestがある
2. product runtime closureに`docs/mocks-ui`、`src/legacy`、raw HTML/script、archive importが無い
3. mock側に同じcomponent/CSSの独立実装が無く、product exportを使う
4. 固定fixtureのlandmark寸法が一致し、既存17-state visual matrixが各landmark 1%以下
5. threshold、oracle commit、viewport、font、goldenを変更していない
6. Storybook、keyboard、focus、selection、tab、search、preset操作が移管前と同じ
7. product routeには正しい画面、diagnostic routeには縮約契約画面が表示される
8. projection decodeがunknown/non-finite/oversized/dangling referenceを拒否する
9. intent以外のReact semantic writeが0で、同じ意味を複数reducer/storeが所有しない
10. offline production bundleがdev server、CDN、HMR、fixture scriptへ依存しない
11. 変更許可外、公開型、serde、Document、journal、plugin contractの差分が0
12. read-only反対側レビューでP0/P1=0、Codex統合審査で本文書の各項を証跡付き確認する

17-state visual matrixは次を閉集合とする。

- `initial`
- Browser: `effects`、`create`、`media`、`grid`、`list`
- Inspector: selected item
- KEYS: `align`、`stagger`、`stretch`
- LAYERS: `align`、`stagger`、`shift`
- Easing: basic、bezier、advanced、overshoot

各stateで`.browser`、`#inspector`、`.candidate-key-tools`、`#easing-panel`のうち表示対象を
固定viewport、DPR 1、dark scheme、`ja-JP`、reduced motion、font ready後に比較する。
非表示landmarkを空画像で合格させず、そのstateの表示契約に従って存在数0または1を明示する。

visual 1%は移管時の回帰検知上限であり、縮約再実装を1%まで近づければ直接移管と同等になる、という
代替条件ではない。source provenance、single owner、closure合格が先である。

## 12. 実行順

発注再開後も次を直列にする。

1. **R0 source inventory**: 固定source、legacy依存、CSS/model/test closureを固定
2. **R1 Browser ownership**: Browserをproduct ownerへ移し、mockをconsumerへ反転。意味変更なし
3. **R2 Easing ownership**: Easing view/modelを同様に移す。Host curve接続は後続
4. **R3 KEYS/LAYERS extraction**: native Timeline本体からReact tool panelだけを抽出・移管
5. **R4 Inspector React化**: legacy DOM/scriptを固定モック内で同形Reactへ抽出後、product ownerへ移す
6. **R5 projection/intent**: 一面ずつmock stateをHost read modelへ交換
7. **R6 diagnostic routes**: 契約確認面をdevelopment専用routeへ分離
8. **H1b WebView Host**: codec、offline bundle、focus/IME/AX、surface topologyへ接続
9. **製品縦切り**: Rectangle D&D → D2 →同一LayerId Timeline/Inspector → key → easing → Undo

R0〜R6のReact asset作業は、WebView platform受入やplugin公開契約を決めない範囲で実施できる。
H1b以降はG0-9、D2/D3、U2h/U3a等の既存停止線を引き続き満たす必要がある。

## 13. 既存文書との関係

- [React / WebView再選定](2026-07-21-m3-react-webview-runtime-reconsideration.md)§2.2の
  「製品候補資産」と§6の「比較oracle」を、本書が**直接所有移管の方法**として具体化する
- 同書§7の製品組込み停止は、WebView Host、公開plugin UI、sandbox、platform受入に維持する。
  R0〜R6のproduct-owned React packageとmock consumer化までを禁止するものではない
- [U0e-2 reference fixture契約](2026-07-21-m3-u0e-2-reference-fixture-contract.md)のbyte一致baselineは
  source provenanceを固定する。製品移管後はmockがproduct exportを読むため、以後の同一性は
  provenance manifestとvisual/interaction oracleで検査する
- [UI参照地図](../ui-reference-map.md)のlegacy領域は、正しいReact sourceが存在しないことを示す移行台帳である。
  legacy出力を製品runtimeへ直接持ち込む許可ではない
- [製品モック一括回収計画](2026-07-21-m3-product-mock-recovery-plan.md)の縦切りは維持するが、
  React各面の入口は本書のproduct-owned componentに限定する
- UI runtime責任境界、D2 single writer、Document/Transient状態所有、native Stage/Timeline、
  community sandbox未決、公開契約停止線は変更しない

## 14. 現在の処分

| 対象 | 状態 | 次の扱い |
|---|---|---|
| 固定React source | 決定 | 発注再開後のR0 authority |
| 縮約`BrowserPanel` / `InspectorPanel` / `KeysLayersPanel` / `EasingPanel` | 棄却 | presentation実装として採用・commit・統合しない |
| visual/CSP/closure/decoder harness | 未検収 | source非依存部分だけ個別レビュー候補 |
| WebView Host / community runtime | 停止線 | G0-9/platform/sandbox審判待ち |
| Rectangle D2 / Vector / Timeline接続 | 停止線 | 既存D2/D3/U2h/U3a契約待ち |
| 発注 | 停止線 | ユーザーの明示的な再開指示待ち |
