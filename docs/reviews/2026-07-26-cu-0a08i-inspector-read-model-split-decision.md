# CU-0A08I Inspector read-model再判定・分割決定

- 日付: 2026-07-26
- 状態: **決定**
- 完了: **CU-0A08IS DONE**
- CU-0A08IP: **DONE**
- 後続: **CU-0A08IT WAIT**

## 1. 再判定の結論

`CU-0A08I`を一つのHost projection / typed intent実装として発注しない。
R4Cの製品所有は成立したが、現行`NodeDesc` / `DocParam`だけではInspectorの全表示を閉じられず、
Host→React transportとprimary selection projectionも未実装である。部分接続は同一panelへ
projection由来とpresentation直書きの二重ownerを作るため、次の3粒へ分ける。

| ID | 状態 | 一成果 | 依存 | 合格 | STOP |
|---|---|---|---|---|---|
| CU-0A08IS | `DONE` | 全表示要素を既決sourceあり／未決へ分類し、projection decoderの閉じた入力と拒否条件を固定 | CU-0A07C | source型・file・test oracleが各要素にあり、未決を推測しない | Document field、plugin公開契約、transport、intent型の発明が必要 |
| CU-0A08IP | `DONE` | 決定済みread modelだけをfixture由来のfail-closed decoderへ落とす。Host transportとは呼ばない | CU-0A08IS | unknown enum/field、non-finite、dangling、fixture revision不一致拒否。React semantic writer 0 | 同一意味の二重store、DOM/class/ARIA変更、Host/WebView公開境界が必要 |
| CU-0A08IT | `CORE / WAIT` | **Direct** Inspector接続: React操作を既存の intent→command→D2 終端へ接続 | CU-0A08IP `DONE`、U4a-2 Direct製品入口 | 1 gesture=1 intent、失敗/Cancel=変更0、並行writer 0 | U4a-2 Direct未成立、別intent終端、React側Undo/selection正本が必要。**U4cはAdvanced入口でありCU-0A08ITの依存ではない。U4cはU2c-2のAdvanced依存** |

## 2. 現行field照合

| Inspector表示群 | 現行source | 判定 |
|---|---|---|
| plugin名、category、tags、parameter id/type/default/domain、input数 | `NodeDesc` / `ParamDef` | 既決。CU-0A08ISで表示単位への採否を固定する |
| parameterのConst / Keyframes / Data / Vec2Axes / LookAt / Follow | `DocParam` | 既決。read-only分類候補 |
| transform / opacity値、key数 | 現行Document / `DocParam` | CU-0A08ISで正確なsourceと表示単位を照合 |
| Group child数 | 現行layer tree候補 | CU-0A08ISで数え方と対象scopeが既決か再照合 |
| Link行 | `DocParam::LookAt` / `Follow`候補 | labelから同一視せずCU-0A08ISで採否を固定 |
| blend mode | Document commandに型はある | 表示read sourceはCU-0A08ISで再照合 |
| effect description、input socket label/type tag | 該当`NodeDesc` fieldなし | 未決。ID・labelから推測しない |
| Fill / Stroke、Z Occlusion、bake point、DRIVER route | 現行Document fieldなし、または対応意味未決 | 未決。新fieldを足さない |
| availability lifecycle、APPLIED PLUGINS履歴 | lifecycle/read-model契約なし | 未決。mock文言を製品意味にしない |
| SELECTED OBJECT / EDITING EFFECT ON OBJECT | primary selection projection未実装 | U2h-1側の正本待ち |

## 3. 再利用と実装順

- decoder先例は`docs/mocks-ui/src/reference/loadReferenceFixtures.js`の
  Document fixture由来・unknown/non-finite/missing fail-closed形を第一候補にする。
  第二consumerが成立する前に汎用helperへ昇格しない。
- typed intent終端は既存`motolii-ui`の`DomainIntent`、
  `DocumentCommandRequest`、D2 macroだけとし、WebView専用の並行writerを作らない。
- transport不在中のCU-0A08IP成果名は「fixture由来read-only projection decoder」とし、
  `Host projection`や`Motolii Studio Preview`完成証拠にしない。

## 4. 非目標と停止線

- 公開API、Document/serde、plugin契約、NodeDesc field、selection/Undo正本を追加しない
- R4CのDOM、class、stable ID、ARIA、interaction、visual threshold/goldenを変更しない
- 未決表示をopaque ID、label、index、thumbnail token、mock literalから推測しない
- CU-0A08ISにdecoder実装、Host transport、typed intent実装を混ぜない
