# Product Direction — the IR between a UI instruction and code

A UI instruction is compiled here before any code: natural language → Direction IR → code.
Before a UI change, state it in one line with the verbs below, e.g.
`PRESERVE Stage semantics + PROMOTE Browser tab grammar + REHOUSE Timeline surfaces`.
If the instruction does not compile into these words, ask — that is the only place to ask.

## Four layers

| Layer | Examples | Agent may |
|---|---|---|
| Product invariant | Work stays loudest; an ordinary person can finish a piece | never change |
| Visual grammar | Browser seat strip (Leaf): strip only for >1 surface, identity travels, fold don't miniaturize, header drops what the strip shows | generalize to other seats |
| Product semantics | Camera is a view of the Stage; Desk ≠ Inspector; Graph/Console belong to Timeline | not generalize, not change |
| Current implementation | `docking`, `PanelDef`, Flutter widgets | change when needed |

Generalizing a **grammar** never changes **semantics**: "apply the Browser rule everywhere" means its mechanics, not Browser's structure.

## Verbs

- **PRESERVE** — keep meaning, operations and owner. No re-implementation to match looks.
- **PROMOTE** — lift a mechanic that already works locally into a shared primitive (Browser tab grammar → workspace tab grammar).
- **CONNECT** — route an existing implementation into the production call path. No substitute implementation.
- **REHOUSE** — change only where something appears; content and semantics unchanged.
- **REPLACE** — only when explicitly asked; the old owner is retired in the same change, never both alive.
- **REFERENCE** — learn looks/mechanics from it; do not copy its internal structure.

## Product Home (`product-home.png`)

Authoritative: hierarchy, density, seat geometry, information architecture, interaction promise.
Not authoritative: fixture content, exact text/values, the artwork, whether a capability exists in production.

## Seats

| | Role | Default seat holds | Not |
|---|---|---|---|
| **Browser** | discover · choose · place | panels: Create, Effects, Colors, Fonts, Media | each surface as its own workspace concept |
| **Stage** | direct manipulation · view | panel: Stage; composition(s); view **mode** Camera / User (a mode, not a panel) | Camera as an independent tool |
| **Inspector** | the selected thing's face | panel: Inspector; **instruments** chosen by selection: Transform, Layout, Camera, Text, Fill, Matte, effect cards (stacked, not panels) | a Desk; a tab group for desks |
| **Desk** | focused relationship / operation workspace | **contextual panels**: Blend, Depth, Ease, History, Notes — opened from the value being edited | Inspector instruments; part of the default grouping |
| **Timeline** | time structure · editing | panels: Timeline, Graph, Console | — |

Stage PRESERVE: production Stage interaction. Timeline PRESERVE: its full production input vocabulary.

### Dock unit (decided 2026-09-28)

**Default is composed. Freedom is available. Identity travels.** Panels may travel; seats are current groupings; Product Home is the default grouping.
- Every **panel** can leave its seat, join another seat as a tab, split, or detach to its own window, and keeps its face, identity, controls and semantic state wherever it goes. A seat is only the housing that groups panels now; pulling a panel out may create one, dropping onto a seat adds a tab. Reset Layout returns to the Home grouping.
- Not everything visible is a panel. Instruments (Inspector) and modes (Stage's Camera View) are not promoted to panels for Dock's sake — that is a Decision-boundary change.
- Contextual panels: *where they open from* keeps its product meaning; *once open* they travel like any panel. They are never part of the default grouping.
- The Dock owns placement and travel, once, for every panel (no per-surface mechanics). Each surface owns its meaning and content. The tab grammar's shared primitive covers only the visual mechanics that change for the same reason (Browser, Dock, Timeline, composition tabs), not their navigation.

## Decision boundary

**Connecting an existing meaning is autonomous. Choosing, merging, splitting, promoting or demoting a meaning is a product decision: present it before implementing.**

Stop when a change would:
- pick between existing precedents that disagree (Browser Leaf tabs vs Timeline's own header vs Stage's Camera View mode)
- change a concept's level (Camera mode → panel; Graph label → Dock panel)
- merge separate things into one primitive, or split one owner into several / several into one
- hide, remove or fold something that existed (a filter rail dropped by a responsive rule)
- make something transient permanent, or the reverse (Desk → permanent Dock tab)
- choose between Product Home and current production
- rest on "this is more natural"

Then bring, not "what should I do?", but: **observation → existing precedents → options (A/B/C) → consequence of each**, and say which layer the decision belongs to. Asking this is part of working autonomously, not a failure of it.

## After a decision: decision → primitive → instances

Once a decision is made, the rest is autonomous:
1. **Inventory** every place the same decision governs (all seats, all tab strips, not the one in view).
2. **Find the owner.** Things that will change together for the same product reason share one owner/primitive; things that change for different reasons stay separate owners even when they look alike (Camera View and the Camera Inspector share no owner). If none exists, PROMOTE the most finished instance (e.g. Browser Leaf → the tab primitive), minimally.
3. **Connect every instance** to it; retire the per-instance copies.
4. **Later fixes go to the owner**, never to one instance ("inactive fold looks wrong" is one edit that changes Browser, Timeline and Dock together).

Asking happens once, when the meaning is decided; consolidation, migration and the full check that follow are not questions.

## Product UI authority (decided 2026-09-28, D1)

- **live_hf** (`lib/live_hf`, `scripts/motolii-ui.sh dev`) is the product UI and the presentation authority, with Product Home.
- **Classic and New** (`MOTOLII_SHELL=classic|new`) are **capability migration sources**, not candidates. What is recovered from them is capability, input vocabulary, semantic behaviour and proven mechanisms — never their presentation.
- **proto_hf** (and `lib/proto`) is prototype / reference. Nothing in production depends on it.
- Authority flows forward: Classic/New capability → shared semantic/mechanism owner → live_hf presentation. No dependency points backwards (live_hf never imports Classic/New; New does not import live_hf; production does not import proto).

Migration status per capability: PRESENT · LIVE_BETTER · MISSING_IN_LIVE · PARTIAL_IN_LIVE · OBSOLETE · DECISION_REQUIRED. A widget that exists is not PRESENT; a user trajectory that works is. A Classic widget absent from live_hf is not MISSING if the same user capability works there another way.

**Legacy deletion gate**: a Classic/New file, widget or adapter may be deleted only when every capability it provided is PRESENT, LIVE_BETTER or OBSOLETE — verified in live_hf, not asserted. Any MISSING, PARTIAL, UNVERIFIED or DECISION_REQUIRED keeps it.
