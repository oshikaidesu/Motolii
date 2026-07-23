# D1l journal／Undo／Writer lineageの価値回収（Unit 4M、2026-07-23）

状態: **停止線**（cutoff 2 historical blobの処分完了、Effect契約は実装済み、Position Add Keyは未実装、fallback driftはGAP-24）

対象: `docs/reviews/2026-07-15-d1l-journal-revert-boundary-decision.md`のcutoff全2版。二つの並行commit対が同じ初版と同じPosition追補版を持つため、処分対象は2 unique blobである。

関連: [journal／Undo追補](2026-07-15-d1l-journal-revert-boundary-decision.md)、[反対側レビュー](2026-07-15-d1l-journal-revert-boundary-counter-review.md)、[D1l検収証拠回収](2026-07-23-historical-d1l-counter-review-evidence-recovery.md)、[D2／Position Add Key回収](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md)、[M2仕様](../specs/M2-document-model.md)、[backlog](../backlog.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

初版が閉じたのは、stable IDを導入する編集だけがUndo後もcounterを巻き戻さない例外、旧JournalEdit v1を失わずv2へ接続するadapter、caller採番を排したWriter prepareの三境界である。現行Effect lifecycleでは次が実装済みである。

- `StableIdReservation`を持つ閉集合はCreate／Link／Copy Localと各inverseの6 variant。
- Undoは`next_stable_id`一つだけを正規化して残り全文一致、実counterは非巻戻し、Redoは初回apply後と全文一致。
- 新規journalはEdit envelope v2、同じWALでv1/v2混在を明示decodeする。v1はD1e共用のcrate-private plannerで完全payloadとwatermarkを先に作る。
- Writer prepareはCreate／Link／Copy Localの3本で、counter clone上にpayloadを作りlive Document、revision、Undo／Redoを変えない。
- Effect applyは全検査後にclone／validate／swapし、部分成功を許さない。

後発版が加えたPosition Add Keyは棄却ではないが、現行コードには`AddPositionKey`、`UndoAddPositionKey`、`prepare_add_position_key`がない。現行正本は専用forward／inverse、old/new `DocParam`、追加Keyframe ID、1-ID reservationを採択済み・未実装とし、exact Rust型、solver tolerance、journal版判断を実装前closed contractへ残す。後発blobの「8 variant／prepare 4本」を現在の完成状態として復活させない。

また、初版が「fallbackは破損時の出口」と限定した一方、live `replay_from_base(..., fallback_on_failure=true)`はdecode失敗だけでなくapply失敗でも直前Snapshotへ戻る。製品recoverはこのflagをtrueで呼ぶ。既存のknown-v1負例には先行Snapshot frameがないため、この分岐を反証していない。既知Editの意味不整合まで編集喪失を伴うfallbackへ送るかは決定とコードが不一致であり、GAP-24として狭く再開する。

## 2. 初版の三つの接続契約

### 2.1 Undo等価と単調identity

identityを導入しないCommandはapply→inverseでDocument全文一致を維持する。Effectの6 variantだけは、Undoが新IDを作品から消しても再利用防止watermarkを巻き戻さない。除外fieldをcounter一つへ限定し、version／minimum、作品内容、extraを比較対象に残した。

この区別により「Undoで見た目が戻る」と「過去IDを再発行する」を混同しない。reservationを持つvariantの集合はmethodとtag集合testで閉じ、新variantを散文の「等」で自動加入させない。

### 2.2 旧wireを変えずlosslessに読む

v1 wireへ予約やDefinition IDを`serde(default)`で捏造せず、envelope versionを先に分岐する。plannerはDocumentとpayloadの全IDから開始watermarkを決め、固定順でDefinition IDだけを追加する。既存Use／Keyframe ID、plugin、params、extraは保持し、Removeは旧inline destroy意味を保つ。D1eとjournalが同じplannerを使い、replay中にIDを選ばない。

旧base／generationはメモリ上でmigrationしてからv1 Editを適用し、原本とWALを上書きしない。未知版はtyped reject、既知v1を「decodeできないのでsnapshotへ戻す」互換戦略へしない。

### 2.3 Writerだけが新identity payloadを準備する

製品callerが`peek_next`／`from_raw`でEffect IDを組み立てる経路を棄却し、identityを持たないruntime-only DraftからWriterが完全Commandを作る。Draftをserde、Document schema、plugin契約へ焼かず、prepare成功前後もDocumentを変えない。

## 3. Position Add Key追補の現行処分

後発blobはEffect 6 variantを現在の合格条件として明示した上で、U4b-0実装後だけPositionのforward／inverseを加えて8 variant、prepare 4本へ拡張する案を記録した。対象link先の旧個別契約pathは現行treeにないため、その文面を直接の実装正本にしない。

現在は[M2 U4b-0](../specs/M2-document-model.md#操作単純化モデルへの割当)と[Unit 2C回収 §4](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#4-u4b-0-durable-add-position-key再採択)が権威である。explicit Add Position Keyだけがfresh IDを導入し、帯域外の公開`allocate_keyframe_id`＋`SetProperty`を製品経路にしない。現行に公開allocator自体が存在することと、その組合せを製品正規路にしないことを区別する。

## 4. 現行コード・試験との照合

| 面 | 現在地 |
|---|---|
| reservation閉集合 | Effect 6 variantを`stable_id_reservation()`とtag testで固定 |
| Undo／Redo | counter一項だけ正規化する共通helper、同一ID Redo、失敗時全文不変を審判 |
| Writer prepare | Create／Link／Copy Local 3本。固定採番順、全点予約、prepare無変更を審判 |
| v1/v2 journal | version先行decode、mixed WAL、v1全tag corpus、D1e planner parityを審判 |
| planner公開面 | `LegacyEffectMigrationPlanner`とprepared型はcrate-private |
| Position Add Key | Command／prepareとも未実装。正本は決定済み・未実装 |
| current version | 歴史v4からcamera後のv5へ進んだ。`EffectLifecycleRequiresV4Document`という名称でもguardはcurrent v5を要求 |
| fallback | decode errorとapply errorの双方が、先行Snapshot＋flag trueならfallback対象。既知apply failure＋先行Snapshotの負例なし |

`allocate_keyframe_id`は現行公開APIにあるため「raw allocatorが全て存在しない」とは主張しない。D1lで閉じたのはEffect identityの公開採番口であり、U4b-0が棄却したのはKeyframe allocatorとSetPropertyを組み合わせる製品動線である。

## 5. GAP-24の再開境界

- 先行Snapshot、その後の有効Edit、既知v1／v2 apply失敗を含むfixtureで、製品recoverと同じ`fallback_on_failure=true`を再現する。
- decode破損、未知version、typed apply failureを別々にし、どれがSnapshot fallback、typed stop、部分replayになるかを現行決定へ一致させる。
- 既知wireの互換失敗を正常なfallbackとして編集喪失へ送らず、原本／WAL不変とdiagnosticを維持する。
- journal wire、Snapshot format、Document schema、公開Commandへ便乗変更しない。期待値を現行実装へ合わせて「apply失敗も破損」と再定義したくなったらSTOPし、仕様改訂を先に行う。

## 6. 復活させないもの

- identityを導入するCommandを開集合の「等」で例外化すること。
- counter以外のversion、minimum、作品fieldをUndo全文一致から外すこと。
- apply、replay、Redoがその場でIDを選ぶこと。
- v1 wireへ必須fieldを足す、現行Commandへ直接serdeする、旧WALを上書きすること。
- migrationとjournalへplannerを二重実装したり、公開APIへ出したりすること。
- Unlinkへ偽reservationを持たせること。
- 後発blobだけを根拠にEffect 8 variant／prepare 4本を実装済みとすること。
- U4b-0を公開allocator＋SetPropertyへ縮退すること。
- 先行Snapshotを欠く既存試験だけでknown apply failureの非fallbackを証明したとすること。
- 歴史v4を現行Document versionへ戻すこと。

## 7. 固定歴史出典とcoverage

初版`13a435f9`を全文で読み、Position Add Key追補版`0ec7e636`との差分を確認した。処分した2 unique blob（27,191 bytes）の完全SHAは`evidence/historical-value-recovery/disposition-receipts/04m-d1l-journal-undo.tsv`を正本とする。cutoff総数1,797のうち処分済みは331、未処分は1,466である。
