# Claude low CLOSED review較正観察

状態: **観察**（`CLOSED=low`の運用根拠を補強するが、能力の一般保証や新しいruntime gateではない）

日付: 2026-08-03

## 問い

同じbounded reviewer packetでClaude Opus 5のeffortだけを変えた時、`low`は既知findingとclean判定を保持できるか。また、
Claude Codeのturn／budget制御を現行binaryで実際に使えるか。

## 条件

- 完全model ID `claude-opus-5`、fresh session、`--no-session-persistence`、safe mode、Read-onlyを固定した
- `scripts/run-observed-cli.py`でprovider-native `stream-json`を途中観測・保存し、同一packetではeffort以外の条件を揃えた
- 合成packetは一契約境界、exact snippet、既知oracleへ閉じ、seeded 3 findingとfully closed cleanを各effort 2回ずつ実行した
- 厳格schemaは各effort 1回ずつ別測定し、構造化出力自体が追加するcycleを分けて観測した
- 保存済みの実diffと当時の判定から、機械粒2件と意味粒2件をbounded packetへ再構成し、`low`を各2回実行した
- cwd誤りでRead permission deniedになった初期3試行は無効として除外した。guard本文がなくcommentだけだった最初のclean候補も、
  false-positive測定へ数えず「packet不完全」の反例として扱った

これは過去session全文の再実行ではなく、保存済みdiffと既知oracleを使うbounded replayである。raw logの一時pathや料金値は
time-specificな実験証跡であり、長期authorityやmodel score DBにしない。

## 結果

| 対象 | effort／反復 | 結果 | wall／costの観測範囲 |
|---|---:|---|---|
| seeded 3 finding合成packet | low／medium／high 各2回 | 全6回で3違反を検出。lowは1回重複列挙、mediumは1回unsupported P2、highは追加findingなし | low約7.5–9.7秒／$0.034–0.047、medium約14.3–14.9秒／$0.045–0.054、high約11.7–12.6秒／$0.042–0.055 |
| fully closed clean合成packet | low／medium／high 各2回 | 全6回`ACCEPT`、P0/P1/P2=0 | low約10.3–10.6秒／$0.040–0.042、medium約20.5–26.4秒／$0.058–0.073、high約24.8–38.6秒／$0.070–0.089 |
| strict JSON Schema | 各effort 1回 | 全3回でexact structured outputに成功し、各3 turn | low 7.35秒／$0.0648、medium 21.19秒／$0.0911、high 33.52秒／$0.1103 |
| P2D-RCB6 accepted history | low 2回 | 2/2 `ACCEPT`、P0/P1/P2=0 | 約8–22秒／$0.025–0.034の過去粒群範囲 |
| P2D-RCB5 rejected history | low 2回 | 2/2 `REJECT`、既知の重複A1を同じP0として検出 | 同上 |
| CU-210P paused ruler seek accepted history | low 2回 | 2/2 `ACCEPT`、P0/P1/P2=0 | 約8–22秒／$0.026–0.034の過去粒群範囲 |
| PR #437 rejected history | low 2回 | 2/2 `REJECT`、compile-only helperのsynthetic reachabilityという既知P1根因を検出。2 file分へ分けて列挙した | 同上 |

通常の自由回答は`Read → answer`で2 turnだった。strict schemaは`Read → StructuredOutput → completion`で3 turnとなった。
mediumのclean反復では同じfileを再読して3 turnになった例があり、turnはfile数や静的tool call数では決まらない。

Claude Code 2.1.216と、同日`npx`で確認したnpm latest 2.1.220の実binary helpには`--max-turns`が無かった。一方、
[Claude Code CLI reference](https://docs.anthropic.com/en/docs/claude-code/cli-usage)には同optionの記載がある。運用時は一次資料の
記載だけでなく起動するexact binaryのhelpを優先し、存在しないhard turn capを渡さない。`--max-budget-usd`、`--json-schema`、
stream outputは実binaryで確認した。

## 解釈

1. exact diff、契約、oracleが閉じた`CLOSED` reviewerでは、`low`を通常候補にする根拠が得られた。
2. lowはaccepted／rejectedの機械粒と意味粒を今回のbounded replayで再現した。単純な文字照合だけへの限定根拠ではない。
3. 最初のclean候補の失敗はeffort差よりpacket closureの不足を示した。前提検索やevidence capsuleが弱い時はeffortを上げず、
   Solへ戻して不足証拠を閉じるか粒を分ける。
4. この観察は`ADJACENT / WIDE / CONFLICTING`、未解決authority、全製品領域、または将来versionでの同等recallを保証しない。
5. strict schemaは形を閉じられるがtool-result cycleとcostを追加する。全reviewへの必須化はこの観察から決めない。

## 非目標

- 全task共通の固定turn、token、wall、cost閾値を新設すること
- `low`を未閉鎖調査や共有・恒久境界へ自動適用すること
- `medium / high / xhigh`を廃止すること
- 新しいeval harness、receipt DB、model score、自動effort昇格を実装すること
- LLM判定をbench、negative test、schema、platform oracleの代わりにすること
