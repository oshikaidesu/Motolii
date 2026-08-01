# CU-201T-S Clip trim / TimeMap 契約決定

- 日付: 2026-08-01
- 状態: **SPEC DONE**
- 親: CU-201 / U3b / VS-2

## 1. 結論

Clip trimはmoveやretimeへ一般化せず、左端の`TrimClipIn`と右端の`TrimClipOut`へ分ける。
いずれも現行`LayerId`を対象にし、`Clip.duration`を半開表示区間の正本、`TimeMap`を
clip-local時刻からsource時刻への純写像として維持する。

- **in trim**: 新しい左端を`new_start`とする。旧右端を固定し、`start`、`duration`、
  `time_map.source_start`を同時に変更する。speedと`overrun_mode`は不変
- **out trim**: `start`と`TimeMap`全fieldを固定し、新しい右端から`duration`だけを変更する

`old_start = s`、`old_duration = d`、`new_start = s + delta`、speedを`v`とすると、in trimは

```text
new_duration = d - delta
new_source_start = old_source_start + delta * v
```

である。したがって`new_start + new_duration == s + d`で右端は不変となり、残る全timeline時刻で
`new_time_map.map(t - new_start) == old_time_map.map(t - s)`が厳密に成立する。全時刻は
`RationalTime`、speedも正の既約有理数なので丸めは行わない。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [CU-201S](2026-08-01-cu-201-u3b-move-trim-snap-responsibility-split-decision.md)、[M3 U3b](../specs/M3-ui-integration.md)、[M2 D1g](../specs/M2-document-model.md) |
| `INTERNAL TARGET` | `Clip::{start,duration,time_map}`、`TimeMap::{source_start,speed_num,speed_den,overrun_mode}`、`Command` / `DocumentWriter` / `JournalEdit` v2 |
| `OWNER` | 確定intervalとTimeMapはDocument。drag candidateは後続CU-201PのHost Transient |
| `WRITE ROUTE` | Writer prepare → typed command → journal-first D2 → history → published snapshot |
| `GAP` | trim用command / prepare / inverse / replay oracleが0 |
| `RESOLUTION ROUTE` | D1gの厳密写像、既存old/new対称command、single writer、JournalEdit v2を`REUSE`し、二つのedge操作へ`REDUCE` |
| `DISPOSITION` | `PASS`。`CU-201T-C`を一つのCORE粒として発行可能 |

## 3. command payloadとprepare

deltaをjournalへ保存せず、apply / inverse / replayがlive speedから再計算しない絶対old/new payloadを使う。

```rust
Command::TrimClipIn {
    target: LayerId,
    old_start: RationalTime,
    old_duration: RationalTime,
    old_time_map: TimeMap,
    new_start: RationalTime,
    new_duration: RationalTime,
    new_time_map: TimeMap,
}
Command::TrimClipOut {
    target: LayerId,
    old_duration: RationalTime,
    new_duration: RationalTime,
}
```

`DocumentWriter`は次の二入口だけを追加する。

```rust
prepare_trim_clip_in(target, new_start) -> Result<Option<Command>, CommandError>
prepare_trim_clip_out(target, new_end) -> Result<Option<Command>, CommandError>
```

prepareはlive Clipからold値を一度収集し、checkedな有理数演算でnew値を導出する。
in側の`new_time_map`は`source_start`だけが上式の値で、speed / overrun_modeはoldと同一。
out側の`new_duration`は`new_end - start`。入力edgeが現在edgeと同じなら`Ok(None)`であり、
journal / revision / Undoを増やさない。

`CommandKind::{TrimClipIn,TrimClipOut}`と`PropertyId::{ClipIn,ClipOut}`を分ける。
同じgesture / target / 同じedgeだけをfirst-old / last-newへmergeし、in/out相互、move、異なるtargetとは
mergeしない。raw applyの`old`は既存D2どおりCAS preconditionではない。

## 4. apply / inverse / 永続互換

in applyはpayloadの`new_start` / `new_duration` / `new_time_map`を、out applyは`new_duration`を
一回のatomic commandとして書く。部分書込みを許さない。inverseは同じvariantのold/newを全て交換する。
速度変更がtrim後に追加されても、LIFO Undoは保存済みold TimeMapを復元し、live speedからdeltaを再計算しない。

- Document schema、reader / writer version、journal file/header versionを変更しない
- 新規writeは既存`JournalEdit.format_version = 2`の正準command variantとして保存する
- v1 legacy tag集合とadapterを変更しない
- `stable_id_reservation()`は`None`
- JSON wire tagは`TrimClipIn` / `TrimClipOut`、field名は上記payloadのsnake_caseを固定する

## 5. 境界と拒否優先順

負の`start`と同一lane overlapは現行どおり許す。durationはframe数でなく厳密な`RationalTime`で、
最小有効値は`> 0`。1 frameへの暗黙snap、clamp、ripple、collision回避は行わない。

prepareとraw applyは、対象解決後に各variantの全new値を一時値としてchecked計算・検証し、成功時だけ書く。
拒否優先順は次で固定する。

1. target不在: 既存`LayerNotFound`
2. Group target: `TrackItemNotClip { layer }`
3. start/end、duration、`delta * speed`、source_start加算の算術失敗: 対応する既存
   `RationalTimeError`または`ClipIntervalOverflow`
4. `new_duration <= 0`: `CommandError::Validate(DocumentError::NonPositiveClipDuration)`
5. in payloadで右端不変、speed / overrun_mode不変、source写像保存のいずれかが崩れる:
   typed `CommandError::InvalidClipTrim`
6. `new_start + new_duration > composition.duration`: 既存`ClipPastComposition`

終端一致は成功。in trimの正規prepareでは旧右端を保つため6へ新たに到達しないが、raw payloadにも同じ
Document不変条件を適用する。out trimの上限判定は`start + new_duration`で行い、負startを理由に
`new_duration <= composition.duration`へ縮約しない。

TimeMapは素材尺を知らず、source始端前・終端後は既決`OverrunMode`をD3がavailable範囲へ適用する。
trim prepareはsource boundsを拒否せず、Black / LoopをFreezeへ縮退させない。

## 6. CU-201T-Cの必須oracle

1. in trim縮小/拡張で右端、全残存timeline時刻のsource時刻、speed、overrun modeが厳密一致
2. out trim縮小/拡張でstartとTimeMapが値同一、durationだけが変わる
3. 負start、overlap、終端一致、source available範囲外は成功
4. missing target → Group target → arithmetic overflow → non-positive duration → 壊れたraw in payload →
   past compositionを§5の優先順どおりtyped拒否し、全件でDocument全文不変
5. same edgeはprepare `None`、raw same-value applyはidentity success
6. raw applyのoldがcurrentと不一致でもCAS拒否せずnew全値を書き、inverseはpayloadのold全値を書く。
   apply直前の任意値をexact復元する保証はなく、Writer prepare生成commandのUndoだけが収集済みoldをexact復元
7. inverse / Undo / Redoでstart、duration、TimeMapをexact復元し、live speedからdelta再計算0
8. 同edge mergeはfirst old / last new、in/out・move・target違いは非merge
9. JournalEdit v2 JSON roundtrip / WAL replayで同じintervalとTimeMap。version / stable id不変
10. keyframe時刻、source、envelope、parent、order、identity、他Clipは不変

## 7. 非目標とSTOP

- move、in trim、out trimを汎用interval commandへ統合する
- speed変更、速度ランプ、reverse、slip、roll、rippleを同時実装する
- source尺をTimeMapへ持たせる、またはtrim時にsource boundsを推測する
- frame grid、snap、zoom、DPI、fps、beat、markerをcommand意味へ入れる
- keyframeをstretch/deleteする、区間外keyを削除する
- public raw mutator、UI専用command、第二writerを追加する
- Document schema / journal version / plugin契約を変更する
