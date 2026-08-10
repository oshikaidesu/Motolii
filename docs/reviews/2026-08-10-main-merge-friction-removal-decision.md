# main統合の段差撤廃決定

日付: 2026-08-10
状態: **決定**
判断者: 利用者(owner)。本文書はその明示判断の記録であり、[同日の回収監査](2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md)は裏付け観察として併読する(調査結論を単独の設計根拠にしない — reviews規律1)。

## 背景 — mainがmainでない

2026-08-10の監査で、開発の実態がmainの外に滞留していることを実測した。

- 未mergeブランチ208本。implementation-ledger更新を抱えたまま滞留するブランチが複数
- ローカルmainがorigin/mainから60コミット遅れ。作業ブランチはorigin/mainから169コミット遅れ
- 決定文書13本がリポ外`~/Documents/Codex/`の絶対パスを正本として参照
- MotoliiRnProbe(RN製品UI 660行)の回収は約2割、skia-timeline-probe(bin 15本+depth-rail v4〜v14)はコード回収0%
- 台帳・地図が08-09/08-10の実装ラッシュに追い越され、`WAIT`/`NOT STARTED`/`COMPILE(禁止)`の粒が実際にはmain着地済み

因果は一つに収束する: **mainへのマージに課された検証段差**(`validate.sh local` = fmt + clippy `-D warnings` + 全workspace test + docs gate、加えてtask固有test、policy lane)**が重く、全員がマージを回避する**。成果はブランチとリポ外workdirに溜まり、mainは真実でなくなり、後続の発注LLMは存在しない世界で作業して再発明する。

## 決定

1. **mainへのマージ条件から検証段差を全廃する。** `./scripts/validate.sh local`、task固有test、policy laneの事前通過をマージ条件にしない。マージを止める条件は「conflictなく成立すること」のみ。
2. **検証laneは事後観測へ降格する。** lane redはmain上でfix-forwardする。マージ拒否・差し戻しの根拠にしない。
3. **成果は当日中にmainへ。** ブランチは短命に保つ。リポ外workdirの成果とブランチ滞留分は、mainに入るまで「完了」と数えない([回収監査](2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md)の未回収一覧が初期対象)。
4. 既存の未mergeブランチ208本は、本決定後に回収(merge)または破棄(tombstone)へ二分する。塩漬け第三状態を作らない。

## 撤廃しないもの(段差ではなく意味保護・報告規律)

- **虚偽green報告の禁止**: 赤laneをgreenと報告しない、未実行gateを実行済みと書かない。これは報告の正直さであってマージの段差ではない。
- **test/golden/threshold/期待値を実装都合で変えてgreenにしない**: test意味の保護。redのままマージしてよいことと、redを偽装してよいことは別。
- **check-docs.sh等の検査そのもの**: ツールとして残す。実行タイミングが「マージ前の関門」から「マージ後の観測」へ変わるだけ。

## 改訂した規約

- `AGENTS.md`「検証と完了報告」: 通常提出の事前gate要件を撤廃し、事後観測+fix-forwardへ差し替え。

## 見直しトリガー

fix-forwardが機能せずmainのredが3日以上放置される事態が繰り返される場合、「マージは自由・redの放置だけを検知して自動issue化する」等の軽量な観測強化を検討する。事前gateの復活は本決定の撤回としてのみ行う。
