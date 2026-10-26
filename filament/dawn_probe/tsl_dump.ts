// Shader Lab's Fluted Glass made ready to draw as three.js would draw it, all by three's own code, written out for a
// plain WebGPU draw: the WGSL (WGSLNodeBuilder), each bind group's uniforms as three packs them (binding.update ->
// binding.buffer, three.webgpu.js 32983-32987, after its NodeFrame update, 55980-55995 and 56124-56134), the plane's arrays.
import "core-js/actual/url"   // QuickJS has no URL or structuredClone: core-js's (MIT), the usual ones
import "core-js/actual/structured-clone"
import * as std from "qjs:std"
import * as THREE from "three/webgpu"
import { FlutedGlassPass } from "/Users/motolii_maker/motolii/research_notes/tracking/shader-lab/packages/shader-lab-react/src/renderer/fluted-glass-pass"
const pass: any = new FlutedGlassPass("fluted"); pass.updateParams({}); pass.flushColorNode()
const renderer: any = new THREE.WebGPURenderer({ canvas: { getContext() { return null }, style: {}, addEventListener() {} } as any })
// The two things the builder asks the GPU (reports/shaderlab_origins.md): plain WebGPU, no optional features.
renderer._initialized = true
renderer.backend.hasFeature = () => false
renderer.backend.utils = new Proxy({}, { get: (_, k: string) => () => k === "getTextureSampleData" ? { primarySamples: 1 } : undefined })
const mesh = pass.compositeMesh
// as three's Renderer does before it draws: the camera in WebGPU's own coordinates (depth 0 to 1)
pass.camera.coordinateSystem = (THREE as any).WebGPUCoordinateSystem; pass.camera.updateProjectionMatrix()
pass.scene.updateMatrixWorld(); pass.camera.updateMatrixWorld()
const b: any = new THREE.WGSLNodeBuilder(mesh, renderer)
b.scene = pass.scene; b.material = pass.material; b.camera = pass.camera; b.context.material = pass.material
b.build()
const frame: any = new (THREE as any).NodeFrame()
frame.renderer = renderer; frame.scene = pass.scene; frame.object = mesh; frame.camera = pass.camera; frame.material = pass.material
frame.update()
for (const node of b.updateNodes) frame.updateNode(node)
const out = (name: string, s: string) => std.open("tsl_draw/" + name, "w").puts(s)
const bin = (name: string, a: ArrayBufferView) => { const f = std.open("tsl_draw/" + name, "wb"); f.write(a.buffer, a.byteOffset, a.byteLength); f.close() }
out("vert.wgsl", b.vertexShader); out("frag.wgsl", b.fragmentShader)
let plan = ""
b.getBindings().forEach((g: any, gi: number) => g.bindings.forEach((x: any, bi: number) => {
  if (x.isUniformsGroup) { x.update(); bin(`g${gi}b${bi}.bin`, x.buffer); plan += `${gi} ${bi} uniform ${x.buffer.byteLength}\n` }
  else if (x.isSampledTexture) plan += `${gi} ${bi} texture 0\n`
  else if (x.isSampler) plan += `${gi} ${bi} sampler 0\n`
}))
out("bindings.txt", plan)
const g = mesh.geometry
bin("position.f32", g.attributes.position.array); bin("uv.f32", g.attributes.uv.array); bin("index.u16", new Uint16Array(g.index.array))
console.log(plan + "vertices " + g.attributes.position.count + ", indices " + g.index.count)
