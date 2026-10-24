// prism.mm, part 2 (cut there by size, as it was): read only through prism.mm, in order.
#pragma once


int main(int argc, char** argv) {
    Cube cu;
    cu.h = 0.16f;
    cu.c = {0.f, cu.h + 0.001f, 0.f};
    cu.a[0] = {1, 0, 0};
    cu.a[1] = {0, 1, 0};
    cu.a[2] = {0, 0, 1};
    for (int i = 0; i < 3; ++i) {
        rotate(cu.a[i], 1, 0.62f);
    }
    for (int i = 0; i < 3; ++i) cu.a[i] = norm(cu.a[i]);
    const float nd = 1.55f;
    const float dispersion = 36.f;
    const float lux = 22000.f;
    V3 sun{0.92f, -0.42f, 0.22f};
    std::vector<float> tex, plain;
    Stats full = build(cu, sun, lux, nd, dispersion, tex);
    Stats mono = build(cu, sun, lux, nd, 0.f, plain);
    std::printf("CHECK dispersed chroma %.3f   same-ior chroma %.3f\n", full.chroma, mono.chroma);
    std::printf("CHECK max %.1f (RGBA16F holds this; upload clamps only above 60000)\n", full.maxv);
    if (argc > 1 && std::string(argv[1]) == "dump") {
        auto save = [](const char* path, const std::vector<float>& src) {
            FILE* f = std::fopen(path, "wb");
            if (!f) return false;
            uint32_t n = (uint32_t)TEX;
            std::fwrite(&n, 4, 1, f);
            std::fwrite(src.data(), sizeof(float), src.size(), f);
            std::fclose(f);
            return true;
        };
        if (!save("out/prism-spec.f32", tex) || !save("out/prism-mono.f32", plain)) return fail("dump");
        std::printf("dumped pre-blur spec and mono %d\n", TEX);
        return 0;
    }

    std::vector<uint8_t> map((size_t)TEX * TEX * 4);
    for (int i = 0; i < TEX * TEX; ++i) {
        float* p = tex.data() + (size_t)i * 4;
        auto enc = [](float v) {
            float x = v / (v + 3500.f);
            return (uint8_t)std::clamp(int(std::pow(x, 0.85f) * 255.f), 0, 255);
        };
        map[(size_t)i * 4 + 0] = enc(p[0]);
        map[(size_t)i * 4 + 1] = enc(p[1]);
        map[(size_t)i * 4 + 2] = enc(p[2]);
        map[(size_t)i * 4 + 3] = 255;
    }
    write_png("out/prism-map.png", TEX, TEX, map.data());

    for (float& v : tex) v = std::min(v, 60000.f);
    {
        std::vector<float> blurred(tex.size());
        for (int y = 0; y < TEX; ++y)
            for (int x = 0; x < TEX; ++x) {
                float r = 0, g = 0, b = 0, wsum = 0;
                for (int dy = -4; dy <= 4; ++dy)
                    for (int dx = -4; dx <= 4; ++dx) {
                        int xx = std::clamp(x + dx, 0, TEX - 1);
                        int yy = std::clamp(y + dy, 0, TEX - 1);
                        float w = std::exp(-(dx * dx + dy * dy) / 3.2f);
                        float* s = tex.data() + ((size_t)yy * TEX + xx) * 4;
                        r += s[0] * w;
                        g += s[1] * w;
                        b += s[2] * w;
                        wsum += w;
                    }
                float* d = blurred.data() + ((size_t)y * TEX + x) * 4;
                d[0] = r / wsum;
                d[1] = g / wsum;
                d[2] = b / wsum;
            }
        tex.swap(blurred);
    }

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(2.8f, 1.0f / 125.0f, 100.0f);
    camera->lookAt({0.55f, 0.78f, 0.70f}, {0.0f, 0.04f, 0.0f}, {0, 1, 0});
    camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);

    Texture* color = Texture::Builder()
                         .width(W).height(H).levels(1)
                         .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                         .format(Texture::InternalFormat::RGBA8).build(*engine);
    Texture* depth = Texture::Builder()
                         .width(W).height(H).levels(1)
                         .usage(Texture::Usage::DEPTH_ATTACHMENT)
                         .format(Texture::InternalFormat::DEPTH24).build(*engine);
    RenderTarget* rt = RenderTarget::Builder()
                           .texture(RenderTarget::AttachmentPoint::COLOR, color)
                           .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
    Texture* cau = Texture::Builder()
                       .width(TEX).height(TEX).levels(1)
                       .sampler(Texture::Sampler::SAMPLER_2D)
                       .format(Texture::InternalFormat::RGBA16F).build(*engine);
    cau->setImage(*engine, 0,
                  Texture::PixelBufferDescriptor(tex.data(), tex.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                 backend::PixelDataType::FLOAT, 1, 0, 0, TEX));

    scene->setSkybox(Skybox::Builder().color({0.72f, 0.74f, 0.78f, 1}).build(*engine));
    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    view->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.04f).saturation(1.08f).build(*engine));
    view->setScene(scene);
    view->setCamera(camera);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setDithering(View::Dithering::NONE);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    view->setShadowType(View::ShadowType::VSM);
    VsmShadowOptions vsm;
    vsm.highPrecision = true;
    view->setVsmShadowOptions(vsm);
    BloomOptions bloom{};
    bloom.enabled = true;
    bloom.strength = 0.18f;
    bloom.levels = 6;
    bloom.resolution = 1080;
    bloom.threshold = true;
    bloom.highlight = 10.f;
    bloom.blendMode = BloomOptions::BlendMode::ADD;
    view->setBloomOptions(bloom);

    auto fpkg = load_file("floor_recv.filamat");
    auto epkg = load_file("emissive.filamat");
    auto ppkg = load_file("xcube_drei.filamat");
    if (fpkg.empty() || epkg.empty() || ppkg.empty()) return fail("filamat");
    Material* fmat = Material::Builder().package(fpkg.data(), fpkg.size()).build(*engine);
    Material* emat = Material::Builder().package(epkg.data(), epkg.size()).build(*engine);
    Material* pmat = Material::Builder().package(ppkg.data(), ppkg.size()).build(*engine);
    if (!fmat || !emat || !pmat) return fail("material");
    TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    lin.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    lin.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* floor_mi = fmat->createInstance();
    floor_mi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    floor_mi->setParameter("roughness", 0.42f);
    floor_mi->setParameter("caustic", cau, lin);
    MaterialInstance* panel = emat->createInstance();
    panel->setParameter("color", float4{2200.f, 2300.f, 2600.f, 1});

    std::vector<float> floor_v;
    std::vector<uint32_t> floor_i = {0, 2, 1, 0, 3, 2};
    {
        V3 center{0, 0, 0}, right{1, 0, 0}, up{0, 0, 1};
        V3 n{0, 1, 0};
        V3 p[4] = {center - right * FLOOR_HW - up * FLOOR_HH, center + right * FLOOR_HW - up * FLOOR_HH,
                   center + right * FLOOR_HW + up * FLOOR_HH, center - right * FLOOR_HW + up * FLOOR_HH};
        float uv[4][2] = {{0, 0}, {1, 0}, {1, 1}, {0, 1}};
        quatf q = mat3f::packTangentFrame(mat3f{float3{1, 0, 0}, float3{0, 0, -1}, float3{0, 1, 0}});
        floor_v.resize(4 * 9);
        for (int i = 0; i < 4; ++i) {
            float* o = floor_v.data() + i * 9;
            o[0] = p[i].x;
            o[1] = p[i].y;
            o[2] = p[i].z;
            o[3] = uv[i][0];
            o[4] = uv[i][1];
            o[5] = q.x;
            o[6] = q.y;
            o[7] = q.z;
            o[8] = q.w;
            (void)n;
        }
    }
    std::vector<float> cube_v;
    std::vector<uint32_t> cube_i;
    for (int face = 0; face < 6; ++face) {
        V3 N = face_normal(cu, face);
        V3 T = cu.a[(face / 2 + 1) % 3];
        V3 B = cross(N, T);
        uint32_t base = (uint32_t)(cube_v.size() / 7);
        V3 c = cu.c + N * cu.h;
        push_vert(cube_v, c - T * cu.h - B * cu.h, T, N);
        push_vert(cube_v, c + T * cu.h - B * cu.h, T, N);
        push_vert(cube_v, c + T * cu.h + B * cu.h, T, N);
        push_vert(cube_v, c - T * cu.h + B * cu.h, T, N);
        cube_i.insert(cube_i.end(), {base, base + 1, base + 2, base, base + 2, base + 3});
    }
    std::vector<float> panel_v;
    std::vector<uint32_t> panel_i = {0, 1, 2, 0, 2, 3};
    {
        V3 c{0.0f, 0.55f, -1.15f};
        V3 right{1, 0, 0}, upn{0, 1, 0};
        float hw = 1.6f, hh = 0.7f;
        V3 p[4] = {c - right * hw - upn * hh, c + right * hw - upn * hh, c + right * hw + upn * hh, c - right * hw + upn * hh};
        for (int i = 0; i < 4; ++i) push_vert(panel_v, p[i], right, {0, 0, 1});
    }

    std::vector<float> triPos;
    triPos.reserve(cube_i.size() * 4);
    for (uint32_t idx : cube_i) {
        const float* p = cube_v.data() + (size_t)idx * 7;
        triPos.insert(triPos.end(), {p[0], p[1], p[2], 1.f});
    }
    const int ntri = (int)(cube_i.size() / 3);
    Texture* triTex = Texture::Builder().width((uint32_t)ntri * 3).height(1).levels(1)
                          .sampler(Texture::Sampler::SAMPLER_2D)
                          .format(Texture::InternalFormat::RGBA32F)
                          .usage(Texture::Usage::DEFAULT)
                          .build(*engine);
    triTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(triPos.data(), triPos.size() * sizeof(float),
                                                                Texture::Format::RGBA, Texture::Type::FLOAT));
    MaterialInstance* path = pmat->createInstance();
    TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    nearest.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    nearest.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    path->setParameter("tris", triTex, nearest);
    path->setParameter("eye", float3{0.55f, 0.78f, 0.70f});
    path->setParameter("triCount", (float)ntri);
    path->setParameter("bounces", 3.f);
    path->setParameter("ior", nd);
    path->setParameter("dispersion", dispersion);
    path->setParameter("aberration", 0.f);
    path->setParameter("chroma", 0.f);
    path->setParameter("fresnel", 0.f);
    path->setParameter("grad", 0.f);
    path->setParameter("envScale", 1.f);

    auto add = [&](std::vector<float>& verts, std::vector<uint32_t>& idx, int stride, bool uv, MaterialInstance* mi, bool cast_s,
                   bool recv) {
        utils::Entity e = utils::EntityManager::get().create();
        uint32_t nv = uv ? 4u : (uint32_t)(verts.size() / 7);
        auto b = VertexBuffer::Builder().vertexCount(nv).bufferCount(1);
        b.attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, (uint32_t)stride);
        if (uv) {
            b.attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, (uint32_t)stride);
            b.attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, (uint32_t)stride);
        } else b.attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, (uint32_t)stride);
        VertexBuffer* vb = b.build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(verts.data(), verts.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(idx.data(), idx.size() * sizeof(uint32_t)));
        RenderableManager::Builder(1)
            .boundingBox({{-4, -4, -4}, {4, 4, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi).culling(false).castShadows(cast_s).receiveShadows(recv)
            .build(*engine, e);
        scene->addEntity(e);
        return e;
    };
    add(floor_v, floor_i, 36, true, floor_mi, false, true);
    utils::Entity cubeE = add(cube_v, cube_i, 28, false, path, true, false);
    add(panel_v, panel_i, 28, false, panel, false, false);

    utils::Entity sun_e = utils::EntityManager::get().create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 2048;
    shopt.vsm.blurWidth = 8.f;
    shopt.vsm.elvsm = true;
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1.f, 0.98f, 0.96f})
        .intensity(lux)
        .direction(normalize(float3{sun.x, sun.y, sun.z}))
        .sunAngularRadius(0.35f)
        .sunHaloSize(1.2f)
        .castShadows(true)
        .shadowOptions(shopt)
        .build(*engine, sun_e);
    scene->addEntity(sun_e);

    // The path leaves the glass into this room. Capture it without the cube, before the grade.
    scene->remove(cubeE);
    const int FACE = 512;
    Texture* faceColor = Texture::Builder()
                             .width(FACE).height(FACE).levels(1)
                             .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::BLIT_SRC)
                             .format(Texture::InternalFormat::RGBA16F)
                             .build(*engine);
    Texture* faceDepth = Texture::Builder()
                             .width(FACE).height(FACE).levels(1)
                             .usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH24)
                             .build(*engine);
    RenderTarget* faceRT = RenderTarget::Builder()
                               .texture(RenderTarget::AttachmentPoint::COLOR, faceColor)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, faceDepth)
                               .build(*engine);
    view->setPostProcessingEnabled(false);
    view->setViewport({0, 0, (uint32_t)FACE, (uint32_t)FACE});
    view->setRenderTarget(faceRT);
    camera->setProjection(90.0, 1.0, 0.02, 20.0, Camera::Fov::VERTICAL);
    float3 origin{cu.c.x, cu.c.y, cu.c.z};
    float3 fwd[6] = {{1, 0, 0}, {-1, 0, 0}, {0, 1, 0}, {0, -1, 0}, {0, 0, 1}, {0, 0, -1}};
    float3 upv[6] = {{0, -1, 0}, {0, -1, 0}, {0, 0, 1}, {0, 0, -1}, {0, -1, 0}, {0, -1, 0}};
    static std::vector<float> roomFace[6];
    for (int f = 0; f < 6; ++f) {
        camera->lookAt(origin, origin + fwd[f], upv[f]);
        roomFace[f].assign((size_t)FACE * FACE * 4, 0.f);
        bool opened = false;
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) {
                opened = true;
                break;
            }
        if (!opened) return fail("room face");
        renderer->render(view);
        renderer->readPixels(faceRT, 0, 0, FACE, FACE,
                             backend::PixelBufferDescriptor(roomFace[f].data(), roomFace[f].size() * sizeof(float),
                                                            backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
        renderer->endFrame();
        engine->flushAndWait();
        float mx = 0.f;
        for (int i = 0; i < FACE * FACE; ++i) mx = std::max(mx, roomFace[f][i * 4]);
        std::printf("room face %d max %.2f\n", f, mx);
    }
    Texture* room = Texture::Builder().width(FACE).height(FACE).levels(1)
                        .sampler(Texture::Sampler::SAMPLER_CUBEMAP)
                        .format(Texture::InternalFormat::RGBA16F)
                        .usage(Texture::Usage::DEFAULT)
                        .build(*engine);
    for (int f = 0; f < 6; ++f) {
        room->setImage(*engine, 0, 0, 0, (uint32_t)f, (uint32_t)FACE, (uint32_t)FACE, 1,
                       Texture::PixelBufferDescriptor(roomFace[f].data(), roomFace[f].size() * sizeof(float),
                                                      Texture::Format::RGBA, Texture::Type::FLOAT));
    }
    path->setParameter("env", room, TextureSampler(TextureSampler::MagFilter::LINEAR));
    scene->addEntity(cubeE);
    view->setPostProcessingEnabled(true);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setRenderTarget(rt);
    camera->setExposure(2.8f, 1.0f / 125.0f, 100.0f);
    camera->lookAt({0.55f, 0.78f, 0.70f}, {0.0f, 0.04f, 0.0f}, {0, 1, 0});
    camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
    std::printf("path triangles %d bounces 3 ior %.2f\n", ntri, nd);

    std::vector<uint8_t> pic((size_t)W * H * 4);
    auto t0 = std::chrono::steady_clock::now();
    bool opened = false;
    for (int i = 0; i < 4000; ++i)
        if (renderer->beginFrame(swap)) {
            opened = true;
            break;
        }
    if (!opened) return fail("frame");
    view->setRenderTarget(rt);
    renderer->render(view);
    renderer->readPixels(rt, 0, 0, W, H,
                         backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                        backend::PixelDataType::UBYTE));
    renderer->endFrame();
    engine->flushAndWait();
    auto t1 = std::chrono::steady_clock::now();
    if (!write_png("out/prism.png", W, H, pic.data())) return fail("png");
    std::printf("frame %.1f ms wall (flushAndWait + readPixels)\n", std::chrono::duration<double, std::milli>(t1 - t0).count());
    std::printf("PASS prism\n");
    return 0;
}
