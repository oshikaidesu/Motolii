// beam_glass.h, part 4 (cut there by size, as it was): read only through beam_glass.h, in order.
#pragma once

namespace beam {

// The same light, for a renderer: triangles x, z, r, g, b, travel (TRI floats per corner).
// Three full traces (both ends and the middle of the spectrum) find every way the light takes and where its
// patch lands at those three indices. A patch that moves less than its `detail` (world units, about one pixel
// on the floor; one value per solid, taken from the solid the light left last) between red and blue is drawn
// once in white. One that moves further is drawn as that many steps of the spectrum. Between two indices where
// it is known exactly, a step is read off the straight line between them, but only after that line has been
// checked against an exact follow in the middle (detail::follow); where the check fails, or the patch appears,
// vanishes or changes shape, the interval is halved and followed exactly again, down to single bands.
inline void trace_fast(const Scene& sc, const Params& par, const std::vector<double>& detailPerSolid, std::vector<float>& tri, FastStats* stats = nullptr) {
    tri.clear();
    const int N = par.bands;
    const std::vector<Band> bands = make_bands(N, par.nmMin, par.nmMax);
    std::vector<double> nb((size_t)N);
    for (int i = 0; i < N; ++i) nb[(size_t)i] = par.glass.n(bands[(size_t)i].nm);
    const double nBlue = nb[0], nRed = nb[(size_t)N - 1];
    constexpr int G = 64;  // the index range is cut into G parts; knot 0 is red, knot G is blue
    auto knot_n = [&](int k) { return nRed + (nBlue - nRed) * (double)k / G; };
    const int traced[3] = {0, G / 2, G};

    // the three full traces, one task per (index, solid)
    const size_t ns = sc.solids.size();
    std::vector<Result> part(3 * ns);
    parallel_for(3 * ns, par.threads, [&](size_t k) {
        detail::trace_one(sc, par, knot_n(traced[k / ns]), V3{1, 1, 1}, 0, 1.0, part[k], (int)(k % ns));
    });
    // every way seen at any of the three, once (middle first), with its patches at each of the three
    struct WayRef {
        const Patch* any;
        std::vector<std::pair<const Result*, const Patch*>> at[3];
    };
    std::vector<WayRef> ways;
    {
        std::unordered_map<uint64_t, size_t> index;
        const int order[3] = {1, 0, 2};
        for (int oi = 0; oi < 3; ++oi)
            for (size_t s = 0; s < ns; ++s) {
                const Result& res = part[(size_t)order[oi] * ns + s];
                for (const Patch& p : res.patches) {
                    auto it = index.find(p.path);
                    if (it == index.end()) {
                        it = index.emplace(p.path, ways.size()).first;
                        ways.push_back(WayRef{&p, {}});
                    }
                    ways[it->second].at[order[oi]].push_back({&res, &p});
                }
            }
    }

    struct Sample {
        bool known = false;          // followed (or traced) at this knot
        std::vector<Poly> F;         // empty = no light takes this way here
        std::vector<double> e;
    };
    auto alike = [](const Sample& a, const Sample& b) {
        if (a.F.empty() || a.F.size() != b.F.size()) return false;
        for (size_t p = 0; p < a.F.size(); ++p)
            if (a.F[p].n != b.F[p].n) return false;
        return true;
    };
    std::vector<std::vector<float>> outTri(ways.size());
    std::vector<FastStats> outSt(ways.size());
    parallel_for(ways.size(), par.threads, [&](size_t wi) {
        const WayRef& wr = ways[wi];
        const Patch* w = wr.any;
        std::vector<float>& tri = outTri[wi];
        FastStats& st = outSt[wi];
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
        };
        const int lastSolid = step_solid(w->seq[w->len - 1]);
        const double detail_ = detailPerSolid.empty() ? 1e-3 : detailPerSolid[std::min((size_t)lastSolid, detailPerSolid.size() - 1)];
        static thread_local std::vector<Sample> samp(G + 1);
        for (Sample& sm : samp) {
            sm.known = false;
            sm.F.clear();
            sm.e.clear();
        }
        for (int k = 0; k < 3; ++k) {
            Sample& sm = samp[(size_t)traced[k]];
            sm.known = true;
            for (const auto& rp : wr.at[k]) {
                Poly F;
                for (int i = 0; i < rp.second->count; ++i)
                    F.push({rp.first->xz[(size_t)(rp.second->first + i) * 2], rp.first->travel[(size_t)(rp.second->first + i)], rp.first->xz[(size_t)(rp.second->first + i) * 2 + 1]});
                sm.F.push_back(F);
                sm.e.push_back(rp.second->rgb.x);
            }
        }
        const Sample &red = samp[0], &mid = samp[(size_t)G / 2], &blue = samp[(size_t)G];
        double move = 0;
        if (alike(red, blue)) {
            for (size_t p = 0; p < red.F.size(); ++p)
                for (int i = 0; i < red.F[p].n; ++i) {
                    const V3 dv = red.F[p].p[(size_t)i] - blue.F[p].p[(size_t)i];
                    move = std::max(move, std::sqrt(dv.x * dv.x + dv.z * dv.z));
                }
        } else {
            // not the same shape across the spectrum: judge by how far apart whatever exists lies
            double x0 = 1e300, x1 = -1e300, z0 = 1e300, z1 = -1e300;
            for (int k = 0; k < 3; ++k)
                for (const Poly& P : samp[(size_t)traced[k]].F)
                    for (int i = 0; i < P.n; ++i) {
                        x0 = std::min(x0, P.p[(size_t)i].x); x1 = std::max(x1, P.p[(size_t)i].x);
                        z0 = std::min(z0, P.p[(size_t)i].z); z1 = std::max(z1, P.p[(size_t)i].z);
                    }
            move = x1 > x0 ? std::max(x1 - x0, z1 - z0) : 0;
        }
        const int m = move <= detail_ ? 1 : (int)std::min<double>(N, std::ceil(move / detail_));
        if (m == 1) {
            // (a patch that exists in only part of the spectrum and does not move: draw it where it is known, in
            // the colour of that part)
            const bool r_ = !red.F.empty(), m_ = !mid.F.empty(), b_ = !blue.F.empty();
            const Sample& use = m_ ? mid : (r_ ? red : blue);
            V3 colour{0, 0, 0};
            for (int i = 0; i < N; ++i) {
                const bool lowHalf = i >= N / 2;  // band 0 is blue, band N - 1 is red
                const bool on = lowHalf ? (m_ || r_) && (r_ || (m_ && !b_ ? true : m_)) : (m_ || b_);
                if (on) colour = colour + bands[(size_t)i].rgb;
            }
            for (size_t p = 0; p < use.F.size(); ++p) {
                emit(use.F[p], colour * use.e[p]);
                ++st.patches;
            }
            return;
        }
        ++st.coloured;
        auto know = [&](int k) {
            Sample& sm = samp[(size_t)k];
            if (sm.known) return;
            sm.known = true;
            ++st.follows;
            detail::follow(sc, par, knot_n(k), *w, sm.F, sm.e);
        };
        // segments of the index range, left to right: (a, b, may the patch be read off the line between them)
        struct Seg {
            int a, b;
            bool line;
        };
        Seg segs[G];
        int nseg = 0;
        const double tol = 0.35 * detail_;
        // the smallest interval worth cutting: finer than the steps that will be drawn makes no difference
        int finest = 1;
        while (finest * 2 * m <= G) finest *= 2;
        auto resolve = [&](auto&& self, int a, int b) -> void {
            const Sample &A = samp[(size_t)a], &B = samp[(size_t)b];
            if (b - a <= finest) {
                segs[nseg++] = Seg{a, b, alike(A, B)};
                return;
            }
            const int h = (a + b) / 2;
            know(h);
            const Sample& H = samp[(size_t)h];
            if (A.F.empty() && B.F.empty() && H.F.empty()) {
                segs[nseg++] = Seg{a, b, false};  // nothing there
                return;
            }
            bool ok = alike(A, B) && alike(A, H);
            for (size_t p = 0; ok && p < H.F.size(); ++p) {
                for (int i = 0; i < H.F[p].n; ++i) {
                    const V3 dv = (A.F[p].p[(size_t)i] + B.F[p].p[(size_t)i]) * 0.5 - H.F[p].p[(size_t)i];
                    if (std::sqrt(dv.x * dv.x + dv.z * dv.z) > tol) ok = false;
                }
                if (std::fabs(0.5 * (A.e[p] + B.e[p]) - H.e[p]) > 0.03 * H.e[p]) ok = false;
            }
            if (ok) {
                ++st.smooth;
                segs[nseg++] = Seg{a, h, true};
                segs[nseg++] = Seg{h, b, true};
                return;
            }
            self(self, a, h);
            self(self, h, b);
        };
        resolve(resolve, 0, G / 2);
        resolve(resolve, G / 2, G);
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
            const double x = std::min((double)G, std::max(0.0, G * (n - nRed) / (nBlue - nRed)));
            int si = 0;
            while (si + 1 < nseg && x > segs[si].b) ++si;
            const Seg& sg = segs[si];
            const Sample &A = samp[(size_t)sg.a], &B = samp[(size_t)sg.b];
            if (sg.line) {
                const double t = (x - sg.a) / (double)(sg.b - sg.a);
                for (size_t p = 0; p < A.F.size(); ++p) {
                    Poly H;
                    H.n = A.F[p].n;
                    for (int i = 0; i < H.n; ++i) H.p[(size_t)i] = A.F[p].p[(size_t)i] * (1.0 - t) + B.F[p].p[(size_t)i] * t;
                    emit(H, colour * (A.e[p] * (1.0 - t) + B.e[p] * t));
                    ++st.patches;
                }
            } else {
                // no line to read from: the nearer end that has light
                const Sample& U = (x - sg.a <= sg.b - x) ? (A.F.empty() ? B : A) : (B.F.empty() ? A : B);
                for (size_t p = 0; p < U.F.size(); ++p) {
                    emit(U.F[p], colour * U.e[p]);
                    ++st.patches;
                }
            }
        }
    });
    FastStats st;
    st.ways = (int)ways.size();
    size_t total = 0;
    for (const auto& t : outTri) total += t.size();
    tri.reserve(total);
    for (size_t i = 0; i < outTri.size(); ++i) {
        tri.insert(tri.end(), outTri[i].begin(), outTri[i].end());
        st.coloured += outSt[i].coloured;
        st.smooth += outSt[i].smooth;
        st.patches += outSt[i].patches;
        st.follows += outSt[i].follows;
    }
    if (stats) *stats = st;
}
inline void trace_fast(const Solid& solid, const Params& par, double detail_, std::vector<float>& tri, FastStats* stats = nullptr) {
    trace_fast(one(solid), par, std::vector<double>{detail_}, tri, stats);
}

}  // namespace beam
