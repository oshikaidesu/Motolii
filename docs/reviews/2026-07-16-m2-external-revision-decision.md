# M2 external project revision decision (2026-07-16)

Status: **Decision — D1n after D1m**. This closes the save-precondition portion of implementation guard 11 beyond provider-name warning without claiming impossible distributed locking.

## Finding and known technology

`detect_cloud_sync()` only recognizes suspicious path names. It neither monitors nor prevents a non-cooperating sync client/editor from replacing the main document, journal, or catalog while Motolii has the project open. D1m's OS lock coordinates Motolii processes but advisory locks do not bind Dropbox/iCloud/SMB peers.

- HTTP `If-Match` is the standard lost-update pattern: mutate only if the current representation still matches the revision observed by the client ([RFC 9110 §13.1.1](https://www.rfc-editor.org/rfc/rfc9110.html#name-if-match)).
- SQLite warns that broken/missing locks and copying or synchronizing a live database separately from its journal can corrupt state ([SQLite How To Corrupt](https://sqlite.org/howtocorrupt.html), [SQLite locking](https://sqlite.org/lockingv3.html)).

Motolii adopts an in-session optimistic revision precondition in addition to D1m's cooperative lock. Filesystem watch events and provider detection are hints only.

## Decision

### 1. `ProjectSession` owns an observed revision

After D1m acquire/recovery completes, the session captures:

```text
ProjectRevision {
  document: Absent | SHA-256(exact bounded document bytes),
  journal: Absent | SHA-256(exact bounded journal bytes),
  catalog: Absent | SHA-256(exact bounded catalog bytes),
  generations: Map<GenerationId, SHA-256(exact bounded generation bytes)>,
}
```

SHA-256 here is transient comparison state, not a Document/journal field. `document_fingerprint()` (FNV64 over serialized semantic data) is not reused: it ignores exact byte representation and is not collision-resistant enough for a lost-update precondition.

The journal digest covers **all file bytes**, including any invalid/ignored tail; `valid_bytes` and `file_len` therefore cannot change unnoticed. It is calculated over the same bytes already read by the existing bounded D1d scan, so D1n does not substitute a tail-only read or add a second journal read. D1d still derives the effective tip generation salt by walking Checkpoint records; neither header salt nor a “last committed record” is used as revision identity. Catalog/header/generation cross-reference validation still runs. D1n does not reinterpret journal records.

Generation digests cover every snapshot referenced by the captured catalog. They are checked only for operations whose read/write set includes generations (checkpoint, rotate, pin/unpin, recovery/migration), not for an edit-only journal append. Main/journal/catalog are checked before every mutation because each participates in project ownership/recovery. All revision reads use the session's same `ResourceLimits`; limit errors retain their existing structured type and cause no write.

### 2. Every mutation checks the revision first

Immediately before journal append, checkpoint/main replace, migration, or other project-family mutation, `&mut ProjectSession` re-reads the current bounded state in that operation's closed read/write set and compares it with its stored revision.

- exact match: perform the existing mutation/durability sequence, then re-read the durable result and replace the session revision;
- mismatch: return typed `ExternalProjectChanged { component, expected, observed }` and perform no mutation in that operation;
- malformed/oversized/unreadable current state: preserve the structured D1d/D1c error and perform no mutation;
- current file disappears/appears relative to `Absent`: mismatch, not automatic recreation/adoption.

An edit already present in memory/Undo remains local after conflict. Motolii does not auto-merge, auto-reload, overwrite, truncate, or attach the external family. Save As/conflict UI is later work.

The public raw-path mutations are already closed by D1m; D1n must not add a bypass that lacks the session revision.

### 3. Watchers and provider detection are diagnostic

M3 may watch paths to surface conflict earlier, but save correctness never depends on event delivery, event order, mtime, inode/file ID, or provider-name detection. Same-length/same-mtime document replacement must still be caught by exact-byte SHA-256.

### 4. Race limitation is explicit

No portable local-filesystem compare-and-swap can force a non-cooperating cloud client to honor Motolii's precondition. A peer may change a path after the last comparison. D1n narrows the window by comparing immediately before mutation and verifies the durable result afterward; a post-write mismatch is typed `ExternalProjectChangedAfterWrite` and retains all available recovery artifacts. If the peer writes between the comparison and Motolii's atomic replace and Motolii wins, the peer's change can be overwritten without post-write detection; D1n is explicitly **not CAS**. It must not claim that arbitrary distributed races are impossible or attempt an OS/vendor-specific coordinator.

This limitation is preferable to a false “cloud safe” guarantee. Cooperative Motolii races are prevented by D1m; detected non-cooperative changes are never knowingly overwritten by D1n.

## D1n completion judgment

1. Same-size/same-mtime external main replacement causes `ExternalProjectChanged` and byte-for-byte no mutation of main/sidecar.
2. External journal replacement (including same-length/same-tail middle-byte change or ignored-tail change), catalog replacement, generation replacement for a generation-touching operation, appearance, and disappearance each conflict before local mutation; errors preserve their component/type.
3. Two successive local edits/checkpoints update the stored revision and succeed without a false conflict.
4. Fault injection at each precondition/read boundary proves **precondition mismatch/malformed/limit-error paths** have no writes. A separate post-write-race test proves detectable peer-wins replacement returns `ExternalProjectChangedAfterWrite` and recovery artifacts remain; it does not claim to detect the documented Motolii-wins race.
5. Watcher/provider hints may be absent and all conflict tests still pass.
6. D1d corruption/recovery meanings, protected semantic expectations, and D1m lock/isolation tests remain unchanged; `cargo test --workspace` is green.
7. Revision hashing adds no second journal read: it consumes D1d scan bytes. Generation hashing occurs only for generation-touching operations; watcher polling is never used as a substitute.

## Non-goals

- Distributed locking or automatic merge.
- Treating mtime/file ID/watch events as revision authority.
- Persisting SHA-256 revision state in Document.
- UI conflict resolution, Save As, or read-only fallback.
- A guarantee against the documented non-cooperating peer-write/Motolii-replace TOCTOU window.
