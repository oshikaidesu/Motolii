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

| | Role | Seat holds | Not | Dock unit |
|---|---|---|---|---|
| **Browser** | discover · choose · place | surfaces: Create, Effects, Colors, Fonts, Media (Leaf tabs) | each surface as its own workspace concept | the seat |
| **Stage** | direct manipulation · view | surfaces: composition(s); view mode: Camera / User (a mode, not a tab) | Camera as an independent tool | open |
| **Inspector** | the selected thing's face | instruments chosen by selection: Transform, Layout, Camera, Text, Fill, Matte, effect cards (stacked, not tabbed) | a Desk; a tab group for desks | open |
| **Desk** | focused relationship / operation workspace | tools: Blend, Depth, Ease, History, Notes; opened from the value being edited, then closed | Inspector instruments; a permanent seat (Home has none) | open |
| **Timeline** | time structure · editing | surfaces: Timeline, Graph, Console (the seat's own header) | Graph/Console as independent Dock panels | open |

Stage PRESERVE: production Stage interaction. Timeline PRESERVE: its full production input vocabulary.
"open" = not decided; do not decide it in code.

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
