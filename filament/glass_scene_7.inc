// glass_scene.mm, piece 7 (cut there by size, as it was): read only in its place in glass_scene.mm.
        for (size_t k = 0; k < items.size(); ++k) {
            float* row = table->data() + k * SCENE_W * 4;
            row[0] = (float)world.centre[k].x; row[1] = (float)world.centre[k].y; row[2] = (float)world.centre[k].z; row[3] = (float)world.radius[k];
            if (k == 0 && std::getenv("STRIP")) {
                // STRIP="half width [degrees round [off to the side]]": the sun comes through a slit and lights a band of the floor
                // only, which passes under the first solid (beam_soften.mat, flat_walk.mat)
                float half = 0.3f, deg = 35.f, side = 0.f, every = 0.f;   // (every: a blind, the band again and again this far apart)
                std::sscanf(std::getenv("STRIP"), "%f %f %f %f", &half, &deg, &side, &every);
                const float a = deg * 3.14159265f / 180.f;
                const float3 down = float3{(float)world.centre[0].x, (float)world.centre[0].y, (float)world.centre[0].z} + sunDir * ((float)world.centre[0].y / -sunDir.y);
                const float4 strip{std::cos(a), std::sin(a), std::cos(a) * down.x + std::sin(a) * down.z + side, half};
                softenMi->setParameter("strip", strip);
                softenMi->setParameter("stripSoft", float2{env_f("STRIPSOFT", 0.02f), every});
                FLAT_WALK_SET("strip", strip);
                FLAT_WALK_SET("stripSoft", float2{env_f("STRIPSOFT", 0.02f), every});
                for (MaterialInstance* mi : routeMis) {
                    mi->setParameter("strip", strip);
                    mi->setParameter("stripSoft", float2{env_f("STRIPSOFT", 0.02f), every});
                }
            }
            int count = 0;
            row[5] = (float)world.solids[k].kind; row[6] = (float)world.solids[k].r; row[7] = (float)world.solids[k].h;
            row[8] = (float)world.solids[k].ax.x; row[9] = (float)world.solids[k].ax.y; row[10] = (float)world.solids[k].ax.z;
            if (world.solids[k].kind == beam::Solid::CONE) {  // the middle of its base
                row[12] = (float)world.solids[k].c.x; row[13] = (float)world.solids[k].c.y; row[14] = (float)world.solids[k].c.z;
            }
            if (world.solids[k].kind == beam::Solid::EXTRUDE) {  // where its outline's origin is and its size; where its x axis points and the line of its shape
                const beam::Solid& so = world.solids[k];
                row[12] = (float)so.c.x; row[13] = (float)so.c.y; row[14] = (float)so.c.z; row[15] = (float)so.sc;
                row[16] = (float)so.ux.x; row[17] = (float)so.ux.y; row[18] = (float)so.ux.z;
                for (const ExShapeUse& sh : exShapes)
                    if (sh.ol == so.shape.get()) row[19] = (float)sh.line;
            }
            for (size_t f = 0; f < world.solids[k].planes.size() && count < SCENE_W - 3; ++f) {
                if (world.solids[k].faces[f].n < 3) continue;
                float* o = row + (3 + count) * 4;
                o[0] = (float)world.solids[k].planes[f].n.x; o[1] = (float)world.solids[k].planes[f].n.y; o[2] = (float)world.solids[k].planes[f].n.z;
                o[3] = (float)world.solids[k].planes[f].o;
                ++count;
            }
            row[4] = (float)count;
            if (world.solids[k].kind == beam::Solid::FLAT) {   // its colour films, after its faces
                int films = 0;
                for (const beam::Plane& fl : world.solids[k].films) {
                    if (films >= 4 || 3 + count + films >= SCENE_W) break;
                    float* o = row + (3 + count + films) * 4;
                    o[0] = (float)fl.n.x; o[1] = (float)fl.n.y; o[2] = (float)fl.n.z; o[3] = (float)fl.o;
                    ++films;
                }
                row[6] = (float)films;
            }
        }
        {
            float4 spheres[64] = {};
            for (size_t k = 0; k < items.size() && k < 64; ++k)
                spheres[k] = float4{(float)world.centre[k].x, (float)world.centre[k].y, (float)world.centre[k].z, (float)world.radius[k]};
            FLAT_WALK_SET("spheres", spheres, 64);
            if (nodesMi) nodesMi->setParameter("spheres", spheres, 64);
            for (Item& it : items) it.glass->setParameter("spheres", spheres, 64);
        }
        sceneTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(table->data(), table->size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT,
                                                                      [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, table));

        const size_t TS = beam::TRI;
        size_t nLight = tri.size() / TS, nOut = outline.size() / TS;
        if (nLight + nOut > MAX_LIGHT_VERTS) nLight = (MAX_LIGHT_VERTS - nOut) / 3 * 3;  // never in practice
        lightTris = nLight / 3;
        const size_t nv = nLight + nOut;
        auto* buf = new std::vector<float>(std::max<size_t>(nv, 3) * 7, 0.f);
        const bool off = std::getenv("NOLIGHT") != nullptr;
        for (size_t i = 0; i < nv; ++i) {
            const float* src = i < nLight ? tri.data() + i * TS : outline.data() + (i - nLight) * TS;
            float* o = buf->data() + i * 7;
            o[0] = src[0];
            o[1] = 0.f;
            o[2] = src[1];
            o[3] = (off && i < nLight) ? 0.f : src[2];
            o[4] = (off && i < nLight) ? 0.f : src[3];
            o[5] = (off && i < nLight) ? 0.f : src[4];
            o[6] = src[5];  // distance flown, for the softening pass
        }
        lightVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(buf->data(), std::max<size_t>(nv, 3) * 7 * sizeof(float),
                                                                         [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, buf));
        rm.setGeometryAt(rm.getInstance(lightE), 0, RenderableManager::PrimitiveType::TRIANGLES, lightVB, lightIB, 0, std::max<size_t>(nLight, 3));
        rm.setLayerMask(rm.getInstance(lightE), 0xFF, nLight ? 0x01 : 0x00);  // no CPU light at all (only balls alike): the buffer starts with the outline, do not draw it as light
        rm.setGeometryAt(rm.getInstance(shadeE), 0, RenderableManager::PrimitiveType::TRIANGLES, lightVB, lightIB, nOut ? nLight : 0, std::max<size_t>(nOut, 3));
        rm.setLayerMask(rm.getInstance(shadeE), 0xFF, nOut ? 0x01 : 0x00);
        for (int l = 0; stageOn && l < (blocksOn ? 2 : 1); ++l) {  // the stage draws the same buffer, once per receiver
            rm.setGeometryAt(rm.getInstance(stageLightE[l]), 0, RenderableManager::PrimitiveType::TRIANGLES, lightVB, lightIB, 0, std::max<size_t>(nLight, 3));
            rm.setLayerMask(rm.getInstance(stageLightE[l]), 0xFF, nLight ? 0x01 : 0x00);
            rm.setGeometryAt(rm.getInstance(stageShadeE[l]), 0, RenderableManager::PrimitiveType::TRIANGLES, lightVB, lightIB, nOut ? nLight : 0, std::max<size_t>(nOut, 3));
            rm.setLayerMask(rm.getInstance(stageShadeE[l]), 0xFF, nOut ? 0x01 : 0x00);
        }
    };

#ifdef GLASS_COMPUTE_DOOR
    // CBEAMCHECK (tests): the GPU's triangles read back and set against the CPU's of the same frame. A triangle is
    // "the same" when its three corners (in order), its light and its distance flown agree within CBEAMTOL.
    auto cbeam_check = [&]() {
        static int frame = 0;
        if (++frame != 1 && frame % 300 != 0) return;
        cbeam::Counters c{};
        door::read(engine, cRun->counters, 0, sizeof(c), &c);
        const uint32_t nv = std::min(c.verts, cRun->vertCap);
        std::vector<float> g((size_t)std::max<uint32_t>(nv, 1) * 7);
        if (nv) door::read(engine, cRun->verts, 0, nv * 28, g.data());
        long beams = 0;
        std::printf("  CBEAMCHECK frame %d: beams by level:", frame);
        for (int i = 0; i < 32; ++i) { beams += c.level[i]; if (c.level[i]) std::printf(" %u", c.level[i]); }
        std::printf("\n    GPU: %ld beams followed, %u triangles; found no room: %u beams, %u vertices, %u corners, %u bands\n", beams, nv / 3, c.lost[0], c.lost[1], c.lost[2], c.lost[3]);
        struct T { float v[3][6]; double light; bool used; };   // per corner: x, z, r, g, b, travel
        auto light_of = [](const T& t) {
            const double a = 0.5 * std::fabs((double)(t.v[1][0] - t.v[0][0]) * (t.v[2][1] - t.v[0][1]) - (double)(t.v[2][0] - t.v[0][0]) * (t.v[1][1] - t.v[0][1]));
            return a * ((double)t.v[0][2] + t.v[0][3] + t.v[0][4]) / 3.0;
        };
        std::vector<T> cpu, gpu;
        for (const std::vector<float>& e : cCpuTris)
            for (size_t i = 0; i + 18 <= e.size(); i += 18) {
                T t{};
                std::memcpy(t.v, e.data() + i, sizeof(t.v));
                t.light = light_of(t);
                cpu.push_back(t);
            }
        for (size_t i = 0; i + 21 <= (size_t)nv * 7; i += 21) {
            T t{};
            for (int k = 0; k < 3; ++k) {
                const float* q = g.data() + i + (size_t)k * 7;
                const float row[6] = {q[0], q[2], q[3], q[4], q[5], q[6]};
                std::memcpy(t.v[k], row, sizeof(row));
            }
            t.light = light_of(t);
            gpu.push_back(t);
        }
        const double tol = env_f("CBEAMTOL", 1e-3f), cellSize = 0.05;
        std::unordered_map<long long, std::vector<size_t>> grid;
        auto key = [&](float x, float z) { return ((long long)std::floor(x / cellSize) << 32) ^ (long long)(uint32_t)(int)std::floor(z / cellSize); };
        for (size_t i = 0; i < gpu.size(); ++i) grid[key(gpu[i].v[0][0], gpu[i].v[0][1])].push_back(i);
        double cpuLight = 0, gpuLight = 0, lostLight = 0, worst = 0;
        size_t matched = 0;
        for (const T& t : cpu) {
            cpuLight += t.light;
            double best = 1e30;
            size_t bi = (size_t)-1;
            for (int dx = -1; dx <= 1; ++dx)
                for (int dz = -1; dz <= 1; ++dz) {
                    auto it = grid.find(key(t.v[0][0] + (float)(dx * cellSize), t.v[0][1] + (float)(dz * cellSize)));
                    if (it == grid.end()) continue;
                    for (size_t i : it->second) {
                        if (gpu[i].used) continue;
                        double d = 0;
                        for (int k = 0; k < 3; ++k)
                            for (int q = 0; q < 6; ++q) d = std::max(d, std::fabs((double)gpu[i].v[k][q] - t.v[k][q]) / (q == 5 ? std::max(1.0, std::fabs((double)t.v[k][q])) : 1.0));
                        if (d < best) { best = d; bi = i; }
                    }
                }
            if (bi != (size_t)-1 && best <= tol) { gpu[bi].used = true; ++matched; worst = std::max(worst, best); }
            else lostLight += t.light;
        }
        double extraLight = 0;
        for (const T& t : gpu) { gpuLight += t.light; if (!t.used) extraLight += t.light; }
        std::printf("    CPU: %zu triangles. The same on both: %zu (%.3f%%), furthest apart among them %.2e; light only the CPU has %.4f%%, only the GPU has %.4f%%; all light GPU / CPU = %.6f\n",
                    cpu.size(), matched, cpu.empty() ? 0.0 : 100.0 * (double)matched / (double)cpu.size(), worst, 100.0 * lostLight / std::max(cpuLight, 1e-30),
                    100.0 * extraLight / std::max(cpuLight, 1e-30), gpuLight / std::max(cpuLight, 1e-30));
    };
#endif
    auto draw_light = [&]() {
#ifdef GLASS_COMPUTE_DOOR
        if (cBeams && !std::getenv("XCRUN")) {   // (XCRUN: timing only, the compute passes left out)
            if (cCheck) cbeam::run(cRun, cSolids, cStepsAll, cbeam_check);
            else cbeam::run(cRun, cSolids, cStepsAll);
        }
#endif
        if (vbodyOn && !std::getenv("VNODESOFF")) {
            renderer->render(vNodes);
            renderer->render(vCells);
        }
        // XWALK (timing only): 1 leaves out every table pass, or any of the letters f (flat), e (dents and holes: find),
        // w (dents and holes: walk), c (cones and rods)
        static const char* xw = std::getenv("XWALK");
        static const bool xF = xw && (std::strchr(xw, '1') || std::strchr(xw, 'f')), xE = xw && (std::strchr(xw, '1') || std::strchr(xw, 'e')),
                          xW = xw && (std::strchr(xw, '1') || std::strchr(xw, 'w')), xC = xw && (std::strchr(xw, '1') || std::strchr(xw, 'c'));
        if (flatGBeams) {
            for (MaterialInstance* mi : gbmCommit) mi->commit(*engine);   // (MaterialInstance.h: post-process instances are committed by hand)
            for (View* v : gbeamViews) renderer->render(v);
        } else if (routeTree) {
            for (const RouteLevel& rl : routeLevels) renderer->render(rl.view);
        } else if (!xF && !flatOnCpu && !flatBeams && !flatKinds.empty()) {
            renderer->render(vFlatWalk);
            if (flatClip && !std::getenv("XWALK2")) renderer->render(vFlatWalk2);   // (XWALK2: timing only, the second turn left out)
        }
        if (exCount && !exOnCpu && !exNew) {
            if (!xE) renderer->render(vExFind);
            if (!xW) renderer->render(vExWalk);
        }
        if (!xC && conesDrawn) renderer->render(vConeWalk);
        static const bool xLight = std::getenv("XLIGHT"), xShade = std::getenv("XSHADE"), xMips = std::getenv("XMIPS"), xSoft = std::getenv("XSOFT");  // timing only
        if (!xLight) renderer->render(vLight);
        if (!xShade) renderer->render(vShade);
        if (!xMips) {
            light.color->generateMipmaps(*engine);
            shade.color->generateMipmaps(*engine);
        }
        if (!xSoft) renderer->render(vSoft);
    };
    auto open = [&]() {
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) return true;
        return false;
    };
    std::vector<uint8_t> frame((size_t)W * H * 4);
    auto grab = [&](const char* path) -> bool {
        if (!open()) return false;
        draw_light();
        if (!noBackdrop) renderer->render(vBackdrop);
        renderer->render(vBeauty);
        renderer->readPixels(beauty.rt, 0, 0, W, H,
                             backend::PixelBufferDescriptor(frame.data(), frame.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
        // VDUMP=file (to count ways): the table of glass_nodes.mat as it is, 289 * 3 x (solids * 6) texels of four floats
        static std::vector<float> nodeDump;
        if (vbodyOn && std::getenv("VDUMP")) {
            nodeDump.assign((size_t)289 * items.size() * 6 * 4, 0.f);
            renderer->readPixels(nodesRT.rt, 0, 0, 289, (uint32_t)items.size() * 6,
                                 backend::PixelBufferDescriptor(nodeDump.data(), nodeDump.size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
        }
        // ROUTESTAT=1: how many leaves each level of route_tree.mat holds, and how much light (read back only to count)
        static std::vector<std::vector<float>> routeDump;
        if (routeTree && std::getenv("ROUTESTAT")) {
            routeDump.resize(routeLevels.size());
            for (size_t i = 0; i < routeLevels.size(); ++i) {
                const RouteLevel& rl = routeLevels[i];
                routeDump[i].assign((size_t)rl.slots * 2 * (size_t)rl.rows * 4, 0.f);
                renderer->readPixels(rl.table.rt, 0, 0, (uint32_t)(rl.slots * 2), (uint32_t)rl.rows,
                                     backend::PixelBufferDescriptor(routeDump[i].data(), routeDump[i].size() * sizeof(float), backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
            }
        }
        renderer->endFrame();
        engine->flushAndWait();
        if (!routeDump.empty()) {
            double all = 0;
            std::vector<double> light(routeDump.size(), 0.0);
            std::vector<long> leaves(routeDump.size(), 0), inner(routeDump.size(), 0), full(routeDump.size(), 0);
            for (size_t i = 0; i < routeDump.size(); ++i) {
                const RouteLevel& rl = routeLevels[i];
                const int fan = i == 0 ? rl.slots : rl.slots / routeLevels[i - 1].slots;
                for (int r = 0; r < rl.rows; ++r)
                    for (int sl = 0; sl < rl.slots; ++sl) {
                        const float* t = routeDump[i].data() + ((size_t)r * (size_t)rl.slots * 2 + (size_t)sl * 2) * 4;
                        if (t[2] > 1.5f) { ++leaves[i]; light[i] += t[3] * (double)((t[6] - t[4]) * (t[7] - t[5])); }
                        else if (t[2] > 0.5f) ++inner[i];
                        if (sl % fan == fan - 1 && t[2] > 0.5f) ++full[i];   // (the last place of its parent is taken: more may have been dropped)
                    }
                all += light[i];
            }
            std::printf("  ROUTESTAT (all solids, all colour steps; light = brightness x area of the box, a rough measure):\n");
            for (size_t i = 0; i < routeDump.size(); ++i)
                std::printf("    level %zu: %6ld leaves, %6ld go on inside, %5.1f%% of the light, %ld parents with every place taken\n", i + 1, leaves[i], inner[i],
                            all > 0 ? 100.0 * light[i] / all : 0.0, full[i]);
        }
        if (!nodeDump.empty())
            if (FILE* f = std::fopen(std::getenv("VDUMP"), "wb")) {
                std::fwrite(nodeDump.data(), sizeof(float), nodeDump.size(), f);
                std::fclose(f);
            }
        return write_png(path, W, H, frame.data());
    };
#ifdef GLASS_LIB
    // ---- live, for a host that shows the picture itself (glass_live.h): every frame is drawn straight into a buffer
    // the host can show without a copy. What those buffers are is the one thing that differs per OS (glass_door.h).
    // Nothing is waited for: only when the GPU has finished a frame is its buffer handed out. Three buffers; a frame is
    // drawn into one that is neither handed out nor still being drawn, or skipped.
    // The sun, the camera and the pace come from the host. Stopping is a pause: the thread and the engine stay.
    {
        struct Slot {
            std::atomic<int> busy{0};
            std::chrono::steady_clock::time_point begun;
        };
        static Slot slot[GLASS_DOOR_BUFFERS];
        if (!glass_door_open(engine, W, H, [](int i) {
                g_live.frameMs = std::chrono::duration<float, std::milli>(std::chrono::steady_clock::now() - slot[i].begun).count();
                g_live.front = i;
                g_live.frames.fetch_add(1);
                slot[i].busy = 0;
            }))
            return fail("pixel buffer");
        vBeauty->setRenderTarget(nullptr);   // the picture goes to the swap chain, i.e. into the pixel buffer
        const float focal = (float)(LH / (2.0 * std::tan(cam[6] * 0.5 * 3.14159265 / 180.0)));
        const float3 target{cam[3], cam[4], cam[5]};
        auto last = std::chrono::steady_clock::now();
        auto due = last;   // when the next frame is to start: one sixtieth of a second after the one before, whatever that one took
        float t = env_f("T", 0.f);
        for (;;) {
            auto begin = std::chrono::steady_clock::now();
            if (!g_live.run.load()) {   // paused
                last = due = begin;
                std::this_thread::sleep_for(std::chrono::milliseconds(30));
                continue;
            }
            t += std::chrono::duration<float>(begin - last).count() * g_live.speed.load();
            last = begin;
            int into = -1;
            for (int i = 0; i < GLASS_DOOR_BUFFERS && into < 0; ++i)
                if (!slot[i].busy.load() && g_live.front.load() != i) into = i;
            if (into >= 0) {
                const float sa = g_live.sunAz.load(), se = std::min(1.5f, std::max(0.05f, g_live.sunEl.load()));
                sunDir = float3{std::cos(se) * std::cos(sa), -std::sin(se), std::cos(se) * std::sin(sa)};
                const float ca = g_live.camAz.load(), ce = std::min(1.5f, std::max(0.03f, g_live.camEl.load())), cd = std::min(12.f, std::max(0.5f, g_live.camDist.load()));
                const float3 eye = target + float3{std::cos(ce) * std::sin(ca), std::sin(ce), std::cos(ce) * std::cos(ca)} * cd;
                cam[0] = eye.x; cam[1] = eye.y; cam[2] = eye.z;
                camera->lookAt(eye, target, float3{0, 1, 0});
                softenMi->setParameter("worldFromClip", mat4f(inverse(camera->getProjectionMatrix() * camera->getViewMatrix())));
                softenMi->setParameter("eye", eye);
                softenMi->setParameter("focal", focal);
                auto c0 = std::chrono::steady_clock::now();
                update(t);
                auto c1 = std::chrono::steady_clock::now();
                const bool go = renderer->beginFrame(glass_door_chain(into));
                auto c2 = std::chrono::steady_clock::now();
                if (go) {
                    slot[into].begun = begin;
                    slot[into].busy = 1;
                    draw_light();
                    if (!noBackdrop) renderer->render(vBackdrop);
                    renderer->render(vBeauty);
                    auto c3 = std::chrono::steady_clock::now();
                    renderer->endFrame();
                    auto c4 = std::chrono::steady_clock::now();
                    if (std::getenv("GLASS_TIMING")) {   // where a turn of this loop spends its time (CPU side)
                        static double acc[4] = {};
                        static int count = 0;
                        auto ms = [](auto a, auto b) { return std::chrono::duration<double, std::milli>(b - a).count(); };
                        acc[0] += ms(c0, c1); acc[1] += ms(c1, c2); acc[2] += ms(c2, c3); acc[3] += ms(c3, c4);
                        if (++count == 120) {
                            std::fprintf(stderr, "LIVE cpu per frame: update %.2f  beginFrame %.2f  views %.2f  endFrame %.2f ms\n", acc[0] / 120, acc[1] / 120, acc[2] / 120, acc[3] / 120);
                            count = 0;
                            acc[0] = acc[1] = acc[2] = acc[3] = 0;
                        }
                    }
                } else {
                    g_live.skipped.fetch_add(1);
                }
            } else {
                g_live.skipped.fetch_add(1);
            }
            due += std::chrono::microseconds(16667);
            if (due < begin) due = begin;   // fell behind: do not try to catch up
            std::this_thread::sleep_until(due);
        }
    }
#endif
    std::system("mkdir -p out");
    const float t0 = env_f("T", 0.f);
    update(t0);
    for (int i = 0; i < 3; ++i) {
        if (!open()) return fail("warm");
        draw_light();
        if (!noBackdrop) renderer->render(vBackdrop);
        renderer->render(vBeauty);
        renderer->endFrame();
        engine->flushAndWait();
    }
    const std::string out = "out/glass-scene-" + name + ".png";
    if (!grab(out.c_str())) return fail("grab");
    update(t0);
    std::printf("wrote %s  (%zu solids, chamfer %.3f, dispersion x%.1f, %s, %u threads: light %.2f ms, %zu light triangles)\n", out.c_str(), items.size(), chamfer, disp,
                exact ? "EXACT trace" : "cheap trace", threads, traceMs, lightTris);
    std::printf("      %zu ball(s) drawn from the one shared shape (%zu triangles on the GPU, none from the CPU)\n", ballsAlike, ballsAlike * (size_t)ballStepCount * discI.size() / 3);
    std::printf("      %zu cone(s) drawn from the one shared mesh (%zu triangles on the GPU, none from the CPU)\n", conesDrawn, conesDrawn * (size_t)(coneStepCount * coneWays) * coneI.size() / 3);
    std::printf("      %zu solid(s) with dents and holes, %zu shape(s): %s (%zu triangles on the GPU, most of them folded up)\n", exCount, exShapes.size(),
                exOnCpu ? "light and shadow from the CPU's copy (EXTRUDECPU)" : "light and shadow worked out on the GPU", exOnCpu ? (size_t)0 : exTriangles);

    // ---- a run of frames for a movie: FRAMES=n writes out/seq/0000.png ... at 60 per second of the same motion
    if (std::getenv("FRAMES")) {
        const int n = (int)env_f("FRAMES", 120.f);
        std::system("mkdir -p out/seq && rm -f out/seq/*.png");
        const float3 sun0 = sunDir;
        for (int i = 0; i < n; ++i) {
            const float t = t0 + (float)i / 120.f * 6.2831853f * env_f("SPEED", 0.5f);
            if (!std::getenv("FIXSUN")) sunDir = normalize(float3{sun0.x * std::cos(0.3f * std::sin(t)), sun0.y, sun0.z + 0.4f * std::sin(t)});
            update(t);
            char path[64];
            std::snprintf(path, sizeof(path), "out/seq/%04d.png", i);
            if (!grab(path)) return fail("frame");
        }
        std::printf("wrote %d frames to out/seq\n", n);
        return 0;
    }

    // ---- paced like an app: every tick every solid turns and slides and the sun swings, so all the light is
    // traced again and uploaded again each frame. beginFrame() returning false is Filament saying it is behind.
    // ---- BENCH=n: n frames one after another, each waited for to the end: what one frame really costs (CPU + GPU),
    // with no waiting for a slot and no frames queued behind one another in the number
#ifdef GLASS_COMPUTE_DOOR
    // CBEAMBENCH=n: the compute passes of one frame, n times one after another with nothing drawn, then one wait: what
    // the beams cost the GPU on their own. (The poses are sent once; the scene does not move.)
    if (cBeams && std::getenv("CBEAMBENCH")) {
        const int n = (int)env_f("CBEAMBENCH", 1000.f);
        update(env_f("T", 1.3f));
        for (int i = 0; i < 50; ++i) cbeam::run(cRun, cSolids, cStepsAll);
        engine->flushAndWait();
        const auto b0 = std::chrono::steady_clock::now();
        for (int i = 0; i < n; ++i) {
            cbeam::run(cRun, cSolids, cStepsAll);
            if (i % 20 == 19) engine->flush();
        }
        engine->flushAndWait();
        const double ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - b0).count();
        std::printf("CBEAMBENCH: %.3f ms a frame of compute alone (%d frames, %u solids, %u levels)\n", ms / n, n, cSolids, cRun->levels);
        return 0;
    }
#endif
    if (std::getenv("BENCH")) {
        const int n = (int)env_f("BENCH", 200.f);
        const float3 sun0 = sunDir;
        std::vector<double> ms;
        for (int i = 0; i < n + 20; ++i) {
            const float t = (float)i / 120.f * 6.2831853f;
            sunDir = normalize(float3{sun0.x * std::cos(0.3f * std::sin(t)), sun0.y, sun0.z + 0.4f * std::sin(t)});
            auto b0 = std::chrono::steady_clock::now();
            update(t);
            if (!open()) return fail("bench");
            static const bool xAll = std::getenv("XALLLIGHT");  // timing only
            if (!xAll) draw_light();
            if (!noBackdrop) renderer->render(vBackdrop);
            renderer->render(vBeauty);
            renderer->endFrame();
            engine->flushAndWait();
            if (i >= 20) ms.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - b0).count());
        }
        {
            // what the GPU itself was busy for, as Filament's frame timer has it (the last frames)
            std::vector<double> g;
            for (auto const& f : renderer->getFrameInfoHistory(32))
                if (f.gpuFrameDuration > 0) g.push_back(f.gpuFrameDuration * 1e-6);
            std::sort(g.begin(), g.end());
            if (!g.empty()) std::printf("BENCH GPU alone (%zu frames):  min %.2f  median %.2f  max %.2f ms\n", g.size(), g.front(), g[g.size() / 2], g.back());
        }
        std::sort(ms.begin(), ms.end());
        std::printf("BENCH %d frames, %zu solids: one frame takes  min %.2f  median %.2f  95%% %.2f  max %.2f ms\n", n, items.size(), ms.front(), ms[ms.size() / 2],
                    ms[(size_t)(ms.size() * 0.95)], ms.back());
        return 0;
    }
    if (std::getenv("PACE")) {
        const int n = (int)env_f("PACE_FRAMES", 600.f);
        const double hz = env_f("PACE_HZ", 60.f);
        const double period = 1000.0 / hz;
        const float3 sun0 = sunDir;
        std::vector<double> cpu, g, lat, upd, mainMs, backMs;
        int skipped = 0, drawn = 0;
        size_t maxTris = 0;
        auto pct = [](std::vector<double> v, double p) {
            if (v.empty()) return -1.0;
            std::sort(v.begin(), v.end());
            return v[std::min(v.size() - 1, (size_t)(v.size() * p))];
        };
        auto start = std::chrono::steady_clock::now();
