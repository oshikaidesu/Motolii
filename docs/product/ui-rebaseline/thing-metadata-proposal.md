# Thing metadata: precedents, proposal, and evidence

Scope: prototype only (`motolii/ui/lib/proto_hf/`). Nothing here decides a Vism manifest field. The docs say manifest fields, `look`/`primitive` tags, a `related` field and default filters are undecided, and forbid adding provisional ones to `NodeDesc` or the manifest (`docs/community-distribution-model.md`, `docs/vism-package-concept.md`). This is a fixture-level test of whether the idea holds, not a schema for production.

## 1. Precedents

| Source | What it does | Lesson |
|---|---|---|
| [freedesktop Menu spec, registered categories](https://specifications.freedesktop.org/menu/latest/category-registry.html) | A closed list of Main and Additional categories. Anything else must be prefixed `X-`. Case-sensitive. | Closed registry plus an extension lane. Categories are data, not UI. |
| [AppStream](https://www.freedesktop.org/software/appstream/docs/chap-Validation.html) | Every component has a globally unique `id`; categories come from the menu spec; `appstreamcli validate` checks the file. | A stable id and a tool that validates the metadata. |
| [Adobe UXP manifest](https://developer.adobe.com/photoshop/uxp/2022/guides/uxp-guide/uxp-misc/manifest-v5/) | `manifestVersion`, `id`, `name`, `version`, `host`, `entrypoints`. The host reads them. A VS Code validator exists. | The host reads a declaration; the plugin writes no UI to be listed. |
| [VS Code extension manifest](https://code.visualstudio.com/api/references/extension-manifest) | A fixed `categories` list plus free `keywords` (capped at 30). `vsce` validates. | Closed categories for browsing, free keywords for finding. |
| [Blender asset catalogs](https://developer.blender.org/docs/features/asset_system/backend/asset_catalogs/) | A catalog is a UUID, a path and a simple name. The asset stores the UUID; the path is a view independent of where the file lives. | Identity is separate from the view. Views can be reorganised without touching the thing. |
| [Ableton Live 12 browser](https://help.ableton.com/hc/en-us/articles/11425042663708-Browser-and-Tags-in-Live-12-FAQ) | Content is pre-tagged, users add tags, seven collections, and a browser view (keywords plus filters) can be saved. | User organisation and saved queries sit beside the content. |
| [After Effects Effects & Presets](https://helpx.adobe.com/after-effects/using/effects-animation-presets-overview.html) | Search shows matches plus the folders and categories containing them. A menu keeps six recent searches and saved searches. Organise by category, folder or alphabet. | Keep context around matches. Saved and recent searches are first-class. |
| [Premiere Pro effects](https://helpx.adobe.com/premiere/desktop/add-video-effects/apply-video-effects/find-and-group-effects.html) | Custom bins (nestable), plus attribute toggles such as accelerated or 32-bit that filter the list. | Attribute filters come from declared properties. Bins are the user's. |

Common shape: **a stable id, a closed registry of categories with an extension lane, free-form keywords, the host reads a declaration, views are queries or user data, and a validator gates it.**

Not verified by this search: prefix search such as `e:` or `p:` in After Effects Quick Apply. It was suggested but not confirmed, so the query syntax below is our own small extension of the confirmed pattern (saved queries), not a copy.

## 2. What the repo already says

- The engine already gives effects facts: an id, a stage, and flags (`persistent`, `usesClock`, `readsBackdrop`, `layerInputs`, `placement`, a parameter count). Tags and capabilities should be derived from these where possible, not typed twice.
- The old Effects shelf keeps a hand-written map from effect id to family in Dart (`effects_shelf.dart`), and Create keeps a switch by id (`create_shelf.dart`). That is exactly the drift a registry prevents.
- `docs/extensible-core-model.md` asks that a second thing pass through the same path with only manifest, data or composition. This work is a test of that at the UI edge.

## 3. Proposal (prototype scope)

**Descriptor** (one JSON object per thing). Fields: `id`, `name`, `kind`, `family`, `tags`, `capabilities`, `source`, `searchTerms`, `face`. Nothing about layout, columns or colours of the UI.

**Registry** (closed, one file): `kinds`, `families` (with an optional parent path and the kinds each accepts, and an optional display `section`), `tags`, `capabilities`, `faces` (each with typed parameters), and `panels` (which kinds a panel lists).

**Derived views.** Classification is not UI structure. The class column is computed each time: the top-level families present in the current results, in registry order. Search keeps the families that hold a match (the After Effects behaviour). User views (favourites, recent, collections, saved queries) are a separate store, not metadata.

**Query.** Free words match name, search terms, tags and family. `kind:`, `family:`, `tag:`, `cap:`, `source:` filter, and a parent family matches its children. Unknown `key:` text is an ordinary word. Everyday users see only a search box.

**Faces.** A thing declares an existing face type with parameters: `mark`, `fx` or `curve`. A new type is added once in one file; a new thing never adds drawing code.

**Static validation** (`tool/validate_things.dart`, also run by the tests; exit code 1 on any issue):

| Rule | Catches |
|---|---|
| THING-ID-FORMAT, THING-ID-DUPLICATE | Unstable or repeated ids |
| THING-NAME | Missing or long names |
| THING-KIND, THING-FAMILY | Invented kinds and families, with a "did you mean" suggestion |
| THING-FAMILY-KIND | A family used for a kind it does not accept |
| THING-TAG-UNKNOWN | Tag typos (an `x-` tag is the extension lane) |
| THING-CAPABILITY-UNKNOWN | Unknown capability |
| THING-CAPABILITY-UNIMPLEMENTED | Declared capability that nothing implements (needs an implemented list; the engine hook is not built) |
| THING-SOURCE | Source not `builtin` or `plugin:vendor.pack` |
| THING-NOT-SEARCHABLE, THING-SEARCHTERMS-LONG | Nothing to find it by; too many terms |
| THING-FACE-MISSING, -TYPE, -PARAM | Missing face, unknown face type, bad parameters |
| REG-PANEL-KIND, REG-KIND-UNBOUND, REG-FAMILY-PARENT, REG-FAMILY-KIND | A registry that would hide things or point nowhere |

## 4. Evidence

The Create and Effects panels were changed once to read descriptors. The Dart source of the panels and capabilities was then checksummed. Then **100 descriptors were added as data only**: 40 Physics relations (with sub-families), 30 plugin effects with source `plugin:acme.lens`, and 30 built-in effects. Result:

- The validator reports 141 descriptors, 22 families, 0 issues.
- The checksums of `create`, `effects`, `shell`, `classify`, `search`, `faces`, `things` and `common` were identical before and after adding the data: **0 files changed**.
- Physics appeared as a new class in the Create panel, with 40 phenomenon curves as faces. A `source:plugin` query in Effects returned the 30 plugin effects and reduced the class column to Lens, Glitch, Film and Depth.
- A saved query (`Lens glitch`) appeared as a user view without any panel code.
- The tests (39, all passing) cover each validation rule, the query language, the derived views, the panels reading only data, and a pixel check that a declared face really draws.

One defect was found and fixed along the way: a local variable named `base` shadowed the field of the same name in the effect painter, so every effect thumbnail rendered blank with no error. That was a code bug in the painter, unrelated to the metadata idea, and a regression test now checks pixels. That fix happened after the "0 files changed" comparison.

Captures: `handoff/browser-panels-things.png`.

## 5. Limits and open points

- A new **face type** (as opposed to a new thing) still needs one Dart addition in `faces.dart`. That is presentation code, added once.
- A new **kind** needs a registry entry and a panel binding, both data. It needs no panel code.
- Families are two levels here (a parent and its children); the class column shows only the top level. Nested display (Premiere-style bins, Blender paths) is not built.
- Colors and Fonts still read their own lists; they are not moved to descriptors yet.
- Capability-versus-implementation checking is a rule with a stub input, not connected to the engine.
- Mapping to real engine descriptors (stage, flags) and to the Vism manifest is undecided, per the docs.
- Whether the registry vocabulary lives in the repo, in a package, or per plugin pack is undecided.
