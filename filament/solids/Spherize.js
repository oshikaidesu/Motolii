// Spherize: the thing's picture read as an equirectangular one round a ball, as Alight Motion's Spherize reads it
// (three.js SphereGeometry, whose UVs are equirectangular: MIT). The picture's middle towards the eye.
// --track@size:Size,0.05,4,1,0.01
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
export function solid(k) {
    return new THREE.SphereGeometry(k.size * k.h / 2, 96, 48, -Math.PI / 2);
}
