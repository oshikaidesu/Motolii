// robust.mm, part 2 (cut there by size, as it was): read only through robust.mm, in order.
#pragma once


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
    ihdr[8] = 8;
    ihdr[9] = 6;
    ihdr[10] = ihdr[11] = ihdr[12] = 0;
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
static std::vector<uint8_t> hstack(const std::vector<uint8_t>& a, const std::vector<uint8_t>& b) {
    std::vector<uint8_t> out((size_t)W * 2 * H * 4);
    for (int y = 0; y < H; ++y) {
        std::memcpy(out.data() + ((size_t)y * W * 2) * 4, a.data() + (size_t)y * W * 4, (size_t)W * 4);
        std::memcpy(out.data() + ((size_t)y * W * 2 + W) * 4, b.data() + (size_t)y * W * 4, (size_t)W * 4);
    }
    return out;
}
static std::vector<uint8_t> grid2(const std::vector<uint8_t>& a, const std::vector<uint8_t>& b, const std::vector<uint8_t>& c,
                                  const std::vector<uint8_t>& d) {
    std::vector<uint8_t> out((size_t)W * 2 * H * 2 * 4);
    const std::vector<uint8_t>* cells[4] = {&a, &b, &c, &d};
    for (int i = 0; i < 4; ++i) {
        int ox = (i % 2) * W;
        int oy = (i / 2) * H;
        for (int y = 0; y < H; ++y)
            std::memcpy(out.data() + ((size_t)(oy + y) * W * 2 + ox) * 4, cells[i]->data() + (size_t)y * W * 4, (size_t)W * 4);
    }
    return out;
}
static double ms_since(std::chrono::steady_clock::time_point t0) {
    return std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
}
static int fail(const char* msg) {
    std::fprintf(stderr, "FAIL %s\n", msg);
    return 1;
}
static double mae_of(const std::vector<uint8_t>& a, const std::vector<uint8_t>& b, int& far) {
    double mae = 0;
    far = 0;
    int n = W * H;
    for (int i = 0; i < n; ++i) {
        int dsum = 0;
        for (int ch = 0; ch < 3; ++ch) {
            int d = std::abs((int)a[(size_t)i * 4 + ch] - (int)b[(size_t)i * 4 + ch]);
            mae += d;
            dsum += d;
        }
        if (dsum > 80) ++far;
    }
    return mae / (double)n / 3.0;
}

struct App {
    Engine* engine = nullptr;
    Renderer* renderer = nullptr;
    Scene* scene = nullptr;
    View* view = nullptr;
    SwapChain* swap = nullptr;
    Camera* camera = nullptr;
    MaterialInstance* mi = nullptr;
    MaterialInstance* gpu = nullptr;
    RenderTarget* rt = nullptr;
};

static bool grab(App& a, std::vector<uint8_t>& picture) {
    picture.resize((size_t)W * H * 4);
    bool drew = false;
    for (int attempt = 0; attempt < 4 && !drew; ++attempt) {
        if (!a.renderer->beginFrame(a.swap)) continue;
        a.renderer->render(a.view);
        a.renderer->readPixels(a.rt, 0, 0, W, H,
                               backend::PixelBufferDescriptor(picture.data(), picture.size(), backend::PixelDataFormat::RGBA,
                                                              backend::PixelDataType::UBYTE));
        a.renderer->endFrame();
        drew = true;
    }
    a.engine->flushAndWait();
    return drew;
}
static utils::Entity upload(App& a, const Mesh& mesh, MaterialInstance* mi) {
    const uint32_t n = (uint32_t)(mesh.v.size() / 7);
    auto* vb = VertexBuffer::Builder()
                   .vertexCount(n)
                   .bufferCount(1)
                   .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                   .attribute(VertexAttribute::COLOR, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                   .build(*a.engine);
    vb->setBufferAt(*a.engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * sizeof(float)));
    auto* ib = IndexBuffer::Builder().indexCount((uint32_t)mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*a.engine);
    ib->setBuffer(*a.engine, IndexBuffer::BufferDescriptor(mesh.idx.data(), mesh.idx.size() * sizeof(uint32_t)));
    utils::Entity e = utils::EntityManager::get().create();
    RenderableManager::Builder(1)
        .boundingBox({{-4, -4, -4}, {4, 4, 4}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
        .material(0, mi)
        .culling(false)
        .castShadows(false)
        .receiveShadows(false)
        .build(*a.engine, e);
    a.scene->addEntity(e);
    return e;
}
static bool shot(App& a, const Mesh& mesh, float3 eye, float3 at, float3 up, std::vector<uint8_t>& rgba) {
    a.camera->lookAt(eye, at, up);
    utils::Entity e = upload(a, mesh, a.mi);
    a.engine->flushAndWait();
    bool ok = grab(a, rgba) && grab(a, rgba);
    a.scene->remove(e);
    return ok;
}

static Mesh gpu_topology(int rings, int sides) {
    Mesh m;
    int n = rings * sides;
    m.v.assign((size_t)n * 7, 0.f);
    for (size_t i = 3; i < m.v.size(); i += 7) m.v[i + 3] = 1.f;
    for (int i = 0; i < rings - 1; ++i) {
        for (int k = 0; k < sides; ++k) {
            uint32_t a = (uint32_t)(i * sides + k);
            uint32_t b = (uint32_t)(i * sides + (k + 1) % sides);
            uint32_t c = (uint32_t)((i + 1) * sides + k);
            uint32_t d = (uint32_t)((i + 1) * sides + (k + 1) % sides);
            m.idx.insert(m.idx.end(), {a, b, c, b, d, c});
        }
    }
    return m;
}
static void pack_controls(const std::vector<Cubic>& spans, int tex_w, int tex_h, std::vector<float>& tex) {
    tex.assign((size_t)tex_w * tex_h * 4, 0.f);
    for (int s = 0; s < (int)spans.size(); ++s) {
        for (int k = 0; k < 4; ++k) {
            int index = s * 4 + k;
            tex[(size_t)index * 4 + 0] = spans[s].p[k].x;
            tex[(size_t)index * 4 + 1] = spans[s].p[k].y;
            tex[(size_t)index * 4 + 2] = spans[s].p[k].z;
            tex[(size_t)index * 4 + 3] = 1.f;
        }
    }
}
static void pack_frames(const std::vector<Sample>& samples, int tex_w, int tex_h, std::vector<float>& tex) {
    tex.assign((size_t)tex_w * tex_h * 4, 0.f);
    for (int i = 0; i < (int)samples.size(); ++i) {
        int n = i * 2;
        int b = n + 1;
        const Sample& s = samples[i];
        tex[(size_t)n * 4 + 0] = s.N.x;
        tex[(size_t)n * 4 + 1] = s.N.y;
        tex[(size_t)n * 4 + 2] = s.N.z;
        tex[(size_t)n * 4 + 3] = 0.f;
        tex[(size_t)b * 4 + 0] = s.B.x;
        tex[(size_t)b * 4 + 1] = s.B.y;
        tex[(size_t)b * 4 + 2] = s.B.z;
        tex[(size_t)b * 4 + 3] = 0.f;
    }
}
static int tex_rows(int texels, int tex_w) { return std::max(1, (texels + tex_w - 1) / tex_w); }
static Texture* float_tex(Engine* engine, int w, int h, const std::vector<float>& data) {
    Texture* tex = Texture::Builder()
                       .width((uint32_t)w)
                       .height((uint32_t)h)
                       .levels(1)
                       .sampler(Texture::Sampler::SAMPLER_2D)
                       .format(Texture::InternalFormat::RGBA32F)
                       .build(*engine);
    if (!tex) return nullptr;
    tex->setImage(*engine, 0,
                  Texture::PixelBufferDescriptor(data.data(), data.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                 backend::PixelDataType::FLOAT, 1, 0, 0, (uint32_t)w));
    return tex;
}

