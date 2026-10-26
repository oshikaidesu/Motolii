// A TSL material turned to WGSL by three.js's own builder (WGSLNodeBuilder), with no GPU, in QuickJS: set up as three's
// Nodes._createNodeBuilder does (three.webgpu.js 55300-55310), the camera in WebGPU's own coordinates as its Renderer
// does before it draws (62657). Each collection's entry (tsl_textures.ts, ...) makes its effects as its author's README
// says and hands them here; the shaders land in filament/out/tsl/<collection>/ (reports/shaderlab_origins.md).
import "core-js/actual/url"   // QuickJS has no URL or structuredClone: core-js's (MIT), the usual ones
import "core-js/actual/structured-clone"
import * as std from "qjs:std"
import * as os from "qjs:os"
import * as THREE from "three/webgpu"

const renderer: any = new THREE.WebGPURenderer({ canvas: { getContext() { return null }, style: {}, addEventListener() {} } as any })
// The two things the builder asks the GPU it will draw on, answered as plain WebGPU with no optional features.
renderer._initialized = true
renderer.backend.hasFeature = () => false
renderer.backend.utils = new Proxy({}, { get: (_, k: string) => () => k === "getTextureSampleData" ? { primarySamples: 1 } : undefined })

export const plane = new THREE.PlaneGeometry(2, 2)

export function wgsl(object: any, material: any, camera?: any, scene?: any, drawable?: string): [string, string] {
  camera ??= new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1)
  scene ??= new THREE.Scene()
  camera.coordinateSystem = (THREE as any).WebGPUCoordinateSystem
  camera.updateProjectionMatrix()
  scene.updateMatrixWorld(); camera.updateMatrixWorld(); object.updateMatrixWorld()
  const b: any = new (THREE as any).WGSLNodeBuilder(object, renderer)
  b.scene = scene; b.material = material; b.camera = camera; b.context.material = material
  b.build()
  if (drawable) writeDrawable(drawable, b, object, material, camera, scene)
  return [b.vertexShader, b.fragmentShader]
}

// What dawn_probe/tsl_on_dawn.cpp needs to draw it, all from three's own code: each bind group's uniforms as three packs
// them (binding.update -> binding.buffer, three.webgpu.js 32983-32987) after its NodeFrame update (55980-55995,
// 56124-56134), and the mesh's arrays.
function writeDrawable(dir: string, b: any, object: any, material: any, camera: any, scene: any) {
  os.mkdir(dir)
  const frame: any = new (THREE as any).NodeFrame()
  frame.renderer = renderer; frame.scene = scene; frame.object = object; frame.camera = camera; frame.material = material
  frame.update()
  for (const node of b.updateNodes) frame.updateNode(node)
  const bin = (name: string, a: ArrayBufferView) => { const f = std.open(`${dir}/${name}`, "wb"); f.write(a.buffer, a.byteOffset, a.byteLength); f.close() }
  let plan = ""
  b.getBindings().forEach((g: any, gi: number) => g.bindings.forEach((x: any, bi: number) => {
    if (x.isUniformsGroup) { x.update(); bin(`g${gi}b${bi}.bin`, x.buffer); plan += `${gi} ${bi} uniform ${x.buffer.byteLength}\n` }
    else if (x.isSampledTexture) plan += `${gi} ${bi} texture 0\n`
    else if (x.isSampler) {   // (how three's texture says it is read: its filters and wraps, as three.js's own numbers)
      const t = x.texture ?? {}
      plan += `${gi} ${bi} sampler 0 ${t.magFilter ?? 1006} ${t.minFilter ?? 1006} ${t.wrapS ?? 1001} ${t.wrapT ?? 1001}\n`
    }
    else plan += `${gi} ${bi} ${x.constructor?.name ?? "other"} 0\n`
  }))
  std.open(`${dir}/bindings.txt`, "w").puts(plan)
  std.open(`${dir}/vert.wgsl`, "w").puts(b.vertexShader)
  std.open(`${dir}/frag.wgsl`, "w").puts(b.fragmentShader)
  // the vertex inputs in the order the builder gave them locations (b.attributes), each "<name> <type>" with its array
  const g = object.geometry
  std.open(`${dir}/attributes.txt`, "w").puts(b.attributes.map((a: any) => `${a.name} ${a.type}\n`).join(""))
  for (const a of b.attributes) bin(`${a.name}.f32`, new Float32Array(g.attributes[a.name].array))
  bin("index.u16", new Uint16Array(g.index.array))
}

// Each named effect made by `make` (a material for the plane) written to out/<collection>/<name>.{vert,frag}.wgsl; a line
// for each in out/<collection>/made.tsv: name, ok or the builder's error.
export function convertAll(collection: string, effects: [string, () => any][]) {
  const dir = "out/" + collection
  os.mkdir("out"); os.mkdir(dir)
  const said = std.open(dir + "/made.tsv", "w")
  let ok = 0
  for (const [name, make] of effects) {
    try {
      const material = make()
      const [v, f] = wgsl(new THREE.Mesh(plane, material), material, undefined, undefined, `${dir}/${name}`)
      std.open(`${dir}/${name}.vert.wgsl`, "w").puts(v)
      std.open(`${dir}/${name}.frag.wgsl`, "w").puts(f)
      said.puts(`${name}\tok\n`); ok++
    } catch (e: any) { said.puts(`${name}\t${String(e?.message ?? e).replace(/\s+/g, " ").slice(0, 160)}\n`) }
  }
  said.close()
  console.log(`${collection}: ${ok} of ${effects.length} turned to WGSL`)
}
