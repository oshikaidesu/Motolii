# 掴む試験の必須要件

Motolii の欠陥はほとんど「掴む」の周りで出る。手当たり次第に触るのではなく、
**外の規格が必須として挙げている観点**を持ってきて、それを表にする。

出典: [W3C Pointer Events Level 3](https://www.w3.org/TR/pointerevents3/)(2026-06-30 勧告)、
[web-platform-tests / pointerevents](https://github.com/web-platform-tests/wpt/tree/master/pointerevents)、
[UIKit の認識器の状態](https://developer.apple.com/documentation/uikit/uigesturerecognizer/state/cancelled)、
[Android の touch slop](https://developer.android.com/develop/ui/views/touch-and-input/gestures/viewgroup)。

## 必須(規格が MUST / 必ず到達すると書いている物)

**1. 掴んだ物が、外へ出ても受け取り続ける(implicit pointer capture)**
> pointerdown で direct manipulation device なら、その pointerId をターゲットへ capture する

掴んだ後は、指が要素の外へ出ても**掴んだ物にイベントが行く**。離した時も。
*Motolii の状態:* **捕捉そのものは無い**が、届く範囲で代わりを置いた ——
指が上がっているのに掴んだままなら、**捨てずに確定する**(2026-09-01)。
離した所までの編集は残る。届かない所(窓の外へ出たまま離す)は塞げていない。

**2. 取り消しが必ず来る(pointercancel)**
規格が MUST で挙げる条件 —— モーダルや메뉴が開いた / 入力装置が物理的に外れた /
その指が**ビューポート操作(パン・ズーム)に使われた** / ドラッグ開始の手順に入った。
MAY —— 画面の向きが変わった、同時ポインタ数の上限、手のひら誤爬。
*Motolii の状態:* **入れた**(2026-09-01)。掴んでいる間の `Esc` は取り消し、
掴んでいない時だけ選択を解く。窓から離れた時(`onfocusout`)も取り消す。
書かずに手放すので、掴む前の値がそのまま残る。
※ 実窓での確かめは、道具が `Escape` を送れないため**別のキーへ一時的に
割り当てて**行った(経路は同じ)。

**3. 掴みは必ず閉じる**
> 認識器は Recognized / Failed / Cancelled / Ended のどれかに必ず到達する

宙に浮いたまま残る状態を作らない。
*Motolii の状態:* 確定・取り消し・閾値で落とす(不成立)が揃った。

**4. 押しただけでは動かない(touch slop)**
> 非移動のジェスチャ(タップ)が移動のジェスチャ(ドラッグ)になるまでに指が動ける距離

*Motolii の状態:* **入れた**(3px、Stage と Timeline の両方)。
それまでは 1px 動いた時点でドラッグになり、押しただけのつもりが値を変えていた。

**5. ボタンの重ね押し(chorded buttons)**
> 重ね押しでは pointerdown/up は重ならない。`button` と `buttons` の変化で見る。
> ドラッグ中の pointermove は `button` が -1

*Motolii の状態:* 未検査。掴んでいる最中に別のボタンを足す/離すを踏んでいない。

## 単一ポインタの窓では優先度が下がる物

**6. ポインタの同一性(pointerId / isPrimary)** —— 有効なポインタの id は一意。
2本目の指や2つ目の装置が来ても掴みが混ざらないこと。
デスクトップの単一ポインタでは当面出ない。

**7. 取りこぼしと束ね(coalesced / predicted events)** —— 速く動かした時に
間の点を取り戻す。Motolii は今、飛んだ間を直線で埋めている。

## 順序の試験(WPT が個別に持っている観点)

`pointerevent_sequence_at_implicit_release_on_click` / `_on_drag` ——
**離す時の順序**。捕捉の解放・離す・クリックがどの順で来るか。
`pointerevent_pointercancel_*` —— 取り消しの後に leave / out がどう続くか。

## Motolii の穴(この表を当てた結果)

| 要件 | 状態 |
|---|---|
| 1 捕捉 | 代わりを置いた。指が上がっていたら確定する(窓の外は塞げていない) |
| 2 取り消し | **入れた**。`Esc` と窓から離れた時 |
| 3 必ず閉じる | 確定・取り消し・閾値で落とす、が揃った |
| 4 閾値 | **入れた**(3px) |
| 5 重ね押し | 未検査 |
| 6 同一性 | 単一ポインタのため保留 |
| 7 束ね | 直線で埋めている |

**そして試験の土台が無い。** `src/ui/gui.rs` の harness は
**カスタム widget にイベントを1件も届けない**ので、上の1〜5はどれも
自動では踏めない。実窓で手で当てるしかなかった(それで13件出た)。
harness を widget まで届かせるのが、この表を回すための前提。
