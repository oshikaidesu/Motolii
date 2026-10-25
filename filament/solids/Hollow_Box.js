// Hollow Box: the thing's picture on six plates of a box, each as thick as Thickness, a hole cut in each (Alight
// Motion's Hollow Box; three.js Shape with Path.absarc holes, ExtrudeGeometry, and examples/jsm
// BufferGeometryUtils.mergeGeometries to make the six one: MIT)
// --track@thickness:Thickness,0.005,0.5,0.05,0.005
// --select@cutout:Cutout,none,circle,square
// --track@progress:Progress,0,0.95,0.5,0.01
// --check@lit:Lit,1
import * as THREE from '../third_party/js/three/three.core.js';
import { mergeGeometries } from '../third_party/js/three/BufferGeometryUtils.js';
import { faceUV } from './face_uv.js';
export function solid(k) {
    const a = k.h, t = k.thickness * a, c = k.progress * a / 2;
    const plate = new THREE.Shape().moveTo(-a / 2, -a / 2).lineTo(a / 2, -a / 2).lineTo(a / 2, a / 2).lineTo(-a / 2, a / 2).lineTo(-a / 2, -a / 2);
    if (k.cutout == 'circle' && c > 0) plate.holes.push(new THREE.Path().absarc(0, 0, c, 0, 2 * Math.PI, true));
    if (k.cutout == 'square' && c > 0) plate.holes.push(new THREE.Path().moveTo(-c, -c).lineTo(-c, c).lineTo(c, c).lineTo(c, -c).lineTo(-c, -c));
    const one = new THREE.ExtrudeGeometry(plate, { depth: t, bevelEnabled: false, curveSegments: 48, UVGenerator: faceUV(-a / 2, -a / 2, a, a) }).translate(0, 0, a / 2 - t);
    const turns = [[0, 0], [0, Math.PI / 2], [0, Math.PI], [0, -Math.PI / 2], [Math.PI / 2, 0], [-Math.PI / 2, 0]];
    return mergeGeometries(turns.map(([x, y]) => one.clone().rotateY(y).rotateX(x)));
}
