---
name: historical-recovery
description: Recover bounded evidence from Motolii's fixed historical Git corpus when users ask about old ideas, lost plans, rejected specifications, prior decisions, or design archaeology such as AviUtl catalog/hostless distribution, Vism Kit, single-camera, or 2.5D. Use for candidate discovery and comparison with current authority; do not use for ordinary current-code search, exhaustive receipt audits, bulk adjudication, or implementation.
---

# Historical Recovery

Recover a small, traceable candidate packet on demand. History supplies evidence, not
authority: read current decisions and code first, then inspect only the selected Git blobs.

## Fixed Authority

Work from the repository root. Read these before retrieving historical candidates:

1. `docs/decision-index.md`
2. `docs/reviews/2026-07-23-historical-semantic-graph-recovery-tooling.md`
3. the current spec/review documents named by the decision index for the topic
4. current code facts when the topic affects implementation

Follow HVR-D03 §4.2 exactly. Treat the fixed corpus, disposition receipts, HVR-D01
projection, and optional HVR-D02 index as separate layers. Semantic similarity and lexical
matches identify candidates only; they never establish coverage, disposition, or adoption.

## Retrieval Workflow

### 1. Bound the question

Restate one topic and list explicit query terms plus useful spelling variants. Prefer terms
that occurred in the discussion, including Japanese/English pairs and former names. If the
request combines unrelated topics, make separate packets rather than one broad search.

Do not start this workflow for ordinary searches of current docs/code, a request to adjudicate
all remaining blobs, or implementation work.

### 2. Prepare a repo-external projection

Use a new temporary parent outside the repository. Generate HVR-D01 output only into a
nonexistent child:

```sh
python3 scripts/project_historical_portable_markdown.py \
  --repo-root "$PWD" \
  --out /absolute/repo-external/path/projection
./scripts/check-historical-docs-recovery.sh
```

Never place projection, packet, index, model, or cache files in the repository. Never alter
an existing projection. Projection generation success establishes the HVR-D01 projection;
the checker separately supplies and validates the corpus/receipt coverage counts. If either
fails, return a `停止` packet.

To reuse an existing repo-external projection, do not trust it in place. Generate a fresh
projection into a new sibling `probe` directory, compute both tree hashes with the command
below, and reuse the existing projection only when the hashes match. Never overwrite the
existing directory. The new probe may be moved to trash after comparison.

Compute the projection hash using the same HVR-D02 function:

```sh
PYTHONPATH="$PWD" python3 -c \
  'import sys; from pathlib import Path; from scripts.historical_semantic_index import projection_tree_hash; print(projection_tree_hash(Path(sys.argv[1])))' \
  /absolute/repo-external/path/projection
```

### 3. Choose retrieval mode

Use semantic search only when the user supplies an absolute repo-external HVR-D02 state and
that state contains a valid `hvr-index.json`:

```sh
python3 scripts/historical_semantic_index.py search \
  --repo-root "$PWD" \
  --state /absolute/repo-external/state \
  --query 'bounded UTF-8 query' \
  --page-size 20 \
  --offline
```

Omit `--offline` only when the user explicitly permits network use. Do not implicitly install
Basic Memory, download a model, or build/rebuild an index.

Before semantic search, read the runner-owned `hvr-index.json` marker and require its
`projection_tree_sha256` to equal the projection hash recorded for this packet. A mismatch
means the index describes another projection; use lexical fallback and report the mismatch.
Do not inspect Basic Memory's database or internal schema.

Always run lexical retrieval as either the primary fallback or a cross-check. Use `rg` over
`projection/nodes/` with the declared terms and variants. Search `manifest.tsv` to map each
node SHA to its historical path, coverage state, and receipt fields. Do not interpret
`edges.tsv` as inferred meaning.

If semantic search is absent, fails, or returns weak candidates, label the mode `lexical` or
`mixed` and continue with `rg`. A failed or empty retrieval does not prove that no relevant
history exists.

### 4. Select and read candidates

Deduplicate by full 40-character blob SHA. If a query yields more than 20 plausible candidates,
narrow the topic or terms explicitly and rerun retrieval; record the broader count under
Limits. Do not silently keep the first or highest-ranked 20. If the question cannot be bounded
to at most 20 without arbitrary omission, return a `停止` packet. Selection reasons must cite
the matched term or semantic query; score and rank are not reasons to adopt.

Before reading bodies, record SHA, historical path, node path, retrieval route, and receipt
state from `manifest.tsv`. Then read every selected node in full. Verify any uncertain body
against:

```sh
git cat-file blob <full-blob-sha>
```

Do not read arbitrary additional blobs after selection. If a candidate cannot be read as the
recorded Git object, return `停止`.

### 5. Compare with current authority

For each fully read candidate, compare it with the current spec and current code facts. Propose
exactly one status:

- `観察`: useful context, not yet an option or decision
- `比較中`: plausible option requiring explicit adjudication
- `決定`: already adopted by current authority; history is supporting evidence only
- `棄却`: contradicted or superseded by current authority
- `停止`: authority, receipt, object identity, or contract boundary is inconsistent

State the relation to current authority and what remains unproven. Never promote historical
text directly into a public API, persistent format, Document meaning, or plugin contract.

### 6. Write and return the packet

Read `references/candidate-packet.md` and create one Markdown packet outside the repository.
Return its absolute path and a concise candidate count. Stop after the packet.

Do not edit receipt files, the decision index, specs, ledgers, source, or tests. A later HVR-D04
unit may adjudicate accepted candidates through the normal one-unit/one-commit workflow.

## Stop Conditions

Return a `停止` packet instead of repairing around any of these:

- HVR-D01 generation or coverage checking fails.
- A candidate SHA is not readable as the recorded Git blob.
- Receipt fields conflict with the candidate body.
- Current authority is unresolved or internally contradictory.
- The question cannot be explicitly narrowed to at most 20 candidates without arbitrary
  omission, or completion requires implicit index construction, receipt mutation, or
  tracked-file edits.
- A candidate appears to require changing a public API, persistence, Document, or plugin
  contract before explicit adjudication.

Never claim “all history was read,” “the search was exhaustive,” or “no value exists.”
