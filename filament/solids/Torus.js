// Torus: the thing's picture round a ring (Alight Motion's Torus; three.js TorusGeometry: MIT): once along the ring
// and once round the tube.
// --track@radius:Radius,0.05,4,1,0.01
// --track@tube:Tube,0.01,1,0.35,0.01
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
export function solid(k) {
    const r = k.radius * k.h / 2;
    return new THREE.TorusGeometry(r, k.tube * r, 48, 128);
}
