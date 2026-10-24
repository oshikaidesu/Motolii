// trio.mm, piece 2 (cut there by size, as it was): read only in its place in trio.mm.
    std::vector<Sample> back_s = tilted(scaled(path, back_scale, {0, 0.01f, back_z}), back_tilt);
    std::vector<Sample> mid_s = tilted(path, mid_tilt);
    std::vector<Sample> front_s = tilted(scaled(path, front_scale, {0, 0, front_z}), front_tilt);
    std::vector<Sample> chrome_s = chrome_shot ? back_s : nudged(mid_s, chrome_nudge);
    std::vector<Sample> chrome2_s = tilted(scaled(path, 0.70f, {0.03f, -0.03f, 0.06f}), -0.25f);
    std::vector<Cubic> spans = tilt_spans(spans0, centroid(path), mid_tilt);
    const float chrome2_r = chrome_shot ? 0.026f : 0.018f;
    auto need = [](float a, float b) { return a + b; };
    float g_bm = gap(back_s, mid_s), g_mf = gap(mid_s, front_s), g_mc = gap(mid_s, chrome_s);
    float n_bm = need(back_r, mid_r), n_mf = need(mid_r, front_r), n_mc = need(mid_r, chrome_r);
    if (chrome_shot) {
        g_mc = gap(chrome_s, mid_s);
        n_mc = need(back_r, mid_r);
        float g_c2 = gap(chrome2_s, mid_s);
        float g_c2f = gap(chrome2_s, front_s);
        float g_c12 = gap(chrome_s, chrome2_s);
        std::printf("chrome gaps metal-glass %.3f>%.3f  sharp-mid %.3f>%.3f  sharp-front %.3f>%.3f  metals %.3f>%.3f\n",
                    g_mc, n_mc, g_c2, chrome2_r + mid_r, g_c2f, chrome2_r + front_r, g_c12, back_r + chrome2_r);
        if (g_mc < n_mc + 0.01f || g_c2 < chrome2_r + mid_r + 0.01f || g_c2f < chrome2_r + front_r + 0.01f ||
            g_c12 < back_r + chrome2_r + 0.01f)
            return fail("intersect");
    } else {
        std::printf("%s gaps back-mid %.3f>%.3f mid-front %.3f>%.3f mid-chrome %.3f>%.3f\n", which, g_bm, n_bm, g_mf, n_mf,
                    g_mc, n_mc);
        if (g_bm < n_bm + 0.01f || g_mf < n_mf + 0.01f || g_mc < n_mc + 0.01f) return fail("intersect");
    }

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(16.0f, 1.0f / 125.0f, 100.0f);
    const float3 eye{0.18f, 0.40f, 1.82f};
    const float3 target{0.0f, 0.02f, 0.0f};
    camera->lookAt(eye, target, {0, 1, 0});
    camera->setProjection(34.0, double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);

    auto make_target = [&](Texture::InternalFormat format) {
        Texture* color = Texture::Builder()
                             .width(W)
                             .height(H)
                             .levels(1)
                             .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                             .format(format)
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
        return std::tuple<Texture*, RenderTarget*>{color, rt};
    };
    auto [colorA, rtA] = make_target(Texture::InternalFormat::RGBA16F);
    auto [colorB, rtB] = make_target(Texture::InternalFormat::RGBA16F);
    auto [color8, rt8] = make_target(Texture::InternalFormat::RGBA8);
    if (!colorA || !colorB || !color8 || !rtA || !rtB || !rt8) return fail("target");
    Texture* colorF[2] = {colorA, colorB};
    RenderTarget* rtF[2] = {rtA, rtB};

    auto ibl_bytes = load_file("out/ibl/ibl_ibl.ktx");
    auto sky_bytes = load_file("out/ibl/ibl_skybox.ktx");
    if (ibl_bytes.empty() || sky_bytes.empty()) return fail("ktx");
    Texture* ibl_tex = ktxreader::Ktx1Reader::createTexture(
        engine, new image::Ktx1Bundle(ibl_bytes.data(), (uint32_t)ibl_bytes.size()), false);
    Texture* sky_tex = ktxreader::Ktx1Reader::createTexture(
        engine, new image::Ktx1Bundle(sky_bytes.data(), (uint32_t)sky_bytes.size()), false);
    if (!ibl_tex || !sky_tex) return fail("cubemap");
    float3 sh[9];
    if (!load_sh("out/ibl/sh.txt", sh)) return fail("sh");
    IndirectLight* ibl = IndirectLight::Builder().reflections(ibl_tex).irradiance(3, sh).intensity(ibl_intensity).build(*engine);
    Skybox* room_sky = Skybox::Builder().environment(sky_tex).showSun(false).build(*engine);
    Skybox* dark_sky = Skybox::Builder().color({0.008f, 0.007f, 0.009f, 1}).build(*engine);
    Skybox* sky = dark_sky;
    std::vector<std::vector<float>> env_mips;
    if (chrome_shot) {
        const int N = 32;
        const int levels = 6;
        auto paint = [](std::vector<float>& dst, int n, float3 base, float3 blob, float bx, float by, float rad) {
            dst.assign((size_t)n * n * 4, 0.f);
            for (int y = 0; y < n; ++y)
                for (int x = 0; x < n; ++x) {
                    float u = ((x + 0.5f) / (float)n) * 2.f - 1.f;
                    float v = ((y + 0.5f) / (float)n) * 2.f - 1.f;
                    float d = std::sqrt((u - bx) * (u - bx) + (v - by) * (v - by));
                    float w = std::clamp(1.f - d / rad, 0.f, 1.f);
                    w = w * w;
                    float3 c = base * (1.f - w) + blob * w;
                    size_t i = ((size_t)y * n + x) * 4;
                    dst[i] = c.x;
                    dst[i + 1] = c.y;
                    dst[i + 2] = c.z;
                    dst[i + 3] = 1.f;
                }
        };
        auto down = [](const std::vector<float>& src, int n, std::vector<float>& dst) {
            int m = n / 2;
            dst.assign((size_t)m * m * 4, 0.f);
            for (int y = 0; y < m; ++y)
                for (int x = 0; x < m; ++x) {
                    float3 c{0, 0, 0};
                    for (int dy = 0; dy < 2; ++dy)
                        for (int dx = 0; dx < 2; ++dx) {
                            size_t i = ((size_t)(y * 2 + dy) * n + (x * 2 + dx)) * 4;
                            c.x += src[i];
                            c.y += src[i + 1];
                            c.z += src[i + 2];
                        }
                    c = c * 0.25f;
                    size_t o = ((size_t)y * m + x) * 4;
                    dst[o] = c.x;
                    dst[o + 1] = c.y;
                    dst[o + 2] = c.z;
                    dst[o + 3] = 1.f;
                }
        };
        Texture* env = Texture::Builder()
                           .width(N)
                           .height(N)
                           .levels((uint8_t)levels)
                           .sampler(Texture::Sampler::SAMPLER_CUBEMAP)
                           .format(Texture::InternalFormat::RGBA16F)
                           .build(*engine);
        float3 bases[6] = {{1.2f, 9.f, 12.f}, {14.f, 5.5f, 1.1f}, {2.2f, 2.2f, 2.6f},
                           {0.45f, 0.35f, 0.55f}, {12.f, 9.f, 3.5f}, {7.f, 1.2f, 9.f}};
        float3 blobs[6] = {{4.f, 40.f, 48.f}, {55.f, 22.f, 4.f}, {10.f, 10.f, 12.f},
                           {2.f, 1.6f, 2.2f}, {50.f, 46.f, 36.f}, {40.f, 4.f, 26.f}};
        float bxy[6][3] = {{0.2f, 0.1f, 0.55f}, {-0.15f, -0.05f, 0.7f}, {0.f, 0.f, 1.2f},
                           {0.f, 0.f, 1.4f}, {0.35f, 0.25f, 0.35f}, {-0.2f, 0.15f, 0.6f}};
        for (int face = 0; face < 6; ++face) {
            std::vector<float> level;
            paint(level, N, bases[face], blobs[face], bxy[face][0], bxy[face][1], bxy[face][2]);
            int n = N;
            for (int lv = 0; lv < levels; ++lv) {
                env_mips.push_back(level);
                env->setImage(*engine, (size_t)lv, 0, 0, (uint32_t)face, (uint32_t)n, (uint32_t)n, 1,
                              Texture::PixelBufferDescriptor(env_mips.back().data(), env_mips.back().size() * sizeof(float),
                                                             backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
                if (lv + 1 < levels) {
                    std::vector<float> next;
                    down(level, n, next);
                    level.swap(next);
                    n /= 2;
                }
            }
        }
        float3 sh_env[9] = {};
        sh_env[0] = float3{0.35f, 0.22f, 0.16f};
        ibl = IndirectLight::Builder().reflections(env).irradiance(3, sh_env).intensity(ibl_intensity).build(*engine);
        sky = Skybox::Builder().environment(env).showSun(false).build(*engine);
    }
    scene->setIndirectLight(ibl);

    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    ColorGrading* grading = ColorGrading::Builder().toneMapper(&agx).contrast(1.06f).saturation(1.14f).build(*engine);

    view->setScene(scene);
    view->setCamera(camera);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setPostProcessingEnabled(false);
    view->setScreenSpaceRefractionEnabled(false);
    view->setColorGrading(grading);
    view->setAntiAliasing(View::AntiAliasing::NONE);
    view->setDithering(View::Dithering::NONE);
    ScreenSpaceReflectionsOptions ssr{};
    ssr.enabled = false;
    view->setScreenSpaceReflectionsOptions(ssr);
    BloomOptions bloom{};
    bloom.enabled = false;
    view->setBloomOptions(bloom);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0.01f, 0.008f, 0.006f, 1};
    clear.clear = true;
    renderer->setClearOptions(clear);

    auto layer_pkg = load_file("glass_layer.filamat");
    auto gpu_pkg = load_file("glass_gpu.filamat");
    auto plate_pkg = load_file("plate.filamat");
    auto chrome_pkg = load_file("chrome_aniso.filamat");
    auto emit_pkg = load_file("emissive.filamat");
    auto floor_pkg = load_file("floor_lit.filamat");
    if (layer_pkg.empty() || gpu_pkg.empty() || plate_pkg.empty() || chrome_pkg.empty() || emit_pkg.empty() || floor_pkg.empty())
        return fail("filamat");
    Material* layer_mat = Material::Builder().package(layer_pkg.data(), layer_pkg.size()).build(*engine);
    Material* gpu_mat = Material::Builder().package(gpu_pkg.data(), gpu_pkg.size()).build(*engine);
    Material* plate_mat = Material::Builder().package(plate_pkg.data(), plate_pkg.size()).build(*engine);
    Material* chrome_mat = Material::Builder().package(chrome_pkg.data(), chrome_pkg.size()).build(*engine);
    Material* emit_mat = Material::Builder().package(emit_pkg.data(), emit_pkg.size()).build(*engine);
    Material* floor_mat = Material::Builder().package(floor_pkg.data(), floor_pkg.size()).build(*engine);
    if (!layer_mat || !gpu_mat || !plate_mat || !chrome_mat || !emit_mat || !floor_mat) return fail("material");

    auto emit = [&](float r, float g, float b) {
        MaterialInstance* mi = emit_mat->createInstance();
        mi->setParameter("color", float4{r, g, b, 1});
        return mi;
    };
    auto floor_of = [&](float4 color, float rough) {
        MaterialInstance* mi = floor_mat->createInstance();
        mi->setParameter("color", color);
        mi->setParameter("roughness", rough);
        return mi;
    };
    MaterialInstance* window_mi = emit(clear_shot ? 14000.f : 8000.f, clear_shot ? 5200.f : 2800.f, clear_shot ? 1600.f : 900.f);
    MaterialInstance* slit_mi = emit(1500.f, 22000.f, 42000.f);
    MaterialInstance* bulb_mi = emit(chrome_shot ? 90000.f : 18000.f, chrome_shot ? 80000.f : 14000.f, chrome_shot ? 60000.f : 8000.f);
    MaterialInstance* floor_mi = floor_of({0.04f, 0.035f, 0.038f, 1}, 0.35f);
    MaterialInstance* panel_mi = floor_of({0.02f, 0.018f, 0.024f, 1}, 0.7f);
    MaterialInstance* wall_mi = floor_of({0.015f, 0.013f, 0.016f, 1}, 0.9f);
    MaterialInstance* mullion_mi = floor_of({0.004f, 0.003f, 0.003f, 1}, 0.95f);
    MaterialInstance* chrome_mi = chrome_mat->createInstance();
    chrome_mi->setParameter("anisotropy", chrome_aniso);
    chrome_mi->setParameter("roughness", chrome_rough);
    MaterialInstance* chrome_sharp = chrome_mat->createInstance();
    chrome_sharp->setParameter("anisotropy", 0.05f);
    chrome_sharp->setParameter("roughness", 0.06f);
    MaterialInstance* bar_cols[4] = {emit(70000.f, 68000.f, 62000.f), emit(110000.f, 4000.f, 2000.f),
                                     emit(2000.f, 90000.f, 8000.f), emit(3000.f, 6000.f, 120000.f)};

    auto glass_of = [&](float ior, float thick, float disp, float refl, float3 absorb) {
        MaterialInstance* mi = layer_mat->createInstance();
        mi->setParameter("ior", ior);
        mi->setParameter("transmission", 1.0f);
        mi->setParameter("thickness", thick);
        mi->setParameter("dispersion", disp);
        mi->setParameter("reflectance", refl);
        mi->setParameter("absorption", absorb);
        return mi;
    };
    MaterialInstance* back_mi = glass_of(back_ior, back_thick, back_disp, back_refl, back_abs);
    MaterialInstance* front_mi = glass_of(front_ior, front_thick, front_disp, front_refl, front_abs);

    const int tex_w = 64;
    auto tex_rows = [](int n) { return std::max(1, (n + 63) / 64); };
    std::vector<float> controls((size_t)tex_rows((int)spans.size() * 4) * tex_w * 4, 0.f);
    for (int s = 0; s < (int)spans.size(); ++s)
        for (int k = 0; k < 4; ++k) {
            int i = s * 4 + k;
            controls[(size_t)i * 4 + 0] = spans[s].p[k].x;
            controls[(size_t)i * 4 + 1] = spans[s].p[k].y;
            controls[(size_t)i * 4 + 2] = spans[s].p[k].z;
        }
    int frame_n = (int)mid_s.size() * 2;
    std::vector<float> frame_tex((size_t)tex_rows(frame_n) * tex_w * 4, 0.f);
    for (int i = 0; i < (int)mid_s.size(); ++i) {
        frame_tex[(size_t)(i * 2) * 4 + 0] = mid_s[i].N.x;
        frame_tex[(size_t)(i * 2) * 4 + 1] = mid_s[i].N.y;
        frame_tex[(size_t)(i * 2) * 4 + 2] = mid_s[i].N.z;
        frame_tex[(size_t)(i * 2 + 1) * 4 + 0] = mid_s[i].B.x;
        frame_tex[(size_t)(i * 2 + 1) * 4 + 1] = mid_s[i].B.y;
        frame_tex[(size_t)(i * 2 + 1) * 4 + 2] = mid_s[i].B.z;
    }
    auto float_tex = [&](int w, int h, const std::vector<float>& data) {
        Texture* tex = Texture::Builder()
                           .width((uint32_t)w)
                           .height((uint32_t)h)
                           .levels(1)
                           .sampler(Texture::Sampler::SAMPLER_2D)
                           .format(Texture::InternalFormat::RGBA32F)
                           .build(*engine);
        tex->setImage(*engine, 0,
                      Texture::PixelBufferDescriptor(data.data(), data.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                     backend::PixelDataType::FLOAT, 1, 0, 0, (uint32_t)w));
        return tex;
    };
    Texture* ctl = float_tex(tex_w, tex_rows((int)spans.size() * 4), controls);
    Texture* frm = float_tex(tex_w, tex_rows(frame_n), frame_tex);
    TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    MaterialInstance* gpu_mi = gpu_mat->createInstance();
    gpu_mi->setParameter("controls", ctl, nearest);
    gpu_mi->setParameter("frames", frm, nearest);
    gpu_mi->setParameter("samples", (float)per);
    gpu_mi->setParameter("sides", (float)sides);
    gpu_mi->setParameter("spans", (float)spans.size());
    gpu_mi->setParameter("radius", mid_r);
    gpu_mi->setParameter("texWidth", (float)tex_w);
    gpu_mi->setParameter("texHeight", (float)tex_rows((int)spans.size() * 4));
    gpu_mi->setParameter("frameHeight", (float)tex_rows(frame_n));
    gpu_mi->setParameter("ior", gpu_ior);
    gpu_mi->setParameter("transmission", 1.0f);
    gpu_mi->setParameter("thickness", gpu_thick);
    gpu_mi->setParameter("dispersion", gpu_disp);
    gpu_mi->setParameter("reflectance", gpu_refl);

    TextureSampler plate_sampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    plate_sampler.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    plate_sampler.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* plate_mi[2] = {plate_mat->createInstance(), plate_mat->createInstance()};
    plate_mi[0]->setParameter("color", colorF[0], plate_sampler);
    plate_mi[1]->setParameter("color", colorF[1], plate_sampler);

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
            .boundingBox({{-6, -6, -6}, {6, 6, 6}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, h.vb, h.ib)
            .material(0, mi)
            .culling(false)
            .castShadows(false)
            .receiveShadows(false)
            .build(*engine, h.entity);
        return h;
    };

    std::vector<Hold> room;
    room.push_back(add(quad({0, -0.64f, 0.15f}, {1, 0, 0}, {0, 0, -1}, 2.8f, 2.2f), floor_mi));
    room.push_back(add(quad({0.0f, 0.35f, -1.35f}, {1, 0, 0}, {0, 1, 0}, 2.4f, 1.5f), wall_mi));
    room.push_back(add(quad({-1.05f, 0.25f, 0.05f}, {0, 0, -1}, {0, 1, 0}, 1.15f, 1.2f), panel_mi));
    room.push_back(add(quad({1.28f, 0.22f, -0.05f}, {0, 0, 1}, {0, 1, 0}, 1.15f, 1.2f), panel_mi));
    if (prism) {
        for (int i = -5; i <= 5; ++i) {
            float x = -0.42f + (float)i * 0.20f;
            room.push_back(add(quad({x, 0.34f, -1.20f}, {1, 0, 0}, {0, 1, 0}, 0.055f, 1.05f), bar_cols[(i + 5) % 4]));
        }
    } else if (clear_shot) {
        room.push_back(add(quad({-0.42f, 0.34f, -1.22f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 1.05f), window_mi));
        room.push_back(add(quad({-0.42f, 0.34f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 0.018f, 1.05f), mullion_mi));
        room.push_back(add(quad({-0.95f, 0.34f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 0.016f, 1.05f), mullion_mi));
        room.push_back(add(quad({0.12f, 0.34f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 0.016f, 1.05f), mullion_mi));
        room.push_back(add(quad({-0.42f, 0.82f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 0.016f), mullion_mi));
        room.push_back(add(quad({-0.42f, -0.08f, -1.18f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 0.016f), mullion_mi));
    } else {
        room.push_back(add(quad({-0.85f, 0.40f, -1.15f}, {1, 0, 0}, {0, 1, 0}, 0.38f, 0.70f), emit(50000.f, 18000.f, 3000.f)));
        room.push_back(add(quad({0.55f, 0.55f, -1.05f}, {1, 0, 0}, {0, 1, 0}, 0.28f, 0.42f), emit(2000.f, 8000.f, 70000.f)));
        room.push_back(add(quad({0.15f, 1.05f, -0.40f}, {1, 0, 0}, {0, 0, 1}, 0.55f, 0.16f), emit(60000.f, 8000.f, 40000.f)));
        room.push_back(add(quad({1.15f, 0.20f, 0.15f}, {0, 0, 1}, {0, 1, 0}, 0.12f, 0.70f), slit_mi));
    }
    room.push_back(add(quad({0.28f, 0.72f, 0.42f}, {1, 0, 0}, {0, 1, 0}, 0.035f, 0.035f), bulb_mi));
    Hold chrome = add(tube(chrome_s, sides, chrome_shot ? back_r : chrome_r, false), chrome_mi);
    Hold chrome2 = add(tube(chrome2_s, sides, chrome2_r, false), chrome_sharp);
    Hold glasses[3] = {
        add(tube(back_s, sides, back_r, false), back_mi),
        add(tube(mid_s, sides, mid_r, true), gpu_mi),
        add(front_is_ribbon ? ribbon(front_s, front_r) : tube(front_s, sides, front_r, false), front_mi),
    };
    (void)room_sky;

    Hold plate;
    std::vector<float> plate_v(4 * 9, 0.f);
    std::vector<uint32_t> plate_idx = {0, 1, 2, 0, 2, 3};
    plate.vb = VertexBuffer::Builder()
                   .vertexCount(4)
                   .bufferCount(1)
                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36)
                   .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36)
                   .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36)
                   .build(*engine);
    plate.ib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    plate.ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(plate_idx.data(), plate_idx.size() * sizeof(uint32_t)));
    plate.entity = utils::EntityManager::get().create();
    RenderableManager::Builder(1)
        .boundingBox({{-30, -30, -30}, {30, 30, 30}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, plate.vb, plate.ib)
        .material(0, plate_mi[0])
        .culling(false)
        .castShadows(false)
        .receiveShadows(false)
        .build(*engine, plate.entity);
    {
        float3 f = normalize(target - eye);
        float3 r = normalize(cross(f, float3{0, 1, 0}));
        float3 u = cross(r, f);
        float dist = 8.0f;
        float hh = dist * std::tan(17.0f * PI / 180.f);
        float hw = hh * ((float)W / (float)H);
        float3 c = eye + f * dist;
        float3 corner[4] = {c - r * hw - u * hh, c + r * hw - u * hh, c + r * hw + u * hh, c - r * hw + u * hh};
        float uv[4][2] = {{0, 0}, {1, 0}, {1, 1}, {0, 1}};
        V3 T{r.x, r.y, r.z};
        V3 Nn = norm(V3{f.x, f.y, f.z} - T * dot(V3{f.x, f.y, f.z}, T));
        V3 B = norm(cross(T, Nn));
        quatf q = mat3f::packTangentFrame(mat3f{float3{T.x, T.y, T.z}, float3{B.x, B.y, B.z}, float3{Nn.x, Nn.y, Nn.z}});
        for (int i = 0; i < 4; ++i) {
            float* o = plate_v.data() + i * 9;
            o[0] = corner[i].x;
            o[1] = corner[i].y;
            o[2] = corner[i].z;
            o[3] = uv[i][0];
            o[4] = uv[i][1];
            o[5] = q.x;
            o[6] = q.y;
            o[7] = q.z;
            o[8] = q.w;
        }
        plate.vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(plate_v.data(), plate_v.size() * sizeof(float)));
    }

    auto lamp = [&](LightManager::Type type, float3 color, float lux, float3 dir, float sun_radius) {
        utils::Entity e = utils::EntityManager::get().create();
        auto b = LightManager::Builder(type).color(color).intensity(lux).direction(normalize(dir)).castShadows(false);
        if (type == LightManager::Type::SUN) b.sunAngularRadius(sun_radius).sunHaloSize(1.4f);
        b.build(*engine, e);
        scene->addEntity(e);
        return e;
    };
    lamp(LightManager::Type::DIRECTIONAL, {1.0f, 0.78f, 0.48f}, 4200.f, {0.42f, -0.22f, 0.88f}, 0);
    lamp(LightManager::Type::DIRECTIONAL, {0.45f, 0.62f, 1.0f}, 900.f, {-0.75f, -0.18f, -0.35f}, 0);
    lamp(LightManager::Type::SUN, {1.0f, 0.96f, 0.90f}, sun_lux, {0.12f, -0.28f, -1.0f}, sun_radius);

    std::vector<utils::Entity> shown;
    auto set_shown = [&](const std::vector<utils::Entity>& next) {
        for (utils::Entity e : shown) scene->remove(e);
        shown = next;
        for (utils::Entity e : shown) scene->addEntity(e);
    };
    auto open_frame = [&]() {
        for (int spin = 0; spin < 4000; ++spin)
            if (renderer->beginFrame(swap)) return true;
        std::printf("beginFrame failed\n");
        return false;
    };

    std::vector<uint8_t> pic((size_t)W * H * 4);
    std::vector<uint16_t> half((size_t)W * H * 4);
    auto render_to = [&](RenderTarget* rt, bool read_half, bool read_byte) {
        view->setRenderTarget(rt);
        renderer->render(view);
        if (read_half) {
            renderer->readPixels(rt, 0, 0, (uint32_t)W, (uint32_t)H,
                                 backend::PixelBufferDescriptor(half.data(), half.size() * sizeof(uint16_t),
                                                                backend::PixelDataFormat::RGBA, backend::PixelDataType::HALF));
        }
        if (read_byte) {
            renderer->readPixels(rt, 0, 0, (uint32_t)W, (uint32_t)H,
                                 backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                                backend::PixelDataType::UBYTE));
        }
    };

    auto& rm = engine->getRenderableManager();
    std::vector<utils::Entity> opaque;
    for (Hold& h : room) opaque.push_back(h.entity);
    opaque.push_back(chrome.entity);
    if (chrome_shot) opaque.push_back(chrome2.entity);
