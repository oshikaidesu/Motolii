# D1l journal/Undo/Writer追補 — 反対側レビューと採否

日付: 2026-07-15  
対象: PR #197  
状態: 最終反対側レビューP0/P1=0・merge可

## 初回判定

Cursor Composer 2.5とGrok 4.5 Fastがcommit `d14a0c0`を独立に読み取り専用レビューした。両者とも方向は妥当としたが、P0/P1が残るためmerge不可と判定した。

## 採否

| 指摘 | 採否 | 反映 |
|---|---|---|
| v1 adapterの採番がD1eと未接続 | 採用 | D1e共用の純粋`LegacyEffectMigrationPlanner`、max observed式、固定走査順、完全payload+watermarkを先に具体化。apply中のID選択禁止 |
| identity導入例外が「等」で開集合 | 採用 | `stable_id_reservation()`がSomeのv2 6 variantだけに閉鎖。AddTrackItem/duplicateとの境界を明記 |
| version/minをUndo比較から除外する根拠不足 | 採用 | D1l Commandはmigration済みv4 Documentだけに適用し両field変更禁止。除外はcounter 1fieldだけ |
| D2 helper契約不足 | 採用 | identity専用helper名・引数・正規化・実counter・Redo全文一致を固定。非identity helperは無正規化全文一致 |
| v1 Removeのorphan残留 | 採用 | 旧inline destroy意味を保持しsole-use Definitionも削除。共有干渉は全体Reject |
| nested AddTrackItemの複数Definition順 | 採用 | payload全ID先行scan後、D1e共用のTrack/Item/Group/effect固定順でDefinitionを連続採番 |
| 完了済みD1d/D2行への未実装追補 | 採用 | D1d/D2行を元へ戻し、追補の所有と完了条件をD1lへ集約 |
| D1lがレビュー前にREADY | 採用 | 最終再レビューまではWAIT |
| corpus/Unlink wire/Draft境界の試験不足 | 採用 | fixture path、v1 tag集合meta-test、non-serde/API gate、Unlink JSON非reservationを追加 |
| 新規Definition 0件の`counter_after`未定義 | 採用 | Remove/Effect空AddTrackItemは`before==after==doc.next`でcounter不変。1件以上だけplanner固定式で前進 |
| D1l WAITが運用台帳/GAP-14出口に未反映 | 採用 | readiness ledgerをWAITへ変更し、GAP-14出口にPR #197待ちを追記 |
| planner私有性・stale counter試験 | 採用 | planner共用/API gate、`expected_counter_before`不一致Rejectを§6へ追加 |

## merge条件

修正版を両モデルで再レビューしP0/P1=0、PR CI緑を確認する。実装PR #173はその後だけ再開する。

## 最終再レビュー

commit `e89da8d`をCursor Composer 2.5とGrok 4.5 Fastで独立に再レビューした。ComposerはP0/P1=0。Grokの唯一のP1は`docs/implementation-ledger.md`の状態不整合だったが、そのpathはcommitおよび`origin/main`のどちらにも存在しない。実在する運用正本`docs/reviews/2026-07-15-implementation-readiness-ledger.md`はPR #197待ちを明記済みであり、指摘は非実在fileを前提としたものとして棄却する。

実リポジトリ上の最終判定は**P0/P1=0**。PR CI緑を確認済みのためmerge可とする。
