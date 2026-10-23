// (splatfx.h says what this is)
#include "splatfx.h"
#include <quickjs.h>
#include <filesystem>
#include <fstream>
#include <regex>
#include <sstream>

// What an effect takes from 'playcanvas', and its base class, as small as it uses them.
static const char* const splatfx_playcanvas = R"(
export class Vec3 { constructor(x = 0, y = 0, z = 0) { this.x = x; this.y = y; this.z = z; } set(x, y, z) { this.x = x; this.y = y; this.z = z; return this; } copy(v) { return this.set(v.x, v.y, v.z); } clone() { return new Vec3(this.x, this.y, this.z); } }
export class Color { constructor(r = 0, g = 0, b = 0, a = 1) { this.r = r; this.g = g; this.b = b; this.a = a; } set(r, g, b, a = 1) { this.r = r; this.g = g; this.b = b; this.a = a; return this; } copy(c) { return this.set(c.r, c.g, c.b, c.a); } clone() { return new Color(this.r, this.g, this.b, this.a); } }
export class Script { constructor() { this.enabled = true; } on() {} off() {} fire() {} }
)";
static const char* const splatfx_base = R"(
import { Script } from 'playcanvas';
export class GSplatShaderEffect extends Script {
    constructor() { super(); this.effectTime = 0; this.__uniforms = {}; }
    setUniform(name, v) { this.__uniforms[name] = typeof v === 'number' ? [v] : Array.from(v); }
    isEffectComplete() { const t = this.getCompletionTime ? this.getCompletionTime() : Infinity; return this.effectTime >= t; }
}
)";

struct SplatEffect::Own {
    JSRuntime* rt = nullptr;
    JSContext* cx = nullptr;
    JSValue effect = JS_UNDEFINED;
    std::string trouble, glsl, file;
    std::vector<ScriptKnob> knobs;
    std::vector<std::pair<std::string, int>> fields;   // (each attribute and how many numbers it is: 1, or 3)
    bool finished = false;
};

static JSModuleDef* splatfx_load(JSContext* cx, const char* name, void*) {
    const std::string n = name;
    std::string src;
    if (n == "playcanvas") src = splatfx_playcanvas;
    else if (std::filesystem::path(n).filename() == "gsplat-shader-effect.mjs") src = splatfx_base;
    else { std::ifstream in(n); std::stringstream all; all << in.rdbuf(); src = all.str(); }
    JSValue made = JS_Eval(cx, src.c_str(), src.size(), name, JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY);
    if (JS_IsException(made)) return nullptr;
    auto* m = (JSModuleDef*)JS_VALUE_GET_PTR(made);
    JS_FreeValue(cx, made);
    return m;
}

static std::string splatfx_said(JSContext* cx) {
    JSValue e = JS_GetException(cx);
    const char* s = JS_ToCString(cx, e);
    std::string out = s ? s : "?";
    JS_FreeCString(cx, s), JS_FreeValue(cx, e);
    return out;
}

SplatEffect::SplatEffect(const std::string& file) : own(new Own) {
    own->file = file;
    own->rt = JS_NewRuntime();
    own->cx = JS_NewContext(own->rt);
    JS_SetModuleLoaderFunc(own->rt, nullptr, splatfx_load, nullptr);
    JSContext* cx = own->cx;
    // (its one class that says a shader, made once, as PlayCanvas makes a script)
    const std::string wrap = "import * as m from '" + std::filesystem::absolute(file).string() + "';\n"
        "const C = Object.values(m).find(v => typeof v === 'function' && v.prototype && v.prototype.getShaderGLSL);\n"
        "globalThis.__effect = C ? new C() : null;\n";
    JSValue ran = JS_Eval(cx, wrap.c_str(), wrap.size(), "<splat effect>", JS_EVAL_TYPE_MODULE);
    if (JS_IsException(ran)) { own->trouble = "the splat effect " + file + ": " + splatfx_said(cx); return; }
    JS_FreeValue(cx, ran);
    for (JSContext* job; JS_ExecutePendingJob(own->rt, &job) > 0;) {}
    JSValue g = JS_GetGlobalObject(cx);
    own->effect = JS_GetPropertyStr(cx, g, "__effect");
    JS_FreeValue(cx, g);
    if (!JS_IsObject(own->effect)) { own->trouble = "the splat effect " + file + " has no class with getShaderGLSL()"; return; }
    JSValue fn = JS_GetPropertyStr(cx, own->effect, "getShaderGLSL"), said = JS_Call(cx, fn, own->effect, 0, nullptr);
    if (const char* s = JS_ToCString(cx, said)) own->glsl = s, JS_FreeCString(cx, s);
    JS_FreeValue(cx, said), JS_FreeValue(cx, fn);
    // (its attributes: the numbers, Vec3s and Colors it holds as it comes, each with the range its comment gives)
    std::ifstream in(file);
    std::stringstream all;
    all << in.rdbuf();
    const std::string text = all.str();
    JSPropertyEnum* props = nullptr;
    uint32_t count = 0;
    JS_GetOwnPropertyNames(cx, &props, &count, own->effect, JS_GPN_STRING_MASK | JS_GPN_ENUM_ONLY);
    for (uint32_t i = 0; i < count; ++i) {
        const char* key = JS_AtomToCString(cx, props[i].atom);
        const std::string name = key ? key : "";
        JS_FreeCString(cx, key);
        if (name.empty() || name[0] == '_' || name == "effectTime" || name == "enabled") continue;
        double low = 0, high = 1;
        std::smatch r;   // (/** ... @range [a, b] ... */ name = ...)
        if (std::regex_search(text, r, std::regex("@range\\s*\\[\\s*(-?[\\d.]+)\\s*,\\s*(-?[\\d.]+)\\s*\\][^/]*\\*/\\s*" + name + "\\s*=")))
            low = std::stod(r[1]), high = std::stod(r[2]);
        JSValue v = JS_GetPropertyStr(cx, own->effect, name.c_str());
        const auto knob = [&](const std::string& n, const std::string& label, double usual, double lo, double hi) {
            ScriptKnob k;
            k.name = n, k.label = label, k.kind = "track", k.low = lo, k.high = hi, k.usual = usual, k.step = 0.001;
            own->knobs.push_back(k);
        };
        double one = 0;
        if (JS_IsNumber(v)) {
            JS_ToFloat64(cx, &one, v);
            knob(name, name, one, std::min(low, one), std::max(high, one));
            own->fields.push_back({name, 1});
        } else if (JS_IsObject(v)) {
            const bool colour = JS_HasProperty(cx, v, JS_NewAtom(cx, "r"));
            const char* parts = colour ? "rgb" : "xyz";
            for (int c = 0; c < 3; ++c) {
                JSValue p = JS_GetPropertyStr(cx, v, std::string(1, parts[c]).c_str());
                JS_ToFloat64(cx, &one, p);
                JS_FreeValue(cx, p);
                knob(name + "_" + parts[c], name + " " + parts[c], one, colour ? 0.0 : std::min(-10.0, one), colour ? std::max(1.0, one) : std::max(10.0, one));
            }
            own->fields.push_back({name, colour ? 3 : -3});
        }
        JS_FreeValue(cx, v);
    }
    js_free(cx, props);
}

SplatEffect::~SplatEffect() {
    if (!own->cx) return;
    JS_FreeValue(own->cx, own->effect);
    JS_FreeContext(own->cx), JS_FreeRuntime(own->rt);
}
bool SplatEffect::ok() const { return own->trouble.empty(); }
const std::string& SplatEffect::trouble() const { return own->trouble; }
const std::string& SplatEffect::glsl() const { return own->glsl; }
const std::vector<ScriptKnob>& SplatEffect::knobs() const { return own->knobs; }
bool SplatEffect::finished() const { return own->finished; }
void SplatEffect::usual(const std::string& knob, double is) {
    for (ScriptKnob& k : own->knobs) if (k.name == knob) k.usual = is, k.low = std::min(k.low, is), k.high = std::max(k.high, is);
}

std::vector<std::pair<std::string, std::vector<float>>> SplatEffect::uniforms(double time, double dt, const std::function<double(const std::string&, double)>& value) {
    std::vector<std::pair<std::string, std::vector<float>>> out;
    if (!ok()) return out;
    JSContext* cx = own->cx;
    size_t k = 0;
    for (const auto& [name, n] : own->fields) {   // (each attribute as the knobs say it now)
        if (n == 1) { JS_SetPropertyStr(cx, own->effect, name.c_str(), JS_NewFloat64(cx, value(own->knobs[k].name, own->knobs[k].usual))); ++k; continue; }
        JSValue v = JS_GetPropertyStr(cx, own->effect, name.c_str());
        const char* parts = n == 3 ? "rgb" : "xyz";
        for (int c = 0; c < 3; ++c, ++k) JS_SetPropertyStr(cx, v, std::string(1, parts[c]).c_str(), JS_NewFloat64(cx, value(own->knobs[k].name, own->knobs[k].usual)));
        JS_FreeValue(cx, v);
    }
    JS_SetPropertyStr(cx, own->effect, "effectTime", JS_NewFloat64(cx, time));
    JS_SetPropertyStr(cx, own->effect, "enabled", JS_TRUE);
    JSValue fn = JS_GetPropertyStr(cx, own->effect, "updateEffect");
    JSValue args[2] = {JS_NewFloat64(cx, time), JS_NewFloat64(cx, dt)};
    JS_FreeValue(cx, JS_Call(cx, fn, own->effect, 2, args));
    JS_FreeValue(cx, fn);
    JSValue on = JS_GetPropertyStr(cx, own->effect, "enabled");
    own->finished = !JS_ToBool(cx, on);
    JS_FreeValue(cx, on);
    JSValue said = JS_GetPropertyStr(cx, own->effect, "__uniforms");
    JSPropertyEnum* props = nullptr;
    uint32_t count = 0;
    JS_GetOwnPropertyNames(cx, &props, &count, said, JS_GPN_STRING_MASK | JS_GPN_ENUM_ONLY);
    for (uint32_t i = 0; i < count; ++i) {
        const char* key = JS_AtomToCString(cx, props[i].atom);
        JSValue a = JS_GetProperty(cx, said, props[i].atom);
        int64_t len = 0;
        JS_GetLength(cx, a, &len);
        std::vector<float> floats;
        for (int64_t j = 0; j < len; ++j) { JSValue x = JS_GetPropertyUint32(cx, a, (uint32_t)j); double d = 0; JS_ToFloat64(cx, &d, x); floats.push_back((float)d); JS_FreeValue(cx, x); }
        out.push_back({key ? key : "", floats});
        JS_FreeCString(cx, key), JS_FreeValue(cx, a);
    }
    js_free(cx, props);
    JS_FreeValue(cx, said);
    return out;
}

#include <filamat/MaterialBuilder.h>
#include <filament-matp/MaterialParser.h>
#include <mutex>

namespace {
struct SplatConfig : matp::Config {   // (as kit_wgpu.sh runs matc: -a webgpu -a metal -p desktop)
    SplatConfig() { mTargetApi = TargetApi::WEBGPU | TargetApi::METAL; mPlatform = Platform::DESKTOP; }
    Output* getOutput() const noexcept override { return nullptr; }
    Input* getInput() const noexcept override { return nullptr; }
    std::string toString() const noexcept override { return "splat effect"; }
    std::string toPIISafeString() const noexcept override { return "splat effect"; }
};
}

std::vector<uint8_t> splat_effect_look(const std::string& mat, const std::string& glsl, utils::JobSystem& jobs, std::string& why) {
    uint64_t hash = 1469598103934665603ull;
    for (unsigned char c : mat + "\x01" + glsl) hash = (hash ^ c) * 1099511628211ull;
    const char* home = std::getenv("HOME");
    const std::filesystem::path dir = std::filesystem::path(home ? home : "/tmp") / "Library/Caches/Motolii/splatfx";
    char name[40];
    std::snprintf(name, sizeof name, "%016llx.filamat", (unsigned long long)hash);
    const std::filesystem::path kept = dir / name;
    if (std::ifstream in(kept, std::ios::binary); in) return std::vector<uint8_t>(std::istreambuf_iterator<char>(in), {});
    // (its uniforms said as the look's parameters, read through their own names; the rest of its text as it is)
    std::string params, names, code;
    std::istringstream lines(glsl);
    const std::regex uniform(R"(^\s*uniform\s+(?:highp\s+|mediump\s+|lowp\s+)?(float|vec2|vec3|vec4|int|mat3|mat4)\s+(\w+)\s*;)");
    for (std::string line; std::getline(lines, line);) {
        std::smatch u;
        if (!std::regex_search(line, u, uniform)) { code += line + "\n"; continue; }
        const std::string t = u[1], n = u[2];
        const std::string type = t == "vec2" ? "float2" : t == "vec3" ? "float3" : t == "vec4" ? "float4" : t;
        params += "        { type : " + type + ", name : " + n + " },\n";
        names += "#define " + n + " materialParams." + n + "\n";
    }
    std::string text = mat;
    const size_t list = text.find("parameters : [");
    const size_t at = text.find("//@effect");
    if (list == std::string::npos || at == std::string::npos) return why = "points.mat has no place for an effect", std::vector<uint8_t>{};
    text.insert(at, "#define SPLAT_EFFECT\n" + names + code + "\n");
    text.insert(list + 14, "\n" + params);
    static std::mutex one;   // (filamat is made ready once, and builds one at a time)
    std::lock_guard<std::mutex> hold(one);
    static bool ready = (filamat::MaterialBuilder::init(), true);
    (void)ready;
    SplatConfig config;
    matp::MaterialParser parser;
    filamat::MaterialBuilder builder;
    ssize_t size = (ssize_t)text.size();
    std::unique_ptr<const char[]> buffer(new char[text.size()]);
    std::memcpy(const_cast<char*>(buffer.get()), text.data(), text.size());
    if (const utils::Status s = parser.parse(builder, config, size, buffer); !s.isOk()) return why = "the effect's look: " + std::string(s.getMessage()), std::vector<uint8_t>{};
    const filamat::Package package = builder.build(jobs);
    if (!package.isValid()) return why = "the effect's look could not be built (its GLSL as Filament takes it)", std::vector<uint8_t>{};
    std::vector<uint8_t> bytes((const uint8_t*)package.getData(), (const uint8_t*)package.getData() + package.getSize());
    std::error_code e;
    std::filesystem::create_directories(dir, e);
    std::ofstream(kept, std::ios::binary).write((const char*)bytes.data(), (std::streamsize)bytes.size());
    return bytes;
}
