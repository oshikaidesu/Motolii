# Terra / Grok 4.5 / Composer 2.5 役割再配置決定

日付: 2026-08-07  
状態: **決定／固定pipelineではない**  
対象: 外部LLMの発注compile、施工、correction、review、Cursor first-party pool活用

## 決定

現行の第一候補を次へ再配置する。

| task状態 | 第一候補 | 用途 |
|---|---|---|
| orderが未閉鎖だがoutcomeと探索範囲は閉じている | Codex directのTerra | current fact、候補、反証、exact target、allowlist、負例、return条件を持つ候補orderを作る |
| 極小で機械的なclosed order | Codex Spark | exact path／symbolと短いcapsuleだけで超高速施工する |
| 通常〜重めのclosed implementation | Cursor Grok 4.5 non-fast | 実装、指定test、`IMPLEMENTED / PARTIAL / CONTEXT_GAP`返却 |
| Grokより安価な通常施工を優先する、またはtask実測で適合した場合 | Composer 2.5 standard | 代替施工。既定fallbackにしない |
| 同一境界の複雑correction | Grok 4.5またはLuna | findingのexact closure。実装担当と同じsessionを継続しない |
| semantic final review | 未関与のClaude Opus等 | fresh read-only、別family、実diffと非LLM oracleを監査 |

TerraとGrokを毎回直列にしない。主担当が既にclosed orderを作れる場合はTerraを省略し、Spark／Grok／Composerの適格な一つへ
直接送る。Terraは一粒ごとの前処理ではなく、同じoutcome、owner、scope、oracleを共有する短waveの候補edgeを整理する。
実行可能なclosed orderは先頭edgeだけを確定し、後続はreturn後にcurrent codeから再選定する候補に留める。

## 根拠

Composerには閉じたM2施工を完了し、独立reviewと非LLM oracleで採用へ到達した実績がある。一方、歴史reviewでは次も観測した。

- Composerが`P0/P1=0`とした後、Grok 4.5 Fastが`P0=1 / P1=2`を検出した
- Composerだけがtimeoutし、Grokが判定を完了した回が複数ある
- U0e-2縮約stub事故の主因はmodel名でなく、base／authority照合不足、stubでも通る弱い因果oracle、timeout時証跡喪失だった

したがってComposerを不採用にはしないが、通常施工の無条件defaultにも置かない。Grok 4.5をCursor first-party poolの第一施工候補へ
上げ、Composerは価格、capacity、task適合の理由がある時の明示候補とする。監督不備をmodel交換で隠さず、どの施工modelにも
closed order、allowlist、因果oracle、raw log、別family reviewを要求する。

Cursorの2026-08-07公開価格（[Composer](https://cursor.com/en-US/composer)、[Grok](https://cursor.com/grok)）では、Composer 2.5 standardはinput `$0.50/M`、output `$2.50/M`、Grok 4.5 standardは
input `$2/M`、output `$6/M`で、standard同士ならComposerが安い。Composer Fastは`$3/M`／`$15/M`、Grok Fastは
`$4/M`／`$18/M`である。価格差だけで品質や総task costを決めず、実案件のstep、token、wall、rework、採用結果を見る。

## 選択規則

```text
orderは閉じているか?
  no  -> boundedならTerraで候補compile、WIDEなら主担当へRETURN
  yes -> 極小・機械的か?
           yes -> Spark
           no  -> 通常〜重い施工か?
                    yes -> Grok 4.5 non-fast
                    price/capacity/task実測でComposer適合 -> Composer 2.5 standard
```

- Grokの通常候補は`cursor-grok-4.5-medium`、複雑な一境界は`cursor-grok-4.5-high`。`-fast`は経過時間を優先する明示理由がある時だけ
- TerraはCodex directの完全IDを現行CLIで確認し、effortをread scope拡大許可にしない。Cursor版TerraはCodex directのlimit／capacity／障害時だけ明示代用する
- Composerは`composer-2.5`を通常候補とし、`composer-2.5-fast`を暗黙defaultにしない
- model不能時は別候補へ黙ってfallbackせず、fresh runのexecution envelopeへ変更理由を残す
- Cursorは通常Grok 4.5／Composer 2.5のfirst-partyだけを使い、GPT／Claude等のthird-party modelは対応direct channelが実際に利用不能な時の明示代用だけにする
- Grok施工後にGrok自身をreviewerへ再利用しない。Spark／Terra／Luna／SolはOpenAI familyなので相互を独立reviewと数えない
- LLMの自己test greenだけで採用せず、主担当が実diffとtask固有oracleを再実行する

## 粒を増やさない境界

Terraのorder compile、実装担当のconstruction step、review findingを別の製品粒として数えない。grainは一つのowner、意味、
allowlist、oracleを持つ一契約境界である。次の場合だけ新しいorderへ分ける。

- ownerまたはwriterが変わる
- 公開契約、Document意味、永続形式が増える
- allowlistまたはterminal consumerが別境界へ広がる
- negative oracleが別failure classを所有する
- return後のcurrent factで先に接続すべきedgeが変わった

## 速度の扱い

Grok第一候補化により、Composerの取りこぼしや再発注が減って総wallが短くなる可能性はあるが、現時点ではMotoliiの同条件A/Bで
証明していない。専用benchmarkのため不要なLLM callを増やさず、最初の実案件から次をexecution envelopeとreturnへ記録する。

- exact model／variant、role、capsule hash
- wall time、provider usage、tool stepが得られる場合は原値
- first-pass test、独立reviewのP0/P1、correction回数
- 最終採用、PARTIAL、CONTEXT_GAP、OBSERVATION_FAILURE

3〜5件の同程度closed implementationが自然に溜まった時点で、Grok／Composerの総wall、rework、採用率を比較する。単発成功や
provider自身のbenchmarkだけで固定model routeへ昇格させない。

## 非目標

- `Terra -> Grok -> Opus`を全task必須にする
- Composerを禁止または自動fallbackにする
- Cursor first-party poolとmodel family独立性を同一視する
- 安いmodelを使うためoracleやreviewを省く
- 比較件数を作るため不要な発注を増やす
