---
name: outcome-rendering-gate
description: Gate product UI, graphics, animation, effect, path, and interaction implementations on the user-visible outcome rather than controls or local state. Use when a request says add, show, change, animate, apply, or edit a visible product behavior; especially after feedback that an implementation only added UI, does not visibly change the view, or breaks under zoom.
---

# Outcome Rendering Gate

Use this before writing product UI or renderer code.

## Close the outcome first

Write one sentence in this form:

`User action -> semantic source -> evaluation -> rendered representation -> observable result.`

For a PathOp, this is not “a PathOp button is selectable.” It is “selecting PathOp changes the selected path in the Stage, and its curve/fill/stroke remains correct at the requested zoom.”

Name the semantic owner and exact representation at every arrow. If any arrow is unknown, inspect existing code or return the exact gap. Do not build a UI-only substitute.

## Rendering preflight

Before implementation, answer all of these from source:

- Is the source a path with Bezier handles, a polygon, an image, or a mesh?
- Does evaluation preserve that representation, or deliberately lower it? Name the lowering.
- How are fill, stroke, joins, clipping, and transforms rendered?
- What bounds visual error under zoom? Fixed point-count sampling is not an answer.
- Is Preview and Export the same evaluation route?
- What existing tessellator, visualizer, or platform renderer can be reused?

If a required visual property cannot survive the available representation, stop before adding controls. Report the missing boundary and adopt or validate a renderer first.

## Build only a complete thin slice

Implement the smallest path that preserves every arrow of the outcome sentence. A catalog, selector, placeholder, fixture, or local state is allowed only when the user explicitly asked for that limited artifact and its label says so.

Never claim an evaluated visual result when it is produced by a second, approximate display path. Do not add fixed-resolution flattening as a product renderer. Use an existing adaptive tessellation or define a pixel-error bound and its owner before lowering curves.

## Verify the user result

Before reporting success, capture evidence for:

1. the action changes the intended Stage view;
2. at least one materially different input changes it differently;
3. normal and enlarged views meet the stated representation requirement;
4. the semantic write/evaluation boundary is distinct from transient UI state.

Pick the cheapest evidence route that can still fail:

- **Semantic boundary** — a headless test asserting the action writes the intended document change. Always required; this is what makes 1, 2, and 4 falsifiable.
- **Rendered appearance** — capture the running window *alone*. A full-desktop screenshot is not evidence: the reader cannot tell what they are looking at.
- **Interaction itself** (drag, hover, press-vs-click) — only this tier justifies desktop automation. Before spending a turn on it, check whether this host's toolkit answers accessibility at all.

Do not re-derive the capture route each session. The host-specific commands, and which toolkits do not answer accessibility, are recorded in the repo `AGENTS.md`; read that instead of a automation skill document.

State separately what is fixture-only, what reaches persistent product data, and what remains unconnected.

## When corrected by the user

Treat “that is not what I meant,” “it does not move,” or “what happens when zoomed?” as an outcome failure. First answer whether the requested user-visible result is true or false. Then reconstruct the missing arrow in the outcome sentence and revise the preflight; do not merely remove or patch the most recent code.
