# Motolii — rerun を AE にするソフト

作ってよいのは**編集の意味**(Document・レイヤー・キーフレーム・合成モード・
エフェクトの口)だけ。技術(GPU・デコード・描画・色)は rerun フォークの部品を使う。
上流に「無い」と思ったら、作る前に探し方を変える(別名・機能・feature 一覧)。

- 編集状態は Document が持つ。書き込みは Intent 経由のみ
- 見た目の合否は利用者が窓で決める。cargo の緑は合格ではない
- 決めたことの記録は `../docs/decision-index.md` を主題で grep。
  コードの現状はコードとテストが正本 — 文書に書かない
- cargo は `motolii/` の中から回す

窓: `target/debug/motolii`。`MOTOLII_TESTDATA=<素材dir>` で Browser に素材が入る
(未設定だと空)。
