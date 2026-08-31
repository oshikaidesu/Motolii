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
*Motolii の状態:* **無い。** 帯の外で離すと widget が離した事を知らず、
掴んだままの絵が残っていた(2026-09-01 に「指が上がっていたら解く」で塞いだが、
これは捕捉ではなく後始末で、**離した所までの編集は失われる**)。

**2. 取り消しが必ず来る(pointercancel)**
規格が MUST で挙げる条件 —— モーダルや메뉴が開いた / 入力装置が物理的に外れた /
その指が**ビューポート操作(パン・ズーム)に使われた** / ドラッグ開始の手順に入った。
MAY —— 画面の向きが変わった、同時ポインタ数の上限、手のひら誤爬。
*Motolii の状態:* **無い。** 窓が背面へ回る、別のパネルが前に出る、
`Esc` を押す —— どれでも掴みが取り消されない。

**3. 掴みは必ず閉じる**
> 認識器は Recognized / Failed / Cancelled / Ended のどれかに必ず到達する

宙に浮いたまま残る状態を作らない。
*Motolii の状態:* 部分的。確定と後始末は在るが、**取り消し**と**不成立**が無い。

**4. 押しただけでは動かない(touch slop)**
> 非移動のジェスチャ(タップ)が移動のジェスチャ(ドラッグ)になるまでに指が動ける距離

*Motolii の状態:* **閾値が無い。** 「動かしていないなら1画素も動かさない」は
入れたが、1px 動いた時点でドラッグになる。押しただけのつもりが値を変える。

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
| 1 捕捉 | **無い**。外で離すと編集が失われる |
| 2 取り消し | **無い**。背面・パネル切替・`Esc` で解けない |
| 3 必ず閉じる | 取り消しと不成立が無い |
| 4 閾値 | **無い**。1px でドラッグになる |
| 5 重ね押し | 未検査 |
| 6 同一性 | 単一ポインタのため保留 |
| 7 束ね | 直線で埋めている |

**そして試験の土台が無い。** `src/ui/gui.rs` の harness は
**カスタム widget にイベントを1件も届けない**ので、上の1〜5はどれも
自動では踏めない。実窓で手で当てるしかなかった(それで13件出た)。
harness を widget まで届かせるのが、この表を回すための前提。
