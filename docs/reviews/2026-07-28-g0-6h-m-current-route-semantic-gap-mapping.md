# G0-6H-M 現行route element-level semantic gap map

- 日付: 2026-07-28
- 状態: **観察**
- G0-6H-M: **DONE**

## 1. 目的

承認済みの5状態と`docs/ui-visual-language.md`「## G0-6の審判」の5画面意図をelement単位で非推測照合し、`G0-6H-V0`前の判断材料だけを記録する。

## 2. 証拠の範囲と限界

以下の事実に限定する。
- `docs/reviews/2026-07-28-g0-6h-e-candidate-approval-observation.md`
- `docs/reviews/evidence/g0-6h-candidate-approval/README.md`
- `docs/reviews/2026-07-28-g0-6h-m0-current-route-semantic-gap-selection.md`
- `docs/reviews/2026-07-28-g0-6h-s-human-judgment-input-route-decision.md`
- `docs/ui-visual-language.md`

証拠はリポジトリに画像byteが存在しないため、判定は記録済みの可視事実のみに基づく。
撮影環境（OS / display / scale / ambient）・派生variant・capture metadataは未取得である。

## 3. state ↔ screen 対応表

名称対応のみであり、element parity の証拠とはしない。

- mixed Timeline ↔ screen 2
- Browser検索0件 ↔ screen 1
- Interval Easing ↔ screen 3
- Hand ↔ screen 4
- Relative Move ↔ screen 5

## 4. element-level gap map

### screen 1: empty project + asset browser

- empty project: 対応なし
- asset browser: 未確認

### screen 2: video/audio/shape/text/groupを含むtimeline

- video: 未確認
- audio: 未確認
- shape: 未確認
- text: 未確認
- group: 未確認
- timeline: 未確認

### screen 3: 選択項目のparameter panel + keyframe/easing popup + warning/disabled状態

- parameter panel: 未確認
- keyframe popup: 未確認
- easing popup: 未確認
- warning: 未確認
- disabled: 未確認

### screen 4: Stage + Output Frame + frame内外object + Camera tool / Hand(Stage View) tool

- Stage: 未確認
- Output Frame: 未確認
- frame内object: 未確認
- frame外object: 未確認
- Camera tool: 未確認
- Hand(Stage View) tool: 未確認
- output-frame boundary: 未確認
- 半透明scrim: 未確認
- selection識別を文字/色以外で行う要件: 未確認

### screen 5: 非隣接layer 3つが同じEffect Definitionを異なるstack位置で使うtimeline

- 非隣接3層の共有Definition利用: 未確認
- connection gutter常時線: 未確認
- from/outとuse/in: 未確認
- 折畳みstub+件数: 未確認
- 通常drag: 未確認
- Relative drag HUD: 未確認

## 5. screen総合判定

- screen 1 = `対応なし`（`G0-6H-M0`の確定事実で、`Browser検索0件`画面に`night_drive`のStage / Inspector / Timelineが残るため`empty project`ではないため）
- screen 2 = `partial`
- screen 3 = `partial`
- screen 4 = `partial`
- screen 5 = `partial`

## 6. panel presence 付記

全5画面にBrowser / Stage / Inspector / Timelineが含まれるという事実を、どの必須表示要素でも`対応`の根拠にはしない。

## 7. 確定しないこと

- 画像の採否
- 具体token
- threshold
- 派生variant判定
- 操作成功
- route parity
- capture metadata
- 隠れ状態
- `G0-6H` / `CU-0B01` / `U0e-3` / `U2c-3` / `U2c-5` の状態

## 8. `G0-6H-V0` の扱い

`G0-6H-V0`は `WAIT` のまま維持する。

## 9. 返す人間裁定（ちょうど1点）

screen 1の未充足処理として、次の2択を提示する。

- (a) 現行routeにおけるempty-project scenarioの意味を、別のscenario / fixture契約粒で新設する。
- (b) `G0-6 screen 1` を独立したspec決定で改訂する。

本粒は(a)を推奨として記載するにとどめ、採択はしない。
推奨理由は、`G0-6H-S`により現行routeが唯一のforward-looking human-judgment入力routeであり、screen 1の意図を変える前に現行routeでの不足を埋める方が影響面が狭いからである。

## 10. 非目標

- empty-projectの表示意味、scenario意味論、adapter / API、route / query shape、fixture schema、manifest形式の定義
- 画像・variant・generation・`CURRENT`・`reference-provenance.json`の生成・再生成・変更・移動・削除
- 具体token値、製品theme、threshold、golden、期待値、component、iconの選定・変更
- `docs/ui-visual-language.md`、`docs/ui-reference-map.md`、`docs/specs/M3-ui-integration.md`、`docs/README.md`、`docs/implementation-ledger.md` の変更
- `G0-6H` / `G0-6H-V0` / `CU-0B01` / `CU-0B02` / `U0e-3` / `U2c-3` / `U2c-5` の状態語変更
- React / CSS / Rust / test / guard / JSON / script / 画像の変更
- 公開API、Document意味、plugin契約、永続形式、serde defaultの変更・新設
- `reference-handoff.md` のDecision template / 5秒課題checklistの記入
- 隣接チケット（`CU-107*` / `CU-110*` / `CU-111` / `U3a-*` / `U2h-*` / `G0-9*` / `U0e-*`）への波及、2件以上の次一粒起票

## 11. 関連

- [G0-6H-M0 選定](2026-07-28-g0-6h-m0-current-route-semantic-gap-selection.md)
- [G0-6H-S 裁定](2026-07-28-g0-6h-s-human-judgment-input-route-decision.md)
- [G0-6H-E 観察](2026-07-28-g0-6h-e-candidate-approval-observation.md)
- [evidence README](evidence/g0-6h-candidate-approval/README.md)
- [reference handoff](../mocks-ui/reference-handoff.md)
- [ui-visual-language](../ui-visual-language.md)
