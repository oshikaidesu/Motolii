# M2 project sidecar identity / session ownership decision (2026-07-16)

Status: **Implementation landed** (D1m, branch `cursor/d1m-project-session`). D1d's corruption recovery remains valid on project-scoped sidecars with inter-process session ownership.

> **2026-07-18 supplement ([VSM-A0S](2026-07-17-vism-a0s-contract-catalog-spec.md) alignment)**: The save/open ownership rows below are amended so D1m implementation cannot invent a second product open path or keep root-public path mutation. Code is not updated by this supplement.

## Finding

The current `motolii_dir_for_document()` maps every project file in the same parent directory to the same `.motolii` directory. Consequently `journal.wal`, `catalog.json`, `generations/`, and restore markers collide across otherwise unrelated projects. In addition, the public open/save path does not hold an inter-process lock, so two Motolii processes can append or checkpoint the same journal concurrently.

Both faults are below the M3 UI boundary. A single-writer editing thread only serializes writers inside one process and cannot repair either fault.

## Known-technology disposition

- SQLite gives each database its own journal/WAL family and permits only one writer at a time. Its corruption guidance explicitly treats broken filesystem locking and concurrent access through different path aliases as unsafe ([SQLite locking](https://sqlite.org/lockingv3.html), [How To Corrupt An SQLite Database File](https://sqlite.org/howtocorrupt.html)).
- Rust's standard `File::try_lock` provides a non-blocking exclusive filesystem lock and releases it when the handle closes. It maps to the platform lock primitive, including `LockFileEx` on Windows ([Rust `File`](https://doc.rust-lang.org/std/fs/struct.File.html), [Windows `LockFileEx`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-lockfileex)).

The adopted design follows those established boundaries: one sidecar family per project identity, one exclusive read-write session owner, OS lock state as authority, and typed refusal instead of guessing.

## Decision

### 1. Sidecar family is project-scoped

For canonical document path `<parent>/<file-name>`, the path is fixed as follows.

```text
project sidecar directory = <parent>/<file-name>.motolii/
session lock              = <parent>/<file-name>.motolii.lock
journal                   = <parent>/<file-name>.motolii/journal.wal
catalog                   = <parent>/<file-name>.motolii/catalog.json
generations               = <parent>/<file-name>.motolii/generations/
restore/corrupt markers   = <parent>/<file-name>.motolii/<existing marker names>
```

`<file-name>.motolii` and `<file-name>.motolii.lock` mean appending the platform-native suffix to the `OsString`; they are not lossy UTF-8 conversions, percent encodings, hashes, or stem replacements. Therefore `a.json` and `b.json` map injectively to different siblings, including for non-UTF Unix names. The lock is a sibling rather than a member of the sidecar directory so an explicitly migrated family can be atomically installed by directory rename while the lock handle remains held. The platform filesystem's own case/normalization identity applies after parent canonicalization.

The sidecar directory is not part of `Document` JSON and does not add a permanent schema field. **Save As and rename APIs are outside D1m**: their later specification must acquire source/target sessions in a fixed order, create/move the target sidecar family, and never silently reuse history. D1m only reserves that rule; Composer must not add those APIs. A manual external rename may lose recovery history but must not attach another project's history.

The old parent-shared `<parent>/.motolii/` journal layout is legacy input only. **D1m never guesses or automatically adopts its owner**: the old catalog has no durable link to a document path, and equal document fingerprints can still belong to copies. A legacy or new **journal family exists** iff at least one of this closed set exists at its layout root: `journal.wal`, `catalog.json`, `generations/`, `restore_attempted.json`, or `journal.wal.corrupt-*`. The shared `.motolii/media` directory, unknown entries, a sibling lock file, and an empty directory do not establish a journal family.

An explicit `ProjectSession::migrate_legacy_sidecar()` operation is the only adoption route. Calling it is the caller's ownership confirmation. While holding the sibling lock, it copies only the known journal family into sibling staging `<file-name>.motolii.importing`, verifies header/catalog/generation cross-references and recovery there, fsyncs it, and atomically renames the verified staging directory to `<file-name>.motolii` followed by parent-directory fsync. It does not copy the shared asset cache `.motolii/media`, delete/truncate the legacy source, or invent ownership for another project. Unknown legacy entries remain untouched on disk; their entry names are reported only through the in-process diagnostic report defined in [Legacy sidecar migration diagnostic report (2026-07-18)](#legacy-sidecar-migration-diagnostic-report-2026-07-18) and are never persisted.

An incomplete staging directory is never treated as an active family. Ordinary open returns typed `IncompleteLegacyMigration` without changing it. A new explicit migration call is the only repair authority: before retry it **must** atomically rename the old staging directory to `<file-name>.motolii.importing.failed-<UUIDv4>` and fsync the parent, then create/verify a fresh exact `.importing` staging copy from the untouched legacy source. If quarantine rename fails, retry stops with a typed I/O error. It must not delete or merge partial bytes, delete the legacy source, or silently prefer a partial/failing destination. Only the exact `<file-name>.motolii.importing` path is active staging; `failed-*` entries are diagnostics and never candidates.

The final destination path has its own closed occupancy rule, independent of the family predicate:

- absent: ordinary new open or atomic legacy install may create it;
- existing empty directory: ordinary new open may use it; an explicit migration must remove this empty directory immediately before atomic install (the caller has authorized migration);
- directory containing only unknown entries, or any non-directory object: typed `DestinationPathOccupied`, filesystem unchanged;
- directory containing a journal family: classify it as valid/invalid in the state table below.

No platform-dependent rename-over-existing behavior is used. Migration never merges staging into an occupied final directory.

The state table is closed:

| Legacy family | Final path/family | Staging | Ordinary open | Explicit legacy migration |
|---|---|---|---|---|
| no | absent/empty | no | open main as a new/no-history session | `NoLegacySidecar` |
| yes | absent/empty | no | `LegacySidecarRequiresExplicitMigration`, FS unchanged | if empty remove it, then copy→verify→atomic install → `Ok(Report { disposition: Installed, untouched_legacy_entries })` |
| yes/no | absent/empty | yes | `IncompleteLegacyMigration`, FS unchanged | quarantine staging, then retry only if legacy=yes; otherwise typed reject (`Err`, no report) |
| yes/no | unknown-only directory or non-directory | any | `DestinationPathOccupied`, FS unchanged | same typed reject (`Err`, no report); no overwrite/merge |
| yes/no | valid family | any | use final; legacy/staging are not auto-merged | idempotent no-op only if final verifies → `Ok(Report { disposition: AlreadyValid, untouched_legacy_entries })`; otherwise typed conflict (`Err`, no report) |
| yes/no | invalid family | any | typed `InvalidProjectSidecar`, no fallback | typed conflict (`Err`, no report); no overwrite/merge |

“Valid” means the existing D1d bounded scan, catalog/header/generation cross-reference, and recovery checks succeed. A lock file or media-only shared directory does not change any row.

### 2. A read-write project session owns an OS lock

`ProjectSession::acquire(path)` canonicalizes the project identity, opens/creates only the sibling lock file, and calls non-blocking exclusive `try_lock` **before** legacy migration, recovery, sidecar-directory creation, or project mutation. `ProjectSession::open(path)` is the acquire-plus-recover convenience and returns both the guard and `OpenProjectOutcome`. The lock handle is retained by the non-`Clone` guard through journal append, checkpoint, migration, and save; mutation methods require `&mut self`. `WouldBlock` becomes typed `ProjectAlreadyOpen`; the UI thread is never made to wait.

PID/start-time metadata may be written for diagnosis only. It is not lock authority. The implementation must not delete a lock file or steal ownership based on PID, timestamps, or a “stale” heuristic. Process termination/handle close releases the OS lock.

Canonicalization must collapse existing-file symlink aliases. For a not-yet-created project, canonicalize the existing parent and append the requested file name. If canonicalization or locking is unsupported/fails, read-write open fails closed with a structured I/O/lock error. A read-only fallback is a separate future decision and must not be invented in D1m.

### 3. Public mutation requires the session capability

All production mutations of the project file or its journal family require `&mut ProjectSession`: journal edit, checkpoint/save, recovery that writes markers/quarantines, document-file migration, and legacy-sidecar migration. D1m removes or makes crate-private the root-public raw-path mutation/open exports `save_project_with_journal`, `save_document`, `save_document_with_options`, `migrate_document_file`, `migrate_document_file_with_limits`, `open_project`, `open_project_with_limits`, `open_project_fs`, and `recover_project`; their internal implementations may remain behind session methods. `WalSession`, `commit_edit`, `checkpoint`, catalog save, recovery mutation, `*_fs`, and fault-injection helpers are crate-private or test-only and must not be root-public bypasses.

The capability core for project open is `ProjectSession::open` (acquire + recover). The sole catalog-required **product** open façade is `open_project_resolved` ([VSM-A0S §7](2026-07-17-vism-a0s-contract-catalog-spec.md)): it wraps `ProjectSession::open`, retains the non-`Clone` exclusive lock guard (`ProjectSession`), and returns `ResolvedOpenProjectOutcome { session, recovered, plugins }`. No second catalog-required product open entry or tuple-split bypass may be added.

`load_document*` and serde unknown payload preservation remain public and catalog-independent per A0S; only **path mutation** moves behind session or crate-private. Pure serialization to bytes and genuinely read-only load/inspect APIs may remain public.

Repair and recovery **mutation** require `ProjectSession::acquire` or `open` first. `recover_project` is crate-private/test-only and must not be a session-less product or repair entry.

The in-process rule remains unchanged: only the editing thread owns `&mut ProjectSession` and mutates `Document`; workers read `Arc<Document>`. The OS lock adds inter-process ownership and does not replace the single-writer rule.

Cloud-sync software can ignore advisory locks. D1m therefore prevents cooperating Motolii processes and path aliases from racing, but does not claim to solve remote synchronization. The existing external-change warning remains; compare-before-replace conflict handling is a separate D1n decision if the M2 follow-up review proves the current fingerprint checks insufficient.

## D1m completion judgment

1. Two different project files in one directory produce disjoint journal, catalog, generation, restore-marker, and lock paths; saving/recovering either leaves the other's bytes and metadata unchanged.
2. A subprocess holding a read-write session makes a second canonical or symlink-alias open fail immediately with `ProjectAlreadyOpen`. D1m adds a targeted Windows CI job in addition to the existing Linux job; macOS is run locally and its command/output recorded in the PR.
3. After normal guard drop and after forced subprocess termination, a new session can acquire the lock and recover normally.
4. All production mutation APIs require the session capability; path mutation and low-level `open_project*` / `recover_project` are crate-private or test-only with no public bypass. The sole catalog-required product open is `open_project_resolved`, which owns and returns `ProjectSession`.
5. Ordinary open never auto-adopts the shared legacy layout and returns `LegacySidecarRequiresExplicitMigration` without changing it. Explicit migration preserves the source, excludes `.motolii/media`, verifies the destination, is restart-idempotent, and returns `Result<LegacySidecarMigrationReport, SessionError>` per [Legacy sidecar migration diagnostic report (2026-07-18)](#legacy-sidecar-migration-diagnostic-report-2026-07-18).
6. Existing D1d corruption/fault/recovery **semantics and protected golden expectations** remain unchanged and green. Tests that hard-code the obsolete `.motolii` path may be mechanically changed to call the production path helper; expected errors, recovery documents, and golden values may not change. Finish with `cargo test --workspace`.
7. `rg`/public-API review proves that root-public project mutation is available only through `&mut ProjectSession`; raw path-only functions and `WalSession` are not exported to product callers.

## Required implementation order

One ticket is one commit. D1m is implemented in the following order inside that commit; if any step exposes a contract conflict, stop for a decision amendment rather than adding a compatibility special-case.

1. Add the exact native-`OsString` sidecar/lock path helpers and isolation tests, without redirecting callers yet.
2. Add `ProjectSession::acquire/open`, typed lock errors, and subprocess tests for same path, symlink alias, guard drop, and forced termination.
3. Add the closed family predicate/state table and explicit legacy staging→verify→atomic-install migration with idempotence/crash/failure-injection tests. Do not auto-attribute or partially merge the legacy family.
4. Route journal edit/checkpoint/recovery and document-file migration through the session capability; remove the root-public raw-path mutation/re-export closure listed above.
5. Move D1d tests from path literals to the production path helper only where mechanically required. Run the protected-golden policy before changing any expectation.
6. Prove same-directory project isolation; add/run the targeted Windows CI lock test; run the macOS lock test locally, package tests, then `cargo test --workspace`; record exact commands/output in the PR.

## Non-goals

- Distributed locking against Dropbox/iCloud/SMB peers.
- Automatic read-only fallback or lock stealing.
- Save As / rename API implementation.
- Storing UI window/session state in `Document`.
- Reinterpreting existing Document fields or changing render pixels.

## A0S save/open ownership amendment (2026-07-18)

### Conflict

D1m originally stated the only product project-open entry is `ProjectSession::open`, while A0I completion documented `open_project_resolved` without session ownership and root-public `save_document*`.

### Decision

Align with [VSM-A0S §11](2026-07-17-vism-a0s-contract-catalog-spec.md): catalog-independent `load_document*` and unknown payload preservation stay public; path writes and recovery mutation require `&mut ProjectSession` or crate-private internals; product open is session-owning `open_project_resolved` wrapping `ProjectSession::open`; low-level open/recover and repair without session are forbidden.

### Unchanged

- Sidecar family, lock semantics, legacy migration table, and D1d recovery semantics.
- No code, public API, schema, or test changes in this supplement.

## Legacy sidecar migration diagnostic report (2026-07-18)

Status: **Implemented** (D1m diagnostic report, branch `cursor/d1m-project-session`). This supplement closes the unknown-legacy-entry diagnostic contract for D1m. It does not change family predicates, staging/atomic-install steps, lock semantics, or D1d recovery meaning.

### Signature

Normative for future D1m implementation — docs only in this supplement:

```rust
// normative for future D1m impl — docs only in this order
fn migrate_legacy_sidecar(&mut self) -> Result<LegacySidecarMigrationReport, SessionError>;

struct LegacySidecarMigrationReport {
    disposition: LegacySidecarMigrationDisposition,
    untouched_legacy_entries: Vec<OsString>,
}

enum LegacySidecarMigrationDisposition {
    Installed,
    AlreadyValid,
}
```

`Ok` carries **only** `Installed` or `AlreadyValid`. Typed rejects (`NoLegacySidecar`, occupied/invalid/incomplete failures, I/O, lock errors, and all other `SessionError` variants) return `Err` with **no** report on the error side.

### `untouched_legacy_entries`

Scope: **legacy layout root** only — the parent-shared directory `<parent>/.motolii/` — **direct children** (non-recursive).

A direct child is **copied** (member of the journal-family copy set) iff its entry name exactly matches one of:

- `journal.wal`
- `catalog.json` (future implementation must use the same name as D1d `CATALOG_FILENAME`)
- `generations` (directory entry name; future implementation must use the same name as D1d `GENERATIONS_DIR`)
- `restore_attempted.json` (future implementation must use the same name as D1d `RESTORE_ATTEMPTED_FILENAME`)
- any name whose `OsStr` has the exact prefix `journal.wal.corrupt-` (same corrupt-stamp family as D1d recovery; no UTF-8 lossy conversion or additional normalization)

Every other direct child name is **untouched**, including `media` and any unknown file or directory. Values are entry names only (`DirEntry::file_name()` equivalent), not absolute paths and not parent/child path concatenation.

Sort order: **lossless platform-native `OsString::cmp` ascending**. UTF-8 conversion, Unicode/NFC normalization, locale sort, and case folding are forbidden.

### Preflight timing and failure

While holding the sibling lock:

1. Perform the existing closed read-only classification, including final valid/invalid verification.
2. After classification succeeds toward `Installed` or `AlreadyValid`, and **before** quarantine, copy, empty-final remove, or rename, read-only enumerate the legacy layout root and fix the report as a preflight snapshot.
3. `read_dir` or entry acquisition failure → typed I/O `SessionError`, filesystem unchanged, and **no** `Ok` with an empty `Vec` masquerading as success.
4. **`Installed`**: after mutation succeeds, return the **same** preflight report; do not re-enumerate and replace it post-install.
5. **`AlreadyValid`** (final valid, idempotent no-op):
   - legacy layout root directory exists → enumerate with the same diagnostic rules;
   - legacy layout root absent → `untouched_legacy_entries = []`.
6. The report is **non-persistent**: it is not saved to `Document`, catalog, journal, schema, wire formats, or any sidecar file, and is not serde-serialized. It exists only as the in-process return value.

Future implementation must reuse existing path/name helpers without inventing duplicates: `motolii_dir_for_document` (current legacy root = parent-shared `.motolii`), D1d `CATALOG_FILENAME` / `GENERATIONS_DIR` / `RESTORE_ATTEMPTED_FILENAME`, and the existing `journal.wal.corrupt-{stamp}` recovery naming.

### State-table return mapping

| Outcome | Return value |
|---|---|
| explicit migrate succeeds (atomic install) | `Ok(LegacySidecarMigrationReport { disposition: Installed, untouched_legacy_entries })` |
| final valid idempotent no-op | `Ok(LegacySidecarMigrationReport { disposition: AlreadyValid, untouched_legacy_entries })` |
| `NoLegacySidecar`, occupied, invalid, incomplete failure, I/O | `Err(SessionError)`, no report |

### Implementation-time required tests (not part of this docs-only supplement)

Positive:

- legacy root contains `media` plus an unknown file → `Installed` and `untouched_legacy_entries` lists both (copy-set names excluded), in `OsString::cmp` ascending order;
- final valid with legacy root present → `AlreadyValid` with the same untouched rules;
- final valid with legacy root absent → `AlreadyValid` with an empty `Vec`;
- non-UTF entry names remain `OsString` without lossy conversion.

Negative:

- `NoLegacySidecar` and other typed rejects → `Err`, no report;
- `read_dir` failure → typed I/O `Err`, filesystem unchanged, no `Ok` with an empty `Vec`;
- quarantine/copy/remove/rename must not run before preflight enumeration (failure injection proves filesystem unchanged).
