# Motolii — rerun を AE にするソフト

**失敗はテストへ。この文書と記憶は、足すとき同じ行数を消す。**

作ってよいのは**編集の意味**と**エフェクトのデータ**だけ。技術は rerun の部品。**新規コードと検証の前に、同じ意味・機構の参考文献・既存実装・外部の定規を必ず探す**(現行repo・既決 → 依存／上流の取説・`reference/`・ソース・oracle → 公式規格・一次資料・製品先例)。無ければ検索範囲を残し、自作testは借りた定規の写像だけにする。
自前の天井は `reference/owned-budget.tsv`、人の違和感の天井は `reference/hygiene-budget.tsv`(`scripts/check-hygiene.sh`、閾値は借り物)。**家は5つ — doc・render・ui・vism・tests。足すなら消す。build も家ごと**(`crates/motolii-doc` が契約、`motolii-render` が描く側、root の `motolii` が ui で hotpatch の tip。`motolii-road` は重い依存を 1 本の dylib に 1 回だけ link する道路)。

- 編集状態は Document が持つ。書き込みは Intent 経由のみ
- 責任は入口→意味→評価→結果→試験まで完結。後の統合・結線が要るなら未完。**並列の lane が同じ file を触る時点で失敗**(責任が集まっている印)。worktree で逃がさず、先にその file を data／manifest の口にして、以後は行を足すだけで載る形にする
- 1つ直せば同族全部が直るcomponent/dataにする。同じ意味を2箇所へ書いたら未完
- 開発窓は `../scripts/motolii-dx.sh serve` の固定版warm processを通常路にし、RSX／CSS／assetとRust関数・event logicは状態・GPU資源を保って差し替える。Computer Useはその`target/dx/.../Motolii.app`絶対pathを使う。**2026-09-04利用者の速度優先指示: 必要な初回・依存・型変更のbuildと意味の試験は途中で時間切れにせず完走させる。** cacheを保ち同じfeature/profileで進め、細かいtarget切替や確認儀式を増やさない。hot 5秒／初回60秒は改善目標であり強制停止条件ではない。full build／restartは原因・状態への影響・通常のhot経路へ戻す修正を一度記録し、普通の変更の失敗をbuild成功で帳消しにしない。無記録の自動full fallbackは止める。ログだけでなく実際の挙動と状態保持を検収し、見た目の採否は利用者が窓で決める。
- **窓に出る文字は英語**。値が意味の物だけ文字、あとは形で見せる
- 決定は `../docs/decision-index.md` を grep。コードの現状・理由はコードに書かない — 腐る

窓: `dx serve --hotpatch` が出す `target/dx/motolii/debug/macos/Motolii.app`。`MOTOLII_TESTDATA=<素材dir>` で Browser に素材が入る。
