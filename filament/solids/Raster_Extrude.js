// Raster Extrude: the thing's own outline pushed out into depth, its picture on the front and back and its edge drawn
// back along the sides (Alight Motion's Raster Extrude). The outline: d3-contour (ISC: marching squares, smoothed by
// linear interpolation) over how covered the picture is (k.alpha: its coverage 0-255, row by row from the top, at most
// Precision cells along its longer side); pushed out by three.js ExtrudeGeometry (MIT, its holes as Shape.holes).
// --track@depth:Depth,0.01,2,0.2,0.01
// --track@precision:Precision,10,800,160,1
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
import '../third_party/js/d3-array/d3-array.min.js';
import '../third_party/js/d3-contour/d3-contour.min.js';
import { faceUV } from './face_uv.js';
export function solid(k) {
    if (!k.alpha) return new THREE.BufferGeometry();   // (its picture not read yet: nothing, till the next)
    const { width: gw, height: gh, values } = k.alpha;
    const [outline] = globalThis.d3.contours().size([gw, gh]).thresholds([128])(values);
    const ring = (r, into) => { r.forEach(([x, y], i) => i ? into.lineTo(x, -y) : into.moveTo(x, -y)); return into; };   // (GeoJSON's rings, y down: three's, y up)
    const shapes = outline.coordinates.map(([outer, ...holes]) => { const s = ring(outer, new THREE.Shape()); s.holes = holes.map(h => ring(h, new THREE.Path())); return s; });
    const d = k.depth * k.h;
    const g = new THREE.ExtrudeGeometry(shapes, { depth: d * gh / k.h, bevelEnabled: false, UVGenerator: faceUV(0, -gh, gw, gh) });
    return g.translate(-gw / 2, gh / 2, -d * gh / k.h / 2).scale(k.w / gw, k.h / gh, k.h / gh);
}
