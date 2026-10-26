// basementstudio/shader-lab (Apache-2.0): every layer type its runtime makes, made as its author makes one: the layer as
// createLayer (src/lib/editor/layers.ts:33) builds it with its definition's own defaults (buildParameterValues; the id
// is the type, as QuickJS has no crypto.randomUUID), the pass from PipelineManager.createPass, set by its applyLayerState
// and sized as pipeline-manager.ts:312-313 sizes it; then every node material inside the pass is turned to WGSL.
import "core-js/actual/url"   // (first: its modules use URL as they load)
import "core-js/actual/structured-clone"
import * as THREE from "three/webgpu"
import { getLayerDefinition } from "../third_party/tsl/basementstudio__shader-lab/src/lib/editor/config/layer-registry"
import { buildParameterValues } from "../third_party/tsl/basementstudio__shader-lab/src/lib/editor/parameter-schema"
import { DEFAULT_MASK_CONFIG } from "../third_party/tsl/basementstudio__shader-lab/src/types/editor"
import { PipelineManager } from "../third_party/tsl/basementstudio__shader-lab/packages/shader-lab-react/src/renderer/pipeline-manager"
import { plane, wgsl } from "./convert"
import * as std from "qjs:std"
import * as os from "qjs:os"

const types = ["photographic-cells", "displaced-rings", "annotations", "ascii", "blob-tracking", "bloom", "circuit-bent",
  "directional-blur", "chromatic-aberration", "crt", "displacement-map", "dithering", "edge-detect", "fluted-glass",
  "halftone", "ink", "particle-grid", "pattern", "pixelation", "pixel-sorting", "plotter", "posterize", "slice", "smear",
  "threshold", "gradient-map", "lumen-print", "grain", "signal-rot", "dot-grid", "erosion", "relief", "flares",
  "focus-blur", "glass", "connected-dots", "photocopy", "outline", "voxel", "gradient", "shape", "fluid", "pixel-trail",
  "magnify-lens"]   // (pipeline-manager.ts:706-811, the ones not needing a picture, a camera or the user's own code)
const manager: any = { renderer: null, onRuntimeError: () => {} }
const dir = "out/shader-lab"
os.mkdir("out"); os.mkdir(dir)
const said = std.open(dir + "/made.tsv", "w")
let made = 0, ok = 0
for (const type of types) {
  try {
    const definition = getLayerDefinition(type as any)
    const layer = { assetId: null, blendMode: "normal", compositeMode: "filter", hue: 0, id: type, kind: definition.kind,
      maskConfig: { ...DEFAULT_MASK_CONFIG }, opacity: 1, params: buildParameterValues(definition.params), saturation: 1,
      type, visible: true, mask: null }
    const pass: any = PipelineManager.prototype["createPass" as keyof PipelineManager].call(manager, layer)
    ;(PipelineManager.prototype as any).applyLayerState.call(manager, pass, layer)
    pass.resize?.(1920, 1080); pass.updateLogicalSize?.(1920, 1080)
    const materials = new Set<any>(), seen = new Set<any>()
    const walk = (o: any, d: number) => { if (!o || typeof o !== "object" || seen.has(o) || d > 3) return; seen.add(o)
      if (o.isNodeMaterial) { materials.add(o); return }
      if (o.isTexture || o.isBufferGeometry) return
      for (const k of Object.keys(o)) walk(o[k], d + 1) }
    walk(pass, 0)
    let i = 0
    for (const m of materials) {
      made++
      try {
        const mesh = pass.compositeMesh?.material === m ? pass.compositeMesh : new THREE.Mesh(plane, m)
        const [v, f] = wgsl(mesh, m, pass.camera, pass.scene, mesh === pass.compositeMesh ? `${dir}/${type}` : undefined)
        std.open(`${dir}/${type}.${i}.vert.wgsl`, "w").puts(v); std.open(`${dir}/${type}.${i}.frag.wgsl`, "w").puts(f)
        said.puts(`${type}.${i}\tok\n`); ok++
      } catch (e: any) { said.puts(`${type}.${i}\t${String(e?.message ?? e).replace(/\s+/g, " ").slice(0, 160)}\n`) }
      i++
    }
  } catch (e: any) { said.puts(`${type}\t${String(e?.message ?? e).replace(/\s+/g, " ").slice(0, 160)}\n`) }
}
said.close()
console.log(`shader-lab: ${types.length} layer types, ${ok} of ${made} materials turned to WGSL`)
