# CU-0B02C-V component-private carry最終処分

- 日付: 2026-07-31
- 状態: **決定 / DONE**
- 親: `CU-0B02C` **SPLIT**
- 依存: `CU-0B02C-P`、`CU-203P` **DONE**
- 次の一粒: `CU-204` **PRODUCT / DO**

## 1. 結論

`CU-0B02C-P`が限定移送した非color値と`#ffffff30`は、すべて
**component-privateのまま維持**する。DTCG、generated token、package export、
共通CSS custom property、Document、User settingsへ昇格する値は0件である。

`CU-203P`で第二consumerとなる`Feedback`が成立したが、同じ数値の出現だけでは
同じ意味ownerを証明しない。primitiveのcontrol geometryとfeedbackのmessage geometryを
一つのspacing / radius / focus metricへ結合すると、片方のvisual調整が他方を暗黙変更する
新しい恒久契約になる。現在のauthorityはその結合を要求していない。

この裁定は既存raw値を正当化する一般的な例外ではない。対象は§3の固定2 CSSに現存する
値だけであり、新しいcomponentが同値を使う根拠にはしない。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [CU-0B02C source / supply裁定](2026-07-31-cu-0b02c-component-state-source-supply-decision.md)、[CU-0B02T token authority](2026-07-29-cu-0b02t-product-token-authority-implementation-decision.md)、[CU-203 feedback分割](2026-07-31-cu-203-feedback-source-ownership-split-decision.md) |
| `INTERNAL TARGET` | `ui/motolii-web/src/primitives/primitives.css`のclass 2/3 carry、`ui/motolii-web/src/feedback/feedback.css`のcomponent-local geometry、既存21 color role |
| `OWNER` | primitive geometryはprimitive CSS、feedback geometryはfeedback CSS。21 color roleだけが`ui/motolii-tokens` owner |
| `WRITE ROUTE` | read-only presentation CSS。Document / selection / Undo / journal / Host transportへのwrite routeなし |
| `GAP` | 第二consumer成立後に、private値を共通tokenへ昇格すべき意味共通性があるか未裁定だった |
| `RESOLUTION ROUTE` | 2 CSSの固定byte、値、selector、使用目的を比較し、同値と同意味を分離した |
| `DISPOSITION` | `PASS`: 既存ownerのまま閉鎖。code / token / visual変更0 |

## 3. 固定証拠

| source | SHA-256 | 役割 |
|---|---|---|
| `ui/motolii-web/src/primitives/primitives.css` | `7ecfe0195f922506429caa0e141fe6104bd845e02265e2bd24f4a67232dafc2b` | control、tab、panel header、fieldのprivate geometry |
| `ui/motolii-web/src/feedback/feedback.css` | `7e22e2a183796732c4f77c4bb018eb2342ecb812181e46f348e1aa3aa827ef50` | feedback shell、placement、tone markerのprivate geometry |
| `ui/motolii-tokens/generated/manifest.json` | `f468684379962b00da4fee753abb123d73d61da41d726526ea4c4aa7ecc33304` | color型21件だけの公開product token inventory |

generated manifestにspacing、radius、font、motion、opacity tokenは無い。
`Feedback`もcolorについては既存`--motolii-color-*`を直接再利用し、raw colorを増やしていない。

## 4. 値ごとの最終処分

| primitive carry | Feedbackでの事実 | 最終処分 |
|---|---|---|
| focus width / offset `2px / 2px` | focus-visibleも`2px / 2px` | **別々にprivate維持**。同じaccessibility表現だが、React全componentとnative surfaceを束ねるmetric authorityは無い |
| control corner `2px` | shell radius `2px`、badge radius `9px` | **別々にprivate維持**。control cornerとmessage placement形状は同じ意味でない |
| control height `25px` | 固定control heightなし | primitive private維持 |
| tab / panel-header height `28px / 29px` | 対応値なし | primitive private維持 |
| tight / control / group gap `3px / 5px / 7px` | context gap `3px`、padding block `5px`、body gap / inline padding `7px` | **別々にprivate維持**。同値でもgap、padding、content rhythmのownerが異なる |
| control inset `9px` | marker `9px`、badge radius `9px` | **別々にprivate維持**。inset、marker size、radiusを結合しない |
| technical font stack | `font: inherit`だけ | primitive private維持。`--motolii-primitive-font-technical`はreference harness用の限定override seamで、公開theme tokenではない |
| panel marker `#ffffff30` | raw colorなし | primitive private raw carryとして維持。feedbackやcolor roleへ一般化しない |

## 5. 負例と停止線

- 数値が2箇所に現れたことだけで`space-1`、`radius-small`、`focus-width`等を命名しない
- `--motolii-feedback-*`をpackage APIまたはglobal theme tokenと扱わない
- primitive private値をfeedbackへ参照させず、feedback private値をprimitiveへ参照させない
- `#ffffff30`をoverlay / highlight / white-alpha等の意味roleへ推測昇格しない
- harness overrideをUser settings、Document、plugin契約へ公開しない
- token数、生成物、CSS byte、visual threshold、goldenをこのSPEC粒で変更しない
- native consumer、Light / custom / high contrast、icon systemの判断を本粒へ束ねない

将来、3つ以上の独立product componentとnative consumerが同じ意味名・同じ変更理由・
同じaccessibility oracleを共有する証拠が揃った場合だけ、新しいSPEC粒で再審判する。
その場合も既存private値を自動的に公開tokenへ移さない。

## 6. 完了条件

- fixed 2 CSSと21 color inventoryのhashを再照合
- class 2/3の全値を第二consumerと比較し、各値を個別処分
- 公開token、CSS、JSX、package export、Document、plugin、永続形式の変更0
- `CU-0B02C-V`を`DONE`、次の唯一のPRODUCT `DO`を`CU-204`として全mirrorを一致
- docs checkと独立read-only review `ACCEPT` P0/P1=0
