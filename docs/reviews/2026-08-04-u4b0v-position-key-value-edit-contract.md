# U4b-0V Position key value edit contract

状態: **DO / CLOSED CONTRACT**

日付: 2026-08-04

## 1. outcome と一契約境界

[Motion Authoring Loop](2026-08-04-outcome-spine-autonomous-gap-research-decision.md#5-m3への適用)の
`key追加 -> 別時刻の値変更`を、明示的Add Position Keyとexact on-key value editの二操作で接続する。
通常製品Inspectorは既存product-owned React Position行を直接使い、Rust-owned current primary/playheadに
exact一致するVec2 keyのvalueだけをX/Y操作で変更する。ID、time、outgoing interpolation、他key、stable-ID
counterは不変とする。

Constとoff-keyは値編集を受理しない。別時刻では先に既存`Add Position Key`を明示実行し、その後同じ時刻の
keyを編集する。global Auto Key、暗黙key挿入、曲線全置換は作らない。

```text
AUTHORITY: historical D2/selection/Timeline lineage recovery §4.1 + U4b-0 + CU-0A08ITIB
INTERNAL TARGET: exact current-playhead Vec2 Position key -> key-local value writer
OWNER: InspectorCandidate presentation / Inspector Host admission / ProductApp primary+playhead /
  DocumentEditRuntime single writer / DocumentWriter dedicated prepare
WRITE ROUTE: transient cloned preview -> one SetPositionKeyValue -> durable commit / Undo / JournalEdit v2 /
  full publish / save-reopen
GAP: dedicated key-value command、gesture projection/admission、product queue consumerが未実装
RESOLUTION ROUTE: PATTERN existing SetPositionKeyInterp CAS + opacity gesture preview/terminal + Add key Host route
DISPOSITION: one D2 contract and one product connection wave; human visual remains M3-final
```

## 2. 既知実装採択

```text
MECHANISM CLASS: exact on-key Position Vec2 value edit with transient preview and one durable terminal
KNOWN IMPLEMENTATION SEARCH: historical lineage §4.1; Command/Undo/JournalEdit v2; SetPositionKeyInterp;
  AddPositionKey; opacity Inspector ScrubControl/Host gesture/preview/commit; ProductApp current playhead
CANDIDATES: A) dedicated key-local SetPositionKeyValue; B) unconstrained SetProperty whole-curve replacement;
  C) off-key Auto Key; D) generic keyframe API; E) second Inspector/native UI
ADOPTION ROUTE: PATTERN A from SetPositionKeyInterp and opacity gesture; REUSE existing Host/queue/writer/publish
REJECTED CANDIDATES: B lacks live key-local CAS; C contradicts explicit Add Key authority; D/E broaden owners
THIN MOTOLII SEAM: X/Y controls -> private gesture -> current Rust key resolution -> key-local prepare/preview/commit
THIN MOTOLII RESIDUAL: exact-time admission, Vec2 finite validation, old/new key-value CAS, stale/cancel handling
RETIREMENT: read-only `animated` placeholder retires only when an exact current key is projected; explicit Add stays
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN beyond sections 3-5
```

## 3. D2 command

`Command::SetPositionKeyValue { target, key, old, new }`を`SetPositionKeyInterp`と同じ専用patternで追加する。
`old/new`はfinite `[f64; 2]`、`CommandKind::SetPositionKeyValue`、
`PropertyId::PositionKeyValue(KeyframeId)`とする。inverseはold/newだけをswapする。

`DocumentWriter::prepare_set_position_key_value(target, key, new)`はlive Positionがnon-empty Vec2
`Keyframes`で、keyが存在し、newがfiniteであることを確認する。same-valueは`Ok(None)`。apply/replayは
mutation前にcurrent key value=`old`、全key valueがVec2、old/new finiteを再検証し、clone上で対象keyの
valueだけを置換してexisting Document validation後にswapする。key ID/time/interp、他key、counterを変更しない。
payload mismatchはtyped errorで無変更拒否する。

既存JournalEdit v2、Undo macro、WAL replayを再利用し、Document/journal versionを変えない。version bump、
public generic key API、`SetProperty`全体のCAS意味変更が必要なら施工を止める。

## 4. Inspector / Host / product route

1. Inspector read modelはConst X/Yに加え、exact current-playhead Vec2 keyだけを
   `{kind:"key",x,y}`としてprojectする。off-keyは`animated`のまま。
2. 既存Position行でX/Yを既存`ScrubControl`文法へ接続する。新component/frameworkを作らない。
3. private messageはPosition専用`start/update/commit/cancel`、session/sequence、axis、finite valueだけを運ぶ。
   target、playhead、key ID、old value、revisionをJSへ持たせない。
4. HostはboundedなPosition専用gesture inboxを所有し、既存opacity sessionと混ぜない。ProductAppがstart時の
   current primary/playheadからexact key identityとbaselineを再導出し、updateはcloned previewだけをrenderする。
5. commit時にもcurrent primary/playhead、key identity、baseline/live curveを再照合し、一件のprivate requestを
   DocumentEditQueueへ送る。cancel、stale、Const、off-key、unsupportedはDocument mutation/publish 0でbaselineへ戻す。
6. successful terminalだけがexisting durable commit/full publishへ入り、Undo/Redo/save/reopenを一件で閉じる。

## 5. allowlist と oracle

`ALLOWLIST production`: `crates/motolii-doc/src/command.rs`, `lib.rs`, `undo.rs`のmerge arm、
`crates/motolii-ui/src/diagnostic_projection.rs`のlabel arm、`inspector_host_runtime.rs`,
`document_edit_runtime.rs`, `product_runtime.rs`, product-owned Inspector candidate/main/codec、既存provenance、
通常Host buildのgenerated assetと既存Rust embed/path表。focused testsだけを追加できる。

`PRIMARY_ORACLE`: exact current keyでX/Y updateはDocument不変のlatest preview、commitは対象key valueだけを変更し
one journal/Undo/publish、Undoでold、Redo/save-reopenでnewへ一致する。ID/time/interp/other keys/counterは全段不変。

`NEGATIVE_ORACLES`: Const、off-key、missing/mismatched primary、playhead/curve/key変更、non-Vec2/empty/unsupported、
non-finite/same value、cancel、malformed/replay/reorder/full inbox、暗黙key追加、preview mutation、multi-commitは0 write。

`REPO_LANES`: focused motolii-doc command/undo/journal tests; focused motolii-ui Host/runtime tests;
Inspector codec/ownership guards; Host build/check; affected crate tests; strict clippy; fmt; docs; diff-check。

`EXTERNAL_GATES`: fresh different-family read-only diff review。native/WebView visual、focus、keyboard、a11y、
recognizabilityはM3-final `EXTERNAL_GATE_PENDING`へ集約する。

`NON-GOALS`: Auto Key、Const/off-key edit、evaluated-value bake、curve flatten、Timeline redesign/dynamic marker width、
key time/ID/interp editing、multi-key Graph View、Easing変更、public keyframe API、schema/version、second writer/UI owner。
