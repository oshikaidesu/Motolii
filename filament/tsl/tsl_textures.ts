// boytchev/tsl-textures (MIT): every texture it exports, with its own defaults, put in the material slot its own page
// puts it in ("model.material.normalNode = concrete ( {", docs/concrete.md:35; colorNode when its page says so or
// says nothing, as README.md:125).
import * as std from "qjs:std"
import * as THREE from "three/webgpu"
import * as T from "../third_party/tsl/boytchev__tsl-textures/src/tsl-textures.js"
import { convertAll } from "./convert"

const docs = "../third_party/tsl/boytchev__tsl-textures/docs/"
const pages = (std.popen(`ls ${docs}`, "r").readAsString() ?? "").split("\n").filter(f => f.endsWith(".md"))
const said = pages.map(p => std.loadFile(docs + p) ?? "")
const slotsOf = (name: string) => {
  const slots = new Set<string>()
  for (const text of said)
    for (const m of text.matchAll(new RegExp(`material\\.(\\w+Node) = ${name}\\b`, "g"))) slots.add(m[1])
  return slots.size ? [...slots] : ["colorNode"]
}

const effects = Object.entries(T)
  .filter(([, f]: any) => typeof f === "function" && f.defaults)
  .map(([name, f]: any) => [name, () => {
    const material: any = new THREE.MeshBasicNodeMaterial()
    for (const slot of slotsOf(name)) material[slot] = f({})
    return material
  }] as [string, () => any])
convertAll("tsl-textures", effects)
