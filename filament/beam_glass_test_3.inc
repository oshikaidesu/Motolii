// beam_glass_test.cpp, piece 3 (cut there by size, as it was): read only in its place in beam_glass_test.cpp.
    std::printf("4. pictures and time\n");
    struct Shot { const char* file; V3 sun; double yaw, chamfer, ex; int bands; };
    const Shot shots[] = {
        {"out/beam-top-perfect.png", {0.62, -0.62, 0.28}, 0.35, 0.0, 1.0, 64},
        {"out/beam-top-chamfer.png", {0.62, -0.62, 0.28}, 0.35, 0.015, 1.0, 64},
        {"out/beam-top-chamfer-x4.png", {0.62, -0.62, 0.28}, 0.35, 0.015, 4.0, 64},
        {"out/beam-top-lowsun-x4.png", {0.85, -0.38, -0.20}, 0.80, 0.015, 4.0, 64},
        {"out/beam-top-lowsun-perfect-x4.png", {0.85, -0.38, -0.20}, 0.80, 0.0, 4.0, 64},
    };
    for (const Shot& sh : shots) {
        Solid s = chamfered_box(center, sh.yaw, half, sh.chamfer * 2.0 * half);
        Params par;
        par.sun = sh.sun;
        par.bands = sh.bands;
        par.glass.exaggerate = sh.ex;
        auto t0 = std::chrono::steady_clock::now();
        Result r = trace(s, par);
        double ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
        std::vector<float> tri;
        to_triangles(r, tri);
        V3 L = normalize(sh.sun);
        save_top(sh.file, s, par, r, 1.2 * L.x / std::max(0.3, -L.y) * 0.6, 1.2 * L.z / std::max(0.3, -L.y) * 0.6, 2.6, 1400);
        std::printf("   %-36s %d bands: %.2f ms, %zu patches, %zu triangles\n", sh.file, sh.bands, ms, r.patches.size(), tri.size() / (3 * TRI));
    }
    std::printf("5. fast trace against the reference (64 bands traced in full)\n");
    for (const Case& c : cases) {
        for (double ex : {1.0, 4.0}) {
            Solid s = chamfered_box(center, c.yaw, half, c.chamfer * 2.0 * half);
            Params par;
            par.sun = c.sun;
            par.glass.exaggerate = ex;
            const double R = 4.0;
            const int Wd = 1536;
            const double detail = 2.0 * R / Wd;  // one pixel of the comparison picture
            auto t0 = std::chrono::steady_clock::now();
            Result ref = trace(s, par);
            double msRef = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
            std::vector<float> tri;
            FastStats st;
            double msFast = 1e30;
            for (int k = 0; k < 20; ++k) {
                t0 = std::chrono::steady_clock::now();
                trace_fast(s, par, detail, tri, &st);
                msFast = std::min(msFast, std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
            }
            Result fr;
            for (size_t i = 0; i + 3 * TRI <= tri.size(); i += 3 * TRI) {
                Patch p;
                p.first = (int)(fr.xz.size() / 2);
                p.count = 3;
                p.rgb = {tri[i + 2], tri[i + 3], tri[i + 4]};
                for (int k = 0; k < 3; ++k) {
                    fr.xz.push_back(tri[i + (size_t)k * TRI]);
                    fr.xz.push_back(tri[i + (size_t)k * TRI + 1]);
                }
                fr.patches.push_back(p);
            }
            Image a(Wd, 0, 0, R), b(Wd, 0, 0, R);
            raster(a, ref, 1.0, 2);
            raster(b, fr, 1.0, 2);
            double l1 = 0, tot = 0, worst = 0;
            for (size_t i = 0; i < a.rgb.size(); ++i) {
                double d = std::fabs(a.rgb[i] - b.rgb[i]);
                l1 += d;
                tot += std::fabs(a.rgb[i]);
                worst = std::max(worst, d);
            }
            bool ok = l1 / tot < 0.02;
            bad += ok ? 0 : 1;
            std::printf("   %-24s x%.0f: picture difference %.2f%% (worst pixel %.3f)   reference %.1f ms   fast %.3f ms   %d ways, %d coloured (%d smooth), %d follows, %zu triangles  %s\n", c.name, ex,
                        100.0 * l1 / tot, worst, msRef, msFast, st.ways, st.coloured, st.smooth, st.follows, tri.size() / (3 * TRI), ok ? "ok" : "FAIL");
        }
    }
    std::printf("receivers: every ray is kept on one receiver and one only (the rule of recv_plane.glsl / recv_lands.glsl, on the CPU)\n");
    {
        // a room with two walls meeting in a corner and three blocks; rays from anywhere inside it, any way
        const double pi = 3.14159265358979323846;
        std::vector<Half> walls = {Half{V3{0, 0, 1}, 1.2}, Half{V3{std::cos(0.2), 0, -std::sin(0.2)}, 1.5}};
        std::vector<Block> blocks(3);
        const double at[3][7] = {{-0.4, 0.07, -0.3, 0.17, 0.07, 0.17, 0.3}, {0.3, 0.20, 0.1, 0.12, 0.20, 0.12, -0.5}, {0.5, 0.09, -0.7, 0.22, 0.09, 0.10, 0.9}};
        for (int b = 0; b < 3; ++b) {
            blocks[(size_t)b].c = {at[b][0], at[b][1], at[b][2]};
            blocks[(size_t)b].ax[0] = {std::cos(at[b][6]), 0, -std::sin(at[b][6])};
            blocks[(size_t)b].ax[1] = {0, 1, 0};
            blocks[(size_t)b].ax[2] = {std::sin(at[b][6]), 0, std::cos(at[b][6])};
            for (int k = 0; k < 3; ++k) blocks[(size_t)b].h[k] = at[b][3 + k];
        }
        const Stage st = make_stage(0.0, walls, 3.0, blocks);
        uint64_t seed = 88172645463325252ull;
        auto rnd = [&]() {
            seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
            return (double)(seed >> 11) / 9007199254740992.0;
        };
        int rays = 0, none = 0, both = 0, wrong = 0, lost = 0, onBlocks = 0, onRoom = 0;
        while (rays < 400000) {
            const V3 E{-1.4 + 3.4 * rnd(), 0.01 + 1.2 * rnd(), -1.1 + 3.0 * rnd()};
            bool inside = true;
            for (const Half& h : st.room) inside = inside && dot(h.a, E) + h.b > 0.02;
            for (const Block& B : st.blocks) {
                bool in = true;
                for (int k = 0; k < 3; ++k) in = in && std::fabs(dot(B.ax[k], E - B.c)) < B.h[k] + 0.02;
                inside = inside && !in;
            }
            if (!inside) continue;
            const double z = 2.0 * rnd() - 1.0, ph = 2.0 * pi * rnd(), rr = std::sqrt(1.0 - z * z);
            const V3 D{rr * std::cos(ph), z, rr * std::sin(ph)};
            ++rays;
            // the truth: the first block the ray enters, else the first plane of the room (if that is not where the walls end)
            int truth = -2;  // -2 nothing, -1 the room, i a block
            double tTruth = 1e30;
            for (size_t j = 0; j < st.room.size(); ++j) {
                const double den = dot(st.room[j].a, D);
                if (den > -1e-12) continue;
                const double t = -(dot(st.room[j].a, E) + st.room[j].b) / den;
                if (t < tTruth) {
                    tTruth = t;
                    truth = j + 1 < st.room.size() ? -1 : -2;
                }
            }
            for (size_t i = 0; i < st.blocks.size(); ++i) {
                const Block& B = st.blocks[i];
                double t0 = 0, t1 = tTruth;
                bool miss = false;
                for (int k = 0; k < 3; ++k) {
                    const double o = dot(B.ax[k], E - B.c), d = dot(B.ax[k], D);
                    if (std::fabs(d) < 1e-12) {
                        miss = miss || std::fabs(o) > B.h[k];
                        continue;
                    }
                    const double ta = (-B.h[k] - o) / d, tb = (B.h[k] - o) / d;
                    t0 = std::max(t0, std::min(ta, tb));
                    t1 = std::min(t1, std::max(ta, tb));
                }
                if (!miss && t0 < t1) {
                    tTruth = t0;
                    truth = (int)i;
                }
            }
            // the rule: once dropped on the room, once on the blocks; which of the two is kept?
            // (the sun is put where no block can be in its way: straight above)
            const V3 sun{0, -1, 0};
            bool kept[2] = {false, false};
            int owner[2] = {-1, -1};
            for (int kind = 0; kind < 2; ++kind) {
                Plane N;
                if (!stage_choose(st, kind, E, D, N, owner[kind])) continue;
                const double heading = -dot(N.n, D);
                if (heading <= 0) continue;
                const double t = (dot(N.n, E) - N.o) / heading;
                if (t < 0) continue;
                const V3 X = E + D * t;
                bool clear = stage_on(st, X, owner[kind]);
                for (size_t i = 0; i < st.blocks.size() && clear; ++i)
                    if (stage_through(st.blocks[i], E, X - E, 0.9999)) clear = false;
                (void)sun;
                kept[kind] = clear;
            }
            const int count = (kept[0] ? 1 : 0) + (kept[1] ? 1 : 0);
            if (count == 2) ++both;
            if (count == 0 && truth != -2) ++lost;
            if (count == 0) ++none;
            if (count == 1) {
                const int got = kept[1] ? owner[1] : -1;
                if (got != truth) ++wrong;
                (got < 0 ? onRoom : onBlocks)++;
            }
        }
        // (a ray that grazes an edge of a block within the rule's own tolerance, a tenth of a millimetre, may go either
        // way or both: a few in 400 000)
        const bool ok = both + lost + wrong <= 40;
        bad += ok ? 0 : 1;
        std::printf("   %d rays: %d kept on the room, %d on a block, %d on nothing (out of the room); kept twice %d, lost %d, on the wrong thing %d  %s\n", rays, onRoom, onBlocks,
                    none, both, lost, wrong, ok ? "ok" : "FAIL");
    }
    std::printf(bad ? "FAILED: %d\n" : "ALL OK\n", bad);
    return bad ? 1 : 0;
}
