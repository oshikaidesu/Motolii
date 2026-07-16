# D1l journal互換・Undo等価・Writer採番境界 — 追補決定

日付: 2026-07-15  
対象: D1l PR #173 / PR #196追補  
状態: 【決定】（最終反対側レビューP0/P1=0・merge可）

## 1. 発見した停止要因

PR #196追随実装のレビューで、次の3点が未決のまま残っていた。

1. Copy LocalのUndoで`next_stable_id`を巻き戻さない契約と、「Document全文一致」という試験文が文字通り両立しない。
2. 既存`JournalEdit.format_version=1`の`AddEffect`/`RemoveEffect`へ必須fieldを追加すると、旧WALをdecodeできない。
3. `prepare_add_effect`がcallerの`peek_next`/`from_raw`採番を前提にし、Writer準備APIと既存allocate APIが二重の正規経路になる。

コード側でテストを部分一致へ弱めたり、serde defaultで予約を捏造したりして閉じてはならない。以下をD1l/D1d/D2の接続契約として固定する。

## 2. Undo/Redoの等価境界

### 2.1 新規stable identityを導入しないCommand

`Set*`、Unlink、orphan Definition削除等は、`apply→inverse`後に**Document全fieldが`==`**であることを維持する。既存D2共通property helperを弱めない。

### 2.2 新規stable identityを導入するCommand

例外の閉集合は`Command::stable_id_reservation()`が`Some`を返すv2 `CreateEffect`、`UndoCreateEffect`、`LinkEffectUse`、`UndoLinkEffectUse`、`CopyLocalEffect`、`UndoCopyLocalEffect`だけとする。将来variantを足す場合も、このmethodと専用property testへの追加なしに例外へ入らない。`AddTrackItem`はCommand構築前に既発行identityをpayloadとして受ける現契約のため、このCommand apply/inverse境界では全文一致を維持する。`duplicate_track_item`はCommand外の既存専用操作であり、既存のcounter非巻戻し審判を維持する。

上記6 variantは、Undoでidentityを消しても`next_stable_id`を巻き戻さない。D1l Commandはversion 4へmigration済みのDocumentだけに適用し、`version`/`min_reader_version`を変更しない。除外fieldは`next_stable_id` **1つだけ**とし、次を同時に審判する。

- `next_stable_id`だけを除く**Documentの全field**はapply前と`==`。`version`/`min_reader_version`も一致対象。
- 実値の`next_stable_id`は初回apply後の`reservation.after`を維持し、Undoで減らない。
- Redo後はcounterを含むDocument全文が初回apply後と`==`。
- apply/Undo/Redoの失敗時は例外なしでcounterを含むDocument全文が不変。

専用helperを`assert_identity_command_roundtrip(before, command, reservation)`として固定する。初回apply後を保存し、Undo後Document cloneの`next_stable_id`だけを`before`値へ正規化して全文`==`、実counterは`reservation.after`、Redo後は保存した初回apply後と全文`==`を1 helper内で全て検査する。非identity用`assert_command_roundtrip_full`は無正規化の全文`==`を維持する。`tracks`と`effect_definitions`だけの部分比較は禁止する。

## 3. JournalEdit v1→v2互換

### 3.1 wire方針

- 新規書込みは`JournalEdit.format_version=2`。
- journal file/header自体のformat versionは変えない。1本のWALにv1/v2 Edit frameが混在できる。
- decoderはv1とv2を明示分岐し、未知版は従来どおり型付き拒否する。
- v2のEffect lifecycle Commandは新variantとして追加する。v1の既存`AddEffect`/`RemoveEffect` wire形へ必須fieldを追加せず、`serde(default)`で予約・Definition IDを捏造しない。

v2の正準variant（wire名を固定）:

| variant | 新identity | 必須payload |
|---|---|---|
| `CreateEffect` / `UndoCreateEffect` | Use + Definition + nested Keyframe | target、index、完全Use、完全Definition、reservation |
| `LinkEffectUse` / `UndoLinkEffectUse` | Use | target、index、完全Use、reservation |
| `UnlinkEffectUse` / `RestoreEffectUse` | なし（既発行Useの除去/復元） | target、index、完全Use |
| `CopyLocalEffect` / `UndoCopyLocalEffect` | Definition + nested Keyframe | PR #196の4 payload |

`UnlinkEffectUse`へ偽のreservationを載せない。Create/Linkのinverseだけが元のreservationを保持し、Redoで同一IDを復元する。

### 3.2 v1 Effect command adapter

v1 Editは失わず、replay境界でのみlegacy adapterを通す。adapterはIDを適用中に選ばない。D1eと共用する非公開の純粋`LegacyEffectMigrationPlanner`が、適用前に移行済みpayloadとcounter watermarkを完全具体化する。

- v1 envelopeはversionを先にraw decodeし、v1 Commandを現行`Command`へ直接serdeしない。旧inline Effectを内包し得る`AddTrackItem`/`RemoveTrackItem`を含むv1 enum全体を明示adapterで扱う。
- plannerの固定式: まずbase Documentと対象v1 payload全体のUse/Keyframe/既存Definition IDを走査し、`plan_start = max(document.next_stable_id, max_observed_id + 1)`（IDなしならcounter）とする。旧Use/Keyframe IDは保持し、新規Definitionが`n>=1`件の時だけDefinition IDを`plan_start`から固定走査順で連続採番し、`counter_after = plan_start + n`とする。新規Definitionが0件（v1 Remove系、Effect空AddTrackItem等）は`counter_after = expected_counter_before = document.next_stable_id`でcounterを一切変えない。overflow、payload内重複、Documentとの衝突は適用前に型付きRejectする。
- 固定走査順はD1eと同じくTrack順→Item順→Group pre-order→各envelope effect stack順。これはidentity採番順であり、子合成後にGroup effectを評価するD3評価順とは別物。単独`AddEffect`はその1 Effectだけ。planner本体を`migrate.rs`から抽出してD1e document migrationとv1 adapterで共用し、同じ走査を2実装しない。
- planner出力は非公開`PreparedLegacyEdit { expected_counter_before, counter_after, complete_payload }`。applyは全検査後に完全payloadを挿入/除去しcounterを`counter_after`へcommitするだけで、`allocate()`やID選択をしない。公開v2 `Command`/Writerからこの型へ到達できない。
- v1 `AddEffect`は旧inline payloadのUse ID・全Keyframe ID・plugin/params/extraを保持し、plannerが追加した1 Definition + 1 Useへ変換する。
- v1 `RemoveEffect`は旧inlineのdestroy意味を保つ。Use IDと全semantic payloadを照合してUseを除去し、そのDefinitionがsole-useなら同時に削除する。他Useから共有されていれば型付きRejectし、部分適用しない。GAP-14のOrphanKeepはv2 Unlinkにだけ適用する。
- v1 `AddTrackItem`はGroupを含むpayloadを上記固定順で再帰し、全inline EffectのUse/Keyframe IDを保持してDefinition IDだけを追加する。v1 `RemoveTrackItem`は移行済みitemと意味照合し、subtree内Useと対応するsole-use Definitionを除去する。外部Use共有があれば全体Rejectする。
- `SetProperty`等、inline Effect/TrackItemを内包しないv1 variantはfieldを変えず現行意味へ写像する。EffectParam targetの旧Effect IDは移行後Use IDとして保持される。
- adapterは通常のv2 Command構築APIへ露出せず、journal replay専用。v2 apply中のID割当禁止を緩めない。

mainまたはgenerationが旧inline schemaの場合、recoveryは既存D1e migrationを**メモリ上**で通してからv1 Editを適用する。原本・旧generation・旧WALは上書きせず、結果は従来どおりrecovered文書へ書く。通常の`load_document`が旧形式を黙ってmigrationする経路は作らない。

旧v1 Editのdecode不能を正常なupgradeとしてsnapshot fallbackへ落とす案は棄却する。fallbackは破損時の出口であり、既知版の編集喪失を互換戦略にしない。

## 4. Writer準備APIを単一路にする

正規APIは次の3本とし、いずれもcounter複製上で完全payloadを作りDocumentを変更しない。

1. `prepare_create_effect`: 新Use→新Definition→params辞書順→Keyframes格納順（Vec2Axes x→y）の順で連続採番する。
2. `prepare_link_effect_use`: 指定済みDefinitionを検証し、新Useだけを採番する。
3. `prepare_copy_local_effect`: PR #196どおり新Definition→nested Keyframeを採番する。

Create入力はidentityを持たないruntime-only `EffectDefinitionDraft`（plugin/version/enabled/params/extra）とする。paramsは`DraftDocParam`、キーフレームはIDを持たない`DraftKeyframe { t, value, interp }`で表し、Const/Data/Vec2Axes/LookAt/Followを現行`DocParam`と同じ意味で保持する。prepareが`DocParam`へ具体化する時だけ全Keyframe IDを採番する。Draft型群はserdeせずDocument schema・journal・plugin契約へ焼かない。

D1l候補で追加された`allocate_effect_id`、`allocate_effect_definition_id`、`allocate_unique_effect_pair`は公開正規APIにしない。migration/testのraw構築は内部helperへ限定し、製品/受け入れテストが`peek_next`/`from_raw`で新規identityを組み立ててprepareへ渡すことを禁止する。

## 5. Atomicity

Effect lifecycle Commandは全参照、index、payload、予約、衝突、counter位置、Undo干渉をmutation前に検査する。検査後のcommit区間に`?`を置かない。安全に一括commitできない場合はDocument cloneへ適用・validate後にswapする。失敗時の全文不変を試験する。

## 6. 自動試験

1. `stable_id_reservation()`閉集合6 variantのtag集合を固定し、それ以外は`assert_command_roundtrip_full`でDocument全文一致を維持する。
2. identity導入3操作と各inverseは`assert_identity_command_roundtrip`でcounter 1field正規化後のDocument全文一致、実counter=`after`、version/min不変、Redo全文一致。
3. v1 raw JSON `AddEffect`/`RemoveEffect`およびnested Effect付き`AddTrackItem`/`RemoveTrackItem` Edit fixtureをv1 baseへ順にreplayし、D1e migration後の意味・order・extra・Use/Keyframe IDを保持する。
4. `crates/motolii-doc/tests/fixtures/journal_v1/commands.jsonl`にv1全serde tagを1件以上置き、現行v1 tag集合との完全一致をmeta-testする。v1/v2混在WALを順にreplayし、全Edit適用・`replay_failures=[]`。
5. 未知Edit版は型付き拒否、破損時だけ既存snapshot fallback。
6. create/link/copy-localの準備前後でDocument全文不変。導入IDと予約全点一致。
7. 公開製品経路に`peek_next`/`from_raw`採番がなく、Createのnested KeyframeもWriterが再採番する。Draft型群がSerializeを実装せず、公開`allocate_effect_*`が存在しないこともAPI/grep gateで固定する。
8. Effect lifecycle全失敗fixtureでDocument全文不変。
9. v1 Add→Removeとnested AddTrackItem→RemoveTrackItemでUse/Definition台帳件数までbaseと一致し、orphanを増やさない。shared干渉は全文不変Reject。
10. `UnlinkEffectUse`/`RestoreEffectUse`のJSONにreservation keyがなく、Undoでcounterが変わらない。
11. v1 `PreparedLegacyEdit.expected_counter_before`不一致は全文不変Reject。新規Definition 0件はcounter不変、1件以上は固定式どおりの`counter_after`。
12. `LegacyEffectMigrationPlanner`/`PreparedLegacyEdit`をcrate公開API（`pub`/`lib.rs` re-export）へ出さず、crate-private共有module（`pub(crate)`以下）に置く。D1e migrateとjournal v1 adapterがその同一planner関数を参照し、二重実装がないことをAPI/grep gateで固定する。

## 7. 非目標

Documentへのtombstone/reservation追加、敵対的raw Commandの完全検出、journal header format変更、旧WAL原本の書換え、M3 UI API、Cascade/Purge。

## 8. 実装順序

1. 本決定の反対側レビューとmerge。
2. D1l commandをv2新variantへ分離しWriter準備APIを実装。
3. v1 adapter + v1/v2混在replay fixture。
4. D2等価helperを復元しidentity専用helperを追加。
5. D1l全条件、clippy、workspace test、独立コードレビュー。

D3eはD1lがmainで閉じるまで発注しない。
