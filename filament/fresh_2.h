// fresh.mm, part 2 (cut there by size, as it was): read only through fresh.mm, in order.
#pragma once


int main() {
    const int sides = 28;
    const int ring_per = 24;
    std::vector<Sample> spiral = helix(120, 0.30f, 1.45f, -0.52f, 0.72f, {0, 0, 0});
    const float spiral_r = 0.084f;

    std::vector<Cubic> ring_spans;
    for (int i = 0; i < 4; ++i) ring_spans.push_back(arc(0.26f, i * PI * 0.5f, (i + 1) * PI * 0.5f));
    const float tilt = 1.15f;
    float ca = std::cos(tilt), sa = std::sin(tilt);
    V3 ring_at{0.0f, 0.04f, 0.72f};
    for (Cubic& c : ring_spans)
        for (int k = 0; k < 4; ++k) {
            rotX(c.p[k], ca, sa);
            c.p[k] = c.p[k] + ring_at;
        }
    std::vector<Sample> ring = close_frames(ring_spans, ring_per);
    const float ring_r = 0.058f;

    std::vector<Sample> chrome;
    {
        Cubic c{{V3{0.46f, -0.40f, 0.02f}, V3{0.70f, 0.00f, 0.16f}, V3{0.66f, 0.38f, 0.04f}, V3{0.42f, 0.64f, 0.10f}}};
        for (int i = 0; i < 40; ++i) push_on_cubic(chrome, c, (float)i / 39.f);
    }
    const float chrome_r = 0.030f;

    float g_sc = gap(spiral, chrome), g_sr = gap(spiral, ring), g_cr = gap(chrome, ring);
    std::printf("gaps spiral-chrome %.3f>%.3f spiral-ring %.3f>%.3f chrome-ring %.3f>%.3f\n", g_sc, spiral_r + chrome_r,
                g_sr, spiral_r + ring_r, g_cr, chrome_r + ring_r);
    if (g_sc < spiral_r + chrome_r + 0.01f || g_sr < spiral_r + ring_r + 0.01f || g_cr < chrome_r + ring_r + 0.01f)
        return fail("intersect");

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(16.0f, 1.0f / 125.0f, 100.0f);
    const float3 eye{0.22f, 0.22f, 1.28f};
    const float3 target{0.04f, 0.06f, 0.12f};
    const float fov = 28.f;
    camera->lookAt(eye, target, {0, 1, 0});
    camera->setProjection(fov, double(W) / double(H), 0.05, 40.0, Camera::Fov::VERTICAL);

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
    if (!colorA || !colorB || !color8) return fail("target");
    Texture* colorF[2] = {colorA, colorB};
    RenderTarget* rtF[2] = {rtA, rtB};

    auto ibl_bytes = load_file("out/ibl/ibl_ibl.ktx");
    auto sky_bytes = load_file("out/ibl/ibl_skybox.ktx");
    if (ibl_bytes.empty() || sky_bytes.empty()) return fail("ktx");
    Texture* ibl_tex = ktxreader::Ktx1Reader::createTexture(
        engine, new image::Ktx1Bundle(ibl_bytes.data(), (uint32_t)ibl_bytes.size()), false);
    if (!ibl_tex) return fail("cubemap");
    float3 sh[9];
    if (!load_sh("out/ibl/sh.txt", sh)) return fail("sh");
    IndirectLight* ibl = IndirectLight::Builder().reflections(ibl_tex).irradiance(3, sh).intensity(1800.f).build(*engine);
    Skybox* sky = Skybox::Builder().color({0.006f, 0.006f, 0.008f, 1}).showSun(false).build(*engine);
    scene->setIndirectLight(ibl);
    (void)sky_bytes;

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
    clear.clearColor = {0.004f, 0.004f, 0.006f, 1};
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
    MaterialInstance* floor_mi = floor_of({0.03f, 0.028f, 0.032f, 1}, 0.4f);
    MaterialInstance* warm = emit(160000.f, 1500.f, 800.f);
    MaterialInstance* green = emit(800.f, 150000.f, 1500.f);
    MaterialInstance* cool = emit(800.f, 2500.f, 170000.f);
    MaterialInstance* horizon = emit(140000.f, 140000.f, 130000.f);
    MaterialInstance* spiral_mi = glass_of(2.05f, 1.7f, 36.f, 0.18f, {0.01f, 0.01f, 0.01f});
    MaterialInstance* chrome_mi = chrome_mat->createInstance();
    chrome_mi->setParameter("anisotropy", 0.9f);
    chrome_mi->setParameter("roughness", 0.18f);

    const int tex_w = 64;
    auto tex_rows = [](int n) { return std::max(1, (n + 63) / 64); };
    std::vector<float> controls((size_t)tex_rows((int)ring_spans.size() * 4) * tex_w * 4, 0.f);
    for (int s = 0; s < (int)ring_spans.size(); ++s)
        for (int k = 0; k < 4; ++k) {
            int i = s * 4 + k;
            controls[(size_t)i * 4 + 0] = ring_spans[s].p[k].x;
            controls[(size_t)i * 4 + 1] = ring_spans[s].p[k].y;
            controls[(size_t)i * 4 + 2] = ring_spans[s].p[k].z;
        }
    int frame_n = (int)ring.size() * 2;
    std::vector<float> frame_tex((size_t)tex_rows(frame_n) * tex_w * 4, 0.f);
    for (int i = 0; i < (int)ring.size(); ++i) {
        frame_tex[(size_t)(i * 2) * 4 + 0] = ring[i].N.x;
        frame_tex[(size_t)(i * 2) * 4 + 1] = ring[i].N.y;
        frame_tex[(size_t)(i * 2) * 4 + 2] = ring[i].N.z;
        frame_tex[(size_t)(i * 2 + 1) * 4 + 0] = ring[i].B.x;
        frame_tex[(size_t)(i * 2 + 1) * 4 + 1] = ring[i].B.y;
        frame_tex[(size_t)(i * 2 + 1) * 4 + 2] = ring[i].B.z;
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
    TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    MaterialInstance* gpu_mi = gpu_mat->createInstance();
    gpu_mi->setParameter("controls", float_tex(tex_w, tex_rows((int)ring_spans.size() * 4), controls), nearest);
    gpu_mi->setParameter("frames", float_tex(tex_w, tex_rows(frame_n), frame_tex), nearest);
    gpu_mi->setParameter("samples", (float)ring_per);
    gpu_mi->setParameter("sides", (float)sides);
    gpu_mi->setParameter("spans", (float)ring_spans.size());
    gpu_mi->setParameter("radius", ring_r);
    gpu_mi->setParameter("texWidth", (float)tex_w);
    gpu_mi->setParameter("texHeight", (float)tex_rows((int)ring_spans.size() * 4));
    gpu_mi->setParameter("frameHeight", (float)tex_rows(frame_n));
    gpu_mi->setParameter("ior", 2.15f);
    gpu_mi->setParameter("transmission", 1.0f);
    gpu_mi->setParameter("thickness", 1.55f);
    gpu_mi->setParameter("dispersion", 52.f);
    gpu_mi->setParameter("reflectance", 0.14f);

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
    room.push_back(add(quad({0, -0.72f, 0.1f}, {1, 0, 0}, {0, 0, -1}, 2.4f, 1.6f), floor_mi));
    room.push_back(add(quad({-0.55f, 0.15f, -1.05f}, {1, 0, 0}, {0, 1, 0}, 0.34f, 0.85f), warm));
    room.push_back(add(quad({0.08f, 0.22f, -1.08f}, {1, 0, 0}, {0, 1, 0}, 0.28f, 0.78f), green));
    room.push_back(add(quad({0.70f, 0.18f, -1.05f}, {1, 0, 0}, {0, 1, 0}, 0.30f, 0.85f), cool));
    room.push_back(add(quad({0.05f, -0.16f, -1.02f}, {1, 0, 0}, {0, 1, 0}, 1.15f, 0.045f), horizon));
    Hold chrome_h = add(tube(chrome, 24, chrome_r, false, false), chrome_mi);
    Hold spiral_h = add(tube(spiral, sides, spiral_r, false, false), spiral_mi);
    Hold ring_h = add(tube(ring, sides, ring_r, true, true), gpu_mi);

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
        float hh = dist * std::tan(fov * 0.5f * PI / 180.f);
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

    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1.0f, 0.95f, 0.88f})
        .intensity(48000.f)
        .direction(normalize(float3{0.35f, -0.25f, -1.0f}))
        .sunAngularRadius(0.42f)
        .sunHaloSize(1.3f)
        .castShadows(false)
        .build(*engine, sun);
    scene->addEntity(sun);

    std::vector<utils::Entity> shown;
    auto set_shown = [&](const std::vector<utils::Entity>& next) {
        for (utils::Entity e : shown) scene->remove(e);
        shown = next;
        for (utils::Entity e : shown) scene->addEntity(e);
    };
    auto open_frame = [&]() {
        for (int spin = 0; spin < 4000; ++spin)
            if (renderer->beginFrame(swap)) return true;
        return false;
    };
    std::vector<uint8_t> pic((size_t)W * H * 4);
    auto render_to = [&](RenderTarget* rt, bool read_byte) {
        view->setRenderTarget(rt);
        renderer->render(view);
        if (read_byte)
            renderer->readPixels(rt, 0, 0, (uint32_t)W, (uint32_t)H,
                                 backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                                backend::PixelDataType::UBYTE));
    };
    auto& rm = engine->getRenderableManager();
    std::vector<utils::Entity> opaque;
    for (Hold& h : room) opaque.push_back(h.entity);
    opaque.push_back(chrome_h.entity);
    utils::Entity glasses[2] = {spiral_h.entity, ring_h.entity};

    for (int frame = 0; frame < 2; ++frame) {
        bool last = frame == 1;
        if (!open_frame()) return fail("begin");
        bloom.enabled = false;
        view->setBloomOptions(bloom);
        view->setPostProcessingEnabled(false);
        view->setAntiAliasing(View::AntiAliasing::NONE);
        view->setScreenSpaceRefractionEnabled(false);
        scene->setSkybox(sky);
        set_shown(opaque);
        render_to(rtF[0], false);
        scene->setSkybox(nullptr);
        view->setScreenSpaceRefractionEnabled(true);
        int src = 0;
        for (int g = 0; g < 2; ++g) {
            int dst = 1 - src;
            rm.setMaterialInstanceAt(rm.getInstance(plate.entity), 0, plate_mi[src]);
            set_shown({plate.entity, glasses[g]});
            render_to(rtF[dst], false);
            src = dst;
        }
        bloom.enabled = true;
        bloom.strength = 0.46f;
        bloom.levels = 6;
        bloom.resolution = 1080;
        bloom.threshold = true;
        bloom.highlight = 8.0f;
        bloom.blendMode = BloomOptions::BlendMode::ADD;
        view->setBloomOptions(bloom);
        view->setPostProcessingEnabled(true);
        view->setAntiAliasing(View::AntiAliasing::FXAA);
        view->setScreenSpaceRefractionEnabled(false);
        rm.setMaterialInstanceAt(rm.getInstance(plate.entity), 0, plate_mi[src]);
        set_shown({plate.entity});
        render_to(rt8, last);
        renderer->endFrame();
    }
    engine->flushAndWait();
    if (!write_png("out/fresh.png", W, H, pic.data())) return fail("png");
    std::printf("wrote out/fresh.png\n");
    return 0;
}
