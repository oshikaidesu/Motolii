// beam_glass.h, part 6 (cut there by size, as it was): read only through beam_glass.h, in order.
#pragma once

namespace beam {

namespace detail {
// The light of one flat face (of a flat solid, or the flat end of a rod): the face is cut into pieces x pieces
// parts along its first edge, and each part goes wherever one ray from its middle goes.
// A face cut into pieces x pieces parts along its first edge (thin faces, the chamfers, along their length only).
inline void face_pieces(const Poly& face, V3 nf, int pieces, std::vector<Poly>& list) {
    list.clear();
    if (face.n < 3) return;
    if (pieces <= 1) {
        list.push_back(face);
        return;
    }
    const V3 u = normalize(face.p[1] - face.p[0]), v = cross(nf, u);
    double u0 = 1e300, u1 = -1e300, v0 = 1e300, v1 = -1e300;
    for (int i = 0; i < face.n; ++i) {
        u0 = std::min(u0, dot(face.p[(size_t)i], u)); u1 = std::max(u1, dot(face.p[(size_t)i], u));
        v0 = std::min(v0, dot(face.p[(size_t)i], v)); v1 = std::max(v1, dot(face.p[(size_t)i], v));
    }
    // thin faces (chamfers) are cut along their length only
    const int kv = (v1 - v0) * 6.0 < (u1 - u0) ? 1 : pieces, ku = (u1 - u0) * 6.0 < (v1 - v0) ? 1 : pieces;
    const double tiny = 1e-12 * area(face);
    for (int a = 0; a < ku; ++a)
        for (int b = 0; b < kv; ++b) {
            Poly t = face;
            clip(t, Half{u, -(u0 + (u1 - u0) * a / ku)});
            clip(t, Half{u * -1.0, u0 + (u1 - u0) * (a + 1) / ku});
            clip(t, Half{v, -(v0 + (v1 - v0) * b / kv)});
            clip(t, Half{v * -1.0, v0 + (v1 - v0) * (b + 1) / kv});
            if (t.n >= 3 && area(t) > tiny) list.push_back(t);
        }
}
inline void flat_light(const Scene& sc, const Params& par, int A, int f, const Poly& face, V3 nf, int pieces, double detail_, const std::vector<Band>& bands,
                       const std::vector<double>& nb, std::vector<float>& tri, CheapStats& st) {
    const int N = par.bands;
    const double nBlue = nb[0], nRed = nb[(size_t)N - 1];
    const V3 L = normalize(par.sun);
    const size_t ns = sc.solids.size();
    auto emit = [&](const Poly& F, V3 rgb) {
        for (int i = 1; i + 1 < F.n; ++i) {
            const int idx[3] = {0, i, i + 1};
            for (int k = 0; k < 3; ++k) {
                tri.push_back((float)F.p[(size_t)idx[k]].x);
                tri.push_back((float)F.p[(size_t)idx[k]].z);
                tri.push_back((float)rgb.x);
                tri.push_back((float)rgb.y);
                tri.push_back((float)rgb.z);
                tri.push_back((float)F.p[(size_t)idx[k]].y);
            }
        }
        ++st.patches;
    };
    static thread_local std::vector<Poly> list;
    face_pieces(face, nf, pieces, list);
    for (const Poly& P : list) {
        V3 mid{0, 0, 0};
        for (int i = 0; i < P.n; ++i) mid = mid + P.p[(size_t)i];
        mid = mid * (1.0 / P.n);
        // does another solid take the sun before it gets here?
        if (ns > 1) {
            int S, g;
            double t;
            V3 nn;
            if (first_solid(sc, mid - L * 1e3, L, -1, S, g, t, nn) && (S != A || std::fabs(t - 1e3) > 1e-6 * 1e3 + 1e-7)) continue;
        }
        ++st.pieces;
        Route rr, rb, r;
        Poly F;
        // mirrored off the face: one white patch
        if (par.outerReflection) {
            ++st.rays;
            if (route(sc, par, 0.5 * (nRed + nBlue), A, f, mid, nf, true, r)) {
                send(r, P, par.floorY, F);
                emit(F, V3{1, 1, 1} * r.gain);
            }
        }
        // through the glass: how far apart do red and blue land?
        st.rays += 2;
        const bool okR = route(sc, par, nRed, A, f, mid, nf, false, rr), okB = route(sc, par, nBlue, A, f, mid, nf, false, rb);
        if (!okR && !okB) continue;
        int m = N / 4;  // red and blue go different ways: a fair number of steps
        if (okR && okB && rr.key == rb.key) {
            Poly Fr, Fb;
            send(rr, P, par.floorY, Fr);
            send(rb, P, par.floorY, Fb);
            double move = 0;
            for (int i = 0; i < Fr.n; ++i) {
                const V3 dv = Fr.p[(size_t)i] - Fb.p[(size_t)i];
                move = std::max(move, std::sqrt(dv.x * dv.x + dv.z * dv.z));
            }
            m = move <= detail_ ? 1 : (int)std::min<double>(N, std::ceil(move / detail_));
        }
        for (int j = 0; j < m; ++j) {
            const int lo = j * N / m, hi = (j + 1) * N / m;
            if (hi <= lo) continue;
            V3 colour{0, 0, 0};
            double n = 0;
            for (int i = lo; i < hi; ++i) {
                colour = colour + bands[(size_t)i].rgb;
                n += nb[(size_t)i];
            }
            n /= (hi - lo);
            ++st.rays;
            if (!route(sc, par, n, A, f, mid, nf, false, r)) continue;
            send(r, P, par.floorY, F);
            emit(F, colour * r.gain);
        }
    }
}
}  // namespace detail

// The sunlit side of a rod as a mesh of points with normals (world space), for its light and its outline.
// (A ball has its own, simpler way: see "A ball by likeness".)
struct LitMesh {
    std::vector<V3> p, n;
    std::vector<int> face;
    std::vector<int> tri;  // three indices each
};
inline void lit_mesh(const Solid& so, V3 L, LitMesh& m, bool roundPartOnly = false) {
    m.p.clear();
    m.n.clear();
    m.face.clear();
    m.tri.clear();
    const double pi = 3.14159265358979323846, edge = 0.497 * pi;  // stop just short of the grazing rim
    auto frame = [](V3 w, V3& e1, V3& e2) {
        const V3 up = std::fabs(w.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
        e1 = normalize(cross(up, w));
        e2 = cross(w, e1);
    };
    auto disc = [&](V3 mid, V3 nrm, V3 e1, V3 e2, double rad, int rings, int sectors, int faceId) {
        // rings around `mid`
        const int base = (int)m.p.size();
        m.p.push_back(mid);
        m.n.push_back(nrm);
        m.face.push_back(faceId);
        for (int i = 1; i <= rings; ++i)
            for (int j = 0; j < sectors; ++j) {
                const double ph = 2.0 * pi * j / sectors, u = (double)i / rings;
                const V3 dir = e1 * std::cos(ph) + e2 * std::sin(ph);
                m.p.push_back(mid + dir * (rad * u));
                m.n.push_back(nrm);
                m.face.push_back(faceId);
            }
        for (int j = 0; j < sectors; ++j) {
            m.tri.insert(m.tri.end(), {base, base + 1 + j, base + 1 + (j + 1) % sectors});
            for (int i = 1; i < rings; ++i) {
                const int a = base + 1 + (i - 1) * sectors + j, b = base + 1 + (i - 1) * sectors + (j + 1) % sectors;
                const int c2 = a + sectors, d2 = b + sectors;
                m.tri.insert(m.tri.end(), {a, c2, d2, a, d2, b});
            }
        }
    };
    if (so.kind == Solid::ROD) {
        // the side: the half that faces the sun
        const V3 w = L * -1.0;
        const V3 wp = w - so.ax * dot(w, so.ax);
        if (length(wp) > 1e-6) {
            const V3 u = normalize(wp), v = cross(so.ax, u);
            const int na = 8, ns = 28, base = (int)m.p.size();
            for (int i = 0; i <= na; ++i)
                for (int j = 0; j <= ns; ++j) {
                    const double ps = -edge + 2.0 * edge * j / ns, sAx = -so.h + 2.0 * so.h * i / na;
                    const V3 nn = u * std::cos(ps) + v * std::sin(ps);
                    m.p.push_back(so.c + so.ax * sAx + nn * so.r);
                    m.n.push_back(nn);
                    m.face.push_back(0);
                }
            for (int i = 0; i < na; ++i)
                for (int j = 0; j < ns; ++j) {
                    const int a = base + i * (ns + 1) + j, b = a + 1, c2 = a + ns + 1, d2 = c2 + 1;
                    m.tri.insert(m.tri.end(), {a, c2, d2, a, d2, b});
                }
        }
        // the end that faces the sun
        for (int sgn = -1; sgn <= 1 && !roundPartOnly; sgn += 2) {
            const V3 nn = so.ax * (double)sgn;
            if (dot(nn, L) > -1e-6) continue;
            V3 e1, e2;
            frame(nn, e1, e2);
            disc(so.c + so.ax * (so.h * sgn), nn, e1, e2, so.r, 8, 32, sgn > 0 ? 1 : 2);
        }
    }
}

// ---- A ball by likeness ------------------------------------------------------------------------------------------
// A ball is the same from every side, and every ball is the same ball at another size. So a ball's light is ONE
// shape, copied: the sunlit side as rings around the middle of the sunbeam (ball_disc), every corner sent through
// a ball of radius 1 by a closed formula that depends on its ring alone (ball_row), turned about the sun's axis,
// scaled by the radius and dropped on the floor (ball_land). Nothing is followed per ball. A renderer keeps the
// mesh once and draws it for every ball and colour step (ball_steps), doing ball_corner per corner on the GPU
// (ball_light.mat); all the CPU hands over per ball is its middle and its radius.
// What is left out on purpose: other solids. A ball's light is the same copy whatever stands in its sun or in
// its light; where lights and shadows overlap on the floor they simply add up. (Following every corner against
// the solids near it was tried: 30 balls looked the same to 0.1 / 255 and it brought back thresholds that can
// make a moving picture jump.) Also left out: the faint light mirrored off the outside of the ball (0.02 / 255).
// Everything here is smooth in the ball's place, its size and the sun, so nothing jumps from frame to frame.
constexpr int BALL_RINGS = 14, BALL_SECTORS = 56;
constexpr double BALL_BEND = 0.497 * 3.14159265358979323846;  // the outermost ring: this far round from the middle of the sunlit side (just short of grazing)
constexpr double BALL_CAP = 40.0;   // a point of light may be this many times the sun, no more
constexpr double BALL_FLAT = 0.02;  // light that leaves upwards, or flatter than this, is not drawn (it lands nowhere near, or never)
constexpr double BALL_REACH = 8.0;  // light that has flown further than this many radii is not drawn: past the ball's focus it thins out with
                                    // the square of the distance, and there it is under half a percent of the sun spread over a huge patch of floor
// (towards both limits the light is faded to nothing first, see ball_fade, so that nothing is cut off visibly)

// The mesh: (ring, sector) per corner, three indices per triangle. Corner 0 is the middle (ring 0).
inline void ball_disc(std::vector<float>& ringSector, std::vector<uint32_t>& idx) {
    ringSector.assign({0.f, 0.f});
    idx.clear();
    for (int i = 1; i <= BALL_RINGS; ++i)
        for (int j = 0; j < BALL_SECTORS; ++j) {
            ringSector.push_back((float)i);
            ringSector.push_back((float)j);
        }
    for (int j = 0; j < BALL_SECTORS; ++j) {
        const uint32_t n1 = (uint32_t)((j + 1) % BALL_SECTORS);
        idx.insert(idx.end(), {0u, 1u + (uint32_t)j, 1u + n1});
        for (int i = 1; i < BALL_RINGS; ++i) {
            const uint32_t a = 1u + (uint32_t)((i - 1) * BALL_SECTORS + j), b = 1u + (uint32_t)(i - 1) * BALL_SECTORS + n1;
            const uint32_t c2 = a + BALL_SECTORS, d2 = b + BALL_SECTORS;
            idx.insert(idx.end(), {a, c2, d2, a, d2, b});
        }
    }
}
inline int ball_corner_index(int ring, int sector) { return ring == 0 ? 0 : 1 + (ring - 1) * BALL_SECTORS + sector; }
// The corners around (ring, sector), in order round it: each one with the next (and the last with the first) makes
// one of the triangles of the mesh that meet at this corner. Around the middle, 8 of its 56. Returns how many;
// a ring beyond the last means "no such corner".
inline int ball_around(int ring, int sector, int at[8][2]) {
    const int S = BALL_SECTORS;
    if (ring == 0) {
        for (int k = 0; k < 8; ++k) {
            at[k][0] = 1;
            at[k][1] = k * S / 8;
        }
        return 8;
    }
    const int next = (sector + 1) % S, prev = (sector + S - 1) % S;
    const int six[6][2] = {{ring + 1, sector}, {ring + 1, next}, {ring, next}, {ring - 1, sector}, {ring - 1, prev}, {ring, prev}};
    for (int k = 0; k < 6; ++k) {
        at[k][0] = six[k][0];
        at[k][1] = six[k][1];
    }
    return 6;
}
// The colour steps a ball's light is drawn in.
struct BallStep {
    V3 rgb;
    double n;
};
inline std::vector<BallStep> ball_steps(const Params& par, int count = 0) {
    const int N = par.bands, steps = std::max(1, std::min(N, count > 0 ? count : par.roundSteps));
    const std::vector<Band> bands = make_bands(N, par.nmMin, par.nmMax);
    std::vector<BallStep> out;
    for (int j = 0; j < steps; ++j) {
        const int lo = j * N / steps, hi = (j + 1) * N / steps;
        BallStep s{V3{0, 0, 0}, 0};
        for (int i = lo; i < hi; ++i) {
            s.rgb = s.rgb + bands[(size_t)i].rgb;
            s.n += par.glass.n(bands[(size_t)i].nm);
        }
        s.n /= std::max(1, hi - lo);
        out.push_back(s);
    }
    return out;
}
// One ray through a ball of radius 1, in the plane of the sun's axis (first number) and the ray (second): where
// it leaves the ball, which way it goes, and how much of the light is left. It depends on the ring alone.
struct BallRow {
    double a, b, da, db, weight;
    bool ok;
};
inline BallRow ball_row(int ring, double n) {
    const double th = (double)ring / BALL_RINGS * BALL_BEND, cs = std::cos(th), sn = std::sin(th);
    const V3 p0{cs, sn, 0}, d{-1, 0, 0};
    const double R0 = fresnel(cs, 1.0, n);
    V3 in, out;
    if (!refract(d, p0, 1.0 / n, in)) return BallRow{0, 0, 0, 0, 0, false};
    const V3 p1 = p0 + in * (-2.0 * dot(p0, in));              // straight across to the far side
    if (!refract(in, p1 * -1.0, n, out)) return BallRow{0, 0, 0, 0, 0, false};
    return BallRow{p1.x, p1.y, out.x, out.y, (1.0 - R0) * (1.0 - R0), true};  // the way out mirrors as much as the way in
}
// Everything about the shape that is the same for every ball of a frame.
struct BallShape {
    V3 w, e1, e2;               // towards the sun, and two directions across the sunbeam
    V3 across[BALL_SECTORS];    // the direction of each sector
    double out[BALL_RINGS + 1]; // how far from the middle of the sunbeam each ring is (ball of radius 1)
    double cs[BALL_SECTORS], sn[BALL_SECTORS];
    std::vector<BallStep> steps;
    struct Rows {
        BallRow ring[BALL_RINGS + 1];
    };
    std::vector<Rows> rows;     // per step
};
inline void sun_frame(V3 sun, V3& w, V3& e1, V3& e2) {
    w = normalize(sun) * -1.0;
    const V3 up = std::fabs(w.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
    e1 = normalize(cross(up, w));
    e2 = cross(w, e1);
}
inline BallShape ball_shape(const Params& par) {
    BallShape sh;
    sun_frame(par.sun, sh.w, sh.e1, sh.e2);
    for (int j = 0; j < BALL_SECTORS; ++j) {
        const double ph = 2.0 * 3.14159265358979323846 * j / BALL_SECTORS;
        sh.cs[j] = std::cos(ph);
        sh.sn[j] = std::sin(ph);
        sh.across[j] = sh.e1 * sh.cs[j] + sh.e2 * sh.sn[j];
    }
    for (int i = 0; i <= BALL_RINGS; ++i) sh.out[i] = std::sin((double)i / BALL_RINGS * BALL_BEND);
    sh.steps = ball_steps(par);
    sh.rows.resize(sh.steps.size());
    for (size_t s = 0; s < sh.steps.size(); ++s)
        for (int i = 0; i <= BALL_RINGS; ++i) sh.rows[s].ring[i] = ball_row(i, sh.steps[s].n);
    return sh;
}
// The rows as a table for a renderer (RGBA float, BALL_RINGS + 1 wide, two lines per step): line 2s holds
// (a, b, da, db) of step s, line 2s + 1 holds (weight, 1 if the ray exists, how far out the ring is, 0).
// It depends on the glass alone, not on the sun and not on any ball.
inline void ball_table(const BallShape& sh, std::vector<float>& rgba, int& width, int& height) {
    width = BALL_RINGS + 1;
    height = 2 * (int)sh.steps.size();
    rgba.assign((size_t)width * height * 4, 0.f);
    for (size_t s = 0; s < sh.steps.size(); ++s)
        for (int i = 0; i <= BALL_RINGS; ++i) {
            const BallRow& r = sh.rows[s].ring[i];
            float* o = rgba.data() + ((2 * s) * (size_t)width + (size_t)i) * 4;
            o[0] = (float)r.a; o[1] = (float)r.b; o[2] = (float)r.da; o[3] = (float)r.db;
            o += (size_t)width * 4;
            o[0] = (float)r.weight; o[1] = r.ok ? 1.f : 0.f; o[2] = (float)sh.out[i];
        }
}
// How much of a corner's light is still drawn: 1, falling smoothly to 0 where it would be left out (flown too far
// for its radius, or leaving too flat), so that light never appears or disappears at full strength.
inline double ball_fade(double flownOverRadius, double down) {
    auto smooth = [](double x) {
        x = std::min(1.0, std::max(0.0, x));
        return x * x * (3.0 - 2.0 * x);
    };
    return smooth((BALL_REACH - flownOverRadius) / (0.5 * BALL_REACH)) * smooth((down - BALL_FLAT) / (4.0 * BALL_FLAT));
}
// Corner (ring, sector) of a ball (middle c, radius r) at one step: where its light lands (x, distance flown from
// the ball, z) and how much of it is still drawn (ball_fade). False if it is left out.
inline bool ball_land(const BallShape& sh, size_t step, V3 c, double r, int ring, int sector, double floorY, V3& q, double& fade) {
    fade = 0;
    const BallRow& row = sh.rows[step].ring[ring];
    if (!row.ok) return false;
    const V3& ac = sh.across[sector];
    const V3 D = sh.w * row.da + ac * row.db;
    if (!(D.y < -BALL_FLAT)) return false;
    const V3 E = c + (sh.w * row.a + ac * row.b) * r;
    const double t = std::max(0.0, (floorY - E.y) / D.y);
    if (t > BALL_REACH * r) return false;
    q = V3{E.x + D.x * t, t, E.z + D.z * t};
    fade = ball_fade(t / r, -D.y);
    return true;
}
// How bright the corner that landed at q is, as a multiple of what is left of the sun: the triangles that meet at
// it (ball_around; p, lands and fades are those corners), the sunbeam they catch over the floor they land on.
// Measured on the very triangles that are drawn, so the light adds up to what went in. Times the least `fade`
// of the corner and its neighbours: a triangle that is about to be left out is already dark at all three corners.
inline double ball_squeeze(const BallShape& sh, double r, int ring, int sector, V3 q, double fade, const int at[8][2], int count, const V3* p, const bool* lands,
                           const double* fades) {
    const double sx = sh.out[ring] * sh.cs[sector], sy = sh.out[ring] * sh.sn[sector];
    double before = 0, after = 0;
    for (int k = 0; k < count; ++k) {
        if (at[k][0] <= BALL_RINGS) fade = std::min(fade, lands[k] ? fades[k] : 0.0);
        const int m = (k + 1) % count;
        if (!lands[k] || !lands[m]) continue;
        const double ax = sh.out[at[k][0]] * sh.cs[at[k][1]] - sx, ay = sh.out[at[k][0]] * sh.sn[at[k][1]] - sy;
        const double bx = sh.out[at[m][0]] * sh.cs[at[m][1]] - sx, by = sh.out[at[m][0]] * sh.sn[at[m][1]] - sy;
        before += std::fabs(ax * by - ay * bx);
        after += std::fabs((p[k].x - q.x) * (p[m].z - q.z) - (p[k].z - q.z) * (p[m].x - q.x));
    }
    before *= r * r;
    if (!(before > 0)) return 0.0;
    return fade * (after * BALL_CAP > before ? before / after : BALL_CAP);
}
// One corner in full: where it lands and how bright it is there, as a multiple of the sun. This is what
// ball_light.mat does per corner.
inline bool ball_corner(const BallShape& sh, size_t step, V3 c, double r, int ring, int sector, double floorY, V3& q, double& light) {
    double fade;
    if (!ball_land(sh, step, c, r, ring, sector, floorY, q, fade)) return false;
    int at[8][2];
    const int count = ball_around(ring, sector, at);
    V3 p[8];
    bool lands[8];
    double fades[8];
    for (int k = 0; k < count; ++k) lands[k] = at[k][0] <= BALL_RINGS && ball_land(sh, step, c, r, at[k][0], at[k][1], floorY, p[k], fades[k]);
    light = sh.rows[step].ring[ring].weight * ball_squeeze(sh, r, ring, sector, q, fade, at, count, p, lands, fades);
    return true;
}
// A ball's light as triangles, worked out on the CPU exactly as a renderer draws the shared shape (for a renderer
// that does not, and for the desk check).
inline void ball_light(const BallShape& sh, V3 c, double r, double floorY, std::vector<float>& tri, int* patches = nullptr) {
    static const std::pair<std::vector<float>, std::vector<uint32_t>> disc = [] {
        std::pair<std::vector<float>, std::vector<uint32_t>> d;
        ball_disc(d.first, d.second);
        return d;
    }();
    const size_t nv = disc.first.size() / 2, nt = disc.second.size() / 3;
    const size_t base = tri.size();
    tri.resize(base + sh.steps.size() * nt * 3 * (size_t)TRI);
    float* o = tri.data() + base;
    std::vector<V3> q(nv);
    std::vector<double> e(nv);
    std::vector<char> ok(nv);
    for (size_t s = 0; s < sh.steps.size(); ++s) {
        for (size_t v = 0; v < nv; ++v) ok[v] = ball_corner(sh, s, c, r, (int)disc.first[v * 2], (int)disc.first[v * 2 + 1], floorY, q[v], e[v]);
        for (size_t t = 0; t < nt; ++t) {
            const uint32_t v[3] = {disc.second[t * 3], disc.second[t * 3 + 1], disc.second[t * 3 + 2]};
            if (!ok[v[0]] || !ok[v[1]] || !ok[v[2]]) continue;
            for (int k = 0; k < 3; ++k) {
                *o++ = (float)q[v[k]].x;
                *o++ = (float)q[v[k]].z;
                *o++ = (float)(sh.steps[s].rgb.x * e[v[k]]);
                *o++ = (float)(sh.steps[s].rgb.y * e[v[k]]);
                *o++ = (float)(sh.steps[s].rgb.z * e[v[k]]);
                *o++ = (float)q[v[k]].y;
            }
            if (patches) ++*patches;
        }
    }
    tri.resize((size_t)(o - tri.data()));
}

// ---- A cone, and a rod, by formula --------------------------------------------------------------------------------
// The light of a cone (and of a rod, which for this is a cone with its tip cut off so far down that both ends are
// as wide: `Taper`), like a ball's, is one mesh drawn many times, with nothing worked out per solid on the CPU.
// Each corner of the mesh is a point of the sunlit surface, sent through the solid by closed formulas (cone_walk)
// and dropped on the floor. What is different from a ball:
// - Light in a cone goes several ways. They are drawn as separate sheets, one instance each (solid, colour step,
//   way): way 0 leaves at the first face it meets inside, way 1 is mirrored there and leaves at the second.
//   Every sheet is the whole mesh; how much light takes a way is what the glass itself says (Fresnel), so a sheet
//   goes dark by itself, smoothly, where the glass lets nothing out or mirrors nothing, and its coloured edge stays.
// - A sheet tears where its light crosses a rim inside: on one side it meets the far side first, on the other an
//   end. Where that happens is found exactly (cone_split; light entering along one line from top to base all
//   bends the same way, which makes it one quadratic), and the mesh is laid so that a row lies on it: the side
//   is two patches, above and below the tear, which slide as the solid or the sun turns. The same for the two
//   lines where the sun grazes the side (cone_frame): the mesh ends on them. So the ends of the sheets are
//   where they belong at any moment and move smoothly.
// - Tears further in (after a mirror) are not found exactly: there the corners carry a `fade` that falls to
//   nothing towards the rims, and a corner is only as bright as the dimmest of itself and its neighbours, so
//   those ends fade out instead of jumping.
// - Light that leaves upwards, too flat or too far is not cut off either: its share (`weight`) falls to nothing
//   first, and every corner keeps a place on the floor that moves smoothly whatever its light does (cone_walk),
//   so no triangle ever has to be dropped.
// Other solids are left out, as for the ball.
using detail::Taper;
using detail::taper_crossings;
using detail::taper_of;
constexpr int CONE_PATCHES = 4;                    // side above the tear, side below it, base, top
constexpr int CONE_ROWS[CONE_PATCHES] = {5, 5, 3, 3};
constexpr int CONE_COLS[CONE_PATCHES] = {32, 32, 24, 24};  // columns: across the sunlit part of the side; round an end
constexpr int CONE_WAYS = 2;                // sheets per colour step: out at the first face met inside, or at the second
                                            // (a third, after two mirrors, changed pictures of six cones by 0.01 / 255)
constexpr double CONE_TIP = 0.03;           // the mesh of a cone's side starts this far (of the way down) from the tip
constexpr double CONE_EDGE = 0.06;          // after a mirror: fade within this far (of the size) of a rim or the tip
constexpr double CONE_INSET = 0.999;        // the mesh stops this short of a rim (light entering on the rim itself has no way through)
constexpr double CONE_CAP = 40.0;

}  // namespace beam
