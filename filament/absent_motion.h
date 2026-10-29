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
// An end of a line as AbsentMotion draws it (absent_ends.cpp): where it is, the way the line goes there (a unit), as
// large as r, in a line as wide as width (cap 0 butt, 1 round, 2 square; join 0 miter, 1 round, 2 bevel; else its own):
// its outline to be filled, as SVG's path data, in the same points; none for a style it does not know.
std::string absent_line_end(float x, float y, float dx, float dy, float r, float width, int cap, int join, const char* style);
// A freehand stroke as AbsentMotion makes and shows it (absent_drawing.cpp): its points x y and pressure, in turn, as
// wide as width (a brush's radius its pressure times half of it), the tool (0 pen, 1 brush, 2 fill), its effects (Stroke
// Taper, Drawing Progress) at a moment: its outline to be filled, as SVG's path data, in the same units; in one piece
// (together: Skia's union, as long again as the rest, for a path to keep), else its pieces to be filled as one.
std::string absent_drawing(const std::vector<float>& xyp, float width, int tool, const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs, bool together);
// A motion blur or shutter blur on it (motion.cpp ShutterBlurOf): how long its shutter is open, in seconds at so many
// frames a second, and whether its way is smeared (AbsentMotion's quick smear takes the way of its middle alone:
// styles.cpp DrawShuttered); false: none is on it.
bool absent_shutter(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs, int fps, double* open, bool* moves);
struct AbsentCopy { double x = 0, y = 0, rotate = 0, scale = 1, alpha = 1; };
std::vector<AbsentCopy> absent_copies(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs, float wide, float high, float atX, float atY, float reach);
