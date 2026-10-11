// One rectangle, four cubics. ThorVG owns the path. The GPU tube does not
// receive expanded vertices: custom0 is (span, t, angle), and gpu_tube.mat
// reads the cubic control points from a float texture inside materialVertex.
#include <thorvg.h>

#include <filament/Camera.h>
#include <filament/ColorGrading.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
#include <filament/LightManager.h>
#include <filament/Material.h>
#include <filament/MaterialInstance.h>
#include <filament/RenderableManager.h>
#include <filament/Renderer.h>
#include <filament/RenderTarget.h>
#include <filament/Scene.h>
#include <filament/Skybox.h>
#include <filament/SwapChain.h>
#include <filament/Texture.h>
#include <filament/TextureSampler.h>
#include <filament/ToneMapper.h>
#include <filament/VertexBuffer.h>
#include <filament/View.h>
#include <filament/Viewport.h>

#include <math/mat3.h>
#include <math/norm.h>
#include <math/vec3.h>
#include <math/vec4.h>

#include <utils/EntityManager.h>

#include <zlib.h>

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 1280;
static const int H = 800;
static const int SPANS = 4;
static const int SAMPLES = 24;
static const int SIDES = 20;
static const float TUBE_R = 0.052f;
static const int GPU_STRIDE = 11;

struct Path2 {
    std::vector<tvg::PathCommand> cmds;
    std::vector<tvg::Point> pts;
};

struct Span {
    tvg::Point a, b, c, d;
};

static int fail(const char* msg) {
    std::fprintf(stderr, "FAIL %s\n", msg);
    return 1;
}

static bool near_pt(tvg::Point a, tvg::Point b) {
    float dx = a.x - b.x, dy = a.y - b.y;
    return dx * dx + dy * dy < 1e-12f;
}
static tvg::Point toward(tvg::Point p, tvg::Point c, float a) {
    return {p.x + (c.x - p.x) * a, p.y + (c.y - p.y) * a};
}
static tvg::Point away(tvg::Point p, tvg::Point c, float a) {
    return {p.x - (c.x - p.x) * a, p.y - (c.y - p.y) * a};
}

static tvg::Point contour_center(const Path2& in) {
    tvg::Point c{0, 0};
    int count = 0;
    const tvg::Point* p = in.pts.data();
    const tvg::Point* start = p;
    for (tvg::PathCommand cmd : in.cmds) {
        switch (cmd) {
        case tvg::PathCommand::MoveTo: start = p; ++p; break;
        case tvg::PathCommand::CubicTo:
            c.x += p[-1].x + p[0].x + p[1].x + p[2].x;
            c.y += p[-1].y + p[0].y + p[1].y + p[2].y;
            p += 3; count += 4; break;
        case tvg::PathCommand::LineTo:
            c.x += p[-1].x + p[0].x; c.y += p[-1].y + p[0].y;
            ++p; count += 2; break;
        case tvg::PathCommand::Close:
            if (!near_pt(p[-1], *start)) {
                c.x += p[-1].x + start->x; c.y += p[-1].y + start->y; count += 2;
            }
            break;
        }
    }
    if (count > 0) { c.x /= (float)count; c.y /= (float)count; }
    return c;
}

static Path2 pucker_bloat(const Path2& in, float amount) {
    const float a = amount * 0.01f;
    const tvg::Point c = contour_center(in);
    Path2 out;
    const tvg::Point* pts = in.pts.data();
    const tvg::Point* start = pts;
    for (tvg::PathCommand cmd : in.cmds) {
        switch (cmd) {
        case tvg::PathCommand::MoveTo:
            start = pts;
            out.pts.push_back(toward(*pts, c, a));
            out.cmds.push_back(tvg::PathCommand::MoveTo);
            ++pts; break;
        case tvg::PathCommand::CubicTo:
            out.pts.push_back(away(pts[0], c, a));
            out.pts.push_back(away(pts[1], c, a));
            out.pts.push_back(toward(pts[2], c, a));
            out.cmds.push_back(tvg::PathCommand::CubicTo);
            pts += 3; break;
        case tvg::PathCommand::LineTo:
            out.pts.push_back(away(pts[-1], c, a));
            out.pts.push_back(away(pts[0], c, a));
            out.pts.push_back(toward(pts[0], c, a));
            out.cmds.push_back(tvg::PathCommand::CubicTo);
            ++pts; break;
        case tvg::PathCommand::Close:
            if (!near_pt(pts[-1], *start)) {
                out.pts.push_back(away(pts[-1], c, a));
                out.pts.push_back(away(*start, c, a));
                out.pts.push_back(toward(*start, c, a));
                out.cmds.push_back(tvg::PathCommand::CubicTo);
            }
            out.cmds.push_back(tvg::PathCommand::Close);
            break;
        }
    }
    return out;
}

static bool write_path(tvg::Shape* shape, const Path2& path) {
    if (shape->reset() != tvg::Result::Success) return false;
    return shape->appendPath(path.cmds.data(), (uint32_t)path.cmds.size(), path.pts.data(),
               (uint32_t)path.pts.size()) == tvg::Result::Success;
}

static bool read_path(const tvg::Shape* shape, Path2& out) {
    const tvg::PathCommand* cmds = nullptr;
    const tvg::Point* pts = nullptr;
    uint32_t nc = 0, np = 0;
    if (shape->path(&cmds, &nc, &pts, &np) != tvg::Result::Success) return false;
    out.cmds.assign(cmds, cmds + nc);
    out.pts.assign(pts, pts + np);
    return true;
}

static Path2 rectangle() {
    Path2 p;
    p.cmds = {tvg::PathCommand::MoveTo, tvg::PathCommand::LineTo, tvg::PathCommand::LineTo,
              tvg::PathCommand::LineTo, tvg::PathCommand::Close};
    p.pts = {{-0.85f, -0.40f}, {0.85f, -0.40f}, {0.85f, 0.40f}, {-0.85f, 0.40f}};
    return p;
}

static std::vector<Span> spans_of(const Path2& path) {
    std::vector<Span> spans;
    const tvg::Point* p = path.pts.data();
    tvg::Point start{}, cur{};
    bool has = false;
    for (tvg::PathCommand cmd : path.cmds) {
        switch (cmd) {
        case tvg::PathCommand::MoveTo: start = cur = *p++; has = true; break;
        case tvg::PathCommand::LineTo: {
            tvg::Point n = *p++;
            spans.push_back({cur, cur, n, n});
            cur = n; break;
        }
        case tvg::PathCommand::CubicTo: {
            tvg::Point c0 = *p++, c1 = *p++, n = *p++;
            spans.push_back({cur, c0, c1, n});
            cur = n; break;
        }
        case tvg::PathCommand::Close:
            if (has && !near_pt(cur, start)) spans.push_back({cur, cur, start, start});
            cur = start; break;
        }
    }
    return spans;
}

static tvg::Point bezier(tvg::Point p0, tvg::Point p1, tvg::Point p2, tvg::Point p3, float t) {
    float u = 1.f - t;
    float b0 = u * u * u, b1 = 3 * u * u * t, b2 = 3 * u * t * t, b3 = t * t * t;
    return {b0 * p0.x + b1 * p1.x + b2 * p2.x + b3 * p3.x, b0 * p0.y + b1 * p1.y + b2 * p2.y + b3 * p3.y};
}
static tvg::Point bezier_d(tvg::Point p0, tvg::Point p1, tvg::Point p2, tvg::Point p3, float t) {
    float u = 1.f - t;
    tvg::Point a{p1.x - p0.x, p1.y - p0.y};
    tvg::Point b{p2.x - p1.x, p2.y - p1.y};
    tvg::Point c{p3.x - p2.x, p3.y - p2.y};
    float k0 = 3 * u * u, k1 = 6 * u * t, k2 = 3 * t * t;
    return {k0 * a.x + k1 * b.x + k2 * c.x, k0 * a.y + k1 * b.y + k2 * c.y};
}

static void cpu_tube(const std::vector<Span>& spans, std::vector<float>& v, std::vector<uint16_t>& idx) {
    const int rings = (int)spans.size() * SAMPLES;
    v.resize((size_t)rings * SIDES * 9);
    for (int s = 0; s < (int)spans.size(); ++s) {
        for (int i = 0; i < SAMPLES; ++i) {
            float t = (float)i / (float)SAMPLES;
            tvg::Point q = bezier(spans[s].a, spans[s].b, spans[s].c, spans[s].d, t);
            tvg::Point dq = bezier_d(spans[s].a, spans[s].b, spans[s].c, spans[s].d, t);
            float len = std::sqrt(dq.x * dq.x + dq.y * dq.y);
            if (len < 1e-6f) { dq = {1, 0}; len = 1; }
            int ring = s * SAMPLES + i;
            float3 T{dq.x / len, dq.y / len, 0};
            float3 side = normalize(cross(T, float3{0, 0, 1}));
            float3 lift = normalize(cross(side, T));
            float3 center{q.x, q.y, 0};
            for (int k = 0; k < SIDES; ++k) {
                float ang = (float)k / SIDES * 6.2831853f;
                float3 radial = std::cos(ang) * lift + std::sin(ang) * side;
                float3 pos = center + TUBE_R * radial;
                quatf qq = mat3f::packTangentFrame(mat3f{T, normalize(cross(T, radial)), radial});
                float* dst = v.data() + ((size_t)ring * SIDES + k) * 9;
                float pack[9] = {pos.x, pos.y, pos.z, qq.x, qq.y, qq.z, qq.w, (float)ring / rings, (float)k / SIDES};
                std::memcpy(dst, pack, sizeof(pack));
            }
        }
    }
    idx.resize((size_t)rings * SIDES * 6);
    size_t w = 0;
    for (int i = 0; i < rings; ++i) {
        int i1 = (i + 1) % rings;
        for (int k = 0; k < SIDES; ++k) {
            uint16_t a = (uint16_t)(i * SIDES + k);
            uint16_t b = (uint16_t)(i * SIDES + (k + 1) % SIDES);
            uint16_t c = (uint16_t)(i1 * SIDES + k);
            uint16_t d = (uint16_t)(i1 * SIDES + (k + 1) % SIDES);
            idx[w++] = a; idx[w++] = b; idx[w++] = c;
            idx[w++] = b; idx[w++] = d; idx[w++] = c;
        }
    }
}

static void gpu_topology(const std::vector<Span>& spans, std::vector<float>& pos, std::vector<float>& uv, std::vector<uint16_t>& idx) {
    const int rings = SPANS * SAMPLES;
    pos.resize((size_t)rings * SIDES * 9);
    uv.clear();
    for (int s = 0; s < SPANS; ++s) {
        for (int i = 0; i < SAMPLES; ++i) {
            float t = (float)i / (float)SAMPLES;
            for (int k = 0; k < SIDES; ++k) {
                float ang = (float)k / SIDES * 6.2831853f;
                size_t vert = ((size_t)s * SAMPLES + i) * SIDES + k;
                float pack[9] = {0, ang, 0, 0, 0, 0, 1, (float)s + t, 0};
                std::memcpy(pos.data() + vert * 9, pack, sizeof(pack));
            }
        }
    }
    idx.resize((size_t)rings * SIDES * 6);
    size_t w = 0;
    for (int i = 0; i < rings; ++i) {
        int i1 = (i + 1) % rings;
        for (int k = 0; k < SIDES; ++k) {
            uint16_t a = (uint16_t)(i * SIDES + k);
            uint16_t b = (uint16_t)(i * SIDES + (k + 1) % SIDES);
            uint16_t c = (uint16_t)(i1 * SIDES + k);
            uint16_t d = (uint16_t)(i1 * SIDES + (k + 1) % SIDES);
            idx[w++] = a; idx[w++] = b; idx[w++] = c;
            idx[w++] = b; idx[w++] = d; idx[w++] = c;
        }
    }
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
