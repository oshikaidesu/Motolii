# U0e-2 / G0-6H reference handoff

状態: **自動生成完了 / 人間審判ACCEPT / U0e-3解禁**

この資料は5画面と派生画像をG0-6Hの人間審判へ渡した記録である。2026-07-29の
最終sessionは現行live `#plugin-browser-candidate`を対象にし、詳細は
[G0-6H人間審判ACCEPT](../reviews/2026-07-29-g0-6h-human-acceptance-decision.md)を正とする。

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

## 5秒課題と最終判定

派生6 variantは機械回帰証拠として各行へ対応する。最終人間sessionはlive normal routeを総合判定する。

| screen | 5秒課題 | 階層 | 識別 | 馴染み | 過剰装飾なし | 所見 |
|---|---|---|---|---|---|---|
| `empty-browser` | asset browser、transport、次に行う操作の説明口を指す | [x] | [x] | [x] | [x] | live route総合ACCEPT |
| `mixed-timeline` | 5種object、選択、mute、keyframe、bake/cacheを指す | [x] | [x] | [x] | [x] | live route総合ACCEPT |
| `parameter-easing` | 選択parameter、easing popup、focus、warning、disabledを指す | [x] | [x] | [x] | [x] | live route総合ACCEPT |
| `stage-frame-tools` | Output Frame、内外object、scrim、Select/Camera/Handを指す | [x] | [x] | [x] | [x] | live route総合ACCEPT |
| `shared-effect-relative` | 共有definitionの3 use、stack差、接続方向、fold数、Relative HUDを指す | [x] | [x] | [x] | [x] | live route総合ACCEPT |

`[x]`はUI作者によるlive routeの総合ACCEPTを表す。派生bitmapを個別に人間確認した
という意味ではなく、派生30 captureは機械回帰証拠として保持する。

## Decision template

- 判定者: プロジェクト所有者 / 対象UI作者
- 実施日: 2026-07-29
- 表示環境（OS / display / scale / ambient）: macOS / MacBook内蔵画面 / 100% / 暗い室内
- 使用generation: current-route `44e538c97807-ead41d4d6562`を自動補助証拠として併置。人間入力はlive `#plugin-browser-candidate`
- 5秒課題の結果: 5状態すべて総合`ACCEPT`
- 採否: `ACCEPT`
- 採否理由: UI作者が現行UIを完成と判断し、次はReact/native接続へ進むと明示
- 修正要求（screen / semantic role / observed problem）: なし
- 採択する候補の送り先: 現行role token候補は`CU-0B02T`のDTCG/生成check、既存component stateは`CU-0B02C`、現行inline SVG/Unicode混在からのicon体系採択は`CU-0B02I`
- 棄却する具体token候補: なし
- 次に解凍する粒: [CU-0B02S分割決定](../reviews/2026-07-29-cu-0b02s-product-token-ownership-split-decision.md)後の`CU-0B02T`

`U0e-3`を解禁する。`U2c-3`、`U2c-5`はU0e-3完了後の既存依存順を維持する。

## G0-6H-E 限定観察の非充足注記

以下の非充足注記は2026-07-29最終人間sessionより前の履歴である。今回の
最終ACCEPTがG0-6H状態を更新し、過去時点の証明範囲そのものは遡及変更しない。

- この限定観察は、現行候補 `#plugin-browser-candidate` の 1440×900 dark normal 5 画面への肯定的応答に対する docs-only 記録を示す。
- `reference-handoff.md` の Decision template と checklist は未充足のままとし、履行は観察台帳へ閉じる。
- 同期参照は [G0-6H-E 限定観察](../reviews/2026-07-28-g0-6h-e-candidate-approval-observation.md)。

## G0-6H-R authority役割の非先取り注記

- 本資料の固定証拠（:9-16）が指す React source authority `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0` は、generation `u0e2-08f96cbd7754-85c0fc529ab1` に限った不変の再現source authorityであり、現行product surfaceの所有authorityではない。
- 現行 `#plugin-browser-candidate` normal色5画面のproduct-owned React source authority は `56c318edcddab7cf95d263cc2f7dd2b4e6791134` であり（`ui/motolii-web/source-provenance.json`）、旧generation `u0e2-08f96cbd7754-85c0fc529ab1` のsource authority欄へ遡及記載しない。
- 二commitのGit ancestry成立は系譜事実に留まり、route横断のvisual parity・人間承認・route同一性の根拠にはならない。`check-reference` 成功も固定generationのread-only再現証拠に留まり、現行候補5画面との同一性・本資料のDecision template充足・G0-6H完了の代替にはならない。
- 本項記録時点では Decision template と checklist は未充足だった。2026-07-29最終sessionの充足は冒頭Decision templateを正とし、履歴上の非充足を遡及変更しない。

## G0-6H-S route裁定の非先取り注記

本項は2026-07-29最終人間sessionより前のroute裁定時点の証明範囲である。
以下の「代替しない」は当時のpartial approvalだけを指し、冒頭の最終ACCEPTを否定しない。

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

## G0-6H-AF 媒体源・provenance裁定の非充足注記

- 本裁定は媒体源と provenance class だけを閉じ、本資料の Decision template / checklist の充足に代替しない。
- 採択は決定的生成、pinned vendoring の棄却は本 fixture に限り repo 全体の禁止ではない。
- 本粒は path、file名、codec、schema、hash algorithm、生成command、tool version、byte を決めていない。次粒 `G0-6H-AG0` へ handoff する。
- 詳細への相対リンク [G0-6H-AF Starter Media 媒体源・provenance class 裁定](../reviews/2026-07-28-g0-6h-af-starter-media-source-provenance-decision.md)。

## G0-6H-AG0 generator責任処分の非充足注記

- 本粒は generator / output closure の棚卸しと `WRAP` / `FROZEN / DELETE-LATER` 責任処分だけを閉じ、本資料の Decision template / checklist の充足に代替しない。
- 採用は既存 `docs/mocks-ui` Node evidence route と外部 ffmpeg CLI 境界の WRAP。Rust test-local WAV helper の JS 境界越え再利用と、新規 codec / media framework / process supervisor / 製品 service の新設は棄却。
- 本粒は path、file名、codec、schema、hash algorithm、生成command、tool version、media byte を決めていない。次粒 `G0-6H-AG` へ handoff する。
- generator / integrity check は製品 runtime から到達しない証拠カプセルとして後続粒に委ね、`G0-6H-V0` は `WAIT` を維持する。
- 詳細への相対リンク [G0-6H-AG0 Starter Media generator / output closure 棚卸しと責任処分](../reviews/2026-07-28-g0-6h-ag0-starter-media-generator-closure-inventory.md)。

## G0-6H-V0 variant evidence契約の非充足注記

- 本契約は current-route evidence の要求だけを閉じ、本資料の Decision template / checklist の充足に代替しない。
- 閉じた対象は5状態 semantic mapping、capture 環境9軸の閉集合、normal＋既存派生5件の計6 variant、immutable generation と SHA-256 manifest＋read-only 照合、human session 記録項目の閉集合。
- 本粒は画像、script、fixture、media byte、token、threshold、golden、route 実装、manifest schema、hash algorithm、check command を決めていない。次粒 `G0-6H-V1` へ handoff する。
- screen 1 の充足は `G0-6H-A` の empty Project + Project 外 `Starter Media` scenario の成立を条件とし、固定 byte は commit `e4ad5c9f` の fixture-only 証拠カプセルに閉じる。
- 詳細への相対リンク [G0-6H-V0 現行route variant evidence契約](../reviews/2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md)。

## G0-6H-V1S capture境界裁定の非充足注記

- 本裁定は `G0-6H-S` / `G0-6H-M` / `G0-6H-A` / `G0-6H-AF` / `G0-6H-AG0` / `G0-6H-V0` の停止線を受けたうえで、`B-1` から `B-4` の責任境界を docs-only で確定する。
- `B-1` は route同一性維持、`B-2` は development 専用 projection 境界、`B-3` は Starter Media の表示意味の停止線、`B-4` は capture 9軸記録責任の分割禁止を裁定する。
- 本粒は画像・variant・generation・manifest schema・hash algorithm・check command の具体値、`route / query` 設定、script / fixture / `node_modules` / `ui/motolii-web/**` / `docs/mocks/**` 変更を決めない。
- 本粒は `G0-6H-V1` の実装（capture 実施、具体値確定、schema、command、画像）へ進めない。
- 詳細は [G0-6H-V1S 現行route capture境界の裁定](../reviews/2026-07-28-g0-6h-v1s-current-route-capture-boundary-decision.md)。
