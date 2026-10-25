// Hexagonal Prism: the thing's picture once round a six-sided prism as high as it is (Alight Motion's Hexagonal
// Prism; three.js CylinderGeometry with six sides, made flat-faced by its own toNonIndexed and computeVertexNormals: MIT)
// --track@radius:Radius,0.05,4,1,0.01
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
export function solid(k) {
    const r = k.radius * k.w / 6;   // (a hexagon's side is its radius: round it, as wide as the picture)
    const g = new THREE.CylinderGeometry(r, r, k.h, 6, 1, false, -Math.PI).toNonIndexed();
    g.computeVertexNormals();
    return g;
}
