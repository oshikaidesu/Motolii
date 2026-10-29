// See aviutl.h. The HLSL read by glslang as glslangValidator -D does (EShSourceHlsl, the entry its registered name,
// registers moved apart by setShiftBinding: t from 0, s from 16, b from 32), SPIR-V made WGSL by Tint (isf.cpp's way),
// the cbuffer's places taken from glslang's reflection (HLSL's packing) and from Tint's own layout of the struct (WGSL's).
#include "aviutl.h"
#include <cstring>
#include <filesystem>
#include <fstream>
#include <map>
#include <mutex>
#include <regex>
#include <sstream>
#include "glslang/Public/ShaderLang.h"
#include "SPIRV/GlslangToSpv.h"
#include "sca/builtinResource.h"
#include "src/tint/lang/core/ir/module.h"
#include "src/tint/lang/core/type/array.h"
#include "src/tint/lang/core/type/matrix.h"
#include "src/tint/lang/core/type/struct.h"
#include "src/tint/lang/core/type/vector.h"
#include "src/tint/lang/wgsl/ast/module.h"
#include "src/tint/lang/wgsl/program/program.h"
#include "src/tint/lang/wgsl/sem/variable.h"
#include "src/tint/lang/spirv/reader/reader.h"
#include "src/tint/lang/wgsl/writer/writer.h"

bool aviutl_is(const std::string& file) {
    for (const char* e : {".anm2", ".obj2"})
        if (const size_t at = file.rfind(e); at != std::string::npos && (at + 5 == file.size() || file[at + 5] == '#')) return true;
    return false;
}

bool aviutl_text(const std::string& file, std::string& own, std::string& all, std::string& why) {
    const size_t hash = file.find('#', file.rfind('/') + 1);
    std::ifstream in(aviutl_path(file));
    if (!in) return why = "there is no such file: " + file, false;
    std::stringstream text;
    text << in.rdbuf();
    own = all = text.str();
    if (std::filesystem::path(aviutl_path(file)).filename().string()[0] != '@') return true;
    // (an @ file: its scripts each from a line "@name" (lua.txt); none named, the first)
    const std::string name = hash == std::string::npos ? "" : file.substr(hash + 1);
    std::smatch m;
    if (!std::regex_search(all, m, std::regex(name.empty() ? std::string("(^|\\n)@[^\\r\\n]*") : "(^|\\n)@" + std::regex_replace(name, std::regex(R"([.^$|()\[\]{}*+?\\])"), "\\$&") + "[ \\t]*\\r?\\n")))
        return why = "no script @" + name + " in " + file, false;
    const size_t from = m.position() + m.length(), next = all.find("\n@", from);
    own = all.substr(from, next == std::string::npos ? std::string::npos : next - from);
    return true;
}

std::vector<AviutlVar> aviutl_vars(const std::string& text) {
    std::vector<AviutlVar> vars;
    std::istringstream lines(text);
    const std::regex said(R"(^--(track|checksection|check|color|select|value|file|folder|font|figure|text|string)(?:@(\w+)|(\d)):([^\r]*))");
    for (std::string line; std::getline(lines, line);) {
        std::smatch m;
        if (!std::regex_search(line, m, said)) continue;
        AviutlVar v;
        v.kind = m[1], v.var = m[2].matched ? m[2].str() : v.kind + m[3].str();   // (--track0: is obj.track0)
        if (v.kind == "checksection") v.kind = "check";   // (a check that opens the knobs under it, its third its own folding: as a check)
        std::vector<std::string> parts;
        std::stringstream rest(m[4].str());
        for (std::string one; std::getline(rest, one, ',');) parts.push_back(std::regex_replace(one, std::regex(R"(^\s+|\s+$)"), ""));
        if (parts.empty()) continue;
        if (v.kind == "track" && parts.size() >= 4) v.usual = std::atof(parts[3].c_str());
        else if (v.kind == "check") v.truth = parts.size() > 1 && (parts[1] == "true" || parts[1] == "false"), v.usual = parts.size() > 1 && (parts[1] == "true" || std::atof(parts[1].c_str()) != 0);
        else if (v.kind == "color") v.unset = parts.size() < 2 || parts[1] == "nil", v.usual = v.unset ? 0xffffff : std::strtod(parts[1].c_str(), nullptr);
        else if (v.kind == "select") {   // (label=as it comes, choice=value, ...)
            const size_t eq = parts[0].find('=');
            const double usual = eq == std::string::npos ? 0 : std::atof(parts[0].c_str() + eq + 1);
            for (size_t i = 1; i < parts.size(); ++i) {
                const size_t e = parts[i].find('=');
                v.values.push_back(e == std::string::npos ? (double)(i - 1) : std::atof(parts[i].c_str() + e + 1));
                if (v.values.back() == usual) v.usual = (double)(i - 1);
            }
        } else if (v.kind == "value") {   // (a Lua value as it is: a number, a string, a table)
            const size_t c = m[4].str().find(',');
            v.expr = c == std::string::npos ? "nil" : m[4].str().substr(c + 1);
        } else {   // (a file's, a folder's, a font's, a figure's name, a text: words, "" for no file chosen)
            const size_t c = m[4].str().find(',');
            v.kind = "value", v.expr = "[==[" + (c == std::string::npos ? "" : m[4].str().substr(c + 1)) + "]==]";
        }
        vars.push_back(v);
    }
    return vars;
}

std::vector<ScriptKnob> aviutl_knobs(const std::string& file) {
    std::string own, all, why;
    std::vector<ScriptKnob> knobs;
    if (!aviutl_text(file, own, all, why)) return knobs;
    std::istringstream lines(own);
    const std::regex said(R"(^--(track|check|color|select)(?:@(\w+)|(\d)):([^,\r]*)(?:,([^\r]*))?)");
    for (std::string line; std::getline(lines, line);) {
        std::smatch m;
        if (!std::regex_search(line, m, said)) continue;
        const AviutlVar v = aviutl_vars(line)[0];
        std::string label = m[4].str();
        const size_t eq = label.find('=');
        if (eq != std::string::npos) label.resize(eq);
        ScriptKnob k;
        k.name = v.var, k.label = label, k.kind = v.kind == "check" ? "check" : "track";
        if (v.kind == "track") {
            std::vector<double> n;
            std::stringstream rest(m[5].str());
            for (std::string one; std::getline(rest, one, ',');) n.push_back(std::atof(one.c_str()));
            if (n.size() < 3) continue;
            k.low = n[0], k.high = n[1], k.usual = n[2], k.step = n.size() > 3 ? n[3] : 1;
        } else if (v.kind == "check") k.high = 1, k.usual = v.usual, k.step = 1;
        else if (v.kind == "select") k.high = std::max<double>(0, (double)v.values.size() - 1), k.usual = v.usual, k.step = 1;
        else {
            if (v.unset) knobs.push_back({v.var + "_on", label, "check", 0, 1, 0, 1, ""});
            for (int c = 0; c < 3; ++c) knobs.push_back({v.var + "_" + "rgb"[c], label + " " + "RGB"[c], "track", 0, 255, (double)(((int)v.usual >> (16 - 8 * c)) & 255), 1, ""});
            continue;
        }
        knobs.push_back(k);
    }
    return knobs;
}

// (each float of the constants, HLSL's place to WGSL's: a scalar or a vector, a matrix's columns or an array's elements
//  each in a register of its own on HLSL's side (16 bytes apart), at Tint's stride on WGSL's; a struct's members where
//  glslang's SPIR-V says (its Offset decorations, by the struct's name))
using AviutlOffsets = std::map<std::string, std::vector<uint32_t>>;
static bool aviutl_place(const tint::core::type::Type* t, uint32_t from, uint32_t to, const AviutlOffsets& hlsl, AviutlShader& sh) {
    using namespace tint::core::type;
    if (t->Is<Vector>() || t->IsScalar()) {
        sh.from.push_back(from), sh.to.push_back(to), sh.count.push_back(t->Is<Vector>() ? t->As<Vector>()->Width() : 1);
        return true;
    }
    if (const auto* m = t->As<Matrix>()) {
        for (uint32_t c = 0; c < m->Columns(); ++c) aviutl_place(m->ColumnType(), from + 16 * c, to + m->ColumnStride() * c, hlsl, sh);
        return true;
    }
    if (const auto* a = t->As<Array>(); a && a->ConstantCount()) {
        for (uint32_t i = 0; i < *a->ConstantCount(); ++i)
            if (!aviutl_place(a->ElemType(), from + 16 * i, to + a->ImplicitStride() * i, hlsl, sh)) return false;
        return true;
    }
    if (const auto* st = t->As<Struct>()) {
        const std::string name = st->Name().Name();
        if (name.rfind("tint_", 0) == 0 && st->Members().Length() == 1)   // (Tint's own wrapping of an element padded to a register)
            return aviutl_place(st->Members()[0]->Type(), from, to + st->Members()[0]->Offset(), hlsl, sh);
        auto at = hlsl.find(name);
        if (at == hlsl.end()) at = hlsl.find(std::regex_replace(name, std::regex("(_\\d+)+$"), ""));   // (Tint's name made apart from another's)
        if (at == hlsl.end() || at->second.size() != st->Members().Length()) return false;
        for (const auto* m : st->Members())
            if (!aviutl_place(m->Type(), from + at->second[m->Index()], to + m->Offset(), hlsl, sh)) return false;
        return true;
    }
    return false;
}

static AviutlShader aviutl_make(const std::string& hlsl, const std::string& entry, bool clip) {
    AviutlShader sh;
    static std::once_flag once;
    std::call_once(once, [] { glslang::InitializeProcess(); });
    glslang::TShader s(EShLangFragment);
    const char* src = hlsl.c_str();
    s.setStrings(&src, 1);
    s.setEntryPoint(entry.c_str());
    s.setSourceEntryPoint(entry.c_str());
    s.setEnvInput(glslang::EShSourceHlsl, EShLangFragment, glslang::EShClientVulkan, 100);
    s.setEnvClient(glslang::EShClientVulkan, glslang::EShTargetVulkan_1_1);
    s.setEnvTarget(glslang::EShTargetSpv, glslang::EShTargetSpv_1_3);
    s.setShiftBinding(glslang::EResTexture, 0), s.setShiftBinding(glslang::EResSampler, 16), s.setShiftBinding(glslang::EResUbo, 32);
    s.setAutoMapBindings(true), s.setAutoMapLocations(true);
    const EShMessages msg = EShMessages(EShMsgSpvRules | EShMsgVulkanRules | EShMsgReadHlsl);
    if (!s.parse(&DefaultTBuiltInResource, 100, false, msg)) return sh.why = std::string("HLSL: ") + s.getInfoLog(), sh;
    glslang::TProgram program;
    program.addShader(&s);
    if (!program.link(msg) || !program.mapIO()) return sh.why = std::string("HLSL: ") + program.getInfoLog(), sh;
    std::vector<uint32_t> spv;
    glslang::GlslangToSpv(*program.getIntermediate(EShLangFragment), spv);
    std::map<uint32_t, std::string> names;   // (SPIR-V read for each struct's member offsets: OpName 5, OpMemberDecorate 72 Offset 35; an empty struct (OpTypeStruct 30 of none) Tint cannot take)
    std::map<uint32_t, std::map<uint32_t, uint32_t>> offsets;
    for (size_t i = 5; i < spv.size() && spv[i] >> 16;) {
        const uint32_t n = spv[i] >> 16, op = spv[i] & 0xffff;
        if (i + n > spv.size()) break;
        if (op == 5 && n > 2) names[spv[i + 1]] = std::string((const char*)&spv[i + 2], strnlen((const char*)&spv[i + 2], 4 * (n - 2)));
        if (op == 72 && n > 4 && spv[i + 3] == 35) offsets[spv[i + 1]][spv[i + 2]] = spv[i + 4];
        if (op == 30 && n == 2) return sh.why = "not yet: an empty struct", sh;
        i += n;
    }
    AviutlOffsets hlslAt;
    for (const auto& [id, at] : offsets) for (const auto& [k, o] : at) hlslAt[names[id]].resize(k + 1), hlslAt[names[id]][k] = o;
    auto ir = tint::spirv::reader::ReadIR(spv);
    if (ir != tint::Success) return sh.why = "SPIR-V: " + ir.Failure().reason, sh;
    tint::wgsl::writer::Options o;
    o.allow_non_uniform_derivatives = true;
    auto made = tint::wgsl::writer::ProgramFromIR(ir.Get(), o);
    if (made != tint::Success) return sh.why = "WGSL: " + made.Failure().reason, sh;
    const tint::Program& p = made.Get();
    for (const auto* g : p.AST().GlobalVariables()) {   // (the constants' struct, b0, as Tint lays it)
        const auto* v = p.Sem().Get<tint::sem::GlobalVariable>(g);
        if (!v || v->AddressSpace() != tint::core::AddressSpace::kUniform || !v->Attributes().binding_point || v->Attributes().binding_point->binding != 32) continue;
        const auto* st = v->Type()->UnwrapRef()->As<tint::core::type::Struct>();
        sh.bytes = st ? st->Size() : 0;
        if (!st || !aviutl_place(st, 0, 0, hlslAt, sh)) return sh.why = "not yet: constants of their kind", sh;
    }
    auto wgsl = tint::wgsl::writer::Generate(p, o);
    if (wgsl != tint::Success) return sh.why = "WGSL: " + wgsl.Failure().reason, sh;
    // (another cbuffer (a global not static: $Global) is bound to nothing in AviUtl2, which reads zero in D3D11: a private
    //  variable, which WGSL begins at zero)
    std::string code = std::regex_replace(wgsl->wgsl, std::regex(R"(@group\(0u?\) @binding\((?!32u?\))\d+u?\) var<uniform>)"), "var<private>");
    // (the door's bindings: picture k's texture at 2k, its sampler at 2k+1, the constants after all the pictures)
    const std::regex bound(R"(@group\(0u?\) @binding\((\d+)u?\) var(<uniform>)?)");
    int most = 0;
    for (std::sregex_iterator m(code.begin(), code.end(), bound), end; m != end; ++m)
        if (!(*m)[2].matched) most = std::max(most, std::stoi((*m)[1]) % 16 + 1);
    sh.pictures = std::max(1, most);
    std::string out;
    auto at = code.cbegin();
    for (std::sregex_iterator m(code.begin(), code.end(), bound), end; m != end; ++m) {
        const int b = std::stoi((*m)[1]);
        if ((*m)[2].matched ? b != 32 : b >= 32) return sh.why = "not yet: a resource of its kind (binding " + std::to_string(b) + ")", sh;
        out.append(at, code.cbegin() + m->position());
        out += "@group(0) @binding(" + std::to_string((*m)[2].matched ? 2 * sh.pictures : b < 16 ? 2 * b : 2 * (b - 16) + 1) + ") var" + (*m)[2].str();
        at = code.cbegin() + m->position() + m->length();
    }
    out.append(at, code.cend());
    out = std::regex_replace(out, std::regex("@fragment\\s*\\nfn " + entry + "\\("), "@fragment\nfn main(");
    if (clip)   // (D3D11's BORDER address (transparent) under linear filtering: the clamped sample faded over the half texel past the edge)
        out = std::regex_replace(out, std::regex(R"(\btextureSample(Level)?\()"), "aviutl_clip$1(") +
              "\nfn aviutl_clipLevel(t: texture_2d<f32>, s: sampler, c: vec2f, l: f32) -> vec4f {\n"
              "  let n = vec2f(textureDimensions(t, 0));\n"
              "  let w = clamp(c * n + 0.5, vec2f(0.0), vec2f(1.0)) * clamp((1.0 - c) * n + 0.5, vec2f(0.0), vec2f(1.0));\n"
              "  return textureSampleLevel(t, s, c, l) * w.x * w.y;\n}\n"
              "fn aviutl_clip(t: texture_2d<f32>, s: sampler, c: vec2f) -> vec4f { return aviutl_clipLevel(t, s, c, 0.0); }\n";
    sh.wgsl = out;
    return sh;
}

const AviutlShader& aviutl_shader(const std::string& hlsl, const std::string& entry, bool clip) {
    static std::mutex lock;
    static std::map<std::string, AviutlShader> made;   // (each shader as written made once: a few milliseconds each)
    std::lock_guard<std::mutex> hold(lock);
    const std::string key = entry + (clip ? "|clip|" : "||") + hlsl;
    if (auto m = made.find(key); m != made.end()) return m->second;
    return made[key] = aviutl_make(hlsl, entry, clip);
}
