// A check, not part of Motolii: Shader Lab's effects drawn by their author's own headless runtime (createHeadlessRenderer:
process.on("unhandledRejection", (e: any) => console.error("unhandled:", e?.message ?? e))
// PipelineManager, each pass's own render()), on Dawn through Dawn's own JS binding (npm webgpu = dawn.node, as its README
// sets it up), in Node. Out: the picture as the browser would show it (three's renderOutput with the author's settings,
// create-webgpu-renderer.ts: SRGBColorSpace, NoToneMapping), raw RGBA.
//   node out.mjs <layers.json> <picture.rgba> <w> <h> <out.rgba>
import { create as npmCreate, globals as npmGlobals } from "webgpu"
import { createRequire } from "node:module"
// DAWN_NODE=<path>: Dawn's binding built from Motolii's own Dawn (filament_src/out/wgpu/third_party/dawn/dawn.node)
// instead of npm's, loaded as Dawn's src/dawn/node/README.md loads it (require)
const dawn: any = process.env.DAWN_NODE ? createRequire(import.meta.url)(process.env.DAWN_NODE) : { create: npmCreate, globals: npmGlobals }
const create = dawn.create
Object.assign(globalThis, dawn.globals)
Object.defineProperty(globalThis, "navigator", { value: { gpu: create([]) }, configurable: true })
// three's renderer takes its frame loop from the browser's self.requestAnimationFrame; Node has none: a 60 Hz timer
Object.assign(globalThis, { self: globalThis, requestAnimationFrame: (f: any) => setTimeout(() => f(performance.now()), 16), cancelAnimationFrame: clearTimeout })
// Canvas 2D for the effects that draw their own pictures (ASCII glyphs, annotations, labels): @napi-rs/canvas (MIT, Skia)
const NapiCanvas: any = await import("@napi-rs/canvas")
Object.assign(globalThis, {
  document: { createElement: (tag: string) => tag === "canvas" ? NapiCanvas.createCanvas(300, 150) : {}, fonts: { check: () => true, load: async () => [], ready: Promise.resolve() } },
  // (a browser Image takes a URL; @napi-rs/canvas's takes a path: file URLs turned to paths)
  Image: class extends NapiCanvas.Image { set src(v: any) { super.src = typeof v === "string" && v.startsWith("file:") ? decodeURIComponent(new URL(v).pathname) : v } get src() { return super.src } },
  // the page's CSS as the author's fonts.css sets it (packages/shader-lab-react/assets/fonts.css :root), its fonts registered
  getComputedStyle: () => ({ getPropertyValue: (v: string) => ({ "--geist-sans": '"Geist Sans"', "--geist-mono": '"Geist Mono"', "--bsmnt-grotesque": '"BSMNT Grotesque"' } as any)[v] ?? "" }),
})
{
  const fonts = "../../third_party/tsl/basementstudio__shader-lab/packages/shader-lab-react/assets/fonts/"
  for (const [file, family] of [["Geist-Sans-VF.woff2", "Geist Sans"], ["Geist-Mono.woff2", "Geist Mono"], ["BasementGrotesqueVF.woff2", "BSMNT Grotesque"]])
    NapiCanvas.GlobalFonts.registerFromPath(fonts + file, family)
}
const THREE: any = await import("three/webgpu")
const TSL: any = await import("three/tsl")
const { createHeadlessRenderer } = await import("../../third_party/tsl/basementstudio__shader-lab/packages/shader-lab-react/src/renderer/create-headless-renderer")
const { buildRendererFrame } = await import("../../third_party/tsl/basementstudio__shader-lab/packages/shader-lab-react/src/renderer/contracts")
import * as fs from "node:fs"

const [layersFile, pictureFile, W, H, outFile] = process.argv.slice(2)
const w = +W, h = +H
// the canvas three asks for at the start; nothing is drawn to it (it makes its context only when drawing to it, 84040-84075)
const canvas = { width: w, height: h, style: {}, addEventListener() {}, removeEventListener() {}, getContext() { throw new Error("not drawn to") } }
const renderer = new THREE.WebGPURenderer({ alpha: false, antialias: false, canvas })
await renderer.init()
// Dawn's binding has no copyExternalImageToTexture (GPUQueue.cpp:120-125: there is no browser image in Node); done as
// Mystral Native does it (src/webgpu/bindings.cpp:1266-1500, MIT): the source's pixels read, flipped if asked, alpha
// multiplied in if the destination asks, then writeTexture (the WebGPU spec's steps for this call)
const queue: any = renderer.backend.device.queue
queue.copyExternalImageToTexture = (source: any, destination: any, size: any) => {
  const from = source.source, w = size.width ?? size[0], h = size.height ?? size[1]
  const rgba = new Uint8Array(from.getContext("2d").getImageData(source.origin?.x ?? 0, source.origin?.y ?? 0, w, h).data)
  if (source.flipY) for (let y = 0; y < h >> 1; y++) { const a = rgba.slice(y * w * 4, (y + 1) * w * 4); rgba.copyWithin(y * w * 4, (h - 1 - y) * w * 4, (h - y) * w * 4); rgba.set(a, (h - 1 - y) * w * 4) }
  if (destination.premultipliedAlpha) for (let i = 0; i < rgba.length; i += 4) { const a = rgba[i + 3] / 255; rgba[i] *= a; rgba[i + 1] *= a; rgba[i + 2] *= a }
  queue.writeTexture({ texture: destination.texture, mipLevel: destination.mipLevel ?? 0, origin: destination.origin ?? { x: 0, y: 0, z: 0 } }, rgba, { bytesPerRow: w * 4, rowsPerImage: h }, { width: w, height: h, depthOrArrayLayers: 1 })
}
renderer.outputColorSpace = THREE.SRGBColorSpace
renderer.toneMapping = THREE.NoToneMapping

const picture = new THREE.DataTexture(new Uint8Array(fs.readFileSync(pictureFile)), w, h, THREE.RGBAFormat)
picture.colorSpace = THREE.SRGBColorSpace
picture.flipY = false
picture.needsUpdate = true

// ICS tilt-shift (webgpu-tiltshift.ts tiltShiftColor): their mix, fed this picture instead of the cube scene.
// TARGET's y is the sharp line.
if (layersFile === "tilt-shift") {
  const { tiltShiftColor } = await import("../../third_party/tsl/ics-creative__250423_vite_threejs_tsl/src/tilt-shift-color.ts")
  const focus = process.env.TARGET ? +process.env.TARGET.split(",")[1] : 0.5
  const target = new THREE.RenderTarget(w, h, { type: THREE.UnsignedByteType })
  const material = new THREE.MeshBasicNodeMaterial()
  material.colorNode = TSL.renderOutput(tiltShiftColor(TSL.texture(picture), focus), THREE.NoToneMapping, THREE.SRGBColorSpace)
  const quad = new THREE.QuadMesh(material)
  for (let i = 0; i < 8; i++) {
    renderer.setRenderTarget(target)
    quad.render(renderer)
    await new Promise(r => setTimeout(r, 30))
  }
  const pixels = await renderer.readRenderTargetPixelsAsync(target, 0, 0, w, h)
  const stride = Math.ceil(w * 4 / 256) * 256, rows = Buffer.alloc(w * h * 4), got = Buffer.from(pixels.buffer, pixels.byteOffset, pixels.byteLength)
  for (let y = 0; y < h; y++) got.copy(rows, y * w * 4, y * stride, y * stride + w * 4)
  fs.writeFileSync(outFile, rows)
  console.log("drawn", w, h)
  process.exit(0)
}

// a list of types ("bloom,ascii") makes each a new layer with its definition's own defaults, as createLayer does
const { getLayerDefinition } = await import("../../third_party/tsl/basementstudio__shader-lab/src/lib/editor/config/layer-registry")
const { buildParameterValues } = await import("../../third_party/tsl/basementstudio__shader-lab/src/lib/editor/parameter-schema")
const layers = layersFile.endsWith(".json") ? JSON.parse(fs.readFileSync(layersFile, "utf8")) : layersFile.split(",").map((type: string) => {
  const d: any = getLayerDefinition(type as any)
  return { id: type, type, kind: d.kind, name: type, visible: true, opacity: 1, blendMode: "normal", compositeMode: "filter", hue: 0, saturation: 1, params: buildParameterValues(d.params) }
})
const config = { composition: { width: 1920, height: 1080 }, layers, timeline: { duration: 6, loop: true, tracks: [] } }
// TARGET=x,y (0..1 on the picture) is a thing's place. The passes that follow a pointer read globalThis.motoliiTarget.
const hostPoint = process.env.TARGET
if (hostPoint) {
  const [x, y] = hostPoint.split(",").map(Number)
  ;(globalThis as any).motoliiTarget = { x, y }
}
const headless = createHeadlessRenderer({ renderer, size: { width: w, height: h } })
await headless.initialize()
headless.resize({ width: w, height: h }, 1)
const frame = buildRendererFrame(config as any, 0, 1 / 60, 1, { width: w, height: h }, { logicalSize: { width: 1920, height: 1080 } })
let out = headless.render(frame, picture)
// its passes are compiled in the background and take effect once ready (pipeline-manager.ts: compilingPasses): draw
// again, letting that work finish in between, until the picture stops changing
// (and its clock running at 60 a second, as the browser's: some passes step only when the time moves, blob tracking's tracker)
for (let i = 1; i <= 40; i++) {
  await new Promise(r => setTimeout(r, 50))
  if (hostPoint) {
    const [x, y] = hostPoint.split(",").map(Number)
    const t = i / 40
    ;(globalThis as any).motoliiTarget = { x: 0.5 + (x - 0.5) * t, y: 0.5 + (y - 0.5) * t }
  }
  out = headless.render(buildRendererFrame(config as any, i / 60, 1 / 60, 1, { width: w, height: h }, { logicalSize: { width: 1920, height: 1080 } }), picture)
}

const target = new THREE.RenderTarget(w, h, { type: THREE.UnsignedByteType })
const material = new THREE.MeshBasicNodeMaterial()
material.colorNode = TSL.renderOutput(TSL.texture(out), THREE.NoToneMapping, THREE.SRGBColorSpace)
const quad = new THREE.QuadMesh(material)
renderer.setRenderTarget(target)
quad.render(renderer)
const pixels = await renderer.readRenderTargetPixelsAsync(target, 0, 0, w, h)
// (rows come back padded to a multiple of 256 bytes, WebGPU's copy rule, the last one not)
const stride = Math.ceil(w * 4 / 256) * 256, rows = Buffer.alloc(w * h * 4), got = Buffer.from(pixels.buffer, pixels.byteOffset, pixels.byteLength)
for (let y = 0; y < h; y++) got.copy(rows, y * w * 4, y * stride, y * stride + w * 4)
fs.writeFileSync(outFile, rows)
// timing (TIME_FRAMES=n): n more pictures, each drawn by the effects and read back, as Motolii would take each frame
const n = +(process.env.TIME_FRAMES ?? 0)
if (n > 0) {
  const t0 = performance.now(); let worst = 0, js = 0
  for (let i = 0; i < n; i++) {
    const a = performance.now()
    out = headless.render(buildRendererFrame(config as any, (41 + i) / 60, 1 / 60, 1, { width: w, height: h }, { logicalSize: { width: 1920, height: 1080 } }), picture)
    material.colorNode = TSL.renderOutput(TSL.texture(out), THREE.NoToneMapping, THREE.SRGBColorSpace)
    renderer.setRenderTarget(target); quad.render(renderer)
    js += performance.now() - a
    await renderer.readRenderTargetPixelsAsync(target, 0, 0, w, h)
    worst = Math.max(worst, performance.now() - a)
  }
  console.log(`${n} frames: ${((performance.now() - t0) / n).toFixed(1)} ms each with reading back (worst ${worst.toFixed(1)}); ` +
    `of it, the author's JS giving the commands ${(js / n).toFixed(1)} ms`)
}
console.log("drawn", w, h)
process.exit(0)
