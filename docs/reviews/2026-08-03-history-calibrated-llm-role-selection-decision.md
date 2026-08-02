# 履歴較正によるLLM役割選択

状態: **決定**（main統合後発効）

日付: 2026-08-03

## 目的

Grok、Claude、Sparkを全task共通の固定順へ戻さず、過去の実行履歴で観測した得意領域と失敗形を、主担当Codexが
taskごとに選ぶ役割へ反映する。transport、監督責任、採用資格は増やさない。

## 履歴から確認したこと

- Grok reviewはallowlist、base、authority hash、削除数、exact oracle、余分な宣言等を具体的に列挙し、実装がtest greenでも
  必須guard不足をP1としてREJECTした。一方で完了stdoutが空の試行もあり、常時応答を前提にできない
- Claude Opusは未閉鎖の意味、owner、共有契約を実装前にSTOPへ戻す用途と、閉じた実diffの意味監査に適した。実例では
  hidden roleとshare再配分の欠落をP1としてREJECTし、修正後にACCEPTした
- 一行fixtureではGrok preflight 22秒、Spark施工18秒、Opus final 9秒で疎通したが、同一条件A/Bではなく品質・速度改善の
  証明ではない。固定routeの復活根拠にはしない
- 同じOpusを設計相談と最終reviewへ使うと、実装担当と別sessionでも判断の独立性が弱い。family分離を優先する
- LLM間の誤りは相関する。性能、安全性、永続性、platform挙動はmodelの賛同でなくbench、negative test、schema fixture、
  OS oracleへ置く

根拠は[監督ループ速度支配項観察](2026-08-01-supervision-loop-cost-driver-observation.md)と
[SD-02G Opus検収記録](2026-07-30-sd-02g-product-host-layout-geometry-implementation-decision.md)に置く。過去のraw streamは
履歴確認用であり、新しいreceipt DBやmodel scoreへ集約しない。

## 役割選択

| taskの状態・判定対象 | 第一候補 | 用途 | 最終reviewer |
|---|---|---|---|
| 意味、owner、原因、共有契約が未閉鎖 | Claude Opus read-only | 反例、STOP、選択肢、閉鎖条件 | 閉鎖後に実装するならGrok等の別family |
| authorityは閉じたがscope、allowlist、exact target、負例が複雑 | Grok read-only | boundedな粒化・preflight、漏れの列挙 | Grokを施工判断へ使ったならClaude等の別family |
| 一契約境界に閉じた機械施工 | Spark | 指定pathの変更と指定試験 | 通常はClaude。Claude familyが施工・設計へ関与済みならGrok等 |
| Sparkがcapacity／rate limitで利用不能 | Composer、Luna Max等の低コストmodel | 同じbase・scope・allowlist・oracleを再確認したfresh実装 | 選択したmodelと異なるfamily |
| 実diffのscope、削除、guard、負例を詳しく監査 | Grok read-only | concrete diff audit | Grok自身は採否しない |
| 実diffの意味、owner、既存契約との統合を監査 | Claude Opus read-only | semantic final audit | Claude自身は採否しない |
| 性能、安全性、永続形式、platform correctness | 非LLM oracle | bench、negative test、schema/OS fixture | LLMは補助監査のみ |

## 分岐規則

1. Codexが先にauthority、base/cwd、worktree、scope、oracle、実装担当候補を確認する
2. 小さく閉じたtaskは外部preflightを省く。全modelを通すことを完了条件にしない
3. 未閉鎖の意味をClaudeへ、具体的な境界列挙をGrokへ送る。両方必要なら独立に並べず、先の回答をCodexが正本へ
   再照合して残った問いだけを次へ送る
4. 設計・契約閉鎖へ深く関与したmodel familyは同じtaskの最終reviewer候補から外す
5. Sparkのcapacity／rate limitは観測失敗として一度Codexへ戻す。同じtask境界がなお有効なら、ComposerまたはLuna Max等を
   新しい実装担当として明示選択できる。CLIで確認した完全model ID、変更理由、fresh sessionをlogへ残し、自動retry列、
   alias推測、途中sessionの引継ぎ、無記録のfallbackは行わない
6. Grokのtimeout、CLI失敗、空の完了結果は観測失敗としてCodexへ戻す。未完了streamの一時的なstdout空とは区別し、
   別modelへ黙ってfallbackしない
7. reviewerはfindingを列挙するだけでscope、order、実装、採用を増やさない。最終採否はCodexが非LLM oracleと合わせて行う

## 非目標

- `Grok → Spark → Opus`または`Claude → Spark → Grok`を固定routeにすること
- modelごとのscore、学習DB、append-only receipt、retry状態機械を作ること
- Spark失敗時に無条件で特定modelへ切り替える固定fallback chainを作ること
- 過去のwall timeだけでmodelを格付けすること
- Claude/Grokの賛同をauthority、ユーザー権限、採用資格にすること
