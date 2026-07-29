# CU-0A08ITP Inspector read projection / JSX connection 実装決定

- 日付: 2026-07-29
- 状態: **決定**
- 対象grain: **CU-0A08ITP**
- authority前提: **CU-0A08ITP-P DONE**

## 1. 目的

`CU-0A08IP`がdecode済みのInspector targetから、既決の`layer_name`、
`item_kind`、group時の`child_count`だけをproduct-owned
`InspectorCandidate`の既存installed identity JSXへ非推測透過する。

## 2. React直接移管契約

1. **REACT AUTHORITY**: 対象面はVS-1 installed Inspector identity。正本は
   [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)、
   [Inspector post-promotion authority改訂](2026-07-29-cu-0a08itp-p-inspector-post-promotion-authority-amendment.md)、
   [Inspector read-model分割決定](2026-07-26-cu-0a08i-inspector-read-model-split-decision.md)、
   [CU-0A08RS裁定](2026-07-29-cu-0a08rs-browser-inspector-read-projection-u4a2-dependency-decision.md)。
   UI runtime境界はproduct component inputで、公開exportは増やさない。
2. **SOURCE ASSET**: 固定source commit
   `56c318edcddab7cf95d263cc2f7dd2b4e6791134`からR4Cで直接移管済みの
   `ui/motolii-web/src/candidates/InspectorCandidate.jsx`、export
   `InspectorCandidate`、既存CSS/model/test closureを用いる。
3. **PRESERVE**: 既存DOM、class、stable ID、ARIA、interaction、5 branch、
   CSS、未入力時のmock表示、effect/transformのvisual stateを維持する。
4. **REPLACE**: installed branchのobject identity名とkind/child表示だけを、
   mock literalからoptional `inspectorReadModel` component inputの
   `target`投影へ交換する。component内でdecode・検索・fallback推測しない。
5. **STATE OWNER**: 入力はHostが将来供給するread-only projection。
   ReactはDocument、selection、Undo、Workspace、Project sessionの正本を持たず、
   本粒のlocal値はrender中のpresentationだけである。
6. **DIAGNOSTIC ROUTE**: 製品componentそのものを変更する。diagnostic route、
   mock route、development専用確認画面を成果にせず、通常製品caller成立も
   本粒単独では主張しない。
7. **NEGATIVE ORACLE**: 二重copy、legacy import、opaque ID/label推測、
   二重state、decoder再実行、effect/transform/他branch接続、DOM/CSS/threshold変更を
   AST正負検査と既存guardで拒否する。
8. **STOP**: target以外の意味、公開契約、runtime producer、Host transport、
   typed intent、Undo、他branchの配線、source不在またはowner境界変更が必要なら停止する。

## 3. 実装

- `InspectorCandidate`へoptional private input `inspectorReadModel`を追加した。
- `mode === "installed"`かつ非effect-focusedの既存identityだけで、
  `target.layer_name`を名前へ、`target.item_kind`とgroup時の
  `target.child_count`を既存kind/child位置へ接続した。
- 入力未指定時は`Pulse rings` / `Group · 1 child`を維持し、現行mock consumerを
  変更していない。入力objectが不完全な場合の黙ったfield fallbackは追加しない。
- Inspector専用`inspectorPostPromotionChanges` append-only hash chainを追加し、
  Browserの`postPromotionChanges`履歴とvalidatorは変更していない。
- `CU-0A08ITP-P`が必須化した空配列、key閉集合、index 0 authority、
  file/task/reason、重複、chain break、tail不一致の正負matrixを追加した。
- read-model inventoryのinstalled identity 2行をdynamic projection bindingとして
  再締結し、他の分類・capability・拒否規則は変更していない。

## 4. 非目標

runtime producer、Host wire、selection、typed intent、scrub/automation、
effect definition表示、discover/blocked/missing branch、`S`分類、Document、
serde、永続形式、plugin契約、Undo、capture/golden/publicationは変更しない。
identity icon `G`はinventory分類`U`の固定presentationであり、本粒では
`item_kind`から推測変更しない。icon mappingが必要なら別の既決表示粒で扱う。

## 5. 検証

```text
node --test ui/motolii-web/guard-tests/browser-ownership.test.mjs
1..7
# tests 7
# pass 7
# fail 0

node --test docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs
1..39
# tests 39
# pass 39
# fail 0

node --test docs/mocks-ui/guard-tests/inspector-read-model-inventory.test.mjs
1..24
# tests 24
# pass 24
# fail 0

node --test docs/mocks-ui/guard-tests/source-asset-inventory.test.mjs
1..23
# tests 23
# pass 23
# fail 0

node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs
1..118
# tests 118
# pass 118
# fail 0

node --test docs/mocks-ui/guard-tests/current-route-provenance.test.mjs
1..17
# tests 17
# pass 17
# fail 0

cd docs/mocks-ui && npx playwright test tests/inspector-parity.spec.js
8 passed

./scripts/check-docs.sh
OK: docs整合チェック全項目通過

cargo test --workspace
test result: ok

npm run test:reference-guard
# pass 516
# fail 2
```

旧immutable current-route generation
`44e538c97807-ead41d4d6562`が保持するpre-BTP/pre-ITP manifest hashにより、
現行provenance manifest hashとの一致検査とCLI checkが`CR2-SCHEMA`で拒否する。
このgenerationの再publicationは本粒へ混ぜず、既存のG0-6H evidence laneに残す。

## 6. handoff

親`CU-0A08IT`を`SPLIT`し、read-only projectionの`CU-0A08ITP`は**DONE**。
typed intent / Host接続は`CU-0A08ITI`として既存`U4a-2`依存の`WAIT`を維持する。
最初の非mock runtime callerと通常製品route到達はR6後のH1bが所有する。
次のPRODUCT-ASSET `DO`は未選定（0件）。
