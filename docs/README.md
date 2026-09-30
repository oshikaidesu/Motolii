# docs map

Start here. Each row names the one document that plays that role today. History lives in git, not here.

| You want | Read |
|---|---|
| What Motolii is and why | [concept.md](concept.md), [design/ideal.md](design/ideal.md) |
| Screens and operations as adopted | [product/product-contract.md](product/product-contract.md), [product/product-direction.md](product/product-direction.md) |
| Architecture: core, renderer, host boundaries | [product/technical-boundaries.md](product/technical-boundaries.md), [product/modules.md](product/modules.md), [product/frame-graph.md](product/frame-graph.md) |
| Design principles behind the code | [design/](design/): extensible-core-model, memory-model, performance-model, simulation-model, freeze-and-flatten, plugin-resources, generative-user-boundary, known-implementation-adoption-model |
| Vism (portable expressions), shelf, sources | [design/vism/](design/vism/) |
| API notes for scripts, blocks, export | [design/skills/](design/skills/) |
| Why a rule exists (still-cited rationale) | [design/rationale/](design/rationale/) |
| User manual and what the window does not do yet | [wiki/index.md](wiki/index.md), [wiki/gap.md](wiki/gap.md) |
| Visual law: typography, spacing, row and control metrics, surface and semantic colors | `motolii/ui/lib/theme/metrics.dart` (numbers and colors), [product/ui-rebaseline/visual-language.md](product/ui-rebaseline/visual-language.md) (the language) |
| Run it, develop it | [product/README.md](product/README.md), [../motolii/ui/README.md](../motolii/ui/README.md) |
| What was decided | [decision-index.md](decision-index.md) (grep by keyword) |

Machine-read contract: [product/workspace.json](product/workspace.json), [product/modules.json](product/modules.json); checks: `scripts/check-workspace.py`, `scripts/check-docs.sh`.

History is git. A source written as `git:<sha>:<path>` is read with `git show <sha>:<path>`; retired dated reviews, specs, mocks and spikes are all under `git:912382f048:docs/...`.
