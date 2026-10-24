// glass_scene.mm, piece 6 (cut there by size, as it was): read only in its place in glass_scene.mm.
    if (const char* sv = std::getenv("SUN")) {
        float x, y, z;
        if (std::sscanf(sv, "%f %f %f", &x, &y, &z) == 3) sunDir = normalize(float3{x, y, z});
    }
    const unsigned threads = (unsigned)std::max(1.f, env_f("THREADS", (float)std::max(1u, std::thread::hardware_concurrency())));
    const int pieces = (int)env_f("PIECES", 4.f);       // each lit face is cut into pieces x pieces parts
    const bool exact = std::getenv("EXACT") != nullptr; // the slow exact trace, to compare
    const bool tumble = std::getenv("TUMBLE") != nullptr; // every solid turns about its own tilted axis and leaves the floor
    const bool ballsOnCpu = std::getenv("BALLCPU") != nullptr;  // work every ball's light out on the CPU (to compare)
    const bool conesOnCpu = std::getenv("CONECPU") != nullptr;  // the same for the cones
    size_t conesDrawn = 0;
    double traceMs = 0;
    size_t lightTris = 0, ballsAlike = 0;
    beam::Scene world;
    static std::vector<float> tri, outline;
    auto update = [&](float t) {
        auto& lm = engine->getLightManager();
        lm.setDirection(lm.getInstance(sunE), sunDir);
        softenMi->setParameter("sunDown", -sunDir.y);
        for (Item& it : items) {
            it.glass->setParameter("sunTo", -sunDir);
            it.glass->setParameter("sunGlint", sunColor * env_f("GLINT", 4.0e5f));
            it.glass->setParameter("sunCos", std::cos(env_f("GLINTDEG", 2.0f) * 3.14159265f / 180.f));
            if (it.cheap) {
                it.cheap->setParameter("sunTo", -sunDir);
                it.cheap->setParameter("sunGlint", sunColor * env_f("GLINT", 4.0e5f));
                it.cheap->setParameter("sunCos", std::cos(env_f("GLINTDEG", 2.0f) * 3.14159265f / 180.f));
            }
        }
        if (stageOn) {  // (all the stage costs the CPU in a frame: these few numbers, and switching its sets on and off below)
            for (MaterialInstance* mi : landers) mi->setParameter("recvSun", sunDir);
            for (MaterialInstance* mi : recvSurfaces) mi->setParameter("sunTo", -sunDir);
        }
        auto t0 = std::chrono::steady_clock::now();
        // where every solid stands now
        world.solids.resize(items.size());
        std::vector<double> detail(items.size());
        auto* pose = new std::vector<float>((size_t)SCENE_H * 4 * 4, 0.f);  // where every solid stands: all the CPU tells the flat-face light
        for (size_t k = 0; k < items.size(); ++k) {
            Item& it = items[k];
            const float yaw = it.yaw + it.spin * t;
            const float x = it.x + it.drift * std::cos(t + it.phase), z = it.z + it.drift * std::sin(t + it.phase);
            beam::V3 R[3];
            item_axes(it, yaw, R);
            double lift = 0;
            if (tumble) {
                // thrown about: turning about its own tilted axis, and off the floor
                const beam::V3 ax = beam::normalize(beam::V3{std::sin(1.3 * k + 0.4), 0.6 + 0.4 * std::cos(2.1 * k), std::cos(1.7 * k + 1.0)});
                const double ang = it.phase + (0.9 + 0.35 * (k % 3)) * t * it.spin, cs = std::cos(ang), sn = std::sin(ang);
                for (beam::V3& v : R) v = v * cs + beam::cross(ax, v) * sn + ax * (beam::dot(ax, v) * (1.0 - cs));
                lift = 0.22 * (0.5 + 0.5 * std::sin(1.7 * t + 2.3 * k));
            }
            double low = 1e30;
            for (const beam::Poly& P : it.local.faces)
                for (int i = 0; i < P.n; ++i) {
                    const beam::V3 v = P.p[(size_t)i];
                    low = std::min(low, (R[0].y * v.x + R[1].y * v.y + R[2].y * v.z) * it.size);
                }
            if (it.local.kind == beam::Solid::BALL) low = -it.local.r * it.size;
            if (it.local.kind == beam::Solid::CONE)  // the tip, or the lowest point of the rim
                low = std::min(0.5 * it.local.h * R[1].y, -0.5 * it.local.h * R[1].y - it.local.r * std::sqrt(std::max(0.0, 1.0 - R[1].y * R[1].y))) * it.size;
            if (it.local.kind == beam::Solid::ROD) low = -(it.local.h * std::fabs(R[1].y) + it.local.r * std::sqrt(std::max(0.0, 1.0 - R[1].y * R[1].y))) * it.size;
            if (it.local.kind == beam::Solid::EXTRUDE) {  // the lowest point of its outline, at whichever cap is lower
                low = 1e30;
                for (const beam::V2& p : it.local.shape->pts) low = std::min(low, R[0].y * p.x + R[1].y * p.y);
                low = (low - std::fabs(R[2].y) * it.local.shape->h) * it.size;
            }
            const beam::V3 at{x, -low + 0.0005 + lift, z};
            world.solids[k] = beam::placed(it.local, R, it.size, at);
            if (flatBeams) beamPose[k] = BeamPose{{R[0], R[1], R[2]}, at, (double)it.size};
            const float s = it.size;
            tm.setTransform(tm.getInstance(it.entity),
                            mat4f(float4{(float)R[0].x * s, (float)R[0].y * s, (float)R[0].z * s, 0}, float4{(float)R[1].x * s, (float)R[1].y * s, (float)R[1].z * s, 0},
                                  float4{(float)R[2].x * s, (float)R[2].y * s, (float)R[2].z * s, 0}, float4{(float)at.x, (float)at.y, (float)at.z, 1}));
            for (int c = 0; c < 3; ++c) {
                float* o = pose->data() + ((size_t)c * SCENE_H + k) * 4;
                o[0] = (float)R[c].x; o[1] = (float)R[c].y; o[2] = (float)R[c].z;
            }
            (*pose)[(0 * (size_t)SCENE_H + k) * 4 + 3] = s;
            if (it.cheap) {
                const float4 where[4] = {{(float)R[0].x, (float)R[0].y, (float)R[0].z, s}, {(float)R[1].x, (float)R[1].y, (float)R[1].z, 0.f},
                                         {(float)R[2].x, (float)R[2].y, (float)R[2].z, 0.f}, {(float)at.x, (float)at.y, (float)at.z, 0.f}};
                it.cheap->setParameter("pose", where, 4);
            }
            (*pose)[(1 * (size_t)SCENE_H + k) * 4 + 3] = (float)flatKindOf[k];
            (*pose)[(2 * (size_t)SCENE_H + k) * 4 + 3] = flatKindOf[k] >= 0 ? (float)flatKinds[(size_t)flatKindOf[k]].shape.pieces : 0.f;
            {
                float* o = pose->data() + (3 * (size_t)SCENE_H + k) * 4;
                o[0] = (float)at.x; o[1] = (float)at.y; o[2] = (float)at.z;
            }
            const float dist = length(float3{cam[0], cam[1], cam[2]} - float3{(float)at.x, (float)at.y, (float)at.z});
            detail[k] = 2.0 * std::tan(cam[6] * 0.5 * 3.14159265 / 180.0) * dist / H * env_f("DETAIL", 1.f);  // one pixel on the floor there
        }
        world.bound();
        poseTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(pose->data(), pose->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                     [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, pose));
        beam::Params par;
        par.sun = {sunDir.x, sunDir.y, sunDir.z};
        par.bands = bands;
        par.glass.nd = nd;
        par.glass.exaggerate = disp;
        par.threads = threads;
        par.coneSteps = coneStepCount;
        par.sunTan = std::tan(sunRadiusDeg * 3.14159265 / 180.0);
        std::vector<int> alike;  // the balls and cones: drawn from their shared shapes
        if (exact) {
            par.minFlux = 30.0 * detail[0] * detail[0];
            beam::trace_fast(world, par, detail, tri);
        } else {
            const unsigned leave = (1u << beam::Solid::BALL) | (1u << beam::Solid::CONE) | (1u << beam::Solid::ROD) | (1u << beam::Solid::EXTRUDE) |
                                   (flatOnCpu ? 0u : 1u << beam::Solid::FLAT);
            beam::trace_cheap(world, par, detail, pieces, tri, nullptr, ballsOnCpu ? nullptr : &alike, leave);
        }
        if (flatBeams && !std::getenv("NOLIGHT")) {
            const auto b0 = std::chrono::steady_clock::now();
            const std::vector<beam::BallStep> bsteps = beam::ball_steps(par, flatSteps - 1);
            double filmN[4][2];
            for (int f = 0; f < 4; ++f) { filmN[f][0] = par.glass.n(filmNm[f].y); filmN[f][1] = par.glass.n(filmNm[f].x); }
            beam::SplitBand band;
            if (const char* sv = std::getenv("STRIP")) {   // (the same band of sun the floor gets: beam_soften.mat)
                float half = 0.3f, deg = 35.f, side = 0.f, every = 0.f;
                std::sscanf(sv, "%f %f %f %f", &half, &deg, &side, &every);
                const float a = deg * 3.14159265f / 180.f;
                const float3 down = float3{(float)world.centre[0].x, (float)world.centre[0].y, (float)world.centre[0].z} + sunDir * ((float)world.centre[0].y / -sunDir.y);
                band = beam::SplitBand{std::cos(a), std::sin(a), std::cos(a) * down.x + std::sin(a) * down.z + side, half, every};
            }
            static std::vector<std::vector<float>> each;
            static std::vector<beam::SplitStats> stats;
            each.resize(items.size());
            stats.assign(items.size(), beam::SplitStats{});
            const double least = env_f("BEAMCUT", 2e-5f);
            const int mostFaces = (int)env_f("BEAMFACES", 8.f);
#ifdef GLASS_COMPUTE_DOOR
            if (cBeams) {
                cbeam::Frame f{};
                const beam::V3 sn = normalize(par.sun);
                f.sun[0] = (float)sn.x; f.sun[1] = (float)sn.y; f.sun[2] = (float)sn.z; f.sun[3] = 0.f;
                f.band[0] = (float)band.ux; f.band[1] = (float)band.uz; f.band[2] = (float)band.c; f.band[3] = (float)band.half;
                f.misc[0] = (float)band.every; f.misc[1] = (float)least; f.misc[2] = (float)mostFaces; f.misc[3] = 1.f;
                f.counts[1] = (uint32_t)bsteps.size();
                for (size_t i = 0; i < bsteps.size() && i < 16; ++i) {
                    f.steps[i][0] = (float)bsteps[i].rgb.x; f.steps[i][1] = (float)bsteps[i].rgb.y; f.steps[i][2] = (float)bsteps[i].rgb.z; f.steps[i][3] = (float)bsteps[i].n;
                }
                for (int i = 0; i < 4; ++i) { f.films[i][0] = (float)filmN[i][0]; f.films[i][1] = (float)filmN[i][1]; }
                static std::vector<float> poses;
                poses.clear();
                for (size_t k = 0; k < items.size(); ++k) {
                    if (flatKindOf[k] < 0) continue;
                    const BeamPose& bp = beamPose[k];
                    const float row[16] = {(float)bp.R[0].x, (float)bp.R[0].y, (float)bp.R[0].z, (float)bp.s, (float)bp.R[1].x, (float)bp.R[1].y, (float)bp.R[1].z, (float)flatKindOf[k],
                                           (float)bp.R[2].x, (float)bp.R[2].y, (float)bp.R[2].z, 0.f,          (float)bp.at.x,   (float)bp.at.y,   (float)bp.at.z,   0.f};
                    poses.insert(poses.end(), row, row + 16);
                }
                cSolids = (uint32_t)(poses.size() / 16);
                cStepsAll = (uint32_t)bsteps.size() + 1;
                cbeam::send(cRun, f, poses.data(), cSolids);
            }
#endif
            // SPEC=1 (with BEAMS=1): the colours spread as Sikachev et al. 2008 do it (beam_spectrum.h): one walk per
            // stretch of the spectrum, every corner kept at three colours, in-between polygons drawn. SPECSTEP: how far
            // apart on the floor the in-between polygons lie; SPECCOPIES: at most so many. SPECSTAT=1 counts.
            // SPECWALK=n: a walk covers at most n of the fine bands (1: every band on its own = the exact light).
            // SPECGAIN: the coloured light that many times brighter than it is (for show).
            if (pairsOn) {
                static math::float4 px[64], py[64], pz[64], pa[64], st[64];
                size_t q = 0;
                for (size_t k = 0; k < items.size() && q < 64; ++k) {
                    if (flatKindOf[k] < 0) continue;
                    const BeamPose& bp = beamPose[k];
                    px[q] = {(float)bp.R[0].x, (float)bp.R[0].y, (float)bp.R[0].z, (float)bp.s};
                    py[q] = {(float)bp.R[1].x, (float)bp.R[1].y, (float)bp.R[1].z, 0.f};
                    pz[q] = {(float)bp.R[2].x, (float)bp.R[2].y, (float)bp.R[2].z, 0.f};
                    pa[q] = {(float)bp.at.x, (float)bp.at.y, (float)bp.at.z, 0.f};
                    ++q;
                }
                const std::vector<beam::BallStep> ps = beam::ball_steps(par, pairSteps);
                for (size_t i = 0; i < ps.size() && i < 64; ++i) st[i] = {(float)ps[i].rgb.x, (float)ps[i].rgb.y, (float)ps[i].rgb.z, (float)ps[i].n};
                const beam::V3 sn = normalize(par.sun);
                pairMi->setParameter("poseX", px, 64);
                pairMi->setParameter("poseY", py, 64);
                pairMi->setParameter("poseZ", pz, 64);
                pairMi->setParameter("poseAt", pa, 64);
                pairMi->setParameter("steps", st, 64);
                pairMi->setParameter("sunDir", math::float3{(float)sn.x, (float)sn.y, (float)sn.z});
                pairMi->setParameter("strip", math::float4{(float)band.ux, (float)band.uz, (float)band.c, (float)band.half});
                pairMi->setParameter("stripEvery", (float)band.every);
            }
            static const bool spec = env_f("SPEC", 0.f) > 0.5f && !cBeams && !pairsOn;
            static std::vector<beam::SpecStats> specStats;
            if (spec) {
                static const std::vector<beam::Band> fine = beam::make_bands(par.bands, par.nmMin, par.nmMax);
                const double step = env_f("SPECSTEP", 0.01f);
                const int copies = (int)env_f("SPECCOPIES", 48.f);
                specStats.assign(items.size(), beam::SpecStats{});
                beam::parallel_for(items.size(), threads, [&](size_t k) {
                    each[k].clear();
                    if (flatKindOf[k] < 0) return;
                    beam::split_light_spectrum(beamShapes[(size_t)flatKindOf[k]].data(), beamPose[k].R, beamPose[k].s, beamPose[k].at, par.sun, fine, par.glass, filmN, band, 0.0, least,
                                               mostFaces, step, copies, (int)env_f("SPECWALK", 0.f), (int)std::min<size_t>(4, items[k].local.films.size()), env_f("SPECGAIN", 1.f), true, each[k], &specStats[k]);
                });
            } else
            if (pairsOn) {
                for (std::vector<float>& e : each) e.clear();   // (the GPU draws it: nothing from the CPU)
            } else
            if (!cBeams || cCheck) beam::parallel_for(items.size(), threads, [&](size_t k) {
                each[k].clear();
                if (flatKindOf[k] < 0) return;
                beam::split_light(beamShapes[(size_t)flatKindOf[k]].data(), beamPose[k].R, beamPose[k].s, beamPose[k].at, par.sun, bsteps, filmN, band, 0.0, least, mostFaces, true, each[k], &stats[k]);
            });
            if (!cBeams) for (const std::vector<float>& e : each) tri.insert(tri.end(), e.begin(), e.end());
#ifdef GLASS_COMPUTE_DOOR
            if (cCheck) cCpuTris = each;
#endif
            static double msSum = 0;
            static int frames = 0;
            msSum += std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - b0).count();
            if (spec && std::getenv("SPECSTAT") && (frames == 0 || (frames + 1) % 300 == 0)) {
                beam::SpecStats a;
                for (const beam::SpecStats& q : specStats) {
                    a.walks += q.walks; a.beams += q.beams; a.leaves += q.leaves; a.polygons += q.polygons; a.apart += q.apart; a.light += q.light; a.lightApart += q.lightApart;
                    a.cornerApart += q.cornerApart; a.faceApart += q.faceApart; a.leaveApart += q.leaveApart; a.mostCopies = std::max(a.mostCopies, q.mostCopies);
                }
                size_t corners = 0;
                for (const std::vector<float>& e : each) corners += e.size() / 6;
                std::printf("  SPECSTAT frame %d: %.2f ms a frame on %u threads (mean so far, with this frame), %ld walks, %ld beams followed, %ld pieces left the glass, drawn as %ld polygons (%zu triangles, most copies of one piece %ld)\n",
                            frames + 1, (msSum) / (frames + 1), threads, a.walks, a.beams, a.leaves, a.polygons, corners / 3, a.mostCopies);
                std::printf("    where an end colour would have gone another way than the middle one: %ld of %ld pieces (%.2f%%), carrying %.2f%% of the light; first cause: other corners kept by a cut %ld, face not ahead %ld, mirrored against let out %ld\n",
                            a.apart, a.leaves, 100.0 * (double)a.apart / std::max(1L, a.leaves), 100.0 * a.lightApart / std::max(a.light, 1e-30), a.cornerApart, a.faceApart, a.leaveApart);
            }
            if (std::getenv("BEAMSTAT") && (++frames == 1 || frames % 300 == 0)) {
                beam::SplitStats all;
                for (const beam::SplitStats& st : stats) {
                    all.beams += st.beams; all.leaves += st.leaves; all.corners += st.corners;
                    all.deepest = std::max(all.deepest, st.deepest);
                    for (int i = 0; i < 12; ++i) all.light[i] += st.light[i];
                    for (int i = 0; i < 20; ++i) { all.byCorners[i] += st.byCorners[i]; all.leavesCorners[i] += st.leavesCorners[i]; }
                    all.cuts += st.cuts; all.cutsBefore += st.cutsBefore; all.steps += st.steps;
                    all.mostLeaves = std::max(all.mostLeaves, st.mostLeaves);
                    all.mostCuts = std::max(all.mostCuts, st.mostCuts);
                }
                static long everLeaves = 0, everCuts = 0, everCorners = 0;
                everLeaves = std::max(everLeaves, all.mostLeaves);
                everCuts = std::max(everCuts, all.mostCuts);
                for (int i = 3; i < 20; ++i)
                    if (all.leavesCorners[i] > 0) everCorners = std::max(everCorners, (long)i);
                double sum = 0;
                for (double l : all.light) sum += l;
                std::printf("  BEAMSTAT frame %d: %.2f ms a frame on %u threads (mean so far), %ld beams followed, %ld patches drawn (%ld corners), deepest %d steps\n", frames,
                            msSum / frames, threads, all.beams, all.leaves, all.corners, all.deepest);
                std::printf("    light by how many faces it met inside:");
                for (int i = 1; i < 12; ++i)
                    if (all.light[i] > 0) std::printf("  %d: %.1f%%", i, 100.0 * all.light[i] / sum);
                {
                    // what a version with no table in between would have to do: a place for every patch a colour
                    // step may give, a fan of triangles for the most corners a patch may have, and every corner of
                    // every place working the splitting out again, as far as its own patch (to the end if its place
                    // is empty)
                    const long places = everLeaves, tris = std::max(1L, everCorners - 2), perPlace = 3 * tris;
                    const double runs = (double)all.steps * (double)places * (double)perPlace;
                    const double filled = all.cutsBefore * (double)perPlace;
                    const double empty = ((double)all.steps * (double)places - (double)all.leaves) * ((double)all.cuts / std::max(1L, all.steps)) * (double)perPlace;
                    std::printf("\n    cuts made here (once each): %ld in %ld colour steps (most in one: %ld); most patches from one colour step so far: %ld, most corners: %ld", all.cuts,
                                all.steps, everCuts, everLeaves, everCorners);
                    std::printf("\n    with nothing kept in between: %ld places x %ld corners of triangles = %.0f runs of the vertex stage, making %.3g cuts (%.0f times as many)", places,
                                perPlace, runs, filled + empty, (filled + empty) / std::max(1.0, (double)all.cuts));
                }
                std::printf("\n    pieces by number of corners:");
                for (int i = 3; i < 20; ++i)
                    if (all.byCorners[i] > 0) std::printf("  %d: %ld", i, all.byCorners[i]);
                std::printf("\n");
            } else if (!std::getenv("BEAMSTAT")) ++frames;
        }
        outline.clear();
        for (const beam::Solid& so : world.solids)  // (on the CPU only to compare: solids with dents and holes have their own switch)
            if (so.kind == beam::Solid::EXTRUDE ? exOnCpu : outlineOnCpu) beam::silhouette_triangles(so, par.sun, 0.0, outline);
        for (OutlineKind& ok : outlineKinds) ok.mi->setParameter("sunDir", sunDir);
        traceMs = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();

        // the cones: where they are and how they stand to the sun into their table, and as many instances switched on
        {
            std::vector<int> balls, cones;
            for (int i : alike) {
                const int kind = world.solids[(size_t)i].kind;
                if (kind == beam::Solid::EXTRUDE) {
                    if (exOnCpu) beam::extrude_light(world.solids[(size_t)i], par, tri, nullptr, -1);
                    continue;
                }
                if (kind == beam::Solid::CONE || kind == beam::Solid::ROD) cones.push_back(i);
                else if (kind == beam::Solid::BALL) balls.push_back(i);
            }
            {
                // the flat-faced solids: their colour steps (the pose table is already on its way)
                if (rainbowOnce) {
                    // the two ends of the spectrum and its middle, and the spectrum summed up from the red end to the
                    // blue in 32 equal steps of the index of refraction (all of it: white)
                    const double nRed = par.glass.n(par.nmMax), nBlue = par.glass.n(par.nmMin);
                    const std::vector<beam::Band> fine = beam::make_bands(2048, par.nmMin, par.nmMax);
                    float4 rainbow[32] = {};
                    for (const beam::Band& b : fine) {
                        const double sAt = (par.glass.n(b.nm) - nRed) / (nBlue - nRed) * 31.0;
                        for (int i = 0; i < 32; ++i)
                            if ((double)i >= sAt) rainbow[i] += float4{(float)b.rgb.x, (float)b.rgb.y, (float)b.rgb.z, 0.f};
                    }
                    FLAT_WALK_SET("ends", float4{(float)(0.5 * (nRed + nBlue)), (float)nRed, (float)nBlue, 0.f});
                    FLAT_WALK_SET("sunDir", sunDir);
                    for (FlatKind& fk : flatKinds) fk.mi->setParameter("rainbow", rainbow, 32);
                } else {
                    const std::vector<beam::BallStep> fsteps = beam::ball_steps(par, flatSteps - 1);
                    float4 table[16] = {};
                    for (int i = 0; i + 1 < flatSteps && i < (int)fsteps.size(); ++i)
                        table[i] = float4{(float)fsteps[(size_t)i].rgb.x, (float)fsteps[(size_t)i].rgb.y, (float)fsteps[(size_t)i].rgb.z, (float)fsteps[(size_t)i].n};
                    table[flatSteps - 1] = table[(flatSteps - 1) / 2];
                    FLAT_WALK_SET("steps", table, 16);
                    FLAT_WALK_SET("sunDir", sunDir);
                    {   // which colours each film mirrors, as indices of refraction (red has the lowest)
                        float4 band[4] = {};
                        for (int f = 0; f < 4; ++f) band[f] = float4{(float)par.glass.n(filmNm[f].y), (float)par.glass.n(filmNm[f].x), 0.f, 0.f};
                        FLAT_WALK_SET("filmBand", band, 4);
                        for (MaterialInstance* mi : routeMis) mi->setParameter("filmBand", band, 4);
                    }
                    for (FlatKind& fk : flatKinds) {
                        fk.mi->setParameter("steps", table, 16);
                        if (fk.mi2) fk.mi2->setParameter("steps", table, 16);
                    }
                    for (MaterialInstance* mi : routeMis) {
                        mi->setParameter("steps", table, 16);
                        mi->setParameter("sunDir", sunDir);
                    }
                }
            }
            alike.swap(balls);
            // the cones first, then the rods (as their two renderables expect)
            std::stable_sort(cones.begin(), cones.end(), [&](int a, int b) { return world.solids[(size_t)a].kind == beam::Solid::CONE && world.solids[(size_t)b].kind != beam::Solid::CONE; });
            if (ballsOnCpu) {  // (then trace_cheap has already written everything, the cones' light too)
                cones.clear();
            } else if (conesOnCpu) {
                for (int i : cones) beam::cone_light(world.solids[(size_t)i], par, tri);
                cones.clear();
            }
            conesDrawn = std::getenv("NOLIGHT") ? 0 : std::min(cones.size(), coneRoom);
            const size_t CONE_TABLE = std::max<size_t>(1, coneRoom);
            if (conesDrawn) {
                auto* ct = new std::vector<float>((size_t)CONE_TABLE * 4 * 4, 0.f);
                for (size_t i = 0; i < conesDrawn; ++i) {
                    const beam::Taper so = beam::taper_of(world.solids[(size_t)cones[i]]);
                    const beam::ConeFrame cf = beam::cone_frame(so, par.sun);
                    float* a = ct->data() + i * 4;
                    float* b = a + (size_t)CONE_TABLE * 4;
                    float* c = b + (size_t)CONE_TABLE * 4;
                    c[(size_t)CONE_TABLE * 4] = (float)so.rt;
                    a[0] = (float)so.c.x; a[1] = (float)so.c.y; a[2] = (float)so.c.z; a[3] = (float)so.r;
                    b[0] = (float)so.ax.x; b[1] = (float)so.ax.y; b[2] = (float)so.ax.z; b[3] = (float)so.h;
                    c[0] = (float)cf.e1.x; c[1] = (float)cf.e1.y; c[2] = (float)cf.e1.z; c[3] = (float)cf.half;
                }
                coneTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(ct->data(), ct->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                             [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, ct));
            }
            const std::vector<beam::BallStep> steps = beam::ball_steps(par, par.coneSteps);
            float4 stepTable[32] = {};
            for (int i = 0; i < coneStepCount && i < (int)steps.size(); ++i)
                stepTable[i] = float4{(float)steps[(size_t)i].rgb.x, (float)steps[(size_t)i].rgb.y, (float)steps[(size_t)i].rgb.z, (float)steps[(size_t)i].n};
            for (int b = 0; b < CONE_SETS; ++b) {
                const bool on = conesDrawn > 0 && (b == 0 ? conesWithTip : coneRoom - conesWithTip) > 0;
                rm.setLayerMask(rm.getInstance(coneE[b]), 0xFF, on ? 0x01 : 0x00);
                if (!on) continue;
                coneMi[b]->setParameter("steps", stepTable, 32);
                coneMi[b]->setParameter("sunDir", sunDir);
            }
            coneWalkMi->setParameter("steps", stepTable, 32);
            coneWalkMi->setParameter("sunDir", sunDir);
        }
        // the solids with dents and holes: where each one is, into their table; the sun and the colour steps
        if (exCount) {
            auto* pt = new std::vector<float>(exRoom * 3 * 4, 0.f);
            for (size_t k = 0; k < items.size(); ++k)
                if (items[k].exColumn >= 0) {
                    float* a = pt->data() + (size_t)items[k].exColumn * 4;
                    beam::ex_pose(world.solids[k], 0, a, a + exRoom * 4, a + 2 * exRoom * 4);
                    for (const ExShapeUse& sh : exShapes)
                        if (sh.ol == world.solids[k].shape.get()) a[exRoom * 4 + 3] = (float)sh.line;
                }
            exPoseTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(pt->data(), pt->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                           [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, pt));
            const std::vector<beam::BallStep> ws = beam::ball_steps(par, beam::EXT_STEPS), cs = beam::ball_steps(par, beam::EXT_CAP_STEPS);
            float4 wall[8] = {}, cap[4] = {};
            for (size_t i = 0; i < 8; ++i) {
                const beam::BallStep& b = ws[std::min(i, ws.size() - 1)];
                wall[i] = i < ws.size() ? float4{(float)b.rgb.x, (float)b.rgb.y, (float)b.rgb.z, (float)b.n} : float4{0, 0, 0, (float)b.n};
            }
            for (size_t i = 0; i < 4; ++i) {
                const beam::BallStep& b = cs[std::min(i, cs.size() - 1)];
                cap[i] = i < cs.size() ? float4{(float)b.rgb.x, (float)b.rgb.y, (float)b.rgb.z, (float)b.n} : float4{0, 0, 0, (float)b.n};
            }
            for (MaterialInstance* mi : exMis) {
                mi->setParameter("steps", wall, 8);
                mi->setParameter("capSteps", cap, 4);
                mi->setParameter("sunDir", sunDir);
            }
            for (MaterialInstance* mi : streakMis) {
                mi->setParameter("steps", wall, 8);
                mi->setParameter("sunDir", sunDir);
            }
            for (MaterialInstance* mi : fillMis) mi->setParameter("sunDir", sunDir);
            const bool on = !exOnCpu && !exNew && !std::getenv("NOLIGHT");
            for (size_t i = 0; i < exEntities.size(); ++i) rm.setLayerMask(rm.getInstance(exEntities[i]), 0xFF, on && (exEntityBit[i] & exShow) ? 0x01 : 0x00);
        }
        // the balls: middle and radius into their table, and as many instances switched on
        ballsAlike = std::min<size_t>(alike.size(), (size_t)BALL_TABLE - 1);
        if (std::getenv("NOLIGHT")) ballsAlike = 0;
        if (ballsAlike) {
            auto* bt = new std::vector<float>((size_t)BALL_TABLE * 4, 0.f);
            for (size_t i = 0; i < ballsAlike; ++i) {
                const beam::Solid& so = world.solids[(size_t)alike[i]];
                float* o = bt->data() + i * 4;
                o[0] = (float)so.c.x; o[1] = (float)so.c.y; o[2] = (float)so.c.z; o[3] = (float)so.r;
            }
            ballTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(bt->data(), bt->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                         [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, bt));
        }
        {
            const std::vector<beam::BallStep> steps = beam::ball_steps(par);
            struct { beam::V3 w, e1, e2; } bf;
            beam::sun_frame(par.sun, bf.w, bf.e1, bf.e2);
            float4 stepTable[12] = {};
            for (int i = 0; i < ballStepCount; ++i)
                stepTable[i] = float4{(float)steps[(size_t)i].rgb.x, (float)steps[(size_t)i].rgb.y, (float)steps[(size_t)i].rgb.z, (float)steps[(size_t)i].n};
            size_t first = 0;
            for (int b = 0; b < BALL_SETS; ++b) {
                const bool on = (ballsAlike >> b) & 1u;
                rm.setLayerMask(rm.getInstance(ballE[b]), 0xFF, on ? 0x01 : 0x00);
                if (!on) continue;
                ballMi[b]->setParameter("steps", stepTable, 12);
                ballMi[b]->setParameter("first", (float)first);
                ballMi[b]->setParameter("sunTo", float3{(float)bf.w.x, (float)bf.w.y, (float)bf.w.z});
                ballMi[b]->setParameter("e1", float3{(float)bf.e1.x, (float)bf.e1.y, (float)bf.e1.z});
                ballMi[b]->setParameter("e2", float3{(float)bf.e2.x, (float)bf.e2.y, (float)bf.e2.z});
                first += (size_t)1 << b;
            }
        }

        // the table of faces for glass seen through glass
        auto* table = new std::vector<float>((size_t)SCENE_W * SCENE_H * 4, 0.f);
