# CU-0A08ITP-P Inspector post-promotion authority 改訂

- 日付: 2026-07-29
- 状態: **決定**
- 対象grain: **CU-0A08ITP-P**
- 次grain: **CU-0A08ITP**

## 1. 一問

R4Cでproduct ownerへ移管したInspector componentを、既決read modelの
read-only projectionへ接続する際のsource byteとprovenanceを、Browserの
`postPromotionChanges`履歴へ混ぜずに記録できるか。

## 2. 現行authorityとの未統一

- [G0-6H-V1ETB-H](2026-07-28-g0-6h-v1etb-h-browser-post-promotion-authority-reclosure-decision.md)
  のK-1は、Browser以外のmigrationを固定byte一致としている。
- [G0-6H-V1ETA](2026-07-28-g0-6h-v1eta-empty-projection-staging-decision.md)
  のprovenance v1追加fieldは、Browser用`postPromotionChanges`一配列だけを許す。
- Inspector componentを変更し期待hashだけを書き換えること、または未承認の
  top-level fieldを追加することは、上記authorityのままでは禁止される。

## 3. 改訂

1. K-1の固定byte一致はBrowser CSS/pattern、Easing、KEYS/LAYERS、Inspector CSSで
   維持する。Inspector componentは、次項の専用chainが有効な場合だけ
   fixed byteからのappend-only変更を許す。
2. `schemaVersion: 1`にoptional top-level
   `inspectorPostPromotionChanges`をちょうど一つ追加できる。Browserの
   `postPromotionChanges`とは別fieldであり、相互のentryを混在させない。
3. Inspector entryのkey閉集合はBrowserと同じ
   `task / file / reason / fixedSourceSha256 / currentSha256`の5つ。
   index 0はtask `CU-0A08ITP`、file
   `ui/motolii-web/src/candidates/InspectorCandidate.jsx`、reason
   `VS-1 Inspector target read-only projection component input`、
   fixed hash `1e0bdd3eebd665e517600af4db090f74d50951aef12fdd476e97a828de91a3e4`
   に固定する。
4. entryは同file、task一意、task/reason非空、前entry
   `currentSha256`から次entry `fixedSourceSha256`への連鎖、tailと現行source
   byte一致を必須とする。
5. K-3を維持する。期待hash更新だけでは完了せず、専用validatorの正例と、
   空配列、key欠落/余剰、index 0 authority相違、file相違、空/重複task、
   空reason、chain break、tail不一致の負例を同じ実装粒へ必須化する。
6. Browserの`postPromotionChanges`実データ、PC-1〜PC-9、R-1〜R-8、
   K-2、既存Browser validatorは1 byteも変更しない。

## 4. 非目標

React source、hash期待値、provenance実データ、guard code、decoder、fixture、
DOM/CSS/ARIA/interaction、公開API、Document、selection、Undo、plugin契約、
runtime producer、Host transport、typed intentを本authority粒では変更しない。

## 5. STOP

Browser chainの再解釈、schemaVersion増加、汎用multi-file chain、第三のcomponent
chain、公開契約変更、Inspector以外のsource byte緩和が必要なら停止する。

## 6. handoff

`CU-0A08ITP-P`は**DONE**。次の唯一のPRODUCT-ASSET `DO`は
`CU-0A08ITP`。同粒がReact直接接続、Inspector専用chain実データ、
正負guard、inventory/hash mirrorを一つのclosed diffで実装する。
