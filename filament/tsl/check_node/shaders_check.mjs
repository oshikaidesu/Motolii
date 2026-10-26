// A check, not part of Motolii: shader-effects-inc/shaders (npm shaders 4.0.5, MIT) drawn by its authors' own renderer,
// the way they offer it to other tools (core/presetRenderer.ts createRendererFromJSON: our own device through `gpu`,
// renderFrame({deltaSeconds}) frame-locked), on Motolii's Dawn through Dawn's own JS binding (dawn.node), in Node.
// The canvas is only what the renderer asks of one (its size, and a WebGPU context whose texture is ours), as
// check.ts stands one in for three.js.
//   DAWN_NODE=<dawn.node> node shaders_check.mjs <preset.json> <w> <h> <frames> <out.rgba>
import { createRequire } from "node:module"
import * as fs from "node:fs"

const [presetFile, W, H, FRAMES, outFile] = globalThis.motolii?.args ?? process.argv.slice(2)
const w = +W, h = +H, frames = +FRAMES
const dawn = createRequire(import.meta.url)(globalThis.motolii?.dawnNode ?? process.env.DAWN_NODE)
Object.assign(globalThis, dawn.globals)
const gpu = dawn.create([])
Object.defineProperty(globalThis, "navigator", { value: { gpu: Object.assign(gpu, { getPreferredCanvasFormat: () => "rgba8unorm" }) }, configurable: true })
Object.assign(globalThis, { self: globalThis, requestAnimationFrame: (f) => setTimeout(() => f(performance.now()), 16), cancelAnimationFrame: clearTimeout })
// (what the renderer reads of a browser window: the page's size and pixel ratio, its events, media queries; nothing happens there)
Object.assign(globalThis, { window: Object.assign(globalThis, { innerWidth: w, innerHeight: h, devicePixelRatio: 1, addEventListener() {}, removeEventListener() {}, matchMedia: () => ({ matches: false, addEventListener() {}, removeEventListener() {} }) }) })

// Inside Motolii (dawn_probe/node_effects.cpp sets globalThis.motolii): Filament's own device and a texture of it, lent
// to Dawn's binding (dawn_lend.patch: wrapDevice, wrapTexture); otherwise a device of our own
const lent = globalThis.motolii
const adapter = lent ? { features: new Set(), info: {} } : await gpu.requestAdapter()
const device = lent ? dawn.wrapDevice(lent.device, lent.instance) : await adapter.requestDevice({ requiredFeatures: [...adapter.features] })   // (as the authors' createSharedDevice asks)
const target = lent ? dawn.wrapTexture(lent.device, lent.texture) : device.createTexture({ size: [w, h], format: "rgba8unorm", usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC | GPUTextureUsage.TEXTURE_BINDING })
const context = { configure() {}, unconfigure() {}, getConfiguration() { return { format: "rgba8unorm" } }, getCurrentTexture: () => target }
const canvas = { width: w, height: h, style: { width: `${w}px`, height: `${h}px` }, parentElement: null, getBoundingClientRect: () => ({ x: 0, y: 0, left: 0, top: 0, width: w, height: h, right: w, bottom: h }),
  addEventListener() {}, removeEventListener() {}, getContext: (kind) => (kind === "webgpu" ? context : null) }

const { createRendererFromJSON, setShadersDebug } = await import("shaders/core")
if (process.env.SHADERS_DEBUG) setShadersDebug(true)
const renderer = createRendererFromJSON(JSON.parse(fs.readFileSync(presetFile, "utf8")), { gpu: { device, adapter } })
await renderer.initialize(canvas)
const t0 = performance.now()
for (let f = 0; f < frames; ++f) await renderer.renderFrame({ deltaSeconds: 1 / 60 })
const ms = (performance.now() - t0) / frames

const stride = Math.ceil(w * 4 / 256) * 256
const back = device.createBuffer({ size: stride * h, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ })
const coder = device.createCommandEncoder()
coder.copyTextureToBuffer({ texture: target }, { buffer: back, bytesPerRow: stride }, [w, h])
device.queue.submit([coder.finish()])
await back.mapAsync(GPUMapMode.READ)
const got = Buffer.from(back.getMappedRange()), rows = Buffer.alloc(w * h * 4)
for (let y = 0; y < h; y++) got.copy(rows, y * w * 4, y * stride, y * stride + w * 4)
fs.writeFileSync(outFile, rows)
console.log(`drawn ${frames} frames ${w}x${h}, ${ms.toFixed(2)} ms a frame (the renderer's own wait for the GPU included)`)
renderer.dispose()
if (!lent) process.exit(0)
