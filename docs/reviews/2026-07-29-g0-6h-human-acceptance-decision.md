# G0-6H 現行React UI 人間審判 ACCEPT

- 日付: 2026-07-29
- 状態: **決定**
- `G0-6H` / `CU-0B01`: **DONE / ACCEPT**
- 次の一粒: `CU-0B02` / `U0e-3`

## 1. 決定

プロジェクト所有者かつ対象UIの作者は、現行
`#plugin-browser-candidate`を「完成されている」と判定し、外観調整ではなく
Reactとnativeの接続へ進むことを明示的に承認した。

この判定をG0-6Hの最終人間審判`ACCEPT`として記録し、`CU-0B01`を完了する。
これにより、具体token、共通component state、icon gridを製品へ導入する
`CU-0B02` / `U0e-3`を次の一粒として解禁する。

## 2. 人間session

- 判定者: プロジェクト所有者 / 対象UI作者
- 実施日: 2026-07-29
- 表示環境: macOS / MacBook内蔵画面 / 100% / 暗い室内
- 人間が判定した入力: live `#plugin-browser-candidate`
- 対象状態: `empty-browser`、`mixed-timeline`、`parameter-easing`、
  `stage-frame-tools`、`shared-effect-relative`に対応する現行route 5状態
- 採否: `ACCEPT`
- 理由: 現行UIは完成しており、次に気にすべき対象はReact/native接続であるという
  UI作者の明示判断
- 修正要求: なし

## 3. 自動証拠との関係

current-route evidence generation
`44e538c97807-ead41d4d6562`は、manifest v2、5 screen、各6 variant、
計30 captureを持つ自動回帰証拠として併置する。

人間sessionはlive normal routeへの総合判断である。派生
`lightness` / `grayscale` / Machado CVD bitmapを1枚ずつ人間が閲覧したとは
記録しない。派生画像の閉集合、hash、read-only照合は機械証拠であり、
人間判断の内容を遡及的に増やさない。

generationは後続のReact product変更より前のimmutable evidenceであるため、
最新sourceとのbyte同一性を主張しない。今回の決定はUI作者による現行live routeの
採否であり、古いgenerationを遡及変更しない。

BTP/ITP後のsource closureと`CURRENT` generationの不一致で生じている既知
`CR2-SCHEMA` 2件は、人間審判とは独立したORACLE-GUARD
`G0-6H-V1G-RP`へ移管する。この粒は旧generationを変更せず、既存V1G commandで
最新sourceから新しいimmutable generationをpublicationして`CURRENT`を原子的に交換し、
read-only checkを緑へ戻す。G0-6Hの採否、token、golden、threshold、React byteを変更しない。

## 4. 採択と停止線

採択する候補は、現行React UIが用いるrole token、既存component state、
既存icon体系である。`docs/mocks-ui/fixtures/reference-candidate-tokens.json`の
具体候補は、U0e-3でDTCG単一正本、型付き生成、contrast/focus、
意味色+形、gradient allowlist、raw color/spacing拒否へ照合してから製品導入する。

棄却する候補はない。ただし本ACCEPTは次を許可しない。

- React UIの再設計、DOM/class/stable ID/interaction/visual thresholdの変更
- mock、legacy script、fixtureを製品runtimeの正本にすること
- concrete color、px、DPI、radiusをDocument、公開API、plugin契約、永続形式へ保存すること
- ReactにDocument、selection、Undoの第二正本を作ること
- native Stage/Timeline、WebView Host接続、H1b、W0bをU0e-3へ束ねること

## 5. 次

次の一粒は`CU-0B02` / `U0e-3`である。目的は受理済みの見た目を変えず、
確定token、共通component state、icon gridを既存generator境界から製品へ導入すること。
接続背骨の次段であり、appearance redesignではない。

`G0-6H-V1G-RP`は独立ORACLE-GUARD laneの修復粒であり、PRODUCT-ASSETの
`CU-0B02`を迂回・代替・直列停止しない。
