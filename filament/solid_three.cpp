// solid_three.cpp: a solid's shape made by three.js (MIT, github.com/mrdoob/three.js r186: build/three.core.js and two
// of its examples, third_party/js/three) as three.js makes it. The shape's file (solids/*.js: its knobs in its head as
// a recipe's, script.h; `export function solid(k)` giving a BufferGeometry) run as it is in QuickJS, as three_isf.cpp
// runs a three.js effect; the geometry's own typed arrays copied out. One QuickJS kept for all (three.core.js read once:
// 24 ms), the modules it has read kept by their names.
#include "solid_three.h"
#include <quickjs.h>
#include <filesystem>
#include <fstream>
#include <mutex>
#include <sstream>

static std::string solid_text(const std::string& file) {
    std::ifstream in(file);
    std::stringstream all;
    all << in.rdbuf();
    return all.str();
}
bool solid_is(const std::string& file) { return std::filesystem::path(file).extension() == ".js" && solid_text(file).find("export function solid") != std::string::npos; }
bool solid_reads(const std::string& file) { return solid_text(file).find("k.alpha") != std::string::npos; }

// A module's name made whole: 'three', as the examples say it, the three.core.js beside them (their own import map,
// examples/*.html); any other relative to the file that names it.
static char* solid_name(JSContext* cx, const char* base, const char* name, void*) {
    const std::filesystem::path dir = std::filesystem::path(base).parent_path();
    const std::filesystem::path to = std::string(name) == "three" ? dir / "three.core.js" : name[0] == '.' ? dir / name : std::filesystem::path(name);
    return js_strdup(cx, to.lexically_normal().string().c_str());
}
static JSModuleDef* solid_load(JSContext* cx, const char* name, void*) {
    const std::string src = solid_text(name);
    JSValue made = JS_Eval(cx, src.c_str(), src.size(), name, JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY);
    if (JS_IsException(made)) return nullptr;
    auto* m = (JSModuleDef*)JS_VALUE_GET_PTR(made);
    JS_FreeValue(cx, made);
    return m;
}

// (the geometry as the wrapper leaves it: typed arrays as globals, made whole and flat first)
static const char* const solid_wrap = R"(
const k = globalThis.__k;
if (globalThis.__cover) k.alpha = { width: k.coverW, height: k.coverH, values: new Uint8Array(globalThis.__cover) };
try {
    const g = solid(k), n = g.attributes.position ? g.attributes.position.count : 0;
    if (n && !g.attributes.normal) g.computeVertexNormals();
    globalThis.__pos = n ? Float32Array.from(g.attributes.position.array) : new Float32Array(0);
    globalThis.__nrm = n ? Float32Array.from(g.attributes.normal.array) : new Float32Array(0);
    globalThis.__uv = n && g.attributes.uv ? Float32Array.from(g.attributes.uv.array) : new Float32Array(2 * n);
    globalThis.__idx = g.index ? Uint32Array.from(g.index.array) : Uint32Array.from({ length: n }, (_, i) => i);
    globalThis.__repeat = !!g.userData.repeat;
} catch (e) { globalThis.__why = String(e) + ' ' + (e.stack || ''); }
)";

template <typename T> static void solid_array(JSContext* cx, JSValue g, const char* name, std::vector<T>& into) {
    JSValue a = JS_GetPropertyStr(cx, g, name);
    size_t from = 0, bytes = 0, each = 0, size = 0;
    JSValue buffer = JS_GetTypedArrayBuffer(cx, a, &from, &bytes, &each);
    if (const uint8_t* p = JS_IsException(buffer) ? nullptr : JS_GetArrayBuffer(cx, &size, buffer)) into.assign((const T*)(p + from), (const T*)(p + from + bytes));
    JS_FreeValue(cx, buffer), JS_FreeValue(cx, a);
}

bool solid_three(const std::string& file, const std::map<std::string, double>& numbers, const std::map<std::string, std::string>& words,
                 double wide, double high, const std::vector<uint8_t>& cover, int coverW, int coverH, SolidMade& out, std::string& why) {
    static std::mutex lock;
    static JSRuntime* rt = nullptr;
    static JSContext* cx = nullptr;
    static int made = 0;
    std::lock_guard<std::mutex> hold(lock);
    if (!rt) rt = JS_NewRuntime(), cx = JS_NewContext(rt), JS_SetModuleLoaderFunc(rt, solid_name, solid_load, nullptr);
    JSValue g = JS_GetGlobalObject(cx), k = JS_NewObject(cx);
    for (const auto& [n, v] : numbers) JS_SetPropertyStr(cx, k, n.c_str(), JS_NewFloat64(cx, v));
    for (const auto& [n, w] : words) JS_SetPropertyStr(cx, k, n.c_str(), JS_NewString(cx, w.c_str()));
    JS_SetPropertyStr(cx, k, "w", JS_NewFloat64(cx, wide)), JS_SetPropertyStr(cx, k, "h", JS_NewFloat64(cx, high));
    JS_SetPropertyStr(cx, k, "coverW", JS_NewInt32(cx, coverW)), JS_SetPropertyStr(cx, k, "coverH", JS_NewInt32(cx, coverH));
    JS_SetPropertyStr(cx, g, "__k", k);
    JS_SetPropertyStr(cx, g, "__cover", cover.empty() ? JS_UNDEFINED : JS_NewArrayBufferCopy(cx, cover.data(), cover.size()));
    JS_SetPropertyStr(cx, g, "__why", JS_NewString(cx, ""));
    const std::string code = "import { solid } from '" + std::filesystem::absolute(file).lexically_normal().string() + "';\n" + solid_wrap;
    const std::string name = "<solid " + std::to_string(++made) + ">";   // (a module of its own each time; the shape's, read once)
    JSValue ran = JS_Eval(cx, code.c_str(), code.size(), name.c_str(), JS_EVAL_TYPE_MODULE);
    for (JSContext* job; JS_ExecutePendingJob(rt, &job) > 0;) {}
    if (JS_IsException(ran)) {
        JSValue e = JS_GetException(cx);
        const char* s = JS_ToCString(cx, e);
        why = std::string("the shape's module: ") + (s ? s : "?");
        JS_FreeCString(cx, s), JS_FreeValue(cx, e);
    } else if (JSValue w = JS_GetPropertyStr(cx, g, "__why"); const char* s = JS_ToCString(cx, w)) {
        why = s;
        JS_FreeCString(cx, s), JS_FreeValue(cx, w);
    }
    JS_FreeValue(cx, ran);
    out = SolidMade{};
    if (why.empty()) {
        solid_array(cx, g, "__pos", out.pos), solid_array(cx, g, "__nrm", out.nrm), solid_array(cx, g, "__uv", out.uv), solid_array(cx, g, "__idx", out.idx);
        JSValue r = JS_GetPropertyStr(cx, g, "__repeat");
        out.repeat = JS_ToBool(cx, r) > 0;
        JS_FreeValue(cx, r);
    }
    JS_FreeValue(cx, g);
    return why.empty();
}
