# D1l反対側レビュー証拠の価値回収（Unit 4K、2026-07-23）

状態: **観察**（cutoff 3 historical blobの処分完了）

対象: D1lのCopy Local内部ID、journal／Undo／Writer、新規Document生成口に対する独立反対側レビュー3文書のcutoff各1版。

関連: [Copy Local検収](2026-07-15-d1l-copylocal-remint-counter-review.md)、[journal／Undo検収](2026-07-15-d1l-journal-revert-boundary-counter-review.md)、[constructor検収](2026-07-16-d1l-current-document-constructor-counter-review.md)、[Shared Effect lifecycle決定](2026-07-15-shared-effect-lifecycle-decision.md)、[journal／Undo追補](2026-07-15-d1l-journal-revert-boundary-decision.md)、[新規Document生成口決定](2026-07-16-d1l-current-document-constructor-decision.md)、[M2仕様](../specs/M2-document-model.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

この3記録の価値は「最後にP0/P1=0だった」という合格票ではない。実装候補と仕様を別方向から反証し、次の欠陥をmerge前に具体的なpayload、拒否条件、試験へ変えた過程にある。

| 検収面 | 初めに残っていた欠陥 | 閉じた境界 |
|---|---|---|
| Copy Local | 保存Documentのcounterとjournal replayが両立しない。旧参照、nested ID順、予約区間の全点充足が不足 | Writerが決定済みpayloadと連続予約を作り、apply／replay／RedoはIDを選ばない。旧Definition参照、deep-copy、nested順、干渉時の原子Rejectを検査 |
| journal／Undo | v1 adapterがD1e採番と別物、identity例外が開集合、Undo全文一致の除外fieldが曖昧 | D1e共用planner、予約を持つ6 variantの閉集合、counterだけを除くUndo等価、v1/v2混在WAL、非serde Draft境界を固定 |
| 新規Document | 公開raw allocatorを削ると`new_v1()`以外の製品生成口がなく、D1l前提へ到達不能 | `new_current()`を製品の唯一の生成口にし、旧版は明示migration、legacy constructorは閉じたallowlistへ限定 |

現行コードはこの3検収が直接閉じたreservation、replay、constructor到達性を実装し、D1lの製品意味は完了済みである。ただし後続の`new_v1` lint競合追補には現行差分が残る。`new_v1()`は`#[doc(hidden)]`ではなく`#[deprecated]`のままで、22 integration testを含む25箇所に`allow(deprecated)`がある。この後発lineageは本receiptへ混ぜず、[Unit 4L](2026-07-23-historical-d1l-constructor-lint-lineage-recovery.md)で「決定済み・実装修復未到達」として処分した。

また、当時の`version == 4`を永久定数として復活させない。D1j camera migration後の現行reader／writerは5であり、現在も生きる意味は「製品新規文書が現行writerと必要minimumを満たす」「古い文書をfield直書きで偽装せず明示migrationする」「lifecycle Commandはversionを勝手に変更しない」である。

## 2. 検収で回収された設計価値

### 2.1 成功経路だけでなくreplayとinverseから契約を読む

Copy Localは通常applyだけを見ると、Definitionを複製してUseを付け替えれば成立したように見える。初回検収は保存時counterからのjournal replay、第一再検収はinverseに必要な旧参照と予約区間の穴、Undo中の新しい共有参照を反例にした。これによりCommandは`previous_definition_id`、採番済みDefinition、全点充足する`StableIdReservation`を持ち、失敗時はcounterを含むDocument全体を変えない形になった。

敵対的raw Commandが過去の削除済みIDを捏造したことまで完全検出するにはtombstoneの恒久面が要るため、そこは保証外へ縮小した。正規Writer、history、journalの非再利用を守ることと、未知の攻撃入力を無限に記憶することを同一要件にしなかった判断も残す。

### 2.2 一つの語を閉集合と機械審判へ変える

「identityを導入するCommand等」のような開いた説明は、新variantの追加時にUndo比較やcounter処理を無自覚に広げる。追補はreservationを持つv2 6 variantだけに例外を閉じ、その他は通常の全文一致にした。v1全tag fixture、v1/v2混在、Unlinkのwireに偽reservationがないこと、plannerがcrate-privateでD1eと共用されることまで審判へ落とした。

UndoはID watermarkを巻き戻さない一方、version／minimumや作品内容を変えない。このため「全文一致」を曖昧な近似へ弱めず、除外fieldを`next_stable_id`一つへ限定した。Redoは初回apply後と全文一致し、同じIDを復元する。

### 2.3 到達可能性を公開API削減と同時に検査する

raw allocatorを公開しない判断自体は正しかったが、実装候補には現行形式の新規Documentを作る製品入口がなかった。constructor検収はAPI削減後の正規到達経路を点検し、`new_current()`、明示migration、legacy constructor allowlistを一組にした。

当時はD1lがv4を導入したため、reviewはreader／writer／Effect Definition minimumを4へ揃えた。その後D1jがcameraを含むv5へ進めた現行では、`Document::new_current()`がwriter 5とcamera minimum 5で生成し、roundtripと製品利用gateが試験される。古い数値ではなく、latest生成口と明示migrationの関係を再利用する。

## 3. 現行コードとの照合

現行`motolii-doc`では次を確認した。

- `Document::new_current()`は現行`WRITER_VERSION`と現行必須minimumから生成する。
- ASTベースのallowlist試験が、非test製品sourceからの`new_v1()`呼出しを文字列走査の誤検出なしで拒否する。
- `prepare_copy_local_effect`は元Definitionをdeep-copyし、Definitionとnested Keyframeを固定順で再採番するが既存Use IDを予約へ含めない。
- reservation検証は空、逆、穴、過大、順序違い、衝突、中間counterを型付きで拒否する。初回／replayだけcounterを前進させ、RedoとUndoで巻き戻さない。
- Copy Local applyは旧参照とpayload完全性を検査し、Undo時に新Definitionが別Useから参照されていれば部分成功せず拒否する。
- legacy Effect migration plannerはcrate-privateでD1e migrationとjournal v1 adapterから共用される。

型名`EffectLifecycleRequiresV4Document`等に歴史的なv4呼称が残っていても、現行guardはwriter 5とcamera minimum 5を要求する。名称だけからv4入力が現在も受理されると推論しない。

一方、後続の[lint競合追補](2026-07-16-d1l-new-v1-lint-conflict-decision.md)が棄却した`#[deprecated]`とtest側のlint抑制は現行sourceに残っている。AST gateが緑でも、この実装方法のdriftを閉じたことにはならない。本Unitはそれら後続2版を処分済みに数えず、[Unit 4L](2026-07-23-historical-d1l-constructor-lint-lineage-recovery.md)へ送った。

## 4. 独立検収の証拠規律

- timeoutや無出力は合格にも不合格にも数えない。
- 複数reviewerの判定が割れたら、安全側の具体的反例をコード／path／仕様で再現して採否する。
- reviewerが挙げたpathや前提が実repositoryに存在しなければ、権威名ではなく事実照合で棄却する。
- P0/P1=0は固定commitと対象diffの証拠であり、後続versionや別featureへ無期限に外挿しない。
- 最終合格票だけを保存せず、採用した反例、縮小した保証、棄却理由、再レビュー条件を一緒に残す。
- test greenは必要条件だが、正規製品経路、replay、Undo／Redo、失敗時不変の反証に代えない。

## 5. 復活させないもの

- apply、journal replay、Redoがその場で新しいIDを選ぶこと。
- reservationを持つCommandの集合を「等」で開いたままにすること。
- Undo時にcounter、version、minimumをまとめて比較対象外にすること。
- legacy v1 Removeを現行orphan保持へ読み替え、旧inline destroy意味を変えること。
- plannerをD1eとjournal adapterへ二重実装したり、公開APIへ露出したりすること。
- 製品新規作成を`new_v1()`、`Default`、raw allocatorへ戻すこと。
- `new_v1()`の製品利用禁止をdeprecation警告とtest側の広いlint抑制で代用すること。
- version fieldだけを書き換えたinline／hybrid文書をmigration済みと扱うこと。
- 当時のliteral v4を現行writerへ戻すこと。
- timeout、存在しないpath、reviewer名、CI greenだけを独立検収の根拠にすること。

## 6. 固定歴史出典とcoverage

3文書の各1版を全文で読み、導入commitと現行コード／仕様を照合した。処分した3 unique blob（8,452 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/04k-d1l-counter-reviews.tsv`を正本とする。cutoff総数1,797のうち処分済みは325、未処分は1,472である。
