// BEAMS=1: the light of a flat-faced solid by splitting beams exactly (beam tracing, Heckbert & Hanrahan 1984, as the
// gem and ice-crystal tracers do it: Voelker & Hamann, Konoshonkin et al.).
// A beam is a flat polygon of light and the one direction it travels in (the sun's rays are parallel, the faces flat,
// so it stays that). Where it meets the faces ahead it is CUT along their outlines: every piece that is left is the
// light that goes on that way, and nothing else is looked at. Seen along the light the faces ahead cover the beam
// without gaps or overlaps, so the pieces add up to the beam: no way is missed and none is made up. A piece that
// leaves the glass is drawn as it is; its outline is the outline of that patch of light.
// Colour films: the films cut the solid into cells (each convex); a film is a wall between two cells that mirrors
// some colours and lets the rest through into the next cell.
// On the CPU, per solid and colour step. Other solids are not looked at (they neither shade nor bend this light).
#pragma once
#include "beam_glass.h"

namespace beam {

struct SplitCell {
    std::vector<Plane> planes;   // outward
    std::vector<Poly> faces;     // faces[i] on planes[i]; n == 0: none
    std::vector<int> what;       // per face: >= 0: that face of the solid; < 0: film -(what + 1)
    int across[4] = {-1, -1, -1, -1};   // the cell on the other side of film f
};
struct SplitShape {
    std::vector<SplitCell> cells;
};
// The cells that the films named in `use` (one bit per film) cut the solid into. Once per shape and set of films: a
// colour only needs the films that mirror it; the others are not there for it and must not cut its light.
inline SplitShape split_shape(const Solid& local, int use = 15) {
    SplitShape sh;
    const int films = (int)std::min<size_t>(4, local.films.size());
    double reach = 0;
    for (const Poly& P : local.faces)
        for (int i = 0; i < P.n; ++i) reach = std::max(reach, length(P.p[(size_t)i]));
    std::vector<int> cellOf((size_t)1 << films, -1);
    for (int mask = 0; mask < (1 << films); ++mask) {
        if (mask & ~use) continue;   // (films not in use have one side only)
        std::vector<Plane> pl;
        std::vector<int> what;
        for (size_t f = 0; f < local.planes.size(); ++f)
            if (local.faces[f].n >= 3) { pl.push_back(local.planes[f]); what.push_back((int)f); }
        for (int f = 0; f < films; ++f) {
            if (!((use >> f) & 1)) continue;
            const Plane& fp = local.films[(size_t)f];
            pl.push_back((mask >> f) & 1 ? Plane{fp.n * -1.0, -fp.o} : fp);
            what.push_back(-(f + 1));
        }
        Solid c = from_planes(pl, 4.0 * reach + 1.0);
        int real = 0;
        for (const Poly& P : c.faces) real += P.n >= 3 && area(P) > 1e-10;
        if (real < 4) continue;
        SplitCell cell;
        cell.planes = c.planes;
        cell.faces = c.faces;
        cell.what = what;
        for (Poly& P : cell.faces)
            if (P.n < 3 || area(P) <= 1e-10) P.n = 0;
        cellOf[(size_t)mask] = (int)sh.cells.size();
        sh.cells.push_back(std::move(cell));
    }
    for (int mask = 0; mask < (1 << films); ++mask)
        if (cellOf[(size_t)mask] >= 0)
            for (int f = 0; f < films; ++f)
                if ((use >> f) & 1) sh.cells[(size_t)cellOf[(size_t)mask]].across[f] = cellOf[(size_t)(mask ^ (1 << f))];
    return sh;
}

struct SplitStats {
    long beams = 0, leaves = 0, corners = 0;
    double light[12] = {};   // light that left the glass after meeting this many of its faces (the last one counted)
    int deepest = 0;
};
struct SplitBand {           // the band of sun on the floor (STRIP): across a unit way u (x, z), middle c, half width, every (0: one band)
    double ux = 1, uz = 0, c = 0, half = 0, every = 0;
};

namespace detail {
inline double fres(double ci, double n1, double n2) {
    const double s2 = (n1 / n2) * (n1 / n2) * (1.0 - ci * ci);
    if (s2 >= 1.0) return 1.0;
    const double ct = std::sqrt(1.0 - s2);
    const double rs = (n1 * ci - n2 * ct) / (n1 * ci + n2 * ct), rp = (n1 * ct - n2 * ci) / (n1 * ct + n2 * ci);
    return 0.5 * (rs * rs + rp * rp);
}
inline bool bend(V3 d, V3 nrm, double eta, V3& out) {   // nrm against d; eta = n from / n to
    const double c = -dot(nrm, d), k = 1.0 - eta * eta * (1.0 - c * c);
    if (k < 0) return false;
    out = d * eta + nrm * (eta * c - std::sqrt(k));
    return true;
}
// what is left of P (already on the face's plane) inside the convex face F with outward normal nrm
inline void cut_to_face(Poly& P, const Poly& F, V3 nrm) {
    V3 mid{0, 0, 0};
    for (int i = 0; i < F.n; ++i) mid = mid + F.p[(size_t)i];
    mid = mid * (1.0 / F.n);
    for (int i = 0; i < F.n && P.n >= 3; ++i) {
        const V3 a = F.p[(size_t)i], b = F.p[(size_t)((i + 1) % F.n)];
        V3 in = cross(nrm, b - a);
        if (dot(in, mid - a) < 0) in = in * -1.0;
        clip(P, Half{in, -dot(in, a)});
    }
}
struct SplitBeam {
    Poly poly;
    V3 d;
    double irr, cosOn;   // irradiance on the polygon's plane (sun = 1), and the cosine the light makes with that plane's normal
    int cell, faces, events;
};
}  // namespace detail

// The light one solid throws on the floor, appended to `tri` (TRI floats per corner). R, scale, at: where the solid's
// own frame is. steps: rgb and index of refraction per colour step. filmN[f]: film f mirrors light of an index
// between x and y; sh[m] is the shape cut by the films of set m (split_shape). least: a piece carrying less than this (irradiance x its area across the light, in the solid's
// own measure) is dropped. mostFaces: faces of the solid a way may meet.
inline void split_light(const SplitShape* shapes, const V3 R[3], double scale, V3 at, V3 sun, const std::vector<BallStep>& steps, const double filmN[4][2],
                        const SplitBand& band, double floorY, double least, int mostFaces, bool mirroredToo, std::vector<float>& tri, SplitStats* st) {
    using namespace detail;
    sun = normalize(sun);
    const V3 sunL = normalize(V3{dot(sun, R[0]), dot(sun, R[1]), dot(sun, R[2])});
    auto world = [&](V3 p) { return (R[0] * p.x + R[1] * p.y + R[2] * p.z) * scale + at; };
    auto turn = [&](V3 v) { return R[0] * v.x + R[1] * v.y + R[2] * v.z; };
    // a flat piece of light leaving the glass along `out`: down to the floor, as triangles
    auto land = [&](const Poly& P, V3 out, V3 rgb, double irrOnPlane, double cosOut) {
        const V3 dW = turn(out);
        if (!(dW.y < -1e-6)) return;
        const double bright = irrOnPlane * (-dW.y) / cosOut;
        V3 X[Poly::CAP];
        double fl[Poly::CAP];
        for (int i = 0; i < P.n; ++i) {
            const V3 pw = world(P.p[(size_t)i]);
            fl[i] = (floorY - pw.y) / dW.y;
            X[i] = pw + dW * fl[i];
        }
        for (int i = 1; i + 1 < P.n; ++i)
            for (int k : {0, i, i + 1})
                tri.insert(tri.end(), {(float)X[k].x, (float)X[k].z, (float)(rgb.x * bright), (float)(rgb.y * bright), (float)(rgb.z * bright), (float)fl[k]});
        if (st) { ++st->leaves; st->corners += P.n; }
    };
    // the band of sun, as halves in the solid's own frame: g(p) = m . world(p) - c, kept where |g - k every| <= half
    const V3 mW{band.ux, -(band.ux * sun.x + band.uz * sun.z) / std::min(sun.y, -1e-6), band.uz};
    const V3 mL = V3{dot(mW, R[0]), dot(mW, R[1]), dot(mW, R[2])} * scale;
    const double m0 = dot(mW, at) - band.c;

    std::vector<SplitBeam> stack;
    for (size_t s = 0; s < steps.size() + (mirroredToo ? 1 : 0); ++s) {
        const bool mirrored = s == steps.size();
        const double n = mirrored ? steps[steps.size() / 2].n : steps[s].n;
        const V3 rgb = mirrored ? V3{1, 1, 1} : steps[s].rgb;
        int use = 0;
        for (int f = 0; f < 4; ++f)
            if (n > filmN[f][0] && n < filmN[f][1]) use |= 1 << f;
        const SplitShape& sh = shapes[mirrored ? 0 : use];
        stack.clear();
        // the light as it comes in: every sunlit face of the solid, by the cells it belongs to, cut to the band of sun
        for (size_t c = 0; c < sh.cells.size(); ++c) {
            const SplitCell& cell = sh.cells[c];
            for (size_t f = 0; f < cell.faces.size(); ++f) {
                if (cell.what[f] < 0 || cell.faces[f].n < 3) continue;
                const V3 nf = cell.planes[f].n;
                const double ci = -dot(nf, sunL);
                if (!(ci > 1e-6)) continue;
                int k0 = 0, k1 = 0;
                if (band.half > 0 && band.every > 0) {
                    double lo = 1e300, hi = -1e300;
                    for (int i = 0; i < cell.faces[f].n; ++i) {
                        const double g = dot(mL, cell.faces[f].p[(size_t)i]) + m0;
                        lo = std::min(lo, g);
                        hi = std::max(hi, g);
                    }
                    k0 = (int)std::ceil((lo - band.half) / band.every);
                    k1 = (int)std::floor((hi + band.half) / band.every);
                }
                for (int k = k0; k <= k1; ++k) {
                    Poly P = cell.faces[f];
                    if (band.half > 0) {
                        const double mid = m0 - k * band.every;
                        clip(P, Half{mL, mid + band.half});
                        if (P.n >= 3) clip(P, Half{mL * -1.0, -mid + band.half});
                    }
                    if (P.n < 3 || area(P) < 1e-10) continue;
                    const double Rf = fres(ci, 1.0, n);
                    if (mirrored) {
                        land(P, sunL + nf * (2.0 * ci), rgb, ci * Rf, ci);
                        continue;
                    }
                    V3 d;
                    if (!bend(sunL, nf, 1.0 / n, d)) continue;
                    stack.push_back(SplitBeam{P, d, ci * (1.0 - Rf), std::fabs(dot(nf, d)), (int)c, 0, 0});
                }
            }
        }
        // each beam: cut by the faces of its cell that lie ahead
        while (!stack.empty()) {
            const SplitBeam b = stack.back();
            stack.pop_back();
            if (st) { ++st->beams; st->deepest = std::max(st->deepest, b.events); }
            const SplitCell& cell = sh.cells[(size_t)b.cell];
            for (size_t f = 0; f < cell.faces.size(); ++f) {
                const Poly& F = cell.faces[f];
                if (F.n < 3) continue;
                const V3 nB = cell.planes[f].n;
                const double db = dot(nB, b.d);
                if (!(db > 1e-7)) continue;
                Poly P = b.poly;
                for (int i = 0; i < P.n; ++i) P.p[(size_t)i] = P.p[(size_t)i] + b.d * ((cell.planes[f].o - dot(nB, P.p[(size_t)i])) / db);
                cut_to_face(P, F, nB);
                if (P.n < 3) continue;
                const double a = area(P);
                const double irr = b.irr * db / b.cosOn;
                if (!(a > 1e-10) || irr * a * db < least) continue;
                const int what = cell.what[f];
                if (what >= 0) {
                    const double Ri = fres(db, n, 1.0);
                    if (Ri < 1.0) {   // the glass lets it out
                        V3 out;
                        if (bend(b.d, nB * -1.0, n, out)) {
                            land(P, out, rgb, irr * (1.0 - Ri), std::fabs(dot(nB, out)));
                            if (st) st->light[std::min(11, b.faces + 1)] += irr * (1.0 - Ri) * a * db * (rgb.x + rgb.y + rgb.z) / 3.0;
                        }
                    } else if (b.faces + 1 < mostFaces && b.events < 30) {
                        stack.push_back(SplitBeam{P, b.d - nB * (2.0 * db), irr, db, b.cell, b.faces + 1, b.events + 1});
                    }
                } else {
                    const int film = -(what + 1);
                    if (b.events >= 30) continue;
                    if (n > filmN[film][0] && n < filmN[film][1]) stack.push_back(SplitBeam{P, b.d - nB * (2.0 * db), irr, db, b.cell, b.faces, b.events + 1});
                    else if (cell.across[film] >= 0) stack.push_back(SplitBeam{P, b.d, irr, db, cell.across[film], b.faces, b.events + 1});
                }
            }
        }
    }
}

}  // namespace beam
