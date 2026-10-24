// xcube.mm, part 3 (cut there by size, as it was): read only through xcube.mm, in order.
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
    auto blur_map = [](std::vector<float>& tex) {
        for (float& v : tex) v = std::min(std::max(v, 0.f), 60000.f);
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
    };
    std::vector<float> mapA, mapB, mapC, mapDraw;
    Stats sa = build(cu, sun, lux, nd, dispersion, CoatMode::Off, mapA);
    Stats sb = build(cu, sun, lux, nd, dispersion, CoatMode::Hard, mapB);
    Stats sc = build(cu, sun, lux, nd, dispersion, CoatMode::Soft, mapC);
    std::printf("A glass cells %d chroma %.3f  %.1f ms\n", sa.cells, sa.chroma, sa.ms);
    std::printf("B hard  cells %d chroma %.3f  %.1f ms\n", sb.cells, sb.chroma, sb.ms);
    std::printf("C soft  cells %d chroma %.3f  %.1f ms\n", sc.cells, sc.chroma, sc.ms);
    mapDraw = mapC;
    {
        auto encmap = [](const std::vector<float>& src, const char* path) {
            std::vector<uint8_t> pic((size_t)TEX * TEX * 4);
            for (int i = 0; i < TEX * TEX; ++i) {
                const float* p = src.data() + (size_t)i * 4;
                auto enc = [](float v) {
                    float x = v / (v + 3500.f);
                    return (uint8_t)std::clamp(int(std::pow(std::max(x, 0.f), 0.85f) * 255.f), 0, 255);
                };
                pic[(size_t)i * 4 + 0] = enc(p[0]);
                pic[(size_t)i * 4 + 1] = enc(p[1]);
                pic[(size_t)i * 4 + 2] = enc(p[2]);
                pic[(size_t)i * 4 + 3] = 255;
            }
            write_png(path, TEX, TEX, pic.data());
        };
        encmap(mapB, "out/xcube-b-map.png");
        encmap(mapC, "out/xcube-c-map.png");
    }
    blur_map(mapA);
    blur_map(mapB);
    blur_map(mapC);
    std::vector<float>& tex = mapA;

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

    auto gpkg = load_file("glass_shadow.filamat");
    auto fpkg = load_file("floor_recv.filamat");
    auto epkg = load_file("emissive.filamat");
    if (gpkg.empty() || fpkg.empty() || epkg.empty()) return fail("filamat");
    Material* gmat = Material::Builder().package(gpkg.data(), gpkg.size()).build(*engine);
    Material* fmat = Material::Builder().package(fpkg.data(), fpkg.size()).build(*engine);
    Material* emat = Material::Builder().package(epkg.data(), epkg.size()).build(*engine);
    if (!gmat || !fmat || !emat) return fail("material");
    TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    lin.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    lin.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* floor_mi = fmat->createInstance();
    floor_mi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    floor_mi->setParameter("roughness", 0.42f);
    floor_mi->setParameter("caustic", cau, lin);
    MaterialInstance* glass = gmat->createInstance();
    glass->setParameter("ior", nd);
    glass->setParameter("transmission", 1.f);
    glass->setParameter("thickness", cu.h * 2.f);
    glass->setParameter("dispersion", dispersion);
    glass->setParameter("reflectance", 0.5f);
    glass->setParameter("alpha", 1.f);
    glass->setParameter("absorption", float3{0.02f, 0.015f, 0.02f});
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
    utils::Entity floorE = add(floor_v, floor_i, 36, true, floor_mi, false, true);
    utils::Entity cubeE = add(cube_v, cube_i, 28, false, glass, true, false);
    utils::Entity panelE = add(panel_v, panel_i, 28, false, panel, false, false);
    auto film_quad = [&](V3 nrm, std::vector<float>& v, std::vector<uint32_t>& idx) {
        V3 U = cu.a[1];
        V3 Vp = norm(cross(nrm, U));
        float s = cu.h * 0.92f;
        V3 q[4] = {cu.c - U * s - Vp * s, cu.c + U * s - Vp * s, cu.c + U * s + Vp * s, cu.c - U * s + Vp * s};
        for (int i = 0; i < 4; ++i) push_vert(v, q[i], Vp, nrm);
        idx = {0, 1, 2, 0, 2, 3};
    };
    std::vector<float> filmAv, filmBv;
    std::vector<uint32_t> filmAi, filmBi;
    film_quad(norm(cu.a[0] - cu.a[2]), filmAv, filmAi);
    film_quad(norm(cu.a[0] + cu.a[2]), filmBv, filmBi);
    MaterialInstance* filmA = emat->createInstance();
    MaterialInstance* filmB = emat->createInstance();
    filmA->setParameter("color", float4{1400.f, 90.f, 40.f, 1});
    filmB->setParameter("color", float4{40.f, 160.f, 1600.f, 1});
    utils::Entity filmAe = add(filmAv, filmAi, 28, false, filmA, false, false);
    utils::Entity filmBe = add(filmBv, filmBi, 28, false, filmB, false, false);
    scene->remove(filmAe);
    scene->remove(filmBe);

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

    auto upload = [&](std::vector<float>& src) {
        cau->setImage(*engine, 0,
                      Texture::PixelBufferDescriptor(src.data(), src.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                     backend::PixelDataType::FLOAT, 1, 0, 0, TEX));
        engine->flushAndWait();
    };
    auto shot = [&](const char* path) -> bool {
        std::vector<uint8_t> pic((size_t)W * H * 4);
        bool opened = false;
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) {
                opened = true;
                break;
            }
        if (!opened) return false;
        view->setRenderTarget(rt);
        renderer->render(view);
        renderer->readPixels(rt, 0, 0, W, H,
                             backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                            backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
        return write_png(path, W, H, pic.data());
    };
    if (!shot("out/xcube-a.png")) return fail("a");
    upload(mapB);
    if (!shot("out/xcube-b.png")) return fail("b");
    upload(mapC);
    if (!shot("out/xcube-c.png")) return fail("c");

    auto apkg = load_file("caustic_atrous.filamat");
    if (apkg.empty()) return fail("atrous");
    Material* atrousMat = Material::Builder().package(apkg.data(), apkg.size()).build(*engine);
    if (!atrousMat) return fail("atrous-mat");
    Texture* srcTex = Texture::Builder().width(TEX).height(TEX).levels(1).format(Texture::InternalFormat::RGBA16F).build(*engine);
    auto usage = Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC | Texture::Usage::BLIT_DST |
                 Texture::Usage::GEN_MIPMAPPABLE;
    Texture* tempTex = Texture::Builder().width(TEX).height(TEX).levels(10).usage(usage).format(Texture::InternalFormat::RGBA16F).build(*engine);
    Texture* tempDepth = Texture::Builder().width(TEX).height(TEX).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH32F).build(*engine);
    RenderTarget* tempRT = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, tempTex)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, tempDepth).build(*engine);
    Texture* filtTex = Texture::Builder().width(TEX).height(TEX).levels(1)
                           .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                           .format(Texture::InternalFormat::RGBA16F).build(*engine);
    Texture* filtDepth = Texture::Builder().width(TEX).height(TEX).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH32F).build(*engine);
    RenderTarget* filtRT = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, filtTex)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, filtDepth).build(*engine);
    TextureSampler mipped(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR);
    mipped.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    mipped.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* atrous1 = atrousMat->createInstance();
    atrous1->setParameter("src", srcTex, lin);
    atrous1->setParameter("texel", float2{1.f / TEX, 1.f / TEX});
    atrous1->setParameter("stride", 1.f);
    atrous1->setParameter("lumWeight", 0.5f);
    atrous1->setParameter("mip", 0.f);
    atrous1->setParameter("second", 0.f);
    MaterialInstance* atrous2 = atrousMat->createInstance();
    atrous2->setParameter("src", tempTex, mipped);
    atrous2->setParameter("texel", float2{1.f / TEX, 1.f / TEX});
    atrous2->setParameter("stride", 2.f);
    atrous2->setParameter("lumWeight", 0.5f);
    atrous2->setParameter("mip", 1.f);
    atrous2->setParameter("second", 1.f);
    utils::Entity filterE = utils::EntityManager::get().create();
    {
        auto b = VertexBuffer::Builder().vertexCount(4).bufferCount(1);
        b.attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36);
        b.attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36);
        b.attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36);
        VertexBuffer* vb = b.build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(floor_v.data(), floor_v.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(floor_i.data(), floor_i.size() * sizeof(uint32_t)));
        RenderableManager::Builder(1).boundingBox({{-4, -4, -4}, {4, 4, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, atrous1).culling(false).castShadows(false).receiveShadows(false)
            .build(*engine, filterE);
    }
    srcTex->setImage(*engine, 0,
                     Texture::PixelBufferDescriptor(mapDraw.data(), mapDraw.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                    backend::PixelDataType::FLOAT, 1, 0, 0, TEX));
    engine->flushAndWait();
    Skybox* sky = scene->getSkybox();
    scene->setSkybox(nullptr);
    scene->remove(floorE);
    scene->remove(cubeE);
    scene->remove(panelE);
    scene->remove(sun_e);
    scene->addEntity(filterE);
    view->setPostProcessingEnabled(false);
    view->setScreenSpaceRefractionEnabled(false);
    view->setBloomOptions(BloomOptions{});
    camera->setExposure(1.f);
    camera->lookAt({0, 4, 0}, {0, 0, 0}, {0, 0, -1});
    camera->setProjection(Camera::Projection::ORTHO, -1.55, 1.55, -1.55, 1.55, 0.01, 10.0);
    view->setViewport({0, 0, (uint32_t)TEX, (uint32_t)TEX});
    Renderer::ClearOptions clearOpt;
    clearOpt.clear = true;
    clearOpt.discard = true;
    clearOpt.clearColor = {0, 0, 0, 0};
    renderer->setClearOptions(clearOpt);
    auto pass = [&](RenderTarget* target, MaterialInstance* mi) -> bool {
        engine->getRenderableManager().setMaterialInstanceAt(engine->getRenderableManager().getInstance(filterE), 0, mi);
        bool opened = false;
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) {
                opened = true;
                break;
            }
        if (!opened) return false;
        view->setRenderTarget(target);
        renderer->render(view);
        renderer->endFrame();
        return true;
    };
    if (!pass(tempRT, atrous1)) return fail("atrous1");
    tempTex->generateMipmaps(*engine);
    engine->flushAndWait();
    if (!pass(filtRT, atrous2)) return fail("atrous2");
    engine->flushAndWait();
    scene->remove(filterE);
    scene->addEntity(floorE);
    scene->addEntity(cubeE);
    scene->addEntity(panelE);
    scene->addEntity(sun_e);
    scene->setSkybox(sky);
    floor_mi->setParameter("caustic", filtTex, lin);
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    view->setBloomOptions(bloom);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    camera->setExposure(2.8f, 1.f / 125.f, 100.f);
    camera->lookAt({0.55f, 0.78f, 0.70f}, {0.f, 0.04f, 0.f}, {0, 1, 0});
    camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
    if (!shot("out/xcube-d.png")) return fail("d");
    std::printf("PASS xcube-cpu\n");
    return 0;
}
