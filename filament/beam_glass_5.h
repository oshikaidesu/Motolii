// beam_glass.h, part 5 (cut there by size, as it was): read only through beam_glass.h, in order.
#pragma once

namespace beam {

// ---- the cheap version: plausible, not exact.
// The exact trace above splits every beam wherever its parts go different ways; with a few solids that is
// hundreds of ways and far too slow for a frame. Here each lit face (or each piece of it, see `pieces`) sends ONE
// ray from its middle. Wherever that ray goes - into the glass, mirrored inside, out, on through another solid,
// down to the floor - the whole piece is sent the same way as one flat patch. Nothing is clipped, nothing is
// split, so pieces of light can overlap or leave gaps where the true beam would have divided. The cost is
// (lit pieces) x (colour steps), whatever the number of solids.
struct CheapStats {
    int pieces = 0, patches = 0, rays = 0;
    int alike = 0;  // balls: their light is the one shared shape (see "A ball by likeness")
};
// beam_extrude.h: one ray against a solid with dents and holes; its light and its shadow as the renderer's shaders draw them
inline bool extrude_meet(const Solid& so, V3 o, V3 d, double& t, V3& n, int& face);
inline bool extrude_leave(const Solid& so, V3 p, V3 d, double& t, V3& n, int& face);
inline void extrude_light(const Solid& so, const Params& par, std::vector<float>& tri, CheapStats* st, int only);
inline void extrude_silhouette(const Solid& so, V3 sun, double floorY, std::vector<float>& tri);
namespace detail {

// A ray in the air meets one solid: how far, the outward normal there, and which face (FLAT: its index;
// ROD: 0 side, 1 the +axis end, 2 the -axis end; BALL: 0).
// A cone, a cone with its tip cut off, or (both ends as wide) a rod: one shape for the formulas of light.
struct Taper {
    V3 c, ax;     // the middle of the base, and the unit axis from there to the top
    double r, rt; // radius of the base and of the top (0: a cone with its tip)
    double h;     // height
};
inline Taper taper_of(const Solid& so) {
    if (so.kind == Solid::ROD) return Taper{so.c - so.ax * so.h, so.ax, so.r, so.r, 2.0 * so.h};
    return Taper{so.c, so.ax, so.r, 0.0, so.h};
}
// Every place a line o + t d crosses its surface (side: face 0, base: face 1, top: face 2): t and the outward
// normal. Returns how many (0 to 2, the shape is convex; touching points may be missing).
inline int taper_crossings(const Taper& so, V3 o, V3 d, double t[2], V3 n[2], int face[2]) {
    const V3 w = o - so.c;
    const double k = (so.r - so.rt) / so.h, y0 = dot(w, so.ax), da = dot(d, so.ax);
    const V3 wp = w - so.ax * y0, dp = d - so.ax * da;
    int count = 0;
    auto add = [&](double tt, V3 nn, int f) {
        if (count < 2) {
            t[count] = tt;
            n[count] = nn;
            face[count++] = f;
        }
    };
    // the side: |across| = r - k y, for y between base and top
    const double A = dot(dp, dp) - k * k * da * da, B = dot(wp, dp) + k * da * (so.r - k * y0), C = dot(wp, wp) - (so.r - k * y0) * (so.r - k * y0);
    double root[2];
    int roots = 0;
    if (std::fabs(A) > 1e-14) {
        const double disc = B * B - A * C;
        if (disc > 0) {
            const double sq = std::sqrt(disc);
            root[0] = (-B - sq) / A;
            root[1] = (-B + sq) / A;
            roots = 2;
        }
    } else if (std::fabs(B) > 1e-14) {
        root[0] = -C / (2.0 * B);
        roots = 1;
    }
    const double slope = 1.0 / std::sqrt(1.0 + k * k);
    for (int i = 0; i < roots; ++i) {
        const double y = y0 + root[i] * da;
        if (y < 0 || y > so.h) continue;
        const V3 q = wp + dp * root[i];
        const double ql = length(q);
        add(root[i], ql > 1e-12 * so.r ? (q * (1.0 / ql) + so.ax * k) * slope : so.ax, 0);
    }
    // the base and the top
    if (std::fabs(da) > 1e-14) {
        const double tb = -y0 / da;
        const V3 q = wp + dp * tb;
        if (dot(q, q) <= so.r * so.r) add(tb, so.ax * -1.0, 1);
        if (so.rt > 0) {
            const double tu = (so.h - y0) / da;
            const V3 qu = wp + dp * tu;
            if (dot(qu, qu) <= so.rt * so.rt) add(tu, so.ax, 2);
        }
    }
    if (count == 2 && t[1] < t[0]) {
        std::swap(t[0], t[1]);
        std::swap(n[0], n[1]);
        std::swap(face[0], face[1]);
    }
    return count;
}
inline int cone_crossings(const Solid& so, V3 o, V3 d, double t[2], V3 n[2], int face[2]) { return taper_crossings(taper_of(so), o, d, t, n, face); }
inline bool meet(const Solid& so, V3 o, V3 d, double& t, V3& n, int& face) {
    if (so.kind == Solid::EXTRUDE) return extrude_meet(so, o, d, t, n, face);
    if (so.kind == Solid::CONE) {
        double tt[2];
        V3 nn[2];
        int ff[2];
        if (cone_crossings(so, o, d, tt, nn, ff) < 2 || tt[0] < -1e-9 || dot(nn[0], d) >= 0) return false;
        t = tt[0];
        n = nn[0];
        face = ff[0];
        return true;
    }
    if (so.kind == Solid::FLAT) {
        double tin = -1e300, tout = 1e300;
        int gin = -1;
        for (int j = 0; j < (int)so.planes.size(); ++j) {
            if (so.faces[(size_t)j].n == 0) continue;
            const double dj = dot(so.planes[(size_t)j].n, d), off = so.planes[(size_t)j].o - dot(so.planes[(size_t)j].n, o);
            if (dj < -1e-12) {
                if (off / dj > tin) {
                    tin = off / dj;
                    gin = j;
                }
            } else if (dj > 1e-12) {
                tout = std::min(tout, off / dj);
            } else if (off < 0) {
                return false;
            }
        }
        if (gin < 0 || tin >= tout || tin < -1e-9) return false;
        t = tin;
        n = so.planes[(size_t)gin].n;
        face = gin;
        return true;
    }
    if (so.kind == Solid::BALL) {
        const V3 w = o - so.c;
        const double b = dot(w, d), cc = dot(w, w) - so.r * so.r, disc = b * b - cc;
        if (disc <= 0) return false;
        t = -b - std::sqrt(disc);
        if (t < -1e-9) return false;
        n = (o + d * t - so.c) * (1.0 / so.r);
        face = 0;
        return true;
    }
    // ROD: the part of an endless cylinder between two planes
    const V3 w = o - so.c;
    const double wa = dot(w, so.ax), da = dot(d, so.ax);
    const V3 wp = w - so.ax * wa, dp = d - so.ax * da;
    double tinS = -1e300, toutS = 1e300, tinC = -1e300, toutC = 1e300;
    if (std::fabs(da) > 1e-12) {
        const double t1 = (-so.h - wa) / da, t2 = (so.h - wa) / da;
        tinS = std::min(t1, t2);
        toutS = std::max(t1, t2);
    } else if (std::fabs(wa) > so.h) {
        return false;
    }
    const double A = dot(dp, dp), B = dot(wp, dp), C = dot(wp, wp) - so.r * so.r;
    if (A > 1e-18) {
        const double disc = B * B - A * C;
        if (disc <= 0) return false;
        const double sq = std::sqrt(disc);
        tinC = (-B - sq) / A;
        toutC = (-B + sq) / A;
    } else if (C > 0) {
        return false;
    }
    const double tin = std::max(tinS, tinC), tout = std::min(toutS, toutC);
    if (tin >= tout || tin < -1e-9) return false;
    t = tin;
    if (tinC >= tinS) {
        const V3 q = o + d * t - so.c;
        n = normalize(q - so.ax * dot(q, so.ax));
        face = 0;
    } else {
        n = da < 0 ? so.ax : so.ax * -1.0;
        face = da < 0 ? 1 : 2;
    }
    return true;
}
// A ray inside a solid (starting on face `from`, or anywhere inside with from = -1) reaches its surface.
inline bool leave(const Solid& so, V3 p, V3 d, int from, double& t, V3& n, int& face) {
    if (so.kind == Solid::EXTRUDE) return extrude_leave(so, p, d, t, n, face);
    if (so.kind == Solid::CONE) {
        double tt[2];
        V3 nn[2];
        int ff[2];
        const int count = cone_crossings(so, p, d, tt, nn, ff);
        for (int i = 0; i < count; ++i)
            if (tt[i] > 1e-9 * so.r && dot(nn[i], d) > 0) {
                t = tt[i];
                n = nn[i];
                face = ff[i];
                return true;
            }
        return false;
    }
    if (so.kind == Solid::FLAT) {
        int best = -1;
        double tb = 1e300;
        for (int j = 0; j < (int)so.planes.size(); ++j) {
            if (j == from || so.faces[(size_t)j].n == 0) continue;
            const double dj = dot(so.planes[(size_t)j].n, d);
            if (dj <= 1e-12) continue;
            const double tt = (so.planes[(size_t)j].o - dot(so.planes[(size_t)j].n, p)) / dj;
            if (tt < tb) {
                tb = tt;
                best = j;
            }
        }
        if (best < 0) return false;
        t = tb;
        n = so.planes[(size_t)best].n;
        face = best;
        return true;
    }
    if (so.kind == Solid::BALL) {
        const V3 w = p - so.c;
        const double b = dot(w, d), disc = b * b - (dot(w, w) - so.r * so.r);
        t = -b + std::sqrt(std::max(0.0, disc));
        if (t <= 1e-12 * so.r) return false;
        n = (p + d * t - so.c) * (1.0 / so.r);
        face = 0;
        return true;
    }
    const V3 w = p - so.c;
    const double wa = dot(w, so.ax), da = dot(d, so.ax);
    const V3 wp = w - so.ax * wa, dp = d - so.ax * da;
    double tS = 1e300, tC = 1e300;
    if (da > 1e-12) tS = (so.h - wa) / da;
    else if (da < -1e-12) tS = (-so.h - wa) / da;
    const double A = dot(dp, dp), B = dot(wp, dp), C = dot(wp, wp) - so.r * so.r;
    if (A > 1e-18) tC = (-B + std::sqrt(std::max(0.0, B * B - A * C))) / A;
    t = std::min(tS, tC);
    if (!(t < 1e299) || t <= 1e-12 * so.r) return false;
    if (tC <= tS) {
        const V3 q = p + d * t - so.c;
        n = normalize(q - so.ax * dot(q, so.ax));
        face = 0;
    } else {
        n = da > 0 ? so.ax : so.ax * -1.0;
        face = da > 0 ? 1 : 2;
    }
    return true;
}
// First solid a ray in the air meets (other than `from`): which, how far, through which face, and the normal there.
inline bool first_solid(const Scene& sc, V3 o, V3 d, int from, int& S, int& g, double& t, V3& n) {
    t = 1e300;
    S = -1;
    for (int k = 0; k < (int)sc.solids.size(); ++k) {
        if (k == from) continue;
        const V3 rel = sc.centre[(size_t)k] - o;
        const double along = dot(rel, d);
        if (along < -sc.radius[(size_t)k] || dot(rel, rel) - along * along > sc.radius[(size_t)k] * sc.radius[(size_t)k]) continue;
        double tk;
        V3 nk;
        int gk;
        if (meet(sc.solids[(size_t)k], o, d, tk, nk, gk) && tk < t) {
            t = tk;
            S = k;
            g = gk;
            n = nk;
        }
    }
    return S >= 0;
}

// The way one ray takes from a point on solid A, as a list of planes to slide along and the direction between
// them. Returns false if the light never reaches the floor.
struct Leg {
    V3 d;        // direction of travel on this leg
    Plane to;    // the plane the leg ends on (the floor for the last one)
};
struct Route {
    Leg leg[Patch::SEQ + 2];
    int n = 0;
    double gain = 0;  // floor irradiance per unit of sun irradiance (flux through the piece / landed area)
    double weight = 1; // the share of the light that is left after every face on the way (Fresnel only)
    uint64_t key = 0; // which faces were touched: two indices with the same key take the same way
};
// `start` lies on solid A where its outward normal is `nStart` (face `fStart`).
inline bool route(const Scene& sc, const Params& par, double n, int A, int fStart, V3 start, V3 nStart, bool mirrored, Route& r) {
    const V3 L = normalize(par.sun);
    r.n = 0;
    r.weight = 1.0;
    r.key = 1469598103934665603ull;
    auto touch = [&](int s, int g, int kind) { r.key = (r.key ^ (uint64_t)(((s << 6 | g) << 2 | kind) + 1)) * 1099511628211ull; };
    V3 p = start, d = L;
    double ci = -dot(nStart, d);
    if (ci <= 1e-9) return false;
    double irr = ci;       // flux per unit area of the plane the light is on now
    double cosOn = ci;     // |n.d| of that plane for the leg that follows
    int S = A, g = fStart, entries = 0;
    V3 ng = nStart;
    bool outside = true;   // about to meet solid S (normal ng) from outside
    for (int guard = 0; guard < Patch::SEQ; ++guard) {
        if (outside) {
            ci = -dot(ng, d);
            const double Rf = fresnel(ci, 1.0, n);
            if (mirrored && guard == 0) {
                irr *= Rf;
                r.weight *= Rf;
                d = d + ng * (2.0 * ci);
                touch(S, g, 1);
                // mirrored off the first face: on through the air
            } else {
                if (entries >= par.maxSolids) return false;
                ++entries;
                irr *= 1.0 - Rf;
                r.weight *= 1.0 - Rf;
                V3 in;
                refract(d, ng, 1.0 / n, in);
                d = in;
                touch(S, g, 0);
                cosOn = std::fabs(dot(ng, d));
                // inside: mirror until the glass lets the light out
                const Solid& so = sc.solids[(size_t)S];
                int from = so.kind == Solid::FLAT ? g : -1;
                bool out = false;
                for (int b = 0; b <= par.maxBounces; ++b) {
                    double tb;
                    V3 nj;
                    int best;
                    if (!leave(so, p, d, from, tb, nj, best)) return false;
                    p = p + d * tb;
                    r.leg[r.n++] = Leg{d, Plane{nj, dot(nj, p)}};  // (a round face counts as its tangent plane here)
                    const double db = dot(nj, d);
                    irr *= db / cosOn;
                    const double Ri = fresnel(db, n, 1.0);
                    if (Ri < 1.0) {
                        irr *= 1.0 - Ri;
                        r.weight *= 1.0 - Ri;
                        V3 o;
                        refract(d, nj * -1.0, n, o);
                        d = o;
                        cosOn = std::fabs(dot(nj, d));
                        touch(S, best, 2);
                        out = true;
                        break;
                    }
                    d = d - nj * (2.0 * db);
                    cosOn = db;
                    if (so.kind == Solid::FLAT) from = best;
                    touch(S, best, 3);
                }
                if (!out) return false;  // still inside after the allowed mirrors: lost
            }
            if (mirrored && guard == 0) cosOn = ci;
            outside = false;
        }
        // in the air: another solid, or the floor
        int S2, g2;
        double t2;
        V3 n2;
        if (sc.solids.size() > 1 && first_solid(sc, p, d, S, S2, g2, t2, n2)) {
            p = p + d * t2;
            r.leg[r.n++] = Leg{d, Plane{n2, dot(n2, p)}};
            irr *= -dot(n2, d) / cosOn;
            S = S2;
            g = g2;
            ng = n2;
            outside = true;
            continue;
        }
        if (d.y > -1e-9) return false;  // up into the sky
        r.leg[r.n++] = Leg{d, Plane{V3{0, 1, 0}, par.floorY}};
        r.gain = irr * (-d.y) / cosOn;
        return true;
    }
    return false;
}
// Sends a polygon lying on the first face along a route. Corners come out as (x, travel, z).
inline void send(const Route& r, const Poly& P, double floorY, Poly& F) {
    F.n = 0;
    for (int i = 0; i < P.n; ++i) {
        V3 v = P.p[(size_t)i];
        double last = 0;
        for (int k = 0; k < r.n; ++k) {
            const double den = dot(r.leg[k].to.n, r.leg[k].d);
            double t = (r.leg[k].to.o - dot(r.leg[k].to.n, v)) / den;
            if (k == r.n - 1 && t < 0) t = 0;  // already on the floor
            v = v + r.leg[k].d * t;
            last = t;
        }
        (void)floorY;
        F.push({v.x, last, v.z});
    }
}

}  // namespace detail

}  // namespace beam
