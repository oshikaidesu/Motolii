// 1920x1080 collage. ThorVG draws the paper. The same paths become Filament meshes,
// so a line and a metal/glass fragment are one contour. A second ThorVG pass is
// composited in front. Dither is the hash in motolii-glue shelf/blit.wgsl.
#include <thorvg.h>

#include <filament/Camera.h>
#include <filament/ColorGrading.h>
#include <filament/Engine.h>
#include <filament/IndexBuffer.h>
#include <filament/IndirectLight.h>
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
static const float X0 = -16.f, X1 = 16.f, Y0 = -9.f, Y1 = 9.f;

struct Rng {
    uint32_t s;
    float u() { s = s * 1664525u + 1013904223u; return (s >> 8) * (1.f / 16777216.f); }
    float r(float a, float b) { return a + (b - a) * u(); }
};

static float3 world(float px, float py, float z) {
    return {(px / W) * (X1 - X0) + X0, Y1 - (py / H) * (Y1 - Y0), z};
}

static float hash01(float x, float y) {
    float qx = x * 0.1031f; qx -= std::floor(qx);
    float qy = y * 0.1030f; qy -= std::floor(qy);
    float d = qx * (qy + 33.33f) + qy * (qx + 33.33f);
    float rx = qx + d, ry = qy + d;
    float h = (rx + ry) * rx;
    return h - std::floor(h);
}

struct Lit {
    std::vector<float> v;
    std::vector<uint32_t> idx;
    void vert(float3 p, float3 n, float3 t) {
        float3 b = normalize(cross(n, t));
        quatf q = mat3f::packTangentFrame(mat3f{t, b, n});
        float pack[7] = {p.x, p.y, p.z, q.x, q.y, q.z, q.w};
        v.insert(v.end(), pack, pack + 7);
    }
};

struct Flat {
    std::vector<float> v;
    std::vector<uint32_t> idx;
};

static void tube(Lit& m, const std::vector<float3>& center, const std::vector<float3>& tang, float radius, int sides) {
    int base = (int)(m.v.size() / 7);
    int rings = (int)center.size();
    for (int i = 0; i < rings; ++i) {
        float3 T = tang[i];
        float3 side = normalize(cross(T, float3{0, 0, 1}));
        float3 lift = normalize(cross(side, T));
        for (int k = 0; k < sides; ++k) {
            float ang = (float)k / sides * 6.2831853f;
            float3 radial = std::cos(ang) * lift + std::sin(ang) * side;
            m.vert(center[i] + radius * radial, radial, T);
        }
    }
    for (int i = 0; i < rings - 1; ++i) {
        for (int k = 0; k < sides; ++k) {
            uint32_t a = base + i * sides + k;
            uint32_t b = base + i * sides + (k + 1) % sides;
            uint32_t c = base + (i + 1) * sides + k;
            uint32_t d = base + (i + 1) * sides + (k + 1) % sides;
            m.idx.insert(m.idx.end(), {a, b, c, b, d, c});
        }
    }
}

static void sphere(Lit& m, float3 c, float r, int stacks, int slices) {
    int base = (int)(m.v.size() / 7);
    for (int i = 0; i <= stacks; ++i) {
        float theta = (float)i / stacks * 3.14159265f;
        for (int j = 0; j <= slices; ++j) {
            float phi = (float)j / slices * 6.2831853f;
            float3 n{std::sin(theta) * std::cos(phi), std::cos(theta), std::sin(theta) * std::sin(phi)};
            float3 t{std::sin(theta) * -std::sin(phi), 0, std::sin(theta) * std::cos(phi)};
            if (length(t) < 1e-4f) t = {1, 0, 0};
            m.vert(c + r * n, n, normalize(t));
        }
    }
    int row = slices + 1;
    for (int i = 0; i < stacks; ++i) {
        for (int j = 0; j < slices; ++j) {
            uint32_t a = base + i * row + j;
            uint32_t b = a + row;
            m.idx.insert(m.idx.end(), {a, b, a + 1, b, b + 1, a + 1});
        }
    }
}

static void slab(Lit& m, const std::vector<float3>& c, const std::vector<float3>& tang, float hw) {
    int n = (int)c.size();
    int base = (int)(m.v.size() / 7);
    for (int i = 0; i < n; ++i) {
        float3 side = normalize(cross(tang[i], float3{0, 0, 1}));
        if (length(side) < 1e-4f) side = {1, 0, 0};
        float wob = std::sin((float)i * 0.65f);
        float3 nrm = normalize(side * 0.55f + tang[i] * wob * 0.35f + float3{0, 0, 1});
        m.vert(c[i] + side * hw, nrm, tang[i]);
        m.vert(c[i] - side * hw, normalize(nrm - side * 0.8f), tang[i]);
    }
    for (int i = 0; i < n - 1; ++i) {
        uint32_t a = base + i * 2, b = a + 1, c2 = a + 2, d = a + 3;
        m.idx.insert(m.idx.end(), {a, c2, b, b, c2, d});
    }
    int lip = (int)(m.v.size() / 7);
    for (int i = 0; i < n; ++i) {
        float3 side = normalize(cross(tang[i], float3{0, 0, 1}));
        float3 nrm = normalize(side + float3{0, 0, 0.85f});
        float3 outer = c[i] + side * (hw + 0.14f);
        outer.z -= 0.22f;
        m.vert(c[i] + side * hw, nrm, tang[i]);
        m.vert(outer, nrm, tang[i]);
    }
    for (int i = 0; i < n - 1; ++i) {
        uint32_t a = lip + i * 2, b = a + 1, c2 = a + 2, d = a + 3;
        m.idx.insert(m.idx.end(), {a, b, c2, b, d, c2});
    }
}

static void blob(Lit& m, float cx, float cy, float rx, float ry, float z, float lift, int n, uint32_t seed) {
    Rng rng{seed};
    std::vector<float> rads(n), ang(n);
    for (int i = 0; i < n; ++i) {
        ang[i] = (float)i / n * 6.2831853f;
        rads[i] = 0.55f + 0.62f * rng.u();
    }
    auto at = [&](int i, float scale, float zz) {
        float a = ang[i];
        return world(cx + std::cos(a) * rx * rads[i] * scale, cy + std::sin(a) * ry * rads[i] * scale, zz);
    };
    int base = (int)(m.v.size() / 7);
    float3 mid = world(cx, cy, z + lift);
    m.vert(mid, {0, 0, 1}, {1, 0, 0});
    for (int i = 0; i < n; ++i) {
        float3 p = at(i, 0.68f, z + lift);
        float3 nrm = normalize(float3{(p.x - mid.x) * 1.4f, (p.y - mid.y) * 1.4f, 1.f});
        m.vert(p, nrm, {1, 0, 0});
    }
    for (int i = 0; i < n; ++i)
        m.idx.insert(m.idx.end(), {(uint32_t)base, (uint32_t)(base + 1 + i), (uint32_t)(base + 1 + (i + 1) % n)});
    int rim = (int)(m.v.size() / 7);
    mid = world(cx, cy, z);
    for (int i = 0; i < n; ++i) {
        float3 outward = at(i, 1.f, z) - mid;
        outward.z = 0.35f;
        outward = normalize(outward);
        m.vert(at(i, 0.68f, z + lift), outward, {0, 0, 1});
        m.vert(at(i, 1.f, z), outward, {0, 0, 1});
    }
    for (int i = 0; i < n; ++i) {
        int j = (i + 1) % n;
        uint32_t a = rim + i * 2, b = a + 1, c2 = rim + j * 2, d = c2 + 1;
        m.idx.insert(m.idx.end(), {a, c2, b, b, c2, d});
    }
}

static void sample_cubic(float3 a, float3 b, float3 c, float3 d, int n, std::vector<float3>& p, std::vector<float3>& t) {
    for (int i = 0; i <= n; ++i) {
        float u = (float)i / n, s = 1.f - u;
        float3 q = (s * s * s) * a + (3 * s * s * u) * b + (3 * s * u * u) * c + (u * u * u) * d;
        float3 dq = 3 * s * s * (b - a) + 6 * s * u * (c - b) + 3 * u * u * (d - c);
        if (length(dq) < 1e-4f) dq = d - a;
        p.push_back(q);
        t.push_back(normalize(dq));
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

static tvg::Shape* rect(float x, float y, float w, float h, uint8_t r, uint8_t g, uint8_t b, uint8_t a = 255) {
    auto* s = tvg::Shape::gen();
    s->appendRect(x, y, w, h);
    s->fill(r, g, b, a);
    return s;
}

static void checker(tvg::Canvas* canvas, float x, float y, float w, float h, float cell) {
    auto* s = tvg::Shape::gen();
    for (float yy = y; yy < y + h; yy += cell) {
        for (float xx = x; xx < x + w; xx += cell) {
            int ix = (int)((xx - x) / cell), iy = (int)((yy - y) / cell);
            if (((ix + iy) & 1) == 0) s->appendRect(xx, yy, std::min(cell, x + w - xx), std::min(cell, y + h - yy));
        }
    }
    s->fill(196, 196, 198, 255);
    canvas->add(s);
}

static void cubic(tvg::Shape* s, float x0, float y0, float x1, float y1, float x2, float y2, float x3, float y3) {
    s->moveTo(x0, y0);
    s->cubicTo(x1, y1, x2, y2, x3, y3);
}

static void label(tvg::Canvas* canvas, const char* font, float x, float y, float size, const char* text, uint8_t r, uint8_t g, uint8_t b) {
    auto* t = tvg::Text::gen();
    if (t->font(font) != tvg::Result::Success) t->font(nullptr);
    t->size(size);
    t->text(text);
    t->fill(r, g, b);
    t->translate(x, y);
    canvas->add(t);
}

static std::vector<uint32_t> photo(int w, int h, bool ocean) {
    std::vector<uint32_t> px((size_t)w * h);
    Rng rng{ocean ? 9u : 3u};
    for (int y = 0; y < h; ++y) {
        for (int x = 0; x < w; ++x) {
            float u = (float)x / w, v = (float)y / h;
            int r, g, b;
            if (!ocean) {
                r = (int)(70 + 90 * (1 - v) + 40 * std::exp(-((u - 0.72f) * (u - 0.72f) + (v - 0.28f) * (v - 0.28f)) * 18.f));
                g = (int)(140 + 70 * (1 - v));
                b = (int)(210 - 30 * v);
                auto puff = [&](float cx, float cy, float rx, float ry) {
                    float d = std::hypot((u - cx) / rx, (v - cy) / ry);
                    if (d < 1.f) {
                        int c = (int)(250 - 30 * d);
                        r = std::max(r, c); g = std::max(g, c); b = std::max(b, c);
                    }
                };
                puff(0.32f, 0.46f, 0.18f, 0.12f);
                puff(0.48f, 0.40f, 0.14f, 0.10f);
                puff(0.22f, 0.55f, 0.10f, 0.08f);
            } else {
                float crest = std::pow(std::max(0.f, std::sin(u * 22.f + v * 4.f)), 8.f);
                float swell = 0.5f + 0.5f * std::sin(u * 9.f - v * 3.f);
                r = (int)(8 + 40 * swell + 180 * crest);
                g = (int)(40 + 70 * (1.f - v) + 160 * crest);
                b = (int)(90 + 100 * swell + 40 * (1.f - v));
            }
            int n = (int)((rng.u() - 0.5f) * 10.f);
            r = std::clamp(r + n, 0, 255);
            g = std::clamp(g + n, 0, 255);
            b = std::clamp(b + n, 0, 255);
            px[(size_t)y * w + x] = (255u << 24) | ((uint32_t)b << 16) | ((uint32_t)g << 8) | (uint32_t)r;
        }
    }
    return px;
}

struct Crv { float x0, y0, x1, y1, x2, y2, x3, y3; };
// The field is the grid. The curves are what leave it.
static const float GX0 = 72, GX1 = 1848;
static const float GW0 = 528, GW1 = 1320, GWY = 288, GWH = 504;
static const float GR0 = 1392, GR1 = 1848;
static const float GRULE = 672;
static const Crv kChromeCrv[] = {
    {90, 920, 260, 780, 480, 640, 760, 500},
    {70, 500, 240, 620, 180, 800, 420, 940},
    {340, 460, 560, 560, 780, 400, 1040, 340},
};
static const Crv kGlassCrv{620, 460, 820, 250, 1080, 240, 1280, 420};
static const Crv kMess[] = {
    {360, 180, 640, 420, 980, 160, 1500, 380},
    {440, 760, 720, 520, 1100, 700, 1560, 460},
    {160, 360, 520, 240, 900, 520, 1400, 220},
    {980, 860, 760, 640, 1180, 720, 420, 980},
    {1500, 80, 1240, 360, 860, 180, 480, 460},
    {80, 180, 400, 480, 880, 260, 1320, 640},
    {600, 640, 900, 480, 1200, 560, 1700, 300},
    {200, 980, 600, 800, 1000, 900, 1500, 700},
};
static const Crv kCross[] = {
    {GX0, GRULE, 420, 520, 1100, 260, 1960, 64},
    {GW0, GWY, 820, 150, 1120, 190, GX1, 88},
    {150, 780, 500, 640, 1000, 400, 1900, 280},
    {GX0, 96, 640, 36, 1200, 150, 1960, 40},
    {1180, 520, 860, 700, 420, 880, 24, 1040},
};

static void splat(tvg::Shape* s, float x, float y, float rx, float ry, float turn) {
    s->moveTo(x + std::cos(turn) * rx, y + std::sin(turn) * ry);
    for (int i = 1; i <= 5; ++i) {
        float a0 = turn + (i - 0.5f) * 1.2566f;
        float a1 = turn + i * 1.2566f;
        float k = (i & 1) ? 1.25f : 0.72f;
        s->cubicTo(x + std::cos(a0) * rx * k, y + std::sin(a0) * ry * k,
                   x + std::cos(a0) * rx * k, y + std::sin(a0) * ry * k,
                   x + std::cos(a1) * rx, y + std::sin(a1) * ry);
    }
    s->close();
}


#include "collage_2.h"
