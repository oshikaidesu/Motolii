# CU-0A09S R6 surface closure boundary 決定

- 日付: 2026-07-29
- 状態: **決定 / DONE**
- 親: `CU-0A09`（R6、`SPLIT`）

## 1. 問い

`CU-0A09B`後のR6残surfaceを列挙し、R6とH1bの間にあった
「通常製品route」の循環を解消する。code、component、diagnostic entryは変更しない。

## 2. React直接移管境界

1. `REACT AUTHORITY`:
   [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)
   R5/R6/H1bと[UI runtime責任境界](../ui-runtime-architecture.md)を正とする。
2. `SOURCE ASSET`:
   product exportのBrowser、Inspector、Easing trigger、KEYS/LAYERSと、
   各mock consumerを現行code事実として分類する。
3. `PRESERVE`:
   product component、CSS、export、DOM、class、stable ID、ARIA、既存oracleを変更しない。
4. `REPLACE`:
   R6の未列挙集合と通常製品route帰属の矛盾だけを台帳上で置換する。
5. `STATE OWNER`:
   R6は状態を新設しない。Document/selection/UndoはHost/D2、local presentationだけReact。
6. `DIAGNOSTIC ROUTE`:
   development observerはR5で交換済みのproduct component契約だけを観測する。
   diagnosticを通常製品routeまたはrelease sourceへ昇格しない。
7. `NEGATIVE ORACLE`:
   legacy stateのno-op/default化、fixture-only成果、skeleton製品化、R6とH1bの相互依存、
   WebView/codec/offline bundleの先行を拒否する。
8. `STOP`:
   Host state/intent、Document意味、公開API、plugin契約、永続形式、
   component/entry/test変更が必要なら本docs粒を停止し、各R5子粒またはH1bへ戻す。

## 3. code事実

| surface | 現状 | R6状態 |
|---|---|---|
| Browser | `CU-0A09B`で同じproduct exportのlegacy-free mount成立 | `DONE` |
| Inspector | parser非依存だが唯一のconsumerはlegacy script由来mutable `state`とcallback。product側もautomation/parameter値を直接mutate | `WAIT`。`CU-0A08ITI`または後続R5裁定なしにdefault/no-opを発明しない |
| KEYS/LAYERS | product componentはprop駆動。現行consumerはmock Timelineのglobal DOM bridgeを含む | `WAIT`。`CU-0A08K`のHost projection/typed intent交換後にR6証跡を閉じる |
| Easing trigger | product componentはprop駆動。現行consumerはlegacy Host fixture経由 | `WAIT`。`CU-0A08E`のHost projection/typed intent交換後にR6証跡を閉じる |

R2Bの所有範囲はEasing **trigger**であり、本粒はmock-owned Easing Panel全体を
新しいproduct assetとして追加しない。

## 4. 帰属と順序

- R6はdevelopment observerとproduction navigationの**分離**だけを所有する。
- 最初のnon-mock runtime caller、offline bundle、Host codec/mountはH1b
  `CU-0B03`が所有する。
- 通常製品windowで全surfaceを表示する成果はW0b `CU-0B04R`以降が所有する。
- よってR6の完了条件へ通常製品routeを含めない。逆にH1bをR6の前提にしない。
- `CU-0A09`は残る3 surfaceが各R5交換待ちのため`SPLIT`を維持する。
  eligibleなR6実装粒は現時点で0件。

## 5. 非目標と次

code差分、HTML entry、Playwright、Host wire、WebView、offline bundle、D2、U4a/U4b、
公開plugin UI、current-route再publicationは非目標。

次はR6のdiagnostic-only leafを増やさず、R5の既存待ち
`CU-0A08E` / `CU-0A08K` / `CU-0A08ITI`から、依存が閉じる一粒を別途選定する。
`CU-0B03`の「別途確定する製品前提」列挙はR6と束ねず、次のH1b authority粒で固定する。
