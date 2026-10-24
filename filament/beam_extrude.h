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

}  // namespace beam

#include "beam_extrude_2.h"
#include "beam_extrude_3.h"
#include "beam_extrude_4.h"
