// glass_beam.mm, piece 2 (cut there by size, as it was): read only in its place in glass_beam.mm.
    // ---- pose and sun: the only inputs
    float3 sunDir = normalize(float3{0.92f, -0.42f, 0.22f});
    Pose pose{{0.f, 0.16f, 0.f}, 0.62f, 0.16f};
    double traceMs = 0;
    size_t lightTris = 0;
    auto update = [&]() {
        const float e = 2.f * pose.half;
        const float c = std::cos(pose.yaw), s = std::sin(pose.yaw);
        float3 ax{c, 0, -s}, ay{0, 1, 0}, az{s, 0, c};
        const mat4f xf(float4{ax * e, 0}, float4{ay * e, 0}, float4{az * e, 0}, float4{pose.center, 1});
        tm.setTransform(tm.getInstance(cube), xf);
        tm.setTransform(tm.getInstance(caster), xf);
        auto& lm = engine->getLightManager();
        lm.setDirection(lm.getInstance(sunE), sunDir);

        glass->setParameter("sunTo", -sunDir);
        glass->setParameter("sunGlint", sunColor * env_f("GLINT", 4.0e5f));
        glass->setParameter("sunCos", std::cos(env_f("GLINTDEG", 2.0f) * 3.14159265f / 180.f));
        auto t0 = std::chrono::steady_clock::now();
        beam::Solid solid = beam::chamfered_box({pose.center.x, pose.center.y, pose.center.z}, pose.yaw, pose.half, chamfer * e);
        beam::Params par;
        par.sun = {sunDir.x, sunDir.y, sunDir.z};
        par.bands = bands;
        par.glass.nd = nd;
        par.glass.exaggerate = disp;
        par.cutoff = env_f("CUTOFF", 0.02f);
        par.maxBounces = (int)env_f("BOUNCES", 4.f);
        static std::vector<float> tri;
        if (std::getenv("REF")) {
            // reference: every band traced in full (slow, exact per band)
            par.maxBounces = 6;
            par.cutoff = 0.002;
            beam::to_triangles(beam::trace(solid, par), tri);
        } else {
            // one pixel on the floor near the block, in world units
            const float dist = length(float3{cam[0], cam[1], cam[2]} - pose.center);
            const double pixel = 2.0 * std::tan(cam[6] * 0.5 * 3.14159265 / 180.0) * dist / H;
            beam::trace_fast(solid, par, pixel * env_f("DETAIL", 1.f), tri);
        }
        traceMs = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
        const size_t TS = beam::TRI;
        size_t nLight = std::min<size_t>(tri.size() / TS, MAX_LIGHT_VERTS - 3000);
        nLight -= nLight % 3;
        tri.resize(nLight * TS);
        if (!filamentShadow) beam::silhouette_triangles(solid, par.sun, 0.0, tri);
        size_t nv = tri.size() / TS;
        lightTris = nLight / 3;
        softenMi->setParameter("sunDown", -sunDir.y);
        auto* buf = new std::vector<float>(std::max<size_t>(nv, 3) * 7, 0.f);
        const bool off = std::getenv("NOLIGHT") != nullptr;
        for (size_t i = 0; i < nv; ++i) {
            float* o = buf->data() + i * 7;
            o[0] = tri[i * TS];
            o[1] = 0.f;
            o[2] = tri[i * TS + 1];
            o[3] = (off && i < nLight) ? 0.f : tri[i * TS + 2];
            o[4] = (off && i < nLight) ? 0.f : tri[i * TS + 3];
            o[5] = (off && i < nLight) ? 0.f : tri[i * TS + 4];
            o[6] = tri[i * TS + 5];  // distance flown, for the softening pass
        }
        lightVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(buf->data(), buf->size() * sizeof(float),
                                                                         [](void*, size_t, void* u) { delete (std::vector<float>*)u; }, buf));
        rm.setGeometryAt(rm.getInstance(lightE), 0, RenderableManager::PrimitiveType::TRIANGLES, lightVB, lightIB, 0, std::max<size_t>(nLight, 3));
        // nothing to draw when there is no outline: point at one corner three times
        rm.setGeometryAt(rm.getInstance(shadeE), 0, RenderableManager::PrimitiveType::TRIANGLES, lightVB, lightIB, nv > nLight ? nLight : 0, nv > nLight ? nv - nLight : 0);
    };
    if (const char* sv = std::getenv("SUN")) {
        float x, y, z;
        if (std::sscanf(sv, "%f %f %f", &x, &y, &z) == 3) sunDir = normalize(float3{x, y, z});
    }
    pose.yaw = env_f("YAW", pose.yaw);
    pose.half = env_f("HALF", pose.half);
    pose.center.y = pose.half + 0.0005f;

    auto draw_light = [&]() {
        renderer->render(vLight);
        renderer->render(vShade);
        light.color->generateMipmaps(*engine);
        shade.color->generateMipmaps(*engine);
        renderer->render(vSoft);
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
        renderer->render(vBackdrop);
        renderer->render(vBeauty);
        renderer->readPixels(beauty.rt, 0, 0, W, H,
                             backend::PixelBufferDescriptor(frame.data(), frame.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
        return write_png(path, W, H, frame.data());
    };
    std::system("mkdir -p out");
    update();
    for (int i = 0; i < 3; ++i) {
        if (!open()) return fail("warm");
        draw_light();
        renderer->render(vBackdrop);
        renderer->render(vBeauty);
        renderer->endFrame();
        engine->flushAndWait();
    }
    const std::string out = "out/glass-beam-" + name + ".png";
    if (!grab(out.c_str())) return fail("grab");

    // ---- paced like an app: every tick the block turns and slides and the sun swings, so the light is traced
    // again and uploaded again each frame. beginFrame() returning false is Filament saying it is behind.
    if (std::getenv("PACE")) {
        const int n = (int)env_f("PACE_FRAMES", 600.f);
        const double hz = env_f("PACE_HZ", 60.f);
        const double period = 1000.0 / hz;
        const Pose pose0 = pose;
        const float3 sun0 = sunDir;
        std::vector<double> cpu, g, lat, upd;
        int skipped = 0, drawn = 0;
        size_t maxTris = 0;
        auto pct = [](std::vector<double> v, double p) {
            if (v.empty()) return -1.0;
            std::sort(v.begin(), v.end());
            return v[std::min(v.size() - 1, (size_t)(v.size() * p))];
        };
        auto start = std::chrono::steady_clock::now();
        for (int i = 0; i < n; ++i) {
            auto due = start + std::chrono::duration<double, std::milli>(period * i);
            while (std::chrono::steady_clock::now() < due) {}
            const float t = (float)i / 120.f * 6.2831853f;
            pose = pose0;
            pose.center.x += 0.10f * std::cos(t);
            pose.center.z += 0.08f * std::sin(t);
            pose.yaw = pose0.yaw + t;
            sunDir = normalize(float3{sun0.x * std::cos(0.3f * std::sin(t)), sun0.y, sun0.z + 0.4f * std::sin(t)});
            auto u0 = std::chrono::steady_clock::now();
            update();
            upd.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - u0).count());
            cpu.push_back(traceMs);
            maxTris = std::max(maxTris, lightTris);
            if (!renderer->beginFrame(swap)) { ++skipped; continue; }
            draw_light();
            renderer->render(vBackdrop);
            renderer->render(vBeauty);
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
        const double total = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - start).count();
        size_t late = 0;
        for (double v : lat) if (v > period) ++late;
        std::printf("PACE %.0f Hz for %d frames (%.1f s, target %.1f s), block and sun moving every frame, %dx%d, light buffer x%d\n", hz, n, total / 1000.0,
                    n * period / 1000.0, W, H, SS);
        auto line = [&](const char* nm, std::vector<double>& v) {
            std::printf("     %-40s n=%3zu  min %6.2f  median %6.2f  95%% %6.2f  99%% %6.2f  max %6.2f ms\n", nm, v.size(), pct(v, 0.0), pct(v, 0.5), pct(v, 0.95), pct(v, 0.99), pct(v, 1.0));
        };
        line("CPU: trace the light", cpu);
        line("CPU: trace + fill + hand to Filament", upd);
        line("GPU time per frame", g);
        line("beginFrame -> GPU finished", lat);
        std::printf("     most light triangles in a frame: %zu\n", maxTris);
        std::printf("     finished after their %.2f ms slot: %zu of %zu measured (%.1f%%)\n", period, late, lat.size(), lat.empty() ? 0.0 : 100.0 * late / lat.size());
        std::printf("     DROPPED (Filament asked to skip): %d of %d ticks (%.1f%%)  -> delivered %.1f fps of %.0f\n", skipped, n, 100.0 * skipped / n, drawn / (total / 1000.0), hz);
    }
    std::printf("wrote %s  (chamfer %.3f, dispersion x%.1f, %d bands, trace %.2f ms, %zu light triangles, light buffer %dx%d)\n", out.c_str(), chamfer, disp,
                bands, traceMs, lightTris, W * SS, H * SS);
    return 0;
}
