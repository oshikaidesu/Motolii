# UI rebaseline

The rebaseline is done: one app (`motolii/ui/lib/main.dart`), one visual canon (`motolii/ui/lib/theme/metrics.dart`). The Classic and New shells of the migration are retired; Git keeps them.

The principle that drove it still holds:

> **The UI is the capability oracle, not the layout oracle.**

| Document | What it holds |
|---|---|
| [brief.md](brief.md) | The user's instruction of 2026-09-25 (capability first, layout free, Modern = dense / flat / sharp / instrument-like) |
| [visual-language.md](visual-language.md) | How new UI is derived: housing, contents, faces. No values; values live in `theme/metrics.dart` |
| [browser-create-media.md](browser-create-media.md) | Create / Media bodies: what is preserved, what is redesigned |
| [thing-metadata-proposal.md](thing-metadata-proposal.md) | Precedents and a proposal for thing metadata. Decides no manifest field |

The Product Home picture and hierarchy are in [../product-direction.md](../product-direction.md). The Phase A report, capability inventories, Classic photographs and Concept Art are in Git at `git:912382f048:docs/product/ui-rebaseline/README.md`, `git:912382f048:docs/product/ui-rebaseline/inventory` and `git:912382f048:docs/product/ui-rebaseline/concept`.
