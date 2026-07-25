# CU-0A07 / R4 Inspector readiness分割決定

- 日付: 2026-07-25
- 状態: **決定**
- 現在粒: **CU-0A07B / R4B DO**
- 後続: **CU-0A07C / R4C WAIT**

## 1. 確認した事実

固定React source inventoryはInspectorを
`react-source-absent-legacy-parity-oracle`、`independent-react-source-absent`
と判定し、`promotionBoundary`は空である。
`docs/mocks-ui/src/surfaces/InspectorSurface.jsx`は
`reduced-skeleton-not-product-source`であり、製品資産へ昇格しない。

正しい現行oracleはarchived HTMLの空の`#inspector`へlegacy
`inspector(mode)`がDOMを描画し、`bindInspector()`が操作を結ぶ実装である。
状態はeffect-focused installed、installed、discover、blocked、missingの5種。
既存の1% whole-frame visual parityだけでは、InspectorのDOM、computed style、
ARIA、操作を固定した証拠として不足する。

## 2. R4の分割

| ID | 状態 | 一成果 | 依存 |
|---|---|---|---|
| CU-0A07S | `DONE` | 本決定で独立source不在と施工順を固定 | CU-0A06B |
| CU-0A07A | `DONE` | [#353](https://github.com/oshikaidesu/Motolii/pull/353)で未変更legacy sourceに構造・style・ARIA・操作oracleを追加 | CU-0A07S |
| CU-0A07B | `DO` | 固定mock内でInspectorを同形React化し、legacy adapterを一方向投影へ封じる | CU-0A07A |
| CU-0A07C | `WAIT` | R4BのJSX/CSSをbyte同一でproduct ownerへ移し、mockをconsumerへ反転 | CU-0A07B |

R4Aは製品source、archived HTML、visual threshold、viewport、font、route、
landmark、goldenを変更しない。既存two-page parity harnessを再利用し、
legacy側とReact側のInspector subtreeについてDOM構造、computed style、ARIAを
比較する。automation toggle、scrub keyboard、color chip、link targetも
再現可能な操作oracleへ固定する。

R4Bはpresentationとadapterを同じ粒で扱う。`innerHTML`再描画とlistener lifetimeが
結合しているため、presentationだけを先に分けない。既存containment helperは
同じtest file内で一般化し、legacy anchorのexact countを検査する。
新しい汎用helper、双方向store、Document/Host正本は作らない。

R4CはR4Bで確定したsource closureだけをbyte同一で移管する。
mock/product二重copy、legacy runtime import、skeleton代用を0にする。

## 3. 状態所有

| 状態 | R4A〜R4Cのowner |
|---|---|
| mode、intensity、spread、automation、fill、stroke、depth等 | mock fixture / legacy adapter |
| Document、selection、Undo、project session | 追加しない |
| Host read projection / typed intent | CU-0A08Iへ留保 |
| open/hover/focus等の表示状態 | local presentation |

## 4. 非目標と停止線

R4A〜R4CはHost projection/typed intent、Document意味、公開API、plugin契約、
永続形式、Studio Preview、diagnostic route分離を実装しない。
次のいずれかで`ORDER: STOP`とする。

- archived HTML、visual threshold、viewport、font、route、landmark、goldenの変更が必要
- skeletonを製品sourceとして昇格する、またはlegacy runtime importを残す
- exact-count containmentが成立しない、cross-surface handlerを一方向adapterへ封じられない
- bidirectional store、未決fieldの意味、公開/Document/plugin/Host境界の変更が必要
- R4B前にR4Cを行う、またはR4C前にHost projectionへ進む

検収はテスト緑だけでなく、上記負例、source closure、state owner、
二重copy 0を再確認する。
