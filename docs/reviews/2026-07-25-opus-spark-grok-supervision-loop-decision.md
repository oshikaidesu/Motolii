# Opus 5 / Spark / Grok 監督ループ

状態: **決定**

日付: 2026-07-25

速度改訂: 2026-07-30

scope自己反証追補: 2026-07-31

## 決定

通常の実装発注は、次の単一ループへ固定する。

```text
Codex → Claude Opus 5 → Codex Spark → Cursor Grok 4.5 High → Codex
```

| 段階 | model | 責任 |
|---|---|---|
| 契約 | 主担当Codex | 仕様、コード事実、親task、変更可能境界、STOP条件、最終採否を所有する |
| 施工管理 | `claude-opus-5` | Codex／runnerが埋めた骨格を検証し、見落としたrisk／負例／STOP／訂正だけを型付きdeltaで返す |
| 施工 | `gpt-5.3-codex-spark` | 承認済みの一粒だけを隔離worktreeで実装し、必須試験を実行する |
| 独立検収 | `cursor-grok-4.5-high` | 実diffと試験をread-onlyで監査し、P0/P1の有無と`VERDICT`を返す |
| 統合 | 主担当Codex | Grokの結果を未検証の助言として再照合し、採用、差戻し、STOPを決める |

Opus 5の委任権は一段だけであり、Spark以外を起動せず、Sparkも再委任しない。一回のrunner実行は一つの
`GRAIN`だけを扱う。複数粒が必要なら、主担当Codexが各粒の契約境界を確認した上でループを個別に回す。

Fable 5は通常ループの段階または必須gateにしない。大地図、設計比較、共有公開境界など、主担当Codexが
高難度の反対側助言を必要と判断した場合だけ、通常ループの外からread-onlyで直接呼ぶ。

## 速度改訂 — 読解を三重化しない

通常粒の遅延原因をmodel固有の思考速度だけに帰さない。従来promptは、Opus 5に`AGENTS.md`と全authorityと
worktree探索、Sparkに同じ規約とsource再読、Grokにauthority再読と広域監査を順番に要求していた。代表的な
入口だけでも`AGENTS.md`、implementation ledger、M3 spec、React移管契約を合わせて100 KiBを超え、sourceを
読む前から三段で同じ固定文脈を消費していた。

2026-07-30以降、広域読解は主担当Codexが一度だけ所有する。通常発注のtaskは、確定成果、対象粒、正本から
検証したコード事実、既存owner／writer、変更不能境界、候補fileを含む小さな入力packetとする。Opus 5は
repoを再調査する設計者でなく、そのpacketを一つのclosed orderへ圧縮する施工管理者に戻す。SparkとGrokは
承認済みorderを完全な規律カプセルとして使い、明記されたread setと実diff／試験だけを読む。

runnerは通常粒へ次の機械予算を課す。

| 項目 | 上限／条件 |
|---|---|
| task本文 | 12 KiB以下 |
| order本文 | 32 KiB以下 |
| `AUTHORITY:` | 1〜4件 |
| `ALLOWED_FILE:` | 1〜8件 |
| `READ_FILE:` | 1〜12件、合計128 KiB以下、worktree内の既存exact file |
| `INTERNAL_TARGET:` | 1〜4件、`<READ_FILE path> :: <一意なtrim済み1行>`、`ALLOWED_FILE`内 |
| `TEST_TARGET:` | 1〜4件、`<READ_FILE path> :: <一意なtrim済み1行>` |
| `REUSE_TARGET:` | 1〜4件、`<READ_FILE path> :: <一意なtrim済み1行>` |
| `NEW_SURFACE:` | 通常粒は`FORBIDDEN`を一つ |
| `READ_MODE:` | `CAPSULE`を一つ |
| `CONTEXT_FACT:` | 1件以上 |

`READ_FILE`は外部modelが実装・検収のため開いてよい集合であり、glob、絶対path、`..`、symlinkを許さない。
三種のtargetは空白を除いた完全な一行が対象file内に一度だけ現れることをrunnerが照合する。runnerはtargetごとに
前40行＋後80行を抽出し、同じpath／anchorを重複除去した48 KiB以下のコードcapsuleをSparkへ先渡しする。
Sparkはwhole fileでなくこのcapsuleから開始し、直接のcaller／test依存が不足した場合だけ同じ`READ_FILE`内の
別の有界近傍を読む。`NEW_SURFACE: FORBIDDEN`では新command、mode、公開API、長寿命owner、汎用helper、並行経路を
作らず、既存targetで閉じなければSTOPする。
authorityのpathとhashはCodex precheckとrunnerが照合するが、authority全文をSpark／Grokへ再読させることを
意味しない。適用規律と関連するauthority事実は`CONTEXT_FACT`およびorder本文へ閉じ込める。カプセルと実コードが
衝突した場合は外部modelが意味を発明せず差戻し、主担当Codexが正本へ再照合する。

公開API、Document意味、plugin契約、永続形式、共有公開境界を変更するにはこの予算内の通常粒へ押し込まず、
Codexが先に独立した仕様・決定粒を閉じる。速度改訂はReact／Rerunの必須ラベル、authority hash、allowlist、
負例、P0/P1=0、Codex最終採否を弱めない。

typed delta移行前のOpus 5 order draftは低effort・session非永続、既定timeout 300秒を維持する。移行後は
delta validatorを低effortで起動し、schema差戻し一回の間だけ同じsession IDを保持して、成功または二回目の
失敗後に破棄する。Grokの既定timeoutは240秒、Sparkは実装量の差が大きいため1800秒を維持する。timeoutは
別modelへのfallbackを起こさず、Codexへ戻してpacket不足、scope過大、CLI障害、実装難度を分類する。

## 速度改訂2 — 全文orderを型付きdeltaへ置換する

GR-D3の既知問題を同一の`claude-opus-5`、同一capsule、`high` effort、toolなしで再比較した。

| 接続形式 | wall time | Opus出力 | cost |
|---|---:|---:|---:|
| 完全なclosed orderを全文生成 | 229.1秒 | 16,404 tokens | $0.468 |
| 上限なしJSON delta | 115.5秒 | 7,827 tokens | $0.265 |
| 最大5件・各220文字のJSON delta | 38.4秒 | 2,206 tokens | $0.116 |

厳格deltaは全文生成比でwall time 83.2%、出力86.6%、cost 75.3%を削減した。短縮だけでなく、空集合、
unset root、symlink判定順、Mac Bash、引数引用を発見した。一方、単一writer下のTOCTOUやfilesystem上
成立しない重複entry等の過剰指摘も残り、歴史上の正確な`[@]:-`欠陥をtoken列としては特定しなかった。
したがって外部modelの鋭さを既知guardの代替にせず、既知の危険構文、正本値、base、authority、allowlist、
read set、実行commandはrunnerが機械骨格へ埋める。Opusは次のschema相当のdeltaだけを返す。

- dispositionは`READY / STOP / ESCALATE`
- findingは最大5件、各220文字以内
- kindは`RISK / NEGATIVE_ORACLE / STOP / CORRECTION`
- schema違反は同じsessionへ一回だけ差戻す
- 二回目も不正ならCodexへ戻し、散文fallback、全文再生成、別model fallbackをしない
- P0/P1 findingはSparkへ渡さず、Codexが骨格とexact oracleへ織り込んで`prepare`を再実行する
- P2 findingを採用する場合も`F1..F5`を付け、各IDに一つの`DELTA_RESOLUTION:`が対応するまでdispatchしない
- Opusは`BASE_SHA`、authority、allowlist、read set、hazard guardをdeltaで変更できない
- runnerはCodexが採用したdeltaだけを骨格へ結合し、完成orderをSparkとGrokへ渡す

### 視野幅と施工粒度を分ける

短いdeltaは施工接続であり、問題発見の視野を常に狭くする命令ではない。CodexはOpus起動前に次をコード事実で
埋め、`VIEW_PROFILE`を機械算出する。

| 軸 | 閉鎖 | 隣接 | 広域 |
|---|---|---|---|
| `AUTHORITY_SPAN` | `ONE` | `MULTIPLE` | `CONFLICTING` |
| `OWNER_CLOSURE` | `CLOSED` | `MULTIPLE_KNOWN` | `UNKNOWN` |
| `CAUSE_CLOSURE` | `LOCALIZED` | — | `COMPETING / UNKNOWN` |
| `CONTRACT_CLOSURE` | `PRIVATE / FROZEN` | — | `UNRESOLVED` |
| `ORACLE_CLOSURE` | `CLOSED` | `PARTIAL` | `ABSENT` |
| `REUSE_CLOSURE` | `REUSE` | `CHOICE` | `NEW` |

`CONTRACT_IMPACT: PRIVATE / SHARED / PERMANENT`は視野幅でなく影響種別として別に持つ。private粒は
`CONTRACT_CLOSURE: PRIVATE`と`CONTRACT_AUTHORITY: NONE`を使う。shared／permanent境界を施工可能な
`FROZEN`とするには、`CONTRACT_AUTHORITY: <path>@SHA256:<hash>`が同じorder内でpathとSHA-256を照合済みの
`AUTHORITY: <path> SHA256:<hash>`一件と完全一致しなければならない。正本が未決、衝突中、またはreceipt不在なら
`UNRESOLVED`であり`WIDE`とする。
これにより、恒久形式へ触れるという危険度だけで決定済み施工を永久に停止せず、未決恒久形式を`PRIVATE`と偽って
通すことも防ぐ。

- 全軸が閉鎖列なら`VIEW_PROFILE: CLOSED`
- 広域列が一つでもあれば`VIEW_PROFILE: WIDE`
- それ以外は`VIEW_PROFILE: ADJACENT`
- 未調査、不明、証拠欠落を閉鎖列へ推定せず、狭いprofileへの手動overrideを許さない
- `ADJACENT`はCodexが隣接caller／consumer／helper／試験をcapsuleへ追加してから厳格deltaへ送る
- `WIDE`はSparkへ送らない。Opus、共有境界・長期展望ならFableへread-only探索を依頼し、authority、
  owner、原因、oracleを閉じた新しいcapsuleを作って再分類する
- 広域探索の出力をそのまま実装orderにせず、最終的なSpark施工は必ず一つの閉じた契約境界へ戻す

視野幅と危険度は別軸とする。`HAZARD_TAG: DESTRUCTIVE_FS / SECURITY / PERSISTENCE / CONCURRENCY /
PLATFORM / NONE`を付け、tagごとの既知負例、禁止構文、機械lintを骨格へ注入する。危険だがauthority、owner、
原因、oracleが閉じたderived-target cleanup単粒を無条件に`WIDE`へ戻さず、危険でなくても共有契約や原因が未閉鎖なら
`CLOSED`へ押し込まない。

### 移行状態

本節の方式は2026-07-30に運用決定し、**2026-08-01に発効した**。発効条件は「typed delta、六軸分類、
hazard guardの機械gateが専用runner試験とともにmainへ入る」ことであり、main `3ce9d169`
(PR #436 compiled-spark-grain-integration)で満たされた。main上の
`scripts/test-delegate-cursor-supervised.sh`は`PASS`し、`scripts/validate.sh`から起動される。
以後この方式を「未発効」として扱わない。移行前の実行を新方式による速度改善の証拠に数えず、発効を理由に
authority hash、scope closure、独立検収、Codex precheckを弱めない。**発効したのは接続方式であって、
複雑粒の一発合格でも速度改善でもない**。

2026-07-30の未commit worktree実装では、専用runner試験で`STOP`、`READY`、同一session一回再試行、
二回schema不正、CLI失敗時の再試行禁止、P0/P1の骨格差戻し、P2 resolution必須化、
`CLOSED / ADJACENT / WIDE`、狭いprofileへのoverride拒否、`DESTRUCTIVE_FS` guard、予算超過、
Spark→Grok順、Grok mutation拒否を通した。さらにHEAD
`2a3e7812808cd0bbe7dcc62ea3f1d48a88cb99e9`から作ったdetached worktreeで実CLIの`prepare`を実行し、
29秒、exit 0、`READY`、3 findingsで完成orderを生成した。order SHA-256は
`382989059df210f9107afb39c5290cad694c8f461a5c91c7c84ef59918c5a3f2`。これは新接続のrunner入口が
実動する証拠である。

同じGR-D3で実modelの`execute`も二回行った。初回は430秒、Spark 216,443 tokens、Grok P1=1で
`VERDICT: REJECT`／exit 4。Sparkが`READ_FILE`外のambient memoryを読み、Opusのtracked／escape負例を
実装しなかった。ambient user config／memory／plugin／app／multi-agentを無効化し、各findingへ
`DELTA_RESOLUTION`を追加した二回目も586秒、Spark 251,617 tokens、Grok P1=2で
`VERDICT: REJECT`／exit 4だった。memory越境は消えたが、巨大な二fileの読解、新しい`cleanup` subcommandの
発明、負例の観測不足は残った。したがって厳格deltaはOpus入口を約27〜29秒へ短縮するが、ループ全体の速度改善や
一発合格をまだ証明しない。P0/P1を後段へ流さず骨格へ戻す停止線をこの実測から追加し、symbol／range単位の
read capsuleと既存internal target／reuse targetの機械gateは次のrunner粒で閉じる。Spark／Grokの実疎通は
証明したが二回とも不採用であり、commit、main統合、運用発効の証拠には使わない。

停止線追加後、同じ実packetを三度目の`prepare`へ入れると19秒でP0=1／P1=2を受け、exit 3、
dispatchable order未生成で終了した。これはP0/P1をSparkへ渡した二回の430秒／586秒を、約20秒の骨格差戻しへ
置換できる実runner証跡である。findingを解消した一発合格やループ全体短縮の証明ではない。

この実測から、未commit runnerへexact internal/test/reuse target、`NEW_SURFACE: FORBIDDEN`、48 KiBの
runner生成target capsuleを追加した。専用fixtureはtarget不在と新surface要求をOpus起動前に拒否し、Spark promptへ
whole-file禁止とtarget近傍が渡ることを確認する。

同じgateを持つ一行置換fixtureで実modelを再計測した。Opus `prepare`は12秒、P2=1をresolutionへ閉じた。
初回executeは71秒、Spark 6,211 tokens、Grok `ACCEPT`だったが、Spark自身は必須試験を実行せずGrokが再実行した。
そこで全`Test:`と`DELTA_RESOLUTION`のobservable checkをSpark終了条件へ追加した。二回目executeは62秒、
Spark 10,070 tokensで全必須試験を自ら実行し、Grok P0/P1/P2=0、`ACCEPT`、exit 0となった。order SHA-256は
`1b2eb6f79646df8925f8d02fdffd967ed26ed4b4d744408f386ade4840545c15`で、二試行ともrunner evidenceは
`spark SUCCESS / grok ACCEPT / EXIT_STATUS 0`。これはtarget-capsule接続と必須試験handoffの実動証拠である。
一行fixtureとGR-D3は難度が異なるため、251,617→10,070 tokensを複雑粒一般の削減率として外挿しない。
実装により旧`INTERNAL_TARGET` anchorが消えた後のcheckpointから`inspect`も再開し、targetを`BASE_SHA` blobへ
照合して50秒、Grok P0/P1/P2=0、`ACCEPT`、exit 0となった。これにより置換粒の検収resumeも閉じた。
複雑な既知問題での一発合格とmain発効はなお未証明である。

### Spark compiled grainのA/Bと接続

2026-07-31、同一baseline、同一`gpt-5.3-codex-spark`、同一sandboxでhandoff形式だけを変える
A/Bを行った。最初のpilotはAが明示`if`、Bが`u32::min`を選び、command数も6対8へ分かれたため、
13秒対18秒というwall timeを因果比較に使わず棄却した。等価な試験緑だけで速度差を採用しない。

次に完成file bytesと唯一の検証command `./verify.sh`を両条件で固定した。Opus 5のread-only preflightは
実行差分0件で`READY`、Spark結果は同一bytes、同一diff、変更file一つ、同一command一回、独立3 tests greenに
揃った。runner形式Aはprompt 2,388 bytes、wall 13.927秒、uncached input 8,084 tokens。compiled形式Bは
804 bytes、12.252秒、4,226 tokensだった。Bはprompt bytesを66.3%、uncached inputを47.7%削減したが、
wall差は12.0%で事前の20%閾値未満だった。したがって**context economyは確認、速度改善は未証明**とする。

この結果を受け、runnerは承認orderをGrok／Codexの検証正本として保持したまま、Spark向けに
`SPARK_GRAIN_VERSION: 1`を決定生成する。生成器はrunner所有のbase、authority receipt、六軸閉鎖判定、
model routing、task hash、precheck metadataだけを除き、`Objective / GRAIN / ALLOWED_FILE / CONTEXT_FACT /
READ_FILE / INTERNAL_TARGET / TEST_TARGET / REUSE_TARGET / NEW_SURFACE / Non-goal / STOP / Test`を元orderと
同数保持する。React／Rerun等の製品固有ラベルと未知の自由記述はallowlist外だからという理由で捨てない。
Spark promptはtarget capsuleとcompiled grainを各一回だけ持ち、元taskと全文orderを再掲しない。

生成grain、target capsule、最終Spark promptは試行evidenceへ保存してparent shell保持hashと照合する。
必須field欠落、16 KiB超過、runner-only metadata漏洩、Spark実行中の生成物改変はfail closedし、Grokへ進めない。
専用負例は欠落拒否、改変拒否、重複task／全文order不在、必須field一回、三生成物hashを固定する。この接続は
2026-08-01にmainで発効した(前掲§移行状態)。ただし単発fixtureの12.0%を全体速度へ外挿しない。

次の複雑粒比較に先立ち、採用済みGR-D3 commit `707800b1`をbase
`2a3e7812808cd0bbe7dcc62ea3f1d48a88cb99e9`へ照合した。実差分はrunner 218行、試験722行の計930行で、
少なくとも次の四境界を同時に含む。

1. known derived targetだけをfail-closedで検査・除去するcleanup境界
2. ambient Git addressing拒否とHEAD／refs digestによる参照保護
3. post-Spark、post-Grok、`inspect` resumeへのlifecycle配線
4. allowlist、空root、未知entry、tracked path、root／nested symlink、除去失敗、ref mutationの負例群

したがって、過去に「GR-D3」と呼んだ親task全体は新分類では一つの`CLOSED`施工粒ではない。危険度が高いから
自動的に`WIDE`なのではなく、複数ownerと新しいprivate helper／配線を束ねたため、まず境界ごとに
`REUSE_CLOSURE`と`NEW_SURFACE`を解決し、独立oracleを持つ粒へ分割する必要がある。旧GR-D3 packetをそのまま
再施工することはtarget-capsuleの複雑粒benchmarkに数えない。前掲のOpus比較は同じ親問題に対する接続形式の
A/B比較としては有効だが、そのpacketが現行runnerでdispatch可能な良い粒だった証拠にはしない。

現行gateでは旧packetのように既存targetだけで閉じず新helper／commandを要求する通常粒は
`NEW_SURFACE: FORBIDDEN`でOpus起動前に拒否する。必要なsurfaceはCodexの仕様・決定粒で先に閉じ、その後の
実装粒だけをexact internal／test／reuse targetへ接続する。次の複雑粒実測は、この分割後に
`VIEW_PROFILE: CLOSED`かつ既存targetへ接続できる一粒を選び、親task全体との速度を混同せず計測する。

GAP-15F1の実運用照合で、旧六軸が`CONTRACT_IMPACT: PERMANENT`だけを理由に、正本で意味を閉じた後の施工まで
常に`WIDE`へ戻す欠陥を確認した。影響種別と閉鎖状態を上記のとおり分離し、未決permanentはOpus前に停止、
検証済みauthority receiptを持つfrozen permanentは他の五軸が閉じていれば施工可能とする。専用runner試験は
未決停止、正しいreceiptの通過、hash不一致receiptのmodel前拒否を固定する。

## scopeを増やす前の自己反証と高速相談

主担当Codexが初動の調査、目的解釈、計画、実行、自己監督を同じ文脈で持つと、検収を比較実験へ、
修復を新機構へ、停止対応を強制介入へ広げた案を、自分で有用と評価して実行まで承認できる。
ここで親taskを停止すると、ループを前提にした開発そのものを中断し、停止対応が新しい目的になる。
疑う対象は親taskではなく、scopeを増やした**候補の次手**とする。

次の一手が元の依頼から動詞、成果物、owner、必要権限、完了条件のいずれかを増やす時、主担当Codexは
mutationまたは外部dispatchの前に次のpacketを短く書く。

1. `ORIGINAL_OUTCOME`: ユーザーが求めた利用者成果
2. `PROPOSED_NEXT_ACTION`: 今から行おうとしている一手
3. `WHY_NEEDED`: 元の成果へ必要だと考えた根拠
4. `ADDED_SCOPE`: 新しく増える成果物、owner、権限、完了条件
5. `DISCONFIRMING_EVIDENCE`: その考えが不要または誤りである反例
6. `SMALLEST_IN_SCOPE_ACTION`: 元の成果を保つ最小の次手
7. `DISPOSITION`: `KEEP / REVISE / DROP / ESCALATE`

packet作成中に、ユーザーが求めていない成果物、自己追加した完成条件、既に完了した施工の再実行、
または「隔離worktreeだから実行してよい」のような安全性と権限の混同が見えた場合、外部回答を待たず
候補案を`DROP / REVISE`する。`DROP`は親taskの終了ではなく、`SMALLEST_IN_SCOPE_ACTION`へ戻る処分である。

自己反証後も処分が割れ、判定で次の実行が変わり得る場合は、fresh-contextの別Codexを一つだけ
read-onlyのscope反対側として呼ぶ。会話全文や長いAGENTSを再送せず、上記packetと検証済みコード事実だけを渡す。
別Codexには編集、外部model起動、再委任を許さず、回答を
`FACTS / SCOPE_DELTA / COUNTEREXAMPLE / SMALLEST_NEXT_ACTION / DISPOSITION`へ限定する。
同じmodelの賛同はauthorityでも実行許可でもなく、主担当Codexが正本とコード事実へ再照合する。

正本とコード事実だけで閉じる通常作業は相談せず続行する。別Codexでも要求解釈、owner、原因、再利用境界が
閉じない場合だけOpus 5へ進み、共有公開境界、恒久契約、長期展望、またはCodexとOpusの結論衝突だけを
Fable 5へ昇格する。いずれの相談も親task全体の直列barrierにしない。

既存規約の`STOP`は、危険な候補操作、契約を発明する施工、または該当粒を実行しない局所信号である。
主担当Codexへ戻し、`REUSE / REMAP / REDUCE / 再調査 / 別lane継続`から次手を選ぶ。安全な次手が無く、
新しい利用者権限または不可逆な外部契約が必要な粒だけをユーザーへ返す。

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
6. 助言してほしい改善機会

Opus 5は批判、反例、欠陥検出だけで回答を終えない。既存の目的、非目標、公開境界、停止線の内側で、
より良い設計、実装順、既存機構の再利用、検証方法、簡素化、具体的な次の一手を能動的に助言する。
Codexの案の良い部分は明示して伸ばし、問題には理由と実行可能な改善案を対で返す。未依頼のscope拡大や
新しい恒久契約を、助言という名目で持ち込まない。

回答は`FACTS / INFERENCES / OPTIONS / OPPORTUNITIES / ADVICE / RECOMMENDATION / STOP CONDITIONS`へ分けさせる。
Codexは事実を再確認し、推論、機会、助言、推奨を採否してから実装判断へ使う。

### Fable昇格

Fable 5は大地図、長期展望、複数仕様の衝突、共有公開境界、恒久契約の新設・変更、またはCodexとOpus 5で
結論が割れた高難度相談に残す。Opus 5を日常的に使えることは、必要なFable相談を省略する理由にしない。

## 権限と停止線

- Opus 5は仕様決定者ではない。親taskの公開API、Document意味、plugin契約、永続形式、変更許可範囲を
  変える必要が見えたら`ORDER: STOP`でCodexへ戻す
- runnerが骨格と承認deltaから作るorderには、対象spec/task ID、目的、現状、`GRAIN`、`BASE_REF`、
  `BASE_SHA`、依存、authority hash、変更許可file、非目標、再利用箇所、STOP条件、必須負例、実行command、
  六つの視野幅入力、`VIEW_PROFILE`、`HAZARD_TAG`、`READ_MODE: CAPSULE`、`CONTEXT_FACT:`、`READ_FILE:`を含める
- 主担当Codexの`CODEX PRECHECK: APPROVED`前にSparkを起動しない
- Sparkはorder外の探索、意味判断、範囲拡張、期待値・golden変更、lint抑制、commit、push、再委任をしない
- Grokは実装もorder再設計もせず、実diff、authority、scope、負例、試験証跡だけを独立検収する
- Grokが`VERDICT: ACCEPT`かつP0/P1=0でなければ採用、commit、pushしない
- REJECT、STOP、timeout後の戻り先はCodexとする。Codexが原因を裁定してから、必要なら新しいOpus粒へ戻す
- model利用不能時に別modelへ黙ってfallbackしない

React製品資産とRerun参照を含む発注は、`AGENTS.md`の追加ラベル、順序、STOP条件をこのループより優先して
満たす。ループの簡略化は製品契約の簡略化を意味しない。

## 2026-08-01改訂 — 粒度上限・検収の有効性条件・独立性の定義・守備範囲

根拠は[速度支配項と計装の観察](2026-08-01-supervision-loop-cost-driver-observation.md)。
本改訂は**独立検収を制約条件として固定したまま**、その内側だけを変える。検収の省略・縮退は行わない。

各項の**発効状態を明記する**。「決定済み・未発効」を積み増さないため、runner改修を待たずに
主担当Codexが今日から適用できる項と、機械gateが要る項を分ける。

### 改訂1 — 粒の上限は行数でなく契約境界数(発効中)

通常粒は**一つの契約境界**で閉じる。複数境界を束ねた粒はOpus起動前にCodexが分割する。

根拠は三方向で一致する。

- Motolii実測: 930行・4境界の粒は430秒＋586秒で2回ともREJECT、採用0。一行fixtureは62秒でACCEPT
- 本書§Spark compiled grainのA/B(2026-07-31)が既に「旧GR-D3は一つの`CLOSED`施工粒ではない」と自己診断していた
- 外部実測([arXiv 2606.15689](https://arxiv.org/abs/2606.15689)): LLM検収のF1はdiff 10行未満で0.657、150行超で**0.043**

`AGENTS.md`の「実装発注は一度に1つの契約境界」を、`REUSE_CLOSURE`と`NEW_SURFACE`が
境界ごとに閉じているかで判定する運用へ具体化する。行数閾値は正本にしない。行数は事後にしか
分からず、[Google eng-practices Small CLs](https://google.github.io/eng-practices/review/developer/small-cls.html)
も self-contained とファイル分散を基準にしている(同書は実証研究の引用が無い人間向け実務ガイドで、
リポジトリは2025-11-21にアーカイブ済み)。

**発効(2026-08-01実装)**: `CONTRACT_BOUNDARY:`を一つだけ宣言させ、`ALLOWED_FILE`が単一の
top-level ownerへ収まることをdispatch前に照合する。`docs/`はledger・decision-index登録が
workflow上必須のため別境界に数えない。

**この機械化の限界を明記する**: 契約境界は意味であり、機械で導出できない。実装したのは
**宣言gateとowner照合**であって境界の導出ではない。それでも930行のGR-D3は「誰も境界数を
宣言しなかったため4境界であることが見えないまま2回dispatchされた」という失敗であり、
宣言を強制するだけでこの失敗は起きなくなる。owner照合は宣言と実態の食い違いを機械で捕える。

### 改訂2 — `VERDICT: ACCEPT`に有効性条件を付ける(発効中)

検収が機能しない規模で得た`ACCEPT`を、採用の有効な根拠にしない。

diff規模が劣化領域にある場合、Codexは`ACCEPT`を受け取っても**採用せず**、粒を分割して再発注する。
これは独立検収を弱める変更ではない。**検収が働かない領域での合格を根拠に使わない**という制限であり、
保証を強める方向である。

Fable 5のread-only掃引(2026-08-01)で、大diffの検出率を回復させる実証済み手法は
**見つからなかった**(実PR規模の最良値はどの手法でもF1 0.19–0.28帯)。したがって
「検収技術で大きな粒を救う」経路は現時点で存在しない。

**暫定**: 具体的な閾値は実データ3件が貯まるまで確定しない。外部研究の150行は8 OSSリポジトリの
実PR由来で、Motoliiのgrain分布と異なる。

### 改訂3 — 検収者固定を独立性条件へ一般化する(条件は発効、実装は未発効)

守るべき不変条件は**Grokを使うこと**ではなく、**実装担当から独立した別LLMによる外部視点の検収を
必ず得ること**である。model固定はその一実装にすぎない。

独立の定義は先例に従う。[NASA SWE-141](https://swehb.nasa.gov/display/SWEHBVC/SWE-141+-+Software+Independent+Verification+and+Validation)
とIEEE 1012はtechnical／managerial／financialの3軸を定める。DO-178Cは「検証者は当該成果物の
作成者であってはならない」と定義し、独立を要求する目標数を保証レベルで変える(Level A=31中16、
Level B=31中7、Level C/D=0)。IEC 61508はSILに応じて独立した個人→部門→組織の梯子を持つ。
**先人は「独立を省くか」でなく「どこまで遠い独立を要求するか」を危険度で決めている。**

Motoliiの床(全粒で必須・段階化しない):

1. reviewerのmodel identityが実装担当と異なる
2. reviewerはfresh sessionから開始する
3. 実装担当の思考過程・自己説明・修正理由を渡さない。渡すのはfrozen contract、実diff、
   test evidence、負例oracleだけ
4. read-only。reviewerは修正しない
5. 検査範囲を実装担当が決めない
6. `ACCEPT / REJECT`とP0/P1を構造化出力する
7. receiptへ実使用modelとfallback有無を記録する

technical independenceの中身は「別modelを使う」ではなく「**問題理解を自分で再構成する**」である
(SWE-141)。実装担当の推論を渡した時点で、model IDが違っても独立性は失われる。現行の
compiled grainが実装担当の思考を含まないのは、この条件を満たしている。

梯子(`HAZARD_TAG`×`CONTRACT_IMPACT`で段階化):

| 影響 | 要求 |
|---|---|
| `NONE` / `PRIVATE` | 別model identity(床のみ) |
| `PERSISTENCE` / `CONCURRENCY` / `PLATFORM` / `SHARED` | 別model family |
| `SECURITY` / `DESTRUCTIVE_FS` / `PERMANENT` | 別provider **＋非LLM oracleの併用を必須** |

最上段が「別provider」だけで足りないのは、[Correlated Errors in LLMs (ICML 2025)](https://arxiv.org/abs/2506.07962)
が350+モデルで**両モデルが誤る場合の60%は同じ誤り**、しかも大型で高精度なモデルほどprovider差が
あっても相関が高いと報告するため。**真に相関しない軸は非LLM oracle(型・静的解析・実行)である。**
同じ理由で「より強いモデルを検収に置けば安心」を採らない。2606.15689でもHaiku 4.5がSonnet 4.6を
F1・recall・コストの全てで上回っている。

financial independenceはMotoliiへ**転移不可**として明記する。

**発効(2026-08-01実装)**: `REVIEW_MODEL`の固定値等値比較を廃し、次をdispatch前に照合する。

- reviewerが承認済み独立検収者の集合に属する(既定は`cursor-grok-4.5-high`のまま)
- reviewerのidentityが実装担当と異なる
- reviewerのidentityがorder managerと異なる(事前設計へ深く関与したmodelを選ばない)
- `HAZARD_TAG: NONE`かつ`CONTRACT_IMPACT: PRIVATE`以外は、reviewerのmodel familyが実装担当と異なる
- `SECURITY` / `DESTRUCTIVE_FS` / `CONTRACT_IMPACT: PERMANENT`は`MECHANICAL_GUARD:`の宣言を必須とする

最上段の非LLM oracleは要求するだけでなくrunnerが供給する。`SECURITY`と`PERMANENT`へ
`MECHANICAL_GUARD:`を機械注入し、「reviewerの意見でなくtestまたは静的検査で証明する」ことを
orderへ焼く。receiptには`IMPLEMENTER_MODEL_FAMILY` / `REVIEW_MODEL_FAMILY` / `MODEL_FALLBACK`を残す。
runnerは黙ったfallbackをしないため`MODEL_FALLBACK`は常に`NONE`であり、**ループ外で別modelへ
差し替えた場合はreceiptに現れない**ことで検出できる。

現在のallowlistには実装担当が含まれないため、identity検査はallowlistに先行して落とされる。
identity検査はallowlistが将来広がった時の多層防御として残す。

### 改訂4 — 性能regressionはLLM検収の守備範囲外(発効中)

[arXiv 2606.15689](https://arxiv.org/abs/2606.15689)は、評価した全モデルで**性能関連バグのrecallが
ほぼ0**と報告する。Fableの掃引でもこの欠陥クラスを回復させる手法は見つからなかった。

Motoliiの絶対規律は「VRAM常駐」「色変換の一元化」「プレビュー/書き出し同一関数」であり、
**製品の中核リスクはLLM検収が最も弱い欠陥クラスに一致している**。

したがって、性能regressionの審判をLLM検収へ期待しない。bench、golden、profiling oracleで持つ。
Grokの`ACCEPT`を性能非退行の根拠にしない。性能に触れる粒は機械oracleを完了条件へ含める。

### 改訂5 — 検収入力のslice化(未決・次段)

計装で、Grokが1行の自明な依頼でも入力16,013 tokensを使うことが分かった。検収コストはdiffの
大きさより**入力の作り方**に支配されている。

唯一の本番デプロイ実証([arXiv 2505.17928](https://arxiv.org/html/2505.17928))は短いsliceが
最大文脈に勝つと報告するが、Key Bug Inclusionは天井約31%である。また境界横断バグがsliceから
落ちる危険があるため、全境界の契約シグネチャだけを見る俯瞰パスの併設が要る。

**設計を閉じてから実装する。本改訂では採らない。**

## 改訂凍結(2026-08-01)

状態: **停止線**

本決定と`AGENTS.md`の監督ループ節は、**実製品粒で連続3件の`VERDICT: ACCEPT`を得るまで改訂しない**。

凍結する理由は、規約の内容が誤っているからではなく、**改訂に終了条件がなかった**ためである。2026-07-25から
08-01の間、計測のたびに規約が改訂され、改訂は「決定済み・未発効」として積まれ、mainへ入った後も発効宣言が
行われなかった。その結果、どの規約が現に効いているのかを実行前に判定できず、同じ論点が繰り返し再裁定された。
これは規約の欠陥ではなく、改訂ループの欠陥である。

凍結中の扱いを次で固定する。

- 監督ループについての計測、論文・先例調査、他LLMからの助言は`docs/reviews/`のobservationに留める。
  規約、runner接続方式、`AGENTS.md`へ昇格させない
- 評価指標は新設せず、[発注パイプライン比較 §8](2026-07-23-parallel-order-pipeline-comparison.md#8-速度と品質の測定案)
  の既存表（lead time / wait time / first-pass accept / rework count / stale-base count / escaped finding /
  Codex integration load）を使う。同書はARCHIVEDだが、アーカイブ対象は旧model配置と複数実装lane案であり、
  §1の診断と§8の指標表は棄却されていない。数値は手写しでなく
  [runner計装](2026-08-01-supervision-loop-cost-driver-observation.md#7-計装実装済み2026-08-01)のreceiptから採る
- 連続3件は同一fixtureの反復でなく、実製品粒で数える。REJECT、STOP、timeoutが一件でも入れば計数をやり直す
- 凍結解除時は、3件のorder SHA-256、検収verdict、token、wall timeを本文書へ追記してから改訂を再開する

**起点**: 本凍結は前節「2026-08-01改訂」をもって発効する。同改訂は凍結の違反ではなく、
churnを止めるための締めくくりである。以後、新しい規約項を追加しない。

**凍結対象外(実装のみ)**: 前節で**未発効**と明記した機械gateの実装は凍結の対象外とする。
新しい規約を作る作業ではなく、既に裁定済みの条項をrunnerへ落とす作業だからである。具体的には
次の三つに限る。これ以外を「実装だから対象外」と拡張しない。

1. ~~改訂1の契約境界数をdispatch前に機械判定するgateと専用負例~~ **完了(2026-08-01)**
2. ~~改訂3の`REVIEW_MODEL`等値比較を独立性述語へ置換するgateと専用負例~~ **完了(2026-08-01)**
3. Spark／GrokのUSD cost計測(両CLIが返さないため、可能なら別経路で) — **未着手**
4. gate発火台帳と文脈パッキングの節約量計測。gate、routing、独立性を変えず記録だけ増やす。
   凍結の解除条件(連続3 ACCEPT)は証跡なしに判定できず、runner三層の要否も実測なしには決められない
   ため、計測手段の追加は凍結の趣旨に反しない。判定基準は
   [観察§10](2026-08-01-supervision-loop-cost-driver-observation.md#10-runner三層の要否を測る手順事前登録)へ
   事前登録する

改訂5(検収入力のslice化)は**設計が閉じていない**ため、実装として着手しない。設計を閉じる作業自体が
規約改訂に当たるため、凍結解除後に扱う。

### 凍結中も動かさない中核保証

検収者の独立性条件は前節[改訂3](#改訂3--検収者固定を独立性条件へ一般化する条件は発効実装は未発効)が正本であり、
ここでは重複させない。

凍結の有無にかかわらず、次の最適化は採らない。**独立検収そのものを省略・代替する変更は、
コスト削減でもmodel routingでも不可とする。**

- 低riskだからreviewerを省く
- mechanical testを独立検収の代替にする
- 実装担当の自己検証や複数候補投票で置き換える
- reviewerのscoring、選択router、provider多様性の必須化を先に作る(実績3件の証跡が先)

## アーカイブした方式

[タスク適応型の発注運用](2026-07-22-terra-grok-delegation-policy.md)で定めた
`mechanical / standard / rapid / complex / cross-boundary`分類、Luna/Terra/Solの実装routing、
`complex / cross-boundary`でのFable必須検収、Grokによるorder draftは、2026-07-25をもって
**ARCHIVED**とする。歴史的な比較根拠として残すが、現行dispatchの根拠にしない。

## 完了条件

- `AGENTS.md`が本ループと同じ責任順序を示す
- 正規runnerがOpus 5 order管理、Spark実装、Grok read-only検収の順だけを起動する
- orderのmodel/loop metadataが固定値と一致しない場合はdispatch前にfail closedする
- task／order／authority／allowlist／read setが速度予算を超える場合はmodel起動前にfail closedする
- internal／test／reuse targetが不在、重複、read set外、または実装targetがallowlist外ならmodel起動前にfail closedする
- `NEW_SURFACE: FORBIDDEN`を固定し、target近傍だけのrunner生成capsuleをSparkへ渡す
- Opus 5が全文orderでなくschema制約済みdeltaだけを返し、schema不正時に散文fallbackしない
- P0/P1 deltaを骨格revisionへ戻し、P2を含む採用findingはIDごとのresolutionなしにdispatchしない
- 六つの視野幅入力から`VIEW_PROFILE`を機械算出し、`WIDE`をSparkへ送らない
- `HAZARD_TAG`ごとの既知負例と機械guardをmodelの自由記述より先に骨格へ注入する
- SparkとGrokが承認済みカプセル外の全規約・全spec・repo横断探索を通常動線で要求されない
- 旧`TASK_CLASS` routingとFable必須検収を正規runnerから起動できない
- runnerの負例試験と`./scripts/check-docs.sh`が通る
- 通常粒が一つの契約境界で閉じ、複数境界を束ねた粒はOpus起動前に分割される(改訂1)
- 検収が機能しない規模のdiffで得た`ACCEPT`を採用根拠にしない(改訂2)
- reviewerが独立性の床7項目を満たし、receiptへ実使用modelとfallback有無が残る(改訂3)
- 性能に触れる粒がbench／golden／profiling oracleを完了条件に含め、検収の`ACCEPT`を
  性能非退行の根拠にしない(改訂4)
- 各stageのwall time、token、costがrunner計装のreceiptに残り、欠測stageが明示される
