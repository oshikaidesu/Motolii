// script.h's door: the head read for knobs, QuickJS-ng behind each().
#include "script.h"
#include <quickjs.h>
#include <cstdlib>
#include <fstream>
#include <sstream>

static std::vector<std::string> split(const std::string& s, char by) {
    std::vector<std::string> out;
    std::stringstream in(s);
    for (std::string one; std::getline(in, one, by);) {
        const size_t a = one.find_first_not_of(' '), b = one.find_last_not_of(' ');
        out.push_back(a == std::string::npos ? "" : one.substr(a, b - a + 1));
    }
    return out;
}

std::vector<ScriptKnob> script_knobs(const std::string& file) {
    std::vector<ScriptKnob> knobs;
    std::ifstream in(file);
    for (std::string line; std::getline(in, line);) {
        if (line.rfind("//", 0) != 0) { if (line.find_first_not_of(" \t\r") == std::string::npos) continue; break; }   // (the head ends where code begins)
        const size_t dash = line.find("--"), at = line.find('@'), colon = line.find(':');
        if (dash == std::string::npos || at == std::string::npos || colon == std::string::npos || !(dash < at && at < colon)) continue;
        ScriptKnob k;
        k.kind = line.substr(dash + 2, at - dash - 2), k.name = line.substr(at + 1, colon - at - 1);
        const std::vector<std::string> said = split(line.substr(colon + 1), ',');
        if (said.empty() || k.name.empty()) continue;
        k.label = said[0];
        if (k.kind == "track" && said.size() >= 4) {
            k.low = std::atof(said[1].c_str()), k.high = std::atof(said[2].c_str()), k.usual = std::atof(said[3].c_str());
            k.step = said.size() >= 5 ? std::atof(said[4].c_str()) : 0;
        } else if (k.kind == "check") {
            k.low = 0, k.high = 1, k.usual = said.size() >= 2 ? std::atof(said[1].c_str()) : 0, k.step = 1;
        } else if (k.kind == "select" && said.size() >= 2) {
            for (size_t i = 1; i < said.size(); ++i) k.words += (i > 1 ? " " : "") + said[i];
        } else continue;
        knobs.push_back(k);
    }
    return knobs;
}

struct Script::Own {
    JSRuntime* rt = nullptr;
    JSContext* cx = nullptr;
    JSValue each = JS_UNDEFINED, knobs = JS_UNDEFINED;
    std::string trouble;
};

Script::Script(const std::string& file) : own(new Own) {
    std::ifstream in(file);
    std::stringstream code;
    code << in.rdbuf();
    const std::string src = code.str();
    own->rt = JS_NewRuntime();
    own->cx = JS_NewContext(own->rt);
    JSValue ran = JS_Eval(own->cx, src.c_str(), src.size(), file.c_str(), JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(ran)) {
        JSValue e = JS_GetException(own->cx);
        const char* s = JS_ToCString(own->cx, e);
        own->trouble = std::string("the script ") + file + ": " + (s ? s : "?");
        JS_FreeCString(own->cx, s), JS_FreeValue(own->cx, e);
    }
    JS_FreeValue(own->cx, ran);
    JSValue g = JS_GetGlobalObject(own->cx);
    own->each = JS_GetPropertyStr(own->cx, g, "each");
    JS_FreeValue(own->cx, g);
    if (own->trouble.empty() && !JS_IsFunction(own->cx, own->each)) own->trouble = "the script " + file + " says no each(i, n, t, k)";
    own->knobs = JS_NewObject(own->cx);
}
Script::~Script() {
    if (!own->cx) return;
    JS_FreeValue(own->cx, own->each), JS_FreeValue(own->cx, own->knobs);
    JS_FreeContext(own->cx), JS_FreeRuntime(own->rt);
}
bool Script::ok() const { return own->trouble.empty(); }
const std::string& Script::trouble() const { return own->trouble; }

ScriptCopy Script::each(int i, int n, double t, const std::map<std::string, double>& numbers, const std::map<std::string, std::string>& words) {
    ScriptCopy c;
    if (!ok()) return c;
    JSContext* cx = own->cx;
    for (const auto& [name, v] : numbers) JS_SetPropertyStr(cx, own->knobs, name.c_str(), JS_NewFloat64(cx, v));
    for (const auto& [name, w] : words) JS_SetPropertyStr(cx, own->knobs, name.c_str(), JS_NewString(cx, w.c_str()));
    JSValue args[4] = {JS_NewInt32(cx, i), JS_NewInt32(cx, n), JS_NewFloat64(cx, t), JS_DupValue(cx, own->knobs)};
    JSValue out = JS_Call(cx, own->each, JS_UNDEFINED, 4, args);
    JS_FreeValue(cx, args[3]);
    if (JS_IsException(out)) {
        JSValue e = JS_GetException(cx);
        const char* s = JS_ToCString(cx, e);
        own->trouble = std::string("each() threw: ") + (s ? s : "?");
        JS_FreeCString(cx, s), JS_FreeValue(cx, e);
        return c;
    }
    const auto read = [&](const char* name, double& to) {
        JSValue v = JS_GetPropertyStr(cx, out, name);
        if (JS_IsNumber(v)) JS_ToFloat64(cx, &to, v);
        JS_FreeValue(cx, v);
    };
    read("x", c.x), read("y", c.y), read("z", c.z), read("size", c.size), read("rotate", c.rotate);
    JS_FreeValue(cx, out);
    return c;
}
