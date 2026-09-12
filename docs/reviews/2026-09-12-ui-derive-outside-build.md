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
