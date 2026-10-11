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

static void draw_back(tvg::Canvas* canvas, const char* font) {
    canvas->add(rect(0, 0, W, H, 246, 246, 244));
    checker(canvas, GX0, 96, GW0 - GX0 - 24, 168, 16);
    checker(canvas, GW0, 96, GW1 - GW0, 144, 16);
    checker(canvas, GX0, 840, GW0 - GX0 - 24, 168, 16);
    checker(canvas, GR0, 864, GR1 - GR0, 144, 16);

    auto* rules = tvg::Shape::gen();
    rules->moveTo(GX0, 96);
    rules->lineTo(GW0 - 40, 96);
    rules->moveTo(GW1 + 80, GWY);
    rules->lineTo(GX1, GWY);
    rules->moveTo(GX0, GWY + GWH);
    rules->lineTo(GW0 - 60, GWY + GWH);
    rules->moveTo(GX0, 96);
    rules->lineTo(GX0, GRULE - 40);
    rules->strokeWidth(0.8f);
    rules->strokeFill(20, 20, 24, 255);
    canvas->add(rules);

    canvas->add(rect(GW0, GWY, GW1 - GW0, GWH, 10, 10, 12));
    checker(canvas, GW0 - 36, GWY - 28, 200, 110, 14);
    checker(canvas, GW1 - 80, GWY + GWH - 40, 220, 120, 14);
    auto sky = photo(560, 320, false);
    auto* pic = tvg::Picture::gen();
    pic->load(sky.data(), 560, 320, tvg::ColorSpace::ABGR8888S, true);
    pic->size(580, 320);
    pic->translate(GW0 - 48, GWY - 56);
    canvas->add(pic);
    auto sea = photo(340, 200, true);
    auto* sea_pic = tvg::Picture::gen();
    sea_pic->load(sea.data(), 340, 200, tvg::ColorSpace::ABGR8888S, true);
    sea_pic->size(320, 180);
    sea_pic->translate(GW1 - 140, GWY + GWH - 70);
    canvas->add(sea_pic);

    auto* clouds = tvg::Shape::gen();
    clouds->appendCircle(860, GWY, 120, 90);
    clouds->appendCircle(740, GWY + 24, 70, 56);
    clouds->appendCircle(980, GWY + 16, 84, 64);
    clouds->fill(214, 214, 216, 255);
    canvas->add(clouds);
    auto* dots = tvg::Shape::gen();
    for (int i = 0; i < 120; ++i) {
        float a = i * 2.399f;
        float rad = 8.f + (i % 10) * 8.f;
        dots->appendCircle(860 + std::cos(a) * rad, GWY + std::sin(a) * rad * 0.7f, 1.4f, 1.4f);
    }
    dots->fill(80, 80, 88, 255);
    canvas->add(dots);

    auto* red = tvg::Shape::gen();
    splat(red, GW0 - 20, GWY + 70, 78, 40, 0.6f);
    splat(red, 700, GWY + 90, 56, 32, 1.4f);
    splat(red, 1100, 500, 90, 48, 0.3f);
    red->fill(226, 24, 58, 230);
    canvas->add(red);

    Rng rng{11};
    auto* pink = tvg::Shape::gen();
    auto* white_fan = tvg::Shape::gen();
    for (int i = 0; i < 56; ++i) {
        float ang = (i < 36) ? 0.05f + rng.r(0.f, 1.2f) : -1.1f + rng.r(0.f, 0.7f);
        float len = rng.r(140.f, 780.f);
        float x = (i % 3 == 0 ? GW0 + 40.f : 1180.f) + rng.r(-40.f, 40.f);
        float y = (i % 3 == 0 ? GWY + 80.f : 560.f) + rng.r(-30.f, 30.f);
        float x1 = x - std::cos(ang) * len, y1 = y + std::sin(ang) * len;
        float w = rng.r(5.f, 28.f);
        float nx = std::sin(ang) * w, ny = std::cos(ang) * w;
        tvg::Shape* dst = (i % 6 == 0) ? white_fan : pink;
        dst->moveTo(x, y);
        dst->lineTo(x1 + nx, y1 + ny);
        dst->lineTo(x1 - nx * 0.35f, y1 - ny * 0.35f);
        dst->close();
    }
    pink->fill(255, 42, 86, 230);
    white_fan->fill(255, 255, 255, 220);
    canvas->add(pink);
    canvas->add(white_fan);

    auto* blue = tvg::Shape::gen();
    auto* blue_fat = tvg::Shape::gen();
    for (int i = 0; i < 28; ++i) {
        float ang = -0.9f + rng.r(0.f, 1.3f);
        float len = rng.r(160.f, 820.f);
        float x = 980 + rng.r(-80.f, 120.f), y = 340 + rng.r(-40.f, 80.f);
        tvg::Shape* dst = (i % 5 == 0) ? blue_fat : blue;
        dst->moveTo(x, y);
        dst->lineTo(x + std::cos(ang) * len, y + std::sin(ang) * len);
    }
    for (const Crv& c : kCross) cubic(blue, c.x0, c.y0 + 10, c.x1, c.y1 + 12, c.x2, c.y2 - 8, c.x3, c.y3 + 6);
    for (const Crv& c : kMess) cubic(blue, c.x0, c.y0, c.x1, c.y1, c.x2, c.y2, c.x3, c.y3);
    blue->strokeWidth(0.9f);
    blue->strokeFill(32, 72, 200, 200);
    blue_fat->strokeWidth(3.4f);
    blue_fat->strokeFill(20, 50, 180, 180);
    canvas->add(blue);
    canvas->add(blue_fat);

    auto* wires = tvg::Shape::gen();
    wires->appendCircle(250, GRULE, 150, 150);
    wires->appendCircle(250, GRULE, 78, 78);
    wires->strokeWidth(0.9f);
    wires->strokeFill(50, 84, 140, 230);
    canvas->add(wires);

    auto* rule = tvg::Shape::gen();
    rule->moveTo(GX0, GRULE);
    rule->lineTo(GW0 - 24, GRULE);
    for (int i = 0; i < 36; ++i) {
        float x = GX0 + 8 + i * 12.f;
        rule->moveTo(x, GRULE - 8);
        rule->lineTo(x, i % 4 == 0 ? GRULE + 14 : GRULE + 6);
    }
    rule->strokeWidth(1.f);
    rule->strokeFill(16, 16, 20, 255);
    canvas->add(rule);

    const char* words[] = {"one", "[w]", "[m]", "\"", "200", "HYBRID", "A1", "v.08", "px", "ior", "path", "01", "0.00", "comp"};
    for (int i = 0; i < 36; ++i) {
        float x = rng.r(GW0 - 80, GW1 + 60), y = rng.r(GWY - 40, GWY + GWH + 80);
        bool on_black = x > GW0 && x < GW1 && y > GWY && y < GWY + GWH;
        label(canvas, font, x, y, rng.r(7.f, i % 7 == 0 ? 26.f : 13.f), words[i % 14],
              on_black ? 230 : 28, on_black ? 230 : 28, on_black ? 232 : 32);
    }
    label(canvas, font, GX0 + 8, 132, 18, "one", 24, 24, 28);
    label(canvas, font, 460, 250, 34, "HYBRID", 24, 24, 28);
    label(canvas, font, GX0 + 12, GRULE - 36, 28, "\"", 24, 24, 28);
}

static void draw_front(tvg::Canvas* canvas, const char* font) {
    auto* over = tvg::Shape::gen();
    for (const Crv& c : kCross) cubic(over, c.x0, c.y0, c.x1, c.y1, c.x2, c.y2, c.x3, c.y3);
    over->strokeWidth(1.45f);
    over->strokeFill(24, 64, 210, 230);
    canvas->add(over);
    auto* white = tvg::Shape::gen();
    cubic(white, 660, 250, 860, 150, 1080, 200, 1280, 250);
    cubic(white, 720, 340, 940, 230, 1140, 270, 1360, 300);
    white->strokeWidth(2.4f);
    white->strokeFill(255, 255, 255, 230);
    canvas->add(white);
    auto* cut = tvg::Shape::gen();
    cubic(cut, 420, 360, 700, 200, 980, 420, 1400, 260);
    cubic(cut, 200, 700, 560, 480, 900, 640, 1500, 500);
    cut->strokeWidth(4.f);
    cut->strokeFill(24, 24, 28, 210);
    canvas->add(cut);
    label(canvas, font, 760, 300, 22, "path", 16, 16, 20);
    label(canvas, font, 140, 700, 16, "[w]", 16, 16, 20);
    label(canvas, font, 1080, 460, 13, "ior", 245, 245, 247);
    label(canvas, font, 620, 640, 11, "v.08", 245, 245, 247);
}

int main() {
    if (tvg::Initializer::init(0) != tvg::Result::Success) {
        std::fprintf(stderr, "FAIL thorvg\n");
        return 1;
    }
    const char* font_file = "/System/Library/Fonts/Supplemental/Andale Mono.ttf";
    tvg::Result font_ok = tvg::Text::load(font_file);
    const char* font = "Andale Mono";
    std::printf("thorvg font %s\n", font_ok == tvg::Result::Success ? "loaded" : "fallback");

    std::vector<uint32_t> back_px((size_t)W * H, 0xffffffffu);
    auto* back = tvg::SwCanvas::gen();
    back->target(back_px.data(), W, W, H, tvg::ColorSpace::ABGR8888S);
    draw_back(back, font);
    back->draw(true);
    back->sync();

    std::vector<uint32_t> front_px((size_t)W * H, 0);
    auto* front = tvg::SwCanvas::gen();
    front->target(front_px.data(), W, W, H, tvg::ColorSpace::ABGR8888S);
    draw_front(front, font);
    front->draw(false);
    front->sync();

    Lit chrome_mesh, glass_mesh, bubble_mesh;
    Flat shards_back, shards_front;
    const float slab_w[] = {0.72f, 0.42f, 0.58f};
    const float slab_z[] = {0.55f, 0.85f, 1.05f};
    for (int i = 0; i < 3; ++i) {
        std::vector<float3> p, t;
        const Crv& c = kChromeCrv[i];
        sample_cubic(world(c.x0, c.y0, slab_z[i]), world(c.x1, c.y1, slab_z[i]), world(c.x2, c.y2, slab_z[i]), world(c.x3, c.y3, slab_z[i]), 36, p, t);
        slab(chrome_mesh, p, t, slab_w[i]);
    }
    blob(chrome_mesh, 500, 430, 150, 90, 0.7f, 0.35f, 18, 7);
    blob(chrome_mesh, 1180, 250, 120, 70, 1.2f, 0.28f, 16, 9);
    {
        std::vector<float3> p, t;
        const Crv& c = kGlassCrv;
        sample_cubic(world(c.x0, c.y0, 1.15f), world(c.x1, c.y1, 1.15f), world(c.x2, c.y2, 1.15f), world(c.x3, c.y3, 1.15f), 28, p, t);
        slab(glass_mesh, p, t, 0.34f);
    }
    blob(bubble_mesh, 860, 300, 170, 100, 1.25f, 0.4f, 20, 3);
    blob(bubble_mesh, 1080, 390, 130, 80, 1.4f, 0.32f, 16, 5);
    for (int i = 0; i < 7; ++i) {
        float u = (float)i / 6.f, s = 1.f - u;
        const Crv& c = kGlassCrv;
        float px = s * s * s * c.x0 + 3 * s * s * u * c.x1 + 3 * s * u * u * c.x2 + u * u * u * c.x3;
        float py = s * s * s * c.y0 + 3 * s * s * u * c.y1 + 3 * s * u * u * c.y2 + u * u * u * c.y3;
        float jx = ((i * 47) % 11 - 5) * 18.f;
        float jy = ((i * 29) % 9 - 4) * 14.f;
        float rad = 0.42f + 0.28f * std::fabs(std::sin(i * 1.7f));
        sphere(bubble_mesh, world(px + jx, py + jy, 1.35f), rad, 10, 14);
    }

    Rng rng{42};
    auto shard = [&](Flat& mesh, float z, int n, float ang0, float ang1, float len0, float len1, float ox, float oy) {
        for (int i = 0; i < n; ++i) {
            float ang = rng.r(ang0, ang1);
            float len = rng.r(len0, len1);
            float x = ox + rng.r(-18.f, 18.f), y = oy + rng.r(-14.f, 14.f);
            float x1 = x - std::cos(ang) * len, y1 = y + std::sin(ang) * len;
            float w = rng.r(16.f, 42.f);
            float3 a = world(x, y, z);
            float3 b = world(x1, y1, z);
            float3 c = world(x1 + std::sin(ang) * w, y1 + std::cos(ang) * w, z);
            uint32_t base = (uint32_t)(mesh.v.size() / 3);
            for (float3 p : {a, b, c}) mesh.v.insert(mesh.v.end(), {p.x, p.y, p.z});
            mesh.idx.insert(mesh.idx.end(), {base, base + 1, base + 2});
        }
    };
    shard(shards_back, 0.28f, 14, 0.15f, 1.2f, 180.f, 520.f, 1120, 520);
    shard(shards_front, 1.65f, 8, 0.2f, 1.05f, 90.f, 280.f, 1000, 480);
    shard(shards_front, 1.8f, 6, -0.9f, -0.05f, 120.f, 320.f, 900, 300);

    auto glass_pkg = load_file("glass.filamat");
    auto pop_pkg = load_file("glass_pop.filamat");
    auto chrome_pkg = load_file("chrome.filamat");
    auto flat_pkg = load_file("solid.filamat");
    auto tex_pkg = load_file("stripes.filamat");
    if (glass_pkg.empty() || pop_pkg.empty() || chrome_pkg.empty() || flat_pkg.empty() || tex_pkg.empty()) {
        std::fprintf(stderr, "FAIL filamat\n");
        return 1;
    }

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    Camera* camera = engine->createCamera(utils::EntityManager::get().create());
    camera->setExposure(1.f);
    camera->lookAt({0, 0, 12}, {0, 0, 0}, {0, 1, 0});
    camera->setProjection(Camera::Projection::ORTHO, X0, X1, Y0, Y1, 0.05, 40);

    Texture* color = Texture::Builder().width(W).height(H).levels(1)
                         .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                         .format(Texture::InternalFormat::RGBA8).build(*engine);
    Texture* depth = Texture::Builder().width(W).height(H).levels(1)
                         .usage(Texture::Usage::DEPTH_ATTACHMENT).format(Texture::InternalFormat::DEPTH24).build(*engine);
    RenderTarget* rt = RenderTarget::Builder()
                           .texture(RenderTarget::AttachmentPoint::COLOR, color)
                           .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
    static LinearToneMapper tone;
    view->setColorGrading(ColorGrading::Builder().toneMapper(&tone).build(*engine));
    view->setScene(scene);
    view->setCamera(camera);
    view->setRenderTarget(rt);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    ScreenSpaceReflectionsOptions ssr;
    ssr.enabled = false;
    view->setScreenSpaceReflectionsOptions(ssr);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setDithering(View::Dithering::NONE);
    BloomOptions bloom;
    bloom.enabled = false;
    bloom.strength = 0.08f;
    bloom.levels = 4;
    bloom.threshold = true;
    bloom.highlight = 8.f;
    view->setBloomOptions(bloom);
    scene->setSkybox(Skybox::Builder().color({0.96f, 0.96f, 0.95f, 1}).build(*engine));
    constexpr int CUBE = 32;
    auto* faces = new std::vector<uint8_t>((size_t)6 * CUBE * CUBE * 4);
    auto face_px = [&](int f, int x, int y, uint8_t r, uint8_t g, uint8_t b) {
        size_t i = (((size_t)f * CUBE + y) * CUBE + x) * 4;
        (*faces)[i] = r; (*faces)[i + 1] = g; (*faces)[i + 2] = b; (*faces)[i + 3] = 255;
    };
    const uint8_t face_rgb[6][3] = {
        {160, 32, 64}, {24, 48, 140}, {168, 172, 180}, {8, 8, 12}, {48, 50, 56}, {90, 120, 160},
    };
    for (int f = 0; f < 6; ++f) {
        for (int y = 0; y < CUBE; ++y) {
            for (int x = 0; x < CUBE; ++x) {
                uint8_t r = face_rgb[f][0], g = face_rgb[f][1], b = face_rgb[f][2];
                if (f == 4) {
                    if (x > 6 && x < 24 && y > 8 && y < 22) { r = 8; g = 8; b = 12; }
                    if (y > 4 && y < 7) { r = 230; g = 232; b = 236; }
                    if (y > 24 && y < 28) { r = 255; g = 48; b = 90; }
                }
                face_px(f, x, y, r, g, b);
            }
        }
    }
    Texture* ibl_tex = Texture::Builder().width(CUBE).height(CUBE).levels(1)
                           .sampler(Texture::Sampler::SAMPLER_CUBEMAP)
                           .format(Texture::InternalFormat::RGBA8).build(*engine);
    ibl_tex->setImage(*engine, 0, 0, 0, 0, CUBE, CUBE, 6,
        Texture::PixelBufferDescriptor(faces->data(), faces->size(),
            backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE,
            [](void*, size_t, void* user) { delete static_cast<std::vector<uint8_t>*>(user); }, faces));
    float3 sh[1] = {{0.28f, 0.30f, 0.34f}};
    scene->setIndirectLight(IndirectLight::Builder().reflections(ibl_tex).radiance(1, sh).intensity(14.f).build(*engine));

    auto* paper_bytes = new std::vector<uint8_t>((size_t)W * H * 4);
    for (int i = 0; i < W * H; ++i) {
        uint32_t p = back_px[i];
        (*paper_bytes)[(size_t)i * 4 + 0] = (uint8_t)(p & 255);
        (*paper_bytes)[(size_t)i * 4 + 1] = (uint8_t)((p >> 8) & 255);
        (*paper_bytes)[(size_t)i * 4 + 2] = (uint8_t)((p >> 16) & 255);
        (*paper_bytes)[(size_t)i * 4 + 3] = 255;
    }
    Texture* paper = Texture::Builder().width(W).height(H).levels(1).sampler(Texture::Sampler::SAMPLER_2D)
                         .format(Texture::InternalFormat::RGBA8).build(*engine);
    paper->setImage(*engine, 0, Texture::PixelBufferDescriptor(paper_bytes->data(), paper_bytes->size(),
        backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE, 1, 0, 0, W,
        [](void*, size_t, void* user) { delete static_cast<std::vector<uint8_t>*>(user); }, paper_bytes));

    Material* glass_mat = Material::Builder().package(glass_pkg.data(), glass_pkg.size()).build(*engine);
    Material* pop_mat = Material::Builder().package(pop_pkg.data(), pop_pkg.size()).build(*engine);
    Material* chrome_mat = Material::Builder().package(chrome_pkg.data(), chrome_pkg.size()).build(*engine);
    Material* flat_mat = Material::Builder().package(flat_pkg.data(), flat_pkg.size()).build(*engine);
    Material* tex_mat = Material::Builder().package(tex_pkg.data(), tex_pkg.size()).build(*engine);
    MaterialInstance* glass = glass_mat->createInstance();
    glass->setParameter("ior", 2.05f);
    glass->setParameter("transmission", 1.f);
    glass->setParameter("thickness", 0.7f);
    glass->setParameter("dispersion", 0.66f);
    MaterialInstance* pop = pop_mat->createInstance();
    pop->setParameter("ior", 1.92f);
    pop->setParameter("transmission", 1.f);
    pop->setParameter("thickness", 0.9f);
    pop->setParameter("dispersion", 0.58f);
    MaterialInstance* chrome = chrome_mat->createInstance();
    MaterialInstance* pink = flat_mat->createInstance();
    pink->setParameter("color", float4{1.f, 0.16f, 0.32f, 1.f});
    MaterialInstance* paper_mi = tex_mat->createInstance();
    paper_mi->setParameter("albedo", paper, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));

    auto add_lit = [&](Lit& mesh, MaterialInstance* mi) {
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(mesh.v.size() / 7)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 28)
                               .attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, 28)
                               .build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * 4));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(mesh.idx.data(), mesh.idx.size() * 4));
        utils::Entity e = utils::EntityManager::get().create();
        RenderableManager::Builder(1).boundingBox({{-20, -12, -2}, {20, 12, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi).culling(false).castShadows(false).receiveShadows(false).build(*engine, e);
        scene->addEntity(e);
    };
    auto add_flat = [&](Flat& mesh, MaterialInstance* mi) {
        VertexBuffer* vb = VertexBuffer::Builder().vertexCount((uint32_t)(mesh.v.size() / 3)).bufferCount(1)
                               .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 12)
                               .build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(mesh.v.data(), mesh.v.size() * 4));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)mesh.idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(mesh.idx.data(), mesh.idx.size() * 4));
        utils::Entity e = utils::EntityManager::get().create();
        RenderableManager::Builder(1).boundingBox({{-20, -12, -2}, {20, 12, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi).culling(false).build(*engine, e);
        scene->addEntity(e);
    };

    static const float quad[] = {
        X0, Y0, 0, 0, 0,  X1, Y0, 0, 1, 0,  X1, Y1, 0, 1, 1,  X0, Y1, 0, 0, 1,
    };
    static const uint32_t qidx[] = {0, 1, 2, 0, 2, 3};
    VertexBuffer* qvb = VertexBuffer::Builder().vertexCount(4).bufferCount(1)
                            .attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 20)
                            .attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 20).build(*engine);
    qvb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(quad, sizeof(quad)));
    IndexBuffer* qib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
    qib->setBuffer(*engine, IndexBuffer::BufferDescriptor(qidx, sizeof(qidx)));
    utils::Entity qe = utils::EntityManager::get().create();
    RenderableManager::Builder(1).boundingBox({{-20, -12, -1}, {20, 12, 1}})
        .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, qvb, qib)
        .material(0, paper_mi).culling(false).build(*engine, qe);
    scene->addEntity(qe);
    add_flat(shards_back, pink);
    add_lit(chrome_mesh, chrome);
    add_lit(glass_mesh, glass);
    add_lit(bubble_mesh, pop);
    add_flat(shards_front, pink);

    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN).color(float3{1, 0.98f, 0.95f}).intensity(6.f)
        .direction(normalize(float3{0.2f, -0.45f, -0.85f})).sunAngularRadius(0.08f).castShadows(false).build(*engine, sun);
    utils::Entity rim = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::DIRECTIONAL).color(float3{0.55f, 0.75f, 1.f}).intensity(3.f)
        .direction(normalize(float3{-0.6f, -0.2f, 0.5f})).castShadows(false).build(*engine, rim);
    scene->addEntity(sun);
    scene->addEntity(rim);

    std::vector<uint8_t> picture((size_t)W * H * 4);
    double ms = 0;
    for (int pass = 0; pass < 2; ++pass) {
        auto t0 = std::chrono::steady_clock::now();
        bool drew = false;
        for (int attempt = 0; attempt < 4 && !drew; ++attempt) {
            if (!renderer->beginFrame(swap)) continue;
            renderer->render(view);
            if (pass == 1) {
                renderer->readPixels(rt, 0, 0, W, H,
                    backend::PixelBufferDescriptor(picture.data(), picture.size(), backend::PixelDataFormat::RGBA, backend::PixelDataType::UBYTE));
            }
            renderer->endFrame();
            drew = true;
        }
        engine->flushAndWait();
        ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
        if (!drew) { std::fprintf(stderr, "FAIL frame\n"); return 1; }
    }

    std::vector<uint8_t> out = picture;
    auto at = [&](int x, int y) -> uint8_t* { return out.data() + ((size_t)y * W + x) * 4; };
    auto src_at = [&](const std::vector<uint8_t>& im, int x, int y) -> const uint8_t* {
        return im.data() + ((size_t)y * W + x) * 4;
    };
    for (int y = 0; y < H; ++y) {
        for (int x = 0; x < W; ++x) {
            uint32_t fp = front_px[(size_t)y * W + x];
            int a = (fp >> 24) & 255;
            if (a < 16) continue;
            float af = a / 255.f;
            uint8_t* o = at(x, y);
            o[0] = (uint8_t)((fp & 255) * af + o[0] * (1 - af));
            o[1] = (uint8_t)(((fp >> 8) & 255) * af + o[1] * (1 - af));
            o[2] = (uint8_t)(((fp >> 16) & 255) * af + o[2] * (1 - af));
        }
    }
    std::vector<uint8_t> split = out;
    for (int y = 0; y < H; ++y) {
        for (int x = 0; x < W; ++x) {
            int xr = std::max(0, x - 2), xb = std::min(W - 1, x + 2);
            split[((size_t)y * W + x) * 4 + 0] = src_at(out, xr, y)[0];
            split[((size_t)y * W + x) * 4 + 2] = src_at(out, xb, y)[2];
        }
    }
    for (int y = 0; y < H; ++y) {
        for (int x = 0; x < W; ++x) {
            uint8_t* p = split.data() + ((size_t)y * W + x) * 4;
            float n = (hash01((float)x, (float)y) - 0.5f) / 255.f;
            int paperish = p[0] > 236 && p[1] > 236 && p[2] > 236;
            float amp = paperish ? 1.f : 7.f;
            for (int c = 0; c < 3; ++c) {
                float v = p[c] / 255.f + n * amp;
                p[c] = (uint8_t)std::clamp(v * 255.f, 0.f, 255.f);
            }
        }
    }
    if (!write_png("out/collage.png", W, H, split.data())) return 1;

    FILE* log = std::fopen("out/collage.txt", "w");
    std::fprintf(log,
        "frame 1920x1080\n"
        "thorvg 1.1.2  Shape path, Picture raw load, Text Andale Mono, SwCanvas\n"
        "filament 1.77.2  ortho scene, glass.mat (ior 2.05 transmission 1 thickness 0.7 dispersion 0.66), glass_pop.mat bubbles (ior 1.92 thickness 0.9 dispersion 0.58), chrome.mat, IBL cubemap, refraction. SSR off: a white page reflects as a white sheet.\n"
        "connection  ThorVG pixels are the Filament backdrop, so glass refracts the paper. kChromeCrv / kGlassCrv / kCross are the same cubics as the 2D strokes: chrome ribbons, the glass tube, and thin glass threads. Pink shards share the fan angles, some behind the glass and some in front. kCross is drawn again in a second ThorVG pass composited over the 3D.\n"
        "dither  hash from motolii-glue shelf/blit.wgsl, one LSB on paper, stronger on marks\n"
        "chromatic  2px RGB split on the composite. shelf/glow.wgsl chromatic is hue rotate, not used\n"
        "frame_ms  %.2f  (second frame, includes readback)\n",
        ms);
    std::fclose(log);
    std::printf("frame_ms %.2f\nPASS collage\n", ms);
    return 0;
}
