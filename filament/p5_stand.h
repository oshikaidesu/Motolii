// p5.js with no page: a sketch from the net, its file as it is, run by p5.js itself (third_party/js/p5, 1.11.13, LGPL-2.1,
// its file as it is released) in the script's QuickJS, as Cavalry's and AviUtl's scripts are run by stand-ins for what they
// ask of their host. No browser and nothing drawn: what p5 draws in WEBGL is taken at drawBuffersScaled, the one door its
// box, plane, ellipsoid, cylinder, cone and torus all go through (src/webgl/p5.RendererGL.Retained.js), with its model
// matrix (uModelMatrix), and said as the copies of a thing. A frame once run is kept: a sketch only goes forward, so going
// back is reading. Frames are its own 60 a second, as requestAnimationFrame and a clock stepped by hand (timesnap's way).
#pragma once

static const char* const p5_before = R"(
var __T = 0, __Q = [], __seed = 12345, __L = 0;   // (__L: the line of the sketch that draws now: script.h sketch_marked)
function __at(line, it) { __L = line; return it; }
Math.random = () => (__seed = (1664525 * __seed + 1013904223) % 4294967296) / 4294967296;   // (the same film each time it is played: p5's own _lcg, src/math/random.js, from one seed)
function __any(name) {
  const f = function () {};
  return new Proxy(f, {
    get(t, k) { if (k === Symbol.toPrimitive) return () => 0; if (k === Symbol.iterator) return function* () {}; if (k === 'length') return 0; if (k === 'name') return __any(name + '.name'); if (k === 'then') return undefined; if (k in t) return t[k]; return t[k] = __any(name + '.' + String(k)); },
    set(t, k, v) { t[k] = v; return true; },
    apply() { return __any(name + '()'); }, construct() { return __any('new ' + name); },
  });
}
// A canvas with no pixels, as a 2D sketch paints on it: what the WHATWG canvas context is told, kept as numbers for ThorVG
// (drawn_sketch.h), each paint its kind (1 fill, 2 stroke, 3 words), colour, blend, line, how many numbers follow of its path
// (0 move, 1 line, 2 curve, 3 close) and of its gradient and the sketch's line that drew it, its points turned by the matrix as they are told, as the spec says. svgcanvas, which writes the same calls as SVG, took
// 45 ms a frame in QuickJS for 1161 shapes and this 6, and has no globalCompositeOperation.
const __blends = { multiply: 1, screen: 2, overlay: 3, darken: 4, lighten: 5, 'color-dodge': 6, 'color-burn': 7, 'hard-light': 8, 'soft-light': 9, difference: 10, exclusion: 11, hue: 12, saturation: 13, color: 14, luminosity: 15, lighter: 16 };   // (ThorVG's BlendMethod)
const __colours = new Map();
function __rgba(style) {   // (a colour as p5 itself reads it)
  let c = __colours.get(style);
  if (!c) __colours.set(style, c = p5.prototype.color.call(p5.instance, style).levels);
  return c;
}
function __flat(e) {
  const keep = ['fillStyle', 'strokeStyle', 'lineWidth', 'lineCap', 'lineJoin', 'miterLimit', 'globalAlpha', 'globalCompositeOperation', 'font', 'textAlign', 'textBaseline', 'shadowBlur', 'shadowColor', 'shadowOffsetX', 'shadowOffsetY', 'filter', 'imageSmoothingEnabled'], saved = [];
  let m = [1, 0, 0, 1, 0, 0], path = [], ux = 0, uy = 0, sx = 0, sy = 0, ops = [], clear = false, seen = 0;
  const X = (x, y) => m[0] * x + m[2] * y + m[4], Y = (x, y) => m[1] * x + m[3] * y + m[5];
  const M = (x, y) => { path.push(0, X(x, y), Y(x, y)), ux = sx = x, uy = sy = y; };
  const L = (x, y) => { if (!path.length) return M(x, y); path.push(1, X(x, y), Y(x, y)), ux = x, uy = y; };
  const C = (a, b, c, d, x, y) => { if (!path.length) M(a, b); path.push(2, X(a, b), Y(a, b), X(c, d), Y(c, d), X(x, y), Y(x, y)), ux = x, uy = y; };
  const large = () => Math.sqrt(Math.abs(m[0] * m[3] - m[1] * m[2]));
  const ramp = g => {   // (a gradient's ends where the matrix puts them when it is used, as the spec says; ThorVG's radial has them as outer and focal)
    const out = [g.kind, X(g.x0, g.y0), Y(g.x0, g.y0), g.r0 * large(), X(g.x1, g.y1), Y(g.x1, g.y1), g.r1 * large()];
    for (const [at, k] of g.stops.slice().sort((a, b) => a[0] - b[0])) { const q = __rgba(k); out.push(at, q[0], q[1], q[2], q[3] * c.globalAlpha); }
    return out;
  };
  const paint = (kind, style) => {
    const g = typeof style === 'string' ? null : style, k = g ? [255, 255, 255, 255] : __rgba(style), a = k[3] * c.globalAlpha, more = g && g.kind ? ramp(g) : [];
    if (!path.length || a <= 0) return;
    ops.push(kind, k[0], k[1], k[2], a, __blends[c.globalCompositeOperation] || 0, kind === 2 ? c.lineWidth * large() : 0, { round: 1, square: 2 }[c.lineCap] || 0, { round: 1, bevel: 2 }[c.lineJoin] || 0, path.length, more.length, __L);
    for (let i = 0; i < path.length; ++i) ops.push(path[i]);
    for (let i = 0; i < more.length; ++i) ops.push(more[i]);
  };
  const words = (t, x, y, style) => {   // (its matrix and its letters as numbers; a gradient's words white, for now)
    const k = typeof style === 'string' ? __rgba(style) : [255, 255, 255, 255], a = k[3] * c.globalAlpha, codes = Array.from(String(t), h => h.codePointAt(0));
    if (!codes.length || a <= 0) return;
    ops.push(3, k[0], k[1], k[2], a, __blends[c.globalCompositeOperation] || 0, parseFloat((/([\d.]+)px/.exec(c.font) || [0, 10])[1]), { center: 1, right: 2, end: 2 }[c.textAlign] || 0, { top: 1, hanging: 1, middle: 2, bottom: 3, ideographic: 3 }[c.textBaseline] || 0, 6 + codes.length, 0, __L, m[0], m[1], m[2], m[3], X(x, y), Y(x, y));
    for (let i = 0; i < codes.length; ++i) ops.push(codes[i]);
  };
  const whole_ink = k => typeof k === 'string' ? __rgba(k)[3] === 255 : k.kind && k.stops.every(q => __rgba(q[1])[3] === 255);
  const whole = (x, y, w, h) => X(x, y) <= 0 && Y(x, y) <= 0 && X(x + w, y + h) >= e.width && Y(x + w, y + h) >= e.height && !m[1] && !m[2];   // (a background(): what was painted before is gone)
  const box = (x, y, w, h) => { M(x, y), L(x + w, y), L(x + w, y + h), L(x, y + h), path.push(3), ux = sx, uy = sy; };
  const wipe = () => {   // (the spec's: a canvas told its size again is new; a page's own colour behind it, __door_canvas's sheet, is what it then shows)
    ops = [], clear = true, path = [], m = [1, 0, 0, 1, 0, 0];
    if (c) Object.assign(c, { fillStyle: '#000000', strokeStyle: '#000000', lineWidth: 1, globalAlpha: 1, globalCompositeOperation: 'source-over' });
    if (c && e.sheet) c.fillStyle = e.sheet, c.fillRect(0, 0, e.width, e.height), c.fillStyle = '#000000';
  };
  for (const side of ['width', 'height']) { let size = e[side]; Object.defineProperty(e, side, { get: () => size, set: v => { size = v, wipe(); }, configurable: true, enumerable: true }); }
  const alone = (f) => { const was = path; path = [], f(), path = was; };   // (fillRect and its like leave the path as it was)
  const c = { canvas: e, fillStyle: '#000000', strokeStyle: '#000000', lineWidth: 1, lineCap: 'butt', lineJoin: 'miter', miterLimit: 10, globalAlpha: 1, globalCompositeOperation: 'source-over', font: '10px sans-serif', textAlign: 'start', textBaseline: 'alphabetic', shadowBlur: 0, shadowColor: '', shadowOffsetX: 0, shadowOffsetY: 0, filter: 'none', imageSmoothingEnabled: true, lineDashOffset: 0,
    save() { saved.push([m.slice(), keep.map(k => c[k])]); },
    restore() { const s = saved.pop(); if (s) m = s[0], keep.forEach((k, i) => c[k] = s[1][i]); },
    setTransform(a, b, cc, d, x, y) { m = typeof a === 'object' ? [a.a, a.b, a.c, a.d, a.e, a.f] : [a, b, cc, d, x, y]; }, resetTransform() { m = [1, 0, 0, 1, 0, 0]; }, getTransform() { return { a: m[0], b: m[1], c: m[2], d: m[3], e: m[4], f: m[5] }; },
    transform(a, b, cc, d, x, y) { const [A, B, C2, D, E, F] = m; m = [A * a + C2 * b, B * a + D * b, A * cc + C2 * d, B * cc + D * d, A * x + C2 * y + E, B * x + D * y + F]; },
    translate(x, y) { c.transform(1, 0, 0, 1, x, y); }, scale(x, y) { c.transform(x, 0, 0, y, 0, 0); }, rotate(a) { const q = Math.cos(a), s = Math.sin(a); c.transform(q, s, -s, q, 0, 0); },
    beginPath() { path = []; }, closePath() { if (path.length) path.push(3), ux = sx, uy = sy; }, moveTo: M, lineTo: L, bezierCurveTo: C, rect: box,
    quadraticCurveTo(a, b, x, y) { if (!path.length) M(a, b); C(ux + 2 / 3 * (a - ux), uy + 2 / 3 * (b - uy), x + 2 / 3 * (a - x), y + 2 / 3 * (b - y), x, y); },   // (a quadratic is the cubic with its handles 2/3 of the way)
    ellipse(x, y, rx, ry, turn, from, to, back) {   // (at most a quarter to a curve, each with handles 4/3 tan(a/4) long: Goldapp 1991)
      const two = 2 * Math.PI, co = Math.cos(turn), si = Math.sin(turn);
      let go = to - from;
      if (!back) go = go >= two ? two : (go % two + two) % two; else go = go <= -two ? -two : -((-go % two + two) % two);
      const n = Math.max(1, Math.ceil(Math.abs(go) / (Math.PI / 2) - 1e-6)), d = go / n, k = 4 / 3 * Math.tan(d / 4);
      const A = (u, v) => x + rx * u * co - ry * v * si, B = (u, v) => y + rx * u * si + ry * v * co;
      let q = Math.cos(from), s = Math.sin(from);
      if (path.length) L(A(q, s), B(q, s)); else M(A(q, s), B(q, s));
      for (let i = 1; i <= n; ++i) { const t = from + i * d, Q = Math.cos(t), S = Math.sin(t); C(A(q - k * s, s + k * q), B(q - k * s, s + k * q), A(Q + k * S, S - k * Q), B(Q + k * S, S - k * Q), A(Q, S), B(Q, S)), q = Q, s = S; }
    },
    arc(x, y, r, from, to, back) { c.ellipse(x, y, r, r, 0, from, to, back); },
    arcTo(a, b, x, y, r) {   // (the circle touching both lines, as one curve from where it touches one to where it touches the other)
      const p = Math.hypot(ux - a, uy - b) || 1, q = Math.hypot(x - a, y - b) || 1, ix = (ux - a) / p, iy = (uy - b) / p, jx = (x - a) / q, jy = (y - b) / q, wide = Math.acos(Math.max(-1, Math.min(1, ix * jx + iy * jy))), far = r / Math.tan(wide / 2), k = 4 / 3 * Math.tan((Math.PI - wide) / 4) * r;
      if (!r || !isFinite(far)) return L(a, b);
      L(a + ix * far, b + iy * far), C(a + ix * (far - k), b + iy * (far - k), a + jx * (far - k), b + jy * (far - k), a + jx * far, b + jy * far);
    },
    roundRect(x, y, w, h, r) {   // (a corner each, as p5's rect() asks them; the spec's order: from the top left, clockwise)
      const [a, b, d, e] = Array.isArray(r) ? (r.length < 4 ? [r[0], r[0], r[0], r[0]] : r) : [r, r, r, r];
      M(x + a, y), c.arcTo(x + w, y, x + w, y + h, b), c.arcTo(x + w, y + h, x, y + h, d), c.arcTo(x, y + h, x, y, e), c.arcTo(x, y, x + w, y, a), path.push(3), ux = sx, uy = sy;
    },
    fill() { paint(1, c.fillStyle); }, stroke() { paint(2, c.strokeStyle); },
    fillRect(x, y, w, h) { if (whole(x, y, w, h) && whole_ink(c.fillStyle) && c.globalAlpha === 1 && !__blends[c.globalCompositeOperation]) ops = [], clear = true; alone(() => { box(x, y, w, h), paint(1, c.fillStyle); }); },
    strokeRect(x, y, w, h) { alone(() => { box(x, y, w, h), paint(2, c.strokeStyle); }); },
    clearRect(x, y, w, h) { if (whole(x, y, w, h)) ops = [], clear = true; },
    createLinearGradient(x0, y0, x1, y1) { const g = { kind: 1, x0, y0, r0: 0, x1, y1, r1: 0, stops: [], addColorStop(at, k) { g.stops.push([at, k]); } }; return g; },
    createRadialGradient(x0, y0, r0, x1, y1, r1) { return Object.assign(c.createLinearGradient(x0, y0, x1, y1), { kind: 2, r0, r1 }); }, createConicGradient() { return Object.assign(c.createLinearGradient(0, 0, 0, 0), { kind: 0 }); }, createPattern() { return { kind: 0, stops: [] }; },
    measureText(t) { return { width: String(t).length * parseFloat(c.font) * 0.5, actualBoundingBoxAscent: 0, actualBoundingBoxDescent: 0 }; },
    getImageData(x, y, w, h) {   // (no pixels but a film's: see __film_step)
      const data = new Uint8ClampedArray(4 * w * h), b = e.bytes;
      if (w === 1 && h === 1 && !b && typeof c.fillStyle === 'string') { const k = __rgba(c.fillStyle); data[0] = k[0], data[1] = k[1], data[2] = k[2], data[3] = k[3]; return { width: 1, height: 1, data }; }   // (a dot just painted read back: how Paper.js and others read a colour's CSS name)
      if (b) for (let r = 0, W = e.width; r < h; ++r) data.set(b.subarray(4 * ((y + r) * W + x), 4 * ((y + r) * W + x + w)), 4 * r * w);
      return { width: w, height: h, data };
    },
    createImageData(w, h) { return { width: w, height: h, data: new Uint8ClampedArray(4 * w * h) }; }, putImageData() {},
    fillText(t, x, y) { words(t, x, y, c.fillStyle); }, strokeText(t, x, y) { words(t, x, y, c.strokeStyle); },
    drawImage(im, a, b, w, h, dx, dy, dw, dh) {   // (another canvas, a createGraphics: what it holds painted again here)
      const from = im && im.flat;
      if (!from) return;
      if (dx === undefined) dx = a, dy = b, dw = w === undefined ? im.width : w, dh = h === undefined ? im.height : h, a = b = 0, w = im.width, h = im.height;
      const o = from.held(), sx = dw / w, sy = dh / h, U = x => dx + (x - a) * sx, V = y => dy + (y - b) * sy, grow = large() * Math.sqrt(Math.abs(sx * sy)), P = (x, y) => { const u = U(x), v = V(y); ops.push(X(u, v), Y(u, v)); };
      for (let i = 0; i < o.length;) {
        const kind = o[i], n = o[i + 9], more = o[i + 10], end = i + 12 + n;
        for (let j = 0; j < 12; ++j) ops.push(j === 6 ? (kind === 3 ? o[i + 6] * Math.sqrt(Math.abs(sx * sy)) : o[i + 6] * grow) : j === 11 ? __L : o[i + j]);   // (what another canvas holds shows where image() says it does)
        let q = i + 12;
        if (kind === 3) {
          ops.push(m[0] * sx * o[q] + m[2] * sy * o[q + 1], m[1] * sx * o[q] + m[3] * sy * o[q + 1], m[0] * sx * o[q + 2] + m[2] * sy * o[q + 3], m[1] * sx * o[q + 2] + m[3] * sy * o[q + 3]), P(o[q + 4], o[q + 5]), q += 6;
          while (q < end) ops.push(o[q++]);
        } else while (q < end) { const t = o[q++]; ops.push(t); for (let p = t === 2 ? 3 : t === 3 ? 0 : 1; p > 0; --p, q += 2) P(o[q], o[q + 1]); }
        if (more) { ops.push(o[q]), P(o[q + 1], o[q + 2]), ops.push(o[q + 3] * grow), P(o[q + 4], o[q + 5]), ops.push(o[q + 6] * grow); for (let j = q + 7; j < q + more; ++j) ops.push(o[j]); }
        i = end + more;
      }
    },
    clip() {}, setLineDash() {}, getLineDash() { return []; }, isPointInPath() { return false; }, isPointInStroke() { return false; }, getContextAttributes() { return {}; },
    held() { return ops; }, take() { const o = Float64Array.from(ops), was = clear; ops = [], clear = false; return [was, o]; },
    fresh() { const o = Float64Array.from(ops.slice(clear ? 0 : seen)), was = clear; clear = false, seen = ops.length; return [was, o]; } };   // (what came since it was last asked, what it holds staying: another canvas's own picture)
  return c;
}
function __element(tag) {
  const kids = [], name = String(tag).toUpperCase();
  const e = { tagName: name, nodeName: name, style: {}, width: 100, height: 100, id: '', className: '', dataset: {}, childNodes: kids, children: kids, classList: { add() {}, remove() {}, contains() { return false; } },
    appendChild(c) { kids.push(c); c.parentNode = e; return c; }, removeChild() {}, remove() {}, setAttribute() {}, getAttribute() { return null; }, hasAttribute() { return false; }, addEventListener() {}, removeEventListener() {}, getBoundingClientRect() { return { left: 0, top: 0, width: e.width, height: e.height }; },
    querySelector() { return __element('div'); }, querySelectorAll() { return []; }, getElementsByTagName() { return []; }, focus() {}, blur() {},
    append(...c) { c.forEach(k => typeof k === 'object' && e.appendChild(k)); }, prepend() {}, replaceChildren() { kids.length = 0; }, replaceWith() {}, insertBefore(c) { return e.appendChild(c); }, insertAdjacentHTML() {}, insertAdjacentElement() {}, after() {}, before() {},
    closest() { return null; }, matches() { return false; }, contains() { return false; }, click() {}, cloneNode() { return __element(tag); }, removeAttribute() {}, toggleAttribute() {}, scrollIntoView() {}, dispatchEvent() {}, setPointerCapture() {}, releasePointerCapture() {},
    getContext(kind) { if (kind === '2d') return e.flat = e.flat || __flat(e); const c = __any('context'); c.getParameter = () => 16384; return c; } };   // (how large a texture may be: p5 clamps the canvas to it)
  e.scrollWidth = e.clientWidth = 100, e.parentNode = null;
  return e;
}
var __main = __element('main'), __ids = {};   // (a page's parts it asks for by name: there, and nothing comes of them. p5 counts up its own canvases' names until one is not there)
var __page = { window: globalThis, self: globalThis, document: Object.assign(__element('document'), { readyState: 'complete', body: __element('body'), documentElement: __element('html'), head: __element('head'), createElement: __element, createElementNS: (_, t) => __element(t), createTextNode: () => __element('text'), getElementById: id => /^(defaultCanvas|p5_loading)/.test(id) ? null : __ids[id] = __ids[id] || Object.assign(__element('div'), { id, parentNode: __main }), getElementsByTagName: t => t === 'main' ? [__main] : [], hasFocus: () => true, fonts: __any('fonts'), hidden: false }),
  location: { href: '', search: '', pathname: '/', hostname: 'localhost', protocol: 'file:' }, screen: { width: 1920, height: 1080 }, navigator: { userAgent: '', platform: '', language: 'en' },
  mocha: true,   // (p5 starts by itself after a promise, not when it is run under a test: new p5() below is the start)
  innerWidth: 1920, innerHeight: 1080, devicePixelRatio: 1, pageXOffset: 0, pageYOffset: 0, scrollX: 0, scrollY: 0,
  addEventListener() {}, removeEventListener() {}, dispatchEvent() {}, getComputedStyle: () => ({ getPropertyValue: () => '' }), matchMedia: () => ({ matches: false, addListener() {}, addEventListener() {} }),
  requestAnimationFrame: f => (__Q.push(f), __Q.length), cancelAnimationFrame() {}, setTimeout: () => 0, clearTimeout() {}, setInterval: () => 0, clearInterval() {},
  performance: { now: () => __T, mark() {}, measure() {} }, console: { log() {}, warn() {}, error() {}, info() {}, debug() {}, trace() {} }, URL: { createObjectURL: () => '' }, fetch: () => Promise.reject(new Error('no net')),
  Image: function () { return __element('img'); } };
for (const k of ['Request', 'Headers', 'Response', 'HTMLElement', 'HTMLCanvasElement', 'Element', 'Node', 'ImageData', 'FileReader', 'Blob', 'XMLHttpRequest', 'DOMMatrix', 'Path2D', 'WebGL2RenderingContext', 'WebGLRenderingContext', 'Event', 'MouseEvent', 'TouchEvent', 'KeyboardEvent', 'WheelEvent', 'CustomEvent', 'FontFace']) __page[k] = function () {};
for (const k in __page) Object.defineProperty(globalThis, k, { value: __page[k], writable: true, configurable: true });
)";

static const char* const p5_after = R"(
// What a sketch's camera or video sees is the film under it: it asks as p5 lets it (createCapture, createVideo) and reads it
// as a p5.Image, by get() and pixels; its image(video) paints nothing, the film itself being there underneath. A picture
// it loads is that film too where there is one, so what reads a photo reads the film each frame.
var __film = null, __film_at = undefined;
function __film_first() { return __film_at === undefined ? __film_at = __film_next() : __film_at; }
function __film_open() {
  if (__film) return __film;
  const first = __film_first() || { w: 320, h: 180, data: null };
  const im = __film = new p5.Image(first.w, first.h), none = () => im;
  __film.source = first;
  Object.assign(im, { hide: none, show: none, play: none, pause: none, stop: none, loop: none, noLoop: none, autoplay: none, volume: none, speed: none, time: () => __T / 1000, duration: () => 600, position: none, style: none, parent: none, id: none, class: none, attribute: none, showControls: none, hideControls: none, elt: im.canvas,
    size(w, h) { if (w === undefined) return { width: im.width, height: im.height }; if (h === undefined || h === 'auto') h = w * first.h / first.w; if (w === 'auto') w = h * first.w / first.h; im.width = im.canvas.width = Math.round(w), im.height = im.canvas.height = Math.round(h); __film_show(); return im; } });
  __film_show();
  return im;
}
function __film_show() {   // (at the size it was asked for, each its nearest of the film's)
  const s = __film.source, w = __film.width, h = __film.height;
  if (!s.data) return;
  const from = new Uint8ClampedArray(s.data);
  if (w === s.w && h === s.h) { __film.canvas.bytes = from; return; }
  const out = new Uint8ClampedArray(4 * w * h);
  for (let y = 0, o = 0; y < h; ++y) { const r = Math.floor(y * s.h / h) * s.w; for (let x = 0; x < w; ++x, o += 4) { const p = 4 * (r + Math.floor(x * s.w / w)); out[o] = from[p], out[o + 1] = from[p + 1], out[o + 2] = from[p + 2], out[o + 3] = 255; } }
  __film.canvas.bytes = out;
}
function __film_step() { const next = __film_next(); if (next) __film.source = next, __film_show(); }
// What the forms of door.h paint on is p5's own canvas: asked for by a page's id or made, it is that one; a second made is another.
function __door_canvas() {
  const e = drawingContext.canvas, make = document.createElement;
  let given = false;
  e.ownerDocument = document, e.nodeType = 1, document.defaultView = globalThis, document.nodeType = 9;
  document.createElement = t => { if (/^canvas$/i.test(t) && !given) return given = true, e; return make.call(document, t); };
  document.getElementById = document.querySelector = () => (given = true, e);
  document.getElementsByTagName = document.querySelectorAll = () => (given = true, [e]);
  document.getElementsByClassName = () => (given = true, [e]);
}
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
var tokenData = { hash: '0x' + '6b3d'.repeat(16), tokenId: '1000000' }, fxhash = 'oo' + 'Mot0lii'.repeat(7), fxrand = Math.random, $fx = { hash: fxhash, rand: fxrand, minter: 'tz1', iteration: 1, context: 'standalone', preview() {}, isPreview: false, features() {}, getFeature() {}, getFeatures: () => ({}), params() {}, getParam() {}, getParams: () => ({}), on() {}, emit() {} };   // (what Art Blocks' and fxhash's pages give a work: a seed, here always the same)
// p5.sound is not here: what a sketch asks of it is there and silent, so that what it draws is drawn. It hears nothing yet.
for (const k of ['SoundFile', 'FFT', 'Amplitude', 'Oscillator', 'SinOsc', 'TriOsc', 'SawOsc', 'SqrOsc', 'Noise', 'Envelope', 'Env', 'AudioIn', 'Reverb', 'Delay', 'Filter', 'LowPass', 'HighPass', 'BandPass', 'PeakDetect', 'Part', 'Phrase', 'Score', 'SoundLoop', 'Gain', 'Pulse', 'MonoSynth', 'PolySynth', 'Panner3D', 'Distortion', 'Compressor', 'EQ', 'Convolver', 'SoundRecorder', 'Signal', 'Effect'])
  p5[k] = p5[k] || function () { return __any('p5.' + k); };
for (const k of ['loadSound', 'getAudioContext', 'userStartAudio', 'soundFormats', 'outputVolume', 'getOutputVolume', 'masterVolume', 'sampleRate'])
  globalThis[k] = globalThis[k] || function () { return __any(k); };
const __load_image = p5.prototype.loadImage;
p5.prototype.loadImage = function (path, ok) { if (!__film_first()) return __load_image.apply(this, arguments); const f = __film_open(); if (typeof ok === 'function') ok(f); return f; };
p5.prototype.createCapture = p5.prototype.createVideo = function () { const f = __film_open(), ready = [...arguments].find(a => typeof a === 'function'); if (ready) ready(f); return f; };
const __packed = c => c ? Math.round(c[0] * 255) << 16 | Math.round(c[1] * 255) << 8 | Math.round(c[2] * 255) : -1;   // (a copy's colour, red green blue; -1: its own solid's)
var __drawn = [], __kinds = {}, __U = 0.01;   // (a pixel of the sketch in the world's unit: as tall as the stage as it comes, new p5() below)
// What WEBGL draws, as p5 has it: which of its shapes where its camera sees them, the triangles and lines it makes as it goes,
// what it clears with. doc_sketch.h stands them in the world, one mesh each frame. p5 makes a line's corners in triangles
// for its line shader here too, 0.8 s a frame in the heaviest; nothing here reads them.
var __mesh = { draws: [], tris: [], lines: [], back: null, wiped: false }, __meshes = [], __mesh_held = 0, __mesh_gone = 0, __shapes = [], __shape_of = new Map(), __lined = false;
const __own = r => p5.instance && r === p5.instance._renderer;
function __seen(r, x, y, z) {   // (what its camera sees, what it draws at z 0 still at z 0: a camera of its own moves things from there)
  const e = r._curCamera.cameraMatrix.mat4, m = r.uModelMatrix.mat4, o = new Array(16);
  for (let c = 0; c < 4; ++c) for (let i = 0; i < 4; ++i) o[c * 4 + i] = e[i] * m[c * 4] + e[4 + i] * m[c * 4 + 1] + e[8 + i] * m[c * 4 + 2] + e[12 + i] * m[c * 4 + 3];
  for (let i = 0; i < 4; ++i) o[i] *= x, o[4 + i] *= y, o[8 + i] *= z;
  for (let c = 0; c < 4; ++c) o[c * 4 + 2] += 800 * o[c * 4 + 3];   // (p5.Camera's defaultEyeZ, which its copy in push() has not got)
  return o;
}
const __fill_of = r => r._tex ? r._tint.map(v => v / 255) : r._useNormalMaterial ? [1, 1, 1, 1] : r.curFillColor;
// What it draws with, texture() or image(): a createGraphics is a canvas of paints like the one a 2D sketch paints on, each
// frame's taken as that one's are (__paints) and drawn by ThorVG (drawn_sketch.h sketch_texture). 0: none.
var __tex = [], __tex_of = new Map();
function __tex_at(r) {
  const e = r._tex && r._tex.canvas, flat = e && e.flat, wraps = { clamp: 0, repeat: 1, mirror: 2 };
  if (!flat) return 0;
  let i = __tex_of.get(flat);
  if (i === undefined) __tex_of.set(flat, i = __tex.length), __tex.push({ flat, w: e.width, h: e.height, x: wraps[r.textureWrapX] | 0, y: wraps[r.textureWrapY] | 0, frames: [], from: [] });
  return i + 1;
}
function __tex_done(f) {
  for (const q of __tex) {
    const [wiped, o] = q.flat.fresh();
    q.frames[f] = o, q.from[f] = wiped || q.from[f - 1] === undefined ? f : q.from[f - 1], __mesh_held += o.byteLength;
  }
}
function __stand(r, gId, x, y, z) {
  const g = r.retainedMode.geometry[gId], m = g && g.model;
  if (!__own(r) || !m || !m.vertices.length) return;
  let i = __shape_of.get(m);
  if (i === undefined) __shape_of.set(m, i = __shapes.length), __shapes.push(m);
  const of = String(gId).split('|')[0], f = __fill_of(r), s = r.curStrokeColor, e = __seen(r, x, y, z);
  __kinds[of] = (__kinds[of] || 0) + 1, __lined = __lined || r._doStroke;
  __mesh.draws.push(i, ...e, f[0], f[1], f[2], f[3], r._doFill ? (r._enableLighting ? 0 : 1) : -1, __L, s[0], s[1], s[2], s[3], r._doStroke ? r.curStrokeWeight : 0, __tex_at(r));
}
p5.RendererGL.prototype.drawBuffers = function (gId) { __stand(this, gId, 1, 1, 1); return this; };
p5.RendererGL.prototype._drawImmediateFill = function () {
  if (!__own(this)) return;
  const g = this.immediateMode.geometry, v = g.vertices, n = g.vertexNormals, c = g.vertexColors, at = g.uvs, e = __seen(this, 1, 1, 1), mode = this.immediateMode.shapeMode, out = __mesh.tris, own = __fill_of(this), flat = this._enableLighting ? 0 : 1, painted = !this._tex && c.length >= v.length * 4, tex = __tex_at(this);
  const put = i => {
    const p = v[i], q = n[i] || { x: 0, y: 0, z: 1 }, a = painted ? i * 4 : -1;
    out.push(e[0] * p.x + e[4] * p.y + e[8] * p.z + e[12], e[1] * p.x + e[5] * p.y + e[9] * p.z + e[13], e[2] * p.x + e[6] * p.y + e[10] * p.z + e[14], e[0] * q.x + e[4] * q.y + e[8] * q.z, e[1] * q.x + e[5] * q.y + e[9] * q.z, e[2] * q.x + e[6] * q.y + e[10] * q.z,
      a < 0 ? own[0] : c[a], a < 0 ? own[1] : c[a + 1], a < 0 ? own[2] : c[a + 2], a < 0 ? own[3] : c[a + 3], flat, __L, at[i * 2] || 0, at[i * 2 + 1] || 0, tex);
  };
  if (mode === 5) for (let i = 0; i + 2 < v.length; ++i) { if (i & 1) put(i + 1), put(i), put(i + 2); else put(i), put(i + 1), put(i + 2); }   // (TRIANGLE_STRIP)
  else if (mode === 6) for (let i = 1; i + 1 < v.length; ++i) put(0), put(i), put(i + 1);   // (TRIANGLE_FAN)
  else for (let i = 0; i + 2 < v.length; i += 3) put(i), put(i + 1), put(i + 2);
};
function __line(r, e, a, b, c, at) {
  __mesh.lines.push(e[0] * a.x + e[4] * a.y + e[8] * a.z + e[12], e[1] * a.x + e[5] * a.y + e[9] * a.z + e[13], e[2] * a.x + e[6] * a.y + e[10] * a.z + e[14], e[0] * b.x + e[4] * b.y + e[8] * b.z + e[12], e[1] * b.x + e[5] * b.y + e[9] * b.z + e[13], e[2] * b.x + e[6] * b.y + e[10] * b.z + e[14],
    c[at], c[at + 1], c[at + 2], c[at + 3], r.curStrokeWeight, __L);
}
p5.Geometry.prototype._edgesToVertices = function () {
  const r = p5.instance && p5.instance._renderer;
  if (!r || !r.isP3D || this !== r.immediateMode.geometry || r.geometryBuilder || !__own(r)) return this;
  const e = __seen(r, 1, 1, 1), v = this.vertices, c = this.vertexStrokeColors, own = c.length >= v.length * 4;
  __lined = true;
  for (const [a, b] of this.edges) __line(r, e, v[a], v[b], own ? c : r.curStrokeColor, own ? a * 4 : 0);
  return this;
};
p5.RendererGL.prototype._drawPoints = function (v) {
  if (!__own(this)) return this;
  const e = __seen(this, 1, 1, 1);
  __lined = true;
  for (const p of v) __line(this, e, p, p, this.curStrokeColor, 0);
  return this;
};
const __clear = p5.RendererGL.prototype.clear;
p5.RendererGL.prototype.clear = function (r, g, b, a) {
  if (__own(this)) __mesh = { draws: [], tris: [], lines: [], back: [r || 0, g || 0, b || 0, a || 0], wiped: true };
  return __clear.apply(this, arguments);
};
function __mesh_done() {
  const m = __mesh, one = { draws: Float32Array.from(m.draws), tris: Float32Array.from(m.tris), lines: Float32Array.from(m.lines), back: m.back, wiped: m.wiped };
  __mesh_held += one.draws.byteLength + one.tris.byteLength + one.lines.byteLength;
  __tex_done(__meshes.length), __meshes.push(one), __mesh = { draws: [], tris: [], lines: [], back: null, wiped: false };
  for (; __mesh_held > 400e6 && __mesh_gone < __meshes.length - 1; ++__mesh_gone) {   // (a long heavy sketch: what it drew first goes, so that going back finds it gone rather than the Mac full)
    const g = __meshes[__mesh_gone];
    __mesh_held -= g.draws.byteLength + g.tris.byteLength + g.lines.byteLength;
    g.draws = g.tris = g.lines = new Float32Array(0);
    for (const q of __tex) if (q.frames[__mesh_gone]) __mesh_held -= q.frames[__mesh_gone].byteLength, q.frames[__mesh_gone] = new Float64Array(0);
  }
}
function __mesh_at(f) {   // (all it has drawn since it last cleared, for doc_sketch.h)
  __to(f);
  let from = f, d = 0, t = 0, l = 0;
  while (from > 0 && !__meshes[from].wiped) --from;
  for (let i = from; i <= f; ++i) d += __meshes[i].draws.length, t += __meshes[i].tris.length, l += __meshes[i].lines.length;
  const o = new Float32Array(11 + d + t + l), b = __meshes[from].back || [0, 0, 0, 0];
  o.set([d / 29, t / 15, l / 12, __U, globalThis.width, globalThis.height, b[0], b[1], b[2], b[3], __tex.length]);
  let at = 11;
  for (const k of ['draws', 'tris', 'lines']) for (let i = from; i <= f; ++i) o.set(__meshes[i][k], at), at += __meshes[i][k].length;
  globalThis.__bytes = o;
}
function __mesh_shape(i) {   // (a shape as p5 made it, once)
  const g = __shapes[i], v = g.vertices, n = g.vertexNormals || [], u = g.uvs || [], f = g.faces, e = g.edges || [], o = new Float32Array(3 + v.length * 8 + f.length * 3 + e.length * 2);
  o.set([v.length, f.length, e.length]);
  let at = 3;
  for (const p of v) o[at++] = p.x, o[at++] = p.y, o[at++] = p.z;
  for (let k = 0; k < v.length; ++k) { const p = n[k] || { x: 0, y: 0, z: 1 }; o[at++] = p.x, o[at++] = p.y, o[at++] = p.z; }
  for (let k = 0; k < v.length * 2; ++k) o[at++] = u[k] || 0;
  for (const q of f) o[at++] = q[0], o[at++] = q[1], o[at++] = q[2];
  for (const q of e) o[at++] = q[0], o[at++] = q[1];
  globalThis.__bytes = o;
}
function __mesh_pic(i) { const q = __tex[i]; globalThis.__bytes = Float32Array.of(q.w, q.h, q.x, q.y); }
p5.RendererGL.prototype.drawBuffersScaled = function (gId, x, y, z) {
  __stand(this, gId, x, y, z);
  const m = Array.from(this.uModelMatrix.mat4);
  for (const i of [1, 4, 6, 9, 13]) m[i] = -m[i];   // (p5's y goes down, the world's up)
  const sx = Math.hypot(m[0], m[1], m[2]) || 1, sy = Math.hypot(m[4], m[5], m[6]) || 1, sz = Math.hypot(m[8], m[9], m[10]) || 1, deg = 180 / Math.PI;
  const of = String(gId).split('|')[0], whole = of === 'ellipsoid' ? 2 : 1;   // (an ellipsoid told by its radii; the world's ball is as wide as 1, as p5's box)
  __drawn.push(m[12] * __U, m[13] * __U, m[14] * __U, whole * x * sx * __U, whole * y * sy * __U, whole * z * sz * __U, Math.atan2(m[1] / sx, m[0] / sx) * deg, Math.asin(Math.max(-1, Math.min(1, -m[2] / sx))) * deg, Math.atan2(m[6] / sy, m[10] / sz) * deg, __L, this._useNormalMaterial || this._tex ? -1 : __packed(this.curFillColor));
  return this;
};
// shader(): nothing runs GLSL here, so what it is told is kept (door_p5_shader.h makes it an effect). A flat thing drawn with
// one is that shader painting the stage, and draws nothing of its own; on a solid it is the solid as ever.
var __shaded = null, __shaded_kind = '';
p5.prototype.shader = function (s) { this._renderer.userFillShader = s, this._renderer._useNormalMaterial = false; return this; };
p5.Shader.prototype.setUniform = function (n, v) {
  (this.__told = this.__told || {})[n] = typeof v === 'number' || typeof v === 'boolean' ? +v : Array.isArray(v) || ArrayBuffer.isView(v) ? Array.from(v) : null;
  return this;
};
for (const way of ['drawBuffers', 'drawBuffersScaled', '_drawImmediateFill']) {
  const was = p5.RendererGL.prototype[way];
  p5.RendererGL.prototype[way] = function (gId) {
    const of = way[0] === '_' ? 'quad' : String(gId).split('|')[0];
    if (!this.userFillShader || !this._doFill || !['rect', 'plane', 'quad'].includes(of)) return was.apply(this, arguments);
    __shaded = this.userFillShader, __shaded_kind = of;
    return this;
  };
}
)";

static const char* const p5_start = R"(
new p5();
if (!globalThis.frameCount && (p5.instance._preloadCount || globalThis._preloadCount)) p5.instance._preloadCount = globalThis._preloadCount = 0, p5.instance._runIfPreloadsAreDone();   // (what preload() asked of the net or of files does not come: on without it)
__U = 3.2154 / (globalThis.height || 400);
var __frames = [], __from = [];
Object.defineProperty(globalThis, '__2d', { get: () => !p5.instance._renderer.isP3D });   // (asked late: an async setup() makes its canvas after new p5())
Object.defineProperty(globalThis, '__canvas', { get: () => p5.instance._renderer.drawingContext });
function __to(f) {
  while (__frames.length <= f) {
    const was = globalThis.frameCount, q = __Q, last = __frames[__frames.length - 1];
    if (q.length) __drawn = [];
    if (__film) __film_step();
    __T += 1000 / 60, __Q = [];
    q.forEach(g => g(__T));
    if (__2d) {   // (2D: what each frame adds to the one before, as its canvas keeps what was painted; __from: the frame it was last wiped at)
      const [wiped, o] = __canvas.take();
      __from.push(wiped || !__frames.length ? __frames.length : __from[__from.length - 1]), __frames.push(o);
    } else __frames.push(last && globalThis.frameCount === was ? last : Float64Array.from(__drawn)), __mesh_done();   // (a noLoop(), a frameRate(30): as it was)
  }
}
function __shaded_take(seconds) {   // (the shader that paints its stage and what the sketch tells it every 0.1 second, a few seconds of waiting at most: "" none)
  const told = {}, from = Date.now();
  for (let k = 0; k <= seconds * 10 && Date.now() - from < 5000; ++k) {
    __to(k * 6);
    if (!__shaded && k < 2) continue;
    if (!__shaded) return '';
    const now = __shaded.__told || {};
    for (const n in now) {
      const row = told[n] = told[n] || [];
      while (row.length < k) row.push(now[n]);   // (told late: as if from the first)
      row.push(now[n]);
    }
  }
  return JSON.stringify({ vert: __shaded._vertSrc, frag: __shaded._fragSrc, kind: __shaded_kind, width: globalThis.width, height: globalThis.height, density: pixelDensity(), told });
}
function __paints(f, had, pic) {   // (what makes frame f on a canvas holding frame `had`: all since it was last wiped, or if `had` is on that way only what came after it; first a 1 if it is to be wiped)
  __to(f);
  const q = __tex[pic], frames = q ? q.frames : __frames, from = q ? q.from : __from;
  if (from[f] === undefined) return Float64Array.of(1);   // (a picture it had not drawn with yet)
  const go = had >= from[f] && had <= f, first = go ? had + 1 : from[f];
  let n = 1;
  for (let i = first; i <= f; ++i) n += frames[i].length;
  const all = new Float64Array(n);
  all[0] = go ? 0 : 1;
  for (let i = first, at = 1; i <= f; at += frames[i++].length) all.set(frames[i], at);
  return all;
}
function __most() {   // (as many copies as its first seconds draw at once, a second of waiting at most)
  const from = Date.now();
  for (let f = 1; f < 300 && Date.now() - from < 1000; ++f) __to(f);
  return Math.max(...__frames.map(c => c.length / 11));
}
function __shape() {   // (only boxes, only balls or only rings, with no lines: the world's own, as real as any. Any more: as it draws them, doc_sketch.h)
  __most();
  if (__parts.length || __three) return '';
  const kit = { box: 'cube', ellipsoid: 'ball', torus: 'ring' }, kinds = Object.keys(__kinds);
  return kinds.length === 1 && kit[kinds[0]] && !__lined && !__tex.length && !__meshes.some(m => m.tris.length) ? kit[kinds[0]] : 'sketch';
}
var __by_at = -1, __by = [];
function __three_each(i, f, part) {   // (the frame's meshes by their parts; each part's copies a solid's: door.h)
  if (__by_at !== f) {
    __three_whole(), __by_at = f, __by = [];
    const c = __frames[f];
    for (let at = 0; at < c.length; at += 11) { const w = __whole.of[c[at + 9]]; if (w !== undefined) (__by[w] = __by[w] || []).push(at); }
  }
  const at = (__by[part] || [])[i], c = __frames[f];
  return at === undefined ? { size: 0 } : { x: c[at], y: c[at + 1], z: c[at + 2], scale_x: c[at + 3], scale_y: c[at + 4], scale_z: c[at + 5], rotate: c[at + 6], rotate_y: c[at + 7], rotate_x: c[at + 8] };
}
function __three_parts() { return __three_whole().first.length; }
function each(i, n, t, k) {
  const f = Math.max(0, Math.round(t * 60));
  __to(f);
  if (__parts.length || __three) return __three_each(i, f, k.part | 0);
  const c = __frames[f], at = i * 11, strong = (k && k['line_' + c[at + 9]]) ?? 1;   // (its line's knob: what that line drew grows out of nothing)
  return at < c.length ? { x: c[at], y: c[at + 1], z: c[at + 2], scale_x: c[at + 3] * strong, scale_y: c[at + 4] * strong, scale_z: c[at + 5] * strong, rotate: c[at + 6], rotate_y: c[at + 7], rotate_x: c[at + 8], colour: c[at + 10] } : { size: 0 };
}
)";
