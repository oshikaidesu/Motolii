# CU-110PI Inspector通常製品接続 分割決定

- 日付: 2026-07-29
- 状態: **決定 / CU-110PI SPLIT**
- 現在粒: **CU-110PIR DO**

## 1. 結論

`CU-110PI`を次の二粒へ分割する。

1. `CU-110PIR`: product-owned `InspectorCandidate`へ、decode済み
   `inspectorReadModel.target`だけで成立するread-only target branchを追加する。
   既存installed / focused / discover / blocked / missing branchは変更しない。
2. `CU-110PIH`: 右上に予約済みのInspector rectへ第二のopaque child WebViewを載せ、
   adopted `current_document`と`primary`を既存`decodeInspectorReadModel`へ渡し、
   `CU-110PIR`を通常製品routeで表示する。

順序は`CU-110PIR → CU-110PIH → CU-106P → CU-111 → CU-108`。
React presentationの安全な受け口とHost transport / WebView統合を同一粒へ束ねない。

## 2. CU-110PIR React契約

REACT AUTHORITY:

対象面はVS-1 Inspector target read-only projection。
[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)、
[CU-0A08ITP実装決定](2026-07-29-cu-0a08itp-inspector-read-projection-jsx-connection-implementation-decision.md)、
M3 `CU-110PI`をauthorityとする。UI runtime境界はproduct component inputで、公開exportを増やさない。

SOURCE ASSET:

固定source commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`からR4Cで直接移管済みの
`ui/motolii-web/src/candidates/InspectorCandidate.jsx`、export `InspectorCandidate`、
`inspector-candidate.css`、Inspector parity / ownership guard、既存
`inspectorPostPromotionChanges` chainを用いる。

PRESERVE:

既存5 branchのDOM、class、stable ID、ARIA、interaction、visual state、CSS、
未入力時mock consumerをbyte-equivalent behaviorで維持する。既存installed identityの
`inspectorReadModel`接続を変更しない。

REPLACE:

`mode`、mock `state`、editing callbackを与えず、decode済み`inspectorReadModel`だけを
与えた場合に限り、既存panel headとtarget identity DOMを同じcomponentから描く。
表示fieldは`layer_name`、`item_kind`、group時`child_count`だけ。

STATE OWNER:

入力正本はHostのadopted Document / primaryから導出するread-only projection。
ReactはDocument、selection、Undo、revision、stable ID、Workspace / Project sessionを
所有しない。branch固有stateは持たず、render中presentationだけとする。

DIAGNOSTIC ROUTE:

product-owned component本体へ受け口を作るが、通常製品WebView接続は後続`CU-110PIH`が所有する。
Story / guardは契約確認であり、成果をdiagnostic routeだけにしない。

NEGATIVE ORACLE:

別Inspector copy、legacy / mock runtime import、opaque ID / label推測、Document clone、
mock state / S値、editing callback、decoder再実行、DOM/CSS threshold変更、既存5 branch変更を拒否する。

STOP:

target 3 field以外、公開API、Document / serde / journal / plugin契約、typed intent、
selection / Undo、source不在、既存branchの意味変更が必要なら停止する。

## 3. CU-110PIH Host境界

- `NativeHostLayout`の既存`BUILT_IN_TOP_SHARES[2]`をInspector logical rectへ写像する。
- Browser islandと別のprivate `InspectorHostRuntime`を持つ。Document / primary正本は増やさない。
- offline product bundleへInspector entryを追加し、既存product-owned
  `InspectorCandidate`と`decodeInspectorReadModel`を直接importする。
- Rustは`Document` JSON、空または既存NodeDesc closure、primary `LayerId`だけを
  initialization / update projectionとして渡す。React側decoderの出力だけをcomponentへ渡す。
- InspectorからHostへのintent / IPCは0。focus / pointer capture / selection inputを追加しない。
- lifecycle replacementはBrowserと別islandの同一snapshot再投影に限定する。

## 4. 非目標

- Inspector transform / appearance / effect parameter値、`S`分類、editing intent。
- selection producer、Timeline hit、focus、Undo/Redo。
- public transport、Document、serde、journal、plugin契約、永続layout。
- React component copy、legacy script、mock state、fixture default。
- Browser / Stage / Timeline責任の再設計。

## 5. STOP

1. safe read-only branchにmock stateまたは既存installed branch全体が必要になる。
2. Host bridgeに公開wire、Document field、selection / Undoの第二正本が必要になる。
3. Inspector islandがBrowser pointer capture / inboxを共有しないと成立しない。
4. generated bundleがdev server、CDN、fixture、legacyへ依存する。
5. visual threshold、golden、既存期待値変更が必要になる。

`CU-110PI`は`SPLIT`。次の唯一のPRODUCT-ASSET `DO`は`CU-110PIR`。
