// hydra_isf.cpp: code for Hydra (hydra-synth, Olivia Jack and others, AGPL-3.0) as an ISF effect. Hydra is not in Motolii:
// extensions/hydra/get.sh puts npm's package in the maker's own folder, and it is run here in a QuickJS of its own, told
// the code and giving back text. Its own generator-factory.js writes the GLSL, with no WebGL and no page. Every number of a
// chain is passed as a function, so Hydra makes it a uniform and it is a knob here; what is moving in the code (a function
// of time, an array) is read every 0.1 second by Hydra's own code and is that knob's keys, to be edited as any keys are.
#include "hydra_isf.h"
#include <quickjs.h>
#include <filesystem>
#include <fstream>
#include <sstream>

static std::string hydra_text(const std::string& file) {
    std::ifstream in(file);
    std::stringstream all;
    all << in.rdbuf();
    return all.str();
}
static char* hydra_name(JSContext* cx, const char* base, const char* name, void*) {
    return js_strdup(cx, (std::filesystem::path(base).parent_path() / name).lexically_normal().string().c_str());
}
static JSModuleDef* hydra_load(JSContext* cx, const char* name, void*) {
    const std::string src = hydra_text(name);
    JSValue made = JS_Eval(cx, src.c_str(), src.size(), name, JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY);
    if (JS_IsException(made)) return nullptr;
    auto* m = (JSModuleDef*)JS_VALUE_GET_PTR(made);
    JS_FreeValue(cx, made);
    return m;
}

static const char* const hydra_make = R"(
import Generator from './generator-factory.js'
import arrayUtils from './lib/array-utils.js'
import utility from './glsl/utility-functions.js'
try {
  const outs = {}, kept = [], names = ['o0', 'o1', 'o2', 'o3'];
  const output = n => ({ name: n, precision: 'highp', getTexture() { return this; }, render(p) { outs[n] = p[0]; } });
  const source = n => { const s = { name: n, getTexture() { return this; } }; for (const k of ['initCam', 'initImage', 'initVideo', 'initScreen', 'initStream', 'init', 'clear']) s[k] = () => s; return s; };
  const g = globalThis, o = names.map(output);
  arrayUtils.init();
  const synth = new Generator({ defaultUniforms: { time: 0, resolution: [1920, 1080] }, defaultOutput: o[0], changeListener() {} });
  const told = v => typeof v === 'number' ? () => v : v;   // (a number as a function: Hydra makes it a uniform)
  for (const [k, f] of Object.entries(synth.generators)) g[k] = (...a) => f(...a.map(told));
  const P = synth.sourceClass.prototype;
  for (const k of Object.keys(synth.glslTransforms)) if (P[k]) { const was = P[k]; P[k] = function (...a) { return was.apply(this, a.map(told)); }; }
  o.forEach(x => g[x.name] = x);
  ['s0', 's1', 's2', 's3'].forEach(n => g[n] = source(n));
  let shown = o[0];
  Object.assign(g, { time: 0, bpm: Object.assign(() => {}, { valueOf: () => 30 }), speed: 1, width: 1920, height: 1080, mouse: { x: 0, y: 0 }, a: { fft: [0, 0, 0, 0], show() {}, hide() {}, setBins() {}, setSmooth() {}, setCutoff() {}, setScale() {} }, render: x => { shown = x || o[0]; }, hush() {}, update() {}, setResolution() {}, setFunction: f => synth.setFunction(f), setBins() {}, console: { log() {}, warn() {}, error() {} } });
  (0, eval)(g.__code);
  const made = names.filter(n => outs[n]);
  if (!made.length) throw new Error('nothing was sent to an output');
  let head = '', bodies = {}, inputs = [], reads = false;
  const sampled = f => { const all = []; for (let t = 0; t <= g.__seconds + 1e-9; t += g.__step) { g.time = t * g.speed; let v = 0; try { v = f(null, { time: g.time, bpm: g.bpm }, 0); } catch (e) {} all.push(typeof v === 'number' && isFinite(v) ? v : 0); } return all; };
  const fns = new Set();
  for (const n of made) {
    const { frag, uniforms } = outs[n];
    for (const k of Object.keys(synth.glslTransforms)) if (frag.includes(synth.glslTransforms[k].glsl)) fns.add(k);
    let body = frag.slice(frag.indexOf('void main'));
    body = body.slice(body.indexOf('{') + 1, body.lastIndexOf('}'));
    for (const [name, value] of Object.entries(uniforms)) {
      if (name === 'time' || name === 'resolution') continue;
      const own = n === 'o0' ? name : name + '_' + n, word = new RegExp('\\b' + name + '\\b', 'g');
      if (typeof value === 'function') {
        const first = value(null, { time: 0, bpm: g.bpm }, 0);
        if (first && first.getTexture) {
          const t = first.name[0] === 's' ? (reads = true, 'inputImage') : first.name;
          body = body.replace(new RegExp('\\bsrc\\(\\s*([^,]+?),\\s*' + name + '\\s*\\)', 'g'), 'IMG_NORM_PIXEL(' + t + ',fract($1))');   // (a sampler is not passed in ISF: read where it is named)
          if (t !== 'inputImage' && !kept.includes(t)) kept.push(t);
          continue;
        }
        const all = sampled(value), low = Math.min(...all), high = Math.max(...all);
        inputs.push({ NAME: own, TYPE: 'float', DEFAULT: all[0], MIN: Math.min(0, low * 2), MAX: Math.max(1, high * 2) });
        if (low !== high) (g.__moving = g.__moving || []).push({ name: own, keys: all });
      }
      if (own !== name) body = body.replace(word, own);
    }
    bodies[n] = body;
  }
  fns.delete('src');
  head = Object.values(utility).map(t => t.glsl).join('\n') + [...fns].map(k => synth.glslTransforms[k].glsl).join('\n');
  const passes = made.length > 1 || kept.length ? made.map(n => n).concat(kept.filter(n => !made.includes(n))) : [];
  const isf = { ISFVSN: '2', DESCRIPTION: 'Hydra', CATEGORIES: ['Hydra'], INPUTS: (reads ? [{ NAME: 'inputImage', TYPE: 'image' }] : []).concat(inputs) };
  if (passes.length) isf.PASSES = passes.map(n => ({ TARGET: n, PERSISTENT: true })).concat([{}]);
  let main = '';
  for (const n of Object.keys(bodies)) main += 'void door_' + n + '(){' + bodies[n] + '}\n';
  if (passes.length) {
    main += 'void main(){';
    passes.forEach((n, i) => main += 'if(PASSINDEX==' + i + '){' + (bodies[n] ? 'door_' + n + '();' : 'gl_FragColor=vec4(0.,0.,0.,1.);') + 'return;}');
    main += 'gl_FragColor=IMG_NORM_PIXEL(' + shown.name + ',isf_FragNormCoord);}\n';
  } else main += 'void main(){door_' + shown.name + '();}\n';
  g.__isf = '/*' + JSON.stringify(isf) + '*/\n' + head + '\n' + main;
  g.__knobs = JSON.stringify(g.__moving || []);
} catch (e) { globalThis.__why = String(e) + ' ' + (e.stack || ''); }
)";

bool hydra_isf(const std::string& folder, const std::string& code, double seconds, std::string& isf, std::vector<HydraKnob>& knobs, std::string& why) {
    const std::filesystem::path src = std::filesystem::path(folder) / "src";
    std::error_code wrong;
    if (!std::filesystem::exists(src / "generator-factory.js", wrong)) return why = "Hydra is a part of its own and is not there: run filament/extensions/hydra/get.sh", false;
    JSRuntime* rt = JS_NewRuntime();
    JSContext* cx = JS_NewContext(rt);
    JS_SetModuleLoaderFunc(rt, hydra_name, hydra_load, nullptr);
    JSValue g = JS_GetGlobalObject(cx);
    JS_SetPropertyStr(cx, g, "__code", JS_NewStringLen(cx, code.c_str(), code.size()));
    JS_SetPropertyStr(cx, g, "__seconds", JS_NewFloat64(cx, seconds));
    JS_SetPropertyStr(cx, g, "__step", JS_NewFloat64(cx, HYDRA_STEP));
    const std::string name = (src / "motolii.js").string();
    JSValue ran = JS_Eval(cx, hydra_make, strlen(hydra_make), name.c_str(), JS_EVAL_TYPE_MODULE);
    if (JS_IsException(ran)) {
        JSValue e = JS_GetException(cx);
        const char* s = JS_ToCString(cx, e);
        why = s ? s : "Hydra could not be read";
        JS_FreeCString(cx, s), JS_FreeValue(cx, e);
    }
    JS_FreeValue(cx, ran);
    for (JSContext* job; JS_ExecutePendingJob(rt, &job) > 0;) {}
    const auto said = [&](const char* n) {
        JSValue v = JS_GetPropertyStr(cx, g, n);
        std::string out;
        if (JS_IsString(v)) { const char* s = JS_ToCString(cx, v); if (s) out = s; JS_FreeCString(cx, s); }
        JS_FreeValue(cx, v);
        return out;
    };
    if (why.empty()) why = said("__why");
    isf = said("__isf");
    JSValue moving = JS_GetPropertyStr(cx, g, "__moving");
    if (JS_IsArray(moving)) {
        int64_t n = 0;
        JS_GetLength(cx, moving, &n);
        for (int64_t i = 0; i < n; ++i) {
            JSValue one = JS_GetPropertyInt64(cx, moving, i), word = JS_GetPropertyStr(cx, one, "name"), keys = JS_GetPropertyStr(cx, one, "keys");
            HydraKnob k;
            if (const char* s = JS_ToCString(cx, word)) k.name = s, JS_FreeCString(cx, s);
            int64_t m = 0;
            JS_GetLength(cx, keys, &m);
            for (int64_t j = 0; j < m; ++j) { double v = 0; JSValue at = JS_GetPropertyInt64(cx, keys, j); JS_ToFloat64(cx, &v, at); JS_FreeValue(cx, at); k.keys.push_back(v); }
            knobs.push_back(std::move(k));
            JS_FreeValue(cx, keys), JS_FreeValue(cx, word), JS_FreeValue(cx, one);
        }
    }
    JS_FreeValue(cx, moving), JS_FreeValue(cx, g);
    JS_FreeContext(cx), JS_FreeRuntime(rt);
    if (isf.empty() && why.empty()) why = "Hydra gave nothing";
    return !isf.empty();
}
