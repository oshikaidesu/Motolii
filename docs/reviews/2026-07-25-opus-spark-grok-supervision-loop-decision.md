# Grok / Spark / Opus 5監督ループ（撤回済み）

状態: **撤回**

初回決定: 2026-07-25
撤回: 2026-08-03

## 現行状態

この文書が定めていた固定model route、order schema、compiled grain、canonical activation、runner hash、checkpoint、
resume/cancel資格、独自receiptはすべて撤回された。新規作業の起動条件、必須field、採用資格、fallback規則として使わない。

現行正本は次の二つだけである。

- [runner非依存の監督責任決定](2026-08-03-runner-independent-supervision-decision.md)
- [薄いobserved CLI harness決定](2026-08-03-thin-observed-cli-harness-decision.md)

## 継承した原則

旧ループから現行監督へ継承したのは次だけである。

- 一回の実装を一つの契約境界へ閉じる
- authority、owner、原因、再利用、oracleが閉じない`WIDE`を実装担当へ送らない
- worktree、base、scope、fingerprint、diffを主担当Codexが直接確認する
- 実装担当とreviewerを分離し、reviewerをread-onlyにする
- reviewer mutation、P0/P1未解決、scope外変更を採用しない
- LLM verdictでなくtask固有test、fixture、bench等を合否の正本にする
- 最終採否は主担当Codexが所有する

これらはrunner protocolではなく監督者の責任であり、transportへ実装しない。

## 歴史証跡

撤回前の全文、route改訂、実験値、旧field、旧launcher手順はGit履歴で参照できる。直前の全文版は
commit `f38a1455`の親履歴に残る。過去receiptと退役scriptも事故分析には使えるが、新規実行のauthorityではない。

本pathを参照する旧decision、review、issueは歴史上の根拠を指す。現在の施工手順を求めてこのfileへ到達した場合は、
上記現行正本へ移動し、Git履歴の旧本文を実行手順として復元しない。
