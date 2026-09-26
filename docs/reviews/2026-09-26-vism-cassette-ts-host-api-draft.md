# Vism cassette distribution and Host capability API — draft

Status: **proposal only / no runtime decision**

This note records a direction for experimentation. It does not commit Motolii to a JavaScript runtime, package format, public SDK, or plugin permission model.

## Question

Can Vism become a distributable “cassette”: a small live package that composes Motolii's existing semantic Host capabilities into a new reusable tool, without requiring the author to implement rendering or a custom Inspector UI?

Example: a **Shatter Repeater** cassette could combine an existing repeater/stagger capability with a destruction effect, while exposing only a small authored surface such as Pieces, Amount, Spread, and Seed.

The important property is that the cassette is not a flattened effect. Its composition remains semantic and inspectable.

## Proposed authoring / shipping split

Prefer **TypeScript for authoring** and a constrained **JavaScript bundle for the first executable form**.

Conceptually:

```text
main.ts
  -> typecheck / bundle
  -> main.js

.vism package
  - manifest
  - main.js
  - thumbnail / metadata
  - optional assets
```

The exact archive/container format and extension are intentionally undecided.

TypeScript is an authoring convenience and SDK surface, not the definition of Vism semantics. A future no-code cassette builder or generated recipe should be able to target the same semantic layer without pretending to be TypeScript source.

## Host capability principle

Do **not** expose Rust internals or a generic raw-document API.

Expose a constrained semantic Host vocabulary corresponding to capabilities Motolii intentionally owns, for example:

- group / composition
- transform
- repeater / stagger
- effects
- matte / mask
- blend
- layout / relations
- timeline / keys / easing
- text and media references

The concrete list is not approved by this proposal.

A useful test is:

> If the product UI can perform a semantic operation, can a cassette compose that operation through a typed, bounded Host capability?

This does not imply that every UI action becomes public API.

## Cassette parameters and UI

Cassette authors should declare meaning, not build Flutter/HTML UI.

Illustrative TypeScript only:

```ts
defineVism({
  inputs: {
    pieces: int({ default: 12 }),
    amount: number({ default: 0.5 }),
    seed: seed({ default: 42 }),
  },

  build(ctx) {
    const repeated = ctx.host.repeater(ctx.input, {
      count: ctx.inputs.pieces,
    });

    return ctx.host.effect(repeated, "Destruction", {
      amount: ctx.inputs.amount,
      seed: ctx.inputs.seed,
    });
  },
});
```

The declaration should project into existing Motolii surfaces:

- Browser: identity, metadata and face
- Inspector: generic/semantic Toys derived from declared parameter meaning
- Timeline: animation of exposed values
- Host graph/document: the semantic composition behind the cassette

A cassette should not require a bespoke UI merely to expose ordinary values.

## Live cassette vs one-shot script

Motolii already has a one-shot JavaScript authoring model: script calls materialize ordinary document edits that remain editable afterward.

A cassette is different when the **relationship must remain live**.

```text
one-shot script
JS -> document operations -> ordinary authored result

cassette
recipe + exposed parameters -> retained semantic composition
                               ^ remains editable
```

Both forms are useful. A camera shake, whip-pan or preset animation may be fine as a one-shot generator. A Shatter Repeater whose Pieces value should continue to drive the internal repeater needs retained cassette semantics.

## Capability / permission boundary

The first cassette probe should not receive arbitrary:

- filesystem access
- network access
- shell/process access
- raw GPU/device access
- renderer caches or scheduling
- unrestricted Rust/document internals

If capabilities need explicit declaration, a manifest could eventually declare them, but the manifest schema is intentionally not decided here.

The runtime should remain deterministic enough for document replay, undo, save/load and export. Exact requirements need a separate decision.

## Two deliberately different probes

Before freezing a public SDK, test the same Host boundary with at least two cassettes from different domains.

### Probe A — Shatter Repeater

Compose existing Host capabilities such as repeater/stagger plus a destruction effect. Expose a small set of meaningful parameters.

This tests whether a CapCut-like catalog of finished, one-click creative treatments can be produced from a smaller set of deep native primitives.

### Probe B — camera technique

Use the existing script/document vocabulary to prototype a camera technique such as orbit, shake, dolly or a generated follow/aim sequence without immediately promoting every filming technique into a Rust enum or permanent Host semantic.

This tests a different domain and helps separate:
- a useful one-shot generator,
- a retained cassette,
- and a relationship that truly deserves promotion into a native Relation.

## Promotion rule

**Hack first. Promote later.**

Do not add a permanent Host semantic merely because it can be imagined.

Prototype through scripts/cassettes first where possible. Promote a concept into a native Host capability/Relation when retained semantic identity, editing, performance, interoperability or correctness demonstrates that the Host needs to own it.

## Non-goals of this draft

This proposal does not decide:

- the final `.vism` archive format
- whether shipped code is always JavaScript
- the JS engine/runtime
- TypeScript compiler/bundler choice
- public package registry or marketplace
- signing/trust model
- capability manifest schema
- dependency resolution
- hot reload
- custom UI extensions
- network/filesystem permissions
- whether recipes eventually become a non-JS IR
- exact Vism manifest compatibility/versioning
- which Host capabilities are public

## Acceptance test for the next investigation

A future spike is interesting if **both** Shatter Repeater and a camera-technique cassette can be expressed cleanly through the same small semantic Host API, without:

1. exposing renderer/Rust internals,
2. adding cassette-specific privileged escape hatches,
3. requiring bespoke UI for ordinary parameters,
4. flattening the resulting composition into an opaque raster/effect,
5. forcing every experimental concept to become a permanent native semantic.

If that fails, revise the capability boundary before expanding the SDK.
