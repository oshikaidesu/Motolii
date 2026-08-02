# Motolii独自監督runnerの非破壊的廃止

状態: **決定・発効は本変更のmain統合後**

日付: 2026-08-02

## 決定

`scripts/delegate-cursor-supervised.sh`と`scripts/activate-supervised-runner.sh`を通常発注の入口から外す。
両scriptは履歴、過去receiptの読解、rollback比較のためGit上に残すが、先頭でretirement markerを出して
exit 64とする。既存のcanonical bundleを削除せず、今後の採用資格だけを与えない。

これは役割分離や契約gateの廃止ではない。正本、base、exact target、allowlist、WIDE拒否、実装者と検収者の
分離、reviewer mutation拒否、P0/P1=0、Codex最終採否は維持する。廃止するのは、それらを約2,300行の
Motolii固有shellへ重複実装し続けるtransportと状態機械である。

## 実地検証

local main `c695652ee041612c1c2a30ae2345b23fb6b298a7`の実在M3粒`CU-201T-C`を、旧runnerを通さず
`@agentex/agent` 0.0.34から実行した。

- Agentexのworkspace APIで隔離worktreeを作成した
- `claude-opus-5`をsafe-mode/read-onlyで監査し、誤ったauthority pathを`ESCALATE`で拒否した
- 同じClaude sessionをresumeし、正しいauthorityだけを追加して再判定した
- `gpt-5.3-codex-spark`をworkspace-writeでtest-only補修へ使い、主担当の反証後に同じsessionへ差戻した
- `cursor-grok-4.5-high`をplan/read-onlyで独立検収し、`ACCEPT / P0=0 / P1=0 / SCOPE PASS`を得た
- targeted test 41件、fmt、clippy、diff checkはgreen。reviewer mutationは0だった

同時に、文書上の次粒とmain実装の不一致、実装不能な旧allowlist、名前だけのkeyframe oracleを発見した。
旧runnerの追加改訂ではなく、authority修正と既存test familyの補修へ限定した。

## Agentexで確認済みの範囲と未閉鎖

確認済み:

- Claude / Codex / Cursorの既存認証とexact model指定
- provider共通入口、session ID、follow-up、resume、cancel、worktree
- Claude safe-mode、Codex workspace-write wrapper、Cursor plan/read-only

未閉鎖:

- Codex providerでambient user config / memory / pluginを完全に無効化する標準option
- Cursor providerの実行中streamを主担当へ継続表示する観測性
- authority/base/order hashと採用資格をappend-onlyで永続化する標準receipt
- provider共通のexact allowlistと`WIDE`機械拒否

未閉鎖項目を埋めるために第二の巨大runnerを作らない。まずAgentex upstreamのAPI、hook、provider optionで
解決し、Motolii側は薄い設定またはadapterに限定する。未閉鎖の保証が必要な粒は局所STOPし、保証が無いまま
「同等」と称さない。

## 非破壊性とrollback

- sourceと専用testsは削除せず、retirement guardより後ろに歴史実装を保存する
- Git common dirの既存bundleやreceiptを削除しない。発効時にactive launcherをexit 64 stubへ置換し、旧launcherを
  `run.retired-20260802`、manifest snapshotを`active.retired-20260802.txt`へ保存する
- rollbackは本決定commitをrevertするだけでは発効しない。旧transportを再採用する新しい利用者決定、正本更新、
  専用負例の再実行、canonical byteの再activateを別粒で必要とする
- 旧receiptは過去の証拠として読めるが、新規実装の採用資格には使わない

## 非目標

- AgentexをMotolii固有frameworkへforkする
- 旧order schema、compiled grain、receipt DBを別言語で再実装する
- 過去branch、worktree、receipt、runner sourceを削除する
- 未閉鎖の隔離・観測・永続性を解決済みと扱う
