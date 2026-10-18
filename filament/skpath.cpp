// skpath.cpp: Skia's own path operations on a flat shape's path (Cavalry draws its shapes' paths with the same):
// union, difference, intersection, exclusion of two paths (SkPathOps, its Boolean), and a path read from SVG's path
// data (SkParsePath), a path roughened (SkDiscretePathEffect), a path's points moved. Apart from the rest because Skia's headers bring its names in; out come its calls (drawn.h).
// Built by skia.sh.
#include "thorvg.h"
#include "include/core/SkPath.h"
#include "include/core/SkPathBuilder.h"
#include "include/pathops/SkPathOps.h"
#include "include/utils/SkParsePath.h"
#include "include/core/SkStrokeRec.h"
#include "include/core/SkString.h"
#include "include/core/SkContourMeasure.h"
#include "include/effects/SkDiscretePathEffect.h"
#include <vector>

static SkPath sk_of(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts) {
    SkPathBuilder b;
    for (uint32_t c = 0, p = 0; c < cmdsCnt; ++c) switch (cmds[c]) {
        case tvg::PathCommand::MoveTo: b.moveTo(pts[p].x, pts[p].y), ++p; break;
        case tvg::PathCommand::LineTo: b.lineTo(pts[p].x, pts[p].y), ++p; break;
        case tvg::PathCommand::CubicTo: b.cubicTo(pts[p].x, pts[p].y, pts[p + 1].x, pts[p + 1].y, pts[p + 2].x, pts[p + 2].y), p += 3; break;
        case tvg::PathCommand::Close: b.close(); break;
    }
    return b.detach();
}

// Skia's path in ThorVG's commands, a quad or conic as a cubic (what ThorVG knows).
static bool tvg_of(const SkPath& made, std::vector<tvg::PathCommand>& cmds, std::vector<tvg::Point>& pts) {
    const auto cubic = [&](SkPoint a, SkPoint q, SkPoint e) {   // (a quad as the cubic it is)
        cmds.push_back(tvg::PathCommand::CubicTo);
        pts.push_back({a.fX + 2.f / 3.f * (q.fX - a.fX), a.fY + 2.f / 3.f * (q.fY - a.fY)});
        pts.push_back({e.fX + 2.f / 3.f * (q.fX - e.fX), e.fY + 2.f / 3.f * (q.fY - e.fY)});
        pts.push_back({e.fX, e.fY});
    };
    SkPathIter it = made.iter();
    while (const std::optional<SkPathIter::Rec> r = it.next()) {
        const SkSpan<const SkPoint> p = r->fPoints;
        switch (r->fVerb) {
            case SkPathVerb::kMove: cmds.push_back(tvg::PathCommand::MoveTo), pts.push_back({p[0].fX, p[0].fY}); break;
            case SkPathVerb::kLine: cmds.push_back(tvg::PathCommand::LineTo), pts.push_back({p[1].fX, p[1].fY}); break;
            case SkPathVerb::kQuad: cubic(p[0], p[1], p[2]); break;
            case SkPathVerb::kConic: {
                SkPoint q[1 + 2 * 8];
                const int n = SkPath::ConvertConicToQuads(p[0], p[1], p[2], r->conicWeight(), q, 3);
                for (int i = 0; i < n; ++i) cubic(q[2 * i], q[2 * i + 1], q[2 * i + 2]);
                break;
            }
            case SkPathVerb::kCubic:
                cmds.push_back(tvg::PathCommand::CubicTo);
                for (int i = 1; i < 4; ++i) pts.push_back({p[i].fX, p[i].fY});
                break;
            case SkPathVerb::kClose: cmds.push_back(tvg::PathCommand::Close); break;
        }
    }
    return !cmds.empty();
}

// `op`: 0 A less B, 1 where both are, 2 either, 3 either but not both (Skia's SkPathOp, in its order). false: nothing left.
bool sk_op(int op, const tvg::PathCommand* ac, uint32_t acn, const tvg::Point* ap, const tvg::PathCommand* bc, uint32_t bcn, const tvg::Point* bp,
           std::vector<tvg::PathCommand>& cmds, std::vector<tvg::Point>& pts) {
    cmds.clear(), pts.clear();
    const SkPath b = sk_of(bc, bcn, bp);
    const std::optional<SkPath> one = Simplify(b);   // (its pieces, if many and over each other, made one first)
    std::optional<SkPath> made = Op(sk_of(ac, acn, ap), one ? *one : b, (SkPathOp)op);
    if (made && made->getFillType() != SkPathFillType::kWinding) made = AsWinding(*made);   // (as it is filled: by its winding, as the shape is cut)
    return made && !made->isEmpty() && tvg_of(*made, cmds, pts);
}

// A path from SVG's path data ("M0 0 L1 1 C... Z"). false: none read.
bool sk_svg(const char* data, std::vector<tvg::PathCommand>& cmds, std::vector<tvg::Point>& pts) {
    cmds.clear(), pts.clear();
    const std::optional<SkPath> made = SkParsePath::FromSVGString(data);
    return made && !made->isEmpty() && tvg_of(*made, cmds, pts);
}

// A path roughened (Skia's discrete path effect): cut in pieces `apart` long, each end pushed up to `far` off at random.
bool sk_rough(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, float apart, float far, uint32_t seed,
              std::vector<tvg::PathCommand>& outCmds, std::vector<tvg::Point>& outPts) {
    outCmds.clear(), outPts.clear();
    SkPathBuilder made;
    SkStrokeRec fill(SkStrokeRec::kFill_InitStyle);
    return SkDiscretePathEffect::Make(apart, far, seed)->filterPath(&made, sk_of(cmds, cmdsCnt, pts), &fill) && tvg_of(made.detach(), outCmds, outPts);
}

// The points a path goes through (each stretch's end, its handles not), as SVG's path data has them: xy two a point.
int sk_svg_anchors(const char* data, std::vector<float>& xy) {
    xy.clear();
    const std::optional<SkPath> made = SkParsePath::FromSVGString(data);
    if (!made) return 0;
    SkPathIter it = made->iter();
    while (const std::optional<SkPathIter::Rec> r = it.next())
        if (r->fVerb != SkPathVerb::kClose) xy.push_back(r->fPoints.back().fX), xy.push_back(r->fPoints.back().fY);
    return (int)xy.size() / 2;
}

// A path with its nth point (and any point lying where it does: a closed path's first and last) moved by dx, dy, its
// handles on either side going with it, as a pen tool moves a point; written again as SVG's path data (Skia's own).
bool sk_svg_move(const char* data, int nth, float dx, float dy, std::string& out) {
    std::vector<float> xy;
    if (nth < 0 || nth >= sk_svg_anchors(data, xy)) return false;
    const SkPoint was{xy[2 * nth], xy[2 * nth + 1]}, by{dx, dy};
    const auto here = [&](SkPoint p) { return SkPoint::Distance(p, was) < 1e-4f; };
    SkPathBuilder b;
    bool after = false;   // (the stretch after a moved point: its first handle goes too)
    const SkPath path = *SkParsePath::FromSVGString(data);   // (kept: its iterator reads it)
    SkPathIter it = path.iter();
    while (const std::optional<SkPathIter::Rec> r = it.next()) {
        std::vector<SkPoint> p(r->fPoints.begin(), r->fPoints.end());
        const bool moved = r->fVerb != SkPathVerb::kClose && here(p.back());
        if (after && p.size() > 2) p[1] += by;
        if (moved) {
            p.back() += by;
            if (r->fVerb == SkPathVerb::kCubic) p[2] += by;
        }
        after = moved;
        switch (r->fVerb) {
            case SkPathVerb::kMove: b.moveTo(p[0]); break;
            case SkPathVerb::kLine: b.lineTo(p[1]); break;
            case SkPathVerb::kQuad: b.quadTo(p[1], p[2]); break;
            case SkPathVerb::kConic: b.conicTo(p[1], p[2], r->conicWeight()); break;
            case SkPathVerb::kCubic: b.cubicTo(p[1], p[2], p[3]); break;
            case SkPathVerb::kClose: b.close(); break;
        }
    }
    out = SkParsePath::ToSVGString(b.detach()).c_str();
    return true;
}

// A path turned `t` of the way into another, as Cavalry's Morph: each outline walked at 256 even steps (Skia's contour
// measure), the other's turned the same way round and started where it lies nearest, point to point between; an
// outline one has and the other has not, kept until half way.
bool sk_morph(const tvg::PathCommand* ac, uint32_t acn, const tvg::Point* ap, const tvg::PathCommand* bc, uint32_t bcn, const tvg::Point* bp, float t,
              std::vector<tvg::PathCommand>& cmds, std::vector<tvg::Point>& pts) {
    constexpr int N = 256;
    cmds.clear(), pts.clear();
    const auto walk = [](const SkPath& path) {
        std::vector<std::pair<std::vector<SkPoint>, bool>> all;
        SkContourMeasureIter it(path, false);
        while (const sk_sp<SkContourMeasure> c = it.next()) {
            std::vector<SkPoint> p(N);
            for (int i = 0; i < N; ++i) (void)c->getPosTan(c->length() * (float)i / (float)(c->isClosed() ? N : N - 1), &p[i], nullptr);
            all.push_back({p, c->isClosed()});
        }
        return all;
    };
    const auto a = walk(sk_of(ac, acn, ap)), b = walk(sk_of(bc, bcn, bp));
    const auto put = [&](const std::vector<SkPoint>& p, bool closed) {
        for (int i = 0; i < N; ++i) cmds.push_back(i ? tvg::PathCommand::LineTo : tvg::PathCommand::MoveTo), pts.push_back({p[i].fX, p[i].fY});
        if (closed) cmds.push_back(tvg::PathCommand::Close);
    };
    const auto area = [](const std::vector<SkPoint>& p) { double s = 0; for (int i = 0; i < N; ++i) s += (double)p[i].fX * p[(i + 1) % N].fY - (double)p[(i + 1) % N].fX * p[i].fY; return s; };
    for (size_t k = 0; k < std::max(a.size(), b.size()); ++k) {
        if (k >= b.size()) { if (t < 0.5f) put(a[k].first, a[k].second); continue; }
        if (k >= a.size()) { if (t >= 0.5f) put(b[k].first, b[k].second); continue; }
        const std::vector<SkPoint>& p = a[k].first;
        std::vector<SkPoint> q = b[k].first;
        int shift = 0;
        if (a[k].second && b[k].second) {   // (two rounds: the same way round, from the nearest start)
            if ((area(p) < 0) != (area(q) < 0)) std::reverse(q.begin(), q.end());
            double best = 1e300;
            for (int s = 0; s < N; ++s) {
                double sum = 0;
                for (int i = 0; i < N && sum < best; i += 4) { const SkPoint e = p[i] - q[(i + s) % N]; sum += (double)e.fX * e.fX + (double)e.fY * e.fY; }
                if (sum < best) best = sum, shift = s;
            }
        }
        std::vector<SkPoint> m(N);
        for (int i = 0; i < N; ++i) m[i] = p[i] + (q[(i + shift) % N] - p[i]) * t;
        put(m, a[k].second || b[k].second);
    }
    return !cmds.empty();
}
