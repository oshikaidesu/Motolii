# SDK-S0 Path2D fixture evidence

This directory owns the language-neutral SDK-S0 test fixture boundary.

- `sdk-s0-path2d.fixture.json` is the canonical consumer-neutral data: contract projection, fixed offset profile, equivalence tolerance, fixture budget, Path2D inputs, distances, times, positive/negative case IDs, and expected diagnostic reason/target pairs.
- The fixture label is `Offset Path`; the contract is `Path2D(canonical local) -> Path2D(canonical local)` with a finite canonical-length distance.
- `crates/motolii-doc/tests/sdk_s0_path2d_semantics.rs` is one typed, executable consumer. LANG-TS-F0 is the planned second consumer of the same JSON data.
- `motolii_doc::pathgeom::apply` is the reused native oracle; no second offset implementation or new numeric golden is defined here.
- The JSON is test representation only. It must not be read as a Document, product serde format, Vism package, plugin manifest, public Rust API, or public TypeScript contract.
- Positive coverage is S0-P1 through S0-P4; negative coverage is S0-N1 through S0-N7.
