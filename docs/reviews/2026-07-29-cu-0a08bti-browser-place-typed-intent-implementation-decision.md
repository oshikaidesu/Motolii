# CU-0A08BTI Browser Place typed intent実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 親: `CU-0A08BT` **SPLIT**
- 次の一粒: `CU-0B03H` **PRODUCT-ASSET / SPEC / DO**

## 1. 成果

ユーザーが再確認した背骨
`Browser → typed intent → Host → Place → Stage / Timeline / Inspector → Undo`
へ実装順を戻した。

product-owned `DiscoveryBrowserCandidate`へprivate `BrowserPlaceIntentContext`を置き、
Rectangleのdrag開始時だけ、decode済み`scope_ref` / `item_id`から次のimmutable intentを
1件生成して`onPlaceIntent`へ渡す。

```text
{ kind: "browser.place", source: { scope_ref, item_id } }
```

これはWebView wireでもPlace commitでもない。Host callbackの入力境界であり、Place意味、
terminal分類、admission、D2、selection、UndoをReactへ所有させない。

## 2. 負例

- bare `itemId`、label、thumbnail、DOM IDからscoped identityを推測しない
- Rectangle以外からVS-1 Place intentを発行しない
- drag開始1回から複数intentを発行しない
- ReactへDocument、selection、history、terminal、commit stateを追加しない
- fixture / diagnostic routeをHost callerとして数えない

既存DataTransferのlegacy bare payloadは本粒でwire契約へ昇格せず、Host接続時に
typed callbackを唯一のsemantic入力として扱う。

## 3. 証拠

- Browser ownership / identity / typed-intent / provenance guard: 8 pass
- product Browser sourceのappend-only provenance chainへ`CU-0A08BTI`を追加
- DOM、class、stable ID、ARIA、CSS、visual threshold、golden変更0

## 4. 次

`CU-107PV`は既決どおり実製品Host再投影`CU-0B05`待ちであり、test/dummy callerで
先行させない。次はH1bの未決を一つの`CU-0B03H`へ限定し、現行source closureに対する
offline bundle、closed Host callback/mount、origin/lifecycleの最小契約だけを閉じる。
token、provenance診断、他surface R5/R6へ戻らない。
