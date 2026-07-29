# M5 Render Contribution typed seam decision

作成日: 2026-07-29

状態: **決定**

## 1. Authorityと非変更面

このdecisionは[Render Contribution統合decision](2026-07-29-m5-render-contribution-integration-decision.md)、[M5仕様](../specs/M5-3d-and-post.md)、[Controlled Microkernel決定](2026-07-25-controlled-microkernel-host-module-parallelism-decision.md)、[換装可能な意味の席決定](2026-07-24-replaceable-semantic-seat-decision.md)を正本とする。

公開Rust API、Document意味、serde／wire／schema、plugin契約、既存test／golden／画素は変更しない。

## 2. 二つの意味対象

admission前に空間表現が出す**型付き要求**と、Hostが要求を受理した後に集める**render contribution**を別の意味対象にする。同じ万能callback、同じ所有物、具体provider IDの分岐へ潰さない。

## 3. 公開観測上の多重度とwhole-request admission

公開観測上、semantic admission面は**一つ**、追加的な型付き能力語彙は**一つ**、admitted contributionとその実装は**many**とする。Host内部registryの個数、配置、分割は非観測であり、このdecisionでは未決とする。第二のsemantic admission面を作らない。

一つの要求全体に対するsemantic admissionは、決定論的な成功または型付き拒否のいずれかとする。部分admissionを作らず、admissionの成否によってDocumentも画素も変えない。

## 4. 所有、追加性、F5継承

contributionはworld、camera、Observation、transform、layer order、`Quality`、`FrameDesc`を所有しない。

Hostがadmission、ordering／phase resolve、共有資源、resource budget、診断と型付きfailureを所有する。P3のObservation形とcamera capabilityは先取りしない。

新能力は追加的に導入し、既存能力の意味を再解釈しない。将来serde／wire境界が生じても、RCI §3 F5をそのまま継承する。

未知contribution／capability不足はDocumentと既存2D compositionを変えず型付き拒否する。

## 5. trust eligibilityとsemantic admission

trust、permission、provenance、package eligibilityはsemantic admissionより前段の別責務であり、その段で拒否できる。

適格性通過後、provenanceはcapability admissionの結果、render意味、順序、authorityを変えず、唯一のsemantic admission面を迂回しない。

## 6. First Vismと将来形の負例

First Vismは専用type、feature、key、順序、package identity、registryを持たず、同じ境界を通る最初のconformance fixtureである。製品機能、package、配布、販売、UIの意味は決めない。

万能callback、opaque／`Any`への再結合、engine／provider公開registry、固定phase enum、sort key、raw mutation API、具体provider ID分岐を将来形にしない。具体Rust item名、trait形、registry実装形は本decisionで決めない。

## 7. RCS1の証拠限界

`P2D-RCS1`は`motolii-render`内部に閉じたprivate feasibility証拠としてだけ扱う。内部の型、resource配置、shader、pixel oracleを公開契約の根拠にしない。

## 8. 後続

`P2D-RCD2`、`P2D-RCF1`、`P2D-RCT1`、`P2D-RCO1`、`P2D-RCFP1`、`P2D-RCR1`、`P2D-RCP1`、`P2D-RCBUD1`はすべて`WAIT`のまま維持する。

## 9. STOP

次のいずれかが必要になった時点で停止する。

- 公開Rust item、trait、registry実装形を決めないと閉じない。
- Document、serde、wire、schema、migrationを決めないと閉じない。
- P3のObservation形またはcamera capabilityを先取りしないと閉じない。
- alpha、OIT、refraction、copy、resource、budget方式を決めないと閉じない。
- First Vismの製品、package、配布、UIの意味を決めないと閉じない。

## 10. 反対側助言の処分

2026-07-29、`claude-fable-5`をread-onlyで呼び、編集、委任、仕様決定を許さず反例監査した。

- 採用: 一／一／many、内部配置の非観測、whole-request admission、追加性とF5継承
- 縮小採用: provenanceをsemantic admissionの内側から除外し、trust eligibilityを前段責務として残す
- 延期: 具体Rust名とP3 context型

Fableは`VERDICT`、`P0`、`P1` markerを返していない。主担当Codexが元authorityへ再照合し、最終判断した。
