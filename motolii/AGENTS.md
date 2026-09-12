# Motolii — rerun を AE にするソフト

**失敗はテストへ。この文書と記憶は、足すとき同じ行数を消す。**

作ってよいのは**編集の意味**と**エフェクトのデータ**だけ。描画は rerun、UI は Flutter の部品。**新規コードと検証の前に、同じ意味・機構の参考文献・既存実装・外部の定規を必ず探す**(現行repo・既決 → 依存／上流の取説・`reference/`・ソース・oracle → 公式規格・一次資料・製品先例)。無ければ検索範囲を残し、自作testは借りた定規の写像だけにする。
自前の天井は `reference/owned-budget.tsv`、人の違和感の天井は `reference/hygiene-budget.tsv`(`scripts/check-hygiene.sh`、閾値は借り物)。**家は5つ — doc・render・ui・vism・tests。足すなら消す。build も家ごと**(`crates/motolii-doc` が契約、`motolii-render` が描く側、`ui/` が Flutter UI、`ui/native` が薄いRust接続。`motolii-road` は重い依存を 1 本の dylib に 1 回だけ link する道路)。

- **手段は、明示された目的と確認済みの意向で測る**。指示された手段より明確に良い道が見えるなら、黙って従わず、実行前に案・根拠・代償を短く出す。ただし「相談」を許可取りに使わない — 判断材料を持っているのはこちら、決めるのはあなた。推した「真意」で明示の制約や権限は動かさない。目的・範囲・重要な制約を変えるなら確認、依頼の中の軽微で戻せる改善は止めずに進める
- **長い作業を黙って進めない**。着手前に「何を・どの順で・どこまで」を 1〜2 行、区切りごとに結果を 1 行。build・計測・多 file の編集など数分かかる物は始めと終わりを必ず出し、詰まったら詰まったと言う。無言は利用者からは「止まっている」と同じに見える
- 編集状態は Document が持つ。書き込みは Intent 経由のみ
- 責任は入口→意味→評価→結果→試験まで完結。後の統合・結線が要るなら未完。**並列の lane が同じ file を触る時点で失敗**(責任が集まっている印)。worktree で逃がさず、先にその file を data／manifest の口にして、以後は行を足すだけで載る形にする
- 1つ直せば同族全部が直るcomponent/dataにする。同じ意味を2箇所へ書いたら未完
- 開発入口は `../scripts/motolii-ui.sh`。`dev`でFlutterを常駐し、通常のUI変更は`reload`または`r`でDocument・GPU資源を保持して反映する。Rust変更だけ`native`でbuildし、保存後にアプリを再起動する。初回・依存・型変更のbuildは途中で時間切れにせず完走させる。cacheを保ち、確認目的だけの全buildを増やさない。hot reloadとhot restartは区別し、初期化で失うUI状態を明示する。見た目と操作は実窓で検収する。Stage 5の現在地と未完は`../docs/stage5/workspace.json`と`../docs/stage5/README.md`。
- **窓に出る文字は英語**。値が意味の物だけ文字、あとは形で見せる
- 決定は `../docs/decision-index.md` を grep。コードの現状・理由はコードに書かない — 腐る

窓: `ui/build/macos/Build/Products/Debug/motolii_stage5.app`。旧`src/ui`・`motolii-dx.sh`は比較用で、新UIの通常路ではない。
