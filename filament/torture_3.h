// torture.mm, part 3 (cut there by size, as it was): read only through torture.mm, in order.
#pragma once


int main() {
    if (tvg::Initializer::init(0) != tvg::Result::Success) return fail("thorvg");
    tvg::Shape* shape = tvg::Shape::gen();

    auto solid_pkg = load_file("solid.filamat");
    auto stripe_pkg = load_file("stripes.filamat");
    if (solid_pkg.empty() || stripe_pkg.empty()) return fail("filamat");

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
    view->setScreenSpaceRefractionEnabled(false);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setDithering(View::Dithering::NONE);
    BloomOptions bloom;
    bloom.enabled = false;
    view->setBloomOptions(bloom);
    Renderer::ClearOptions clear{};
    clear.clearColor = {0.015f, 0.016f, 0.02f, 1};
    clear.clear = true;
    renderer->setClearOptions(clear);

    Material* solid_mat = Material::Builder().package(solid_pkg.data(), solid_pkg.size()).build(*engine);
    Material* stripe_mat = Material::Builder().package(stripe_pkg.data(), stripe_pkg.size()).build(*engine);
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

    Gpu gpu{engine, scene, flat};
    Frame frame{renderer, view, swap, rt, engine};
    FILE* log = std::fopen("out/torture.txt", "w");
    if (!log) return fail("log");
    std::fprintf(log,
        "Path torture. link.mm と同じ pucker とチューブ。インデックスは uint16 のまま。\n"
        "描画は 1920x1080 の無地ピンク。ブルームと屈折は切ってある。形状を見るため。\n"
        "upload は setBufferAt のあと flushAndWait。draw はその次のフレームで、読み戻しは含まない。\n"
        "頂点数 65535 を超えるメッシュはインデックスを作らず、転送もしない。\n"
        "チューブ半径 %.3f、1 スパン %d サンプル、円周 %d。1 スパンあたり %d 頂点。uint16 に入るスパンは %d まで。\n\n",
        TUBE_R, SAMPLES, SIDES, SAMPLES * SIDES, MAX_U16 / (SAMPLES * SIDES));

    auto run_frames = [&](const char* name, int frames, auto source_at, auto amount_at, bool geometry) {
        Agg a;
        for (int i = 0; i < frames; ++i) {
            float amount = amount_at(i, frames);
            Path2 source = source_at(i);
            if (i == 0) std::printf(".. %s\n", name);
            auto t0 = std::chrono::steady_clock::now();
            Hand h = handoff(shape, source, amount, geometry);
            Times t;
            t.path = ms_since(t0) - (h.mesh_ok ? h.mesh.sample_ms + h.mesh.mesh_ms : 0);
            if (t.path < 0) t.path = 0;
            if (h.mesh_ok) {
                t.sample = h.mesh.sample_ms;
                t.mesh = h.mesh.mesh_ms;
                t.uploaded = upload_verts(gpu, h.mesh, t.upload);
                if (t.uploaded) t.drew = draw_frame(frame, nullptr, &t.draw);
            }
            accumulate(a, h, t);
            std::fflush(stdout);
        }
        report_agg(log, name, a);
        std::fflush(log);
        return a;
    };

    const Path2 rect = rectangle();
    const float amounts[] = {-200, -100, -60, -36, -30, 0, 30, 36, 60, 100, 200};
    std::fprintf(log, "1 極端な Pucker & Bloat  矩形 1 本。amount は ThorVG 符号（正でアンカーが中心へ）。\n");
    for (float amount : amounts) {
        Hand h = handoff(shape, rect, amount, true);
        const char* collide = (h.clearance >= 0 && h.clearance < 2.f * TUBE_R) ? "collide" : "clear";
        std::fprintf(log,
            "  amount %+7.0f  roundtrip %s delta %.3g  radius %.4f  spans %d  verts %d  index %s  spine %.3g  self-x %s  clearance %.4f %s\n",
            amount, h.roundtrip ? "yes" : "NO", h.delta, h.radius, h.mesh.spans,
            h.mesh_ok ? (int)(h.mesh.v.size() / STRIDE) : 0,
            h.mesh.index_overflow ? "OVERFLOW" : "ok", h.mesh.center_err,
            h.self_x ? "YES" : "no", h.clearance, collide);
    }
    run_frames("pucker-anim", 8, [&](int) { return rect; }, [](int i, int n) { return 100.f * std::sin((float)i / n * 6.2831853f); }, true);

    const float sheet_amt[] = {-100, -60, -30, 0, 30, 60, 100};
    const int cols = 7;
    const int cell_w = 480, cell_h = 270;
    std::vector<uint8_t> sheet((size_t)cell_w * cols * cell_h * 2 * 4, 18);
    for (int c = 0; c < cols; ++c) {
        Hand h = handoff(shape, rect, sheet_amt[c], false);
        double up = 0, dr = 0;
        std::vector<uint8_t> picture;
        if (upload_verts(gpu, h.mesh, up)) draw_frame(frame, &picture, &dr);
        if (!picture.empty()) blit(sheet, cell_w * cols, picture.data(), W, H, c * cell_w, 0, cell_w, cell_h);
        Path2 stored;
        read_path(shape, stored);
        Raster d2;
        if (raster_2d(stored, d2, cell_w, cell_h, 150.f, false))
            blit(sheet, cell_w * cols, d2.rgba.data(), d2.w, d2.h, c * cell_w, cell_h, cell_w, cell_h);
    }
    if (!write_png("out/torture-pucker.png", cell_w * cols, cell_h * 2, sheet.data())) return fail("pucker png");
    std::fprintf(log, "  sheet out/torture-pucker.png  上 3D 下 2D  左から amount -100 -60 -30 0 30 60 100\n\n");

    std::fprintf(log, "2 複合パス  楕円 3 つを 1 本の Shape に入れた。Pucker の中心はサブパスごとではなく全体の平均。\n");
    const Path2 compound = compound_shape();
    run_frames("compound", 8, [&](int) { return compound; }, [](int i, int n) { return 18.f * std::sin((float)i / n * 6.2831853f); }, false);
    {
        Hand h = handoff(shape, compound, 0, false);
        double up = 0, dr = 0;
        std::vector<uint8_t> picture;
        std::vector<uint8_t> pair((size_t)1920 * 540 * 4, 18);
        if (upload_verts(gpu, h.mesh, up)) draw_frame(frame, &picture, &dr);
        if (!picture.empty()) blit(pair, 1920, picture.data(), W, H, 0, 0, 960, 540);
        Path2 stored;
        read_path(shape, stored);
        Raster d2;
        if (raster_2d(stored, d2, 960, 540, 280.f, false))
            blit(pair, 1920, d2.rgba.data(), d2.w, d2.h, 960, 0, 960, 540);
        if (!write_png("out/torture-compound.png", 1920, 540, pair.data())) return fail("compound png");
        std::fprintf(log, "  bridges %d  （0 ならサブパスは分かれたまま。正ならリングが次の輪郭へ溶接される）\n", h.bridges);
        std::fprintf(log, "  sheet out/torture-compound.png  左 3D 右 2D\n\n");
    }

    std::fprintf(log, "3 穴  外形 CCW、内形 CW、2D は EvenOdd。チューブは塗りルールを受け取らない。\n");
    const Path2 hole = hole_shape();
    run_frames("hole", 8, [&](int) { return hole; }, [](int i, int n) { return 12.f * std::sin((float)i / n * 6.2831853f); }, false);
    {
        Hand h = handoff(shape, hole, 0, false);
        double up = 0, dr = 0;
        std::vector<uint8_t> picture;
        std::vector<uint8_t> pair((size_t)1920 * 540 * 4, 18);
        if (upload_verts(gpu, h.mesh, up)) draw_frame(frame, &picture, &dr);
        if (!picture.empty()) blit(pair, 1920, picture.data(), W, H, 0, 0, 960, 540);
        Path2 stored;
        read_path(shape, stored);
        Raster d2;
        int hole_px[3] = {0, 0, 0}, body_px[3] = {0, 0, 0};
        if (raster_2d(stored, d2, 960, 540, 280.f, true)) {
            blit(pair, 1920, d2.rgba.data(), d2.w, d2.h, 960, 0, 960, 540);
            auto sample = [&](int x, int y, int* rgb) {
                const uint8_t* p = d2.rgba.data() + ((size_t)y * d2.w + x) * 4;
                rgb[0] = p[0]; rgb[1] = p[1]; rgb[2] = p[2];
            };
            sample(480, 270, hole_px);
            sample(480 + (int)(0.50f * 280.f), 270, body_px);
        }
        if (!write_png("out/torture-hole.png", 1920, 540, pair.data())) return fail("hole png");
        std::fprintf(log, "  bridges %d  2D center rgb %d %d %d  2D body rgb %d %d %d\n",
            h.bridges, hole_px[0], hole_px[1], hole_px[2], body_px[0], body_px[1], body_px[2]);
        std::fprintf(log, "  中心が地色 (18,20,24) なら 2D の穴は開いている。3D は穴ではなく 2 本の輪郭のチューブ。\n");
        std::fprintf(log, "  sheet out/torture-hole.png  左 3D 右 2D\n\n");
    }

    std::fprintf(log, "4 エフェクトの連鎖  毎フレーム pucker のあとにもう一度 pucker。静止画のあとで trimpath と strokeDash。\n");
    {
        Agg chain_agg;
        std::printf(".. chain\n");
        for (int i = 0; i < 8; ++i) {
            float s = std::sin((float)i / 8.f * 6.2831853f);
            auto t0 = std::chrono::steady_clock::now();
            Path2 step = pucker_bloat(pucker_bloat(rect, 40.f * s), -22.f * s);
            bool wrote_chain = write_path(shape, step);
            Path2 stored_chain;
            bool read_chain = wrote_chain && read_path(shape, stored_chain);
            float delta = read_chain ? path_delta(stored_chain, step) : -1.f;
            Times t;
            t.path = ms_since(t0);
            Hand h;
            h.roundtrip = delta >= 0.f && delta < 1e-4f;
            h.delta = delta;
            h.anchors = count_anchors(rect);
            h.subpaths = read_chain ? count_subpaths(stored_chain) : 0;
            if (!wrote_chain) h.error = "appendPath";
            if (read_chain) {
                try {
                    h.mesh = tube_from(stored_chain);
                    h.mesh_ok = true;
                    h.bridges = bridge_count(h.mesh);
                } catch (const std::bad_alloc&) {
                    h.error = "mesh alloc";
                }
            }
            if (h.mesh_ok) {
                t.sample = h.mesh.sample_ms;
                t.mesh = h.mesh.mesh_ms;
                t.uploaded = upload_verts(gpu, h.mesh, t.upload);
                if (t.uploaded) t.drew = draw_frame(frame, nullptr, &t.draw);
            }
            accumulate(chain_agg, h, t);
        }
        report_agg(log, "chain", chain_agg);
        Path2 once = pucker_bloat(rect, 50.f);
        Path2 twice = pucker_bloat(once, -30.f);
        Path2 first = pucker_bloat(rect, 50.f);
        if (!write_path(shape, first)) return fail("chain write");
        Path2 mid;
        read_path(shape, mid);
        Path2 second = pucker_bloat(mid, -30.f);
        if (!write_path(shape, second)) return fail("chain write 2");
        Path2 stored;
        read_path(shape, stored);
        float d_chain = path_delta(stored, twice);
        shape->trimpath(0.15f, 0.72f, true);
        Path2 trimmed;
        read_path(shape, trimmed);
        float d_trim = path_delta(trimmed, stored);
        const float dash[2] = {8.f, 4.f};
        shape->strokeDash(dash, 2, 0.f);
        Path2 dashed;
        read_path(shape, dashed);
        float d_dash = path_delta(dashed, stored);
        std::fprintf(log, "  pucker(50) then pucker(-30)  roundtrip delta %.3g  anchors %d\n", d_chain, count_anchors(stored));
        std::fprintf(log, "  trimpath(0.15, 0.72) のあと path の差 %.3g  （0 ならメッシュに届いていない）\n", d_trim);
        std::fprintf(log, "  strokeDash(8,4) のあと path の差 %.3g  （0 ならメッシュに届いていない）\n", d_dash);
        double up = 0, dr = 0;
        Path2 chained = pucker_bloat(pucker_bloat(rect, 50.f), -30.f);
        write_path(shape, chained);
        read_path(shape, stored);
        Mesh chained_mesh = tube_from(stored);
        std::vector<uint8_t> pair((size_t)1920 * 540 * 4, 18);
        std::vector<uint8_t> picture;
        if (upload_verts(gpu, chained_mesh, up)) draw_frame(frame, &picture, &dr);
        if (!picture.empty()) blit(pair, 1920, picture.data(), W, H, 960, 0, 960, 270);
        Raster d2;
        if (raster_2d(stored, d2, 960, 270, 120.f, false))
            blit(pair, 1920, d2.rgba.data(), d2.w, d2.h, 960, 270, 960, 270);
        Path2 only = pucker_bloat(rect, 50.f);
        write_path(shape, only);
        read_path(shape, stored);
        Mesh one = tube_from(stored);
        picture.clear();
        if (upload_verts(gpu, one, up)) draw_frame(frame, &picture, &dr);
        if (!picture.empty()) blit(pair, 1920, picture.data(), W, H, 0, 0, 960, 270);
        if (raster_2d(stored, d2, 960, 270, 120.f, false))
            blit(pair, 1920, d2.rgba.data(), d2.w, d2.h, 0, 270, 960, 270);
        if (!write_png("out/torture-chain.png", 1920, 540, pair.data())) return fail("chain png");
        std::fprintf(log, "  sheet out/torture-chain.png  左 pucker(50) 右 pucker(50)→pucker(-30)  上 3D 下 2D\n\n");
    }

    std::fprintf(log, "5 アンカー数  波打つ閉路。毎フレーム位相と pucker を変える。\n");
    const int counts[] = {4, 32, 136, 137, 1000, 10000};
    Agg biggest;
    for (int count : counts) {
        int frames = count >= 1000 ? 3 : 8;
        char name[64];
        std::snprintf(name, sizeof(name), "anchors-%d", count);
        Agg a = run_frames(name, frames, [&](int i) { return wave(count, i * 0.35f); },
            [](int i, int) { return 10.f * std::sin((float)i * 0.7f); }, false);
        if (count == 10000) biggest = a;
        if (count == 136 || count == 10000) {
            float phase = 0.4f;
            Path2 src = wave(count, phase);
            Hand h = handoff(shape, src, 10.f, false);
            if (count == 136) {
                double up = 0, dr = 0;
                std::vector<uint8_t> picture;
                std::vector<uint8_t> pair((size_t)1920 * 540 * 4, 18);
                if (upload_verts(gpu, h.mesh, up)) draw_frame(frame, &picture, &dr);
                if (!picture.empty()) blit(pair, 1920, picture.data(), W, H, 0, 0, 960, 540);
                Path2 stored;
                read_path(shape, stored);
                Raster d2;
                if (raster_2d(stored, d2, 960, 540, 280.f, false))
                    blit(pair, 1920, d2.rgba.data(), d2.w, d2.h, 960, 0, 960, 540);
                if (!write_png("out/torture-anchors.png", 1920, 540, pair.data())) return fail("anchors png");
                std::fprintf(log, "  136 anchors sheet out/torture-anchors.png  左 3D 右 2D  uint16 に入る最大のスパン数\n");
            } else {
                Path2 stored;
                read_path(shape, stored);
                Raster d2;
                if (raster_2d(stored, d2, 960, 540, 280.f, false)) {
                    if (!write_png("out/torture-10k.png", d2.w, d2.h, d2.rgba.data())) return fail("10k png");
                    std::fprintf(log, "  10000 anchors は 3D に出していない。2D だけ out/torture-10k.png\n");
                }
                std::fprintf(log, "  10000 roundtrip %s delta %.3g anchors %d spans %d verts %d overflow %s\n",
                    h.roundtrip ? "yes" : "NO", h.delta, h.anchors, h.mesh.spans,
                    h.mesh_ok ? (int)(h.mesh.v.size() / STRIDE) : 0,
                    h.mesh.index_overflow ? "YES" : "no");
            }
        }
    }
    (void)biggest;
    std::fprintf(log, "\n");

    std::fprintf(log, "6 テキスト輪郭  ThorVG Text から PathCommand を取る API があるかを試した。\n");
    const char* font_file = "/System/Library/Fonts/Supplemental/Andale Mono.ttf";
    tvg::Result font_load = tvg::Text::load(font_file);
    auto* text = tvg::Text::gen();
    tvg::Result font_set = text->font("Andale Mono");
    tvg::Result sized = text->size(96.f);
    tvg::Result wrote = text->text("PATH");
    text->fill(236, 224, 196);
    text->translate(280.f, 300.f);
    tvg::GlyphMetrics glyph{};
    tvg::Result glyph_ok = text->metrics("P", glyph);
    std::fprintf(log, "  font load %d  font() %d  size %d  text %d  glyph metrics %d\n",
        (int)font_load, (int)font_set, (int)sized, (int)wrote, (int)glyph_ok);
    std::fprintf(log, "  glyph P advance %.2f bounds (%.1f,%.1f)-(%.1f,%.1f)\n",
        glyph.advance, glyph.min.x, glyph.min.y, glyph.max.x, glyph.max.y);
    std::fprintf(log, "  Text は Shape ではない。path() も appendPath に渡す輪郭もない。メッシュは作っていない。\n");
    {
        const int tw = 960, th = 540;
        std::vector<uint32_t> buf((size_t)tw * th, 0);
        auto* canvas = tvg::SwCanvas::gen();
        canvas->target(buf.data(), tw, tw, th, tvg::ColorSpace::ABGR8888S);
        auto* bg = tvg::Shape::gen();
        bg->appendRect(0, 0, (float)tw, (float)th);
        bg->fill(18, 20, 24, 255);
        canvas->add(bg);
        canvas->add(text);
        canvas->draw(true);
        canvas->sync();
        std::vector<uint8_t> rgba((size_t)tw * th * 4);
        for (int i = 0; i < tw * th; ++i) {
            uint32_t px = buf[i];
            rgba[(size_t)i * 4 + 0] = (uint8_t)(px & 255);
            rgba[(size_t)i * 4 + 1] = (uint8_t)((px >> 8) & 255);
            rgba[(size_t)i * 4 + 2] = (uint8_t)((px >> 16) & 255);
            rgba[(size_t)i * 4 + 3] = 255;
        }
        if (!write_png("out/torture-text.png", tw, th, rgba.data())) return fail("text png");
        std::fprintf(log, "  2D の文字は出た。out/torture-text.png  3D は無し。\n");
    }
    double text_ms = 0;
    for (int i = 0; i < 8; ++i) {
        auto t0 = std::chrono::steady_clock::now();
        text->size(72.f + 24.f * std::sin((float)i / 8.f));
        text->text("PATH");
        text_ms += ms_since(t0);
    }
    std::fprintf(log, "  8 フレーム size を変えた。合計 %.2f ms。Text に path() は無いので、どのフレームもメッシュに渡していない。\n", text_ms);

    std::fprintf(log,
        "\n限界\n"
        "  パスの往復は ThorVG の Shape::path。ここで失敗したら以降は無意味。\n"
        "  メッシュの打ち止めは uint16。スパン %d まで。137 以上は転送しない。\n"
        "  複合パスと穴はパスとしては届く。チューブはサブパスを 1 本のリングに溶接する。塗りと穴は Filament に渡るものがない。\n"
        "  trimpath と strokeDash は描画の属性で、path() が変わらなければメッシュは元の形のまま。\n"
        "  テキストは輪郭がパスにならない。\n",
        MAX_U16 / (SAMPLES * SIDES));
    std::fclose(log);
    std::printf("PASS torture\n");
    return 0;
}
