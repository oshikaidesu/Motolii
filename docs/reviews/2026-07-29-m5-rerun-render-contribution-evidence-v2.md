# M5 Rerun Render Contribution証拠転記 v2

作成日: 2026-07-29

状態: **停止／P2D-RCB2差分不採用**（Grok `REJECT`、P0=1/P1=1）

変更許可: 本fileのみ

単一動詞: **転記する**

## MOTOLII AUTHORITY

M5 task `P2/P3/P2D`、Render Contribution証拠Wave親task §2〜§3と、
同節が固定するsemantic seat／Controlled Microkernelの元authority。完成条件はMotolii fixtureで判定し、
Rerunへの類似を要件にしない。

## CODE FACT GAP

親task §3の固定hashが示す`LayerSourcePlugin::render`、`build_source`、`dispatch_plugin`の現行call pathと、
provider非依存Observation要求、shared depth admission、複数phase contribution、
transparent／refraction capability交渉が未成立である事実。hash不一致なら再解釈せず停止する。

## RERUN EVIDENCE

commit `954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`の固定6 assetと、次の三capsuleだけを読む。

- [custom visualizer capsule](2026-07-29-m5-capsule-rerun-custom-visualizer.md)
- [custom view capsule](2026-07-29-m5-capsule-rerun-custom-view.md)
- [draw phases capsule](2026-07-29-m5-capsule-rerun-draw-phases.md)

全関数、LFS snapshot、性能、Motolii適合は非証明範囲である。

## TRANSFER CLASS

[主担当Codex裁定](2026-07-29-m5-rerun-transfer-adjudication.md)を変更せず転記する。

- A1 `PATTERN`
- A2 `PATTERN`
- A3 `PATTERN`
- A4 `PATTERN`
- A5 `PATTERN`
- A6 `REJECT`

## TRANSFER LIMIT

変更許可は本fileだけ。Rerunの型、Entity、ViewClass、Blueprint、store、draw-phase enum、serde、shader、
dependencyを持ち込まない。固定6 asset外、network、repo横断探索、分類変更は`ORDER: STOP`。

## MOTOLII ORACLE

主担当Codex裁定の6 fixture候補、すなわち2D pixel不変、同一world/camera、
opaque／cutout／soft alpha、scene-color／refraction、unknown capability型付き拒否、
Preview／Export同一だけへ証拠を対応づける。Rerun類似を合格条件にしない。

## 出力

P2D-RCB2がこの節だけを置換し、A1〜A6ごとに
`固定観察 / 裏づける比較軸 / 非証明 / class / 持込禁止`
を転記する。新しいMotolii要件、推奨、裁定、公開契約を足さない。

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

- Rerun構造をMotolii要件へ昇格したくなる。
- 固定6 asset外、未裁定asset、分類変更が必要になる。
- 公開API、Document、plugin契約、wire、Vism/package、実装を変更する必要がある。
- 本file以外の変更、network、repo archaeologyが必要になる。
