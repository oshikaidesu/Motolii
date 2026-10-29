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
// Its repeat effects' copies (Repeat, Linear, Grid, Radial, Scatter Field: motion.cpp RepeatCopies), each where it lies
// from the layer's own place, in its pixels; none: no repeat. The composition's size, where the layer stands on it (from
// the top left) and how far it reaches from there, as AbsentMotion's CopiesOf tells them (a scatter field is laid on the
// whole composition, its copies as large as the layer reaches).
// Its opacity effects' share of the layer's opacity (Pulse Opacity: motion.cpp MotionOpacity), 1 when none.
float absent_opacity(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs);
// A text's words as AbsentMotion's text effects on it show them now (Count Up/Down, Timecode, Text Randomizer: its
// text.cpp Shown, by its textanim.cpp), its frames so many a second; as typed when none of them is on it.
bool absent_text_on(const std::vector<AbsentEffect>& effects);
std::string absent_text(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs, int fps, std::string text);
struct AbsentCopy { double x = 0, y = 0, rotate = 0, scale = 1, alpha = 1; };
std::vector<AbsentCopy> absent_copies(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs, float wide, float high, float atX, float atY, float reach);
