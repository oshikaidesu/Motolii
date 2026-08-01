# U4b-1 outgoing Interp command契約決定

作成日: 2026-08-02  
状態: **決定**。CORE `U4b-1C`を実装可能。React trigger / native popup / Host lifecycleは別粒。

## 1. 利用者成果

Positionの隣接2 key間を選び、Easingを変更してもkeyの個数・ID・時刻・値と外側区間を変えず、
1 release / 1 command / 1 Undoで左keyのoutgoing `Interp`だけを更新する。

## 2. 既存契約接続票

| field | resolution |
|---|---|
| `AUTHORITY` | [M3 U4b](../specs/M3-ui-integration.md)、[native Easing popup受入契約](2026-07-22-m3-native-easing-popup-acceptance.md)、[AM観察 §5.2](2026-07-19-am-keyframe-graph-observation.md#52-複数点カーブ構築は停止し単一区間graph-viewを優先する) |
| `INTERNAL TARGET` | `DocParam::Keyframes`内の左`DocKeyframe.interp`。評価は既存`motolii-eval::Interp` |
| `OWNER` | `Document / D2 single writer`。選択とdrag previewはHost `Transient` |
| `WRITE ROUTE` | `prepare_set_position_key_interp` → durable `Command::SetPositionKeyInterp` → `DocumentWriter::apply_command` → JournalEdit v2 |
| `GAP` | 左右key identityを固定し、stale intervalを拒否する専用command / prepareが無い |
| `RESOLUTION ROUTE` | `REUSE`: `Command` old/new対称、JournalEdit v2、既存`validate_interp`、atomic clone/swapを再利用 |
| `DISPOSITION` | `PASS`。公開schema、Journal version、plugin契約の変更0 |

## 3. command契約

`Command::SetPositionKeyInterp`は次を完全payloadとして持つ。

- `target: LayerId`
- `left_key_id: KeyframeId`
- `right_key_id: KeyframeId`
- `old: Interp`
- `new: Interp`

`CommandKind::SetPositionKeyInterp`をforward / inverseで共有する。`target_stable_id`はLayerId、
`PropertyId`は既存`Position`を使う。fresh identityを導入しないため`StableIdReservation`は持たない。

applyはPositionが`DocParam::Keyframes`であり、left直後のkeyがrightであり、leftの現行`interp == old`で
ある場合だけ、clone上でleft `interp = new`、Document全体validate後にswapする。inverseはold/newを交換する。
same-value prepareは`None`。last key、非隣接right、欠落key、unsupported Position、invalid new Interp、
stale oldは型付き拒否し、Document / counter / historyを変えない。

## 4. active interval read projection

`layer + left_key_id`からread-onlyに直後のrightを導出し、次を返す。

- layer、left/right key ID
- start/endの厳密`RationalTime`
- 左keyのoutgoing `Interp`

last key、非Position、unsupported source、欠落identityは`None`。selection正本、revision、popup state、
screen座標をこのprojectionへ追加しない。Hostはpublished snapshot revisionと組み合わせて使用する。

## 5. 負例と非目標

- key追加・削除・移動・値変更、右key `Interp`変更、外側区間変更を拒否する
- raw `SetProperty`、indexだけのaddressing、second selection store、generic keyframe frameworkを作らない
- Bounce / Elastic / Stepsの未実装variant、preset codec、React/native popup、AX/z-order/DPIを本粒へ束ねない
- journal/schema versionを上げず、JournalEdit v2 roundtrip/replayを必須oracleにする

## 6. 完了oracle

1. Hold / Linear / Bezierのforward → Undo → Redoが同一payloadで成立
2. key count / IDs / times / values、right/outside Interp、Document versionが不変
3. stale old / stale successor / last key / invalid curveが無変更拒否
4. active intervalがleftの直後rightとoutgoing Interpを決定的に返す
5. JournalEdit v2 roundtrip/replay、docs、Rust workspace回帰が緑
