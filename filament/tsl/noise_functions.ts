// bandinopla/threejs-noise-functions (MIT): every noise it exports, shown as its README shows one:
// `colorNode: gabor12( uv().mul(6.0).add(time.sin()) )` (README.md:33). The last digit of a name is the size of its
// input (README.md:40-44: 1 a float, 2 a 2D vector, 3 a 3D vector), so the same input is made that size.
import * as THREE from "three/webgpu"
import { uv, time, vec3 } from "three/tsl"
import * as N from "../third_party/tsl/bandinopla__threejs-noise-functions/src/index"
import { convertAll } from "./convert"

const input = (size: string) => {
  const p = uv().mul(6.0).add(time.sin())
  return size === "1" ? p.x : size === "3" ? vec3(p, time.sin()) : p
}
const effects: [string, () => any][] = []
for (const [name, f] of Object.entries(N) as [string, any][]) {
  const size = /[123]([123])$/.exec(name)?.[1]
  if (typeof f !== "function" || !size) continue   // (its hashes and helpers)
  effects.push([name, () => new THREE.MeshBasicNodeMaterial({ colorNode: f(input(size)) } as any)])
}
convertAll("noise-functions", effects)
