# U4b-0 Add Position Key closed contract決定

- 日付: 2026-08-02
- 状態: **SPEC DONE / CORE DO**
- 親: M3 U4b / M2 D2

## 1. 結論

explicit `Add Position Key @ playhead`を、Position専用のdurable commandへ閉じる。
UI、Auto Key、汎用property pathからidentityを発行せず、pre-edit snapshotからjournal replayしても
同じ`KeyframeId`とcounterを再現する。

正準variantは`AddPositionKey` / `UndoAddPositionKey`、共通kindは`AddPositionKey`とする。
両variantは次の完全payloadを持つ。

- target `LayerId`
- 完全な`old_value: DocParam` / `new_value: DocParam`
- `added_key_id: KeyframeId`
- ちょうど1 IDの`StableIdReservation`

既存`SetProperty` wire、Document schema、JournalEdit format version、plugin契約は変更しない。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [U4b-0歴史回収 §4](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#4-u4b-0-durable-add-position-key再採択)、[M2 D2](../specs/M2-document-model.md)、[M3 U4b](../specs/M3-ui-integration.md) |
| `INTERNAL TARGET` | `Command` / `CommandKind` / `PropertyId::Position`、stable reservation helper、`DocKeyframeTrack`、`Interp`、`DocumentWriter` prepare façade |
| `OWNER` | Position curveとKeyframeIdはDocument。playhead / active intervalはProject session / Host Transient |
| `WRITE ROUTE` | explicit request → pure prepare → JournalEdit v2 durable commit → live single writer → history → one publish |
| `GAP` | identity導入Command、prepare result、Bezier split API、区間外canonical形、拒否順が未固定だった |
| `RESOLUTION ROUTE` | effect lifecycle reservation型紙とeval solverを`REUSE`し、Position専用へ`REDUCE` |
| `DISPOSITION` | `PASS`。CORE `U4b-0C`を実装可能 |

## 3. prepare resultと入力

runtime-only resultを次の閉集合にする。serdeせずDocument / journalへ保存しない。

```text
PreparedAddPositionKey =
  Edit { command, key_id }
  AlreadyPresent { key_id }
```

prepare入力はsnapshot、target `LayerId`、playhead `RationalTime`だけ。現在の
`Transform2D.position`が`Const(Vec2)`またはVec2 `Keyframes`の時だけ成立する。
`Data` / `Vec2Axes` / `LookAt` / `Follow`、非Vec2 key trackはtyped rejectする。

- `Const(Vec2)`: 同じ値のkeyを一件作り、outgoing `Linear`
- same-time key: `AlreadyPresent`。採番、command、journal、revision、Undoは0
- animated off-key: 現行trackの評価値を新key valueにする

prepareはlive counterのcloneで一件だけ予約し、old/new trackを組み立て、Commandをclone Documentへ
適用・validateしてから返す。raw `allocate_keyframe_id`や`from_raw(peek_next)`を製品経路に使わない。

## 4. curve-preserving insertion

### 4.1 既存区間内部

- `Hold`: 左key outgoingと新key outgoingを`Hold`
- `Linear`: 左key outgoingと新key outgoingを`Linear`
- `Bezier`: `motolii-eval`が所有する`Interp::split_at_progress(x)`相当の一つの公開意味APIで、
  現行private `solve_curve_x`と同じ数値規則を使ってde Casteljau分割する

`motolii-doc`へ第二solver、EPS、sample探索を複製しない。分割APIは`Interp`から
`(left_interp, right_interp)`を返し、入力はtimeline区間の正規化progress `0 < x < 1`。
非有限、範囲外、分割点の正規化分母が0、非有限control生成はtyped rejectする。

### 4.2 既存key範囲外

従来の端値clampを厳密に保存する正準形を固定する。

- 最初のkeyより前: 新key valueは旧先頭value、新key outgoingは`Hold`、旧先頭以後は不変
- 最後のkeyより後: 旧末尾outgoingを`Hold`へ変更、新key valueは旧末尾value、新key outgoingは`Linear`
- 1 key trackも同じ規則

表示上未使用のoutgoingを任意に保持せず、同じ入力から同じwire形を作る。

## 5. apply / inverse / version

forward applyはmutation前に次を検査する。

1. targetが存在し、current Positionがpayload `old_value`と完全一致
2. `new_value`がvalid Vec2 key trackで、oldとの差が`added_key_id`一件だけ
3. reservationが一件、added IDと完全一致し、既存Document identityと衝突しない
4. counterが初回apply可能な`before`、またはRedo可能な`after`以降
5. Documentがstable-ID対応の`min_reader_version >= 2`

全検査後にcloneへnew valueとreservation commitを適用し、Document全体validate後にswapする。
inverseはcurrent Positionが`new_value`と一致することを要求し、old valueへ戻すがcounterを戻さない。
Redoは同じID / payloadを復元する。version / min-readerをCommand適用中に暗黙更新しない。

JournalEdit v2は既存effect lifecycleと同様、新variant追加で維持する。v1 wireや既存variantへ
fieldを足さず、旧readerは未知variantをtyped rejectする。format version変更が必要と判明したらSTOPする。

## 6. 拒否優先順

1. target不在
2. stable-ID非対応Document
3. Position source / value kind不適合
4. same-time（errorでなく`AlreadyPresent`）
5. invalid existing track / Bezier
6. curve split不能
7. ID exhaustion / reservation不成立
8. constructed payload / Document validation失敗

apply側はtarget / current payload mismatchをreservation commitより先に拒否する。すべての失敗で
Document、counter、history、revision、journal bytesを不変にする。

## 7. 必須oracle

1. Const Vec2 → fresh ID一件 / Linear / one command / one Undo
2. Hold / Linear / Bezier区間内部で挿入前後の評価一致
3. before-first / after-last / one-keyで全時刻の端値clamp一致
4. same-timeは既存IDを返し全state不変
5. apply → inverseはcounter以外全文一致、counter=`after`、Redo全文一致・同じID
6. pre-edit snapshot + JournalEdit v2 replay、Save / reopen一致
7. stale payload、ID collision、reservation hole/intermediate counter、invalid curve、4 MiB payload超過はdurable write 0
8. `SetProperty`、v1 adapter、Effect lifecycle 6 variantの意味不変

## 8. 非目標とSTOP

- key移動、key value編集、outgoing Interp変更、Easing popupを本CORE粒へ束ねる
- Auto Key、汎用key command、EffectParam key作成へ一般化する
- solver / toleranceを`motolii-doc`へ複製する
- Document / JournalEdit versionを黙って上げる
- UI local stateやplayheadをDocumentへ保存する
