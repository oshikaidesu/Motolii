# Ecosystem ideas: what creators were forced to build (AE / Blender / Cavalry / Figma / Ableton / Premiere)

Date 2026-10-02. Research only. Tag: [seen] = page fetched and read, [snippet] = search-result text only, [inferred] = my reading of the pattern.
Side: B = Browser-side (library/finding/applying), I = Inspector-side (properties/values/links), T = timeline/other-host-surface.
Many aescripts.com pages return 403 to fetch, so most are [snippet].

## 1. Finding (where is the thing?)
- Animation Composer (aescripts). Need: find a ready motion/effect without knowing its name. B. Grid of presets, hover plays a video preview, click adds. [snippet] https://aescripts.com/animation-composer/
- Effects & Presets panel complaints. Need: find an effect among hundreds. B. Name-prefix search only; panel glitches with ~30+ user preset folders; "Browse Presets" bounces out to Bridge. [snippet] https://community.adobe.com/bug-reports-528/effects-and-presets-panel-going-black-not-working-1217140
- KBar. Need: reach your own few effects/scripts in one click. B. User-built button toolbar (apply effect/preset, expression, script, menu command, up to 4 bars via modifiers). [snippet] https://aescripts.com/kbar/
- Expression Kit / Expression List Manager. Need: find and reuse an expression snippet. B. Searchable list with title+description, preview, one-click apply to selection. [snippet] https://aescripts.com/expression-kit/
- aereference.com expressions. Need: look up syntax without leaving work. B (web). Searchable library. [snippet] https://aereference.com/expressions
- Figma Unsplash / Iconify. Need: get stock images/icons in place without leaving the tool. B. Search box in a panel, result arrives as editable object. [snippet] https://medium.com/@kaankiziltug/10-must-have-figma-plugins-to-streamline-your-design-workflow-3bdf56dacae2
- Blender Node Wrangler add-on search/shift-A style menus ("better node menu"). Need: fast node adding by typing. B. Search-add in node editor. [snippet] https://quackers.gumroad.com/l/better_node_menu
- Premiere: Essential Graphics panel replaced by Graphics Templates (browse) + Properties (edit) split. Need: separate "what template" from "edit this one". B+I. [snippet] https://blog.frame.io/2024/08/12/mogrt-guide-after-effects-2024-motion-graphics-workflow/

## 2. Previewing before committing
- Animation Composer previews. Need: judge motion before applying. B. Per-item video preview that plays on hover; presets load as separate layers so removal is one click. [snippet] https://aescripts.com/animation-composer/
- Blender Asset Browser thumbnails + Thumb Mate. Need: recognisable thumbnails (default Workbench render poor, custom thumbnail loading bug). B. Add-on re-renders previews with Eevee. [snippet] https://braverabbit.gumroad.com/l/thumbMateBlender
- Blender Pose Library v2. Need: see a pose, apply it with blend. B. Pose stored as asset with embedded preview. Complaint: apply needs pose mode + asset browser + exit; only alphabetical order. [snippet] https://code.blender.org/2021/05/pose-library-v2-0/
- Node Wrangler preview (Ctrl+Shift+click). Need: see what any node outputs without rewiring. I/B. Click a node = temporary viewer link. [snippet] https://github.com/gregzaal/node_wrangler/
- Flow library. Need: see the shape of an easing as a thumbnail before using it. B. Library of named curves. [snippet] https://aescripts.com/flow/

## 3. Applying (non-destructive, controlled)
- Animation Composer Edit tab. Need: adjust an applied preset without opening comps. I. Each preset exposes its own controls (colours, timing, font) in a dedicated tab; a preset is its own layer. [snippet] https://aescripts.com/animation-composer/
- Premiere/AE Essential Properties + MOGRT. Need: expose only the 5 controls a template author wants editable. I. Author picks properties; user sees only them. Limits: can't interact with other timeline layers. [snippet] https://blog.frame.io/2024/08/12/mogrt-guide-after-effects-2024-motion-graphics-workflow/
- Overlord (Battle Axe). Need: move artwork between apps without export/import. B (bridge). Push/pull selection Illustrator/Figma/Photoshop to AE keeping gradients, live text, names; also swatches and guides. [seen] https://battleaxe.co/overlord/docs/bonus
- Node Wrangler PBR setup (Ctrl+Shift+T). Need: apply a known multi-part setup from a folder of files at once. B. Name-matching of texture maps to inputs. [snippet] https://github.com/gregzaal/node_wrangler/
- Mt. Mograph Null / Parent / Trim / Burst. Need: apply a common rig/setup to the selection in one click. T. One-shot operations on selection. [seen] https://mtmograph.com/products/motion/tools

## 4. Organizing (library and project hygiene)
- Mt. Mograph Sort. Need: project items auto-categorised. B. One-click order by type. [seen] https://mtmograph.com/products/motion/tools
- Layer Library (aescripts). Need: store and recall layers/comps as personal assets. B. Library of saved layers. [snippet] https://aescripts.com/layer-library/
- Premiere MOGRT folders. Need: group templates; limit: subfolders ignored. B. Folder list in panel. [snippet] https://photography.tutsplus.com/tutorials/how-to-organize-motion-graphics-mogrts-in-adobe-premiere-pro--cms-108800
- AE timeline folders request. Need: collapse groups of layers without precomp (rasterization cost). T. "Folders in timeline" idea thread, thousands of votes. [snippet] https://community.adobe.com/t5/after-effects-ideas/folders-in-after-effects-timeline/idc-p/13230893
- Mt. Mograph Focus Group (lock/shy/select/visible per group, isolate). Need: work on part of a crowded comp. T. Named groups with toggles. [seen] https://mtmograph.com/products/motion/tools
- Cavalry Assets window / Asset Array. Need: a list of images/assets addressable by index. B. Assets panel + node that exposes list. [snippet] https://cavalry.studio/docs/nodes/utilities/asset-array/
- Blender Asset Browser libraries + catalogs. Need: tag/catalog assets across files. B. [snippet] https://docs.blender.org/manual/en/latest/editors/asset_browser.html

## 5. Naming and sorting
- Sortcery. Need: order layers by name/position/selection/colour label. T. Sort command. [snippet] https://5kstudio.gumroad.com/l/Sortcery
- layerNamer. Need: rename many layers with numbers/letters/effect name. T. [snippet] https://kylenmotion.gumroad.com/l/layer-namer
- Mt. Mograph Rename / Arrange / Reverse / Textbreak. Need: serialise names, reorder stack, split text into pieces. T. [seen] https://mtmograph.com/products/motion/tools
- Duplicate & Rename (aescripts). Need: duplicate while search/replacing the name (L to R). T. [snippet] https://aescripts.com/duplicate-and-rename/
- Blender Batch Rename & Replace Pro. Need: rename many objects with rules. T. [snippet] https://studio156.itch.io/batch-rename-replace-pro
- Layer names on timeline bars (100+ layer comps). Need: identify a bar without hovering. T. Forum request. [snippet] https://community.adobe.com/t5/after-effects-discussions/layer-name-appear-on-timeline-bar-in-after-effects/m-p/9555905

## 6. Reusing / copying values and structure
- Copy with Property Links (native AE). Need: copy an effect so edits stay shared. I. Cmd+Opt+C. [snippet] https://ukramedia.com/copy-with-property-links-in-adobe-after-effects/
- AE copy attributes pain. Need: copy "just the look" of a layer to others (attributes, not whole layer); copy of whole Transform group only from CS3. I. Forum threads. [snippet] https://community.adobe.com/t5/after-effects-discussions/is-there-a-way-to-copy-attributes-from-one-composition-to-another/m-p/2234164
- Layers Pro. Need: bulk copy/replace/remove effects, properties, masks across selected layers and comps. I/T. [snippet] https://aescripts.com/layers-pro/
- Blender Copy Attributes Menu. Need: copy modifiers, constraints, custom props from active to selected objects. I. Extends Ctrl-C. [snippet] https://docs.blender.org/manual/en/4.1/addons/interface/copy_attributes.html
- Mt. Mograph Copy Color / Paste Color, Copy Ease / Paste Ease, Re-Key, Clone. Need: carry one aspect (colour, ease, keys) to other targets. I. Clipboard of a single aspect. [seen] https://mtmograph.com/products/motion/tools
- Pseudo Effects + save-as-preset. Need: ship your own control panel (sliders, checkboxes) with a setup. I. Group of expression controls dragged into Presets. [snippet] https://www.batchframe.com/support/manuals/pseudo-effect-maker
- Ray Dynamic Color. Need: one palette driving all colours; update once, everything follows. I (links). Apply/link/update swatches across project. [snippet] https://aescripts.com/ray-dynamic-color/

## 7. Linking parameters (one value drives another)
- Expression Controls (Slider, Checkbox, Angle, Colour, Point, Layer). Need: a visible handle for a value that other properties read. I. Do nothing alone; hold a value for expressions. [snippet] https://blog.pond5.com/10203-using-expression-controllers-in-after-effects/
- Pick-whip (native AE). Need: link property to property by dragging. I. [inferred] common AE practice; see Duik "Link"/"Connector" below.
- Duik Link and Connector. Need: map one property's range to another without writing expressions. I. UI-built expressions. [seen] https://duik.org/guide/Angela-pre/guide/bones/autorig/
- Cavalry Connections. Need: attribute-to-attribute data flow as a first-class idea. I. Drag to connect; node graph behind. [snippet] https://cavalry.studio/docs/getting-started/key-concepts/connections/
- Blender Animation Nodes. Need: a driver alternative, text manipulation, replication. I/graph. Visual scripting for motion graphics. [snippet] https://github.com/JacquesLucke/animation_nodes
- Ableton Max for Live LFO / Envelope Follower / Shaper. Need: map a modulator to up to 8 parameters of any device with a range each. I. Map button then move target. [snippet] https://www.ableton.com/en/manual/max-for-live-devices/
- Ableton Macro knobs. Need: one knob over many parameters, each with own range. I. [inferred] (seen in Shaper-to-macro snippet) https://www.soundonsound.com/techniques/midi-control-change-ableton-live

## 8. Bulk edit / multi-selection
- Mt. Mograph Grab (select all same property type), Delay/Stagger/Falloff/Distribute/Align. Need: edit many layers' same property with offsets. I/T. [seen] https://mtmograph.com/products/motion/tools
- Blender Massive Editor. Need: edit a property on several objects/nodes at once. I. [snippet] https://blenderartists.org/t/addon-massive-editor-edit-objects-add-modifier-on-multiple-objects/610371
- Blender Attribute Manager. Need: spreadsheet-like batch editing. I. [snippet] https://khalibloo.itch.io/khalibloo-panel
- Find & Replace Keyframes. Need: change every keyframe with value X to Y. I. [snippet] https://aescripts.com/find-and-replace-keyframes/
- pt_SearchAndEditBundle / Reach Text Snooper. Need: search/replace in expressions and text across the whole project. I. [snippet] https://aescripts.com/pt_searchandeditbundle/
- Mt. Mograph Trash. Need: remove stray effects and linked expressions in one go. I. [seen] https://mtmograph.com/products/motion/tools

## 9. Easing / curve feel
- Flow (Zack Lovatt/renderTom). Need: shape easing quickly, outside AE's graph editor, save curves as brand presets. I+B. Extension graph editor (normalised) + curve library. [snippet] https://aescripts.com/flow/
- Ease and Wizz. Need: easings AE lacks (Expo, Back, Elastic, Bounce; In/Out/InOut) while keys stay editable. I. Click a name, expression written. [snippet] https://aescripts.com/ease-and-wizz/
- Mt. Mograph Easing: Cubic Ease (CSS function), Ease Influence, Ease Speed, Copy/Paste Ease, Next/Prev. Need: set ease by familiar CSS bezier or real values. I. [seen] https://mtmograph.com/products/motion/tools
- Mt. Mograph Excite / Jump / Dynamics. Need: overshoot and bounce without hand-keying. I. [seen] https://mtmograph.com/products/motion/tools
- Motion tool "Separate" preserving easing. Need: split X/Y without losing curves. I. [seen] same URL.

## 10. Rigging / controllers
- Duik Bassel/Angela. Need: IK/FK, bones, controllers, auto-rig, so the animator touches 1 handle not 30 properties. I+T. Controller layers drive constraints; auto-rig in two steps. [snippet] https://simonfairbairn.com/understanding-duik-bassel-after-effects/
- Rubberhose. Need: flexible limbs; controls live on the end controller you animate most. I. Two pseudo effects, hose + bend point. [snippet] https://battleaxe.co/rubberhose/docs/controls
- Mt. Mograph Null, Stare, Orbit, Trace, Pin+. Need: one-click control objects. T. [seen] https://mtmograph.com/products/motion/tools
- Joysticks 'n Sliders. Need: 2D slider that blends poses/shapes. I. [snippet] https://schoolofmotion.com/blog/after-effects-tool-review-joysticks-n-sliders-vs-duik-bassel
- Cavalry Behaviours (40+ drag-on drivers). Need: add noise/random/modulate without keys or code. I. [snippet] https://cavalry.studio/docs/getting-started/key-concepts/connections/

## 11. Property clutter / navigation
- AE Properties panel (2022+). Need: reach key properties without twirling open nested shape groups. I. Contextual panel for selected layer. [snippet] https://helpx.adobe.com/after-effects/using/properties-panel.html
- Solo/hide properties in timeline. Need: temporarily show only what you edit. I. (SS / U shortcuts) [snippet] https://aereference.com/tips/solo-hide-layer-properties-in-the-timeline-panel
- Premiere Properties panel replaced Essential Graphics. Need: one contextual place for editing. I. [snippet] https://blog.frame.io/2024/08/12/mogrt-guide-after-effects-2024-motion-graphics-workflow/

## 12. State, versioning, snapshots
- Mt. Mograph Snapshot / Re-Key (save and reapply keyframes). Need: stash state, return to it. I/T. [seen] https://mtmograph.com/products/motion/tools
- Overlord round trip. Need: keep editing in the source app and re-push. B. [snippet] https://battleaxe.co/overlord
- Frame.io panel. Need: review comments on the timeline; native in Premiere 25.2+. T. [snippet] https://help.frame.io/en/articles/9859849-premiere-pro-frame-io-v4-panel-overview
- Content Reel (Figma). Need: realistic stand-in content, shared collections for a team. B. [snippet] https://medium.com/@kaankiziltug/10-must-have-figma-plugins-to-streamline-your-design-workflow-3bdf56dacae2
- Stark (Figma). Need: live check (contrast, colour-blind sim) during editing. I. [snippet] same URL.
- Autoflow (Figma). Need: arrows between frames that follow when you move them. T/I (derived link). [snippet] same URL.

## Cross-cutting observations (inferred)
1. Most forced extensions are one-click operations on the selection (Motion has ~88 tools), not new capabilities. The host lacked a verb, not a feature.
2. The recurring Inspector pattern is "a visible handle for a value other things read" (expression controls, controllers, macros, connections). Users build it because direct linking is hard to see.
3. Easing was rebuilt three times (Flow, Ease and Wizz, Motion Easing): numbers/handles are a poor feel; named, previewable, copyable curves win.
4. Browsing wins on hover-preview + non-destructive removal (Animation Composer); losses come from apply flow needing mode switches (Blender pose).
5. Copy "one aspect" (ease, colour, keys, property group) is a repeated gap; hosts only copy the whole thing.
6. Layer/property naming and sorting are bulk problems nobody solved natively (timeline folders request still open).
