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


#include "torture_2.h"
#include "torture_3.h"
