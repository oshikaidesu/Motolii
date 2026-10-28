// absent_motion.h: AbsentMotion's own motion effects on a thing (absent_motion.cpp, built apart with AbsentMotion's
// sources and Skia's headers): each effect its id and its knobs' values by the names AbsentMotion reads them by (a point's
// or a colour's numbers in order), the thing's number (its randomness's seed, as AbsentMotion's layer id) and its time;
// what AbsentMotion's ApplyMotion moves it by, in its pixels (y down, degrees clockwise: Skia's way).
#pragma once
#include <cstdint>
#include <string>
#include <utility>
#include <vector>

struct AbsentEffect { std::string id; std::vector<std::pair<std::string, std::vector<float>>> params; };
struct AbsentMoved { double x = 0, y = 0, z = 0, rotate = 0, scale_x = 1, scale_y = 1; };
AbsentMoved absent_motion(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs);
