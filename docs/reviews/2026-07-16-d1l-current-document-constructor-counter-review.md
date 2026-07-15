# D1l新規Document v4生成契約 — 反対側レビューと採否

日付: 2026-07-16
対象: [追補決定](2026-07-16-d1l-current-document-constructor-decision.md)
状態: 最終反対側レビューP0/P1=0・merge可

## 発見経緯

D1l実装候補`26aea92`の発注レビューで、Grok 4.5 Fastが「公開`allocate_effect_*`を削除すると、`Document::new_v1()`しかない新規文書がv4に到達できない」とSTOPした。現行constructor、version/min検証、PR #197のversion/min不変契約を独立に照合し、指摘を採用した。

## 初回判定と採否

並列再レビューはComposer/Grokとも300秒で無出力タイムアウトしたため、合格証跡に使用しない。対象worktreeでComposer 2.5に焦点レビューを再発注し、P0=0、P1=4を得た。

| P1 | 採否 | 反映 |
|---|---|---|
| `READER_VERSION`の4化が明示されていない | 採用 | reader/writer/effect minの3定数を4に固定し、`new_current()` roundtripを`ReadWrite`審判にした |
| 版fieldだけ4の未移行inline/hybridがgateをすり抜ける | 採用 | schema/validationの構造拒否と失敗時全文不変を追加 |
| D1e v3→v4 inline migrationが実装義務として弱い | 採用 | PR #173の必須範囲・実装順2とし、別タスクへの先送りを禁止 |
| `new_v1()`非製品利用のgrep境界が未定義 | 採用 | migration / `cfg(test)` / integration testの閉allowlistと負例ポリシーテストを固定 |

## 最終再レビュー

修正後の全diffをGrok 4.5 FastとComposer 2.5が独立に読み、両者とも**P0/P1=0**と判定した。意味の未決は残っていない。本決定merge後、D1l実装PR #173を追補契約に合わせて再開できる。
