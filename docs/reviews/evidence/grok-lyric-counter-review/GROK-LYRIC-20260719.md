# GROK-LYRIC-20260719 — Grok反対側レビュー原文

## 実施情報

- 実施日: 2026-07-19(JST)
- 実行model指定: `cursor-grok-4.5-high`
- 実行面: Cursor Agent CLI、`ask` mode、read-only指示
- 対象: [リリックモーション比較台帳](../../2026-07-19-lyric-motion-text-sequence-comparison.md)
- repository変更: なし(応答後に主担当が`git status --short`で確認)
- 会話固有ID・アカウント情報: 応答原文には含まれていない

## 入力文書

Grokには次を必読対象として指定した。

1. `AGENTS.md`
2. `docs/README.md`
3. `docs/decision-index.md`
4. `docs/reviews/README.md`
5. `docs/reviews/2026-07-19-lyric-motion-text-sequence-comparison.md`
6. `docs/text-model.md`
7. `docs/extensible-core-model.md`(特に§7〜§10)
8. `docs/ui-score-model.md`
9. `docs/reviews/2026-07-14-motion-foundation-known-tech-disposition.md`
10. `docs/reviews/2026-07-14-repeated-wheel-standardization-audit.md`のText / Materialize関連
11. `docs/reviews/2026-07-14-motion-tools-praise-diy-gap-audit.md`のG4 / G6 / G8
12. `docs/reviews/2026-07-12-m2-permanence-prevention.md`
13. `docs/reviews/2026-07-14-m3-ui-boundary-prevention.md`

## Prompt要旨

実装案を作らず、独立したread-only反対側reviewerとして次を審判するよう依頼した。

- 4製品命題の整合性と採択先取りの有無
- 親規則/子overrideの評価所有、空間差分とtiming差分の非対称、複数Animator stack
- 本文編集、反復文字、書記素、shaped cluster、Font/style、合字、ルビ、fallback後のidentity和解
- CONTROL/RESULT投影とinverse dragが第二のTimeline正本にならないか
- Live Text内Position animationの恒久面、Undo/journal、cache、Preview/Export、性能
- Optional Detach/部分Detachの所有、advance、外部Effect/接続/生存期間
- 既決との衝突範囲、fixtureの反証可能性、TextからClonerへ一般化する停止線
- AE市場証拠が支持する範囲

findingをP0/P1/P2で分類し、`ACCEPT FOR FIXTURE / REVISE MEMO BEFORE FIXTURE / STOP`の判定、次段前の最小修正、fixtureでしか答えられない問い、repository無変更確認を求めた。

## 応答原文

以下はCursor Agent CLIが返した全文である。重要度ラベルと文言は外部reviewerの原文を保持し、主担当の補正を混ぜていない。

---

**VERDICT: REVISE MEMO BEFORE FIXTURE**

比較メモは「決定ではない」と正しく宣言しているが、§0.1の4不変条件が実質Live Split採択条件になっており、identity・timing所有・部分Detachが未定義のままfixtureへ進むと審判不能になる。DO NOT ADOPT（現状メモのままfixture正本化・一般化・スキーマ焼き込みは不可）。

---

## Findings

### P0 — DO NOT ADOPT

**P0-1. 4不変条件は比較軸ではなくLive Split採択条件**
- 根拠: `docs/reviews/2026-07-19-lyric-motion-text-sequence-comparison.md` §0.1 / §3.1
- 違反: レビュー規律1（調査結論を設計根拠にしない）＋比較中文書としての中立性
- §3.1はA/B/C比較を謳うが、§0.1はLIVE TEXT＋DIRECT CHARACTERSを同時必須にするため、**B（明示Split＋本文凍結）は定義上「同等案ではない」**。これは比較不変条件ではなく、C/Aの製品命題の先取り。
- 共存不能ペア（現状定義のまま）:
  - **LIVE TEXT × DIRECT CHARACTERS**: identity和解（§3.2）未決のままでは、触っている対象が「どの文字か」をDocument意味で閉じられない。extensible-core §7.1 / P0I「編集で再生成されるindexを永続参照にしない」と衝突し得る。
  - **LIVE TEXT × OPTIONAL DETACH（部分）**: slot予約はTextが配置穴を所有し続け、独立生存期間と両立しない（メモ自身が§3.3で認める）。

**P0-2. identity和解なしに疎な介入をLive Textへ載せる契約が無い**
- 根拠: 同メモ §3.2 / §3.3; `docs/extensible-core-model.md` §7.1; `docs/reviews/2026-07-14-motion-foundation-known-tech-disposition.md` §5; `docs/text-model.md` §2（selectorはcluster index）; `docs/concept.md`「文書とキャッシュを分離」
- 違反: extensible-core §7.1「別個体へ黙って付け替えない」＋恒久予防「意味が先」（`docs/reviews/2026-07-12-m2-permanence-prevention.md` 手順1）
- 反復文字列、合字on/off、font fallback、ルビ、shaped-cluster再形成、range style変更は、いずれも**クラスタ位相を変え得る**。4候補は列挙のみで、どれが「誤接続ゼロ」を機械判定するか無い。安定cluster IDをDocumentへ入れる案は、導出物の永続化としてconceptの文書/キャッシュ分離と正面衝突し得る（メモが触れているが未審判のままLIVE TEXT要件に組み込まれている）。

**P0-3. 親規則＋子overrideの評価所有が未定義のままUI仮説が先行**
- 根拠: 同メモ §2.8（自身が非対称とstack未決を認める）; `docs/text-model.md` §2.2（複数animatorは逐次＋加算）
- 違反: 評価意味の閉集合不足（GR-PV手順1）／Ghost Pose「選択Animatorへ書く」vs「最終合成を描く」の逆変換一意性
- 事実として分離すべき2種:
  - **出力側**: Position / Visual Scale / Rotation → 生成poseへの加算
  - **入力側**: Start Offset / Duration Scale → selector時間の書換え
- 後者が「全stack共通か、対象Sequence 1枚か」未決のままfixture 14を回しても、合否が解釈依存になる。多枚stackでghost最終合成から単一animator差分への逆変換は、加算の下で**一般に一意でない**（G1/G8の「逆変換できる場合のみ」が停止条件になるが、不能時の正本が無い）。

### P1 — DO NOT ADOPT（次段に進める前にメモ改訂必須）

**P1-1. §2.1「新機構を発明しない」は事実過大**
- 根拠: §2.1 vs §2.7 / §3.3; `docs/text-model.md` §1–2
- 分類: **事実矛盾**（未決ではなく、既存text-modelへの帰属誤り）
- selector+propertiesのタイポライター一般化は既存骨格に収まる。しかしaddressable clusterへの疎な手動介入・RESULT逆編集・Live Split保存候補は、text-model二層に**無い新しい所有意味**。評価器ParamSource契約不変と「Document意味を増やさない」は別物。

**P1-2. CONTROL/RESULTの逆dragは、override schema未採用のまま第二Timeline所有に転落しやすい**
- 根拠: §2.4 / §2.6; R1 `docs/reviews/2026-07-14-repeated-wheel-standardization-audit.md` §4.3（Timing RailはTransient→通常時刻へ焼く）; GR-UI-1 `docs/reviews/2026-07-14-m3-ui-boundary-prevention.md`
- 違反: 「Laneを所有者にしない」（`docs/ui-score-model.md` §1–2）の精神を、RESULT nodeが**編集可能な時間正本**に見える形で侵食し得る
- 投影としては妥当。だが「採用済み明示overrideへ逆変換できる時だけ」は、§3.2が未採用のままでは**書き込み先が無い**。不能時に絶対keyへ焼く／UI-only stateを残す／操作拒否する、の閉集合が無い。

**P1-3. 文字別Position animation（§3.3案2）は恒久面・性能・cacheの境界を跨ぐ**
- 根拠: §3.3; text-model §1（animatorはリフローしない＝空間Δとしては経路上あり得る）; GR-PV手順2; ui-score回帰審判14の規模則
- 分類: **未決の設計問い**だが、fixture 12を「体験比較」だけで進めると恒久焼き込み圧力になる
- preview/export同一関数は保てる（純関数なら）。問題は (a) cluster×channelのjournal密度、(b) shape無効化境界、(c) 本文編集後のchannel再結線、(d) 「見た目の直接編集」需要がMaterialize限定を迂回する経路になる点。案1（静的のみ、時間可変はDetach）と案2は**同じfixtureでは審判できない**ほど恒久コストが違う。

**P1-4. OPTIONAL DETACHの部分Detach境界が不変条件として破綻**
- 根拠: §3.3; G4 `docs/reviews/2026-07-14-motion-tools-praise-diy-gap-audit.md` G4
- slot予約＝完全独立ではない。削除再行組＝LIVE TEXTの後続配置を壊す。全体DetachだけがG4の「確定後の独立」に近い。部分DetachをOPTIONAL DETACHの充足例に数えるなら、**advance所有・二重描画・欠落参照・Undo・後続本文編集**の意味論表が先。

**P1-5. fixtureに反証可能な合否閾値が無い**
- 根拠: §4
- 「生存率」「誤接続せず」「追従」は観測語であり、合格線（例: 誤接続0件、指定cluster集合のbit一致、Undo 1でrevision一致、lane数≤選択数、Document項目Δ=0 等）が無い。緑のUI感だけでは不十分（レビュー規律＋GR-PV手順5）。

### P2

**P2-1.** ui-score Particle「個体編集はMaterialize後」のスコープ限定は**正しい**（§2.7）。全domain一般化と誤読しない記述は維持すべき。衝突はCloner/高数量昇格時のみ。

**P2-2.** AE市場証拠の読み分け（需要＝製品成立、Host制約＝推論、方式決定に使わない）は**規律適合**。ただし「G4優先度再審理に値する」は需要再優先の仮説であり、Live Split正当性の証拠ではない（§1は概ね自制できている）。

**P2-3.** Text→Clonerの順序宣言は健全。しかし§2.7の一般図は既にHost外殻候補を先取りして見える。共有capability許可前の必須証拠をメモ正面に「停止線」として再掲すべき。

**P2-4.** Font Size vs Visual Scaleの分離はtext-model §4未決への妥当な回答**候補**。採用ではない、の注記は十分。

---

## 質問別判定（要約）

| # | 判定 |
|---|---|
| 1 四不変条件 | 比較基準としては不適。採択条件の先取り。LIVE TEXT×DIRECT、LIVE TEXT×部分DETACHが未解決共存 |
| 2 親/子 | 評価・所有モデル未定義。出力加算と入力timingの非対称＋stack scopeが中核未決 |
| 3 identity | 現状reconcile契約なし。誤接続防止はスローガンのみ |
| 4 CONTROL/RESULT | 投影原理は可。逆dragの書き込み先と不能時閉集合が無いと第二Timeline化 |
| 5 文字Position | 静的Δはanimator層に寄せ得るが、時間可変channelは恒久・密度・和解を跨ぐ |
| 6 Detach | 全体DetachはG4整合候補。部分は所有が分裂 |
| 7 既決衝突 | Particle文の過一般化はメモが回避。text-model「発明しない」は過大。P0I/文書キャッシュ/規模則は未閉 |
| 8 fixture | 網羅は良いが閾値不足。UIモック不能な問いが本体 |
| 9 Cloner一般化 | 順序は可。共有schema/traitは両domainのidentity＋密度＋逆編集証拠まで禁止のまま |
| 10 市場証拠 | 需要のみ支持。Host制約は推論。それ以上は不可 |

---

## 事実矛盾 vs 未決の設計問い

**事実矛盾**
- §2.1「新機構を発明しない／text-modelがそのまま」↔ §2.7–3.3の疎介入・Live Split所有
- §0.1「同等案条件」↔ §3.1のBを比較対象に含む体裁

**未決（埋めずに止めるべき）**
- identity生成・継承・欠落・Needs Reviewの所有
- timing差分のstack scopeとGhost逆変換の一意性
- 部分Detachのadvance所有
- overrideの時間可変性（静的のみ vs per-cluster ParamSource）
- 安定IDのDocument焼き込み可否（concept文書/キャッシュ）

---

## Minimal corrections（次段前）

1. §0.1を「Live Split採択条件」と「A/B/C比較軸」に分割。Bを落とすなら比較表からBを外し、落とさないならLIVE TEXT必須を外す。
2. §2.1の「新機構なし」を削除または「selector型Sequenceに限る／疎介入はtext-model外の新意味」と明示。
3. identity和解を**最初のfixture gate**にし、4不変条件より前に置く。誤接続0の負例を先に定義。
4. timing差分のstack scopeとGhost逆変換不能時の閉集合を、UI仮説から切り離して未決STOPとして前面化。
5. 部分DetachをOPTIONAL DETACHの充足から外すか、所有意味論表を先に書く。
6. §4に定量合否（誤接続件数、Undo、lane≤選択、Document項目非比例、preview/export同一）を足す。
7. 共有Host capability / 公開trait / 共通Element schema / Document fieldは、Text＋Cloner双方のgate通過まで停止線のまま再掲。

---

## Fixturesが答えるべき問い（チャットでは閉じない）

1. 挿入・削除・反復・合字・fallback・ルビ後、overrideは誤接続0か、Needs Reviewか、全破棄か——どれが機械判定で再現するか
2. 2枚animator下でStart Offsetはどの規則の入力か。Ghost最終合成から単一animatorへの逆変換成功率と不能時挙動
3. RESULT dragの確定先は親パラメータか疎介入か。不能時に絶対key化しないことのjournal証跡
4. 静的local Δのみ vs 文字別Position keyで、本文編集後保持・Timeline密度・1 Undo・cache invalidationの差
5. 部分Detach三案の二重描画・後続行組・font変更・欠落参照
6. 明示展開N文字／N instanceで、lane・marker・Document項目が個体数に比例しないこと（回帰14）
7. previewとexportの同一評価経路で文字別空間Δが一致するか（見た目fixture）

UIモックだけでは閉じない: identity、journal/Undo、cache key、preview/export同一性、高数量密度、stack評価意味。

---

**リポジトリへの変更は一切行っていない**（read-only。ファイルの作成・編集・削除・stage・commit・外部状態変更なし）。
