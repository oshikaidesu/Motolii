# Motolii — rerun を AE にするソフト

## PRODUCT ORACLE

**完成の審判は利用者の制作行為。** テストが緑・APIがある・widgetが表示された・コード上到達可能は「使える」の代わりにしない。UI変更は production UI から**発見→操作→期待する結果→cancel/undo→再訪**まで実窓で通して初めて完了。fixture・CLI・debug op・test harness・sourceを読まなければ分からない隠し導線は利用者経路の代わりにしない。長期作業でも不変：adapter・test・contract・Dock・TimelineCoreのような局所taskを最終目的に昇格させない。区切りごとに「利用者の制作行為を何ができるようにしたか」で進捗を測る。commit数・test数・接続数は製品進捗の代理ではない。

既存UIの移植・置換・再presentationでは、**capabilityを落としていないことの証明責任は新実装側にある**。見た目とAPI一覧だけの比較は不可。旧実装が受け付ける入力語彙(click・double click・drag・drag&drop・wheel・trackpad pan/zoom・modifier・keyboard・context menu・hover・focus・marquee・cancel/Escape・undo/redo)と、旧来のvisual hierarchy・information architecture・state・feedback・recoveryを実コードと実窓で確認する。「知らなかった入力」は「存在しない入力」ではない — **未調査は未完**。「不要そう」「pre-existing」「out of scope」は未確認項目を消す理由にならない。普通の制作で要るなら対象。合否は最終スクリーンショットではなく利用者の操作列(trajectory)そのもので取る。click/drag/wheel/key/DnD後にvisual feedbackもselectionもStageも変わらなければ、exceptionが無くてもFAIL — **無反応は成功ではない**。変更対象が有限集合(全panel・全shelf・全face・全gesture等)なら列挙して全項目を確認する。**一例の成功は集合のPASSではなく、未確認はPARTIAL**。実装されているだけでは存在すると数えない。機能名を知らない利用者が通常のUI慣習(right click・drag&drop・menu・⌘S・⌘Z等)だけで辿れて初めて完成。複数surfaceにまたがる作業の最終oracleは実際の作品制作 — 新規document→Text→Shape→Color→Font→Effect→Stage操作→keyframe→Timeline編集→Ease→Camera→再生→undo/redo→保存→再起動→再度開く→exportまでproduction UIだけで通す。

**失敗はテストへ。この文書と記憶は、足すとき同じ行数を消す。**

作ってよいのは**編集の意味**と**エフェクトのデータ**だけ。描画は rerun、UI は Flutter の部品。**新規コードと検証の前に、同じ意味・機構の参考文献・既存実装・外部の定規を必ず探す**(現行repo・既決 → 依存／上流の取説・`reference/`・ソース・oracle → 公式規格・一次資料・製品先例)。無ければ検索範囲を残し、自作testは借りた定規の写像だけにする。
自前の天井は `reference/owned-budget.tsv`、人の違和感の天井は `reference/hygiene-budget.tsv`(`../scripts/check-hygiene.sh`)。**家は5つ — doc・render・ui・vism・tests。足すなら消す。** `crates/motolii-doc` が契約、`crates/motolii-render` が描く側、`ui/` がFlutter UI、`ui/native` がRust接続。Cargoの入口はrepo rootだけ。

- **手段は、明示された目的と確認済みの意向で測る**。指示された手段より明確に良い道が見えるなら、黙って従わず、実行前に案・根拠・代償を短く出す。ただし「相談」を許可取りに使わない — 判断材料を持っているのはこちら、決めるのはあなた。推した「真意」で明示の制約や権限は動かさない。目的・範囲・重要な制約を変えるなら確認、依頼の中の軽微で戻せる改善は止めずに進める
- **長い作業を黙って進めない**。着手前に「何を・どの順で・どこまで」を 1〜2 行、区切りごとに結果を 1 行。build・計測・多 file の編集など数分かかる物は始めと終わりを必ず出し、詰まったら詰まったと言う。無言は利用者からは「止まっている」と同じに見える
- 編集状態は Document が持つ。書き込みは Intent 経由のみ。1つ直せば同族全部が直るcomponent/dataにする。同じ意味を2箇所へ書いたら未完
- 責任は入口→意味→評価→結果→試験→**production UIでの検収**まで完結。後の統合・結線・確認が要るなら未完。**並列の lane が同じ file を触る時点で失敗**(責任が集まっている印)。worktree で逃がさず、先にその file を data／manifest の口にして、以後は行を足すだけで載る形にする
- **CPU の常時反復を既定にしない。** 不変入力の再計算・全体再構築・状態を知るだけの定期pollは禁止。描画・時間変化は既存GPU/engineの仕組みを先に使い、CPUは変更入力に対する必要な差分だけ。毎秒・毎フレームのCPU仕事を足すなら、変更入力・発火/停止条件・GPU/イベントで代替できない根拠・同条件の実測を先に示す。違反は採用不可、該当差分を直して再検証するまで完了扱い禁止。罰則と検収は [UIの重さの法](../docs/reviews/2026-09-12-ui-derive-outside-build.md)「CPU反復」節。
- 開発入口は `../scripts/motolii-ui.sh`。`dev`でFlutterを常駐し、通常のUI変更は`reload`または`r`でDocument・GPU資源を保持して反映する。Rust変更だけ`native`でbuildし、保存後にアプリを再起動する。初回・依存・型変更のbuildは途中で時間切れにせず完走させる。cacheを保ち、確認目的だけの全buildを増やさない。hot reloadとhot restartは区別し、初期化で失うUI状態を明示する。**UI作業はproduction windowが一次環境**：操作→問題発見→最小修正→hot reload→同じdocumentで同じ操作を再試行→制作続行のループを基本とし、fixtureやtest documentへ逃げない。unit testが緑でも元の操作が直るまで修理完了ではない。Stage 5の現在地と未完は`../docs/stage5/workspace.json`と`../docs/stage5/README.md`。
- **窓に出る文字は英語**。値が意味の物だけ文字、あとは形で見せる
- 決定は `../docs/decision-index.md` を grep。コードの現状・理由はコードに書かない — 腐る

窓: `ui/build/macos/Build/Products/Debug/motolii_stage5.app`。旧実装は[Git履歴](../docs/stage5/history/retired-source.md)へ退役済み。
