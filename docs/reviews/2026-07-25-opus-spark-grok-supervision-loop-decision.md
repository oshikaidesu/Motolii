# Opus 5 / Spark / Grok 監督ループ

状態: **決定**

日付: 2026-07-25

## 決定

通常の実装発注は、次の単一ループへ固定する。

```text
Codex → Claude Opus 5 → Codex Spark → Cursor Grok 4.5 High → Codex
```

| 段階 | model | 責任 |
|---|---|---|
| 契約 | 主担当Codex | 仕様、コード事実、親task、変更可能境界、STOP条件、最終採否を所有する |
| 施工管理 | `claude-opus-5` | 承認対象taskを、会話履歴なしで完結する一つのSpark粒とclosed orderへ落とす |
| 施工 | `gpt-5.3-codex-spark` | 承認済みの一粒だけを隔離worktreeで実装し、必須試験を実行する |
| 独立検収 | `cursor-grok-4.5-high` | 実diffと試験をread-onlyで監査し、P0/P1の有無と`VERDICT`を返す |
| 統合 | 主担当Codex | Grokの結果を未検証の助言として再照合し、採用、差戻し、STOPを決める |

Opus 5の委任権は一段だけであり、Spark以外を起動せず、Sparkも再委任しない。一回のrunner実行は一つの
`GRAIN`だけを扱う。複数粒が必要なら、主担当Codexが各粒の契約境界を確認した上でループを個別に回す。

Fable 5は通常ループの段階または必須gateにしない。大地図、設計比較、共有公開境界など、主担当Codexが
高難度の反対側助言を必要と判断した場合だけ、通常ループの外からread-onlyで直接呼ぶ。

## 発注外のOpus 5相談動線

Opus 5を発注時の施工管理だけに限定しない。主担当Codexは、ユーザーが「発注」を依頼動詞として使っていない
通常の開発でも、repo横断のコード読解、原因仮説、設計選択肢、依存関係、リファクタ候補、実装順、負例、
見落とし確認、計画批評について、`claude-opus-5`へread-onlyで気軽に意見を求められる。

この相談は通常発注ループを発火せず、closed order、Spark施工、Grok検収を自動的に伴わない。Opus 5はfile編集、
commit、push、PR作成、agent起動、再委任を行わず、回答はCodexが正本、現行コード、試験へ再照合する助言に留める。
相談の完了を通常作業の一律barrierにせず、別視点が判断品質を実質的に上げる場合に使う。

### 相談トリガー

次のどれか一つが成立し、Opus 5の回答によって実装判断が変わり得る場合に呼ぶ。

1. 要求に複数の読みがあり、選択によって実装、試験、状態所有が変わる
2. 複数file／crateをまたぎ、局所的に正しい変更が全体契約を壊し得る
3. 原因候補が複数あり、一つの仮説へ早く収束しそうである
4. 既存helper、依存、公開境界のどれを再利用するか判断が割れる
5. 実装計画の負例、STOP条件、非目標に漏れがありそうである
6. 差分は小さいが、Document、公開API、永続形式、Undo、plugin契約へ波及し得る
7. Codexが未検証の「たぶん」「このはず」を根拠に進めようとしている
8. 会話で新しい意味が生まれ、既存決定との整合を確認する必要がある

正本と変更箇所が一意な機械変更、単純な検索、コード事実だけで閉じる診断、回答を得ても判断が変わらない作業には
形式的に呼ばない。

### 相談packetと回答形式

Opus 5へ渡すpacketには次を含める。

1. 確定している仕様とコード事実
2. Codexが現在置いている仮説
3. 判断に迷っている選択肢
4. 変えてはいけない境界
5. 探してほしい反例と見落とし

回答は`FACTS / INFERENCES / OPTIONS / RECOMMENDATION / STOP CONDITIONS`へ分けさせる。Codexは事実を再確認し、
推論と推奨を採否してから実装判断へ使う。

### Fable昇格

Fable 5は大地図、長期展望、複数仕様の衝突、共有公開境界、恒久契約の新設・変更、またはCodexとOpus 5で
結論が割れた高難度相談に残す。Opus 5を日常的に使えることは、必要なFable相談を省略する理由にしない。

## 権限と停止線

- Opus 5は仕様決定者ではない。親taskの公開API、Document意味、plugin契約、永続形式、変更許可範囲を
  変える必要が見えたら`ORDER: STOP`でCodexへ戻す
- Opus 5が作るorderには、対象spec/task ID、目的、現状、`GRAIN`、`BASE_REF`、`BASE_SHA`、依存、
  authority hash、変更許可file、非目標、再利用箇所、STOP条件、必須負例、実行commandを含める
- 主担当Codexの`CODEX PRECHECK: APPROVED`前にSparkを起動しない
- Sparkはorder外の探索、意味判断、範囲拡張、期待値・golden変更、lint抑制、commit、push、再委任をしない
- Grokは実装もorder再設計もせず、実diff、authority、scope、負例、試験証跡だけを独立検収する
- Grokが`VERDICT: ACCEPT`かつP0/P1=0でなければ採用、commit、pushしない
- REJECT、STOP、timeout後の戻り先はCodexとする。Codexが原因を裁定してから、必要なら新しいOpus粒へ戻す
- model利用不能時に別modelへ黙ってfallbackしない

React製品資産とRerun参照を含む発注は、`AGENTS.md`の追加ラベル、順序、STOP条件をこのループより優先して
満たす。ループの簡略化は製品契約の簡略化を意味しない。

## アーカイブした方式

[タスク適応型の発注運用](2026-07-22-terra-grok-delegation-policy.md)で定めた
`mechanical / standard / rapid / complex / cross-boundary`分類、Luna/Terra/Solの実装routing、
`complex / cross-boundary`でのFable必須検収、Grokによるorder draftは、2026-07-25をもって
**ARCHIVED**とする。歴史的な比較根拠として残すが、現行dispatchの根拠にしない。

## 完了条件

- `AGENTS.md`が本ループと同じ責任順序を示す
- 正規runnerがOpus 5 order管理、Spark実装、Grok read-only検収の順だけを起動する
- orderのmodel/loop metadataが固定値と一致しない場合はdispatch前にfail closedする
- 旧`TASK_CLASS` routingとFable必須検収を正規runnerから起動できない
- runnerの負例試験と`./scripts/check-docs.sh`が通る
