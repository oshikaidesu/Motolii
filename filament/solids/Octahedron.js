// Octahedron: the thing's picture on an octahedron (Alight Motion's Octahedron; three.js OctahedronGeometry, its UVs
// as a ball's: MIT)
// --track@size:Size,0.05,4,1,0.01
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
export function solid(k) {
    return new THREE.OctahedronGeometry(k.size * k.h / 2);
}
