// torture.mm, part 2 (cut there by size, as it was): read only through torture.mm, in order.
#pragma once


static Path2 wave(int n, float phase) {
    Path2 p;
    p.cmds.reserve((size_t)n + 1);
    p.pts.reserve(n);
    for (int i = 0; i < n; ++i) {
        float ang = (float)i / (float)n * 6.2831853f;
        float r = 0.62f + 0.16f * std::sin(ang * 13.f + phase);
        p.pts.push_back({r * std::cos(ang), r * std::sin(ang)});
        p.cmds.push_back(i ? tvg::PathCommand::LineTo : tvg::PathCommand::MoveTo);
    }
    p.cmds.push_back(tvg::PathCommand::Close);
    return p;
}

static bool write_png(const char* path, int w, int h, const uint8_t* rgba) {
    std::vector<uint8_t> raw((size_t)h * (w * 4 + 1));
    for (int y = 0; y < h; ++y) {
        raw[(size_t)y * (w * 4 + 1)] = 0;
        std::memcpy(raw.data() + (size_t)y * (w * 4 + 1) + 1, rgba + (size_t)y * w * 4, (size_t)w * 4);
    }
    uLongf bound = compressBound((uLong)raw.size());
    std::vector<uint8_t> z(bound);
    if (compress(z.data(), &bound, raw.data(), (uLong)raw.size()) != Z_OK) return false;
    FILE* f = std::fopen(path, "wb");
    if (!f) return false;
    auto chunk = [&](const char* tag, const uint8_t* data, uint32_t n) {
        uint32_t be = __builtin_bswap32(n);
        std::fwrite(&be, 4, 1, f);
        std::fwrite(tag, 1, 4, f);
        if (n) std::fwrite(data, 1, n, f);
        uint32_t crc = crc32(0, (const Bytef*)tag, 4);
        if (n) crc = crc32(crc, data, n);
        crc = __builtin_bswap32(crc);
        std::fwrite(&crc, 4, 1, f);
    };
    const uint8_t sig[8] = {137, 80, 78, 71, 13, 10, 26, 10};
    std::fwrite(sig, 1, 8, f);
    uint8_t ihdr[13];
    uint32_t bw = __builtin_bswap32((uint32_t)w), bh = __builtin_bswap32((uint32_t)h);
    std::memcpy(ihdr, &bw, 4);
    std::memcpy(ihdr + 4, &bh, 4);
    ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = ihdr[11] = ihdr[12] = 0;
    chunk("IHDR", ihdr, 13);
    chunk("IDAT", z.data(), (uint32_t)bound);
    chunk("IEND", nullptr, 0);
    std::fclose(f);
    return true;
}

static std::vector<uint8_t> load_file(const char* path) {
    std::ifstream in(path, std::ios::binary);
    return std::vector<uint8_t>((std::istreambuf_iterator<char>(in)), {});
}

struct Raster {
    std::vector<uint8_t> rgba;
    int w = 0;
    int h = 0;
};

static bool raster_2d(const Path2& path, Raster& out, int w, int h, float scale, bool even_odd) {
    std::vector<uint32_t> buf((size_t)w * h, 0);
    auto* canvas = tvg::SwCanvas::gen();
    if (canvas->target(buf.data(), w, w, h, tvg::ColorSpace::ABGR8888S) != tvg::Result::Success) return false;
    auto* bg = tvg::Shape::gen();
    bg->appendRect(0, 0, (float)w, (float)h);
    bg->fill(18, 20, 24, 255);
    Path2 screen = path;
    for (tvg::Point& q : screen.pts) {
        q.x = q.x * scale + w * 0.5f;
        q.y = -q.y * scale + h * 0.5f;
    }
    auto* shape = tvg::Shape::gen();
    if (!write_path(shape, screen)) return false;
    if (even_odd) {
        shape->fill(236, 224, 196, 255);
        shape->fillRule(tvg::FillRule::EvenOdd);
    }
    shape->strokeWidth(even_odd ? 2.f : 2.4f);
    shape->strokeJoin(tvg::StrokeJoin::Round);
    shape->strokeFill(236, 224, 196, 255);
    canvas->add(bg);
    canvas->add(shape);
    if (canvas->draw(true) != tvg::Result::Success) return false;
    canvas->sync();
    out.w = w; out.h = h;
    out.rgba.resize((size_t)w * h * 4);
    for (int i = 0; i < w * h; ++i) {
        uint32_t px = buf[i];
        out.rgba[(size_t)i * 4 + 0] = (uint8_t)(px & 255);
        out.rgba[(size_t)i * 4 + 1] = (uint8_t)((px >> 8) & 255);
        out.rgba[(size_t)i * 4 + 2] = (uint8_t)((px >> 16) & 255);
        out.rgba[(size_t)i * 4 + 3] = 255;
    }
    return true;
}

static void blit(std::vector<uint8_t>& dst, int dw, const uint8_t* src, int sw, int sh, int x0, int y0, int cw, int ch) {
    for (int y = 0; y < ch; ++y)
        for (int x = 0; x < cw; ++x) {
            int sx = std::min(sw - 1, x * sw / cw);
            int sy = std::min(sh - 1, y * sh / ch);
            std::memcpy(dst.data() + ((size_t)(y0 + y) * dw + x0 + x) * 4, src + ((size_t)sy * sw + sx) * 4, 4);
        }
}

struct Gpu {
    Engine* engine = nullptr;
    Scene* scene = nullptr;
    MaterialInstance* mi = nullptr;
    VertexBuffer* vb = nullptr;
    IndexBuffer* ib = nullptr;
    utils::Entity entity{};
    int vcount = -1;
    int icount = -1;
    bool live = false;
};

static void release_mesh(Gpu& g) {
    if (!g.live) return;
    g.scene->remove(g.entity);
    g.engine->getRenderableManager().destroy(g.entity);
    g.engine->destroy(g.vb);
    g.engine->destroy(g.ib);
    utils::EntityManager::get().destroy(g.entity);
    g.vb = nullptr;
    g.ib = nullptr;
    g.live = false;
    g.vcount = g.icount = -1;
}

static bool ensure_mesh(Gpu& g, const Mesh& m) {
    if (m.index_overflow || m.idx.empty()) return false;
    const int vcount = (int)(m.v.size() / STRIDE);
    const int icount = (int)m.idx.size();
    if (g.live && vcount == g.vcount && icount == g.icount) return true;
    release_mesh(g);
    g.engine->flushAndWait();
    g.vb = VertexBuffer::Builder().vertexCount((uint32_t)vcount).bufferCount(1)
               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, STRIDE * 4)
               .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, STRIDE * 4)
               .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 28, STRIDE * 4)
               .build(*g.engine);
    g.ib = IndexBuffer::Builder().indexCount((uint32_t)icount).bufferType(IndexBuffer::IndexType::USHORT).build(*g.engine);
    g.ib->setBuffer(*g.engine, IndexBuffer::BufferDescriptor(m.idx.data(), m.idx.size() * sizeof(uint16_t)));
    g.entity = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-8, -8, -8}, {8, 8, 8}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, g.vb, g.ib)
        .material(0, g.mi).culling(false).castShadows(false).receiveShadows(false)
        .build(*g.engine, g.entity);
    g.scene->addEntity(g.entity);
    g.vcount = vcount;
    g.icount = icount;
    g.live = true;
    g.engine->flushAndWait();
    return true;
}

static bool upload_verts(Gpu& g, const Mesh& m, double& upload_ms) {
    if (!ensure_mesh(g, m)) return false;
    auto t0 = std::chrono::steady_clock::now();
    g.vb->setBufferAt(*g.engine, 0, VertexBuffer::BufferDescriptor(m.v.data(), m.v.size() * sizeof(float)));
    g.engine->flushAndWait();
    upload_ms = ms_since(t0);
    return true;
}

struct Frame {
    Renderer* renderer;
    View* view;
    SwapChain* swap;
    RenderTarget* rt;
    Engine* engine;
};

static bool draw_frame(Frame& f, std::vector<uint8_t>* picture, double* draw_ms) {
    auto t0 = std::chrono::steady_clock::now();
    bool drew = false;
    for (int attempt = 0; attempt < 4 && !drew; ++attempt) {
        if (!f.renderer->beginFrame(f.swap)) continue;
        f.renderer->render(f.view);
        if (picture) {
            picture->resize((size_t)W * H * 4);
            f.renderer->readPixels(f.rt, 0, 0, W, H, backend::PixelBufferDescriptor(
                picture->data(), picture->size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
        }
        f.renderer->endFrame();
        drew = true;
    }
    f.engine->flushAndWait();
    if (draw_ms) *draw_ms = ms_since(t0);
    return drew;
}

struct Hand {
    bool roundtrip = false;
    float delta = -1;
    int anchors = 0;
    int subpaths = 0;
    Mesh mesh;
    bool mesh_ok = false;
    int bridges = 0;
    bool self_x = false;
    float clearance = -1;
    float radius = 0;
    std::string error;
};

static Hand handoff(tvg::Shape* shape, const Path2& source, float amount, bool geometry) {
    Hand h;
    Path2 deformed;
    Path2 stored;
    try {
        deformed = pucker_bloat(source, amount);
    } catch (const std::bad_alloc&) {
        h.error = "pucker alloc";
        return h;
    }
    h.anchors = count_anchors(source);
    h.subpaths = count_subpaths(deformed);
    if (!write_path(shape, deformed)) {
        h.error = "appendPath";
        return h;
    }
    if (!read_path(shape, stored)) {
        h.error = "path read";
        return h;
    }
    h.delta = path_delta(stored, deformed);
    h.roundtrip = h.delta >= 0.f && h.delta < 1e-4f;
    h.radius = anchor_mean_radius(stored);
    try {
        h.mesh = tube_from(stored);
        h.mesh_ok = true;
    } catch (const std::bad_alloc&) {
        h.error = "mesh alloc";
        return h;
    }
    h.bridges = bridge_count(h.mesh);
    if (geometry && h.mesh.spans * SAMPLES <= 4000) {
        std::vector<tvg::Point> line;
        centerline(stored, line);
        h.self_x = self_intersects(line);
        h.clearance = min_clearance(line);
    }
    return h;
}

static Times time_one(tvg::Shape* shape, Gpu& gpu, Frame& frame, const Path2& source, float amount, bool geometry) {
    Times t;
    auto t0 = std::chrono::steady_clock::now();
    Hand h = handoff(shape, source, amount, geometry);
    t.path = ms_since(t0) - (h.mesh_ok ? h.mesh.sample_ms + h.mesh.mesh_ms : 0);
    if (t.path < 0) t.path = 0;
    if (!h.mesh_ok) return t;
    t.sample = h.mesh.sample_ms;
    t.mesh = h.mesh.mesh_ms;
    t.uploaded = upload_verts(gpu, h.mesh, t.upload);
    if (t.uploaded) t.drew = draw_frame(frame, nullptr, &t.draw);
    return t;
}

struct Agg {
    std::vector<double> path, sample, mesh, upload, draw;
    int frames = 0;
    int roundtrip = 0;
    float delta_max = 0;
    double spine_max = 0;
    int bridges = 0;
    bool overflow = false;
    int spans = 0;
    int verts = 0;
    int subpaths = 0;
    std::string error;
};

static void accumulate(Agg& a, const Hand& h, const Times& t) {
    ++a.frames;
    a.path.push_back(t.path);
    if (!h.mesh_ok) {
        if (a.error.empty()) a.error = h.error.empty() ? "mesh" : h.error;
        return;
    }
    a.sample.push_back(t.sample);
    a.mesh.push_back(t.mesh);
    a.spans = h.mesh.spans;
    a.verts = (int)(h.mesh.v.size() / STRIDE);
    a.subpaths = h.mesh.subpaths;
    a.overflow = h.mesh.index_overflow;
    a.bridges = h.bridges;
    a.spine_max = std::max(a.spine_max, h.mesh.center_err);
    if (h.roundtrip) ++a.roundtrip;
    a.delta_max = std::max(a.delta_max, h.delta);
    if (t.uploaded) a.upload.push_back(t.upload);
    if (t.drew) a.draw.push_back(t.draw);
    if (!h.error.empty() && a.error.empty()) a.error = h.error;
}

static void report_agg(FILE* log, const char* name, const Agg& a) {
    std::fprintf(log,
        "%s  frames %d  roundtrip %d/%d  delta-max %.3g  subpaths %d  spans %d  verts %d\n"
        "  bridges %d  index %s  spine-max %.3g%s%s\n"
        "  path %.2f  sample %.2f  mesh %.2f  upload %.2f  draw %.2f   ms median\n",
        name, a.frames, a.roundtrip, a.frames, a.delta_max, a.subpaths, a.spans, a.verts,
        a.bridges, a.overflow ? "uint16 OVERFLOW" : "uint16 ok", a.spine_max,
        a.error.empty() ? "" : "  error ", a.error.c_str(),
        median_of(a.path), median_of(a.sample), median_of(a.mesh), median_of(a.upload), median_of(a.draw));
    std::printf("%s  rt %d/%d  spans %d  verts %d  %s  path %.2f sample %.2f mesh %.2f upload %.2f draw %.2f\n",
        name, a.roundtrip, a.frames, a.spans, a.verts, a.overflow ? "OVERFLOW" : "index-ok",
        median_of(a.path), median_of(a.sample), median_of(a.mesh), median_of(a.upload), median_of(a.draw));
    std::fflush(stdout);
}

