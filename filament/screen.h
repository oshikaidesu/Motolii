// A whole-screen effect on the stage: one WGSL file in vgpu's manner (a fragment shader only, `fn fs_main(@builtin(position)
// frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f`, the picture a `texture_2d<f32>`, a `sampler`, and its
// knobs one `var<uniform>` struct), with its knobs declared in its head as a recipe's are (script.h: // --track@name:...).
// The stage's picture is drawn to a texture of its own instead of the screen, and the effect draws it on the screen
// (screen_door.h: through Filament's own driver, in order with its pictures). Knobs the struct has and the head does not
// name are told as they come; `time` (seconds), `aspect` (wide over high), `resolution`, `width` and `height` (points) by the stage.
#pragma once
// (kit.h: read before this, as doc.h does)
#include "script.h"
#include "dawn_probe/screen_door.h"
#include <fstream>
#include <regex>
#include <sstream>

struct ScreenFx {
    std::string file, trouble;
    std::vector<ScriptKnob> knobs;
    struct Field { std::string name; uint32_t at = 0, floats = 1; bool whole = false; };
    std::vector<Field> fields;   // the knobs' struct as WGSL lays a uniform out
    uint32_t bytes = 0;
    door::Screen* door = nullptr;
    Texture* shot = nullptr;
    Texture* depth = nullptr;
    RenderTarget* target = nullptr;
};

// The file as Filament's driver takes it: the picture at binding 0, its sampler at 1, the knobs at 2 (Filament's
// descriptor 0 a sampled texture, 1 a uniform buffer), its entry `main`; and the knobs' struct read for where each is.
static bool screen_read(ScreenFx& fx, std::string& code) {
    std::ifstream in(fx.file);
    if (!in) return fx.trouble = "there is no such file: " + fx.file, false;
    std::stringstream all;
    all << in.rdbuf();
    code = all.str();
    const std::regex bound(R"(@group\(\s*\d+\s*\)\s*@binding\(\s*\d+\s*\)\s*var(<\s*uniform\s*>)?\s+(\w+)\s*:\s*([A-Za-z_][\w<>]*))");
    std::string out, knobsType;
    int pictures = 0, samplers = 0, uniforms = 0;
    auto at = code.cbegin();
    for (std::sregex_iterator m(code.begin(), code.end(), bound), end; m != end; ++m) {
        const std::string type = (*m)[3];
        int binding = -1;
        if (type.rfind("texture_2d", 0) == 0) binding = 0, ++pictures;
        else if (type == "sampler") binding = 1, ++samplers;
        else if ((*m)[1].matched) binding = 2, ++uniforms, knobsType = type;
        if (binding < 0) return fx.trouble = "the effect has a binding of a kind it cannot be given: " + type, false;
        out.append(at, code.cbegin() + m->position());
        out += "@group(0) @binding(" + std::to_string(binding) + ") var" + ((*m)[1].matched ? "<uniform>" : "") + " " + (*m)[2].str() + ": " + type;
        at = code.cbegin() + m->position() + m->length();
    }
    out.append(at, code.cend());
    if (pictures != 1 || samplers > 1 || uniforms > 1) return fx.trouble = "an effect reads one picture, with at most one sampler and one struct of knobs", false;
    code = std::regex_replace(out, std::regex(R"(\bfn\s+fs_main\s*\()"), "fn main(");
    if (!knobsType.empty()) {   // (WGSL's own rule: each as far on as its size, rounded up; vec3 as far as a vec4)
        std::smatch body;
        if (!std::regex_search(code, body, std::regex("struct\\s+" + knobsType + "\\s*\\{([^}]*)\\}"))) return fx.trouble = "no struct " + knobsType, false;
        const std::regex field(R"((\w+)\s*:\s*([\w<>]+))");
        const std::string inside = body[1];
        uint32_t end = 0, most = 4;
        for (std::sregex_iterator f(inside.begin(), inside.end(), field), stop; f != stop; ++f) {
            const std::string t = (*f)[2];
            const uint32_t n = t == "f32" || t == "i32" || t == "u32" ? 1 : t.find("vec2") == 0 ? 2 : t.find("vec3") == 0 ? 3 : t.find("vec4") == 0 ? 4 : 0;
            if (!n) return fx.trouble = "a knob of a kind an effect cannot be given: " + t, false;
            const uint32_t align = n == 1 ? 4 : n == 2 ? 8 : 16;
            end = (end + align - 1) / align * align;
            fx.fields.push_back({(*f)[1], end, n, t == "i32" || t == "u32"});
            end += 4 * n, most = std::max(most, align);
        }
        fx.bytes = (end + most - 1) / most * most;
    }
    fx.knobs = script_knobs(fx.file);
    return true;
}

// The stage given an effect (file empty: none). numbers(name, usual): a knob's value now. false: not made (fx.trouble).
static bool screen_effect(Stage& s, ScreenFx*& fx, const std::string& file, std::function<double(const std::string&, double)> numbers) {
    if (fx && fx->file == file) return true;
    if (fx) {   // (the one before let go: the stage draws to the screen again)
        s.over = nullptr;
        s.view->setRenderTarget(nullptr);
        for (View* v : s.layers) v->setRenderTarget(nullptr);
        door::drop(s.engine, fx->door);
        s.engine->destroy(fx->target), s.engine->destroy(fx->shot), s.engine->destroy(fx->depth);
        delete fx;
        fx = nullptr;
    }
    if (file.empty() || file == "none") return true;
    auto* made = new ScreenFx;
    made->file = file;
    std::string code;
    if (!screen_read(*made, code)) { const std::string why = made->trouble; delete made; fx = nullptr; return std::fprintf(stderr, "%s\n", why.c_str()), false; }
    made->door = door::screen(s.engine, file.c_str(), code.c_str(), made->bytes);
    made->shot = Texture::Builder().width((uint32_t)s.w).height((uint32_t)s.h).levels(1).format(Texture::InternalFormat::RGBA8)
                     .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE).build(*s.engine);
    made->depth = Texture::Builder().width((uint32_t)s.w).height((uint32_t)s.h).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT).format(Texture::InternalFormat::DEPTH24).build(*s.engine);
    made->target = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, made->shot).texture(RenderTarget::AttachmentPoint::DEPTH, made->depth).build(*s.engine);
    s.view->setRenderTarget(made->target);
    for (View* v : s.layers) v->setRenderTarget(made->target);
    fx = made;
    s.over = [&s, made, numbers] {
        std::vector<float> at(made->bytes / 4, 0.f);
        for (const ScreenFx::Field& f : made->fields) {
            float v[4] = {0, 0, 0, 0};
            if (f.name == "time") v[0] = (float)s.now;
            else if (f.name == "aspect") v[0] = (float)s.w / (float)s.h;
            else if (f.name == "resolution") v[0] = (float)s.w, v[1] = (float)s.h;
            else if (f.name == "width" || f.name == "height") v[0] = (float)(f.name == "width" ? s.w : s.h);   // (three's way of saying it)
            else
                for (uint32_t i = 0; i < f.floats; ++i) {   // (a vector's numbers each a knob of its own, name_x, name_y...)
                    const std::string k = f.floats == 1 ? f.name : f.name + "_" + "xyzw"[i];
                    double usual = 0;
                    for (const ScriptKnob& one : made->knobs) if (one.name == k) usual = one.usual;
                    v[i] = (float)numbers(k, usual);
                }
            for (uint32_t i = 0; i < f.floats; ++i) {
                if (f.whole) { const int32_t w = (int32_t)std::lround(v[i]); std::memcpy(&at[f.at / 4 + i], &w, 4); }
                else at[f.at / 4 + i] = v[i];
            }
        }
        if (!at.empty()) door::knobs(s.engine, made->door, at.data(), (uint32_t)(at.size() * 4));
        door::draw(s.engine, made->door, made->shot, nullptr, (uint32_t)s.w, (uint32_t)s.h);
    };
    return true;
}
