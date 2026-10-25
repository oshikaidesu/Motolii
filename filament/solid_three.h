// A solid's shape made by three.js (solids/*.js, solid_three.cpp): a thing's picture laid on its faces (thing_fx.h).
#pragma once
#include <cstdint>
#include <map>
#include <string>
#include <vector>

// Whether the file is one: a .js that says `export function solid`.
bool solid_is(const std::string& file);
// Whether its shape is made of the thing's own outline (k.alpha: Raster Extrude): the picture is read back for it.
bool solid_reads(const std::string& file);
// Its points, normals, UVs (three's v up) and triangles; repeat: its picture repeats (geometry.userData.repeat).
struct SolidMade { std::vector<float> pos, nrm, uv; std::vector<uint32_t> idx; bool repeat = false; };
// The shape made with its knobs as said now and the thing's size (wide, high: its own units); `cover` the picture's
// coverage (0-255, coverW by coverH, rows from the top), for one that reads it. false: `why` says.
bool solid_three(const std::string& file, const std::map<std::string, double>& numbers, const std::map<std::string, std::string>& words,
                 double wide, double high, const std::vector<uint8_t>& cover, int coverW, int coverH, SolidMade& out, std::string& why);
