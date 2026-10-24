// beam_glass.h, part 7 (cut there by size, as it was): read only through beam_glass.h, in order.
#pragma once

namespace beam {

// The mesh: (row, column, patch) per corner; three indices per triangle.
inline int cone_corner_index(int patch, int i, int j) {
    int base = 0;
    for (int p = 0; p < patch; ++p) base += (CONE_ROWS[p] + 1) * (CONE_COLS[p] + 1);
    return base + i * (CONE_COLS[patch] + 1) + j;
}
// withTop = false: without the triangles of the top (a cone with its tip has none there).
inline void cone_mesh(std::vector<float>& corners, std::vector<uint32_t>& idx, bool withTop = true) {
    corners.clear();
    idx.clear();
    for (int patch = 0; patch < CONE_PATCHES; ++patch) {
        const int rows = CONE_ROWS[patch], cols = CONE_COLS[patch];
        for (int i = 0; i <= rows; ++i)
            for (int j = 0; j <= cols; ++j) corners.insert(corners.end(), {(float)i, (float)j, (float)patch});
        if (patch == 3 && !withTop) continue;
        for (int i = 0; i < rows; ++i)
            for (int j = 0; j < cols; ++j) {
                const uint32_t a = (uint32_t)cone_corner_index(patch, i, j), b = (uint32_t)cone_corner_index(patch, i, j + 1);
                const uint32_t c2 = (uint32_t)cone_corner_index(patch, i + 1, j), d2 = (uint32_t)cone_corner_index(patch, i + 1, j + 1);
                idx.insert(idx.end(), {a, c2, d2, a, d2, b});
            }
    }
}
// The corners around (i, j) of a patch, in order round it: each one with the next (and the last with the first)
// makes one of the triangles of the mesh that meet there. A corner outside the patch has row -1.
inline void cone_around(int patch, int i, int j, int at[6][2]) {
    const int rows = CONE_ROWS[patch], cols = CONE_COLS[patch];
    const int six[6][2] = {{i + 1, j}, {i + 1, j + 1}, {i, j + 1}, {i - 1, j}, {i - 1, j - 1}, {i, j - 1}};
    for (int k = 0; k < 6; ++k) {
        const bool in = six[k][0] >= 0 && six[k][0] <= rows && six[k][1] >= 0 && six[k][1] <= cols;
        at[k][0] = in ? six[k][0] : -1;
        at[k][1] = six[k][1];
    }
}
// How the solid stands to the sun.
struct ConeFrame {
    V3 e1, e2;    // across the axis: e1 points the way the sunlight goes (the far side), e2 = axis x e1
    double half;  // the sunlit part of the side reaches this far round to either side of the line facing the sun (0: none, pi: all)
};
inline ConeFrame cone_frame(const Taper& so, V3 sun) {
    const V3 L = normalize(sun);
    const double La = dot(L, so.ax), k = (so.r - so.rt) / so.h;
    const V3 Lp = L - so.ax * La;
    const double Lr = length(Lp);
    ConeFrame f;
    if (Lr > 1e-9) {
        f.e1 = Lp * (1.0 / Lr);
    } else {
        const V3 up = std::fabs(so.ax.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
        f.e1 = normalize(cross(up, so.ax));
    }
    f.e2 = cross(so.ax, f.e1);
    const double q = Lr > 1e-9 ? -k * La / Lr : (La < 0 ? 2.0 : -2.0);
    f.half = q >= 1.0 ? 3.14159265358979323846 : (q <= -1.0 ? 0.0 : 3.14159265358979323846 - std::acos(q));
    return f;
}
// The side between `lo` and `hi` of the way down from top (0) to base (1) is what the mesh covers.
inline double cone_lo(const Taper& so) { return so.rt > 0 ? 1.0 - CONE_INSET : CONE_TIP; }
// Along the line from top to base at `across` on the side: how far down the tear is. Light (index of refraction n)
// that enters above it and light that enters below it meet different faces first: if it travels towards the
// base, the far side above and the base below; if towards the top, the top above and the far side below.
inline double cone_split(const Taper& so, V3 sun, V3 across, double n) {
    const V3 L = normalize(sun);
    const double k = (so.r - so.rt) / so.h, lo = cone_lo(so), hi = CONE_INSET;
    const V3 N = (across + so.ax * k) * (1.0 / std::sqrt(1.0 + k * k));
    V3 d;
    if (!(-dot(L, N) > 1e-5) || !refract(L, N, 1.0 / n, d)) return hi;
    const double da = dot(d, so.ax);
    if (da < -1e-9) {
        // towards the base: from height y the light meets the base's plane at across * r + y * g; on the rim at y = 0 and once more
        const V3 g = (d - so.ax * da) * (1.0 / -da) - across * k;
        const double gg = dot(g, g);
        if (!(gg > 1e-18)) return hi;
        const double y = -2.0 * so.r * dot(across, g) / gg;
        return std::min(hi, std::max(lo, 1.0 - y / so.h));
    }
    if (da > 1e-9 && so.rt > 0) {
        // towards the top: from u below the top it meets the top's plane at across * rt + u * g; on the rim at u = 0 and once more
        const V3 g = (d - so.ax * da) * (1.0 / da) + across * k;
        const double gg = dot(g, g);
        if (!(gg > 1e-18)) return lo;
        const double u = -2.0 * so.rt * dot(across, g) / gg;
        return std::min(hi, std::max(lo, u / so.h));
    }
    return da > 0 ? lo : hi;
}
// Where corner (i, j) of a patch is on the solid, and the outward normal there. n: the index of refraction of the
// colour step (the tear, and with it the rows of the side, lie a little differently for every colour).
inline void cone_entry(const Taper& so, const ConeFrame& f, V3 sun, double n, int patch, int i, int j, V3& P, V3& N) {
    const double pi = 3.14159265358979323846;
    if (patch >= 2) {
        const double ph = 2.0 * pi * j / CONE_COLS[patch], rad = (patch == 2 ? so.r : so.rt) * CONE_INSET * i / CONE_ROWS[patch];
        P = so.c + (patch == 2 ? V3{0, 0, 0} : so.ax * so.h) + (f.e1 * std::cos(ph) + f.e2 * std::sin(ph)) * rad;
        N = patch == 2 ? so.ax * -1.0 : so.ax;
        return;
    }
    const double ph = pi + (2.0 * j / CONE_COLS[patch] - 1.0) * f.half, k = (so.r - so.rt) / so.h;
    const V3 across = f.e1 * std::cos(ph) + f.e2 * std::sin(ph);
    const double split = cone_split(so, sun, across, n), part = (double)i / CONE_ROWS[patch], lo = cone_lo(so);
    const double down = patch == 0 ? lo + (split - lo) * part : split + (CONE_INSET - split) * part;
    P = so.c + so.ax * (so.h * (1.0 - down)) + across * (so.rt + (so.r - so.rt) * down);
    N = (across + so.ax * k) * (1.0 / std::sqrt(1.0 + k * k));
}
struct ConeHit {
    V3 q;           // where the corner is put on the floor: x, distance flown from the solid, z. Where its light lands; if that is
                    // nowhere near (it leaves upwards, too flat, too far), as far along its way as light is drawn at all, so
                    // that the place moves smoothly whatever the light does. If !ok: under the middle of the base.
    double weight;  // how much of the light takes this way (Fresnel at every face), and of that how much is still drawn
                    // (ball_fade: nothing once it leaves too flat or lands too far)
    double fade;    // 1, falling to 0 towards a tear that the mesh does not follow (the rims and the tip, after a mirror)
    int faces;      // which faces it met inside: corners with the same number belong to the same piece of the sheet
    bool ok;
};
// Corner at P (outward normal N) of patch `patch`, index of refraction n, `way` mirrors inside before it leaves.
// The two patches of the side are told which face they meet first (so that the row on the tear belongs to both
// sides exactly).
inline ConeHit cone_walk(const Taper& so, V3 sun, V3 P, V3 N, int patch, double n, int way, double floorY) {
    auto smooth = [](double x) {
        x = std::min(1.0, std::max(0.0, x));
        return x * x * (3.0 - 2.0 * x);
    };
    const V3 L = normalize(sun);
    const double size = std::max(so.r, so.h), k = (so.r - so.rt) / so.h;
    ConeHit hit;
    hit.q = V3{so.c.x, 0, so.c.z};  // a corner without light: all of them on one spot, so that their triangles cover no pixels
    hit.weight = 0;
    hit.fade = 0;
    hit.faces = 0;
    hit.ok = false;
    const double ci = -dot(L, N);
    if (!(ci > 1e-5)) return hit;
    double weight = 1.0 - fresnel(ci, 1.0, n), fade = 1.0;
    V3 d, D{0, 0, 0}, p = P;
    if (!refract(L, N, 1.0 / n, d)) return hit;
    int faces = 1;
    for (int b = 0; b <= way; ++b) {
        double t = 0;
        V3 nj;
        int face = 0;
        const double da = dot(d, so.ax), y0 = dot(p - so.c, so.ax);
        // the two parts of the side are told what they meet first: 1 the base, 2 the top, 0 the far side
        const int told = b > 0 || patch >= 2 ? -1 : (patch == 0 ? (da > 1e-9 && so.rt > 0 ? 2 : 0) : (da < -1e-9 ? 1 : 0));
        if (told == 1) {
            t = y0 / -da;
            nj = so.ax * -1.0;
            face = 1;
        } else if (told == 2) {
            t = (so.h - y0) / da;
            nj = so.ax;
            face = 2;
        } else {
            double tt[2];
            V3 nn[2];
            int ff[2];
            const int count = taper_crossings(so, p, d, tt, nn, ff);
            bool found = false;
            for (int i = 0; i < count && !found; ++i)
                if ((told < 0 || ff[i] == 0) && tt[i] > 1e-7 * size && dot(nn[i], d) > 0) {
                    t = tt[i];
                    nj = nn[i];
                    face = ff[i];
                    found = true;
                }
            if (!found) {
                if (told < 0) return hit;
                // on the tear itself the far side is met at a rim, where the search may just miss it
                if (da < -1e-9) t = y0 / -da;
                else if (da > 1e-9) t = (so.h - y0) / da;
                else return hit;
                const V3 w = p + d * t - so.c;
                const V3 out = normalize(w - so.ax * dot(w, so.ax));
                nj = (out + so.ax * k) * (1.0 / std::sqrt(1.0 + k * k));
                face = 0;
            }
        }
        p = p + d * t;
        if (b > 0) {
            const V3 w = p - so.c;
            const double y = dot(w, so.ax), rho = length(w - so.ax * y);
            // how far from a rim (or the tip)
            const double margin = face == 1 ? 1.0 - rho / so.r : (face == 2 ? 1.0 - rho / so.rt : std::min(y, so.h - y) / so.h);
            fade *= smooth(margin / CONE_EDGE);
        }
        faces = faces * 3 + face;
        const double co = dot(d, nj), mirrored = fresnel(co, n, 1.0);  // 1 when the glass lets nothing out
        if (b < way) {
            weight *= mirrored;
            d = d - nj * (2.0 * co);
        } else {
            weight *= 1.0 - mirrored;
            // Where the glass lets nothing out, the corner still needs a place (its light is 0, but the sheet must
            // not tear there): it is sent out as steeply as the last light that does get out.
            const double s2 = n * n * (1.0 - co * co), most = 0.98;
            if (s2 < most) {
                if (!refract(d, nj * -1.0, n, D)) return hit;
            } else {
                V3 along = d - nj * co;
                const double al = length(along);
                if (!(al > 1e-12)) return hit;
                D = along * (std::sqrt(most) / al) + nj * std::sqrt(1.0 - most);
            }
        }
    }
    hit.faces = faces;
    hit.ok = true;
    hit.fade = fade;
    const double t = D.y < -1e-6 ? std::max(0.0, (floorY - p.y) / D.y) : 1e30, far = std::min(t, BALL_REACH * size);
    hit.q = V3{p.x + D.x * far, far, p.z + D.z * far};
    hit.weight = weight * (D.y < -BALL_FLAT ? ball_fade(t / size, -D.y) : 0.0);
    return hit;
}
// How bright a corner is, as a multiple of the sun: the triangles that meet at it and belong to the same piece of
// the sheet, the sunbeam they catch over the floor they land on; times how much of the light takes this way; times
// the least `fade` of the corner and its neighbours. P: where the corners are on the solid, hit: what became of
// them; at: cone_around.
inline double cone_bright(V3 L, V3 Pv, const ConeHit& hv, const int at[6][2], const V3* Pk, const ConeHit* hk) {
    if (!hv.ok || !(hv.weight > 0)) return 0.0;
    double before = 0, after = 0, fade = hv.fade;
    for (int k = 0; k < 6; ++k) {
        if (at[k][0] < 0) continue;
        const bool same = hk[k].ok && hk[k].faces == hv.faces;
        fade = std::min(fade, same ? hk[k].fade : 0.0);
        const int m = (k + 1) % 6;
        if (at[m][0] < 0 || !same || !hk[m].ok || hk[m].faces != hv.faces) continue;
        before += std::fabs(dot(cross(Pk[k] - Pv, Pk[m] - Pv), L));
        after += std::fabs((hk[k].q.x - hv.q.x) * (hk[m].q.z - hv.q.z) - (hk[k].q.z - hv.q.z) * (hk[m].q.x - hv.q.x));
    }
    if (!(before > 0)) return 0.0;
    return hv.weight * fade * (after * CONE_CAP > before ? before / after : CONE_CAP);
}
// The light of a cone or a rod as triangles, worked out on the CPU exactly as cone_walk.mat + cone_light.mat draw
// it (for a renderer that does not, and for the desk check).
inline void cone_light(const Solid& solid, const Params& par, std::vector<float>& tri, int* patches = nullptr) {
    struct Mesh {
        std::vector<float> corners;
        std::vector<uint32_t> idx;
    };
    static const Mesh mesh = [] {
        Mesh m;
        cone_mesh(m.corners, m.idx);
        return m;
    }();
    const Taper so = taper_of(solid);
    const V3 L = normalize(par.sun);
    const ConeFrame f = cone_frame(so, par.sun);
    const size_t nv = mesh.corners.size() / 3, nt = mesh.idx.size() / 3;
    const std::vector<BallStep> steps = ball_steps(par, par.coneSteps);
    const size_t base = tri.size();
    tri.resize(base + steps.size() * (size_t)CONE_WAYS * nt * 3 * (size_t)TRI);
    float* o = tri.data() + base;
    std::vector<V3> P(nv), N(nv);
    std::vector<ConeHit> hit(nv);
    std::vector<double> e(nv), fly(nv);
    for (size_t si = 0; si < steps.size(); ++si) {
        const BallStep& st = steps[si];
        const double nNext = steps[si + 1 < steps.size() ? si + 1 : (si > 0 ? si - 1 : si)].n;  // the neighbouring colour
        for (size_t v = 0; v < nv; ++v) cone_entry(so, f, par.sun, st.n, (int)mesh.corners[v * 3 + 2], (int)mesh.corners[v * 3], (int)mesh.corners[v * 3 + 1], P[v], N[v]);
        for (int way = 0; way < CONE_WAYS; ++way) {
            for (size_t v = 0; v < nv; ++v) {
                const int patch = (int)mesh.corners[v * 3 + 2];
                hit[v] = cone_walk(so, par.sun, P[v], N[v], patch, st.n, way, par.floorY);
                // soften by half the gap to where the next colour lands, on top of what the distance flown asks for
                fly[v] = hit[v].q.y;
                if (hit[v].ok && hit[v].weight > 0) {
                    V3 Pn, Nn;
                    cone_entry(so, f, par.sun, nNext, patch, (int)mesh.corners[v * 3], (int)mesh.corners[v * 3 + 1], Pn, Nn);
                    const ConeHit next = cone_walk(so, par.sun, Pn, Nn, patch, nNext, way, par.floorY);
                    if (next.ok && next.weight > 0) fly[v] += 0.5 * std::sqrt((next.q.x - hit[v].q.x) * (next.q.x - hit[v].q.x) + (next.q.z - hit[v].q.z) * (next.q.z - hit[v].q.z)) / par.sunTan;
                }
            }
            for (size_t v = 0; v < nv; ++v) {
                const int patch = (int)mesh.corners[v * 3 + 2];
                int at[6][2];
                cone_around(patch, (int)mesh.corners[v * 3], (int)mesh.corners[v * 3 + 1], at);
                V3 Pk[6];
                ConeHit hk[6];
                for (int k = 0; k < 6; ++k) {
                    if (at[k][0] < 0) continue;
                    const size_t u = (size_t)cone_corner_index(patch, at[k][0], at[k][1]);
                    Pk[k] = P[u];
                    hk[k] = hit[u];
                }
                e[v] = cone_bright(L, P[v], hit[v], at, Pk, hk);
            }
            for (size_t t = 0; t < nt; ++t) {
                const uint32_t v[3] = {mesh.idx[t * 3], mesh.idx[t * 3 + 1], mesh.idx[t * 3 + 2]};
                if (!hit[v[0]].ok || !hit[v[1]].ok || !hit[v[2]].ok || hit[v[0]].faces != hit[v[1]].faces || hit[v[1]].faces != hit[v[2]].faces) continue;
                if (e[v[0]] + e[v[1]] + e[v[2]] <= 0) continue;
                for (int k = 0; k < 3; ++k) {
                    *o++ = (float)hit[v[k]].q.x;
                    *o++ = (float)hit[v[k]].q.z;
                    *o++ = (float)(st.rgb.x * e[v[k]]);
                    *o++ = (float)(st.rgb.y * e[v[k]]);
                    *o++ = (float)(st.rgb.z * e[v[k]]);
                    *o++ = (float)fly[v[k]];
                }
                if (patches) ++*patches;
            }
        }
    }
    tri.resize((size_t)(o - tri.data()));
}

// pieces: 1 = every lit face is one piece; k = each face is cut into a k x k grid of pieces.
// balls: if given, it comes back as the list of all balls and cones, and they get no triangles here: the renderer
// draws them from their shared shapes (see "A ball by likeness", "A cone by formula"). If not given, their light
// is worked out here as triangles, exactly as those shapes would be drawn.
// leave: which kinds those are (one bit per Solid::Kind). Flat-faced solids can be left to the renderer too (see
// "Flat faces on the GPU"); then this function has nothing to do for them.
inline void trace_cheap(const Scene& sc, const Params& par, const std::vector<double>& detailPerSolid, int pieces, std::vector<float>& tri, CheapStats* stats = nullptr,
                        std::vector<int>* balls = nullptr,
                        unsigned leave = (1u << Solid::BALL) | (1u << Solid::CONE) | (1u << Solid::ROD) | (1u << Solid::EXTRUDE)) {
    tri.clear();
    if (balls) balls->clear();
    if (!balls) leave = 0;
    const int N = par.bands;
    const std::vector<Band> bands = make_bands(N, par.nmMin, par.nmMax);
    std::vector<double> nb((size_t)N);
    for (int i = 0; i < N; ++i) nb[(size_t)i] = par.glass.n(bands[(size_t)i].nm);
    const V3 L = normalize(par.sun);
    const size_t ns = sc.solids.size();
    std::vector<std::vector<float>> outTri(ns);
    std::vector<CheapStats> outSt(ns);
    BallShape shape;  // the one shape of every ball's light, if a ball's triangles are wanted here
    if (!((leave >> Solid::BALL) & 1u))
        for (const Solid& so : sc.solids)
            if (so.kind == Solid::BALL) {
                shape = ball_shape(par);
                break;
            }
    parallel_for(ns, par.threads, [&](size_t A) {
        std::vector<float>& tri = outTri[A];
        CheapStats& st = outSt[A];
        const Solid& so = sc.solids[A];
        const double detail_ = detailPerSolid.empty() ? 1e-3 : detailPerSolid[std::min(A, detailPerSolid.size() - 1)];
        if ((leave >> so.kind) & 1u) {
            st.alike = 1;
            return;
        }
        if (so.kind == Solid::BALL) {
            ball_light(shape, so.c, so.r, par.floorY, tri, &st.patches);
            return;
        }
        if (so.kind == Solid::EXTRUDE) {
            extrude_light(so, par, tri, &st, -1);
            return;
        }
        if (so.kind == Solid::CONE || so.kind == Solid::ROD) {
            cone_light(so, par, tri, &st.patches);
            return;
        }
        for (int f = 0; f < (int)so.planes.size(); ++f) {
            const Poly& face = so.faces[(size_t)f];
            if (face.n < 3 || dot(so.planes[(size_t)f].n, L) > -1e-9) continue;
            detail::flat_light(sc, par, (int)A, f, face, so.planes[(size_t)f].n, pieces, detail_, bands, nb, tri, st);
        }
    });
    CheapStats st;
    size_t total = 0;
    for (const auto& t : outTri) total += t.size();
    tri.reserve(total);
    for (size_t i = 0; i < ns; ++i) {
        tri.insert(tri.end(), outTri[i].begin(), outTri[i].end());
        st.pieces += outSt[i].pieces;
        st.patches += outSt[i].patches;
        st.rays += outSt[i].rays;
        st.alike += outSt[i].alike;
        if (balls && outSt[i].alike) balls->push_back((int)i);
    }
    if (stats) *stats = st;
}

}  // namespace beam
