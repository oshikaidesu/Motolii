# Motolii — rerun を AE にするソフト

**失敗はテストへ。この文書と記憶は、足すとき同じ行数を消す。**

作ってよいのは**編集の意味**と**エフェクトのデータ**だけ。技術(GPU・デコード・
描画・色)は rerun の部品。**作る前に幹から逆引きする** — `reference/` の地図を引き、
幹が無ければ立てる(名前で探すと外れる)。**家は5つ — doc・render・ui・vism・tests。足すなら消す。**

- 編集状態は Document が持つ。書き込みは Intent 経由のみ
- 見た目の合否は利用者が窓で決める。cargo の緑は合格ではない
- 決めたことの記録は `../docs/decision-index.md` を主題で grep。
  コードの現状はコードとテストが正本 — 文書に書かない
- cargo は `motolii/` の中から回す

窓: `target/debug/motolii`。`MOTOLII_TESTDATA=<素材dir>` で Browser に素材が入る。
