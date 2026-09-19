---
name: wgsl-wgpu
description: WGSL and native wgpu 29 / naga 29 / wesl 0.4.2 contract for Motolii's shelf (vism/*.wgsl, block_program.rs, effect programs). Use when writing or debugging a shader, a buffer layout, a compute dispatch, a bind group, or a "rejected:" line from zz_shelf. Not the browser: no canvas, no requestAdapter, no JS.
---

# WGSL on native wgpu (Motolii)

Motolii compiles WGSL text three times: **wesl 0.4.2** links `import package::…` (no mangling, `validate: false`,
block_program.rs:208-212), **naga 29** parses + validates (`validate()`, block_program.rs:431-439), then
**wgpu 29** builds the pipeline. A shader that passes step 2 can still fail step 3 on this device.

## 一次資料

- WGSL spec: https://www.w3.org/TR/WGSL/ (draft https://gpuweb.github.io/gpuweb/wgsl/) — layout §"Memory Layout", reserved words §"Keyword and Identifier Tokens"
- WebGPU spec: https://www.w3.org/TR/webgpu/ (draft https://gpuweb.github.io/gpuweb/) — limits, formats, validation rules wgpu copies
- wgpu 29: https://docs.rs/wgpu/29 — `Features`, `Limits`, `TextureFormat`, `PipelineCompilationOptions`
- naga 29: https://docs.rs/naga/29 — `front::wgsl`, `valid::{Validator, Capabilities}`, `keywords::wgsl::RESERVED`
- WESL: https://wesl-lang.dev (spec: /spec/Imports, /spec/ConditionalTranslation); crate https://docs.rs/wesl/0.4.2
- Local spec search: `sh .claude/skills/wgsl-wgpu/references/gfx-rs-webgpu-specs/download.sh` → `target/claude/{webgpu,wgsl}-spec.bs`; cite `#anchor` URLs, not line numbers

## 契約

**Data layout** (WGSL host-shareable; compute offsets, never assume "16 for everything"):

| type | align | size | note |
| --- | ---: | ---: | --- |
| f32 / u32 / i32 | 4 | 4 | `bool` is not host-shareable — pass `u32` |
| vec2 | 8 | 8 | |
| vec3 | 16 | 12 | a scalar may follow at +12 in storage; `array<vec3f>` stride is 16 |
| vec4 | 16 | 16 | |
| mat3x3f | 16 | 48 | three vec4 columns |
| mat4x4f | 16 | 64 | |
| struct | max member align | roundUp(align, end) | |
| array\<T, N\> | align(T) | N × roundUp(align(T), size(T)) | |

- member offset = roundUp(align, prevOffset + prevSize). `@align`/`@size` only when a binary contract needs explicit padding.
- **uniform** address space adds: array stride multiple of 16, struct members 16-aligned, no runtime-sized arrays.
  Motolii's `block_params: array<vec4f, 6>` (block_program.rs:120) exists because of this rule.
- **storage** uses natural layout; runtime-sized array only as the last member. `arrayLength()` sees the bound range, so pass a logical count too.
- Buffer sizes the host writes must match: `Item` = 48, `Offset` = 32, `Basis` = 48, `BlockHost` 20 → buffer 32 (block_program.rs:49-50, 114-116, 524).
  `write_buffer` offsets/sizes: multiples of 4. Dynamic offsets: `min_{uniform,storage}_buffer_offset_alignment` (256 default). `bytes_per_row` for texture copies: multiple of 256.
- Mixed scalars: write `u32` fields with `to_le_bytes` of a u32, never via `f32` — same value, different bits.

**Entry points / compute**: `@compute @workgroup_size(X, Y, Z)` per entry; guard `if gid.x >= count { return; }` before any access (block_program.rs:196);
total invocations ≤ `max_compute_invocations_per_workgroup` (Motolii asks 256), each axis ≤ `max_compute_workgroup_size_{x,y,z}` (256/16/1),
dispatch axis ≤ `max_compute_workgroups_per_dimension` (65535) — headless.rs:39-45. Beyond that: add a base index and dispatch again.
`workgroupBarrier()`/`storageBarrier()` only in uniform control flow; a barrier orders one workgroup, a pass boundary orders the whole dispatch.

**Bindings**: `@group(g) @binding(b)`; `var<uniform>`, `var<storage, read>`, `var<storage, read_write>`. A vertex/fragment stage may not bind
`read_write` storage without the feature — declare a separate `read` binding for render. BindGroupLayout `visibility` and `ty` must match the WGSL
declaration exactly (block_program.rs:442-460). Immediates (`immediate_size` in PipelineLayoutDescriptor, wgpu 29's name for push constants) need `Features::IMMEDIATES`.

**Ping-pong**: when invocation k reads state of j ≠ k, read `state_in`, write `state_out`, swap after submit (block_program.rs:118-121, 534-556).
In-place `read_write` is only safe when no invocation observes another's write.

**Readback**: never map a simulation buffer per frame. Reduce/compact on GPU into a bounded buffer, `copy_buffer_to_buffer` into `MAP_READ`, `map_async`, `device.poll`.
Rotate 2-3 staging buffers for recurring reads. `Rgba16Float` premultiplied-linear is the effect chain's exit (analysis.rs:15).

**Overrides / const**: `override x: f32 = 1.0;` is a pipeline-overridable constant set at pipeline creation through
`PipelineCompilationOptions.constants` (by name or `@id(n)`), usable in `@workgroup_size`. `const` is compile-time only. Motolii's shelf does
**not** use overrides as overrides: `wgsl_manifest` rewrites each into `var<private>` filled from a uniform every frame (block_program.rs:319-320, 381) —
so a dial can change without rebuilding the pipeline, but it can't size a workgroup.

**Features (native-only or gated)** — request only what the workload uses; `adapter.features()` is a possibility, the device has what you asked
(headless.rs:48-49 asks TIMESTAMP_QUERY + TIMESTAMP_QUERY_INSIDE_ENCODERS): `SHADER_F16` (+ `enable f16;`), `SUBGROUP` / `SUBGROUP_VERTEX` / `SUBGROUP_BARRIER`
(no `enable subgroups;` — naga 29 lists it as unimplemented and rejects the directive), `IMMEDIATES`, `TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES`,
`BGRA8UNORM_STORAGE`, `TEXTURE_FORMAT_16BIT_NORM`, `VERTEX_WRITABLE_STORAGE`, `TIMESTAMP_QUERY_INSIDE_PASSES`, `DUAL_SOURCE_BLENDING` (+ `enable dual_source_blending;`).
naga 29 `enable`: `f16`, `dual_source_blending`, `clip_distances` (+ wgpu-native `wgpu_mesh_shader`, `wgpu_ray_query`). `requires` language extensions implemented:
`readonly_and_readwrite_storage_textures`, `packed_4x8_integer_dot_product`, `pointer_composite_access`.

**Storage textures**: `texture_storage_2d<FORMAT, write|read|read_write>`; core storage-capable formats: `rgba8unorm/snorm/uint/sint`, `rgba16float/uint/sint`,
`r32float/uint/sint`, `rg32*`, `rgba32*`. `rgba8unorm-srgb`, `bgra8unorm` (without the feature), `rg16float`, `r16float` are **not** storage-capable — render to them instead.
Motolii effect targets: `Rgba16Float` (vism.rs:60), `Rgba8Unorm`, `Rgba8UnormSrgb` blend target (compositor.rs:80), `Rg8Uint` selection (selection_bounds.rs:196).

## 罠 (Motolii already hit these)

- **Reserved words**: `from`, `catch`, `of`, `do`, `new`, `class`, `self`, `this`, `yield`, `super`, `where`, … (naga 29 `keywords/wgsl.rs:11`; `to` and `in` are NOT reserved there — docs/skills/motolii-block says `to`, unverified). One reserved
  name breaks every block (the shelf prepends modules, 2026-09-18). Rename: `from` → `origin`, `catch` → `contagion`.
- **`Offset.rotate` is degrees**, added; `radians()`/`degrees()` are builtins — write `degrees(angle)` when the law is in radians (hang.wgsl:27,33).
- **`Offset.tint` multiplies**: `vec4f(1.0)` = unchanged, >1 brightens, `a` is opacity factor (block_program.rs:7, 200).
- **WESL imports first**: `import …;` before any declaration; a block is `package::block`, prelude is `package::motolii`, modules are named by file stem
  (`_cavalry.wgsl` → `package::cavalry`). **No wildcard `::*` in wesl 0.4.2** — name every item. `import` of an unused item is fine; a missing one fails at wesl.
- **Unknown attributes are rejected by naga** (`unknown attribute: `@label``, naga error.rs:212,701). `@label` `@range` `@options` `@reach` `@id` `@description`
  `@rounds` `@scope` `@physics` are stripped by Motolii's scanner before wesl (block_program.rs:314-320); anything else must be plain WGSL. wgsl-parse 0.4.2 has no string
  literal, so `@x("…")` can only exist where the scanner reads it (file head, before `override` / `fn block`).
- **Literal types**: `1u` vs `1` vs `1.0`; `f32(x)` before mixing; `switch` cases need the selector's type (`case 1u:`).
- **`vec3` in structs**: 12 bytes, 16 aligned — the host packer must skip 4 bytes; prefer `vec4` for GPU-side arrays (`Basis { u, v, centre: vec4f }`).
- **Uniform writes coalesce at submit**: the last `write_buffer` to one buffer wins for the whole frame — per-round/per-block values need separate buffers or dynamic offsets (block_program.rs:518-524).
- **`Capabilities::all()` in the shelf's naga validator** (block_program.rs:433) accepts f16/subgroup/ray-query shaders the device may then refuse at `create_compute_pipeline`.
- **Device error is async**: `create_*_pipeline` returns even when invalid; wrap with `push_error_scope(Validation)` and `block_on(scope.pop())` (catalog.rs:696-702).
- **Vertex stage can't read `read_write` storage**; **`textureSample` needs a filterable format + filtering sampler** (`Rgba16Float` is filterable, `Rgba32Float` is not without `FLOAT32_FILTERABLE`).

## 検証

1. `cargo run -p motolii-render --example zz_shelf` — wesl + naga (`Capabilities::all()`); prints `rejected: <file>: <reason>` with naga's span.
2. Pipeline-level: a test that `push_error_scope(wgpu::ErrorFilter::Validation)` → compile → `pollster::block_on(scope.pop())` and asserts `None`
   (catalog.rs:696-723, block_program.rs:929, surface_program.rs:201-213). GPU tests live in the render crate; run cargo from the repo root.
3. For a symptom: blank → target format vs pipeline format, load/store ops, viewport; frozen compute → dispatch count 0, guard count, bind group still points at the
   old ping-pong side; corruption at a threshold → stride / capacity / dynamic offset / bin overflow; works on one GPU only → an unrequested feature or limit.
4. Spec doubt → `references/gfx-rs-webgpu-specs/download.sh`, grep the `.bs`, link the anchor.

## 出典と license

- cazala/webgpu-skill (https://github.com/cazala/webgpu-skill, commit e439805, 2026-07-23): source of the layout table, contract list, compute/ping-pong/readback rules,
  symptom table. **No license file** → not copied; only the rules re-stated here, browser parts (canvas, DPR, requestAdapter, `@webgpu/types`, Vite starter) dropped.
- gfx-rs/wgpu `.claude/skills/webgpu-specs` (MIT OR Apache-2.0): copied verbatim to `references/gfx-rs-webgpu-specs/` with LICENSE.MIT + NOTICE. The mcpmarket
  "WebGPU Specification Reference" and lobehub "gfx-rs-wgpu-webgpu-specs" listings are re-hostings of this same two-file skill.
- naga 29.0.4 sources (`~/.cargo/registry`): `front/wgsl/parse/directive/{enable_extension,language_extension}.rs`, `keywords/wgsl.rs`, `front/wgsl/error.rs`.
- Motolii: `docs/skills/motolii-block/SKILL.md` (the block contract itself), `docs/reviews/2026-09-18-daily.md:69,120,127`.
