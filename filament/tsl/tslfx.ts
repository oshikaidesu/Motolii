// verekia/tslfx (MIT): every effect it exports, as its README's "Vanilla Three.js" uses one: `const { uniforms, nodes } =
// impact()`, then `material.colorNode = nodes.colorNode` (README.md:12-18); its other nodes (opacityNode, ...) likewise.
import * as THREE from "three/webgpu"
import * as FX from "../third_party/tsl/verekia__tslfx/src/shaders/index"
import { convertAll } from "./convert"

const effects: [string, () => any][] = []
for (const [name, f] of Object.entries(FX) as [string, any][]) {
  if (typeof f !== "function" || !/^[a-z]/.test(name)) continue
  let made: any
  try { made = f() } catch { continue }
  if (!made?.nodes) continue   // (its SDFs, noises and helpers: parts, not effects)
  effects.push([name, () => {
    const material: any = new THREE.MeshBasicNodeMaterial()
    for (const [slot, node] of Object.entries(f().nodes)) material[slot] = node
    material.transparent = true
    return material
  }])
}
convertAll("tslfx", effects)
