// Ring: the thing's picture on a flat-faced ring, a rectangle spun (Alight Motion's Ring; three.js
// LatheGeometry, made flat-faced by its own toNonIndexed and computeVertexNormals: MIT)
// --track@radius:Radius,0.05,4,1,0.01
// --track@width:Width,0.05,0.95,0.35,0.01
// --track@length:Length,0.05,2,0.28,0.01
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
export function solid(k) {
    const outer = k.radius * k.h / 2;
    const inner = Math.max(outer * (1 - k.width), 0.01);
    const half = k.length * k.h / 2;
    const pts = [
        new THREE.Vector2(inner, -half),
        new THREE.Vector2(outer, -half),
        new THREE.Vector2(outer, half),
        new THREE.Vector2(inner, half),
        new THREE.Vector2(inner, -half),
    ];
    const g = new THREE.LatheGeometry(pts, 8).toNonIndexed();
    g.computeVertexNormals();
    return g;
}
