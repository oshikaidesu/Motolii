// bend.cpp: After Effects' path operators on a flat shape, done by ThorVG's own (its Lottie reader's modifiers:
// round corners, pucker & bloat, zig zag, offset paths), in that order, offset last as ThorVG chains them.
// And a shape's edge as triangles, as ThorVG's own WebGPU engine strokes it (tvg_edge). Apart from the rest because
// ThorVG's inside headers bring all of its names in; out come two calls (drawn.h).
#include "tvgLottieModifier.h"
#include "tvgGpuCommon.h"
#include "tvgWgTessellator.h"
#include <vector>

// A path as curves only, as an After Effects path is (a line a straight curve, a closed path back to its start: what
// the operators know), into `into`; `sharp`: a line's handles on its ends, as After Effects writes them (its round
// corners look for that, its pucker and bloat bend from it), else even along it (a zig zag's ridges); each line and
// curve cut in pieces no longer than `fine` (0: whole), for what bends it as it is drawn.
static void curves(RenderPath& into, const PathCommand* cmds, uint32_t cmdsCnt, const Point* pts, bool sharp, float fine) {
    Point start{}, at{};
    // (how many pieces a stretch is cut into: by `fine`, at most 1024, and one where its length is no number: a shape
    //  carried far out or a fine near nothing once asked for more pieces than memory has)
    const auto cut = [&](float along) { return fine > 0.f && std::isfinite(along) ? (int)std::min(1024.f, std::max(1.f, std::ceil(along / fine))) : 1; };
    const auto line = [&](const Point& to) {
        const Point from = at;
        const int pieces = cut(length(to - from));
        for (int k = 1; k <= pieces; ++k) {
            const Point a = from + (to - from) * ((float)(k - 1) / (float)pieces), b = from + (to - from) * ((float)k / (float)pieces);
            if (sharp) into.cubicTo(a, b, b);
            else into.cubicTo(a + (b - a) * (1.f / 3.f), a + (b - a) * (2.f / 3.f), b);
        }
    };
    const auto curve = [&](const Point& c1, const Point& c2, const Point& to) {
        const Point c0 = at;
        const int pieces = cut(length(c1 - c0) + length(c2 - c1) + length(to - c2));
        if (pieces == 1) return into.cubicTo(c1, c2, to);
        const auto on = [&](float t) {   // (where it is at t, and which way it goes there)
            const float u = 1.f - t;
            return std::pair<Point, Point>{c0 * (u * u * u) + c1 * (3.f * u * u * t) + c2 * (3.f * u * t * t) + to * (t * t * t),
                                           (c1 - c0) * (3.f * u * u) + (c2 - c1) * (6.f * u * t) + (to - c2) * (3.f * t * t)};
        };
        for (int k = 1; k <= pieces; ++k) {
            const float t0 = (float)(k - 1) / (float)pieces, t1 = (float)k / (float)pieces, d = (t1 - t0) / 3.f;
            const auto [a, da] = on(t0);
            const auto [b, db] = on(t1);
            into.cubicTo(a + da * d, b - db * d, b);
        }
    };
    for (uint32_t c = 0, p = 0; c < cmdsCnt; ++c) switch (cmds[c]) {
        case PathCommand::MoveTo: into.moveTo(start = at = pts[p++]); break;
        case PathCommand::LineTo: line(pts[p]), at = pts[p++]; break;
        case PathCommand::CubicTo: curve(pts[p], pts[p + 1], pts[p + 2]), at = pts[p + 2], p += 3; break;
        case PathCommand::Close: if (!tvg::zero(at - start)) line(start); into.close(), at = start; break;
    }
}

// (whether every point is a number: one carried to no number, a shape pushed far past its knobs, sends ThorVG's own
//  measuring round for ever; such a shape is not drawn)
static bool numbers(const Point* p, uint32_t n) {
    for (uint32_t i = 0; i < n; ++i) if (!std::isfinite(p[i].x) || !std::isfinite(p[i].y)) return false;
    return true;
}

void tvg_bend(tvg::Shape* shape, const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, uint32_t ptsCnt,
              float round, float bloat, float zigzag, int ridges, bool smooth, float offset, float fine) {
    shape->reset();
    if (!numbers(pts, ptsCnt) || !std::isfinite(round + bloat + zigzag + offset + fine)) return;
    LottieModifier* chain = nullptr;
    const auto on = [&](LottieModifier* m) { chain = chain ? chain->decorate(m) : m; };   // (the one put on last is done first; offset goes to the end)
    if (zigzag != 0.f) on(new LottieZigZagModifier(zigzag, ridges, smooth ? LottieZigZagModifier::Smooth : LottieZigZagModifier::Corner));
    if (bloat != 0.f) on(new LottiePuckerBloatModifier(bloat));
    if (round > 0.f) on(new LottieRoundnessModifier(round));
    if (offset != 0.f) on(new LottieOffsetModifier(offset));
    RenderPath in, out, cut;
    curves(in, cmds, cmdsCnt, pts, round > 0.f || bloat != 0.f, 0.f);
    const bool any = chain;
    if (chain) chain->path(in, out, nullptr);
    delete chain;
    const RenderPath& bent = any ? out : in;
    if (!numbers(bent.pts.data, bent.pts.count)) return;
    if (fine > 0.f) curves(cut, bent.cmds.data, bent.cmds.count, bent.pts.data, false, fine);   // (cut after it is bent: the operators see its whole lines)
    const RenderPath& to = fine > 0.f ? cut : bent;
    shape->appendPath(to.cmds.data, to.cmds.count, to.pts.data, to.pts.count);
}

// A shape's edge as triangles, straight, as ThorVG's own WebGPU engine makes them (its trim, its dash, its stroker:
// nothing cut by a tessellator): so wide (points), its cap and join (ThorVG's), dashed as long as its gaps (0: not),
// the part from `from` to `to` of its length (After Effects' trim paths, as ThorVG's trimpath goes round past 1).
void tvg_edge(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, uint32_t ptsCnt, float width, int cap, int join, float dash, float dashFrom,
              float from, float to, std::vector<float>& xy, std::vector<uint32_t>& idx) {
    xy.clear(), idx.clear();
    if (!numbers(pts, ptsCnt) || !std::isfinite(width + dash + dashFrom + from + to)) return;
    RenderShape rs;
    for (uint32_t i = 0; i < cmdsCnt; ++i) rs.path.cmds.push(cmds[i]);
    for (uint32_t i = 0; i < ptsCnt; ++i) rs.path.pts.push(pts[i]);
    rs.stroke = new RenderStroke;
    rs.stroke->width = width, rs.stroke->cap = (StrokeCap)cap, rs.stroke->join = (StrokeJoin)join;
    rs.stroke->trim.begin = from, rs.stroke->trim.end = to;
    if (dash > 0.f) {
        rs.stroke->dash.pattern = tvg::malloc<float>(2 * sizeof(float));
        rs.stroke->dash.pattern[0] = rs.stroke->dash.pattern[1] = dash;
        rs.stroke->dash.count = 2, rs.stroke->dash.offset = dashFrom, rs.stroke->dash.length = 2.f * dash;
    }
    RenderPath stroked;
    const RenderPath* along = &rs.path;
    if (gpuStrokeDash(rs, stroked, nullptr)) along = &stroked;
    else if (rs.trimpath()) {
        if (!rs.stroke->trim.trim(rs.path, stroked)) return;   // (trimmed to nothing)
        along = &stroked;
    }
    WgMesh mesh;
    WgStroker stroker(&mesh, width, (StrokeCap)cap, (StrokeJoin)join);
    stroker.run(*along);
    for (uint32_t i = 0; i < mesh.vbuffer.count; ++i) xy.push_back(mesh.vbuffer[i].x), xy.push_back(mesh.vbuffer[i].y);
    for (uint32_t i = 0; i < mesh.ibuffer.count; ++i) idx.push_back(mesh.ibuffer[i]);
}
