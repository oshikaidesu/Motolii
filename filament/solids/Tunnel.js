// Tunnel: the thing's picture on the inside of a tube going away from where the thing is, again and again along it
// (Alight Motion's Tunnel; three.js CylinderGeometry, open-ended: MIT). Its UVs times Repeat, the picture repeated
// (userData.repeat: the solid's sampler wraps).
// --track@radius:Radius,0.05,4,1,0.01
// --track@length:Length,0.1,40,8,0.1
// --track@repeat:Repeat,1,40,6,1
// --check@lit:Lit,0
import * as THREE from '../third_party/js/three/three.core.js';
export function solid(k) {
    const r = k.radius * k.h / 2, long = k.length * k.h;
    const g = new THREE.CylinderGeometry(r, r, long, 64, 1, true).rotateX(-Math.PI / 2).translate(0, 0, -long / 2);   // (its mouth on the thing)
    const uv = g.attributes.uv;
    for (let i = 0; i < uv.count; i++) uv.setXY(i, 2 * uv.getX(i), k.repeat * uv.getY(i));   // (twice round, Repeat times along)
    g.userData.repeat = true;
    return g;
}
