// xcube_gpu.mm, piece 2 (cut there by size, as it was): read only in its place in xcube_gpu.mm.
    echoVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(echo_v.data(), echo_v.size() * sizeof(float)));
    MaterialInstance* echoMi = echoMat->createInstance();
    echoMi->setParameter("eye", float3{0.55f, 0.78f, 0.70f});
    echoMi->setParameter("center", float3{center.x, center.y, center.z});
    echoMi->setParameter("ax", float3{axis[0].x, axis[0].y, axis[0].z});
    echoMi->setParameter("ay", float3{axis[1].x, axis[1].y, axis[1].z});
    echoMi->setParameter("az", float3{axis[2].x, axis[2].y, axis[2].z});
    echoMi->setParameter("halfExtent", half);
    echoMi->setParameter("bevel", kBevel);
    echoMi->setParameter("tilt", kTilt);
    echoMi->setParameter("ior", nd);
    echoMi->setParameter("bounces", 1.f);
    echoMi->setParameter("colorA", float3{0.95f, 0.28f, 0.62f});
    echoMi->setParameter("colorB", float3{0.20f, 0.72f, 0.95f});
    utils::Entity echoE = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, echoVB, bevIB)
        .material(0, echoMi).culling(false).castShadows(false).receiveShadows(false)
        .build(*engine, echoE);
    std::vector<float> panel_v;
    std::vector<uint32_t> panel_i = {0, 1, 2, 0, 2, 3};
    {
        V3 c{0.0f, 0.55f, -1.15f};
        float hw = 1.6f, hh = 0.7f;
        V3 p[4] = {c + V3{-hw, -hh, 0}, c + V3{hw, -hh, 0}, c + V3{hw, hh, 0}, c + V3{-hw, hh, 0}};
        for (int i = 0; i < 4; ++i) push_vert(panel_v, p[i], {1, 0, 0}, {0, 0, 1});
    }
    VertexBuffer* panelVB = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                                .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                                .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                                .build(*engine);
    panelVB->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(panel_v.data(), panel_v.size() * sizeof(float)));
    IndexBuffer* panelIB = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    panelIB->setBuffer(*engine, IndexBuffer::BufferDescriptor(panel_i.data(), panel_i.size() * sizeof(uint32_t)));
    MaterialInstance* panel = emat->createInstance();
    panel->setParameter("color", float4{2200.f, 2300.f, 2600.f, 1});
    utils::Entity panelE = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-4, -4, -4}, {4, 4, 4}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, panelVB, panelIB)
        .material(0, panel).culling(false).castShadows(false).receiveShadows(false)
        .build(*engine, panelE);

    auto make_film = [&](V3 N0, float tilt, float4 color, float4 glow, Material* mat) {
        N0 = norm(N0);
        V3 up = norm(axis[1]);
        V3 side = norm(cross(up, N0));
        float band = (2.f * half) / 3.f;
        float s = half * 0.90f;
        int quads = tilt == 0.f ? 1 : 3;
        static std::vector<float> store[8];
        static std::vector<uint32_t> index[8];
        static int which = 0;
        std::vector<float>& fv = store[which];
        std::vector<uint32_t>& fi = index[which];
        which += 1;
        fv.clear();
        fi.clear();
        for (int k = 0; k < quads; ++k) {
            int kk = quads == 1 ? 0 : k - 1;
            float ang = (float)kk * tilt;
            V3 N = norm(N0 * std::cos(ang) + side * std::sin(ang));
            V3 T = norm(cross(up, N));
            V3 c0 = center + up * ((float)kk * band);
            float vs = quads == 1 ? s : band * 0.55f;
            V3 q[4] = {c0 - T * s - up * vs, c0 + T * s - up * vs, c0 + T * s + up * vs, c0 - T * s + up * vs};
            uint32_t base = (uint32_t)(fv.size() / 7);
            for (int i = 0; i < 4; ++i) push_vert(fv, q[i], T, N);
            fi.insert(fi.end(), {base, base + 1, base + 2, base, base + 2, base + 3});
        }
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(fv.size() / 7)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                               .build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(fv.data(), fv.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)fi.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(fi.data(), fi.size() * sizeof(uint32_t)));
        MaterialInstance* mi = mat->createInstance();
        mi->setParameter("color", color);
        mi->setParameter("glow", glow);
        utils::Entity e = utils::EntityManager::get().create();
        RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi).culling(false).castShadows(false).receiveShadows(false)
            .build(*engine, e);
        return e;
    };
    float4 pink{0.95f, 0.28f, 0.62f, 0.42f};
    float4 cyan{0.20f, 0.72f, 0.95f, 0.42f};
    float4 pinkMirror{0.95f, 0.28f, 0.62f, 1.f};
    float4 cyanMirror{0.20f, 0.72f, 0.95f, 1.f};
    utils::Entity filmA = make_film(norm(axis[0] - axis[2]), 0.f, pink, float4{0, 0, 0, 1}, filmMat);
    utils::Entity filmB = make_film(norm(axis[0] + axis[2]), 0.f, cyan, float4{0, 0, 0, 1}, filmMat);
    utils::Entity filmAf = make_film(norm(axis[0] - axis[2]), kTilt, pink, float4{0, 0, 0, 1}, filmMat);
    utils::Entity filmBf = make_film(norm(axis[0] + axis[2]), kTilt, cyan, float4{0, 0, 0, 1}, filmMat);
    utils::Entity filmAl = make_film(norm(axis[0] - axis[2]), kTilt, pinkMirror, float4{0, 0, 0, 1}, filmLitMat);
    utils::Entity filmBl = make_film(norm(axis[0] + axis[2]), kTilt, cyanMirror, float4{0, 0, 0, 1}, filmLitMat);

    utils::Entity sun_e = utils::EntityManager::get().create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 512;
    shopt.shadowNearHint = 1.0f;
    shopt.shadowFarHint = 100.0f;
    shopt.constantBias = 0.001f;
    shopt.normalBias = 1.0f;
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1, 0.98f, 0.96f}).intensity(lux).direction(float3{sun0.x, sun0.y, sun0.z})
        .sunAngularRadius(0.35f).sunHaloSize(1.2f).castShadows(true).shadowOptions(shopt)
        .build(*engine, sun_e);

    auto& rm = engine->getRenderableManager();
    auto open = [&]() {
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) return true;
        return false;
    };
    Renderer::ClearOptions clearOpt;
    clearOpt.clear = true;
    clearOpt.discard = true;
    clearOpt.clearColor = {0, 0, 0, 0};
    renderer->setClearOptions(clearOpt);
    Skybox* sky = Skybox::Builder().color({0.72f, 0.74f, 0.78f, 1}).build(*engine);
    Skybox* skyDark = Skybox::Builder().color({0.22f, 0.23f, 0.26f, 1}).build(*engine);
    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    BloomOptions bloom{};
    bloom.enabled = true;
    bloom.strength = 0.18f;
    bloom.levels = 6;
    bloom.resolution = 1080;
    bloom.threshold = true;
    bloom.highlight = 10.f;
    bloom.blendMode = BloomOptions::BlendMode::ADD;

    auto bind_sun = [&](V3 sun) {
        V3 D = norm(sun);
        V3 up = std::fabs(D.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0};
        V3 U = norm(cross(up, D));
        V3 V = cross(D, U);
        hitMi->setParameter("dir", float3{D.x, D.y, D.z});
        hitMi->setParameter("tangent", float3{U.x, U.y, U.z});
        hitMi->setParameter("bitangent", float3{V.x, V.y, V.z});
        auto& lm = engine->getLightManager();
        lm.setDirection(lm.getInstance(sun_e), float3{D.x, D.y, D.z});
    };
    auto draw_hits = [&]() -> bool {
        view->setPostProcessingEnabled(false);
        view->setScreenSpaceRefractionEnabled(false);
        view->setBloomOptions(BloomOptions{});
        scene->setSkybox(nullptr);
        camera->setExposure(1.f);
        view->setViewport({0, 0, (uint32_t)hitW, (uint32_t)hitH});
        scene->remove(floor);
        scene->remove(cube);
        scene->remove(panelE);
        scene->remove(sun_e);
        scene->remove(filter);
        scene->remove(splat);
        scene->addEntity(hitE);
        if (!open()) return false;
        view->setRenderTarget(hitRT);
        renderer->render(view);
        renderer->endFrame();
        engine->flushAndWait();
        return true;
    };
    auto draw_paths = [&]() -> bool {
        view->setPostProcessingEnabled(false);
        view->setScreenSpaceRefractionEnabled(false);
        view->setBloomOptions(BloomOptions{});
        scene->setSkybox(nullptr);
        camera->setExposure(1.f);
        camera->lookAt(float3{0, 4, 0}, float3{0, 0, 0}, float3{0, 0, -1});
        camera->setProjection(Camera::Projection::ORTHO, -HW, HW, -HW, HW, 0.01, 10.0);
        view->setViewport({0, 0, (uint32_t)MAP, (uint32_t)MAP});
        scene->remove(floor);
        scene->remove(cube);
        scene->remove(panelE);
        scene->remove(sun_e);
        scene->remove(filter);
        scene->remove(hitE);
        scene->addEntity(splat);
        if (!open()) return false;
        view->setRenderTarget(mapRT);
        renderer->render(view);
        renderer->endFrame();
        engine->flushAndWait();
        return true;
    };
    auto draw_atrous = [&]() -> bool {
        scene->remove(splat);
        scene->remove(hitE);
        scene->addEntity(filter);
        rm.setMaterialInstanceAt(rm.getInstance(filter), 0, atrous1);
        if (!open()) return false;
        view->setRenderTarget(tempRT);
        renderer->render(view);
        renderer->endFrame();
        tempTex->generateMipmaps(*engine);
        engine->flushAndWait();
        rm.setMaterialInstanceAt(rm.getInstance(filter), 0, atrous2);
        if (!open()) return false;
        view->setRenderTarget(filtRT);
        renderer->render(view);
        renderer->endFrame();
        engine->flushAndWait();
        return true;
    };
    bool graded = false;
    auto arm_beauty = [&]() {
        scene->remove(splat);
        scene->remove(hitE);
        scene->remove(filter);
        scene->addEntity(floor);
        scene->addEntity(cube);
        scene->addEntity(panelE);
        scene->addEntity(sun_e);
        scene->setSkybox(sky);
        if (!graded) {
            view->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.04f).saturation(1.08f).build(*engine));
            graded = true;
        }
        view->setPostProcessingEnabled(true);
        view->setScreenSpaceRefractionEnabled(true);
        view->setShadowType(View::ShadowType::PCF);
        view->setBloomOptions(bloom);
        view->setAntiAliasing(View::AntiAliasing::FXAA);
        view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
        camera->setExposure(2.8f, 1.f / 125.f, 100.f);
        camera->lookAt(float3{0.55f, 0.78f, 0.70f}, float3{0, 0.04f, 0}, float3{0, 1, 0});
        camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
    };
    auto draw_beauty = [&]() -> bool {
        if (!open()) return false;
        view->setRenderTarget(beautyRT);
        renderer->render(view);
        renderer->endFrame();
        engine->flushAndWait();
        return true;
    };
    auto grab = [&](const char* path) -> bool {
        if (!open()) return false;
        view->setRenderTarget(beautyRT);
        renderer->render(view);
        std::vector<uint8_t> frame((size_t)W * H * 4);
        renderer->readPixels(beautyRT, 0, 0, W, H,
                             backend::PixelBufferDescriptor(frame.data(), frame.size(), backend::PixelDataFormat::RGBA,
                                                            backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
        return write_png(path, W, H, frame.data());
    };

    bind_sun(sun0);
    if (!draw_hits()) return fail("hits");
    if (!draw_paths()) return fail("paths");
    if (!draw_atrous()) return fail("atrous");
    arm_beauty();
    if (!draw_beauty()) return fail("beauty");
    std::vector<double> hitMs, pathMs, atrousMs, beautyMs, totalMs;
    for (int i = 0; i < 5; ++i) {
        auto t0 = std::chrono::steady_clock::now();
        if (!draw_hits()) return fail("time-hits");
        hitMs.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
        auto tq = std::chrono::steady_clock::now();
        if (!draw_paths()) return fail("time-paths");
        pathMs.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - tq).count());
        auto t1 = std::chrono::steady_clock::now();
        if (!draw_atrous()) return fail("time-atrous");
        atrousMs.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t1).count());
        arm_beauty();
        auto t2 = std::chrono::steady_clock::now();
        if (!draw_beauty()) return fail("time-beauty");
        beautyMs.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t2).count());
        totalMs.push_back(hitMs.back() + pathMs.back() + atrousMs.back() + beautyMs.back());
    }
    auto med = [](std::vector<double> v) {
        std::sort(v.begin(), v.end());
        return v[v.size() / 2];
    };
    std::printf("gpu wall ms, flushAndWait, readPixels excluded, median of 5\n");
    std::printf("hits %.2f  quads %.2f  atrous %.2f  beauty %.2f  total %.2f\n", med(hitMs), med(pathMs), med(atrousMs), med(beautyMs), med(totalMs));
    std::printf("%d wavelengths, grid %d, PCF 512. Not a Metal timestamp.\n", NW, GRID);
    if (tryMode) {
        char tryPath[64];
        std::snprintf(tryPath, sizeof(tryPath), "out/xcube-try-%d-%d.png", NW, GRID);
        arm_beauty();
        if (!grab(tryPath)) return fail("png");
        std::printf("PASS xcube-gpu\n");
        return 0;
    }

    auto& lm = engine->getLightManager();
    auto light = lm.getInstance(sun_e);
    LightManager::ShadowOptions wide = shopt;
    LightManager::ShadowOptions tight = shopt;
    tight.shadowNearHint = 0.2f;
    tight.shadowFarHint = 2.2f;
    tight.constantBias = 0.0006f;
    tight.normalBias = 0.4f;
    tight.polygonOffsetConstant = 0.3f;
    tight.polygonOffsetSlope = 1.0f;
    auto show_films = [&](bool on) {
        scene->remove(filmA);
        scene->remove(filmB);
        if (on) {
            scene->addEntity(filmA);
            scene->addEntity(filmB);
        }
    };
    auto milk = [&]() {
        arm_beauty();
        show_films(false);
        scene->addEntity(panelE);
        scene->setSkybox(sky);
        floorMi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
        glass->setParameter("reflectance", 0.5f);
        glass->setParameter("absorption", float3{0.02f, 0.015f, 0.02f});
        camera->setExposure(2.8f, 1.f / 125.f, 100.f);
        lm.setSunAngularRadius(light, 0.35f);
        lm.setShadowOptions(light, wide);
        view->setShadowType(View::ShadowType::PCF);
        view->setShadowingEnabled(true);
        view->setBloomOptions(bloom);
    };
    auto clear_glass = [&]() {
        arm_beauty();
        scene->remove(panelE);
        scene->setSkybox(sky);
        floorMi->setParameter("color", float4{0.90f, 0.89f, 0.88f, 1});
        glass->setParameter("reflectance", 0.42f);
        glass->setParameter("absorption", float3{0.004f, 0.004f, 0.006f, });
        camera->setExposure(2.8f, 1.f / 180.f, 100.f);
        BloomOptions lean = bloom;
        lean.strength = 0.08f;
        view->setBloomOptions(lean);
        view->setShadowType(View::ShadowType::PCF);
        view->setShadowingEnabled(true);
    };
    milk();
    if (!grab("out/xcube-rw-a.png")) return fail("a");
    clear_glass();
    lm.setShadowOptions(light, wide);
    if (!grab("out/xcube-rw-b.png")) return fail("b");
    clear_glass();
    show_films(true);
    lm.setShadowOptions(light, wide);
    if (!grab("out/xcube-rw-c.png")) return fail("c");
    milk();
    lm.setShadowOptions(light, tight);
    if (!grab("out/xcube-rw-d.png")) return fail("d");
    clear_glass();
    show_films(true);
    lm.setShadowOptions(light, tight);
    lm.setSunAngularRadius(light, 0.02f);
    view->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.2f).saturation(1.4f).build(*engine));
    if (!grab("out/xcube-rw-e.png")) return fail("e");

    auto time_beauty = [&](const char* name) {
        if (!draw_beauty()) return;
        std::vector<double> v;
        for (int i = 0; i < 5; ++i) {
            auto t = std::chrono::steady_clock::now();
            if (!draw_beauty()) return;
            v.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t).count());
        }
        std::printf("beauty %-16s %.2f\n", name, med(v));
    };
    time_beauty("E pcf512 tight");
    tight.mapSize = 1024;
    lm.setShadowOptions(light, tight);
    time_beauty("E pcf1024 tight");
    if (!grab("out/xcube-rw-e-1k.png")) return fail("e1k");
    tight.mapSize = 512;
    lm.setShadowOptions(light, tight);
    view->setShadowingEnabled(false);
    time_beauty("E no shadow");
    view->setShadowingEnabled(true);
    view->setScreenSpaceRefractionEnabled(false);
    time_beauty("E no refraction");
    view->setScreenSpaceRefractionEnabled(true);
    BloomOptions bloomOff{};
    view->setBloomOptions(bloomOff);
    time_beauty("E no bloom");
    view->setBloomOptions(bloom);
    BloomOptions lean = bloom;
    lean.strength = 0.08f;
    view->setBloomOptions(lean);

    auto bake = [&](float bevel, float tilt) {
        hitMi->setParameter("bevel", bevel);
        hitMi->setParameter("tilt", tilt);
        if (!draw_hits()) return false;
        if (!draw_paths()) return false;
        if (!draw_atrous()) return false;
        return true;
    };
    auto facet0 = std::chrono::steady_clock::now();
    if (!bake(kBevel, kTilt)) return fail("facet");
    double facetMs = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - facet0).count();
    clear_glass();
    show_films(false);
    scene->remove(cube);
    scene->addEntity(cubeBevel);
    scene->addEntity(filmAf);
    scene->addEntity(filmBf);
    lm.setShadowOptions(light, tight);
    lm.setSunAngularRadius(light, 0.02f);
    if (!grab("out/xcube-rw-f.png")) return fail("f");
    time_beauty("F pcf512 tight");
    std::printf("facet bake %.2f bevel %.3f tilt %.3f\n", facetMs, kBevel, kTilt);
    scene->addEntity(echoE);
    echoMi->setParameter("bounces", 1.f);
    if (!grab("out/xcube-rw-g1.png")) return fail("g1");
    time_beauty("G1 one bounce");
    echoMi->setParameter("bounces", 2.f);
    if (!grab("out/xcube-rw-g2.png")) return fail("g2");
    time_beauty("G2 two bounces");
    scene->remove(echoE);

    const int kFace = 128;
    const float kIbl = 2200.f;
    auto face_dir = [](int face, float u, float v) {
        switch (face) {
        case 0: return norm({1.f, v, -u});
        case 1: return norm({-1.f, v, u});
        case 2: return norm({u, 1.f, -v});
        case 3: return norm({u, -1.f, v});
        case 4: return norm({u, v, 1.f});
        default: return norm({-u, v, -1.f});
        }
    };
    auto add_strip = [](float3 c, V3 d, V3 n, float rad, float3 col, float gain) {
        float w = std::exp(-(dot(d, norm(n)) * dot(d, norm(n))) / (rad * rad));
        return float3{c.x + col.x * w * gain, c.y + col.y * w * gain, c.z + col.z * w * gain};
    };
    auto add_spot = [](float3 c, V3 d, V3 a, float rad, float3 col, float gain) {
        float cd = std::clamp(dot(d, norm(a)), -1.f, 1.f);
        float ang = std::acos(cd);
        float w = std::exp(-(ang * ang) / (rad * rad));
        return float3{c.x + col.x * w * gain, c.y + col.y * w * gain, c.z + col.z * w * gain};
    };
