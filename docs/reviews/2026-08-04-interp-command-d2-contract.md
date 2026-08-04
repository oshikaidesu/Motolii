# P04-C2 INTERP-COMMAND Position outgoing interpolation D2 contract

状態: **決定 / SPEC DONE / IMPLEMENT_READY（D2 command sub-boundary のみ）**

日付: 2026-08-04

## 1. 閉じる境界

`P04-C2` の既存 `INTERP-COMMAND` graph node を、Position key の左 key が持つ outgoing
`motolii_eval::Interp` だけを durable に差し替える D2 command として閉じる。これは active interval
read-only projection の次 edge であり、React click、Host intent/codec、popup、input、focus、preset、
settings、Easing UI は閉じない。

| 項目 | 確定 |
|---|---|
| AUTHORITY | [M3 U4b](../specs/M3-ui-integration.md)、[P04-C2 ACTIVE-INTERVAL](2026-08-04-position-active-interval-read-model-contract.md)、[Stage transport acceptance](2026-08-04-stage-transport-easing-trigger-implementation-acceptance.md) |
| TARGET | existing `Document` / `ItemEnvelope::transform.position` / `DocParam::Keyframes(DocKeyframeTrack)` の一つの existing `KeyframeId` outgoing `Interp` |
| OWNER | key ID、time、value、`Interp`、Document version は `Document`。command admission/apply/inverse は existing D2 `Command` / `DocumentWriter` / journal-first runtime |
| WRITE ROUTE | future typed producer → `DocumentWriter` prepare → `Command` → journal-first D2 → history → published snapshot |
| GAP | `AddPositionKey` はkeyを導入するだけで既存keyの `interp` を更新しない。current enum にinterpolation-only variant/prepare/oracleはない |
| RESOLUTION | existing dedicated `Command` variant + `CommandKind`/`PropertyId`/inverse + `JournalEdit` v2 + `UndoHistory` をREUSEし、Position/key admissionだけをthin residualとして追加 |
| DISPOSITION | `INTERP-COMMAND` D2 contractはIMPLEMENT_READY。producer/Host/popupは別 `WAIT_TARGET` のまま |

この文書は command identity と admission を決めるだけで、通常製品から command を発行する権限を作らない。

## 2. exact command identity and payload

既存 `SetClipStart` / `TrimClip*` の immutable old/new payload command 命名に合わせ、次の dedicated
variant を採択する。`SetProperty` への押し込み、generic channel、time/valueを含む track replacement はしない。

```rust
Command::SetPositionKeyInterp {
    target: LayerId,
    key: KeyframeId,
    old: Interp,
    new: Interp,
}
```

- `target` は envelope owner の existing `LayerId`、`key` は existing document-local `KeyframeId` である。
- `old` / `new` は `motolii_eval::Interp` をそのまま保持する。新しい easing enum、curve payload、preset
  ID、channel ID、display name、playhead、left/right time/valueは durable payload に含めない。
- `CommandKind::SetPositionKeyInterp` と `PropertyId::PositionKeyInterp(key)` を dedicated にする。merge identityは
  exactに `kind=SetPositionKeyInterp`、`target_stable_id=target.get()`、`property=PositionKeyInterp(key)` である。
  same gesture + same layer + same key だけを first-old / last-new に畳む。別key、layer、Add Position Key、
  Position value editとはmergeしない。
- `stable_id_reservation()` は `None`。これはIDを導入・削除せず、Undo/Redo/replayで既存IDを再採番しない。

## 3. admission, apply, inverse

本SPECは exact public API 変更を先に閉じる。`command.rs` にprivate prepare
`prepare_set_position_key_interp(doc, target, key, new) -> Result<Option<Command>, CommandError>` を置き、
`lib.rs` のexisting-style public wrapper を次に固定する。

```rust
DocumentWriter::prepare_set_position_key_interp(
    target: LayerId,
    key: KeyframeId,
    new: Interp,
) -> Result<Option<Command>, CommandError>
```

prepareは current Document を一度だけ読み、成功時に current `interp` を `old` に写して
`Some(Command::SetPositionKeyInterp { ... })` を返す。newが
currentと等しい場合は existing `SetClipStart` / trim prepare precedent に従い `Ok(None)` とし、journal、revision、
Undo、projection publishを増やさない。raw apply の same old/new はidentity successである。

prepare と raw apply は次の順で fail-closed にする。失敗時は Document、journal、history、queue、projection
generation を変更しない。

1. `target` の envelope がない → `CommandError::LayerNotFound(layer)`
2. `transform.position` が `DocParam::Keyframes` でない →
   `CommandError::PositionKeyInterpSourceUnsupported { layer }`
3. track内に `DocValue::Vec2` でない keyがある →
   `CommandError::PositionKeyInterpValueTypeMismatch { layer }`
4. `key` がtrackにない（empty trackを含む） →
   `CommandError::PositionKeyNotFound { layer, key_id }`
5. `old` 又は `new` が `crate::doc_keyframe::validate_interp` を通らない →
   `CommandError::PositionKeyInterpInvalid { layer, key_id, source: DocKeyframeError }`
6. raw payload の `old` がcurrentと一致しない →
   `CommandError::PositionKeyInterpPayloadMismatch { layer, key_id }`

`old` のCAS扱いはこの command に限る。既存 `SetClipStart` / trim の raw old/new はnon-CASだが、ここでは
stable key identityを誤った interval/obsolete command に適用しないことが D2 admission の意味である。producer
prepareが生成したcommandならcurrent==oldが成立する。future stale/queue policyをこの文書で追加しない。

active interval の「左 key」制約は producer/read-model owner の意味であり、D2 admissionへplayhead又は
right-neighborを入れない。existing Position keyなら末尾keyも許可する。apply は target trackをpreflightしてから、
同じ key の `interp` だけを `new` に置換する。key ID、time、
`DocValue::Vec2`、key順序、他keyのinterp、envelope、layer name、stable ID counter、schemaは不変。`Command::apply`
はmutation前に `crate::doc_keyframe::validate_interp(old/new)` とexisting track/value/key/CAS preflightを完了し、
新track/full validatorを追加しない。journal replayはexisting `apply_decoded_edit` のpost-apply
`Document::validate()` を維持する。`crate::doc_keyframe::validate_interp` はinternal reuseだけでありreexportしない。
inverse は同variantの `old` / `new` を厳密にswapする。

## 4. durability and compatibility

- 1 command = 1 existing D2 Undo macro。Undo/Redo は同じ `LayerId`/`KeyframeId` と payload `Interp` を復元する。
- wire tag/fieldsはexactに `SetPositionKeyInterp` / `target` / `key` / `old` / `new`。existing
  `JournalEdit::FORMAT_VERSION == 2` の direct serde canonical `Command` envelopeに保存し、JSON roundtrip と
  WAL replay はlive applyと同じ `Interp` を得る。
- 新Command tagは追加するが、既存tagを変更しない。旧readerがunknown v2 tagを読めるとは主張しない。
  unknown v2 payloadはexisting `InvalidEditPayload` とfallbackでtyped rejectされる。v1 legacy tag集合/adapter、
  journal file/header versionは不変である。version bumpが必要と判明した時はIMPLEMENTをSTOPする。
- Document schema、writer/min reader version、plugin contractを変更しない。
- no-op prepareは`None`、rejected admissionはjournal edit / Undo / queue / projection generation 0。raw identity
  applyはDocument bytes不変のsuccessである。

## 5. known implementation adoption preflight

```text
MECHANISM CLASS: stable-key identity addressed immutable interpolation replacement with exact inverse and journal replay
KNOWN IMPLEMENTATION SEARCH: crates/motolii-doc/src/command.rs AddPositionKey/SetClipStart/TrimClip*;
  crates/motolii-doc/src/position_key_prepare.rs; crates/motolii-doc/src/journal/replay.rs;
  crates/motolii-doc/src/undo.rs; docs/reviews/2026-08-01-cu-201m-s-clip-start-command-contract-decision.md;
  docs/reviews/2026-08-01-cu-201t-s-clip-trim-timemap-contract-decision.md
CANDIDATES: existing Command dedicated variants + kind/property/inverse/stable-id reservation; existing
  DocKeyframeTrack::keys/validate and motolii_eval::Interp; JournalEdit v2; UndoHistory
ADOPTION ROUTE: REUSE
REJECTED CANDIDATES: SetProperty :: would replace whole DocParam and loses exact key admission; AddPositionKey :: introduces ID;
  generic channel command :: owner/payload not present; UI/Host intent :: no producer target
THIN MOTOLII SEAM: one Position-key-only Command/prepare/apply/inverse using existing D2 and existing Interp validation
THIN MOTOLII RESIDUAL: key-exists/current-interp admission and interpolation-only invariants
RETIREMENT: NONE; no old interpolation writer exists
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

No dependency, framework, Document field, codec, or general channel is introduced. This is product semantic ownership
inside the existing D2 command route, not a new general mechanism.

## 6. implementation allowlist and oracles

Admitted future implementation scope is only the dedicated D2 command route and its focused tests:

- `crates/motolii-doc/src/command.rs`（variant/kind/property/prepare/apply/inverse）、
  `crates/motolii-doc/src/lib.rs`（exact `DocumentWriter::prepare_set_position_key_interp` wrapper）、
  `crates/motolii-doc/src/undo.rs`（`merge_pair` のdedicated arm）、existing journal command coverage、focused
  `motolii-doc` tests necessary for command/inverse/replay。
- `position_key_prepare.rs` は AddPositionKey の検索precedentだけであり、このproduction mutation allowlistから外す。
- dedicated public `Command` variant と上記 existing-style public `DocumentWriter` prepare wrapperだけを許可する。
  別raw mutator、generic API、UI、Host、browser、React、popup、input、ProductApp consumer、新dependency、
  schema/version fileは許可しない。

`PRIMARY_ORACLE`:

1. a strict-interior Position track changes only selected left key `interp`; all IDs/times/values/other keys and serialized
   Document bytes except the selected interp are exact;
2. inverse/Undo/Redo restores exact old/new values and no stable ID allocation occurs;
3. same gesture/same layer+keyの2 commandはfirst old/last newにmergeし、Undo一回でgesture前を復元する。
   different layer又はkeyはmergeせず別commandのまま;
4. `JournalEdit` v2 JSON/WAL replay matches live apply;
5. missing layer, non-keyframe/unsupported/non-Vec2 Position, empty track, missing key, invalid `Interp`, old mismatch,
   and same-value prepare each preserve Document bytes and have the fixed typed outcome.

`REPO_LANES`: focused `motolii-doc` command/Writer/journal/Undo tests, `cargo fmt --check`, strict clippy, and
`git diff --check`. `EXTERNAL_GATES`: none for D2 semantics. Native popup/input/focus/accessibility remain future M3
external/product gates and cannot be replaced by these tests.

## 7. non-goals and next edge

This contract does not select or implement a Host intent, React handler, button enablement beyond existing read-only state,
native popup/window, curve editor, drag preview, presets, settings, playback, Auto Key, Add Position Key entry, generic
channel model, or a public API beyond the explicit `Command` variant and `DocumentWriter` wrapper above. It does not complete
parent `P04-C2` / `U4b-1`.

After this D2 command is implemented and independently accepted, the next edge is a separate producer/admission contract
from the existing Easing trigger to exactly one typed command. Its target, click/focus/cancel semantics, and external visual
gate remain `WAIT_TARGET` until separately closed.
