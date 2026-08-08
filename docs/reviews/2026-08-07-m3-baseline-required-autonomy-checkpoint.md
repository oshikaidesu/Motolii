# M3 baseline-required自走checkpoint決定

日付: 2026-08-07  
状態: **決定**

## 1. checkpoint

M3の自走は、一般的なdesktop動画編集ソフトで欠落すると製品が購入候補から外れるuser-visible outcomeを
`BASELINE_REQUIRED`として扱う。`BASELINE_REQUIRED`へ採用されたoutcomeは、必要性について利用者へ毎回再確認せず、
current authorityとcode factから一契約境界へcompileできる。

これは、一般的な機能名、特定製品のUI、内部機構、公開API、Document意味、実装順を主担当が推測してよいという決定ではない。
必要性の自動承認と施工資格を分け、owner、exact target、write route、positive／negative oracleが閉じた時だけ`DO`へ上げる。
targetが無いoutcomeは削除せず、前ownerへ`RESEARCH_RETURN / TARGET_MISSING`として返す。

## 2. baseline抽出の責任分離

baseline候補の抽出は総監督Codexが独力で行わない。OpenAI利用枠をauthority整理と最終採否へ縮約し、次の責任へ分ける。

| role | owns | does not own |
|---|---|---|
| fresh non-OpenAI researcher | 現行desktop NLEのsample選定、current official vendor source、user-visible outcome matrix、同義語候補、非証明範囲 | Motoliiへの採用、実装順、owner／API／schemaの発明 |
| different-family challenger | source hit、sample偏り、outcome統合、threshold感度、marketing claim混入の反例監査 | 自分のfeature listへの置換、Motolii施工 |
| Codex supervisor | raw evidence packet、source／hash、authority mapping、current code照合、scope／oracle、`BASELINE_REQUIRED / PRODUCT_CHOICE / NOT_COMMON_OR_UNPROVEN`の最終処分 | evidence前のcandidate feature列挙、外部modelの結論のauthority化 |

model名をroleへ固定しない。各runでtask、family independence、利用可能枠、current CLI capabilityから再導出し、黙ったfallbackをしない。
普遍的なprovider共通framework、巨大runner、receipt DB、provider共通parserは本checkpointの完成条件外である。

## 3. 採用可能な調査証拠

baseline調査は次を満たすまでMotolii authorityへ入れない。

1. researcher自身が製品sampleと再現可能な採用thresholdを提示する
2. positive cellごとにcurrent official manual／help／vendor documentationのdirect URLと原文根拠を持つ
3. 検索missをabsenceへせず、証拠不足は`UNKNOWN`またはexact evidence gapにする
4. vendor固有UI名でなく独立したuser exitへ正規化する
5. different-family challenge後もsourceとoutcome統合が成立する
6. CodexがMotoliiの既存owner／command／projection／oracleへ再照合する

sample、threshold、candidate outcomeはこの決定で先取りしない。調査runが失敗した場合、そのpromptや途中回答に現れた製品名、
feature row、countを次runの期待値やauthorityへ流用しない。

## 4. 2026-08-07 Web research失敗の処分

直前の二runはprovider capability不足でなく、呼出側permission設定の不適合として処分する。

- Claude Code 2.1.223はinit eventへ`WebSearch`／`WebFetch`を登録したが、read-only researchへ
  `--permission-mode dontAsk`を指定したため`WebSearch`がpermission denialになった
- Cursor Agent 2026.08.04-aaa8809はread-only ask runで`WebFetch` interaction queryを生成したが、非対話runで
  safe-tool approvalを閉じなかったため`User Rejected`になった
- 両runともofficial page bodyを取得できず、baseline matrix、sample、threshold、feature候補は**不採用**である

mutation用のfail-closed permissionをread-only Web researchへ転用しない。次回は空workspace、repo toolなし、単一の無害な検索query、
provider-native途中stream保存だけの安価なprobeをfresh runで行う。実tool result、terminal result、repo read 0、mutation 0を確認してから、
本調査を別のfresh runとして起動する。probe成功はbaseline抽出成功や製品機能の証拠ではない。

### 4.1 1-query capability probeの限定観察

2026-08-07にClaude Code 2.1.223／`claude-fable-5`のfresh empty workspaceで、repo tool 0、
`WebSearch,WebFetch`だけを許可し、exact query `IANA-managed Reserved Domains`を一回実行した。
`--permission-mode default`では非対話承認が成立せずpermission denialになったが、別fresh runの
`--permission-mode auto`ではWebSearch一回と公式`https://www.iana.org/domains/reserved`へのWebFetch一回、
official body、provider-native途中stream、terminal result、repo read 0、mutation 0を確認した。

これはClaude directのWeb-only capability routeが現環境で利用可能という**限定観察**である。
`auto`を施工、repo read、他provider、他toolへ一般化せず、runnerへ特別処理、固定permission、query counter、
receipt DB、read-set enforcementを追加しない。current harnessのexact read setはexecution envelopeと事後監査を持つが、
workspace内の全readをOSレベルで遮断する一般機構ではない。強いread containmentが必要な調査／reviewはempty workspace、
tool allowlist、または一つのblind evidence envelopeで閉じる。

## 5. M3 current state

| boundary | state | disposition |
|---|---|---|
| M3 runtime選定 | `DONE` | React Native + rust-skia + wgpu再基線を維持する |
| M3 R0 candidates | `READY-RECHECK / MAIN NOT REACHED` | `R0-HOST`、`R0-MAC-SEAT`、`R0-STAGE-LIFECYCLE`を別々にcurrent-main再照合する。再実装を先行しない |
| baseline extraction | `OPEN / ACCEPTED ITEM 0 / WEB PROBE PASSED` | fresh researchを別runで開始できる。probe結果自体はbaseline evidenceにしない |
| baselineからM3地図へのmapping | `OPEN` | 独立抽出・challenge合格後にだけ既存nodeへ写す |
| M3 product implementation | `NOT AUTHORIZED BY THIS CHECKPOINT` | R0と対象orderが閉じるまで開始しない |

baseline laneの未完了はR0候補のread-only再検収を止めない。R0検収もbaseline候補を捏造したり、R1以降の施工を先取りしない。

## 6. execution-envelope候補の分離

`/private/tmp/motolii-run-evidence-20260807.d5vJds/worktree`に存在すると報告された
`scripts/run-observed-cli.py`／`scripts/test_run_observed_cli.py`のexecution-envelope候補は、直前観測では
`TESTED / UNINTEGRATED`である。temp path、外部review、focused test greenはcurrent branch統合を意味しない。

この候補の再現、採否、current authorityへの統合は別のtransport契約境界であり、本M3 checkpointへコードやcommitを混ぜない。

## 7. checkpoint後の一手

1. fresh non-OpenAI researcherへbaseline本調査を一契約として渡す
2. 並行してR0三候補をcurrent-mainへ別々に`VERIFY_CANDIDATE`する
3. baseline調査が証拠付きで返った時だけdifferent-family challengeへ渡す
4. 採用outcomeを既存M3 nodeへ写し、`CLOSED / OBSERVED ONLY / OPEN`を付ける
5. `DO`になった一契約だけを施工へ送る

## 8. 非目標

- 総監督が記憶や一般論からbaseline feature listを作る
- 失敗runのproduct sample、candidate row、thresholdを再利用する
- 一般製品にあるという理由だけでMotoliiの公開API、Document意味、第二writer、第二GPU ownerを発明する
- R0未検収のままR1以降を連続施工する
- provider共通runner、receipt DB、固定model pipeline、自動fallbackを本checkpointへ追加する
- execution-envelope候補、commit、push、main統合を本checkpointの完成へ含める
