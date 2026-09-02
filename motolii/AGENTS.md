# Motolii — rerun を AE にするソフト

**失敗はテストへ。この文書と記憶は、足すとき同じ行数を消す。**

作ってよいのは**編集の意味**と**エフェクトのデータ**だけ。技術は rerun の部品。**新規コードと検証の前に、同じ意味・機構の参考文献・既存実装・外部の定規を必ず探す**(現行repo・既決 → 依存／上流の取説・`reference/`・ソース・oracle → 公式規格・一次資料・製品先例)。無ければ検索範囲を残し、自作testは借りた定規の写像だけにする。
自前の天井は `reference/owned-budget.tsv`。**家は5つ — doc・render・ui・vism・tests。足すなら消す。**

- 編集状態は Document が持つ。書き込みは Intent 経由のみ
- 責任は入口→意味→評価→結果→試験まで完結。後の統合・結線が要るなら未完。**並列の lane が同じ file を触る時点で失敗**(責任が集まっている印)。worktree で逃がさず、先にその file を data／manifest の口にして、以後は行を足すだけで載る形にする
- 1つ直せば同族全部が直るcomponent/dataにする。同じ意味を2箇所へ書いたら未完
- 開発窓は `dx serve --hotpatch` の warm process を通常路にし、RSX／CSS／assetとRust関数・event logicはまず状態・GPU資源を保ったまま差し替える。Computer Useもその`target/dx/.../Motolii.app`絶対pathだけを指し、bundle IDで旧`target/debug`を起こさない。手動Cargo build／全suite／再起動は、実行前に `BUILD_NEED`(観測対象)・`EXTERNAL_RULER`(Dioxus／Cargo／platform一次資料)・`HOTPATCH_OR_CHECK_GAP`(代替不能理由)を示せなければ禁止。「締め」「念のため」「最終だから」は理由にならない。dx自身が要求する再compile、codegen／link固有の検証、実際に配布物を要求された時のbundleだけを外的要因で個別許可する。hot reloadから隠れた実装は迂回せず接続を直す。見た目の合否は利用者が窓で決め、Cargoの緑は合格ではない
- **窓に出る文字は英語**。値が意味の物だけ文字、あとは形で見せる
- 決定は `../docs/decision-index.md` を grep。コードの現状・理由はコードに書かない — 腐る

窓: `dx serve --hotpatch` が出す `target/dx/motolii/debug/macos/Motolii.app`。`MOTOLII_TESTDATA=<素材dir>` で Browser に素材が入る。
