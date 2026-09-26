# Browser-family panels: capability audit

Audit only. Nothing is connected to production and the visual prototype (`proto_hf/bp/`, `main_browser.dart`) is unchanged.

## 0. Retraction

Two lines in the last report were wrong and are withdrawn:

- "BrowserPanel / BrowserShelf abstraction dropped."
- "The only thing the four panels shared is the dock tab strip."

What the experiment proved: **Create, Effects, Colors and Fonts keep different bodies at wide, narrow, very narrow and tab-stacked sizes, and they should not share one visual component or one responsive rule.** That is about presentation. It says nothing about whether they share behaviour. The captures themselves show they do: all four have a search, and three have the same classification row.

Separate two questions and do not merge them:

| Question | Answer from the prototype |
|---|---|
| Do the four bodies need different looks and different folding? | Yes. |
| Do the four panels share discovery behaviour? | Yes. It is visible on screen and in the old code. |

## 1. What the prototype contains

The prototype is static. It has no state, no selection handling, no apply and no keyboard. So the duplication measured below is duplicated **affordances and decisions**, not duplicated logic. The behaviour evidence comes from the old code (section 3).

Repeated across the four bodies:

| Repeated thing | Create | Effects | Colors | Fonts |
|---|---|---|---|---|
| Search field, which folds to a search key when narrow | yes | yes | yes | yes |
| Classification row (All plus groups) | yes | yes | no (three groups appear as sections: Used here, Saved, Starter) | yes |
| A list of items | marks | rendered results | swatches | specimens |
| A current or chosen item | none shown | none shown | current colour | first row chosen |
| An action beside search | view switch | reload | From image | none |
| Scrolling body | yes | yes | yes | yes |
| A width test that decides folding (the same 240 px cut) | yes | yes | no (its own rule) | yes |

What is common is behaviour (query, classify, list, select, act). What is not common is how each item looks and how each body folds. The 240 px cut being copied three times is the one visual rule that leaked between them, and it should not be shared either.

## 2. What the old Browser provides

Read from `panels/browser.dart` and `panels/browser/`.

The frame (`BrowserPanel`) owns: search box and its tools, view switch (grid, list, thumbnails), filter view, category rail with resize and collapse, grid layout, tile scale, selection and multi-select, keyboard handling, drag and drop of files and labels, collections and labels the user keeps (`BrowserLibrary`, seven collections, tags), persisted preferences, and a notice line.

A shelf (`BrowserShelf`) declares: `items`, `rails`, `classification`, `groups` (filter groups), `tagsOf`, `valueOf`, `numberOf`, `supported`, `identity`, `apply`, `applyLabel`, `menu` and `act`, `facts`, `delete`, `draggable`, `multiSelect`, `doubleClick`, `derived` (what state it depends on), `deskKeys` (what it stores), `enter` and `dispose`. It also declares presentation: `preview`, `format`, `bare`, `showViews`, `defaultView`, `layout`, `header`, `editor`, `tools`, `overlay`.

Which shelves actually use which (read from the six shelf files):

| Capability | Create | Effects | Colors | Fonts | Media | Files |
|---|---|---|---|---|---|---|
| items, rails, classification, supported, apply, identity | x | x | x | x | x | x |
| filter groups (groups, tagsOf, valueOf) | . | x | x | x | x | . |
| multi-select | . | x | . | . | x | . |
| context menu and actions (menu, act) | . | . | x | . | x | x |
| drag out | . | . | . | . | x | . |
| delete | . | . | . | . | x | . |
| facts and badge | . | . | . | . | x | x |
| depends-on and store keys (derived, deskKeys) | . | x | x | x | . | . |
| double-click applies | . | . | . | x | . | . |

Reading: capabilities are already opt-in in practice. Create uses almost none of the extras; Media uses most. That is evidence for a capability model, not against it.

## 3. Capability inventory

Behaviour that is shared or shareable, and presentation that is not. This is a list of behaviours, not of Classic classes.

| Capability | What it is | Shared? | Notes |
|---|---|---|---|
| Items | The list of things, with an id | yes | Source is engine data (catalog, palette, fonts, assets). |
| Query | Search text narrows the list | yes | |
| Classification | Categories an item belongs to; picking one narrows | yes | Colors uses it as Used here, Saved, Starter. |
| Filter | Tag groups with values and ranges | opt-in | Used by four shelves. |
| Availability | Whether an item can be applied now | yes | Depends on selection or engine support. |
| Selection | One or many picked items | yes | Single or multi is declared. |
| Apply | What activating an item does; label of that action | yes | Click or double-click is declared. |
| Item actions | Menu, delete, favourite, reveal | opt-in | |
| Drag out | Item as a drag payload | opt-in | |
| Keys | Arrows, enter, delete, search focus | yes | Should be the same in every panel. |
| Collections and labels | User-kept groups and tags | opt-in, shared across panels | Live 12 style. Data lives per panel today. |
| Live dependencies | Which session state redraws it; which preferences it stores | yes | Needed for Physics too. |
| Presentation | Tile face, layout, captions, editor, header, overlay, view modes | no | Per panel. |
| Morphology | What to fold, hide, rearrange at a given space | no | Per panel. |

Mixed and worth splitting: the old shelf contract holds presentation (`bare`, `showViews`, `defaultView`, `layout`, `preview`, `format`, `header`, `editor`, `overlay`) next to behaviour, and the frame reads both. That is why `BrowserPanel` looks heavy. It is a design smell of one class doing two jobs, not proof that behaviour should be per panel.

## 4. Minimal capability architecture (proposal, not code)

Share capabilities, not bodies.

```
Discovery   (behaviour, per panel instance, opt-in parts)
  items          what exists
  query          search text
  classes        categories, and which one is chosen
  filters        optional tag groups
  selection      single or multi, and what is picked
  actions        apply, label, menu, delete, drag payload
  availability   can this item be applied now
  deps           what session state it depends on, what preferences it stores

Presentation (per panel: looks and morphology)
  body(space, discovery)   builds the body from the state and the space
  which discovery controls it shows at this space
```

Rules:

- **Composition, not inheritance.** A panel lists the capabilities it has. Create: items, classes, actions. Effects: items, classes, filters, multi-select, actions on the selection. Colors: items, classes, single selection, actions. Fonts: items, classes, filters, single selection, actions.
- **Controls are parts, not a bar.** The capability supplies the state and the control widgets (search box, class row, filter). The presentation decides whether and where to place them, and how they fold. The prototype already does this: each body places search and chips itself.
- **The chassis** keeps what is about the seat: dock tab, close, detach, focus, keyboard routing, persistence of view choices, drop hint, the notice line.
- **Only what a panel needs is built.** No panel instantiates other panels' sources. This removes the old cost of constructing all six shelves in each panel without dropping the capability model.
- Nothing here copies the class hierarchy. `BrowserHost`, `BrowserShelf` and the panel state can all be reshaped; the list above is what must survive.

### Extensibility test

A new panel (Physics, a plugin, procedural presets, an AI panel) passes when it needs only:

1. a source of items (and their classes),
2. a list of the capabilities it wants,
3. its own body and its own morphology.

It must not re-implement search, classification, filtering, selection, apply, actions or keys. Physics would supply its items and a body that shows the phenomenon; everything else comes from the capabilities.

## 5. Not decided (needs a human)

1. **Who draws the discovery controls.** Recommendation above: capabilities supply them, presentations place them. The old frame drew a fixed bar. Confirm.
2. **Collections, labels and tags across panels.** Today per panel. Should a label span Create and Effects?
3. **Colors' instrument.** In the old code it is the shelf's editor slot. It is presentation plus a target, not a discovery item. Confirm it stays separate from the list of swatches.
4. **Selection rules.** Single or multi per panel, and what "apply" means with several selected.
5. **Keyboard.** One shared set, or per panel.
6. **Where sources come from.** Engine catalog, palette, system fonts, assets: which of these already have a stable item id.

## 6. What this does not claim

- It does not claim the current `BrowserPanel` code is good; it likely holds too much.
- It does not claim the new model should look like the old classes.
- It does not decide production migration, or touch any production code.
- It does not change the visual prototype: the four bodies, their morphologies and the fixtures stand.
