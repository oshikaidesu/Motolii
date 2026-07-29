# M5 Rerun観察転記 v3

作成日: 2026-07-29

状態: **停止**（`P2D-RCB3` Grok REJECT。後継`P2D-RCB6`を`P2D-RCI`へ統合済み。再発注禁止）

変更許可: 本fileの`転記欄`だけ

単一動詞: **転記する**

## MOTOLII AUTHORITY

M5 task `P2/P3/P2D`、[Render Contribution証拠Wave親task](2026-07-29-m5-render-contribution-evidence-wave.md)
§2〜§3、§6。Rerunへの類似をMotolii要件または完成条件にしない。

## CODE FACT GAP

親task §3の固定hashが示す`LayerSourcePlugin::render`、`build_source`、`dispatch_plugin`の現行call path。
provider非依存Observation要求、shared depth admission、複数phase contribution、
transparent／refraction capability交渉は未成立である。hash不一致なら再解釈せず停止する。

## RERUN EVIDENCE

commit `954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`の固定6 assetを収めた次の三capsuleだけを読む。

- [custom visualizer capsule](2026-07-29-m5-capsule-rerun-custom-visualizer.md)
- [custom view capsule](2026-07-29-m5-capsule-rerun-custom-view.md)
- [draw phases capsule](2026-07-29-m5-capsule-rerun-draw-phases.md)

## TRANSFER CLASS

[主担当Codex裁定](2026-07-29-m5-rerun-transfer-adjudication.md)のA1〜A5=`PATTERN`、A6=`REJECT`を変更しない。

## TRANSFER LIMIT

変更許可は本fileの`転記欄`だけ。固定6 asset外を読まず、Rerun型、Entity、ViewClass、Blueprint、
store、draw-phase enum、serde、shader、dependencyを持ち込まない。分類変更、要約による意味追加、
Motolii fixtureへの対応づけを行わない。

## MOTOLII ORACLE

このgrainの合否は、下記A1〜A6と5 fieldが維持され、各fieldが指定capsuleまたは裁定書の記載へ
一対一で遡れることで判定する。Rerun類似、製品実装、Motolii fixture対応は判定しない。

## 固定転記欄

### A1 built-in Spatial3DViewへの拡張登録

- 固定観察: <!-- 転記欄 -->
- 裏づける比較軸: <!-- 転記欄 -->
- 非証明: <!-- 転記欄 -->
- class: <!-- 転記欄。PATTERNをそのまま転記 -->
- 持込禁止: <!-- 転記欄 -->

### A2 VisualizerSystemとdraw data生成

- 固定観察: <!-- 転記欄 -->
- 裏づける比較軸: <!-- 転記欄 -->
- 非証明: <!-- 転記欄 -->
- class: <!-- 転記欄。PATTERNをそのまま転記 -->
- 持込禁止: <!-- 転記欄 -->

### A3 Rendererとphase参加

- 固定観察: <!-- 転記欄 -->
- 裏づける比較軸: <!-- 転記欄 -->
- 非証明: <!-- 転記欄 -->
- class: <!-- 転記欄。PATTERNをそのまま転記 -->
- 持込禁止: <!-- 転記欄 -->

### A4 custom ViewClass登録

- 固定観察: <!-- 転記欄 -->
- 裏づける比較軸: <!-- 転記欄 -->
- 非証明: <!-- 転記欄 -->
- class: <!-- 転記欄。PATTERNをそのまま転記 -->
- 持込禁止: <!-- 転記欄 -->

### A5 draw phase managerの登録・実行責任

- 固定観察: <!-- 転記欄 -->
- 裏づける比較軸: <!-- 転記欄 -->
- 非証明: <!-- 転記欄 -->
- class: <!-- 転記欄。PATTERNをそのまま転記 -->
- 持込禁止: <!-- 転記欄 -->

### A6 固定draw phase語彙

- 固定観察: <!-- 転記欄 -->
- 裏づける比較軸: <!-- 転記欄 -->
- 非証明: <!-- 転記欄 -->
- class: <!-- 転記欄。REJECTをそのまま転記 -->
- 持込禁止: <!-- 転記欄 -->

<!-- P2D-RC COMMON NON-GOALS BEGIN -->
- 公開API、Document schema、plugin契約、wire形式、Vism/package/schema、実装コード、fixtureコードを変更しない。
- `RenderContribution`等のRust名、trait signature、serde形、registry多重度、phase enumを決定しない。
- P2Dの初期3 policy、P3 Observation、Host authorityを別設計へ置換しない。
- Rerun／ゲームエンジンの型、状態所有、render graph、package名、material／phase enumをMotolii authorityにしない。
- Host enum、具体provider ID、raw JSON／文字列走査、opaque ID／private type走査、公開raw mutation、
  invented serde default、重複planner/helper、lint抑制、期待値／golden変更で境界を迂回しない。
- leaf単独で採用決定、実装解禁、P2D完了を宣言しない。
<!-- P2D-RC COMMON NON-GOALS END -->

## STOP

- 固定見出しまたは5 fieldの変更、固定6 asset外の参照、分類変更が必要になる。
- Rerun構造をMotolii要件へ昇格する、またはfixture対応づけを始める。
- 本fileの`転記欄`以外の変更が必要になる。
