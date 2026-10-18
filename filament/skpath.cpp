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
#include "include/core/SkRegion.h"
#include "include/core/SkMatrix.h"
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

// A path's handles (each curve's control points, in order), as SVG's path data has them: xy two a handle; `of`, if
// asked, the point each is the handle of (a curve's first, the point it leaves; its last, the point it comes to).
int sk_svg_handles(const char* data, std::vector<float>& xy, std::vector<float>* of = nullptr) {
    xy.clear();
    if (of) of->clear();
    const std::optional<SkPath> made = SkParsePath::FromSVGString(data);
    if (!made) return 0;
    SkPathIter it = made->iter();
    while (const std::optional<SkPathIter::Rec> r = it.next())
        if (r->fVerb == SkPathVerb::kCubic || r->fVerb == SkPathVerb::kQuad || r->fVerb == SkPathVerb::kConic)
            for (size_t i = 1; i + 1 < r->fPoints.size(); ++i) {
                xy.push_back(r->fPoints[i].fX), xy.push_back(r->fPoints[i].fY);
                const SkPoint owner = i == 1 && r->fPoints.size() == 4 ? r->fPoints.front() : r->fPoints.back();
                if (of) of->push_back(owner.fX), of->push_back(owner.fY);
            }
    return (int)xy.size() / 2;
}

// A path with its nth handle moved by dx, dy; the handle on the other side of its point turned with it if the two lay
// in one line through it (a smooth point, as a pen tool keeps it smooth), as long as it was; written again as SVG's data.
bool sk_svg_move_handle(const char* data, int nth, float dx, float dy, std::string& out) {
    std::vector<float> xy;
    if (nth < 0 || nth >= sk_svg_handles(data, xy)) return false;
    const SkPath path = *SkParsePath::FromSVGString(data);   // (kept: its iterator reads it)
    struct Rec { SkPathVerb verb; std::vector<SkPoint> p; float w; };
    std::vector<Rec> all;
    SkPathIter it = path.iter();
    while (const std::optional<SkPathIter::Rec> r = it.next())
        all.push_back({r->fVerb, std::vector<SkPoint>(r->fPoints.begin(), r->fPoints.end()), r->fVerb == SkPathVerb::kConic ? r->conicWeight() : 1.f});
    const auto curved = [&](size_t v) { return all[v].verb == SkPathVerb::kCubic; };
    size_t hv = 0, hp = 0;   // (the handle: which stretch, which of its points)
    for (size_t v = 0, k = 0; v < all.size(); ++v)
        if (all[v].verb == SkPathVerb::kCubic || all[v].verb == SkPathVerb::kQuad || all[v].verb == SkPathVerb::kConic)
            for (size_t i = 1; i + 1 < all[v].p.size(); ++i, ++k) if ((int)k == nth) hv = v, hp = i;
    const SkPoint was = all[hv].p[hp];
    all[hv].p[hp] += SkPoint{dx, dy};
    if (curved(hv)) {   // (the other side of its point: the stretch before's last handle, or the next's first, round a closed outline)
        size_t first = hv, last = hv;
        while (first > 0 && all[first].verb != SkPathVerb::kMove) --first;
        while (last + 1 < all.size() && all[last + 1].verb != SkPathVerb::kMove) ++last;
        while (last > first && all[last].verb == SkPathVerb::kClose) --last;
        const bool closed = last + 1 < all.size() && all[last + 1].verb == SkPathVerb::kClose;
        size_t ov = SIZE_MAX, op = 0;
        SkPoint at{};
        if (hp == 2) {   // (its last handle: the point it comes to, and the next stretch's first)
            at = all[hv].p[3];
            size_t next = hv + 1 <= last ? hv + 1 : (closed && SkPoint::Distance(at, all[first].p[0]) < 1e-4f ? first + 1 : SIZE_MAX);
            if (next != SIZE_MAX && next <= last && curved(next)) ov = next, op = 1;
        } else {   // (its first: the point it leaves, and the stretch before's last)
            at = all[hv].p[0];
            size_t before = hv - 1 > first ? hv - 1 : (closed && SkPoint::Distance(all[last].p.back(), at) < 1e-4f ? last : SIZE_MAX);
            if (before != SIZE_MAX && before > first && curved(before)) ov = before, op = 2;
        }
        if (ov != SIZE_MAX) {
            SkVector a = was - at, b = all[ov].p[op] - at;
            const float la = a.length(), lb = b.length();
            if (la > 1e-6f && lb > 1e-6f && (a.fX * b.fX + a.fY * b.fY) / (la * lb) < -0.985f) {   // (in one line through it, within ten degrees)
                SkVector now = all[hv].p[hp] - at;
                if (now.length() > 1e-6f) all[ov].p[op] = at - now * (lb / now.length());
            }
        }
    }
    SkPathBuilder b;
    for (const Rec& r : all) switch (r.verb) {
        case SkPathVerb::kMove: b.moveTo(r.p[0]); break;
        case SkPathVerb::kLine: b.lineTo(r.p[1]); break;
        case SkPathVerb::kQuad: b.quadTo(r.p[1], r.p[2]); break;
        case SkPathVerb::kConic: b.conicTo(r.p[1], r.p[2], r.w); break;
        case SkPathVerb::kCubic: b.cubicTo(r.p[1], r.p[2], r.p[3]); break;
        case SkPathVerb::kClose: b.close(); break;
    }
    out = SkParsePath::ToSVGString(b.detach()).c_str();
    return true;
}

// A path as its points, each with the handle it comes in by and the one it leaves by (a line's on its ends), a
// contour each, and whether it is closed; and back. What a pen adds a point to or takes one from.
struct SkNode { SkPoint in, at, out; };
using SkNodes = std::vector<std::pair<std::vector<SkNode>, bool>>;
static SkNodes sk_nodes(const SkPath& path) {
    SkNodes all;
    SkPathIter it = path.iter();
    while (const std::optional<SkPathIter::Rec> r = it.next()) {
        const SkSpan<const SkPoint> p = r->fPoints;
        if (r->fVerb == SkPathVerb::kMove) { all.push_back({{{p[0], p[0], p[0]}}, false}); continue; }
        if (all.empty()) continue;
        std::vector<SkNode>& n = all.back().first;
        switch (r->fVerb) {
            case SkPathVerb::kLine: n.push_back({p[1], p[1], p[1]}); break;
            case SkPathVerb::kQuad: case SkPathVerb::kConic:   // (a quad as the cubic it is)
                n.back().out = p[0] + (p[1] - p[0]) * (2.f / 3.f), n.push_back({p[2] + (p[1] - p[2]) * (2.f / 3.f), p[2], p[2]}); break;
            case SkPathVerb::kCubic: n.back().out = p[1], n.push_back({p[2], p[3], p[3]}); break;
            case SkPathVerb::kClose:
                if (n.size() > 1 && SkPoint::Distance(n.back().at, n.front().at) < 1e-4f) n.front().in = n.back().in, n.pop_back();   // (its last point its first again: one)
                all.back().second = true;
                break;
            default: break;
        }
    }
    return all;
}
static SkPath sk_path(const SkNodes& all) {
    SkPathBuilder b;
    for (const auto& [n, closed] : all) {
        if (n.empty()) continue;
        b.moveTo(n[0].at);
        for (size_t i = 1; i < n.size(); ++i) b.cubicTo(n[i - 1].out, n[i].in, n[i].at);
        if (closed) b.cubicTo(n.back().out, n[0].in, n[0].at), b.close();
    }
    return b.detach();
}

// A pen on a path: on one of its points (within `reach`), that point taken away; on its outline, a point put there,
// the curve split where it lies (de Casteljau: the same curve); else nothing (0). 1 taken, 2 put.
int sk_svg_edit(const char* data, float x, float y, float reach, std::string& out) {
    const std::optional<SkPath> made = SkParsePath::FromSVGString(data);
    if (!made) return 0;
    SkNodes all = sk_nodes(*made);
    const SkPoint hand{x, y};
    for (auto& [n, closed] : all)
        for (size_t i = 0; i < n.size(); ++i)
            if (SkPoint::Distance(n[i].at, hand) < reach) {
                n.erase(n.begin() + (long)i);
                if (n.size() < 2) n.clear();
                out = SkParsePath::ToSVGString(sk_path(all)).c_str();
                return 1;
            }
    float best = reach, at = 0.f;
    int contour = -1;
    size_t seg = 0;
    const auto on = [](SkPoint a, SkPoint b, SkPoint c, SkPoint d, float t) { const float u = 1.f - t; return a * (u * u * u) + b * (3.f * u * u * t) + c * (3.f * u * t * t) + d * (t * t * t); };
    for (size_t k = 0; k < all.size(); ++k) {
        const auto& [n, closed] = all[k];
        for (size_t i = 0; i + (closed ? 0 : 1) < n.size(); ++i) {
            const SkNode &a = n[i], &b = n[(i + 1) % n.size()];
            for (int s = 1; s < 128; ++s)
                if (const float t = (float)s / 128.f, far = SkPoint::Distance(on(a.at, a.out, b.in, b.at, t), hand); far < best) best = far, at = t, contour = (int)k, seg = i;
        }
    }
    if (contour < 0) return 0;
    std::vector<SkNode>& n = all[(size_t)contour].first;
    SkNode &a = n[seg], &b = n[(seg + 1) % n.size()];
    const auto mix = [&](SkPoint p, SkPoint q) { return p + (q - p) * at; };
    const SkPoint ab = mix(a.at, a.out), bc = mix(a.out, b.in), cd = mix(b.in, b.at), abc = mix(ab, bc), bcd = mix(bc, cd), mid = mix(abc, bcd);
    a.out = ab, b.in = cd;
    n.insert(n.begin() + (long)seg + 1, SkNode{abc, mid, bcd});
    out = SkParsePath::ToSVGString(sk_path(all)).c_str();
    return 2;
}

// Each open outline's two ends as its trim leaves them (from, to: shares of its own length, as ThorVG trims each
// outline), where they are and the way the line goes there (Skia's contour measure): 8 numbers an outline, its end
// (x, y, dx, dy) then its start (x, y, dx, dy, pointing back out of it).
int sk_ends(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, float from, float to, std::vector<float>& out) {
    out.clear();
    if (to < from) std::swap(from, to);
    from = std::clamp(from, 0.f, 1.f), to = std::clamp(to, 0.f, 1.f);
    SkContourMeasureIter it(sk_of(cmds, cmdsCnt, pts), false);
    while (const sk_sp<SkContourMeasure> c = it.next()) {
        if (c->isClosed() || to - from < 1e-4f) continue;
        SkPoint e, s; SkVector et, st;
        if (!c->getPosTan(c->length() * to, &e, &et) || !c->getPosTan(c->length() * from, &s, &st)) continue;
        out.insert(out.end(), {e.fX, e.fY, et.fX, et.fY, s.fX, s.fY, -st.fX, -st.fY});
    }
    return (int)out.size() / 8;
}

// A path walked again at `n` even steps an outline (Skia's contour measure), straight between, as Cavalry's Resample
// Path: a round made a polygon, every outline as finely pointed as every other.
bool sk_resample(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, int n, std::vector<tvg::PathCommand>& outCmds, std::vector<tvg::Point>& outPts) {
    outCmds.clear(), outPts.clear();
    SkContourMeasureIter it(sk_of(cmds, cmdsCnt, pts), false);
    while (const sk_sp<SkContourMeasure> c = it.next()) {
        const int steps = c->isClosed() ? n : n - 1;
        for (int i = 0; i <= steps; ++i) {
            if (c->isClosed() && i == steps) break;
            SkPoint p;
            if (!c->getPosTan(c->length() * (float)i / (float)std::max(1, steps), &p, nullptr)) continue;
            outCmds.push_back(i ? tvg::PathCommand::LineTo : tvg::PathCommand::MoveTo), outPts.push_back({p.fX, p.fY});
        }
        if (c->isClosed()) outCmds.push_back(tvg::PathCommand::Close);
    }
    return !outCmds.empty();
}

// `n` places along a path, evenly by its length (all its outlines one after another; a closed path's last place not
// on its first), slid on by `from` (a share of its length, going round): x, y and the way it goes there (radians), three
// numbers a place (Skia's contour measure), as Cavalry's Duplicator on a shape's outline.
int sk_along(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, int n, float from, std::vector<float>& out) {
    out.clear();
    std::vector<sk_sp<SkContourMeasure>> all;
    float total = 0.f;
    bool closed = true;
    SkContourMeasureIter it(sk_of(cmds, cmdsCnt, pts), false);
    while (sk_sp<SkContourMeasure> c = it.next()) total += c->length(), closed = closed && c->isClosed(), all.push_back(std::move(c));
    if (all.empty() || total <= 0.f || n < 1) return 0;
    for (int i = 0; i < n; ++i) {
        float at = (float)i / (float)(closed || n < 2 ? n : n - 1) + from;
        at = closed ? at - std::floor(at) : std::clamp(at, 0.f, 1.f);
        float d = at * total;
        for (const sk_sp<SkContourMeasure>& c : all) {
            if (d > c->length() && &c != &all.back()) { d -= c->length(); continue; }
            SkPoint p; SkVector t;
            if (c->getPosTan(std::min(d, c->length()), &p, &t)) out.insert(out.end(), {p.fX, p.fY, std::atan2(t.fY, t.fX)});
            break;
        }
    }
    return (int)out.size() / 3;
}

// A path made of squares `cell` across, one where a grid's square has its middle inside it, as Cavalry's Voxelize:
// the path scan-converted on that grid by Skia's own SkRegion (a pixel is in where its middle is). At most 256 a side.
bool sk_voxel(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, float cell, std::vector<tvg::PathCommand>& outCmds, std::vector<tvg::Point>& outPts) {
    outCmds.clear(), outPts.clear();
    const SkPath path = sk_of(cmds, cmdsCnt, pts);
    const SkRect box = path.getBounds();
    if (box.isEmpty() || cell <= 0.f) return false;
    cell = std::max(cell, std::max(box.width(), box.height()) / 256.f);
    const float x0 = std::floor(box.fLeft / cell) * cell, y0 = std::floor(box.fTop / cell) * cell;
    const SkPath grid = path.makeTransform(SkMatrix::Translate(-x0, -y0).postScale(1.f / cell, 1.f / cell));   // (a square one pixel)
    SkRegion in;
    in.setPath(grid, SkRegion(SkIRect::MakeWH((int)std::ceil((box.fRight - x0) / cell) + 1, (int)std::ceil((box.fBottom - y0) / cell) + 1)));
    for (SkRegion::Iterator it(in); !it.done(); it.next())
        for (int y = it.rect().fTop; y < it.rect().fBottom; ++y)
            for (int x = it.rect().fLeft; x < it.rect().fRight; ++x) {
                const float px = x0 + (float)x * cell, py = y0 + (float)y * cell;
                outCmds.insert(outCmds.end(), {tvg::PathCommand::MoveTo, tvg::PathCommand::LineTo, tvg::PathCommand::LineTo, tvg::PathCommand::LineTo, tvg::PathCommand::Close});
                outPts.insert(outPts.end(), {{px, py}, {px + cell, py}, {px + cell, py + cell}, {px, py + cell}});
            }
    return !outCmds.empty();
}

// Whether a point lies inside a path (Skia's own contains, its winding) and how far it is from its outline (the nearest
// of its stretches, a curve walked in 16 steps), in the path's own units.
bool sk_inside(const tvg::PathCommand* cmds, uint32_t cmdsCnt, const tvg::Point* pts, float x, float y, float* far) {
    const SkPath path = sk_of(cmds, cmdsCnt, pts);
    const SkPoint at{x, y};
    float best = 1e30f;
    const auto seg = [&](SkPoint a, SkPoint b) {
        const SkVector ab = b - a, ap = at - a;
        const float l2 = ab.fX * ab.fX + ab.fY * ab.fY, u = l2 > 0.f ? std::clamp((ap.fX * ab.fX + ap.fY * ab.fY) / l2, 0.f, 1.f) : 0.f;
        best = std::min(best, SkPoint::Distance(at, a + ab * u));
    };
    SkPathIter it = path.iter();
    while (const std::optional<SkPathIter::Rec> r = it.next()) {
        const SkSpan<const SkPoint> p = r->fPoints;
        switch (r->fVerb) {
            case SkPathVerb::kLine: case SkPathVerb::kClose: seg(p[0], p[1]); break;
            case SkPathVerb::kQuad: case SkPathVerb::kConic: case SkPathVerb::kCubic: {
                const auto on = [&](float t) {
                    const float u = 1.f - t;
                    if (p.size() == 4) return p[0] * (u * u * u) + p[1] * (3.f * u * u * t) + p[2] * (3.f * u * t * t) + p[3] * (t * t * t);
                    return p[0] * (u * u) + p[1] * (2.f * u * t) + p[2] * (t * t);
                };
                for (int i = 0; i < 16; ++i) seg(on((float)i / 16.f), on((float)(i + 1) / 16.f));
                break;
            }
            default: break;
        }
    }
    if (far) *far = best;
    return path.contains(x, y);
}
