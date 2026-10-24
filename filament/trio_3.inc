// trio.mm, piece 3 (cut there by size, as it was): read only in its place in trio.mm.
    for (int frame = 0; frame < 2; ++frame) {
        bool last = frame == 1;
        if (!open_frame()) return fail("begin");
        bloom.enabled = false;
        view->setBloomOptions(bloom);
        view->setPostProcessingEnabled(false);
        view->setAntiAliasing(View::AntiAliasing::NONE);
        view->setScreenSpaceRefractionEnabled(false);
        if (chrome_shot) {
            ScreenSpaceReflectionsOptions refl{};
            refl.enabled = true;
            refl.thickness = 0.18f;
            refl.bias = 0.02f;
            refl.maxDistance = 8.0f;
            refl.stride = 2.0f;
            view->setScreenSpaceReflectionsOptions(refl);
        }
        scene->setSkybox(sky);
        set_shown(opaque);
        render_to(rtF[0], false, false);

        scene->setSkybox(nullptr);
        view->setScreenSpaceRefractionEnabled(true);
        if (chrome_shot) {
            ScreenSpaceReflectionsOptions refl{};
            refl.enabled = false;
            view->setScreenSpaceReflectionsOptions(refl);
        }
        int src = 0;
        int g0 = chrome_shot ? 1 : 0;
        for (int k = 0; k < nglass; ++k) {
            int g = g0 + k;
            int dst = 1 - src;
            rm.setMaterialInstanceAt(rm.getInstance(plate.entity), 0, plate_mi[src]);
            set_shown({plate.entity, glasses[g].entity});
            render_to(rtF[dst], last && k == nglass - 1, false);
            src = dst;
        }

        bloom.enabled = true;
        bloom.strength = bloom_strength;
        bloom.levels = 7;
        bloom.resolution = 1080;
        bloom.threshold = true;
        bloom.highlight = 12.0f;
        bloom.blendMode = BloomOptions::BlendMode::ADD;
        view->setBloomOptions(bloom);
        view->setPostProcessingEnabled(true);
        view->setAntiAliasing(View::AntiAliasing::FXAA);
        view->setScreenSpaceRefractionEnabled(false);
        rm.setMaterialInstanceAt(rm.getInstance(plate.entity), 0, plate_mi[src]);
        set_shown({plate.entity});
        render_to(rt8, false, last);
        renderer->endFrame();
    }
    engine->flushAndWait();
    if (!write_png(out_path, W, H, pic.data())) return fail("png");

    auto atf = [&](int x, int y) {
        size_t i = ((size_t)y * W + x) * 4;
        return float3{half_to_float(half[i]), half_to_float(half[i + 1]), half_to_float(half[i + 2])};
    };
    float maxv = 0;
    int mx = 0, my = 0;
    int above = 0;
    for (int y = 0; y < H; y += 4) {
        for (int x = 0; x < W; x += 4) {
            float3 c = atf(x, y);
            float m = std::max(c.x, std::max(c.y, c.z));
            if (m > 1.f) above++;
            if (m > maxv) {
                maxv = m;
                mx = x;
                my = y;
            }
        }
    }
    std::printf("HDR before grade  max %.3f at %d,%d  samples>1 %d\n", maxv, mx, my, above);
    std::printf("wrote %s\n", out_path);
    return 0;
}
