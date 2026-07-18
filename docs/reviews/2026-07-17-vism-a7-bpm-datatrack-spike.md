# VSM-A7 — BPMから既存DataTrackへの意味spike

作成日: 2026-07-17

状態: **意味spike完了／公開型・Document schema・plugin入力portの追加なし**。現行`Document.bpm`から既存`DataTrackId`→`DocParam::Data`だけでparameterを駆動できる範囲をコードfixtureで確認した。

関連文書: [VSM-A0 inventory](2026-07-17-vism-a0-plugin-boundary-inventory.md)、[Vism実装計画](2026-07-17-vism-implementation-plan.md)、[Vism / Kitモデル](../vism-kit-model.md)

## 1. 結論

固定BPMに同期する最小の値pipeは、新しいBeat型なしで成立する。

```text
Document.bpm
  → beat_position(t) = t / beat_duration
  → DataTrack<Value::F64>
  → DataTracks[DataTrackId]
  → DocParam::Data
  → 既存parameter評価
```

DataTrackへ保存するのは0〜1へ折り返したphaseではなく、時間ゼロからの**連続拍位置**である。

```text
beat_duration = 60 / bpm
beat_position(t) = t / beat_duration = t × bpm / 60
```

連続拍位置は時刻`t`の一次関数なので、DataTrackの既存線形補間がサンプル間でも同じ意味を復元する。折返しphaseを直接サンプル化すると、1→0を跨ぐ区間で線形補間が逆方向を通るため、A7の共通値には採らない。

これは「BeatEventsが決まった」「BPMをVism化した」「consumer pluginができた」という結果ではない。固定BPMを既存parameterへ渡す最小核が既にある、という証拠である。

## 2. fixture

実装: `crates/motolii-doc/tests/vism_a7_bpm_datatrack.rs`

test-only helperが`&Document`、track開始時刻、既存`Fps`、sample数を受け、`DataTrack<Value::F64>`を返す。製品crateへ新しい関数・型・traitを追加していない。

| fixture | 証明すること |
|---|---|
| `fractional_bpm_and_ntsc_rate_produce_deterministic_positions` | 120.35 BPMと30000/1001 fpsでも、累積加算せず有理時刻から同じ値列を再生成できる |
| `doc_param_reads_the_same_beat_position_in_any_seek_order` | forward／reverseの評価順に関係なく、`DocParam::Data`が任意時刻の同じ拍位置を返す |
| `quality_does_not_change_data_track_meaning_or_document_bytes` | Draft／Finalで値が変わらず、track生成はDocumentを変更・直列化しない |

時刻とBPMの比較・サンプル位置は既存有理数を使う。最終出力が`Value::F64`なのは新しい選択ではなく、現行DataTrack値型の制約である。fixtureは逐次的な`phase += delta`を使わないため、seek前の評価履歴を持たない。

## 3. GR-PV判定

| 項目 | 判定 |
|---|---|
| 意味が先か | **Yes**。固定BPMの拍長はM2仕様で`60 / bpm`と決定済み |
| 恒久面は狭いか | **Yes**。schema、version、serde fieldを追加しない |
| 追加的か | **Yes**。test-only fixtureで既存意味を観測する |
| 依存直列か | **Yes**。完了済みBPM／DataTrack／DocParam評価だけを使う |
| 意味の審判があるか | **Yes**。fractional BPM×NTSC、seek順、Quality、Document bytesを自動試験する |

## 4. 確定したこと

1. 固定BPMをparameterへ渡すためだけなら、BPM専用plugin traitやconsumer input portは不要。
2. BPM由来データはDocumentへ保存せず、既存BPMから決定的に再生成できる。
3. global phaseの原点は現行timeline time zeroで表現でき、track開始が負でも意味は変わらない。
4. Preview／Export差はQualityだけという既存規律に対し、BPM値列はQuality非依存でよい。
5. provider→既存parameterのpipeは成立するが、provider→consumer pluginの一般接続は未成立のままである。

## 5. まだ決めていないこと

- 0〜1 phase、pulse、sin、swing等の派生値を誰が生成するか。
- beat／bar／meter／labelを持つstructured event型。
- tempo map、拍子変更、曲途中の位相原点、pickup。
- DataTrack生成をHost built-in、ParamDriver、Vism entryのどこへ置くか。
- 具体provider選択とKit materialize方式。
- live MIDI／audio解析等のfork固有provider。

これらをA7のtest helperから公開APIへ昇格しない。BPM連続拍位置だけで複数の利用例が成立するかを観察し、一般consumer方式はVSM-B2で既存DataTrack、input port、Authoring Toolを比較して決める。

## 6. 次の順序

```text
VSM-A7 完了
  ↓
VSM-A0D migration／Document既知表の所有処分
  ↓
VSM-A0S contract catalog／prepared migration仕様
  ↓
VSM-A0I-1〜3 採択仕様の直列実装
  ↓
VSM-A1 Opacity別crate実証
  ↓
VSM-A2 Sine別crate実証
  ↓
VSM-B0/B1 identity／成果物境界
  ↓
VSM-B2 provider／consumer／Kit方式決定
```

A7の結果だけでBPMをVism packageへ移したり、BeatEventsをcoreへ追加したりしない。次に処分すべきものは値pipeではなく、外部plugin化を妨げているmigrationとDocument既知契約の二重所有である。
