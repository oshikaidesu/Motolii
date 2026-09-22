# FrameGraph node migration manifest

This is a **cutover inventory**, not a second architecture document. Run:

```bash
python3 scripts/framegraph-migration-inventory.py
```

The scanner answers “where are the old seams still named?” quickly. It does **not**
declare code deletable by name alone. A cluster is deletable only when:

1. the legacy responsibility is named;
2. its replacement is a FrameGraph node/edge, scheduler/cache policy, or explicitly
   allowed lower-level GPU/resource adapter;
3. the production caller reaches that replacement directly;
4. product-path references to the old owner are zero; and
5. the remaining references, if any, are explicit oracle/test compatibility.

## Clusters

| Cluster | Legacy responsibility | Replacement / evidence | Delete gate |
| --- | --- | --- | --- |
| legacy-scene-owner | StoreView/time → whole `ResolvedLayer` scene, `build_layers` orchestration | `SceneValue` evaluation → `prepare_gpu_scene`; production playback is FrameGraph-owned | no production caller may enter `layers_from_resolved` / whole-scene `build_layers` |
| compat-resolved-projection | projecting evaluated semantic scene back to `ResolvedLayer` for old consumers | consumers read `SceneValue` / `SceneLayerValue` directly | projection remains only where a test/tool explicitly needs the old type |
| resolved-layer-type | old resolved scene DTO used across helpers | `SceneLayerValue` for product meaning; proven raster/decode/GPU helpers may remain below the graph | classify each hit: semantic owner vs allowed adapter/oracle; remove semantic-owner hits |
| legacy-transform-resolve | independent transform evaluation outside graph | `TransformProgram` local/world values from FrameGraph | editor/render/export product paths contain no second transform evaluator |

## Current cutover reading

PR #506 already records the semantic coverage checklist and the architectural rule:
**extract old semantics into node/edge/lowering owners; do not revive the old owner**.
The remaining work should therefore be attacked by cluster rather than by repeatedly
rediscovering the repository.

Suggested order:

1. **compat-resolved-projection** — usually the smallest, clearest leaf seam.
2. **legacy-transform-resolve** — remove any surviving second evaluator.
3. **legacy-scene-owner** — once callers are gone, delete orchestration that only
   exists to build the old whole scene.
4. **resolved-layer-type** — broad cleanup last; keep legitimate low-level adapters
   and oracle fixtures until their consumers disappear.

Do not turn the scanner into a giant Rust-aware migration framework. Add a symbol to
a cluster when repeated rediscovery is costing time; otherwise keep moving the
cutover forward.


## Static ownership smell gate

The inventory also scans for behavior that indicates a second planner even when
the old symbol names have disappeared:

- semantic relation fields (`matte`, `clip_to_below`, masks/effects) read by
  concrete `gpu_exec` backends;
- `LayerId` maps/indexes used to rediscover cross-contribution dependencies;
- mutation/filtering of `Vec<LayerWithPasses>` to decide scene survivors;
- concrete GPU backends reading `SceneValue` / `SceneLayerValue` instead of
  resource-keyed operation payloads.

Run the strict GPU ownership check with:

```bash
python3 scripts/framegraph-migration-inventory.py --fail-on-gpu-planner-smell
```

This is intentionally a negative architecture gate, not a semantic-parity proof.
During migration it may remain red. The useful invariant is that the reported
product-owner-smell set only shrinks; new execution owners must not be added to
the allowlist merely to make the gate green.
