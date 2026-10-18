// skpath.cpp: Skia's own path operations on a flat shape's path (Cavalry draws its shapes' paths with the same):
// union, difference, intersection, exclusion of two paths (SkPathOps, its Boolean). Apart from the rest because Skia's
// headers bring its names in; out comes one call (drawn.h). Built by skia.sh.
#include "thorvg.h"
#include "include/core/SkPath.h"
#include "include/core/SkPathBuilder.h"
#include "include/pathops/SkPathOps.h"
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

// `op`: 0 A less B, 1 where both are, 2 either, 3 either but not both (Skia's SkPathOp, in its order). Out: its path
// in ThorVG's commands, a quad or conic as a cubic (what ThorVG knows); false: nothing left.
bool sk_op(int op, const tvg::PathCommand* ac, uint32_t acn, const tvg::Point* ap, const tvg::PathCommand* bc, uint32_t bcn, const tvg::Point* bp,
           std::vector<tvg::PathCommand>& cmds, std::vector<tvg::Point>& pts) {
    cmds.clear(), pts.clear();
    const SkPath b = sk_of(bc, bcn, bp);
    const std::optional<SkPath> one = Simplify(b);   // (its pieces, if many and over each other, made one first)
    std::optional<SkPath> made = Op(sk_of(ac, acn, ap), one ? *one : b, (SkPathOp)op);
    if (made && made->getFillType() != SkPathFillType::kWinding) made = AsWinding(*made);   // (as it is filled: by its winding, as the shape is cut)
    if (!made || made->isEmpty()) return false;
    const auto cubic = [&](SkPoint a, SkPoint q, SkPoint e) {   // (a quad as the cubic it is)
        cmds.push_back(tvg::PathCommand::CubicTo);
        pts.push_back({a.fX + 2.f / 3.f * (q.fX - a.fX), a.fY + 2.f / 3.f * (q.fY - a.fY)});
        pts.push_back({e.fX + 2.f / 3.f * (q.fX - e.fX), e.fY + 2.f / 3.f * (q.fY - e.fY)});
        pts.push_back({e.fX, e.fY});
    };
    SkPathIter it = made->iter();
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
