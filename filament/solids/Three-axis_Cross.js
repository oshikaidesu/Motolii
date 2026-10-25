// Three-axis Cross: the thing's picture on three bars crossing along x, y and z (Alight Motion's Three-axis Cross;
// three.js BoxGeometry, three of them made one by examples/jsm BufferGeometryUtils.mergeGeometries: MIT)
// --track@length:Length,0.1,4,1,0.01
// --track@thickness:Thickness,0.01,1,0.3,0.01
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
import { mergeGeometries } from '../third_party/js/three/BufferGeometryUtils.js';
export function solid(k) {
    const a = k.length * k.h, t = k.thickness * k.h;
    return mergeGeometries([new THREE.BoxGeometry(a, t, t), new THREE.BoxGeometry(t, a, t), new THREE.BoxGeometry(t, t, a)]);
}
