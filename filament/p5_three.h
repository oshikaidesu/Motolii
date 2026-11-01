// three.js in a p5.js sketch's script (p5_stand.h), run after p5_after and before the library itself.
#pragma once

static const char* const p5_three = R"(
// A three.js scene with no WebGLRenderer: three.core.js makes everything else. What its renderer is told to draw is taken
// mesh by mesh, each pair of a shape and a material one part, a solid of its own in the world, its meshes that solid's copies
// (doc_door.h). Seen as its camera sees it: the eye's view brought to the world's, so the sketch's camera moves its solids
// and the world's own camera stays free.
var __parts = [], __part_of = new Map(), __three = null;
function __orbit(camera) {   // (what OrbitControls does to a camera it is given: it looks at the target, three.js's examples/jsm/controls; the hand on it is the world's camera here)
  const c = __any('controls');
  c.target = new THREE.Vector3(), c.update = () => { camera.lookAt(c.target); return true; };
  c.update();
  return c;
}
const __any_names = new Proxy({ OrbitControls: __orbit, MapControls: __orbit }, { get: (t, k) => k in t ? t[k] : __any(String(k)) });
function __three_stand() {
  const stand = { domElement: drawingContext.canvas, shadowMap: {}, capabilities: __any('capabilities'), info: __any('info'), xr: __any('xr'), getPixelRatio: () => 1, getSize: v => v.set(1920, 1080),
    setAnimationLoop(f) { if (f) { const go = t => { f(t), requestAnimationFrame(go); }; requestAnimationFrame(go); } }, render: (scene, camera) => __three_render(scene, camera) };
  const WebGLRenderer = function () { return new Proxy(stand, { get: (t, k) => k in t ? t[k] : () => undefined, set: (t, k, v) => (t[k] = v, true) }); };
  globalThis.THREE = new Proxy(Object.assign({}, THREE, { WebGLRenderer, WebGLRenderTarget: THREE.RenderTarget }), { get: (t, k) => k in t ? t[k] : __any(String(k)) });   // (what lives with the renderer, not in three.core.js)
}
function __three_part(mesh, material) {
  const key = mesh.geometry.uuid + '|' + material.uuid;
  let p = __part_of.get(key);
  if (p === undefined) {
    const g = mesh.geometry;
    g.computeBoundingBox();
    const b = g.boundingBox, c = b.getCenter(new THREE.Vector3());
    __part_of.set(key, p = __parts.length), __parts.push({ geometry: g, material, c, high: Math.max(b.max.y - b.min.y, 1e-6), most: 0, now: 0, frame: -1 });
  }
  return p;
}
function __three_render(scene, camera) {
  scene.updateMatrixWorld();
  if (!camera.parent) camera.updateMatrixWorld();
  const V = camera.matrixWorldInverse, deg = 180 / Math.PI, w = new THREE.Matrix4(), A = new THREE.Matrix4(), T = new THREE.Matrix4(), one = new THREE.Matrix4(), P = new THREE.Vector3();
  if (!__three) {   // (the distance its things stand from its eye is the stage: what its camera shows there fills it)
    const far = [];
    scene.traverseVisible(o => { if (o.isMesh) { const z = -P.setFromMatrixPosition(o.matrixWorld).applyMatrix4(V).z; if (z > 0) far.push(z); } });
    far.sort((a, b) => a - b);
    const d = Math.max(far.length ? far[far.length >> 1] : 5, 1e-3), high = camera.isPerspectiveCamera ? 2 * d * Math.tan(camera.fov * Math.PI / 360) / (camera.zoom || 1) : (camera.top - camera.bottom) / (camera.zoom || 1);
    __three = { d, s: 3.2154 / high };   // (the height the world's own camera sees at its stage: 2 · 6 · tan 15°)
  }
  const say = (mesh, m, world) => {
    const p = __three_part(mesh, m), q = __parts[p], e = A.copy(V).multiply(world).multiply(T.makeTranslation(q.c.x, q.c.y, q.c.z)).scale(P.set(q.high, q.high, q.high)).elements, s = __three.s;
    const sx = Math.hypot(e[0], e[1], e[2]) || 1, sy = Math.hypot(e[4], e[5], e[6]) || 1, sz = Math.hypot(e[8], e[9], e[10]) || 1;
    if (q.frame !== __T) q.frame = __T, q.now = 0;
    q.most = Math.max(q.most, ++q.now);
    __drawn.push(e[12] * s, e[13] * s, (e[14] + __three.d) * s, sx * s, sy * s, sz * s, Math.atan2(e[1] / sx, e[0] / sx) * deg, Math.asin(Math.max(-1, Math.min(1, -e[2] / sx))) * deg, Math.atan2(e[6] / sy, e[10] / sz) * deg, p, -1);
  };
  scene.traverseVisible(o => {
    if (!o.isMesh || !o.geometry || !o.geometry.attributes.position) return;
    const m = Array.isArray(o.material) ? o.material[0] : o.material;
    if (m.isShaderMaterial) return;   // (its look is its own GLSL, which a solid has not got; a sky dome over everything, as a rule)
    if (o.isInstancedMesh) for (let i = 0; i < o.count; ++i) o.getMatrixAt(i, one), say(o, m, w.copy(o.matrixWorld).multiply(one));
    else say(o, m, o.matrixWorld);
  });
}
var __whole = null;
function __three_whole() {   // (the solids a scene stands as. A part a pair; so many that they would crowd the timeline, the same shape in much the same colour one, then the same shape one)
  if (__whole) return __whole;
  __to(60);
  const tone = q => { const c = q.material.color; return c ? [c.r, c.g, c.b].map(v => Math.floor(Math.min(v, 0.99) * 3)).join('') : ''; };
  for (const by of [(q, i) => i, q => q.geometry.uuid + tone(q), q => q.geometry.uuid]) {
    const of = new Map();
    __whole = { of: __parts.map((q, i) => { const k = by(q, i); if (!of.has(k)) of.set(k, of.size); return of.get(k); }), first: [], most: [] };
    if (of.size <= 48) break;
  }
  __whole.of.forEach((w, i) => { if (__whole.first[w] === undefined) __whole.first[w] = i; __whole.most[w] = (__whole.most[w] || 0) + __parts[i].most; });
  return __whole;
}
function __three_glb(w) {   // (a part's shape as a .glb, what a solid is read from: the glTF 2.0 container's chunks, written out)
  const p = __three_whole().first[w], g = __parts[p].geometry, n = g.attributes.position.count;
  if (!g.attributes.normal) g.computeVertexNormals();
  const at = (a, size) => { const o = new Float32Array(n * size); for (let i = 0; i < n; ++i) { o[i * size] = a.getX(i); o[i * size + 1] = a.getY(i); if (size > 2) o[i * size + 2] = a.getZ(i); } return o; };
  const pos = at(g.attributes.position, 3), nrm = at(g.attributes.normal, 3), idx = g.index ? Uint32Array.from(g.index.array) : Uint32Array.from({ length: n }, (_, i) => i);
  const low = [0, 1, 2].map(k => { let v = Infinity; for (let i = k; i < pos.length; i += 3) v = Math.min(v, pos[i]); return v; }), high = [0, 1, 2].map(k => { let v = -Infinity; for (let i = k; i < pos.length; i += 3) v = Math.max(v, pos[i]); return v; });
  const json = JSON.stringify({ asset: { version: '2.0' }, scene: 0, scenes: [{ nodes: [0] }], nodes: [{ mesh: 0 }], meshes: [{ primitives: [{ attributes: { POSITION: 0, NORMAL: 1 }, indices: 2 }] }], buffers: [{ byteLength: pos.byteLength + nrm.byteLength + idx.byteLength }],
    bufferViews: [{ buffer: 0, byteOffset: 0, byteLength: pos.byteLength }, { buffer: 0, byteOffset: pos.byteLength, byteLength: nrm.byteLength }, { buffer: 0, byteOffset: pos.byteLength + nrm.byteLength, byteLength: idx.byteLength }],
    accessors: [{ bufferView: 0, componentType: 5126, count: n, type: 'VEC3', min: low, max: high }, { bufferView: 1, componentType: 5126, count: n, type: 'VEC3' }, { bufferView: 2, componentType: 5125, count: idx.length, type: 'SCALAR' }] });
  const pad = (4 - json.length % 4) % 4, J = json.length + pad, B = pos.byteLength + nrm.byteLength + idx.byteLength, all = new Uint8Array(12 + 8 + J + 8 + B), d = new DataView(all.buffer);
  d.setUint32(0, 0x46546C67, true), d.setUint32(4, 2, true), d.setUint32(8, all.length, true), d.setUint32(12, J, true), d.setUint32(16, 0x4E4F534A, true);
  for (let i = 0; i < J; ++i) all[20 + i] = i < json.length ? json.charCodeAt(i) : 32;
  d.setUint32(20 + J, B, true), d.setUint32(24 + J, 0x004E4942, true);
  all.set(new Uint8Array(pos.buffer), 28 + J), all.set(new Uint8Array(nrm.buffer), 28 + J + pos.byteLength), all.set(new Uint8Array(idx.buffer), 28 + J + pos.byteLength + nrm.byteLength);
  return globalThis.__bytes = all, all.length;
}
function __three_says(w) {   // (what a part is, for doc_door.h)
  const q = __parts[__three_whole().first[w]], m = q.material, c = m.color || new THREE.Color(1, 1, 1);
  return [__whole.most[w], c.getHexString(), m.metalness === undefined ? 0 : m.metalness, m.roughness === undefined ? 1 : m.roughness].join(' ');
}
)";
