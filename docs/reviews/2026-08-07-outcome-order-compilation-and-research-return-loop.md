# 利用者成果からの発注コンパイルと調査返却ループ

日付: 2026-08-07
状態: **決定**
対象: Motoliiの計画、調査、外部LLM発注、実装、検収、次粒再選定

## 1. 決めること

Motoliiの発注は、task ID、候補技術、担当model、file一覧から始めない。通常製品routeの**利用者成果の背骨**を
先に固定し、現行authorityとcode fact、既知実装採択結果から、実装可能な一契約境界へコンパイルする。

発注の正規成果はコードだけではない。実装targetを閉じられない場合、検索場所、候補、採否、不適合理由、
exact gap、再入場条件を持つ**調査返却**を成果として受け取る。主担当監督は返却後、古い`next`へ戻らず、
現行codeから背骨上の次edgeを再選定する。

```text
利用者成果の背骨
  -> authority / current fact / known implementation
  -> closed order
  -> 実装成果 | 調査返却 | evidence gap
  -> 主担当の再照合と処分
  -> 現行codeから次edgeを再選定
  -> 次のclosed order
```

これは特定runner、CLI、model family、固定工程数のschemaではない。短いorderでも、この遷移を失ってはならない。

## 2. 二つの計画層と一つの監督loop

| 層 | 所有するもの | 所有しないもの |
|---|---|---|
| 成果の背骨 | 操作列、同一identity、成功出口、失敗回復、Undo/Redo、Preview/Export、reopen、external gate | 一枚の巨大実装order、全将来API、固定担当列 |
| closed order | 一つのowner、意味、exact target、allowlist、transition、正負oracle、STOP/RETURN、handoff | 背骨全体の完成宣言、隣接owner、未決の製品意味 |
| 監督loop | authority、order compile、返却の採否、再検索、次edge再選定、最終統合 | 実装担当への意味選択委任、外部LLM回答のauthority化 |

背骨は粒を大きくする許可ではない。closed orderは地図の行を転記したものでもない。監督loopは粒数を順に
消化するschedulerではなく、各return後に現在事実を測り直すrolling horizonである。

## 3. order compileの固定順序

主担当は次を順番どおり閉じる。

1. **USER OUTCOME**: 利用者が通常routeで完走する結果と失敗回復
2. **OUTCOME EDGE**: 今回接続する一つの`operation -> write/read -> projection -> feedback`
3. **AUTHORITY**: spec、decision、絶対規律、既存公開境界、非目標
4. **CURRENT FACT**: current mainの実在type、source、owner、writer、consumer、test、未接続
5. **MECHANISM CLASS / SURVEY**: repo、decision index、references、製品先例、必要時だけ一次資料
6. **DISPOSITION**: `REUSE / ADOPT / WRAP / PORT / PATTERN / EXTERNAL / REJECT`、または`REMAP / REDUCE`
7. **CONTRACT**: 一owner、一意味、stable identity、入力、transition、terminal、failure/cancel
8. **SCOPE**: exact read set、allowlist、非目標、共有ownerとの直列点
9. **ORACLE**: positive、negative、zero-write、stale/late、primary/repository/external gate
10. **STOP / RETURN / HANDOFF**: 何を実装せず、どのauthorityへ何を返し、何があれば再入場するか

[既知実装採択・置換開発モデル](../known-implementation-adoption-model.md)のpreflightは5〜6を所有する。
本書はその調査を利用者成果、closed order、return、再選定へ接続する。既決routeを継承できる場合、長い調査を
再生成せず正本pathと具体targetを参照する。

次を閉じられないorderは実装担当へ送らない。

- 利用者出口を削除しても残る「便利な技術導入」
- 実在ownerまたはterminal consumerがない
- allowlistが`必要なら追加`で開いている
- 正例しかなく、誤配線や第二ownerを直接失敗させられない
- STOP後の戻り先と再入場条件がない
- 外部LLMにadoption class、公開API、Document意味、代替設計を選ばせる

## 4. closed orderの最小形

欄名は対象正本の既存形式を再利用してよい。次の意味が欠落しなければ、毎回同じ長いtemplateへ展開する必要はない。

```text
OUTCOME / EDGE:
BASE / AUTHORITY / CURRENT FACT:
OWNER / EXACT TARGET / STABLE IDENTITY:
KNOWN ROUTE / DISPOSITION / THIN SEAM:
INPUT -> VALIDATE -> PREVIEW OR READ -> TERMINAL -> PUBLISH:
ALLOWLIST / READ SET / CAPSULE-EXTERNAL READ: FORBIDDEN / NON-GOALS:
POSITIVE ORACLE / NEGATIVE ORACLE / EXTERNAL GATES:
STOP / RETURN:
HANDOFF / REENTRY:
```

dynamic surfaceではmount、resize、focus、capture、unmount、late event、generation/epochをtransitionに含める。
永続編集ではsingle writer、journal、Undo/Redo、publish、reopenを含める。read-only粒ではzero-writeをprimary oracleにする。

## 5. return contract

### 5.1 実装返却

実装担当は有用性や自己評価ではなく、次を返す。

```text
RETURN_KIND: IMPLEMENTED | PARTIAL | STOP | CONTEXT_GAP
BASE / FINAL HEAD / STATUS:
DIFF / ALLOWLIST:
OWNER / WRITE ROUTE:
PRIMARY ORACLE / REPO LANES / EXTERNAL GATES:
NEGATIVE RESULTS:
FINDINGS:
HANDOFF CANDIDATE:
```

`PARTIAL`は採用資格ではない。dead route、片側だけのbridge、未閉鎖terminalをmainへ混ぜず、監督が再処分する。
`CONTEXT_GAP`は許可済みread setだけでは閉じない時に、必要なexact path、range、symbol、oracleを示す返却である。
実装担当はcapsule外read、repo横断探索、同じsessionへの全文追加で補わない。主担当が不足を現行sourceへ照合し、read setと予算を
更新したfreshな短wave、`REDUCE`、または別契約境界を選ぶ。

### 5.2 調査返却

実在targetを閉じられない場合、実装を捏造せず次を返す。

```text
RETURN_KIND: RESEARCH_RETURN
SEARCH_SCOPE: <repo path / decision keyword / references / primary sources>
CURRENT_FACT: <path:symbol または不在を示すexact evidence>
CANDIDATES: <候補と証明範囲>
ADOPTED: NONE | <routeとtarget>
REJECTED: <候補 :: oracle/license/platform/thread/owner等の不適合>
EXACT_GAP: <identity / owner / command / consumer / layout slot / test / contract>
DISPOSITION: REUSE | REMAP | REDUCE | RESEARCH_MORE | WAIT_TARGET | AUTHORITY_CONFLICT
REENTRY_CONDITION: <何が実在すれば再開できるか>
SAFE_PARALLEL_EDGES: NONE | <同じ背骨上で独立して継続できるedge>
```

単一keywordの0 hit、古い文書の不在、modelの「無い」は検索完了ではない。検索済みとするには、検索場所、候補、
採否、不適合理由が必要である。`TARGET_MISSING`は状態語だけで終えず、この返却を伴う。

### 5.3 review返却

reviewerは`ACCEPT`をauthorityにしない。証拠不足は`EVIDENCE_GAP: <path>:<range>`、契約違反はseverity付きfinding、
問題なしは監査範囲と非証明範囲を返す。主担当は原文、実diff、非LLM oracleへ再照合する。

## 6. 主担当の再選定義務

returnを受けた主担当は、次の順で処分する。

1. base、authority hash、実diff、scope、oracle、reviewer mutationを再確認する
2. findingを`scope内blocker / scope外finding / authority conflict / external gate`へ分ける
3. 調査返却を現行repoと一次資料へ再照合する
4. 背骨上で接続済みedgeと未接続edgeをcurrent codeから再計測する
5. `REUSE -> REMAP -> REDUCE -> 再調査 -> WAIT_TARGET`で当該edgeを処分する
6. file-disjointだけでなく、writer、Host ABI、GPU device、artifact publicationの衝突を見て並列可否を決める
7. 次の一契約境界をclosed orderへ再コンパイルする

完了前に書いた`next`、古い粒数、過去branch、外部LLMのhandoff候補をそのまま次orderにしない。
`WAIT_TARGET`は該当edgeの局所状態であり、同じ背骨上の独立edgeを停止しない。利用者判断なしに意味を選ぶ必要があり、
安全な別edgeも無い場合だけ利用者へ返す。

## 7. 外部LLMの使い方

model名をloopの役割へ固定しない。主担当がauthority、scope、oracle、最終採否を所有し、対象の閉じ具合と視野幅から
調査、施工、correction、reviewの担当を都度選ぶ。

- 調査: 条件と非目標を渡し、候補、反証、一次資料、非証明範囲を返させる
- 施工: closed orderとallowlistの範囲だけを変更する
- correction: 同じ契約境界のexact findingだけを直す
- review: fresh/read-onlyで実diff、負例、境界漏出を監査する

固定model列、黙ったfallback、長寿命session、modelの賛同による採択、実装担当への再委任許可を作らない。
途中stream、fresh session、blind evidence envelope、effort選択は各現行監督正本に従う。

closed orderを実際のCLIへ渡す時は、order本文へ変動するmodel名とflagを焼き込まず、別の**execution envelope**で
`allocation profile / limit group / model family / role / exact model / effort / permission / CLI version / log dir`を閉じる。
全呼出しはprovider-native途中streamを有効化し、生stdout/stderr、lifecycle、exit、hash、provider固有terminal resultを保存する。
具体的な引数、主要model候補、`balanced / codex-conserve / cursor-conserve / claude-conserve`の可変配分は
[外部LLM発注の観測・実行・可変配分runbook](../llm-dispatch-observation-and-allocation-runbook.md)を運用正本とする。

配分weightは適格候補間のsoft targetであり、fixed routeや自動fallbackではない。どの利用枠を消費するかという`limit group`と、
独立reviewを判定する`model family`を分ける。Codex枠が逼迫した時も、repo sweep、候補order、施工、別family reviewを外へ寄せ、
Codexはauthority再照合と最終採否へ縮約できるが、その責任自体を移譲しない。

## 8. M3 RN runtimeへの適用

[M3 RN runtime実行地図](../m3-rn-runtime-execution-map.md)のwaveは成果の背骨、nodeはorder compile候補である。
`DO`以外をコード発注しない一方、`COMPILE`と`TARGET_MISSING`を静的な待ち一覧にしない。

- `VERIFY_CANDIDATE`: diff review、current-main再現、oracle結果を実装返却形式で閉じる
- `COMPILE`: exact target、transition、allowlist、negative oracleの不足を調査返却で閉じる
- `TARGET_MISSING`: 前ownerへexact gap、候補、reentry、safe parallel edgeを返す
- `SPEC_ONLY`: 一つの製品意味と反例をdocs-onlyで閉じ、実装handoffを分離する
- `ADOPTION_PROBE`: platform/upstream証拠と不採用理由を返し、製品接続しない

R0〜R4の地図を一度書いたことで再選定を終了しない。各return後に同じ利用者出口へ最短で接続できるnodeを測り直す。

## 9. 成立史と較正例

この手法は旧M3固有の成功例ではなく、次の部品が合流した横断発注契約である。

1. [Rerun学習・転移計画 §9](2026-07-20-rerun-learning-transfer-plan.md#9-rerun参照を発注へ入れる強制動線): 外部技術をMotolii要件へ逆流させない
2. [快適利用Work Map](2026-07-22-m3-comfortable-use-work-map.md): UI部品一覧より利用者の一本道を上位に置く
3. [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md): owner、正負例、STOPを持つ発注母集団
4. [U0e-2事故と現行読替え](2026-07-22-u0e-2-delegation-guardrails.md): base、authority、allowlist、因果oracle、証跡
5. [縦slice実行方針](2026-07-24-m3-vertical-slice-execution-decision.md): `M3 STOP / RETURN`と`M3 HANDOFF`
6. [既知実装採択モデル](../known-implementation-adoption-model.md): 一般化した検索preflightとadoption map
7. [runner非依存監督](2026-08-03-runner-independent-supervision-decision.md): model/runnerからauthorityを分離
8. session `019fcbf0-396a-71c1-bf86-81533ff5c7b4`: Easingの接続成功とP07の`TARGET_MISSING / BUILD FORBIDDEN`を同じ自律loopの正規成果として実証

成功較正は、Position key、playhead、active interval、Interp command、React Host、native popupを一契約ずつ接続した
Motion Authoring Loopである。停止較正は、PlaybackSession等の4 route不在を捏造せず具体的gapへしたP07である。
前者だけを模倣すると作りすぎ、後者だけを模倣すると止めすぎになる。

## 10. 完了と非目標

一反復の完了は、次のいずれかである。

- **接続完了**: 一契約境界がmainへ到達し、primary oracleと必要laneを通り、次edgeを再選定した
- **調査完了**: 調査返却がexact gap、候補、採否、reentryを持ち、別edgeまたは局所WAITを再選定した
- **外部gate化**: 実機、人間、providerだけが残り、未実行をPASSへ上げず別台帳へ送った

非目標:

- 一つの巨大order、固定model pipeline、全task共通runner/state machineを作る
- 全return欄をtransport schemaや恒久receiptへする
- 調査成果をコード未生成という理由で失敗扱いする
- `TARGET_MISSING`を理由に仮owner、仮UI、第二writer、一般frameworkを作る
- 背骨の一部greenをphase完成へ繰り上げる
- 歴史的な旧M3 ID、renderer、model選択を現行authorityへ復活させる
