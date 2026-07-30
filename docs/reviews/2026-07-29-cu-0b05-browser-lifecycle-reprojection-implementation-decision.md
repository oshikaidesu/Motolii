# CU-0B05 Browser lifecycle再投影実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 前提: `CU-0B03`、`CU-0B04N`、`CU-0B04R`、`CU-0B05S`

## 1. 実装

通常project sessionのdirect product Hostへ、Browser lifecycle callbackと
event-loop上の再生成coordinatorを接続した。

- reloadまたはWeb content process terminationを発生instance epoch付きのtyped eventへする。
- old epochの遅延eventを無視し、old WebViewをdropしてからnew epochのWebViewをbuildする。
- Rectangle source、latest layout epoch / bounds、Host requested focus targetを再投影する。
- Web senderとRust sessionのsequenceはnew instance内で0から始め、同一instanceでは巻き戻さない。
- process自動回復はHost lifetime中1回。二回目はBrowserだけをdropしてdegradedへ止める。
- callback内のDocument編集、D2配送、WebView再生成は0。再生成はproduct event loopだけが行う。

React source、DOM/CSS、Browser wire、公開API、Document、selection、history、journalは変更していない。
resizeは再生成せず、既存latest layout epoch経路でboundsだけを更新する。

## 2. 自動審判

- replacement sessionはstable sourceを維持し、old epochを拒否してnew epoch sequence 1だけを受理する。
- initial loadの`Started`をreload扱いせず、live projection後のreloadだけを一回報告する。
- reloadごとにepochを進め、old callbackを無視する。
- process terminationの自動回復を一回に制限し、二回目をdegradedにする。
- source検査でdrop-before-build、stable source、latest layout再適用を固定する。
- lifecycle実装から`DocumentWriter`、edit queue、apply / Undo / Redoへの到達がないことを固定する。

## 3. 実Mac証跡

MacBook内蔵画面 / 100% / 暗い室内の`MotoliiNativeProduct.app`で確認した。

1. appと同時刻に起動した専用WebContent processだけを終了した。
2. app、native Stage、Timelineを維持したまま新WebContent processが生成され、Browserが再表示された。
3. 回復後のRectangle dragでBrowserからnative parentへfocusが移り、Place開始配送が再成立した。
4. 同じHost lifetimeで新WebContent processをもう一度終了すると、Browserだけが消え、
   native Stage / Timelineとapp本体は生存した。

他appのWebKit process、Document、selection、history、journalは操作していない。

## 4. 非目標

RectangleのD2 commit、Timeline / InspectorのDocument projection、Undo / Redo、利用者向け
Browser再試行UI、token後続、plugin UI公開契約は本粒へ含めない。本粒単体を
Motolii Studio Preview完成とは扱わない。

## 5. 次

`CU-0B05`を`DONE`とする。次PRODUCT-ASSET `DO`は`CU-107PV`。
通常製品Hostの一active dragに非空虚なpreview phaseを接続し、previewからterminalを
生成しない一成果だけを閉じる。
