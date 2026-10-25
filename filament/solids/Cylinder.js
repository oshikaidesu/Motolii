// Cylinder: the thing's picture once round a cylinder as high as it is, its middle towards the eye, the ends round
// (Alight Motion's Cylinder; three.js CylinderGeometry: MIT). Radius 1: as far round as the picture is wide.
// --track@radius:Radius,0.05,4,1,0.01
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
export function solid(k) {
    const r = k.radius * k.w / (2 * Math.PI);
    return new THREE.CylinderGeometry(r, r, k.h, 96, 1, false, -Math.PI);   // (from behind: u 0.5, the picture's middle, at +z)
}
