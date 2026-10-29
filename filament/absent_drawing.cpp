// absent_drawing.cpp: a freehand stroke as AbsentMotion makes and shows it (its drawing.cpp: BuildGeometry, a brush a
// band of round pieces each as wide as its pressure says; EachPiece, Stroke Taper and Drawing Progress on it; Apache-2.0,
// Copyright 2026 The AbsentMotion authors). Its drawing.cpp is read whole into this file as it is, so that its own
// making of a stroke, kept to that file, is called from here; its pieces put together into one outline by Skia's own
// union (SkPathOps) when it is to be kept, else laid beside one another, to be filled as one (libtess2's nonzero, drawn_flat.h).
#include "drawing.cpp"
#include "absent_motion.h"
#include "include/pathops/SkPathOps.h"
#include "include/utils/SkParsePath.h"

am::Layer absent_layer(const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs);

std::string absent_drawing(const std::vector<float>& xyp, float width, int tool, const std::vector<AbsentEffect>& effects, int64_t thing, double startMs, double endMs, double nowMs, bool together) {
    constexpr float k = 1000.f;   // (counted in thousandths: Skia's curves and strokes are as fine as a pixel of what they are given)
    std::vector<SkPoint> pts;
    std::vector<float> pressure;
    for (size_t i = 0; i + 2 < xyp.size(); i += 3) pts.push_back({k * xyp[i], k * xyp[i + 1]}), pressure.push_back(std::max(0.0f, xyp[i + 2]));
    if (pts.empty()) return {};
    am::Layer l = absent_layer(effects, thing, startMs, endMs);
    am::FreehandStroke s;
    s.tool = tool, s.width = k * width;
    am::BuildGeometry(&s, pts, pressure);
    l.strokes.push_back(std::move(s));
    SkPath all;
    am::EachPiece(l, l.localTime(nowMs), [&](const am::FreehandStroke& one, const SkPath& geometry, bool filled) {
        am::FreehandStroke piece = one;   // (a pen's line as wide as it is, round at its ends and corners: its StrokedOutline)
        piece.geometry = geometry;
        const SkPath shape = filled ? geometry : am::StrokedOutline(piece);
        if (!together || !Op(all, shape, kUnion_SkPathOp, &all)) all = SkPathBuilder(all).addPath(shape).detach();   // (not put together: laid beside, filled as one)
    });
    return SkParsePath::ToSVGString(all.makeTransform(SkMatrix::Scale(1.f / k, 1.f / k))).c_str();
}
