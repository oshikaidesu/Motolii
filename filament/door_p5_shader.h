// door_p5_shader.h: part of doc_door.h. A p5.js sketch that paints its stage with shader(): its GLSL as an ISF effect (with
// its vertex shader as ISF's own .vs, isf.cpp), what the sketch tells it each frame (p5_stand.h's __shaded_take) as knobs,
// keyed where it moves, as Hydra's are (hydra_isf.h). A shape's own vertices do not come: what reaches a pixel is what its
// place across a flat rect, plane or quad gives, the same, being straight in the place.
#pragma once
#include <rapidjson/document.h>
#include "door.h"
#include "hydra_isf.h"

// A value that goes as a straight line: two keys say it, and it goes on as far as a video does (HYDRA_STEP apart it stops at 30 s).
inline constexpr double DOOR_LINE_ENDS = 600.0;
inline bool door_line(const std::vector<double>& keys) {
    if (keys.size() < 3) return false;
    const double by = (keys.back() - keys[0]) / (double)(keys.size() - 1);
    for (size_t i = 0; i < keys.size(); ++i) if (std::abs(keys[i] - keys[0] - by * (double)i) > 1e-6 * (1.0 + std::abs(keys.back()))) return false;
    return true;
}
inline double door_line_end(const std::vector<double>& keys) { return keys[0] + (keys.back() - keys[0]) / (double)(keys.size() - 1) * DOOR_LINE_ENDS / HYDRA_STEP; }

// The GLSL ES a browser takes as what isf.cpp's route reads: one thing said on each line, handed on by one word.
inline std::string door_p5_glsl(const std::string& text) {
    std::string t = std::regex_replace(text, std::regex(R"(#version[^\n]*|#extension[^\n]*|\bprecision\s+\w+\s+\w+\s*;)"), "");
    return std::regex_replace(t, std::regex(R"((^|[;\n}])\s*(?:(?:flat|smooth|centroid)\s+)?(?:varying|attribute|in|out)\s+(?:(?:highp|mediump|lowp)\s+)?(\w+)\s+(\w+)\s*(\[[^\]]*\])?\s*;)"), "$1\nvarying $2 $3$4;\n");
}

// json: __shaded_take's. The .fs and the .vs, and what moves; or false and `why`.
inline bool door_p5_shader(const std::string& json, std::string& fs, std::string& vs, std::vector<HydraKnob>& moving, std::string& why) {
    rapidjson::Document all;
    if (all.Parse(json.c_str()).HasParseError() || !all.IsObject()) return why = "this sketch's shader did not come out whole", false;
    if (!all.HasMember("vert") || !all.HasMember("frag") || !all["vert"].IsString() || !all["frag"].IsString()) return why = "this sketch's shaders are in files beside it: bring the .js with them", false;
    std::string vert = all["vert"].GetString(), frag = all["frag"].GetString();
    const rapidjson::Value& told = all["told"];
    const double width = all["width"].GetDouble(), height = all["height"].GetDouble(), density = all["density"].GetDouble();
    const std::string kind = all["kind"].GetString();
    const bool down = door_has(vert, "uProjectionMatrix");   // (p5's own matrices turn y over: a sketch's corners numbers from the top. Without them, from the bottom)
    std::string head, set, inputs;
    static const std::regex output(R"((^|[;\n}])\s*(?:layout\s*\([^)]*\)\s*)?out\s+(?:(?:highp|mediump|lowp)\s+)?vec4\s+(\w+)\s*;)");
    std::smatch said;
    if (std::regex_search(frag, said, output)) head += "#define " + said[2].str() + " gl_FragColor\n", frag = std::regex_replace(frag, output, "$1");
    vert = door_p5_glsl(vert), frag = door_p5_glsl(frag);
    static const std::regex uniform(R"(\buniform\s+(?:(?:highp|mediump|lowp)\s+)?(\w+)\s+([^;]+);)"), each(R"((\w+)\s*(?:\[\s*(\d+)\s*\])?)");
    std::map<std::string, std::pair<std::string, int>> names;   // (each by its name: its type, how many in a row)
    for (std::string* t : {&vert, &frag}) {
        for (std::sregex_iterator u(t->begin(), t->end(), uniform), end; u != end; ++u) {
            const std::string list = (*u)[2];
            for (std::sregex_iterator n(list.begin(), list.end(), each); n != end; ++n) names[(*n)[1].str()] = {(*u)[1].str(), (*n)[2].matched ? std::atoi((*n)[2].str().c_str()) : 0};
        }
        *t = std::regex_replace(*t, uniform, "");
    }
    const auto number = [](double v) { char o[32]; std::snprintf(o, sizeof o, "%.9g", v); return std::string(o); };
    const auto made = [&](const std::string& type, const std::vector<double>& v, size_t from, size_t n) {
        std::string o = type + "(";
        for (size_t i = 0; i < n; ++i) o += (i ? "," : "") + number(from + i < v.size() ? v[from + i] : 0.0);
        return o + ")";
    };
    const auto knob = [&](const std::string& name, const std::string& label, const std::vector<double>& keys) {
        double low = keys[0], high = keys[0];
        for (const double k : keys) low = std::min(low, k), high = std::max(high, k);
        if (door_line(keys)) low = std::min(low, door_line_end(keys)), high = std::max(high, door_line_end(keys));
        inputs += ",{\"NAME\":\"" + name + "\",\"LABEL\":\"" + label + "\",\"TYPE\":\"float\",\"DEFAULT\":" + number(keys[0]) + ",\"MIN\":" + number(std::min(0.0, low * 2)) + ",\"MAX\":" + number(std::max(1.0, high * 2)) + "}";
        if (low != high) moving.push_back({name, keys});
    };
    bool reads = false;
    static const std::set<std::string> taken = {"time", "resolution", "frame", "passindex", "clip_time", "clip_length", "frameRect", "passRect"};   // (isf.cpp's knobs)
    for (const auto& [name, kept] : names) {
        const auto& [type, row] = kept;
        if (type.rfind("sampler", 0) == 0) {   // (the first picture is what it is put on, a film as much as anything; p5's rows start at the top)
            const std::regex read(R"(\b(?:texture2D|texture)\s*\(\s*)" + name + R"(\s*,)");
            for (std::string* t : {&vert, &frag}) *t = std::regex_replace(*t, read, reads ? "door_none(" : "door_read(");
            reads = true;
            continue;
        }
        if (taken.count(name)) head += "#define " + name + " door_own_" + name + "\n";
        head += type + " " + name + (row ? "[" + std::to_string(row) + "]" : "") + ";\n";
        const bool whole = type == "float" || type == "int" || type == "bool";
        const int wide = whole ? 1 : std::isdigit((unsigned char)type.back()) ? type.back() - '0' : 1, size = type.rfind("mat", 0) == 0 ? wide * wide : wide;
        std::vector<std::vector<double>> samples;
        if (told.HasMember(name.c_str()) && told[name.c_str()].IsArray())
            for (const auto& s : told[name.c_str()].GetArray()) {
                std::vector<double> v;
                if (s.IsNumber()) v.push_back(s.GetDouble());
                else if (s.IsArray()) for (const auto& n : s.GetArray()) v.push_back(n.IsNumber() ? n.GetDouble() : 0.0);
                if (!v.empty()) samples.push_back(v);
            }
        if (samples.empty()) {   // (never told: p5's own matrices, as nothing moved them)
            if (type.rfind("mat", 0) == 0 && !row) set += name + "=" + type + "(1.);";
            continue;
        }
        const std::vector<double>& first = samples[0];
        if (row) {   // (a row: as it stood first)
            set += name + "=" + type + "[" + std::to_string(row) + "](";
            for (int i = 0; i < row; ++i) set += (i ? "," : "") + made(type, first, (size_t)(i * size), (size_t)size);
            set += ");";
            continue;
        }
        if (type.rfind("mat", 0) == 0) { set += name + "=" + made(type, first, 0, (size_t)size) + ";"; continue; }
        bool still = true, stage = (type == "vec2" || type == "vec3" || type == "ivec2") && first.size() >= 2 && first[0] > 0;
        const double times = stage ? first[0] / width : 1.0;
        for (const auto& s : samples) {
            still = still && s == first;
            stage = stage && s.size() >= 2 && std::abs(s[0] / width - times) < 1e-6 && std::abs(s[1] / height - times) < 1e-6;
        }
        if (stage) {   // (the sketch's size as it was said, the stage's here)
            const std::string size2 = "door_size()*" + number(times / density);
            set += name + "=" + (type == "vec3" ? "vec3(" + size2 + "," + number(first.size() > 2 ? first[2] : 1.0) + ")" : type + "(" + size2 + ")") + ";";
            continue;
        }
        if ((type == "vec3" || type == "vec4") && still && std::all_of(first.begin(), first.end(), [](double v) { return v >= 0.0 && v <= 1.0; })) {   // (a colour)
            inputs += ",{\"NAME\":\"door_" + name + "\",\"LABEL\":\"" + name + "\",\"TYPE\":\"color\",\"DEFAULT\":[" + number(first[0]) + "," + number(first[1]) + "," + number(first[2]) + "," + number(first.size() > 3 ? first[3] : 1.0) + "]}";
            set += name + "=door_" + name + (type == "vec3" ? ".rgb;" : ";");
            continue;
        }
        std::string parts;
        for (int i = 0; i < size; ++i) {
            const std::string one = "door_" + name + (size > 1 ? "_" + std::to_string(i) : "");
            std::vector<double> keys;
            for (const auto& s : samples) keys.push_back(i < (int)s.size() ? s[(size_t)i] : 0.0);
            knob(one, name + (size > 1 ? " " + std::to_string(i + 1) : ""), keys);
            parts += (i ? "," : "") + one;
        }
        set += name + "=" + type + (type == "bool" ? "(" + parts + ">.5);" : "(" + parts + ");");
    }
    const double lo = kind == "rect" ? 0.0 : kind == "plane" ? -0.5 : -1.0, hi = kind == "rect" ? 1.0 : kind == "plane" ? 0.5 : 1.0;
    std::string start = "door_told();vec2 door_uv=vec2(isf_FragNormCoord.x," + std::string(down ? "1.-isf_FragNormCoord.y" : "isf_FragNormCoord.y") + ");";
    if (door_has(vert, "aTexCoord")) start += "aTexCoord=door_uv;";
    if (door_has(vert, "aPosition")) start += "aPosition=vec3(" + number(lo) + "+door_uv*" + number(hi - lo) + ",0.);";
    if (door_has(vert, "aNormal")) start += "aNormal=vec3(0.,0.,1.);";
    vert = std::regex_replace(vert, std::regex(R"(\bvoid\s+main\s*\(\s*(?:void)?\s*\)\s*\{)"), "void main(){" + start);
    vs = "vec2 door_size(){return RENDERSIZE;}\n" + head + "vec4 door_none(vec2 a){return vec4(0.);}\nvec4 door_none(vec2 a,float b){return vec4(0.);}\n#define door_read(c) texture(inputImage,vec2((c).x,1.-(c).y))\nvoid door_told(){" + set + "}\n" + vert;
    fs = "/*{\"ISFVSN\":\"2\",\"DESCRIPTION\":\"p5.js\",\"CATEGORIES\":[\"p5.js\"],\"INPUTS\":[" + (reads ? std::string("{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}") + inputs : inputs.empty() ? "" : inputs.substr(1)) + "]}*/\n" + frag;
    return true;
}
