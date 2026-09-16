# Color selection flow — 実窓所見

状態: **観察 / 未修正**。2026-09-13 の macOS 実窓で、Shape の Fill を起点に Colors、HEX、色相面、保存色、Fill mode、gradient stop、スポイト、Undo を順に操作した結果。

設計の正本は [decision-index](../decision-index.md) の「色 property」「色の原子」。この文書は採用案ではなく、現在の操作で確認した欠陥と再現境界を残す。

## 確認した問題

| ID | 重大度 | 種別 | 問題 | 再現と根拠 | 影響 |
|---|---:|---|---|---|---|
| COLOR-FLOW-1 | P0 | 実窓観察 | HEX のエラーから別操作へ移るとアプリが落ちる | Colors の HEX に `#ggg` → Return → Escape → 色見本を押す。`motolii_stage5-2026-09-13-203738.ips` は `EXC_BAD_ACCESS / SIGSEGV`、faulting thread は `flutter::AccessibilityBridge::CreateRemoveReparentedNodesUpdate()`。同日のカラー操作中に同型が再発 | 編集を続けられず、未保存作業を失う |
| COLOR-FLOW-2 | P1 | 実窓観察 | Fill mode / Blend の札が色見本に見える | HEX 直下の無記名タイル右端を色見本のつもりで押すと、Solid が Diamond gradient になり、Inspector に Angle / Center / Spread が現れた | 色を選ぶ操作で構造を変更してしまう |
| COLOR-FLOW-3 | P1 | 実窓観察 + コード照合 | Gradient を Solid に戻すと現在見えている stop 色を失う | 左 stop をスポイトで青にした後に Solid を押すと、青ではなく元の緑へ戻った。`set_shape_gradient` は評価された stop property ではなく authored brush の端を読む | キー・式・現在時刻の色と確定結果が一致しない |
| COLOR-FLOW-4 | P1 | 利用者報告 + コード照合 | 単色 Fill の帯が色編集の入口ではなく gradient 分割になる | 利用者報告「普通に Fill 部分の色を押すと split」。`GradientInspector` は solid / gradient を分けず bar の `onTapUp` から `addStop` を呼ぶ | 最も自然な色選択操作が、意図しない構造変更になる |
| COLOR-FLOW-5 | P2 | 実窓観察 | stop 操作が Browser と Inspector の二系統に見える | Inspector の bar は Document の stop。Browser の `+` は別の swatch 草稿で、押すと縦帯が現れるが Fill は変化しない | どちらが作品を編集するのか判断できない |
| COLOR-FLOW-6 | P2 | 実窓観察 | Browser の stop 草稿は Document Undo と別に残る | Fill の変更を Undo して Solid に戻しても Browser の縦帯は残り、右クリックで別途消す必要があった | Undo 後も画面が変更前の状態へ戻らないように見える |
| COLOR-FLOW-7 | P2 | 実窓観察 | 無効な HEX の理由が読めない | `#ggg` は適用されないが、見えるのは弱い枠変化だけ。エラー文は tooltip の message にあり、現在の窓は tooltip 非表示 | 修正方法が分からず、入力欄に留まる |
| COLOR-FLOW-8 | P2 | 実窓観察 | スポイトの待機状態が見えない | スポイトを押しても外観の差が弱く、次の Stage click で即確定した | いつ通常選択へ戻ったか、Esc で取消せるか判断しづらい |
| COLOR-FLOW-9 | P3 | 実窓観察 | Square / Triangle の現在状態と切替結果が読みにくい | HEX 横の小さな図形だけが切替入口。常時見える語はない | 現在の入力方式を記憶に頼る |
| COLOR-FLOW-10 | P3 | 実窓観察 | Saved / Starter の空状態が次の行動を示さない | rail を開くと `No matches` だけ。保存入口は Browser の無記名 stop 草稿の末尾にある | 最初の色・gradientを保存する方法へ到達しづらい |

## 問題ではなかったもの

- 三角形で白へ近づくと赤へ飛ぶ件は別途修正済み。ドラッグ開始時の hue を保持し、三角形外の座標を辺へ射影する。白へ下げて戻しても同じ hue へ戻ることを実窓と `color_wheel_test.dart` で確認した。
- 色相面・HEX・スポイトからの通常変更は Stage と Inspector に反映され、通常の Undo は働いた。
- Colors 上部の `Layer name · Fill / Fill · stop` は、現在の書き込み先を識別する材料になっている。

## 検収条件

1. Solid の Fill bar を押しても stop 数と Fill modeを変えず、Colors がその Fill を向く。
2. gradient の空所を押した時だけ stop が増える。既存 stop を押す操作とは分かれる。
3. Fill mode と Blend は、色見本や保存色とは異なる操作として常時判別できる。
4. gradient → Solid は、その時刻に見えている採用 stop の色を保つ。キーのある stop でも同じ。
5. 作品の stop と swatch 作成を同じ外観で二重に持たない。Undo 後の表示も作品状態と矛盾しない。
6. HEX の不正理由とスポイト待機を、tooltip に依存せず読める。
7. HEX invalid → Escape → swatch、連続drag、選択変更、focus loss を繰り返しても native crash しない。

## 未確認

- Text / effect color の alpha と同じフロー。
- detached Colors window での再現。
- From image のOSファイル選択後から保存まで。
- stop が32個ある場合の密度、キーボード選択、削除。

