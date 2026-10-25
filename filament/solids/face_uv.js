// The picture laid straight through an extruded shape, for the shapes below that are extruded (three.js
// ExtrudeGeometry's UVGenerator, the hook its options take): a point's x and y as shares of the box x0, y0, wide, high
// (three's v up); the sides take the picture's edge drawn back, as Alight Motion's Raster Extrude with Interior 0%.
import { Vector2 } from '../third_party/js/three/three.core.js';
export function faceUV(x0, y0, wide, high) {
    const at = (v, i) => new Vector2((v[i * 3] - x0) / wide, (v[i * 3 + 1] - y0) / high);
    return { generateTopUV: (g, v, a, b, c) => [at(v, a), at(v, b), at(v, c)], generateSideWallUV: (g, v, a, b, c, d) => [at(v, a), at(v, b), at(v, c), at(v, d)] };
}
