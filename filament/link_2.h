// link.mm, part 2 (cut there by size, as it was): read only through link.mm, in order.
#pragma once


int main() {
    if (tvg::Initializer::init(0) != tvg::Result::Success) return fail("thorvg");
    tvg::Shape* shape = tvg::Shape::gen();
    const Path2 source = rectangle();

    auto glass_pkg = load_file("glass.filamat");
    auto chrome_pkg = load_file("chrome.filamat");
    auto solid_pkg = load_file("solid.filamat");
    auto stripe_pkg = load_file("stripes.filamat");
    if (glass_pkg.empty() || chrome_pkg.empty() || solid_pkg.empty() || stripe_pkg.empty()) return fail("filamat");

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    Camera* camera = engine->createCamera(utils::EntityManager::get().create());
    camera->setExposure(1.f);
    camera->lookAt({0.85, 0.72, 3.7}, {0, 0, 0}, {0, 1, 0});
    camera->setProjection(32.0, double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);

    Texture* color = Texture::Builder().width(W).height(H).levels(1)
                         .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                         .format(Texture::InternalFormat::RGBA8).build(*engine);
    Texture* depth = Texture::Builder().width(W).height(H).levels(1)
                         .usage(Texture::Usage::DEPTH_ATTACHMENT).format(Texture::InternalFormat::DEPTH24).build(*engine);
    RenderTarget* rt = RenderTarget::Builder()
                           .texture(RenderTarget::AttachmentPoint::COLOR, color)
                           .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
    static LinearToneMapper tone;
    view->setColorGrading(ColorGrading::Builder().toneMapper(&tone).contrast(1.18f).saturation(1.28f).build(*engine));
    scene->setSkybox(Skybox::Builder().color({0.015f, 0.016f, 0.02f, 1}).build(*engine));
    view->setScene(scene);
    view->setCamera(camera);
    view->setRenderTarget(rt);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setDithering(View::Dithering::NONE);
    BloomOptions bloom;
    bloom.enabled = true;
    bloom.strength = 0.28f;
    bloom.levels = 8;
    bloom.resolution = 768;
    bloom.threshold = true;
    bloom.highlight = 6.f;
    view->setBloomOptions(bloom);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0.015f, 0.016f, 0.02f, 1};
    clear.clear = true;
    renderer->setClearOptions(clear);

    Material* glass_mat = Material::Builder().package(glass_pkg.data(), glass_pkg.size()).build(*engine);
    Material* chrome_mat = Material::Builder().package(chrome_pkg.data(), chrome_pkg.size()).build(*engine);
    Material* solid_mat = Material::Builder().package(solid_pkg.data(), solid_pkg.size()).build(*engine);
    Material* stripe_mat = Material::Builder().package(stripe_pkg.data(), stripe_pkg.size()).build(*engine);
    MaterialInstance* glass = glass_mat->createInstance();
    glass->setParameter("ior", 2.15f);
    glass->setParameter("transmission", 1.f);
    glass->setParameter("thickness", 0.62f);
    glass->setParameter("dispersion", 0.72f);
    MaterialInstance* chrome = chrome_mat->createInstance();
    MaterialInstance* flat = solid_mat->createInstance();
    flat->setParameter("color", float4{0.95f, 0.18f, 0.32f, 1.f});
    MaterialInstance* wall_mi = stripe_mat->createInstance();

    const int n = 64;
    auto* bytes = new std::vector<uint8_t>((size_t)n * n * 4);
    const uint8_t band[4][3] = {{186, 28, 36}, {214, 148, 28}, {28, 78, 196}, {248, 248, 252}};
    for (int y = 0; y < n; ++y)
        for (int x = 0; x < n; ++x) {
            int b = (x * 4) / n;
            size_t i = ((size_t)y * n + x) * 4;
            (*bytes)[i] = band[b][0]; (*bytes)[i + 1] = band[b][1]; (*bytes)[i + 2] = band[b][2]; (*bytes)[i + 3] = 255;
        }
    Texture* wall_tex = Texture::Builder().width(n).height(n).levels(1).sampler(Texture::Sampler::SAMPLER_2D)
                            .format(Texture::InternalFormat::RGBA8).build(*engine);
    wall_tex->setImage(*engine, 0, Texture::PixelBufferDescriptor(bytes->data(), bytes->size(),
        backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE, 1, 0, 0, n,
        [](void*, size_t, void* user) { delete static_cast<std::vector<uint8_t>*>(user); }, bytes));
    wall_mi->setParameter("albedo", wall_tex, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));

    static const float wall[] = {
        -1.9f, -0.85f, -0.9f, 0, 0, 1.9f, -0.85f, -0.9f, 1, 0, 1.9f, 1.15f, -0.9f, 1, 1, -1.9f, 1.15f, -0.9f, 0, 1,
    };
    static const uint16_t quad[] = {0, 1, 2, 0, 2, 3};
    VertexBuffer* qvb = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                            .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                            .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
    qvb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(wall, sizeof(wall)));
    IndexBuffer* qib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::USHORT).build(*engine);
    qib->setBuffer(*engine, IndexBuffer::BufferDescriptor(quad, sizeof(quad)));
    utils::Entity qe = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-4, -4, -4}, {4, 4, 4}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, qvb, qib)
        .material(0, wall_mi).culling(false).build(*engine, qe);
    scene->addEntity(qe);

    Mesh mesh;
    float radius = 0;
    if (!build_mesh(shape, 0, source, mesh, radius)) return fail("mesh");
    const int vcount = (int)(mesh.v.size() / STRIDE);
    VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)vcount).bufferCount(1)
                           .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, STRIDE * 4)
                           .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, STRIDE * 4)
                           .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 28, STRIDE * 4)
                           .build(*engine);
    vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * 4));
    IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)mesh.idx.size()).bufferType(IndexBuffer::IndexType::USHORT).build(*engine);
    ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(mesh.idx.data(), mesh.idx.size() * 2));
    utils::Entity tube = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
        .material(0, glass).culling(false).castShadows(false).receiveShadows(false).build(*engine, tube);
    scene->addEntity(tube);

    auto sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN).color(float3{1, 0.98f, 0.94f}).intensity(42.f)
        .direction(normalize(float3{0.15f, -0.55f, -0.82f})).sunAngularRadius(0.35f).castShadows(false).build(*engine, sun);
    auto fill = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::DIRECTIONAL).color(float3{1, 0.62f, 0.32f}).intensity(18.f)
        .direction(normalize(float3{-0.85f, -0.15f, 0.45f})).castShadows(false).build(*engine, fill);
    auto rim = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::DIRECTIONAL).color(float3{0.45f, 0.75f, 1}).intensity(14.f)
        .direction(normalize(float3{0.7f, -0.2f, 0.55f})).castShadows(false).build(*engine, rim);
    scene->addEntity(sun);
    scene->addEntity(fill);
    scene->addEntity(rim);

    auto draw = [&](std::vector<uint8_t>* picture) -> bool {
        bool drew = false;
        for (int attempt = 0; attempt < 4 && !drew; ++attempt) {
            if (!renderer->beginFrame(swap)) continue;
            renderer->render(view);
            if (picture) {
                picture->resize((size_t)W * H * 4);
                renderer->readPixels(rt, 0, 0, W, H, backend::PixelBufferDescriptor(
                    picture->data(), picture->size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
            }
            renderer->endFrame();
            drew = true;
        }
        engine->flushAndWait();
        return drew;
    };

    const int FRAMES = 60;
    std::vector<double> times;
    times.reserve(FRAMES);
    float rmin = 1e9f, rmax = 0;
    double spine_max = 0;
    int followed = 0;
    for (int warm = 0; warm < 2; ++warm) {
        if (!draw(nullptr)) return fail("warmup");
    }
    for (int frame = 0; frame < FRAMES; ++frame) {
        float amount = 30.f * std::sin((float)frame / FRAMES * 6.2831853f);
        auto t0 = std::chrono::steady_clock::now();
        if (!build_mesh(shape, amount, source, mesh, radius)) return fail("live mesh");
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * 4));
        if (!draw(nullptr)) return fail("live frame");
        times.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
        rmin = std::min(rmin, radius);
        rmax = std::max(rmax, radius);
        spine_max = std::max(spine_max, spine_error(mesh));
        ++followed;
    }
    std::vector<double> sorted = times;
    std::sort(sorted.begin(), sorted.end());
    double median = sorted[sorted.size() / 2];
    int under = 0;
    for (double t : times)
        if (t <= 8.33) ++under;

    const float shots[4] = {-30.f, -10.f, 10.f, 30.f};
    std::vector<uint8_t> sheet((size_t)1920 * 540 * 4, 0);
    auto blit = [&](const std::vector<uint8_t>& src, int sw, int sh, int col, int row) {
        for (int y = 0; y < 270; ++y) {
            for (int x = 0; x < 480; ++x) {
                int sx = x * sw / 480, sy = y * sh / 270;
                const uint8_t* s = src.data() + ((size_t)sy * sw + sx) * 4;
                uint8_t* d = sheet.data() + ((size_t)(row * 270 + y) * 1920 + col * 480 + x) * 4;
                std::memcpy(d, s, 4);
            }
        }
    };
    for (int s = 0; s < 4; ++s) {
        if (!build_mesh(shape, shots[s], source, mesh, radius)) return fail("shot mesh");
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * 4));
        engine->flushAndWait();
        std::vector<uint8_t> picture;
        auto t0 = std::chrono::steady_clock::now();
        if (!draw(&picture)) return fail("shot");
        double shot_ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
        std::printf("shot amount %+.0f  anchor-radius %.4f  spine %.3g  with-readback %.2f ms\n", shots[s], radius, spine_error(mesh), shot_ms);
        blit(picture, W, H, s, 0);
        std::vector<uint8_t> d2;
        Path2 stored;
        read_path(shape, stored);
        if (!raster_2d(stored, d2)) return fail("2d");
        if (!d2.empty()) blit(d2, 480, 270, s, 1);
        if (s == 3 && !write_png("out/link-live.png", W, H, picture.data())) return fail("png");
    }
    if (!write_png("out/link-sheet.png", 1920, 540, sheet.data())) return fail("sheet");

    if (!build_mesh(shape, 0, source, mesh, radius)) return fail("rest mesh");
    vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * 4));
    engine->flushAndWait();
    uint64_t hash = position_hash(mesh);
    auto& rm = engine->getRenderableManager();
    auto inst = rm.getInstance(tube);
    struct MatShot { const char* name; const char* file; MaterialInstance* mi; };
    MatShot mats[3] = {{"glass", "out/link-glass.png", glass}, {"chrome", "out/link-chrome.png", chrome}, {"flat", "out/link-solid.png", flat}};
    for (MatShot& m : mats) {
        if (position_hash(mesh) != hash) return fail("mesh changed before material swap");
        rm.setMaterialInstanceAt(inst, 0, m.mi);
        std::vector<uint8_t> picture;
        if (!draw(&picture)) return fail(m.name);
        if (!write_png(m.file, W, H, picture.data())) return fail("png");
        std::printf("material %s  vertices %d  hash %llx\n", m.name, vcount, (unsigned long long)hash);
    }
    if (position_hash(mesh) != hash) return fail("mesh changed after material swap");

    int led_w = 0, led_h = 0;
    std::vector<uint8_t> led;
    bool wgsl = read_png("out/tri_led.png", led_w, led_h, led);
    if (wgsl) {
        auto* owned = new std::vector<uint8_t>(std::move(led));
        Texture* led_tex = Texture::Builder().width((uint32_t)led_w).height((uint32_t)led_h).levels(1)
                               .sampler(Texture::Sampler::SAMPLER_2D).format(Texture::InternalFormat::RGBA8).build(*engine);
        led_tex->setImage(*engine, 0, Texture::PixelBufferDescriptor(owned->data(), owned->size(),
            backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE, 1, 0, 0, (uint32_t)led_w,
            [](void*, size_t, void* user) { delete static_cast<std::vector<uint8_t>*>(user); }, owned));
        MaterialInstance* wg = stripe_mat->createInstance();
        wg->setParameter("albedo", led_tex, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
        rm.setMaterialInstanceAt(inst, 0, wg);
        std::vector<uint8_t> picture;
        if (!draw(&picture)) return fail("wgsl material");
        if (position_hash(mesh) != hash) return fail("mesh changed for wgsl");
        if (!write_png("out/link-wgsl.png", W, H, picture.data())) return fail("png");
        std::printf("material wgsl-tri_led  hash %llx\n", (unsigned long long)hash);
    }

    FILE* log = std::fopen("out/link.txt", "w");
    std::fprintf(log,
        "scene  one rectangle, ThorVG pucker each frame, one tube\n"
        "1 path  %d/%d frames  spine-max %.3g  anchor-radius %.4f .. %.4f\n"
        "2 material  same vertex hash %llx  glass chrome flat%s\n"
        "3 frame  1920x1080  %d frames  path+upload+glass+bloom+fxaa+refraction  no readback\n"
        "   min %.2f  median %.2f  max %.2f ms   under 8.33ms %d/%d\n"
        "   target 8.33ms is 120fps. This number is the measured median, not a claim.\n"
        "4 wgsl  %s\n",
        followed, FRAMES, spine_max, rmin, rmax,
        (unsigned long long)hash, wgsl ? " + tri_led texture" : "",
        FRAMES, sorted.front(), median, sorted.back(), under, FRAMES,
        wgsl ? "tri_led.wgsl via wgpu (vgpu MIT), same mesh, stripes.mat samples it" : "tri_led.png missing");
    std::fclose(log);
    std::printf("median %.2f ms  under 8.33 %d/%d  wgsl %s\nPASS link\n", median, under, FRAMES, wgsl ? "yes" : "no");
    return 0;
}
