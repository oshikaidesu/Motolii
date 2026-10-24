// beauty.mm, part 2 (cut there by size, as it was): read only through beauty.mm, in order.
#pragma once


int main() {
    std::vector<Sample> path = close_frames(the_path(), 32);
    float L = closed_len(path);
    float kappa = 0, sep = 1e9f;
    int n = (int)path.size();
    for (int i = 0; i < n; ++i) {
        int a = (i + n - 1) % n, b = (i + 1) % n;
        float ds = path[b].s - path[a].s;
        if (ds < 0) ds += L;
        if (ds > 1e-6f) kappa = std::max(kappa, std::acos(std::clamp(dot(path[a].T, path[b].T), -1.f, 1.f)) / ds);
    }
    for (int i = 0; i < n; ++i)
        for (int j = i + 1; j < n; ++j) {
            float ds = path[j].s - path[i].s;
            if (std::min(ds, L - ds) < 0.35f) continue;
            sep = std::min(sep, len(path[j].p - path[i].p));
        }
    const float glass_r = 0.050f;
    std::printf("path rings %d  length %.3f  kappa %.3f  r*kappa %.3f  separation %.3f  diameter %.3f\n", n, L, kappa,
                glass_r * kappa, sep, glass_r * 2.f);

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(16.0f, 1.0f / 125.0f, 100.0f);
    camera->lookAt({0.18, 0.40, 1.82}, {0.0, 0.02, 0.0}, {0, 1, 0});
    camera->setProjection(34.0, double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);

    Texture* color = Texture::Builder()
                         .width(W)
                         .height(H)
                         .levels(1)
                         .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                         .format(Texture::InternalFormat::RGBA8)
                         .build(*engine);
    Texture* depth = Texture::Builder()
                         .width(W)
                         .height(H)
                         .levels(1)
                         .usage(Texture::Usage::DEPTH_ATTACHMENT)
                         .format(Texture::InternalFormat::DEPTH24)
                         .build(*engine);
    RenderTarget* rt = RenderTarget::Builder()
                           .texture(RenderTarget::AttachmentPoint::COLOR, color)
                           .texture(RenderTarget::AttachmentPoint::DEPTH, depth)
                           .build(*engine);

    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    ColorGrading* grading = ColorGrading::Builder().toneMapper(&agx).contrast(1.06f).saturation(1.14f).build(*engine);

    auto ibl_bytes = load_file("out/ibl/ibl_ibl.ktx");
    auto sky_bytes = load_file("out/ibl/ibl_skybox.ktx");
    if (ibl_bytes.empty() || sky_bytes.empty()) return fail("ktx");
    Texture* ibl_tex = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(ibl_bytes.data(), (uint32_t)ibl_bytes.size()), false);
    Texture* sky_tex = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(sky_bytes.data(), (uint32_t)sky_bytes.size()), false);
    if (!ibl_tex || !sky_tex) return fail("cubemap");
    float3 sh[9];
    if (!load_sh("out/ibl/sh.txt", sh)) return fail("sh");
    const float ibl_intensity = 9000.f;
    IndirectLight* ibl = IndirectLight::Builder().reflections(ibl_tex).irradiance(3, sh).intensity(ibl_intensity).build(*engine);
    Skybox* sky = Skybox::Builder().environment(sky_tex).showSun(false).build(*engine);
    scene->setIndirectLight(ibl);
    scene->setSkybox(sky);

    view->setScene(scene);
    view->setCamera(camera);
    view->setRenderTarget(rt);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setDithering(View::Dithering::NONE);
    view->setColorGrading(grading);
    ScreenSpaceReflectionsOptions ssr;
    ssr.enabled = true;
    ssr.thickness = 0.12f;
    ssr.bias = 0.02f;
    ssr.maxDistance = 6.0f;
    ssr.stride = 2.0f;
    view->setScreenSpaceReflectionsOptions(ssr);
    GuardBandOptions guard;
    guard.enabled = true;
    view->setGuardBandOptions(guard);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0.01f, 0.012f, 0.016f, 1};
    clear.clear = true;
    renderer->setClearOptions(clear);

    auto pkg = [](const char* p) { return load_file(p); };
    auto glass_pkg = pkg("glass.filamat");
    auto pop_pkg = pkg("glass_pop.filamat");
    auto chrome_pkg = pkg("chrome_aniso.filamat");
    auto emit_pkg = pkg("emissive.filamat");
    auto floor_pkg = pkg("floor_lit.filamat");
    if (glass_pkg.empty() || pop_pkg.empty() || chrome_pkg.empty() || emit_pkg.empty() || floor_pkg.empty()) return fail("filamat");
    Material* glass_mat = Material::Builder().package(glass_pkg.data(), glass_pkg.size()).build(*engine);
    Material* pop_mat = Material::Builder().package(pop_pkg.data(), pop_pkg.size()).build(*engine);
    Material* chrome_mat = Material::Builder().package(chrome_pkg.data(), chrome_pkg.size()).build(*engine);
    Material* emit_mat = Material::Builder().package(emit_pkg.data(), emit_pkg.size()).build(*engine);
    Material* floor_mat = Material::Builder().package(floor_pkg.data(), floor_pkg.size()).build(*engine);
    if (!glass_mat || !pop_mat || !chrome_mat || !emit_mat || !floor_mat) return fail("material");

    auto glass_of = [&](Material* mat, float ior, float thick, float dispersion) {
        MaterialInstance* mi = mat->createInstance();
        mi->setParameter("ior", ior);
        mi->setParameter("transmission", 1.0f);
        mi->setParameter("thickness", thick);
        mi->setParameter("dispersion", dispersion);
        return mi;
    };
    MaterialInstance* glass = glass_of(glass_mat, 2.15f, 0.85f, 1.0f);
    MaterialInstance* amber = glass_of(pop_mat, 1.85f, 0.48f, 0.55f);
    MaterialInstance* chrome = chrome_mat->createInstance();
    chrome->setParameter("anisotropy", 0.9f);
    chrome->setParameter("roughness", 0.28f);
    MaterialInstance* chrome_sharp = chrome_mat->createInstance();
    chrome_sharp->setParameter("anisotropy", 0.0f);
    chrome_sharp->setParameter("roughness", 0.28f);
    MaterialInstance* brushed = chrome_mat->createInstance();
    brushed->setParameter("anisotropy", 0.92f);
    brushed->setParameter("roughness", 0.32f);
    auto emit = [&](float r, float g, float b) {
        MaterialInstance* mi = emit_mat->createInstance();
        mi->setParameter("color", float4{r, g, b, 1});
        return mi;
    };
    MaterialInstance* core = emit(140000.f, 42000.f, 9000.f);
    MaterialInstance* filament_w = emit(90000.f, 98000.f, 70000.f);
    MaterialInstance* filament_c = emit(6000.f, 70000.f, 160000.f);
    MaterialInstance* bulb_mi = emit(480000.f, 420000.f, 320000.f);
    MaterialInstance* window_mi = emit(220000.f, 86000.f, 22000.f);
    MaterialInstance* slit_mi = emit(4000.f, 80000.f, 150000.f);
    auto floor_of = [&](float4 color, float rough) {
        MaterialInstance* mi = floor_mat->createInstance();
        mi->setParameter("color", color);
        mi->setParameter("roughness", rough);
        return mi;
    };
    MaterialInstance* floor_mi = floor_of({0.11f, 0.10f, 0.115f, 1}, 0.12f);
    MaterialInstance* panel_mi = floor_of({0.045f, 0.042f, 0.055f, 1}, 0.55f);
    MaterialInstance* wall_mi = floor_of({0.030f, 0.027f, 0.034f, 1}, 0.8f);
    MaterialInstance* mullion_mi = floor_of({0.012f, 0.008f, 0.006f, 1}, 0.9f);

    auto add = [&](Built mesh, MaterialInstance* mi) {
        Hold h;
        h.mesh = std::move(mesh);
        uint32_t nv = (uint32_t)(h.mesh.v.size() / 7);
        h.vb = VertexBuffer::Builder()
                   .vertexCount(nv)
                   .bufferCount(1)
                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                   .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                   .build(*engine);
        h.vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(h.mesh.v.data(), h.mesh.v.size() * sizeof(float)));
        h.ib = IndexBuffer::Builder().indexCount((uint32_t)h.mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        h.ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(h.mesh.idx.data(), h.mesh.idx.size() * sizeof(uint32_t)));
        h.entity = utils::EntityManager::get().create();
        RenderableManager::Builder(1)
            .boundingBox({{-4, -4, -4}, {4, 4, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, h.vb, h.ib)
            .material(0, mi)
            .culling(false)
            .castShadows(false)
            .receiveShadows(false)
            .build(*engine, h.entity);
        scene->addEntity(h.entity);
        return h;
    };
    auto drop = [&](std::vector<Hold>& hs) {
        for (Hold& h : hs) {
            scene->remove(h.entity);
            engine->destroy(h.entity);
            engine->destroy(h.vb);
            engine->destroy(h.ib);
            utils::EntityManager::get().destroy(h.entity);
        }
        hs.clear();
    };

    std::vector<Hold> room;
    room.push_back(add(quad({0, -0.64f, 0.15f}, {1, 0, 0}, {0, 0, -1}, 2.8f, 2.2f), floor_mi));
    room.push_back(add(quad({0.0f, 0.35f, -1.35f}, {1, 0, 0}, {0, 1, 0}, 2.4f, 1.5f), wall_mi));
    room.push_back(add(quad({-1.05f, 0.25f, 0.05f}, {0, 0, -1}, {0, 1, 0}, 1.15f, 1.2f), panel_mi));
    room.push_back(add(quad({1.28f, 0.22f, -0.05f}, {0, 0, 1}, {0, 1, 0}, 1.15f, 1.2f), panel_mi));
    room.push_back(add(quad({-0.42f, 0.34f, -1.22f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 1.05f), window_mi));
    room.push_back(add(quad({-0.42f, 0.34f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 0.014f, 1.05f), mullion_mi));
    room.push_back(add(quad({-0.95f, 0.34f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 0.012f, 1.05f), mullion_mi));
    room.push_back(add(quad({0.12f, 0.34f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 0.012f, 1.05f), mullion_mi));
    room.push_back(add(quad({-0.42f, 0.82f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 0.012f), mullion_mi));
    room.push_back(add(quad({-0.42f, -0.08f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 0.012f), mullion_mi));
    room.push_back(add(quad({1.05f, 0.15f, 0.55f}, norm(V3{-0.15f, 0, 0.2f}), {0, 1, 0}, 0.028f, 0.85f), slit_mi));
    room.push_back(add(quad({0.28f, 0.72f, 0.42f}, {1, 0, 0}, {0, 1, 0}, 0.04f, 0.04f), bulb_mi));

    auto lamp = [&](float3 color, float lux, float3 dir) {
        utils::Entity e = utils::EntityManager::get().create();
        LightManager::Builder(LightManager::Type::DIRECTIONAL)
            .color(color)
            .intensity(lux)
            .direction(normalize(dir))
            .castShadows(false)
            .build(*engine, e);
        scene->addEntity(e);
        return e;
    };
    utils::Entity key = lamp({1.0f, 0.78f, 0.48f}, 4200.f, {0.42f, -0.22f, 0.88f});
    utils::Entity fill = lamp({0.45f, 0.62f, 1.0f}, 900.f, {-0.75f, -0.18f, -0.35f});
    (void)key;
    (void)fill;

    auto constant = [](float r) { return [r](float) { return r; }; };
    auto vary = [](float a, float b, float freq, float phase) {
        return [=](float u) { return a + (b - a) * (0.5f + 0.5f * std::sin(u * TAU * freq + phase)); };
    };

    struct Shot {
        const char* name;
        float bloom;
        float highlight;
        std::function<void(std::vector<Hold>&)> build;
    };
    Shot shots[5] = {
        {"glass", 0.22f, 8.0f,
         [&](std::vector<Hold>& hs) {
             hs.push_back(add(tube(path, 48, constant(0.078f)), glass));
             hs.push_back(add(tube(scaled(path, 0.58f, {0.04f, 0.03f, 0.05f}), 40, constant(0.036f)), amber));
         }},
        {"chrome", 0.16f, 10.0f,
         [&](std::vector<Hold>& hs) {
             hs.push_back(add(tube(nudged(path, -0.09f, 0), 40, constant(0.024f)), chrome));
             hs.push_back(add(tube(nudged(path, 0.09f, 0), 40, constant(0.024f)), chrome_sharp));
         }},
        {"emissive", 0.55f, 14.0f,
         [&](std::vector<Hold>& hs) {
             hs.push_back(add(ribbon(path, 0.0f, vary(0.004f, 0.012f, 3.f, 0.f)), core));
             hs.push_back(add(ribbon(nudged(path, 0.028f, 0.01f), 1.5f, vary(0.003f, 0.008f, 2.f, 1.f)), filament_w));
             hs.push_back(add(ribbon(nudged(path, -0.02f, -0.02f), 2.0f, vary(0.003f, 0.007f, 2.f, 2.f)), filament_c));
         }},
        {"ribbon", 0.20f, 8.0f,
         [&](std::vector<Hold>& hs) {
             hs.push_back(add(ribbon(nudged(path, 0.0f, 0.16f), 2.5f, vary(0.012f, 0.072f, 2.f, 0.4f)), brushed));
             hs.push_back(add(sweep_rect(nudged(path, 0.0f, -0.16f), 0.75f, vary(0.03f, 0.11f, 1.f, 0.2f),
                                         vary(0.012f, 0.045f, 2.f, 1.1f)),
                              glass));
         }},
        {"composite", 0.42f, 12.0f,
         [&](std::vector<Hold>& hs) {
             hs.push_back(add(sweep_rect(scaled(path, 1.18f, {0, -0.02f, 0}), 0.35f, vary(0.02f, 0.055f, 1.f, 0.f),
                                         constant(0.012f)),
                              glass));
             hs.push_back(add(tube(path, 48, constant(0.064f)), glass));
             hs.push_back(add(tube(scaled(path, 0.60f, {0.04f, 0.03f, 0.05f}), 36, constant(0.026f)), amber));
             hs.push_back(add(tube(nudged(path, 0.075f, 0.02f), 28, constant(0.012f)), chrome));
             hs.push_back(add(ribbon(path, 1.0f, vary(0.0035f, 0.009f, 3.f, 0.5f)), core));
         }},
    };

    FILE* log = std::fopen("out/limit.txt", "w");
    std::fprintf(log, "1920x1080  exposure f/16 1/125 ISO100  ibl %.0f lux  AgX punchy\n", ibl_intensity);
    std::fprintf(log, "path rings %d length %.3f kappa %.3f r*kappa %.3f separation %.3f\n", n, L, kappa, glass_r * kappa, sep);
    auto ms = [](std::chrono::steady_clock::duration d) {
        return std::chrono::duration<double, std::milli>(d).count();
    };
    bool png_ok = true;
    for (Shot& shot : shots) {
        BloomOptions bloom;
        bloom.enabled = true;
        bloom.strength = shot.bloom;
        bloom.levels = 7;
        bloom.resolution = 1080;
        bloom.threshold = true;
        bloom.highlight = shot.highlight;
        bloom.blendMode = BloomOptions::BlendMode::ADD;
        view->setBloomOptions(bloom);
        std::vector<Hold> hs;
        shot.build(hs);
        for (int i = 0; i < 2; ++i) {
            if (!renderer->beginFrame(swap)) {
                --i;
                continue;
            }
            renderer->render(view);
            renderer->endFrame();
        }
        engine->flushAndWait();
        std::vector<double> bare;
        for (int i = 0; i < 5; ++i) {
            auto t0 = std::chrono::steady_clock::now();
            while (!renderer->beginFrame(swap)) {}
            renderer->render(view);
            renderer->endFrame();
            engine->flushAndWait();
            bare.push_back(ms(std::chrono::steady_clock::now() - t0));
        }
        std::vector<uint8_t> pic((size_t)W * H * 4);
        std::vector<double> full;
        for (int i = 0; i < 3; ++i) {
            auto t1 = std::chrono::steady_clock::now();
            while (!renderer->beginFrame(swap)) {}
            renderer->render(view);
            renderer->readPixels(rt, 0, 0, (uint32_t)W, (uint32_t)H,
                                 backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                                backend::PixelDataType::UBYTE));
            renderer->endFrame();
            engine->flushAndWait();
            full.push_back(ms(std::chrono::steady_clock::now() - t1));
        }
        std::printf("  bare");
        for (double t : bare) std::printf(" %.2f", t);
        std::printf("  full");
        for (double t : full) std::printf(" %.2f", t);
        std::printf("\n");
        std::sort(bare.begin(), bare.end());
        std::sort(full.begin(), full.end());
        double draw = bare[bare.size() / 2];
        double with_read = full[full.size() / 2];
        double sum = 0;
        int hot = 0, dead = 0, peak = 0;
        for (int i = 0; i < W * H; ++i) {
            int r = pic[i * 4], g = pic[i * 4 + 1], b = pic[i * 4 + 2];
            sum += (r + g + b) / 3.0;
            peak = std::max(peak, std::max(r, std::max(g, b)));
            if (r > 248 && g > 248 && b > 248) ++hot;
            if (r < 3 && g < 3 && b < 3) ++dead;
        }
        std::string file = std::string("out/limit-") + shot.name + ".png";
        if (!write_png(file.c_str(), W, H, pic.data())) png_ok = false;
        std::printf("%-10s  draw %.2f ms  with-readback %.2f ms  readback %.2f ms  mean %.1f  peak %d  hot %.2f%%  dead %.1f%%\n",
                    shot.name, draw, with_read, with_read - draw, sum / (W * H), peak, 100.0 * hot / (W * H), 100.0 * dead / (W * H));
        std::fprintf(log, "%s draw_ms %.3f with_readback_ms %.3f readback_ms %.3f mean %.2f peak %d hot_pct %.3f dead_pct %.2f\n",
                     shot.name, draw, with_read, with_read - draw, sum / (W * H), peak, 100.0 * hot / (W * H), 100.0 * dead / (W * H));
        drop(hs);
    }
    std::fclose(log);
    if (!png_ok) return fail("png");
    std::printf("PASS beauty\n");
    return 0;
}
