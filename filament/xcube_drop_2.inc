// xcube_drop.mm, piece 2 (cut there by size, as it was): read only in its place in xcube_drop.mm.
    // ---- the only per-frame CPU work: a handful of uniforms and two transforms
    auto set_sun = [&](float3 sun) {
        float3 D = normalize(sun);
        float3 up = std::fabs(D.y) < 0.9f ? float3{0, 1, 0} : float3{1, 0, 0};
        float3 U = normalize(cross(up, D));
        float3 V = cross(D, U);
        hitMi->setParameter("dir", D);
        hitMi->setParameter("tangent", U);
        hitMi->setParameter("bitangent", V);
        auto& lm = engine->getLightManager();
        lm.setDirection(lm.getInstance(sunE), D);
    };
    auto set_pose = [&](const Pose& p) {
        float c = std::cos(p.yaw), s = std::sin(p.yaw);
        float3 ax{c, 0, -s}, ay{0, 1, 0}, az{s, 0, c};  // rotateY applied to the unit axes
        hitMi->setParameter("center", p.center);
        hitMi->setParameter("ax", ax);
        hitMi->setParameter("ay", ay);
        hitMi->setParameter("az", az);
        hitMi->setParameter("halfExtent", p.half);
        hitMi->setParameter("extent", p.half * 3.1f);
        splatMi->setParameter("extent", p.half * 3.1f);
        glass->setParameter("thickness", p.half * 2.f);
        const float e = 2.f * p.half;
        tm.setTransform(tm.getInstance(cube), mat4f(float4{ax * e, 0}, float4{ay * e, 0}, float4{az * e, 0}, float4{p.center, 1}));
        // the light map rides under the cube
        const float3 m{p.center.x, 0, p.center.z};
        camTop->lookAt(m + float3{0, 4, 0}, m, float3{0, 0, -1});
        tm.setTransform(tm.getInstance(blur1), mat4f::translation(m));
        tm.setTransform(tm.getInstance(blur2), mat4f::translation(m));
        floorMi->setParameter("mapCenter", float2{m.x, m.z});
    };
    auto open = [&]() {
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) return true;
        return false;
    };
    const bool split = std::getenv("SPLIT") != nullptr;  // debug: one frame per pass, like xcube_gpu.mm
    auto step = [&]() {
        if (!split) return;
        renderer->endFrame();
        engine->flushAndWait();
        open();
    };
    auto draw = [&]() {
        renderer->render(vHit);
        step();
        renderer->render(vPath);
        step();
        renderer->render(vBlur1);
        if (split) { renderer->endFrame(); temp.color->generateMipmaps(*engine); engine->flushAndWait(); open(); }
        else temp.color->generateMipmaps(*engine);
        renderer->render(vBlur2);
        step();
        renderer->render(vBeauty);
    };
    std::vector<uint8_t> frame((size_t)W * H * 4);
    auto grab = [&](const char* path) -> bool {
        if (!open()) return false;
        draw();
        renderer->readPixels(beauty.rt, 0, 0, W, H,
                             backend::PixelBufferDescriptor(frame.data(), frame.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
        return write_png(path, W, H, frame.data());
    };
    std::system("mkdir -p out");

    // ---- 1. the keeper pose: must match xcube_gpu.mm `try 11 56`
    const float3 sun0 = normalize(float3{0.92f, -0.42f, 0.22f});
    const Pose keeper{{0.f, 0.16f + 0.001f, 0.f}, 0.62f, 0.16f};
    set_sun(sun0);
    set_pose(keeper);
    for (int i = 0; i < 3; ++i) {  // warm up (shader compilation, first-use mip chain)
        if (!open()) return fail("warm");
        draw();
        renderer->endFrame();
        engine->flushAndWait();
    }
    if (!grab("out/xcube-drop-first.png")) return fail("keeper");
    std::vector<uint8_t> keeperPic = frame;

    // ---- 2. drop it somewhere else, turn it, resize it, move the sun: everything follows
    if (true) {
        set_pose({{0.30f, 0.11f + 0.001f, -0.22f}, 0.15f, 0.11f});
        if (!grab("out/xcube-drop-moved.png")) return fail("moved");
        set_pose({{-0.22f, 0.20f + 0.001f, 0.10f}, 1.10f, 0.20f});
        set_sun(normalize(float3{0.55f, -0.50f, -0.45f}));
        if (!grab("out/xcube-drop-turned.png")) return fail("turned");
    }

    // ---- 3. timing. Wall clock per frame, plus Filament's own FrameInfo (GPU duration, main thread, backend thread).
    const bool still = std::getenv("STILL") != nullptr;      // do not move the cube / sun
    const bool nowait = std::getenv("NOWAIT") != nullptr;    // do not flushAndWait every frame (as a real app would not)
    const int skip = std::getenv("SKIP") ? std::atoi(std::getenv("SKIP")) : 0;  // 1 = picture only, 2 = light only
    auto draw_timed = [&]() {
        if (skip != 1) {
            renderer->render(vHit);
            renderer->render(vPath);
            renderer->render(vBlur1);
            temp.color->generateMipmaps(*engine);
            renderer->render(vBlur2);
        }
        if (skip != 2) renderer->render(vBeauty);
    };
    const int frames = 240;
    std::vector<double> wall, gpu, mainT, backT;
    auto pct = [](std::vector<double> v, double p) {
        if (v.empty()) return -1.0;
        std::sort(v.begin(), v.end());
        return v[std::min(v.size() - 1, (size_t)(v.size() * p))];
    };
    auto tAll = std::chrono::steady_clock::now();
    for (int i = 0; i < frames; ++i) {
        if (!still) {
            float t = (float)i / 120.f * 6.2831853f;
            set_pose({{0.25f * std::cos(t), 0.161f, 0.20f * std::sin(t)}, 0.62f + t, 0.16f});
            set_sun(normalize(float3{0.92f * std::cos(0.3f * std::sin(t)), -0.42f, 0.22f + 0.4f * std::sin(t)}));
        }
        auto t0 = std::chrono::steady_clock::now();
        if (!open()) return fail("frame");
        draw_timed();
        renderer->endFrame();
        if (!nowait) engine->flushAndWait();
        wall.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
        if (i % 16 == 15 || i == frames - 1) {
            auto hist = renderer->getFrameInfoHistory(16);
            for (auto const& f : hist) {
                if (f.gpuFrameDuration > 0) gpu.push_back(f.gpuFrameDuration * 1e-6);
                if (f.endFrame > 0 && f.beginFrame > 0) mainT.push_back((f.endFrame - f.beginFrame) * 1e-6);
                if (f.backendEndFrame > 0 && f.backendBeginFrame > 0) backT.push_back((f.backendEndFrame - f.backendBeginFrame) * 1e-6);
            }
        }
    }
    engine->flushAndWait();
    double total = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - tAll).count();
    std::printf("TIME %dx%d, %d wavelengths, grid %d, PCF 512%s%s%s: %d frames in %.0f ms = %.2f ms/frame average\n", W, H, NW, GRID,
                still ? ", still" : ", cube+sun moving", nowait ? ", no per-frame wait" : ", flushAndWait each frame",
                skip == 1 ? ", PICTURE ONLY" : (skip == 2 ? ", LIGHT ONLY" : ""), frames, total, total / frames);
    auto line = [&](const char* name, std::vector<double>& v) {
        std::printf("     %-34s n=%3zu  min %6.2f  median %6.2f  95%% %6.2f  max %6.2f ms\n", name, v.size(), pct(v, 0.0), pct(v, 0.5), pct(v, 0.95), pct(v, 1.0));
    };
    line("wall clock per frame", wall);
    line("GPU (Filament gpuFrameDuration)", gpu);
    line("main thread beginFrame..endFrame", mainT);
    line("backend thread", backT);
    std::printf("     max history %zu frames; 60fps budget 16.67 ms\n", renderer->getMaxFrameHistorySize());

    // ---- 3b. the real question: paced at 60 Hz like an app, does every frame finish inside its 16.67 ms slot?
    if (std::getenv("PACE")) {
        const int n = std::getenv("PACE_FRAMES") ? std::atoi(std::getenv("PACE_FRAMES")) : 600;
        const double hz = std::getenv("PACE_HZ") ? std::atof(std::getenv("PACE_HZ")) : 60.0;
        const double period = 1000.0 / hz;
        std::vector<double> g, lat;
        int skipped = 0, drawn = 0;
        auto start = std::chrono::steady_clock::now();
        for (int i = 0; i < n; ++i) {
            auto due = start + std::chrono::duration<double, std::milli>(period * i);
            while (std::chrono::steady_clock::now() < due) {}
            float t = (float)i / 120.f * 6.2831853f;
            if (!still) {
                set_pose({{0.25f * std::cos(t), 0.161f, 0.20f * std::sin(t)}, 0.62f + t, 0.16f});
                set_sun(normalize(float3{0.92f * std::cos(0.3f * std::sin(t)), -0.42f, 0.22f + 0.4f * std::sin(t)}));
            }
            // One attempt per tick. beginFrame() returning false is Filament saying "skip this frame, I am behind".
            if (!renderer->beginFrame(swap)) { ++skipped; continue; }
            draw_timed();
            renderer->endFrame();
            ++drawn;
            if (drawn % 16 == 0 || i == n - 1) {
                engine->flush();
                auto hist = renderer->getFrameInfoHistory(16);
                for (auto const& f : hist) {
                    if (f.gpuFrameDuration > 0) g.push_back(f.gpuFrameDuration * 1e-6);
                    if (f.gpuFrameComplete > 0 && f.beginFrame > 0) lat.push_back((f.gpuFrameComplete - f.beginFrame) * 1e-6);
                }
            }
        }
        engine->flushAndWait();
        double total2 = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - start).count();
        size_t late = 0;
        for (double v : lat) if (v > period) ++late;
        std::printf("PACE %.0f Hz for %d frames (%.1f s, target %.1f s)%s\n", hz, n, total2 / 1000.0, n * period / 1000.0, still ? ", still" : ", cube+sun moving");
        std::printf("     %-34s n=%3zu  min %6.2f  median %6.2f  95%% %6.2f  99%% %6.2f  max %6.2f ms\n", "GPU time per frame", g.size(), pct(g, 0.0), pct(g, 0.5), pct(g, 0.95), pct(g, 0.99), pct(g, 1.0));
        std::printf("     %-34s n=%3zu  min %6.2f  median %6.2f  95%% %6.2f  99%% %6.2f  max %6.2f ms\n", "beginFrame -> GPU finished", lat.size(), pct(lat, 0.0), pct(lat, 0.5), pct(lat, 0.95), pct(lat, 0.99), pct(lat, 1.0));
        std::printf("     finished after their %.2f ms slot: %zu of %zu measured (%.1f%%)\n", period, late, lat.size(), lat.empty() ? 0.0 : 100.0 * late / lat.size());
        std::printf("     DROPPED (Filament asked to skip): %d of %d ticks (%.1f%%)  -> delivered %.1f fps of %.0f\n", skipped, n, 100.0 * skipped / n, drawn / (total2 / 1000.0), hz);
    }

    // ---- 4. compare the keeper pose with the picture xcube_gpu.mm made
    {
        // re-render the keeper pose last so the saved file is what was compared
        set_sun(sun0);
        set_pose(keeper);
        if (!grab("out/xcube-drop-keeper.png")) return fail("keeper2");
        (void)keeperPic;
        std::printf("wrote out/xcube-drop-keeper.png, out/xcube-drop-moved.png, out/xcube-drop-turned.png\n");
    }
    return 0;
}
