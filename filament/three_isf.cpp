// three_isf.cpp: a three.js screen effect (examples/jsm/shaders/*.js: a module whose export holds uniforms and a GLSL
// fragmentShader reading tDiffuse at vUv) made the text of an ISF effect, so that it goes the ISF way (isf.cpp).
// The module run as it is in QuickJS, as splatfx.cpp runs a PlayCanvas effect: 'three' given as a module of stand-ins
// for what uniforms' values are made of (QuickJS's module loader). Its shader adapted to ISF as ISF's own primer says
// (docs.isf.video, chapter 9: the uniforms to INPUTS, their declarations taken out; texture2D as it is): tDiffuse is
// ISF's inputImage, three's vUv ISF's isf_FragNormCoord, its time ISF's TIME. A knob's range as from_three.py made it
// (from 0, or four times below, to four times its value, at least 1). Not taken: three's own #include pieces.
#include "three_isf.h"
#include <quickjs.h>
#include <filesystem>
#include <fstream>
#include <regex>
#include <sstream>
#include "third_party/filament_src/third_party/libassimp/contrib/rapidjson/include/rapidjson/document.h"
#include "third_party/filament_src/third_party/libassimp/contrib/rapidjson/include/rapidjson/stringbuffer.h"
#include "third_party/filament_src/third_party/libassimp/contrib/rapidjson/include/rapidjson/writer.h"

static const char* const three_stand_ins = R"(
export class Vector2 { constructor(x = 0, y = 0) { this.x = x; this.y = y; } }
export class Vector3 { constructor(x = 0, y = 0, z = 0) { this.x = x; this.y = y; this.z = z; } }
export class Vector4 { constructor(x = 0, y = 0, z = 0, w = 1) { this.x = x; this.y = y; this.z = z; this.w = w; } }
export class Color { constructor(r = 1, g, b) { if (g === undefined) { const h = typeof r === 'number' ? r : 0xffffff; this.r = ((h >> 16) & 255) / 255; this.g = ((h >> 8) & 255) / 255; this.b = (h & 255) / 255; } else { this.r = r; this.g = g; this.b = b; } } }
export class Matrix3 {} export class Matrix4 {}
)";

static JSModuleDef* three_load(JSContext* cx, const char* name, void*) {
    std::string src;
    if (std::string(name) == "three") src = three_stand_ins;
    else { std::ifstream in(name); std::stringstream all; all << in.rdbuf(); src = all.str(); }
    JSValue made = JS_Eval(cx, src.c_str(), src.size(), name, JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY);
    if (JS_IsException(made)) return nullptr;
    auto* m = (JSModuleDef*)JS_VALUE_GET_PTR(made);
    JS_FreeValue(cx, made);
    return m;
}

bool three_is(const std::string& file) {
    if (std::filesystem::path(file).extension() != ".js") return false;
    std::ifstream in(file);
    std::stringstream all;
    all << in.rdbuf();
    return all.str().find("fragmentShader") != std::string::npos;
}

// Its export that says a fragment shader, as JSON: {"frag": ..., "u": {name: value}} (a Vector2 {x, y}, a Color {r, g, b}).
static std::string three_said(const std::string& file, std::string& why) {
    JSRuntime* rt = JS_NewRuntime();
    JSContext* cx = JS_NewContext(rt);
    JS_SetModuleLoaderFunc(rt, nullptr, three_load, nullptr);
    const std::string wrap = "import * as m from '" + std::filesystem::absolute(file).string() + "';\n"
        "const S = Object.values(m).find(v => v && typeof v === 'object' && typeof v.fragmentShader === 'string');\n"
        "globalThis.__said = S ? JSON.stringify({ frag: S.fragmentShader, d: S.defines || {}, u: Object.fromEntries(Object.entries(S.uniforms || {}).map(([k, v]) => [k, v ? v.value : null])) }) : '';\n";
    std::string out;
    JSValue ran = JS_Eval(cx, wrap.c_str(), wrap.size(), "<three effect>", JS_EVAL_TYPE_MODULE);
    if (JS_IsException(ran)) {
        JSValue e = JS_GetException(cx);
        const char* s = JS_ToCString(cx, e);
        why = std::string("the three.js module: ") + (s ? s : "?");
        JS_FreeCString(cx, s), JS_FreeValue(cx, e);
    } else {
        for (JSContext* job; JS_ExecutePendingJob(rt, &job) > 0;) {}
        JSValue g = JS_GetGlobalObject(cx), s = JS_GetPropertyStr(cx, g, "__said");
        if (const char* c = JS_ToCString(cx, s)) out = c, JS_FreeCString(cx, c);
        JS_FreeValue(cx, s), JS_FreeValue(cx, g);
        if (out.empty()) why = "no export says a fragmentShader";
    }
    JS_FreeValue(cx, ran);
    JS_FreeContext(cx), JS_FreeRuntime(rt);
    return out;
}

std::string three_isf(const std::string& file, std::string& why) {
    const std::string said = three_said(file, why);
    rapidjson::Document d;
    if (said.empty() || d.Parse(said.c_str()).HasParseError() || !d.HasMember("frag") || !d["frag"].IsString()) return why.empty() ? why = "its export does not read" : why, "";
    std::string frag = d["frag"].GetString();
    if (std::regex_search(frag, std::regex(R"(^\s*#include\s*<)", std::regex::multiline))) return why = "it names three's own #include pieces", "";
    const rapidjson::Value none(rapidjson::kObjectType);
    const rapidjson::Value& u = d.HasMember("u") && d["u"].IsObject() ? d["u"] : none;
    const auto value = [&](const std::string& n, const char* field, int i, double otherwise) {
        const std::string k = n.size() > 6 && n.compare(n.size() - 6, 6, "_three") == 0 ? n.substr(0, n.size() - 6) : n;
        if (!u.HasMember(k.c_str())) return otherwise;
        const rapidjson::Value& v = u[k.c_str()];
        if (v.IsNumber()) return i == 0 ? v.GetDouble() : otherwise;
        if (v.IsBool()) return (double)v.GetBool();
        if (v.IsObject() && v.HasMember(field) && v[field].IsNumber()) return v[field].GetDouble();
        return otherwise;
    };
    rapidjson::StringBuffer head;
    rapidjson::Writer<rapidjson::StringBuffer> w(head);
    w.StartObject();
    w.Key("CREDIT"), w.String(("three.js (MIT, github.com/mrdoob/three.js): " + std::filesystem::path(file).filename().string()).c_str());
    w.Key("ISFVSN"), w.String("2");
    w.Key("INPUTS"), w.StartArray();
    std::string defines;
    bool first = true;
    if (d.HasMember("d") && d["d"].IsObject())   // (its defines, put before it as three's ShaderMaterial does)
        for (auto& one : d["d"].GetObject())
            defines += "#define " + std::string(one.name.GetString()) + " " + (one.value.IsString() ? std::string(one.value.GetString()) : one.value.IsNumber() ? std::to_string(one.value.GetDouble()) : one.value.IsBool() ? std::string(one.value.GetBool() ? "1" : "0") : std::string()) + "\n";
    const std::regex uniform(R"(^[ \t]*uniform[ \t]+(\w+)[ \t]+(\w+)[ \t]*;[ \t]*(//.*)?$)", std::regex::multiline);
    for (std::sregex_iterator m(frag.begin(), frag.end(), uniform), end; m != end; ++m) {
        const std::string type = (*m)[1], own = (*m)[2];
        const bool taken = own == "resolution" || own == "frame" || own == "passindex";   // (names isf.cpp's knobs have of their own: the shader's renamed)
        const std::string n = taken ? own + "_three" : own;
        if (taken) defines += "#define " + own + " " + n + "\n";
        const auto range = [&](double v) { w.Key("MIN"), w.Double(v < 0 ? 4 * v : 0), w.Key("MAX"), w.Double(std::max(1.0, 4 * std::fabs(v))); };
        if (n == "time" && type == "float") { defines += "#define time TIME\n"; continue; }
        if (type != "vec3") w.StartObject();
        if (type == "sampler2D") {   // (the first its picture in; any other another thing's picture)
            const std::string as = first ? "inputImage" : n;
            if (as != n) defines += "#define " + n + " " + as + "\n";
            w.Key("NAME"), w.String(as.c_str()), w.Key("TYPE"), w.String("image");
            first = false;
        } else if (type == "float" || type == "int") {
            const double v = value(n, "", 0, 0);
            w.Key("NAME"), w.String(n.c_str()), w.Key("TYPE"), w.String(type == "int" ? "long" : "float"), w.Key("DEFAULT"), w.Double(v), range(v);
        } else if (type == "bool") {
            w.Key("NAME"), w.String(n.c_str()), w.Key("TYPE"), w.String("bool"), w.Key("DEFAULT"), w.Bool(value(n, "", 0, 0) != 0);
        } else if (type == "vec2") {
            const double x = value(n, "x", 0, 0), y = value(n, "y", 1, 0);
            w.Key("NAME"), w.String(n.c_str()), w.Key("TYPE"), w.String("point2D"), w.Key("DEFAULT"), w.StartArray(), w.Double(x), w.Double(y), w.EndArray();
            w.Key("MIN"), w.StartArray(), w.Double(std::min(0.0, 4 * x)), w.Double(std::min(0.0, 4 * y)), w.EndArray();
            w.Key("MAX"), w.StartArray(), w.Double(std::max(1.0, 4 * std::fabs(x))), w.Double(std::max(1.0, 4 * std::fabs(y))), w.EndArray();
        } else if (type == "vec4") {
            w.Key("NAME"), w.String(n.c_str()), w.Key("TYPE"), w.String("color"), w.Key("DEFAULT"), w.StartArray();
            for (const char* f : {"x", "y", "z", "w"}) w.Double(value(n, f, 1, 0));
            w.EndArray();
        } else if (type == "vec3") {   // (ISF has no three numbers: three knobs, the shader's own name made of them)
            const bool colour = u.HasMember(n.c_str()) && u[n.c_str()].IsObject() && u[n.c_str()].HasMember("r");
            const char* const fields[] = {colour ? "r" : "x", colour ? "g" : "y", colour ? "b" : "z"};
            for (int i = 0; i < 3; ++i) {
                const double v = value(n, fields[i], i, 0);
                w.StartObject(), w.Key("NAME"), w.String((n + "_" + fields[i]).c_str()), w.Key("TYPE"), w.String("float"), w.Key("DEFAULT"), w.Double(v);
                if (colour) w.Key("MIN"), w.Double(0), w.Key("MAX"), w.Double(1);
                else range(v);
                if (i < 2) w.EndObject();
            }   // (the last closed below, as every other)
            defines += "#define " + n + " vec3(" + n + "_" + fields[0] + ", " + n + "_" + fields[1] + ", " + n + "_" + fields[2] + ")\n";
        } else { w.EndObject(), w.EndArray(), w.EndObject(); return why = "a uniform of a kind ISF has no knob for: " + type + " " + n, ""; }
        w.EndObject();
    }
    if (first) return why = "it reads no picture: not an effect", "";
    w.EndArray(), w.EndObject();
    frag = std::regex_replace(frag, uniform, "");
    std::smatch at;
    if (std::regex_search(frag, at, std::regex(R"(^[ \t]*varying[ \t]+vec2[ \t]+(\w+)[ \t]*;[ \t]*$)", std::regex::multiline)))   // (where on the picture: three's vUv, by whatever name)
        defines += "#define " + at[1].str() + " isf_FragNormCoord\n", frag = std::regex_replace(frag, std::regex(R"(^[ \t]*varying[ \t]+\w+[ \t]+\w+[ \t]*;[ \t]*$)", std::regex::multiline), "");
    if (frag.find("luminance(") != std::string::npos && frag.find("float luminance") == std::string::npos)   // (three puts it before every shader: WebGLProgram.js getLuminanceFunction)
        defines += "float luminance(vec3 rgb) { return dot(vec3(0.2126, 0.7152, 0.0722), rgb); }\n";
    return "/*" + std::string(head.GetString()) + "*/\n" + defines + frag;
}
