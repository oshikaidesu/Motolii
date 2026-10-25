// Cube: the thing's picture on each of the six faces of a cube as high as it is (Alight Motion's Cube; three.js
// BoxGeometry, and examples/jsm RoundedBoxGeometry for Smoothing: MIT). A square picture is not stretched.
// --track@size:Size,0.05,4,1,0.01
// --track@smoothing:Smoothing,0,0.5,0,0.01
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
import { RoundedBoxGeometry } from '../third_party/js/three/RoundedBoxGeometry.js';
export function solid(k) {
    const a = k.size * k.h;
    return k.smoothing > 0 ? new RoundedBoxGeometry(a, a, a, 4, k.smoothing * a) : new THREE.BoxGeometry(a, a, a);
}
