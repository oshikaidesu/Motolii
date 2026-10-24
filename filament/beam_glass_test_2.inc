// beam_glass_test.cpp, piece 2 (cut there by size, as it was): read only in its place in beam_glass_test.cpp.
    std::printf("2. polygons against rays walked one by one (single wavelength, 96 x 96 bins)\n");
    for (const Case& c : cases) {
        Solid s = chamfered_box(center, c.yaw, half, c.chamfer * 2.0 * half);
        Params par;
        par.sun = c.sun;
        par.bands = 1;
        par.nmMin = par.nmMax = 550.0;
        Result r = trace(s, par);
        const double R = 4.0;
        Image a(96 * 8, 0, 0, R), b(96, 0, 0, R);
        raster(a, r, 1.0, 2);
        double rayFloor = walk_all(s, par, 550.0, b, 3000);
        // bring the polygon image down to the same bins (band weight for one band is (1, 1, 1); use the red channel)
        double l1 = 0, tot = 0, polyFloor = 0;
        for (int z = 0; z < 96; ++z)
            for (int x = 0; x < 96; ++x) {
                double sum = 0;
                for (int dz = 0; dz < 8; ++dz)
                    for (int dx = 0; dx < 8; ++dx) sum += a.rgb[((size_t)(z * 8 + dz) * a.W + (x * 8 + dx)) * 3];
                sum /= 64.0;
                double ref = b.rgb[((size_t)z * b.W + x) * 3];
                l1 += std::fabs(sum - ref);
                tot += ref;
                polyFloor += sum * b.px() * b.px();
            }
        double inside = 0;
        for (size_t i = 0; i < b.rgb.size(); i += 3) inside += b.rgb[i] * b.px() * b.px();
        bool ok = l1 / tot < 0.02 && std::fabs(r.fluxFloor - rayFloor) / rayFloor < 0.003;
        bad += ok ? 0 : 1;
        std::printf("   %-24s floor light: polygons %.5f  rays %.5f  (%.2f%% apart)   picture difference %.2f%%  (in frame %.1f%%)  %s\n", c.name, r.fluxFloor, rayFloor,
                    100.0 * std::fabs(r.fluxFloor - rayFloor) / rayFloor, 100.0 * l1 / tot, 100.0 * inside / rayFloor, ok ? "ok" : "FAIL");
        (void)polyFloor;
    }

    std::printf("2b. other solids: energy, and polygons against rays walked one by one\n");
    {
        struct Shape { const char* name; Solid s; };
        auto rot = [](double yaw, double tilt, V3 R[3]) {
            // tilt about x, then yaw about y
            const double cy = std::cos(yaw), sy = std::sin(yaw), ct = std::cos(tilt), st = std::sin(tilt);
            const V3 X{1, 0, 0}, Y{0, ct, st}, Z{0, -st, ct};
            auto yawed = [&](V3 v) { return V3{cy * v.x + sy * v.z, v.y, -sy * v.x + cy * v.z}; };
            R[0] = yawed(X); R[1] = yawed(Y); R[2] = yawed(Z);
        };
        auto rest = [&](const Solid& local, double yaw, double tilt) {
            V3 R[3];
            rot(yaw, tilt, R);
            Solid t = placed(local, R, 1.0, {0, 0, 0});
            double low = 1e30;
            for (const Poly& P : t.faces)
                for (int i = 0; i < P.n; ++i) low = std::min(low, P.p[(size_t)i].y);
            return placed(local, R, 1.0, {0, -low + 1e-4, 0});
        };
        const double pi = 3.14159265358979;
        std::vector<Shape> shapes;
        shapes.push_back({"slab", rest(chamfered(box(0.6, 0.12, 0.4), 0.012), 0.4, 0)});
        shapes.push_back({"pillar", rest(chamfered(box(0.2, 0.6, 0.2), 0.012), 0.7, 0)});
        shapes.push_back({"triangular prism, lying", rest(chamfered(prism(3, 0.4, 0.5), 0.012), 0.5, pi / 2)});
        shapes.push_back({"triangular prism, standing", rest(chamfered(prism(3, 0.4, 0.5), 0.012), 0.2, 0)});
        shapes.push_back({"hexagonal prism", rest(chamfered(prism(6, 0.4, 0.3), 0.012), 0.3, 0)});
        shapes.push_back({"octagonal prism, lying", rest(chamfered(prism(8, 0.3, 0.5), 0.012), 0.9, pi / 2)});
        shapes.push_back({"square pyramid", rest(chamfered(pyramid(4, 0.5, 0.6), 0.012), 0.3, 0)});
        shapes.push_back({"octahedron", rest(chamfered(octahedron(0.45), 0.012), 0.2, 0.6155)});
        shapes.push_back({"tetrahedron", rest(chamfered(pyramid(3, 0.5, 0.7), 0.012), 0.1, 0)});
        for (const Shape& sh : shapes) {
            Params par;
            par.sun = {0.62, -0.62, 0.28};
            par.bands = 1;
            par.nmMin = par.nmMax = 550.0;
            Result r = trace(sh.s, par);
            double out = r.fluxFloor + r.fluxSky + r.fluxDropped, err = std::fabs(out - r.fluxIn) / r.fluxIn;
            const double R = 5.0;
            Image a(96 * 8, 0, 0, R), b(96, 0, 0, R);
            raster(a, r, 1.0, 2);
            double rayFloor = walk_all(sh.s, par, 550.0, b, 2400);
            double l1 = 0, tot = 0;
            for (int z = 0; z < 96; ++z)
                for (int x = 0; x < 96; ++x) {
                    double sum = 0;
                    for (int dz = 0; dz < 8; ++dz)
                        for (int dx = 0; dx < 8; ++dx) sum += a.rgb[((size_t)(z * 8 + dz) * a.W + (x * 8 + dx)) * 3];
                    sum /= 64.0;
                    double ref = b.rgb[((size_t)z * b.W + x) * 3];
                    l1 += std::fabs(sum - ref);
                    tot += ref;
                }
            std::vector<float> tri;
            FastStats st;
            Params pc;
            pc.sun = par.sun;
            pc.glass.exaggerate = 10;
            auto t0 = std::chrono::steady_clock::now();
            trace_fast(sh.s, pc, 0.003, tri, &st);
            double ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
            bool ok = err < 1e-9 && l1 / tot < 0.03 && std::fabs(r.fluxFloor - rayFloor) / rayFloor < 0.005;
            bad += ok ? 0 : 1;
            std::printf("   %-28s %2zu faces  energy error %.1e  floor %.3f sky %.3f dropped %.3f   vs rays: light %.2f%% apart, picture %.2f%%   fast %.2f ms, %d ways  %s\n",
                        sh.name, sh.s.planes.size(), err, r.fluxFloor / r.fluxIn, r.fluxSky / r.fluxIn, r.fluxDropped / r.fluxIn,
                        100.0 * std::fabs(r.fluxFloor - rayFloor) / rayFloor, 100.0 * l1 / tot, ms, st.ways, ok ? "ok" : "FAIL");
        }
    }

    std::printf("2c. several solids together: energy, polygons against rays, fast against reference\n");
    {
        auto rest_on = [](const Solid& local, double yaw, double x, double y0, double z) {
            const double cy = std::cos(yaw), sy = std::sin(yaw);
            V3 R[3] = {{cy, 0, -sy}, {0, 1, 0}, {sy, 0, cy}};
            Solid t = placed(local, R, 1.0, {0, 0, 0});
            double low = 1e30;
            for (const Poly& P : t.faces)
                for (int i = 0; i < P.n; ++i) low = std::min(low, P.p[(size_t)i].y);
            return placed(local, R, 1.0, {x, y0 - low + 1e-4, z});
        };
        const Solid cube = chamfered(box(0.5, 0.5, 0.5), 0.02), slab = chamfered(box(0.7, 0.14, 0.5), 0.02);
        const Solid tri3 = chamfered(prism(3, 0.5, 0.5), 0.02), pil = chamfered(box(0.22, 0.7, 0.22), 0.02);
        struct Set { const char* name; Scene sc; V3 sun; };
        std::vector<Set> sets;
        {
            Scene sc;  // the second cube stands in the first one's shadow and light
            sc.solids = {rest_on(cube, 0.3, 0, 0, 0), rest_on(cube, 0.9, 1.25, 0, 0.45)};
            sc.bound();
            sets.push_back({"two cubes, one behind the other", sc, {0.92, -0.42, 0.22}});
        }
        {
            Scene sc;  // a slab lying on a cube
            sc.solids = {rest_on(cube, 0.2, 0, 0, 0), rest_on(slab, 0.7, 0.05, 1.0002, 0.0)};
            sc.bound();
            sets.push_back({"slab resting on a cube", sc, {0.62, -0.62, 0.28}});
        }
        {
            Scene sc;  // three shapes close together
            sc.solids = {rest_on(tri3, 0.4, -0.2, 0, 0.1), rest_on(cube, 1.1, 0.95, 0, -0.35), rest_on(pil, 0.2, 0.55, 0, 0.75)};
            sc.bound();
            sets.push_back({"prism, cube and pillar close together", sc, {0.80, -0.45, -0.25}});
        }
        for (const Set& st : sets) {
            Params par;
            par.sun = st.sun;
            par.bands = 1;
            par.nmMin = par.nmMax = 550.0;
            Result r = trace(st.sc, par);
            double out = r.fluxFloor + r.fluxSky + r.fluxDropped, err = std::fabs(out - r.fluxIn) / r.fluxIn;
            const double R = 6.0;
            Image a(96 * 8, 0.8, 0, R), b(96, 0.8, 0, R);
            raster(a, r, 1.0, 2);
            double rayFloor = walk_scene(st.sc, par, 550.0, b, 3000);
            double l1 = 0, tot = 0;
            for (int z = 0; z < 96; ++z)
                for (int x = 0; x < 96; ++x) {
                    double sum = 0;
                    for (int dz = 0; dz < 8; ++dz)
                        for (int dx = 0; dx < 8; ++dx) sum += a.rgb[((size_t)(z * 8 + dz) * a.W + (x * 8 + dx)) * 3];
                    sum /= 64.0;
                    double ref = b.rgb[((size_t)z * b.W + x) * 3];
                    l1 += std::fabs(sum - ref);
                    tot += ref;
                }
            // how much of the floor light went through more than one solid
            double shared = 0, all = 0;
            for (const Patch& p : r.patches) {
                double ar = 0;
                for (int i = 1; i + 1 < p.count; ++i) {
                    const double* q = &r.xz[(size_t)p.first * 2];
                    ar += 0.5 * std::fabs((q[i * 2] - q[0]) * (q[i * 2 + 3] - q[1]) - (q[i * 2 + 2] - q[0]) * (q[i * 2 + 1] - q[1]));
                }
                bool two = false;
                for (int i = 1; i < p.len; ++i) two = two || step_solid(p.seq[i]) != step_solid(p.seq[0]);
                all += ar * p.rgb.x;
                if (two) shared += ar * p.rgb.x;
            }
            // fast against the reference, all colours
            Params pc;
            pc.sun = st.sun;
            pc.glass.exaggerate = 10;
            const int Wd = 1280;
            const double detail = 2.0 * R / Wd;
            Result ref = trace(st.sc, pc);
            std::vector<float> tri;
            FastStats fs;
            auto t0 = std::chrono::steady_clock::now();
            trace_fast(st.sc, pc, std::vector<double>(st.sc.solids.size(), detail), tri, &fs);
            double ms1 = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
            pc.threads = 8;
            std::vector<float> tri8;
            t0 = std::chrono::steady_clock::now();
            trace_fast(st.sc, pc, std::vector<double>(st.sc.solids.size(), detail), tri8);
            double ms8 = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
            const bool sameThreads = tri == tri8;
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
            Image fa(Wd, 0.8, 0, R), fb(Wd, 0.8, 0, R);
            raster(fa, ref, 1.0, 2);
            raster(fb, fr, 1.0, 2);
            double fl1 = 0, ftot = 0;
            for (size_t i = 0; i < fa.rgb.size(); ++i) {
                fl1 += std::fabs(fa.rgb[i] - fb.rgb[i]);
                ftot += std::fabs(fa.rgb[i]);
            }
            bool ok = err < 1e-9 && l1 / tot < 0.03 && std::fabs(r.fluxFloor - rayFloor) / rayFloor < 0.005 && fl1 / ftot < 0.03 && sameThreads;
            bad += ok ? 0 : 1;
            std::printf("   %-38s energy error %.1e   vs rays: light %.2f%% apart, picture %.2f%%   through more than one solid: %.1f%% of the floor light\n", st.name, err,
                        100.0 * std::fabs(r.fluxFloor - rayFloor) / rayFloor, 100.0 * l1 / tot, 100.0 * shared / all);
            std::printf("   %-38s fast vs reference %.2f%%   %d ways, %.2f ms on 1 thread, %.2f ms on 8, same triangles: %s  %s\n", "", 100.0 * fl1 / ftot, fs.ways, ms1, ms8,
                        sameThreads ? "yes" : "NO", ok ? "ok" : "FAIL");
            // the cheap version next to it: time, how far its picture is from the exact one, and its own top view
            for (int pieces : {1, 2, 4}) {
                Params ph = pc;
                ph.threads = 1;
                std::vector<float> tc;
                CheapStats cs;
                double msC = 1e30;
                for (int k = 0; k < 20; ++k) {
                    t0 = std::chrono::steady_clock::now();
                    trace_cheap(st.sc, ph, std::vector<double>(st.sc.solids.size(), detail), pieces, tc, &cs);
                    msC = std::min(msC, std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
                }
                Result cr;
                for (size_t i = 0; i + 3 * TRI <= tc.size(); i += 3 * TRI) {
                    Patch p;
                    p.first = (int)(cr.xz.size() / 2);
                    p.count = 3;
                    p.rgb = {tc[i + 2], tc[i + 3], tc[i + 4]};
                    for (int k = 0; k < 3; ++k) {
                        cr.xz.push_back(tc[i + (size_t)k * TRI]);
                        cr.xz.push_back(tc[i + (size_t)k * TRI + 1]);
                    }
                    cr.patches.push_back(p);
                }
                Image ca(Wd, 0.8, 0, R);
                raster(ca, cr, 1.0, 2);
                double cl1 = 0, sumC = 0, sumE = 0;
                for (size_t i = 0; i < fa.rgb.size(); ++i) {
                    cl1 += std::fabs(fa.rgb[i] - ca.rgb[i]);
                    sumC += ca.rgb[i];
                    sumE += fa.rgb[i];
                }
                std::printf("   %-38s cheap, pieces %d: %.3f ms on 1 thread, %d pieces, %d patches, %d rays; light on the floor %.0f%% of exact, picture %.0f%% away from exact\n", "",
                            pieces, msC, cs.pieces, cs.patches, cs.rays, 100.0 * sumC / sumE, 100.0 * cl1 / ftot);
                Image im(1400, 0.8, 0, 3.4), cov(1400, 0.8, 0, 3.4);
                const V3 L = normalize(pc.sun);
                for (const Solid& so : st.sc.solids) raster(cov, silhouette(so, pc.sun, 0.0), 1.0);
                raster(im, cr, 1.0);
                std::vector<uint8_t> px((size_t)1400 * 1400 * 4);
                for (int i = 0; i < 1400 * 1400; ++i) {
                    for (int c = 0; c < 3; ++c) {
                        const double shade = std::min(1.0, cov.rgb[(size_t)i * 3] / (-L.y));
                        double v = (std::max(0.0, im.rgb[(size_t)i * 3 + c]) + (-L.y) * (1.0 - shade) + 0.10) * 0.80 * 0.95;
                        v = v / (1.0 + 0.18 * v);
                        v = std::min(1.0, std::max(0.0, v));
                        v = v <= 0.0031308 ? 12.92 * v : 1.055 * std::pow(v, 1.0 / 2.4) - 0.055;
                        px[(size_t)i * 4 + c] = (uint8_t)std::lround(v * 255.0);
                    }
                    px[(size_t)i * 4 + 3] = 255;
                }
                std::string cf = std::string("out/beam-top-scene-") + std::to_string(&st - &sets[0]) + "-cheap" + std::to_string(pieces) + ".png";
                write_png(cf.c_str(), 1400, 1400, px.data());
            }
            Params pv = pc;
            std::string file = std::string("out/beam-top-scene-") + std::to_string(&st - &sets[0]) + ".png";
            // top view picture: bare floor, minus every outline, plus the light
            {
                Image im(1400, 0.8, 0, 3.4);
                const V3 L = normalize(pv.sun);
                for (size_t i = 0; i < im.rgb.size(); ++i) im.rgb[i] = 0;
                Image cov(1400, 0.8, 0, 3.4);
                for (const Solid& so : st.sc.solids) raster(cov, silhouette(so, pv.sun, 0.0), 1.0);
                raster(im, ref, 1.0);
                std::vector<uint8_t> px((size_t)1400 * 1400 * 4);
                for (int i = 0; i < 1400 * 1400; ++i) {
                    for (int c = 0; c < 3; ++c) {
                        const double shade = std::min(1.0, cov.rgb[(size_t)i * 3] / (-L.y));
                        double v = (std::max(0.0, im.rgb[(size_t)i * 3 + c]) + (-L.y) * (1.0 - shade) + 0.10) * 0.80 * 0.95;
                        v = v / (1.0 + 0.18 * v);
                        v = std::min(1.0, std::max(0.0, v));
                        v = v <= 0.0031308 ? 12.92 * v : 1.055 * std::pow(v, 1.0 / 2.4) - 0.055;
                        px[(size_t)i * 4 + c] = (uint8_t)std::lround(v * 255.0);
                    }
                    px[(size_t)i * 4 + 3] = 255;
                }
                write_png(file.c_str(), 1400, 1400, px.data());
            }
        }
    }

    std::printf("2d. round solids: the cheap trace next to rays walked one by one (three wavelengths as red, green, blue)\n");
    {
        auto at = [](const Solid& local, V3 turnAxisTo, double x, double y, double z) {
            // the solid's +Y goes to `turnAxisTo`
            const V3 Y = normalize(turnAxisTo);
            const V3 up = std::fabs(Y.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
            const V3 X = normalize(cross(up, Y)), Z = cross(X, Y);
            V3 R[3] = {X, Y, Z};
            return placed(local, R, 1.0, {x, y, z});
        };
        struct Set { const char* name; const char* file; Scene sc; V3 sun; };
        std::vector<Set> sets;
        { Scene sc; sc.solids = {at(ball(0.5), {0, 1, 0}, 0, 0.5005, 0)}; sc.bound(); sets.push_back({"ball", "ball", sc, {0.62, -0.62, 0.28}}); }
        { Scene sc; sc.solids = {at(rod(0.3, 0.6), {0, 1, 0}, 0, 0.6005, 0)}; sc.bound(); sets.push_back({"rod, standing", "rod-up", sc, {0.62, -0.62, 0.28}}); }
        { Scene sc; sc.solids = {at(rod(0.3, 0.7), {0.3, 0, 1}, 0, 0.3005, 0)}; sc.bound(); sets.push_back({"rod, lying", "rod-flat", sc, {0.62, -0.62, 0.28}}); }
        {
            Scene sc;
            V3 I[3] = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
            sc.solids = {at(ball(0.4), {0, 1, 0}, 0, 0.4005, 0), placed(chamfered(box(0.4, 0.4, 0.4), 0.02), I, 1.0, {1.1, 0.4005, 0.35})};
            sc.bound();
            sets.push_back({"ball, with a cube in its light", "ball-cube", sc, {0.85, -0.45, 0.22}});
        }
        { Scene sc; sc.solids = {at(cone(0.45, 0.95), {0, 1, 0}, 0, 0.4755, 0)}; sc.bound(); sets.push_back({"cone, standing", "cone-up", sc, {0.62, -0.62, 0.28}}); }
        { Scene sc; sc.solids = {at(cone(0.45, 0.95), {0, 1, 0}, 0, 0.4755, 0)}; sc.bound(); sets.push_back({"cone, standing, low sun", "cone-low", sc, {0.85, -0.40, 0.22}}); }
        { Scene sc; sc.solids = {at(cone(0.45, 0.95), {0.9, -0.45, 0.3}, 0, 0.206, 0)}; sc.bound(); sets.push_back({"cone, lying", "cone-flat", sc, {0.62, -0.62, 0.28}}); }
        { Scene sc; sc.solids = {at(cone(0.45, 0.95), {0.2, -1, 0.1}, 0, 0.47, 0)}; sc.bound(); sets.push_back({"cone, tip down", "cone-down", sc, {0.62, -0.62, 0.28}}); }
        for (const Set& st : sets) {
            Params par;
            par.sun = st.sun;
            par.glass.exaggerate = 10;
            const int Wp = 900;
            const double R = 3.0, cx = 0.8, detail = 2.0 * R / Wp;
            // rays: three wavelengths into three channels
            Image ref(Wp, cx, 0, R);
            const double nm3[3] = {640, 550, 460};
            double rayFloor = 0;
            for (int c = 0; c < 3; ++c) rayFloor += walk_any(st.sc, par, par.glass.n(nm3[c]), ref, c, 1.0, 2600) / 3.0;
            std::vector<float> tc;
            CheapStats cs;
            double msC = 1e30;
            for (int k = 0; k < 10; ++k) {
                auto t0 = std::chrono::steady_clock::now();
                trace_cheap(st.sc, par, std::vector<double>(st.sc.solids.size(), detail), 4, tc, &cs);
                msC = std::min(msC, std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
            }
            Result cr;
            for (size_t i = 0; i + 3 * TRI <= tc.size(); i += 3 * TRI) {
                Patch p;
                p.first = (int)(cr.xz.size() / 2);
                p.count = 3;
                p.rgb = {tc[i + 2], tc[i + 3], tc[i + 4]};
                for (int k = 0; k < 3; ++k) {
                    cr.xz.push_back(tc[i + (size_t)k * TRI]);
                    cr.xz.push_back(tc[i + (size_t)k * TRI + 1]);
                }
                cr.patches.push_back(p);
            }
            Image ch(Wp, cx, 0, R);
            raster(ch, cr, 1.0, 2);
            double sumC = 0, sumR = 0;
            for (size_t i = 0; i < ch.rgb.size(); ++i) {
                sumC += ch.rgb[i];
                sumR += ref.rgb[i];
            }
            (void)sumC;
            (void)sumR;
            // the light that is drawn, added up from the triangles themselves (area x brightness, mean of the three channels)
            double drawnLight = 0;
            for (size_t i = 0; i + 3 * TRI <= tc.size(); i += 3 * TRI) {
                const float *a = &tc[i], *b = a + TRI, *c = b + TRI;
                const double ar = 0.5 * std::fabs((double)(b[0] - a[0]) * (c[1] - a[1]) - (double)(c[0] - a[0]) * (b[1] - a[1]));
                drawnLight += ar * ((double)a[2] + a[3] + a[4] + b[2] + b[3] + b[4] + c[2] + c[3] + c[4]) / 9.0;
            }
            std::printf("   %-32s cheap %.3f ms, %d triangles of surface, %d patches, %d rays; light on the floor %.0f%% of the rays' (%.4f)\n", st.name, msC, cs.pieces, cs.patches,
                        cs.rays, 100.0 * drawnLight / rayFloor, rayFloor);
            {
                // the same with the balls that are alike left to the renderer's one shared shape: what the CPU still does
                std::vector<float> tl;
                std::vector<int> balls;
                CheapStats ls;
                double msL = 1e30;
                for (int k = 0; k < 200; ++k) {
                    auto t0 = std::chrono::steady_clock::now();
                    trace_cheap(st.sc, par, std::vector<double>(st.sc.solids.size(), detail), 4, tl, &ls, &balls);
                    msL = std::min(msL, std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
                }
                if (!balls.empty())
                    std::printf("   %-32s   with the %zu ball(s) left to the renderer's shared shape: %.4f ms on the CPU for the whole scene\n", "", balls.size(), msL);
            }
            // pictures: rays | cheap, with the outline as shadow
            Image cov(Wp, cx, 0, R);
            std::vector<float> ot;
            silhouette_triangles(st.sc, par.sun, 0.0, ot);
            Result orr;
            for (size_t i = 0; i + 3 * TRI <= ot.size(); i += 3 * TRI) {
                Patch p;
                p.first = (int)(orr.xz.size() / 2);
                p.count = 3;
                p.rgb = {1, 1, 1};
                for (int k = 0; k < 3; ++k) {
                    orr.xz.push_back(ot[i + (size_t)k * TRI]);
                    orr.xz.push_back(ot[i + (size_t)k * TRI + 1]);
                }
                orr.patches.push_back(p);
            }
            raster(cov, orr, 1.0, 2);
            const V3 L = normalize(par.sun);
            std::vector<uint8_t> px((size_t)Wp * 2 * Wp * 4);
            for (int y = 0; y < Wp; ++y)
                for (int x = 0; x < Wp * 2; ++x) {
                    const Image& im = x < Wp ? ref : ch;
                    const size_t i = (size_t)y * Wp + (size_t)(x % Wp);
                    for (int c = 0; c < 3; ++c) {
                        const double shade = std::min(1.0, cov.rgb[i * 3]);
                        double v = (std::max(0.0, im.rgb[i * 3 + c]) + (-L.y) * (1.0 - shade) + 0.10) * 0.80 * 0.95;
                        v = v / (1.0 + 0.18 * v);
                        v = std::min(1.0, std::max(0.0, v));
                        v = v <= 0.0031308 ? 12.92 * v : 1.055 * std::pow(v, 1.0 / 2.4) - 0.055;
                        px[((size_t)y * Wp * 2 + x) * 4 + c] = (uint8_t)std::lround(v * 255.0);
                    }
                    px[((size_t)y * Wp * 2 + x) * 4 + 3] = 255;
                }
            write_png((std::string("out/beam-top-round-") + st.file + ".png").c_str(), Wp * 2, Wp, px.data());
        }
    }

    std::printf("3. colour on the floor (0 = white light only)\n");
    for (double ex : {1.0, 4.0}) {
        double v[3];
        int k = 0;
        for (double ch : {0.0, 0.015, 0.05}) {
            Solid s = chamfered_box(center, 0.35, half, ch * 2.0 * half);
            Params par;
            par.sun = {0.62, -0.62, 0.28};
            par.bands = 48;
            par.glass.exaggerate = ex;
            v[k++] = chroma(trace(s, par), 0, 0, 3.0);
        }
        std::printf("   dispersion x%.0f:  perfect box %.4f   chamfer 1.5%% %.4f   chamfer 5%% %.4f\n", ex, v[0], v[1], v[2]);
    }
