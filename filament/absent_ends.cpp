// absent_ends.cpp: AbsentMotion's own ends of a line (Alight Motion's line start and end: arrow, t-line, square,
// diamond, arrow, circle and teardrop, hollow or filled: its styles.cpp DrawStrokeEnd; Apache-2.0, Copyright 2026 The
// AbsentMotion authors). Its styles.cpp is read whole into this file as it is, so that its own drawing of an end, kept
// to that file, is called from here; what else of it is not reached is stripped (dawn_probe/kit_wgpu.sh, -dead_strip).
#include "styles.cpp"
#include "absent_motion.h"
#include "include/utils/SkParsePath.h"

std::string absent_line_end(float x, float y, float dx, float dy, float r, float width, int cap, int join, const char* style) {
    SkPaint paint;   // (as its StrokePaint makes a line's: round and bevelled unless the line says otherwise)
    paint.setAntiAlias(true);
    paint.setStrokeWidth(width);
    paint.setStrokeCap(cap == 0 ? SkPaint::kButt_Cap : cap == 2 ? SkPaint::kSquare_Cap : SkPaint::kRound_Cap);
    paint.setStrokeJoin(join == 0 ? SkPaint::kMiter_Join : join == 1 ? SkPaint::kRound_Join : SkPaint::kBevel_Join);
    return SkParsePath::ToSVGString(am::DrawStrokeEnd(nullptr, paint, {x, y}, {dx, dy}, r, style)).c_str();
}
