// p5.js with no page: a sketch from the net, its file as it is, run by p5.js itself (third_party/js/p5, 1.11.13, LGPL-2.1,
// its file as it is released) in the script's QuickJS, as Cavalry's and AviUtl's scripts are run by stand-ins for what they
// ask of their host. No browser and nothing drawn: what p5 draws in WEBGL is taken at drawBuffersScaled, the one door its
// box, plane, ellipsoid, cylinder, cone and torus all go through (src/webgl/p5.RendererGL.Retained.js), with its model
// matrix (uModelMatrix), and said as the copies of a thing. A frame once run is kept: a sketch only goes forward, so going
// back is reading. Frames are its own 60 a second, as requestAnimationFrame and a clock stepped by hand (timesnap's way).
#pragma once

static const char* const p5_before = R"(
var __T = 0, __Q = [];
function __any(name) {
  const f = function () {};
  return new Proxy(f, {
    get(t, k) { if (k === Symbol.toPrimitive) return () => 0; if (k === Symbol.iterator) return function* () {}; if (k === 'length') return 0; if (k === 'then') return undefined; if (k in t) return t[k]; return t[k] = __any(name + '.' + String(k)); },
    set(t, k, v) { t[k] = v; return true; },
    apply() { return __any(name + '()'); }, construct() { return __any('new ' + name); },
  });
}
function __element(tag) {
  const kids = [], name = String(tag).toUpperCase();
  const e = { tagName: name, nodeName: name, style: {}, width: 100, height: 100, id: '', className: '', dataset: {}, childNodes: kids, children: kids, classList: { add() {}, remove() {}, contains() { return false; } },
    appendChild(c) { kids.push(c); c.parentNode = e; return c; }, removeChild() {}, remove() {}, setAttribute() {}, getAttribute() { return null; }, addEventListener() {}, removeEventListener() {}, getBoundingClientRect() { return { left: 0, top: 0, width: e.width, height: e.height }; },
    querySelector() { return null; }, querySelectorAll() { return []; }, getElementsByTagName() { return []; }, focus() {}, blur() {},
    getContext(kind) { const c = __any('context'); c.getParameter = () => 16384; return c; } };   // (how large a texture may be: p5 clamps the canvas to it)
  e.scrollWidth = e.clientWidth = 100, e.parentNode = null;
  return e;
}
var __main = __element('main');
var __page = { window: globalThis, self: globalThis, document: Object.assign(__element('document'), { readyState: 'complete', body: __element('body'), documentElement: __element('html'), head: __element('head'), createElement: __element, createElementNS: (_, t) => __element(t), createTextNode: () => __element('text'), getElementById: () => null, getElementsByTagName: t => t === 'main' ? [__main] : [], hasFocus: () => true, fonts: __any('fonts'), hidden: false }),
  location: { href: '', search: '', pathname: '/', hostname: 'localhost', protocol: 'file:' }, screen: { width: 1920, height: 1080 }, navigator: { userAgent: '', platform: '', language: 'en' },
  mocha: true,   // (p5 starts by itself after a promise, not when it is run under a test: new p5() below is the start)
  innerWidth: 1920, innerHeight: 1080, devicePixelRatio: 1, pageXOffset: 0, pageYOffset: 0, scrollX: 0, scrollY: 0,
  addEventListener() {}, removeEventListener() {}, dispatchEvent() {}, getComputedStyle: () => ({ getPropertyValue: () => '' }), matchMedia: () => ({ matches: false, addListener() {}, addEventListener() {} }),
  requestAnimationFrame: f => (__Q.push(f), __Q.length), cancelAnimationFrame() {}, setTimeout: () => 0, clearTimeout() {}, setInterval: () => 0, clearInterval() {},
  performance: { now: () => __T, mark() {}, measure() {} }, console: { log() {}, warn() {}, error() {}, info() {}, debug() {}, trace() {} }, URL: { createObjectURL: () => '' }, fetch: () => Promise.reject(new Error('no net')),
  Image: function () { return __element('img'); } };
for (const k of ['HTMLElement', 'HTMLCanvasElement', 'Element', 'Node', 'ImageData', 'FileReader', 'Blob', 'XMLHttpRequest', 'DOMMatrix', 'Path2D', 'WebGL2RenderingContext', 'WebGLRenderingContext', 'Event', 'MouseEvent', 'TouchEvent', 'KeyboardEvent', 'WheelEvent', 'CustomEvent', 'FontFace']) __page[k] = function () {};
for (const k in __page) Object.defineProperty(globalThis, k, { value: __page[k], writable: true, configurable: true });
)";

static const char* const p5_after = R"(
var __drawn = [], __kinds = {}, __U = 0.01;   // (a pixel of the sketch in the world's unit: as it comes half the stage's height, new p5() below)
p5.RendererGL.prototype.drawBuffersScaled = function (gId, x, y, z) {
  const m = Array.from(this.uModelMatrix.mat4);
  for (const i of [1, 4, 6, 9, 13]) m[i] = -m[i];   // (p5's y goes down, the world's up)
  const sx = Math.hypot(m[0], m[1], m[2]) || 1, sy = Math.hypot(m[4], m[5], m[6]) || 1, sz = Math.hypot(m[8], m[9], m[10]) || 1, deg = 180 / Math.PI;
  const of = String(gId).split('|')[0], whole = of === 'ellipsoid' ? 2 : 1;   // (an ellipsoid told by its radii; the world's ball is as wide as 1, as p5's box)
  __kinds[of] = (__kinds[of] || 0) + 1;
  __drawn.push(m[12] * __U, m[13] * __U, m[14] * __U, whole * x * sx * __U, whole * y * sy * __U, whole * z * sz * __U, Math.atan2(m[1] / sx, m[0] / sx) * deg, Math.asin(Math.max(-1, Math.min(1, -m[2] / sx))) * deg, Math.atan2(m[6] / sy, m[10] / sz) * deg);
  return this;
};
)";

static const char* const p5_start = R"(
new p5();
__U = 2 / (globalThis.height || 400);
var __frames = [];
function __to(f) {
  while (__frames.length <= f) {
    const was = globalThis.frameCount, q = __Q, last = __frames[__frames.length - 1];
    if (q.length) __drawn = [];
    __T += 1000 / 60, __Q = [];
    q.forEach(g => g(__T));
    __frames.push(last && globalThis.frameCount === was ? last : Float64Array.from(__drawn));   // (a noLoop(), a frameRate(30): as it was)
  }
}
function __most() {   // (as many copies as its first seconds draw at once, a second of waiting at most)
  const from = Date.now();
  for (let f = 1; f < 300 && Date.now() - from < 1000; ++f) __to(f);
  return Math.max(...__frames.map(c => c.length / 9));
}
function __shape() {   // (the world's shape nearest what it draws most of)
  const most = Object.keys(__kinds).sort((a, b) => __kinds[b] - __kinds[a])[0];
  return { ellipsoid: 'ball', torus: 'ring' }[most] || 'cube';
}
function each(i, n, t, k) {
  const f = Math.max(0, Math.round(t * 60));
  __to(f);
  const c = __frames[f], at = i * 9;
  return at < c.length ? { x: c[at], y: c[at + 1], z: c[at + 2], scale_x: c[at + 3], scale_y: c[at + 4], scale_z: c[at + 5], rotate: c[at + 6], rotate_y: c[at + 7], rotate_x: c[at + 8] } : { size: 0 };
}
)";
