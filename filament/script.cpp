// script.h's door: the head read for knobs, QuickJS-ng behind each().
#include "script.h"
#include "door.h"
#include "aviutl.h"
#include "p5_stand.h"
#include "film_read.h"
#include "ways.h"
#include <quickjs.h>
#include <algorithm>
#include <filesystem>
#include <set>
#include <cstdlib>
#include <fstream>
#include <mutex>
#include <sstream>
#include <sys/stat.h>

static std::vector<std::string> split(const std::string& s, char by) {
    std::vector<std::string> out;
    std::stringstream in(s);
    for (std::string one; std::getline(in, one, by);) {
        const size_t a = one.find_first_not_of(' '), b = one.find_last_not_of(' ');
        out.push_back(a == std::string::npos ? "" : one.substr(a, b - a + 1));
    }
    return out;
}

// (what stands in a string or a comment is not a call; a name after a dot is another canvas's, a createGraphics': image()
// says where that shows; a name followed by a { is a method of the sketch's own)
std::string sketch_marked(const std::string& src, std::vector<SketchLine>* lines) {
    static const std::set<std::string> draws{"rect", "square", "ellipse", "circle", "arc", "line", "point", "triangle", "quad", "bezier", "curve", "image", "text", "background", "beginShape",
                                             "box", "sphere", "plane", "cone", "cylinder", "torus", "ellipsoid", "model"};
    const auto word = [](char c) { return std::isalnum((unsigned char)c) || c == '_' || c == '$'; };
    std::string out, last;
    int line = 1, seen = 0;
    for (size_t i = 0; i < src.size();) {
        const char c = src[i];
        if (c == '\n') ++line;
        if (c == '/' && i + 1 < src.size() && src[i + 1] == '/') {
            const size_t end = std::min(src.find('\n', i), src.size());
            out.append(src, i, end - i), i = end;
        } else if (c == '/' && i + 1 < src.size() && src[i + 1] == '*') {
            const size_t end = std::min(src.find("*/", i + 2), src.size() - 2) + 2;
            line += (int)std::count(src.begin() + (long)i, src.begin() + (long)end, '\n');
            out.append(src, i, end - i), i = end;
        } else if (c == '"' || c == '\'' || c == '`') {
            size_t end = i + 1;
            while (end < src.size() && src[end] != c && (c == '`' || src[end] != '\n')) end += src[end] == '\\' ? 2 : 1;
            end = std::min(end + 1, src.size());
            line += (int)std::count(src.begin() + (long)i, src.begin() + (long)end, '\n');
            out.append(src, i, end - i), i = end;
        } else if (word(c)) {
            size_t end = i;
            while (end < src.size() && word(src[end])) ++end;
            const std::string name = src.substr(i, end - i);
            size_t next = end;
            while (next < src.size() && (src[next] == ' ' || src[next] == '\t')) ++next;
            const size_t before = out.find_last_not_of(" \t\r\n");
            bool mark = next < src.size() && src[next] == '(' && draws.count(name) && last != "function" && (before == std::string::npos || out[before] != '.');
            if (mark) {
                int deep = 0;
                size_t close = next;
                for (; close < src.size(); ++close) { if (src[close] == '(') ++deep; else if (src[close] == ')' && --deep == 0) break; }
                size_t after = close + 1;
                while (after < src.size() && std::isspace((unsigned char)src[after])) ++after;
                mark = after >= src.size() || src[after] != '{';
            }
            if (mark) {
                out += "__at(" + std::to_string(line) + "," + name + ")";   // (a word as it was, so a line before it with no semicolon still ends there)
                if (lines && seen != line) lines->push_back({line, name}), seen = line;
            } else out += name;
            last = name, i = end;
        } else {
            if (!std::isspace((unsigned char)c)) last.clear();
            out += c, ++i;
        }
    }
    return out;
}
std::vector<ScriptKnob> script_knobs(const std::string& file) {
    if (aviutl_is(file)) return aviutl_knobs(file);   // (an AviUtl2 script's head: aviutl.cpp)
    std::vector<ScriptKnob> knobs;
    std::ifstream in(file);
    if (std::filesystem::path(file).extension() == ".js") {
        std::stringstream all;
        all << in.rdbuf();
        const std::string told = door_js(all.str(), door_form(all.str()));   // (a form from the net told as a sketch: door.h)
        if (script_is_sketch(told)) {
            if (door_form(all.str()) == "three") {   // (which part of the scene this thing is: doc_door.h)
                ScriptKnob part;
                part.kind = "track", part.name = "part", part.label = "Part", part.high = 9999, part.step = 1;
                return {part};
            }
            std::vector<SketchLine> lines;
            sketch_marked(told, &lines);
            for (const SketchLine& l : lines) {
                ScriptKnob k;
                k.kind = "track", k.name = "line_" + std::to_string(l.line), k.label = l.what + " " + std::to_string(l.line), k.high = 1, k.usual = 1;
                knobs.push_back(k);
            }
            return knobs;
        }
        in.clear(), in.seekg(0);
    }
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

void script_first(const std::string& file, std::vector<ScriptKnob>& knobs) {
    static std::mutex hold;
    static std::map<std::string, std::pair<long long, std::map<std::string, std::map<std::string, double>>>> read;   // (each folder's first.tsv, as it was when last written)
    const size_t slash = file.find_last_of('/');
    if (slash == std::string::npos) return;
    const std::string table = file.substr(0, slash) + "/first.tsv", name = file.substr(slash + 1);
    struct stat st{};
    const long long when = stat(table.c_str(), &st) == 0 ? (long long)st.st_mtimespec.tv_sec * 1000000000LL + st.st_mtimespec.tv_nsec : -1;
    std::lock_guard<std::mutex> one(hold);
    auto& [stamp, values] = read[table];
    if (stamp != when) {
        stamp = when, values.clear();
        std::ifstream in(table);
        for (std::string line; std::getline(in, line);) {
            const std::vector<std::string> w = split(line, '\t');
            if (w.size() >= 3 && !w[0].empty() && w[0][0] != '#') values[w[0]][w[1]] = std::atof(w[2].c_str());
        }
    }
    const auto own = values.find(name);
    if (own == values.end()) return;
    for (ScriptKnob& k : knobs)
        if (const auto v = own->second.find(k.name); v != own->second.end()) k.usual = v->second;
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

static JSValue film_next(JSContext* cx, JSValueConst, int, JSValueConst*) {   // (the film's next picture, 1/60 s on: its size and its bytes)
    FilmRead* f = (FilmRead*)JS_GetContextOpaque(cx);
    if (!f->reader || !f->next()) return JS_NULL;
    JSValue o = JS_NewObject(cx);
    JS_SetPropertyStr(cx, o, "w", JS_NewInt32(cx, f->w)), JS_SetPropertyStr(cx, o, "h", JS_NewInt32(cx, f->h));
    JS_SetPropertyStr(cx, o, "data", JS_NewArrayBufferCopy(cx, f->last.data(), f->last.size()));
    return o;
}
static JSValue image_read(JSContext* cx, JSValueConst, int, JSValueConst* a) {   // (a picture its sketch came with, by the name it gives it: only the last part of that name, only from that folder)
    const char* name = JS_ToCString(cx, a[0]);
    const std::filesystem::path file = std::filesystem::path(((FilmRead*)JS_GetContextOpaque(cx))->beside) / std::filesystem::path(name ? name : "").filename();
    JS_FreeCString(cx, name);
    std::error_code wrong;
    FilmRead f;
    if (!std::filesystem::is_regular_file(file, wrong) || !f.open(file.string(), 0)) return JS_NULL;
    JSValue o = JS_NewObject(cx);
    JS_SetPropertyStr(cx, o, "w", JS_NewInt32(cx, f.w)), JS_SetPropertyStr(cx, o, "h", JS_NewInt32(cx, f.h));
    JS_SetPropertyStr(cx, o, "data", JS_NewArrayBufferCopy(cx, f.last.data(), f.last.size()));
    return o;
}
struct Script::Own {
    JSRuntime* rt = nullptr;
    JSContext* cx = nullptr;
    JSValue each = JS_UNDEFINED, knobs = JS_UNDEFINED;
    std::string trouble, file, src;
    bool whole = false;   // no each(): the whole file says every copy at once, as a Canvalry distribution (a PointCloud)
    bool sketch = false;   // a p5.js sketch (p5_stand.h)
    FilmRead film;         // what its createCapture and createVideo see
    std::vector<SketchLine> lines;   // its lines that draw
};

Script::Script(const std::string& file, const std::string& p5, const std::string& film) : own(new Own) {
    std::ifstream in(file);
    std::stringstream code;
    code << in.rdbuf();
    const std::string form = door_form(code.str()), src = door_js(code.str(), form);   // (a form from the net told as a sketch: door.h)
    own->rt = JS_NewRuntime();
    JS_SetMemoryLimit(own->rt, 2ull << 30);   // (a sketch keeps its frames: 100,000 rectangles a frame are 1.5 GB a second. It stops with an error, not with the machine)
    own->cx = JS_NewContext(own->rt);
    own->file = file, own->src = src;
    const std::string ways = ways_kept();
    JS_FreeValue(own->cx, JS_Eval(own->cx, ways.c_str(), ways.size(), "ways", JS_EVAL_TYPE_GLOBAL));
    JS_FreeValue(own->cx, JS_Eval(own->cx, cavalry_kept, std::char_traits<char>::length(cavalry_kept), "cavalry", JS_EVAL_TYPE_GLOBAL));
    const bool sketch = own->sketch = !p5.empty() && script_is_sketch(src);
    if (sketch) {
        std::ifstream lib(p5);
        std::stringstream all;
        all << lib.rdbuf();
        const std::string p5js = all.str();
        JS_FreeValue(own->cx, JS_Eval(own->cx, p5_before, std::char_traits<char>::length(p5_before), "p5_before", JS_EVAL_TYPE_GLOBAL));
        JS_FreeValue(own->cx, JS_Eval(own->cx, p5js.c_str(), p5js.size(), "p5.min.js", JS_EVAL_TYPE_GLOBAL));
        for (const char* stand : {p5_after, p5_three, p5_mesh}) JS_FreeValue(own->cx, JS_Eval(own->cx, stand, std::char_traits<char>::length(stand), "p5_after", JS_EVAL_TYPE_GLOBAL));
        if (const std::string needs = door_library(form); !needs.empty()) {
            std::ifstream one(std::filesystem::path(p5).parent_path().parent_path() / needs);
            std::stringstream text;
            text << one.rdbuf();
            const std::string lib = door_library_text(form, text.str());
            JS_FreeValue(own->cx, JS_Eval(own->cx, lib.c_str(), lib.size(), needs.c_str(), JS_EVAL_TYPE_GLOBAL));
        }
        if (!film.empty() && (src.find("createCapture") != std::string::npos || src.find("createVideo") != std::string::npos || src.find("loadImage") != std::string::npos)) own->film.open(film, 320);   // (as much of it as a webcam sketch reads, and light)
        JS_SetContextOpaque(own->cx, &own->film);
        own->film.beside = (std::filesystem::path(file).parent_path() / std::filesystem::path(file).stem()).string();
        JSValue g = JS_GetGlobalObject(own->cx);
        JS_SetPropertyStr(own->cx, g, "__film_next", JS_NewCFunction(own->cx, film_next, "__film_next", 0));
        JS_SetPropertyStr(own->cx, g, "__image_read", JS_NewCFunction(own->cx, image_read, "__image_read", 1));
        JS_FreeValue(own->cx, g);
    }
    std::string body = src;
    if (sketch) {
        std::string marked = sketch_marked(src, &own->lines);
        JSValue told = JS_Eval(own->cx, marked.c_str(), marked.size(), file.c_str(), JS_EVAL_TYPE_GLOBAL | JS_EVAL_FLAG_COMPILE_ONLY);
        if (JS_IsException(told)) JS_FreeValue(own->cx, JS_GetException(own->cx)), own->lines.clear();   // (a file this reader misread runs as it is, its lines not told)
        else body = std::move(marked);
        JS_FreeValue(own->cx, told);
    }
    const std::string run = sketch ? body + "\n;\n" + p5_start : src;
    JSValue ran = JS_Eval(own->cx, run.c_str(), run.size(), file.c_str(), JS_EVAL_TYPE_GLOBAL);
    if (sketch) for (JSContext* job = nullptr; JS_ExecutePendingJob(own->rt, &job) > 0;) {}   // (an async setup(), as p5 2 sketches are, goes on to its end)
    if (JS_IsException(ran)) {
        JSValue e = JS_GetException(own->cx);
        JSValue stack = JS_GetPropertyStr(own->cx, e, "stack");
        const char* s = JS_ToCString(own->cx, e);
        const char* where = JS_IsString(stack) ? JS_ToCString(own->cx, stack) : nullptr;
        own->trouble = std::string("the script ") + file + ": " + (s ? s : "?") + (where ? std::string(" ") + where : "");
        JS_FreeCString(own->cx, s), JS_FreeValue(own->cx, e);
        if (where) JS_FreeCString(own->cx, where);
        JS_FreeValue(own->cx, stack);
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
bool Script::sketch() const { return own->sketch; }
static std::string script_said(JSContext* cx, const char* js) {
    JSValue out = JS_Eval(cx, js, strlen(js), "sketch", JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(out)) {   // (its draw() has failed where it is asked: said here, and it goes on as far as it got)
        JSValue e = JS_GetException(cx), stack = JS_GetPropertyStr(cx, e, "stack");
        const char* m = JS_ToCString(cx, e), *where = JS_IsString(stack) ? JS_ToCString(cx, stack) : nullptr;
        std::fprintf(stderr, "the sketch, asked %.40s: %s\n%s\n", js, m ? m : "?", where ? where : "");
        if (where) JS_FreeCString(cx, where);
        JS_FreeCString(cx, m), JS_FreeValue(cx, stack), JS_FreeValue(cx, e);
    }
    const char* s = JS_IsException(out) ? nullptr : JS_ToCString(cx, out);
    const std::string said = s ? s : "";
    if (s) JS_FreeCString(cx, s);
    JS_FreeValue(cx, out);
    return said;
}
std::string Script::said(const std::string& js) { return own->sketch && ok() ? script_said(own->cx, js.c_str()) : ""; }
std::vector<uint8_t> Script::bytes(const std::string& js) {
    std::vector<uint8_t> out;
    if (!own->sketch || !ok()) return out;
    script_said(own->cx, js.c_str());
    JSValue g = JS_GetGlobalObject(own->cx), a = JS_GetPropertyStr(own->cx, g, "__bytes");
    size_t from = 0, bytes = 0, each = 0, size = 0;
    JSValue buffer = JS_GetTypedArrayBuffer(own->cx, a, &from, &bytes, &each);
    if (const uint8_t* p = JS_IsException(buffer) ? nullptr : JS_GetArrayBuffer(own->cx, &size, buffer)) out.assign(p + from, p + from + bytes);
    JS_FreeValue(own->cx, buffer), JS_FreeValue(own->cx, a), JS_FreeValue(own->cx, g);
    return out;
}
int Script::most() { return own->sketch && ok() ? std::atoi(script_said(own->cx, "__most()").c_str()) : 0; }
std::string Script::shape() { return own->sketch && ok() ? script_said(own->cx, "__shape()") : ""; }
bool Script::flat() { return own->sketch && ok() && script_said(own->cx, "__2d") == "true"; }
int Script::width() { return own->sketch && ok() ? std::atoi(script_said(own->cx, "width").c_str()) : 0; }
int Script::height() { return own->sketch && ok() ? std::atoi(script_said(own->cx, "height").c_str()) : 0; }
std::vector<int> Script::lines() const {
    std::vector<int> all;
    for (const SketchLine& l : own->lines) all.push_back(l.line);
    return all;
}
std::vector<double> Script::paints(int frame, int had, int pic) {
    const std::string js = "__paints(" + std::to_string(frame) + "," + std::to_string(had) + "," + std::to_string(pic) + ")";
    JSValue out = JS_Eval(own->cx, js.c_str(), js.size(), "sketch", JS_EVAL_TYPE_GLOBAL);
    if (JS_IsException(out) && own->trouble.empty()) {   // (its draw() has failed: it stays as it was, and says so once)
        JSValue e = JS_GetException(own->cx);
        const char* s = JS_ToCString(own->cx, e);
        own->trouble = std::string("the sketch ") + own->file + ": " + (s ? s : "?");
        std::fprintf(stderr, "%s\n", own->trouble.c_str());
        JS_FreeCString(own->cx, s), JS_FreeValue(own->cx, e);
    }
    size_t at = 0, bytes = 0, unit = 0, size = 0;
    JSValue buffer = JS_IsException(out) ? JS_EXCEPTION : JS_GetTypedArrayBuffer(own->cx, out, &at, &bytes, &unit);
    std::vector<double> all;
    if (const uint8_t* data = JS_IsException(buffer) ? nullptr : JS_GetArrayBuffer(own->cx, &size, buffer)) {
        const double* from = (const double*)(data + at);
        all.assign(from, from + bytes / sizeof(double));
    }
    JS_FreeValue(own->cx, buffer), JS_FreeValue(own->cx, out);
    return all;
}
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
    read("x", c.x), read("y", c.y), read("z", c.z), read("size", c.size), read("rotate", c.rotate), read("rotate_x", c.rotate_x), read("rotate_y", c.rotate_y), read("colour", c.colour), read("opacity", c.opacity), read("scale_x", c.scale_x), read("scale_y", c.scale_y), read("scale_z", c.scale_z);
    JS_FreeValue(cx, out);
    return c;
}

// anime.js's easings (third_party/anime, MIT) run as they are: one QuickJS context for the whole library, the bundle
// read once from where the library keeps it, and each ease said made once and kept by what it says. irregular() draws
// its points with Math.random: while an ease is made, anime.js's own seeded one stands in, its seed from the words, so
// the same words give the same curve every time.
static const char anime_kept[] = R"(
var __eases = {};
function __ease(said) {
  var made = __eases[said];
  if (made) return made;
  var seed = 0;
  for (var i = 0; i < said.length; i++) seed = (seed * 31 + said.charCodeAt(i)) | 0;
  var r = anime.utils.createSeededRandom(seed), kept = Math.random;
  Math.random = function () { return r(0, 1, 6); };
  try { with (anime.eases) { with (anime) { made = eval('(' + said + ')'); } } } finally { Math.random = kept; }   // (anime's own linear(), steps()... before eases.linear)
  if (made && typeof made.ease === 'function') made = made.ease;   // (a spring: its ease)
  if (typeof made !== 'function') throw new Error(said + ': not an ease of anime.js');
  return __eases[said] = made;
}
function __ease_at(said, u) { return __ease(said)(Math.min(1, Math.max(0, u))); }
)";
double anime_ease(const std::string& bundle, const std::string& said, double u, std::string* trouble) {
    static std::mutex one;
    static JSRuntime* rt = nullptr;
    static JSContext* cx = nullptr;
    static JSValue at = JS_UNDEFINED;
    static std::string why;
    std::lock_guard<std::mutex> held(one);
    if (!rt) {
        rt = JS_NewRuntime(), cx = JS_NewContext(rt);
        std::ifstream in(bundle);
        std::stringstream code;
        code << "globalThis.setImmediate = function () { return 0; }; globalThis.clearImmediate = function () {};\n" << in.rdbuf() << anime_kept;
        const std::string src = code.str();
        JSValue ran = JS_Eval(cx, src.c_str(), src.size(), "anime", JS_EVAL_TYPE_GLOBAL);
        if (!in || JS_IsException(ran)) why = "anime.js is not where it is kept: " + bundle;
        JS_FreeValue(cx, ran);
        JSValue g = JS_GetGlobalObject(cx);
        at = JS_GetPropertyStr(cx, g, "__ease_at");
        JS_FreeValue(cx, g);
    }
    if (!why.empty()) { if (trouble) *trouble = why; return u; }
    JS_UpdateStackTop(rt);   // (asked on whichever thread asks: the hand's on the stage, the window's to draw a curve. QuickJS measures its stack from where it last was told)
    JSValue args[2] = {JS_NewStringLen(cx, said.c_str(), said.size()), JS_NewFloat64(cx, u)};
    JSValue out = JS_Call(cx, at, JS_UNDEFINED, 2, args);
    JS_FreeValue(cx, args[0]);
    double v = u;
    if (JS_IsException(out)) {
        JSValue e = JS_GetException(cx);
        const char* s = JS_ToCString(cx, e);
        if (trouble) *trouble = s ? s : "?";
        JS_FreeCString(cx, s), JS_FreeValue(cx, e);
    } else JS_ToFloat64(cx, &v, out);
    JS_FreeValue(cx, out);
    return v;
}
