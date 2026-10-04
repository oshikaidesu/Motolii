# Ideas survey: Browser/library and Inspector/properties mechanisms in shipped tools

Scope: MECHANISMS only (what the tool does, the abstract problem it answers). No taste judgement, no looks.
Tags: [seen] = I used the product myself (none; this was desk research, so no [seen] claims),
[snippet] = confirmed from a search-result excerpt of official docs this session (URL given),
[inferred] = from background knowledge, not re-verified this session. Treat [inferred] as leads.

## A. Browser / library side

### 1. Ableton Live browser
- Hot-Swap: Q links browser to a device/clip; stepping items auditions them in place. Problem: audition in context, no commit. [snippet] https://www.ableton.com/en/live-manual/11/managing-files-and-sets/
- Preview switch plus Raw toggle: preview at original tempo, or synced to the Set. Problem: judge fit before apply. [snippet] same URL
- Collections: colour-labelled user tags (hotkey-assignable) shown as a flat cross-cut list. Problem: personal grouping independent of disk. [inferred]
- Quick search at top of the browser filters all categories at once. Problem: find fast. [inferred]

### 2. Blender Asset Browser
- Catalogs: virtual folders independent of file location; assign by dragging an asset onto a catalog; nestable. Problem: group by intent without moving files. [snippet] https://docs.blender.org/manual/en/latest/files/asset_libraries/catalogs.html
- Drag-to-apply by type: objects/worlds into scene, materials onto the target object. Problem: apply onto a target in one gesture. [snippet] https://docs.blender.org/manual/en/latest/editors/asset_browser.html
- Tags and Mark-as-asset on any datablock: your own work becomes library content. Problem: authoring and consuming share one mechanism. [inferred]
- Asset Library = a registered path; several libraries mounted at once, plus "current file". Problem: shared vs project scope. [inferred]

### 3. Figma Assets panel / libraries
- Alt-drag a component from Assets onto an instance to swap it. Problem: replace while keeping placement and overrides. [snippet] https://help.figma.com/hc/en-us/articles/360039150413-Swap-components-and-instances
- Swap libraries: re-point a file to another/newer library of styles, components, variables. Problem: change the source of truth wholesale. [snippet] https://help.figma.com/hc/en-us/articles/4404856784663-Swap-libraries
- Published libraries with update notices, per-file enable/disable. Problem: shared library versioning. [inferred]
- Component properties (variant, boolean, text, instance-swap) exposed as the instance's small control surface. Problem: curated parameters over a deep structure. [snippet] https://help.figma.com/hc/en-us/articles/8883757553943-Edit-instances-with-component-properties

### 4. DaVinci Resolve Effects library / Media Pool
- Effects library is a categorised list with a search box; double-click or drag onto a clip/node. Problem: find and apply by type. [inferred] https://www.blackmagicdesign.com/products/davinciresolve
- Hover-scrub thumbnail preview on media and some effects (Fusion-page and Cut-page viewers). Problem: preview before apply. [inferred]
- Bins, Smart Bins (rule-based, auto-populating from metadata), Power Bins shared across projects. Problem: group by rule, not by manual filing. [inferred]
- Favourites section inside the Effects library. Problem: shortcut to the few used. [inferred]

### 5. Cavalry assets / library
- Assets window lists compositions and linked files; assets are dragged into attributes (Asset Array, JSON/CSV/Google Sheet assets). Problem: bind data/media to a parameter by dropping. [snippet] https://cavalry.studio/docs/nodes/utilities/asset-array/
- Asset from Smart Folder: a folder on disk becomes a live array input. Problem: a folder as a data source. [snippet] https://cavalry.studio/docs/nodes/utilities/asset-from-smart-folder/
- Presets saved from a layer/group and re-applied from the browser. Problem: reuse a parameter-set. [inferred]

### 6. Houdini Tab menu / TouchDesigner OP Create dialog
- Tab (or double-click background) opens a type-ahead creation menu; Tab again cycles families. Problem: create by name without leaving the canvas. [snippet] https://learn.derivative.ca/courses/100-fundamentals/lessons/101-navigating-the-environment/topic/using-the-op-create-dialog/
- Right-click an output/input to open creation pre-wired to that port. Problem: create and connect in one act. [snippet] same URL
- Houdini Tab menu: fuzzy/abbreviation search with recents ranked up; also filters by what the current selection can accept. Problem: context-filtered find. [inferred] https://www.sidefx.com/docs/houdini/network/tabmenu.html
- Custom families/tabs registered by user components (TD "Custom" tab). Problem: extensible catalogue. [snippet] https://github.com/dotsimulate/TDFam

### 7. Notion / Raycast / VS Code-style command palettes (as find mechanism)
- Single text box, fuzzy match over every command/object, results grouped by kind. Problem: find fast across heterogeneous things. [inferred]
- Frecency ranking (recent plus frequent), last-used on empty query. Problem: favourites/recents without curation. [inferred]
- Prefix modes (">" commands, "@" symbols, "/" in Notion for blocks) switch the result domain. Problem: scope the search with one character. [inferred]
- Inline arguments/second step (Raycast: pick command, then fill fields in the same surface). Problem: apply with parameters without a dialog. [inferred]

### 8. Procreate brush library / Sketch libraries and styles
- Brush sets (folders), drag-reorder, recent set; tap a brush to select, same list holds user imports. Problem: group by intent, custom order. [inferred]
- Live stroke preview inside the brush tile/settings before use; settings pane edits that brush with a scratch pad. Problem: preview and tune before apply. [inferred]
- Duplicate-to-customise: edit never mutates the shipped brush, you fork it. Problem: safe modification of library items. [inferred]
- Sketch: shared styles/symbols from library documents with "update from library" and override tracking. Problem: linked copies that can be refreshed. [inferred]

### 9. Splice (sample library)
- Search by tags/key/BPM facets plus audition on click, auto-conformed to project key/tempo. Problem: preview already adapted to the current project. [inferred]
- "Similar sounds" from a selected item (embedding lookup). Problem: find by example. [inferred]
- Packs/collections and "Likes" as favourites; drag a preview onto the DAW track. Problem: favourites plus drag-to-apply. [inferred]

## B. Inspector / properties side

### 10. After Effects Effect Controls (and Timeline properties)
- Stopwatch per parameter toggles animation state; keyframe navigator beside every value. Problem: per-parameter animation state. [inferred] https://helpx.adobe.com/after-effects/using/effect-basics.html
- Effects stack = ordered, collapsible, toggle-able (fx switch) list; "Reset" link per effect. Problem: reorder, bypass, reset whole group. [inferred]
- Pick-whip on any property creates an expression link; Alt-click stopwatch opens expression. Problem: link/drive one value from another. [inferred]
- Shift-click twirl / U / UU in the timeline shows only animated or only modified properties. Problem: diff-from-default / show only what changed. [inferred]
- Animation Presets (.ffx) store a property set, apply to selected layers. Problem: preset of parameter sets. [inferred]

### 11. Blender Properties editor / N-panel
- Tabs by context (Object, Modifiers, Material...) with panels that collapse, Ctrl-click collapses all others (accordion). Problem: progressive disclosure. [inferred] https://docs.blender.org/manual/en/latest/editors/properties_editor.html
- Alt-edit applies to all selected; right-click "Copy to selected". Problem: bulk edit. [inferred]
- Colour state on fields: yellow/green = keyframed, purple = driver; right-click Add Driver. Problem: animation/link state readable in the field itself. [inferred]
- Ctrl+C/V over a hovered field, Backspace to reset to default, drag-over-fields to edit a column; number fields drag-scrub, Ctrl-wheel step. Problem: scrubbing values, reset. [inferred]
- N-panel / Sidebar: context-sensitive tabs registered by add-ons. Problem: extension surface on the viewport edge. [inferred]

### 12. Figma Design panel (and Resolve Inspector)
- Selection-driven sections (Layout, Appearance, Fill, Stroke, Effects), only sections relevant to the type appear; "+" adds an optional section. Problem: progressive disclosure by type. [inferred] https://help.figma.com/hc/en-us/articles/360039832014
- Mixed value shown as "Mixed" on multi-selection; typing sets all. Problem: bulk edit with honest state. [inferred]
- Variable binding chip on a field (colour, number): value replaced by a named token; detach to return to literal. Problem: link to a named source. [inferred]
- Instance override markers; Reset > property or all changes. Problem: reset/diff-from-main. [snippet] https://help.figma.com/hc/en-us/articles/360039150733-Apply-changes-to-instances
- Resolve Inspector: per-tab (Video/Audio/Effects) sections with a per-parameter reset arrow and keyframe diamond, group-level reset. Problem: reset and animation state, per group. [inferred]

### 13. Unity Inspector / Unreal Details
- Unreal: modified-from-default indicator next to a property with click-to-reset; "Show Only Modified" filter; star Favorites pin to top; search box filters properties live. Problem: diff-from-default, favourites, find. [snippet] https://dev.epicgames.com/documentation/en-us/unreal-engine/level-editor-details-panel-in-unreal-engine
- Unreal: edit conditions (grey out/hide fields by another field) and category grouping with Advanced fold. Problem: progressive disclosure by dependency. [snippet] https://dev.epicgames.com/documentation/en-us/unreal-engine/details-panel-customizations-in-unreal-engine
- Unity: prefab overrides shown bold with blue bar; Overrides dropdown applies/reverts per property or whole. Problem: diff vs. source with apply-back. [inferred]
- Unity: per-component presets (Preset Manager), multi-object editing with mixed dash, Debug/Normal inspector toggle, custom drawers via attributes. Problem: preset, bulk, raw vs curated view. [inferred]

### 14. Houdini parameter pane / TouchDesigner parameters / Cavalry attributes / Procreate adjustments
- Houdini: parameter label colour/italic = changed from default; Alt-middle-click revert; expressions in-field (green), keyframes (orange); channel-reference by drag; promote parameter to HDA interface. Problem: state in label, link, curate. [inferred] https://www.sidefx.com/docs/houdini/ref/ui/parmpane.html
- Houdini/TD: multiparm blocks (instanced groups) and folder tabs; TD parameter pages with modes (Constant/Expression/Export/Bind) switched per parameter by hover-click. Problem: one control, many sources. [inferred] https://docs.derivative.ca/Parameter
- Cavalry: connect any attribute to another by dragging from a Connection Anchor; incompatible attributes dim; Attribute Editor shows only compatible targets. Problem: link with type-filtered targets. [snippet] https://cavalry.studio/docs/getting-started/key-concepts/connections/
- Procreate Adjustments: pick effect, then drag finger across the canvas to set value (slider-less scrub), live preview of the whole layer, apply/cancel on release. Problem: scrub with immediate preview, no panel. [inferred] https://help.procreate.com/procreate/handbook/adjustments/adjustments-adjustments
- Procreate: Pencil/Layer mode choice before applying (whole layer vs masked). Problem: scope selection of an apply. [inferred]

## Matrix: abstract problem x tool mechanism
| Problem | Mechanism by tool |
|---|---|
| Find fast | Ableton quick search; TD/Houdini Tab type-ahead; palettes fuzzy+frecency; Unreal property search; Splice facets |
| Find by example | Splice "similar"; (none elsewhere found) |
| Context-filtered find | Houdini Tab filters by selection; Cavalry dims incompatible attrs; Figma Design shows type sections |
| Preview before apply | Ableton preview+Raw; Resolve hover scrub; Procreate live adjust/stroke preview; Splice auto-conform |
| Audition in context | Ableton Hot-Swap; Figma Alt-drag swap; Procreate live adjust |
| Group by intent | Blender catalogs; Resolve Smart Bins; Procreate sets; Ableton Collections |
| Favourites / recents | Ableton Collections; Resolve favourites; palettes frecency; Unreal star; Splice likes |
| Drag-to-apply | Blender (type-dependent target); Figma Alt-drag; Cavalry drop into attribute; Splice to track |
| Create + connect in one act | TD/Houdini port right-click create; Cavalry drag anchor |
| Parameter grouping | AE effect stack; Blender tabs/panels; Figma sections; Unreal categories; Houdini folders/multiparm |
| Progressive disclosure | Blender accordion; Figma "+" sections; Unreal Advanced fold + edit conditions |
| Scrubbing values | Blender drag/Ctrl-wheel; Procreate canvas drag; AE scrub on number |
| Link / drive | AE pick-whip; Blender drivers; Cavalry connections; Figma variables; Houdini channel ref; TD Export/Bind |
| Preset of parameter set | AE .ffx; Cavalry presets; Unity Preset Manager; Procreate brush fork |
| Bulk edit | Blender Alt-edit/copy to selected; Figma "Mixed"; Unity multi-edit |
| Reset / diff from default | Unreal modified dot + Only Modified; Houdini changed label; Figma Reset; Unity override bar; AE modified-only twirl; Resolve reset arrow |
| Per-parameter animation state | AE stopwatch+navigator; Blender field colour; Houdini field colour; Resolve diamond |
| Library versioning / re-source | Figma swap libraries + update; Sketch update from library |
| Curate a small surface from a deep one | Figma component properties; Houdini promote parameter; Unreal customization; Unity drawers |
| Extensible catalogue | TD Custom tab; Blender add-on tabs; Cavalry Smart Folder |

Gaps in this survey: no first-hand [seen] verification; Resolve, Splice, Sketch, Procreate rows rest on memory and should be spot-checked before any decision depends on them.
