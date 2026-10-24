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


#include "gpu_tube_2.h"
