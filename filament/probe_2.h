// probe.mm, part 2 (cut there by size, as it was): read only through probe.mm, in order.
#pragma once


static Texture* stripe_texture(Engine& engine) {
    const int n = 256;
    auto* bytes = new std::vector<uint8_t>((size_t)n * n * 4);
    const uint8_t band[8][3] = {
        {186, 28, 36}, {214, 148, 28}, {18, 150, 86}, {28, 78, 196},
        {248, 248, 252}, {168, 32, 150}, {12, 156, 176}, {214, 92, 28},
    };
    for (int y = 0; y < n; ++y) {
        for (int x = 0; x < n; ++x) {
            int b = std::min(7, x * 8 / n);
            int local = x - b * (n / 8);
            bool core = local > 10 && local < 18;
            size_t i = ((size_t)y * n + x) * 4;
            (*bytes)[i] = core ? 255 : band[b][0];
            (*bytes)[i + 1] = core ? 255 : band[b][1];
            (*bytes)[i + 2] = core ? 255 : band[b][2];
            (*bytes)[i + 3] = 255;
        }
    }
    Texture* tex = Texture::Builder()
                       .width(n)
                       .height(n)
                       .levels(1)
                       .sampler(Texture::Sampler::SAMPLER_2D)
                       .format(Texture::InternalFormat::RGBA8)
                       .build(engine);
    tex->setImage(engine, 0,
        Texture::PixelBufferDescriptor(bytes->data(), bytes->size(), backend::PixelDataFormat::RGBA,
            backend::PixelDataType::UBYTE, 1, 0, 0, n,
            [](void*, size_t, void* user) { delete static_cast<std::vector<uint8_t>*>(user); }, bytes));
    return tex;
}

int main() {
    if (tvg::Initializer::init(0) != tvg::Result::Success) return fail("thorvg init");
    tvg::Shape* contour = tvg::Shape::gen();
    if (!contour) return fail("shape");

    const Path2 source = rectangle();
    const struct State {
        const char* name;
        const char* file;
        float amount;
    } states[3] = {
        {"rest", "rest", 0},
        {"pucker", "pucker", 30},
        {"bloat", "bloat", -30},
    };

    Engine* engine = Engine::Builder()
                         .backend(Engine::Backend::METAL)
                         .featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3)
                         .build();
    if (!engine) return fail("filament engine");
    std::printf("feature level %d\n", (int)engine->getActiveFeatureLevel());
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(1.0f);
    camera->lookAt({0.85, 0.72, 3.7}, {0, 0, 0}, {0, 1, 0});
    camera->setProjection(32.0, double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);

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

    static LinearToneMapper linear_tone;
    ColorGrading* grading = ColorGrading::Builder()
                                .toneMapper(&linear_tone)
                                .contrast(1.18f)
                                .saturation(1.28f)
                                .build(*engine);
    Skybox* sky = Skybox::Builder().color({0.015f, 0.016f, 0.02f, 1}).build(*engine);
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
    BloomOptions bloom;
    bloom.enabled = true;
    bloom.strength = 0.28f;
    bloom.levels = 8;
    bloom.resolution = 768;
    bloom.threshold = true;
    bloom.highlight = 6.0f;
    bloom.blendMode = BloomOptions::BlendMode::ADD;
    view->setBloomOptions(bloom);
    VignetteOptions vignette;
    vignette.enabled = true;
    vignette.midPoint = 0.62f;
    vignette.roundness = 0.55f;
    vignette.feather = 0.45f;
    vignette.color = {0.0f, 0.0f, 0.0f, 1.0f};
    view->setVignetteOptions(vignette);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0.015f, 0.016f, 0.02f, 1};
    clear.clear = true;
    renderer->setClearOptions(clear);

    auto sun_pkg = load_file("glass.filamat");
    auto stripe_pkg = load_file("stripes.filamat");
    auto solid_pkg = load_file("solid.filamat");
    if (sun_pkg.empty() || stripe_pkg.empty() || solid_pkg.empty()) return fail("filamat");
    Material* glass_mat = Material::Builder().package(sun_pkg.data(), sun_pkg.size()).build(*engine);
    Material* stripe_mat = Material::Builder().package(stripe_pkg.data(), stripe_pkg.size()).build(*engine);
    Material* solid_mat = Material::Builder().package(solid_pkg.data(), solid_pkg.size()).build(*engine);
    if (!glass_mat || !stripe_mat || !solid_mat) return fail("material");
    MaterialInstance* glass = glass_mat->createInstance();
    glass->setParameter("ior", GLASS_IOR);
    glass->setParameter("transmission", GLASS_TRANSMISSION);
    glass->setParameter("thickness", GLASS_THICKNESS);
    glass->setParameter("dispersion", GLASS_DISPERSION);
    MaterialInstance* stripes = stripe_mat->createInstance();
    Texture* tex = stripe_texture(*engine);
    stripes->setParameter("albedo", tex, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
    MaterialInstance* floor_mi = solid_mat->createInstance();
    floor_mi->setParameter("color", float4{0.012f, 0.013f, 0.016f, 1});

    auto add_pos_uv = [&](const float* data, int nvert, int stride, const uint16_t* idx, int nidx, MaterialInstance* mi) {
        VertexBuffer* vb = VertexBuffer::Builder()
                               .vertexCount((uint32_t)nvert)
                               .bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, (uint8_t)stride)
                               .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, (uint8_t)stride)
                               .build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(data, (size_t)nvert * stride));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)nidx).bufferType(IndexBuffer::IndexType::USHORT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(idx, (size_t)nidx * sizeof(uint16_t)));
        utils::Entity e = utils::EntityManager::get().create();
        RenderableManager::Builder(1)
            .boundingBox({{-4, -4, -4}, {4, 4, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi)
            .culling(false)
            .receiveShadows(false)
            .castShadows(false)
            .build(*engine, e);
        scene->addEntity(e);
        return e;
    };
    // Backdrop stands behind the tube. Floor sits under the lowest bloat.
    static const float wall[] = {
        -1.9f, -0.85f, -0.9f, 0, 0, 1.9f, -0.85f, -0.9f, 1, 0, 1.9f, 1.15f, -0.9f, 1, 1, -1.9f, 1.15f, -0.9f, 0, 1,
    };
    static const float ground[] = {
        -2.3f, -0.82f, 2.4f, 0, 0, 2.3f, -0.82f, 2.4f, 1, 0, 2.3f, -0.82f, -0.9f, 1, 1, -2.3f, -0.82f, -0.9f, 0, 1,
    };
    static const uint16_t quad[] = {0, 1, 2, 0, 2, 3};
    add_pos_uv(wall, 4, 20, quad, 6, stripes);
    add_pos_uv(ground, 4, 20, quad, 6, floor_mi);

    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1.0f, 0.98f, 0.94f})
        .intensity(42.0f)
        .direction(normalize(float3{0.15f, -0.55f, -0.82f}))
        .sunAngularRadius(0.35f)
        .castShadows(false)
        .build(*engine, sun);
    utils::Entity fill = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::DIRECTIONAL)
        .color(float3{1.0f, 0.62f, 0.32f})
        .intensity(18.0f)
        .direction(normalize(float3{-0.85f, -0.15f, 0.45f}))
        .castShadows(false)
        .build(*engine, fill);
    utils::Entity rim = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::DIRECTIONAL)
        .color(float3{0.45f, 0.75f, 1.0f})
        .intensity(14.0f)
        .direction(normalize(float3{0.7f, -0.2f, 0.55f}))
        .castShadows(false)
        .build(*engine, rim);
    scene->addEntity(sun);
    scene->addEntity(fill);
    scene->addEntity(rim);

    std::printf("glass ior %.2f  transmission %.2f  thickness %.2f  dispersion %.2f\n", GLASS_IOR,
        GLASS_TRANSMISSION, GLASS_THICKNESS, GLASS_DISPERSION);
    std::printf("camera eye (0.85, 0.72, 3.70)  target origin  fov 32 deg\n");

    std::vector<uint8_t> pictures[3];
    std::vector<uint8_t> strokes[3];
    float radii[3] = {};
    bool ok = true;

    for (int s = 0; s < 3; ++s) {
        Path2 deformed = pucker_bloat(source, states[s].amount);
        if (!write_path(contour, deformed)) return fail("appendPath");
        Path2 stored;
        if (!read_path(contour, stored)) return fail("path()");
        if (!same_path(stored, deformed)) return fail("readback differs from written path");
        std::vector<Sample> samples = sample_path(stored);
        if (samples.size() < 16) return fail("sample");
        Mesh mesh = tube_from(samples);
        double spine = 0;
        for (size_t i = 0; i < samples.size(); ++i) {
            float3 d = mesh.centers[i] - samples[i].p;
            spine = std::max(spine, (double)length(d));
        }
        radii[s] = anchor_mean_radius(stored);
        std::printf("%s amount %+.0f  cmds %zu pts %zu  anchor-radius %.4f  samples %zu  spine-error %.3g\n",
            states[s].name, states[s].amount, stored.cmds.size(), stored.pts.size(), radii[s], samples.size(), spine);
        if (spine > 1e-5) return fail("tube spine left the ThorVG path");

        VertexBuffer* vb = VertexBuffer::Builder()
                               .vertexCount((uint32_t)(mesh.verts.size() / 7))
                               .bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                               .build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.verts.data(), mesh.verts.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder()
                              .indexCount((uint32_t)mesh.idx.size())
                              .bufferType(IndexBuffer::IndexType::USHORT)
                              .build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(mesh.idx.data(), mesh.idx.size() * sizeof(uint16_t)));
        utils::Entity tube = utils::EntityManager::get().create();
        RenderableManager::Builder(1)
            .boundingBox({{-2, -2, -1}, {2, 2, 1}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, glass)
            .culling(false)
            .castShadows(false)
            .receiveShadows(false)
            .build(*engine, tube);
        scene->addEntity(tube);

        pictures[s].resize((size_t)W * H * 4);
        bool drew = false;
        for (int attempt = 0; attempt < 4 && !drew; ++attempt) {
            if (!renderer->beginFrame(swap)) continue;
            renderer->render(view);
            renderer->readPixels(rt, 0, 0, (uint32_t)W, (uint32_t)H,
                backend::PixelBufferDescriptor(pictures[s].data(), pictures[s].size(), backend::PixelDataFormat::RGBA,
                    backend::PixelDataType::UBYTE));
            renderer->endFrame();
            drew = true;
        }
        engine->flushAndWait();
        if (!drew) return fail("beginFrame");

        scene->remove(tube);
        engine->destroy(tube);
        engine->destroy(vb);
        engine->destroy(ib);
        utils::EntityManager::get().destroy(tube);

        if (!raster_2d(stored, strokes[s])) return fail("2d raster empty");
        std::string p3 = std::string("out/") + states[s].file + ".png";
        std::string p2 = std::string("out/") + states[s].file + "-2d.png";
        if (!write_png(p3.c_str(), W, H, pictures[s].data())) return fail("png 3d");
        if (!write_png(p2.c_str(), W2, H2, strokes[s].data())) return fail("png 2d");
    }

    if (!(radii[1] < radii[0] && radii[0] < radii[2])) {
        std::printf("anchor radii rest %.4f pucker %.4f bloat %.4f\n", radii[0], radii[1], radii[2]);
        ok = false;
    }
    double d01 = mean_abs(pictures[0], pictures[1]);
    double d02 = mean_abs(pictures[0], pictures[2]);
    double d12 = mean_abs(pictures[1], pictures[2]);
    int c01 = max_corner_diff(pictures[0], pictures[1]);
    int c02 = max_corner_diff(pictures[0], pictures[2]);
    std::printf("3d mean-abs  rest/pucker %.2f  rest/bloat %.2f  pucker/bloat %.2f\n", d01, d02, d12);
    std::printf("corner invariant rest/pucker %d  rest/bloat %d\n", c01, c02);
    if (d01 < 4 || d02 < 4 || d12 < 4) ok = false;
    if (c01 > 2 || c02 > 2) ok = false;

    std::vector<uint8_t> sheet((size_t)(W * 3) * (H + H2) * 4, 12);
    for (int s = 0; s < 3; ++s) {
        blit(sheet.data(), W * 3, s * W, 0, pictures[s].data(), W, H);
        blit(sheet.data(), W * 3, s * W, H, strokes[s].data(), W2, H2);
    }
    if (!write_png("out/sheet.png", W * 3, H + H2, sheet.data())) return fail("sheet");

    std::printf("%s\n", ok ? "PASS  same path drives the 2D stroke and the glass tube" : "FAIL  pictures or radii did not follow");
    return ok ? 0 : 1;
}
