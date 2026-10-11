// Path torture. Same Path → tube → Filament handoff as link.mm.
// link.mm, probe.mm, and the materials are not modified.
#include <thorvg.h>

#include <filament/Camera.h>
#include <filament/ColorGrading.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
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
#include <new>
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
static const int MAX_U16 = 65535;

struct Path2 {
    std::vector<tvg::PathCommand> cmds;
    std::vector<tvg::Point> pts;
};

struct Mesh {
    std::vector<float> v;
    std::vector<uint16_t> idx;
    std::vector<float3> centers;
    std::vector<int> ring_sub;
    int spans = 0;
    int subpaths = 0;
    bool index_overflow = false;
    double center_err = 0;
    double sample_ms = 0;
    double mesh_ms = 0;
};

struct Times {
    double path = 0;
    double sample = 0;
    double mesh = 0;
    double upload = 0;
    double draw = 0;
    bool uploaded = false;
    bool drew = false;
};

static int fail(const char* msg) {
    std::fprintf(stderr, "FAIL %s\n", msg);
    return 1;
}

static double ms_since(std::chrono::steady_clock::time_point t0) {
    return std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
}

static double median_of(std::vector<double> v) {
    if (v.empty()) return -1;
    std::sort(v.begin(), v.end());
    return v[v.size() / 2];
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

static float path_delta(const Path2& a, const Path2& b) {
    if (a.cmds != b.cmds || a.pts.size() != b.pts.size()) return -1.f;
    float m = 0;
    for (size_t i = 0; i < a.pts.size(); ++i) {
        m = std::max(m, std::fabs(a.pts[i].x - b.pts[i].x));
        m = std::max(m, std::fabs(a.pts[i].y - b.pts[i].y));
    }
    return m;
}

static int count_anchors(const Path2& path) {
    int n = 0;
    for (tvg::PathCommand cmd : path.cmds)
        if (cmd != tvg::PathCommand::Close) ++n;
    return n;
}

static int count_subpaths(const Path2& path) {
    int n = 0;
    for (tvg::PathCommand cmd : path.cmds)
        if (cmd == tvg::PathCommand::MoveTo) ++n;
    return n;
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
    struct Span { tvg::Point a, b, c, d; int sub; };
    Mesh m;
    auto t0 = std::chrono::steady_clock::now();
    std::vector<Span> spans;
    const tvg::Point* p = path.pts.data();
    tvg::Point start{}, cur{};
    bool has = false;
    int sub = -1;
    for (tvg::PathCommand cmd : path.cmds) {
        switch (cmd) {
        case tvg::PathCommand::MoveTo: start = cur = *p++; has = true; ++sub; break;
        case tvg::PathCommand::LineTo: {
            tvg::Point n = *p++;
            spans.push_back({cur, cur, n, n, sub});
            cur = n; break;
        }
        case tvg::PathCommand::CubicTo: {
            tvg::Point c0 = *p++, c1 = *p++, n = *p++;
            spans.push_back({cur, c0, c1, n, sub});
            cur = n; break;
        }
        case tvg::PathCommand::Close:
            if (has && !near_pt(cur, start)) spans.push_back({cur, cur, start, start, sub});
            cur = start; break;
        }
    }
    m.spans = (int)spans.size();
    m.subpaths = sub + 1;
    const int rings = m.spans * SAMPLES;
    const int verts = rings * SIDES;
    m.index_overflow = verts > MAX_U16;
    m.centers.resize(rings);
    m.ring_sub.resize(rings);
    std::vector<float3> T(rings), side(rings), lift(rings);
    std::vector<tvg::Point> expect(rings);
    for (int s = 0; s < m.spans; ++s) {
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
            expect[ring] = q;
            T[ring] = {dq.x / len, dq.y / len, 0};
            side[ring] = normalize(cross(T[ring], float3{0, 0, 1}));
            lift[ring] = normalize(cross(side[ring], T[ring]));
            m.centers[ring] = {q.x, q.y, 0};
            m.ring_sub[ring] = spans[s].sub;
        }
    }
    m.sample_ms = ms_since(t0);
    auto t1 = std::chrono::steady_clock::now();
    m.v.resize((size_t)verts * STRIDE);
    for (int ring = 0; ring < rings; ++ring) {
        for (int k = 0; k < SIDES; ++k) {
            float ang = (float)k / SIDES * 6.2831853f;
            float3 radial = std::cos(ang) * lift[ring] + std::sin(ang) * side[ring];
            float3 pos = m.centers[ring] + TUBE_R * radial;
            quatf qq = mat3f::packTangentFrame(mat3f{T[ring], normalize(cross(T[ring], radial)), radial});
            float u = rings ? (float)ring / (float)rings : 0;
            float v = (float)k / SIDES;
            float* dst = m.v.data() + ((size_t)ring * SIDES + k) * STRIDE;
            float pack[STRIDE] = {pos.x, pos.y, pos.z, qq.x, qq.y, qq.z, qq.w, u, v};
            std::memcpy(dst, pack, sizeof(pack));
        }
        float3 radial{m.v[(size_t)ring * SIDES * STRIDE] - m.centers[ring].x,
                      m.v[(size_t)ring * SIDES * STRIDE + 1] - m.centers[ring].y,
                      m.v[(size_t)ring * SIDES * STRIDE + 2] - m.centers[ring].z};
        m.center_err = std::max(m.center_err, (double)std::fabs(length(radial) - TUBE_R));
        m.center_err = std::max(m.center_err, (double)std::fabs(m.centers[ring].x - expect[ring].x));
        m.center_err = std::max(m.center_err, (double)std::fabs(m.centers[ring].y - expect[ring].y));
    }
    if (!m.index_overflow && rings > 0) {
        m.idx.resize((size_t)rings * SIDES * 6);
        size_t w = 0;
        for (int i = 0; i < rings; ++i) {
            int i1 = (i + 1) % rings;
            for (int k = 0; k < SIDES; ++k) {
                uint16_t a = (uint16_t)(i * SIDES + k);
                uint16_t b = (uint16_t)(i * SIDES + (k + 1) % SIDES);
                uint16_t c = (uint16_t)(i1 * SIDES + k);
                uint16_t d = (uint16_t)(i1 * SIDES + (k + 1) % SIDES);
                m.idx[w++] = a; m.idx[w++] = b; m.idx[w++] = c;
                m.idx[w++] = b; m.idx[w++] = d; m.idx[w++] = c;
            }
        }
    }
    m.mesh_ms = ms_since(t1);
    return m;
}

static int bridge_count(const Mesh& m) {
    int rings = (int)m.ring_sub.size();
    int n = 0;
    if (rings < 2) return 0;
    for (int i = 0; i < rings; ++i)
        if (m.ring_sub[i] != m.ring_sub[(i + 1) % rings]) ++n;
    return n;
}

static float cross2(float ax, float ay, float bx, float by) { return ax * by - ay * bx; }

static bool seg_hit(tvg::Point a, tvg::Point b, tvg::Point c, tvg::Point d) {
    float d1 = cross2(b.x - a.x, b.y - a.y, c.x - a.x, c.y - a.y);
    float d2 = cross2(b.x - a.x, b.y - a.y, d.x - a.x, d.y - a.y);
    float d3 = cross2(d.x - c.x, d.y - c.y, a.x - c.x, a.y - c.y);
    float d4 = cross2(d.x - c.x, d.y - c.y, b.x - c.x, b.y - c.y);
    return ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0)) && ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0));
}

static void centerline(const Path2& path, std::vector<tvg::Point>& pts) {
    struct Span { tvg::Point a, b, c, d; };
    std::vector<Span> spans;
    const tvg::Point* p = path.pts.data();
    tvg::Point start{}, cur{};
    bool has = false;
    for (tvg::PathCommand cmd : path.cmds) {
        switch (cmd) {
        case tvg::PathCommand::MoveTo: start = cur = *p++; has = true; break;
        case tvg::PathCommand::LineTo: {
            tvg::Point n = *p++; spans.push_back({cur, cur, n, n}); cur = n; break;
        }
        case tvg::PathCommand::CubicTo: {
            tvg::Point c0 = *p++, c1 = *p++, n = *p++;
            spans.push_back({cur, c0, c1, n}); cur = n; break;
        }
        case tvg::PathCommand::Close:
            if (has && !near_pt(cur, start)) spans.push_back({cur, cur, start, start});
            cur = start; break;
        }
    }
    pts.clear();
    for (const Span& s : spans)
        for (int i = 0; i < SAMPLES; ++i) {
            float t = (float)i / (float)SAMPLES;
            pts.push_back(bezier(s.a, s.b, s.c, s.d, t));
        }
}

static bool self_intersects(const std::vector<tvg::Point>& pts) {
    const int n = (int)pts.size();
    if (n > 4000) return false;
    for (int i = 0; i < n; ++i) {
        tvg::Point a = pts[i], b = pts[(i + 1) % n];
        for (int j = i + 2; j < n; ++j) {
            if (i == 0 && j == n - 1) continue;
            int gap = std::min(j - i, n - (j - i));
            if (gap <= 2) continue;
            if (seg_hit(a, b, pts[j], pts[(j + 1) % n])) return true;
        }
    }
    return false;
}

static float min_clearance(const std::vector<tvg::Point>& pts) {
    const int n = (int)pts.size();
    if (n > 4000) return -1.f;
    float m = 1e9f;
    for (int i = 0; i < n; ++i)
        for (int j = i + 1; j < n; ++j) {
            int gap = std::min(j - i, n - (j - i));
            if (gap <= SAMPLES) continue;
            float dx = pts[i].x - pts[j].x, dy = pts[i].y - pts[j].y;
            m = std::min(m, std::sqrt(dx * dx + dy * dy));
        }
    return m;
}

static void add_rect(Path2& p, float x0, float y0, float x1, float y1, bool ccw) {
    tvg::Point bl{x0, y0}, br{x1, y0}, tr{x1, y1}, tl{x0, y1};
    p.cmds.push_back(tvg::PathCommand::MoveTo);
    p.pts.push_back(bl);
    const tvg::Point seq[3] = {ccw ? br : tl, ccw ? tr : tr, ccw ? tl : br};
    for (tvg::Point q : seq) {
        p.cmds.push_back(tvg::PathCommand::LineTo);
        p.pts.push_back(q);
    }
    p.cmds.push_back(tvg::PathCommand::Close);
}

static Path2 rectangle() {
    Path2 p;
    add_rect(p, -0.85f, -0.40f, 0.85f, 0.40f, true);
    return p;
}

static Path2 hole_shape() {
    Path2 p;
    add_rect(p, -0.70f, -0.45f, 0.70f, 0.45f, true);
    add_rect(p, -0.28f, -0.18f, 0.28f, 0.18f, false);
    return p;
}

static void add_oval(Path2& p, float cx, float cy, float rx, float ry) {
    const float k = 0.55228475f;
    auto P = [&](float x, float y) { return tvg::Point{cx + x, cy + y}; };
    auto cub = [&](tvg::Point c0, tvg::Point c1, tvg::Point n) {
        p.cmds.push_back(tvg::PathCommand::CubicTo);
        p.pts.push_back(c0); p.pts.push_back(c1); p.pts.push_back(n);
    };
    p.cmds.push_back(tvg::PathCommand::MoveTo);
    p.pts.push_back(P(rx, 0));
    cub(P(rx, k * ry), P(k * rx, ry), P(0, ry));
    cub(P(-k * rx, ry), P(-rx, k * ry), P(-rx, 0));
    cub(P(-rx, -k * ry), P(-k * rx, -ry), P(0, -ry));
    cub(P(k * rx, -ry), P(rx, -k * ry), P(rx, 0));
    p.cmds.push_back(tvg::PathCommand::Close);
}

static Path2 compound_shape() {
    Path2 p;
    add_oval(p, -0.48f, 0.22f, 0.24f, 0.18f);
    add_oval(p, 0.46f, 0.20f, 0.20f, 0.26f);
    add_oval(p, 0.02f, -0.32f, 0.28f, 0.16f);
    return p;
}

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
