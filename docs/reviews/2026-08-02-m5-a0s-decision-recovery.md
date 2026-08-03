# M5-A0S 3D import／Render Contribution 作品意味の決定回収

状態: **決定（docs-only）**（2026-08-02）

## 1. 目的と境界

`M5-A0T`で技術routeを固定した後、mainへ未統合の`416aa2c2`系列と、main外の
`33e957df`系列に残る作品意味を、現行M5正本・現行code fact・既存停止線へ再照合する。
これは歴史資料をruntimeや公開契約へ自動昇格する作業ではない。採用するのは現行境界内の
最小意味だけであり、field、schema、crate内部型、公開API、serde、Document、plugin契約は
本書から新設しない。

母集団は2026-07-23 cutoffの固定`corpus.tsv`へ混ぜず、後発refを別deltaへ分離した。
delta manifestは[`2026-08-02-m5-a0s-refs.tsv`](evidence/historical-value-recovery/deltas/2026-08-02-m5-a0s-refs.tsv)、
処分receiptは[`2026-08-02-m5-a0s.tsv`](evidence/historical-value-recovery/delta-disposition-receipts/2026-08-02-m5-a0s.tsv)である。

## 2. authorityとの照合

現行の意味authorityは[M5仕様](../specs/M5-3d-and-post.md)、[M5既知実装採択・検証地図](../m5-known-implementation-adoption-map.md)、
[決定逆引き台帳](../decision-index.md)、[実装台帳](../implementation-ledger.md)である。
現行codeにはGLB／OBJ importer、faithful asset、spatial Observation、Render Contributionの
製品経路、M4 resource owner、製品post／picking／Duplicatorはまだ存在しない。従って本書の
処分は`M5-A0S`の完了条件を満たすが、製品runtimeの着手・実装完了を意味しない。

## 3. 候補blobの処分

| blob SHA | observed path | 処分 | 回収した価値 | 採用しないもの／停止線 |
|---|---|---|---|---|
| `b1ec748946dc242033abc248c21e7f3cc3ebb81b` | `docs/reviews/2026-08-01-m5-3d-import-rendering-boundary-decision.md` | 縮小採用 | faithful import assetとrenderer-compiled assetを分離し、core PBRとneutral environmentを基準にする。`gltf`／`tobj`はprivate leaf、bare一灯／auto-unlitへの無言縮退を拒否する | exact schema、extension全対応、public asset型、製品受理範囲は未決。`gltf`成功をrenderer成功とみなさない |
| `bf8254dc39db4a210cd0c0e52d57ddcca33863c8` | `docs/reviews/2026-07-29-m5-render-contribution-boundary-facts-v3.md` | 棄却（archive-only negative） | 停止されたtemplateがwhole-request、format、budget、resource ownerを先に閉じる必要を示す | templateのorder、未確定field、失効したIDを現行実装へ戻さない |
| `f5bbaeb38321f0bd3c9e517ccca6ec165d85a083` | `docs/reviews/2026-07-29-m5-render-contribution-contract-draft.md` | 棄却（archive-only negative） | 公開seamを発明してはならないという負例 | draftの公開型、永続形式、renderer ownershipを採らない |
| `9f7edb06ef47cee9dcc0b11c6d56ef05ecfe74bb` | `docs/reviews/2026-07-29-m5-render-contribution-design-closure-map.md` | 縮小採用 | requirementとcontributionを分け、Hostがworld／camera／Observation／resource／budget／diagnosticを所有し、全requestを一度にadmitする | P3 Observationの具体公開形、GPU format／copy／hard budgetは別decisionで閉じる |
| `431640c62c610332d5b435ef05d28bd96527e7e5` | `docs/reviews/2026-07-29-m5-render-contribution-parallel-wave-lessons.md` | 観察 | 並行waveで意味契約を先に閉じ、実装・検収を混ぜない運用上の教訓 | wave順をM5 runtimeの依存や完了証拠へ読み替えない |
| `0d1fb059fea35148356568832348143476a710e1` | `docs/reviews/2026-07-29-m5-render-phase-primary-source-evidence.md` | 棄却（archive-only negative） | 一次資料が未取得のtemplateを根拠にしない | source未取得の主張、phase番号、外部実装の採用理由を復活させない |
| `7877f14b3748c709466c26fd3a827f4d81cd0501` | `docs/reviews/2026-07-29-m5-render-contribution-typed-seam-decision.md` | 縮小採用 | linear-premultiplied scene color、soft alphaのtyped unsupported、format／copy／budget evidence gate、Host resource owner | typed seamの具体公開API、serde／Document field、soft alphaの近似実装は未成立 |

候補7 blobは外部delta projectionで全文を読み、各byte列を対応commitの`git cat-file blob`
へ照合した。候補packetとprojectionは再生成可能な外部作業物であり、現行repoの固定corpusの
代替ではない。

## 4. 現行へ回収する最小意味

上表の`縮小採用`だけを、M5正本の既存境界へ次のように回収する。

1. importはfaithful assetとrenderer-compiled assetを分離し、parserの成功を描画の成功としない。
2. Render Contributionはrequirementを単一のadmissionで判定し、複数のcontributionをHost所有の
   world／camera／Observation／resource／budgetへ接続する。未対応は無言のunlit／opaque化でなくtyped refusalとする。
3. scene colorはlinear-premultipliedを合流境界の意味とし、soft alphaをsingle depthへ無言で
   格上げしない。具体format、copy／alias、budgetは別のP3／M4 evidence gateで決める。
4. runtimeへ進む順序は、既存M5 mapの`M5-C0` ObservationとM4 resource gateを先に閉じ、
   その後にLayer Orderへの薄い接続を行う。

## 5. 非採用と停止線

- `416aa2c2`／`33e957df`をmain統合済み、または製品runtime実装済みとは報告しない。
- 歴史文書のtemplate、未取得一次資料、失効IDから公開API、Document、serde、plugin契約を発明しない。
- parser／headless PBR／algorithm fixtureの検証成功だけでM5 runtime、3 OS、PBR conformance、
  M4 ownership、GPU pass／readbackを完了扱いしない。
- P3 Observation、scene-color format／copy／budget、soft-alpha対応、製品resource ownerは、
  個別の正本・fixture・負例が閉じるまで実装へ送らない。

## 6. HVR-D04 evidenceと完了条件

| 項目 | 値 |
|---|---|
| delta refs | `416aa2c22c39bdfea30344ce2cbec87ccacedbec`, `33e957df290857b5123db9b97006438fbcfbbfc6` |
| external projection tree | `44ba35d493feb37816fb9c3639ad8765f3e9bc60a68180d02e66a68bdf6d3170` |
| delta corpus | 3,300 blobs（read 0／dispositioned 0／remaining 3,300 の外部初期状態） |
| selected and disposed | 7 blobs、全文読了、byte照合済み |
| fixed cutoff corpus | 1,797 blobsを変更せず、固定checkerの母集団を増やしていない |

`M5-A0S`は本書とdelta receiptの追加をもって**DONE（docs-only）**とする。次のhandoffは
`M5-C0`（Planar／Spatial Observation）とM4 resource gateの意味・evidence閉鎖であり、
それらが閉じるまで製品runtimeの実装・依存追加・公開API変更は行わない。
