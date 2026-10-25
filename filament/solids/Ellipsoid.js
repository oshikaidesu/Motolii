// Ellipsoid: the thing's picture round a ball drawn out each way (Alight Motion's Ellipsoid; three.js SphereGeometry,
// its own scale: MIT). The picture's middle towards the eye.
// --track@width:Width,0.05,4,1,0.01
// --track@height:Height,0.05,4,1,0.01
// --track@depth:Depth,0.05,4,1,0.01
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
export function solid(k) {
    return new THREE.SphereGeometry(k.h / 2, 96, 48, -Math.PI / 2).scale(k.width, k.height, k.depth);
}
