// AviUtl2's scripts (.anm2 effects, .obj2 objects: Lua with pixel shaders in HLSL kept in its comments, as KENくん's
// lua.txt says them) dropped in a thing's fx run as they are on its picture (aviutl_fx.h). Here what needs no picture:
// the file read (one script of an @ file, "@Stylize_K.anm2#Mosaic"), its head's knobs, and each shader made WGSL by
// glslang's own HLSL front end and Tint, as isf.cpp makes an ISF file's (built apart: aviutl.cpp, its headers heavy).
#pragma once
#include "script.h"
#include <cstdint>
#include <string>
#include <vector>

bool aviutl_is(const std::string& file);
inline std::string aviutl_path(const std::string& file) { return file.substr(0, file.find('#', file.rfind('/') + 1)); }   // (the file of "@x.anm2#name")
// The script's own text (the file, or its part from "@name" to the next) and the whole file's (any part's shaders are its).
bool aviutl_text(const std::string& file, std::string& own, std::string& all, std::string& why);
// A variable its head says (--track@, --check@, --color@, --select@, --value@; the old --track0: and --check0: as obj's),
// and the knobs Motolii shows for them: a colour three tracks (0..255, and an on/off when it may be nil), a choice a
// track of its choices' order.
struct AviutlVar { std::string var, kind, expr; double usual = 0; bool truth = false, unset = false, named = false; std::vector<double> values; };   // (named: --track@name, a variable of its own; else --track0, obj.track0)
std::vector<AviutlVar> aviutl_vars(const std::string& text);
std::vector<ScriptKnob> aviutl_knobs(const std::string& file);
// One embedded pixel shader made WGSL in the door's manner (screen_door.h: tN at binding 2N, sN at 2N+1, b0 after them,
// its entry `main`), `clip`: each sample transparent past the picture (the sampler "clip"). Its constants as Lua gives
// them (floats in HLSL's cbuffer packing) moved where WGSL lays them: `count` floats from byte `from` to byte `to`.
struct AviutlShader {
    std::string wgsl, why;
    int pictures = 1;
    uint32_t bytes = 0;
    std::vector<uint32_t> from, to, count;
};
const AviutlShader& aviutl_shader(const std::string& hlsl, const std::string& entry, bool clip);
