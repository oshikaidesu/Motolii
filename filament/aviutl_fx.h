// aviutl_fx.h: an AviUtl2 script (aviutl.h) as one of a thing's effects, run as it is on the thing's picture each frame:
// LuaJIT 2.1 (MIT, AviUtl2's own default Lua) with os, io, debug, ffi.C and ffi.load taken out as lua.txt says, and an
// `obj` of its spec's names, each done by what Motolii already has: its buffers ("object", "tempbuffer", "cache:...",
// "framebuffer": the thing's own picture as it leaves) RGBA16F pictures premultiplied as AviUtl2's, its shaders drawn
// by the door (screen_door.h), obj.draw by one placing shader. What it asks for that is not here stops it with
// "not yet: obj.X" (said once, its picture left as it came): never a picture silently other than the script's.
#pragma once
#include "aviutl.h"
#include <lua.hpp>
#include <array>
#include <random>

struct AviutlBuf { Texture* t = nullptr; RenderTarget* rt = nullptr; uint32_t w = 0, h = 0; };
struct AviutlRun {
    std::string file, own, all, trouble;
    std::vector<AviutlVar> vars;
    std::map<std::string, double> usual;            // (each knob's value as it comes)
    lua_State* L = nullptr;
    int chunk = LUA_NOREF;
    std::map<std::string, AviutlBuf> bufs;
    std::map<std::string, AviutlBuf> loaded;        // (figures and picture files obj.load drew, by what was asked: aviutl_load.h)
    std::map<std::string, std::vector<uint8_t>> pixels;   // (the pixels obj.getpixeldata handed out, by buffer)
    std::map<const void*, std::string> order;
    std::multimap<std::pair<uint32_t, uint32_t>, AviutlBuf> spare;   // (pictures let go, kept by size for the next asked: a buffer made and let go each frame takes the same ones)
    std::map<std::string, door::Screen*> doors;     // (each shader made, by its name and way of sampling)
    std::map<door::Screen*, int> draws;             // (draws of each this frame: each its own constants, the door's pass)
    door::Screen *place = nullptr, *poly = nullptr, *blur = nullptr;
    Texture *none = nullptr, *random = nullptr;
    std::set<std::string> said;
    Stage* s = nullptr;                             // (this frame's)
    std::string target, blend, sampler;
    bool drew = false;
    bool behind = false;                            // (it reads the frame: the stage behind the thing, as AviUtl2's framebuffer holds what is drawn before it)
    double box[4] = {0, 0, 0, 0};                   // (where the thing is seen, points of the stage: its frame is that of the stage's)
    int ground = -1;                                // (the stage's colour behind everything, 0xRRGGBB; -1 none)
};

// The placing shader: a picture (`size` pixels) put with its pixel `from` at `at` of the target, turned and sized by
// `m` (the target's step back to the picture's), faded by alpha; past its edge clear (AviUtl2's sampler "clip", as
// aviutl.cpp's), or not (way 2); way 1: the colour alone (a buffer cleared to it); otherwise colour.x 1: read straight
// and made premultiplied (a thing's picture, straight as Motolii's effects keep it: effected.mat), 2: made straight again.
static const char* const aviutl_placing = R"(
struct K { m: vec4f, at: vec2f, src: vec2f, size: vec2f, alpha: f32, way: f32, colour: vec4f }
@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var near: sampler;
@group(0) @binding(2) var<uniform> k: K;
@fragment fn main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {
  if (k.way == 1.0) { return k.colour; }
  let d = frag.xy - k.at;
  let c = (vec2f(dot(k.m.xy, d), dot(k.m.zw, d)) + k.src) / k.size;
  let w = clamp(c * k.size + 0.5, vec2f(0.0), vec2f(1.0)) * clamp((1.0 - c) * k.size + 0.5, vec2f(0.0), vec2f(1.0));
  var s = textureSampleLevel(picture, near, c, 0.0);
  if (k.colour.x == 1.0) { s = vec4f(s.rgb * s.a, s.a); }
  s = s * (k.alpha * select(w.x * w.y, 1.0, k.way == 2.0));
  if (k.colour.x == 2.0 && s.a > 0.0) { s = vec4f(s.rgb / s.a, s.a); }
  return s;
}
)";
// obj.drawpoly's: the picture on triangles (the door's mesh), at each corner's uv, faded by alpha.
static const char* const aviutl_poly = R"(
struct K { alpha: f32 }
@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var near: sampler;
@group(0) @binding(2) var<uniform> k: K;
@fragment fn main(@location(0) uv: vec2f) -> @location(0) vec4f {
  return textureSampleLevel(picture, near, uv, 0.0) * k.alpha;
}
)";
// obj.effect ぼかし's: sashimono-edit's blur1d (MIT, effects/builtin.py), one way a pass: the pixel and `radius` either
// side (96 at most) weighed by a Gaussian of half the radius; the buffer premultiplied already, as its premul makes it.
static const char* const aviutl_blur = R"(
struct K { step: vec2f, radius: f32, unused: f32 }
@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var near: sampler;
@group(0) @binding(2) var<uniform> k: K;
@fragment fn main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {
  let at = frag.xy / vec2f(textureDimensions(picture));
  let sigma = max(k.radius * 0.5, 0.5);
  var sum = textureSampleLevel(picture, near, at, 0.0);
  var total = 1.0;
  let taps = i32(min(k.radius, 96.0));
  for (var i = 1; i <= taps; i++) {
    let o = f32(i);
    let w = exp(-0.5 * o * o / (sigma * sigma));
    sum += (textureSampleLevel(picture, near, at + k.step * o, 0.0) + textureSampleLevel(picture, near, at - k.step * o, 0.0)) * w;
    total += 2.0 * w;
  }
  return sum / total;
}
)";

static AviutlRun& aviutl_of(lua_State* L) { return *(AviutlRun*)lua_touserdata(L, lua_upvalueindex(1)); }
static std::string aviutl_name(std::string n) { return n == "obj" ? "object" : n == "tmp" ? "tempbuffer" : n == "frm" ? "framebuffer" : n; }
static AviutlBuf& aviutl_buf(AviutlRun& r, const std::string& name, uint32_t w = 0, uint32_t h = 0) {   // (w, h: made that size)
    AviutlBuf& b = r.bufs[aviutl_name(name)];
    w = std::clamp<uint32_t>(w ? w : b.w, 1, 8192), h = std::clamp<uint32_t>(h ? h : b.h, 1, 8192);
    if (b.t && b.w == w && b.h == h) return b;
    if (b.t) r.spare.insert({{b.w, b.h}, b});
    if (const auto kept = r.spare.find({w, h}); kept != r.spare.end()) return b = kept->second, r.spare.erase(kept), b;
    b.w = w, b.h = h;
    b.t = Texture::Builder().width(w).height(h).levels(1).format(Texture::InternalFormat::RGBA16F).usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE).build(*r.s->engine);
    b.rt = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, b.t).build(*r.s->engine);
    return b;
}
// One placing (blend: -1 in its place, 0 over, 1 added, 2 masked; sampler as the door's), knobs as the struct above.
static void aviutl_put(AviutlRun& r, Texture* from, RenderTarget* to, uint32_t w, uint32_t h, const float* k, int blend = -1, int sampler = 0) {
    const int pass = r.draws[r.place]++;
    door::knobs(r.s->engine, r.place, k, 64, pass);
    door::draw(r.s->engine, r.place, &from, to, w, h, pass, nullptr, blend >= 0, std::max(blend, 0), sampler);
}
// A picture laid on another: its middle (+ c, its own centre) at the target's middle + o, zoomed (sx, sy) and turned rz degrees.
static void aviutl_lay(AviutlRun& r, const AviutlBuf& from, RenderTarget* to, uint32_t w, uint32_t h, double ox, double oy, double sx, double sy, double rz, double alpha, double cx, double cy, int blend, int sampler, float way = 0, float as = 0) {
    const double a = rz * M_PI / 180, co = std::cos(a), si = std::sin(a);
    const float k[16] = {(float)(co / sx), (float)(si / sx), (float)(-si / sy), (float)(co / sy), (float)(w / 2.0 + ox), (float)(h / 2.0 + oy), (float)(from.w / 2.0 + cx), (float)(from.h / 2.0 + cy), (float)from.w, (float)from.h, (float)alpha, way, as, 0, 0, 0};
    aviutl_put(r, from.t, to, w, h, k, blend, sampler);
}
static void aviutl_copy(AviutlRun& r, const std::string& to, const std::string& from) {
    const AviutlBuf f = aviutl_buf(r, from);
    const AviutlBuf& t = aviutl_buf(r, to, f.w, f.h);
    aviutl_lay(r, f, t.rt, t.w, t.h, 0, 0, 1, 1, 0, 1, 0, 0, -1, 1);
}
static void aviutl_clear(AviutlRun& r, const AviutlBuf& b, double colour, bool opaque) {
    const int c = (int)colour;
    const float k[16] = {1, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 1, opaque ? ((c >> 16) & 255) / 255.f : 0, opaque ? ((c >> 8) & 255) / 255.f : 0, opaque ? (c & 255) / 255.f : 0, opaque ? 1.f : 0};
    aviutl_put(r, r.none, b.rt, b.w, b.h, k);
}
static bool aviutl_old(const AviutlVar& v) { return v.var.size() == 6 && (v.var.rfind("track", 0) == 0 || v.var.rfind("check", 0) == 0) && std::isdigit((unsigned char)v.var[5]); }   // (--track0: is obj.track0)
static int aviutl_no(lua_State* L, const std::string& what) { return luaL_error(L, "not yet: %s", what.c_str()); }
static std::string aviutl_str(lua_State* L, int i) { return lua_type(L, i) == LUA_TSTRING ? lua_tostring(L, i) : ""; }
static double aviutl_num(lua_State* L, int i, double otherwise = 0) { return lua_isnumber(L, i) ? lua_tonumber(L, i) : lua_isboolean(L, i) ? lua_toboolean(L, i) : otherwise; }
static std::string aviutl_hlsl(AviutlRun& r, const std::string& name) {   // (a shader by its registered name: this script's, this file's, or "name@script" another's beside it ("name@part@file": a part of an @ file))
    const size_t at = name.find('@');
    const std::string entry = name.substr(0, at), head = "--[[pixelshader@" + entry + ":";
    std::vector<std::string> texts = {r.own, r.all};
    if (at != std::string::npos) {
        const std::string other = name.substr(at + 1), part = other.substr(0, other.rfind('@') == std::string::npos ? 0 : other.rfind('@')), stem = other.substr(other.rfind('@') + 1);
        const std::filesystem::path dir = std::filesystem::path(aviutl_path(r.file)).parent_path();
        for (const std::string f : {stem + ".anm2", "@" + stem + ".anm2", stem + ".obj2", "@" + stem + ".obj2"}) {
            std::string own, all, why;
            if (aviutl_text((dir / f).string() + (part.empty() ? "" : "#" + part), own, all, why)) texts.insert(texts.begin(), own);
        }
    }
    for (const std::string& t : texts)
        if (const size_t p = t.find(head); p != std::string::npos) return t.substr(p + head.size(), t.find("]]", p) - p - head.size());
    return "";
}
static door::Screen* aviutl_door(AviutlRun& r, const std::string& name, bool clip, std::string& why, const AviutlShader** made = nullptr) {
    const std::string hlsl = aviutl_hlsl(r, name), entry = name.substr(0, name.find('@'));
    if (hlsl.empty()) return why = "no pixelshader@" + name, nullptr;
    const AviutlShader& sh = aviutl_shader(hlsl, entry, clip);
    if (made) *made = &sh;
    door::Screen*& d = r.doors[name + (clip ? "|clip" : "")];
    if (!d && sh.wgsl.empty()) return why = sh.why, nullptr;
    if (!d) d = door::screen(r.s->engine, entry.c_str(), sh.wgsl.c_str(), sh.bytes, &why, sh.pictures);
    return d;
}

// ---- obj, as lua.txt says each (only these; any other name: not yet) ----
static int aviutl_pixelshader(lua_State* L) {   // (name, target, {resource...}, {constant...}, blend, sampler)
    AviutlRun& r = aviutl_of(L);
    const std::string name = luaL_checkstring(L, 1), target = aviutl_name(luaL_checkstring(L, 2)), blend = lua_isstring(L, 5) ? lua_tostring(L, 5) : "copy", sampler = aviutl_str(L, 6);
    const AviutlShader* sh = nullptr;
    std::string why;
    door::Screen* d = aviutl_door(r, name, sampler == "clip" || sampler == "dot", why, &sh);
    if (!d) return aviutl_no(L, "obj.pixelshader " + name + ": " + why.substr(0, 300));
    if (target.rfind("image:", 0) == 0 || !r.bufs.count(target)) return aviutl_no(L, "obj.pixelshader into " + target);
    std::vector<std::string> res;
    if (lua_istable(L, 3)) for (int i = 1; i <= (int)lua_objlen(L, 3); ++i) lua_rawgeti(L, 3, i), res.push_back(aviutl_name(aviutl_str(L, -1))), lua_pop(L, 1);
    else if (lua_isstring(L, 3)) res.push_back(aviutl_name(lua_tostring(L, 3)));
    std::vector<Texture*> read((size_t)sh->pictures, r.none);
    for (size_t i = 0; i < res.size() && i < read.size(); ++i) {
        if (res[i] == "random") { read[i] = r.random; continue; }
        if ((res[i] == "framebuffer" && !r.behind) || !r.bufs.count(res[i])) return aviutl_no(L, "obj.pixelshader reading " + res[i]);
        if (res[i] == target) aviutl_copy(r, "~" + std::to_string(i), target), read[i] = r.bufs["~" + std::to_string(i)].t;   // (the target read as a copy, as the spec says)
        else read[i] = r.bufs[res[i]].t;
    }
    std::vector<float> lua, k(sh->bytes / 4, 0.f);   // (the floats as Lua gives them, moved where WGSL lays them: aviutl.cpp)
    if (lua_istable(L, 4)) for (int i = 1; i <= (int)lua_objlen(L, 4); ++i) lua_rawgeti(L, 4, i), lua.push_back((float)aviutl_num(L, -1)), lua_pop(L, 1);
    for (size_t s = 0; s < sh->from.size(); ++s)
        for (uint32_t c = 0; c < sh->count[s]; ++c)
            if (sh->from[s] / 4 + c < lua.size() && sh->to[s] / 4 + c < k.size()) k[sh->to[s] / 4 + c] = lua[sh->from[s] / 4 + c];
    const int way = blend == "copy" ? -1 : blend == "draw" ? 0 : blend == "add" ? 1 : blend == "mask" ? 2 : -2;
    if (way == -2) return aviutl_no(L, "obj.pixelshader blend " + blend);
    const AviutlBuf& t = r.bufs[target];
    const int pass = r.draws[d]++;
    if (!k.empty()) door::knobs(r.s->engine, d, k.data(), (uint32_t)k.size() * 4, pass);
    door::draw(r.s->engine, d, read.data(), t.rt, t.w, t.h, pass, nullptr, way >= 0, std::max(way, 0), sampler == "dot" ? 1 : sampler == "loop" ? 2 : sampler == "mirror" ? 3 : 0);
    return 0;
}
static int aviutl_clearbuffer(lua_State* L) {   // (target[, w, h][, colour])
    AviutlRun& r = aviutl_of(L);
    const std::string target = aviutl_name(luaL_checkstring(L, 1));
    const bool sized = lua_isnumber(L, 3);
    if (target.rfind("cache:", 0) != 0 && target != "object" && target != "tempbuffer" && !(target == "framebuffer" && !sized)) return aviutl_no(L, "obj.clearbuffer " + target);
    const AviutlBuf& b = aviutl_buf(r, target, sized ? (uint32_t)aviutl_num(L, 2) : 0, sized ? (uint32_t)aviutl_num(L, 3) : 0);
    aviutl_clear(r, b, aviutl_num(L, sized ? 4 : 2), lua_isnumber(L, sized ? 4 : 2));
    return 0;
}
static const AviutlBuf* aviutl_image(AviutlRun& r, std::string file);   // (aviutl_load.h)
static int aviutl_copybuffer(lua_State* L) {   // (dst, src; src "image:" a picture file, read as obj.load reads one: lua.txt)
    AviutlRun& r = aviutl_of(L);
    const std::string to = aviutl_name(luaL_checkstring(L, 1)), from = aviutl_name(luaL_checkstring(L, 2));
    if (from.rfind("image:", 0) == 0 && to != "framebuffer") {
        const AviutlBuf* const picture = aviutl_image(r, from.substr(6));
        if (!picture) return lua_pushboolean(L, 0), 1;
        const AviutlBuf& t = aviutl_buf(r, to, picture->w, picture->h);
        aviutl_lay(r, *picture, t.rt, t.w, t.h, 0, 0, 1, 1, 0, 1, 0, 0, -1, 1);
        return lua_pushboolean(L, 1), 1;
    }
    if ((from == "framebuffer" || to == "framebuffer") && !r.behind) return aviutl_no(L, "obj.copybuffer " + to + " from " + from);
    if (!r.bufs.count(from)) return lua_pushboolean(L, 0), 1;
    aviutl_copy(r, to, from);
    return lua_pushboolean(L, 1), 1;
}
#include "aviutl_load.h"   // (obj.load: a buffer, a figure, a picture file)
#include "aviutl_pixels.h"   // (obj.getpixeldata, obj.putpixeldata)
static int aviutl_setoption(lua_State* L) {
    AviutlRun& r = aviutl_of(L);
    const std::string n = luaL_checkstring(L, 1);
    if (n == "drawtarget" || n == "dst") {
        const std::string t = aviutl_name(aviutl_str(L, 2));
        if (t != "tempbuffer" && t != "framebuffer") return aviutl_no(L, "obj.setoption drawtarget " + t);
        r.target = t;
        if (t == "tempbuffer" && lua_isnumber(L, 3)) aviutl_clear(r, aviutl_buf(r, t, (uint32_t)aviutl_num(L, 3), (uint32_t)aviutl_num(L, 4)), 0, false);
    } else if (n == "blend") r.blend = lua_isnumber(L, 2) ? std::to_string((int)lua_tonumber(L, 2)) : aviutl_str(L, 2);
    else if (n == "draw_state") r.drew = lua_toboolean(L, 2);
    else if (n == "sampler") r.sampler = aviutl_str(L, 2);
    else if (n != "culling" && n != "billboard" && n != "focus_mode") return aviutl_no(L, "obj.setoption " + n);   // (those three change nothing flat)
    return 0;
}
static int aviutl_getoption(lua_State* L) {
    AviutlRun& r = aviutl_of(L);
    const std::string n = luaL_checkstring(L, 1);
    if (n == "track_mode" || n == "camera_mode" || n == "culling" || n == "billboard") lua_pushinteger(L, 0);   // (no track moves by itself here; no camera, nothing turned)
    else if (n == "section_num") lua_pushinteger(L, 1);
    else if (n == "gui" || n == "multi_object") lua_pushboolean(L, 0);
    else if (n == "draw_state") lua_pushboolean(L, r.drew);
    else if (n == "drawtarget") lua_pushstring(L, r.target.c_str());
    else if (n == "blend") lua_pushstring(L, r.blend.c_str());
    else if (n == "script_name" && lua_gettop(L) == 1) lua_pushstring(L, std::filesystem::path(r.file).stem().string().c_str());
    else return aviutl_no(L, "obj.getoption " + n);
    return 1;
}
static int aviutl_getinfo(lua_State* L) {
    AviutlRun& r = aviutl_of(L);
    const std::string n = luaL_checkstring(L, 1);
    if (n == "version") lua_pushinteger(L, 2011500);   // (the spec read: v2.1.15)
    else if (n == "filter" || n == "saving" || n == "reference_frame") lua_pushboolean(L, 0);
    else if (n == "image_max") return lua_pushinteger(L, r.bufs["framebuffer"].w), lua_pushinteger(L, r.bufs["framebuffer"].h), 2;   // (a thing's picture grows no larger than its box)
    else if (n == "script_path") lua_pushstring(L, (std::filesystem::path(r.file).parent_path().string() + "/").c_str());
    else if (n == "clock") lua_pushnumber(L, std::chrono::duration<double>(std::chrono::steady_clock::now().time_since_epoch()).count());
    else return aviutl_no(L, "obj.getinfo " + n);
    return 1;
}
static int aviutl_getvalue(lua_State* L) {   // (target): its own tracks, and the thing's base values as a flat thing's are
    AviutlRun& r = aviutl_of(L);
    std::string n = lua_isnumber(L, 1) ? std::to_string((int)lua_tonumber(L, 1)) : aviutl_str(L, 1);
    int tracks = 0;
    for (const AviutlVar& v : r.vars)
        if (v.kind == "track" && (n == std::to_string(tracks++) || n == "track." + v.var)) return aviutl_old(v) ? (lua_getglobal(L, "obj"), lua_getfield(L, -1, v.var.c_str())) : lua_getglobal(L, v.var.c_str()), 1;
    if (n == "time") return lua_pushnumber(L, r.s->now), 1;
    if (n == "x" || n == "y" || n == "z" || n == "rx" || n == "ry" || n == "rz" || n == "cx" || n == "cy" || n == "cz" || n == "aspect") return lua_pushnumber(L, 0), 1;
    if (n == "sx" || n == "sy" || n == "sz" || n == "alpha") return lua_pushnumber(L, 1), 1;
    if (n == "zoom") return lua_pushnumber(L, 100), 1;
    if (n == "pos" || n == "angle" || n == "center") return lua_pushnumber(L, 0), lua_pushnumber(L, 0), lua_pushnumber(L, 0), 3;
    if (n == "scale") return lua_pushnumber(L, 1), lua_pushnumber(L, 1), lua_pushnumber(L, 1), 3;
    return aviutl_no(L, "obj.getvalue " + n);
}
static int aviutl_setanchor(lua_State* L) { return lua_pushinteger(L, std::max(1, (int)aviutl_num(L, 2))), 1; }   // (its points are not on the screen yet: the values stay the knobs')
static int aviutl_getpixel(lua_State* L) {
    AviutlRun& r = aviutl_of(L);
    if (lua_gettop(L)) return aviutl_no(L, "obj.getpixel of a pixel");
    return lua_pushinteger(L, r.bufs["object"].w), lua_pushinteger(L, r.bufs["object"].h), 2;
}
static int aviutl_setfont(lua_State*) { return 0; }   // (kept for a text, which is not read yet)
static int aviutl_draw(lua_State* L) {   // (ox, oy, oz, zoom, alpha, rx, ry, rz): flat only
    AviutlRun& r = aviutl_of(L);
    if (aviutl_num(L, 3) != 0 || aviutl_num(L, 6) != 0 || aviutl_num(L, 7) != 0) return aviutl_no(L, "obj.draw in depth");
    const int way = r.blend == "none" || r.blend == "0" || r.blend.empty() ? 0 : r.blend == "add" || r.blend == "1" || r.blend == "rgba_add" || r.blend == "alpha_add" ? 1 : -1;
    if (way < 0) return aviutl_no(L, "obj.draw blend " + r.blend);
    lua_getglobal(L, "obj");
    const auto f = [&](const char* k, double otherwise) { lua_getfield(L, -1, k); const double v = aviutl_num(L, -1, otherwise); lua_pop(L, 1); return v; };
    const bool own = r.target == "framebuffer";   // (into the frame: as the thing stands; into the temporary buffer: by its arguments alone)
    const double zoom = aviutl_num(L, 4, 1) * (own ? f("zoom", 1) : 1), aspect = own ? f("aspect", 0) : 0;
    const AviutlBuf& to = r.bufs[r.target];
    if (!to.t) return aviutl_no(L, "obj.draw into a temporary buffer never made");
    aviutl_lay(r, r.bufs["object"], to.rt, to.w, to.h, aviutl_num(L, 1) + (own ? f("ox", 0) : 0), aviutl_num(L, 2) + (own ? f("oy", 0) : 0), zoom * (aspect > 0 ? 1 - aspect : 1), zoom * (aspect < 0 ? 1 + aspect : 1),
        aviutl_num(L, 8) + (own ? f("rz", 0) : 0), aviutl_num(L, 5, 1) * (own ? f("alpha", 1) : 1), own ? f("cx", 0) : 0, own ? f("cy", 0) : 0, way, r.sampler == "dot" ? 1 : r.sampler == "loop" ? 2 : r.sampler == "mirror" ? 3 : 0, r.sampler == "clamp" || r.sampler == "loop" || r.sampler == "mirror" ? 2.f : 0.f);
    if (own) r.drew = true;
    return 0;
}
// obj.drawpoly: each four corners two triangles, 0-1-2 and 0-2-3 (sashimono-edit's _fill_quad, MIT), the picture on
// them at the corners' u, v, all of one call drawn at once on the door's mesh (as Filament draws its own triangle). As
// lua.txt has it (the AviUtl2 script skill's
// references/api.md): corners from the object's place into the frame, as they are into the temporary buffer, the
// frame's alpha the object's too (sashimono-edit's lua_drawpoly); u, v pixels (none: the picture's four corners), the
// alpha the 13th or 21st; a table of faces of 20 ({x0 .. z3, u0 .. v3}[, alpha]), or of corners {x, y, z, u, v} with
// u, v 0 to 1, 4 or 3 a face ([, count, alpha]).
static int aviutl_drawpoly(lua_State* L) {
    AviutlRun& r = aviutl_of(L);
    const AviutlBuf& from = r.bufs["object"];
    struct Corner { double x, y, z, u, v; };
    std::vector<std::vector<Corner>> faces;
    double alpha = 1;
    const auto item = [&]() { std::vector<double> v; for (int j = 1; j <= (int)lua_objlen(L, -1); ++j) lua_rawgeti(L, -1, j), v.push_back(aviutl_num(L, -1)), lua_pop(L, 1); return v; };
    const auto face = [&](const double* p) { std::vector<Corner> c(4); for (int i = 0; i < 4; ++i) c[i] = {p[i * 3], p[i * 3 + 1], p[i * 3 + 2], p[12 + i * 2], p[13 + i * 2]}; return c; };
    if (lua_istable(L, 1)) {
        std::vector<std::vector<double>> items;
        for (int i = 1; i <= (int)lua_objlen(L, 1); ++i) {
            lua_rawgeti(L, 1, i);
            if (!lua_istable(L, -1)) return aviutl_no(L, "obj.drawpoly of a table of numbers");
            items.push_back(item()), lua_pop(L, 1);
        }
        if (items.empty()) return 0;
        const size_t width = items[0].size();
        for (const auto& v : items) if (v.size() != width) return aviutl_no(L, "obj.drawpoly of mixed faces");
        if (width == 20) {   // (faces)
            alpha = aviutl_num(L, 2, 1);
            for (const auto& v : items) faces.push_back(face(v.data()));
        } else if (width == 5) {   // (corners)
            const int count = lua_isnumber(L, 2) ? (int)lua_tonumber(L, 2) : 4;
            alpha = aviutl_num(L, 3, 1);
            if ((count != 3 && count != 4) || items.size() % count) return aviutl_no(L, "obj.drawpoly of " + std::to_string(count) + "-cornered faces");
            for (size_t f = 0; f < items.size(); f += count) {
                std::vector<Corner> c;
                for (int i = 0; i < count; ++i) c.push_back({items[f + i][0], items[f + i][1], items[f + i][2], items[f + i][3] * from.w, items[f + i][4] * from.h});
                faces.push_back(c);
            }
        } else return aviutl_no(L, "obj.drawpoly of " + std::to_string(width) + " numbers a " + (width == 7 || width == 28 ? "face with its corners' colours" : "face with its normals"));
    } else {
        const int n = lua_gettop(L);
        if (n < 12) return aviutl_no(L, "obj.drawpoly with fewer than four corners");
        double p[20] = {};
        for (int i = 0; i < 12; ++i) p[i] = aviutl_num(L, i + 1);
        if (n >= 20) for (int i = 0; i < 8; ++i) p[12 + i] = aviutl_num(L, 13 + i);
        else p[14] = from.w, p[16] = from.w, p[17] = from.h, p[19] = from.h;
        alpha = n == 13 ? aviutl_num(L, 13, 1) : n >= 21 ? aviutl_num(L, 21, 1) : 1;
        faces.push_back(face(p));
    }
    const int way = r.blend == "none" || r.blend == "0" || r.blend.empty() ? 0 : r.blend == "add" || r.blend == "1" || r.blend == "rgba_add" || r.blend == "alpha_add" ? 1 : -1;
    if (way < 0) return aviutl_no(L, "obj.drawpoly blend " + r.blend);
    const AviutlBuf& to = r.bufs[r.target];
    if (!to.t) return aviutl_no(L, "obj.drawpoly into a temporary buffer never made");
    if (!r.poly) return aviutl_no(L, "obj.drawpoly: " + r.trouble);
    const bool own = r.target == "framebuffer";
    lua_getglobal(L, "obj");
    const auto f = [&](const char* k, double otherwise) { lua_getfield(L, -1, k); const double v = aviutl_num(L, -1, otherwise); lua_pop(L, 1); return v; };
    const double ox = own ? f("ox", 0) : 0, oy = own ? f("oy", 0) : 0, fade = alpha * (own ? f("alpha", 1) : 1);
    lua_pop(L, 1);
    for (const auto& c : faces)
        for (const Corner& p : c) if (p.z != 0) return aviutl_no(L, "obj.drawpoly in depth");
    std::vector<float> corners;   // (the target's -1 to 1, up; the picture's 0 to 1)
    const auto corner = [&](const Corner& p) { corners.insert(corners.end(), {(float)((to.w / 2.0 + ox + p.x) / to.w * 2 - 1), (float)(1 - (to.h / 2.0 + oy + p.y) / to.h * 2), (float)(p.u / from.w), (float)(p.v / from.h)}); };
    for (const auto& c : faces) {
        corner(c[0]), corner(c[1]), corner(c[2]);
        if (c.size() == 4) corner(c[0]), corner(c[2]), corner(c[3]);
    }
    const float k[4] = {(float)fade, 0, 0, 0};
    const int pass = r.draws[r.poly]++;
    door::knobs(r.s->engine, r.poly, k, sizeof k, pass);
    Texture* picture = from.t;
    door::draw(r.s->engine, r.poly, &picture, to.rt, to.w, to.h, pass, nullptr, true, way, r.sampler == "dot" ? 1 : r.sampler == "loop" ? 2 : r.sampler == "mirror" ? 3 : 0, corners.data(), (uint32_t)corners.size() / 4);
    if (own) r.drew = true;
    return 0;
}
static int aviutl_effect(lua_State* L) {   // (a built-in effect by its name: these, as sashimono-edit makes them)
    AviutlRun& r = aviutl_of(L);
    const std::string n = aviutl_str(L, 1);
    std::map<std::string, double> said;
    for (int i = 2; i + 1 <= lua_gettop(L); i += 2) said[aviutl_str(L, i)] = aviutl_num(L, i + 1);
    const auto at = [&](const char* k, double otherwise) { return said.count(k) ? said[k] : otherwise; };
    if (n == "ぼかし" && at("範囲", 0) == 0) return 0;
    if (n == "ぼかし" && said.count("範囲") && at("縦横比", 0) == 0 && at("光の強さ", 0) == 0) {   // (sashimono-edit's blur, effects/builtin.py _BLUR: across, then down; サイズ固定 reads no further than the picture (the sampler's clamp), else the picture grows by the radius each side, as far as it reaches: strokes.py pixel_reach)
        const double radius = std::min(at("範囲", 0), 4096.0);
        if (!(radius > 0)) return 0;
        if (!r.blur) return aviutl_no(L, "obj.effect ぼかし: " + r.trouble);
        if (at("サイズ固定", 0) == 0) {
            const double grow = std::ceil(radius);
            aviutl_copy(r, "~blur", "object");
            const AviutlBuf from = r.bufs["~blur"];
            const AviutlBuf& to = aviutl_buf(r, "object", (uint32_t)std::min(8192.0, from.w + 2 * grow), (uint32_t)std::min(8192.0, from.h + 2 * grow));
            aviutl_lay(r, from, to.rt, to.w, to.h, 0, 0, 1, 1, 0, 1, 0, 0, -1, 1);
        }
        const AviutlBuf object = r.bufs["object"];
        const AviutlBuf& across = aviutl_buf(r, "~blur", object.w, object.h);
        for (const auto& [from, to, x] : {std::tuple{object, across, true}, std::tuple{across, object, false}}) {
            const float k[4] = {x ? 1.f / from.w : 0.f, x ? 0.f : 1.f / from.h, (float)radius, 0};
            const int pass = r.draws[r.blur]++;
            door::knobs(r.s->engine, r.blur, k, sizeof k, pass);
            Texture* picture = from.t;
            door::draw(r.s->engine, r.blur, &picture, to.rt, to.w, to.h, pass);
        }
        return 0;
    }
    if (n == "リサイズ" && at("拡大率", 100) == 100 && at("X", 100) == 100 && at("Y", 100) == 100) return 0;
    if (n == "領域拡張" && !at("塗りつぶし", 0)) {   // (margins of nothing round it)
        const double t = at("上", 0), b = at("下", 0), l = at("左", 0), rt = at("右", 0);
        aviutl_copy(r, "~expand", "object");
        const AviutlBuf from = r.bufs["~expand"];
        const AviutlBuf& to = aviutl_buf(r, "object", (uint32_t)std::max(1.0, from.w + l + rt), (uint32_t)std::max(1.0, from.h + t + b));
        aviutl_lay(r, from, to.rt, to.w, to.h, (l - rt) / 2, (t - b) / 2, 1, 1, 0, 1, 0, 0, -1, 1);
        return 0;
    }
    if (n == "リサイズ") {   // (sashimono-edit objapi.py _resize: "ドット数でサイズ指定" then X, Y in pixels, else 拡大率 times X %, Y %; "補間なし" the nearest pixel; no larger than its MAX_FIGURE_SIZE, 4096)
        aviutl_copy(r, "~resize", "object");
        const AviutlBuf from = r.bufs["~resize"];
        const bool dots = at("ドット数でサイズ指定", 0) != 0;
        const double zoom = at("拡大率", 100) / 100, w = dots ? at("X", from.w) : from.w * zoom * at("X", 100) / 100, h = dots ? at("Y", from.h) : from.h * zoom * at("Y", 100) / 100;
        if (!std::isfinite(w) || !std::isfinite(h)) return aviutl_no(L, "obj.effect リサイズ of no number");
        const AviutlBuf& to = aviutl_buf(r, "object", (uint32_t)std::clamp(std::round(w), 1.0, 4096.0), (uint32_t)std::clamp(std::round(h), 1.0, 4096.0));
        aviutl_lay(r, from, to.rt, to.w, to.h, 0, 0, (double)to.w / from.w, (double)to.h / from.h, 0, 1, 0, 0, -1, at("補間なし", 0) != 0 ? 1 : 0);
        return 0;
    }
    if (n == "クリッピング") {   // (sashimono-edit objapi.py _clip: 上 下 左 右 cut off in pixels, all of it a clear pixel; what is left stays where it was, ox, oy, cx, cy moved back by half the cut, unless 中心の位置を変更)
        const double t = std::max(0.0, std::round(at("上", 0))), b = std::max(0.0, std::round(at("下", 0))), l = std::max(0.0, std::round(at("左", 0))), rt = std::max(0.0, std::round(at("右", 0)));
        if (!std::isfinite(t + b + l + rt)) return aviutl_no(L, "obj.effect クリッピング of no number");
        aviutl_copy(r, "~clip", "object");
        const AviutlBuf from = r.bufs["~clip"];
        const bool all = t + b >= from.h || l + rt >= from.w;
        const AviutlBuf& to = aviutl_buf(r, "object", all ? 1 : (uint32_t)(from.w - l - rt), all ? 1 : (uint32_t)(from.h - t - b));
        if (all) aviutl_clear(r, to, 0, false);
        else aviutl_lay(r, from, to.rt, to.w, to.h, 0, 0, 1, 1, 0, 1, (l - rt) / 2, (t - b) / 2, -1, 1);
        if (at("中心の位置を変更", 0) == 0) {
            lua_getglobal(L, "obj");
            const auto move = [&](const char* k, double by) { lua_getfield(L, -1, k); const double v = aviutl_num(L, -1, 0); lua_pop(L, 1); lua_pushnumber(L, v + by); lua_setfield(L, -2, k); };
            move("ox", (l - rt) / 2), move("oy", (t - b) / 2), move("cx", -(l - rt) / 2), move("cy", -(t - b) / 2);
            lua_pop(L, 1);
        }
        return 0;
    }
    if (n == "オフスクリーン描画") return 0;   // (sashimono-edit objapi.py _offscreen: the effects laid up so far made into the picture; here each is made at once, so none wait)
    if (n.empty()) return 0;   // (no name: lua.txt says the filters after the script, which run after it here anyway; YMM4-LuaScript native/worker.lua and sashimono-edit objapi.py lua_effect return the same)
    return aviutl_no(L, "obj.effect " + n);
}

static const char* const aviutl_globals = R"(
local bit = require("bit")
local function byte(v) return math.max(0, math.min(255, math.floor(v))) end
function RGB(r, g, b, ...)
  if select("#", ...) > 0 then error("not yet: RGB between two colours") end
  if g == nil then r = math.floor(r); return bit.band(bit.rshift(r, 16), 255), bit.band(bit.rshift(r, 8), 255), bit.band(r, 255) end
  return bit.bor(bit.lshift(byte(r), 16), bit.lshift(byte(g), 8), byte(b))
end
function OR(a, b) return bit.bor(a, b) end
function AND(a, b) return bit.band(a, b) end
function XOR(a, b) return bit.bxor(a, b) end
function SHIFT(a, s) if s >= 0 then return bit.lshift(a, s) end return bit.arshift(a, -s) end
function HSV() error("not yet: HSV") end
-- obj.interpolation(t, x0,y0,z0, x1,y1,z1, x2,y2,z2, x3,y3,z3): four points' Catmull-Rom, t 0 to 1 from the second to the
-- third, y and z left out as wanted (the AviUtl2 script skill's references/api.md, gasdyueer, MIT); each axis by
-- sashimono-edit's _catmull_rom (MIT, compat/aviutl/objapi.py), of the four points around it.
function aviutl_interpolation(t, ...)
  local p, n = {...}, select("#", ...)
  local d = math.floor(n / 4)
  if d < 1 then return 0 end
  local out = {}
  for a = 1, d do
    local p0, p1, p2, p3 = p[a], p[a + d], p[a + 2 * d], p[a + 3 * d]
    out[a] = 0.5 * ((2 * p1) + (-p0 + p2) * t + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t * t + (-p0 + 3 * p1 - 3 * p2 + p3) * t * t * t)
  end
  return unpack(out)
end
function rotation() error("not yet: rotation") end
debug_print = print
global = {}
-- obj.rand: YMM4-LuaScript's, as it is (native/shim.lua 4-14, 310-328; native/worker.lua 445-447).
-- MIT License, Copyright (c) 2026 routersys (https://github.com/routersys/YMM4-LuaScript).
local i64 = ffi.typeof("int64_t")
local u32 = ffi.typeof("uint32_t")
local u64 = ffi.typeof("uint64_t")

local function tou(d)
    return tonumber(u32(i64(d)))
end

local function umul32(a, b)
    return tonumber(u32(u64(a) * u64(b)))
end
local floor = math.floor
function aviutl_rand(a, b, seed, frame)
    local ps = ffi.new("double[1]", seed)
    local ws = ffi.cast("uint32_t*", ps)
    local s = tou(bit.bxor(ws[0], ws[1]))
    local pf = ffi.new("double[1]", frame)
    local wf = ffi.cast("uint32_t*", pf)
    local f = tou(bit.bxor(wf[0], wf[1]))
    local h = tou(umul32(s, 374761393) + umul32(f, 668265263))
    h = umul32(tou(bit.bxor(h, bit.rshift(h, 13))), 1274126177)
    h = tou(bit.bxor(h, bit.rshift(h, 16)))
    local r = h / 4294967295.0
    local lo = floor(math.min(a, b))
    local hi = floor(math.max(a, b))
    local span = hi - lo + 1
    if span <= 1 then return lo end
    local v = lo + floor(r * span)
    if v > hi then return hi end
    return v
end
function rand(a, b, seed, frame)
    return aviutl_rand(a or 0, b or 0, seed or 0, frame or obj.frame or 0)
end
-- obj.rand1: lua.txt's 0.0 up to (not) 1.0, from the same
function rand1(seed, frame) return aviutl_rand(0, 4294967294, seed or 0, frame or obj.frame or 0) / 4294967295 end
)";

// Made once a file: its Lua state (the libraries lua.txt leaves, require from its own folder), obj, its knobs' first
// values, and each of its shaders made (said on stderr: one that cannot be is "not yet" when it is used).
static std::shared_ptr<AviutlRun> aviutl_made(Stage& s, const std::string& file) {
    auto r = std::make_shared<AviutlRun>();
    r->file = file, r->s = &s;
    if (!aviutl_text(file, r->own, r->all, r->trouble)) return std::fprintf(stderr, "the AviUtl2 script %s is not taken: %s\n", file.c_str(), r->trouble.c_str()), r;
    r->vars = aviutl_vars(r->own);
    r->behind = std::regex_search(r->all, std::regex(R"re(copybuffer\s*\([^)]*"(framebuffer|frm)"|getoption\s*\(\s*"drawtarget"\s*\))re"));   // (it reads the frame: its picture there is the stage's behind it)
    for (const ScriptKnob& k : aviutl_knobs(file)) r->usual[k.name] = k.usual;
    lua_State* L = r->L = luaL_newstate();
    luaL_openlibs(L);
    const std::string dir = std::filesystem::path(file).parent_path().string();
    const std::string sandbox = "os, io, debug, dofile, loadfile = nil, nil, nil, nil, nil\npackage.loaded.os, package.loaded.io, package.loaded.debug = nil, nil, nil\n"
                                "local ffi = require('ffi'); ffi.C, ffi.load = nil, nil\npackage.loadlib, package.cpath = nil, ''\npackage.path = [==[" + dir + "/?.lua;" + dir + "/?/init.lua]==]\n";
    if (luaL_dostring(L, (sandbox + aviutl_globals).c_str())) r->trouble = lua_tostring(L, -1);
    lua_newtable(L);
    const std::pair<const char*, lua_CFunction> calls[] = {{"pixelshader", aviutl_pixelshader}, {"clearbuffer", aviutl_clearbuffer}, {"copybuffer", aviutl_copybuffer}, {"load", aviutl_load}, {"setoption", aviutl_setoption}, {"getoption", aviutl_getoption},
        {"getinfo", aviutl_getinfo}, {"getvalue", aviutl_getvalue}, {"setanchor", aviutl_setanchor}, {"getpixel", aviutl_getpixel}, {"setfont", aviutl_setfont}, {"draw", aviutl_draw}, {"drawpoly", aviutl_drawpoly}, {"effect", aviutl_effect},
        {"getpixeldata", aviutl_getpixeldata}, {"putpixeldata", aviutl_putpixeldata}};
    for (const auto& [n, f] : calls) lua_pushlightuserdata(L, r.get()), lua_pushcclosure(L, f, 1), lua_setfield(L, -2, n);
    luaL_dostring(L, "return { __index = function(_, k) error('not yet: obj.' .. tostring(k), 2) end }");   // (any other name of obj)
    lua_setmetatable(L, -2);
    lua_setglobal(L, "obj");
    luaL_dostring(L, "obj.rand, obj.rand1, obj.interpolation = rand, rand1, aviutl_interpolation");   // (lua.txt: obj. or not)
    std::string text = r->own.rfind("\xEF\xBB\xBF", 0) == 0 ? r->own.substr(3) : r->own;
    if (luaL_loadbuffer(L, text.data(), text.size(), ("@" + std::filesystem::path(file).filename().string()).c_str())) r->trouble = lua_tostring(L, -1);
    else r->chunk = luaL_ref(L, LUA_REGISTRYINDEX);
    std::string why;
    if (!(r->place = door::screen(s.engine, "aviutl place", aviutl_placing, 64, &why))) r->trouble = "its placing shader: " + why;
    if (!(r->poly = door::screen(s.engine, "aviutl drawpoly", aviutl_poly, 16, &why, 1, 0, true))) r->trouble = "its drawpoly shader: " + why;
    if (!(r->blur = door::screen(s.engine, "aviutl blur", aviutl_blur, 16, &why))) r->trouble = "its blur shader: " + why;
    r->none = Texture::Builder().width(1).height(1).levels(1).format(Texture::InternalFormat::RGBA8).build(*s.engine);
    r->none->setImage(*s.engine, 0, backend::PixelBufferDescriptor(new uint32_t(0), 4, backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE, [](void* p, size_t, void*) { delete (uint32_t*)p; }));
    auto* noise = new std::vector<math::half>(256 * 256);   // ("random": 256 x 256 of 0 to 1 (R32F there; a half float here, as a door's picture is filtered))
    std::mt19937 rng(1);
    for (math::half& h : *noise) h = math::half(std::uniform_real_distribution<float>(0.f, 1.f)(rng));
    r->random = Texture::Builder().width(256).height(256).levels(1).format(Texture::InternalFormat::R16F).build(*s.engine);
    r->random->setImage(*s.engine, 0, backend::PixelBufferDescriptor(noise->data(), noise->size() * 2, backend::PixelDataFormat::R, backend::PixelDataType::HALF, [](void*, size_t, void* u) { delete (std::vector<math::half>*)u; }, noise));
    int made = 0, all = 0;
    const std::regex shader(R"(--\[\[pixelshader@(\w+):)");
    for (std::sregex_iterator m(r->all.begin(), r->all.end(), shader), end; m != end; ++m, ++all) {
        std::string why;
        if (aviutl_door(*r, (*m)[1], false, why) && (aviutl_door(*r, (*m)[1], true, why) || true)) ++made;   // (and the way it samples "clip", made now: not inside a frame)
        else std::fprintf(stderr, "the AviUtl2 shader %s of %s is not made: %s\n", (*m)[1].str().c_str(), file.c_str(), why.substr(0, 400).c_str());
    }
    std::fprintf(stderr, "AviUtl2 script %s: %d of %d shaders made%s%s\n", file.c_str(), made, all, r->trouble.empty() ? "" : ", not taken: ", r->trouble.substr(0, 300).c_str());
    return r;
}

// One frame of it: the thing's picture `from` (w x h) its object, its knobs and obj's values told, the script run, and
// what it leaves (the frame it drew, else the object as obj now places it) drawn into `to`.
static void aviutl_draw_step(Stage& s, AviutlRun& r, Texture* from, RenderTarget* to, const ScreenNumbers& numbers, uint32_t w, uint32_t h) {
    r.s = &s, r.target = "framebuffer", r.blend = "none", r.sampler.clear(), r.drew = false, r.draws.clear();
    if (!r.place) return screen_place(s, from, to, w, h, nullptr);   // (none made: the picture as it came)
    AviutlBuf in{from, nullptr, w, h};
    const AviutlBuf& object = aviutl_buf(r, "object", w, h);
    aviutl_lay(r, in, object.rt, w, h, 0, 0, 1, 1, 0, 1, 0, 0, -1, 1, 0, 1);   // (premultiplied, as AviUtl2's pictures are)
    const AviutlBuf& frame = aviutl_buf(r, "framebuffer", w, h);
    const bool behind = r.behind && s.behind && r.box[2] > r.box[0] && r.box[3] > r.box[1];
    aviutl_clear(r, frame, behind ? std::max(r.ground, 0) : 0, behind && r.ground >= 0);
    if (behind) {   // (what is drawn before it, there: the stage's colour, then the stage's picture without the thing (kit_things.h, as AbsentMotion's backdrop reads it) in sRGB as the thing's own picture is given (screen_tone), over it)
        const double bw = s.behind->getWidth(), bh = s.behind->getHeight(), kx = bw / std::max(1, s.w), ky = bh / std::max(1, s.h);
        const AviutlBuf all{s.behind, nullptr, (uint32_t)bw, (uint32_t)bh};
        const AviutlBuf& there = aviutl_buf(r, "~behind", w, h);
        aviutl_lay(r, all, there.rt, w, h, 0, 0, w / ((r.box[2] - r.box[0]) * kx), h / ((r.box[3] - r.box[1]) * ky), 0, 1, (r.box[0] + r.box[2]) / 2 * kx - bw / 2, (r.box[1] + r.box[3]) / 2 * ky - bh / 2, -1, 1);
        const AviutlBuf& toned = aviutl_buf(r, "~behind_srgb", w, h);
        screen_tone(s, there.t, toned.rt, w, h, true);
        aviutl_lay(r, toned, frame.rt, w, h, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1);
    }
    std::string trouble = r.trouble;
    lua_State* L = r.L;
    if (trouble.empty()) {
        for (const AviutlVar& v : r.vars) {
            const auto num = [&](const std::string& k) { return numbers(r.file, k, r.usual.count(k) ? r.usual[k] : v.usual); };
            if (v.kind == "track") lua_pushnumber(L, num(v.var));
            else if (v.kind == "check" && v.truth) lua_pushboolean(L, num(v.var) != 0);
            else if (v.kind == "check") lua_pushinteger(L, num(v.var) != 0);
            else if (v.kind == "select") lua_pushnumber(L, v.values.empty() ? 0 : v.values[(size_t)std::clamp<double>(std::round(num(v.var)), 0, (double)v.values.size() - 1)]);
            else if (v.kind == "color" && v.unset && num(v.var + "_on") == 0) lua_pushnil(L);
            else if (v.kind == "color") lua_pushinteger(L, ((int)num(v.var + "_r") << 16) | ((int)num(v.var + "_g") << 8) | (int)num(v.var + "_b"));
            else if (luaL_dostring(L, ("return " + v.expr).c_str())) lua_pop(L, 1), lua_pushnil(L);
            if (aviutl_old(v)) lua_getglobal(L, "obj"), lua_insert(L, -2), lua_setfield(L, -2, v.var.c_str()), lua_pop(L, 1);
            else lua_setglobal(L, v.var.c_str());
        }
        lua_getglobal(L, "obj");
        const double frame = std::round(s.now * 60);
        const std::pair<const char*, double> values[] = {{"ox", 0}, {"oy", 0}, {"oz", 0}, {"rx", 0}, {"ry", 0}, {"rz", 0}, {"cx", 0}, {"cy", 0}, {"cz", 0}, {"sx", 1}, {"sy", 1}, {"sz", 1}, {"zoom", 1}, {"aspect", 0}, {"alpha", 1},
            {"x", 0}, {"y", 0}, {"z", 0}, {"w", (double)w}, {"h", (double)h}, {"screen_w", (double)s.w}, {"screen_h", (double)s.h}, {"framerate", 60}, {"frame", frame}, {"time", s.now}, {"totalframe", frame + 1}, {"totaltime", s.now},
            {"layer", 1}, {"index", 0}, {"num", 1}, {"id", 1}, {"effect_id", 1}, {"frame_s", 0}, {"frame_e", frame}, {"effect_layer", 1}, {"originframe", 0}, {"scene_id", 0}};
        for (const auto& [n, v] : values) lua_pushnumber(L, v), lua_setfield(L, -2, n);
        lua_pop(L, 1);
        lua_rawgeti(L, LUA_REGISTRYINDEX, r.chunk);
        if (lua_pcall(L, 0, 0, 0)) trouble = lua_tostring(L, -1) ? lua_tostring(L, -1) : "?", lua_pop(L, 1);
    }
    if (!trouble.empty()) {   // (the picture as it came, the reason said once)
        if (r.said.insert(trouble).second) std::fprintf(stderr, "the AviUtl2 script %s stops: %s\n", r.file.c_str(), trouble.substr(0, 400).c_str());
        return aviutl_lay(r, in, to, w, h, 0, 0, 1, 1, 0, 1, 0, 0, -1, 1);
    }
    const AviutlBuf& out = aviutl_buf(r, "~out", w, h);   // (what it drew, the object over it unless it drew it, made straight again into `to`)
    aviutl_lay(r, r.bufs["framebuffer"], out.rt, w, h, 0, 0, 1, 1, 0, 1, 0, 0, -1, 1);
    lua_getglobal(L, "obj");
    const auto f = [&](const char* k, double otherwise) { lua_getfield(L, -1, k); const double v = aviutl_num(L, -1, otherwise); lua_pop(L, 1); return v; };
    if (!r.drew && (f("rx", 0) != 0 || f("ry", 0) != 0 || f("oz", 0) != 0)) {
        if (r.said.insert("depth").second) std::fprintf(stderr, "the AviUtl2 script %s stops: not yet: obj.rx, ry, oz (in depth)\n", r.file.c_str());
        return lua_pop(L, 1), aviutl_lay(r, in, to, w, h, 0, 0, 1, 1, 0, 1, 0, 0, -1, 1);
    }
    const double zoom = f("zoom", 1), aspect = f("aspect", 0);
    if (!r.drew) aviutl_lay(r, r.bufs["object"], out.rt, w, h, f("ox", 0), f("oy", 0), zoom * f("sx", 1) * (aspect > 0 ? 1 - aspect : 1), zoom * f("sy", 1) * (aspect < 0 ? 1 + aspect : 1), f("rz", 0), f("alpha", 1), f("cx", 0), f("cy", 0), 0, 0);
    lua_pop(L, 1);
    aviutl_lay(r, out, to, w, h, 0, 0, 1, 1, 0, 1, 0, 0, -1, 1, 0, 2);
}
