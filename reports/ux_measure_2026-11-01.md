# 細かい UX の測り方（11/1）

ユーザーの問い：「細かい UX はどうやったら測れるのかな。さすがに手動のままだったら無理」。総当たりは「エフェクトが 1000 種類ある」ので筋が悪い。「要はやっぱりゴルフの試験をやるしかないってことか」。

## 結論

単位はゴルフのまま。手で打つのをやめて、機械が打って機械が数える。

- 総当たりは掛け算で増える。ゴルフは「やりたい事」の数だけで、今 24 ホール（reports/scenarios.md の P01〜P24）。効果が増えてもホールは増えない。
- 機械だけでは測れない物が 2 つ残る。「ここだろう」と押して当たるかと、見た目の気持ち悪さ。

| 測る物 | 誰が測る | 合格線 | 出典 |
|---|---|---|---|
| 打数：1 ホールの手数 | 機械 | パー以下。パーは公式説明書の手順の数 | golf_noise.md の決め方 |
| 所要時間の見積もり | 機械 | 前の版より長くならない | KLM、Fitts の法則 |
| 打率：最初の 1 手で当たるか | 初見の採点役 | 外した所を直す | Bailey と Wolfson の first click |
| 押してから絵が変わるまで | 機械（実物の窓） | ドラッグ 55 ms、タップ 96 ms | Deber ほか（トラックパッド） |
| 気持ち悪さ | ユーザーの目 | 無い | Ivory と Hearst：満足は機械で予測しにくい |

## 効果 1000 種類の扱い

数で割る。1 本ずつ試さない。

- 数えた物：`.fs` 420、`.wgsl` 168、`.am` 20、AviUtl の台本 269。計 877。
- 触る部品は少ない：カードは `EffectCard` の 1 種類。ISF のつまみの型は 9 種（float、image、bool、long、color、point2D、event、audioFFT、audio）。WGSL は `param_infer.dart` の見え方 13 種に振られる。
- 同じ部品なら代表 1 つで足りる（同値分割、Myers）。組み合わせは少数の要素で起きる物がほとんど（Kuhn ほか）なので、掛け算は要らない。
- 「その効果の絵が正しいか」は UX ではない。別の検査（integration_test/alight_check_test.dart）の仕事。

## 手元にある物

| 物 | 測る事 | 状態 |
|---|---|---|
| golf_noise.md | 手で打った打数と打率 | 手作業 |
| integration_test/firstclick_test.dart | 初見の 1 手目 | 絵と課題を作る。採点の grade.py は今は無い |
| integration_test/frames_test.dart | 窓の 1 枚の時間 | CI に入っていない |
| integration_test/rampage_test.dart | 400 手のランダム | 総当たりではない。11/1 は途中で窓を閉じ、点は出ていない |

## 次に作る物

1. 打数を数える口。テスト側の手順に手を入れず、窓に届いた押す・離す・キーを 1 か所で数える（Flutter の `PointerRouter.addGlobalRoute` と `HardwareKeyboard.addHandler`。試していない）。
2. ホールを 1 つ機械で打つ。出来ている統合テストの中から、今の手順がそのままホールになる物を使う。
3. パーの表。各ホールに、出典の説明書の手順の数を書く。

## 測れない事

- 機械が打つのは書いた道順。道が見つかるかは測れない。打率は別に要る。
- KLM は慣れた人が間違えない前提で、CogTool の見積もりが実測から外れた報告もある。絶対値で見ず、前の版との比較に使う。
- Deber の数字は論文本体を読めず、引用した論文 2 本からの孫引き。トラックパッドの値で、マウスの値ではない。
- 窓の 1 枚の時間は測れているが、押してから画面に出るまでは測っていない。
- Kuhn の割合（2 要素で 70〜95%）は NIST の記事の要約。論文本体では確かめていない。

## 出典

- [Ivory & Hearst, The State of the Art in Automating Usability Evaluation of User Interfaces](https://webtango.berkeley.edu/papers/ue-survey/p470-ivory.pdf)
- [John & Prevas, Predictive Human Performance Modeling Made Easy (CHI 2004)](https://hci.rwth-aachen.de/materials/conferences/CHI2004/1p455.pdf)
- [KLM の操作の時間（Wikipedia の GOMS）](https://en.wikipedia.org/wiki/GOMS)
- [CogTool の見積もりと実測の比較（LSU の修士論文）](https://digitalcommons.lsu.edu/gradschool_theses/1401)
- [Deber ほか CHI 2015 の所在](https://www.dgp.toronto.edu/?p=697)
- [Deber の数字を引いた論文](https://arxiv.org/pdf/2408.02525)
- [Nielsen, Response Time Limits](https://www.nngroup.com/articles/response-times-3-important-limits/)
- [MeasuringU の first click の再現](https://measuringu.com/?p=36842)
- [Myers 4 章の要約（同値分割）](https://www.cs.purdue.edu/homes/bxd/307/MyersChap04.html)
- [Kuhn, Wallace & Gallo, Software Fault Interactions](https://csrc.nist.gov/pubs/journal/2004/06/software-fault-interactions-and-implications-for-s/final)
- [NIST の組み合わせ試験の記事](https://www.nist.gov/news-events/news/2007/12/combinatorial-approach-squashes-software-bugs-faster-cheaper)
- [UXAgent（LLM で模擬する使い勝手の試験）](https://arxiv.org/pdf/2502.12561)
- [Quickstrom（規則で書く UI の試験）](https://arxiv.org/abs/2203.11532v1)
- [Blouin ほか、GUI の不具合の分類（自作部品は道具の判定が届かない）](https://arxiv.org/pdf/1703.09567)
- [Flutter の meetsGuideline](https://api.flutter.dev/flutter/flutter_test/meetsGuideline.html)
- [Flutter の性能測定](https://docs.flutter.dev/cookbook/testing/integration/profiling)
- [WCAG 2.5.8 Target Size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html)
