// One ThorVG Shape is the path. Pucker & Bloat rewrites that Shape, then both the
// 2D stroke and the Filament tube are built from Shape::path() after the write.
// Positive amount pulls anchors toward the centre (Lottie / ThorVG). AE's slider
// is the opposite sign; the pictures are named by the contour, not by AE's number.
#include <thorvg.h>

#include <filament/Box.h>
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
#include <math/vec2.h>
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
static const int W2 = 1280;
static const int H2 = 420;
static const float TUBE_R = 0.052f;
static const int SAMPLES = 48;
static const int SIDES = 36;

static const float GLASS_IOR = 2.15f;
static const float GLASS_TRANSMISSION = 1.0f;
static const float GLASS_THICKNESS = 0.62f;
static const float GLASS_DISPERSION = 0.72f;

struct Path2 {
    std::vector<tvg::PathCommand> cmds;
    std::vector<tvg::Point> pts;
};

struct Sample {
    float3 p;
    float3 t;
};

struct Mesh {
    std::vector<float> verts; // px,py,pz, qx,qy,qz,qw
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

// tvgLottieModifier.cpp LottiePuckerBloatModifier::center / path, v1.1.2.
static tvg::Point contour_center(const Path2& in) {
    tvg::Point c{0, 0};
    int count = 0;
    const tvg::Point* p = in.pts.data();
    const tvg::Point* start = p;
    for (tvg::PathCommand cmd : in.cmds) {
        switch (cmd) {
        case tvg::PathCommand::MoveTo:
            start = p;
            ++p;
            break;
        case tvg::PathCommand::CubicTo:
            c.x += (p[-1].x + p[0].x + p[1].x + p[2].x);
            c.y += (p[-1].y + p[0].y + p[1].y + p[2].y);
            p += 3;
            count += 4;
            break;
        case tvg::PathCommand::LineTo:
            c.x += p[-1].x + p[0].x;
            c.y += p[-1].y + p[0].y;
            ++p;
            count += 2;
            break;
        case tvg::PathCommand::Close:
            if (!near_pt(p[-1], *start)) {
                c.x += p[-1].x + start->x;
                c.y += p[-1].y + start->y;
                count += 2;
            }
            break;
        }
    }
    if (count > 0) {
        c.x /= (float)count;
        c.y /= (float)count;
    }
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
            ++pts;
            break;
        case tvg::PathCommand::CubicTo:
            out.pts.push_back(away(pts[0], c, a));
            out.pts.push_back(away(pts[1], c, a));
            out.pts.push_back(toward(pts[2], c, a));
            out.cmds.push_back(tvg::PathCommand::CubicTo);
            pts += 3;
            break;
        case tvg::PathCommand::LineTo:
            out.pts.push_back(away(pts[-1], c, a));
            out.pts.push_back(away(pts[0], c, a));
            out.pts.push_back(toward(pts[0], c, a));
            out.cmds.push_back(tvg::PathCommand::CubicTo);
            ++pts;
            break;
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
    p.cmds = {
        tvg::PathCommand::MoveTo, tvg::PathCommand::LineTo, tvg::PathCommand::LineTo,
        tvg::PathCommand::LineTo, tvg::PathCommand::Close,
    };
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
        case tvg::PathCommand::MoveTo:
            v = *p++;
            break;
        case tvg::PathCommand::LineTo:
            v = *p++;
            break;
        case tvg::PathCommand::CubicTo:
            p += 2;
            v = *p++;
            break;
        case tvg::PathCommand::Close:
            continue;
        }
        sum += std::sqrt((double)v.x * v.x + (double)v.y * v.y);
        ++n;
    }
    return n ? (float)(sum / n) : 0;
}

static tvg::Point bezier(tvg::Point p0, tvg::Point p1, tvg::Point p2, tvg::Point p3, float t) {
    float u = 1.f - t;
    float b0 = u * u * u, b1 = 3 * u * u * t, b2 = 3 * u * t * t, b3 = t * t * t;
    return {b0 * p0.x + b1 * p1.x + b2 * p2.x + b3 * p3.x,
            b0 * p0.y + b1 * p1.y + b2 * p2.y + b3 * p3.y};
}

static tvg::Point bezier_d(tvg::Point p0, tvg::Point p1, tvg::Point p2, tvg::Point p3, float t) {
    float u = 1.f - t;
    tvg::Point a{p1.x - p0.x, p1.y - p0.y};
    tvg::Point b{p2.x - p1.x, p2.y - p1.y};
    tvg::Point c{p3.x - p2.x, p3.y - p2.y};
    float k0 = 3 * u * u, k1 = 6 * u * t, k2 = 3 * t * t;
    return {k0 * a.x + k1 * b.x + k2 * c.x, k0 * a.y + k1 * b.y + k2 * c.y};
}

static std::vector<Sample> sample_path(const Path2& path) {
    std::vector<Sample> out;
    const tvg::Point* p = path.pts.data();
    tvg::Point start{}, cur{};
    bool has = false;
    auto push_span = [&](tvg::Point a, tvg::Point b, tvg::Point c, tvg::Point d) {
        for (int i = 0; i < SAMPLES; ++i) {
            float t = (float)i / (float)SAMPLES;
            tvg::Point q = bezier(a, b, c, d, t);
            tvg::Point dq = bezier_d(a, b, c, d, t);
            float len = std::sqrt(dq.x * dq.x + dq.y * dq.y);
            if (len < 1e-6f) {
                dq = {d.x - a.x, d.y - a.y};
                len = std::sqrt(dq.x * dq.x + dq.y * dq.y);
            }
            if (len < 1e-6f) dq = {1, 0}, len = 1;
            out.push_back({float3{q.x, q.y, 0}, float3{dq.x / len, dq.y / len, 0}});
        }
    };
    for (tvg::PathCommand cmd : path.cmds) {
        switch (cmd) {
        case tvg::PathCommand::MoveTo:
            start = cur = *p++;
            has = true;
            break;
        case tvg::PathCommand::LineTo: {
            tvg::Point n = *p++;
            push_span(cur, cur, n, n);
            cur = n;
            break;
        }
        case tvg::PathCommand::CubicTo: {
            tvg::Point c0 = *p++, c1 = *p++, n = *p++;
            push_span(cur, c0, c1, n);
            cur = n;
            break;
        }
        case tvg::PathCommand::Close:
            if (has && !near_pt(cur, start)) push_span(cur, cur, start, start);
            cur = start;
            break;
        }
    }
    return out;
}

static Mesh tube_from(const std::vector<Sample>& samples) {
    Mesh m;
    const int rings = (int)samples.size();
    m.verts.reserve((size_t)rings * SIDES * 7);
    m.centers.resize(rings);
    for (int i = 0; i < rings; ++i) {
        float3 T = samples[i].t;
        float3 side = normalize(cross(T, float3{0, 0, 1}));
        float3 lift = normalize(cross(side, T));
        m.centers[i] = samples[i].p;
        for (int k = 0; k < SIDES; ++k) {
            float ang = (float)k / (float)SIDES * 2.f * (float)M_PI;
            float3 radial = std::cos(ang) * lift + std::sin(ang) * side;
            float3 pos = samples[i].p + TUBE_R * radial;
            float3 B = normalize(cross(T, radial));
            quatf q = mat3f::packTangentFrame(mat3f{T, B, radial});
            float pack[7] = {pos.x, pos.y, pos.z, q.x, q.y, q.z, q.w};
            m.verts.insert(m.verts.end(), pack, pack + 7);
        }
    }
    for (int i = 0; i < rings; ++i) {
        int i1 = (i + 1) % rings;
        for (int k = 0; k < SIDES; ++k) {
            int k1 = (k + 1) % SIDES;
            uint16_t a = (uint16_t)(i * SIDES + k);
            uint16_t b = (uint16_t)(i * SIDES + k1);
            uint16_t c = (uint16_t)(i1 * SIDES + k);
            uint16_t d = (uint16_t)(i1 * SIDES + k1);
            m.idx.insert(m.idx.end(), {a, b, c, b, d, c});
        }
    }
    return m;
}

static std::vector<uint8_t> load_file(const char* path) {
    std::ifstream in(path, std::ios::binary);
    return std::vector<uint8_t>((std::istreambuf_iterator<char>(in)), {});
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
    ihdr[8] = 8;
    ihdr[9] = 6;
    ihdr[10] = 0;
    ihdr[11] = 0;
    ihdr[12] = 0;
    chunk("IHDR", ihdr, 13);
    chunk("IDAT", z.data(), (uint32_t)bound);
    chunk("IEND", nullptr, 0);
    std::fclose(f);
    return true;
}

static void blit(uint8_t* dst, int dw, int dx, int dy, const uint8_t* src, int sw, int sh) {
    for (int y = 0; y < sh; ++y) {
        std::memcpy(dst + ((size_t)(dy + y) * dw + dx) * 4, src + (size_t)y * sw * 4, (size_t)sw * 4);
    }
}

static double mean_abs(const std::vector<uint8_t>& a, const std::vector<uint8_t>& b) {
    double s = 0;
    for (size_t i = 0; i < a.size(); i += 4) {
        s += std::abs((int)a[i] - (int)b[i]) + std::abs((int)a[i + 1] - (int)b[i + 1]) +
             std::abs((int)a[i + 2] - (int)b[i + 2]);
    }
    return s / (double)(a.size() / 4 * 3);
}

static int max_corner_diff(const std::vector<uint8_t>& a, const std::vector<uint8_t>& b) {
    int m = 0;
    for (int y = 0; y < 6; ++y) {
        for (int x = 0; x < 6; ++x) {
            size_t i = ((size_t)y * W + x) * 4;
            for (int c = 0; c < 3; ++c) m = std::max(m, std::abs((int)a[i + c] - (int)b[i + c]));
        }
    }
    return m;
}

static bool raster_2d(const Path2& path, std::vector<uint8_t>& rgba) {
    std::vector<uint32_t> buf((size_t)W2 * H2, 0);
    auto* canvas = tvg::SwCanvas::gen();
    if (!canvas) return false;
    if (canvas->target(buf.data(), W2, W2, H2, tvg::ColorSpace::ABGR8888S) != tvg::Result::Success) return false;
    auto* bg = tvg::Shape::gen();
    bg->appendRect(0, 0, (float)W2, (float)H2);
    bg->fill(18, 20, 24, 255);
    // Stroke width is in local units and ThorVG scales it with the paint transform,
    // which turned a 3px stroke into a canvas-sized blob. Bake the view here.
    Path2 screen = path;
    for (tvg::Point& p : screen.pts) {
        float x = p.x * 300.f + 640.f;
        float y = -p.y * 300.f + 210.f;
        p.x = x;
        p.y = y;
    }
    auto* stroke = tvg::Shape::gen();
    if (!write_path(stroke, screen)) return false;
    stroke->strokeWidth(3.2f);
    stroke->strokeJoin(tvg::StrokeJoin::Round);
    stroke->strokeCap(tvg::StrokeCap::Round);
    stroke->strokeFill(236, 224, 196, 255);
    if (canvas->add(bg) != tvg::Result::Success) return false;
    if (canvas->add(stroke) != tvg::Result::Success) return false;
    if (canvas->draw(true) != tvg::Result::Success) return false;
    if (canvas->sync() != tvg::Result::Success) return false;
    rgba.resize((size_t)W2 * H2 * 4);
    int ink = 0;
    for (int i = 0; i < W2 * H2; ++i) {
        uint32_t p = buf[i];
        uint8_t r = (uint8_t)(p & 255);
        uint8_t g = (uint8_t)((p >> 8) & 255);
        uint8_t b = (uint8_t)((p >> 16) & 255);
        uint8_t a = (uint8_t)((p >> 24) & 255);
        rgba[(size_t)i * 4 + 0] = r;
        rgba[(size_t)i * 4 + 1] = g;
        rgba[(size_t)i * 4 + 2] = b;
        rgba[(size_t)i * 4 + 3] = 255;
        if (a > 0 && (r > 40 || g > 40 || b > 40) && !(r < 30 && g < 32 && b < 36)) ++ink;
    }
    return ink > 500;
}

struct R {
    Engine* engine = nullptr;
    Renderer* renderer = nullptr;
    Scene* scene = nullptr;
    View* view = nullptr;
    SwapChain* swap = nullptr;
    Camera* camera = nullptr;
    MaterialInstance* glass = nullptr;
    utils::Entity sun, fill, backdrop, floor, tube = {};
    VertexBuffer* tube_vb = nullptr;
    IndexBuffer* tube_ib = nullptr;
};


#include "probe_2.h"
