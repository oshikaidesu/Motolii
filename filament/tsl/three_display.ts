// three.js's own TSL post-processing (examples/jsm/tsl/display, MIT): each exported effect given one picture, as its
// examples hand them the scene's colour (webgpu_postprocessing_*.html: `bloom( scenePassColor )`, `film( scenePassColor )`).
// Those that also need the scene's depth, normals or camera say so in made.tsv.
import * as std from "qjs:std"
import * as THREE from "three/webgpu"
import { texture } from "three/tsl"
import { convertAll } from "./convert"

const dir = "node_modules/three/examples/jsm/tsl/display/"
const files = (std.popen(`ls ${dir}`, "r").readAsString() ?? "").split("\n").filter(f => f.endsWith(".js"))
const picture = new THREE.Texture()
const effects: [string, () => any][] = []
for (const file of files) {
  const module: any = await import(`./node_modules/three/examples/jsm/tsl/display/${file}`)   // (esbuild bundles every file this pattern can name)
  for (const [name, f] of Object.entries(module))
    if (typeof f === "function" && /^[a-z]/.test(name) && name !== "default")
      effects.push([name, () => {
        const material = new THREE.MeshBasicNodeMaterial()
        material.colorNode = (f as any)(texture(picture))
        return material
      }])
}
convertAll("three-display", effects)
