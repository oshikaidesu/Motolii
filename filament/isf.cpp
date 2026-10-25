// An ISF effect (Vidvox's Interactive Shader Format: a GLSL fragment shader with a JSON head of its knobs) dropped in an
// effects folder works as a .wgsl does: made one in vgpu's manner (screen.h) inside Motolii, by the parts Filament's
// build already has (glslang to SPIR-V, Tint from SPIR-V to WGSL, as Dawn itself reads SPIR-V), its head read by
// rapidjson (as Filament's assimp has it). The shader's own lines are kept; around them ISF's words (IMG_NORM_PIXEL, IMG_THIS_PIXEL,
// isf_FragNormCoord, TIME, RENDERSIZE...) are said in plain GLSL, each picture split into a texture and a sampler, the
// knobs gathered into one struct and written in the head (script.h: // --track@...), its passes too (// --pass@...).
// Made once into ~/Library/Caches/Motolii/isf/<hash>.wgsl, named by the file as it is. Its other images are
// other things' pictures (screen.h others), its audio inputs other things' sound; its own vertex shader is worked out at each pixel.
#include "isf.h"
#include "three_isf.h"
#include <sys/stat.h>
#include <cstdio>
#include <filesystem>
#include <fstream>
#include <map>
#include <mutex>
#include <regex>
#include <sstream>
#include <vector>
#include "rapidjson/document.h"
#include <sqlite3.h>
#include "glslang/Public/ShaderLang.h"
#include "SPIRV/GlslangToSpv.h"
#include "sca/builtinResource.h"
#include "src/tint/lang/core/ir/module.h"
#include "src/tint/lang/wgsl/program/program.h"
#include "src/tint/lang/spirv/reader/reader.h"
#include "src/tint/lang/wgsl/writer/writer.h"

static std::string isf_wgsl(const std::string& text, const std::string& name, std::string& why, std::string vertex = "") {
    const size_t open = text.find("/*"), close = text.find("*/");
    rapidjson::Document head;
    if (open == std::string::npos || close == std::string::npos || head.Parse(text.c_str() + open + 2, close - open - 2).HasParseError() || !head.IsObject())
        return why = "no ISF head (a JSON comment at the top)", "";
    std::string body = text.substr(close + 2);
    const auto str = [](const rapidjson::Value& o, const char* k, std::string fallback = "") { return o.IsObject() && o.HasMember(k) && o[k].IsString() ? std::string(o[k].GetString()) : fallback; };
    const auto num = [](const rapidjson::Value& v, double fallback) { return v.IsNumber() ? v.GetDouble() : v.IsBool() ? (double)v.GetBool() : fallback; };

    // (passes: each drawn in turn, into a picture of its own (kept from frame to frame when PERSISTENT, finer when FLOAT)
    //  or, the last, out; a target's size a share of the stage's or so many points)
    const auto size = [&](const rapidjson::Value& p, const char* k, const char* side, std::string& said) {
        std::string e = p.HasMember(k) ? (p[k].IsString() ? p[k].GetString() : std::to_string((int)num(p[k], 0))) : std::string("$") + side;
        e.erase(std::remove(e.begin(), e.end(), ' '), e.end());
        std::smatch m;
        char out[32];
        if (std::regex_match(e, m, std::regex(std::string(R"((?:floor\()?\$)") + side + R"((?:/([\d.]+))?(?:\*([\d.]+))?\)?)")))
            return std::snprintf(out, sizeof out, "s%g", (m[1].matched ? 1 / std::stod(m[1]) : 1) * (m[2].matched ? std::stod(m[2]) : 1)), said = out, true;
        if (std::regex_match(e, std::regex(R"([\d.]+)"))) return said = "a" + std::to_string((int)std::stod(e)), true;
        // (any other: the spec's expression worked out by SQLite, $WIDTH and $HEIGHT bound as its parameters are; at 1000
        //  points a share of the stage's when it reads the stage's size, so many points when not, none when it reads a knob)
        sqlite3* db = nullptr;
        sqlite3_stmt* q = nullptr;
        double v = -1;
        if (sqlite3_open(":memory:", &db) == SQLITE_OK && sqlite3_prepare_v2(db, ("SELECT " + e).c_str(), -1, &q, nullptr) == SQLITE_OK) {
            bool known = true;
            for (int i = 1; i <= sqlite3_bind_parameter_count(q); ++i) {
                const std::string n = sqlite3_bind_parameter_name(q, i) ? sqlite3_bind_parameter_name(q, i) : "";
                if (n == "$WIDTH" || n == "$HEIGHT") sqlite3_bind_double(q, i, 1000);
                else known = false;
            }
            if (known && sqlite3_step(q) == SQLITE_ROW && sqlite3_column_type(q, 0) != SQLITE_NULL) v = sqlite3_column_double(q, 0);
        }
        sqlite3_finalize(q);
        sqlite3_close(db);
        if (v <= 0) return false;
        if (e.find('$') != std::string::npos) return std::snprintf(out, sizeof out, "s%g", v / 1000), said = out, true;
        return said = "a" + std::to_string((int)v), true;
    };
    std::vector<std::string> targets, passLines;
    if (head.HasMember("PASSES") && head["PASSES"].IsArray())
        for (const auto& p : head["PASSES"].GetArray()) {
            const std::string t = str(p, "TARGET");
            std::string w, h;
            if (!size(p, "WIDTH", "WIDTH", w) || !size(p, "HEIGHT", "HEIGHT", h)) return why = "a pass's size it cannot say", "";
            if (!t.empty() && std::find(targets.begin(), targets.end(), t) == targets.end()) targets.push_back(t);
            const bool kept = p.HasMember("PERSISTENT") && num(p["PERSISTENT"], 0), fine = p.HasMember("FLOAT") && num(p["FLOAT"], 0);
            passLines.push_back("// --pass@" + t + ":" + (kept ? "1" : "0") + "," + (fine ? "1" : "0") + "," + w + "," + h);
        }

    const rapidjson::Value none(rapidjson::kArrayType);
    const rapidjson::Value& inputs = head.HasMember("INPUTS") && head["INPUTS"].IsArray() ? head["INPUTS"] : none;
    std::vector<std::string> pictures;
    for (const auto& i : inputs.GetArray()) if (str(i, "TYPE") == "image") pictures.push_back(str(i, "NAME"));
    // (none: a generator, ISF's "source" (ISF_Spec: sources, filters, transitions): on a thing it paints only where the thing
    //  is, After Effects' Preserve Underlying Transparency; the thing's own picture is still at binding 0, the door's form)
    const std::string picture = pictures.empty() ? "inputImage" : std::find(pictures.begin(), pictures.end(), "inputImage") != pictures.end() ? "inputImage" : pictures[0];
    std::vector<std::string> others;   // (its other images: each another thing's picture, chosen on its card, bound after the passes' pictures)
    for (const std::string& other : pictures) if (other != picture) others.push_back(other);
    for (const auto& i : inputs.GetArray()) if (str(i, "TYPE") == "audio" || str(i, "TYPE") == "audioFFT") others.push_back(str(i, "NAME"));   // (its sound: an image as the spec says, bound after its other images)

    const std::map<std::string, std::string> kinds = {{"float", "float"}, {"bool", "int"}, {"long", "int"}, {"event", "int"}, {"point2D", "vec2"}, {"color", "vec4"}};
    std::string glsl = "#version 450\n";
    // (each picture a texture and a sampler named after it, read through ISF's own words by pasting the name: a name
    //  the shader also gives a variable of its own is left alone; only the picture in is also its own name outright)
    std::vector<std::string> read = {picture};
    read.insert(read.end(), targets.begin(), targets.end());
    for (size_t k = 0; k < read.size(); ++k)
        glsl += "layout(set = 0, binding = " + std::to_string(2 * k) + ") uniform texture2D isf_tex_" + read[k] + ";\nlayout(set = 0, binding = " + std::to_string(2 * k + 1) + ") uniform sampler isf_smp_" + read[k] + ";\n";
    for (size_t k = 0; k < others.size(); ++k)
        glsl += "layout(set = 0, binding = " + std::to_string(2 * (read.size() + k)) + ") uniform texture2D isf_tex_" + others[k] + ";\nlayout(set = 0, binding = " + std::to_string(2 * (read.size() + k) + 1) + ") uniform sampler isf_smp_" + others[k] + ";\n";
    glsl += "layout(set = 0, binding = " + std::to_string(2 * (read.size() + others.size())) + ") uniform Knobs {\n  float time;\n  vec2 resolution;\n  float frame;\n  float passindex;\n";
    std::vector<std::string> switches;   // (an on/off knob: an int, as WGSL keeps no bool in a uniform, read as a bool where the shader names it, not where a colour's .r is)
    std::string lines;
    for (const auto& k : inputs.GetArray()) {
        const std::string t = str(k, "TYPE"), n = str(k, "NAME");
        if (!kinds.count(t)) continue;
        const bool onoff = t == "bool" || t == "event";
        glsl += "  " + kinds.at(t) + " " + n + (onoff ? "_on" : "") + ";\n";
        if (onoff) switches.push_back(n);
        std::string label = str(k, "LABEL", n);
        std::replace(label.begin(), label.end(), ',', ' ');
        const auto at = [&](const char* key, int i, double fallback) {
            if (!k.HasMember(key)) return fallback;
            const rapidjson::Value& v = k[key];
            return v.IsArray() ? (i < (int)v.Size() ? num(v[(rapidjson::SizeType)i], fallback) : fallback) : num(v, fallback);
        };
        char line[256];
        if (onoff) std::snprintf(line, sizeof line, "// --check@%s_on:%s,%d\n", n.c_str(), label.c_str(), (int)at("DEFAULT", 0, 0)), lines += line;
        else if (t == "float" || t == "long")
        {   // (a long's choices are its VALUES (ISF_Spec), its range from the least to the most of them)
            double lo = at("MIN", 0, 0), hi = at("MAX", 0, 1);
            if (t == "long" && k.HasMember("VALUES") && k["VALUES"].IsArray() && !k["VALUES"].Empty()) {
                lo = hi = num(k["VALUES"][0], 0);
                for (const auto& v : k["VALUES"].GetArray()) lo = std::min(lo, num(v, lo)), hi = std::max(hi, num(v, hi));
            }
            std::snprintf(line, sizeof line, "// --track@%s:%s,%g,%g,%g,%g\n", n.c_str(), label.c_str(), lo, hi, at("DEFAULT", 0, 0), t == "long" ? 1.0 : 0.001), lines += line;
        }
        else
            for (int i = 0; i < (t == "point2D" ? 2 : 4); ++i)
                std::snprintf(line, sizeof line, "// --track@%s_%c:%s %c,%g,%g,%g,0.001\n", n.c_str(), "xyzw"[i], label.c_str(), (t == "point2D" ? "xyzw" : "rgba")[i], at("MIN", i, 0), at("MAX", i, 1), at("DEFAULT", i, 0)), lines += line;
    }
    glsl += "};\n";
    for (const std::string& n : switches)
        for (std::string* t : {&body, &vertex}) *t = std::regex_replace(*t, std::regex("(^|[^.\\w])" + n + "\\b"), "$1(" + n + "_on != 0)");
    // (ISF's place runs up from the bottom left, as WebGL's does (ISF_Spec); WebGPU's runs down from the top left: the place,
    //  the pixel's own and every picture read turned over alike, so a picture read where it is stays as it was)
    glsl += "layout(location = 0) in vec2 isf_FragNormCoord_down;\nlayout(location = 0) out vec4 isf_FragColor;\n";
    glsl += "#define isf_FragNormCoord vec2(isf_FragNormCoord_down.x, 1.0 - isf_FragNormCoord_down.y)\n#define isf_up(c) vec2((c).x, 1.0 - (c).y)\n";
    for (const std::string& n : others) {
        std::string label = n;
        std::string sound;   // (an audio input: which, and how many it wants (MAX; none said, as many as are read))
        for (const auto& i : inputs.GetArray())
            if (str(i, "NAME") == n) label = str(i, "LABEL", n), sound = str(i, "TYPE") == "audioFFT" ? ",fft," : str(i, "TYPE") == "audio" ? ",wave," : "", sound += sound.empty() ? "" : std::to_string((int)(i.HasMember("MAX") ? num(i["MAX"], 0) : 0));
        std::replace(label.begin(), label.end(), ',', ' ');
        lines += (sound.empty() ? "// --picture@" : "// --sound@") + n + ":" + label + sound + "\n";
        read.push_back(n);   // (read through ISF's words as the others are)
    }
    for (const std::string& n : others) glsl += "#define " + n + " sampler2D(isf_tex_" + n + ", isf_smp_" + n + ")\n";
    glsl += "#define " + picture + " sampler2D(isf_tex_" + picture + ", isf_smp_" + picture + ")\n";
    glsl += "#define TIME time\n#define RENDERSIZE resolution\n#define TIMEDELTA (1.0 / 60.0)\n#define FRAMEINDEX int(frame)\n#define PASSINDEX int(passindex)\n#define DATE vec4(0.0, 0.0, 0.0, TIME)\n";   // (ISF_Spec: DATE.w the seconds of the day; here the document's own time, a clock that runs as the video does (Alight Motion's Timecode))
    for (const std::string& n : read) glsl += "#define isf_img_" + n + " sampler2D(isf_tex_" + n + ", isf_smp_" + n + ")\n#define _" + n + "_imgRect vec4(0.0, 0.0, RENDERSIZE)\n#define _" + n + "_imgSize RENDERSIZE\n#define _" + n + "_flip false\n";   // (VVISF's own words for a picture's place and size: each as large as the stage here)
    glsl += "#define vv_FragNormCoord isf_FragNormCoord\n";   // (ISF 1's name for it, as VVISF still says)
    glsl += "#define IMG_NORM_PIXEL(i, c) texture(i, c)\n#define IMG_PIXEL(i, c) texture(i, (c) / RENDERSIZE)\n";
    glsl += "#define IMG_THIS_NORM_PIXEL(i) texture(i, isf_FragNormCoord)\n#define IMG_THIS_PIXEL(i) texture(i, isf_FragNormCoord)\n#define IMG_SIZE(i) RENDERSIZE\n";
    body = std::regex_replace(body, std::regex(R"((\bIMG_(?:NORM_PIXEL|PIXEL|THIS_PIXEL|THIS_NORM_PIXEL|SIZE)\s*\(\s*)(\w+))"), "$1isf_img_$2");
    glsl += "#define gl_FragColor isf_FragColor\n#define texture2D texture\n#define varying //\n";
    glsl += "#define texture(i, c) textureLod(i, isf_up(c), 0.0)\n#define gl_FragCoord vec4(gl_FragCoord.x, RENDERSIZE.y - gl_FragCoord.y, gl_FragCoord.zw)\n";   // (WGSL samples with derivatives only where every pixel goes the same way: these read the top level anywhere)
    if (!vertex.empty()) {   // (its own vertex shader: what it hands on (neighbours' places, mostly) worked out at each pixel instead, before the pixel's own work: the same, being straight in the place)
        const std::regex said(R"(^[ \t]*(?:varying|out|in)[ \t]+(\w+)[ \t]+(\w+)([ \t]*\[[^\]]*\])?[ \t]*;)", std::regex::multiline);
        std::map<std::string, std::string> handed;   // (each by its name: its type, and how many when it is a row of them)
        for (const std::string* t : {&vertex, &body})
            for (std::sregex_iterator m(t->begin(), t->end(), said), end; m != end; ++m) handed[(*m)[2].str() + (*m)[3].str()] = (*m)[1];
        vertex = std::regex_replace(std::regex_replace(vertex, said, ""), std::regex(R"(\bvoid\s+main\s*\()"), "void isf_vs_main(");
        body = std::regex_replace(std::regex_replace(body, said, ""), std::regex(R"(\bvoid\s+main\s*\()"), "void isf_fs_main(");
        glsl += "#define isf_vertShaderInit()\nvec4 isf_vs_position;\n#define gl_Position isf_vs_position\n";
        for (const auto& [n, type] : handed) glsl += type + " " + n + ";\n";
        vertex = std::regex_replace(vertex, std::regex(R"(\b(IMG_(?:NORM_PIXEL|PIXEL|THIS_PIXEL|THIS_NORM_PIXEL|SIZE)\s*\(\s*)(\w+))"), "$1isf_img_$2");
        body = std::regex_replace(vertex, std::regex(R"(^[ \t]*#version.*$)"), "") + "\n" + body + "\nvoid main() { isf_vs_main(); isf_fs_main(); }\n";
    }
    if (pictures.empty())   // (a generator: its last pass as clear as the thing under it is, Preserve Underlying Transparency)
        body = std::regex_replace(body, std::regex(R"(\bvoid\s+main\s*\()"), "void isf_source_main(") + "\nvoid main() { isf_source_main(); if (PASSINDEX == " + std::to_string(std::max<int>((int)passLines.size() - 1, 0)) + ") isf_FragColor.a *= texture(" + picture + ", isf_FragNormCoord).a; }\n";
    glsl += "#line 1\n" + std::regex_replace(body, std::regex(R"(^[ \t]*#version.*$)"), "");

    static std::once_flag once;
    std::call_once(once, [] { glslang::InitializeProcess(); });
    glslang::TShader sh(EShLangFragment);
    const char* src = glsl.c_str();
    sh.setStrings(&src, 1);
    sh.setPreamble("#define GL_ES 1\n");   // (WebGPU is GLSL ES's side, as WebGL is, where GL_ES is said by the compiler itself: an ISF file's own #ifndef GL_ES parts left out, as Vidvox's ISF-JS-Renderer runs them)
    sh.setEnvInput(glslang::EShSourceGlsl, EShLangFragment, glslang::EShClientVulkan, 100);
    sh.setEnvClient(glslang::EShClientVulkan, glslang::EShTargetVulkan_1_1);
    sh.setEnvTarget(glslang::EShTargetSpv, glslang::EShTargetSpv_1_3);
    const EShMessages msg = EShMessages(EShMsgSpvRules | EShMsgVulkanRules);
    if (!sh.parse(&DefaultTBuiltInResource, 450, false, msg)) return why = std::string("GLSL: ") + sh.getInfoLog(), "";
    glslang::TProgram program;
    program.addShader(&sh);
    if (!program.link(msg)) return why = std::string("GLSL: ") + program.getInfoLog(), "";
    std::vector<uint32_t> spv;
    glslang::GlslangToSpv(*program.getIntermediate(EShLangFragment), spv);
    auto ir = tint::spirv::reader::ReadIR(spv);
    if (ir != tint::Success) return why = "SPIR-V: " + ir.Failure().reason, "";
    tint::wgsl::writer::Options o;
    o.allow_non_uniform_derivatives = true;
    auto made = tint::wgsl::writer::ProgramFromIR(ir.Get(), o);
    if (made != tint::Success) return why = "WGSL: " + made.Failure().reason, "";
    auto wgsl = tint::wgsl::writer::Generate(made.Get(), o);
    if (wgsl != tint::Success) return why = "WGSL: " + wgsl.Failure().reason, "";

    const std::string credit = str(head, "CREDIT");
    std::string top = "// " + name + ": " + name + ".fs, ISF" + (credit.empty() ? "" : ", " + credit) + ", made WGSL in Motolii (glslang, Tint)\n";
    if (head.HasMember("CATEGORIES") && head["CATEGORIES"].IsArray() && !head["CATEGORIES"].Empty() && head["CATEGORIES"][0].IsString()) {   // (its shelf: the shelf's own words where ISF's say the same)
        std::string all, shelf = head["CATEGORIES"][0].GetString();
        for (const auto& c : head["CATEGORIES"].GetArray()) if (c.IsString()) all += std::string(c.GetString()) + " ";
        for (const char* own : {"Glitch", "Distort", "Color", "Film", "Light", "Time", "Stylize"}) if (all.find(own) != std::string::npos) { shelf = own; break; }
        top += "// --label:" + shelf + "\n";
    }
    for (const std::string& p : passLines) lines += p + "\n";
    return top + lines + "\n" + wgsl->wgsl;
}

std::string isf_file(const std::string& file) {
    if (std::filesystem::path(file).extension() != ".fs" && !three_is(file)) return file;   // (a three.js effect: made ISF first, three_isf.cpp)
    struct stat st {};
    if (stat(file.c_str(), &st) != 0) return "";
    const std::string key = "isf 8|" + file + "|" + std::to_string(st.st_mtimespec.tv_sec) + "." + std::to_string(st.st_mtimespec.tv_nsec) + "|" + std::to_string(st.st_size);
    static std::mutex lock;
    static std::map<std::string, std::string> made;   // (what each file as it is became: "" could not be)
    std::lock_guard<std::mutex> hold(lock);
    if (auto m = made.find(key); m != made.end()) return m->second;
    uint64_t hash = 1469598103934665603ull;
    for (unsigned char c : key) hash = (hash ^ c) * 1099511628211ull;
    const char* home = std::getenv("HOME");
    const std::filesystem::path dir = std::filesystem::path(home ? home : "/tmp") / "Library/Caches/Motolii/isf";
    char name[40];
    std::snprintf(name, sizeof name, "%016llx.wgsl", (unsigned long long)hash);
    const std::string out = (dir / name).string();
    std::error_code e;
    if (std::filesystem::exists(out, e)) return made[key] = out;
    std::ifstream in(file);
    std::stringstream text;
    text << in.rdbuf();
    std::string why;
    if (std::filesystem::path(file).extension() == ".js") {
        const std::string isf = three_isf(file, why);
        if (isf.empty()) {
            std::fprintf(stderr, "the three.js effect %s is not taken: %s\n", file.c_str(), why.substr(0, 600).c_str());
            return made[key] = "";
        }
        text.str(isf);
    }
    const std::string stem = std::filesystem::path(file).stem().string();
    std::stringstream vertex;   // (its own vertex shader, if it has one beside it)
    if (std::ifstream vs(std::filesystem::path(file).replace_extension(".vs")); vs) vertex << vs.rdbuf();
    const std::string wgsl = isf_wgsl(text.str(), stem, why, vertex.str());
    if (wgsl.empty()) {
        std::fprintf(stderr, "the ISF effect %s is not taken: %s\n", file.c_str(), why.substr(0, 600).c_str());
        return made[key] = "";
    }
    std::filesystem::create_directories(dir, e);
    std::ofstream(out + ".part") << wgsl;
    std::filesystem::rename(out + ".part", out, e);
    return made[key] = out;
}
