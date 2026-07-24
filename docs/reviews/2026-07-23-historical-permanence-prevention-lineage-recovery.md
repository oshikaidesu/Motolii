# GR-PV恒久焼き込み予防lineageの価値回収（Unit 4B、2026-07-23）

状態: **処分完了**（cutoff 9 historical blob）

対象: `docs/reviews/2026-07-12-m2-permanence-prevention.md`のcutoff全版。

関連: [恒久焼き込み予防](2026-07-12-m2-permanence-prevention.md)、[失敗と復旧の先例](2026-07-12-rework-prior-art.md)、[M2仕様](../specs/M2-document-model.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

9版を通じて変わらなかった正本は、恒久形式を作った後のmigration技法ではなく、誤った意味をそもそも焼かない予防5手である。

1. **意味が先**: 型、serde、goldenを足す前に、対象の意味論表／宣言を仕様へ置く。
2. **恒久面を狭く**: UI都合、未決、画素改善、将来候補をユーザーデータへ混ぜない。
3. **追加的に変える**: 新field+default、新variant、明示migrationを使い、既存fieldの意味を静かに変えない。
4. **依存を直列化**: 共通limit、open mode、validation、migration等の正本が成立する前に後続実装を並走させない。
5. **意味で完了判定**: cargo test緑だけでなく、typed rejectionまたは意味論goldenを審判にする。期待値更新で通さない。

Olive型の全書直し、OpenCut型の後付け境界、Natron型の後付け所有、Legacy/migrationは、予防が破れた時の出口であって第一選択ではない。

## 2. 九版の処分

| blob | 変更 | 現在の判定 |
|---|---|---|
| `739574c3` | 予防5手の初版 | **現行運用正本の核** |
| `779d84f6` | PathOp意味論を文書済みと早期判定 | **後続で撤回された進捗誤認** |
| `4b8d252b` | AE/Lottie案だけでは未確定としてCavalry比較後へ戻す | **正しい停止記録** |
| `916fde82` | command粒度を#103決定待ちと明示 | 歴史的進捗 |
| `3c4e0813` | #103採択を反映 | 採択済み進捗 |
| `23517e73` | PathOpを決定パック採択後の決定済みへ更新 | **現行状態へ採用** |
| `fe931dd4` | D1c-FU→D1d/D1e、D4→D6等の依存を厳密化 | **依存規律へ採用** |
| `7f1f24d6` | stale branch由来でPathOpを未確定へ戻し、D1c-FU依存を落とす一方、Camera/Shared Effectを追加 | **部分採用**。新規Camera/Shared Effect節だけ保持し、進捗・依存回帰は棄却 |
| `b621d163` | mergeでPathOp決定と依存厳密化を復元 | **現行版** |

このlineageは、後から現れたblobが常に意味上の上位版とは限らないことを示す。branch mergeで古い進捗へ戻った差分は、明示撤回文書が無ければ新しい設計決定として採用しない。

## 3. 個別主題から一般規律を分ける

PathOp、command粒度、CompCamera、Shared Effect、D1c-FU等は予防5手を具体的に適用した当時の例である。各対象の現在意味は個別spec／decisionを正本とする。

- PathOpは比較案を仕様と誤認した後、Cavalry比較へ戻し、決定パックで採択してからD1i-2へ進んだ。
- command粒度は未決中にdefaultを焼かず、#103⑨採択後にだけ反映した。
- CompCameraは意味決定、schema+default migration、runtime解凍、graph接続を直列化した。
- Shared EffectはDefinition/Use、lifecycle、inline migration、unknown保持、pixel/order不変を閉じ、Composite Set等を便乗させなかった。
- `ResourceLimits`／`OpenMode`はD1c-FUの正本を待ち、D1d/D1eの重複定義を防いだ。

初回の各依存は現在完了済みだが、順序を守った事実を「今後は順序不要」と解釈しない。現行未実装D1nはD1m後という個別依存を維持する。

## 4. 現行文書の時点補正

予防手順の前提に残っていた「いまの出戻りはまだ最小」を、2026-07-12の成立時点として明記した。§4の硬い線も初回M2で充足済みの例とし、現在のtask表を読むよう補正した。

またShared Effectの一行がDefinition/Use決定だけを指し、lifecycle決定を逆引けなかったため、両文書へ分けて接続した。意味変更ではなく、既存の決定正本を正しく指す修正である。

## 5. 復活させない短絡

- 比較調査やprototype表を「文書がある」だけで意味決定済みとしない。
- UIに必要そう、便利そうという理由でschema field、enum、defaultを足さない。
- pixel改善を同じvariant／IDの意味変更として入れない。
- `serde(default)`を未決意味の決定代わりにしない。
- migrationがあることを、誤ったschemaを先に焼く許可にしない。
- 並列化のためにtask dependencyを実装者判断で落とさない。
- test緑を意味一致の代わりにせず、golden更新／負例削除で完了させない。
- stale branchの進捗回帰を明示撤回なしに現行判断へ上書きしない。

## 6. 固定歴史出典とcoverage

初版`739574c3`を全文で読み、PathOpの早期完了→未確定復帰→決定採択、command粒度、I/O依存、Camera/Shared Effect追加、merge復元まで8遷移を確認した。最終2 commitが同一blobを持つためblob単位で1件と数えた。

処分した9 blobの完全SHAは`evidence/historical-value-recovery/disposition-receipts/04b-permanence-prevention.tsv`を正本とする。cutoff総数1,797のうち処分済みは233、未処分は1,564である。
