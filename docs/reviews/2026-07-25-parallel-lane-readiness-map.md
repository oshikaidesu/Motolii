# 並列レーン着手地図（2026-07-25）

状態: **実行決定**。既存specの意味・完了条件は変更せず、現在着手できる契約境界を
laneへ分離する。Fable 5反対側レビューのP1二件を訂正済み。

## 1. 目的

[並列Human Response Frontier](2026-07-25-parallel-human-response-frontier-execution-decision.md)
を現場の着手単位へ落とす。M3の製品資産順序は一粒ずつ維持する一方、M4/M5の独立contract
spike、M2の狭い修復、Vism作者入口を同じ待ち列へ入れない。

この地図はschema、公開API、Document意味、plugin trust境界を解凍しない。各laneが共有contract
変更を必要とした時は、そのlaneだけを止めてdecision/specへ戻す。

## 2. 状態語

| 状態 | 意味 |
|---|---|
| `READY` | 現行authorityに意味・依存・完了条件・STOPがあり、closed order作成へ進める |
| `READY-CHECK-PATH` | 意味は成立済み。変更許可file listの非重複を確認した時だけ起動できる |
| `READY-SPEC` | 独立したspec/decisionだけ開始でき、実装はまだ待つ |
| `READY-HUMAN` | 成立済み成果物への人間応答。無関係laneを止めない |
| `WAIT` | 依存または意味が未成立。read-only調査を越えない |
| `CONTROL` | task単位の短い照合。全件完了を共通barrierにしない |

## 3. Wave 0

| lane | 現在粒 | 状態 | 最初の成果 | STOP / 負例 | Human Response Frontier |
|---|---|---|---|---|---|
| PRODUCT-ASSET | `CU-0A05A / R2A` | `READY` | 固定mock内でEasing triggerを同形React化し、既存visual/interaction parityを閉じる。product差分は0 | product file、popup全体、curve/Undo state、visible summary chrome、別leafが必要ならSTOP | Wave 0はarchived HTMLと同形React triggerのparity比較。product面の比較は`CU-0A05B`後 |
| VISUAL-RESPONSE | `G0-6H` | `READY-HUMAN` | 5 reference screen / 30 PNGへの人間応答を記録 | `U0e-3`以外を止めない。pixel testで人間判断を代替しない | visual tokenと認知の応答 |
| AUTHORING-SCAFFOLD | `VSM-A4S` | `READY-SPEC` | 外部crate作者scaffoldと既存in-tree generatorの責任を分けたclosed contract | package/install/manifest、dynamic loader、第三者配布完成、実装を含めない | `VSM-A4I`後に外部crate生成からconformanceまでのdeveloper response |
| SPATIAL-CONTRACT | `M4-K0 / #167` | `READY` | `Finite / Infinite / Unknown`、RoD/RoIのfixtureと凍結判定。schema/最適化変更0 | 未検証pluginのFinite扱い、同期readback、px/Document焼込み、legacy/deprecated constructorを使わない | Blur/transform/Unknown fallbackの比較fixture。製品操作面とは称さない |
| IDENTITY-CONTRACT | `M5-P0I / #170` | `READY` | `InstanceId != index`、slot key、nested context、seed goldenのfixture | schema/公開Effector API追加、index/thread/GPU順のidentity化、legacy/deprecated constructorを使わない | count/reorder後の個体追従を比較するfixture。製品操作面とは称さない |
| M2-REPAIR | `GAP-23` → `GAP-24` | `READY` | suppression無しlint policy、known apply failureのtyped stop | Document/wire/snapshot形式、oracle値、期待値を変えない。同一lane内では直列 | 人間応答なし。基礎の出戻り半径を縮める |
| ORACLE-GUARD | `GAP-25` | `READY-CHECK-PATH` | workflow/script/protected pathのfail-closed負例 | oracle値、variant、toleranceを変えない。GAP-23との変更path重複時はGAP-23後へ直列化 | 人間応答なし。並列laneによるgate自己弱体化を拒否 |

最小の即時並列集合はPRODUCT-ASSET、VISUAL-RESPONSE、AUTHORING-SCAFFOLD、M2-REPAIRの
4 laneである。本決定と同じ変更で旧全体直列文言を撤回したため、K0/P0IもWave 0へ合流できる。
GAP-25はGAP-23との変更許可pathを機械照合した後だけ起動する。

## 4. lane所有と衝突規則

- PRODUCT-ASSETは固定mockのReact source closureだけを触る。`CU-0A05B`までproduct packageを触らない。
- VISUAL-RESPONSEの`reference-handoff.md`とPRODUCT-ASSETの変更file listを起動前に照合する。
- AUTHORING-SCAFFOLDはspec/decisionだけ。runtime、package、Document、loaderを触らない。
- K0はschemaと最適化を、P0Iは製品schemaと公開Effector APIを触らない。
- K0/P0Iのfixtureは`new_v1`等のlegacy/deprecated constructorを使わない。
- GAP-23/24は同じ`motolii-doc`を触り得るため、一つのM2-REPAIR lane内で直列にする。
- GAP-23とGAP-25の変更許可pathに重複があれば、GAP-25をGAP-23後へ移す。
- isolated worktree、1 ticket=1 commit、各task classに必要な独立検収は維持する。
- 同時起動前に変更許可file listの積集合を機械確認する。共有contract変更が必要なら当該laneだけSTOPする。

## 5. lane-localな直列性

旧「Selected U series中はK0/P0Iも同時着手しない」運用は撤回する。一時点で`DO`一粒という規律は
PRODUCT-ASSET lane内だけに残し、`CU-0A05A → CU-0A05B → CU-0A06...`をrollingに解禁する。
これはM3の意味・所有境界の順序を保つためで、M4-K0、M5-P0I、M2修復、Vism仕様laneへ
波及させない。

## 6. CONTROL

| control | 役割 | barrierにしない条件 |
|---|---|---|
| SEAT-INVENTORY | taskごとにowner、input/output、failure、多重度、変更path、test、分類を確認 | readyな一taskを確認した時点で当該laneを解禁し、全seatを待たない |
| FABLE-SHARED-REVIEW | 共有contract、hidden dependency、P0/P1をintegration waveで監査 | leafごとの必須待ちにしない。該当laneまたはintegration waveだけを止める |

各closed orderに必要なGrok/Fable検収を、この横断controlで代替してはならない。検収queueが
実質的な全体直列背骨になった場合はlane数とintegration waveを縮め、review品質を下げない。

## 7. WAIT

| candidate | 不足 | 次に可能なこと |
|---|---|---|
| render worker instance交換 | respawn/quiescence/rollback contractとtask IDが無い | current seamのread-only inventoryとfixture案 |
| `INF-6` journal/session完全復元 | 通常編集commit点へのjournal接続と製品reopen routeが未成立 | apply→append→kill→replay oracleのspec候補 |
| `INF-8` hot reload | INF-6、M4 cache、React product packageの依存 | WGSL watcher/HMR/restartを別粒へ分解するspec候補 |
| `K1a`以後 | K0 | K0結果を自動採用せず各seatを再判定 |
| `P7a`以後 | P0I | P0Iからschemaを自動生成せずGR-PV decisionへ戻す |
| `VSM-A4I` | A4SとVism計画§8.1の全体レビュー | scaffold実装だけ。package/loaderと束ねない |
| `VSM-A9` | A4、A5、対象lane contract | plugin量産の非干渉gate。共有API変更候補はSTOP |
| `CU-0A05B`以後 | PRODUCT-ASSETの直前粒 | lane内で一粒ずつ解禁 |

## 8. rolling merge

1. READY laneごとにclosed orderと変更許可file listを固定する。
2. 同時起動前にfile listの積集合と意味衝突を確認する。
3. 各laneは自分のfixture/frontierまで進み、他lane完了を待たない。
4. 共有contractを変えないleafからmergeする。
5. integration waveだけFableへ横断P0/P1を問い、Codexが現行authorityへ再照合する。
6. 人間へはPRODUCT-ASSET、VISUAL-RESPONSE、将来AUTHORINGのfrontierが届いた順に返す。

## 9. 反対側レビュー

Fable 5の初回判定は`VERDICT: REVISE`、P0=0/P1=2だった。

1. PRODUCT-ASSETが`CU-0A05A`とWAIT中の`CU-0A05B`を混同していた。
2. K0/P0IのREADY化と旧台帳の全体直列文言が衝突していた。

本書§3でR2Aをmock-side parityへ限定し、§5と
[実装進行台帳](../implementation-ledger.md)を同じ変更でlane-local運用へ改訂した。
path衝突、legacy constructor、VSM-A4I全体review gateのP2も§4/§7へ反映した。
