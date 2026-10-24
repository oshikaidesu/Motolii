// beam_glass.h, part 3 (cut there by size, as it was): read only through beam_glass.h, in order.
#pragma once

namespace beam {

namespace detail {

struct Way {
    uint16_t seq[Patch::SEQ];
    uint8_t len;
    bool full;  // more steps than fit: such a branch is dropped
    void add(uint16_t c) {
        if (len < Patch::SEQ) seq[len++] = c;
        else full = true;
    }
    uint64_t hash() const {
        uint64_t h = 1469598103934665603ull;
        for (int i = 0; i < len; ++i) h = (h ^ (uint64_t)(seq[i] + 1)) * 1099511628211ull;
        return h;
    }
};

struct Ctx {
    const Scene* scene;
    const Params* par;
    Result* res;
    double n;       // index of this band
    V3 rgb;
    int band;
    double share;   // 1 / bands
    // When set, only this one way is followed and nothing is counted; its landed polygons go to `out`.
    const Patch* only = nullptr;
    std::vector<Poly>* out = nullptr;
    std::vector<double>* outE = nullptr;
};

// Where the beam lands. Returns false if it goes up.
// Each landed corner is (x, travel, z): its place on the floor and, in y, how far the light flew from the glass
// to get there. A renderer uses the travel to soften the light as a sun with some width would.
inline bool to_floor(const Poly& P, V3 dir, double floorY, Poly& F) {
    if (dir.y > -1e-9) return false;
    F.n = 0;
    for (int i = 0; i < P.n; ++i) {
        V3 v = P.p[(size_t)i];
        double t = (floorY - v.y) / dir.y;
        if (t < 0) t = 0;  // a face resting on the floor
        V3 q = v + dir * t;
        F.push({q.x, t, q.z});
    }
    return true;
}

// The beam (polygon P on some plane, leaving along dir, carrying `irr` per unit area of that plane) reaches the floor.
inline void land(Ctx& c, const Poly& P, V3 dir, double irr, double cosAtPlane, const Way& way) {
    const double flux = irr * area(P);
    Poly F;
    if (!to_floor(P, dir, c.par->floorY, F)) {
        if (!c.only) c.res->fluxSky += flux * c.share;
        return;
    }
    const double e = irr * (-dir.y) / cosAtPlane;  // flux / landed area
    if (c.only) {
        c.out->push_back(F);
        c.outE->push_back(e);
        return;
    }
    Patch p;
    p.first = (int)(c.res->xz.size() / 2);
    p.count = F.n;
    p.rgb = c.rgb * e;
    p.band = c.band;
    std::memcpy(p.seq, way.seq, sizeof(p.seq));
    p.len = way.len;
    p.path = way.hash();
    for (int i = 0; i < F.n; ++i) {
        c.res->xz.push_back(F.p[(size_t)i].x);
        c.res->xz.push_back(F.p[(size_t)i].z);
        c.res->travel.push_back(F.p[(size_t)i].y);
    }
    c.res->patches.push_back(p);
    c.res->fluxFloor += flux * c.share;
}
inline void drop(Ctx& c, double flux) {
    if (!c.only) c.res->fluxDropped += flux * c.share;
}

// Face G of some solid as seen from plane (pn, po) along d: the halves that bound its shadow on that plane.
// Returns the number of halves (0 if the face is degenerate).
inline int shadow_halves(const Poly& G, V3 pn, double po, V3 d, Half* out) {
    const double den = dot(pn, d);
    Poly S;
    for (int i = 0; i < G.n; ++i) {
        const V3 v = G.p[(size_t)i];
        S.push(v - d * ((dot(pn, v) - po) / den));
    }
    V3 nrm{0, 0, 0};
    for (int i = 1; i + 1 < S.n; ++i) nrm = nrm + cross(S.p[(size_t)i] - S.p[0], S.p[(size_t)i + 1] - S.p[0]);
    const double sg = dot(nrm, pn) >= 0 ? 1.0 : -1.0;
    for (int i = 0; i < S.n; ++i) {
        const V3 a = S.p[(size_t)i], b = S.p[(size_t)((i + 1) % S.n)];
        const V3 h = cross(pn, b - a) * sg;
        out[i] = Half{h, -dot(h, a)};
    }
    return S.n;
}

inline void inside(Ctx& c, int S, const Poly& P, int from, V3 d, double irr, double weight, int bounces, int entries, const Way& way);

// A face of another solid that a beam in the air may meet.
struct Cand {
    int S, g;
    Half region[Poly::CAP + 1];  // its shadow on the beam's plane, and "lies ahead"
    int nh;
    Half t;                      // distance along the beam to the face's plane, as a.p + b
};
// Working lists, one set per thread and per depth of the recursion, so that nothing is allocated per beam.
struct Scratch {
    std::vector<Cand> cand;
    std::vector<Poly> a, b, c;
};
inline Scratch& scratch(int depth) {
    static thread_local std::vector<Scratch> pool(Patch::SEQ + 4);
    return pool[(size_t)std::min(depth, Patch::SEQ + 3)];
}

// Light arrives from outside at face g of solid S (polygon R on that face, irr per unit face area): a few percent
// are mirrored and travel on through the air, the rest goes in.
inline void air(Ctx& c, const Poly& P, V3 pn, V3 d, double irr, int from, double weight, int entries, const Way& way);
inline void enter(Ctx& c, int S, int g, const Poly& R, V3 d, double irr, double weight, int entries, const Way& way) {
    const V3 ng = c.scene->solids[(size_t)S].planes[(size_t)g].n;
    const double ci = -dot(ng, d);
    const double Rf = fresnel(ci, 1.0, c.n);
    const uint16_t cIn = step(S, g), cMirror = step(S, g, true);
    const bool wantMirror = !c.only || (way.len < c.only->len && c.only->seq[way.len] == cMirror);
    const bool wantIn = !c.only || (way.len < c.only->len && c.only->seq[way.len] == cIn);
    if (wantMirror) {
        if (c.par->outerReflection && weight * Rf > c.par->cutoff * 0.25) {
            Way w = way;
            w.add(cMirror);
            if (!w.full) air(c, R, ng, d + ng * (2.0 * ci), irr * Rf, S, weight * Rf, entries, w);
            else drop(c, irr * Rf * area(R));
        } else {
            drop(c, irr * Rf * area(R));
        }
    }
    if (wantIn) {
        Way w = way;
        w.add(cIn);
        if (entries < c.par->maxSolids && !w.full) {
            V3 in;
            refract(d, ng, 1.0 / c.n, in);
            inside(c, S, R, g, in, irr * (1.0 - Rf), weight * (1.0 - Rf), 0, entries + 1, w);
        } else {
            drop(c, irr * (1.0 - Rf) * area(R));
        }
    }
}

// A beam in the air: polygon P on a plane with normal pn, travelling along d, carrying `irr` per unit area of
// that plane. `from` is the solid it just left (it cannot meet that one again right away), -1 for none.
// Whatever part meets another solid first goes into (or mirrors off) that solid; the rest lands on the floor.
inline void air(Ctx& c, const Poly& P, V3 pn, V3 d, double irr, int from, double weight, int entries, const Way& way) {
    const Scene& sc = *c.scene;
    if (!c.only) ++c.res->beams;
    const double areaP = area(P);
    if (irr * areaP < c.par->minFlux) {
        drop(c, irr * areaP);
        return;
    }
    const double po = dot(pn, P.p[0]);
    const double cosP = std::fabs(dot(pn, d));
    const double tiny = 1e-13 * areaP + 1e-300;
    Scratch& sx = scratch(way.len);
    // where the way being followed goes next
    const bool atEnd = c.only && way.len >= c.only->len;
    const uint16_t next = (c.only && !atEnd) ? c.only->seq[way.len] : 0;

    std::vector<Cand>& cand = sx.cand;
    cand.clear();
    if (sc.solids.size() > 1) {
        V3 mid{0, 0, 0};
        for (int i = 0; i < P.n; ++i) mid = mid + P.p[(size_t)i];
        mid = mid * (1.0 / P.n);
        double rb = 0;
        for (int i = 0; i < P.n; ++i) rb = std::max(rb, length(P.p[(size_t)i] - mid));
        for (int S = 0; S < (int)sc.solids.size(); ++S) {
            if (S == from) continue;
            const V3 rel = sc.centre[(size_t)S] - mid;
            const double along = dot(rel, d), reach = sc.radius[(size_t)S] + rb;
            if (along < -reach || dot(rel, rel) - along * along > reach * reach) continue;
            const Solid& so = sc.solids[(size_t)S];
            for (int g = 0; g < (int)so.planes.size(); ++g) {
                const Poly& G = so.faces[(size_t)g];
                const double den = dot(so.planes[(size_t)g].n, d);
                if (G.n < 3 || den > -1e-12) continue;  // only faces turned against the beam let it in
                cand.emplace_back();
                Cand& k = cand.back();
                k.S = S;
                k.g = g;
                k.nh = shadow_halves(G, pn, po, d, k.region);
                k.t = Half{so.planes[(size_t)g].n * (-1.0 / den), so.planes[(size_t)g].o / den};
                k.region[k.nh++] = Half{k.t.a, k.t.b + 1e-9};  // the face must lie ahead
                Poly Q = P;
                for (int h = 0; h < k.nh && Q.n >= 3; ++h) clip(Q, k.region[h]);
                if (!(Q.n >= 3 && area(Q) > tiny)) cand.pop_back();
            }
        }
    }
    // the part that meets nothing: on to the floor
    if (!c.only || atEnd) {
        std::vector<Poly>&rest = sx.a, &nextRest = sx.b;
        rest.clear();
        rest.push_back(P);
        for (const Cand& k : cand) {
            nextRest.clear();
            for (Poly& Q : rest) split(Q, k.region, k.nh, nextRest, tiny);
            rest.swap(nextRest);
        }
        for (const Poly& Q : rest) land(c, Q, d, irr, cosP, way);
    }
    if (atEnd) return;
    // the part that meets each face first
    for (size_t i = 0; i < cand.size(); ++i) {
        const Cand& k = cand[i];
        if (c.only && (step_solid(next) != k.S || step_face(next) != k.g)) continue;
        Poly first = P;
        for (int h = 0; h < k.nh && first.n >= 3; ++h) clip(first, k.region[h]);
        if (first.n < 3) continue;
        std::vector<Poly>&mine = sx.a, &tmp = sx.b;
        mine.clear();
        mine.push_back(first);
        for (size_t j = 0; j < cand.size() && !mine.empty(); ++j) {
            const Cand& o = cand[j];
            if (o.S == k.S) continue;  // faces of one solid never hide each other from the beam
            // where the other face is met earlier: inside its shadow and t_other < t_mine
            Half C[Poly::CAP + 2];
            for (int h = 0; h < o.nh; ++h) C[h] = o.region[h];
            C[o.nh] = Half{k.t.a - o.t.a, k.t.b - o.t.b};
            tmp.clear();
            for (Poly& Q : mine) split(Q, C, o.nh + 1, tmp, tiny);
            mine.swap(tmp);
        }
        const Plane& pg = sc.solids[(size_t)k.S].planes[(size_t)k.g];
        const double den = dot(pg.n, d);
        // (the pieces are moved to their own list: `mine` is reused for the next face)
        std::vector<Poly>& go = sx.c;
        go.clear();
        for (const Poly& Q : mine) {
            go.emplace_back();
            Poly& R = go.back();
            for (int q = 0; q < Q.n; ++q) {
                const V3 v = Q.p[(size_t)q];
                R.push(v + d * ((pg.o - dot(pg.n, v)) / den));
            }
        }
        const Cand kk = k;  // `cand` belongs to this depth only, but keep what is needed across the calls anyway
        for (size_t q = 0; q < go.size(); ++q) {
            const Poly R = go[q];
            enter(c, kk.S, kk.g, R, d, irr * (-den) / cosP, weight, entries, way);
        }
    }
}

// A beam inside solid S: polygon P on face `from`, travelling along d, carrying `irr` per unit area of that face.
// `weight` is the share of the sun this branch still carries (Fresnel factors only), used for the cutoff.
inline void inside(Ctx& c, int S, const Poly& P, int from, V3 d, double irr, double weight, int bounces, int entries, const Way& way) {
    const Solid& s = c.scene->solids[(size_t)S];
    if (!c.only) ++c.res->beams;
    const double cosFrom = std::fabs(dot(s.planes[(size_t)from].n, d));
    int cand[64];
    double den[64];
    int nc = 0;
    for (int j = 0; j < (int)s.planes.size() && nc < 64; ++j) {
        if (j == from || s.faces[(size_t)j].n == 0) continue;
        const double dj = dot(s.planes[(size_t)j].n, d);
        if (dj > 1e-12) {
            cand[nc] = j;
            den[nc] = dj;
            ++nc;
        }
    }
    const bool haveNext = c.only && way.len < c.only->len;
    if (c.only && !haveNext) return;  // the way ends inside the glass: not a way that lands
    int live[64], nl = 0;
    if (c.only) {
        // one known face is wanted: cut against every other candidate, no sorting out needed
        for (int a = 0; a < nc; ++a) live[nl++] = a;
    } else {
        // Distance to every candidate at every corner. A plane that some other plane beats at every corner is never
        // reached first anywhere on the polygon, and the cuts against it are implied by the cuts against its winner.
        double T[64][Poly::CAP];
        for (int a = 0; a < nc; ++a) {
            const Plane& pa = s.planes[(size_t)cand[a]];
            for (int q = 0; q < P.n; ++q) T[a][q] = (pa.o - dot(pa.n, P.p[(size_t)q])) / den[a];
        }
        for (int a = 0; a < nc; ++a) {
            bool beaten = false;
            for (int b = 0; b < nc && !beaten; ++b) {
                if (b == a) continue;
                bool all = true, strict = false;
                for (int q = 0; q < P.n; ++q) {
                    if (T[b][q] > T[a][q]) { all = false; break; }
                    if (T[b][q] < T[a][q]) strict = true;
                }
                if (all && (strict || b < a)) beaten = true;
            }
            if (!beaten) live[nl++] = a;
        }
    }
    const double tiny = 1e-13 * area(P) + 1e-300;
    for (int ia = 0; ia < nl; ++ia) {
        const int a = live[ia];
        const int j = cand[a];
        const uint16_t cj = step(S, j);
        if (c.only && c.only->seq[way.len] != cj) continue;
        Poly Q = P;
        const Plane& pj = s.planes[(size_t)j];
        for (int ib = 0; ib < nl && Q.n >= 3; ++ib) {
            if (ib == ia) continue;
            const Plane& pb = s.planes[(size_t)cand[live[ib]]];
            clip(Q, Half{pb.n * (-1.0 / den[live[ib]]) - pj.n * (-1.0 / den[a]), pb.o / den[live[ib]] - pj.o / den[a]});
        }
        if (Q.n < 3 || area(Q) <= tiny) continue;
        Poly R;
        for (int q = 0; q < Q.n; ++q) {
            const V3 v = Q.p[(size_t)q];
            R.push(v + d * ((pj.o - dot(pj.n, v)) / den[a]));
        }
        const double ci = den[a];
        const double irrJ = irr * ci / cosFrom;  // same flux, spread over the footprint on face j
        if (irrJ * area(R) < c.par->minFlux) {
            drop(c, irrJ * area(R));
            continue;
        }
        Way wj = way;
        wj.add(cj);
        const double Rf = fresnel(ci, c.n, 1.0);
        // what the way being followed does after this face
        bool wantOut = true, wantOn = true;
        if (c.only) {
            const bool more = wj.len < c.only->len;
            const bool sameSolid = more && step_solid(c.only->seq[wj.len]) == S && !step_mirrored(c.only->seq[wj.len]);
            wantOn = sameSolid;
            wantOut = !sameSolid;
        }
        if (Rf < 1.0 && wantOut) {
            V3 out;
            refract(d, pj.n * -1.0, c.n, out);
            if (!wj.full) air(c, R, pj.n, out, irrJ * (1.0 - Rf), S, weight * (1.0 - Rf), entries, wj);
            else drop(c, irrJ * (1.0 - Rf) * area(R));
        }
        if (wantOn) {
            const double w = weight * Rf;
            if (bounces < c.par->maxBounces && w > c.par->cutoff && !wj.full) inside(c, S, R, j, d - pj.n * (2.0 * ci), irrJ * Rf, w, bounces + 1, entries, wj);
            else drop(c, irrJ * Rf * area(R));
        }
    }
}

// Sunlight on one face of one solid, minus what other solids standing in the sun's way take first.
inline void sun_on_face(Ctx& c, int A, int f) {
    const Scene& sc = *c.scene;
    const V3 L = normalize(c.par->sun);
    const Solid& sa = sc.solids[(size_t)A];
    const Poly& F = sa.faces[(size_t)f];
    if (F.n < 3) return;
    const V3 nf = sa.planes[(size_t)f].n;
    const double ci = -dot(nf, L);
    if (ci <= 1e-9) return;  // faces away from the sun
    const double tiny = 1e-13 * area(F) + 1e-300;
    std::vector<Poly> lit{F}, tmp;
    if (sc.solids.size() > 1) {
        const double po = sa.planes[(size_t)f].o;
        V3 mid{0, 0, 0};
        for (int i = 0; i < F.n; ++i) mid = mid + F.p[(size_t)i];
        mid = mid * (1.0 / F.n);
        double rb = 0;
        for (int i = 0; i < F.n; ++i) rb = std::max(rb, length(F.p[(size_t)i] - mid));
        for (int S = 0; S < (int)sc.solids.size() && !lit.empty(); ++S) {
            if (S == A) continue;
            const V3 rel = sc.centre[(size_t)S] - mid;
            const double along = dot(rel, L), reach = sc.radius[(size_t)S] + rb;
            if (along > reach || dot(rel, rel) - along * along > reach * reach) continue;  // not upstream of this face
            const Solid& so = sc.solids[(size_t)S];
            for (int g = 0; g < (int)so.planes.size() && !lit.empty(); ++g) {
                const Poly& G = so.faces[(size_t)g];
                const double den = dot(so.planes[(size_t)g].n, L);
                if (G.n < 3 || den > -1e-12) continue;
                Half C[Poly::CAP + 1];
                int nh = shadow_halves(G, nf, po, L, C);
                // the sun met that face before it could reach this one: t < 0 seen from here
                C[nh++] = Half{so.planes[(size_t)g].n * (1.0 / den), -so.planes[(size_t)g].o / den};
                tmp.clear();
                for (Poly& Q : lit) {
                    split(Q, C, nh, tmp, tiny);
                }
                lit.swap(tmp);
            }
        }
    }
    detail::Way way{};
    for (const Poly& Q : lit) {
        if (!c.only) c.res->fluxIn += ci * area(Q) * c.share;
        enter(c, A, f, Q, L, ci, 1.0, 0, way);
    }
}

// Everything one wavelength (index n) does, for the faces [first, last) of the flat list of all faces.
inline void trace_one(const Scene& sc, const Params& par, double n, V3 rgb, int band, double share, Result& res, int onlySolid = -1, int onlyFace = -1) {
    Ctx c{&sc, &par, &res, n, rgb, band, share};
    for (int A = 0; A < (int)sc.solids.size(); ++A) {
        if (onlySolid >= 0 && A != onlySolid) continue;
        for (int f = 0; f < (int)sc.solids[(size_t)A].planes.size(); ++f) {
            if (onlyFace >= 0 && f != onlyFace) continue;
            sun_on_face(c, A, f);
        }
    }
}

// One known way through the glass, at index n: the exact landed polygons (there can be several pieces when
// other solids cut the beam) and their brightness.
inline void follow(const Scene& sc, const Params& par, double n, const Patch& way, std::vector<Poly>& F, std::vector<double>& e) {
    F.clear();
    e.clear();
    if (way.len == 0) return;
    Result none;
    Ctx c{&sc, &par, &none, n, V3{1, 1, 1}, 0, 1.0};
    c.only = &way;
    c.out = &F;
    c.outE = &e;
    sun_on_face(c, step_solid(way.seq[0]), step_face(way.seq[0]));
}

}  // namespace detail

// Reference: every band traced in full. Exact for each band, slow (bands x everything).
inline Result trace(const Scene& sc, const Params& par) {
    const std::vector<Band> bands = make_bands(par.bands, par.nmMin, par.nmMax);
    std::vector<Result> part((size_t)par.bands);
    parallel_for((size_t)par.bands, par.threads, [&](size_t b) {
        detail::trace_one(sc, par, par.glass.n(bands[b].nm), bands[b].rgb, (int)b, 1.0 / par.bands, part[b]);
    });
    Result res;
    for (Result& r : part) {
        const int base = (int)(res.xz.size() / 2);
        res.xz.insert(res.xz.end(), r.xz.begin(), r.xz.end());
        res.travel.insert(res.travel.end(), r.travel.begin(), r.travel.end());
        for (Patch p : r.patches) {
            p.first += base;
            res.patches.push_back(p);
        }
        res.fluxIn += r.fluxIn;
        res.fluxFloor += r.fluxFloor;
        res.fluxSky += r.fluxSky;
        res.fluxDropped += r.fluxDropped;
        res.beams += r.beams;
    }
    return res;
}
inline Result trace(const Solid& solid, const Params& par) { return trace(one(solid), par); }

constexpr int TRI = 6;  // floats per triangle corner: x, z, r, g, b, travel

struct FastStats {
    int ways = 0, coloured = 0, smooth = 0, patches = 0, follows = 0;
};

}  // namespace beam
