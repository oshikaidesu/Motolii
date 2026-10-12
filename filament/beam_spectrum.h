// SPEC=1: the exact beams of beam_split.h, with the colours spread as Sikachev, Tisevich and Ignatenko do it
// ("Rendering Smooth Spectrum Caustics on Plane for Refractive Polyhedrons", GraphiCon 2008).
// There: every corner of a piece of light is followed at three wavelengths at once, so the pieces of one way at the
// three colours have the same corners; the colours in between are not traced but drawn as in-between polygons, their
// corners taken between those of the neighbouring traced colours. Hard side edges stay; the steps between the
// colours go.
// Here: the spectrum is cut where the colour films change what they do (inside one stretch every colour meets the
// same cells), and each stretch is ONE walk through the solid. A piece keeps its corners at the stretch's two ends
// and at its middle. The MIDDLE colour decides everything (which faces are met, which corners a cut keeps, mirrored
// or let out); the two end colours follow it: their corner is where THEIR edge meets the same cutting line. A way is
// right at the middle colour and taken along a straight line (two, bent at the middle) towards the ends.
// Where an end colour would have gone another way than the middle one (met another face, or been mirrored where the
// middle leaves, or the other way round) the piece is still drawn, by the middle's decision, and counted in SpecStats:
// that is where this is not the exact light any more.
// Drawn: per piece that leaves, `copies` in-between polygons, each carrying its share of the stretch's colours and as
// bright as its own area makes it (the same light on a larger patch is dimmer).
#pragma once
#include "beam_split.h"

namespace beam {

struct SpecStats {
    long walks = 0, beams = 0, leaves = 0, polygons = 0;
    long apart = 0;                    // pieces that left where an end colour had gone another way somewhere on the way
    double light = 0, lightApart = 0;  // light (all colours) of the pieces that left, and of those counted in `apart`
    long cornerApart = 0, faceApart = 0, leaveApart = 0;   // what kind: a cut kept other corners; a face was not ahead / the piece missed it; mirrored against let out
    long mostCopies = 0;
};

namespace detail {
constexpr int SPEC_CAP = 24;
struct Poly3 {
    int n = 0;
    V3 p[3][SPEC_CAP];   // [1]: the middle colour (decides); [0], [2]: the ends of the stretch
    Poly3() {}
    Poly3(const Poly3& o) { *this = o; }
    Poly3& operator=(const Poly3& o) {   // only the corners in use (as Poly does)
        n = o.n;
        for (int c = 0; c < 3; ++c) std::memcpy(p[c], o.p[c], (size_t)o.n * sizeof(V3));
        return *this;
    }
};
inline double area3(const Poly3& P, int c) {
    V3 s{0, 0, 0};
    for (int i = 1; i + 1 < P.n; ++i) s = s + cross(P.p[c][i] - P.p[c][0], P.p[c][i + 1] - P.p[c][0]);
    return 0.5 * length(s);
}
// Keeps the part where h holds, as the MIDDLE colour sees it. Returns whether an end colour's corners lie otherwise.
inline bool clip3(Poly3& P, const Half& h) {
    double g[3][SPEC_CAP];
    bool anyNeg = false, apart = false;
    for (int q = 0; q < P.n; ++q) {
        for (int c = 0; c < 3; ++c) g[c][q] = dot(h.a, P.p[c][q]) + h.b;
        if (g[1][q] < 0) anyNeg = true;
        if ((g[0][q] < 0) != (g[1][q] < 0) || (g[2][q] < 0) != (g[1][q] < 0)) apart = true;
    }
    if (!anyNeg) return apart;
    Poly3 out;
    for (int i = 0; i < P.n; ++i) {
        const int j = (i + 1) % P.n;
        const bool in = g[1][i] >= 0, inNext = g[1][j] >= 0;
        if (in && out.n < SPEC_CAP) {
            for (int c = 0; c < 3; ++c) out.p[c][out.n] = P.p[c][i];
            ++out.n;
        }
        if (in != inNext && out.n < SPEC_CAP) {
            for (int c = 0; c < 3; ++c) {
                // (where this colour's own edge meets the line; if it does not cross it, where its line does)
                const double den = g[c][i] - g[c][j];
                const double t = std::fabs(den) > 1e-300 ? g[c][i] / den : g[1][i] / (g[1][i] - g[1][j]);
                out.p[c][out.n] = P.p[c][i] + (P.p[c][j] - P.p[c][i]) * std::max(-4.0, std::min(5.0, t));
            }
            ++out.n;
        }
    }
    P = out;
    return apart;
}
struct SpecBeam {
    Poly3 poly;
    V3 d[3];
    double cosOn[3];
    double irr;      // of the middle colour, on its plane
    int cell, faces, events;
    bool apart;
};
}  // namespace detail

// As split_light, for all colours of the spectrum at once. bands: make_bands(par.bands, ...), their index of
// refraction from `glass`. step: in-between polygons are put this far apart on the floor (world units), at most
// mostCopies of them. mostBands: a walk covers at most so many bands (0: the whole stretch between two changes of the
// films; 1: every band walked on its own, which is the exact light to compare against). films: how many films the
// solid has (a solid without films is one stretch: the whole spectrum in one walk). gain: the coloured light is made
// this many times brighter than it is (1: as it is; more is a deliberate lie, for show). mirroredToo: the white light mirrored off the outside as well (it has no colours to spread).
inline void split_light_spectrum(const SplitShape* shapes, const V3 R[3], double scale, V3 at, V3 sun, const std::vector<Band>& bands, const Glass& glass,
                                 const double filmN[4][2], const SplitBand& band, double floorY, double least, int mostFaces, double step, int mostCopies,
                                 int mostBands, int films, double gain, bool mirroredToo, std::vector<float>& tri, SpecStats* st) {
    using namespace detail;
    sun = normalize(sun);
    const V3 sunL = normalize(V3{dot(sun, R[0]), dot(sun, R[1]), dot(sun, R[2])});
    auto world = [&](V3 p) { return (R[0] * p.x + R[1] * p.y + R[2] * p.z) * scale + at; };
    auto turn = [&](V3 v) { return R[0] * v.x + R[1] * v.y + R[2] * v.z; };
    const V3 mW{band.ux, -(band.ux * sun.x + band.uz * sun.z) / std::min(sun.y, -1e-6), band.uz};
    const V3 mL = V3{dot(mW, R[0]), dot(mW, R[1]), dot(mW, R[2])} * scale;
    const double m0 = dot(mW, at) - band.c;
    const int N = (int)bands.size();
    const double nmMin = bands[0].nm - 0.5 * (bands[1].nm - bands[0].nm), nmWide = (bands[1].nm - bands[0].nm) * N;
    // all the light up to place x of the spectrum (in bands, from the long... from bands[0] on)
    std::vector<V3> upTo((size_t)N + 1, V3{0, 0, 0});
    for (int i = 0; i < N; ++i) upTo[(size_t)i + 1] = upTo[(size_t)i] + bands[(size_t)i].rgb;
    auto light_to = [&](double x) {
        x = std::max(0.0, std::min((double)N, x));
        const int i = std::min(N - 1, (int)x);
        return upTo[(size_t)i] + bands[(size_t)i].rgb * (x - i);
    };
    auto use_of = [&](double n) {
        int use = 0;
        for (int f = 0; f < 4 && f < films; ++f)
            if (n > filmN[f][0] && n < filmN[f][1]) use |= 1 << f;
        return use;
    };

    // a flat piece leaving the glass: the floor's polygon at three colours, then the in-between ones
    auto land = [&](const Poly3& P, const V3 out[3], double irrOnPlane, double cosOut, double x0, double x1, double xMid, bool apart) {
        V3 X[3][SPEC_CAP];   // x, z, distance flown
        for (int c = 0; c < 3; ++c) {
            const V3 dW = turn(out[c]);
            if (!(dW.y < -1e-6)) return;
            for (int i = 0; i < P.n; ++i) {
                const V3 pw = world(P.p[c][i]);
                const double fl = (floorY - pw.y) / dW.y;
                X[c][i] = V3{pw.x + dW.x * fl, pw.z + dW.z * fl, fl};
            }
        }
        auto floor_area = [&](const V3* x) {
            double a = 0;
            for (int i = 1; i + 1 < P.n; ++i) a += (x[i].x - x[0].x) * (x[i + 1].y - x[0].y) - (x[i + 1].x - x[0].x) * (x[i].y - x[0].y);
            return 0.5 * std::fabs(a);
        };
        const V3 dMid = turn(out[1]);
        const double brightMid = gain * irrOnPlane * (-dMid.y) / cosOut, areaMid = floor_area(X[1]);
        if (!(areaMid > 1e-14)) return;
        double far = 0;
        for (int i = 0; i < P.n; ++i) far = std::max(far, std::hypot(X[2][i].x - X[0][i].x, X[2][i].y - X[0][i].y));
        const int copies = std::max(1, std::min(mostCopies, (int)std::ceil(far / step)));
        const double tMid = (xMid - x0) / (x1 - x0);
        V3 Y[SPEC_CAP];
        for (int k = 0; k < copies; ++k) {
            const double ta = (double)k / copies, tb = (double)(k + 1) / copies, t = 0.5 * (ta + tb);
            const V3 rgb = light_to(x0 + (x1 - x0) * tb) - light_to(x0 + (x1 - x0) * ta);
            for (int i = 0; i < P.n; ++i)
                Y[i] = t < tMid ? X[0][i] + (X[1][i] - X[0][i]) * (t / tMid) : X[1][i] + (X[2][i] - X[1][i]) * ((t - tMid) / (1.0 - tMid));
            const double a = floor_area(Y);
            if (!(a > 1e-14)) continue;
            const double bright = brightMid * areaMid / a;   // the same light on this copy's own area
            const float cr = (float)(rgb.x * bright), cg = (float)(rgb.y * bright), cb = (float)(rgb.z * bright);
            const size_t at0 = tri.size();
            tri.resize(at0 + (size_t)(P.n - 2) * 18);
            float* o = tri.data() + at0;
            for (int i = 1; i + 1 < P.n; ++i)
                for (int q : {0, i, i + 1}) {
                    o[0] = (float)Y[q].x; o[1] = (float)Y[q].y; o[2] = cr; o[3] = cg; o[4] = cb; o[5] = (float)Y[q].z;
                    o += 6;
                }
            if (st) ++st->polygons;
        }
        if (st) {
            const V3 all = light_to(x1) - light_to(x0);
            const double l = brightMid * areaMid * (all.x + all.y + all.z) / 3.0;
            ++st->leaves;
            st->light += l;
            st->mostCopies = std::max(st->mostCopies, (long)copies);
            if (apart) { ++st->apart; st->lightApart += l; }
        }
    };

    // the white light mirrored off the outside (as split_light does it; no colours to spread)
    if (mirroredToo) {
        const double n = glass.n(bands[(size_t)N / 2].nm);
        const SplitShape& sh = shapes[0];
        for (const SplitCell& cell : sh.cells)
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
                        lo = std::min(lo, g); hi = std::max(hi, g);
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
                    const V3 dW = turn(sunL + nf * (2.0 * ci));
                    if (!(dW.y < -1e-6)) continue;
                    const double bright = ci * fres(ci, 1.0, n) * (-dW.y) / ci;
                    V3 X[Poly::CAP];
                    for (int i = 0; i < P.n; ++i) {
                        const V3 pw = world(P.p[(size_t)i]);
                        const double fl = (floorY - pw.y) / dW.y;
                        X[i] = V3{pw.x + dW.x * fl, pw.z + dW.z * fl, fl};
                    }
                    for (int i = 1; i + 1 < P.n; ++i)
                        for (int q : {0, i, i + 1})
                            tri.insert(tri.end(), {(float)X[q].x, (float)X[q].y, (float)bright, (float)bright, (float)bright, (float)X[q].z});
                }
            }
    }

    // the stretches of the spectrum in which the films do the same thing
    std::vector<SpecBeam> stack;
    for (int b0 = 0; b0 < N;) {
        const int use = use_of(glass.n(bands[(size_t)b0].nm));
        int b1 = b0 + 1;
        while (b1 < N && use_of(glass.n(bands[(size_t)b1].nm)) == use && (mostBands <= 0 || b1 - b0 < mostBands)) ++b1;
        // ends and middle, as places in the spectrum (in bands) and as indices of refraction
        const double x[3] = {(double)b0, 0.5 * (b0 + b1), (double)b1};
        double n[3];
        for (int c = 0; c < 3; ++c) n[c] = glass.n(nmMin + nmWide * x[c] / N);
        // (an end sits on the very edge of what a film mirrors: it is taken along by the middle, which is well inside)
        const SplitShape& sh = shapes[use];
        if (st) ++st->walks;
        stack.clear();
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
                        lo = std::min(lo, g); hi = std::max(hi, g);
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
                    if (P.n < 3 || P.n > SPEC_CAP || area(P) < 1e-10) continue;
                    SpecBeam b;
                    b.poly.n = P.n;
                    bool ok = true;
                    for (int col = 0; col < 3; ++col) {
                        for (int i = 0; i < P.n; ++i) b.poly.p[col][i] = P.p[(size_t)i];
                        ok = ok && bend(sunL, nf, 1.0 / n[col], b.d[col]);
                        b.cosOn[col] = std::fabs(dot(nf, b.d[col]));
                    }
                    if (!ok) continue;
                    b.irr = ci * (1.0 - fres(ci, 1.0, n[1]));
                    b.cell = (int)c; b.faces = 0; b.events = 0; b.apart = false;
                    stack.push_back(b);
                }
            }
        }
        while (!stack.empty()) {
            const SpecBeam b = stack.back();
            stack.pop_back();
            if (st) ++st->beams;
            const SplitCell& cell = sh.cells[(size_t)b.cell];
            for (size_t f = 0; f < cell.faces.size(); ++f) {
                const Poly& F = cell.faces[f];
                if (F.n < 3) continue;
                const V3 nB = cell.planes[f].n;
                double db[3];
                for (int c = 0; c < 3; ++c) db[c] = dot(nB, b.d[c]);
                if (!(db[1] > 1e-7)) continue;
                bool apart = b.apart;
                if (!(db[0] > 1e-7) || !(db[2] > 1e-7)) {   // an end colour does not go towards this face at all: it keeps to the middle's way
                    if (st && !apart) ++st->faceApart;
                    apart = true;
                }
                Poly3 P = b.poly;
                for (int c = 0; c < 3; ++c) {
                    const int from = db[c] > 1e-7 ? c : 1;
                    for (int i = 0; i < P.n; ++i) P.p[c][i] = b.poly.p[from][i] + b.d[from] * ((cell.planes[f].o - dot(nB, b.poly.p[from][i])) / db[from]);
                }
                {   // cut to the face (beam_split.h: cut_to_face)
                    V3 mid{0, 0, 0};
                    for (int i = 0; i < F.n; ++i) mid = mid + F.p[(size_t)i];
                    mid = mid * (1.0 / F.n);
                    bool cornersApart = false;
                    for (int i = 0; i < F.n && P.n >= 3; ++i) {
                        const V3 a = F.p[(size_t)i], e = F.p[(size_t)((i + 1) % F.n)];
                        V3 in = cross(nB, e - a);
                        if (dot(in, mid - a) < 0) in = in * -1.0;
                        cornersApart = clip3(P, Half{in, -dot(in, a)}) || cornersApart;
                    }
                    if (P.n < 3) continue;   // (the middle colour does not meet this face; an end colour that would is lost here: see leaveApart / faceApart for the cases counted)
                    if (cornersApart) {
                        if (st && !apart) ++st->cornerApart;
                        apart = true;
                    }
                }
                const double a = area3(P, 1);
                const double irr = b.irr * db[1] / b.cosOn[1];
                if (!(a > 1e-10) || irr * a * db[1] < least) continue;
                const int what = cell.what[f];
                V3 d2[3];
                double cos2[3];
                if (what >= 0) {
                    const double Ri = fres(db[1], n[1], 1.0);
                    if (Ri < 1.0) {   // the glass lets the middle colour out: the ends go with it
                        V3 out[3];
                        bool ok = bend(b.d[1], nB * -1.0, n[1], out[1]);
                        if (!ok) continue;
                        for (int c = 0; c < 3; c += 2) {
                            const int from = db[c] > 1e-7 ? c : 1;
                            if (!bend(b.d[from], nB * -1.0, n[c], out[c])) {   // this end would be mirrored back in: drawn where the middle goes
                                out[c] = out[1];
                                for (int i = 0; i < P.n; ++i) P.p[c][i] = P.p[1][i];
                                if (st && !apart) ++st->leaveApart;
                                apart = true;
                            }
                        }
                        land(P, out, irr * (1.0 - Ri), std::fabs(dot(nB, out[1])), x[0], x[2], x[1], apart);
                    } else if (b.faces + 1 < mostFaces && b.events < 30) {
                        for (int c = 0; c < 3; ++c) {
                            const int from = db[c] > 1e-7 ? c : 1;
                            if (c != 1 && fres(db[from], n[c], 1.0) < 1.0) {   // this end would have left here
                                if (st && !apart) ++st->leaveApart;
                                apart = true;
                            }
                            d2[c] = b.d[from] - nB * (2.0 * db[from]);
                            cos2[c] = db[from];
                        }
                        SpecBeam nb{P, {d2[0], d2[1], d2[2]}, {cos2[0], cos2[1], cos2[2]}, irr, b.cell, b.faces + 1, b.events + 1, apart};
                        stack.push_back(nb);
                    }
                } else {
                    const int film = -(what + 1);
                    if (b.events >= 30) continue;
                    const bool mirrors = n[1] > filmN[film][0] && n[1] < filmN[film][1];
                    if (!mirrors && cell.across[film] < 0) continue;
                    for (int c = 0; c < 3; ++c) {
                        const int from = db[c] > 1e-7 ? c : 1;
                        d2[c] = mirrors ? b.d[from] - nB * (2.0 * db[from]) : b.d[from];
                        cos2[c] = db[from];
                    }
                    SpecBeam nb{P, {d2[0], d2[1], d2[2]}, {cos2[0], cos2[1], cos2[2]}, irr, mirrors ? b.cell : cell.across[film], b.faces, b.events + 1, apart};
                    stack.push_back(nb);
                }
            }
        }
        b0 = b1;
    }
}

}  // namespace beam
