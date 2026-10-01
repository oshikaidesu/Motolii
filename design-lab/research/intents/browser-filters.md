# Intent cards: Media / Browser filters (and effects/presets browsing)

Scope: intent and interaction contract only, in product language. The old Motolii is a counter-example; nothing of its look or code is a spec. Code refs are given only to say whether the old app did a thing well or poorly. Items marked [inferred] are not stated in any page I read.

Source legend (pages actually fetched, 2026-10-02):
- [AM12] https://www.ableton.com/en/live-manual/12/working-with-the-browser/ (Ableton Live 12 manual, "Working with the Browser")
- [AM11] https://www.ableton.com/en/live-manual/11/managing-files-and-sets/ (Live 11 manual)
- [AM12b] https://www.ableton.com/en/live-manual/12/managing-files-and-sets/ (thin extract only)
- Not fetchable (403): help.ableton.com "Browser and Tags in Live 12 FAQ" and "The Live 12 Browser". I only saw their search-result snippets; claims from those are tagged (snippet).
- Search-result summaries only (no page opened in full): Resolve Media Pool (steakunderwater.com Resolve 18 manual part474/part516 snippets; smart bins articles), Premiere Pro Project panel (helpx.adobe.com / premiumbeat snippets). Splice, Cavalry asset browser, Finder tags and Adobe Bridge were NOT researched; I make no claims about them.
- Old code root: /private/tmp/wt/tok/motolii/ui/lib/browser (paths below relative to it). Inventory: design-sense-lab/research/inventory/browser.md (secondhand, I spot-checked lines).

---

## 1. Sources / Places

### 1.1 One fixed list of "where things live"
- **Intent**: I can say which pool of stuff I am looking in (this project, built-in, my folders) before I look for anything.
- **Contract**:
  - The sidebar lists the places; exactly one is chosen at a time and its contents fill the list.
  - The places include the project's own items, built-in items, and user-added folders.
  - User-added folders can be added from the panel itself.
- **Where it came from**: [AM12]: Places labels = "your Live Packs, User Library, Current Project folder, and any external folders that you've added to the browser" (plus Splice/Cloud/Push when enabled). Old: `media/catalog_controls.dart:168` "SOURCES" chip row (This project, Bundled, All, Favorites, Recent, folder chips, "+ Folder…", "+ Import…"). Old did it MIXED: it did put sources, favourites and recents in one row of equal chips, so a place, a saved view (Favorites) and a verb ("+ Import") sit side by side with the same weight. Starting on "This project" when the project already holds assets (`media/seat.dart`) is a good touch; offline sources showing "·off" is good (no silent disappearance).
- **Candidate for the owner?** Yes. Core "where am I looking". Open question for the owner: sidebar (Ableton) vs a row of chips (old).

### 1.2 Sidebar is separate from the content list
- **Intent**: Choosing a place and choosing an item are two different gestures in two different areas.
- **Contract**:
  - Left/right arrow moves focus between the sidebar and the content pane; up/down moves within one.
  - In the tree, left/right also closes/opens folders.
- **Where it came from**: [AM12]: "Close and open folders, or move between the sidebar and content pane with the left and right arrow keys"; up/down arrows scroll within a label. Old: sources were chips above the list, so no sidebar/content keyboard split existed (its arrow keys only move the item pick, `media/media_browser.dart:235-238`). Old did not have this.
- **Candidate for the owner?** Yes, only if a sidebar is chosen; skip otherwise.

### 1.3 Folder navigation inside a place
- **Intent**: Drill into folders and get back out without losing my place.
- **Contract**:
  - A breadcrumb shows where I am; clicking any crumb jumps there.
  - Sub-folders are listed with item counts.
- **Where it came from**: Old `media/catalog_controls.dart:377` "FOLDER" row: "/" crumbs plus sub-folder chips with counts, only when one source is in view. Ableton shows it as an unfoldable tree [AM12 "unfold ... just as if it were a folder" from the thin extract, [AM12b]]. Old did it OK (crumbs + counts are useful) but as a third row of chips it costs vertical space; in a narrow panel this stacks to 4 control rows before any content [inferred from the row structure].
- **Candidate for the owner?** Yes (breadcrumb with counts); the "third chip row" form: No.

---

## 2. Type filters

### 2.1 Filter by kind of media (image / video / audio / 3D / environment)
- **Intent**: Show me only the sort of thing I can use right now.
- **Contract**:
  - Kind chips: All, image, video, audio, 3D, 360. Choosing one narrows the list; All resets.
  - The active kind is visibly on; the count updates.
- **Where it came from**: Ableton has no kind chips; it splits by Library labels (Sounds, Drums, Instruments, Audio Effects, MIDI Effects, Clips, Samples ... [AM12]) which are "type" as a place. Old: `media/catalog_controls.dart:131,199` kind chips; the code accepts a set of kinds but the chips call `session.choose(kinds: {kind})`, so it is single-choice only (line 200). Did it POORLY on one point: the data model allows several kinds but the UI cannot express "image + video"; and the glyph-only chips (picture, play, note) lean on recognition. Did well that "All" is a real state, not a missing one.
- **Candidate for the owner?** Yes. Decide single vs multi; Ableton's own rule (see 3.2) suggests multi within a group.

---

## 3. Tags / categories and how filters combine

### 3.1 Tag groups ("filter view")
- **Intent**: Narrow by describing what I want (kind of sound / look), without knowing where a file is stored.
- **Contract**:
  - Filters are groups; each group holds a set of tags; clicking a tag narrows the results.
  - Tags describe the item (type, character, etc.) independent of folder.
  - Results are the same list, just narrower; no separate "filter results" screen.
- **Where it came from**: [AM12]: "Each filter group contains a set of tags that you can click on to narrow the results that appear in the content pane." (Live 12 headline feature; snippet of the help article: "each filter is a set of on/off tags for specific characteristics".) Old: only a weak form. Effects/Create had families (Blur, Light, Color, Distort...) as a one-row class strip (`classify.dart:132`, content `things.dart:272`) and a `tag:` query syntax (`things.dart:203`, hidden power feature); Media had no tags, only kind + source + folder. Old did the class strip WELL as a quick one-axis browse; it did the tag part POORLY (invisible, typed-only).
- **Candidate for the owner?** Yes. The single most Ableton-specific idea.

### 3.2 How several selected filters combine
- **Intent**: Predictable narrowing: adding a filter never makes the list bigger by surprise.
- **Contract**:
  - Several tags inside one group: need Cmd/Ctrl-click to select more than one in that group ("To select multiple tags in one group, use the Ctrl (Win) / Cmd (Mac) modifier when clicking on tags" [AM12]); a plain click selects just that tag.
  - Search terms combine as AND: "'acoustic bass' ... all acoustic bass sounds — not all acoustic sounds and all bass sounds" [AM11].
  - Filters across groups combine as AND (narrow) [inferred; the page states "narrow the results" but I did not read an explicit cross-group rule, and I did not read whether multi-select within a group is AND or OR].
- **Where it came from**: [AM12], [AM11] as quoted. Old: search = whitespace tokens, every token must match (`search.dart:30-33`) = AND, same as Ableton; the old app's class strip was single-choice, so combination never arose (Favorites, Recent, a colour, a class were all alternative "views", not stackable). Old did it POORLY here: in the Create/Effects shelf you cannot say "Blur AND Favorites".
- **Candidate for the owner?** Yes. Pick: always-AND, plain click = single, modifier = multi.

### 3.3 Search by tag
- **Intent**: Reach a tag from the keyboard without leaving the search box.
- **Contract**:
  - Typing `#` plus a name in search filters by that tag (e.g. `#Drums`).
- **Where it came from**: [AM12]: "typing # into the search bar followed by the tag name, e.g., #Drums". Old: `tag:`, `kind:`, `family:`, `cap:`, `source:`, `duration:5-30` prefixes, only in Create/Effects (inventory `things.dart:203`; per inventory, not re-read). Old did it MIXED: far more powerful than Ableton, but a typed mini-language with no on-screen chip, so nobody discovers it; Ableton's `#tag` is the lighter form.
- **Candidate for the owner?** Maybe. Yes for `#tag`; the `kind:/duration:` language: No unless each term also shows as a chip (see 3.4).

### 3.4 Showing that a filter is on, and clearing it
- **Intent**: I can always tell I am looking at a narrowed list and undo it in one move.
- **Contract**:
  - A results bar shows how many filters are applied ("The Results bar shows how many filters are applied at any given time" [AM12]).
  - One Clear action removes all selected tags and the search text ("Use the Clear button in the Results bar to remove any selected tags as well as entered search terms" [AM12]).
  - Zero-filter state looks plainly "unfiltered".
- **Where it came from**: [AM12]. Old: a "class chip" `<class> x` appears in the header only when narrow (`classify.dart:217`; inventory `panel_chrome.dart:190`); wide mode showed the chosen class as a raised pill in the strip, which is not distinguishable from "just a tab". There was no "N filters" count nor single clear for source+kind+search together. Old did POORLY: three independent states (source, kind, search) with no combined indicator.
- **Candidate for the owner?** Yes.

### 3.5 Tags on the selected item (add / see / auto)
- **Intent**: Tag things my way, from where I am looking at them.
- **Contract**:
  - A "Quick Tags" area shows the tags of the selected item; "Add…" then type a name creates/assigns a tag.
  - Short user samples get tags assigned automatically by analysis.
- **Where it came from**: [AM12]: Quick Tags "any tags assigned to the selected item"; "assign additional tags by clicking Add… and then typing the name of a tag"; auto-tagging: "Live periodically runs a sound analysis ... assigns tags to user samples that are up to 60 seconds long." Old: nothing (no user tagging; only 7 colour collections).
- **Candidate for the owner?** Maybe. Yes if user media will be big; auto-tagging is a large feature [inferred] and probably No for v1.

---

## 4. Search

### 4.1 Live text search
- **Intent**: Type a few words and see only the matching things, immediately.
- **Contract**:
  - Results update on every keystroke; no Enter needed.
  - Multiple words are AND, case-insensitive.
  - Results are in alphabetical order by default ("display results in alphabetical order" (snippet)).
  - Esc clears the text first, then leaves the field.
- **Where it came from**: [AM11] (AND rule), snippet (alphabetical). Old: `search.dart:30-33` token AND; `/` or Cmd/Ctrl+F opens, Esc clears then leaves (`search.dart:58-69`). Media: "Search assets" box queries on every change (`media/catalog_controls.dart:203`). Old did it WELL: the Esc-twice behaviour and the "/" shortcut are right. It did the search UI POORLY in that Create/Effects/Colors/Fonts fold the field behind a magnifier icon (inventory `panel_chrome.dart:39`) so search was hidden by default, while Media always showed a box: two behaviours for one concept.
- **Candidate for the owner?** Yes.

### 4.2 Search scope = what I am looking at
- **Intent**: The search never silently looks somewhere I cannot see.
- **Contract**:
  - Search covers the chosen place (and its sub-folders); changing place keeps the query.
  - Matching includes metadata/tags, not only the name (Premiere's filter "matches names or metadata", snippet).
- **Where it came from**: Premiere Project panel search "display only clips with names or metadata matching ... even if they are inside a closed bin" (snippet, helpx). Old: not clearly specified; whether the query survives a source change I did not verify.
- **Candidate for the owner?** Yes (decide whether search is per place or global).

### 4.3 Saved search
- **Intent**: Keep a filter I use a lot as a place of its own.
- **Contract**: A saved filter appears in the sidebar and re-evaluates as items arrive.
- **Where it came from**: Resolve "Smart Bins are saved searches that live in your Media Pool sidebar" (snippet, 4kshooters). Old: "saved searches" existed as a class in the shelf views (inventory `user_state.dart:7`); I did not see a UI to create one.
- **Candidate for the owner?** Maybe.

---

## 5. Favourites, collections, recents

### 5.1 Colour-coded collections (Ableton "Collections")
- **Intent**: Mark things I care about with one keystroke and get back to them with one click.
- **Contract**:
  - Seven colour labels; select an item and press 1-7 to assign, 0 to reset; also via the item's context menu.
  - One item can hold several colours; at most three are shown on the row.
  - Each colour is a place in the sidebar; the first ("Favorites") is the default favourite.
- **Where it came from**: [AM12]: "assign Collections labels via a selected item's context menu, or by using the number key shortcuts 1 through to 7. Use 0 to reset color assignments"; "no more than three of those colors will be shown in the content pane." [AM11]: labels "quickly organize and access particular browser items (for example, your favorite or most-used items)". Old: seven collections (Favorites, Orange... Gray) mirrored from Ableton (`user_state.dart:75`), assigned ONLY through the context menu "Add to Favorites / Orange..." and the `F` key for favourite (`media/media_browser.dart:228`). Old did it POORLY: copied the colours but not the 1-7 / 0 keys (the fast part), and it put Favorites/Recent/colours in the class strip next to normal categories. Honest note: the owner said the filters were modelled on Ableton, and this is the clearest case.
- **Candidate for the owner?** Yes (collections), with the number-key assignment as the part worth restoring.

### 5.2 Recent
- **Intent**: Get back to what I just used without searching.
- **Contract**:
  - A "Recent" place lists the last items I used (placed in the project), newest first.
  - It is a view, not a tag; it cannot be edited by hand.
  - Hidden/dim when empty.
- **Where it came from**: Ableton has a Recent concept in its sidebar [inferred from the help snippets; I did not read the exact text in a fetched page]. Old: Recent = last 12 per shelf (`user_state.dart:15 recentLimit = 12`, push on use at :99); chip dims when empty (`media/catalog_controls.dart:168`); Create board also shows a "Recent" section (max 8, hidden while searching, inventory `board.dart:16`). Old did it WELL: small cap, dim when empty, hidden during search.
- **Candidate for the owner?** Yes.

---

## 6. Sort and view modes

### 6.1 Sortable columns in list view
- **Intent**: See facts and order by the one that matters.
- **Contract**:
  - List view has columns; clicking a column name sorts by it, click again reverses; the active column shows an arrow.
  - More columns can be shown, and reordered by dragging a column name.
  - Narrow panel shows fewer columns.
- **Where it came from**: [AM12]: "select additional columns to be displayed", "reorder them by clicking on a column name and dragging", "Sorting occurs by clicking on the column name". Old: NAME, TYPE, LENGTH, SIZE (width >= 300), DATE (width >= 420), sortable except LENGTH (`media/list.dart:7`). Old did the width-driven dropping WELL (columns vanish by width, not wrap). Poorly: length not sortable; user cannot choose columns.
- **Candidate for the owner?** Yes.

### 6.2 List vs Thumbnail
- **Intent**: Pick between "read facts" and "look at pictures".
- **Contract**:
  - A two-way switch (list / thumbnails); the choice persists.
  - Thumbnail size is adjustable in thumbnail mode.
  - Selection survives the switch.
- **Where it came from**: Premiere: icon vs list view, toggle Shift+\ (snippet, helx/premiumbeat); Resolve: Thumbnail / List / Metadata views (snippet; Metadata = "a card with a thumbnail and basic clip metadata", a middle level). Old: tabs List | Thumbnail | Explore + size slider 44-140px (`media/media_browser.dart:383`, `fluid.dart:29`); the List<->Thumbnail change animates faces 200 ms ease-out and the selected face scrolls to centre. Old did it WELL (the continuity of the same faces moving). Ableton has no thumbnail view for samples (list only, with a Preview tab) [inferred].
- **Candidate for the owner?** Yes (list + thumbnail). Resolve's middle "metadata card" view is an option to consider.

### 6.3 Explore (map of related assets)
- **Intent**: Find things by resemblance rather than by name or folder.
- **Contract**:
  - A pannable/zoomable map of the current results; items near each other are related.
  - Choosing an item brightens it and its neighbours and dims the rest; Global vs Local (1 or 2 hops).
  - It refuses (with a message) when too many items are in view (> 300).
- **Where it came from**: No precedent found in Ableton, Premiere or Resolve; old-only. Old: `media/explore/graph.dart:199`, `fluid.dart:313`, bar `graph.dart:364`; per the inventory `exploreLayout` returns no hubs and "spokes" `links` are always empty, so the relation lines may largely be placeholder. Old did it AMBITIOUSLY but unproven: the 300-item cap makes it unusable on exactly the large libraries it would help, and its relations (similar / folder / source / type / project / duplicate) were not verified to be real. I am not saying it was bad, only that I cannot show it worked.
- **Candidate for the owner?** Maybe. Ask the owner whether this was something they liked; it is the only one with no outside precedent.

---

## 7. Previewing

### 7.1 Preview toggle and auto-play
- **Intent**: Hear/see a candidate by just selecting it, and switch that off when I don't want noise.
- **Contract**:
  - A Preview switch (next to the Preview tab) turns auto-preview on; with it on, selecting an item previews it.
  - With it off, Shift+Enter or Right-arrow still previews the selected item.
  - A "Raw" option: off = start at the next bar to stay in time with the playing transport; on = play at original tempo, unsynced.
  - Preview volume is its own knob in the mixer ("Preview/Cue Volume"), separate from the project's main output.
- **Where it came from**: [AM12] (all four bullets, quoted above); [AM11] ("activate the Preview switch next to the Preview Tab"; "Preview Volume knob"). Old: Space toggles a preview panel in place under the board (`media/media_browser.dart:227`, `media/preview.dart:25`); no auto-play toggle, no separate preview volume found in the files I read [inferred: grep for volume was not done]. Old did it MIXED: a big preview panel with zoom/pan, 3D turn, waveform with moving line is rich (`preview.dart:103`), but it takes panel space and is a mode, not a "just select it" audition; sync-to-bar is audio-only and probably not relevant for pictures.
- **Candidate for the owner?** Yes (toggle + separate volume if audio exists); the bar-sync part: No for visual media [inferred].

### 7.2 Hover/skim preview on thumbnails
- **Intent**: Look inside a clip without opening it.
- **Contract**:
  - In thumbnail view, moving the pointer across a clip scrubs through it left-to-right (media start to end), no click needed.
  - Moving off the clip returns it to its poster frame.
- **Where it came from**: Resolve: "when you move the pointer over a clip's icon, DaVinci Resolve automatically scrubs" (snippet, steakunderwater part474); Premiere Icon View hover scrub: "scrubbing left to right to preview from the Media Start (left) to the Media End (right)" (snippet). Old: a thin scrub strip at the foot of a video face; the frame only changes while the pointer is PRESSED on that strip (`media/fluid.dart:603-694`, "dragging [the face] never scrubs"). Old did it POORLY relative to precedent: it requires press, not hover, and a 2-4 px target. The inventory says hover = outline only.
- **Candidate for the owner?** Yes. Hover-scrub is a standard NLE expectation.

### 7.3 Audition by arrow keys
- **Intent**: Move through candidates with the keyboard and see/hear each one instantly.
- **Contract**:
  - Up/Down moves the selection; with auto-preview on, each step previews.
  - Enter/double-click uses the item.
- **Where it came from**: [AM11] Hot-Swap: "pressing the up or down arrow key moves to the next file in the content pane, and pressing Enter or double-clicking the file loads it". Old: arrows move the pick; `Home`/`End` too (`media/media_browser.dart:233-238`); it did not preview as it went.
- **Candidate for the owner?** Yes.

---

## 8. Using an item

### 8.1 Drag into the project
- **Intent**: Drop a thing where I want it.
- **Contract**:
  - Dragging an item onto a track/timeline places it at the drop point; dropping into an empty area creates a new track/layer.
  - The dragged item shows a ghost and the source stays visible but dimmed.
- **Where it came from**: [AM12]: "Drag-and-drop adds items to tracks or creates new tracks." Old: drag carries the asset to the Timeline; ghost 85%, original 40% (`media/fluid.dart:165` per inventory). Old did it WELL (visible ghost + dimmed original).
- **Candidate for the owner?** Yes.

### 8.2 Double-click / Enter = use
- **Intent**: Use the thing without dragging.
- **Contract**:
  - Double-click or Enter loads/places the selected item (media = place at playhead/selection; effect = apply to selected layers).
  - Single click only selects (and optionally previews).
- **Where it came from**: [AM12]: "Double-clicking or pressing Enter loads devices/samples." Old: media double-click places, Enter places (`media/fluid.dart:392,408`; `media_browser.dart:229`); but effect/create tiles apply on a SINGLE click (inventory `browser.dart:102`), and fonts: click dresses the active layer, double-click makes a new text layer (`browser/browser.dart:90-91`). Old did it POORLY: single click = act on effect tiles but = select on media, so the same gesture means two things in sibling shelves. Also a single-click apply is risky because it is destructive-ish and also fights with browse-by-click.
- **Candidate for the owner?** Yes; the contract to settle: one meaning of click across all shelves.

### 8.3 Hot-swap (try alternatives on the thing already in place)
- **Intent**: Replace one effect/preset on a layer with another by stepping through candidates, hearing/seeing each in place.
- **Contract**:
  - A key (Q in Ableton) links the browser to the selected target; Up/Down plus Enter (or double-click) loads the next candidate onto that target, replacing the previous.
  - Leaves on Q again, Esc, or switching view.
  - Works while playback runs.
- **Where it came from**: [AM11]: "Hot-Swap Mode can be toggled on and off with the Q key"; "pressing the up or down arrow key moves to the next file ... Enter or double-clicking the file loads it"; "The link ... will be broken if a different view is selected, or if the Q key or the Hot-Swap button is pressed again. Hot-swapping can also be cancelled with a press of the ESC key". [AM12] only mentions that candidates can be double-clicked "as the music plays" (extract); it did not state the Q key itself (the fetch found no Q; the Live 11 page does). Old: nothing equivalent. Closest: "Replace selected layer" menu row for media (`session/media_actions.dart`). Not done.
- **Candidate for the owner?** Yes for effects/presets: very strong for "try a look".

### 8.4 Dim what cannot be used right now
- **Intent**: Don't let me click something that will do nothing.
- **Contract**: Items with no valid target are visibly disabled and say why on hover. [inferred]
- **Where it came from**: Old: tiles with no host binding drawn at 48% opacity and still clickable (`browser/browser.dart:122`); inventory notes 8 of 25 Create tiles are dimmed this way. Old did it POORLY: a dim with no explanation reads as "broken", and the shelf shipped a third of its tiles inert.
- **Candidate for the owner?** Counter-example. Keep the intent, not the practice.

---

## 9. Effects / presets browsing

### 9.1 Effect tiles show the effect on a sample picture
- **Intent**: See what an effect does before applying it.
- **Contract**:
  - Each effect tile is a thumbnail of that effect on a standard image (or, better, on the user's current frame).
  - Name under each; applying is as 8.2.
- **Where it came from**: Old `effects/shelf.dart:79 FxPainter` + `EffectScene` (a fixed sample picture with the effect painted on it; host snapshot where available, `browser/browser.dart:138`). Ableton equivalent is the Preview tab for an audio item. Old did it WELL as intent: the picture is the name. Poorly: sample scene is fixed, not the user's frame, per the inventory.
- **Candidate for the owner?** Yes (live preview on the actual layer is the stronger form [inferred]).

### 9.2 Families as quick axes (Blur, Light, Color, Distort...)
- **Intent**: Browse effects by what they do.
- **Contract**: One-axis category list; "All" resets; counts shown; combinable with search and Favorites.
- **Where it came from**: Ableton Library labels and filter groups [AM12]; Old `classify.dart:132` class strip (+ narrow chip). Old: WELL for one axis, POORLY for combining (see 3.2).
- **Candidate for the owner?** Yes.

---

## 10. Empty / no-result / failure states

- **Intent**: When there is nothing, say why and offer the fix.
- **Contract**:
  - No results: say that no item matches and show a one-click way back (Clear).
  - Empty place: say the place is empty and how to fill it (drop files / add folder).
  - Missing file: mark the item and offer "Locate...".
  - Dragging files over the window shows a drop-to-import veil.
- **Where it came from**: Ableton: Clear button in the Results bar [AM12]. Old: "Nothing matches." muted text (`media/media_browser.dart:354`) and `No effect matches "<q>".` (inventory), no Clear button in it; "Drop to import" veil (`media/catalog_controls.dart:215-227`); missing-file "!" badge + "Locate file…" (inventory); source chip "·off". Old did POORLY on no-result (dead-end text, no action) and WELL on the drop veil and missing-file handling.
- **Candidate for the owner?** Yes.

---

## 11. Narrow-panel behaviour

- **Intent**: The panel stays usable when docked small.
- **Contract**:
  - Width drives what shows: list columns drop in a fixed order; thumbnails reflow to fewer columns; filters collapse into a single control with the active count.
  - Height low enough: switch to a one-row strip.
  - Nothing wraps or truncates mid-word [inferred].
- **Where it came from**: Old: three morphologies wide / narrow (<260 px: class chip) / strip (<170 px high) (`panel_chrome.dart:143`), list columns at 300/420 px (`media/list.dart`), preview side-by-side at >= 520 px else stacked, tile captions hidden under 60 px. Ableton does not document this (desktop sidebar that is resizable) [inferred]. Old did it WELL on the principle (explicit, width-keyed morphs); but the Media panel stacked Sources + Types + Folder + Search above the list, so the controls consumed the narrow panel (see 1.3).
- **Candidate for the owner?** Yes, as a requirement.

---

# Summary

## What Ableton does that is worth taking
1. One chosen place at a time, with a sidebar apart from the content list, with Left/Right to move between them [AM12].
2. Filters are tag groups that narrow, and several in one group need Cmd/Ctrl-click; always "narrow" (AND), never widen [AM12][AM11].
3. A results bar that says how many filters are on, plus ONE Clear for tags and text together [AM12].
4. Search is live, AND across words, and `#tag` reaches a tag from the keyboard [AM12][AM11].
5. Colour collections with keys 1-7 and 0 to reset; up to three colours displayed [AM12].
6. Preview as a switch with a fallback key (Shift+Enter / Right), a separate preview volume, and the "Raw" choice between synced and as-is [AM12].
7. Hot-Swap: Q to link to the target, Up/Down + Enter/double-click to try each candidate in place, Esc/Q to leave [AM11].
8. Double-click / Enter = use; drag = place or create new track [AM12].
9. Click a column to sort; choose and reorder columns [AM12].

## What the old Motolii added or changed
Good:
- Esc clears search then leaves the field; `/` and Cmd+F focus it (`search.dart`).
- Same faces move between List and Thumbnail (200 ms), size slider, selection stays.
- Narrow morphs keyed to width, with columns dropping in order.
- Missing-file badge + Locate, offline source stays visible, drop-to-import veil.
- Recents capped and dim when empty; start on "This project" when it has assets.
- Effects tiles that show the effect on a picture.
Bad / unproven:
- Copied the 7 colours but not the 1-7/0 keys; Favorites, Recent, colours and categories lumped into one class strip as alternatives, so no stacking (cannot say "Blur AND Favorites").
- Three or four control rows stacked above media in a narrow panel.
- No combined "filtered" indicator and no single Clear; no-result text is a dead end.
- Kind filter single-choice only although the model allows sets.
- Tag/kind query syntax exists but is invisible (no chips).
- Single click applies on effects but only selects on media; dimmed (inert) tiles without a reason.
- Scrub needs a press on a 2-4 px strip rather than hover.
- Search hidden behind a magnifier in four shelves but always visible in Media.
- Explore: no outside precedent, capped at 300 items, relation lines partly empty per the inventory.
- No hot-swap, no audio preview volume or auto-preview switch found.

## Caveats
- Ableton help-centre pages returned 403; those claims are tagged "(snippet)" or omitted.
- I did not research Splice, Cavalry's asset browser, Finder tags, or Adobe Bridge. Resolve and Premiere claims come from search-result summaries, not full pages.
- [AM12] did not state the Hot-Swap Q key in my fetch; the Q key is quoted from [AM11] (Live 11 manual). Whether the same holds in 12 is probable, not read.
