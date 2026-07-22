# Shared Effect lifecycle lineageの価値回収（Unit 4J、2026-07-23）

状態: **観察**（cutoff 3 historical blobの処分完了）

対象: `docs/reviews/2026-07-15-shared-effect-lifecycle-decision.md`のcutoff全3版。

関連: [Shared Effect lifecycle決定](2026-07-15-shared-effect-lifecycle-decision.md)、[Definition / Use決定](2026-07-15-relative-scope-duplicator-decision.md)、[journal / Undo追補](2026-07-15-d1l-journal-revert-boundary-decision.md)、[M2仕様](../specs/M2-document-model.md)、[M3仕様](../specs/M3-ui-integration.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

このlineageが固定したのはpluginの供給元ではなく、共有recipeのDocument所有である。Effect DefinitionとUseを分け、複数Useが一つのDefinitionを参照するとき、削除、切断、ローカル化、未参照状態を暗黙操作へしない。

| 操作 | 採択された結果 |
|---|---|
| 参照中Definition削除 | `DefinitionInUse`でReject、Document不変 |
| Unlink / Remove Use | 対象Useだけをstackから除去、Definitionは保持 |
| Copy Local / Make Unique | Definitionをdeep-copyし内部IDを再採番、対象Useだけ新Definitionへ付替え |
| 最後のUseを除去 | Definitionをorphanとして保持 |
| 未参照Definition削除 | 明示`DeleteDefinition`だけが削除する |
| unknown plugin Definition | 既知pluginと同じlifecycle、`extra`を保持 |

D1lのschema／command／migrationとD3eの評価は現在実装・審判済みである。一方、Timeline Effect LinkのU2gは`WAIT`であり、Document lifecycle完成を実UI完成へ数えない。first-party、third-party、unknownの違いも、この所有規則を分岐させない。

## 2. 3版で起きた意味追加

### 2.1 初版: lifecycleを暗黙破壊から分離

初版はDefinition／Useの別identityを前提に、参照中削除をcascadeしない、UnlinkをCopy Localにしない、最後のUse削除やsaveをGCにしないと決めた。共有状態を解除する操作とrecipeを捨てる操作を別gestureへ分けたことが中心価値である。

### 2.2 第二版: Copy Localを内部IDまで本当の複製にする

Definition本文だけを複製して配下Keyframe IDを共有すると、後の編集・Undo・journalでidentityが衝突する。この版はnested `Vec2Axes`を含む全Keyframe IDを固定順で再採番し、Definition→params辞書順→Keyframes格納順→x/yの順を凍結した。

さらにWriter準備時にcounterのcloneから連続予約区間を作り、Command payloadへ旧参照、完全な新Definition、`[before, after)`を持たせた。applyはIDを選ばず、導入ID集合が区間の全点と一致することを検査する。初回／journal replayとRedoでcounter規則を分け、失敗時はcounterを含むDocument全体を不変にする。

この追補はreservationやtombstoneをDocument schemaへ追加しなかった。削除済み任意IDを`from_raw`で偽造する敵対的Commandの完全識別までを保証せず、正規Writer／Command／Undo／journal経路を閉じた。

### 2.3 第三版: Undo watermark例外と後続境界へ接続

最終版はUndo後も`next_stable_id`を巻き戻さないため、Undo等価を「counterを除く全文一致」と訂正した。Redoは初回apply後と全文一致し、同一のDefinition／Keyframe IDを復元する。journal v1→v2 adapterとWriter内部採番の追補が閉じた後にD1lを再開する、と依存も更新した。

## 3. 現行コードとの照合

現行コードは、Definition参照整合、orphan roundtrip、参照中削除拒否、Use単独unlink、Copy Localのdeep-copy／再採番、予約区間閉集合、payload完全性、Undo干渉拒否を実装している。再採番walkerは`duplicate.rs`で共用され、prepare成功前にDocumentを変更しない。

試験は次を含む。

- 参照中Definition削除のtyped rejectとDocument不変。
- shared Useの一つだけをunlink／Copy Localし、他Useを変えない。
- unknown pluginの`extra`、orphan、新旧二Definitionのsave/reload保持。
- nested Keyframe IDの固定順再採番と元集合との非交差。
- 空、逆、穴、過大、予約外、衝突区間とstale payloadの無変更Reject。
- Undo中に新Definitionが別Useから参照された場合の部分成功拒否。
- D3eで非隣接Use、Group合成後一回、rename／reorder、preview/export同一をpixel審判。

したがってGAP-14は完了済みである。ただしCascade delete、Purge unused、一斉Make Uniqueは「実装漏れ」ではなく延期項目であり、明示仕様改訂なしに通常操作へ足さない。

## 4. 再利用する設計原則

- Definitionの共有identityと、各stack位置のUse identityを分ける。
- Unlink、Make Unique、Delete recipeを一つの便利操作へ畳まない。
- 最後の参照が消えても自動GCせず、破壊は明示commandにする。
- Copy Localは未知fieldとnested identityまでdeep-copyし、対象Use以外を変えない。
- ID選択はWriter prepareへ閉じ、apply／journal replay／Redoを決定済みpayloadで再現する。
- 予約区間は導入ID集合との完全一致を審判し、失敗時のcounterを含む原子性を守る。
- Undoの意味等価と単調watermarkの非巻戻しを区別する。
- unknown／third-party pluginだからlifecycleを弱めたり特例削除したりしない。
- Document操作とTimelineの接続可視化を別完了条件にする。

## 5. 復活させないもの

- Definition削除が全Useを暗黙cascadeすること。
- Unlinkが自動Copy Localしたり、最後のUseとDefinitionを同時削除したりすること。
- save/load時またはbackground GCでorphanを消すこと。
- Copy Localがparamsだけを複製し、plugin/version/enabled/extraやnested IDを落とすこと。
- 一つのUseのMake Uniqueが全Useを一斉unique化すること。
- apply／Redoがその場でIDを選び直すこと。
- reservation／tombstoneを新しいDocument恒久fieldとして追加すること。
- counter巻戻しをUndo全文一致の条件に戻すこと。
- unknown pluginへ強制materializeや強い削除拒否の別lifecycleを設けること。
- D1l/D3e完了をU2g、Composite Set、Backdrop、K2 invalidationの完了へ広げること。

## 6. 固定歴史出典とcoverage

初版`51f419bd`を全文で読み、差分`51f419bd..d3b1e412`、`d3b1e412..a6a015c1`を確認した。処分した3 unique blob（42,457 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/04j-shared-effect-lifecycle.tsv`を正本とする。cutoff総数1,797のうち処分済みは322、未処分は1,475である。
