# 履歴較正によるLLM役割選択

状態: **決定**（main統合後発効）

日付: 2026-08-03

## 目的

Luna Maxを通常監督の低コストな第一候補、Solを難所と最終統合の疎な昇格先とする。一方で、Grok、Claude、Sparkを
全task共通の固定順へ戻さず、過去の実行履歴で観測した得意領域と失敗形をtask別の役割へ反映する。transport、監督責任、
採用資格は増やさない。

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
- `gpt-5.6-luna` maxへ6文書と履歴を広く探索させた試行は正しく`STOP`したが、360.865秒、input 1,535,651、stdout
  1.53 MBとなり、長いcommit SHAを2回誤った。安価なmodelでも全履歴の反復読込を許せば運用costと転記riskが膨らむ
- 同じLuna sessionへ検証済みの小さなcapsuleだけを渡した二turn試行は、合計約39.5秒、input 33,616、cached input
  27,904、output 1,964で、`CU-201P = WAIT_TARGET`を発明で迂回せず、履歴記述と現行状態を区別した。これは接続flowの
  bounded fixtureであり、一般的な品質優位や実料金の証明ではない

根拠は[監督ループ速度支配項観察](2026-08-01-supervision-loop-cost-driver-observation.md)と
[SD-02G Opus検収記録](2026-07-30-sd-02g-product-host-layout-geometry-implementation-decision.md)に置く。過去のraw streamは
履歴確認用であり、新しいreceipt DBやmodel scoreへ集約しない。

## 通常姿勢

1. 通常監督は完全model ID `gpt-5.6-luna`、`model_reasoning_effort=max`を第一候補とする。これは固定routeではなく、
   小さく閉じたtaskならLunaだけで調査・施工管理を終えられるというcost姿勢である
2. 主担当Codexは起動前に、利用者outcome、base/HEAD、現行authority、状態、exact target、owner/write route、oracle、非目標、
   STOPと、それらを直接示す少数の検証済みsnippetをcontext capsuleへまとめる。外部modelへrepo全体、全正本、会話全履歴を
   初動から再読させない。不足は推測で埋めず、Codexへ返して検索結果を更新する
3. 一sessionは一契約境界または同じoutcome、owner、scope、oracleに閉じた短いwaveだけを扱う。その四点が不変な間だけresumeし、
   統合、STOP、境界変更のいずれかで閉じる。新しい粒はfresh sessionとfresh capsuleから始める
4. 長期状態はGit、現行正本、decision index、implementation ledger、raw実行logへ置く。会話履歴、session token、modelの記憶を
   authority、採用DB、project memoryにしない
5. `gpt-5.6-sol`は通常`medium` effortで、authority衝突、複数の意味／owner／原因、共有公開境界、Document／永続形式／plugin契約、
   Lunaの探索膨張または反復STOP、main統合直前の全体照合へだけ昇格する。riskに応じたeffort増加は許すが全粒の直列gateにしない
6. LunaとSolは同じOpenAI familyなので相互に独立検収者とはみなさない。独立reviewが必要な施工では、設計・施工へ深く関与して
   いないClaudeまたはGrok等の別familyをfresh read-only sessionで選ぶ

## 役割選択

| taskの状態・判定対象 | 第一候補 | 用途 | 最終reviewer |
|---|---|---|---|
| 一契約境界または短いwaveの通常監督 | Luna Max | capsule内の調査、施工管理、進捗判断 | 施工を伴う場合は別familyをriskに応じて選択 |
| authority衝突、意味、owner、原因、共有契約が未閉鎖 | Sol medium以上、必要ならClaude Opus read-only | 反例、STOP、選択肢、閉鎖条件 | 閉鎖後に実装するなら関与していない別family |
| authorityは閉じたがscope、allowlist、exact target、負例が複雑 | Grok read-only | boundedな粒化・preflight、漏れの列挙 | Grokを施工判断へ使ったならClaude等の別family |
| 一契約境界に閉じた機械施工 | Luna Max、Spark、Composer等 | 指定pathの変更と指定試験。cost、capacity、task適性で明示選択 | 選択したmodelと異なるfamily |
| main統合直前、複数粒の整合、Lunaの探索膨張 | Sol medium以上 | authority、非目標、diff、oracleの全体照合 | SolはLuna施工の独立reviewを兼ねない |
| 実diffのscope、削除、guard、負例を詳しく監査 | Grok read-only | concrete diff audit | Grok自身は採否しない |
| 実diffの意味、owner、既存契約との統合を監査 | Claude Opus read-only | semantic final audit | Claude自身は採否しない |
| 性能、安全性、永続形式、platform correctness | 非LLM oracle | bench、negative test、schema/OS fixture | LLMは補助監査のみ |

## 分岐規則

1. Codexが先にauthority、base/cwd、worktree、scope、oracle、実装担当候補を確認する
2. 小さく閉じたtaskはLunaだけで監督でき、外部preflightやSolを省く。全modelを通すことを完了条件にしない
3. Lunaで閉じない意味をSolまたはClaudeへ、具体的な境界列挙をGrokへ送る。複数が必要なら独立に並べず、先の回答をCodexが正本へ
   再照合して残った問いだけを次へ送る
4. 設計・契約閉鎖へ深く関与したmodel familyは同じtaskの最終reviewer候補から外す
5. modelのcapacity／rate limitは観測失敗として一度Codexへ戻す。同じtask境界がなお有効なら、利用可能なmodelを新しい
   実装担当として明示選択できる。CLIで確認した完全model ID、変更理由、fresh sessionをlogへ残し、自動retry列、alias推測、
   途中sessionのmodel間引継ぎ、無記録のfallbackは行わない
6. Grokのtimeout、CLI失敗、空の完了結果は観測失敗としてCodexへ戻す。未完了streamの一時的なstdout空とは区別し、
   別modelへ黙ってfallbackしない
7. reviewerはfindingを列挙するだけでscope、order、実装、採用を増やさない。最終採否はCodexが非LLM oracleと合わせて行う
8. Lunaのtool call、読込量、転記誤り、wall timeがcapsuleの意図を超えて膨らんだ場合は、同じsessionへ全文を追加せず閉じる。
   Codexがauthority検索とcapsuleを修正し、意味難所ならSolへ昇格する

## 非目標

- `Grok → Spark → Opus`または`Claude → Spark → Grok`を固定routeにすること
- `Luna → Sol → reviewer`を全task共通の固定routeにすること
- modelごとのscore、学習DB、append-only receipt、retry状態機械を作ること
- model失敗時に無条件で特定modelへ切り替える固定fallback chainを作ること
- 過去のwall timeだけでmodelを格付けすること
- Claude/Grokの賛同をauthority、ユーザー権限、採用資格にすること
- 一つのLuna sessionへproject全履歴を保持し続けること

## Fable read-only経路

Fableは大地図、長期展望、複数仕様衝突、共有公開境界、恒久契約、CodexとOpusの結論衝突、または
一般機構の既知routeが具体的反証で尽きた時の一回の取りこぼし検査だけに使う。正規model IDは
`claude-fable-5`で、Claude Code CLIから薄いCLI harnessを介してread-onlyで直接起動する。
Cursorの同名modelや別modelへ黙ってfallbackせず、編集、Bash、commit、push、外部model起動、再委任を許可しない。
出力は助言であり、Codexが正本、現行コード、取得済み一次資料へ再照合して採否する。
