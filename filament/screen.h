// A whole-screen effect on the stage: one WGSL file in vgpu's manner (a fragment shader only, `fn fs_main(@builtin(position)
// frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f`, the picture a `texture_2d<f32>`, a `sampler`, and its
// knobs one `var<uniform>` struct), with its knobs declared in its head as a recipe's are (script.h: // --track@name:...).
// The stage's picture is drawn to a texture of its own instead of the screen, and the effect draws it on the screen
// (screen_door.h: through Filament's own driver, in order with its pictures). Knobs the struct has and the head does not
// name are told as they come; `time` (seconds), `frame` (its number at sixty a second), `aspect` (wide over high), `resolution`, `width` and `height` (points) by the stage.
#pragma once
// (kit.h: read before this, as doc.h does)
#include "script.h"
#include "dawn_probe/screen_door.h"
#include <fstream>
#include <regex>
#include <sstream>

// A knob's value now, asked by the effect's file and the knob's name (two effects may each have a knob of one name).
using ScreenNumbers = std::function<double(const std::string& file, const std::string& knob, double usual)>;

struct ScreenFx {
    std::string file, trouble;
    std::vector<ScriptKnob> knobs;
    struct Field { std::string name; uint32_t at = 0, floats = 1; bool whole = false; };
    std::vector<Field> fields;   // the knobs' struct as WGSL lays a uniform out
    uint32_t bytes = 0;
    door::Screen* door = nullptr;
    int pictures = 1;   // the stage's picture, then each target's (ISF's passes)
    // ISF's passes, drawn in turn (knob `pass`): into a target of the effect's own (kept, its last picture read the
    // next frame, when persistent; finer when float; a share of the stage's size or so many points), or, the last, out.
    struct Target { std::string name; bool kept = false, fine = false; double share[2] = {1, 1}; int points[2] = {0, 0}; Texture* picture[2] = {nullptr, nullptr}; RenderTarget* into[2] = {nullptr, nullptr}; int front = 0; uint32_t w = 0, h = 0; };
    std::vector<Target> targets;
    std::vector<int> passes;   // each pass's target (-1: out)
    // The stage's past frames, when its head asks for them (// --history@frames:share): an array of layers, the frame
    // drawn last at `history_head` (a knob), `history_count` of them, read as a `texture_2d_array<f32>`.
    unsigned layered = 0;   // which of its pictures are that array
    int historyFrames = 0, head = -1;
    double historyShare = 1;
    Texture* history = nullptr;
    std::vector<RenderTarget*> historyInto;
    uint32_t hw = 0, hh = 0;
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
    int reads = 0;   // (the pictures it reads: the k-th at 2k, the k-th sampler at 2k+1, the knobs after them all)
    for (std::sregex_iterator m(code.begin(), code.end(), bound), end; m != end; ++m) reads += (*m)[3].str().rfind("texture_2d", 0) == 0;   // (texture_2d and texture_2d_array alike)
    std::string out, knobsType;
    int pictures = 0, samplers = 0, uniforms = 0;
    auto at = code.cbegin();
    for (std::sregex_iterator m(code.begin(), code.end(), bound), end; m != end; ++m) {
        const std::string type = (*m)[3];
        int binding = -1;
        if (type.rfind("texture_2d_array", 0) == 0) fx.layered |= 1u << pictures, binding = 2 * pictures++;
        else if (type.rfind("texture_2d", 0) == 0) binding = 2 * pictures++;
        else if (type == "sampler") binding = 2 * samplers++ + 1;
        else if ((*m)[1].matched) binding = 2 * reads, ++uniforms, knobsType = type;
        if (binding < 0) return fx.trouble = "the effect has a binding of a kind it cannot be given: " + type, false;
        out.append(at, code.cbegin() + m->position());
        out += "@group(0) @binding(" + std::to_string(binding) + ") var" + ((*m)[1].matched ? "<uniform>" : "") + " " + (*m)[2].str() + ": " + type;
        at = code.cbegin() + m->position() + m->length();
    }
    out.append(at, code.cend());
    if (pictures < 1 || samplers > pictures || uniforms > 1) return fx.trouble = "an effect reads a picture or more, a sampler for each at most, and one struct of knobs at most", false;
    fx.pictures = pictures;
    std::istringstream lines(code);   // (its passes, as the head says them: // --pass@target:kept,fine,width,height)
    for (std::string line; std::getline(lines, line) && line.rfind("//", 0) == 0;) {
        if (const size_t h = line.find("--history@"); h != std::string::npos) {   // (--history@frames:share)
            fx.historyFrames = std::clamp(std::atoi(line.c_str() + h + 10), 2, 240);
            if (const size_t c = line.find(':', h); c != std::string::npos) fx.historyShare = std::clamp(std::atof(line.c_str() + c + 1), 0.05, 1.0);
            continue;
        }
        const size_t p = line.find("--pass@"), colon = line.find(':', p);
        if (p == std::string::npos || colon == std::string::npos) continue;
        const std::string name = line.substr(p + 7, colon - p - 7);
        std::vector<std::string> said;
        std::stringstream rest(line.substr(colon + 1));
        for (std::string one; std::getline(rest, one, ',');) said.push_back(one);
        if (said.size() < 4) return fx.trouble = "a pass said wrong: " + line, false;
        int target = -1;
        if (!name.empty()) {
            for (size_t k = 0; k < fx.targets.size(); ++k) if (fx.targets[k].name == name) target = (int)k;
            if (target < 0) {
                ScreenFx::Target t;
                t.name = name, t.kept = said[0] == "1", t.fine = said[1] == "1";
                for (int side = 0; side < 2; ++side) {
                    const std::string& w = said[(size_t)side + 2];
                    if (!w.empty() && w[0] == 'a') t.points[side] = std::max(1, std::atoi(w.c_str() + 1)), t.share[side] = 0;
                    else t.share[side] = w.size() > 1 ? std::atof(w.c_str() + 1) : 1.0;
                }
                fx.targets.push_back(t);
                target = (int)fx.targets.size() - 1;
            }
        }
        fx.passes.push_back(target);
    }
    if (!fx.targets.empty() && (int)fx.targets.size() + 1 + (int)__builtin_popcount(fx.layered) != pictures) return fx.trouble = "its passes' pictures and the pictures it reads do not agree", false;
    if (fx.layered && !fx.historyFrames) return fx.trouble = "it reads an array of past frames and its head does not ask for them (--history@frames:share)", false;
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
    door::Screen* copy = nullptr;   // a picture put out as it is (an effect whose last pass draws into its own picture)
};
static Texture* screen_picture(Stage& s) {
    return Texture::Builder().width((uint32_t)s.w).height((uint32_t)s.h).levels(1).format(Texture::InternalFormat::RGBA8)
        .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE).build(*s.engine);
}
static void screen_knobs(Stage& s, ScreenFx& fx, const ScreenNumbers& numbers, int pass = 0) {
    std::vector<float> at(fx.bytes / 4, 0.f);
    for (const ScreenFx::Field& f : fx.fields) {
        float v[4] = {0, 0, 0, 0};
        if (f.name == "time") v[0] = (float)s.now;
        else if (f.name == "aspect") v[0] = (float)s.w / (float)s.h;
        else if (f.name == "resolution") v[0] = (float)s.w, v[1] = (float)s.h;
        else if (f.name == "width" || f.name == "height") v[0] = (float)(f.name == "width" ? s.w : s.h);   // (three's way of saying it)
        else if (f.name == "frame") v[0] = (float)std::lround(s.now * 60.0);   // (ISF's FRAMEINDEX: the picture's number at sixty a second)
        else if (f.name == "passindex") v[0] = (float)pass;   // (ISF's PASSINDEX: `pass` is a word WGSL keeps for itself)
        else if (f.name == "history_head") v[0] = (float)std::max(0, fx.head);
        else if (f.name == "history_count") v[0] = (float)fx.historyFrames;
        else
            for (uint32_t i = 0; i < f.floats; ++i) {   // (a vector's numbers each a knob of its own, name_x, name_y...)
                const std::string k = f.floats == 1 ? f.name : f.name + "_" + "xyzw"[i];
                double usual = 0;
                for (const ScriptKnob& one : fx.knobs) if (one.name == k) usual = one.usual;
                v[i] = (float)numbers(fx.file, k, usual);
            }
        for (uint32_t i = 0; i < f.floats; ++i) {
            if (f.whole) { const int32_t w = (int32_t)std::lround(v[i]); std::memcpy(&at[f.at / 4 + i], &w, 4); }
            else at[f.at / 4 + i] = v[i];
        }
    }
    if (!at.empty()) door::knobs(s.engine, fx.door, at.data(), (uint32_t)(at.size() * 4), pass);
}
// An effect's own pictures (its passes' targets), made when first drawn, as large as the stage says, two each (one
// read while the other is drawn into, then turned round: a kept one is the last frame's when read before it is drawn).
static void screen_targets(Stage& s, ScreenFx& fx, bool drop = false) {
    for (ScreenFx::Target& t : fx.targets)
        for (int k = 0; k < 2; ++k) {
            if (drop) {
                if (t.into[k]) s.engine->destroy(t.into[k]), t.into[k] = nullptr;
                if (t.picture[k]) s.engine->destroy(t.picture[k]), t.picture[k] = nullptr;
                continue;
            }
            if (t.picture[k]) continue;
            t.w = t.points[0] ? (uint32_t)t.points[0] : (uint32_t)std::max(1L, std::lround(s.w * t.share[0]));
            t.h = t.points[1] ? (uint32_t)t.points[1] : (uint32_t)std::max(1L, std::lround(s.h * t.share[1]));
            t.picture[k] = Texture::Builder().width(t.w).height(t.h).levels(1).format(t.fine ? Texture::InternalFormat::RGBA16F : Texture::InternalFormat::RGBA8)
                               .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE).build(*s.engine);
            t.into[k] = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, t.picture[k]).build(*s.engine);
        }
}
static door::Screen* screen_copy(Stage& s, ScreenStack& st) {   // (a picture put out as it is)
    if (!st.copy) {
        static const char* const same = "@group(0) @binding(0) var picture: texture_2d<f32>;\n@group(0) @binding(1) var near: sampler;\n"
                                        "@fragment fn main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f { return textureSampleLevel(picture, near, uv, 0.0); }\n";
        st.copy = door::screen(s.engine, "copy", same, 0);
    }
    return st.copy;
}
// A picture put on another where `at` says (left, bottom, wide, high), as it is; all else of it clear (the desk's
// stage, its whole-screen effects drawn as a written film has them: live.h).
static void screen_place(Stage& s, Texture* from, RenderTarget* to, uint32_t w, uint32_t h, const int* at);
// The kit's copier (a picture as it is, to another or to the screen): what lies under a picture that reads it.
static void screen_copier(Stage& s) {
    if (!s.copyDoor) s.copyDoor = [&s](Texture* from, RenderTarget* to) { screen_place(s, from, to, from->getWidth(), from->getHeight(), nullptr); };
}
static void screen_place(Stage& s, Texture* from, RenderTarget* to, uint32_t w, uint32_t h, const int* at) {
    static std::map<Engine*, door::Screen*> copies;
    door::Screen*& copy = copies[s.engine];
    if (!copy) {
        static const char* const same = "@group(0) @binding(0) var picture: texture_2d<f32>;\n@group(0) @binding(1) var near: sampler;\n"
                                        "@fragment fn main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f { return textureSampleLevel(picture, near, uv, 0.0); }\n";
        copy = door::screen(s.engine, "place", same, 0);
    }
    if (copy) door::draw(s.engine, copy, from, to, w, h, at);
}
// One effect drawn: reading `from` (the stage's picture or the effect before's), drawing to `to`, through its passes;
// its own pictures in order: `from`, then each target (ISF's) or the stage's past frames (the array).
static void screen_draw(Stage& s, ScreenStack& st, ScreenFx& fx, Texture* from, RenderTarget* to, const ScreenNumbers& numbers) {
    screen_targets(s, fx);
    if (fx.historyFrames) {   // (this frame written into the next layer of the past frames, before it is read)
        if (!fx.history) {
            fx.hw = (uint32_t)std::max(2L, std::lround(s.w * fx.historyShare)), fx.hh = (uint32_t)std::max(2L, std::lround(s.h * fx.historyShare));
            fx.history = Texture::Builder().width(fx.hw).height(fx.hh).depth((uint32_t)fx.historyFrames).levels(1).sampler(Texture::Sampler::SAMPLER_2D_ARRAY)
                             .format(Texture::InternalFormat::RGBA8).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE).build(*s.engine);
            for (int k = 0; k < fx.historyFrames; ++k)
                fx.historyInto.push_back(RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, fx.history).layer(RenderTarget::AttachmentPoint::COLOR, (uint32_t)k).build(*s.engine));
        }
        fx.head = (fx.head + 1) % fx.historyFrames;
        door::draw(s.engine, screen_copy(s, st), from, fx.historyInto[(size_t)fx.head], fx.hw, fx.hh);
    }
    std::vector<Texture*> read((size_t)fx.pictures, from);
    const auto gather = [&] {
        size_t target = 0;
        for (size_t k = 1; k < read.size(); ++k)
            read[k] = (fx.layered >> k) & 1u ? fx.history : target < fx.targets.size() ? fx.targets[target].picture[fx.targets[target].front] : from, target += !((fx.layered >> k) & 1u);
    };
    if (fx.passes.empty()) {
        gather();
        screen_knobs(s, fx, numbers);
        door::draw(s.engine, fx.door, read.data(), to, (uint32_t)s.w, (uint32_t)s.h);
        return;
    }
    for (size_t p = 0; p < fx.passes.size(); ++p) {
        gather();
        screen_knobs(s, fx, numbers, (int)p);
        const int t = fx.passes[p];
        if (t < 0) { door::draw(s.engine, fx.door, read.data(), to, (uint32_t)s.w, (uint32_t)s.h, (int)p); continue; }
        ScreenFx::Target& into = fx.targets[(size_t)t];
        const int back = 1 - into.front;
        door::draw(s.engine, fx.door, read.data(), into.into[back], into.w, into.h, (int)p);
        into.front = back;
    }
    if (fx.passes.back() >= 0) {   // (its last pass drew into its own picture: that is what it shows)
        const ScreenFx::Target& last = fx.targets[(size_t)fx.passes.back()];
        door::draw(s.engine, screen_copy(s, st), last.picture[last.front], to, (uint32_t)s.w, (uint32_t)s.h);
    }
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
    door::Screen* made = door::screen(s.engine, fx.file.c_str(), code.c_str(), again.bytes, &again.trouble, again.pictures, again.layered);
    if (!made) { std::fprintf(stderr, "the effect %s as written now is not taken (the one before stays): %s\n", fx.file.c_str(), again.trouble.c_str()); return; }
    door::drop(s.engine, fx.door);
    screen_targets(s, fx, true);
    fx.knobs = again.knobs, fx.fields = again.fields, fx.bytes = again.bytes, fx.door = made;
    fx.pictures = again.pictures, fx.targets = again.targets, fx.passes = again.passes, fx.layered = again.layered;   // (the past frames kept as they are if their count stays)
}
// Whether a file of the stage's effects has been written over since it was made (what wakes a stage that is not drawing).
static bool screen_moved(const ScreenStack& st) {
    for (const ScreenFx* fx : st.fx) if (screen_stamp(fx->file) != fx->stamp) return true;
    return false;
}
static void screen_effects(Stage& s, ScreenStack& st, const std::string& files, ScreenNumbers numbers) {
    if (st.files == files) return;
    st.files = files;
    s.over = nullptr;   // (what there was let go: the stage draws to the screen again)
    s.drawnInto = nullptr;
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
            made->door = door::screen(s.engine, file.c_str(), code.c_str(), made->bytes, &made->trouble, made->pictures, made->layered);
            if (!made->door) std::fprintf(stderr, "the effect %s is not taken: %s\n", file.c_str(), made->trouble.c_str());
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
    s.drawnInto = st.shot;   // (what lies under a picture that reads it is copied from here)
    s.over = [&s, &st, numbers] {
        for (size_t i = 0; i < st.fx.size(); ++i) {
            screen_fresh(s, *st.fx[i]);
            Texture* from = i == 0 ? st.shot : st.between[(i - 1) % 2];
            RenderTarget* to = i + 1 == st.fx.size() ? s.out : st.betweenTarget[i % 2];   // (the last where the stage's picture goes: the screen, or the window's live picture)
            screen_draw(s, st, *st.fx[i], from, to, numbers);
        }
    };
}
