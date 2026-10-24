// caustics.mm, part 3 (cut there by size, as it was): read only through caustics.mm, in order.
#pragma once


int main() {
    std::vector<Sample> path = close_frames(the_path(), 28);
    Glass back, mid, front;
    back.s = tilted(scaled(path, 1.22f, {0, 0.02f, -0.20f}), 0.62f);
    mid.s = tilted(path, -0.42f);
    front.s = tilted(scaled(path, 0.50f, {0, 0, 0.16f}), 0.22f);
    back.radius = 0.056f;
    mid.radius = 0.042f;
    front.radius = 0.026f;
    back.ior = 1.85f;
    mid.ior = 2.05f;
    front.ior = 2.20f;
    back.dispersion = 9.f;
    mid.dispersion = 13.f;
    front.dispersion = 11.f;
    back.absorb[0] = 0.02f;
    back.absorb[1] = 0.08f;
    back.absorb[2] = 0.55f;
    front.absorb[0] = 0.42f;
    front.absorb[1] = 0.04f;
    front.absorb[2] = 0.02f;
    std::vector<Sample> chrome_s = nudged(mid.s, 0.11f);

    Field field;
    field.build({back, mid, front});
    V3 sun0{0.12f, -0.28f, -1.0f};
    const float lux = 90000.f;
    std::vector<float> tex;
    TraceStats full = trace(field, sun0, lux, false, tex);

    Field none;
    std::vector<float> zero;
    TraceStats empty = trace(none, sun0, lux, false, zero);

    Glass plain = back;
    plain.dispersion = 0;
    Glass plain2 = mid;
    plain2.dispersion = 0;
    Glass plain3 = front;
    plain3.dispersion = 0;
    Field mono;
    mono.build({plain, plain2, plain3});
    std::vector<float> tex0;
    TraceStats same = trace(mono, sun0, lux, true, tex0);

    Glass moved_b = back, moved_m = mid, moved_f = front;
    for (Sample& a : moved_b.s) a.p.x += 0.35f;
    for (Sample& a : moved_m.s) a.p.x += 0.35f;
    for (Sample& a : moved_f.s) a.p.x += 0.35f;
    Field shifted;
    shifted.build({moved_b, moved_m, moved_f});
    std::vector<float> texm;
    TraceStats mv = trace(shifted, sun0, lux, false, texm);

    std::vector<float> texl;
    TraceStats lit = trace(field, {0.55f, -0.28f, -1.0f}, lux, false, texl);
    std::vector<float> tex2;
    TraceStats dbl = trace(field, sun0, lux * 2.f, false, tex2);

    std::printf("CHECK glass-off sum %.3e (want 0)\n", empty.sum);
    std::printf("CHECK same-ior sep %.3f chroma %.3f   dispersed sep %.3f chroma %.3f\n", same.sep, same.chroma, full.sep,
                full.chroma);
    std::printf("CHECK move-glass cxR %.1f -> %.1f\n", full.cx_r, mv.cx_r);
    std::printf("CHECK move-sun cxR %.1f -> %.1f\n", full.cx_r, lit.cx_r);
    std::printf("CHECK lux x2 sum ratio %.3f\n", full.sum > 0 ? dbl.sum / full.sum : 0);

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(16.0f, 1.0f / 125.0f, 100.0f);

    auto make_rt = [&](Texture::InternalFormat fmt) {
        Texture* color = Texture::Builder()
                             .width(W).height(H).levels(1)
                             .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                             .format(fmt).build(*engine);
        Texture* depth = Texture::Builder()
                             .width(W).height(H).levels(1)
                             .usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH24).build(*engine);
        RenderTarget* rt = RenderTarget::Builder()
                               .texture(RenderTarget::AttachmentPoint::COLOR, color)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
        return std::tuple<Texture*, RenderTarget*>{color, rt};
    };
    auto [cA, rtA] = make_rt(Texture::InternalFormat::RGBA16F);
    auto [cB, rtB] = make_rt(Texture::InternalFormat::RGBA16F);
    auto [c8, rt8] = make_rt(Texture::InternalFormat::RGBA8);
    Texture* cf[2] = {cA, cB};
    RenderTarget* rf[2] = {rtA, rtB};

    Texture* cau = Texture::Builder()
                       .width(TEX).height(TEX).levels(1)
                       .sampler(Texture::Sampler::SAMPLER_2D)
                       .format(Texture::InternalFormat::RGBA16F).build(*engine);
    auto upload = [&](const std::vector<float>& src) {
        cau->setImage(*engine, 0,
                      Texture::PixelBufferDescriptor(src.data(), src.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                     backend::PixelDataType::FLOAT, 1, 0, 0, TEX));
    };
    upload(tex);

    auto ibl_bytes = load_file("out/ibl/ibl_ibl.ktx");
    auto sky_bytes = load_file("out/ibl/ibl_skybox.ktx");
    if (ibl_bytes.empty() || sky_bytes.empty()) return fail("ktx");
    Texture* ibl_tex = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(ibl_bytes.data(), (uint32_t)ibl_bytes.size()), false);
    Texture* sky_tex = ktxreader::Ktx1Reader::createTexture(engine, new image::Ktx1Bundle(sky_bytes.data(), (uint32_t)sky_bytes.size()), false);
    float3 sh[9];
    if (!load_sh("out/ibl/sh.txt", sh) || !ibl_tex || !sky_tex) return fail("ibl");
    scene->setIndirectLight(IndirectLight::Builder().reflections(ibl_tex).irradiance(3, sh).intensity(9000.f).build(*engine));
    Skybox* sky = Skybox::Builder().environment(sky_tex).showSun(false).build(*engine);
    Skybox* dark = Skybox::Builder().color({0.01f, 0.008f, 0.006f, 1}).build(*engine);

    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    view->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.06f).saturation(1.14f).build(*engine));
    view->setScene(scene);
    view->setCamera(camera);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setDithering(View::Dithering::NONE);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setShadowType(View::ShadowType::VSM);
    VsmShadowOptions vsm;
    vsm.highPrecision = true;
    view->setVsmShadowOptions(vsm);
    ScreenSpaceReflectionsOptions ssr{};
    ssr.enabled = false;
    view->setScreenSpaceReflectionsOptions(ssr);

    auto gpkg = load_file("glass_shadow.filamat");
    auto fpkg = load_file("floor_recv.filamat");
    auto epkg = load_file("emissive.filamat");
    auto cpkg = load_file("chrome_aniso.filamat");
    auto ppkg = load_file("plate.filamat");
    auto lpkg = load_file("floor_lit.filamat");
    if (gpkg.empty() || fpkg.empty() || epkg.empty() || cpkg.empty() || ppkg.empty() || lpkg.empty()) return fail("filamat");
    Material* gmat = Material::Builder().package(gpkg.data(), gpkg.size()).build(*engine);
    Material* fmat = Material::Builder().package(fpkg.data(), fpkg.size()).build(*engine);
    Material* emat = Material::Builder().package(epkg.data(), epkg.size()).build(*engine);
    Material* cmat = Material::Builder().package(cpkg.data(), cpkg.size()).build(*engine);
    Material* pmat = Material::Builder().package(ppkg.data(), ppkg.size()).build(*engine);
    Material* lmat = Material::Builder().package(lpkg.data(), lpkg.size()).build(*engine);
    if (!gmat || !fmat || !emat || !cmat || !pmat || !lmat) return fail("material");

    TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    lin.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    lin.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* floor_mi = fmat->createInstance();
    floor_mi->setParameter("color", float4{0.045f, 0.040f, 0.038f, 1});
    floor_mi->setParameter("roughness", 0.55f);
    floor_mi->setParameter("caustic", cau, lin);

    auto emit = [&](float r, float g, float b) {
        MaterialInstance* mi = emat->createInstance();
        mi->setParameter("color", float4{r, g, b, 1});
        return mi;
    };
    auto glass_mi = [&](const Glass& g, float alpha) {
        MaterialInstance* mi = gmat->createInstance();
        mi->setParameter("ior", g.ior);
        mi->setParameter("transmission", 1.f);
        mi->setParameter("thickness", 1.05f);
        mi->setParameter("dispersion", g.dispersion);
        mi->setParameter("reflectance", 0.42f);
        mi->setParameter("alpha", alpha);
        mi->setParameter("absorption", float3{g.absorb[0], g.absorb[1], g.absorb[2]});
        return mi;
    };
    MaterialInstance* window_mi = emit(220000.f, 86000.f, 22000.f);
    MaterialInstance* mullion_mi = lmat->createInstance();
    mullion_mi->setParameter("color", float4{0.012f, 0.008f, 0.006f, 1});
    mullion_mi->setParameter("roughness", 0.9f);
    MaterialInstance* wall_mi = lmat->createInstance();
    wall_mi->setParameter("color", float4{0.03f, 0.027f, 0.034f, 1});
    wall_mi->setParameter("roughness", 0.8f);
    MaterialInstance* chrome_mi = cmat->createInstance();
    chrome_mi->setParameter("anisotropy", 0.92f);
    chrome_mi->setParameter("roughness", 0.20f);

    auto add_floor = [&](MaterialInstance* mi) {
        Hold h;
        quad_uv({0, FLOOR_Y, FLOOR_CZ}, {1, 0, 0}, {0, 0, -1}, FLOOR_HW, FLOOR_HH, h.uv);
        h.mesh.idx = {0, 1, 2, 0, 2, 3};
        h.vb = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36)
                   .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36)
                   .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36)
                   .build(*engine);
        h.vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(h.uv.data(), h.uv.size() * sizeof(float)));
        h.ib = IndexBuffer::Builder().indexCount((uint32_t)h.mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        h.ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(h.mesh.idx.data(), h.mesh.idx.size() * sizeof(uint32_t)));
        h.entity = utils::EntityManager::get().create();
        RenderableManager::Builder(1)
            .boundingBox({{-8, -8, -8}, {8, 8, 8}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, h.vb, h.ib)
            .material(0, mi).culling(false).castShadows(false).receiveShadows(true)
            .build(*engine, h.entity);
        return h;
    };
    Hold floor_h = add_floor(floor_mi);

    auto add_solid = [&](Built mesh, MaterialInstance* mi, bool cast_s, bool recv) {
        Hold h;
        h.mesh = std::move(mesh);
        uint32_t nv = (uint32_t)(h.mesh.v.size() / 7);
        h.vb = VertexBuffer::Builder().vertexCount(nv).bufferCount(1)
                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                   .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                   .build(*engine);
        h.vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(h.mesh.v.data(), h.mesh.v.size() * sizeof(float)));
        h.ib = IndexBuffer::Builder().indexCount((uint32_t)h.mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        h.ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(h.mesh.idx.data(), h.mesh.idx.size() * sizeof(uint32_t)));
        h.entity = utils::EntityManager::get().create();
        RenderableManager::Builder(1)
            .boundingBox({{-8, -8, -8}, {8, 8, 8}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, h.vb, h.ib)
            .material(0, mi).culling(false).castShadows(cast_s).receiveShadows(recv)
            .build(*engine, h.entity);
        return h;
    };

    Hold window_h = add_solid(quad({-0.42f, 0.34f, -1.22f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 1.05f), window_mi, false, false);
    Hold mull_h = add_solid(quad({-0.42f, 0.34f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 0.016f, 1.05f), mullion_mi, false, false);
    Hold mull2 = add_solid(quad({-0.42f, 0.82f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 0.014f), mullion_mi, false, false);
    Hold wall_h = add_solid(quad({0.0f, 0.15f, -1.35f}, {1, 0, 0}, {0, 1, 0}, 2.2f, 1.3f), wall_mi, false, true);
    Hold chrome_h = add_solid(tube(chrome_s, 28, 0.013f), chrome_mi, true, false);
    MaterialInstance* gb = glass_mi(back, 1.f);
    MaterialInstance* gm = glass_mi(mid, 1.f);
    MaterialInstance* gf = glass_mi(front, 1.f);
    MaterialInstance* gshadow = glass_mi(back, 0.35f);
    Hold hb = add_solid(tube(back.s, 32, back.radius), gb, true, false);
    Hold hm = add_solid(tube(mid.s, 32, mid.radius), gm, true, false);
    Hold hf = add_solid(tube(front.s, 32, front.radius), gf, true, false);

    // Shadow demo: one partial-alpha tube, floor, no caustic contribution.
    std::vector<Sample> demo;
    for (int i = 0; i < 48; ++i) {
        float a = (float)i / 48.f * TAU;
        Sample s;
        s.p = {0.42f * std::cos(a), 0.05f, 0.42f * std::sin(a)};
        s.T = norm({-std::sin(a), 0, std::cos(a)});
        s.N = norm(s.p);
        s.B = norm(cross(s.T, s.N));
        demo.push_back(s);
    }
    Hold demo_h = add_solid(tube(demo, 28, 0.07f), gshadow, true, false);

    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 2048;
    shopt.vsm.blurWidth = 16.f;
    shopt.vsm.elvsm = true;
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1, 0.96f, 0.9f}).intensity(lux).direction(normalize(float3{sun0.x, sun0.y, sun0.z}))
        .sunAngularRadius(0.55f).sunHaloSize(1.4f).castShadows(true).shadowOptions(shopt)
        .build(*engine, sun);
    scene->addEntity(sun);

    std::vector<utils::Entity> shown;
    auto set_shown = [&](const std::vector<utils::Entity>& next) {
        for (utils::Entity e : shown) scene->remove(e);
        shown = next;
        for (utils::Entity e : shown) scene->addEntity(e);
    };
    auto open_frame = [&]() {
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) return true;
        return false;
    };
    std::vector<uint8_t> pic((size_t)W * H * 4);
    auto grab = [&](RenderTarget* rt, const char* path) {
        if (!open_frame()) return false;
        view->setRenderTarget(rt);
        renderer->render(view);
        renderer->readPixels(rt, 0, 0, W, H,
                             backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                            backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
        if (!write_png(path, W, H, pic.data())) return false;
        std::printf("wrote %s\n", path);
        return true;
    };

    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    auto& lm0 = engine->getLightManager();
    lm0.setDirection(lm0.getInstance(sun), normalize(float3{0.35f, -1.f, 0.25f}));
    engine->getTransformManager().setTransform(engine->getTransformManager().getInstance(demo_h.entity),
                                                mat4f::translation(float3{0, 0.45f, 0}));
    floor_mi->setParameter("color", float4{0.22f, 0.20f, 0.18f, 1});
    camera->lookAt({1.35f, 0.72f, 1.55f}, {0.05f, -0.35f, 0}, {0, 1, 0});
    camera->setProjection(28.0, double(W) / double(H), 0.05, 30.0, Camera::Fov::VERTICAL);
    std::vector<float> black((size_t)TEX * TEX * 4, 0.f);
    upload(black);
    scene->setSkybox(dark);
    set_shown({floor_h.entity, demo_h.entity});
    auto tA0 = std::chrono::steady_clock::now();
    if (!grab(rt8, "out/caustics-shadow.png")) return fail("shadow");
    auto tA1 = std::chrono::steady_clock::now();
    std::printf("shadow frame %.1f ms (flushAndWait, includes readPixels)\n",
                std::chrono::duration<double, std::milli>(tA1 - tA0).count());
    lm0.setDirection(lm0.getInstance(sun), normalize(float3{sun0.x, sun0.y, sun0.z}));
    floor_mi->setParameter("color", float4{0.045f, 0.040f, 0.038f, 1});

    // Caustic only, seen from above so the floor pattern is the picture.
    upload(tex);
    camera->lookAt({0.0f, 1.55f, 0.15f}, {0, FLOOR_Y, FLOOR_CZ}, {0, 0, -1});
    camera->setProjection(48.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
    scene->remove(sun);
    set_shown({floor_h.entity});
    if (!grab(rt8, "out/caustics-only.png")) return fail("only");
    scene->addEntity(sun);

    // Hero camera, caustics off then on. Opaque room + three refractive passes.
    const float3 eye{0.18f, 0.40f, 1.82f};
    const float3 target{0, 0.02f, 0};
    camera->lookAt(eye, target, {0, 1, 0});
    camera->setProjection(34.0, double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);

    MaterialInstance* plate_mi[2] = {pmat->createInstance(), pmat->createInstance()};
    plate_mi[0]->setParameter("color", cf[0], lin);
    plate_mi[1]->setParameter("color", cf[1], lin);
    Hold plate;
    std::vector<float> plate_v(4 * 9, 0.f);
    std::vector<uint32_t> plate_idx = {0, 1, 2, 0, 2, 3};
    plate.vb = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36)
                   .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36)
                   .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36)
                   .build(*engine);
    plate.ib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    plate.ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(plate_idx.data(), plate_idx.size() * sizeof(uint32_t)));
    plate.entity = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-30, -30, -30}, {30, 30, 30}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, plate.vb, plate.ib)
        .material(0, plate_mi[0]).culling(false).castShadows(false).receiveShadows(false)
        .build(*engine, plate.entity);
    {
        float3 f = normalize(target - eye);
        float3 r = normalize(cross(f, float3{0, 1, 0}));
        float3 u = cross(r, f);
        float dist = 8.f;
        float hh = dist * std::tan(17.f * PI / 180.f);
        float hw = hh * ((float)W / (float)H);
        float3 c = eye + f * dist;
        float3 corner[4] = {c - r * hw - u * hh, c + r * hw - u * hh, c + r * hw + u * hh, c - r * hw + u * hh};
        float uv[4][2] = {{0, 0}, {1, 0}, {1, 1}, {0, 1}};
        quatf q = mat3f::packTangentFrame(mat3f{r, normalize(cross(r, f)), f});
        for (int i = 0; i < 4; ++i) {
            float* o = plate_v.data() + i * 9;
            o[0] = corner[i].x; o[1] = corner[i].y; o[2] = corner[i].z;
            o[3] = uv[i][0]; o[4] = uv[i][1];
            o[5] = q.x; o[6] = q.y; o[7] = q.z; o[8] = q.w;
        }
        plate.vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(plate_v.data(), plate_v.size() * sizeof(float)));
    }

    utils::Entity glasses[3] = {hb.entity, hm.entity, hf.entity};
    std::vector<utils::Entity> opaque = {floor_h.entity, wall_h.entity, window_h.entity, mull_h.entity, mull2.entity, chrome_h.entity};
    auto& rm = engine->getRenderableManager();
    BloomOptions bloom{};
    bloom.enabled = false;
    auto shoot = [&](const char* path) {
        if (!open_frame()) return false;
        auto t0 = std::chrono::steady_clock::now();
        bloom.enabled = false;
        view->setBloomOptions(bloom);
        view->setPostProcessingEnabled(false);
        view->setScreenSpaceRefractionEnabled(false);
        scene->setSkybox(sky);
        set_shown(opaque);
        view->setRenderTarget(rf[0]);
        renderer->render(view);
        scene->setSkybox(nullptr);
        view->setScreenSpaceRefractionEnabled(true);
        int src = 0;
        for (int g = 0; g < 3; ++g) {
            int dst = 1 - src;
            rm.setMaterialInstanceAt(rm.getInstance(plate.entity), 0, plate_mi[src]);
            set_shown({plate.entity, glasses[g]});
            view->setRenderTarget(rf[dst]);
            renderer->render(view);
            src = dst;
        }
        bloom.enabled = true;
        bloom.strength = 0.26f;
        bloom.levels = 7;
        bloom.resolution = 1080;
        bloom.threshold = true;
        bloom.highlight = 12.f;
        bloom.blendMode = BloomOptions::BlendMode::ADD;
        view->setBloomOptions(bloom);
        view->setPostProcessingEnabled(true);
        view->setScreenSpaceRefractionEnabled(false);
        rm.setMaterialInstanceAt(rm.getInstance(plate.entity), 0, plate_mi[src]);
        set_shown({plate.entity});
        view->setRenderTarget(rt8);
        renderer->render(view);
        renderer->readPixels(rt8, 0, 0, W, H,
                             backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                            backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
        auto t1 = std::chrono::steady_clock::now();
        if (!write_png(path, W, H, pic.data())) return false;
        std::printf("frame %s %.1f ms wall (flushAndWait + readPixels, no geometry readback)\n", path,
                    std::chrono::duration<double, std::milli>(t1 - t0).count());
        return true;
    };

    upload(black);
    if (!shoot("out/caustics-off.png")) return fail("off");
    upload(tex);
    if (!shoot("out/caustics-hero.png")) return fail("hero");

    for (int f = 0; f < 6; ++f) {
        float u = (float)f / 5.f;
        float dx = -0.22f + 0.44f * u;
        Glass mb = back, mm = mid, mf = front;
        for (Sample& a : mb.s) a.p.x += dx;
        for (Sample& a : mm.s) a.p.x += dx;
        for (Sample& a : mf.s) a.p.x += dx;
        Field fr;
        fr.build({mb, mm, mf});
        V3 sd{sun0.x + dx * 0.55f, sun0.y, sun0.z};
        std::vector<float> frame_tex;
        trace(fr, sd, lux, false, frame_tex);
        upload(frame_tex);
        auto& tm = engine->getTransformManager();
        tm.setTransform(tm.getInstance(hb.entity), mat4f::translation(float3{dx, 0, 0}));
        tm.setTransform(tm.getInstance(hm.entity), mat4f::translation(float3{dx, 0, 0}));
        tm.setTransform(tm.getInstance(hf.entity), mat4f::translation(float3{dx, 0, 0}));
        tm.setTransform(tm.getInstance(chrome_h.entity), mat4f::translation(float3{dx, 0, 0}));
        auto& lm = engine->getLightManager();
        lm.setDirection(lm.getInstance(sun), normalize(float3{sd.x, sd.y, sd.z}));
        char name[64];
        std::snprintf(name, sizeof(name), "out/caustics-motion-%02d.png", f);
        if (!shoot(name)) return fail("motion");
    }
    std::printf("PASS caustics\n");
    return 0;
}
