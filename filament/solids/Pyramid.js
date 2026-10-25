// Pyramid: the thing's picture round a pyramid on a square base, as high as the thing (Alight Motion's Pyramid;
// three.js ConeGeometry with four sides, made flat-faced by its own toNonIndexed and computeVertexNormals: MIT)
// --track@base:Base,0.05,4,1,0.01
// --track@height:Height,0.05,4,1,0.01
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
export function solid(k) {
    const g = new THREE.ConeGeometry(k.base * k.h / Math.SQRT2, k.height * k.h, 4, 1, false, -3 * Math.PI / 4).toNonIndexed();   // (a face, not an edge, towards the eye)
    g.computeVertexNormals();
    return g;
}
