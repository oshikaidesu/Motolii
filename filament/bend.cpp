// bend.cpp: After Effects' path operators on a flat shape, done by ThorVG's own (its Lottie reader's modifiers:
// round corners, pucker & bloat, zig zag, offset paths), in that order, offset last as ThorVG chains them.
// Apart from the rest because ThorVG's inside headers bring all of its names in; out comes one call (drawn.h).
#include "tvgLottieModifier.h"

void tvg_bend(tvg::Shape* shape, const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, uint32_t ptsCnt,
              float round, float bloat, float zigzag, int ridges, bool smooth, float offset) {
    LottieModifier* chain = nullptr;
    const auto on = [&](LottieModifier* m) { chain = chain ? chain->decorate(m) : m; };   // (the one put on last is done first; offset goes to the end)
    if (zigzag != 0.f) on(new LottieZigZagModifier(zigzag, ridges, smooth ? LottieZigZagModifier::Smooth : LottieZigZagModifier::Corner));
    if (bloat != 0.f) on(new LottiePuckerBloatModifier(bloat));
    if (round > 0.f) on(new LottieRoundnessModifier(round));
    if (offset != 0.f) on(new LottieOffsetModifier(offset));
    RenderPath in, out;
    Point start{}, at{};
    const auto line = [&](const Point& to) {   // (as After Effects writes it, which its round corners look for and its pucker and bloat bend from; else even along it, for a zig zag's ridges)
        if (round > 0.f || bloat != 0.f) in.cubicTo(at, to, to);
        else in.cubicTo(at + (to - at) * (1.f / 3.f), at + (to - at) * (2.f / 3.f), to);
    };   // (they know only curves, as an After Effects path has: a line is a straight curve, and a closed path goes back to its start)
    for (uint32_t c = 0, p = 0; c < cmdsCnt; ++c) switch (cmds[c]) {
        case PathCommand::MoveTo: in.moveTo(start = at = pts[p++]); break;
        case PathCommand::LineTo: line(pts[p]), at = pts[p++]; break;
        case PathCommand::CubicTo: in.cubicTo(pts[p], pts[p + 1], pts[p + 2]), at = pts[p + 2], p += 3; break;
        case PathCommand::Close: if (!tvg::zero(at - start)) line(start); in.close(), at = start; break;
    }
    if (chain) chain->path(in, out, nullptr);
    const RenderPath& to = chain ? out : in;
    delete chain;
    shape->reset();
    shape->appendPath(to.cmds.data, to.cmds.count, to.pts.data, to.pts.count);
}
