# M5 Render Contribution統合decision

作成日: 2026-07-29

状態: **決定／P2D-RCI・P2D-RCS1・P2D-RCD1 DONE**

## 1. Authorityと入力

このdecisionはM5 `P2/P3/P2D`、[換装可能な意味の席／Provider決定](2026-07-24-replaceable-semantic-seat-decision.md)、
[Controlled Microkernel決定](2026-07-25-controlled-microkernel-host-module-parallelism-decision.md)を正本とし、
次のACCEPT済み証拠を反例照合へ使う。

- [Motolii境界map](2026-07-29-m5-render-contribution-boundary-map-v4.md): `P2D-RCA8`
- [Rerun観察map](2026-07-29-m5-rerun-observation-map-v4.md): `P2D-RCB6`
- [provider横断fixture map](2026-07-29-m5-provider-fixture-map-v5.md): `P2D-RCC5`

Rerun／engineの多数決でMotoliiの意味を決めない。外部先例がないF5／F6もMotolii authorityだけで維持する。

## 2. 決定

### 2.1 二つの意味対象

空間表現がadmission前に出す**型付き要求**と、Hostが要求を受理した後に集める
**render contribution**を別の意味対象にする。同じ万能callback、同じ所有物、具体provider IDの分岐へ潰さない。

### 2.2 所有

contributionはworld、camera、Observation、transform、layer order、`Quality`、`FrameDesc`を所有しない。
Hostがadmission、ordering／phase resolve、共有資源、resource budget、診断と型付きfailureを所有する。
shared depthへの参加要求そのものは禁止しないが、受理と資源決定をcontributionへ渡さない。

### 2.3 能力

opaque、cutout、soft alpha、scene-color／refractionは、要求能力、順序、alpha保証、fallback可否、
診断を別々に扱う。これは閉じた公開phase enumの採択ではない。copy、subpass、resource lifetime、同期、
OIT、screen-space方式は未決のまま後続へ分ける。

### 2.4 進化

新能力は追加的に導入し、既存能力の意味を再解釈しない。未知能力を黙示fallbackせず型付き拒否する。
固定phase enum、engine phase名、sort key、raw JSON、opaque ID、private型走査を公開契約へ持ち込まない。

### 2.5 First Vism

First Vismはfirst-party専用口を持たず、同じ境界を通る最初のconformance fixtureである。
具体表現、製品機能、package、販売／配布、UIはこのdecisionで決めない。

## 3. Conformance oracle骨格

F1〜F6はP2D全体の完了条件を置換せず、Render Contribution境界だけを判定する。

1. 同じworld／cameraのopaque 2面がZ交差で前後反転する。
2. cutoutは明示depth参加し、soft alphaをopaque depth writeへ黙示格上げしない。
3. soft alphaの順序依存と非対応診断を明示する。
4. scene-color／refraction要求は入力snapshot、範囲、順序、failureを宣言し、隠れcopyや別Export経路を作らない。
5. 未知contribution／capability不足はDocumentと既存2D compositionを変えず型付き拒否する。
6. contribution未使用時は既存pixel不変で、Preview／Exportは同じ評価関数を通る。

F1、F5、F6は外部先例の不足を理由に弱めない。AE-style Bins、Undo、選択、切替不変等のP2D既存fixtureは
この6項目とは別に残る。

## 4. 必須負例

- contributionが非所有Host状態を宣言または変更しようとする。
- unknown capabilityが黙示fallbackまたはDocument変更を起こす。
- soft alphaをopaque depth writeへ黙示格上げする。
- 公開語彙へ固定phase enum、engine phase名、sort keyが現れる。
- 第二の未知表現がHost enum、具体provider ID、raw JSON、private型走査を要求する。
- contribution未使用compでpixelが変わる、またはPreview／Exportが分岐する。
- First Vismだけが通れるAPIを作る。
- 同じ資源を同一段でread/writeする循環を黙って受理する。

## 5. 後続ticketと順序

| ID | 契約境界 | 依存 | 状態／完成条件 |
|---|---|---|---|
| `P2D-RCS1` | private opaque Group Depth spike | P2D-RCI | **DONE**。Grok `ACCEPT` P0/P1=0。`motolii-render`内部の実depth attachmentでF1／F6、group外pixel不変、FINAL／DRAFT同一評価関数を確認。Document／serde／公開API変更0 |
| `P2D-RCD1` | 型付き要求／contribution seam decision | P2D-RCS1、RCI §2.2のcamera／Observation非所有 | **DONE**。[typed seam decision](2026-07-29-m5-render-contribution-typed-seam-decision.md)がseam意味を固定。後続8件はWAIT、P3のObservation形は先取りしない |
| `P2D-RCD2` | P2D policy／Depth Participant schema decision | P2D-RCD1、M2-D1e | **WAIT**。GR-PV、追加migration、Undo意味を別decisionで閉じる |
| `P2D-RCF1` | 共通conformance harnessとFirst Vism fixture | P2D-RCD1 | **WAIT**。first-party専用口なし、F1〜F6とseat固有fixtureの分担を固定 |
| `P2D-RCT1` | cutout／soft alpha意味と診断 | P2D-RCD1 | **WAIT**。F2／F3、黙示depth格上げ拒否。OIT方式は別裁定 |
| `P2D-RCO1` | transparent交差のOIT方式decision | P2D-RCT1、P2D-RCS1 | **WAIT**。方式、品質、budget、unsupportedを比較し、公開phase語彙へしない |
| `P2D-RCFP1` | scene-color中間形式decision | M1、M4-K0 | **WAIT**。linear FP16推奨案を色一元化とbudgetへ照合 |
| `P2D-RCR1` | scene-color／refraction入力契約 | P2D-RCD1、P2D-RCFP1 | **WAIT**。snapshot、範囲、順序、failure。copy／subpass方式を同時に固定しない |
| `P2D-RCP1` | scene-color copy／subpass方式decision | P2D-RCR1、P2D-RCFP1 | **WAIT**。resource lifetime、同期、画面外sample、budgetを実機比較 |
| `P2D-RCBUD1` | contribution cache key／resource budget統合 | P2D-RCD1、M4-K1 | **WAIT**。cache入力完全性とHost計上を固定 |

## 6. private seamの実機証拠と次の解禁

`P2D-RCS1`は`motolii-render`内部のHost側spikeへ閉じ、既存
`RenderSession`／`LinearRenderGraph`の内側でopaque限定の2面を扱う。fixture-localな非永続値で駆動し、
`motolii-plugin`、Document、serde、wire、UI、soft alpha、refractionへ出さない。spikeの内部形を
`P2D-RCD1`の公開契約根拠にせず、実機反例としてだけ入力した。この条件でF1／F6の実機証拠が
Grok `ACCEPT` P0/P1=0に到達した。その証拠を入力として、[typed seam decision](2026-07-29-m5-render-contribution-typed-seam-decision.md)が`P2D-RCD1`を閉じた。

RCI §2.2でcontributionによるcamera／Observation所有を既に除外している。typed seam decisionは
非所有境界を維持し、P3のObservation形やcamera capabilityを発明していない。

## 7. 失効と非決定

[証拠Wave親task](2026-07-29-m5-render-contribution-evidence-wave.md) §2のnavigation語彙は本decisionで失効する。
[境界map](2026-07-29-m5-render-contribution-boundary-map-v4.md)の`C-*`、
[Rerun map](2026-07-29-m5-rerun-observation-map-v4.md)の`R-*`、
[Bevy map](2026-07-29-m5-bevy-observation-map-v4.md)の`B-*`、
[Unreal map](2026-07-29-m5-unreal-observation-map-v4.md)の`U-*`、
[provider map](2026-07-29-m5-provider-fixture-map-v5.md)が定義する`G-*`／`X-*` fragment IDを、
後続spec／公開契約の規範語彙にしない。
capsuleはFROZEN証拠として残すが、新しい発注は本decisionと元authorityを直接引用する。

このdecisionは公開trait、Rust名、Document field、serde／wire、phase enum、package形式、具体OIT／copy方式、
First Vismの製品意味を決めない。

## 8. STOP

- `P2D-RCS1`が公開crate、Document、serde、既存2D pixelへ波及する。
- `P2D-RCD1`がP3未決のObservation／camera能力を先取りする。
- trait、schema、packageを一つのticketで同時決定する必要が出る。
- Rerun／engine内部責任、固定phase語彙、具体方式を採らないと成立しない。
- F5／F6の外部証拠不足を理由にfixtureを弱める。
- First Vismのconformance役割と製品／配布意味を再び混ぜる。

## 9. 反対側助言の処分

2026-07-29、`claude-fable-5`をread-onlyで呼び、編集、委任、仕様決定を許さず反例監査した。
意味だけをRCIで閉じ、private spikeを先にし、公開契約／schema／alpha／refraction／budgetを直列分離する
推奨を採用した。F1／F5／F6を外部証拠不足で弱めない指摘、fragment ID失効、P3との停止線も採用した。
具体名や方式はauthorityで未決のため採用していない。最終判断は主担当Codexが元authorityとACCEPT済み
三mapへ再照合した。
