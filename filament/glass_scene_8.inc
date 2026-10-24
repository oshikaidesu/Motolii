// glass_scene.mm, piece 8 (cut there by size, as it was): read only in its place in glass_scene.mm.
        for (int i = 0; i < n; ++i) {
            auto due = start + std::chrono::duration<double, std::milli>(period * i);
            while (std::chrono::steady_clock::now() < due) {}
            const float t = (float)i / 120.f * 6.2831853f;
            sunDir = normalize(float3{sun0.x * std::cos(0.3f * std::sin(t)), sun0.y, sun0.z + 0.4f * std::sin(t)});
            auto u0 = std::chrono::steady_clock::now();
            update(t);
            upd.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - u0).count());
            cpu.push_back(traceMs);
            maxTris = std::max(maxTris, lightTris);
            if (!renderer->beginFrame(swap)) { ++skipped; continue; }
            draw_light();
            if (!noBackdrop) renderer->render(vBackdrop);
            renderer->render(vBeauty);
            renderer->endFrame();
            ++drawn;
            if (drawn % 16 == 0 || i == n - 1) {
                engine->flush();
                auto hist = renderer->getFrameInfoHistory(16);
                for (auto const& f : hist) {
                    if (f.endFrame > 0 && f.beginFrame > 0) mainMs.push_back((f.endFrame - f.beginFrame) * 1e-6);
                    if (f.backendEndFrame > 0 && f.backendBeginFrame > 0) backMs.push_back((f.backendEndFrame - f.backendBeginFrame) * 1e-6);
                    if (f.gpuFrameDuration > 0) g.push_back(f.gpuFrameDuration * 1e-6);
                    if (f.gpuFrameComplete > 0 && f.beginFrame > 0) lat.push_back((f.gpuFrameComplete - f.beginFrame) * 1e-6);
                }
            }
        }
        engine->flushAndWait();
        const double total = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - start).count();
        size_t late = 0;
        for (double v : lat) if (v > period) ++late;
        std::printf("PACE %.0f Hz for %d frames (%.1f s, target %.1f s), %zu solids and the sun moving every frame, %dx%d, %u threads\n", hz, n, total / 1000.0,
                    n * period / 1000.0, items.size(), W, H, threads);
        auto line = [&](const char* nm, std::vector<double>& v) {
            std::printf("     %-44s n=%3zu  min %6.2f  median %6.2f  95%% %6.2f  99%% %6.2f  max %6.2f ms\n", nm, v.size(), pct(v, 0.0), pct(v, 0.5), pct(v, 0.95), pct(v, 0.99), pct(v, 1.0));
        };
        line("CPU: the light of all solids", cpu);
        line("CPU: trace + fill + hand to Filament", upd);
        line("main thread, beginFrame -> endFrame", mainMs);
        line("backend thread, per frame", backMs);
        line("GPU time per frame", g);
        line("beginFrame -> GPU finished", lat);
        std::printf("     most light triangles in a frame: %zu from the CPU; balls drawn from the one shared shape: %zu, cones: %zu\n", maxTris, ballsAlike, conesDrawn);
        std::printf("     finished after their %.2f ms slot: %zu of %zu measured (%.1f%%)\n", period, late, lat.size(), lat.empty() ? 0.0 : 100.0 * late / lat.size());
        std::printf("     DROPPED (Filament asked to skip): %d of %d ticks (%.1f%%)  -> delivered %.1f fps of %.0f\n", skipped, n, 100.0 * skipped / n, drawn / (total / 1000.0), hz);
    }
    return 0;
}
