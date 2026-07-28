# U0e-2 / G0-6H reference handoff

状態: **自動生成完了 / 人間審判未実施 / U0e-3停止**

この資料は見た目の採否を記録しない。5画面と派生画像をG0-6Hの人間審判へ渡すための未記入templateである。

## 固定証拠

- React source authority: `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0`
- capture generation: `u0e2-08f96cbd7754-85c0fc529ab1`
- source manifest SHA-256: `08f96cbd77545e1734cc285970137ba20e1b9f31f3fac8f4e3704c467daa64a4`
- capture: Chromium Headless Shell `149.0.7827.55`, revision `1228`, `1440x900`, scale `1`, `en-US`, `UTC`, dark, reduced motion
- font fixture: `@fontsource/inter 5.3.0`のInter 400/600とOFL-1.1 license。製品fontの採択ではない
- images: `reference-output/CURRENT`が指すgeneration内の5画面×6 variant、計30 PNG
- provenance: `reference-provenance.json`

再現コマンド:

```sh
npm ci
npm run check-reference
```

`check-reference`は同じ三層fixtureからnormalを再captureし、全派生をnormal RGBAから再計算して、30 PNGとmanifestをread-only照合する。

## 自動report

自動確認済みの範囲:

- 5 screen IDと各semantic IDの閉集合、重複・欠落0
- 同じ`reference-document.json` / `reference-scenes.json` / `reference-candidate-tokens.json`から全画面を投影
- 三層それぞれのprobeが指定normal captureだけを変え、2順序で決定的
- React reference routeはcatalog/archiveから隔離され、legacy runtime importとsource copyが0
- Inter 400/600を実ロードし、外部network 0、固定browser version/revision一致
- normal 5枚、lightness/grayscale/Machado CVD 25枚、計30枚。欠落・余分・1 pixel差を拒否
- immutable generationとatomic `CURRENT`交換、check実行前後のGit可視file/status不変

自動reportが判定しない範囲:

- 階層が5秒で読めるか
- video/audio/shape/text/group、選択、警告、無効、接続を色だけに頼らず識別できるか
- 新しい状態表示が既存のBrowser / Stage / Inspector / Timelineへ馴染むか
- 装飾・余白・彩度・丸角が過剰でないか
- CVD/lightness/grayscale画像で意味の衝突が知覚されるか
- 具体token値、製品theme、製品font、component stateを採択するか

## 5秒課題と未記入checklist

各行についてnormal、lightness、grayscale、protanopia、deuteranopia、tritanopiaを同じ表示環境で確認する。

| screen | 5秒課題 | 階層 | 識別 | 馴染み | 過剰装飾なし | 所見 |
|---|---|---|---|---|---|---|
| `empty-browser` | asset browser、transport、次に行う操作の説明口を指す | [ ] | [ ] | [ ] | [ ] | 未記入 |
| `mixed-timeline` | 5種object、選択、mute、keyframe、bake/cacheを指す | [ ] | [ ] | [ ] | [ ] | 未記入 |
| `parameter-easing` | 選択parameter、easing popup、focus、warning、disabledを指す | [ ] | [ ] | [ ] | [ ] | 未記入 |
| `stage-frame-tools` | Output Frame、内外object、scrim、Select/Camera/Handを指す | [ ] | [ ] | [ ] | [ ] | 未記入 |
| `shared-effect-relative` | 共有definitionの3 use、stack差、接続方向、fold数、Relative HUDを指す | [ ] | [ ] | [ ] | [ ] | 未記入 |

## Decision template

- 判定者: 未記入
- 実施日: 未記入
- 表示環境（OS / display / scale / ambient）: 未記入
- 使用generation: `u0e2-08f96cbd7754-85c0fc529ab1`
- 5秒課題の結果: 未記入
- 採否: 未記入（`ACCEPT` / `REVISE`）
- 採否理由: 未記入
- 修正要求（screen / semantic role / observed problem）: 未記入
- 採択する具体token候補: 未記入
- 棄却する具体token候補: 未記入
- 次に解凍する粒: 未記入

人間の記録が正本へ入るまで`U0e-3`、`U2c-3`、`U2c-5`へ進まない。

## G0-6H-E 限定観察の非充足注記

- この限定観察は、現行候補 `#plugin-browser-candidate` の 1440×900 dark normal 5 画面への肯定的応答に対する docs-only 記録を示す。
- `reference-handoff.md` の Decision template と checklist は未充足のままとし、履行は観察台帳へ閉じる。
- 同期参照は [G0-6H-E 限定観察](../reviews/2026-07-28-g0-6h-e-candidate-approval-observation.md)。

## G0-6H-R authority役割の非先取り注記

- 本資料の固定証拠（:9-16）が指す React source authority `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0` は、generation `u0e2-08f96cbd7754-85c0fc529ab1` に限った不変の再現source authorityであり、現行product surfaceの所有authorityではない。
- 現行 `#plugin-browser-candidate` normal色5画面のproduct-owned React source authority は `56c318edcddab7cf95d263cc2f7dd2b4e6791134` であり（`ui/motolii-web/source-provenance.json`）、旧generation `u0e2-08f96cbd7754-85c0fc529ab1` のsource authority欄へ遡及記載しない。
- 二commitのGit ancestry成立は系譜事実に留まり、route横断のvisual parity・人間承認・route同一性の根拠にはならない。`check-reference` 成功も固定generationのread-only再現証拠に留まり、現行候補5画面との同一性・本資料のDecision template充足・G0-6H完了の代替にはならない。
- 本資料の Decision template と checklist は未充足のままとし、履行は [G0-6H-R authority役割再照合](../reviews/2026-07-28-g0-6h-r-reference-authority-role-reconciliation-decision.md) へ閉じる。`#reference/*` と `#plugin-browser-candidate` のどちらをG0-6H人間審判の入力routeとするかは `G0-6H-S` の裁定待ちとする。

## G0-6H-S route裁定の非先取り注記

- 本資料が固定する generation `u0e2-08f96cbd7754-85c0fc529ab1` と source authority `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0` は不変のまま保存され、再現証拠および派生生成のderivation-regression証拠として維持される。
- `G0-6H-S` により、以後のG0-6H人間審判入力routeは `#plugin-browser-candidate`（product-owned React source authority `56c318edcddab7cf95d263cc2f7dd2b4e6791134`）だけとなり、本資料の30 PNGはrequired human-judgment inputではなくなった。
- 現行候補normal色5画面の承認はpartial evidenceに留まり、本資料のDecision template / checklistの充足、`G0-6H` / `CU-0B01` / `U0e-3` の完了・解禁に代替しない。
- 現行route用のevidence contract（5状態semantic mapping、固定capture環境、normal＋lightness / grayscale / Machado CVD派生、immutable manifestとread-only check、記録されたhuman session）は要求として `G0-6H-V0` へhandoffし、本粒では実装しない。

## G0-6H-M semantic gap mapの非充足注記

- element-level gap mapは `G0-6H-M` の観察記録へ閉じ、本資料の Decision template / checklist の充足には代替しない。
- 記録済み可視事実だけを使い、隠れ状態、操作成功、capture metadata、route parityは判定していない。
- `G0-6H-V0` は `WAIT` を維持し、empty-project scenarioの意味は本粒で定義していない。
- 詳細は [G0-6H-M現行route element-level semantic gap map](../reviews/2026-07-28-g0-6h-m-current-route-semantic-gap-mapping.md)

## G0-6H-A scenario / fixture契約の非充足注記

- 本契約はscenario意味とfixture所有の停止線だけを閉じ、本資料のDecision template / checklistの充足には代替しない。
- `Starter Media` fixtureはProject外のfixture-only源であり、Document / 製品runtime / 公開API / plugin / 永続形式 / production Registered folderの正本にならない。
- 本粒はmedia byte、path、schema、route、生成手段を決めていない。次粒 `G0-6H-AF` へhandoffする。
- `G0-6H-V0`は本契約の統合まで`WAIT`を維持する。
- 詳細への相対リンク [G0-6H-A empty Project + local Starter Media scenario / fixture 所有契約](../reviews/2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md)。
