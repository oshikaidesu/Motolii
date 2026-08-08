# cold-replaceable監督と停止封じ込め決定

日付: 2026-08-09  
状態: **決定 / failure injectionとfresh closure review通過前は全体並列発注不可**

## 1. 決定

Motoliiの並列実装は、判断権を下位seatへ複製して停止を避けない。durable authorityを一つに保ち、総監督を担当するsessionだけをcoldに置換可能にする。

総監督不在中の正しいworst caseは次である。

```text
new external launch: 0
new edge selection: 0
adoption / integration: 0
in-flight observed run: finish or harness reclamation
candidate worktree / raw log: preserved, unadopted
recovery: fresh top-seat session reconstructs from durable authority
```

停止耐性は「常時進み続けること」ではなく、権威・成果・復元可能性を壊さず、総監督席を短時間で再開できることと定義する。

## 2. 一つのtop seat

本文では、`主担当Codex`と`総監督Sol`の重複表現を**top seat**へ統一する。通常の担当model baselineは`gpt-5.6-sol`だが、model identityやconversationがauthorityなのではない。

top seatだけが次を所有する。

- ユーザー許可と`STOP`
- 現行正本、base/cwd、worktree、fingerprint
- outcome、次edge、owner、scope、allowlist、read set、oracle、`WIDE`
- exact external model、family、limit group、argv、permission
- finding処分、return後の再選定
- reviewer独立性、採用、統合順、main統合

同時にactiveなtop-seat decision ownerは最大1とする。shadow、deputy、field、reviewerは第二の採否票を持たない。

## 3. durable authorityとcold replacement

sessionを越えてauthorityになれるのは次だけである。

- ユーザーの明示指示
- Gitとcurrent working-tree governance
- 正本、decision index、implementation ledger
- prepared worktreeの実diffと開始前後fingerprint
- task固有oracleの生結果
- observed CLIのraw stdout／stderr／lifecycle／exit／signal

conversation history、session token、warm standbyのmemory、deputy summary、handoffの`next safe action`はrouting hintに限り、authority、project memory、採用証拠にしない。

後任はwarm resumeせず、fresh sessionとfresh capsuleでcold reconstructionする。旧sessionが所有した全process groupの終了・signal・回収をharnessで確認し、全ACTIVE worktreeのfingerprintを読み直すまでtakeoverしない。heartbeat、provider沈黙、wall clock、自己申告を死亡証明にしない。

## 4. deputy／field／implementer／reviewer

階層は権威階層ではなく、観測と配送の分担である。

| seat | 許可 | 禁止 |
|---|---|---|
| deputy | top seatが閉じたfrontierのobserved run監視、RETURN収集、exact preauthorized argvの起動 | dependency再解釈、次edge、代替order、finding修理、model変更、採用、統合 |
| field | 一つのowner／write-set内でcapsule配布、allowlist監視、指定oracle実行 | scope拡張、shared owner変更、実装者／reviewer再選択、要約による採用 |
| implementer | 一契約境界の施工と指定試験 | 再委任、公開契約変更、capsule外read/write、finding由来の追加施工 |
| reviewer | fresh、read-only、別familyで実diffと負例を監査 | mutation、修正、再設計、採用、深く関与したgrainの自己review |

deputyまたはfieldがREADYをACTIVEにできるのは、top seatが現在存在し、ユーザーの明示`発注`scope内で、exact argvまでpreauthorizeされ、base／authority／dependency／allowlist／oracleが不変で、全ACTIVE frontierとwrite-setおよびsemantic ownerが非交差の場合だけである。

RETURN、authority conflict、base change、predicate入力の変更が一つでも生じた時点で関連preauthorizationは失効する。top seat不在中は新規launch 0とする。

## 5. campaignと短wave

24時間運転は一つの長寿命sessionやwaveではない。多数の短waveからなるcampaignである。

一つのユーザー`発注`は、campaignが起動できるoutcome、grain集合、mutation、validation、外部model利用を明示的に囲う。deputyがqueueを自動補充したり、findingからcorrection grainを生成したり、許可外grainへ進んだりしない。

各短waveは一つのoutcome、owner、scope、oracleを持つfresh sessionである。return後はtop seatがcurrent codeから次edgeを再選定する。24時間というwall-clock目標はoracle、reviewer独立性、effort、negative caseを弱める理由にならない。

## 6. state transition

許可する遷移:

| from → to | actor | condition |
|---|---|---|
| `IDLE → DISPATCH` | top seat | ユーザー発注scope内、not WIDE、closed capsule、途中stream条件、clean worktree、disjoint allowlist |
| `READY → ACTIVE` | deputy／field | §4の全precondition、top seat present |
| `ACTIVE → RETURN(done/fail/context-gap)` | 任意seat | 常に可能。証拠不足は`CONTEXT_GAP: exact missing evidence` |
| `RETURN → ADOPT / same-boundary correction / split / local STOP` | top seat | current authorityと実diffを直接再照合 |
| `CANDIDATE → ADOPTED → INTEGRATED` | top seat | §7の採用gateとcurrent main再確認 |
| `session → fresh session` | cold replacement | prior process-group reclamationとworktree fingerprint確認 |
| `any → GLOBAL STOP` | user | 対象process停止、新規edit/test/review/launch 0 |

禁止する遷移:

- top seatが二つ同時にdispatch、adopt、integrateする
- top seat不在中の新規launch、採用、統合、次edge選定
- RETURN／base drift／authority conflict後の古いREADY activation
- provider沈黙やheartbeatだけによるkill、retry、takeover
- failed sessionの別model resume、silent fallback、固定fallback順
- lease、checkpoint、handoff、summary、receiptを採用資格にする
- reviewer mutation、same-family self-review、設計／施工関与familyの最終review

## 7. 採用gate

top seatはcandidateごとに次を直接確認する。

1. 正しいbase/cwdと開始前後fingerprint
2. allowlist内の実diffと、write-set／semantic ownerの非交差
3. task固有test、fixture、negative case、必要な非LLM oracle
4. freshな別family reviewerで、設計・施工へ深く関与せず、review中mutation 0
5. P0/P1 0、scope違反 0、unresolved shared boundary 0、`EVIDENCE_GAP` 0
6. grainへ触れた全sessionのmodel family／limit group provenance
7. current mainに対するstaleness 0。古ければrebase後にoracleとreviewをやり直す

RETURN集合はboundedにし、上限へ達したら新規activationを止める。idleは誤採用より正しい。

## 8. failure injection oracle

全体並列発注前に、LLM判定ではなくprocess、Git、hash、exit、diffで次を検証する。

1. **evidence preflight**: `scripts/check-evidence-envelope.py`がsource SHA-256、range、literal query scopeのcomputed hit集合をpacket内manifestへ生成し、全hit raw bytesが収録range内にあるpacketだけをbyte単位で再検証する
2. **double top seat**: 二processが同時取得を試み、exactly oneだけがdispatch可能
3. **stalled but alive**: top seatを`SIGSTOP`し、新規launch 0、child raw stream保全、死亡誤判定0
4. **real death**: `SIGKILL`後、exit/signalとprocess-group回収後だけfresh successorが復元する
5. **base drift**: candidate中にmainを進め、古いfingerprintで採用拒否する
6. **write-set collision**: allowlist交差またはsemantic owner共有で第二dispatch／採用を拒否する
7. **allowlist and reviewer purity**: scope外pathまたはreviewer mutation 1件でacceptanceを無効化する
8. **RETURN bound**: 上限到達でactivation停止、既存candidate保全
9. **false progress**: heartbeatだけのstreamを進捗、成功、kill理由にしない
10. **channel unavailable**: direct channel失敗時にsilent fallbackせず、exact reasonとIDを記録して局所停止する
11. **user STOP**: 全対象process group停止後、新規edit/test/review/launch 0、candidateは未採用で保全
12. **integration crash**: commit途中killでpreまたはpostの一方へ復元し、decision／ledgerを同じcommitに保つ
13. **campaign dry run**: fake CLIでsuccess、nonzero、truncated output、hang、long silenceを再生し、許可された遷移だけをdurable stateとraw logから再構成できる

evidence selectionは次の閉schemaだけを受ける。`ranges`と`scope`は1-origin inclusive lineであり、packet内のsource SHA-256、range hash、literal hit line／columnはtoolがcurrent bytesから生成する。選択者がhit一覧やhashを手入力しない。

```json
{
  "schema_version": 1,
  "sources": [{
    "path": "docs/example.md",
    "ranges": [[1, 20]],
    "queries": [{"literal": "exact phrase", "scope": [1, 100]}]
  }]
}
```

新規packetは`--write-envelope`で既存pathへの上書きを拒否し、review起動直前に同じselectionを`--check-envelope`へ渡してcurrent sourceとbyte一致を再検査する。packet自身へ自己参照hashを入れず、commandが出すenvelope SHA-256をobserved run logへ記録する。

まず圧縮dry runを通し、その後に24時間scaleでqueue、disk、idle、recovery timeを測る。総監督自身の要約はoracle入力にしない。

### 2026-08-09圧縮dry run

`scripts/test_supervision_failure_containment.py`はproduction runner、activation bundle、採用DBではなく、temp directory／temp Git repo／fake CLIだけを使うtest-only fixtureである。既存`run-observed-cli.py`と合わせ、§8の13項目を次へ写像して通過した。

| §8 | 非LLM oracle |
|---|---|
| 1 evidence preflight | `scripts/test_check_evidence_envelope.py` 7 test。source drift、range外hit、path escape、repo内symlink、range重複、上書き、byte再検査 |
| 2 double top seat / 3 stalled but alive | real `fcntl.flock`を二processで競合し、holderを`SIGSTOP`中もsecond acquisitionと新規launchを拒否。`SIGCONT`後の正常回収だけを許可 |
| 4 real death | lock holderを`SIGKILL`し、process exit後のlock取得と全ACTIVE treeのcurrent fingerprint再取得を確認。古いfingerprintを継承しない |
| 5 base drift | temp Git repoのpreauthorized HEAD後に新commitを作り、不一致を採用拒否へ返す |
| 6 write-set collision / 7 allowlist and reviewer purity | path交差またはsemantic owner共有を拒否し、実Git diffのallowlist外pathとreviewer前後fingerprint差を検出 |
| 8 RETURN bound | 3 return後の新規activation 0と、未採用candidate fileのbyte保全 |
| 9 false progress / 10 channel unavailable | heartbeatをterminal resultにせず、direct fake CLI exit 69後のfallback log 0、success／nonzero／partial／hang／silenceをraw metaとlifecycleから`RETURN(done/fail)`へだけ再構成 |
| 11 user STOP | harness親へ`SIGTERM`し、child＋grandchild process group回収、`received_signal=SIGTERM`、後続launch 0 |
| 12 integration crash | Git pre-commit hook中にprocess groupを`SIGKILL`し、HEAD pre-state、exact `.git/index.lock`残留を確認してtemp repo内で回収後、decision＋ledgerの二fileを一commitへatomic反映 |
| 13 campaign dry run | 上記fake CLI 5形をdisjoint logへ通し、全runが`started → ... → completed`、採用遷移0、raw partial byte保全 |

`python3 -m unittest scripts/test_check_evidence_envelope.py scripts/test_run_observed_cli.py scripts/test_supervision_failure_containment.py`は23 test、`./scripts/validate.sh tooling`も23 testを含めてPASSした。これは圧縮dry runの合格であり、実外部modelの24時間campaign、queue／disk scale、製品実装、main統合の証拠ではない。

## 9. 既知実装preflight

```text
MECHANISM CLASS: external run observation, process-group reclamation, Git/worktree isolation, deterministic evidence checking
KNOWN IMPLEMENTATION SEARCH: scripts/run-observed-cli.py, scripts/test_run_observed_cli.py, runner-independent supervision, blind evidence envelope observation, Git worktree and process-group OS primitives
CANDIDATES: existing observed CLI harness; Git/worktree/fingerprint; scripts/check-evidence-envelope.py using Python stdlib hashlib/json/pathlib
ADOPTION ROUTE: REUSE observed harness and Git; WRAP stdlib only for deterministic evidence preflight
REJECTED CANDIDATES: new runner/state DB/queue service; warm session pool; harness JSON semantic interpreter; heartbeat daemon
THIN MOTOLII SEAM: top-seat policy, exact preauthorization invalidation, standalone deterministic evidence packet preflight, failure fixtures
THIN MOTOLII RESIDUAL: user dispatch scope, authority ownership, adoption oracle, reviewer independence
RETIREMENT: no old runner revival; replace only the standalone evidence checker if an equivalent accepted repository lint exists
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN for a new supervision framework, receipt DB, broker, or authority service
```

## 10. 外部counter-reviewの処分

2026-08-09の初回fresh Cursor Grok 4.5 Highは`REVISE`、初回fresh Claude Opus 5 xhighはpacket inventory不整合を検出して`EVIDENCE_GAP`を返した。Fable指定runはproviderが別modelへ自動fallbackしたため停止し無効とした。

共通findingは本文へ採用した。特に、総監督不在中の下位activation禁止、cold replacement、process-group reclamation、write-set交差拒否、bounded RETURN、candidate staleness、family provenance、user発注scopeをP0として回収した。

修正後の初回closure packetはcheckerを通ったが153,157 bytes／3,441行となり、Claudeの一回Readは1,259行でtruncateされた。ComposerとGrokは同一fileを追加paginationして最終文を返したため、one-read blind envelope reviewとしては無効である。Opusは追加Readせず`EVIDENCE_GAP`を返した。三runから、failure injection未実施、symlink負例不足、one-read予算超過をscope内blockerとして採用した。

symlink負例と圧縮failure injectionは§8の追補で回収済み。raw rangeまで一回Readへ収まる縮小packetを非LLM preflightへ通し、freshなComposer 2.5短waveでP0/P1とgapを再検査するまで本決定のclosure reviewは未完了である。旧Grok／Opusは本修正へ関与したため最終独立reviewerに数えない。

## 11. 棄却する複雑性

- `prepare / execute / inspect / cancel`やcanonical activation bundle
- versioned route／lease／handoff schemaを起動条件・必須field・採用資格にすること
- ACTIVE／READY／RETURN用の新DB、broker、queue service
- seatごとの常時shadow、heartbeat monitor daemon、warm context pool
- mandatory `deputy → field → implementer → reviewer`固定pipeline
- thin CLI harnessへworktree、JSON意味、採否、session資格を持たせること
- summary／checkpoint／lease cleanを実diff・oracleの代わりにすること

seat排他の実装記録はdispatchの二重実行を防ぐ補助であり、authorityや採用資格ではない。長期状態を増やさず、Git、正本、ledger、raw logからcoldに復元できる範囲へ留める。
