// shader-effects-inc/shaders (npm shaders 4.0.5) drawn by its authors' own renderer (createRendererFromJSON:
// our device through `gpu`, renderFrame({deltaSeconds}) frame-locked) in Motolii's QuickJS (jsrh_napi) on Motolii's
// Dawn (dawn.node). As check_node/shaders_check.mjs does it in Node: the canvas only what the renderer asks of one,
// `window` only what it reads.   qjs_napi shaders.mjs <preset.json> <w> <h> <frames> <out.rgba>
import "./web_globals"
import * as std from "qjs:std"
import * as os from "qjs:os"
import { createRendererFromJSON } from "shaders/core"

const [presetFile, W, H, FRAMES, outFile] = scriptArgs.slice(1)
const w = +W, h = +H, frames = +FRAMES
const dawn = (globalThis as any).loadAddon(std.getenv("DAWN_NODE"))
Object.assign(globalThis, dawn.globals)
const gpu = dawn.create([])
Object.assign(globalThis, {
  navigator: { gpu: Object.assign(gpu, { getPreferredCanvasFormat: () => "rgba8unorm" }) },
  self: globalThis, requestAnimationFrame: (f: any) => os.setTimeout(() => f(Date.now()), 16), cancelAnimationFrame: os.clearTimeout,
})
Object.assign(globalThis, { window: Object.assign(globalThis, { innerWidth: w, innerHeight: h, devicePixelRatio: 1, addEventListener() {}, removeEventListener() {},
  matchMedia: () => ({ matches: false, addEventListener() {}, removeEventListener() {} }) }) })

const adapter = await gpu.requestAdapter()
const device = await adapter.requestDevice({ requiredFeatures: [...adapter.features] })   // (as the authors' createSharedDevice asks)
const target = device.createTexture({ size: [w, h], format: "rgba8unorm", usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC | GPUTextureUsage.TEXTURE_BINDING })
const context = { configure() {}, unconfigure() {}, getConfiguration() { return { format: "rgba8unorm" } }, getCurrentTexture: () => target }
const canvas = { width: w, height: h, style: { width: `${w}px`, height: `${h}px` }, parentElement: null, getBoundingClientRect: () => ({ x: 0, y: 0, left: 0, top: 0, width: w, height: h, right: w, bottom: h }),
  addEventListener() {}, removeEventListener() {}, getContext: (kind: string) => (kind === "webgpu" ? context : null) }

const renderer = createRendererFromJSON(JSON.parse(std.loadFile(presetFile)!), { gpu: { device, adapter } })
await renderer.initialize(canvas as any)
const t0 = Date.now()
for (let f = 0; f < frames; ++f) await renderer.renderFrame({ deltaSeconds: 1 / 60 })
const ms = (Date.now() - t0) / frames

const stride = Math.ceil(w * 4 / 256) * 256
const back = device.createBuffer({ size: stride * h, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ })
const coder = device.createCommandEncoder()
coder.copyTextureToBuffer({ texture: target }, { buffer: back, bytesPerRow: stride }, [w, h])
device.queue.submit([coder.finish()])
await back.mapAsync(GPUMapMode.READ)
const got = new Uint8Array(back.getMappedRange())
const out = std.open(outFile, "wb")!
for (let y = 0; y < h; y++) out.write(got.buffer, y * stride, w * 4)
out.close()
print(`drawn ${frames} frames ${w}x${h} in QuickJS, ${ms.toFixed(2)} ms a frame (the renderer's own wait for the GPU included)`)
renderer.dispose()
device.destroy()
