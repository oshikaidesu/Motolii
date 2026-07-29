# M5 Render Contribution設計締結地図

作成日: 2026-07-29

状態: **決定**

[Fable 5最終反対側レビュー](2026-07-29-m5-render-contribution-fable-final-review.md)は、
初回REVISEの依存同期漏れを訂正後、`ACCEPT`、P0=0／P1=0／P2=0となった。

## 0. Start Here

M5 Render Contributionへ着手する時は、次の順だけを使う。

1. 本地図§1〜§2と§8で、締結済み意味、残gate、「完了」の語彙を分離する。
2. [M5仕様](../specs/M5-3d-and-post.md)で意味と完了条件を、
   [implementation ledger](../implementation-ledger.md)で現在状態と発注順を確認する。
3. 個別の意味が必要な時だけ、本地図§2から各decisionへ入る。
4. [証拠Wave](2026-07-29-m5-render-contribution-evidence-wave.md)とcapsule／map／停止済みtemplateは、
   決定理由を監査する履歴としてだけ読む。現行要件、状態、再発注の根拠にしない。

M5内の権限順位は、repo共通の
[台帳の責任分離](../implementation-ledger.md#この台帳の責任)を具体化して次の順とする。

1. M5仕様と各decision: 意味、公開境界、完了条件。
2. 本締結地図: decision間のgate構造と完了語彙。
3. implementation ledger: 現在状態、発注順、後継粒。
4. Fable最終レビューと並列Waveの教訓: 反対側審判／観察。authorityではない。
5. evidence wave、capsule、map、停止済みtemplate: 履歴証拠。失効したnavigationや状態を復活させない。

`P2D-RCA`／`RCB`／`RCC`から`RCA8`／`RCB6`／`RCC5`までの証拠粒は
`P2D-RCI`へ統合済みであり、同じID、後継ID、別名で再発注しない。既決意味を再度開くのは、
本地図§9のSTOPに該当する新しいコード反例、仕様衝突、またはscope改訂が生じた場合だけとする。

## 1. 結論

Render Contributionの**意味設計**は閉じた。残る未決は、思考だけで埋める仕様穴ではなく、
P3 contract、GPU実機証拠、M4-K1 ownerへ入力が明示された四つのgateである。

したがって「M5実装は全部すぐ発注可能」ではない。設計済み実装粒と、証拠取得後にだけ閉じる
method／public-boundary decisionを区別する。

## 2. 閉じた意味境界

| ID | 閉じたもの |
|---|---|
| [P2D-RCI](2026-07-29-m5-render-contribution-integration-decision.md) | requirement／contribution分離、Host owner、追加的能力 |
| [P2D-RCS1](2026-07-29-m5-render-contribution-integration-decision.md#6-private-seamの実機証拠と次の解禁) | opaque private feasibility。公開根拠にはしない |
| [P2D-RCD1](2026-07-29-m5-render-contribution-typed-seam-decision.md) | 一／一／many、whole-request admission、trust前段 |
| [P2D-RCD2](2026-07-29-m5-occlusion-policy-schema-decision.md) | Group policy semantic key、Item participant、migration／Undo設計 |
| [P2D-RCF1](2026-07-29-m5-render-contribution-conformance-harness-decision.md) | provider中立harness、F1〜F6分担、First Vism無特権 |
| [P2D-RCT1](2026-07-29-m5-render-contribution-alpha-semantics-decision.md) | opaque／cutout／soft-alpha意味とtyped failure |
| [P2D-RCO1](2026-07-29-m5-soft-alpha-oit-disposition.md) | v1 OIT方式非採択、shared-depth soft alpha typed unsupported |
| [P2D-RCFP1S](2026-07-29-m5-scene-color-semantics-decision.md) | linear-light／premultiplied scene-color意味 |
| [P2D-RCR1](2026-07-29-m5-scene-color-input-contract-decision.md) | immutable upstream snapshot、K0 RoI、domain外、failure |

## 3. public seam gate

`P2D-RCD1`は公開Rust item形を意図的に決めていない。contributionがworld／camera／Observationを
所有しない一方、評価時にHostから読むtyped context形はP3 Observation Contractに依存する。

| ID | 内容 | 依存 | STOP |
|---|---|---|---|
| `P2D-RCD1A` | public typed request／admitted contribution API shape decision | P3 Observation decision、RCD1/RCT1/RCR1 | `CompCamera`を将来Observationへ偽装、P3能力先取り、万能callback／raw texture／phase enum |
| `P2D-RCD1I` | approved public seam implementation | P2D-RCD1A | Document／schema／package変更、private RCS1昇格、First Vism分岐 |

RCF1実装は`RCD1I`後に始める。P3前にprivate adapterを公開seamと称しない。

## 4. concrete format evidence gate

`P2D-RCFP1F`は次を固定fixtureで測り、formatを採択する。

- `Rgba32Float` referenceに対する`Rgba16Float`候補のblack／white／中間調／near-zero alpha、
  複数over、1.0超highlight、blur、subpixel scene sample。
- DRAFT／FINAL別absolute／relative error、banding、NaN／Inf／subnormal、反復決定性。
- 対象backendのrender attachment／blend／sample／filter／copy usage。
- K0 `Finite / Infinite / Unknown`とclamp後extent。
- format、sample、mip、alignment、同時live resourceを含むbyte量。
- unsupported時のtyped refusal。RGBA8／sRGB fallbackなし。

numeric toleranceは測定前に変更せず、semantic oracleとformat comparison artifactを分離する。
hard capの採択はRCBUD1へ残す。

## 5. copy／subpass evidence gate

`P2D-RCP1`はRCR1の同じimmutable snapshot意味を保つHost内部methodを比較する。

最低候補:

- distinct scene-color copy。
- render graphが所有するalias-safe immutable resource。
- 対象backendが実際に許す場合だけinput attachment／subpass相当。

合否:

- canonical pixel、transparent-black domain外、Preview／Export、DRAFT／FINALがmethod間で一致。
- requesterのlive outputと同一resource／stepでread-writeしない。
- snapshot lifetimeが全consumer終了まで保たれ、stale generationを読まない。
- copy bytes、追加attachment、pass／barrier、peak live bytes、GPU timeを1080pで計測。
- unsupported backend、budget不足、offscreen realization不能はtyped no-output。
- method名、texture handle、barrier、backend APIを公開contract／cache keyへ出さない。

Host内部で複数backend methodを残してよいが、観測意味とresource accountingは同一contractへ通す。

## 6. cache／budget owner gate

`P2D-RCBUD1`はM4-K1の次が成立するまで閉じない。

- allocationと同寿命のowned／reference handle。
- product Host budget policyと一つのshared ledger。
- output、scene-color copy、depth、transient contribution resourceのowner分類。
- hard-cap admissionとtyped no-output。

RCBUD1のcache identityはRCR1 §8のsemantic inputsを完全に含め、GPU handle、copy method、
backend pass ID、UI stateを含めない。format／extent／sample／mip／alignmentの実byteはRCFP1F／RCP1から
受け取る。budget不足でrange、Quality保証、alpha class、policyを黙って変えない。

M4-K1aの一owner accountingやprivate grantだけを、外部cloneを含むoutput lifetimeの完成証拠にしない。

## 7. 実装可能な並列

今すぐ並列に発注前準備できる（状態は各仕様／台帳の`WAIT`を維持し、発注時に`ISSUE`／`DO`へ上げる）:

- `P2D-RCD2I`: schema／migration／D2 command。render接続なし。
- `P2D-RCFP1F`: private format evidence fixture。
- `P2D-RCP1`用benchmark harnessのfixture設計。ただしmethod採択はRCFP1F後。
- M4側のoutput handle／budget policy closure。

依存後:

- P3→`RCD1A`→`RCD1I`→`RCF1I-BASE`。
- `RCF1I-BASE`→`RCF1I-ALPHA`。
- `RCFP1F`→`RCP1`→`RCF1I-SCENE`。
- M4-K1 handle／budget + RCFP1F + RCP1→`RCBUD1`。
- 製品表現選定後だけ`RCF1I-VISM`。

## 8. 完了語彙

- **意味設計完了**: 本地図§2。
- **public seam設計完了**: RCD1A後。
- **resource method設計完了**: RCFP1F／RCP1／RCBUD1後。
- **Render Contribution実装完了**: RCD1I、RCD2I、RCF1I各枝、通常製品routeの自動審判後。

この四つを同じ「M5完成」に畳み込まない。

## 9. STOP

- evidence gateを「実装の細部」と呼んで測定前にformat／method／budgetを固定する。
- P3前に現行`CompCamera`やprivate spikeをpublic Observation／seamへ昇格する。
- schema、public API、GPU method、budgetを一commit／一発注へ束ねる。
- WAITを黙示default、巨大budget、RGBA8 fallback、test-only adapterで緑にする。
