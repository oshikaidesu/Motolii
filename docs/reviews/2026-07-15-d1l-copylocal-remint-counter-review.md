# D1l Copy Local内部ID契約 — 反対側レビューと採否

日付: 2026-07-15  
対象: PR #196  
目的: D1l実装へ恒久化する前に、Keyframe ID再採番、単調counter、journal replay、Undo/Redoの境界を反証する。

歴史注記（2026-07-23）: 本記録のcutoff 1版は、journal再生不能、予約区間の穴、旧参照不足を段階的に発見した独立検収証拠として[Unit 4K回収](2026-07-23-historical-d1l-counter-review-evidence-recovery.md)で処分した。最終P0/P1=0だけを結論として切り出さず、初回・再レビューで採用した反例と一緒に読む。

## 初回判定

Cursor Grok 4.5 Fastの読み取り専用レビューは **P0=1 / P1=4 / P2=3**。Composer 2.5は240秒でタイムアウトしたため、合格票として数えない。

### P0

- `id >= next_stable_id`拒否とapply非採番の組合せでは、counterが進んでいない保存DocumentへCommandだけをjournal replayすると拒否される。

### P1

1. 現在衝突とcounter上限だけでは、削除済みIDを捏造した敵対的Commandをtombstoneなしに完全検出できない。
2. nested `Vec2Axes`を含む再採番走査順が未固定。
3. `new_definition`が本当に参照元のdeep-copyかをapplyで検証するか未決。
4. 未発行ID拒否試験がnested Keyframe IDを明示していない。

追加STOPとして、Undo時に新Definitionへ別Useが付いた場合のReject/残存が未決だった。

## 採否

| 指摘 | 採否 | 反映 |
|---|---|---|
| journal/counter矛盾 | 採用 | Commandへ`stable_id_reservation=[before,after)`を持たせる。準備はDocument不変、初回apply/replayがcounterをcommit、Redoは同じIDを復元 |
| 削除済みIDの敵対的捏造 | 一部採用 | 正規Writer/history/journalで非再利用を保証。完全検出に必要なDocument tombstoneは恒久面を広げるため棄却し、敵対的Commandは保証外と明記 |
| 走査順 | 採用 | Definition ID→params辞書順→Keyframes格納順、Vec2Axes x→y。既存duplicate再採番関数を共用 |
| payload完全性 | 採用 | apply時にID以外が参照元Definitionのdeep-copyであることを検査 |
| nested拒否試験 | 採用 | Use/Definition/nested Keyframeの予約区間・衝突を完了条件へ追加 |
| Undo参照干渉 | 採用 | 他Use参照または対象Use付替え済みなら全体Reject、Document不変 |
| 変更履歴/Vec2Axes/API名 | 採用 | §8、試験名、Writer準備API責務を追記。最終Rust名は実装範囲 |

## 再レビュー条件

本修正後に別の読み取り専用レビューを実行し、P0/P1=0、PR CI緑を確認するまでmergeしない。実装PR #173は本PR merge後にこの契約へ追随させる。

## 第1再レビュー

Composer 2.5はP0/P1=0と判定したが、Grok 4.5 Fastは **P0=1 / P1=2** を検出した。安全側として後者を採用した。

| 指摘 | 採否 | 反映 |
|---|---|---|
| inverseに旧Definition IDが不足 | 採用 | 必須payloadへ`previous_definition_id`を追加し、applyでUseの旧参照を検査 |
| 予約区間が穴あき/過大でも通る | 採用 | 導入ID集合と半開区間の全点充足を必須化。空区間もReject |
| Copy Localの既存Useを予約対象と読める | 採用 | Copy Localは新Definition+Keyframeのみ、Add/Linkは新Useを含むと分離 |

この修正後もP0/P1=0の再確認とCI緑まではmergeしない。

## 最終再レビュー

commit `f4e6713`をComposer 2.5とGrok 4.5 Fastが別々に読み取り専用レビューし、両者とも **P0=0 / P1=0** と判定した。前回のpayload不足、旧参照検査、予約区間全点充足、Copy Local既存Useの対象分離が閉じたことを確認した。

残件はRust variant名、既存`duplicate.rs`から再採番走査だけを抽出する実装上の注意、敵対的`from_raw`を保証外とした境界で、いずれも仕様mergeを妨げるP0/P1ではない。PR CI緑を別途確認してmergeする。
