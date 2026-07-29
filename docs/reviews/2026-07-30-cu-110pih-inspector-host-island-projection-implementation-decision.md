# CU-110PIH Inspector Host island projection 実装決定

- 日付: 2026-07-30
- 状態: **決定 / DONE**
- commit: 本文と同一commit

## 1. 結論

通常製品native Hostの予約済みright rectへ、product-owned
`InspectorCandidate`を載せる第二のoffline child WebViewを接続した。

起動時はprimary不在を`null`として表示を発明せず、Place成功時はD2が採用した
同じ`current_document`と`primary`を既存`decodeInspectorReadModel`へ渡す。
別Document snapshot、selection store、public wire、Inspector intentは作らない。

## 2. 実装

- `NativeHostLayout`の既存top share `1:3:1`の右1をInspector rectとして公開。
- `InspectorHostRuntime`
  - `motolii-inspector:` custom protocol
  - CSP付きoffline `inspector.html`
  - compile-time埋込bundleだけを配信
  - private subscribe / publish bridge
  - stale layout epochを再適用しない
- Host read-model wrapper
  - `fixtureRevision: 1`
  - adopted current Document
  - `nodes: []`
  - primary `layer_id`
- React Host entryは既存decoderと既存`InspectorCandidate` safe branchだけを使用。
- Browser / Inspector WebViewをnative Surfaceより先、parent Windowより前にdropする。

Host screen CSSは独立component copyではなく、移管済みclass/DOMへnative islandの
theme tokenと既存panel / identity chromeを与えるprivate runtime adapterである。

## 3. 証跡

- generated offline Host manifest check: pass
- product web guard: 10 passed
- `motolii-ui` lib: 104 passed
- CU-110 / PS / PT / PIH chain tests: 5 passed
- 実Mac:
  - `/private/tmp/MotoliiNativeProduct.app`
  - `/private/tmp/motolii-timeline-110pt-project.json`
  - Rectangle drop直後、再起動なしでStage / Timeline更新
  - 右Inspectorへ`Inspector / Rectangle / Clip`
  - journal 6698 bytes

## 4. 非目標と停止線

- selection input、focus owner変更、Inspector編集、effect panelなし。
- public API、Document、serde、journal形式、plugin契約、Undo/history変更なし。
- React Timeline、legacy/mock runtime import、別Inspector leaf、二重stateなし。
- unresolved plugin NodeDescをDocumentから推測しない。VS-1のprimary Rectangleは
  effect definitionを持たないため`nodes: []`で閉じる。

`CU-110PIH`は`DONE`。次の唯一のPRODUCT-ASSET `DO`は`CU-106P`。
