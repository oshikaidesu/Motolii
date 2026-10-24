// gpu_tube.mm, part 2 (cut there by size, as it was): read only through gpu_tube.mm, in order.
#pragma once


int main() {
    if (tvg::Initializer::init(0) != tvg::Result::Success) return fail("thorvg");
    tvg::Shape* shape = tvg::Shape::gen();
    Path2 deformed = pucker_bloat(rectangle(), 0.f);
    if (!write_path(shape, deformed)) return fail("write");
    Path2 stored;
    if (!read_path(shape, stored)) return fail("read");
    std::vector<Span> spans = spans_of(stored);
    if ((int)spans.size() != SPANS) return fail("span count");

    static std::vector<float> texels_keep;
    texels_keep.resize((size_t)SPANS * 4 * 4);
    auto* texels = &texels_keep;
    for (int s = 0; s < SPANS; ++s) {
        tvg::Point p[4] = {spans[s].a, spans[s].b, spans[s].c, spans[s].d};
        for (int k = 0; k < 4; ++k) {
            (*texels)[((size_t)s * 4 + k) * 4 + 0] = p[k].x;
            (*texels)[((size_t)s * 4 + k) * 4 + 1] = p[k].y;
            (*texels)[((size_t)s * 4 + k) * 4 + 2] = 0.f;
            (*texels)[((size_t)s * 4 + k) * 4 + 3] = 1.f;
        }
    }

    auto gpu_pkg = load_file("gpu_tube.filamat");
    auto glass_pkg = load_file("glass.filamat");
    auto stripe_pkg = load_file("stripes.filamat");
    if (gpu_pkg.empty() || glass_pkg.empty() || stripe_pkg.empty()) return fail("filamat");

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

    Material* gpu_mat = Material::Builder().package(gpu_pkg.data(), gpu_pkg.size()).build(*engine);
    Material* glass_mat = Material::Builder().package(glass_pkg.data(), glass_pkg.size()).build(*engine);
    Material* stripe_mat = Material::Builder().package(stripe_pkg.data(), stripe_pkg.size()).build(*engine);
    if (!gpu_mat || !glass_mat) return fail("material");
    MaterialInstance* gpu_mi = gpu_mat->createInstance();
    gpu_mi->setParameter("tubeRadius", TUBE_R);
    gpu_mi->setParameter("spanCount", (float)SPANS);
    gpu_mi->setParameter("ior", 2.15f);
    gpu_mi->setParameter("transmission", 1.f);
    gpu_mi->setParameter("thickness", 0.62f);
    gpu_mi->setParameter("dispersion", 0.72f);
    MaterialInstance* glass_mi = glass_mat->createInstance();
    glass_mi->setParameter("ior", 2.15f);
    glass_mi->setParameter("transmission", 1.f);
    glass_mi->setParameter("thickness", 0.62f);
    glass_mi->setParameter("dispersion", 0.72f);

    const uint32_t tw = (uint32_t)SPANS * 4;
    Texture* controls = Texture::Builder().width(tw).height(1).levels(1)
                            .sampler(Texture::Sampler::SAMPLER_2D)
                            .format(Texture::InternalFormat::RGBA32F).build(*engine);
    if (!controls) return fail("control texture");
    controls->setImage(*engine, 0, Texture::PixelBufferDescriptor(
        texels->data(), texels->size() * sizeof(float),
        backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT, 1, 0, 0, tw));
    gpu_mi->setParameter("controls", controls, TextureSampler(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST));

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

    std::vector<float> gpu_pos, gpu_uv;
    std::vector<uint16_t> gpu_idx;
    gpu_topology(spans, gpu_pos, gpu_uv, gpu_idx);
    const int vcount = SPANS * SAMPLES * SIDES;
    VertexBuffer* gvb = VertexBuffer::Builder().vertexCount((uint32_t)vcount).bufferCount(1)
                            .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36)
                            .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 36)
                            .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 28, 36)
                            .build(*engine);
    gvb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(gpu_pos.data(), gpu_pos.size() * sizeof(float)));
    IndexBuffer* gib = IndexBuffer::Builder().indexCount((uint32_t)gpu_idx.size()).bufferType(IndexBuffer::IndexType::USHORT).build(*engine);
    gib->setBuffer(*engine, IndexBuffer::BufferDescriptor(gpu_idx.data(), gpu_idx.size() * sizeof(uint16_t)));
    utils::Entity ge = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, gvb, gib)
        .material(0, gpu_mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, ge);
    std::printf("gpu param %.3f ang %.3f  p0 %.3f %.3f\n", gpu_pos[7], gpu_pos[1], texels_keep[0], texels_keep[1]);

    std::vector<float> cpu_v;
    std::vector<uint16_t> cpu_idx;
    cpu_tube(spans, cpu_v, cpu_idx);
    VertexBuffer* cvb = VertexBuffer::Builder().vertexCount((uint32_t)vcount).bufferCount(1)
                            .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36)
                            .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 36)
                            .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 28, 36)
                            .build(*engine);
    cvb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(cpu_v.data(), cpu_v.size() * sizeof(float)));
    IndexBuffer* cib = IndexBuffer::Builder().indexCount((uint32_t)cpu_idx.size()).bufferType(IndexBuffer::IndexType::USHORT).build(*engine);
    cib->setBuffer(*engine, IndexBuffer::BufferDescriptor(cpu_idx.data(), cpu_idx.size() * sizeof(uint16_t)));
    utils::Entity ce = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, cvb, cib)
        .material(0, glass_mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, ce);

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

    auto draw = [&](std::vector<uint8_t>& picture) -> bool {
        picture.resize((size_t)W * H * 4);
        bool drew = false;
        for (int attempt = 0; attempt < 4 && !drew; ++attempt) {
            if (!renderer->beginFrame(swap)) continue;
            renderer->render(view);
            renderer->readPixels(rt, 0, 0, W, H, backend::PixelBufferDescriptor(
                picture.data(), picture.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
            renderer->endFrame();
            drew = true;
        }
        engine->flushAndWait();
        return drew;
    };

    scene->addEntity(ge);
    engine->flushAndWait();
    std::vector<uint8_t> gpu_pic, cpu_pic;
    for (int i = 0; i < 3; ++i)
        if (!draw(gpu_pic)) return fail("gpu draw");
    scene->remove(ge);
    scene->addEntity(ce);
    if (!draw(cpu_pic)) return fail("cpu draw");
    if (!write_png("out/gpu-tube.png", W, H, gpu_pic.data())) return fail("png");
    if (!write_png("out/gpu-tube-cpu.png", W, H, cpu_pic.data())) return fail("png");

    double mae = 0;
    int far = 0;
    for (int i = 0; i < W * H; ++i) {
        for (int c = 0; c < 3; ++c) {
            int d = std::abs((int)gpu_pic[(size_t)i * 4 + c] - (int)cpu_pic[(size_t)i * 4 + c]);
            mae += d;
            if (d > 40) ++far;
        }
    }
    mae /= (double)W * H * 3;
    int far_px = far / 3;

    FILE* log = std::fopen("out/gpu-tube.txt", "w");
    std::fprintf(log,
        "materialVertex が RGBA32F の制御点を評価した。矩形 1 本、4 スパン、断面 24×20。\n"
        "頂点位置は全部 0。custom0 は (span, t, angle)。インデックスは固定。\n"
        "制御点は ThorVG の pucker 0 で直線が三次になった 16 点。%u×1 RGBA32F。\n"
        "サンプラは materialParams_controls。textureLod。頂点段階でコンパイルできた。\n"
        "断面の向きは接線と Z の外積。曲線が XY にある間の既存 CPU と同じ。\n"
        "ガラスは ior 2.15 transmission 1 thickness 0.62 dispersion 0.72。屈折とブルームはオン。\n"
        "GPU と CPU 展開の画素差  mean abs %.2f   差が 40 を超える画素 %d / %d\n"
        "out/gpu-tube.png が GPU。out/gpu-tube-cpu.png が同じ制御点の CPU 展開。\n",
        tw, mae, far_px, W * H);
    std::fclose(log);
    std::printf("spans %d  controls %u texels  mae %.2f  far %d\n", SPANS, tw, mae, far_px);
    if (mae > 0.5 || far_px > 100) return fail("gpu tube differs from the cpu tube");
    std::printf("PASS gpu-tube\n");
    return 0;
}
