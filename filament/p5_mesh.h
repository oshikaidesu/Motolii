// What a p5.js sketch draws in WEBGL, taken for doc_sketch.h: its mesh, its lights, its shaders, its pictures and words (p5_stand.h, run after p5_after).
#pragma once

static const char* const p5_mesh = R"(
// What WEBGL draws, as p5 has it: which of its shapes where its camera sees them, the triangles and lines it makes as it goes,
// what it clears with. doc_sketch.h stands them in the world, one mesh each frame. p5 makes a line's corners in triangles
// for its line shader here too, 0.8 s a frame in the heaviest; nothing here reads them.
const __mesh_new = (back, wiped) => ({ draws: [], tris: [], lines: [], back, wiped, rows: new Map(), sets: new Map(), groups: new Map() });
var __mesh = __mesh_new(null, false), __meshes = [], __mesh_held = 0, __mesh_gone = 0, __shapes = [], __shape_of = new Map(), __lined = false;
const __own = r => p5.instance && r === p5.instance._renderer;
function __view(r) {   // (how what its camera has lands on the stage. What is at z 0 stays as it is; what is not is carried from its eye to the world's by the one homography that keeps that plane and takes the one eye to the other, so that from the world's first camera it looks as its own camera has it, ortho or any perspective(), and is still a thing in depth from anywhere else. ortho: as wide as its own box says, and it stands as a mesh, the world's solids knowing none of that)
  const c = r._curCamera, P = c.projMatrix.mat4, far = 6 / __U;
  if (P[15] !== 1) { const eye = globalThis.height * Math.abs(P[5]) / 2; return { x: 1, y: 1, z: 1, back: eye, pull: 1 / far - 1 / eye }; }
  const x = globalThis.width * P[0] / 2, y = globalThis.height * Math.abs(P[5]) / 2;
  __lined = true;
  return { x, y, z: (x + y) / 2, back: Math.hypot(c.eyeX - c.centerX, c.eyeY - c.centerY, c.eyeZ - c.centerZ), pull: 1 / far };
}
function __seen(r, x, y, z) {   // (what its camera sees, what it draws at z 0 still at z 0: a camera of its own moves things from there)
  const e = r._curCamera.cameraMatrix.mat4, m = r.uModelMatrix.mat4, o = new Array(16), v = __view(r);
  for (let c = 0; c < 4; ++c) for (let i = 0; i < 4; ++i) o[c * 4 + i] = e[i] * m[c * 4] + e[4 + i] * m[c * 4 + 1] + e[8 + i] * m[c * 4 + 2] + e[12 + i] * m[c * 4 + 3];
  for (let i = 0; i < 4; ++i) o[i] *= x, o[4 + i] *= y, o[8 + i] *= z;
  for (let c = 0; c < 4; ++c) o[c * 4] *= v.x, o[c * 4 + 1] *= v.y, o[c * 4 + 2] = (o[c * 4 + 2] + v.back * o[c * 4 + 3]) * v.z, o[c * 4 + 3] += v.pull * o[c * 4 + 2];
  return o;
}
const __w = (e, x, y, z) => Math.max(e[3] * x + e[7] * y + e[11] * z + e[15], 0.1);   // (doc_sketch.h too: what is past the world's eye's far end stays this side of it)
const __fill_of = r => r._tex ? r._tint.map(v => v / 255) : r._useNormalMaterial ? [1, 1, 1, 1] : r.curFillColor;
// What _setFillUniforms tells p5's lit shader, as it stands where this is drawn, in sketch.mat's table: a row of four for each way
// of being lit (ambient material, specular material and shininess, emissive and metalness, how many lights and where), and
// its lights after them, where the sketch's camera has them. 1: it takes no light. 2 and on: its row, in this frame's table.
function __lit(r) {
  if (!r._enableLighting || r._useNormalMaterial) return 1;
  const V = r._curCamera.cameraMatrix.mat4, m = __mesh, metal = r._useMetalness, v = __view(r), at = (a, i, w) => [V[0] * a[i] + V[4] * a[i + 1] + V[8] * a[i + 2] + w * V[12], V[1] * a[i] + V[5] * a[i + 1] + V[9] * a[i + 2] + w * V[13], V[2] * a[i] + V[6] * a[i + 1] + V[10] * a[i + 2] + w * V[14]];
  const way = (a, i) => { const p = at(a, i, 0); return [p[0], -p[1], p[2]]; }, place = (a, i) => { const p = at(a, i, 1), z = (p[2] + v.back) * v.z, w = Math.max(1 + v.pull * z, 0.1); return [p[0] * v.x * __U / w, -p[1] * v.y * __U / w, z * __U / w]; }, three = (a, i) => [a[i], a[i + 1], a[i + 2]];
  const kept = [r.ambientLightColors, r.directionalLightDirections, r.directionalLightDiffuseColors, r.directionalLightSpecularColors, r.pointLightPositions, r.pointLightDiffuseColors, r.pointLightSpecularColors, r.spotLightPositions, r.spotLightDirections, r.spotLightDiffuseColors, r.spotLightSpecularColors, r.spotLightAngle, r.spotLightConc, r.constantAttenuation, r.linearAttenuation, r.quadraticAttenuation, metal, V].join();
  let set = m.sets.get(kept);
  if (set === undefined) {
    const o = [0, 0, 0, __U, r.constantAttenuation, r.linearAttenuation, r.quadraticAttenuation, 0];
    for (let i = 0; i + 2 < r.ambientLightColors.length && i < 15; i += 3) for (let k = 0; k < 3; ++k) o[k] += Math.max(0, r.ambientLightColors[i + k] - metal);
    const dirs = Math.min(5, r.directionalLightDiffuseColors.length / 3), points = Math.min(5, r.pointLightDiffuseColors.length / 3), spots = Math.min(5, r.spotLightDiffuseColors.length / 3);
    for (let j = 0; j < dirs; ++j) o.push(...way(r.directionalLightDirections, j * 3), 0, ...three(r.directionalLightDiffuseColors, j * 3), 0, ...three(r.directionalLightSpecularColors, j * 3), 0);
    for (let j = 0; j < points; ++j) o.push(...place(r.pointLightPositions, j * 3), 0, ...three(r.pointLightDiffuseColors, j * 3), 0, ...three(r.pointLightSpecularColors, j * 3), 0);
    for (let j = 0; j < spots; ++j) o.push(...place(r.spotLightPositions, j * 3), r.spotLightAngle[j], ...way(r.spotLightDirections, j * 3), r.spotLightConc[j], ...three(r.spotLightDiffuseColors, j * 3), 0, ...three(r.spotLightSpecularColors, j * 3), 0);
    m.sets.set(kept, set = { o, dirs, points, spots, i: m.sets.size });
  }
  const own = r.curSpecularColor, shine = metal > 0 ? own.map((v, i) => r.curFillColor[i] * metal + v * (1 - metal)) : own, row = [...three(r.curAmbientColor, 0), (r._hasSetAmbient ? 1 : 0) + (r._useSpecularMaterial ? 2 : 0), ...three(shine, 0), r._useShininess, ...three(r.curEmissiveColor, 0), metal, set.dirs, set.points, set.spots, set.i];
  const key = row.join();
  let i = m.rows.get(key);
  if (i === undefined) m.rows.set(key, i = m.rows.size);
  return 2 + i;
}
// shader() on what it draws: its own GLSL runs, as a Filament material made from it (doc_sketch.h, by matc). What is drawn is
// already where its camera sees it, so its matrices are nothing and aPosition is that place; what its vertex shader leaves there
// stands in the world, moved or not. What it tells a shader goes with each draw: those told the same are drawn as one.
var __shaders = [], __shader_of = new Map();
const __glsl = { float: ['float', 1], int: ['int', 1], bool: ['bool', 1], vec2: ['float2', 2], vec3: ['float3', 3], vec4: ['float4', 4], mat3: ['mat3', 9], mat4: ['mat4', 16], sampler2D: ['sampler2d', 0] };
const __given = { uModelViewMatrix: 'mat4(1.0)', uProjectionMatrix: 'mat4(1.0)', uModelViewProjectionMatrix: 'mat4(1.0)', uViewMatrix: 'mat4(1.0)', uModelMatrix: 'mat4(1.0)', uCameraMatrix: 'mat4(1.0)', uNormalMatrix: 'mat3(1.0)' };
function __shader_text(s) {   // (the two shaders in a Filament material's own blocks, what each stage reads put there as a global before it runs. The names a sketch makes for itself get s_, for Filament's own are beside them)
  const clean = t => String(t).replace(/#version[^\n]*|#extension[^\n]*|#include[^\n]*|\bprecision\s+\w+\s+\w+\s*;/g, '').replace(/\btexture(?:2D|Cube)\b/g, 'texture').replace(/\bgl_FragData\[0\]/g, 'gl_FragColor');
  const q = '(?:(?:flat|smooth|centroid|highp|mediump|lowp)\\s+)*', mine = new Set(['aPosition', 'aNormal', 'aTexCoord', 'aVertexColor']), told = new Map(), vary = new Map(), macros = new Set();
  const take = t => t.replace(new RegExp('\\buniform\\s+' + q + '(\\w+)\\s+([^;]+);', 'g'), (_, type, list) => {
    for (const m of list.matchAll(/(\w+)\s*(?:\[\s*(\d+)\s*\])?/g)) if (__glsl[type] && !__given[m[1]] && !told.has(m[1])) told.set(m[1], { name: m[1], type, count: m[2] ? +m[2] : 1, size: __glsl[type][1] });
    return '';
  });
  const own = t => {
    const n = new Set(['main']);
    for (const m of t.matchAll(/^\s*(?:(?:const|highp|mediump|lowp)\s+)*(?:float|int|bool|void|[iu]?vec[234]|mat[234])\s+(\w+)\s*(?:\(|=|\[)/gm)) n.add(m[1]);
    for (const m of t.matchAll(/\bstruct\s+(\w+)/g)) n.add(m[1]);
    for (const m of t.matchAll(/#define\s+(\w+)/g)) macros.add(m[1]);
    n.delete('main');
    return n.size ? t.replace(new RegExp('\\b(' + [...n].join('|') + ')\\b', 'g'), 's_$1') : t;
  };
  let vert = take(clean(s._vertSrc)), frag = take(clean(s._fragSrc));
  frag = frag.replace(new RegExp('(?<=^|[;\\n}])\\s*' + q + '(?:varying|in)\\s+' + q + '(\\w+)\\s+(\\w+)\\s*;', 'g'), (_, type, name) => (vary.set(name, type), ''));
  vert = vert.replace(new RegExp('(?<=^|[;\\n}])\\s*' + q + '(?:varying|attribute|in|out)\\s+' + q + '(\\w+)\\s+(\\w+)\\s*;', 'g'), (_, type, name) => mine.has(name) ? '' : type + ' ' + name + ';');
  let out = 'gl_FragColor';
  frag = frag.replace(/(?<=^|[;\n}])\s*(?:layout\s*\([^)]*\)\s*)?out\s+(?:(?:highp|mediump|lowp)\s+)?vec4\s+(\w+)\s*;/g, (_, name) => (out = name, ''));
  frag = own(frag).replace(new RegExp('\\b' + out + '\\b|\\bgl_FragColor\\b', 'g'), 's_out').replace(/\bmain\b/, 's_stage');
  vert = own(vert).replace(/\bgl_Position\b/g, 's_position').replace(/\bgl_PointSize\b/g, 's_size').replace(/\bmain\b/, 's_stage');
  const names = [...told.values()], swizzle = { float: 'x', vec2: 'xy', vec3: 'xyz', vec4: 'xyzw' };
  const on = [...names.map(u => u.type === 'sampler2D' ? ['materialParams_' + u.name, u.name] : ['materialParams.' + u.name, u.name]), ...Object.entries(__given).map(([k, v]) => [v, k])];
  const says = on.map(([what, name]) => '#define ' + name + ' ' + what + '\n').join(''), done = [...on.map(u => u[1]), ...macros, ...vary.keys()].map(n => '#undef ' + n + '\n').join('');
  const vec = (n, type) => type === 'float' ? 'vec4(' + n + ', 0.0, 0.0, 0.0)' : type === 'vec2' ? 'vec4(' + n + ', 0.0, 0.0)' : type === 'vec3' ? 'vec4(' + n + ', 0.0)' : n;
  const params = [{ name: 's_scale', type: 'vec4', count: 1 }, ...names].map(u => '        { type : ' + __glsl[u.type][0] + (u.count > 1 ? '[' + u.count + ']' : '') + ', name : ' + u.name + ' }').join(',\n');
  const undo = [...macros, 'PI', 'HALF_PI', 'saturate'].map(n => '#undef ' + n + '\n').join('');
  return { names, text: `material {
    name : Painted,
    shadingModel : unlit,
    requires : [uv0, uv1, color, tangents],
    blending : transparent,
    culling : none,
    flipUV : false,
    depthWrite : true,
    parameters : [
${params}
    ],
    variables : [ ${[...vary.keys()].join(', ')} ]
}

vertex {
${undo}${says}${[...vary].filter(([n]) => !vert.includes(' ' + n + ';')).map(([n, t]) => t + ' ' + n + ';\n').join('')}
vec4 s_position = vec4(0.0); float s_size; vec3 aPosition; vec3 aNormal; vec2 aTexCoord; vec4 aVertexColor;
${vert}
    void materialVertex(inout MaterialVertexInputs material) {
        vec3 turned;
        toTangentFrame(mesh_tangents, turned);
        vec4 k = materialParams.s_scale;
        aPosition = getPosition().xyz / k.xyz - vec3(0.0, 0.0, k.w), aNormal = turned * vec3(1.0, -1.0, 1.0), aTexCoord = mesh_uv1, aVertexColor = mesh_color;
        s_stage();
        material.worldPosition = mulMat4x4Float3(getWorldFromModelMatrix(), (s_position.xyz / s_position.w + vec3(0.0, 0.0, k.w)) * k.xyz);
${[...vary].map(([n, t]) => '        material.' + n + ' = ' + vec(n, t) + ';\n').join('')}    }
${done}}

fragment {
${undo}${says}${[...vary].map(([n, t]) => '#define ' + n + ' variable_' + n + '.' + (swizzle[t] || 'xyzw') + '\n').join('')}
vec4 s_out = vec4(1.0);
${frag}
    void material(inout MaterialInputs material) {
        prepareMaterial(material);
        s_stage();
        vec3 c = clamp(s_out.rgb, 0.0, 1.0);
        material.baseColor = vec4(mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c)), s_out.a);
    }
${done}}
` };
}
function __paint(r) {   // (what sketch.mat reads, or 1000 and on: which of this frame's told shaders)
  const s = r.userFillShader;
  if (!s) return __lit(r);
  let i = __shader_of.get(s);
  if (i === undefined) __shader_of.set(s, i = __shaders.length), __shaders.push(__shader_text(s));
  const told = s.__told || {}, v = __view(r), row = [i, __U * v.x, -__U * v.y, __U * v.z, v.back];
  for (const u of __shaders[i].names) {
    const had = u.name in told ? told[u.name] : u.name === 'uMaterialColor' ? r.curFillColor : u.name === 'uTint' ? r._tint : 0, a = Array.isArray(had) ? had : [had];
    for (let k = 0; k < u.size * u.count; ++k) row.push(+a[k] || 0);
  }
  const key = row.join();
  let g = __mesh.groups.get(key);
  if (g === undefined) __mesh.groups.set(key, g = __mesh.groups.size);
  return 1000 + g;
}
// What it draws with, texture() or image(): a createGraphics is a canvas of paints like the one a 2D sketch paints on, each
// frame's taken as that one's are (__paints) and drawn by ThorVG (drawn_sketch.h sketch_texture). 0: none.
var __tex = [], __tex_of = new Map();
function __tex_at(r) {
  const e = r._tex && r._tex.canvas, flat = e && e.flat;
  return !flat || e.bytes || (r._tex instanceof p5.Image && !flat.held().length) ? 0 : __tex_on(e, r.textureWrapX, r.textureWrapY);   // (a picture that is not there: white, not nothing)
}
function __tex_on(e, x, y) {
  const wraps = { clamp: 0, repeat: 1, mirror: 2 }, flat = e.flat;
  let i = __tex_of.get(flat);
  if (i === undefined) __tex_of.set(flat, i = __tex.length), __tex.push({ flat, w: e.width, h: e.height, x: wraps[x] | 0, y: wraps[y] | 0, frames: [], from: [] });
  return i + 1;
}
// text() in WEBGL: p5 asks for a font file and its own shader. Each line is painted once, white, on one sheet of words (ThorVG's,
// as a picture is), and stands on a rectangle where p5 would set it, its colour the fill's. The rectangle is wider than the
// words and they are set from the side they are aligned to, so nothing here needs to know how wide they are.
var __sheet = null, __sheet_of = new Map(), __sheet_x = 0, __sheet_y = 0, __sheet_row = 0;
const __SHEET = 4096;
function __sheet_word(r, line) {
  const size = r._textSize, how = r._textAlign === 'center' ? 'center' : r._textAlign === 'right' ? 'right' : 'left', key = line + '|' + size + '|' + how;
  let q = __sheet_of.get(key);
  if (q) return q;
  if (!__sheet) (__sheet = document.createElement('canvas')).width = __SHEET, __sheet.height = __SHEET, __sheet.getContext('2d');
  const c = __sheet.flat, fine = Math.min(2, 200 / size), px = size * fine, W = Math.min(__SHEET, Math.ceil(String(line).length * px * 0.75 + px)), H = Math.ceil(px * 1.5);
  if (__sheet_x + W > __SHEET) __sheet_x = 0, __sheet_y += __sheet_row, __sheet_row = 0;
  if (__sheet_y + H > __SHEET) c.clearRect(0, 0, __SHEET, __SHEET), __sheet_of.clear(), __sheet_x = __sheet_y = __sheet_row = 0;   // (words that change each frame: a sheet again, each frame's kept as a picture's are)
  c.font = px + 'px sans-serif', c.fillStyle = '#ffffff', c.textAlign = how, c.textBaseline = 'alphabetic';
  c.fillText(String(line), __sheet_x + (how === 'center' ? W / 2 : how === 'right' ? W : 0), __sheet_y + px * 1.1);
  q = { x: __sheet_x, y: __sheet_y, W, H, fine, base: px * 1.1 };
  __sheet_x += W, __sheet_row = Math.max(__sheet_row, H), __sheet_of.set(key, q);
  return q;
}
p5.RendererGL.prototype._renderText = function (p, line, x, y, maxY, minY) {
  if (!__own(this) || !this._doFill || !line || y > maxY || y < minY) return p;
  const q = __sheet_word(this, line), w = q.W / q.fine, h = q.H / q.fine, left = x - (this._textAlign === 'center' ? w / 2 : this._textAlign === 'right' ? w : 0), down = { top: 0.8, center: 0.35, bottom: -0.2 }[this._textBaseline] || 0, top = y + down * this._textSize - q.base / q.fine;
  const e = __seen(this, 1, 1, 1), f = this.curFillColor, tex = __tex_on(__sheet, 'clamp', 'clamp'), S = 1 / __SHEET;
  const put = (a, b) => { const X = left + a * w, Y = top + b * h, W = __w(e, X, Y, 0); __mesh.tris.push((e[0] * X + e[4] * Y + e[12]) / W, (e[1] * X + e[5] * Y + e[13]) / W, (e[2] * X + e[6] * Y + e[14]) / W, 0, 0, 1, f[0], f[1], f[2], f[3], 1, __L, (q.x + a * q.W) * S, (q.y + b * q.H) * S, tex); };
  put(0, 0), put(1, 0), put(1, 1), put(0, 0), put(1, 1), put(0, 1);
  return p;
};
p5.RendererGL.prototype.textWidth = function (s) { return String(s).length * this._textSize * 0.5; };   // (as measureText has it above)
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
  __mesh.draws.push(i, ...e, f[0], f[1], f[2], f[3], r._doFill ? __paint(r) : -1, __L, s[0], s[1], s[2], s[3], r._doStroke ? r.curStrokeWeight : 0, __tex_at(r));
}
p5.RendererGL.prototype.drawBuffers = function (gId) { __stand(this, gId, 1, 1, 1); return this; };
p5.RendererGL.prototype._drawImmediateFill = function () {
  if (!__own(this)) return;
  const g = this.immediateMode.geometry, v = g.vertices, n = g.vertexNormals, c = g.vertexColors, at = g.uvs, e = __seen(this, 1, 1, 1), mode = this.immediateMode.shapeMode, out = __mesh.tris, own = __fill_of(this), flat = __paint(this), painted = !this._tex && c.length >= v.length * 4, tex = __tex_at(this);
  const put = i => {
    const p = v[i], q = n[i] || { x: 0, y: 0, z: 1 }, a = painted ? i * 4 : -1, w = __w(e, p.x, p.y, p.z);
    out.push((e[0] * p.x + e[4] * p.y + e[8] * p.z + e[12]) / w, (e[1] * p.x + e[5] * p.y + e[9] * p.z + e[13]) / w, (e[2] * p.x + e[6] * p.y + e[10] * p.z + e[14]) / w, e[0] * q.x + e[4] * q.y + e[8] * q.z, e[1] * q.x + e[5] * q.y + e[9] * q.z, e[2] * q.x + e[6] * q.y + e[10] * q.z,
      a < 0 ? own[0] : c[a], a < 0 ? own[1] : c[a + 1], a < 0 ? own[2] : c[a + 2], a < 0 ? own[3] : c[a + 3], flat, __L, at[i * 2] || 0, at[i * 2 + 1] || 0, tex);
  };
  if (mode === 5) for (let i = 0; i + 2 < v.length; ++i) { if (i & 1) put(i + 1), put(i), put(i + 2); else put(i), put(i + 1), put(i + 2); }   // (TRIANGLE_STRIP)
  else if (mode === 6) for (let i = 1; i + 1 < v.length; ++i) put(0), put(i), put(i + 1);   // (TRIANGLE_FAN)
  else for (let i = 0; i + 2 < v.length; i += 3) put(i), put(i + 1), put(i + 2);
};
function __line(r, e, a, b, c, at) {
  const w = __w(e, a.x, a.y, a.z), W = __w(e, b.x, b.y, b.z);
  __mesh.lines.push((e[0] * a.x + e[4] * a.y + e[8] * a.z + e[12]) / w, (e[1] * a.x + e[5] * a.y + e[9] * a.z + e[13]) / w, (e[2] * a.x + e[6] * a.y + e[10] * a.z + e[14]) / w, (e[0] * b.x + e[4] * b.y + e[8] * b.z + e[12]) / W, (e[1] * b.x + e[5] * b.y + e[9] * b.z + e[13]) / W, (e[2] * b.x + e[6] * b.y + e[10] * b.z + e[14]) / W,
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
  if (__own(this)) __mesh = __mesh_new([r || 0, g || 0, b || 0, a || 0], true);
  return __clear.apply(this, arguments);
};
function __mesh_done() {
  const m = __mesh, one = { draws: Float32Array.from(m.draws), tris: Float32Array.from(m.tris), lines: Float32Array.from(m.lines), back: m.back, wiped: m.wiped, rows: [...m.rows.keys()].map(k => k.split(',').map(Number)), sets: [...m.sets.values()], groups: [...m.groups.keys()].map(k => k.split(',').map(Number)) };
  __mesh_held += one.draws.byteLength + one.tris.byteLength + one.lines.byteLength;
  __draws_most = Math.max(__draws_most, __draws_now), __draws_now = 0;
  __tex_done(__meshes.length), __meshes.push(one), __mesh = __mesh_new(null, false);
  for (; __mesh_held > 400e6 && __mesh_gone < __meshes.length - 1; ++__mesh_gone) {   // (a long heavy sketch: what it drew first goes, so that going back finds it gone rather than the Mac full)
    const g = __meshes[__mesh_gone];
    __mesh_held -= g.draws.byteLength + g.tris.byteLength + g.lines.byteLength;
    g.draws = g.tris = g.lines = new Float32Array(0), g.rows = g.sets = g.groups = [];
    for (const q of __tex) if (q.frames[__mesh_gone]) __mesh_held -= q.frames[__mesh_gone].byteLength, q.frames[__mesh_gone] = new Float64Array(0);
  }
}
function __mesh_at(f) {   // (all it has drawn since it last cleared, for doc_sketch.h)
  __to(f);
  let from = f, d = 0, t = 0, l = 0;
  while (from > 0 && !__meshes[from].wiped) --from;
  for (let i = from; i <= f; ++i) d += __meshes[i].draws.length, t += __meshes[i].tris.length, l += __meshes[i].lines.length;
  // (sketch.mat's table: each frame kept its own, so those of a sketch that never clears go after one another; past 80 rows what is drawn is flat, past 720 places a row reads the last lights that fit)
  const rows = [], sets = [];
  let count = 0;
  for (let i = from; i <= f; ++i) count += __meshes[i].rows.length;
  const shown = Math.min(count, 80);
  for (let i = from, row = 0, place = shown * 4, last = place; i <= f; ++i) {
    const m = __meshes[i], where = [];
    m.base = row;
    if (row < shown) for (const q of m.sets) {
      const n = q.o.length / 4;
      if (place + n <= 720) sets.push(...q.o), last = place, place += n;
      where.push(last);
    }
    for (const q of m.rows) { if (row < shown) rows.push(...q.slice(0, 15), where[q[15]]); ++row; }
  }
  const groups = [];
  let told = 0;
  for (let i = from; i <= f; ++i) { __meshes[i].told = told, told += __meshes[i].groups.length; for (const q of __meshes[i].groups) groups.push(...q); }
  const o = new Float32Array(15 + d + t + l + rows.length + sets.length + groups.length), b = __meshes[from].back || [0, 0, 0, 0];
  o.set([d / 29, t / 15, l / 12, __U, globalThis.width, globalThis.height, b[0], b[1], b[2], b[3], __tex.length, (rows.length + sets.length) / 4, told, groups.length, __shaders.length]);
  let at = 15;
  for (const k of ['draws', 'tris', 'lines']) for (let i = from; i <= f; ++i) {
    const q = __meshes[i][k], step = k === 'draws' ? 29 : 15, of = k === 'draws' ? 21 : 10, base = __meshes[i].base, more = __meshes[i].told;
    o.set(q, at);
    if (k !== 'lines' && (base || more)) for (let v = at + of; v < at + q.length; v += step) if (o[v] >= 1000) o[v] += more; else if (o[v] >= 2) o[v] = o[v] + base < 2 + shown ? o[v] + base : 1;
    at += q.length;
  }
  o.set(rows, at), o.set(sets, at + rows.length), o.set(groups, at + rows.length + sets.length);
  globalThis.__bytes = o;
}
function __mesh_shader(i) { return __shaders[i].text; }
function __mesh_shader_names(i) { return __shaders[i].names.map(u => [u.name, u.type, u.count, u.size].join(' ')).join('\n'); }
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
// shader(): what it is told is kept. A flat thing drawn with one, and nothing else in any frame, is that shader painting the
// stage (door_p5_shader.h makes it an effect, its numbers knobs). Any more, and it is what paints what it draws: __paint.
var __shaded = null, __shaded_kind = '', __draws_most = 0, __draws_now = 0;
p5.prototype.shader = function (s) { this._renderer.userFillShader = s, this._renderer._useNormalMaterial = false; return this; };
p5.Shader.prototype.setUniform = function (n, v) {
  (this.__told = this.__told || {})[n] = typeof v === 'number' || typeof v === 'boolean' ? +v : Array.isArray(v) || ArrayBuffer.isView(v) ? Array.from(v) : null;
  return this;
};
for (const way of ['drawBuffers', 'drawBuffersScaled', '_drawImmediateFill']) {
  const was = p5.RendererGL.prototype[way];
  p5.RendererGL.prototype[way] = function (gId) {
    const of = way[0] === '_' ? 'quad' : String(gId).split('|')[0];
    if (this.userFillShader && this._doFill && ['rect', 'plane', 'quad'].includes(of)) __shaded = this.userFillShader, __shaded_kind = of;
    ++__draws_now;
    return was.apply(this, arguments);
  };
}
)";
