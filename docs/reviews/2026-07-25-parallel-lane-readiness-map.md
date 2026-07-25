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
| `DONE` | main到達済み。後続は成果範囲を自動拡張せず、各依存を再判定する |
| `READY` | 現行authorityに意味・依存・完了条件・STOPがあり、closed order作成へ進める |
| `READY-RECHECK` | 依存の完了条件は満たされたが、依存元の成果が当該粒の必要責任を含むかを再判定するまでclosed orderを作らない |
| `READY-CHECK-PATH` | 意味は成立済み。変更許可file listの非重複を確認した時だけ起動できる |
| `READY-SPEC` | 独立したspec/decisionだけ開始でき、実装はまだ待つ |
| `READY-HUMAN` | 成立済み成果物への人間応答。無関係laneを止めない |
| `WAIT` | 依存または意味が未成立。read-only調査を越えない |
| `CONTROL` | task単位の短い照合。全件完了を共通barrierにしない |

## 3. Wave 0

| lane | 現在粒 | 状態 | 最初の成果 | STOP / 負例 | Human Response Frontier |
|---|---|---|---|---|---|
| PRODUCT-ASSET | `CU-0A07B / R4B` | `READY`（ledger `DO`） | [#353](https://github.com/oshikaidesu/Motolii/pull/353)の未変更source oracleを保ち、固定mock内でInspectorを同形React化してlegacy adapterを一方向へ封じる | oracle/archived HTML/threshold/golden変更、skeleton代用、双方向store、Host projectionが必要ならSTOP | R4B完了時だけR4Cを再判定する |
| VISUAL-RESPONSE | `G0-6H` | `READY-HUMAN` | 5 reference screen / 30 PNGへの人間応答を記録 | `U0e-3`以外を止めない。pixel testで人間判断を代替しない | visual tokenと認知の応答 |
| AUTHORING-SCAFFOLD | `VSM-A4S` | `READY-SPEC` | 外部crate作者scaffoldと既存in-tree generatorの責任を分けたclosed contract | package/install/manifest、dynamic loader、第三者配布完成、実装を含めない | `VSM-A4I`後に外部crate生成からconformanceまでのdeveloper response |
| DELEGATION-GUARD | `GR-D3 / #329` | `DONE` | [#336](https://github.com/oshikaidesu/Motolii/pull/336)で既知のworktree-root派生物だけを検収前にfail-closed清掃し、ignored監査を維持 | `target/**` allowlist、fingerprint除外、`.gitignore`／build script／製品code変更、未知entry削除は引き続きSTOP | 人間応答なし。実K0停止形、Grok `ACCEPT`、CI 4/4で完了 |
| SPATIAL-CONTRACT | `M4-K0 / #167` | `DONE` | [PR #338](https://github.com/oshikaidesu/Motolii/pull/338)で15 test契約凍結、Grok `ACCEPT` P0=0 P1=0 P2=1 | 旧未検収worktreeの自動採用、未検証pluginのFinite扱い、同期readback、px/Document焼込み、legacy/deprecated constructorを使わない | 人間応答なし。runtime昇格はK1a以降の別判断 |
| IDENTITY-CONTRACT | `M5-P0I / #170` | `READY-SPEC` | Distribution continuity、transform合成、nested identity、domain寿命、cache入力境界、PRNG処分をdocs-onlyで決める | schema／公開Effector API／Rust fixture／golden追加、TextCluster内部写像やPrototype ownerの先取りをしない | 決定merge後にcount/reorder後の個体追従fixtureを分割して再入 |
| M2-REPAIR | `GAP-23` → `GAP-24` | `WAIT` | 独立したD1i-4 LookAt/Follow oracle分離の採番・完了後に、25 suppressionの除去へ戻る | whole-file semantic分類、oracle値、期待値、regenerate markerを修復都合で変えない | 人間応答なし。先行oracle分離だけを別粒にする |
| ORACLE-GUARD | `GAP-25` | `READY-CHECK-PATH` | workflow/script/protected pathのfail-closed負例 | oracle値、variant、toleranceを変えない。GAP-23との変更path重複時はGAP-23後へ直列化 | 人間応答なし。並列laneによるgate自己弱体化を拒否 |

最小の即時並列集合は、PRODUCT-ASSETのR2B product ownership、P0Iのdocs decision、
VISUAL-RESPONSE、AUTHORING-SCAFFOLDである。旧全体直列文言は撤回したままだが、P0I fixtureと
GAP-23実装は各lane-localな前提へ戻す。K0は完了済みで、K1aは責任最小化の再判定まで起動しない。
GAP-25はGAP-23との変更許可pathを機械照合した後だけ起動する。

## 4. lane所有と衝突規則

- PRODUCT-ASSETはR2Aのinventory／mock owner実装を[#341](https://github.com/oshikaidesu/Motolii/pull/341)で閉じた。
  R2Bは同じsource closureをproduct ownerへ直接移し、mockをconsumerへ反転する範囲だけを扱う。
- VISUAL-RESPONSEの`reference-handoff.md`とPRODUCT-ASSETの変更file listを起動前に照合する。
- AUTHORING-SCAFFOLDはspec/decisionだけ。runtime、package、Document、loaderを触らない。
- K0はschemaと最適化を触らない。P0Iはdocs decisionだけを進め、製品schema、公開Effector API、
  fixture、goldenをまだ追加しない。
- GR-D3は[#336](https://github.com/oshikaidesu/Motolii/pull/336)でmain統合済み。ignored pathの監査を弱めず、
  未知の`target/` entryを削除しない契約を維持する。解禁後のK0は[#338](https://github.com/oshikaidesu/Motolii/pull/338)で完了した。
- K0のfixtureは`new_v1`等のlegacy/deprecated constructorを使わない。P0I fixtureはdecision merge後に
  同じ負例を持つclosed orderへ分割する。
- GAP-23/24は同じ`motolii-doc`を触り得るため、一つのM2-REPAIR lane内で直列にする。GAP-23の前に
  LookAt/Follow oracle分離を独立taskとして採番し、whole-file semantic分類を直接変更しない。
- GAP-23とGAP-25の変更許可pathに重複があれば、GAP-25をGAP-23後へ移す。
- isolated worktree、1 ticket=1 commit、各task classに必要な独立検収は維持する。
- 同時起動前に変更許可file listの積集合を機械確認する。共有contract変更が必要なら当該laneだけSTOPする。

## 5. lane-localな直列性

旧「Selected U series中はK0/P0Iも同時着手しない」運用は撤回する。一時点で`DO`一粒という規律は
PRODUCT-ASSET lane内だけに残し、`CU-0A05A → CU-0A05B → CU-0A06...`をrollingに解禁する。
これはM3の意味・所有境界の順序を保つためで、M4-K0、M5-P0Iのdocs decision、M2修復の前提粒、
Vism仕様laneへ波及させない。P0I fixtureとGAP-23実装のWAITは全体直列ではなくlane-localである。

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
| `K1a`以後 | 依存先行の責任最小化ゲート(K0依存は充足済み) | K0はtest-only契約凍結でruntime region関数を提供しない。K1aが必要とする責任を列挙し、K0成果を自動採用せずseat単位で再判定する(`READY-RECHECK`) |
| `P0I` fixture | P0I docs decision | 意味decisionをmergeし、fixture粒と負例を分割して再判定 |
| `P7a`以後 | P0I完了 | P0Iからschemaを自動生成せずGR-PV decisionへ戻す |
| `GAP-23` | 独立D1i-4 LookAt/Follow oracle分離 | task IDとoracle artifact／harness閉包をspec化して先行 |
| `VSM-A4I` | A4SとVism計画§8.1の全体レビュー | scaffold実装だけ。package/loaderと束ねない |
| `VSM-A9` | A4、A5、対象lane contract | plugin量産の非干渉gate。共有API変更候補はSTOP |
| `CU-0A05B`以後 | PRODUCT-ASSETの直前粒 | lane内で一粒ずつ解禁 |

## 8. rolling merge

1. `READY` laneはclosed orderと変更許可file list、`READY-SPEC` laneはdocs decisionの閉包を固定する。
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

## 10. Wave 0 prepare後の訂正

2026-07-25のOpus 5 prepareとFable 5 read-only助言で、Spark起動前に次のlane-localな前提を検出した。
いずれも実装、commit、pushは行っていない。

1. `CU-0A05A`: 固定commit／trigger authorityは不変oracle、抽出後sourceは既存runtime closureの
   mock-side provenanceとし、phase固有hash表を追加しない。decision merge後は本地図で`READY`
   （implementation ledgerで`DO`）、R2A ACCEPT後は`DONE`かつ`CU-0A05B READY-RECHECK`とする。
   Timelineだけが`activeInterval`を導出し、fixture adapterがlegacy trigger mutator/listenerを
   exact-matchで封じ、controlled triggerへ一方向に配線する。
2. `P0I`: 一つのfixture粒へ意味決定まで押し込んでいた。P0I自身が閉じるdocs decisionと、Text／Prototype
   側へ残す明示留保を分け、その後にfixtureを複数粒へ閉じる。
3. `GAP-23`: 25件目のsuppressionがwhole-file semantic保護中のLookAt/Follow harnessにあり、
   GAP-23正本どおり独立D1i-4 oracle分離が先行する。

K0のprivate test-only contract spikeはこの三件と契約境界が重ならない。実施工で生じた
workspace試験の派生`target/`とrunner scope closureの衝突はGR-D3で解消し、旧K0隔離差分を採用せず
fresh main/worktree/orderから再入した結果、[#338](https://github.com/oshikaidesu/Motolii/pull/338)で`DONE`となった。
次はK1aを`READY-RECHECK`に留め、K0のprivate modelをruntimeへ自動昇格しない。
