// Heart: the thing's picture on a heart pushed out into depth (Alight Motion's Heart; the heart of three.js's own
// example examples/webgl_geometry_shapes.html, ExtrudeGeometry, its bevel for Smoothing: MIT)
// --track@size:Size,0.05,4,1,0.01
// --track@depth:Depth,0.01,2,0.3,0.01
// --track@smoothing:Smoothing,0,0.2,0.03,0.005
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
import { faceUV } from './face_uv.js';
export function solid(k) {
    const x = 0, y = 0;   // (as the example says it, upside down: turned below, as it turns its mesh)
    const heartShape = new THREE.Shape()
        .moveTo( x + 25, y + 25 )
        .bezierCurveTo( x + 25, y + 25, x + 20, y, x, y )
        .bezierCurveTo( x - 30, y, x - 30, y + 35, x - 30, y + 35 )
        .bezierCurveTo( x - 30, y + 55, x - 10, y + 77, x + 25, y + 95 )
        .bezierCurveTo( x + 60, y + 77, x + 80, y + 55, x + 80, y + 35 )
        .bezierCurveTo( x + 80, y + 35, x + 80, y, x + 50, y )
        .bezierCurveTo( x + 35, y, x + 25, y + 25, x + 25, y + 25 );
    const box = new THREE.Box2().setFromPoints(heartShape.getPoints(32));
    const high = box.max.y - box.min.y, by = k.size * k.h / high, bevel = k.smoothing * high;
    const g = new THREE.ExtrudeGeometry(heartShape, { depth: k.depth * high, bevelEnabled: bevel > 0, bevelSize: bevel, bevelThickness: bevel, bevelSegments: 4, curveSegments: 32,
        UVGenerator: faceUV(box.max.x, box.max.y, box.min.x - box.max.x, box.min.y - box.max.y) });   // (turned half round: the picture the right way up)
    return g.rotateZ(Math.PI).center().scale(by, by, by);
}
