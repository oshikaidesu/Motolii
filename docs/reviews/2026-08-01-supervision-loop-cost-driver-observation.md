# 監督ループの速度支配項と計装

状態: **観察**

日付: 2026-08-01

非目標: 独立検収の省略・縮退。本書は「独立検収を制約条件として固定した内側で、
どこに時間とコストが消えているか」だけを扱う。

## 1. 発端

ユーザーが挙げた不満は三つだった。

1. 他LLMの応答が遅い（毎回セッションを一から聞いている）
2. ログが取れていない
3. コストが割高

runnerの実装事実を確認した結果、三つとも原因の帰属が誤っていた。

| 不満 | 実装事実 | 実際の原因 |
|---|---|---|
| 毎回一から | Opusは毎回新規`--session-id`、Grokはsession指定なし、Sparkは`--ephemeral` | **仕様通りかつ必須**。Grokのfresh sessionはtechnical independenceの条件そのもの |
| ログが無い | `run_agent`はstdout/stderrをファイルへ書いている | **不在ではなく非ストリーム**。実行中は30秒ごとの`実行継続中`だけ |
| 高い | `usage`/`total_cost`を扱う箇所がrunnerに一つも無かった | **測っていないので下げられない** |

## 2. 支配項は latency ではなく rework

| 経路 | wall time | Spark tokens | 採用 |
|---|---:|---:|---:|
| 930行・4契約境界の粒（1回目） | 430秒 | 216,443 | REJECT |
| 同（2回目） | 586秒 | 251,617 | REJECT |
| 同（停止線追加後の3回目） | 19秒 | 0 | 骨格差戻し |
| 一行fixture | 62秒 | 10,070 | **ACCEPT** |

複雑粒は合計1,016秒・468,060 tokensを消費して**採用ゼロ**。一方で良い粒は62秒でACCEPTしている。
遅いのはモデルではなく、やり直している。

`session resume`は自前実測で fresh 5.87秒 / resume 5.49秒（6.5%差）、総input tokenはむしろ増加
（18,421→37,964）。**resumeは支配項ではない。**

## 3. 文献監査 — 1回目の調査は目的関数を誤っていた

2026-08-01の1回目の調査は次の系統だけを検索していた。

```text
LLM routing cascade / token efficient orchestration / FrugalGPT / RouteLLM /
selective escalation / adaptive verification / SWE-bench multi vs single agent
```

全クエリが`cost / token / latency / cascade / routing`軸で、`independent` `separation of duties`
`second party` `adversarial`は一語も含まれていなかった。この文献群で`verifier`は
**同一システム内の自己検証部品**を指し、実装者と別人格の監査者という概念が存在しない。
したがって素直に読むと必ず「reviewerを省ける」に着地する。**推論ではなくクエリの時点で結論が決まっていた。**

| 文献 | 転移条件 | 判定 |
|---|---|---|
| CodeAgents (typed通信で入力token 55–87%減) | 成立。独立性を損なわない | 採用（Motoliiは既に実装・自前実測229.1→38.4秒の方が強い証拠） |
| Agentless (決定的3段) | 既に実施済み。`Validation`は機械oracleであり独立検収ではない | 縮小 |
| FrugalGPT / RouteLLM | 検収席へは不成立（独立性という制約が存在しない）。数値もQAデータセット由来 | 棄却（検収席）／採用（助言席のみ、既存規約と同内容） |
| RLM-Cascade | preprint、評価規模限定、自己申告 | 棄却 |
| LLMCompiler | 検収は最終diffを見るため直列。read-only検収中のfingerprint変化は既存gateが`exit 8`で拒否 | 縮小 |
| Anthropic multi-agent research（約15倍token、依存の強いcoding taskに不向き） | 成立 | 採用（**最も関連が高いのに方針へ反映されなかった**） |

## 4. 制約条件を明示した2回目の調査

問いを「どの検収を省けるか」から「**独立の機械的定義と、独立が必須になる境界**」へ変えた。

- **IEEE 1012 / NASA IV&V**: 独立は technical / managerial / financial の3軸。technical independenceの中身は
  「別モデルを使う」ではなく「**問題理解を自分で再構成する**」。実装者の推論を渡した時点でmodel IDが違っても
  独立性は失われる（[NASA SWE-141](https://swehb.nasa.gov/display/SWEHBVC/SWE-141+-+Software+Independent+Verification+and+Validation) 全文確認）
- **DO-178C**: 独立とは「検証者は当該成果物の作成者であってはならない」。独立を要求する目標数は
  Level A=31中16、Level B=31中7、Level C/D=0（検索確認）
- **IEC 61508**: SILに応じて 独立した個人 → 独立した部門 → 独立した組織 の梯子（検索確認）
- financial independenceはMotoliiへ**転移不可**

先人は「独立を省くか」ではなく「**どこまで遠い独立を要求するか**」を危険度で決めていた。
Motoliiは`HAZARD_TAG`と`CONTRACT_IMPACT`を既に持っており、これが梯子の入力になり得る。

## 5. LLM検収者の規模劣化（2026年の実測）

- [arXiv 2606.15689](https://arxiv.org/abs/2606.15689)（5モデル・150サンプル）:
  F1が diff 10行未満で **0.657**、150行超で **0.043**。実PRのみでは最良でもF1 0.066（合成0.847から92%劣化）。
  **Haiku 4.5がSonnet 4.6を上回る**（F1 0.365 vs 0.343、recall +18%、コスト3.2分の1）。
  **全モデルで性能関連バグのrecallがほぼ0**
- [LongCodeBench](https://arxiv.org/html/2505.07897v2): Claude 3.5 SonnetはLongSWE-Benchで32K→256Kで29%→3%。
  context窓の拡大は解決しない
- [Google eng-practices Small CLs](https://google.github.io/eng-practices/review/developer/small-cls.html) は
  実証研究の引用が無い人間向け実務ガイドで、リポジトリは2025-11-21にアーカイブ済み。
  「100行は概ね妥当、1000行は概ね大きすぎる」の緩い数値を self-contained と**ファイル分散**へ従属させている

Motoliiの930行・4境界の粒は F1≒0.043 の領域にあった。**2回のREJECTは検収が働いた例に見えるが、
この領域では同時に見逃しも起きていると読むべきである。**

## 6. Fable 5による解法の掃引（2026-08-01、read-only）

「独立検収を維持したまま大diffの検出率を回復させる実証のある手法」を7仮説で掃引した結果:

| 仮説 | 結果 | 判定 |
|---|---|---|
| 機械的分解（untangling） | 分解精度は測られているが**下流の検収品質を測った研究が無い**。唯一の統制実験（di Biase et al. 2019、被験者28名）は「誤報は減るが発見欠陥数は変わらない」 | 延期 |
| context選択・slice化 | 本番デプロイ実証あり（[2505.17928](https://arxiv.org/html/2505.17928)、短いsliceが最大文脈に勝つ）。ただしKey Bug Inclusionは天井約31% | 採用 |
| 役割分解した複数検収者 | **PR規模では負の証拠**。[SWR-Bench](https://arxiv.org/html/2509.01494v1)（実PR 1,000件）でmulti-agent 9.22% vs 単純prompting 19.38% | 縮小採用 |
| 反復サンプリング | PR規模で唯一の正の実証（Multi-Review、相対+43.67%）。ただし**多数決は禁止、union集約のみ**。絶対天井は約0.28 | 縮小採用 |
| 静的解析hybrid | [IRIS (ICLR 2025)](https://arxiv.org/abs/2405.17238) でCodeQL単体27件→55件。[CodeQLのRust対応は2025-10-14にGA](https://github.blog/changelog/2025-10-14-codeql-scanning-rust-and-c-c-without-builds-is-now-generally-available/) | 採用（security/UB系限定） |
| 検収特化fine-tuned model | 大diff劣化の軽減実証が**見つからない**。token長制約でむしろ不利 | 棄却 |

**銀の弾丸は存在しない。** 実PR規模の最良値はどの手法でもF1 0.19–0.28帯で、小diffの0.657へ回復させた
実証は見つからなかった。**大きな効果量を持つ唯一の変数はdiffサイズ自身であり、それは検収側でなく
発注側の粒度上限にある。**

さらに[Correlated Errors in LLMs (ICML 2025)](https://arxiv.org/abs/2506.07962)は350+モデルで
**両モデルが誤る場合の60%は同じ誤り**、しかも大型で高精度なモデルほどprovider差があっても相関が高いと報告する。
これは独立検収を否定しないが、「強いモデルを検収に置けば安心」を否定し、**真に相関しない軸は非LLM oracle
（型・静的解析・実行）**であることを示す。

## 7. 計装（実装済み・2026-08-01）

`scripts/delegate-cursor-supervised.sh`へ計装を入れた。ゲート・model routing・独立性は一切変更していない。

- 各stageの`WALL_SECONDS / INPUT_TOKENS / OUTPUT_TOKENS / CACHE_READ_TOKENS / COST_USD / API_MS`を
  telemetry receiptへ記録（`prepare`は`<order>.evidence/prepare-telemetry.txt`、
  `execute`/`inspect`は`<attempt>/telemetry.txt`）
- 失敗・timeout・REJECT・骨格差戻しの経路でも必ず記録して合計を出す
- 測れないstageは`UNKNOWN`と明記し、合計行に`UNMEASURED_TOKEN_STAGES`として欠測stage名を残す
- stage完了ごとにstderrへ1行出し、実行中に何も見えない状態を塞ぐ
- field名はevidence receipt限定。OpenTelemetry GenAI規約は2026-07時点でDevelopmentのため、
  恒久形式・公開契約へは焼かない

Claude CLIの`--output-format json`は`total_cost_usd`と`usage`を**既に返していた**。従来は
`.structured_output`だけを読んで捨てていたため、新しいflagも依存も追加していない。

### 実測（実`claude-opus-5`、最小の閉じた粒、n=3）

| 回 | wall | api_ms | uncached input | cache_read | output | cost |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 13秒 | 11,009 | 2 | (cache creation) | 637 | $0.0619 |
| 2 | 12秒 | 11,572 | 2 | 987 | 665 | $0.0532 |
| 3 | 10秒 | 10,177 | 2 | 987 | 567 | $0.0508 |

この実測から二つ分かった。

1. **prompt cacheが既に効いている。** 毎回fresh sessionにもかかわらず2回目以降のcache_readは987 tokens、
   非cache入力はわずか2 tokens。**「毎回セッションを一から聞いている」という懸念は、少なくともOpus段では
   既に緩和されていた。** session resumeが6.5%しか効かなかった理由と整合する
2. **P0/P1骨格差戻しは実測10〜13秒・約$0.05。** 430〜586秒のSpark実行と比べて桁違いに安い経路が実データで
   確認できた。3回とも独立に同じ欠陥（`git diff --check`ではdone/not-doneを判別できない）を指摘しており、
   Opus段の再現性は高い

## 8. 次に検討する候補（未決・実装しない）

本書は観察であり、次は決定ではない。実施には別途裁定が要る。

1. 契約境界が2つ以上の粒をdispatch前に分割する機械gate（既存規約`AGENTS.md`の一契約境界の機械化）
2. 実diffが劣化領域にある場合、`VERDICT: ACCEPT`を有効な採用根拠にしない有効性条件
3. 独立性の梯子の最上段を「別provider」だけでなく「別provider＋非LLM oracle」とする
4. 性能regressionはLLM検収の守備範囲外と明記し、bench/golden oracleで持つ
5. Spark(`codex`)とGrok(`cursor-agent`)のtoken/cost計測（現在`UNKNOWN`）

## 9. 導線 — 過去のやり方へ戻さない

- **コスト・token・wall timeを人が画面から書き写さない。** runnerのtelemetry receiptを正本にする。
  本書§2の1,016秒/468,060 tokensは計装前の手写し値であり、以後この方式で新しい数値を作らない
- **速度の評価指標にwall timeを単独で使わない。** 指標は
  [発注パイプライン並列化の比較案 §8](2026-07-23-parallel-order-pipeline-comparison.md#8-速度と品質の測定案)
  の表（lead time / wait time / first-pass accept / rework count / stale-base count / escaped finding /
  Codex integration load）を使う。同書は**ARCHIVED**だが、アーカイブ対象は旧model配置と複数実装lane案であり、
  **§1の診断（工程間の待ち時間が全体速度を支配する）と§8の指標表は棄却されていない**。新しい指標語彙を発明しない
- **`session resume`・model routing・reviewer変更で速度を解決しようとしない。** 本書§2と§7で反証済み
- **調査を規約改訂へ直結させない。** 1回目の失敗（§3）は制約条件を言語化する前に検索したことが原因であり、
  検索の巧拙ではない。[docs/reviews/README.md](README.md)規律1・6を先に通す
