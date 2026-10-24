// beam_extrude.h, part 3 (cut there by size, as it was): read only through beam_extrude.h, in order.
#pragma once

namespace beam {
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

}  // namespace detail
}  // namespace beam
