# Motolii — rerun を AE にするソフト

**失敗はテストへ。この文書と記憶は、足すとき同じ行数を消す。**

作ってよいのは**編集の意味**と**エフェクトのデータ**だけ。技術(GPU・デコード・描画・色・器具)は
rerun の部品 — **作る前に上流を探す**(取説 `docs/content/` → `reference/` の地図 → ソース)。
自前の天井は `reference/owned-budget.tsv`。**家は5つ — doc・render・ui・vism・tests。足すなら消す。**

- 編集状態は Document が持つ。書き込みは Intent 経由のみ
- 見た目の合否は利用者が窓で決める。cargo の緑は合格ではない
- **窓に出る文字は英語**。値が意味の物だけ文字、あとは形で見せる
- 決めたことの記録は `../docs/decision-index.md` を主題で grep。
  コードの現状は文書に書かない。**理由をコードに書かない** — 腐るし読めなくなる

窓: `target/debug/motolii`。`MOTOLII_TESTDATA=<素材dir>` で Browser に素材が入る。
