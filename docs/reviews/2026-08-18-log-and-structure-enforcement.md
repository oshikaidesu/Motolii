# ログと構造の強制 — egui を iced の性質へ

日付: 2026-08-18
状態: **決定**(利用者裁定: 「ログと構造の強制が組めれば egui は iced になれる」)

## 裁定の意味

iced の構造的優位([調査](2026-08-18-iced-reentry-survey.md))は、フレームワークが
2つの規律を**強制**することに尽きる:

1. **ログ**: 全 UI 操作が型付き Message 列として残り、初期状態+列=任意時点を再現できる
2. **構造**: 状態変化は `update()` 経由でしか起きない(UI 層は入力→Message の翻訳だけ)

Motolii は toolkit を替えずに、この2つを**自分のフェンスで強制**する。柵で規律を
守らせるのは本リポジトリの実証済みの型である(eprintln 禁止フェンス・oracle 保護・
UI toolkit 依存 policy — すべて「破ると落ちるテスト」)。

## 施工の骨(不変量として)

1. **UiIntent journal(ログの強制)**
   - shell 層に型付き `UiIntent` を置き、利用者に見える状態変化は全てここを通る。
     D2 Command(document)は既に journal/replay を持つ — 足すのは
     view/shell 層(選択・座席・camera・panel 操作)の列
   - **replay oracle を常設**: DrivenShell で「記録した intent 列を replay →
     document revision・帯・選択が一致」。iced の time-travel に相当する検証を
     自前で持つ
   - ShellTranscript(結果のログ)と対になる「原因のログ」である
2. **単一ゲートウェイ(構造の強制)**
   - UI コードから `DocumentWriter`/製品状態への直接書き込みを、intent を記録する
     唯一の口の裏へ隠す
   - **フェンス**: (a) blitz_shell / panels から writer API を直接呼ぶ箇所の
     走査テスト(ゼロ化) (b) 対話 widget は「intent を返す」か「view_only と
     名乗る」かのどちらかであることの規約化。診断 F-03(M/S が local bool だけ)の
     型を構造で再発不能にする

## 診断 finding との接続

- D 類(F-07〜F-10: 黙殺)= **ログの強制**の欠け → transcript 合流+フェンス拡張
- B 類(F-03 M/S 見た目だけ・F-06 Stage 選択読み捨て)= **構造の強制**の欠け →
  intent 経由への結線。修正 wave はこの不変量の下で施工する
- 以後の新 UI は「intent を返さない対話面」をフェンスが赤で止める

## iced との関係

これが成立すると、iced の残る優位は「フレームワークが強制してくれる(自前フェンス
の維持費が要らない)」ことだけになる。toolkit 再入場トリガー
([運転席決定](2026-08-18-cli-gui-driver-seat.md))は不変 — 自前強制が破綻して
「繋がっていない」が散見された時こそ、フレームワーク強制へ乗り換える実測根拠になる。
