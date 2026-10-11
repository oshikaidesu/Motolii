// Glass solids with dents and holes: an outline (straight lines and arcs of circles, any number of holes) pushed
// straight through a thickness. Stars, washers, letters.
//
// Included at the end of beam_glass.h (which declares the few functions of this file that it calls).
//
// The solid in its own frame: the outline lies in x, y, the thickness runs from z = -h to z = +h. The two flat
// faces are called caps, the faces round the outline walls. The rims between a cap and a wall carry a 45 degree
// chamfer (the same protective cut the other solids have; it is where most of the colour comes from).
//
// The light is not followed ray against solid. It is put together from pictures, each of which is either the
// outline itself, copied, or a strip that runs along the outline and is known in closed form across:
//   - the shadow: the outline (once), plus one strip per wall that looks away from the sun, as long as that
//     wall's own shadow is (shorter where another part of the solid stands behind it). Together they cover the
//     solid's shadow exactly once, with the holes open;
//   - the window: light that went in through one cap and out through the other. It is the outline again,
//     shifted by the sideways step the light takes inside the glass (a little different for every colour);
//   - what the walls mirror of that light: whatever in the shifted outline lies outside the outline has met a
//     wall on its way (always totally reflected: a cap and a wall are at right angles). Per wall that is a strip:
//     taken out of the window as it is, put back mirrored about the wall (and about one more wall it may meet);
//   - the light that goes in through a wall: across the outline it is a problem in the plane (the caps only
//     mirror it, always totally, so along the thickness it is folded back and forth), followed once per point of
//     the wall and unfolded exactly across the thickness. Walls that are not parallel split the colours;
//   - the chamfers: a thin strip of light per rim, in through the cut or out through it: thin lines of colour;
//   - the sun mirrored off the outside of the caps and walls.
//
// Who does what. Everything about a shape is built ONCE (ex_shape_table, ex_strip_mesh, ex_cap_mesh): its edges,
// the stretches its walls are divided into, its triangles. Per frame a renderer hands over only where each solid
// is (ex_pose). The rest is done on the GPU in two steps, as for the cone:
//   extrude_walk.mat   a fragment pass: for every stretch of wall of every solid, what becomes of the light at
//                      its start, and if the light at its two ends takes different ways (a corner of the outline
//                      comes in between) the place where the way changes, found by halving, with the light just
//                      before and just after it. One line of a float texture per kind of light and colour step.
//   extrude_strip.mat  the strips: one mesh for all shapes, drawn per (solid, colour step), every corner placed
//   extrude_cap.mat    from that texture; and the outline's triangles, shifted or slid along the sun.
// Because a strip is cut exactly where the way changes, nothing pops when the solid turns. Brightness per corner
// is the light caught over the floor it lands on, measured on the very pieces that are drawn; what leaves nearly
// level or lands far away fades to nothing first.
// The functions below (ex_walk_texel, ex_strip_vertex, ex_cap_vertex) are those shaders line for line, on the
// CPU: for the desk check and for a renderer told to compare. Change them together.
//
// Left out on purpose: light that leaves the solid and meets it again across a dent or a hole flies on as if
// nothing were there; other solids neither shade this light nor bend it (as for balls and cones); the chamfers
// only carry light that came in through a cap or through themselves; light from a cap is followed past two walls,
// no further; where two changes of way fall into one stretch of wall only the first is cut exactly.
#pragma once

#include <memory>

namespace beam {

struct V2 {
    double x, y;
};
inline V2 operator+(V2 a, V2 b) { return {a.x + b.x, a.y + b.y}; }
inline V2 operator-(V2 a, V2 b) { return {a.x - b.x, a.y - b.y}; }
inline V2 operator*(V2 a, double s) { return {a.x * s, a.y * s}; }
inline double dot(V2 a, V2 b) { return a.x * b.x + a.y * b.y; }
inline double cross(V2 a, V2 b) { return a.x * b.y - a.y * b.x; }
inline double length(V2 a) { return std::sqrt(dot(a, a)); }

constexpr double EXT_ARC_STEP = 7.5 * 3.14159265358979323846 / 180.0;  // an arc is drawn as chords this long (in angle)
constexpr double EXT_LINE_STEP = 0.08;  // a straight wall is divided into stretches at most this long (the outline is about 1 across)
constexpr double EXT_TMIN = 1e-5;       // a ray that starts on the outline does not meet it again right there
constexpr int EXT_HALVINGS = 10;        // how finely the place where the light changes its way is found (a thousandth of a stretch)
constexpr int EXT_BOUNCES = 8;          // total reflections followed inside for light that came in through a wall or a chamfer
constexpr double EXT_CAP = 40.0;        // a point of light may be this many times the sun, no more
constexpr double EXT_GAP = 1.0;         // colour steps are taken to leave at most this far apart (in direction)
constexpr double EXT_EDGE = 0.25;        // light that only just gets out of the glass, or is only just kept in, is dimmed over this much
                                        // (of n^2 sin^2, about 6 degrees), so that a patch does not appear at full strength when the glass turns
constexpr double EXT_SOFT = 0.1;        // and are softened into one another over at most this much of the solid's size
constexpr double EXT_BIG = 1e30;
constexpr double EXT_OVER = 2e-3;       // strips that must leave no crack reach this far over their edge
constexpr int EXT_CAP_STEPS = 4;        // colour steps of the light through the caps (it only shifts with colour, by a pixel or two)
constexpr int EXT_STEPS = 8;            // colour steps of the light through walls and chamfers
// The lines of the textures the two fragment passes write, per solid (see ex_find_texel, ex_walk_texel):
constexpr int EXT_FIND_ROWS = 2 + EXT_STEPS;
constexpr int EXT_ROW_SHADOW = 0, EXT_ROW_MIRROR = 2, EXT_ROW_WALL = 7, EXT_ROW_CUT = EXT_ROW_WALL + 10 * EXT_STEPS, EXT_ROWS = EXT_ROW_CUT + 4 * EXT_STEPS;
constexpr int EXT_COLUMNS = 160;        // most stretches of wall one outline may have
constexpr int EXT_TABLE = 512;          // width of the table of shapes

// One piece of the outline, from a to b, with the glass on its left.
struct OutlineEdge {
    V2 a, b;
    V2 c{0, 0};    // arc: the middle of its circle
    double r = 0;  // 0: straight. > 0: an arc going round c counter-clockwise (it bulges out of the glass).
                   // < 0: clockwise (a hollow, such as the wall of a round hole). Never more than half a turn.
    // worked out by Outline::finish()
    double a0 = 0, sweep = 0;  // arc: the angle at a, and how far round it goes (signed)
    int chords = 1;            // how many straight pieces it is drawn as
    int first = 0;             // its first point in Outline::pts
};

inline std::vector<int> outline_triangulate(const std::vector<V2>& p, const std::vector<std::vector<int>>& loops);

struct Outline {
    std::vector<OutlineEdge> edges;  // loop after loop: the outside counter-clockwise, holes clockwise
    std::vector<int> next;           // the edge that follows in its loop
    double h = 0.1;                  // half the thickness
    double chamfer = 0;              // flat width of the cut on the rims
    // worked out by finish()
    std::vector<V2> pts;                   // the outline as a many-sided figure: every edge's chords, in order
    std::vector<std::vector<int>> loops;   // indices into pts
    std::vector<int> tris;                 // that figure cut into triangles (three indices into pts each)
    std::vector<V2> inset;                 // pts moved in by the chamfer: the rim of the caps
    std::vector<int> insetTris;
    V2 lo{0, 0}, hi{0, 0};                 // its box
    double reach = 0;                      // the furthest point of the outline from the origin
    struct Stretch {
        int edge;
        double u0, u1;  // from where to where along the edge (0 to 1); u0 == u1: the end of the edge, nothing to draw
    };
    std::vector<Stretch> stretches;        // every edge divided into the stretches its strips of light are made of

    // ---- drawing it: move, then lines and arcs, then close (again for every hole)
    void move(double x, double y) {
        close();
        pen = start = V2{x, y};
        loopFirst = (int)edges.size();
    }
    void line(double x, double y) {
        if (length(V2{x, y} - pen) < 1e-12) return;
        OutlineEdge e;
        e.a = pen;
        e.b = V2{x, y};
        edges.push_back(e);
        pen = e.b;
    }
    // to (x, y) round (cx, cy); the start must be as far from the middle as the end
    void arc(double x, double y, double cx, double cy, bool counterClockwise) {
        const double pi = 3.14159265358979323846;
        const V2 c{cx, cy};
        const double rad = length(pen - c), a0 = std::atan2(pen.y - cy, pen.x - cx);
        double a1 = std::atan2(y - cy, x - cx), sweep = a1 - a0;
        if (counterClockwise) {
            while (sweep <= 1e-12) sweep += 2.0 * pi;
        } else {
            while (sweep >= -1e-12) sweep -= 2.0 * pi;
        }
        const int parts = std::fabs(sweep) > pi * (1.0 + 1e-9) ? (int)std::ceil(std::fabs(sweep) / pi - 1e-9) : 1;
        for (int i = 1; i <= parts; ++i) {
            OutlineEdge e;
            e.a = pen;
            const double a = a0 + sweep * i / parts;
            e.b = i == parts ? V2{x, y} : V2{cx + rad * std::cos(a), cy + rad * std::sin(a)};
            e.c = c;
            e.r = counterClockwise ? rad : -rad;
            edges.push_back(e);
            pen = e.b;
        }
    }
    void close() {
        if (loopFirst < 0) return;
        line(start.x, start.y);
        if (!edges.empty()) edges.back().b = start;
        next.resize(edges.size());
        for (int i = loopFirst; i < (int)edges.size(); ++i) next[(size_t)i] = i + 1 < (int)edges.size() ? i + 1 : loopFirst;
        loopFirst = -1;
    }
    V2 point(int e, double u) const {
        const OutlineEdge& ed = edges[(size_t)e];
        if (u <= 0) return ed.a;
        if (u >= 1) return ed.b;
        if (ed.r == 0) return ed.a + (ed.b - ed.a) * u;
        const double a = ed.a0 + ed.sweep * u, rad = std::fabs(ed.r);
        return V2{ed.c.x + rad * std::cos(a), ed.c.y + rad * std::sin(a)};
    }
    // out of the glass
    V2 normal(int e, double u) const {
        const OutlineEdge& ed = edges[(size_t)e];
        if (ed.r == 0) {
            const V2 ab = ed.b - ed.a;
            return V2{ab.y, -ab.x} * (1.0 / length(ab));
        }
        const double a = ed.a0 + ed.sweep * u, sg = ed.r > 0 ? 1.0 : -1.0;
        return V2{sg * std::cos(a), sg * std::sin(a)};
    }
    // Centres the outline on its box and works out everything else. Call once, after the last close().
    void finish() {
        close();
        const double pi = 3.14159265358979323846;
        // the box (arcs: their chords are close enough for this)
        auto each = [&](auto&& f) {
            for (OutlineEdge& e : edges) {
                f(e.a);
                if (e.r != 0) {
                    const double a0 = std::atan2(e.a.y - e.c.y, e.a.x - e.c.x);
                    double sw = std::atan2(e.b.y - e.c.y, e.b.x - e.c.x) - a0;
                    if (e.r > 0) {
                        while (sw <= 1e-12) sw += 2.0 * pi;
                    } else {
                        while (sw >= -1e-12) sw -= 2.0 * pi;
                    }
                    for (int i = 1; i < 16; ++i) {
                        const double a = a0 + sw * i / 16.0;
                        f(V2{e.c.x + std::fabs(e.r) * std::cos(a), e.c.y + std::fabs(e.r) * std::sin(a)});
                    }
                }
            }
        };
        lo = V2{1e300, 1e300};
        hi = V2{-1e300, -1e300};
        each([&](V2 p) {
            lo = V2{std::min(lo.x, p.x), std::min(lo.y, p.y)};
            hi = V2{std::max(hi.x, p.x), std::max(hi.y, p.y)};
        });
        const V2 mid = (lo + hi) * 0.5;
        for (OutlineEdge& e : edges) {
            e.a = e.a - mid;
            e.b = e.b - mid;
            if (e.r != 0) e.c = e.c - mid;
        }
        lo = lo - mid;
        hi = hi - mid;
        // angles, chords, points
        pts.clear();
        for (OutlineEdge& e : edges) {
            if (e.r != 0) {
                e.a0 = std::atan2(e.a.y - e.c.y, e.a.x - e.c.x);
                e.sweep = std::atan2(e.b.y - e.c.y, e.b.x - e.c.x) - e.a0;
                if (e.r > 0) {
                    while (e.sweep <= 1e-12) e.sweep += 2.0 * pi;
                } else {
                    while (e.sweep >= -1e-12) e.sweep -= 2.0 * pi;
                }
                e.chords = std::max(1, (int)std::ceil(std::fabs(e.sweep) / EXT_ARC_STEP - 1e-9));
            } else {
                e.chords = 1;
            }
        }
        loops.clear();
        std::vector<char> seen(edges.size(), 0);
        for (size_t s = 0; s < edges.size(); ++s) {
            if (seen[s]) continue;
            loops.emplace_back();
            for (int e = (int)s; !seen[(size_t)e]; e = next[(size_t)e]) {
                seen[(size_t)e] = 1;
                edges[(size_t)e].first = (int)pts.size();
                for (int j = 0; j < edges[(size_t)e].chords; ++j) {
                    loops.back().push_back((int)pts.size());
                    pts.push_back(point(e, (double)j / edges[(size_t)e].chords));
                }
            }
        }
        reach = 0;
        for (V2 p : pts) reach = std::max(reach, length(p));
        tris = outline_triangulate(pts, loops);
        // the rim of the caps: every point moved in by the leg of the chamfer (at a corner: along both walls)
        inset = pts;
        const double leg = chamfer / std::sqrt(2.0);
        if (leg > 0) {
            for (size_t ei = 0; ei < edges.size(); ++ei) {
                const OutlineEdge& e = edges[ei];
                for (int j = 0; j < e.chords; ++j) {
                    V2 n1 = normal((int)ei, (double)j / e.chords), n0 = n1;
                    if (j == 0) {
                        int prev = 0;
                        for (size_t k = 0; k < edges.size(); ++k)
                            if (next[k] == (int)ei) prev = (int)k;
                        n0 = normal(prev, 1.0);
                    }
                    const double k = std::max(0.25, 1.0 + dot(n0, n1));
                    inset[(size_t)(e.first + j)] = pts[(size_t)(e.first + j)] - (n0 + n1) * (leg / k);
                }
            }
        }
        insetTris = leg > 0 ? outline_triangulate(inset, loops) : tris;
        stretches.clear();
        for (size_t ei = 0; ei < edges.size(); ++ei) {
            const OutlineEdge& e = edges[ei];
            const int parts = e.r != 0 ? e.chords : std::max(1, (int)std::ceil(length(e.b - e.a) / EXT_LINE_STEP));
            for (int j = 0; j < parts; ++j) stretches.push_back(Stretch{(int)ei, (double)j / parts, (double)(j + 1) / parts});
            stretches.push_back(Stretch{(int)ei, 1.0, 1.0});
        }
    }

  private:
    V2 pen{0, 0}, start{0, 0};
    int loopFirst = -1;
};

// Cuts a figure with holes into triangles (ears, after every hole has been joined to the outside by a bridge).
// loops: indices into p; counter-clockwise loops are outsides, clockwise ones holes. Small figures only (n^3).
inline std::vector<int> outline_triangulate(const std::vector<V2>& p, const std::vector<std::vector<int>>& loops) {
    std::vector<int> out;
    auto area2 = [&](const std::vector<int>& l) {
        double a = 0;
        for (size_t i = 0; i < l.size(); ++i) a += cross(p[(size_t)l[i]], p[(size_t)l[(i + 1) % l.size()]]);
        return a;
    };
    auto inside = [&](V2 q, const std::vector<int>& l) {
        bool in = false;
        for (size_t i = 0, j = l.size() - 1; i < l.size(); j = i++) {
            const V2 a = p[(size_t)l[i]], b = p[(size_t)l[j]];
            if ((a.y > q.y) != (b.y > q.y) && q.x < (b.x - a.x) * (q.y - a.y) / (b.y - a.y) + a.x) in = !in;
        }
        return in;
    };
    auto same = [&](int i, int j) { return i == j || (p[(size_t)i].x == p[(size_t)j].x && p[(size_t)i].y == p[(size_t)j].y); };
    // do the segments a-b and c-d cross (touching at a shared end does not count)?
    auto crosses = [&](int a, int b, int c, int d) {
        if (same(a, c) || same(a, d) || same(b, c) || same(b, d)) return false;
        const V2 A = p[(size_t)a], B = p[(size_t)b], C = p[(size_t)c], D = p[(size_t)d];
        const double d1 = cross(B - A, C - A), d2 = cross(B - A, D - A), d3 = cross(D - C, A - C), d4 = cross(D - C, B - C);
        return ((d1 > 0) != (d2 > 0)) && ((d3 > 0) != (d4 > 0));
    };
    for (size_t o = 0; o < loops.size(); ++o) {
        if (!(area2(loops[o]) > 0)) continue;
        std::vector<int> poly = loops[o];
        std::vector<const std::vector<int>*> holes;
        for (size_t hI = 0; hI < loops.size(); ++hI)
            if (area2(loops[hI]) < 0 && inside(p[(size_t)loops[hI][0]], loops[o])) holes.push_back(&loops[hI]);
        // the hole that reaches furthest right first
        auto rightmost = [&](const std::vector<int>& l) {
            size_t m = 0;
            for (size_t i = 1; i < l.size(); ++i)
                if (p[(size_t)l[i]].x > p[(size_t)l[m]].x) m = i;
            return m;
        };
        std::sort(holes.begin(), holes.end(), [&](const std::vector<int>* a, const std::vector<int>* b) {
            const double xa = p[(size_t)(*a)[rightmost(*a)]].x, xb = p[(size_t)(*b)[rightmost(*b)]].x;
            return xa != xb ? xa > xb : a < b;
        });
        for (size_t hI = 0; hI < holes.size(); ++hI) {
            const std::vector<int>& hole = *holes[hI];
            const size_t m = rightmost(hole);
            const int M = hole[m];
            // the nearest corner of the figure so far that M can see
            int best = -1;
            double bd = 1e300;
            for (size_t i = 0; i < poly.size(); ++i) {
                const int P = poly[i];
                const double d = length(p[(size_t)P] - p[(size_t)M]);
                if (d >= bd) continue;
                bool clear = true;
                for (size_t k = 0; k < poly.size() && clear; ++k) clear = !crosses(M, P, poly[k], poly[(k + 1) % poly.size()]);
                for (size_t g = hI; g < holes.size() && clear; ++g)
                    for (size_t k = 0; k < holes[g]->size() && clear; ++k) clear = !crosses(M, P, (*holes[g])[k], (*holes[g])[(k + 1) % holes[g]->size()]);
                if (!clear) continue;
                // M must lie in the corner at this place of the list (a point may be in the list more than once)
                const V2 A = p[(size_t)poly[(i + poly.size() - 1) % poly.size()]], B = p[(size_t)P], C = p[(size_t)poly[(i + 1) % poly.size()]], Q = p[(size_t)M];
                const bool convex = cross(B - A, C - B) >= 0;
                const bool l1 = cross(B - A, Q - A) > 0, l2 = cross(C - B, Q - B) > 0;
                if (convex ? !(l1 && l2) : !(l1 || l2)) continue;
                bd = d;
                best = (int)i;
            }
            if (best < 0) best = 0;  // (never for a sane figure)
            std::vector<int> joined(poly.begin(), poly.begin() + best + 1);
            for (size_t k = 0; k <= hole.size(); ++k) joined.push_back(hole[(m + k) % hole.size()]);
            joined.push_back(poly[(size_t)best]);
            joined.insert(joined.end(), poly.begin() + best + 1, poly.end());
            poly.swap(joined);
        }
        // ears
        while (poly.size() >= 3) {
            const size_t n = poly.size();
            size_t ear = n;
            double widest = -1e300;
            size_t fallback = 0;
            for (size_t i = 0; i < n && ear == n; ++i) {
                const int ia = poly[(i + n - 1) % n], ib = poly[i], ic = poly[(i + 1) % n];
                const V2 A = p[(size_t)ia], B = p[(size_t)ib], C = p[(size_t)ic];
                const double turn = cross(B - A, C - B);
                if (turn > widest) {
                    widest = turn;
                    fallback = i;
                }
                if (!(turn > 1e-14)) continue;
                bool empty = true;
                for (size_t k = 0; k < n && empty; ++k) {
                    const int q = poly[k];
                    if (same(q, ia) || same(q, ib) || same(q, ic)) continue;
                    const V2 Q = p[(size_t)q];
                    if (cross(B - A, Q - A) >= 0 && cross(C - B, Q - B) >= 0 && cross(A - C, Q - C) >= 0) empty = false;
                }
                if (empty) ear = i;
            }
            if (ear == n) ear = fallback;  // nothing clean left (rounding): take the widest corner
            const int ia = poly[(ear + n - 1) % n], ib = poly[ear], ic = poly[(ear + 1) % n];
            if (cross(p[(size_t)ib] - p[(size_t)ia], p[(size_t)ic] - p[(size_t)ib]) > 1e-14) out.insert(out.end(), {ia, ib, ic});
            poly.erase(poly.begin() + (std::ptrdiff_t)ear);
        }
    }
    return out;
}

// ---- shapes -------------------------------------------------------------------------------------------------------
// Any finished outline as a solid: the outline in x, y, the thickness along z, centred.
inline Solid extruded(Outline ol) {
    Solid s;
    s.kind = Solid::EXTRUDE;
    s.c = {0, 0, 0};
    s.ax = {0, 0, 1};
    s.ux = {1, 0, 0};
    s.sc = 1;
    s.h = ol.h;
    s.r = ol.reach;
    s.shape = std::make_shared<const Outline>(std::move(ol));
    return s;
}
// A star with `points` tips at `outer` from the middle and the corners between them at `inner`; one tip points up (+y).
inline Solid star(int points, double outer, double inner, double halfThickness, double chamfer) {
    const double pi = 3.14159265358979323846;
    Outline ol;
    ol.h = halfThickness;
    ol.chamfer = chamfer;
    for (int i = 0; i < 2 * points; ++i) {
        const double a = 0.5 * pi + pi * i / points, rad = (i & 1) ? inner : outer;
        if (i == 0) ol.move(rad * std::cos(a), rad * std::sin(a));
        else ol.line(rad * std::cos(a), rad * std::sin(a));
    }
    ol.finish();
    return extruded(std::move(ol));
}
// A flat ring: a disc with a round hole.
inline Solid washer(double outer, double inner, double halfThickness, double chamfer) {
    Outline ol;
    ol.h = halfThickness;
    ol.chamfer = chamfer;
    ol.move(outer, 0);
    ol.arc(outer, 0, 0, 0, true);
    ol.move(inner, 0);
    ol.arc(inner, 0, 0, 0, false);
    ol.finish();
    return extruded(std::move(ol));
}
// A few capital letters, drawn here by hand from straight lines and arcs of circles (no font file): A B M O R.
// About 1 high. Returns an empty solid for any other letter.
inline Solid letter(char which, double halfThickness, double chamfer) {
    Outline ol;
    ol.h = halfThickness;
    ol.chamfer = chamfer;
    if (which == 'A') {
        // two legs that lean together under a flat top, a bar across, a three-cornered hole above the bar
        const double lean = 0.345, foot = 0.21, bar0 = 0.24, bar1 = 0.42, W = 0.90;
        ol.move(0, 0);
        ol.line(foot, 0);
        ol.line(foot + lean * bar0, bar0);
        ol.line(W - foot - lean * bar0, bar0);
        ol.line(W - foot, 0);
        ol.line(W, 0);
        ol.line(W - lean, 1);
        ol.line(lean, 1);
        const double top = (W - 2.0 * foot) / (2.0 * lean);  // where the insides of the legs meet
        ol.move(foot + lean * bar1, bar1);
        ol.line(0.5 * W, top);
        ol.line(W - foot - lean * bar1, bar1);
    } else if (which == 'M') {
        // two posts and a V between them, cut out above and below
        ol.move(0, 0);
        ol.line(0.2, 0);
        ol.line(0.2, 0.63);
        ol.line(0.5, 0.05);
        ol.line(0.8, 0.63);
        ol.line(0.8, 0);
        ol.line(1.0, 0);
        ol.line(1.0, 1);
        ol.line(0.8, 1);
        ol.line(0.5, 0.42);
        ol.line(0.2, 1);
        ol.line(0, 1);
    } else if (which == 'O') {
        // two half circles joined by short straight sides, and the same again inside
        ol.move(0.8, 0.4);
        ol.line(0.8, 0.6);
        ol.arc(0.0, 0.6, 0.4, 0.6, true);
        ol.line(0.0, 0.4);
        ol.arc(0.8, 0.4, 0.4, 0.4, true);
        ol.move(0.6, 0.6);
        ol.line(0.6, 0.4);
        ol.arc(0.2, 0.4, 0.4, 0.4, false);
        ol.line(0.2, 0.6);
        ol.arc(0.6, 0.6, 0.4, 0.6, false);
    } else if (which == 'B') {
        // a post with two bowls; the bowls are half circles that run into each other at the waist
        const double x1 = 0.40, y1 = 0.30, r1 = 0.30;  // lower bowl
        const double x2 = 0.36, y2 = 0.72, r2 = 0.28;  // upper bowl
        // where the two circles cross on the right
        const double dx = x2 - x1, dy = y2 - y1, d = std::sqrt(dx * dx + dy * dy);
        const double along = (d * d + r1 * r1 - r2 * r2) / (2.0 * d), off = std::sqrt(std::max(0.0, r1 * r1 - along * along));
        const double wx = x1 + dx * along / d + dy * off / d, wy = y1 + dy * along / d - dx * off / d;
        ol.move(0, 0);
        ol.line(x1, 0);
        ol.arc(wx, wy, x1, y1, true);
        ol.arc(x2, y2 + r2, x2, y2, true);
        ol.line(0, 1);
        const double in1 = 0.14, in2 = 0.12;
        ol.move(0.2, y1 - in1);
        ol.line(0.2, y1 + in1);
        ol.line(x1, y1 + in1);
        ol.arc(x1, y1 - in1, x1, y1, false);
        ol.move(0.2, y2 - in2);
        ol.line(0.2, y2 + in2);
        ol.line(x2, y2 + in2);
        ol.arc(x2, y2 - in2, x2, y2, false);
    } else if (which == 'R') {
        // a post, one bowl, and a leg that runs from under the bowl to the foot
        const double xb = 0.40, yb = 0.71, rb = 0.29, in = 0.13;
        const V2 f0{0.72, 0.0}, f1{0.52, 0.42};  // the outside of the leg, from the foot upwards
        // where that line meets the bowl's circle
        const V2 dir = f1 - f0, oc = f0 - V2{xb, yb};
        const double A = dot(dir, dir), B = dot(oc, dir), C = dot(oc, oc) - rb * rb, t = (-B - std::sqrt(std::max(0.0, B * B - A * C))) / A;
        const V2 meet = f0 + dir * t;
        ol.move(0, 0);
        ol.line(0.2, 0);
        ol.line(0.2, 0.42);
        ol.line(0.30, 0.42);
        ol.line(0.50, 0);
        ol.line(f0.x, f0.y);
        ol.line(meet.x, meet.y);
        ol.arc(xb, yb + rb, xb, yb, true);
        ol.line(0, 1);
        ol.move(0.2, yb - in);
        ol.line(0.2, yb + in);
        ol.line(xb, yb + in);
        ol.arc(xb, yb - in, xb, yb, false);
    } else {
        return Solid{};
    }
    ol.finish();
    return extruded(std::move(ol));
}

// ---- rays against the outline ---------------------------------------------------------------------------------------
struct OutlineHit {
    double t = 1e300;
    int edge = -1;
    V2 n{0, 0};  // out of the glass (unit)
};
// The nearest place beyond tmin where o + t d crosses the outline. want > 0: only where it leaves the glass,
// < 0: only where it goes in, 0: either.
inline bool outline_cast(const Outline& ol, V2 o, V2 d, double tmin, int want, OutlineHit& hit) {
    double best = 1e300;
    int be = -1;
    V2 bn{0, 0};
    bool straight = false;
    const size_t ne = ol.edges.size();
    for (size_t i = 0; i < ne; ++i) {
        const OutlineEdge& e = ol.edges[i];
        if (e.r == 0) {
            const V2 ab = e.b - e.a;
            const double den = cross(d, ab);  // d . (outward normal) times the length of the edge
            if (want > 0 ? den <= 0 : (want < 0 ? den >= 0 : den == 0)) continue;
            const V2 ao = e.a - o;
            const double t = cross(ao, ab) / den;
            if (!(t > tmin) || t >= best) continue;
            const double w = cross(ao, d) / den;
            if (w < -1e-5 || w > 1.0 + 1e-5) continue;
            best = t;
            be = (int)i;
            bn = V2{ab.y, -ab.x};
            straight = true;
        } else {
            const V2 oc = o - e.c;
            const double A = dot(d, d), B = dot(oc, d), C = dot(oc, oc) - e.r * e.r, disc = B * B - A * C;
            if (!(disc > 0)) continue;
            const double sq = std::sqrt(disc);
            const V2 ab = e.b - e.a;
            for (int k = 0; k < 2; ++k) {
                const double t = (k ? -B + sq : -B - sq) / A;
                if (!(t > tmin) || t >= best) continue;
                const V2 p = o + d * t;
                if (cross(ab, p - e.a) * e.r > 1e-5 * e.r * e.r) continue;  // on the circle, but not on this arc
                const V2 n = (p - e.c) * (1.0 / e.r);
                const double dn = dot(d, n);
                if (want > 0 ? dn <= 0 : (want < 0 ? dn >= 0 : false)) continue;
                best = t;
                be = (int)i;
                bn = n;
                straight = false;
            }
        }
    }
    if (be < 0) return false;
    hit.t = best;
    hit.edge = be;
    hit.n = straight ? bn * (1.0 / length(bn)) : bn;
    return true;
}
// For a ray from outside: how many times it crosses the outline beyond `from` (odd: it is inside the outline
// there), and the first place between `from` and `to` where it goes into the glass.
inline void outline_scan(const Outline& ol, V2 o, V2 d, double from, double to, int& crossings, OutlineHit& hit) {
    crossings = 0;
    hit.t = 1e300;
    hit.edge = -1;
    for (size_t i = 0; i < ol.edges.size(); ++i) {
        const OutlineEdge& e = ol.edges[i];
        if (e.r == 0) {
            const V2 ab = e.b - e.a;
            const double den = cross(d, ab);
            if (den == 0) continue;
            const V2 ao = e.a - o;
            const double t = cross(ao, ab) / den;
            if (!(t > from)) continue;
            const double w = cross(ao, d) / den;
            if (w < 0 || w >= 1.0) continue;  // (half open: a corner counts for one of its two edges)
            ++crossings;
            if (den < 0 && t < to && t < hit.t) {
                hit.t = t;
                hit.edge = (int)i;
                hit.n = V2{ab.y, -ab.x} * (1.0 / length(ab));
            }
        } else {
            const V2 oc = o - e.c;
            const double A = dot(d, d), B = dot(oc, d), C = dot(oc, oc) - e.r * e.r, disc = B * B - A * C;
            if (!(disc > 0)) continue;
            const double sq = std::sqrt(disc);
            const V2 ab = e.b - e.a;
            for (int k = 0; k < 2; ++k) {
                const double t = (k ? -B + sq : -B - sq) / A;
                if (!(t > from)) continue;
                const V2 p = o + d * t;
                if (cross(ab, p - e.a) * e.r > 0) continue;
                ++crossings;
                const V2 n = (p - e.c) * (1.0 / e.r);
                if (dot(d, n) < 0 && t < to && t < hit.t) {
                    hit.t = t;
                    hit.edge = (int)i;
                    hit.n = n;
                }
            }
        }
    }
}

// ---- for the rays of the other solids, and for the desk check: one ray against the whole solid ----------------------
// (the solid with sharp rims: the chamfers are too small to matter to a ray from elsewhere)
// Faces: the walls by the number of their edge; then the cap at +z, then the cap at -z.
inline bool extrude_leave(const Solid& so, V3 p, V3 d, double& t, V3& n, int& face) {
    const Outline& ol = *so.shape;
    const V3 ey = cross(so.ax, so.ux), w = (p - so.c) * (1.0 / so.sc);
    const V3 lo{dot(w, so.ux), dot(w, ey), dot(w, so.ax)}, ld{dot(d, so.ux), dot(d, ey), dot(d, so.ax)};
    double tc = 1e300;
    if (ld.z > 1e-12) tc = (ol.h - lo.z) / ld.z;
    else if (ld.z < -1e-12) tc = (-ol.h - lo.z) / ld.z;
    OutlineHit hit;
    if (outline_cast(ol, V2{lo.x, lo.y}, V2{ld.x, ld.y}, EXT_TMIN, 1, hit) && hit.t < tc) {
        t = hit.t * so.sc;
        n = so.ux * hit.n.x + ey * hit.n.y;
        face = hit.edge;
        return true;
    }
    if (!(tc < 1e299) || !(tc > EXT_TMIN)) return false;
    t = tc * so.sc;
    n = ld.z > 0 ? so.ax : so.ax * -1.0;
    face = (int)ol.edges.size() + (ld.z > 0 ? 0 : 1);
    return true;
}
inline bool extrude_meet(const Solid& so, V3 o, V3 d, double& t, V3& n, int& face) {
    const Outline& ol = *so.shape;
    const V3 ey = cross(so.ax, so.ux), w = (o - so.c) * (1.0 / so.sc);
    const V3 lo{dot(w, so.ux), dot(w, ey), dot(w, so.ax)}, ld{dot(d, so.ux), dot(d, ey), dot(d, so.ax)};
    // while the ray is between the planes of the caps
    double t0 = -1e300, t1 = 1e300;
    if (std::fabs(ld.z) > 1e-12) {
        const double a = (-ol.h - lo.z) / ld.z, b = (ol.h - lo.z) / ld.z;
        t0 = std::min(a, b);
        t1 = std::max(a, b);
    } else if (std::fabs(lo.z) >= ol.h) {
        return false;
    }
    const double tmin = 1e-7;
    if (!(t1 > tmin)) return false;
    const double from = std::max(t0, tmin);
    int crossings;
    OutlineHit hit;
    outline_scan(ol, V2{lo.x, lo.y}, V2{ld.x, ld.y}, from, t1, crossings, hit);
    if (crossings & 1) {
        if (!(t0 > tmin)) return false;  // it starts inside the glass
        t = t0 * so.sc;
        n = ld.z > 0 ? so.ax * -1.0 : so.ax;
        face = (int)ol.edges.size() + (ld.z > 0 ? 1 : 0);
        return true;
    }
    if (hit.edge < 0) return false;
    t = hit.t * so.sc;
    n = so.ux * hit.n.x + ey * hit.n.y;
    face = hit.edge;
    return true;
}
inline void extrude_bound(const Solid& so, V3& centre, double& radius) {
    const Outline& ol = *so.shape;
    centre = so.c;
    radius = std::sqrt(ol.reach * ol.reach + ol.h * ol.h) * so.sc;
}

// ---- the mesh a renderer draws the body with (chamfers included), in the solid's own frame --------------------------
// Flat faces: every triangle has its own three corners. pos and nrm get three entries per triangle.
inline void extrude_mesh(const Outline& ol, std::vector<V3>& pos, std::vector<V3>& nrm) {
    pos.clear();
    nrm.clear();
    const double leg = ol.chamfer / std::sqrt(2.0), h = ol.h;
    auto tri = [&](V3 a, V3 b, V3 c) {
        V3 n = cross(b - a, c - a);
        const double l = length(n);
        if (!(l > 1e-18)) return;
        n = n * (1.0 / l);
        pos.insert(pos.end(), {a, b, c});
        nrm.insert(nrm.end(), {n, n, n});
    };
    for (size_t t = 0; t + 2 < ol.insetTris.size(); t += 3) {
        const V2 a = ol.inset[(size_t)ol.insetTris[t]], b = ol.inset[(size_t)ol.insetTris[t + 1]], c = ol.inset[(size_t)ol.insetTris[t + 2]];
        tri(V3{a.x, a.y, h}, V3{b.x, b.y, h}, V3{c.x, c.y, h});
        tri(V3{a.x, a.y, -h}, V3{c.x, c.y, -h}, V3{b.x, b.y, -h});
    }
    for (const std::vector<int>& loop : ol.loops)
        for (size_t i = 0; i < loop.size(); ++i) {
            const int ia = loop[i], ib = loop[(i + 1) % loop.size()];
            const V2 a = ol.pts[(size_t)ia], b = ol.pts[(size_t)ib], ai = ol.inset[(size_t)ia], bi = ol.inset[(size_t)ib];
            const double w = h - leg;
            // the wall (seen from outside the glass is on the left of a -> b, so a, b, up is counter-clockwise)
            tri(V3{a.x, a.y, -w}, V3{b.x, b.y, -w}, V3{b.x, b.y, w});
            tri(V3{a.x, a.y, -w}, V3{b.x, b.y, w}, V3{a.x, a.y, w});
            if (leg > 0) {
                tri(V3{a.x, a.y, w}, V3{b.x, b.y, w}, V3{bi.x, bi.y, h});
                tri(V3{a.x, a.y, w}, V3{bi.x, bi.y, h}, V3{ai.x, ai.y, h});
                tri(V3{b.x, b.y, -w}, V3{a.x, a.y, -w}, V3{ai.x, ai.y, -h});
                tri(V3{b.x, b.y, -w}, V3{ai.x, ai.y, -h}, V3{bi.x, bi.y, -h});
            }
        }
}


// ---- the light --------------------------------------------------------------------------------------------------------
// What a renderer keeps per shape, built once. Three lines of a float texture, EXT_TABLE wide:
//   line 0: texel 0 = (edges, stretches, triangles, 0), texel 1 = (half thickness, leg of the chamfer, reach, 0),
//           then two texels per edge: (a.x, a.y, b.x, b.y), (c.x, c.y, r, sweep)
//   line 1: one texel per stretch: (edge, u0, u1, 0)
//   line 2: two texels per triangle of the outline: (x0, y0, x1, y1), (x2, y2, 0, 0)
inline bool ex_shape_table(const Outline& ol, float* rows /* 3 * EXT_TABLE * 4 */) {
    std::memset(rows, 0, (size_t)3 * EXT_TABLE * 4 * sizeof(float));
    if (2 + 2 * ol.edges.size() > (size_t)EXT_TABLE || ol.stretches.size() > (size_t)EXT_COLUMNS || 2 * (ol.tris.size() / 3) > (size_t)EXT_TABLE) return false;
    float* a = rows;
    a[0] = (float)ol.edges.size(); a[1] = (float)ol.stretches.size(); a[2] = (float)(ol.tris.size() / 3);
    a[4] = (float)ol.h; a[5] = (float)(ol.chamfer / std::sqrt(2.0)); a[6] = (float)ol.reach;
    for (size_t i = 0; i < ol.edges.size(); ++i) {
        const OutlineEdge& e = ol.edges[i];
        float* o = a + (2 + 2 * i) * 4;
        o[0] = (float)e.a.x; o[1] = (float)e.a.y; o[2] = (float)e.b.x; o[3] = (float)e.b.y;
        o[4] = (float)e.c.x; o[5] = (float)e.c.y; o[6] = (float)e.r; o[7] = (float)e.sweep;
    }
    float* b = rows + (size_t)EXT_TABLE * 4;
    for (size_t i = 0; i < ol.stretches.size(); ++i) {
        b[i * 4] = (float)ol.stretches[i].edge; b[i * 4 + 1] = (float)ol.stretches[i].u0; b[i * 4 + 2] = (float)ol.stretches[i].u1;
    }
    float* t = rows + (size_t)2 * EXT_TABLE * 4;
    for (size_t i = 0; i + 2 < ol.tris.size(); i += 3) {
        const V2 p0 = ol.pts[(size_t)ol.tris[i]], p1 = ol.pts[(size_t)ol.tris[i + 1]], p2 = ol.pts[(size_t)ol.tris[i + 2]];
        float* o = t + (i / 3) * 8;
        o[0] = (float)p0.x; o[1] = (float)p0.y; o[2] = (float)p1.x; o[3] = (float)p1.y; o[4] = (float)p2.x; o[5] = (float)p2.y;
    }
    return true;
}
// The one mesh of all strips: per corner (stretch, part * 4 + corner, slot).
// part: a stretch is cut in three at the (up to two) places where the light changes its way within it.
// corner: 0 = start of the part, near edge; 1 = start, far edge; 2 = end, far edge; 3 = end, near edge.
// slot: up to three strips lie side by side across (see ex_strip_vertex).
// Three lists of indices: `slim` draws slot 0 only (18 indices per stretch), `pair` slots 1 and 2 (36), `full` all
// three (54); in each the stretches come in order, so a shape with fewer stretches is drawn with the first so many.
inline void ex_strip_mesh(std::vector<float>& corners, std::vector<uint32_t>& slim, std::vector<uint32_t>& pair, std::vector<uint32_t>& full) {
    corners.clear();
    slim.clear();
    pair.clear();
    full.clear();
    for (int m = 0; m < EXT_COLUMNS; ++m)
        for (int slot = 0; slot < 3; ++slot)
            for (int part = 0; part < 3; ++part) {
                const uint32_t base = (uint32_t)(corners.size() / 3);
                for (int k = 0; k < 4; ++k) corners.insert(corners.end(), {(float)m, (float)(part * 4 + k), (float)slot});
                full.insert(full.end(), {base, base + 1, base + 2, base, base + 2, base + 3});
                (slot == 0 ? slim : pair).insert((slot == 0 ? slim : pair).end(), {base, base + 1, base + 2, base, base + 2, base + 3});
            }
}
// The one mesh of all outlines: per corner (triangle, corner, 0).
inline void ex_cap_mesh(std::vector<float>& corners, std::vector<uint32_t>& idx) {
    corners.clear();
    idx.clear();
    for (int t = 0; t < EXT_TABLE / 2; ++t)
        for (int k = 0; k < 3; ++k) {
            idx.push_back((uint32_t)(corners.size() / 3));
            corners.insert(corners.end(), {(float)t, (float)k, 0.f});
        }
}
// What a renderer hands over per solid and frame: three texels (middle + size, x axis + line of its shape in the
// table, y axis).
inline void ex_pose(const Solid& so, int shapeLine, float* a, float* b, float* c) {
    const V3 ey = cross(so.ax, so.ux);
    a[0] = (float)so.c.x; a[1] = (float)so.c.y; a[2] = (float)so.c.z; a[3] = (float)so.sc;
    b[0] = (float)so.ux.x; b[1] = (float)so.ux.y; b[2] = (float)so.ux.z; b[3] = (float)shapeLine;
    c[0] = (float)ey.x; c[1] = (float)ey.y; c[2] = (float)ey.z; c[3] = 0.f;
}

namespace detail {

// Where the solid is, the sun as the solid sees it, and the colour steps.
struct ExCtx {
    const Outline* ol;
    V3 O, ex, ey, ez;
    double sc;
    V3 Lw, L;      // the sun: in the world, and in the solid's own frame
    V2 sh{0, 0};   // which way the sunlight goes across the outline (unit); nothing if it comes straight along the thickness
    double lat;    // how much of the sun's direction lies across
    double h, leg, floorY, size, sunTan;
    double top;    // +1: the cap at +z has the sun, -1: the one at -z
    double rise;   // how much higher the sun's ray is per unit across
    double capN[EXT_CAP_STEPS], stepN[EXT_STEPS];
    V3 capRgb[EXT_CAP_STEPS], stepRgb[EXT_STEPS];
    double longest;  // the longest sideways step of light through the caps (the reddest)
    bool mirrors;
    V3 world(V2 p, double z) const { return O + (ex * p.x + ey * p.y + ez * z) * sc; }
    V3 dir(V3 d) const { return ex * d.x + ey * d.y + ez * d.z; }
    // where light from P along D (unit) lands: x, distance flown, z
    V3 land(V3 P, V3 D) const {
        const double t = std::max(0.0, (floorY - P.y) / std::min(D.y, -1e-6));
        return V3{P.x + D.x * t, t, P.z + D.z * t};
    }
    // the same without stopping at the floor (for measuring how the light spreads)
    V3 slide(V3 P, V3 D) const {
        const double t = (floorY - P.y) / std::min(D.y, -1e-6);
        return V3{P.x + D.x * t, t, P.z + D.z * t};
    }
    // how much of the light is still drawn: 1, falling to 0 where it leaves too level or lands too far away
    double fade(double flown, double down) const { return ball_fade(flown / size, down); }
    double capStep(double n) const { return 2.0 * h * lat / std::sqrt(n * n - lat * lat); }
};
inline ExCtx ex_ctx(const Solid& so, const Params& par) {
    ExCtx c;
    c.ol = so.shape.get();
    c.O = so.c;
    c.ex = so.ux;
    c.ez = so.ax;
    c.ey = cross(so.ax, so.ux);
    c.sc = so.sc;
    c.Lw = normalize(par.sun);
    c.L = V3{dot(c.Lw, c.ex), dot(c.Lw, c.ey), dot(c.Lw, c.ez)};
    c.lat = std::sqrt(c.L.x * c.L.x + c.L.y * c.L.y);
    if (c.lat > 1e-6) c.sh = V2{c.L.x / c.lat, c.L.y / c.lat};
    c.h = c.ol->h;
    c.leg = c.ol->chamfer / std::sqrt(2.0);
    c.floorY = par.floorY;
    c.size = std::sqrt(c.ol->reach * c.ol->reach + c.h * c.h) * c.sc;
    c.sunTan = par.sunTan;
    c.top = c.L.z < 0 ? 1.0 : -1.0;
    c.rise = std::fabs(c.L.z) / std::max(c.lat, 1e-6);
    const std::vector<BallStep> cs = ball_steps(par, EXT_CAP_STEPS), ws = ball_steps(par, EXT_STEPS);
    c.longest = 0;
    for (int k = 0; k < EXT_CAP_STEPS; ++k) {
        c.capN[k] = cs[std::min((size_t)k, cs.size() - 1)].n;
        c.capRgb[k] = (size_t)k < cs.size() ? cs[(size_t)k].rgb : V3{0, 0, 0};
        c.longest = std::max(c.longest, c.capStep(c.capN[k]));
    }
    for (int k = 0; k < EXT_STEPS; ++k) {
        c.stepN[k] = ws[std::min((size_t)k, ws.size() - 1)].n;
        c.stepRgb[k] = (size_t)k < ws.size() ? ws[(size_t)k].rgb : V3{0, 0, 0};
    }
    c.mirrors = par.outerReflection;
    return c;
}
inline int ex_mix(int key, int v) { return (key * 31 + v + 1) % 65521; }
inline double ex_smooth(double x) {
    x = std::min(1.0, std::max(0.0, x));
    return x * x * (3.0 - 2.0 * x);
}
// At a face inside the glass (sin2 = n^2 sin^2 of the angle the light meets it at; 1 = the limit): how much of the
// light is taken to be mirrored if it is past the limit, or let out if it is short of it. Nothing right at the
// limit, all of it a little way off: the truth switches there at once, and a moving picture would show it.
inline double ex_kept(double sin2) { return ex_smooth((sin2 - 1.0) / EXT_EDGE); }
inline double ex_let(double sin2) { return ex_smooth((1.0 - sin2) / EXT_EDGE); }
inline double ex_area(V3 a, V3 b, V3 c, V3 d) {  // of a four-cornered patch on the floor (x, ., z)
    return 0.5 * std::fabs((c.x - a.x) * (d.z - b.z) - (d.x - b.x) * (c.z - a.z));
}
// (a ray is started a hair inside the ends of an edge, so that it never starts on two edges at once)
inline double ex_in(double u) { return std::min(1.0 - 1e-4, std::max(1e-4, u)); }
inline int ex_next_step(int k) { return k + 1 < EXT_STEPS ? k + 1 : k - 1; }

// -- what becomes of the light at one point of the outline (edge e, u along it), per kind of light

// The wall's own shadow: how much of the wall's height throws it (the rest falls on the solid itself).
struct ExShadow {
    double up;
    int key;
};
inline ExShadow ex_shadow_at(const ExCtx& c, int e, double u) {
    OutlineHit hit;
    if (outline_cast(*c.ol, c.ol->point(e, ex_in(u)), c.sh, EXT_TMIN, 0, hit)) return ExShadow{std::min(2.0 * c.h, hit.t * c.rise), hit.edge + 1};
    return ExShadow{2.0 * c.h, 0};
}
// Light from the cap that this wall mirrors: how far behind this point (against the light) the glass goes on,
// and the next wall the mirrored light meets (how far along its way, and that wall's normal there).
struct ExMirror {
    double back, at1;
    V2 n1;
    int key;
};
inline ExMirror ex_mirror_at(const ExCtx& c, int e, double u) {
    const Outline& ol = *c.ol;
    ExMirror m{0, EXT_BIG, V2{1, 0}, 1};
    const V2 n = ol.normal(e, u);
    const double sn = dot(c.sh, n);
    if (!(sn > 1e-6)) return m;
    const V2 from = ol.point(e, ex_in(u));
    OutlineHit hit;
    if (outline_cast(ol, from, c.sh * -1.0, EXT_TMIN, 0, hit) && hit.t < c.longest) {
        m.back = hit.t;
        m.key = ex_mix(m.key, hit.edge + 1);
    } else {
        m.back = EXT_BIG;
        m.key = ex_mix(m.key, 0);
    }
    if (outline_cast(ol, from, c.sh - n * (2.0 * sn), EXT_TMIN, 1, hit) && hit.t < c.longest) {
        m.at1 = hit.t;
        m.n1 = hit.n;
        m.key = ex_mix(m.key, hit.edge + 1);
    }
    return m;
}
// Light that goes in through the wall here (index of refraction n): how high the solid's own shadow stands on the
// wall, how far the light goes inside, where and which way it comes out (as if no cap had turned it over; the
// third part of that direction is the sun's own), and how much of it is left. weight < 0: none.
struct ExWall {
    double lift, path;
    V2 q, o;
    double weight;
    int key;
};
inline ExWall ex_wall_at(const ExCtx& c, int e, double u, double n) {
    const Outline& ol = *c.ol;
    ExWall m{EXT_BIG, 0, V2{0, 0}, V2{c.L.x, c.L.y}, -1.0, 1};
    const V2 nrm = ol.normal(e, u);
    const double ci = -c.lat * dot(c.sh, nrm);
    if (!(ci > 1e-6)) return m;
    const V2 from = ol.point(e, ex_in(u));
    m.q = from;
    OutlineHit hit;
    if (outline_cast(ol, from, c.sh * -1.0, EXT_TMIN, 0, hit)) {
        m.lift = hit.t * c.rise;
        m.key = ex_mix(m.key, hit.edge + 1);
    } else {
        m.key = ex_mix(m.key, 0);
    }
    V3 d;
    const V3 n3{nrm.x, nrm.y, 0};
    if (!refract(c.L, n3, 1.0 / n, d)) return m;
    double weight = 1.0 - fresnel(ci, 1.0, n);
    V2 at = from, d2{d.x, d.y};
    for (int b = 0; b <= EXT_BOUNCES; ++b) {
        if (!outline_cast(ol, at, d2, EXT_TMIN, 1, hit)) break;
        m.path += hit.t;
        at = at + d2 * hit.t;
        const double co = dot(d2, hit.n), sin2 = n * n * (1.0 - co * co);
        if (sin2 >= 1.0) {
            d2 = d2 - hit.n * (2.0 * co);
            weight *= ex_kept(sin2);
            m.key = ex_mix(m.key, hit.edge * 2 + 1);
            continue;
        }
        V3 o;
        if (!refract(V3{d2.x, d2.y, d.z}, V3{-hit.n.x, -hit.n.y, 0}, n, o)) break;
        m.weight = weight * (1.0 - fresnel(co, n, 1.0)) * ex_let(sin2);
        m.q = at;
        m.o = V2{o.x, o.y};
        m.key = ex_mix(m.key, hit.edge * 2);
        return m;
    }
    m.key = ex_mix(m.key, 4001);
    return m;
}
// Light that goes in through the chamfer of a rim here (rim: +1 the one at +z, -1 at -z): one ray from the middle
// of the cut through the solid (sharp rims), mirrored while the glass cannot let it out; the two edges of the cut
// are sent the same way. Where they land (x, z), how much light a unit of the cut carries (0: none), how much of
// it is still drawn, and how far it flew.
struct ExCut {
    V2 qin, qout;
    double e, fade, fly;
    int key;
};
inline ExCut ex_cut_at(const ExCtx& c, int e, double u, int rim, double n) {
    const Outline& ol = *c.ol;
    ExCut m{V2{0, 0}, V2{0, 0}, 0, 0, 0, 1};
    const V2 nn = ol.normal(e, u), p = ol.point(e, u), from = ol.point(e, ex_in(u));
    const V3 cut = V3{nn.x, nn.y, (double)rim} * (1.0 / std::sqrt(2.0));
    const double ci = -dot(c.L, cut);
    if (!(ci > 1e-6)) return m;
    double open = 1;
    OutlineHit hit;
    if ((double)rim != c.top && c.lat > 1e-6 && outline_cast(ol, from, c.sh * -1.0, EXT_TMIN, 0, hit)) {
        // the rim away from the sun: lit only if the sun gets past the rest of the solid
        open = std::min(1.0, std::max(0.0, (hit.t * c.rise - (2.0 * c.h - c.leg)) / c.leg));
        m.key = ex_mix(m.key, hit.edge + 1);
    }
    V3 d;
    if (!(open > 0) || !refract(c.L, cut, 1.0 / n, d)) {
        m.key = ex_mix(m.key, 4001);
        return m;
    }
    // the two edges of the cut and its middle, carried along: each face on the way moves them along the light to
    // the face's plane
    V3 pin{p.x - nn.x * c.leg, p.y - nn.y * c.leg, rim * c.h}, pout{p.x, p.y, rim * (c.h - c.leg)};
    V3 at{from.x - nn.x * 0.5 * c.leg, from.y - nn.y * 0.5 * c.leg, rim * (c.h - 0.5 * c.leg)};
    double weight = 1.0 - fresnel(ci, 1.0, n);
    const int ne = (int)ol.edges.size();
    for (int b = 0; b <= EXT_BOUNCES; ++b) {
        double tc = EXT_BIG;
        if (d.z > 1e-6) tc = (c.h - at.z) / d.z;
        else if (d.z < -1e-6) tc = (-c.h - at.z) / d.z;
        V3 N;
        double t;
        int id;
        if (outline_cast(ol, V2{at.x, at.y}, V2{d.x, d.y}, EXT_TMIN, 1, hit) && hit.t < tc) {
            t = hit.t;
            N = V3{hit.n.x, hit.n.y, 0};
            id = hit.edge;
        } else if (tc < 0.5 * EXT_BIG) {
            t = tc;
            N = V3{0, 0, d.z > 0 ? 1.0 : -1.0};
            id = ne + (d.z > 0 ? 0 : 1);
        } else {
            break;
        }
        at = at + d * t;
        const double off = dot(N, at), den = dot(N, d);
        pin = pin + d * ((off - dot(N, pin)) / den);
        pout = pout + d * ((off - dot(N, pout)) / den);
        const double sin2 = n * n * (1.0 - den * den);
        if (sin2 >= 1.0) {
            d = d - N * (2.0 * den);
            weight *= ex_kept(sin2);
            m.key = ex_mix(m.key, id * 2 + 1);
            continue;
        }
        V3 o;
        if (!refract(d, N * -1.0, n, o)) break;
        m.key = ex_mix(m.key, id * 2);
        const V3 D = c.dir(o);
        if (!(D.y < -1e-6)) break;
        const V3 a = c.land(c.world(V2{pin.x, pin.y}, pin.z), D), b2 = c.land(c.world(V2{pout.x, pout.y}, pout.z), D), mid = c.land(c.world(V2{at.x, at.y}, at.z), D);
        m.qin = V2{a.x, a.z};
        m.qout = V2{b2.x, b2.z};
        m.e = ci * weight * (1.0 - fresnel(den, n, 1.0)) * ex_let(sin2) * open;
        m.fade = c.fade(mid.y, -D.y);
        m.fly = mid.y;
        return m;
    }
    m.key = ex_mix(m.key, 4001);
    return m;
}

// Where along a stretch of wall (u0 to u1) the light changes its way, up to twice: for each change the last place
// found with the way before it (ua) and the first with the way after it (ub), by halving; `torn` says there is one.
// Without a change the places are just thirds of the stretch (ua = ub). keyAt(u): a number that is the same for
// all light that takes the same way.
struct ExFind {
    double ua[2], ub[2];
    bool torn[2];
};
template <class F>
inline ExFind ex_find(double u0, double u1, F&& keyAt) {
    ExFind f;
    f.ua[0] = f.ub[0] = f.ua[1] = f.ub[1] = u0;
    f.torn[0] = f.torn[1] = false;
    if (!(u1 > u0)) return f;
    const double third = (u1 - u0) / 3.0;
    const int ka = keyAt(u0), kb = keyAt(u1);
    if (ka == kb) {
        f.ua[0] = f.ub[0] = u0 + third;
        f.ua[1] = f.ub[1] = u0 + 2.0 * third;
        return f;
    }
    double lo = u0, hi = u1;
    int khi = kb;
    for (int it = 0; it < EXT_HALVINGS; ++it) {
        const double um = 0.5 * (lo + hi);
        const int km = keyAt(um);
        if (km == ka) {
            lo = um;
        } else {
            hi = um;
            khi = km;
        }
    }
    f.ua[0] = lo;
    f.ub[0] = hi;
    f.torn[0] = true;
    if (khi == kb) {
        f.ua[1] = f.ub[1] = 0.5 * (hi + u1);
        return f;
    }
    lo = hi;
    hi = u1;
    for (int it = 0; it < EXT_HALVINGS; ++it) {
        const double um = 0.5 * (lo + hi);
        if (keyAt(um) == khi) lo = um;
        else hi = um;
    }
    f.ua[1] = lo;
    f.ub[1] = hi;
    f.torn[1] = true;
    return f;
}
// Step one (extrude_find.mat): one texel per stretch (column) and kind of light (line 0: the walls' shadows,
// 1: light from the cap mirrored by the walls, 2 + k: light through the walls in colour step k):
// (ua, ub or -ub if there is a change there) for the two places.
inline void ex_find_texel(const ExCtx& c, int column, int line, float* out) {
    const Outline& ol = *c.ol;
    out[0] = out[1] = out[2] = out[3] = 0.f;
    if (column >= (int)ol.stretches.size() || !(c.lat > 1e-6)) return;
    const Outline::Stretch st = ol.stretches[(size_t)column];
    const int e = st.edge;
    const OutlineEdge& ed = ol.edges[(size_t)e];
    const double side = ed.r == 0 ? cross(c.sh, ed.b - ed.a) : 0.0;  // > 0: a straight wall that looks away from the sun
    ExFind f;
    f.ua[0] = f.ub[0] = st.u0 + (st.u1 - st.u0) / 3.0;
    f.ua[1] = f.ub[1] = st.u0 + (st.u1 - st.u0) * 2.0 / 3.0;
    f.torn[0] = f.torn[1] = false;
    if (line == 0) {
        if (!(side < 0)) f = ex_find(st.u0, st.u1, [&](double u) { return ex_shadow_at(c, e, u).key; });
    } else if (line == 1) {
        if (!(side < 0)) f = ex_find(st.u0, st.u1, [&](double u) { return ex_mirror_at(c, e, u).key; });
    } else {
        if (!(side > 0)) f = ex_find(st.u0, st.u1, [&](double u) { return ex_wall_at(c, e, u, c.stepN[line - 2]).key; });
    }
    out[0] = (float)f.ua[0]; out[1] = (float)(f.torn[0] ? -f.ub[0] : f.ub[0]);
    out[2] = (float)f.ua[1]; out[3] = (float)(f.torn[1] ? -f.ub[1] : f.ub[1]);
}
// The five points of a stretch that the strips are hung on: its start (0), just before and just after the first
// place (1, 2), just before and just after the second (3, 4). "Just": one more width of what the halving left,
// so that a point is safely on its side of the change.
inline double ex_point_u(const Outline::Stretch& st, const float* find, int which) {
    if (which == 0) return st.u0;
    const int i = (which - 1) / 2;
    const double ua = find[i * 2], ub = std::fabs((double)find[i * 2 + 1]), gap = ub - ua;
    const double u = (which - 1) % 2 == 0 ? ua - gap : ub + gap;
    return std::min(st.u1, std::max(st.u0, u));
}
// where the strips are cut there
inline double ex_point_at(const Outline::Stretch& st, const float* find, int which) {
    if (which == 0) return st.u0;
    const int i = (which - 1) / 2;
    return 0.5 * ((double)find[i * 2] + std::fabs((double)find[i * 2 + 1]));
}
// Step two (extrude_walk.mat): what becomes of the light at those points. Column = stretch; line =
//   EXT_ROW_SHADOW          (shadow at points 0, 1, 2, 3),   EXT_ROW_SHADOW + 1  (shadow at point 4, 0, 0, 0)
//   EXT_ROW_MIRROR + point  (behind, next wall at, that wall's normal x, y)
//   EXT_ROW_WALL + 10 step + 2 point      (shade on the wall, way inside, where it comes out x, y)
//   EXT_ROW_WALL + 10 step + 2 point + 1  (which way x, y; how much is left; how far from the next colour's way)
//   EXT_ROW_CUT + 2 (step + EXT_STEPS if the rim at -z)      (lands: inner edge x, z, outer edge x, z)    [point 0 only]
//   EXT_ROW_CUT + ... + 1                                    (light per unit, how much is drawn, flown, key)
// find: the solid's texture of step one (EXT_COLUMNS wide).
inline void ex_walk_texel(const ExCtx& c, const float* find, int column, int line, float* out) {
    const Outline& ol = *c.ol;
    out[0] = out[1] = out[2] = out[3] = 0.f;
    if (column >= (int)ol.stretches.size()) return;
    const Outline::Stretch st = ol.stretches[(size_t)column];
    const int e = st.edge;
    const OutlineEdge& ed = ol.edges[(size_t)e];
    const double side = ed.r == 0 ? cross(c.sh, ed.b - ed.a) : 0.0;
    auto at = [&](int findLine, int which) { return ex_point_u(st, find + ((size_t)findLine * EXT_COLUMNS + (size_t)column) * 4, which); };
    if (line < EXT_ROW_MIRROR) {
        if (!(c.lat > 1e-6) || side < 0) return;
        if (line == EXT_ROW_SHADOW)
            for (int i = 0; i < 4; ++i) out[i] = (float)ex_shadow_at(c, e, at(0, i)).up;
        else out[0] = (float)ex_shadow_at(c, e, at(0, 4)).up;
    } else if (line < EXT_ROW_WALL) {
        if (!(c.lat > 1e-6) || side < 0) return;
        const ExMirror m = ex_mirror_at(c, e, at(1, line - EXT_ROW_MIRROR));
        out[0] = (float)std::min(m.back, EXT_BIG); out[1] = (float)std::min(m.at1, EXT_BIG); out[2] = (float)m.n1.x; out[3] = (float)m.n1.y;
    } else if (line < EXT_ROW_CUT) {
        const int k = (line - EXT_ROW_WALL) / 10, rest = (line - EXT_ROW_WALL) % 10;
        if (!(c.lat > 1e-6) || side > 0) {
            if (rest % 2 == 1) out[2] = -1.f;
            return;
        }
        const double u = at(2 + k, rest / 2);
        const ExWall m = ex_wall_at(c, e, u, c.stepN[k]);
        if (rest % 2 == 0) {
            out[0] = (float)std::min(m.lift, EXT_BIG); out[1] = (float)m.path; out[2] = (float)m.q.x; out[3] = (float)m.q.y;
        } else {
            // how far apart this colour and its neighbour leave: the light is softened by half of that as well, so
            // that the colour steps run into one another. Measured a quarter of the way to the neighbour (to one
            // side or the other, whichever takes the same way). If neither does, this colour is about to be kept
            // in by the glass altogether, and there the colours fly apart without limit: as soft as it gets.
            double gap = 0;
            if (m.weight >= 0) {
                const double quarter = 0.25 * (c.stepN[ex_next_step(k)] - c.stepN[k]);
                ExWall o = ex_wall_at(c, e, u, c.stepN[k] + quarter);
                if (o.key != m.key) o = ex_wall_at(c, e, u, c.stepN[k] - quarter);
                gap = o.key == m.key ? std::min(EXT_GAP, 4.0 * length(o.o - m.o)) : EXT_GAP;
            }
            out[0] = (float)m.o.x; out[1] = (float)m.o.y; out[2] = (float)m.weight; out[3] = (float)gap;
        }
    } else {
        if (!(c.leg > 0)) return;
        const int k2 = (line - EXT_ROW_CUT) / 2, rim = k2 < EXT_STEPS ? 1 : -1, k = k2 % EXT_STEPS;
        const ExCut m = ex_cut_at(c, e, st.u0, rim, c.stepN[k]);
        if ((line - EXT_ROW_CUT) % 2 == 0) {
            out[0] = (float)m.qin.x; out[1] = (float)m.qin.y; out[2] = (float)m.qout.x; out[3] = (float)m.qout.y;
        } else {
            out[0] = (float)m.e; out[1] = (float)m.fade; out[2] = (float)m.fly; out[3] = (float)m.key;
        }
    }
}
// Both textures of one solid (EXT_COLUMNS x EXT_FIND_ROWS and EXT_COLUMNS x EXT_ROWS texels).
inline void ex_walk(const ExCtx& c, std::vector<float>& find, std::vector<float>& tex, int from = 0, int to = EXT_ROWS) {
    find.assign((size_t)EXT_COLUMNS * EXT_FIND_ROWS * 4, 0.f);
    tex.assign((size_t)EXT_COLUMNS * EXT_ROWS * 4, 0.f);
    const int cols = std::min(EXT_COLUMNS, (int)c.ol->stretches.size());
    for (int line = 0; line < EXT_FIND_ROWS; ++line) {
        const int needs = line == 0 ? EXT_ROW_SHADOW : (line == 1 ? EXT_ROW_MIRROR : EXT_ROW_WALL + 10 * (line - 2));
        const int upTo = line == 0 ? EXT_ROW_MIRROR : (line == 1 ? EXT_ROW_WALL : needs + 10);
        if (upTo <= from || needs >= to) continue;
        for (int m = 0; m < cols; ++m) ex_find_texel(c, m, line, &find[((size_t)line * EXT_COLUMNS + (size_t)m) * 4]);
    }
    for (int line = from; line < to; ++line)
        for (int m = 0; m < cols; ++m) ex_walk_texel(c, find.data(), m, line, &tex[((size_t)line * EXT_COLUMNS + (size_t)m) * 4]);
}

// -- the strips (extrude_strip.mat)

// One corner of a strip, as drawn: where on the floor, how bright, how far flown. ok = false: not drawn (the
// renderer folds the strip up into one point).
struct ExVert {
    V3 q{0, 0, 0};  // x, distance flown, z
    V3 rgb{0, 0, 0};
    double fly = 0;
    bool ok = false;
};
// A point of the chain along an edge: point `which` (0 to 4, see ex_point_u) of stretch m. With its neighbours
// along the chain, as far as they belong to the same piece of light.
struct ExAt {
    int m = 0, which = 0;
    bool is = false;
};
struct ExChain {
    const Outline* ol;
    const float* find;  // the line of step one's texture for the kind of light at hand
    const float* tex;   // step two's texture of the solid
    const float* texel(int m, int ln) const { return tex + ((size_t)ln * EXT_COLUMNS + (size_t)m) * 4; }
    bool torn(int m, int i) const { return find[(size_t)m * 4 + (size_t)i * 2 + 1] < 0; }
    bool ends(int m) const { return !(ol->stretches[(size_t)m].u1 > ol->stretches[(size_t)m].u0); }  // the last point of an edge
    bool first(int m) const { return m == 0 || ol->stretches[(size_t)m - 1].edge != ol->stretches[(size_t)m].edge; }
    double u(ExAt a) const { return ex_point_at(ol->stretches[(size_t)a.m], find + (size_t)a.m * 4, a.which); }
    ExAt prev(ExAt a) const {
        if (a.which == 0) return first(a.m) ? ExAt{} : ExAt{a.m - 1, 4, true};
        if (a.which == 1) return ExAt{a.m, 0, true};
        if (a.which == 2) return torn(a.m, 0) ? ExAt{} : ExAt{a.m, 0, true};
        if (a.which == 3) return ExAt{a.m, 2, true};
        return torn(a.m, 1) ? ExAt{} : ExAt{a.m, 2, true};
    }
    ExAt next(ExAt a) const {
        if (a.which == 0) return ends(a.m) ? ExAt{} : ExAt{a.m, 1, true};
        if (a.which == 1) return torn(a.m, 0) ? ExAt{} : ExAt{a.m, 3, true};
        if (a.which == 2) return ExAt{a.m, 3, true};
        if (a.which == 3) return torn(a.m, 1) ? ExAt{} : ExAt{a.m + 1, 0, true};
        return ExAt{a.m + 1, 0, true};
    }
};

// family: which kind of light the strip carries.
enum ExFamily { EX_SHADOW = 0, EX_MIRROR = 1, EX_FOOT = 2, EX_WALL = 3, EX_GLINT = 4, EX_CUT = 5 };
// How many slots across each family uses (the renderer draws that many thirds of the strip mesh).
inline int ex_slots(int family) { return family == EX_MIRROR || family == EX_WALL ? 3 : 1; }
// Which line of step one's texture a family's strips are cut by.
inline int ex_find_line(int family, int k) { return family == EX_SHADOW ? 0 : (family == EX_MIRROR || family == EX_FOOT ? 1 : 2 + k); }

// EX_MIRROR and EX_FOOT: a point of a wall that looks away from the sun
struct ExMirrorPt {
    V2 p, n, dir0, dir1;
    double back, at1, sn;
};
inline ExMirrorPt ex_mirror_pt(const ExCtx& c, const ExChain& ch, ExAt a) {
    const int e = ch.ol->stretches[(size_t)a.m].edge;
    const double u = ch.u(a);
    const float* t = ch.texel(a.m, EXT_ROW_MIRROR + a.which);
    ExMirrorPt m;
    m.p = ch.ol->point(e, u);
    m.n = ch.ol->normal(e, u);
    m.sn = dot(c.sh, m.n);
    m.dir0 = c.sh - m.n * (2.0 * m.sn);
    m.back = t[0];
    m.at1 = t[1];
    const V2 n1{t[2], t[3]};
    m.dir1 = m.dir0 - n1 * (2.0 * dot(m.dir0, n1));
    return m;
}
// the light of point m that is mirrored once (cell 0) or twice (cell 1), for a sideways step S: from where to
// where it is on the cap as it would be without the walls (lo, hi, as lengths along the step), where that
// lands mirrored (qlo, qhi), and how much of it is drawn
struct ExMirrorCell {
    double lo, hi, fade;
    V3 qlo, qhi;
};
inline ExMirrorCell ex_mirror_cell(const ExCtx& c, const ExMirrorPt& m, int cell, double S) {
    const double first = std::max(0.0, S - m.back), mid = std::min(std::max(m.at1, first), S);
    ExMirrorCell r;
    V2 plo, phi, d;
    if (cell == 0) {
        r.lo = first;
        r.hi = mid;
        d = m.dir0;
        plo = m.p + d * std::min(r.lo, m.at1);
        phi = m.p + d * std::min(r.hi, m.at1);
    } else {
        r.lo = mid;
        r.hi = S;
        d = m.dir1;
        if (m.at1 < S) {
            const V2 hit = m.p + m.dir0 * m.at1;
            plo = hit + d * (r.lo - m.at1);
            phi = hit + d * (r.hi - m.at1);
        } else {
            plo = phi = m.p + m.dir0 * S;
            d = m.dir0;
        }
    }
    const double ze = -c.top * c.h;
    const V3 D = c.dir(V3{d.x * c.lat, d.y * c.lat, c.L.z});
    r.qlo = c.land(c.world(plo, ze), D);
    r.qhi = c.land(c.world(phi, ze), D);
    r.fade = D.y < -1e-6 ? c.fade(0.5 * (r.qlo.y + r.qhi.y), -D.y) : 0.0;
    return r;
}
// EX_FOOT: light from the cap that leaves through the chamfer at the foot of a wall looking away from the sun
struct ExFoot {
    V3 qin, qout;
    double e, fade, fly;
};
inline ExFoot ex_foot(const ExCtx& c, const ExMirrorPt& m, int k) {
    ExFoot f{V3{0, 0, 0}, V3{0, 0, 0}, 0, 0, 0};
    const double n = c.stepN[k], n2 = c.stepN[ex_next_step(k)], cosCap = std::fabs(c.L.z), ze = -c.top * c.h;
    const double dz = std::sqrt(1.0 - c.lat * c.lat / (n * n)), dz2 = std::sqrt(1.0 - c.lat * c.lat / (n2 * n2));
    const V3 d{c.sh.x * c.lat / n, c.sh.y * c.lat / n, -c.top * dz}, d2{c.sh.x * c.lat / n2, c.sh.y * c.lat / n2, -c.top * dz2};
    const V3 cut = V3{m.n.x, m.n.y, -c.top} * (1.0 / std::sqrt(2.0));
    const double cm = dot(d, cut);
    V3 o, o2;
    if (!(cm > 1e-6) || !refract(d, cut * -1.0, n, o)) return f;
    const V3 D = c.dir(o);
    if (!(D.y < -1e-6)) return f;
    const V3 Pin = c.world(m.p - m.n * c.leg, ze), Pout = c.world(m.p, ze + c.top * c.leg);
    f.qin = c.land(Pin, D);
    f.qout = c.land(Pout, D);
    // is there glass all the way back to where this light came in?
    const double S = c.capStep(n), open = std::min(1.0, std::max(0.0, (m.back - 0.85 * S) / (0.15 * S)));
    f.e = (1.0 - fresnel(cosCap, 1.0, n)) * (1.0 - fresnel(cm, n, 1.0)) * ex_let(n * n * (1.0 - cm * cm)) * cosCap / dz * cm * open;
    f.fade = c.fade(0.5 * (f.qin.y + f.qout.y), -D.y);
    // how far the next colour lands from this one: the line is softened by half of that as well
    if (refract(d2, cut * -1.0, n2, o2)) {
        const V3 D2 = c.dir(o2);
        if (D2.y < -1e-6) {
            const V3 q2 = c.land(Pin, D2);
            f.fly = std::min(0.5 * std::sqrt((q2.x - f.qin.x) * (q2.x - f.qin.x) + (q2.z - f.qin.z) * (q2.z - f.qin.z)), EXT_SOFT * c.size) / c.sunTan;
        }
    }
    return f;
}
// EX_WALL and EX_GLINT: a point of a wall that has the sun
struct ExWallPt {
    V2 p, n, q, o;
    double ci, lo, hi, shift, weight, gap;
};
inline ExWallPt ex_wall_pt(const ExCtx& c, const ExChain& ch, ExAt a, int k) {
    const int e = ch.ol->stretches[(size_t)a.m].edge;
    const double u = ch.u(a);
    const int base = EXT_ROW_WALL + 10 * k + 2 * a.which;
    const float *t0 = ch.texel(a.m, base), *t1 = ch.texel(a.m, base + 1);
    ExWallPt m;
    m.p = ch.ol->point(e, u);
    m.n = ch.ol->normal(e, u);
    m.ci = -c.lat * dot(c.sh, m.n);
    m.lo = -c.h;
    m.hi = c.h;
    if (c.top > 0) m.lo = std::max(-c.h, c.h - (double)t0[0]);
    else m.hi = std::min(c.h, (double)t0[0] - c.h);
    m.shift = (double)t0[1] * c.L.z / c.stepN[k];
    m.q = V2{t0[2], t0[3]};
    m.o = V2{t1[0], t1[1]};
    m.weight = t1[2];
    m.gap = t1[3];
    return m;
}
// Along the thickness the light is folded back and forth by the caps. Unfolded it is at (height it came in at) +
// shift; fold number j is the part between (2j - 1) h and (2j + 1) h, turned over if j is odd. What of the light
// of point m is in fold j: how much of the wall's height (0: none), where its two ends land, how much is drawn.
struct ExWallFold {
    V3 qlo, qhi;
    double height, fade;
    bool down;  // false: this light goes up into the sky
};
inline ExWallFold ex_wall_fold(const ExCtx& c, const ExWallPt& m, int j) {
    ExWallFold f{V3{0, 0, 0}, V3{0, 0, 0}, 0, 0, false};
    const bool turned = (j & 1) != 0;
    const V3 D = normalize(c.dir(V3{m.o.x, m.o.y, turned ? -c.L.z : c.L.z}));
    if (!(m.weight >= 0) || !(D.y < -1e-6)) return f;
    const double h = c.h;
    const double ylo = std::min(std::max((2.0 * j - 1.0) * h, m.lo + m.shift), m.hi + m.shift), yhi = std::min(std::max((2.0 * j + 1.0) * h, m.lo + m.shift), m.hi + m.shift);
    f.qlo = c.land(c.world(m.q, turned ? 2.0 * j * h - ylo : ylo - 2.0 * j * h), D);
    f.qhi = c.land(c.world(m.q, turned ? 2.0 * j * h - yhi : yhi - 2.0 * j * h), D);
    f.height = std::max(0.0, yhi - ylo);
    f.fade = c.fade(0.5 * (f.qlo.y + f.qhi.y), -D.y);
    f.down = true;
    return f;
}
// EX_GLINT: the sun mirrored off the outside of the wall at that point: where the lit part of its height lands
struct ExGlint {
    V3 qlo, qhi;
    double fade;
};
inline ExGlint ex_glint(const ExCtx& c, const ExWallPt& m) {
    ExGlint g{V3{0, 0, 0}, V3{0, 0, 0}, 0};
    if (!(m.ci > 1e-6) || !(m.hi > m.lo)) return g;
    const V3 D = c.dir(V3{c.L.x + 2.0 * m.ci * m.n.x, c.L.y + 2.0 * m.ci * m.n.y, c.L.z});
    if (!(D.y < -1e-6)) return g;
    g.qlo = c.land(c.world(m.p, m.lo), D);
    g.qhi = c.land(c.world(m.p, m.hi), D);
    g.fade = c.fade(0.5 * (g.qlo.y + g.qhi.y), -D.y);
    return g;
}

// Corner `corner` (see ex_strip_mesh) of part `part` of stretch m, slot `slot`, of the strip of one family, for
// colour step k (EX_CUT: k + EXT_STEPS for the rim at -z). find, tex: the solid's two textures (ex_walk).
inline ExVert ex_strip_vertex(const ExCtx& c, const float* find, const float* tex, int family, int k, int m, int part, int corner, int slot) {
    const Outline& ol = *c.ol;
    ExVert v;
    if (m >= (int)ol.stretches.size() || !(c.Lw.y < -1e-6)) return v;
    const double ze = -c.top * c.h, cosCap = std::fabs(c.L.z), down = -c.Lw.y;
    const int e = ol.stretches[(size_t)m].edge;
    const bool end = corner >= 2, far = corner == 1 || corner == 2;
    const ExChain ch{&ol, find + (size_t)ex_find_line(family, family == EX_CUT ? 0 : k) * EXT_COLUMNS * 4, tex};
    if (ch.ends(m)) return v;
    // the point of the chain this corner hangs on, and the one at the other end of its part
    const ExAt self = end ? (part == 2 ? ExAt{m + 1, 0, true} : ExAt{m, 2 * part + 1, true}) : ExAt{m, 2 * part, true};
    const ExAt before = ch.prev(self), after = ch.next(self), other = end ? before : after;
    if (family == EX_SHADOW) {
        if (!(c.lat > 1e-6) || !other.is) return v;
        auto up = [&](ExAt a) { return a.which < 4 ? (double)ch.texel(a.m, EXT_ROW_SHADOW)[a.which] : (double)ch.texel(a.m, EXT_ROW_SHADOW + 1)[0]; };
        const V2 p = ol.point(e, ch.u(self)), po = ol.point(e, ch.u(other));
        if (!(cross(c.sh, end ? p - po : po - p) > 0)) return v;
        // (it starts a hair inside the outline, so that no crack can open between the two)
        v.q = c.land(far ? c.world(p, ze + c.top * up(self)) : c.world(p - c.sh * EXT_OVER, ze), c.Lw);
        v.rgb = V3{1, 1, 1};
        v.fly = v.q.y;
        v.ok = true;
        return v;
    }
    const ExAt nb[2] = {before, after};
    const int oi = end ? 0 : 1;  // which of the two neighbours is the other end of this part
    if (family == EX_MIRROR || family == EX_FOOT) {
        if (!(c.lat > 1e-6) || !(cosCap > 1e-6) || !other.is) return v;
        const ExMirrorPt ms = ex_mirror_pt(c, ch, self);
        ExMirrorPt mn[2] = {ms, ms};
        bool has[2] = {false, false};
        for (int i = 0; i < 2; ++i) {
            if (!nb[i].is) continue;
            mn[i] = ex_mirror_pt(c, ch, nb[i]);
            has[i] = cross(c.sh, i == 0 ? ms.p - mn[i].p : mn[i].p - ms.p) > 0;  // (the wall between must look away from the sun)
        }
        if (!has[oi]) return v;
        if (family == EX_MIRROR) {
            if (k >= EXT_CAP_STEPS) return v;
            const double n = c.capN[k], S = c.capStep(n), T = (1.0 - fresnel(cosCap, 1.0, n)) * (1.0 - fresnel(cosCap, 1.0, n));
            if (slot == 0) {
                // out of the window, as it is. (It reaches a hair past the edge of the shifted outline, so that
                // no line of light is left standing there.)
                const double first = std::max(0.0, S - ms.back);
                v.q = c.land(c.world(ms.p + c.sh * (far ? S + EXT_OVER : (first > 0 ? first - EXT_OVER : 0.0)), ze), c.Lw);
                v.rgb = c.capRgb[k] * (-T * down);
                v.fly = v.q.y;
                v.ok = true;
                return v;
            }
            const int cell = slot - 1;
            const ExMirrorCell cs = ex_mirror_cell(c, ms, cell, S);
            // how much the mirror squeezes the light: what the strip is on the cap over what it is on the floor,
            // over the pieces on both sides of this point
            double num = 0, den = 0, low = cs.fade;
            bool drawn = cs.fade > 0;
            for (int i = 0; i < 2; ++i) {
                if (!has[i]) continue;
                const ExMirrorCell cn = ex_mirror_cell(c, mn[i], cell, S);
                low = std::min(low, cn.fade);
                if (i == oi) drawn = drawn && cn.fade > 0;
                const V2 u0 = ms.p + c.sh * cs.lo, u1 = ms.p + c.sh * cs.hi, u2 = mn[i].p + c.sh * cn.hi, u3 = mn[i].p + c.sh * cn.lo;
                num += 0.5 * std::fabs(cross(u2 - u0, u3 - u1)) * c.sc * c.sc * cosCap;
                den += ex_area(cs.qlo, cs.qhi, cn.qhi, cn.qlo);
            }
            if (!drawn) return v;  // (a strip that is dark at either end is not drawn at all)
            v.q = far ? cs.qhi : cs.qlo;
            v.rgb = c.capRgb[k] * (T * low * (den * EXT_CAP > num ? num / den : EXT_CAP));
            v.fly = v.q.y;
            v.ok = true;
            return v;
        }
        if (!(c.leg > 0) || k >= EXT_STEPS) return v;
        const ExFoot fs = ex_foot(c, ms, k);
        double num = 0, den = 0, low = fs.fade;
        bool drawn = fs.fade > 0;
        for (int i = 0; i < 2; ++i) {
            if (!has[i]) continue;
            const ExFoot fn = ex_foot(c, mn[i], k);
            low = std::min(low, fn.fade);  // (where the line ends it is dark already)
            if (i == oi) drawn = drawn && fn.fade > 0;
            num += length(mn[i].p - ms.p) * ol.chamfer * c.sc * c.sc;
            den += ex_area(fs.qin, fs.qout, fn.qout, fn.qin);
        }
        if (!drawn) return v;
        v.q = far ? fs.qout : fs.qin;
        v.rgb = c.stepRgb[k] * (fs.e * low * (den * EXT_CAP > num ? num / den : EXT_CAP));
        v.fly = v.q.y + fs.fly;
        v.ok = true;
        return v;
    }
    if (family == EX_WALL || family == EX_GLINT) {
        if (!(c.lat > 1e-6) || k >= EXT_STEPS || !other.is) return v;
        const ExWallPt ws = ex_wall_pt(c, ch, self, k);
        ExWallPt wn[2] = {ws, ws};
        bool has[2] = {false, false};
        for (int i = 0; i < 2; ++i)
            if (nb[i].is) {
                wn[i] = ex_wall_pt(c, ch, nb[i], k);
                has[i] = true;
            }
        if (family == EX_GLINT) {
            if (!c.mirrors) return v;
            const ExGlint gs = ex_glint(c, ws);
            double num = 0, den = 0, low = gs.fade;
            bool drawn = gs.fade > 0;
            for (int i = 0; i < 2; ++i) {
                if (!has[i]) continue;
                const ExGlint gn = ex_glint(c, wn[i]);
                if (i == oi) drawn = drawn && gn.fade > 0;
                if (!(wn[i].ci > 1e-6)) continue;
                low = std::min(low, gn.fade);
                num += length(wn[i].p - ws.p) * 0.5 * (ws.hi - ws.lo + wn[i].hi - wn[i].lo) * c.sc * c.sc;
                den += ex_area(gs.qlo, gs.qhi, gn.qhi, gn.qlo);
            }
            if (!drawn) return v;
            v.q = far ? gs.qhi : gs.qlo;
            const double eg = ws.ci * fresnel(ws.ci, 1.0, c.stepN[k]) * low * (den * EXT_CAP > num ? num / den : EXT_CAP);
            v.rgb = V3{eg, eg, eg};
            v.fly = v.q.y;
            v.ok = true;
            return v;
        }
        const ExWallPt& wo = wn[oi];
        if (!(ws.weight >= 0) || !(wo.weight >= 0)) return v;
        // Three folds are drawn, from the lowest that the piece between this point and the other end of the part reaches.
        const int j = (int)std::floor((std::min(ws.lo + ws.shift, wo.lo + wo.shift) + c.h) / (2.0 * c.h)) + slot;
        const ExWallFold fs = ex_wall_fold(c, ws, j);
        // how bright: the sun this fold's piece of wall catches over the floor it lands on, on both sides of this point
        double num = 0, den = 0, low = fs.fade;
        bool drawn = fs.fade > 0;
        for (int i = 0; i < 2; ++i) {
            if (!has[i] || !(wn[i].weight >= 0)) continue;
            const ExWallFold fn = ex_wall_fold(c, wn[i], j);
            low = std::min(low, fn.fade);
            if (i == oi) drawn = drawn && fn.fade > 0;
            if (!fn.down || !fs.down) continue;
            num += length(wn[i].p - ws.p) * 0.5 * (fs.height + fn.height) * c.sc * c.sc;
            den += ex_area(fs.qlo, fs.qhi, fn.qhi, fn.qlo);
        }
        if (!drawn) return v;
        v.q = far ? fs.qhi : fs.qlo;
        v.rgb = c.stepRgb[k] * (ws.ci * ws.weight * low * (den * EXT_CAP > num ? num / den : EXT_CAP));
        v.fly = v.q.y + std::min(0.5 * v.q.y * ws.gap, EXT_SOFT * c.size) / c.sunTan;
        v.ok = true;
        return v;
    }
    if (family == EX_CUT) {
        // no place of change is looked for here: the line is thin; it goes dark towards a change instead
        if (part != 0 || !(c.leg > 0) || k >= 2 * EXT_STEPS) return v;
        const int step = k % EXT_STEPS, base = EXT_ROW_CUT + 2 * k, baseNext = EXT_ROW_CUT + 2 * (k - step + ex_next_step(step));
        const int ms = end ? m + 1 : m, mo = end ? m : m + 1;
        const float *s0 = ch.texel(ms, base), *s1 = ch.texel(ms, base + 1), *o1 = ch.texel(mo, base + 1);
        if (!(s1[0] > 0) || !(o1[0] > 0) || s1[3] != o1[3] || !(s1[1] > 0) || !(o1[1] > 0)) return v;
        const V2 ps = ol.point(e, ol.stretches[(size_t)ms].u0);
        double num = 0, den = 0, low = s1[1];
        for (int i = 0; i < 2; ++i) {
            const int mn = i == 0 ? ms - 1 : ms + 1;
            if (i == 0 ? ch.first(ms) : ch.ends(ms)) continue;
            const float *n0 = ch.texel(mn, base), *n1 = ch.texel(mn, base + 1);
            if (!(n1[0] > 0) || n1[3] != s1[3]) {
                low = 0;  // the line ends, or changes its way, next to this point
                continue;
            }
            low = std::min(low, (double)n1[1]);
            const V2 pn = ol.point(e, ol.stretches[(size_t)mn].u0);
            num += length(pn - ps) * ol.chamfer * c.sc * c.sc;
            den += ex_area(V3{s0[0], 0, s0[1]}, V3{s0[2], 0, s0[3]}, V3{n0[2], 0, n0[3]}, V3{n0[0], 0, n0[1]});
        }
        const float *g0 = ch.texel(ms, baseNext), *g1 = ch.texel(ms, baseNext + 1);
        double fly = s1[2];
        if (g1[0] > 0 && g1[3] == s1[3]) {
            const double dx = 0.5 * ((double)g0[0] + g0[2] - s0[0] - s0[2]), dzz = 0.5 * ((double)g0[1] + g0[3] - s0[1] - s0[3]);
            fly += std::min(0.5 * std::sqrt(dx * dx + dzz * dzz), EXT_SOFT * c.size) / c.sunTan;
        }
        v.q = far ? V3{s0[2], s1[2], s0[3]} : V3{s0[0], s1[2], s0[1]};
        v.rgb = c.stepRgb[step] * ((double)s1[0] * low * (den * EXT_CAP > num ? num / den : EXT_CAP));
        v.fly = fly;
        v.ok = true;
        return v;
    }
    return v;
}

// -- the outline's own triangles (extrude_cap.mat)
// role 0: the shadow of the cap. 1: the window (light in through one cap and out through the other). 2: the sun
// mirrored off the cap.
// The window. Light that came in through the cap at p and met no wall leaves the other cap at p + (its sideways
// step). So a point q of the other cap is lit in a colour if the glass goes on behind it (against the light) for
// at least that colour's step: the shader draws the outline once and asks that of every pixel (ex_window). Here,
// on the CPU, the same picture is made of triangles: the outline shifted by the step, in every colour (role
// 100 + k), and the strips that met a wall taken out again as light of the opposite sign (EX_MIRROR, slot 0).
// (The shader does not do it that way because sums that cancel leave rounding dust in a half-float picture.)
inline ExVert ex_cap_vertex(const ExCtx& c, int role, int tri, int corner) {
    const Outline& ol = *c.ol;
    ExVert v;
    if (tri < 0 || (size_t)tri * 3 + 2 >= ol.tris.size()) return v;
    if (!(c.Lw.y < -1e-6)) return v;
    const V2 p = ol.pts[(size_t)ol.tris[(size_t)tri * 3 + (size_t)corner]];
    const double ze = -c.top * c.h, cosCap = std::fabs(c.L.z), down = -c.Lw.y;
    if (role == 0) {
        v.q = c.land(c.world(p, ze), c.Lw);
        v.rgb = V3{1, 1, 1};
    } else if (role >= 100) {
        if (!(cosCap > 1e-6)) return v;
        const int k = role - 100;
        const double n = c.capN[k], T = 1.0 - fresnel(cosCap, 1.0, n);
        v.q = c.land(c.world(p + c.sh * c.capStep(n), ze), c.Lw);
        v.rgb = c.capRgb[k] * (T * T * down);
    } else if (role == 2) {
        if (!c.mirrors || !(cosCap > 1e-6)) return v;
        const V3 D = c.dir(V3{c.L.x, c.L.y, -c.L.z});
        if (!(D.y < -1e-6)) return v;
        v.q = c.land(c.world(p, c.top * c.h), D);
        // (faded as a whole, by where the middle of the outline lands: either all of it is drawn or none)
        const double f = c.fade(c.land(c.world(V2{0, 0}, c.top * c.h), D).y, -D.y) * fresnel(cosCap, 1.0, c.stepN[EXT_STEPS / 2]) * -D.y;
        if (!(f > 0)) return v;
        v.rgb = V3{f, f, f};
    } else {
        return v;
    }
    v.fly = v.q.y;
    v.ok = true;
    return v;
}
// The window as the shader shades it: the light at point q of the cap the light leaves through (per unit of sun).
// width: how far apart neighbouring pixels are along the light's sideways way (the edge is that soft).
inline V3 ex_window(const ExCtx& c, V2 q, double width) {
    const double cosCap = std::fabs(c.L.z), down = -c.Lw.y;
    V3 rgb{0, 0, 0};
    if (!(cosCap > 1e-6)) return rgb;
    double behind = EXT_BIG;
    OutlineHit hit;
    if (c.lat > 1e-6 && outline_cast(*c.ol, q, c.sh * -1.0, 0.0, 0, hit)) behind = hit.t;
    for (int k = 0; k < EXT_CAP_STEPS; ++k) {
        const double n = c.capN[k], T = 1.0 - fresnel(cosCap, 1.0, n);
        const double lit = std::min(1.0, std::max(0.0, (behind - c.capStep(n)) / std::max(width, 1e-9) + 0.5));
        rgb = rgb + c.capRgb[k] * (T * T * down * lit);
    }
    return rgb;
}

inline void ex_emit(std::vector<float>& tri, const ExVert* v, int count) {
    for (int i = 0; i < count; ++i)
        if (!v[i].ok) return;
    for (int i = 1; i + 1 < count; ++i)
        for (int k : {0, i, i + 1}) tri.insert(tri.end(), {(float)v[k].q.x, (float)v[k].q.z, (float)v[k].rgb.x, (float)v[k].rgb.y, (float)v[k].rgb.z, (float)v[k].fly});
}
inline void ex_strips(const ExCtx& c, const float* find, const float* tex, int family, int k, std::vector<float>& tri) {
    const int cols = (int)c.ol->stretches.size(), slots = ex_slots(family);
    for (int slot = 0; slot < slots; ++slot)
        for (int m = 0; m < cols; ++m)
            for (int part = 0; part < 3; ++part) {
                ExVert v[4];
                for (int i = 0; i < 4; ++i) v[i] = ex_strip_vertex(c, find, tex, family, k, m, part, i, slot);
                ex_emit(tri, v, 4);
            }
}

}  // namespace detail

// The shadow, as triangles (x, z, 1, 1, 1, distance flown) that cover it exactly once: what the renderer's
// shaders draw, worked out here on the CPU (desk check, comparison).
inline void extrude_silhouette(const Solid& so, V3 sun, double floorY, std::vector<float>& tri) {
    using namespace detail;
    Params par;
    par.sun = sun;
    par.floorY = floorY;
    const ExCtx c = ex_ctx(so, par);
    if (!(c.Lw.y < -1e-6)) return;
    static thread_local std::vector<float> find, tex;
    ex_walk(c, find, tex, EXT_ROW_SHADOW, EXT_ROW_MIRROR);
    for (int t = 0; t < (int)(c.ol->tris.size() / 3); ++t) {
        ExVert v[3];
        for (int i = 0; i < 3; ++i) v[i] = ex_cap_vertex(c, 0, t, i);
        ex_emit(tri, v, 3);
    }
    ex_strips(c, find.data(), tex.data(), EX_SHADOW, 0, tri);
}
// The light, as triangles (x, z, r, g, b, distance flown), appended to `tri`: likewise the CPU's copy of what the
// shaders draw. `only`: one family (ExFamily, or 100 for the outline's own triangles), -1 for all.
inline void extrude_light(const Solid& so, const Params& par, std::vector<float>& tri, CheapStats* st, int only) {
    using namespace detail;
    const ExCtx c = ex_ctx(so, par);
    if (!(c.Lw.y < -1e-6)) return;
    const size_t triStart = tri.size();
    static thread_local std::vector<float> find, tex;
    ex_walk(c, find, tex, EXT_ROW_MIRROR, EXT_ROWS);
    if (only < 0 || only == 100)
        for (int role : {100, 101, 102, 103, 2})
            for (int t = 0; t < (int)(c.ol->tris.size() / 3); ++t) {
                ExVert v[3];
                for (int i = 0; i < 3; ++i) v[i] = ex_cap_vertex(c, role, t, i);
                ex_emit(tri, v, 3);
            }
    if (only < 0 || only == EX_MIRROR)
        for (int k = 0; k < EXT_CAP_STEPS; ++k) ex_strips(c, find.data(), tex.data(), EX_MIRROR, k, tri);
    if (only < 0 || only == EX_FOOT)
        for (int k = 0; k < EXT_STEPS; ++k) ex_strips(c, find.data(), tex.data(), EX_FOOT, k, tri);
    if (only < 0 || only == EX_WALL)
        for (int k = 0; k < EXT_STEPS; ++k) ex_strips(c, find.data(), tex.data(), EX_WALL, k, tri);
    if (only < 0 || only == EX_GLINT) ex_strips(c, find.data(), tex.data(), EX_GLINT, EXT_STEPS / 2, tri);
    if (only < 0 || only == EX_CUT)
        for (int k = 0; k < 2 * EXT_STEPS; ++k) ex_strips(c, find.data(), tex.data(), EX_CUT, k, tri);
    if (st) st->patches += (int)((tri.size() - triStart) / (3 * (size_t)TRI));
}

}  // namespace beam
