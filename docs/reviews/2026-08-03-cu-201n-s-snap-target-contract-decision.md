# CU-201N-S snap target / priority / unit 契約決定

- 日付: 2026-08-03
- 状態: **SPEC DONE / CU-201N-S**
- 親: CU-201 / U3b / VS-2

## 1. 結論

現行の製品targetだけで、Timelineのbaseline snap候補を次の二種類に限定する。

1. `TimelineKey::t` — 既存keyframeの時刻
2. `TimelineBar::{start,end}` — 既存Clipの左右edge

候補は現在の`Document` snapshotから導出する一時値であり、Document、serde、journal、Undo、
project sessionへ保存しない。候補の表示名やlabelをidentityに使わず、`LayerId`と`KeyframeId`を
そのまま保持する。

候補は距離の小さい順に選ぶ。同距離の場合は、既存`TimelineProjection::hit_test`と同じ
key優先を維持し、`TimelineKey`をClip edgeより優先する。同一種類の同距離は次で決定する。

- keyframe: `(LayerId, KeyframeId)`の辞書順
- Clip edge: `(LayerId, edge-order)`の辞書順。edge-orderは`start`を`end`より先にする

許容距離はgesture中だけが所有する一時的な`RationalTime`値とする。正の有限値を超える候補は
snapせず、距離が0または不正な値もsnapを発生させない。thresholdをDocumentや公開設定へ追加せず、
logical px、DPI、fps、beat、markerから暗黙に導出しない。

composition boundary、playhead、frame gridは現行M3製品targetとして採用しない。playheadは
`ProductRuntime`のplace requestに現れる値だけで、snapを所有する現在のconsumer/intentがなく、
frame gridは`RationalTime`のtimebaseをsnap policyへ昇格する既決契約がない。beat gridとuser markerは
U7/GAP-16へ残す。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [M3 U3b](../specs/M3-ui-integration.md)、[CU-201S](2026-08-01-cu-201-u3b-move-trim-snap-responsibility-split-decision.md)、[CU-201T-C](2026-08-01-cu-201t-s-clip-trim-timemap-contract-decision.md) |
| `INTERNAL TARGET` | `TimelineBar::{start,end}`、`TimelineKey::{t,layer,key}`、`TimelineProjection::hit_test`、`RationalTime`、既存`SetClipStart`/`TrimClipIn`/`TrimClipOut` command |
| `OWNER` | Clip/keyframe時刻はDocument。candidateとthresholdはHost Transient。確定値を書くのは既存single writerだけ |
| `WRITE ROUTE` | gesture previewはread-only snapshot、releaseだけ既存D2 command→journal-first→history→published snapshot |
| `GAP` | snap候補の集合、同距離優先、距離単位、no-snap条件が未決。既存targetは確認済みで新しいidentityは不要 |
| `RESOLUTION ROUTE` | `REUSE`。Timeline projectionのstable identityと`RationalTime`を使い、候補policyだけをdocsで閉じる |
| `DISPOSITION` | `PASS`。次の実装境界はnative gestureの`CU-201P` |

## 3. candidate source と不変条件

- candidateは同じDocument snapshotから一回だけ導出する。drag中にlive Documentを読む、再走査して
  targetを増減する、labelからLayerIdを再解決する、のいずれも行わない。
- `TimelineBar`の`start/end`と`TimelineKey`の`t`は`RationalTime`のまま比較する。f64座標へ変換して
  距離を判定しない。
- 同じ時刻に複数candidateがあってもidentityごとのcandidateを消さない。priorityと辞書順だけで
  一件を選ぶ。
- 現在のfull-composition projectionが対象を供給できない場合、製品gestureはsnapなしへ縮退し、
  sourceを推測して補わない。

## 4. oracle

1. 同一snapshot・同一threshold・同一pointer時刻は同じcandidateとresultになる。
2. keyとClip edgeが同距離ならkeyが勝ち、同kindの同距離はstable identity順になる。
3. threshold外、0、不正値、candidate空はno-snapで、Document/history/journal/revisionを変更しない。
4. 表示labelが同じでも`LayerId`/`KeyframeId`ごとのcandidateを混同しない。
5. preview中のDocument writeは0、releaseの採用は既存P02 commandを一回だけ使う。

## 5. 非目標とSTOP

- playhead、frame grid、beat grid、user marker、BPM、marker persistenceをこの粒へ追加しない。
- thresholdをUser settings、Document、serde、journal、public APIへ追加しない。
- logical px、DPI、zoom、fpsから暗黙の恒久値を作らない。
- snap専用の第二writer、generic interval command、UI-local identity、別のTimeline projectionを作らない。
- full-composition sourceが実在しないままviewport内の見た目からcandidateを推測しない。その場合は
  `CU-201P`を止め、missing source targetをCodexへ返す。

## 6. 次handoff

`CU-201P`はこのpolicyを一時candidateとして既存`TimelineHit`/pointer captureへ接続し、drag中write 0、
release 1 Undo、Cancel 0だけを閉じる。random列と通常window reopenはそれぞれ`CU-201R`/`CU-201E`へ残す。
