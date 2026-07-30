# G0-6H-V1G-O-H 現行route command authority hash 補正決定

- 日付: 2026-07-29
- 状態: **決定**
- G0-6H-V1G-O-H: **DONE**

## 目的

`G0-6H-V1G-O`の実装では、決定済みの`package.json` script追加と既存read-only authority guardの固定hashが同一commit内で必ず衝突する。本粒はその解消手順、許可範囲、停止線だけをdocs-onlyで確定する。実装、script追加、hash literalの書き換えは本粒では行わない。

## 現行コード事実

1. `docs/mocks-ui/package.json`の`scripts`に`generate-current-route`と`check-current-route`は存在しない。
2. `docs/mocks-ui/scripts/current-route-generation.mjs`は`CURRENT_ROUTE_COMMANDS`として上記2 commandを固定している。
3. [G0-6H-V1G-P mechanics決定](2026-07-29-g0-6h-v1g-p-current-route-generation-mechanics-decision.md) M-2は、上記2 command名の再利用と旧commandの多重化禁止を決めている。
4. `docs/mocks-ui/package.json`を`AUTHORITY_SHA256`で固定するliteralは、次の3 fileに各1箇所だけ存在する。
   - `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`
   - `docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs`
   - `docs/mocks-ui/guard-tests/inspector-read-model-inventory.test.mjs`
5. 各guardは`assert.equal(sha256File(rel), expected, rel)`で完全一致を検査する。
6. 上記3 fileの`AUTHORITY_SHA256`合計26 entryのうち、`docs/mocks-ui/package.json`以外の23 entryは本補正の対象外である。
7. `test:reference-guard`は3 guardを同時実行するため、hash更新の中間状態は必ず失敗する。

## 裁定

### OH-1 同一commit原子性

`G0-6H-V1G-O`は、`docs/mocks-ui/package.json`への`generate-current-route`と`check-current-route`の2 script追加と、次の3 fileの`AUTHORITY_SHA256["docs/mocks-ui/package.json"]` literal更新を、同一の1 commitで行う。分割commit、先行commit、後追い修正commitを作らない。

- `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`
- `docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs`
- `docs/mocks-ui/guard-tests/inspector-read-model-inventory.test.mjs`

### OH-2 更新の許可範囲

各guard fileで許可される変更は、`AUTHORITY_SHA256` objectの`"docs/mocks-ui/package.json"`に対応する64-hex文字列1個の置換だけである。keyやentryの追加・削除、object構造、assertion、test名、import、その他のbyteは変更しない。

### OH-3 更新の事前条件

新しい64-hex値を書く前に次を機械確認する。1つでも不成立なら`ORDER: STOP`として更新しない。

1. `package.json`の実差分が`scripts` objectへの決定済み2行追加だけであり、他key、依存、version、整形、行末、末尾改行が不変である。
2. 新しい値は変更後の`package.json`からその場で再計算し、想定値へfileを合わせない。
3. 3 fileすべてが同一の再計算値を持つ。
4. 残り23 entryがbyte不変で、各authority sourceの再hashにも一致する。
5. 2 scriptの値が`CURRENT_ROUTE_COMMANDS`と文字列一致する。

### OH-4 禁止形

- 旧hashと新hashの両方を許容する配列、`Set`、fallback、`previous`欄。
- wildcard、接頭辞、正規表現、長さ緩和によるhash照合。
- assertionの緩和、skip、条件分岐、環境変数gate、test除外。
- forbidden key、decoder期待値、fixture、stale pattern、MIRRORS、test名、test件数のsemantic変更。
- 空文字列、`echo`、`true`、`exit 0`等のscript stub。
- 旧commandの改名、分岐、別名追加。
- 3 file以外へのhash literal追加、または第二のpackage authority owner新設。
- `package.json`の整形、key並べ替え、lock更新。

## 停止線

- allowlist外のfile変更が必要になった。
- package authority entryが上記3 file以外に現れた。
- 変更後package SHA-256具体値をdocsへ焼く必要が生じた。
- 決定済み2 command以外のscript追加、または既存command改名が必要になった。
- `G0-6H-V1G-O`本体の未決意味を本決定で確定する必要が生じた。

## 根拠authority

- [G0-6H-V1G-P mechanics決定](2026-07-29-g0-6h-v1g-p-current-route-generation-mechanics-decision.md)
- `docs/mocks-ui/scripts/current-route-generation.mjs`
- `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`
- `docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs`
- `docs/mocks-ui/guard-tests/inspector-read-model-inventory.test.mjs`
- [G0-6H-V1G-C-P capture環境 authority 補正決定](2026-07-29-g0-6h-v1g-c-p-current-route-capture-environment-authority-correction-decision.md)

## 非目標

- `package.json`、3 guard、CLI実体、生成物、manifest、PNG、公開契約の変更。
- 変更後`package.json` SHA-256 literalのdocsへの記載。
- `G0-6H-V1G-O`のimmutable publication設計の追加・変更。
- `AGENTS.md`、`docs/specs/`、隣接ticketへの波及。

## 後続への効果

| 後続 | 状態 | 条件 |
| --- | --- | --- |
| G0-6H-V1G-O | `観察` | OH-1〜OH-4を発注条件として起動する |
| G0-6H-V1G | `未統一` | `G0-6H-V1G-O`完了後に締結する |

## Reactラベル

### REACT AUTHORITY

対象面は`#plugin-browser-candidate`上のproduct-owned Browser / Inspectorと、それらを固定する上記3 guard。移管契約は[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)。対応spec IDはG0-6H-V0 / V1S / V1P / V1R / V1G-P / V1G-I / V1G-C-P / V1G-C。

### SOURCE ASSET

固定source commitは`ui/motolii-web/source-provenance.json#fixedSourceCommit`の`56c318edcddab7cf95d263cc2f7dd2b4e6791134`。`package.json`と上記3 guardは本粒で1 byteも変更しない。

### PRESERVE

既存DOM、stable ID、class、ARIA、interaction、visual state、`#plugin-browser-candidate`、`.app[data-parity-ready]`、`#root[data-current-route-capture-ready]`、全authority entryと照合assertion、ownership guard、post-promotion provenanceを維持する。

### REPLACE

なし。本粒はdocs-onlyであり、mock / legacy stateからprojection / typed intentへの交換を行わない。

### STATE OWNER

本粒の成果は`docs/reviews`、`docs/decision-index.md`、`docs/implementation-ledger.md`上の決定記録だけである。

### DIAGNOSTIC ROUTE

製品画面は`#plugin-browser-candidate`のまま。開発確認は既存`current-route-capture` Vite modeと旧`#reference/*`生成に限り、新route、hash、query、mode、served entryを追加しない。

### NEGATIVE ORACLE

各guardで変更してよいのはpackage authority hash literal1個だけであり、`package.json`の差分は決定済み2 scriptの追加だけである。旧新hash併記、wildcard、期待値緩和、semantic test変更、stub、第二ownerを棄却する。

### STOP

未決product意味、公開契約変更、source asset不在、state owner違反、allowlist外変更、変更後SHAのdocsへの焼き込み、3 file以外へのhash波及が必要になった時点で停止する。

## 関連

- [AGENTS.md](../../AGENTS.md)
- [G0-6H-V1G-P mechanics決定](2026-07-29-g0-6h-v1g-p-current-route-generation-mechanics-decision.md)
- [G0-6H-V1G-C-P capture環境 authority 補正決定](2026-07-29-g0-6h-v1g-c-p-current-route-capture-environment-authority-correction-decision.md)
- [implementation-ledger](../implementation-ledger.md)
