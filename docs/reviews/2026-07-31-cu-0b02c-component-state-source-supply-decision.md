# CU-0B02C component state source / supply裁定

- 日付: 2026-07-31
- 状態: **決定 / CU-0B02C-S DONE**
- 親: `CU-0B02C` **SPLIT**
- 実装状態: `CU-0B02C-P` **PRODUCT-ASSET / DONE**
- 次の一粒: `CU-203` **PRODUCT-ASSET / DO**

## 1. 決定

`CU-0B02C`が直接所有移管する既存React sourceは、固定commit
`56c318edcddab7cf95d263cc2f7dd2b4e6791134`の次の2ファイルだけである。

| source | fixed/current SHA256 | ownership |
|---|---|---|
| `docs/mocks-ui/src/primitives/index.jsx` | `005b5db5a71f75ab139d26f44169538f74d3711ca2244748e9b4a016088c9f8b` | `Button` / `Icon` / `IconButton` / `TabList` / `Tab` / `PanelHeader` / `Field` |
| `docs/mocks-ui/src/primitives/primitives.css` | `f625758bbfb9db6577618584a79ef9e900510ffc96662c6b1f6191393590959c` | focus / hover / pressed-active / disabled / selectedの表示state |

親を次へ分割する。

| ID | 状態 | 一成果 |
|---|---|---|
| `CU-0B02C-S` | `SPEC / DONE` | source closure、state owner、mock supplierと非color値の処分を固定 |
| `CU-0B02C-P` | `PRODUCT / DONE` | 2ファイルを`ui/motolii-web/src/primitives/`へ直接所有移管し、mockをproduct export consumerへ反転 |
| `CU-0B02C-V` | `SPEC / WAIT` | `CU-203`の第二consumer成立後、component-private carryを共通tokenへ昇格するかcomponent内へ留めるか再審判 |

`CU-0B02C-P`完了でcomponent state ownershipは閉じ、`CU-203`を解禁する。
`CU-0B02C-V`は`U0e-3`親閉鎖前のraw color / spacing最終処分であり、`CU-203`の
common feedback componentを先に発明する粒ではない。

## 2. コード事実とdependency分類

JSXにはhook、store、Document mutation、selection、Undo、永続化が無い。
hover / focus-visibleはUA / Transient、pressed / active / disabled / selectedは
callerから渡すread-only presentation inputであり、primitiveは意味stateを所有しない。
generic error stateはsourceに存在せず、`CU-0B02C-P`で追加しない。

CSSが直接importする`docs/mocks-ui/src/tokens/mock-candidates.css`
（SHA256 `7b7ee84a021f3caa0fc327509e41bb7efe30424d032468260159f971ee35b4af`）
はsource assetではなく交換対象のmock supplierである。dependencyを次の3 classへ固定する。

1. **product authority成立済み**: 21 Dark color role。`CU-0B02T/R`の
   `ui/motolii-tokens/generated/tokens.css`を直接使う。
2. **component-private carry**: focus width、control / tab / panel-header height、
   tight / control / group gap、control inset、control corner、technical font。
   fixed sourceが参照した値を同じcomputed valueでproduct CSS内へ限定移送し、
   DTCG / 公開theme / Document / User settingsへ昇格しない。
3. **component-private raw carry**: panel header markerの`#ffffff30`。
   装飾の意味roleを推測命名せず、固定visual byteとして限定保持する。

class 2/3は新しい恒久token正本ではない。`CU-203`で第二consumerが実在する前に
汎用spacing、font、alpha color roleを作らない。

class 2のexact carryは次だけである。

| private input | value |
|---|---|
| focus width / control corner | `2px` / `2px` |
| control / tab / panel-header height | `25px` / `28px` / `29px` |
| tight / control / group gap / control inset | `3px` / `5px` / `7px` / `9px` |
| technical font | `ui-monospace, "SFMono-Regular", Menlo, Consolas, monospace` |

21 colorのうちprimitiveが参照するroleは、既存`--mock-role-*`のsemantic suffixと
同じ`--motolii-color-*`へ一対一で直接接続し、別alias正本をproduct CSSへ作らない。

## 3. CU-0B02C-Pの直接移管契約

`REACT AUTHORITY`:

対象は共通React primitive 7 export。正本は
[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)、
[UI runtime境界](../ui-runtime-architecture.md)、`U0e-3`である。

`SOURCE ASSET`:

§1の固定commit、2 path、2 hashをexact oracleとする。移管先は
`ui/motolii-web/src/primitives/index.jsx`と`primitives.css`、product exportは
`ui/motolii-web/src/index.js`の既存7名称とする。

`PRESERVE`:

DOM、class、stable ID、ARIA、tabIndex、interaction、全7 export、
focus / hover / pressed-active / disabled / selected selector、computed visual stateを保持する。
error state、loading state、validation意味を追加しない。

`REPLACE`:

product CSSからmock token importを除去し、21 color roleをgenerated product CSSへ接続する。
class 2/3だけをcomponent-private exact carryへ交換する。旧
`docs/mocks-ui/src/primitives/index.jsx`はmock supplierを読むconsumer shimとproduct exportの
re-exportだけにし、JSX/CSS実装copyを残さない。

`STATE OWNER`:

hover / focus-visibleはUA / Transient。pressed / active / disabled / selectedはcallerの
read-only presentation input。Document / User settings / Workspace / Project session /
selection / Undo / journal / plugin契約のownerをReact primitiveへ追加しない。

`DIAGNOSTIC ROUTE`:

reference / catalogはproduct exportのconsumerであって正本ではない。
通常製品routeへ未使用primitiveを無理に挿入してDOMを変えず、`CU-203`が実在する
common feedback consumerを接続する。product export、mock consumer反転、current-route
reference captureの同一性を移管証拠とする。

`NEGATIVE ORACLE`:

mock/product二重copy、productから`docs/mocks-ui` import、mock token supplierのproduct持込、
state selectorの縮約再実装、error state発明、hook/store/localStorage/Document mutation、
未知の恒久token追加、visual threshold / golden変更を拒否する。

`STOP`:

固定hash drift、既存computed visualの不一致、class 2/3を公開theme roleへ昇格する必要、
`#ffffff30`の意味推測、公開API / Document / plugin / 永続形式変更、
通常製品DOM変更が必要になった時点で停止する。

## 4. 完了条件

`CU-0B02C-P`は、product path 2件、package export 7件、mock consumer re-export、
旧実装copy 0、product→docs/mock import 0、5 state selector一致、ARIA / tabIndex一致、
class 2/3 exact value、fixed/current provenance、current-route 30 PNG byte一致、
既存reference interaction、build、source inventory、product ownership guard、
独立read-only review `ACCEPT` P0/P1=0で完了する。

非目標はCU-203 feedback component、error意味、accepted routeのDOM置換、
Light / custom / high contrast、icon、native token、公開component API、
Document / selection / Undo / Host transportである。

## 5. 実装証跡

`CU-0B02C-P`はproduct 2 pathと7 package exportへ直接移管し、旧JSXを
mock supplier import + product re-exportだけへ反転、旧CSSを削除した。product CSSは
`--motolii-color-*`を直接参照し、非color値と`#ffffff30`だけをcomponent内へexact carryした。
新publication `ee3c1a2d44fd-ead41d4d6562`は旧generation
`f4f355510cb2-ead41d4d6562`の30 PNGと全byte一致した。product ownership、
source inventory、token、publication guard、build、Browser 16件、panel layout 5件を通過した。
reference harnessもprivate font overrideを保持し、generation
`u0e2-02b795bf37b7-85c0fc529ab1`の30 PNGを旧generationとbyte一致させた。
