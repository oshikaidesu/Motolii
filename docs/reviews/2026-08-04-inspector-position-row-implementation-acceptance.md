# Inspector Position read-only row implementation acceptance

- 日付: 2026-08-04
- 実装commit: `2c20e88ed3090d30ff9fb2730ade834b81b65194`
- 状態: **DONE / ACCEPTED（`CU-0A08ITIA` のみ） / EXTERNAL_GATE_PENDING**
- 正本: [Inspector Position row direct-promotion contract](2026-08-04-inspector-position-row-direct-promotion-contract.md)

## 1. 受入結果と実diff

commit `2c20e88e` は、既存 `inspectorReadModelDecoder` と product-owned
`InspectorCandidate` の同じsource内にread-only Position行を直接接続した。有限
`Const(Vec2)` は閉じた `{ kind: "const", x, y }` summary、decoderがobject existenceだけを
確認した `keyframes` tagは `{ kind: "animated" }` として投影する。animated caseはkeyの
contents、value、time、ID、interp、countを読まず、評価値も作らない。

通常製品default branchはinstalled branchと同じ既存 `objectRow` grammarを再利用し、Constでは
X/Y、Keyframesでは非数値 `animated` だけを表示する。行はhandler、focusable control、automation、
key count、local semantic state、`postMessage`、Host codec、intent/write routeを持たない。
Inspector post-promotion provenanceは既存tailから `CU-0A08ITIA` を一件appendし、fixed baselineや
他entryを置換していない。

## 2. validation

- `node --test docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs`: PASS（40 passed）
- `node --test ui/motolii-web/guard-tests/browser-ownership.test.mjs`: PASS（15 passed）
- `npm --prefix ui/motolii-web run check:host`: PASS
- `./scripts/check-docs.sh`: PASS
- `git diff --check`: PASS

`PRIMARY_ORACLE` は、有限Const X/Y、tag-only animated、unsupported/malformed/non-finite rejection、
同一source row grammar、default branchのinertness、append-only provenanceをfocused decoder/AST/product
guardで照合する。実装commitは`refs/heads/main`へ到達済みである。

## 3. 残る境界

本受入はpresentation/read-model boundary A `CU-0A08ITIA` だけを閉じる。typed one-shot Inspector
intent、private Host/queue consumer、`ProductApp::editor_playhead.current` の入力、既存
`prepare_add_position_key(primary, current_playhead)` への接続は boundary B `CU-0A08ITIB` の
`WAIT_TARGET` に残す。Aからwrite routeを推測せず、U4b-0、playhead、Easing、Document/schema/journal、
public APIを変更しない。

human visual review、native/WebView product-window、focus/accessibilityの確認はM3 final checklistの
`EXTERNAL_GATE_PENDING` のままであり、repository laneのgreenで代替しない。
