// Star Prism: the thing's picture on a star pushed out into depth (Alight Motion's Star Prism; the star of three.js's
// own example examples/webgl_geometry_extrude_shapes.html, ExtrudeGeometry, its bevel for Smoothing: MIT)
// --track@points:Points,3,16,5,1
// --track@inner:Inner,0.05,1,0.5,0.01
// --track@size:Size,0.05,4,1,0.01
// --track@depth:Depth,0.01,2,0.3,0.01
// --track@smoothing:Smoothing,0,0.2,0,0.005
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
import { faceUV } from './face_uv.js';
export function solid(k) {
    const pts2 = [], numPts = Math.round(k.points), r = k.size * k.h / 2;
    for ( let i = 0; i < numPts * 2; i ++ ) {
        const l = i % 2 == 1 ? k.inner * r : r;
        const a = i / numPts * Math.PI + Math.PI / 2;   // (a point straight up)
        pts2.push( new THREE.Vector2( Math.cos( a ) * l, Math.sin( a ) * l ) );
    }
    const shape2 = new THREE.Shape( pts2 ), bevel = k.smoothing * 2 * r;
    const g = new THREE.ExtrudeGeometry(shape2, { depth: k.depth * 2 * r, bevelEnabled: bevel > 0, bevelSize: bevel, bevelThickness: bevel, bevelSegments: 4, UVGenerator: faceUV(-r, -r, 2 * r, 2 * r) });
    return g.center();
}
