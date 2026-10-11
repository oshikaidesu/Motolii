// Separate scene from the collage. One ThorVG rectangle. Each frame rewrites that
// path, reads it back, and rebuilds one tube. The same vertex buffer is then drawn
// with glass, chrome, and flat color. A community WGSL image, if present, is a
// fourth material on that same buffer.
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
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>

using namespace filament;
using namespace filament::math;

static const int W = 1920;
static const int H = 1080;
static const int SAMPLES = 24;
static const int SIDES = 20;
static const float TUBE_R = 0.07f;
static const int STRIDE = 9;

struct Path2 {
    std::vector<tvg::PathCommand> cmds;
    std::vector<tvg::Point> pts;
};

struct Mesh {
    std::vector<float> v;
    std::vector<uint16_t> idx;
    std::vector<float3> centers;
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

static Path2 rectangle() {
    Path2 p;
    p.cmds = {tvg::PathCommand::MoveTo, tvg::PathCommand::LineTo, tvg::PathCommand::LineTo,
              tvg::PathCommand::LineTo, tvg::PathCommand::Close};
    p.pts = {{-0.85f, -0.40f}, {0.85f, -0.40f}, {0.85f, 0.40f}, {-0.85f, 0.40f}};
    return p;
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

static bool same_path(const Path2& a, const Path2& b) {
    return a.cmds == b.cmds && a.pts.size() == b.pts.size() &&
           std::memcmp(a.pts.data(), b.pts.data(), a.pts.size() * sizeof(tvg::Point)) == 0;
}

static float anchor_mean_radius(const Path2& path) {
    double sum = 0;
    int n = 0;
    const tvg::Point* p = path.pts.data();
    for (tvg::PathCommand cmd : path.cmds) {
        tvg::Point v{};
        switch (cmd) {
        case tvg::PathCommand::MoveTo: v = *p++; break;
        case tvg::PathCommand::LineTo: v = *p++; break;
        case tvg::PathCommand::CubicTo: p += 2; v = *p++; break;
        case tvg::PathCommand::Close: continue;
        }
        sum += std::sqrt((double)v.x * v.x + (double)v.y * v.y);
        ++n;
    }
    return n ? (float)(sum / n) : 0;
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

static Mesh tube_from(const Path2& path) {
    struct Span { tvg::Point a, b, c, d; };
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
    Mesh m;
    const int rings = (int)spans.size() * SAMPLES;
    m.centers.resize(rings);
    for (int s = 0; s < (int)spans.size(); ++s) {
        for (int i = 0; i < SAMPLES; ++i) {
            float t = (float)i / (float)SAMPLES;
            tvg::Point q = bezier(spans[s].a, spans[s].b, spans[s].c, spans[s].d, t);
            tvg::Point dq = bezier_d(spans[s].a, spans[s].b, spans[s].c, spans[s].d, t);
            float len = std::sqrt(dq.x * dq.x + dq.y * dq.y);
            if (len < 1e-6f) {
                dq = {spans[s].d.x - spans[s].a.x, spans[s].d.y - spans[s].a.y};
                len = std::sqrt(dq.x * dq.x + dq.y * dq.y);
            }
            if (len < 1e-6f) { dq = {1, 0}; len = 1; }
            int ring = s * SAMPLES + i;
            float3 T{dq.x / len, dq.y / len, 0};
            float3 side = normalize(cross(T, float3{0, 0, 1}));
            float3 lift = normalize(cross(side, T));
            m.centers[ring] = {q.x, q.y, 0};
            for (int k = 0; k < SIDES; ++k) {
                float ang = (float)k / SIDES * 6.2831853f;
                float3 radial = std::cos(ang) * lift + std::sin(ang) * side;
                float3 pos = m.centers[ring] + TUBE_R * radial;
                quatf qq = mat3f::packTangentFrame(mat3f{T, normalize(cross(T, radial)), radial});
                float u = (float)ring / (float)rings;
                float v = (float)k / SIDES;
                float pack[STRIDE] = {pos.x, pos.y, pos.z, qq.x, qq.y, qq.z, qq.w, u, v};
                m.v.insert(m.v.end(), pack, pack + STRIDE);
            }
        }
    }
    for (int i = 0; i < rings; ++i) {
        int i1 = (i + 1) % rings;
        for (int k = 0; k < SIDES; ++k) {
            uint16_t a = (uint16_t)(i * SIDES + k);
            uint16_t b = (uint16_t)(i * SIDES + (k + 1) % SIDES);
            uint16_t c = (uint16_t)(i1 * SIDES + k);
            uint16_t d = (uint16_t)(i1 * SIDES + (k + 1) % SIDES);
            m.idx.insert(m.idx.end(), {a, b, c, b, d, c});
        }
    }
    return m;
}

static double spine_error(const Mesh& mesh) {
    double e = 0;
    const int rings = (int)mesh.centers.size();
    for (int i = 0; i < rings; ++i) {
        const float* p = mesh.v.data() + ((size_t)i * SIDES) * STRIDE;
        float3 radial{p[0] - mesh.centers[i].x, p[1] - mesh.centers[i].y, p[2] - mesh.centers[i].z};
        e = std::max(e, (double)std::fabs(length(radial) - TUBE_R));
    }
    return e;
}

static uint64_t position_hash(const Mesh& mesh) {
    uint64_t h = 14695981039346656037ull;
    const int n = (int)(mesh.v.size() / STRIDE);
    for (int i = 0; i < n; ++i) {
        const uint8_t* b = reinterpret_cast<const uint8_t*>(mesh.v.data() + (size_t)i * STRIDE);
        for (int k = 0; k < 12; ++k) { h ^= b[k]; h *= 1099511628211ull; }
    }
    return h;
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

static int paeth(int a, int b, int c) {
    int p = a + b - c, pa = std::abs(p - a), pb = std::abs(p - b), pc = std::abs(p - c);
    if (pa <= pb && pa <= pc) return a;
    if (pb <= pc) return b;
    return c;
}

static bool read_png(const char* path, int& w, int& h, std::vector<uint8_t>& rgba) {
    std::ifstream in(path, std::ios::binary);
    std::vector<uint8_t> file((std::istreambuf_iterator<char>(in)), {});
    if (file.size() < 8 || std::memcmp(file.data(), "\x89PNG\r\n\x1a\n", 8) != 0) return false;
    size_t i = 8;
    std::vector<uint8_t> idat;
    w = h = 0;
    int ctype = 0;
    while (i + 8 <= file.size()) {
        uint32_t n = __builtin_bswap32(*reinterpret_cast<uint32_t*>(file.data() + i));
        const char* tag = (const char*)file.data() + i + 4;
        i += 8;
        if (i + n + 4 > file.size()) return false;
        if (std::memcmp(tag, "IHDR", 4) == 0 && n >= 13) {
            w = (int)__builtin_bswap32(*reinterpret_cast<uint32_t*>(file.data() + i));
            h = (int)__builtin_bswap32(*reinterpret_cast<uint32_t*>(file.data() + i + 4));
            ctype = file[i + 9];
        } else if (std::memcmp(tag, "IDAT", 4) == 0) {
            idat.insert(idat.end(), file.begin() + i, file.begin() + i + n);
        } else if (std::memcmp(tag, "IEND", 4) == 0) break;
        i += n + 4;
    }
    if (w <= 0 || h <= 0 || ctype != 6 || idat.empty()) return false;
    uLongf dest = (uLongf)h * (w * 4 + 1);
    std::vector<uint8_t> raw(dest);
    if (uncompress(raw.data(), &dest, idat.data(), (uLong)idat.size()) != Z_OK) return false;
    rgba.assign((size_t)w * h * 4, 0);
    int stride = w * 4;
    std::vector<uint8_t> prev(stride, 0), cur(stride, 0);
    size_t at = 0;
    for (int y = 0; y < h; ++y) {
        int filter = raw[at++];
        std::memcpy(cur.data(), raw.data() + at, stride);
        at += stride;
        for (int x = 0; x < stride; ++x) {
            int left = x >= 4 ? cur[x - 4] : 0;
            int up = prev[x];
            int ul = x >= 4 ? prev[x - 4] : 0;
            int v = cur[x];
            if (filter == 1) v += left;
            else if (filter == 2) v += up;
            else if (filter == 3) v += (left + up) / 2;
            else if (filter == 4) v += paeth(left, up, ul);
            cur[x] = (uint8_t)v;
        }
        std::memcpy(rgba.data() + (size_t)y * stride, cur.data(), stride);
        prev.swap(cur);
    }
    return true;
}

static std::vector<uint8_t> load_file(const char* path) {
    std::ifstream in(path, std::ios::binary);
    return std::vector<uint8_t>((std::istreambuf_iterator<char>(in)), {});
}

static bool raster_2d(const Path2& path, std::vector<uint8_t>& rgba) {
    const int w = 480, h = 270;
    std::vector<uint32_t> buf((size_t)w * h, 0);
    auto* canvas = tvg::SwCanvas::gen();
    if (!canvas) {
        static bool told = false;
        if (!told) std::fprintf(stderr, "note: this ThorVG has no software rasterizer, skipping the 2D row\n");
        told = true;
        rgba.clear();
        return true;
    }
    canvas->target(buf.data(), w, w, h, tvg::ColorSpace::ABGR8888S);
    auto* bg = tvg::Shape::gen();
    bg->appendRect(0, 0, (float)w, (float)h);
    bg->fill(18, 20, 24, 255);
    Path2 screen = path;
    for (tvg::Point& p : screen.pts) {
        p.x = p.x * 150.f + 240.f;
        p.y = -p.y * 150.f + 135.f;
    }
    auto* stroke = tvg::Shape::gen();
    if (!write_path(stroke, screen)) return false;
    stroke->strokeWidth(3.f);
    stroke->strokeJoin(tvg::StrokeJoin::Round);
    stroke->strokeFill(236, 224, 196, 255);
    canvas->add(bg);
    canvas->add(stroke);
    canvas->draw(true);
    canvas->sync();
    rgba.resize((size_t)w * h * 4);
    for (int i = 0; i < w * h; ++i) {
        uint32_t p = buf[i];
        rgba[(size_t)i * 4 + 0] = (uint8_t)(p & 255);
        rgba[(size_t)i * 4 + 1] = (uint8_t)((p >> 8) & 255);
        rgba[(size_t)i * 4 + 2] = (uint8_t)((p >> 16) & 255);
        rgba[(size_t)i * 4 + 3] = 255;
    }
    return true;
}

static bool build_mesh(tvg::Shape* shape, float amount, const Path2& source, Mesh& mesh, float& radius) {
    Path2 deformed = pucker_bloat(source, amount);
    if (!write_path(shape, deformed)) return false;
    Path2 stored;
    if (!read_path(shape, stored) || !same_path(stored, deformed)) return false;
    mesh = tube_from(stored);
    radius = anchor_mean_radius(stored);
    return spine_error(mesh) < 1e-4 && !mesh.idx.empty();
}

int main() {
    if (tvg::Initializer::init(0) != tvg::Result::Success) return fail("thorvg");
    tvg::Shape* shape = tvg::Shape::gen();
    const Path2 source = rectangle();

    auto glass_pkg = load_file("glass.filamat");
    auto chrome_pkg = load_file("chrome.filamat");
    auto solid_pkg = load_file("solid.filamat");
    auto stripe_pkg = load_file("stripes.filamat");
    if (glass_pkg.empty() || chrome_pkg.empty() || solid_pkg.empty() || stripe_pkg.empty()) return fail("filamat");

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

    Material* glass_mat = Material::Builder().package(glass_pkg.data(), glass_pkg.size()).build(*engine);
    Material* chrome_mat = Material::Builder().package(chrome_pkg.data(), chrome_pkg.size()).build(*engine);
    Material* solid_mat = Material::Builder().package(solid_pkg.data(), solid_pkg.size()).build(*engine);
    Material* stripe_mat = Material::Builder().package(stripe_pkg.data(), stripe_pkg.size()).build(*engine);
    MaterialInstance* glass = glass_mat->createInstance();
    glass->setParameter("ior", 2.15f);
    glass->setParameter("transmission", 1.f);
    glass->setParameter("thickness", 0.62f);
    glass->setParameter("dispersion", 0.72f);
    MaterialInstance* chrome = chrome_mat->createInstance();
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

    Mesh mesh;
    float radius = 0;
    if (!build_mesh(shape, 0, source, mesh, radius)) return fail("mesh");
    const int vcount = (int)(mesh.v.size() / STRIDE);
    VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)vcount).bufferCount(1)
                           .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, STRIDE * 4)
                           .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, STRIDE * 4)
                           .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 28, STRIDE * 4)
                           .build(*engine);
    vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * 4));
    IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)mesh.idx.size()).bufferType(IndexBuffer::IndexType::USHORT).build(*engine);
    ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(mesh.idx.data(), mesh.idx.size() * 2));
    utils::Entity tube = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-2, -2, -2}, {2, 2, 2}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
        .material(0, glass).culling(false).castShadows(false).receiveShadows(false).build(*engine, tube);
    scene->addEntity(tube);

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

    auto draw = [&](std::vector<uint8_t>* picture) -> bool {
        bool drew = false;
        for (int attempt = 0; attempt < 4 && !drew; ++attempt) {
            if (!renderer->beginFrame(swap)) continue;
            renderer->render(view);
            if (picture) {
                picture->resize((size_t)W * H * 4);
                renderer->readPixels(rt, 0, 0, W, H, backend::PixelBufferDescriptor(
                    picture->data(), picture->size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
            }
            renderer->endFrame();
            drew = true;
        }
        engine->flushAndWait();
        return drew;
    };

    const int FRAMES = 60;
    std::vector<double> times;
    times.reserve(FRAMES);
    float rmin = 1e9f, rmax = 0;
    double spine_max = 0;
    int followed = 0;
    for (int warm = 0; warm < 2; ++warm) {
        if (!draw(nullptr)) return fail("warmup");
    }
    for (int frame = 0; frame < FRAMES; ++frame) {
        float amount = 30.f * std::sin((float)frame / FRAMES * 6.2831853f);
        auto t0 = std::chrono::steady_clock::now();
        if (!build_mesh(shape, amount, source, mesh, radius)) return fail("live mesh");
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * 4));
        if (!draw(nullptr)) return fail("live frame");
        times.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
        rmin = std::min(rmin, radius);
        rmax = std::max(rmax, radius);
        spine_max = std::max(spine_max, spine_error(mesh));
        ++followed;
    }
    std::vector<double> sorted = times;
    std::sort(sorted.begin(), sorted.end());
    double median = sorted[sorted.size() / 2];
    int under = 0;
    for (double t : times)
        if (t <= 8.33) ++under;

    const float shots[4] = {-30.f, -10.f, 10.f, 30.f};
    std::vector<uint8_t> sheet((size_t)1920 * 540 * 4, 0);
    auto blit = [&](const std::vector<uint8_t>& src, int sw, int sh, int col, int row) {
        for (int y = 0; y < 270; ++y) {
            for (int x = 0; x < 480; ++x) {
                int sx = x * sw / 480, sy = y * sh / 270;
                const uint8_t* s = src.data() + ((size_t)sy * sw + sx) * 4;
                uint8_t* d = sheet.data() + ((size_t)(row * 270 + y) * 1920 + col * 480 + x) * 4;
                std::memcpy(d, s, 4);
            }
        }
    };
    for (int s = 0; s < 4; ++s) {
        if (!build_mesh(shape, shots[s], source, mesh, radius)) return fail("shot mesh");
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * 4));
        engine->flushAndWait();
        std::vector<uint8_t> picture;
        auto t0 = std::chrono::steady_clock::now();
        if (!draw(&picture)) return fail("shot");
        double shot_ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
        std::printf("shot amount %+.0f  anchor-radius %.4f  spine %.3g  with-readback %.2f ms\n", shots[s], radius, spine_error(mesh), shot_ms);
        blit(picture, W, H, s, 0);
        std::vector<uint8_t> d2;
        Path2 stored;
        read_path(shape, stored);
        if (!raster_2d(stored, d2)) return fail("2d");
        if (!d2.empty()) blit(d2, 480, 270, s, 1);
        if (s == 3 && !write_png("out/link-live.png", W, H, picture.data())) return fail("png");
    }
    if (!write_png("out/link-sheet.png", 1920, 540, sheet.data())) return fail("sheet");

    if (!build_mesh(shape, 0, source, mesh, radius)) return fail("rest mesh");
    vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * 4));
    engine->flushAndWait();
    uint64_t hash = position_hash(mesh);
    auto& rm = engine->getRenderableManager();
    auto inst = rm.getInstance(tube);
    struct MatShot { const char* name; const char* file; MaterialInstance* mi; };
    MatShot mats[3] = {{"glass", "out/link-glass.png", glass}, {"chrome", "out/link-chrome.png", chrome}, {"flat", "out/link-solid.png", flat}};
    for (MatShot& m : mats) {
        if (position_hash(mesh) != hash) return fail("mesh changed before material swap");
        rm.setMaterialInstanceAt(inst, 0, m.mi);
        std::vector<uint8_t> picture;
        if (!draw(&picture)) return fail(m.name);
        if (!write_png(m.file, W, H, picture.data())) return fail("png");
        std::printf("material %s  vertices %d  hash %llx\n", m.name, vcount, (unsigned long long)hash);
    }
    if (position_hash(mesh) != hash) return fail("mesh changed after material swap");

    int led_w = 0, led_h = 0;
    std::vector<uint8_t> led;
    bool wgsl = read_png("out/tri_led.png", led_w, led_h, led);
    if (wgsl) {
        auto* owned = new std::vector<uint8_t>(std::move(led));
        Texture* led_tex = Texture::Builder().width((uint32_t)led_w).height((uint32_t)led_h).levels(1)
                               .sampler(Texture::Sampler::SAMPLER_2D).format(Texture::InternalFormat::RGBA8).build(*engine);
        led_tex->setImage(*engine, 0, Texture::PixelBufferDescriptor(owned->data(), owned->size(),
            backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE, 1, 0, 0, (uint32_t)led_w,
            [](void*, size_t, void* user) { delete static_cast<std::vector<uint8_t>*>(user); }, owned));
        MaterialInstance* wg = stripe_mat->createInstance();
        wg->setParameter("albedo", led_tex, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
        rm.setMaterialInstanceAt(inst, 0, wg);
        std::vector<uint8_t> picture;
        if (!draw(&picture)) return fail("wgsl material");
        if (position_hash(mesh) != hash) return fail("mesh changed for wgsl");
        if (!write_png("out/link-wgsl.png", W, H, picture.data())) return fail("png");
        std::printf("material wgsl-tri_led  hash %llx\n", (unsigned long long)hash);
    }

    FILE* log = std::fopen("out/link.txt", "w");
    std::fprintf(log,
        "scene  one rectangle, ThorVG pucker each frame, one tube\n"
        "1 path  %d/%d frames  spine-max %.3g  anchor-radius %.4f .. %.4f\n"
        "2 material  same vertex hash %llx  glass chrome flat%s\n"
        "3 frame  1920x1080  %d frames  path+upload+glass+bloom+fxaa+refraction  no readback\n"
        "   min %.2f  median %.2f  max %.2f ms   under 8.33ms %d/%d\n"
        "   target 8.33ms is 120fps. This number is the measured median, not a claim.\n"
        "4 wgsl  %s\n",
        followed, FRAMES, spine_max, rmin, rmax,
        (unsigned long long)hash, wgsl ? " + tri_led texture" : "",
        FRAMES, sorted.front(), median, sorted.back(), under, FRAMES,
        wgsl ? "tri_led.wgsl via wgpu (vgpu MIT), same mesh, stripes.mat samples it" : "tri_led.png missing");
    std::fclose(log);
    std::printf("median %.2f ms  under 8.33 %d/%d  wgsl %s\nPASS link\n", median, under, FRAMES, wgsl ? "yes" : "no");
    return 0;
}
