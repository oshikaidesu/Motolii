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
    std::filesystem::file_time_type stamp{};       // (the file's time when it was made: written over, it is made again)
    std::chrono::steady_clock::time_point looked{};
};
static std::filesystem::file_time_type screen_stamp(const std::string& file) {
    std::error_code wrong;
    return std::filesystem::last_write_time(file, wrong);
}

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

// The stage given its effects, one over another (files: paths with spaces between, "" or "none": none). numbers(name,
// usual): a knob's value now. The first reads the stage's picture, each next the one before's, the last draws on the
// screen: two pictures between them taken in turn. An effect that cannot be made is left out (said on stderr).
struct ScreenStack {
    std::string files;
    std::vector<ScreenFx*> fx;
    std::map<std::string, ScreenFx*> made;   // every effect made so far, by its file, kept (a shader is made once: making one takes a picture's time or ten)
    Texture* shot = nullptr;
    Texture* depth = nullptr;
    RenderTarget* target = nullptr;
    Texture* between[2] = {nullptr, nullptr};
    RenderTarget* betweenTarget[2] = {nullptr, nullptr};
};
static Texture* screen_picture(Stage& s) {
    return Texture::Builder().width((uint32_t)s.w).height((uint32_t)s.h).levels(1).format(Texture::InternalFormat::RGBA8)
        .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE).build(*s.engine);
}
static void screen_knobs(Stage& s, ScreenFx& fx, const std::function<double(const std::string&, double)>& numbers) {
    std::vector<float> at(fx.bytes / 4, 0.f);
    for (const ScreenFx::Field& f : fx.fields) {
        float v[4] = {0, 0, 0, 0};
        if (f.name == "time") v[0] = (float)s.now;
        else if (f.name == "aspect") v[0] = (float)s.w / (float)s.h;
        else if (f.name == "resolution") v[0] = (float)s.w, v[1] = (float)s.h;
        else if (f.name == "width" || f.name == "height") v[0] = (float)(f.name == "width" ? s.w : s.h);   // (three's way of saying it)
        else
            for (uint32_t i = 0; i < f.floats; ++i) {   // (a vector's numbers each a knob of its own, name_x, name_y...)
                const std::string k = f.floats == 1 ? f.name : f.name + "_" + "xyzw"[i];
                double usual = 0;
                for (const ScriptKnob& one : fx.knobs) if (one.name == k) usual = one.usual;
                v[i] = (float)numbers(k, usual);
            }
        for (uint32_t i = 0; i < f.floats; ++i) {
            if (f.whole) { const int32_t w = (int32_t)std::lround(v[i]); std::memcpy(&at[f.at / 4 + i], &w, 4); }
            else at[f.at / 4 + i] = v[i];
        }
    }
    if (!at.empty()) door::knobs(s.engine, fx.door, at.data(), (uint32_t)(at.size() * 4));
}
// An effect whose file was written over made again where it stands (looked at four times a second at most): a shader
// saved shows at once. A file that no longer reads keeps the one before (said on stderr).
static void screen_fresh(Stage& s, ScreenFx& fx) {
    const auto now = std::chrono::steady_clock::now();
    if (now - fx.looked < std::chrono::milliseconds(250)) return;
    fx.looked = now;
    const auto stamp = screen_stamp(fx.file);
    if (stamp == fx.stamp) return;
    fx.stamp = stamp;
    ScreenFx again;
    again.file = fx.file;
    std::string code;
    if (!screen_read(again, code)) { std::fprintf(stderr, "%s\n", again.trouble.c_str()); return; }
    door::drop(s.engine, fx.door);
    fx.knobs = again.knobs, fx.fields = again.fields, fx.bytes = again.bytes;
    fx.door = door::screen(s.engine, fx.file.c_str(), code.c_str(), fx.bytes);
}
// Whether a file of the stage's effects has been written over since it was made (what wakes a stage that is not drawing).
static bool screen_moved(const ScreenStack& st) {
    for (const ScreenFx* fx : st.fx) if (screen_stamp(fx->file) != fx->stamp) return true;
    return false;
}
static void screen_effects(Stage& s, ScreenStack& st, const std::string& files, std::function<double(const std::string&, double)> numbers) {
    if (st.files == files) return;
    st.files = files;
    s.over = nullptr;   // (what there was let go: the stage draws to the screen again)
    s.view->setRenderTarget(nullptr);
    for (View* v : s.layers) v->setRenderTarget(nullptr);
    st.fx.clear();
    std::istringstream each(files);
    for (std::string file; each >> file;) {
        if (file == "none") continue;
        ScreenFx*& made = st.made[file];
        if (!made) {
            made = new ScreenFx;
            made->file = file;
            std::string code;
            if (!screen_read(*made, code)) { std::fprintf(stderr, "%s\n", made->trouble.c_str()); continue; }   // (kept as not made: not read again each time)
            made->door = door::screen(s.engine, file.c_str(), code.c_str(), made->bytes);
            made->stamp = screen_stamp(file);
        }
        if (made->door) st.fx.push_back(made);
    }
    if (st.fx.empty()) return;
    if (!st.shot) {   // (the pictures between them made once, kept while the stage is)
        st.shot = screen_picture(s);
        st.depth = Texture::Builder().width((uint32_t)s.w).height((uint32_t)s.h).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT).format(Texture::InternalFormat::DEPTH24).build(*s.engine);
        st.target = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, st.shot).texture(RenderTarget::AttachmentPoint::DEPTH, st.depth).build(*s.engine);
    }
    for (int k = 0; k < 2 && st.fx.size() > 1; ++k)
        if (!st.between[k]) st.between[k] = screen_picture(s), st.betweenTarget[k] = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, st.between[k]).build(*s.engine);
    s.view->setRenderTarget(st.target);
    for (View* v : s.layers) v->setRenderTarget(st.target);
    s.over = [&s, &st, numbers] {
        for (size_t i = 0; i < st.fx.size(); ++i) {
            screen_fresh(s, *st.fx[i]);
            screen_knobs(s, *st.fx[i], numbers);
            Texture* from = i == 0 ? st.shot : st.between[(i - 1) % 2];
            RenderTarget* to = i + 1 == st.fx.size() ? nullptr : st.betweenTarget[i % 2];
            door::draw(s.engine, st.fx[i]->door, from, to, (uint32_t)s.w, (uint32_t)s.h);
        }
    };
}
