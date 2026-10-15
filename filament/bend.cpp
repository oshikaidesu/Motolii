// bend.cpp: After Effects' path operators on a flat shape, done by ThorVG's own (its Lottie reader's modifiers:
// round corners, pucker & bloat, zig zag, offset paths), in that order, offset last as ThorVG chains them.
// Apart from the rest because ThorVG's inside headers bring all of its names in; out comes one call (drawn.h).
#include "tvgLottieModifier.h"

// A path as curves only, as an After Effects path is (a line a straight curve, a closed path back to its start: what
// the operators know), into `into`; `sharp`: a line's handles on its ends, as After Effects writes them (its round
// corners look for that, its pucker and bloat bend from it), else even along it (a zig zag's ridges); each line and
// curve cut in pieces no longer than `fine` (0: whole), for what bends it as it is drawn.
static void curves(RenderPath& into, const PathCommand* cmds, uint32_t cmdsCnt, const Point* pts, bool sharp, float fine) {
    Point start{}, at{};
    const auto line = [&](const Point& to) {
        const Point from = at;
        const int pieces = fine > 0.f ? std::max(1, (int)std::ceil(length(to - from) / fine)) : 1;
        for (int k = 1; k <= pieces; ++k) {
            const Point a = from + (to - from) * ((float)(k - 1) / (float)pieces), b = from + (to - from) * ((float)k / (float)pieces);
            if (sharp) into.cubicTo(a, b, b);
            else into.cubicTo(a + (b - a) * (1.f / 3.f), a + (b - a) * (2.f / 3.f), b);
        }
    };
    const auto curve = [&](const Point& c1, const Point& c2, const Point& to) {
        const Point c0 = at;
        const int pieces = fine > 0.f ? std::max(1, (int)std::ceil((length(c1 - c0) + length(c2 - c1) + length(to - c2)) / fine)) : 1;
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

void tvg_bend(tvg::Shape* shape, const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, uint32_t ptsCnt,
              float round, float bloat, float zigzag, int ridges, bool smooth, float offset, float fine) {
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
    if (fine > 0.f) curves(cut, bent.cmds.data, bent.cmds.count, bent.pts.data, false, fine);   // (cut after it is bent: the operators see its whole lines)
    const RenderPath& to = fine > 0.f ? cut : bent;
    shape->reset();
    shape->appendPath(to.cmds.data, to.cmds.count, to.pts.data, to.pts.count);
}
