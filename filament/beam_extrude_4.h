// beam_extrude.h, part 4 (cut there by size, as it was): read only through beam_extrude.h, in order.
#pragma once

namespace beam {
namespace detail {

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
