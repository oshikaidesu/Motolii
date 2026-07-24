# 並列Human Response Frontier実行決定

作成日: 2026-07-25

状態: **決定**

対象: Controlled Microkernel締結後の実装、検収、製品接続、Fable反対側レビュー、人間確認の配置

## 1. 決定

Controlled Microkernel、Host capability module、plugin trust、hot reload／crash recoveryの
architectural judgmentは完了した。同じ意味を各実装laneで人間へ問い直さない。

以後の目的は、milestone全体や全seat inventoryの完了を待つことではなく、締結済みcontractごとに
provider、consumer、fault fixture、製品接続を並列に進め、**人間が実際に触る、見る、聞く、比較する、
性能を体感する地点**へ最短で到達させることである。この地点を`Human Response Frontier`と呼ぶ。

```text
決定済みauthority／contract
          │
          ├─ provider lane ───────┐
          ├─ consumer／UI lane ───┤
          ├─ fault／recovery lane ├─→ runnable product response
          ├─ fixture／measure lane┤          │
          └─ integration adapter ─┘          ▼
                                      human response
```

人間は各moduleの内部設計を逐次承認する担当ではない。既存決定では判定できない感覚品質、操作結果、
速度、失敗時体験、優先順位を、通常製品route上の具体物へ応答する。

## 2. 「人間審判が終わった」の範囲

次は再審判せず、実装とfixtureの前提にする。

- Coreをauthority＋typed protocolへ細くすること。
- 具体実装をadmitted Host capability moduleへ分離できること。
- first-party／third-party公開pluginを等しく非信頼とすること。
- TCBをControlled Microkernel＋admitted Host moduleへ限定すること。
- hot reloadとcrash recoveryをHost所有状態からのinstance交換へ畳むこと。
- single writer、atomic commit、effect順等の意図的直列点を維持すること。
- contract／実装／runtime／lifecycleの四面を別々に並列化すること。

ただし、既存docsが明示的に未決または人間gateとして残した具体token、見た目、操作感、hardware実測、
公開API、永続形式、runtime方式を、この宣言だけで決定済みに繰り上げない。新しい意味が必要になったlaneは
STOPするが、無関係なlaneを同時に止めない。

## 3. laneの開始条件

全seatのinventory完了を共通barrierにしない。各laneは次を満たした時点で個別に開始できる。

1. 対象seatのowner、input、output、failure、多重度が既存正本から読める。
2. 変更許可範囲と、変更してはいけない共有contractが閉じている。
3. fake、参照provider、既存consumer、または固定fixtureのどれかで単体成果を判定できる。
4. public API、Document、serde面を変えずに最初のproofへ到達できる。
5. lane固有のSTOP条件と負例がある。

不足を見つけたら、そのseatだけを`EXTRACT CONTRACT`または`NOT YET PROVEN`へ戻す。他seatの
provider／consumer実装を全体待ちにしない。

## 4. Human Response Frontierの完成形

人間へ返すのは内部trait、isolated test、diagnostic routeだけではない。最低一つ、次の形へ到達させる。

| response種別 | 人間へ返す具体物 |
|---|---|
| visual | 通常製品windowで同じDocument revisionを描く製品surface |
| interaction | gesture、preview、commit、Undo／Cancelまで通る操作 |
| recovery | plugin／worker停止を起こし、作品を失わず局所診断・再生成する挙動 |
| performance | 固定fixtureと基準機で、待ち時間、frame time、memory、復旧時間を比較できる結果 |
| authoring | 一つの表現を公開façadeだけで追加し、製品Browserから使用・保存・再openできる経路 |

人間へ返す前にも自動審判は通すが、自動test緑をHuman Response Frontier到達の代わりにしない。
逆に、人間の感想待ちを理由に、意味が独立した別laneのfixture、provider、consumer実装を止めない。

## 5. rolling wave

実装は一つの巨大waveでなく、frontierへ届く単位のrolling waveにする。

1. 共有contractの最小proofを閉じる。
2. 同contract上のprovider、consumer、fault、measureを並列起動する。
3. 最初に通常製品routeへ届いたsliceを人間へ返す。
4. 人間応答は該当surface／体験へだけ反映し、同じcontract上の無関係laneを巻き戻さない。
5. 次のseatを解禁しながら、前waveのintegrationとrecoveryを閉じる。

全lane完了後にまとめて見せるwaterfallへ戻さない。一方、isolated moduleの完成数を製品進捗にしない。

## 6. Fableの配置

Fableは人間またはCodexの代わりに仕様を決めるauthorityではなく、**共有境界の反対側reviewer**である。

Fableを使う:

- 複数seatへ波及するcontract変更候補。
- runtime並列、failure containment、cache key、resource、lifecycle等の横断境界。
- rolling waveを通常製品routeへ合流する前のP0/P1監査。
- 現行コード事実とdocsの完成扱いが一致するかの定期監査。
- 人間へ返す比較軸や負例の漏れを探す時。

Fableを共通barrierにしない:

- 締結済みcontract上の独立leafすべてを一件ずつ待たせない。
- Fable実行中も、変更範囲が重ならないprovider、consumer、fixture laneを進める。
- 一つのreview失敗は該当contract／integration waveだけを止め、無関係laneへ伝播させない。
- Fableの助言を未検証のまま正本化しない。Codexが現行コード、fixture、既存決定へ再照合する。

Fableを多用しても、review queueが新しい直列の背骨になった時点で運用失敗である。leaf reviewでは
既存conformanceを優先し、Fableは共有面と高risk境界へ集中させる。

## 7. 直近の並列frontier候補

[Fable反対側レビュー](2026-07-25-controlled-microkernel-fable-counter-review.md)の現行コード事実から、
read-only seat inventoryは全件完了待ちのphaseでなく、laneを順次解禁する短いcontrol taskとして扱う。

| lane候補 | 最初のproof | response frontier |
|---|---|---|
| 表現plugin | 既存contract／conformance上の独立provider | Browserから配置、保存、再open、Preview／Export |
| Preview／Export consumer | 同じsnapshot、時刻、Final意味のoracle | 製品Previewとartifactの意味比較 |
| render worker lifecycle | worker破棄→再spawn→同一revision再評価 | 局所停止と復旧時間を見せる |
| journal durability | apply→append→kill→replayの変更0 fixture | 強制終了後に同じ作品へ戻る |
| UI projection | revision付きsnapshot＋typed intent | Stage／Timeline／Inspectorの同一対象同期 |
| cache／resource | fake／null providerとbudget fault | 重いfixtureの応答性と縮退理由 |

この表は実装orderや公開contractの承認ではない。各laneの既存task、STOP、現在の製品enabling orderを
照合し、開始条件を満たしたものから並列にfrontierへ送る。

## 8. 成功指標

- architecture判断から最初のHuman Response Frontierまでの時間。
- 同時に進められた独立lane数と、共有file／contract衝突数。
- provider追加時に変更したCore、Document、consumerの数。
- isolated test完了から通常製品route到達までの滞留時間。
- failureが該当capabilityへ局所化された割合と、復旧時間。
- Fable待ちで停止した無関係lane数。目標は0。
- 人間応答が巻き戻した範囲。該当surface／contractを越えて広がるなら境界を再監査する。

## 9. 非目標

- 人間確認を廃止すること。
- 未決の公開API、Document schema、runtime、GPU共有方式を自動決定すること。
- すべての処理を同時実行し、意味上必要な順序を消すこと。
- Fableを仕様authorityまたは全leafの必須承認者にすること。
- 通常製品routeへ届かない大量のmodule完成を進捗として積むこと。
