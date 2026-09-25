# UI rebaseline — brief(正本)

2026-09-25、利用者から受けた指示そのまま。以後の UI 移行はこの文書を正本とし、
[2026-09-20 ui-product-feel](../../reviews/2026-09-20-ui-product-feel.md) の
「新しいレイアウトを作る話ではない」はこの指示で上書きされた(利用者裁定)。

Concept Art: [concept/north-star-dark.png](concept/north-star-dark.png)(第一案)・
[concept/north-star-light.png](concept/north-star-light.png)・
[concept/relation-gadgets.png](concept/relation-gadgets.png)。
pixel-perfect target ではなく visual grammar の参照。

---

Motolii の Flutter UI を、新しい製品UIへ移行するための UI rebaseline を行ってください。

これは単純な reskin ではありません。
一方で、既存機能を捨てて新しいアプリを作り直す仕事でもありません。

今回の原則はこれです。

> Current UI is the capability oracle, not the layout oracle.
> 現行UIは「何ができなければならないか」の正本であり、
> 「どこに、どう表示しなければならないか」の正本ではない。

現行UIに存在する要素は、過去に必要性があって追加されたものです。
したがって、新UIを作るために機能を削除してはいけません。

ただし、
panel位置、画面構成、常時表示かcontextual表示か、controlの表現、
visual hierarchy、navigation、skinは再設計して構いません。

## 0. 最重要：現行UIを残す

現在のFlutter UIを破壊的に置換しないでください。

現行UIは Classic UI / capability oracle として、
今後も起動・比較できる状態を維持します。

新UIは別projection / shellとして実装してください。

理想的には同じ、

- Document
- Domain Intent / Commands
- selection
- undo / redo
- renderer
- timeline model
- Rust bridge
- persisted project data

を共有し、Classic UI / New UI のどちらからでも同じDocumentを操作できる構造にします。

UIを2つにするためにDomain logicを複製しないでください。

既存UIの見た目を再現するためのコードも、今回削除しません。

## 1. コードを書く前に Capability Inventory を作る

最初に現行Flutter UIをコードと実窓の両方から監査してください。

Browser / Stage / Inspector / Timeline / Transport / Fonts / Effects /
Media / Composition / Settings / Context menus / Toolbar /
Markers / Depth / Camera / World / etc. を含め、
ユーザーが現在到達可能な操作を可能な限り全件列挙してください。

単にpanel名を書くのではなく、実際のcapability単位にします。

このinventoryが完成するまでNew UIを実装しないでください。

## 2. Inventory各項目に migration disposition を付ける

DELETE は今回の選択肢にありません。

- PRESERVE — 新UIでもほぼ同じ場所・表現で残す。
- MOVE — 機能はそのまま、より自然な場所へ移す。
- MERGE — 複数の既存UIを、一つの意味のあるUIへ統合する。
- CONTEXTUALIZE — 常時表示をやめ、対象を選択した時など必要な時だけ表示する。
- VISUALIZE — 数値入力は残すが、より情報量の高いvisual representationをprimaryにする。

例:

- Position X/Y → PRESERVE + VISUALIZE(exact numeric inputを残しつつStage handleでも操作)
- Falloff → PRESERVE + VISUALIZE(numeric precisionを残しつつ将来的に2D gadgetへ)
- Fonts → PRESERVE + MOVE + CONTEXTUALIZE(Text選択時にアクセスしやすくする案を検討)
- Files → PRESERVE + MOVE(常時巨大なFinder cloneである必要はないが、全機能は残す)

既存機能の到達経路が新UIから消える案は不合格です。

## 3. 新UIのVisual North Star

添付したConcept Artをvisual referenceとして使用する。pixel-perfect targetではない。

Concept Artに存在するが現行Motoliiに存在しない機能は、勝手に実装しない。特に、

- Relations
- EDIT / PLAY / EXPORTという新mode
- 新しいGenerator semantics
- 新しいInspector semantics

等は future concept の可能性がある。「画像に描いてあるから」という理由だけでDomainへ追加しないこと。
Concept Artから採用したいのは主にvisual grammar。

## 4. Visual direction

Ableton × Teenage Engineering OP-1 × Swiss design × Flat Pop。ただし、どれかのコピーにはしない。

**Base** — 動画制作ソフトなのでdarkを第一案とする。near-black / charcoalを基調にする。
全面を同じ黒で潰さず、thin divider / value / typographyでsurface hierarchyを作る。

**Geometry** — 基本は矩形。corner radius 0–3px程度。thin rules / dividers。grid alignment。
floating card・pill / capsuleを乱用しない。glassmorphism禁止。SaaS dashboard化禁止。
大きなrounded cardの入れ子禁止。「カードが並んでいる」のではなく、
「一枚の機械の面に機能が組み込まれている」ようにする。

**Color** — Base: near black / charcoal / neutral grey / off-white。
Semantic accent: pink / mint / sky blue / lemon / peach・orange / lavender。
Flat Popな彩度を持たせる。ただし色はdecorativeではなく、
selection / category / relation / track / stateなど意味のある箇所へ限定する。

**Typography** — Swiss / instrument-like。hierarchyを文字サイズだけでなく
weight / alignment / spacingで作る。navigationや短いmode labelはuppercase可。
numeric readout / short technical labelはmonoを検証。bodyまで無理にmonoにしない。
tabular numbersを優先。小さい文字でも明確に読めること。

**Density** — 情報量を減らして「モダン」にしない。これは動画制作ソフト。
現在の高密度性を維持しながら、alignment / hierarchy / visual groupingによって読みやすくする。

## 5. 新UIで特に再検討してよいもの

Browser / Files / Fonts / Effects / Media / Inspector sections / toolbar / tabs /
panel headers / Stage controls / auxiliary panels / property placement は
位置・表示方法を変更して構わない。ただしcapabilityは残す。

## 6. 比較的安定している領域

- Stage — 作品を見る・選択する・spatial manipulationを行う
- Timeline — 時間を見る・layer/objectの時間的存在を見る・keyframe / marker / playhead等を扱う
- Inspector — 選択対象の意味・状態を編集する
- Browser — 追加可能なもの / assetへ到達する

「責任」が安定しているだけで、panel位置やvisual treatmentを固定する意味ではない。
Concept Artの四面構成は有力な基準案として扱う。

## 7. Relation Gadgetについて

今回のmigrationで、新しいRelation機能そのものを実装しない。ただし今後のUI languageとして

> visual representation itself can become the control surface

という原則を壊さない設計にする。将来的には
Falloff → field、Scatter → point distribution、Along Path → path + marks、
Stagger → space/time slope、Curve → curve、Gradient → gradient、Audio → spectrum
のように、「数値rowの集合」ではなく「現象を表した2D gadget + precision values」を
Inspectorへ置ける余地を残す。VST / FabFilter Pro-Q / Ableton Envelope等の
visual-first, precision-secondを参考にする。ただし今回は未来機能をfakeしない。

## 8. Flutter implementation policy

見た目を各Widgetへhardcodeして終わらせない。まず現在の
ThemeData / ColorScheme / TextTheme / ThemeExtension / shared controls / panel primitives /
hardcoded colors / padding / borders / text styles を監査する。

New UIのvisual tokensは可能な限り共通化する。ただしarchitecture cleanupを目的にした
大規模refactorは禁止。UI migrationに必要な範囲だけ変更する。

Widgetを再配置・包み直す場合も keys / focus / semantics / gestures / drag/drop /
keyboard shortcuts / state / undo/redo / selection を壊さない。

Domain logicをFlutter Widgetへ移さない。
Rust / renderer / Document schemaは、visual migrationの都合で変更しない。

## 9. 実装順

**Phase A — Audit**(コード変更なし)

1. Current UI screenshot
2. Capability Inventory
3. Widget / panel ownership map
4. visual hardcode audit
5. migration disposition
6. Classic UIを保存する最小境界の確認

ここまでをreportにする。

**Phase B — New shell skeleton** — 同じDocumentを使ってNew UI shellを作る。
まだ全機能を美しくしなくてよい。Capability Inventoryの全項目について
「新UIではどこから到達する予定か」が追えること。Classic UIは残す。

**Phase C — Visual system** — dark surfaces / typography / dividers / tabs / buttons /
inputs / sliders / selection / focus / semantic colors / timeline styling を
共通token/componentとして適用。

**Phase D — migrate one complete workflow** — 一つの既存fixtureを使い、
open/import → select → edit → timeline → preview → undo/redo → save/export
までNew UIだけで完走する。その後に残りを移す。

## 10. 比較方法

Classic UIとNew UIを same document / same time / same window size / same selection
で並べて撮影する。見た目だけでなく、Capability Inventoryを使って機能差を比較する。
新UIに存在しないcapabilityがあれば未完成。

## 11. PASS

- Classic UIがそのまま起動できる
- New UIでも既存capabilityを失わない
- Document / renderer / commandsを共有している
- panel位置は必要に応じて再設計されている
- prototype/debug-tool感が明確に減っている
- Stageが作品制作の主役である
- darkでもFlat Pop accentが濁らない
- Ableton / OP-1的な道具感がある
- Swiss的なalignment / typography / ruleがある
- high-density creative softwareのままである
- skin/tokenがFlutter上で維持可能
- Classicとのbehavior差が説明できる

## 12. FAIL

- Concept Artに合わせるため機能を削った
- 現行UIのcapabilityを見落とした
- Classic UIを上書きした
- Domain logicを二重実装した
- 新UI専用Document semanticsを勝手に追加した
- rounded-card dashboardになった
- SaaS/mobile app風になった
- whitespaceを増やして制作情報量を減らした
- visual変更のためRust rendererを変更した
- mockだけ綺麗で実操作が通らない
- 「実装しやすいから」で現行layoutをそのまま採用した

## 13. 自走ルール

Phase Aは判断を仰がず最後まで行う。既存capabilityの保存、Flutterのvisual token化、
Classic UIの保持については自走してよい。不明点があっても、まずコードと実窓から事実を確認する。

ユーザー判断が必要なのは、

- capability同士が本当に意味的に重複しており統合で挙動が変わる
- 新UIのためにDocument / Domain Intentを変える必要がある
- ClassicとNewで同じ操作に異なる意味を持たせる必要がある

場合だけ。単なる色、余白、罫線、typography、control styling、panel内の視覚配置については逐一止まらない。

最終的に欲しいのは「新しい絵を描いた」という報告ではない。
**現行Motoliiの全能力を保持したまま、同じ製品を新しいUI projectionから操作できる状態**
を作るためのrebaseline。
