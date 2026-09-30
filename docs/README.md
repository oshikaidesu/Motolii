# docs map

Start here. Each row names the one document that plays that role today.

| You want | Read |
|---|---|
| What Motolii is and why (product concept) | [concept.md](concept.md) |
| Screens and operations as adopted | [product/product-contract.md](product/product-contract.md) |
| Architecture: core, renderer, host boundaries | [product/technical-boundaries.md](product/technical-boundaries.md), [product/modules.md](product/modules.md), [product/frame-graph.md](product/frame-graph.md) |
| Run it, develop it | [product/README.md](product/README.md) (launch entries, test commands), [../motolii/ui/README.md](../motolii/ui/README.md) |
| What exists today and what does not | [wiki/gap.md](wiki/gap.md) (ideal vs. the window; written 2026-09-08, may be stale), [product/product-direction.md](product/product-direction.md) (per-capability migration gate), `pending` in [product/workspace.json](product/workspace.json) |
| What was decided and why | [decision-index.md](decision-index.md) (lookup by keyword), [product/product-direction.md](product/product-direction.md) |
| Words used when reporting what was launched | [ui-artifact-terminology.md](ui-artifact-terminology.md) |
| Where a document or an idea went | [product/document-map.md](product/document-map.md) |

Machine-read contract: [product/workspace.json](product/workspace.json) and [product/modules.json](product/modules.json), checked by `scripts/check-workspace.py`. Docs consistency: `scripts/check-docs.sh`.

## History (read only when investigating the past)

- [reviews/](reviews/README.md): dated investigations and decisions, all indexed there.
- [archive/](archive/docs-index-2026-08.md): documents and index of the generations before 2026-09.
- `mocks/`, `mocks-ui/`, `product/evidence/`, `product/history/`: generated or retained records.

Other top-level files in this directory are topic references (plugins, shaders/Vism, performance, text, simulation, UI language). Find them through the concept, the decision index or a keyword search.
