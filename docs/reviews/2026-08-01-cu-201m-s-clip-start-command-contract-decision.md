# CU-201M-S Clip start command 契約決定

- 日付: 2026-08-01
- 状態: **SPEC DONE**
- 親: CU-201 / U3b / VS-2

## 1. 結論

U3b moveの最小永続意味を、同じparent lane内にある一つのClipの`start`だけを変更する
`SetClipStart`へ固定する。

公開command shapeは次とする。

```rust
Command::SetClipStart {
    target: LayerId,
    old: RationalTime,
    new: RationalTime,
}
```

同時に`CommandKind::SetClipStart`、`PropertyId::ClipStart`、
`DocumentWriter::prepare_set_clip_start(target, new) -> Result<Option<Command>, CommandError>`を
追加する。`target`は現行TrackItemのstable identityである`LayerId`。別のClip IDを新設しない。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| `AUTHORITY` | [CU-201S](2026-08-01-cu-201-u3b-move-trim-snap-responsibility-split-decision.md)、[M3 U3b](../specs/M3-ui-integration.md)、[D2 command](../specs/M2-document-model.md) |
| `INTERNAL TARGET` | `Clip::start`、`Command` / `CommandKind` / `PropertyId`、`DocumentWriter`、`JournalEdit` v2 |
| `OWNER` | Clip startはDocument。move previewは後続PのHost Transient。single writerだけが確定値を書く |
| `WRITE ROUTE` | Writer prepare → typed command → journal-first D2 → history → published snapshot |
| `GAP` | Clip start用command / prepare / merge / replay oracleが0 |
| `RESOLUTION ROUTE` | 既存old/new対称command、merge key、`apply_macro`、JournalEdit v2を`REUSE`し、同一lane start変更へ`REDUCE` |
| `DISPOSITION` | `PASS`。次はM-C実装とT-S docsを別ownerで並列可能 |

## 3. 値と不変条件

適用成功時に変わるのは対象Clipの`start`だけである。次はbyte/値同一に保つ。

- parent track/groupとsibling index
- `LayerId`と全stable identity
- `duration`
- `time_map`全field
- source、ItemEnvelope、effect stack、transform、audio
- 他のTrackItemとその順序

現行validationどおり、負の`start`を許す。`new + duration`はchecked arithmeticで求め、
overflowを拒否し、半開終端が`composition.duration`を越える場合も拒否する。終端一致は成功。
Track内のClip overlapは現行Documentで許可済みなので、このcommandでも許す。

clamp、ripple、collision回避、lane変更、snapを行わない。これらをcommand payloadへ保存しない。

## 4. apply / inverse / merge

既存atomic command契約に合わせ、`old`はinverse payloadであってCAS preconditionではない。
`Command::apply`は現在値と`old`を比較せず、検証済み`new`を書き込む。
`inverse()`はold/newを交換した同じvariantを返す。

理由は次である。

- single writerとjournal前candidate preflightがlive applyとの間の同時変更を許さない
- 既存`SetProperty`等もoldをpreconditionにせず、決定済み値の対称書込みを使う
- stale drag generationの拒否は後続CU-201Pがlive snapshotへ再照合してからprepareする責任

`old == new`のraw command applyはidentity successとする。Writer prepareはlive Clipを解決した後、
`new == current start`なら`Ok(None)`を返し、journal / revision / Undoを増やさない。

raw commandで`old != current start`でもCAS拒否せず`new`を書き、inverseはpayload対称に`old`を
書き戻す。この場合のinverseはcommand payloadの逆写像であり、raw apply直前の任意の値を復元する
保証ではない。直前値をexactに復元するUndo保証は、同じsnapshotからcurrent startを`old`へ収集した
Writer prepare生成commandに限る。

同じgesture / target / `PropertyId::ClipStart`の複数commandは、既存merge規則どおり最初のoldと
最後のnewへ畳む。異なるtargetやpropertyとはmergeしない。通常product releaseは1 commandだけを渡す。

## 5. 拒否優先順

applyとWriter prepareは次の順で判定し、全失敗でDocument / writer stateを変えない。

1. `LayerId`が存在しない: 既存`LayerNotFound`
2. targetがGroupでClipでない: typed `TrackItemNotClip { layer }`
3. `new + duration`がoverflow: 既存`DocumentError::ClipIntervalOverflow`
4. endがcomposition durationを越える: 既存`DocumentError::ClipPastComposition`

負開始、overlap、old/current不一致は拒否理由ではない。全Document validateを後置して別errorを
先に返さず、この局所preflightと既存validateが同じ結果になることを試験する。

## 6. 永続互換

- Document schema、`WRITER_VERSION`、minimum reader versionは変更しない
- journal file/header versionは1のまま
- 新規writeは既存`JournalEdit.format_version = 2`の新しい正準command variantとして保存する
- v1 legacy command tag集合とv1 adapterは変更しない
- `stable_id_reservation()`は`None`
- JSON wire tag `SetClipStart`とfield `target` / `old` / `new`を固定する

cross-lane moveは本粒で実装しない。将来採択する場合は、既存Remove/Add TrackItemと
`SetClipStart`をatomic macroで組むかを別SPECで決める。曖昧な`MoveTrackItem` variantを今追加しない。

## 7. CU-201M-Cの必須oracle

1. startだけが変わり、duration / TimeMap / source / envelope / parent / order / identityが不変
2. 負開始、overlap、end == composition durationは成功
3. missing target → Group target → overflow → past compositionのtyped拒否とDocument全文不変
4. Writer prepareのsame-valueは`None`、raw same-value applyはidentity success
5. raw `old != current`はCAS拒否せず`new`を書き、inverseはpayloadの`old`を書く。Writer prepare生成commandのUndo / Redoは同じLayerIdと直前値 / `new`をexactに復元
6. 同gesture mergeはfirst old / last new、target違いは非merge
7. JournalEdit v2 JSON roundtripとWAL replayで同じstart
8. v1 tag fixture、Document version/min、stable id counter不変
9. randomな有効start列の全Undoで初期Document一致

## 8. 非目標とSTOP

- durationまたはTimeMapを変更する
- parent lane / sibling orderを変更する
- snap target、threshold、zoom、DPI、fps意味を決める
- clamp、ripple、collision avoidanceを追加する
- beat grid、user marker、transport、visible rangeを追加する
- Clip startを`SetProperty`へ押し込む
- public raw mutatorまたはjournal外のUI commandを追加する
- Document schema / journal versionを実装都合で上げる
