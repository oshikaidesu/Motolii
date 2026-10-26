// requestAdapter, requestDevice, a buffer written and read back, on Motolii's dawn.node in Motolii's QuickJS (jsrh_napi).
import "./web_globals"
import * as std from "qjs:std"
const dawn = (globalThis as any).loadAddon(std.getenv("DAWN_NODE"))
Object.assign(globalThis, dawn.globals)
const gpu = dawn.create([])
const adapter = await gpu.requestAdapter()
const device = await adapter.requestDevice()
const src = device.createBuffer({ size: 16, usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST })
const dst = device.createBuffer({ size: 16, usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST })
device.queue.writeBuffer(src, 0, new Uint32Array([1, 2, 3, 4]))
const enc = device.createCommandEncoder()
enc.copyBufferToBuffer(src, 0, dst, 0, 16)
device.queue.submit([enc.finish()])
await dst.mapAsync(GPUMapMode.READ)
print("adapter", adapter.info.vendor, "readback", Array.from(new Uint32Array(dst.getMappedRange())).join(","))
dst.unmap()
device.destroy()
