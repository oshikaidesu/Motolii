# M2 CompCamera decision — planar v1, additive spatial future (2026-07-16)

Status: **Decision — adopt one composition camera, narrow the first permanent variant**. Implementation is D1j → D1k → D3f and must not start from the broader unmerged #176 shape.

## Known-technology check

- glTF separates a camera's projection from its node transform; it does not encode pose as `position + target`. A camera looks down local -Z with local +Y up, while perspective `yfov` is radians and orthographic magnification is explicit ([Khronos glTF 2.0](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-camera)).
- OpenUSD separates camera selection/pose from render raster settings such as resolution, pixel aspect, and data window. Output dimensions therefore must not be duplicated into camera state ([OpenUSD Render Settings](https://openusd.org/release/api/usd_render_page_front.html)).
- After Effects separates the active camera from a working 3D view; changing a working view does not change layer properties or output ([Adobe cameras and views](https://helpx.adobe.com/after-effects/using/cameras-lights-points-interest.html)).
- Blender documents that target-plus-up tracking becomes unstable when the owner is nearly aligned over the target. That is the same singularity left open by the unmerged `position + target + roll` proposal ([Blender Track To](https://docs.blender.org/manual/en/latest/animation/constraints/tracking/track_to.html)).

The durable lesson is to keep output raster, edit view, projection, and pose distinct. It does **not** justify permanently choosing a spatial orientation/interpolation contract before M5 has evidence.

## Decision

### 1. One camera and one world

- Every current Document has exactly one `Composition.camera`; no camera layer, group camera, shot switch, or per-layer camera is added.
- 2D objects remain in the canonical XYZ world on `z=0`. The first camera variant has a fixed -Z viewing direction; this is not a separate 2D world or an “enable 3D” mode.
- `Composition` remains the sole owner of rational output aspect. Pixel resolution, DPI, logical/physical px, and UI viewport size are not camera fields.
- Stage View pan/zoom/fit is transient workspace/session state and never changes Document. Output Frame manipulation changes the Document camera through D2 commands and is export-visible.

This adopts the unified Stage/Output Frame direction of #176, but narrows its permanent schema.

### 2. Exact v1 schema meaning

The first serialized camera is an internally tagged enum (`#[serde(tag = "kind", rename_all = "snake_case")]`) with only one accepted wire variant:

```text
CompCameraDoc::PlanarOrthographic {
    center: DocParam<Vec2>,          // canonical XY; default [0, 0]
    roll_radians: DocParam<F64>,     // counter-clockwise in canonical Y-up; default 0
    height: DocParam<F64>,           // visible canonical height; default 1
}

wire: { "kind": "planar_orthographic", "center": ..., "roll_radians": ..., "height": ... }
```

Unknown `kind` values are typed rejection. At time `t`, let output aspect be `a = Composition.aspect_num / Composition.aspect_den` (never inferred from `FrameDesc`), world XY point be `p`, camera center `c`, roll `r`, and visible height `h`. After translating by `p-c` and rotating by `-r`:

```text
q = R(-r) * (p - c)
ndc_x = 2 * q.x / (h * a)
ndc_y = 2 * q.y / h
```

The unrotated visible interval is `x ∈ [c.x-h*a/2, c.x+h*a/2]`, `y ∈ [c.y-h/2, c.y+h/2]`; with roll, its four world corners are `c + R(r)*(±h*a/2, ±h/2)`. The render request must satisfy exact rational aspect equality `FrameDesc.width / FrameDesc.height == a` by cross multiplication; mismatch is typed rejection, not stretch/crop/letterbox.

`Quality::render_desc` must never manufacture a mismatched Draft size by truncating integer division. For requested scale `s`, it applies the reduction only when `W % s == 0`, `H % s == 0`, and the reduced descriptor still passes exact aspect cross multiplication; otherwise it returns the input descriptor unchanged, so DRS skips that unavailable tier and continues frame dropping. Fixed judgments include `1920×1080 / 2 = 960×540` and `16×9 / 2 = unchanged 16×9`.

For raster size `W×H`, the **only** NDC-to-pixel boundary is:

```text
px_x = (ndc_x + 1) * W / 2
px_y = (1 - ndc_y) * H / 2
```

For the default camera and matching aspect, this reduces exactly to the current `ViewportTransform`: `px_x=W/2+p.x*H`, `px_y=H/2-p.y*H`. D3f replaces that identity mapping with the general camera mapping; it must not compose camera NDC with a second old canonical transform.

NDC is Y-up. Output raster conversion remains the existing single render-boundary conversion. Layer compositing order remains the current Document order; the planar variant does not introduce depth testing or reinterpret layer Z.

Validation evaluates the existing `DocParam` contract and rejects non-finite center/roll/height and `height <= 0` with typed errors. Aspect is the existing positive rational Composition value. Positive roll is counter-clockwise in Y-up and uses the same sign convention as `Transform2D.rotation`; UI may display degrees but writes radians. No epsilon fallback, target/up vector, near/far plane, overscan, or pixel value is serialized.

### 3. Versioning and existing pixels

Before D1j, CAM-G0 adds a fixed existing-render 2D identity fixture. The harness at `crates/motolii-render/tests/cam_g0_planar_identity.rs` is unclassified and may change with API, fixture construction, and runtime wiring; it reads the oracle and compares actual output. The immutable semantic oracle at `crates/motolii-render/tests/oracles/cam_g0_planar_identity.tsv` holds fixed input metadata and the exact expected RGBA bytes in reviewable text; only this oracle is registered `semantic` in `crates/motolii-testkit/golden_policy/classification.tsv`. It is generated and reviewed while the old `ViewportTransform` path is still authoritative; later camera PRs may not alter the oracle expected bytes.

D1j runs only after D1l reaches `main` **and a preflight proves `LATEST_DOCUMENT_VERSION == READER_VERSION == WRITER_VERSION == 4`**. D1l owns Document v4; D1j adds the camera field as the immediately following Document version (currently v5) and raises writer/reader/min-reader consistently. If another schema version reaches main first, implementation stops for a decision amendment rather than taking v5 opportunistically. D1e migration from v1–v4 inserts exactly the default `PlanarOrthographic` value. A v5-shaped payload disguised as v1–v4 is typed rejection, not serde default acceptance.

The default camera is the identity view for existing 2D output: canonical height 1, center 0, roll 0, and Composition aspect. D3f must prove that CAM-G0 is byte-identical and that the existing protected `d1i3_transform_compose.rs` and `d3_eval_order.rs` expectations remain unchanged. The golden policy classification and expected values are not updated to achieve this.

### 4. Spatial/Perspective is an additive future variant

M2 does not serialize spatial pose or Perspective. M5 may add `CompCameraDoc::Spatial { ... }` as a new tagged variant with a version/min-reader increase after a camera-pose decision fixes:

- orientation representation and keyframe interpolation;
- handedness/local axes and transform order;
- perspective/orthographic projection fields and clip policy;
- singularity behavior for optional target constraints;
- switching/migration between PlanarOrthographic and Spatial;
- semantic goldens for animation, depth, and preview/export identity.

It must not reinterpret `center`, `roll_radians`, or `height`, and must not retrofit `position + target + implicit world-up` into the planar variant. A target constraint, if later desired, is evaluated into a spatial pose rather than becoming the pose storage itself.

## Implementation tickets and order

| ID | Scope | Depends on | Completion judgment |
|---|---|---|---|
| CAM-G0 | pre-camera protected identity fixture: unclassified harness + semantic oracle | this decision, current render path | fixed matching-aspect 2D fixture records current output bytes in the oracle; golden-policy rejection proves later oracle expectation edits fail |
| D1j | v5 `CompCameraDoc::PlanarOrthographic`, validation, D1e default migration | CAM-G0, D1l, D1e | version preflight is exactly 4; v1–v4 migration is idempotent; disguised versions reject; roundtrip preserves all params/extra; existing counts/IDs/unknown fields unchanged |
| D1k | runtime planar camera in radians and camera-bearing render input; replace the degree/Perspective-only assumption without exposing Slint | D1j, CQ-5 thaw review | dedicated thaw record lists old/new API, `LayerSourceContext` and `RenderGraphInputs.camera`, degree→radian impact, migration/non-impact and goldens; equation/raster fixtures map center/corners/roll exactly; aspect mismatch/invalid input typed; Draft reduction applies only when divisibility+exact aspect survive, otherwise input desc unchanged; preview/export same function; no Planar→old position/target/FOV shim |
| D3f | evaluate Document camera at `t` and connect it to 2D graph/render | D1k, D3 | CAM-G0 byte-identical; existing transform/eval expectations unchanged; animated center/roll/height fixtures match equations; preview/export identical; no double viewport transform or depth/order change |

Each ID is one commit/PR. Do not combine the baseline fixture, schema, runtime thaw, and render connection. D1j may be issued only after this decision, CAM-G0, and D1l reach `main`; D1k/D3f remain waiting in order. The D1k thaw record is a required deliverable, not a later review note; `CompCameraDoc` is the only durable camera type and the old core Perspective/degree type must not be reused as storage or an adapter shim.

## M3 boundary

This decision does not release the active M3 stop. After M2 reclosure and a new M3 entry decision, Stage/Output Frame UI may be translated into M3 task IDs. Camera gestures must use D2 one-gesture/one-history commands; Stage View gestures remain transient; Slint types and px/DPI do not enter the camera contract.

## Non-goals

- Perspective or arbitrary 3D pose in M2/M3 initial UI.
- Multiple cameras or camera switching.
- A second preview camera stored in Document.
- Full-quality rendering of an unbounded Stage outside Output Frame.
- Adopting #176's separate Param Pipeline, Element Domain, Constraint Graph, or implementation-ledger proposals; each remains an independent disposition.
