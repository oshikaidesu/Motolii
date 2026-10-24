// collage.mm, part 2 (cut there by size, as it was): read only through collage.mm, in order.
#pragma once


static void draw_back(tvg::Canvas* canvas, const char* font) {
    canvas->add(rect(0, 0, W, H, 246, 246, 244));
    checker(canvas, GX0, 96, GW0 - GX0 - 24, 168, 16);
    checker(canvas, GW0, 96, GW1 - GW0, 144, 16);
    checker(canvas, GX0, 840, GW0 - GX0 - 24, 168, 16);
    checker(canvas, GR0, 864, GR1 - GR0, 144, 16);

    auto* rules = tvg::Shape::gen();
    rules->moveTo(GX0, 96);
    rules->lineTo(GW0 - 40, 96);
    rules->moveTo(GW1 + 80, GWY);
    rules->lineTo(GX1, GWY);
    rules->moveTo(GX0, GWY + GWH);
    rules->lineTo(GW0 - 60, GWY + GWH);
    rules->moveTo(GX0, 96);
    rules->lineTo(GX0, GRULE - 40);
    rules->strokeWidth(0.8f);
    rules->strokeFill(20, 20, 24, 255);
    canvas->add(rules);

    canvas->add(rect(GW0, GWY, GW1 - GW0, GWH, 10, 10, 12));
    checker(canvas, GW0 - 36, GWY - 28, 200, 110, 14);
    checker(canvas, GW1 - 80, GWY + GWH - 40, 220, 120, 14);
    auto sky = photo(560, 320, false);
    auto* pic = tvg::Picture::gen();
    pic->load(sky.data(), 560, 320, tvg::ColorSpace::ABGR8888S, true);
    pic->size(580, 320);
    pic->translate(GW0 - 48, GWY - 56);
    canvas->add(pic);
    auto sea = photo(340, 200, true);
    auto* sea_pic = tvg::Picture::gen();
    sea_pic->load(sea.data(), 340, 200, tvg::ColorSpace::ABGR8888S, true);
    sea_pic->size(320, 180);
    sea_pic->translate(GW1 - 140, GWY + GWH - 70);
    canvas->add(sea_pic);

    auto* clouds = tvg::Shape::gen();
    clouds->appendCircle(860, GWY, 120, 90);
    clouds->appendCircle(740, GWY + 24, 70, 56);
    clouds->appendCircle(980, GWY + 16, 84, 64);
    clouds->fill(214, 214, 216, 255);
    canvas->add(clouds);
    auto* dots = tvg::Shape::gen();
    for (int i = 0; i < 120; ++i) {
        float a = i * 2.399f;
        float rad = 8.f + (i % 10) * 8.f;
        dots->appendCircle(860 + std::cos(a) * rad, GWY + std::sin(a) * rad * 0.7f, 1.4f, 1.4f);
    }
    dots->fill(80, 80, 88, 255);
    canvas->add(dots);

    auto* red = tvg::Shape::gen();
    splat(red, GW0 - 20, GWY + 70, 78, 40, 0.6f);
    splat(red, 700, GWY + 90, 56, 32, 1.4f);
    splat(red, 1100, 500, 90, 48, 0.3f);
    red->fill(226, 24, 58, 230);
    canvas->add(red);

    Rng rng{11};
    auto* pink = tvg::Shape::gen();
    auto* white_fan = tvg::Shape::gen();
    for (int i = 0; i < 56; ++i) {
        float ang = (i < 36) ? 0.05f + rng.r(0.f, 1.2f) : -1.1f + rng.r(0.f, 0.7f);
        float len = rng.r(140.f, 780.f);
        float x = (i % 3 == 0 ? GW0 + 40.f : 1180.f) + rng.r(-40.f, 40.f);
        float y = (i % 3 == 0 ? GWY + 80.f : 560.f) + rng.r(-30.f, 30.f);
        float x1 = x - std::cos(ang) * len, y1 = y + std::sin(ang) * len;
        float w = rng.r(5.f, 28.f);
        float nx = std::sin(ang) * w, ny = std::cos(ang) * w;
        tvg::Shape* dst = (i % 6 == 0) ? white_fan : pink;
        dst->moveTo(x, y);
        dst->lineTo(x1 + nx, y1 + ny);
        dst->lineTo(x1 - nx * 0.35f, y1 - ny * 0.35f);
        dst->close();
    }
    pink->fill(255, 42, 86, 230);
    white_fan->fill(255, 255, 255, 220);
    canvas->add(pink);
    canvas->add(white_fan);

    auto* blue = tvg::Shape::gen();
    auto* blue_fat = tvg::Shape::gen();
    for (int i = 0; i < 28; ++i) {
        float ang = -0.9f + rng.r(0.f, 1.3f);
        float len = rng.r(160.f, 820.f);
        float x = 980 + rng.r(-80.f, 120.f), y = 340 + rng.r(-40.f, 80.f);
        tvg::Shape* dst = (i % 5 == 0) ? blue_fat : blue;
        dst->moveTo(x, y);
        dst->lineTo(x + std::cos(ang) * len, y + std::sin(ang) * len);
    }
    for (const Crv& c : kCross) cubic(blue, c.x0, c.y0 + 10, c.x1, c.y1 + 12, c.x2, c.y2 - 8, c.x3, c.y3 + 6);
    for (const Crv& c : kMess) cubic(blue, c.x0, c.y0, c.x1, c.y1, c.x2, c.y2, c.x3, c.y3);
    blue->strokeWidth(0.9f);
    blue->strokeFill(32, 72, 200, 200);
    blue_fat->strokeWidth(3.4f);
    blue_fat->strokeFill(20, 50, 180, 180);
    canvas->add(blue);
    canvas->add(blue_fat);

    auto* wires = tvg::Shape::gen();
    wires->appendCircle(250, GRULE, 150, 150);
    wires->appendCircle(250, GRULE, 78, 78);
    wires->strokeWidth(0.9f);
    wires->strokeFill(50, 84, 140, 230);
    canvas->add(wires);

    auto* rule = tvg::Shape::gen();
    rule->moveTo(GX0, GRULE);
    rule->lineTo(GW0 - 24, GRULE);
    for (int i = 0; i < 36; ++i) {
        float x = GX0 + 8 + i * 12.f;
        rule->moveTo(x, GRULE - 8);
        rule->lineTo(x, i % 4 == 0 ? GRULE + 14 : GRULE + 6);
    }
    rule->strokeWidth(1.f);
    rule->strokeFill(16, 16, 20, 255);
    canvas->add(rule);

    const char* words[] = {"one", "[w]", "[m]", "\"", "200", "HYBRID", "A1", "v.08", "px", "ior", "path", "01", "0.00", "comp"};
    for (int i = 0; i < 36; ++i) {
        float x = rng.r(GW0 - 80, GW1 + 60), y = rng.r(GWY - 40, GWY + GWH + 80);
        bool on_black = x > GW0 && x < GW1 && y > GWY && y < GWY + GWH;
        label(canvas, font, x, y, rng.r(7.f, i % 7 == 0 ? 26.f : 13.f), words[i % 14],
              on_black ? 230 : 28, on_black ? 230 : 28, on_black ? 232 : 32);
    }
    label(canvas, font, GX0 + 8, 132, 18, "one", 24, 24, 28);
    label(canvas, font, 460, 250, 34, "HYBRID", 24, 24, 28);
    label(canvas, font, GX0 + 12, GRULE - 36, 28, "\"", 24, 24, 28);
}

static void draw_front(tvg::Canvas* canvas, const char* font) {
    auto* over = tvg::Shape::gen();
    for (const Crv& c : kCross) cubic(over, c.x0, c.y0, c.x1, c.y1, c.x2, c.y2, c.x3, c.y3);
    over->strokeWidth(1.45f);
    over->strokeFill(24, 64, 210, 230);
    canvas->add(over);
    auto* white = tvg::Shape::gen();
    cubic(white, 660, 250, 860, 150, 1080, 200, 1280, 250);
    cubic(white, 720, 340, 940, 230, 1140, 270, 1360, 300);
    white->strokeWidth(2.4f);
    white->strokeFill(255, 255, 255, 230);
    canvas->add(white);
    auto* cut = tvg::Shape::gen();
    cubic(cut, 420, 360, 700, 200, 980, 420, 1400, 260);
    cubic(cut, 200, 700, 560, 480, 900, 640, 1500, 500);
    cut->strokeWidth(4.f);
    cut->strokeFill(24, 24, 28, 210);
    canvas->add(cut);
    label(canvas, font, 760, 300, 22, "path", 16, 16, 20);
    label(canvas, font, 140, 700, 16, "[w]", 16, 16, 20);
    label(canvas, font, 1080, 460, 13, "ior", 245, 245, 247);
    label(canvas, font, 620, 640, 11, "v.08", 245, 245, 247);
}

int main() {
    if (tvg::Initializer::init(0) != tvg::Result::Success) {
        std::fprintf(stderr, "FAIL thorvg\n");
        return 1;
    }
    const char* font_file = "/System/Library/Fonts/Supplemental/Andale Mono.ttf";
    tvg::Result font_ok = tvg::Text::load(font_file);
    const char* font = "Andale Mono";
    std::printf("thorvg font %s\n", font_ok == tvg::Result::Success ? "loaded" : "fallback");

    std::vector<uint32_t> back_px((size_t)W * H, 0xffffffffu);
    auto* back = tvg::SwCanvas::gen();
    back->target(back_px.data(), W, W, H, tvg::ColorSpace::ABGR8888S);
    draw_back(back, font);
    back->draw(true);
    back->sync();

    std::vector<uint32_t> front_px((size_t)W * H, 0);
    auto* front = tvg::SwCanvas::gen();
    front->target(front_px.data(), W, W, H, tvg::ColorSpace::ABGR8888S);
    draw_front(front, font);
    front->draw(false);
    front->sync();

    Lit chrome_mesh, glass_mesh, bubble_mesh;
    Flat shards_back, shards_front;
    const float slab_w[] = {0.72f, 0.42f, 0.58f};
    const float slab_z[] = {0.55f, 0.85f, 1.05f};
    for (int i = 0; i < 3; ++i) {
        std::vector<float3> p, t;
        const Crv& c = kChromeCrv[i];
        sample_cubic(world(c.x0, c.y0, slab_z[i]), world(c.x1, c.y1, slab_z[i]), world(c.x2, c.y2, slab_z[i]), world(c.x3, c.y3, slab_z[i]), 36, p, t);
        slab(chrome_mesh, p, t, slab_w[i]);
    }
    blob(chrome_mesh, 500, 430, 150, 90, 0.7f, 0.35f, 18, 7);
    blob(chrome_mesh, 1180, 250, 120, 70, 1.2f, 0.28f, 16, 9);
    {
        std::vector<float3> p, t;
        const Crv& c = kGlassCrv;
        sample_cubic(world(c.x0, c.y0, 1.15f), world(c.x1, c.y1, 1.15f), world(c.x2, c.y2, 1.15f), world(c.x3, c.y3, 1.15f), 28, p, t);
        slab(glass_mesh, p, t, 0.34f);
    }
    blob(bubble_mesh, 860, 300, 170, 100, 1.25f, 0.4f, 20, 3);
    blob(bubble_mesh, 1080, 390, 130, 80, 1.4f, 0.32f, 16, 5);
    for (int i = 0; i < 7; ++i) {
        float u = (float)i / 6.f, s = 1.f - u;
        const Crv& c = kGlassCrv;
        float px = s * s * s * c.x0 + 3 * s * s * u * c.x1 + 3 * s * u * u * c.x2 + u * u * u * c.x3;
        float py = s * s * s * c.y0 + 3 * s * s * u * c.y1 + 3 * s * u * u * c.y2 + u * u * u * c.y3;
        float jx = ((i * 47) % 11 - 5) * 18.f;
        float jy = ((i * 29) % 9 - 4) * 14.f;
        float rad = 0.42f + 0.28f * std::fabs(std::sin(i * 1.7f));
        sphere(bubble_mesh, world(px + jx, py + jy, 1.35f), rad, 10, 14);
    }

    Rng rng{42};
    auto shard = [&](Flat& mesh, float z, int n, float ang0, float ang1, float len0, float len1, float ox, float oy) {
        for (int i = 0; i < n; ++i) {
            float ang = rng.r(ang0, ang1);
            float len = rng.r(len0, len1);
            float x = ox + rng.r(-18.f, 18.f), y = oy + rng.r(-14.f, 14.f);
            float x1 = x - std::cos(ang) * len, y1 = y + std::sin(ang) * len;
            float w = rng.r(16.f, 42.f);
            float3 a = world(x, y, z);
            float3 b = world(x1, y1, z);
            float3 c = world(x1 + std::sin(ang) * w, y1 + std::cos(ang) * w, z);
            uint32_t base = (uint32_t)(mesh.v.size() / 3);
            for (float3 p : {a, b, c}) mesh.v.insert(mesh.v.end(), {p.x, p.y, p.z});
            mesh.idx.insert(mesh.idx.end(), {base, base + 1, base + 2});
        }
    };
    shard(shards_back, 0.28f, 14, 0.15f, 1.2f, 180.f, 520.f, 1120, 520);
    shard(shards_front, 1.65f, 8, 0.2f, 1.05f, 90.f, 280.f, 1000, 480);
    shard(shards_front, 1.8f, 6, -0.9f, -0.05f, 120.f, 320.f, 900, 300);

    auto glass_pkg = load_file("glass.filamat");
    auto pop_pkg = load_file("glass_pop.filamat");
    auto chrome_pkg = load_file("chrome.filamat");
    auto flat_pkg = load_file("solid.filamat");
    auto tex_pkg = load_file("stripes.filamat");
    if (glass_pkg.empty() || pop_pkg.empty() || chrome_pkg.empty() || flat_pkg.empty() || tex_pkg.empty()) {
        std::fprintf(stderr, "FAIL filamat\n");
        return 1;
    }

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    Camera* camera = engine->createCamera(utils::EntityManager::get().create());
    camera->setExposure(1.f);
    camera->lookAt({0, 0, 12}, {0, 0, 0}, {0, 1, 0});
    camera->setProjection(Camera::Projection::ORTHO, X0, X1, Y0, Y1, 0.05, 40);

    Texture* color = Texture::Builder().width(W).height(H).levels(1)
                         .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                         .format(Texture::InternalFormat::RGBA8).build(*engine);
    Texture* depth = Texture::Builder().width(W).height(H).levels(1)
                         .usage(Texture::Usage::DEPTH_ATTACHMENT).format(Texture::InternalFormat::DEPTH24).build(*engine);
    RenderTarget* rt = RenderTarget::Builder()
                           .texture(RenderTarget::AttachmentPoint::COLOR, color)
                           .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
    static LinearToneMapper tone;
    view->setColorGrading(ColorGrading::Builder().toneMapper(&tone).build(*engine));
    view->setScene(scene);
    view->setCamera(camera);
    view->setRenderTarget(rt);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    ScreenSpaceReflectionsOptions ssr;
    ssr.enabled = false;
    view->setScreenSpaceReflectionsOptions(ssr);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setDithering(View::Dithering::NONE);
    BloomOptions bloom;
    bloom.enabled = false;
    bloom.strength = 0.08f;
    bloom.levels = 4;
    bloom.threshold = true;
    bloom.highlight = 8.f;
    view->setBloomOptions(bloom);
    scene->setSkybox(Skybox::Builder().color({0.96f, 0.96f, 0.95f, 1}).build(*engine));
    constexpr int CUBE = 32;
    auto* faces = new std::vector<uint8_t>((size_t)6 * CUBE * CUBE * 4);
    auto face_px = [&](int f, int x, int y, uint8_t r, uint8_t g, uint8_t b) {
        size_t i = (((size_t)f * CUBE + y) * CUBE + x) * 4;
        (*faces)[i] = r; (*faces)[i + 1] = g; (*faces)[i + 2] = b; (*faces)[i + 3] = 255;
    };
    const uint8_t face_rgb[6][3] = {
        {160, 32, 64}, {24, 48, 140}, {168, 172, 180}, {8, 8, 12}, {48, 50, 56}, {90, 120, 160},
    };
    for (int f = 0; f < 6; ++f) {
        for (int y = 0; y < CUBE; ++y) {
            for (int x = 0; x < CUBE; ++x) {
                uint8_t r = face_rgb[f][0], g = face_rgb[f][1], b = face_rgb[f][2];
                if (f == 4) {
                    if (x > 6 && x < 24 && y > 8 && y < 22) { r = 8; g = 8; b = 12; }
                    if (y > 4 && y < 7) { r = 230; g = 232; b = 236; }
                    if (y > 24 && y < 28) { r = 255; g = 48; b = 90; }
                }
                face_px(f, x, y, r, g, b);
            }
        }
    }
    Texture* ibl_tex = Texture::Builder().width(CUBE).height(CUBE).levels(1)
                           .sampler(Texture::Sampler::SAMPLER_CUBEMAP)
                           .format(Texture::InternalFormat::RGBA8).build(*engine);
    ibl_tex->setImage(*engine, 0, 0, 0, 0, CUBE, CUBE, 6,
        Texture::PixelBufferDescriptor(faces->data(), faces->size(),
            backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE,
            [](void*, size_t, void* user) { delete static_cast<std::vector<uint8_t>*>(user); }, faces));
    float3 sh[1] = {{0.28f, 0.30f, 0.34f}};
    scene->setIndirectLight(IndirectLight::Builder().reflections(ibl_tex).radiance(1, sh).intensity(14.f).build(*engine));

    auto* paper_bytes = new std::vector<uint8_t>((size_t)W * H * 4);
    for (int i = 0; i < W * H; ++i) {
        uint32_t p = back_px[i];
        (*paper_bytes)[(size_t)i * 4 + 0] = (uint8_t)(p & 255);
        (*paper_bytes)[(size_t)i * 4 + 1] = (uint8_t)((p >> 8) & 255);
        (*paper_bytes)[(size_t)i * 4 + 2] = (uint8_t)((p >> 16) & 255);
        (*paper_bytes)[(size_t)i * 4 + 3] = 255;
    }
    Texture* paper = Texture::Builder().width(W).height(H).levels(1).sampler(Texture::Sampler::SAMPLER_2D)
                         .format(Texture::InternalFormat::RGBA8).build(*engine);
    paper->setImage(*engine, 0, Texture::PixelBufferDescriptor(paper_bytes->data(), paper_bytes->size(),
        backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE, 1, 0, 0, W,
        [](void*, size_t, void* user) { delete static_cast<std::vector<uint8_t>*>(user); }, paper_bytes));

    Material* glass_mat = Material::Builder().package(glass_pkg.data(), glass_pkg.size()).build(*engine);
    Material* pop_mat = Material::Builder().package(pop_pkg.data(), pop_pkg.size()).build(*engine);
    Material* chrome_mat = Material::Builder().package(chrome_pkg.data(), chrome_pkg.size()).build(*engine);
    Material* flat_mat = Material::Builder().package(flat_pkg.data(), flat_pkg.size()).build(*engine);
    Material* tex_mat = Material::Builder().package(tex_pkg.data(), tex_pkg.size()).build(*engine);
    MaterialInstance* glass = glass_mat->createInstance();
    glass->setParameter("ior", 2.05f);
    glass->setParameter("transmission", 1.f);
    glass->setParameter("thickness", 0.7f);
    glass->setParameter("dispersion", 0.66f);
    MaterialInstance* pop = pop_mat->createInstance();
    pop->setParameter("ior", 1.92f);
    pop->setParameter("transmission", 1.f);
    pop->setParameter("thickness", 0.9f);
    pop->setParameter("dispersion", 0.58f);
    MaterialInstance* chrome = chrome_mat->createInstance();
    MaterialInstance* pink = flat_mat->createInstance();
    pink->setParameter("color", float4{1.f, 0.16f, 0.32f, 1.f});
    MaterialInstance* paper_mi = tex_mat->createInstance();
    paper_mi->setParameter("albedo", paper, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));

    auto add_lit = [&](Lit& mesh, MaterialInstance* mi) {
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(mesh.v.size() / 7)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                               .build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * 4));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(mesh.idx.data(), mesh.idx.size() * 4));
        utils::Entity e = utils::EntityManager::get().create();
        RenderableManager::Builder(1).boundingBox({{-20, -12, -2}, {20, 12, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
        scene->addEntity(e);
    };
    auto add_flat = [&](Flat& mesh, MaterialInstance* mi) {
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(mesh.v.size() / 3)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12)
                               .build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * 4));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(mesh.idx.data(), mesh.idx.size() * 4));
        utils::Entity e = utils::EntityManager::get().create();
        RenderableManager::Builder(1).boundingBox({{-20, -12, -2}, {20, 12, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi).culling(false).build(*engine, e);
        scene->addEntity(e);
    };

    static const float quad[] = {
        X0, Y0, 0, 0, 0,  X1, Y0, 0, 1, 0,  X1, Y1, 0, 1, 1,  X0, Y1, 0, 0, 1,
    };
    static const uint32_t qidx[] = {0, 1, 2, 0, 2, 3};
    VertexBuffer* qvb = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                            .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                            .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
    qvb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(quad, sizeof(quad)));
    IndexBuffer* qib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    qib->setBuffer(*engine, IndexBuffer::BufferDescriptor(qidx, sizeof(qidx)));
    utils::Entity qe = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-20, -12, -1}, {20, 12, 1}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, qvb, qib)
        .material(0, paper_mi).culling(false).build(*engine, qe);
    scene->addEntity(qe);
    add_flat(shards_back, pink);
    add_lit(chrome_mesh, chrome);
    add_lit(glass_mesh, glass);
    add_lit(bubble_mesh, pop);
    add_flat(shards_front, pink);

    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN).color(float3{1, 0.98f, 0.95f}).intensity(6.f)
        .direction(normalize(float3{0.2f, -0.45f, -0.85f})).sunAngularRadius(0.08f).castShadows(false).build(*engine, sun);
    utils::Entity rim = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::DIRECTIONAL).color(float3{0.55f, 0.75f, 1.f}).intensity(3.f)
        .direction(normalize(float3{-0.6f, -0.2f, 0.5f})).castShadows(false).build(*engine, rim);
    scene->addEntity(sun);
    scene->addEntity(rim);

    std::vector<uint8_t> picture((size_t)W * H * 4);
    double ms = 0;
    for (int pass = 0; pass < 2; ++pass) {
        auto t0 = std::chrono::steady_clock::now();
        bool drew = false;
        for (int attempt = 0; attempt < 4 && !drew; ++attempt) {
            if (!renderer->beginFrame(swap)) continue;
            renderer->render(view);
            if (pass == 1) {
                renderer->readPixels(rt, 0, 0, W, H,
                    backend::PixelBufferDescriptor(picture.data(), picture.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
            }
            renderer->endFrame();
            drew = true;
        }
        engine->flushAndWait();
        ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
        if (!drew) { std::fprintf(stderr, "FAIL frame\n"); return 1; }
    }

    std::vector<uint8_t> out = picture;
    auto at = [&](int x, int y) -> uint8_t* { return out.data() + ((size_t)y * W + x) * 4; };
    auto src_at = [&](const std::vector<uint8_t>& im, int x, int y) -> const uint8_t* {
        return im.data() + ((size_t)y * W + x) * 4;
    };
    for (int y = 0; y < H; ++y) {
        for (int x = 0; x < W; ++x) {
            uint32_t fp = front_px[(size_t)y * W + x];
            int a = (fp >> 24) & 255;
            if (a < 16) continue;
            float af = a / 255.f;
            uint8_t* o = at(x, y);
            o[0] = (uint8_t)((fp & 255) * af + o[0] * (1 - af));
            o[1] = (uint8_t)(((fp >> 8) & 255) * af + o[1] * (1 - af));
            o[2] = (uint8_t)(((fp >> 16) & 255) * af + o[2] * (1 - af));
        }
    }
    std::vector<uint8_t> split = out;
    for (int y = 0; y < H; ++y) {
        for (int x = 0; x < W; ++x) {
            int xr = std::max(0, x - 2), xb = std::min(W - 1, x + 2);
            split[((size_t)y * W + x) * 4 + 0] = src_at(out, xr, y)[0];
            split[((size_t)y * W + x) * 4 + 2] = src_at(out, xb, y)[2];
        }
    }
    for (int y = 0; y < H; ++y) {
        for (int x = 0; x < W; ++x) {
            uint8_t* p = split.data() + ((size_t)y * W + x) * 4;
            float n = (hash01((float)x, (float)y) - 0.5f) / 255.f;
            int paperish = p[0] > 236 && p[1] > 236 && p[2] > 236;
            float amp = paperish ? 1.f : 7.f;
            for (int c = 0; c < 3; ++c) {
                float v = p[c] / 255.f + n * amp;
                p[c] = (uint8_t)std::clamp(v * 255.f, 0.f, 255.f);
            }
        }
    }
    if (!write_png("out/collage.png", W, H, split.data())) return 1;

    FILE* log = std::fopen("out/collage.txt", "w");
    std::fprintf(log,
        "frame 1920x1080\n"
        "thorvg 1.1.2  Shape path, Picture raw load, Text Andale Mono, SwCanvas\n"
        "filament 1.77.2  ortho scene, glass.mat (ior 2.05 transmission 1 thickness 0.7 dispersion 0.66), glass_pop.mat bubbles (ior 1.92 thickness 0.9 dispersion 0.58), chrome.mat, IBL cubemap, refraction. SSR off: a white page reflects as a white sheet.\n"
        "connection  ThorVG pixels are the Filament backdrop, so glass refracts the paper. kChromeCrv / kGlassCrv / kCross are the same cubics as the 2D strokes: chrome ribbons, the glass tube, and thin glass threads. Pink shards share the fan angles, some behind the glass and some in front. kCross is drawn again in a second ThorVG pass composited over the 3D.\n"
        "dither  hash from motolii-glue shelf/blit.wgsl, one LSB on paper, stronger on marks\n"
        "chromatic  2px RGB split on the composite. shelf/glow.wgsl chromatic is hue rotate, not used\n"
        "frame_ms  %.2f  (second frame, includes readback)\n",
        ms);
    std::fclose(log);
    std::printf("frame_ms %.2f\nPASS collage\n", ms);
    return 0;
}
