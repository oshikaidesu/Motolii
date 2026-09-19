# UI の重さの法 — 導出は build の外、選択は信号、押下は arena の下

2026-09-12 利用者裁定。きっかけは「ブラウザパネルの選択が遅すぎてダブルクリックが間に合わない」。
1 クリックで 3000 行の State が丸ごと build し直し、その中で素材全件の絞り込みと base64 の復号を毎回やっていた。
利用者の指示: **せっかくビューが超軽量なのに他が足を引っ張るのは本末転倒。他のバカも直し、法則を書き出す。**

なぜ通ったか(Flutter のせいではない): (1) 各レーンが自分の機能を build() の中に一段足す形で入れ、誰もパネルの形を持っていなかった。
(2) 理由をコードのコメントで正当化して通した(「数百なら 1ms」「temp」)。(3) 合否の定規が「表示されるか」だけで、素材 300 で 1 クリックに何が走るかを測る定規が無かった。

## 法

1. **導出は build の外。** 一覧・並び・絞り込み・重ね合わせ(`visible`、`_LaneLayout`、`_state`)は入力が変わった所で 1 回組み、build は読むだけ。
   `StatefulWidget` の build に置いた `ValueListenableBuilder` は親の `setState` ごとに builder が走り直す(値が同じでも)。slice は `initState` で listen する。
2. **選択・ホバー・押下中は信号。** `ValueNotifier` にして、見る側(タイル・行・件数)が自分の出入りだけで描き直す(browser の `_Picked`)。パネルの `setState` は入力の変化にだけ使う。
3. **押下は arena の下。** `onTapDown` は同じ detector に `onDoubleTap`/`onLongPress` が居ると 100ms の押下期限の後に届く。「押した瞬間に選ぶ」は `Listener.onPointerDown`(主ボタン)。
4. **描く側は field 比較。** `shouldRepaint => true` は書かない。描くたびの割り当て(`TextPainter().layout()`、sort、`jsonEncode`、map コピー)は relane/derive の時に済ませ、paint は読むだけ。
   比較は `identical` → 駄目なら `sameValue`(割り当て無しの深い比較)。`jsonEncode` や `toString()` で比べない。
   **導出した入れ物の identity は指紋にならない**: 入力が動けば導出は新しい list を返すが中身は同じことがある(層を動かしても選んだキーは空のまま)。描き直すか決める指紋は導出の中身で比べる(ease の `_read` で 0 → 417 builds に跳ねて定規が止めた)。
5. **UI thread のイベント経路に同期 IO を置かない。** 診断ログ(`DIAG(temp)`)は commit しない。フォルダの `listSync` は押した時の 1 回に限る。
6. **定規が先。** [panel_layout_cost_test.dart](../../motolii/ui/test/panel_layout_cost_test.dart): 状態更新 1 回・全体 layout 1 回・選択変化 1 回・**素材 500 で 1 クリック**(押したその frame に枠が出る、builds ≤ 100 / layouts ≤ 20)。予算は測った数で置き、上げるときは理由を予算のコメントに。全パネルを印字してから判定する(1 回の走行で赤が全部並ぶ)。
7. **理由をコードのコメントで正当化しない。** 「数百なら速い」「temp」は定規で示す。示せないなら書かない。

## CPU反復を増やさない(2026-09-19)

利用者裁定「なぜ毎秒CPUを動かすという思考になる。そのコードを書いた時点で罰則を出す。ルールの制定」。対象はFlutterだけでなくRust・FFI・描画準備を含む毎秒・毎フレームのCPU経路。

1. **何も変わらなければ、アプリ由来の再計算・再構築・ポーリングは0回。** 周期で状態を聞く設計を既定にせず、変更通知で必要な箇所だけを起こす。OS/Flutter自身のイベント処理までCPU使用率0%を要求する規則ではない。
2. **時刻が進むことは全体を組み直す理由にならない。** 本当に時間依存の入力だけを更新する。不変の層・レイアウト・一覧・資源は保持し、無関係なパネルへ再生の更新を配らない。入力処理や必要なCPU評価も、影響範囲を限定する。
3. **GPU/engineが持つ計算を、CPUの毎コマ処理へ持ち出さない。** 描画・時間変化は既存のGPU/engineの口を先に調べる。既存の仕組みで済む物を、独自のCPU loop・全件走査・GPU readbackで作り直さない。GPU上で処理するだけでも全体再構築や不要な転送の免罪にはならない。
4. **反復経路を足す前の証拠は必須。** Timer/Ticker、frame callback、継続poll、毎コマの走査・導出・割り当て・転送を新設/増加する変更は、(a)どの入力が変わるか、(b)開始・停止条件と対象範囲、(c)既存GPU処理やイベント通知で代替できない理由、(d)同一書類・同一操作・同一build条件の前後実測をレビューに載せる。静止と再生、初回と定常を混ぜず、呼び出し回数と中央値/p90/最大を残す。実装前に比較条件を決め、実装後に結果を埋める。
5. **罰則は変更の差し戻し。** 上記を満たさない変更は、動く・コンパイルできる・テストが緑でも採用不可。違反した経路を除去/修正し、同じ条件で再検証するまで、その仕事を完了と報告しない。自分の該当差分だけを対象にし、他者の変更を巻き戻さない。既存コードも発見時に違反として記録し、新規実装の先例にしない。
6. **迂回で合格にしない。** 同じ無駄を別thread/非同期へ移すだけ、常時pollを別の時計に置き換えるだけ、計測を削るだけでは是正にならない。cacheには入力の鍵・無効化・保持量の上限が必要。画質・解像度・操作や絵の一致・機能を黙って落として予算に入れることも不可。

検収は、既存のlayout/build回数試験と実行計測へ、違反を再現するケースを追加する。たとえば不変入力の導出0回、選択変更で無関係な棚の再構築0回、対象素材なしの先読み評価0回。数値予算を上げたり、既存の失敗に混ぜて通したことにしない。必要な再生クロック自体を一律に禁止する静的lintでは、仕事の中身や停止条件までは証明できない。

**制定時点ではレビュー上の拒否条件。自動検出ゲートを新設したという意味ではない。** 自動化する場合は再現ケースと実測可能な違反を既存の検査口へ加え、偽陽性と取りこぼしを示す。

外部の定規: [FlutterのUI/Rasterの分担](https://docs.flutter.dev/tools/devtools/performance)、[描画までの段階](https://docs.flutter.dev/resources/architectural-overview)、[Flutter GPUの役割](https://api.flutter.dev/flutter/flutter_gpu/)。いずれも「GPU描画ならCPUでの準備も無償」「Rustなら反復しても速い」という保証ではない。

## 実装(2026-09-12)

- browser: [browser.dart](../../motolii/ui/lib/panels/browser.dart) `_derive` / `_publish` / `_Picked` / `Listener`、data: URI の復号は 1 回
- timeline: `_relane()` を `LayoutBuilder` から出す、painter 2 つの `shouldRepaint`、行ラベルの `TextPainter` と `summaryFrames` を使い回す
- stage: `_state`(文書 + 描画フレーム)を (document, rendered) ごとに 1 回、`_layers` も
- fonts: 並びは `fontFamilies` が変わった時だけ、標本キーは build ごと 1 回
- panel_controls: 数値欄の全イベントで `/tmp` へ書いていた `DIAG(temp)` を撤去。`Picked<T>` をここへ(browser・notes・inspector の fold が共用)
- dock: [workspace_view.dart](../../motolii/ui/lib/workspace/workspace_view.dart) を `_Split` / `_Leaf` に。仕切りのドラッグとタブ切替はその node の状態、渡すパネルは同じ object(全体 layout 867 → 626、ring の押下 80 → 37 builds)
- inspector: `deskWork` の `AnimatedBuilder` を「cell 幅と Animate 既定だけ比べる」listener に、`_targets` を (gesture, document, frame) ごと 1 回、行の平坦化と `_characterOf` は `Expando`、Advanced の fold は `Picked`
- ease: `_segments` を (selectedKeys, layers) ごと 1 回、`_read` の指紋は中身比較、preset の `jsonEncode` は build ごと 1 回ずつ、ラベル高さは幅ごと 1 回、painter 3 つは `sameValue`
- timeline 概観の押下は `Listener`、notes の card 選択は `Picked`、rich_text_editor は run ごとに 1 つの `TextStyle`、stage の overlay は `listEquals`/`mapEquals`、blend は文書をコピーしない
- 残り(小): ease のタイル hover はまだパネルの `setState`(graph が `_audition` を読む)、stage の marquee は 1 move ごと `setState`、depth_desk・gradient_inspector・history_records・export_controls の小さな導出
