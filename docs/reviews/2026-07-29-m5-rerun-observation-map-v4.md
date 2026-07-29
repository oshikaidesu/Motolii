# M5 Rerun観察fragment map v4

作成日: 2026-07-29

状態: **登録済み／P2D-RCB4未実行**

変更許可: 本fileの`配置欄`だけ

単一動詞: **配置する**

## MOTOLII AUTHORITY

M5 task `P2/P3/P2D`と[Render Contribution証拠Wave親task](2026-07-29-m5-render-contribution-evidence-wave.md)
§2〜§3、§6。Rerunへの類似をMotolii要件または完成条件にしない。

## CODE FACT GAP

親task §3の固定hashが示す現行call pathを使う。provider非依存Observation要求、shared depth admission、
複数phase contribution、transparent／refraction capability交渉は未成立である。

## RERUN EVIDENCE

固定commit `954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`の三capsuleと
[転移裁定](2026-07-29-m5-rerun-transfer-adjudication.md)だけを入力にする。

## TRANSFER CLASS

A1〜A5=`PATTERN`、A6=`REJECT`を変更しない。

## TRANSFER LIMIT

固定fragment IDの配置以外を行わず、Rerun型、phase語彙、fixture対応、分類変更を持ち込まない。

## MOTOLII ORACLE

A1〜A6の各行が固定fragment IDへ一対一で遡れ、A6が`R-A6-LIM`を持つことで判定する。

## 固定fragment

- `R-A1-OBS`: `App::extend_view_class`が既存`Spatial3DView`へvisualizerとfallback providerを登録する。
- `R-A2-OBS`: visualizerはquery／transform／highlightからrenderer固有draw dataを生成する。
- `R-A3-OBS`: draw dataは複数phaseへの参加を宣言し、例ではOpaque、Picking、Outlineへ分かれる。
- `R-A4-OBS`: 既存Viewの能力追加と、新View追加は別の登録動線である。
- `R-A5-OBS`: draw dataがphase別drawableを収集し、managerが全draw dataを保持してphase単位にsort／dispatchする。
- `R-A6-OBS`: phase集合は固定enumで、source自身がphase abstractionを進行中と注記する。
- `R-A1-AXIS`: 既存Viewへ能力を追加する動線と、新View追加を別責任にする比較先例だけを使う。
- `R-A2-AXIS`: query／transform等からrenderer固有draw dataを生成する責任分離だけを比較する。
- `R-A3-AXIS`: 表現固有resource／pipelineとHost側phase参加を分ける先例だけを比較する。
- `R-A4-AXIS`: 新しいView登録は既存View拡張と別動線である、という負例比較だけを使う。
- `R-A5-AXIS`: 複数contributionをHost側で収集し、ordering／dispatchする責任分離だけを比較する。
- `R-A6-AXIS`: 固定phase enum、phase名、sort keyをMotoliiの公開契約または閉じた能力集合へ転移しない。
- `R-VIS-NP`: Motoliiの公開plugin契約、万能renderer trait、Document形、phase enum、第三者UI、性能適合を証明しない。
- `R-VIEW-NP`: RerunのApp／ViewClass／Blueprint／component UI責任はMotoliiのHost／plugin／Document境界を証明しない。
- `R-PHASE-NP`: soft alpha交差の正解、OIT、scene-color lifetime、Motolii resource budgetを証明しない。
- `R-PATTERN`: PATTERN。
- `R-REJECT`: REJECT。
- `R-COMMON-LIM`: RerunのApp、Entity、ViewClass、Blueprint、store、query、draw-phase enum、sort key、serde、shader、dependency、UI stateをMotoliiへ持ち込まない。
- `R-A6-LIM`: 固定phase enum、phase名、sort keyをMotoliiの公開契約または閉じた能力集合へ転移しない。

## 固定配置matrix

各`配置欄`へ上記IDだけを一つ置く。本文の複製、言い換え、複数IDの合成をしない。

| asset | 固定観察 | 比較軸 | 非証明 | class | 持込禁止 |
|---|---|---|---|---|---|
| A1 | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| A2 | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| A3 | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| A4 | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| A5 | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |
| A6 | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> | <!-- 配置欄 --> |

## STOP

- 固定fragmentだけでは配置できず、新しい文、ID、資料、解釈が必要になる。
- fixture対応、分類変更、公開契約、実装を始める。
- 本fileの`配置欄`以外を変更する。
