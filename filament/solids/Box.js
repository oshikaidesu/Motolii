// Box: the thing's picture on each of the six faces of a box as wide and high as it is (Alight Motion's Box;
// three.js BoxGeometry, and examples/jsm RoundedBoxGeometry for Smoothing: MIT)
// --track@depth:Depth,0.01,4,1,0.01
// --track@smoothing:Smoothing,0,0.5,0,0.01
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
import { RoundedBoxGeometry } from '../third_party/js/three/RoundedBoxGeometry.js';
export function solid(k) {   // (k: its knobs, and k.w, k.h: how wide and high the thing is, in its own units)
    const d = k.depth * k.h, round = k.smoothing * Math.min(k.w, k.h, d);
    return round > 0 ? new RoundedBoxGeometry(k.w, k.h, d, 4, round) : new THREE.BoxGeometry(k.w, k.h, d);
}
