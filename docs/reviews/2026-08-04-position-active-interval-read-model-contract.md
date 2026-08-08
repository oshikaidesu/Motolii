# P04-C2 ACTIVE-INTERVAL Position read-model contract

状態: **決定 / IMPLEMENT（ACTIVE-INTERVAL read-only consumer sub-boundary のみ） / EXTERNAL_VISUAL_DEFERRED**

日付: 2026-08-04

## 1. 閉じる境界

既存 [P04-C2 decomposition/graph](../m3-executable-dispatch-map.md#6-ゴールへ至る依存ir) の
`ACTIVE-INTERVAL` node の read rule だけを閉じる。これは新 ticket ID ではなく、
`INTERP-COMMAND` より前の read-only node である。dispatch の `IMPLEMENT` / `DO` は
implementation ledger が所有し、本契約自身や相談結果が状態を認可しない。旧 compiler rejection は
[Stage transport Easing trigger consumer contract](2026-08-04-stage-transport-easing-trigger-consumer-contract.md)
が実在 mount/slot を選定したことで、この read-only consumer に限り解消した。親 `P04-C2` は、outgoing
`Interp` command と製品 Easing route が未成立なので `TARGET_MISSING` のまま完了にしない。

private `ProductApp` は、現在の `current_document`、primary `LayerId`、
`editor_playhead.current` から、Position の active interval を都度導出してよい。成功値は
正確に次だけを返す。

- `layer: LayerId`
- left / right の `KeyframeId`
- left / right の `RationalTime`
- left key の outgoing `Interp`

これは表示・将来の command admission の入力であって、selection、channel、popup state、
Document の新しい意味を推測するものではない。

```text
ProductApp private current_document + primary + editor_playhead.current
  -> existing ItemEnvelope transform.position
  -> DocParam::Keyframes(DocKeyframeTrack) の Vec2 key 隣接二件を strict interior で探索
  -> ActiveInterval { layer, left_id, left_t, right_id, right_t, left_interp }
```

## 2. exact read rule と ownership

`Document` は `LayerId`、`KeyframeId`、`RationalTime`、各 key の `Interp` の唯一の正本である。
`ProductApp` は ephemeral な read-only recomputation だけを所有する。cache、revision、
projection generation、serialized/read-model persistence を追加しない。

導出は次の順で fail-closed に行う。

1. primary `LayerId` があり、現行 Document がその ID の既存 `ItemEnvelope` を返すことを確認する。
2. その `transform.position` が `DocParam::Keyframes(track)` であり、`track` が
   `DocKeyframeTrack` であることを確認する。
3. track が少なくとも二 key を持ち、全 key value が `DocValue::Vec2`、times が既存 validation
   の strict ascending であることを確認する。
4. 連続する `left`, `right` をただ一組、`left.t < editor_playhead.current < right.t` で探す。
5. `layer`、両 key の ID/time、`left.interp` をそのまま返す。

key endpoint は interval に含めない。playhead が最初の key より前、最後の key より後、同じ
時刻、または malformed input なら result はない。`Const`、`Vec2Axes`、`Data`、`LookAt`、`Follow`、
one/zero key、primary 不在、missing envelope、unsupported value はすべて `None` であり、別 channel
や selection を代入してはならない。

## 3. known-implementation adoption preflight

```text
MECHANISM CLASS: DocKeyframeTrack interval-centered read model with the left key's outgoing interpolation
KNOWN IMPLEMENTATION SEARCH: docs/m3-executable-dispatch-map.md ACTIVE-INTERVAL graph;
  docs/reviews/2026-07-22-m3-native-easing-popup-acceptance.md; DocKeyframeTrack;
  ui/motolii-web/src/candidates/EasingTriggerCandidate.jsx; ProductApp editor playhead
CANDIDATES: existing sorted DocKeyframeTrack/key IDs/times/Interp; existing ProductApp private
  current_document/primary/editor_playhead.current; React trigger's activeInterval disabled state
ADOPTION ROUTE: PATTERN for interval-centered and left-key-outgoing semantics; REUSE for source
  identities and ProductApp inputs
REJECTED CANDIDATES: dependency/vendor/port = no need; spike import = isolated evidence only;
  generic popup/channel model = broader owner; React trigger = not a product semantic owner
THIN MOTOLII SEAM: one private ProductApp derivation over existing DocParam::Keyframes(DocKeyframeTrack)
  whose values are DocValue::Vec2
THIN MOTOLII RESIDUAL: exact Position admission and strict-interior negative cases
RETIREMENT: NONE
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

Fable blind review returned `ACCEPT_ISOLATED` as a read-only consultation. It is not authority
for the graph, state, owner, or implementation.

The React `EasingTriggerCandidate` is the selected existing presentation consumer, but neither
supplies interval identity nor authorizes input intent, popup, or product semantic ownership. Its
private output projection and direct import are closed only by the
[Stage transport consumer contract](2026-08-04-stage-transport-easing-trigger-consumer-contract.md).

## 4. implementation boundary and oracle

The admitted implementation allowlist and Stage transport publication lifecycle are exactly those
in the [Stage transport consumer contract](2026-08-04-stage-transport-easing-trigger-consumer-contract.md).
The derivation remains private to `ProductApp`; no public/general Timeline API is introduced.

`PRIMARY_ORACLE` is exact identity/time and the left outgoing `Interp` for a strict interior
Position interval. Focused negative cases prove no result for every rejected case in §2. Before
and after each derivation, serialized `Document` equality is required. The private pure helper
takes read-only inputs and has no queue or projection-mutation access; it must not write
Document, journal, history, Undo, queue, or projection generation. Queue/generation counters
are therefore not a helper oracle.

`REPO_LANES` for the admitted consumer are the focused Rust/Stage Host and React guard lanes
fixed by that contract, plus `git diff --check` and `./scripts/check-docs.sh`. The prior focused
test passed but clippy rejected its unused production helper; it remains historical evidence, not
implementation evidence. `EXTERNAL_GATES`: human visual verification is deferred; no test
closes native popup/window, preset, settings, or accessibility acceptance.

## 5. explicit non-goals and next handoff

This node does not create an outgoing `Interp` command, Host input codec, React intent,
popup/native window, presets/settings, Inspector Add Position Key, generalized channel model,
or complete `P04-C2` / `U4b-1`. The private Stage transport output projection is its sole exception.
It does not alter the separately `WAIT_TARGET` normal Inspector Position row route.

This selected consumer may emit only existing graph value `active_interval_identity` for
`INTERP-COMMAND`. That later node remains blocked until a
separately closed command owner, write route, consumer, and one-command/one-Undo oracle exist.
