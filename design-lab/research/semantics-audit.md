# Semantics 監査(使い捨て widget test の結果)

目的: 「盲目の人が触って分かる」を物差しにして、6 つの代表 panel が screen reader にどう見えるかを数えた。直してはいない。
方法: 6 component の全 use case(計 33)を Widgetbook の preview 経路で実際に pump し、semantics を有効にして数えた。空の use case を基準として差し引いてあるので、Widgetbook 自身の部品は混ざらない(ただし Scrollable 等の framework 内部の GestureDetector が少し残る)。test は削除済み。

## 数え方
- 意味木の操作可能ノード: button / text field / toggle / checked / tap / increase / decrease を持つ node の数。うち label・value・hint が全部空の物が「空」。
- ポインタ入口: onTap / onTapDown / onPan* / onDoubleTap / onSecondaryTap / onLongPress / drag 系のどれかを持つ GestureDetector と、onPointerDown を持つ Listener の数。うち、その入口の意味ノード(無ければ最も近い親ノード)に label・value・hint が無い物が「名無し」。
- 重要: 空ラベル 0 でも安心ではない。Text を子に持つボタンは Text の文字が label になるだけで、Words = Hidden にすると意味が消える UI は多い(label が「見える文字」の使い回しで、操作の意味ではない)。

## 結果(use case をまたいだ合計)
| panel | use case | 意味木の操作可能 | うち空ラベル | ポインタ入口 | うち名無し | 名無し率 |
|---|---|---|---|---|---|---|
| Inspector I2 Number | 5 | 104 | 3 | 329 | 40 | 12% |
| Inspector I4 Animate | 6 | 234 | 106 | 552 | 280 | 51% |
| Browser B3 Find | 9 | 147 | 0 | 379 | 64 | 17% |
| Browser B2 Tag bands | 4 | 151 | 0 | 307 | 14 | 5% |
| Browser B9 Swap | 6 | 153 | 0 | 361 | 20 | 6% |
| Inspector I9 Whip | 3 | 47 | 0 | 170 | 60 | 35% |
| 計 | 33 | 836 | 109 | 2098 | 478 | 23% |

## 最悪の犯人(名無しの入口が多い順、同じ型が use case ごとに繰り返し数えられている)
1. I4 Animate: `_Ctl`(汎用の hover/focus/press 外殻)170、`_Lane`(行内の時間線)46、`_Playhead` 14、`_AnimateBar` 12。鍵のダイヤ・スイッチ・時間線は label 無しの図形入口で、意味ノード上でも空 106 件。最悪の panel。
2. I9 Whip: `_Ring`(つなぎの輪)52。pick-whip の入口が輪そのもので、何に繋ぐ物かが木に無い。
3. I2 Number: `_Ground` 15、`_Cell` 12(数値セルの掴み面)。値面 = 掴む/打つ/既定へ戻すの三役だが、木は「タップできる」だけで、何の値かは周りの Text 頼み。空 3。
4. B3 Find: `_H`(hover 外殻)32、`_FadeScroll` 11。結果行の一部。
5. B2 / B9: 少ない(名無し 14 / 20)。チップや行は文字がそのまま label になっているため。ただし文字が label の元なので Words = Hidden で同じ画面を見ると意味の手がかりが無くなる点は別問題(目で確認すること)。

## 読み取れること
- 空の label は「図形だけの入口」(ダイヤ・輪・時間線・つまみ)に集中している。これは lab の panel が「文字で説明する」部品と「図形で操作する」部品に分かれており、後者には semantics の宣言が元から無いことを示す。
- 掴む/ドラッグする系(onPan*)は意味木に載りにくく、increase / decrease の action も付いていない。スライダ相当の物に Slider の意味(値・範囲・増減)が無い。
- 直す時の最小の手は、`_Ctl` のような共通外殻に label を必須引数にすること(1 か所で I4 の 170 件が動く)。今回は直していない。

## 道具
- Words addon(Full | Hidden): 目で「文字が無いと分かるか」を見る。
- A11y addon(Off | Show semantics labels): 上の数字が何であるかを画面上に重ねて見る。
