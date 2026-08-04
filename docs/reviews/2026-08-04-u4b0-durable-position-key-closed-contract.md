# U4b-0 durable Add Position Key closed contract

状態: **決定 / code実装受入済み / main統合済み**

日付: 2026-08-04

## 1. 利用者出口と一契約境界

[Motion Authoring Loop](2026-08-04-outcome-spine-autonomous-gap-research-decision.md#61-motion-authoring-loopの背骨)のうち、選択済みRectangleのPositionへplayhead時刻のkeyを明示追加し、Undo／Redo／journal replayで同じ`KeyframeId`を保つD2境界だけを閉じる。UI、Auto Key、on-key value編集、active interval、outgoing Interp変更、Easing、Transportは本粒へ入れない。

```text
AUTHORITY: historical-d2-selection-timeline-lineage-recovery §4
INTERNAL TARGET: DocumentWriter::prepare_add_position_key -> Command::AddPositionKey
OWNER: DocumentWriter / Command / existing motolii-eval Interp
WRITE ROUTE: one prepared durable Command -> existing single writer / Undo / JournalEdit v2
GAP: exact Rust DTO、Bezier split、numeric tolerance、journal version要否
RESOLUTION ROUTE: existing lifecycle reservation pattern + existing RationalTime/Interp/Bezier solver
DISPOSITION: docs contract, bounded code acceptance, and main integration closed
```

## 2. 既知実装採択

```text
MECHANISM CLASS: stable-IDを1件導入するPosition専用atomic Document command
KNOWN IMPLEMENTATION SEARCH: repo Command forward/inverse、effect lifecycle reservation、DocKeyframeTrack、Interp、cubic_bezier_ease、UndoHistory、JournalEdit v2 decoder、歴史回収§4
CANDIDATES: lifecycle reservation pattern / SetProperty + external allocate / generic keyframe command / second Bezier solver
ADOPTION ROUTE: PATTERN（lifecycle reservation）+ REUSE（RationalTime、track eval、Interp、Bezier solver、Undo、journal v2）
REJECTED CANDIDATES: SetProperty + external allocate=journal counter不再現 / Auto Key=非目標 / generic keyframe API=境界過大 / second solver=重複owner
THIN MOTOLII SEAM: Position専用forward/inverse、prepare outcome、既存Interpのde Casteljau split
THIN MOTOLII RESIDUAL: old/new payload関係と1-ID reservationの検証だけ
RETIREMENT: allocate_keyframe_id + SetPropertyを製品Add Position Key routeにしない
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN（generic framework、新solver、新schema、別journal形式）
```

## 3. exact Rust contract

`Command`へ次の2 variantだけを追加する。既存`SetProperty` wireへfieldを足さない。

```rust
AddPositionKey {
    target: LayerId,
    old_value: DocParam,
    new_value: DocParam,
    added_key_id: KeyframeId,
    stable_id_reservation: StableIdReservation,
},
UndoAddPositionKey {
    target: LayerId,
    old_value: DocParam,
    new_value: DocParam,
    added_key_id: KeyframeId,
    stable_id_reservation: StableIdReservation,
},
```

両者は`CommandKind::AddPositionKey`、`PropertyId::Position`、同じ`target`を使い、`stable_id_reservation()`は`Some`を返す。inverseはvariantだけを反転し、payloadとIDを保持する。構造不一致は`CommandError::AddPositionKeyPayloadMismatch { layer, key_id }`でtyped rejectする。

prepareはeffect専用moduleへ混ぜず、Position専用`position_key_prepare`が所有し、`DocumentWriter`の公開入口から呼ぶ。

```rust
pub enum AddPositionKeyPreparation {
    Prepared { key_id: KeyframeId, command: Command },
    AlreadyPresent { key_id: KeyframeId },
}

pub enum AddPositionKeyPrepareError {
    // existing LayerNotFound / StableId / DocKeyframe / Command errors are transparent
    PositionSourceUnsupported { layer: u64 },
    PositionValueTypeMismatch { layer: u64 },
}

pub fn DocumentWriter::prepare_add_position_key(
    &self,
    target: LayerId,
    t: RationalTime,
) -> Result<AddPositionKeyPreparation, AddPositionKeyPrepareError>;
```

`AlreadyPresent`はexact `RationalTime::cmp`でallocationより先に返す。`Prepared`の作成はlive counterを変更せず、counter cloneから`KeyframeId`を一件だけ取り、reservationを`[id,id+1)`へ固定する。入力sourceは`Const(Vec2)`と、全valueがVec2の`Keyframes`だけである。`Data`、`Vec2Axes`、`LookAt`、`Follow`を暗黙にbake／flattenしない。

forward applyはmutation前に、live Position=`old_value`、added IDがoldに0件・newにexact 1件、newがoldからの決定的挿入、reservationがadded ID一件と一致、ID未使用、counterが`before`またはRedo可能な`>=after`であることを検査する。cloneへ`new_value`を書いてDocument validation後にswapし、counter=`before`の時だけ`after`へ進める。inverseはlive Position=`new_value`、同じpayload関係、counter `>=after`を検査して`old_value`へ戻し、counterを巻き戻さない。

## 4. curve-preserving insertion

- Const(Vec2): playheadへ元値の一keyを作り、outgoingは`Linear`。
- same-time: 既存IDを返すだけでallocation、Command、revision、Undo、journalを0件にする。
- first keyより前／last keyより後: 現行端値と同じ値を挿入する。新key outgoingは`Linear`とし、既存有効区間のinterpを変えない。
- interior `Hold`: 挿入値は左値。左keyと新keyのoutgoingを`Hold`にする。
- interior `Linear`: 既存track評価値を挿入し、左keyと新keyのoutgoingを`Linear`にする。
- interior `Bezier`: 既存solverで時間progress `u`からcurve parameter `s`と評価progress `v`を求め、同じcubicをde Casteljau分割する。左／右controlを各部分区間の0..1へ正規化し、左key／新keyのoutgoingへ置く。

`Interp::split_at(progress)`を既存`motolii-eval`へ追加し、`Hold`／`Linear`は同種2件、`Bezier`は既存private `solve_curve_x`を再利用する。別solver、sample fitting、clamp、Linear fallbackを作らない。Position両端値が等しい区間は値が恒常なので`Hold`へ正規化してから挿入する。両端値が異なり、Bezier分割点の`v`または`1-v`が0、非有限、あるいは正規化後x controlが現行`[0,1]`契約を満たさない場合はtyped `TrackError::UnrepresentableBezierSplit`で無変更拒否する。

時刻同一性とkey順序は`RationalTime`でexact判定する。新しいsolver toleranceは設けず、現行solverのNewton 8回、`EPS=1e-7`、derivative cutoff `1e-6`、binary fallbackを維持する。curve-preservation oracleは挿入前後の両区間sampleでVec2各component誤差`<=1e-6`とする。このtest toleranceをDocument意味やsame-time epsilonへ昇格しない。

## 5. journal / version

`JournalEdit::FORMAT_VERSION`は2のまま、Document version／`min_reader_version`も変更しない。現行v2はcurrent `Command`を直接serdeし、新variantは同じenvelopeでroundtripする。旧shape readerは未知externally-tagged enum variantをserde errorとして`InvalidEditPayload`／既存fallbackへ送り、黙って適用しない。v1は別の`LegacyJournalCommand`をdecodeするためvariantを追加しない。

version bump、v1 adapter追加、unknown variant無視が必要になった場合は実装を止め、M2 persistence amendmentへ戻す。

## 6. 実装allowlistとoracle

```text
ALLOWLIST production:
  crates/motolii-doc/src/command.rs
  crates/motolii-doc/src/lib.rs
  crates/motolii-doc/src/position_key_prepare.rs (new)
  crates/motolii-eval/src/track.rs
  crates/motolii-eval/src/bezier.rs
  crates/motolii-ui/src/diagnostic_projection.rs (CommandKind exhaustive label one arm only)
ALLOWLIST tests:
  inline tests in the files above
  crates/motolii-doc/src/undo.rs (test only)
  crates/motolii-doc/src/journal/replay.rs (test only)
  focused existing integration test file only if the same oracle cannot be expressed inline
PRIMARY_ORACLE:
  pre-edit snapshot + one serialized v2 AddPositionKey -> replay result equals live apply in Position, added ID, and counter; Undo preserves counter; Redo restores same ID
NEGATIVE_ORACLES:
  same-time 0 mutation / missing layer / unsupported or non-Vec2 source / stale old-new payload / duplicate or colliding ID / malformed reservation / unrepresentable Bezier / old reader typed reject / v1 non-acceptance
REPO_LANES: motolii-eval focused tests / motolii-doc focused tests / affected-crate tests / normal rust lane
EXTERNAL_GATES: fresh different-family read-only diff review; UI、product E2E、human目視はU4b-0外
```

workspace compileで`CommandKind::AddPositionKey`追加が既存のexhaustive診断labelを必ず要求するため、既存patternと同形の`"Add position key"`一armだけを薄い接続として許可する。UI操作、layout、component、診断policyの変更へ広げない。

## 7. STOPと次手

別Position source、generic keyframe API、`SetProperty` wire変更、別solver/tolerance、近似／clamp、journal version、Document schema／min-reader変更が必要なら当該施工だけを止める。

このallowlistの`U4b-0 durable Add Position Key implementation`は受入・main統合済みである。現行codeからMotion Authoring Loopの次edgeを再計測し、active interval／Easingを同じ粒へ束ねない。
