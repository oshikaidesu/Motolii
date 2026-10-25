// script.h's door: the head read for knobs, QuickJS-ng behind each().
#include "script.h"
#include "ways.h"
#include <quickjs.h>
#include <algorithm>
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
        } else if (k.kind == "picture") {   // (another thing's picture read as one of its images: the thing, by its number)
            k.low = 0, k.high = 1e9, k.usual = 0, k.step = 1;
        } else if (k.kind == "sound" && said.size() >= 3) {   // (another thing's sound read as an ISF audio input: the thing, by its number; fft or wave, how many)
            k.low = 0, k.high = 1e9, k.usual = 0, k.step = 1, k.words = said[1] + " " + said[2];
        } else if (k.kind == "select" && said.size() >= 2) {
            for (size_t i = 1; i < said.size(); ++i) k.words += (i > 1 ? " " : "") + said[i];
        } else continue;
        knobs.push_back(k);
    }
    return knobs;
}

// What a Canvalry distribution asks of Cavalry's JavaScript (the four names its 26 files use), so one runs as it is.
static const char* cavalry_kept = R"(
var cavalry = {
  Point: function (x, y) { this.x = x; this.y = y; },
  PointCloud: function () { this.points = []; this.sizes = []; },
  random: function (low, high, seed, sequence) {
    var h = Math.imul((seed | 0) ^ 0x9E3779B9, 0x85EBCA6B) ^ Math.imul((sequence | 0) + 1, 0xC2B2AE35);
    h ^= h >>> 16; h = Math.imul(h, 0x7FEB352D); h ^= h >>> 15; h = Math.imul(h, 0x846CA68B); h ^= h >>> 16;
    return low + (high - low) * ((h >>> 0) / 4294967296);
  }
};
cavalry.PointCloud.prototype.appendPoint = function (p) { this.points.push(p); };
cavalry.PointCloud.prototype.appendSize = function (s) { this.sizes.push(s); };
// Cavalry's Animators, all of one shape: a value of each copy (a letter of a split text) from one number to another
// over a length of seconds, a way (ease), each copy later than the one before (or in a random order) by up to a delay;
// an In from when the thing begins, an Out to end just before it does. what: x, y, rotate_x, opacity, or print (the
// copies seen one after another, no way).
function animate(i, n, t, k, a) {
  var length = k.length || 40 / 60, delay = k.delay === undefined ? 20 / 60 : k.delay;
  var order = a.random ? cavalry.random(0, 1, a.seed || 50, i) : n > 1 ? i / (n - 1) : 0;
  var from = a.out ? Math.max(0, (k.duration > 0 ? k.duration : 4) - 1 / 60 - delay - length) : 0;
  if (a.what === 'print') {
    var seen = Math.min(1, Math.max(0, (t - from) / length)) * n;
    return { opacity: a.out ? (i < seen ? 0 : 1) : (i < seen ? 1 : 0) };
  }
  var u = Math.min(1, Math.max(0, (t - from - delay * order) / length));
  var out = {};
  out[a.what] = a.from + (a.to - a.from) * ease(a.way, u);
  return out;
}
)";

// ease(name, u): a way by its name (ways.h), the same expression as a key's stretch; not one of them: as it is.
static std::string ways_kept() {
    std::string js = "var ease = (function () { var pow = Math.pow, exp = Math.exp, cos = Math.cos, abs = Math.abs, pi = function () { return Math.PI; }; var w = {";
    for (const auto& one : named_ways) js += std::string(one.name) + ": function (u) { return " + one.goes + "; }, ";
    return js + "}; return function (name, u) { u = Math.min(1, Math.max(0, u)); return w[name] ? w[name](u) : u; }; })();\n";
}

struct Script::Own {
    JSRuntime* rt = nullptr;
    JSContext* cx = nullptr;
    JSValue each = JS_UNDEFINED, knobs = JS_UNDEFINED;
    std::string trouble, file, src;
    bool whole = false;   // no each(): the whole file says every copy at once, as a Canvalry distribution (a PointCloud)
};

Script::Script(const std::string& file) : own(new Own) {
    std::ifstream in(file);
    std::stringstream code;
    code << in.rdbuf();
    const std::string src = code.str();
    own->rt = JS_NewRuntime();
    own->cx = JS_NewContext(own->rt);
    own->file = file, own->src = src;
    const std::string ways = ways_kept();
    JS_FreeValue(own->cx, JS_Eval(own->cx, ways.c_str(), ways.size(), "ways", JS_EVAL_TYPE_GLOBAL));
    JS_FreeValue(own->cx, JS_Eval(own->cx, cavalry_kept, std::char_traits<char>::length(cavalry_kept), "cavalry", JS_EVAL_TYPE_GLOBAL));
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
    own->whole = !JS_IsFunction(own->cx, own->each);
    own->knobs = JS_NewObject(own->cx);
}
Script::~Script() {
    if (!own->cx) return;
    JS_FreeValue(own->cx, own->each), JS_FreeValue(own->cx, own->knobs);
    JS_FreeContext(own->cx), JS_FreeRuntime(own->rt);
}
bool Script::ok() const { return own->trouble.empty(); }
const std::string& Script::trouble() const { return own->trouble; }

std::vector<ScriptCopy> Script::copies(int n, double t, const std::map<std::string, double>& numbers, const std::map<std::string, std::string>& words, double unit) {
    std::vector<ScriptCopy> all((size_t)std::max(n, 0));
    if (!ok()) return all;
    if (!own->whole) {   // (the knobs told once, then each copy asked)
        knobs(numbers, words);
        for (int i = 0; i < n; ++i) all[(size_t)i] = ask(i, n, t);
        return all;
    }
    JSContext* cx = own->cx;
    JSValue g = JS_GetGlobalObject(cx);
    for (const auto& [name, v] : numbers) JS_SetPropertyStr(cx, g, name.c_str(), JS_NewFloat64(cx, v));
    for (const auto& [name, w] : words) JS_SetPropertyStr(cx, g, name.c_str(), JS_NewString(cx, w.c_str()));
    JS_SetPropertyStr(cx, g, "time", JS_NewFloat64(cx, t)), JS_SetPropertyStr(cx, g, "frame", JS_NewFloat64(cx, t * 60.0));
    JS_FreeValue(cx, g);
    JSValue out = JS_Eval(cx, own->src.c_str(), own->src.size(), own->file.c_str(), JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(out)) {
        JSValue e = JS_GetException(cx);
        const char* s = JS_ToCString(cx, e);
        own->trouble = std::string("the script threw: ") + (s ? s : "?");
        JS_FreeCString(cx, s), JS_FreeValue(cx, e);
        return all;
    }
    const auto list = [&](const char* name, auto put) {   // (a cloud's points or sizes, each an {x, y})
        JSValue a = JS_GetPropertyStr(cx, out, name);
        int64_t len = 0;
        if (JS_IsArray(a)) JS_GetLength(cx, a, &len);
        for (int64_t i = 0; i < len && i < n; ++i) {
            JSValue p = JS_GetPropertyUint32(cx, a, (uint32_t)i), x = JS_GetPropertyStr(cx, p, "x"), y = JS_GetPropertyStr(cx, p, "y");
            double px = 0, py = 0;
            JS_ToFloat64(cx, &px, x), JS_ToFloat64(cx, &py, y);
            put(all[(size_t)i], px, py);
            JS_FreeValue(cx, x), JS_FreeValue(cx, y), JS_FreeValue(cx, p);
        }
        JS_FreeValue(cx, a);
        return len;
    };
    const int64_t made = list("points", [&](ScriptCopy& c, double x, double y) { c.x = x * unit, c.y = y * unit; });
    list("sizes", [](ScriptCopy& c, double x, double) { c.size = x; });
    for (int64_t i = made; i < n; ++i) all[(size_t)i].size = 0;   // (copies past the cloud's last point are not seen)
    JS_FreeValue(cx, out);
    return all;
}

ScriptCopy Script::each(int i, int n, double t, const std::map<std::string, double>& numbers, const std::map<std::string, std::string>& words) {
    if (!ok() || own->whole) return ScriptCopy{};
    knobs(numbers, words);
    return ask(i, n, t);
}
void Script::knobs(const std::map<std::string, double>& numbers, const std::map<std::string, std::string>& words) {
    JSContext* cx = own->cx;
    for (const auto& [name, v] : numbers) JS_SetPropertyStr(cx, own->knobs, name.c_str(), JS_NewFloat64(cx, v));
    for (const auto& [name, w] : words) JS_SetPropertyStr(cx, own->knobs, name.c_str(), JS_NewString(cx, w.c_str()));
}
ScriptCopy Script::ask(int i, int n, double t) {   // (each(), the knobs as last told)
    ScriptCopy c;
    JSContext* cx = own->cx;
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
    read("x", c.x), read("y", c.y), read("z", c.z), read("size", c.size), read("rotate", c.rotate), read("rotate_x", c.rotate_x), read("rotate_y", c.rotate_y), read("opacity", c.opacity);
    JS_FreeValue(cx, out);
    return c;
}
