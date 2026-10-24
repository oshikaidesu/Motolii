// A glass cube bends a sunbeam onto the floor.
// The cube's color is the path through the glass: enter, bounce inside, leave into this room.
// Filament draws the floor, the shadow, and the grade.
// The rainbow is the refracted beam, one patch per wavelength.
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
static const int TEX = 768;
static const int G = 84;
static const int NW = 21;
static const int NB = 3;
static const float PI = 3.14159265f;

struct V3 {
    float x, y, z;
    V3 operator+(V3 o) const { return {x + o.x, y + o.y, z + o.z}; }
    V3 operator-(V3 o) const { return {x - o.x, y - o.y, z - o.z}; }
    V3 operator*(float s) const { return {x * s, y * s, z * s}; }
};
static float dot(V3 a, V3 b) { return a.x * b.x + a.y * b.y + a.z * b.z; }
static V3 cross(V3 a, V3 b) { return {a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x}; }
static float len(V3 a) { return std::sqrt(dot(a, a)); }
static V3 norm(V3 a) {
    float L = len(a);
    return L > 1e-8f ? a * (1.f / L) : V3{0, 1, 0};
}

struct Cube {
    V3 c;
    V3 a[3];
    float h;
};

static void rotate(V3& v, int axis, float ang) {
    float c = std::cos(ang), s = std::sin(ang);
    float p[3] = {v.x, v.y, v.z};
    int i = (axis + 1) % 3, j = (axis + 2) % 3;
    float pi = p[i] * c - p[j] * s, pj = p[i] * s + p[j] * c;
    p[i] = pi;
    p[j] = pj;
    v = {p[0], p[1], p[2]};
}

static bool ray_cube(V3 ro, V3 rd, const Cube& cu, float& t_near, float& t_far, int& face_near, int& face_far) {
    float o[3], d[3];
    for (int i = 0; i < 3; ++i) {
        o[i] = dot(ro - cu.c, cu.a[i]);
        d[i] = dot(rd, cu.a[i]);
    }
    t_near = -1e9f;
    t_far = 1e9f;
    face_near = face_far = 0;
    for (int i = 0; i < 3; ++i) {
        if (std::fabs(d[i]) < 1e-8f) {
            if (o[i] < -cu.h || o[i] > cu.h) return false;
            continue;
        }
        float ta = (-cu.h - o[i]) / d[i];
        float tb = (cu.h - o[i]) / d[i];
        int fa = i * 2, fb = i * 2 + 1;
        if (ta > tb) {
            std::swap(ta, tb);
            std::swap(fa, fb);
        }
        if (ta > t_near) {
            t_near = ta;
            face_near = fa;
        }
        if (tb < t_far) {
            t_far = tb;
            face_far = fb;
        }
        if (t_near > t_far) return false;
    }
    return t_far > 0.f;
}

static V3 face_normal(const Cube& cu, int face) {
    return cu.a[face / 2] * ((face & 1) ? 1.f : -1.f);
}

static bool refract(V3 I, V3 N, float eta, V3& T) {
    float cosi = std::clamp(-dot(I, N), -1.f, 1.f);
    float k = 1.f - eta * eta * (1.f - cosi * cosi);
    if (k < 0.f) return false;
    T = norm(I * eta + N * (eta * cosi - std::sqrt(k)));
    return true;
}
static float fresnel(float cosi, float n1, float n2) {
    float r0 = (n1 - n2) / (n1 + n2);
    r0 *= r0;
    float c = std::clamp(1.f - cosi, 0.f, 1.f);
    return r0 + (1.f - r0) * c * c * c * c * c;
}
static V3 reflect(V3 I, V3 N) { return I - N * (2.f * dot(I, N)); }

static float n_of(float nm, float nd, float dispersion) {
    float offset = 0.70795215f + (nm - 486.1f) / (656.3f - 486.1f) * (-0.29204785f - 0.70795215f);
    float P = dispersion / 20.f;
    return std::max(1.05f, nd + P * (nd - 1.f) * offset);
}

static void wave_rgb(float nm, float c[3]) {
    float r = 0, g = 0, b = 0;
    if (nm < 440.f) {
        r = -(nm - 440.f) / (440.f - 380.f);
        b = 1.f;
    } else if (nm < 490.f) {
        g = (nm - 440.f) / (490.f - 440.f);
        b = 1.f;
    } else if (nm < 510.f) {
        g = 1.f;
        b = -(nm - 510.f) / (510.f - 490.f);
    } else if (nm < 580.f) {
        r = (nm - 510.f) / (580.f - 510.f);
        g = 1.f;
    } else if (nm < 645.f) {
        r = 1.f;
        g = -(nm - 645.f) / (645.f - 580.f);
    } else r = 1.f;
    float f = 1.f;
    if (nm < 420.f) f = 0.3f + 0.7f * (nm - 380.f) / 40.f;
    else if (nm > 700.f) f = 0.3f + 0.7f * (780.f - nm) / 80.f;
    c[0] = std::max(r, 0.f) * f;
    c[1] = std::max(g, 0.f) * f;
    c[2] = std::max(b, 0.f) * f;
}

struct Spot {
    float x, z, e;
    bool ok;
};

static const float FLOOR_HW = 1.55f;
static const float FLOOR_HH = 1.55f;

static void floor_uv(float x, float z, float& u, float& v) {
    u = (x / FLOOR_HW + 1.f) * 0.5f;
    v = (z / FLOOR_HH + 1.f) * 0.5f;
}

static bool to_floor(V3 o, V3 d, float e, Spot& s) {
    s.ok = false;
    if (d.y >= -1e-4f || e < 1e-4f) return false;
    float t = -o.y / d.y;
    if (t < 0.002f || t > 6.f) return false;
    V3 p = o + d * t;
    s.x = p.x;
    s.z = p.z;
    s.e = e;
    s.ok = true;
    return true;
}

static void trace_ray(V3 ro, V3 rd, const Cube& cu, float n, Spot out[NB]) {
    for (int i = 0; i < NB; ++i) out[i].ok = false;
    V3 o = ro, d = rd;
    bool inside = false;
    float e = 1.f;
    int nb = 0;
    for (int step = 0; step < 7 && e > 0.03f && nb < NB; ++step) {
        float t0, t1;
        int f0, f1;
        if (!ray_cube(o, d, cu, t0, t1, f0, f1)) {
            if (!inside) to_floor(o, d, 0.f, out[0]);
            return;
        }
        if (!inside) {
            if (t0 < 0.001f) return;
            V3 p = o + d * t0;
            V3 N = face_normal(cu, f0);
            if (dot(d, N) > 0.f) N = N * -1.f;
            float cosi = std::clamp(-dot(d, N), 0.f, 1.f);
            float F = fresnel(cosi, 1.f, n);
            V3 T;
            if (!refract(d, N, 1.f / n, T)) return;
            o = p + T * 2e-4f;
            d = T;
            e *= 1.f - F;
            inside = true;
            continue;
        }
        if (t1 < 0.001f) return;
        V3 p = o + d * t1;
        V3 Nout = face_normal(cu, f1);
        V3 Nin = dot(d, Nout) > 0.f ? Nout * -1.f : Nout;
        float cosi = std::clamp(-dot(d, Nin), 0.f, 1.f);
        float F = fresnel(cosi, n, 1.f);
        V3 Tout;
        if (refract(d, Nin, n, Tout)) {
            to_floor(p + Tout * 2e-4f, Tout, e * (1.f - F), out[nb]);
            if (out[nb].ok) ++nb;
            V3 R = reflect(d, Nout);
            o = p + R * 2e-4f;
            d = R;
            e *= F;
        } else {
            V3 R = reflect(d, Nout);
            o = p + R * 2e-4f;
            d = R;
        }
    }
}

static void fill_tri(std::vector<float>& tex, float u0, float v0, float u1, float v1, float u2, float v2, float r, float g,
                     float b) {
    auto X = [](float u) { return u * (TEX - 1); };
    auto Y = [](float v) { return v * (TEX - 1); };
    float x0 = X(u0), y0 = Y(v0), x1 = X(u1), y1 = Y(v1), x2 = X(u2), y2 = Y(v2);
    auto edge = [](float ax, float ay, float bx, float by, float px, float py) {
        return (px - ax) * (by - ay) - (py - ay) * (bx - ax);
    };
    float area = edge(x0, y0, x1, y1, x2, y2);
    if (std::fabs(area) < 0.2f) return;
    int minx = std::max(0, (int)std::floor(std::min({x0, x1, x2})));
    int maxx = std::min(TEX - 1, (int)std::ceil(std::max({x0, x1, x2})));
    int miny = std::max(0, (int)std::floor(std::min({y0, y1, y2})));
    int maxy = std::min(TEX - 1, (int)std::ceil(std::max({y0, y1, y2})));
    if (maxx - minx > 180 || maxy - miny > 180) return;
    for (int y = miny; y <= maxy; ++y) {
        for (int x = minx; x <= maxx; ++x) {
            float px = x + 0.5f, py = y + 0.5f;
            float w0 = edge(x1, y1, x2, y2, px, py) / area;
            float w1 = edge(x2, y2, x0, y0, px, py) / area;
            float w2 = edge(x0, y0, x1, y1, px, py) / area;
            if (w0 >= -0.001f && w1 >= -0.001f && w2 >= -0.001f) {
                float* p = tex.data() + ((size_t)y * TEX + x) * 4;
                p[0] += r;
                p[1] += g;
                p[2] += b;
            }
        }
    }
}

static float dist2(const Spot& a, const Spot& b) {
    float dx = a.x - b.x, dz = a.z - b.z;
    return dx * dx + dz * dz;
}

static void paint_cell(std::vector<float>& tex, const Spot& a, const Spot& b, const Spot& c, const Spot& d, float lux,
                       float cell, const float rgb[3]) {
    if (!a.ok || !b.ok || !c.ok || !d.ok) return;
    const float lim = 0.55f * 0.55f;
    if (dist2(a, b) > lim || dist2(a, c) > lim || dist2(b, d) > lim || dist2(c, d) > lim) return;
    auto uv = [](const Spot& s, float& u, float& v) { floor_uv(s.x, s.z, u, v); };
    float ua, va, ub, vb, uc, vc, ud, vd;
    uv(a, ua, va);
    uv(b, ub, vb);
    uv(c, uc, vc);
    uv(d, ud, vd);
    if (ua < -0.05f || ub < -0.05f || uc < -0.05f || ud < -0.05f) return;
    if (ua > 1.05f || ub > 1.05f || uc > 1.05f || ud > 1.05f) return;
    if (va < -0.05f || vb < -0.05f || vc < -0.05f || vd < -0.05f) return;
    if (va > 1.05f || vb > 1.05f || vc > 1.05f || vd > 1.05f) return;
    float flux = lux * cell * cell * (a.e + b.e + c.e + d.e) * 0.25f;
    auto area = [](const Spot& p, const Spot& q, const Spot& r) {
        return 0.5f * std::fabs((q.x - p.x) * (r.z - p.z) - (r.x - p.x) * (q.z - p.z));
    };
    float a0 = area(a, b, d);
    float a1 = area(a, d, c);
    if (a0 < 1e-8f || a1 < 1e-8f) return;
    float i0 = std::min(flux * 0.5f / a0, 50000.f);
    float i1 = std::min(flux * 0.5f / a1, 50000.f);
    fill_tri(tex, ua, va, ub, vb, ud, vd, rgb[0] * i0, rgb[1] * i0, rgb[2] * i0);
    fill_tri(tex, ua, va, ud, vd, uc, vc, rgb[0] * i1, rgb[1] * i1, rgb[2] * i1);
}

struct Stats {
    double ms = 0;
    float sum = 0, maxv = 0, chroma = 0;
    int cells = 0;
};

static Stats build(const Cube& cu, V3 sun, float lux, float nd, float dispersion, std::vector<float>& tex) {
    Stats st;
    tex.assign((size_t)TEX * TEX * 4, 0.f);
    auto t0 = std::chrono::steady_clock::now();
    float nm0 = 450.f, nm1 = 670.f;
    float rgb[NW][3];
    float sumc[3] = {};
    for (int w = 0; w < NW; ++w) {
        float nm = nm0 + (nm1 - nm0) * ((w + 0.5f) / NW);
        wave_rgb(nm, rgb[w]);
        sumc[0] += rgb[w][0];
        sumc[1] += rgb[w][1];
        sumc[2] += rgb[w][2];
    }
    for (int w = 0; w < NW; ++w)
        for (int k = 0; k < 3; ++k) rgb[w][k] = sumc[k] > 1e-6f ? rgb[w][k] / sumc[k] : 0.f;

    V3 D = norm(sun);
    V3 U = norm(cross(std::fabs(D.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0}, D));
    V3 V = cross(D, U);
    float extent = cu.h * 3.1f;
    float cell = (2.f * extent) / (float)(G - 1);
    std::vector<Spot> spots((size_t)NW * NB * G * G);
    auto at = [&](int w, int b, int j, int i) -> Spot& {
        return spots[(((size_t)w * NB + b) * G + j) * G + i];
    };
    for (int j = 0; j < G; ++j) {
        for (int i = 0; i < G; ++i) {
            float su = -extent + i * cell;
            float sv = -extent + j * cell;
            V3 ro = cu.c - D * 1.6f + U * su + V * sv;
            for (int w = 0; w < NW; ++w) {
                float nm = nm0 + (nm1 - nm0) * ((w + 0.5f) / NW);
                Spot hit[NB];
                trace_ray(ro, D, cu, n_of(nm, nd, dispersion), hit);
                for (int b = 0; b < NB; ++b) at(w, b, j, i) = hit[b];
            }
        }
    }
    int cells = 0;
    for (int w = 0; w < NW; ++w)
        for (int b = 0; b < NB; ++b)
            for (int j = 0; j < G - 1; ++j)
                for (int i = 0; i < G - 1; ++i) {
                    const Spot& a = at(w, b, j, i);
                    const Spot& c = at(w, b, j, i + 1);
                    const Spot& bpt = at(w, b, j + 1, i);
                    const Spot& d = at(w, b, j + 1, i + 1);
                    int before = 0;
                    (void)before;
                    if (a.ok && c.ok && bpt.ok && d.ok) ++cells;
                    paint_cell(tex, a, c, bpt, d, lux, cell, rgb[w]);
                }
    float sum = 0, maxv = 0, chroma = 0, lum = 0;
    for (float& v : tex) v = std::max(v, 0.f);
    for (int i = 0; i < TEX * TEX; ++i) {
        float* p = tex.data() + (size_t)i * 4;
        float m = std::max(p[0], std::max(p[1], p[2]));
        sum += m;
        maxv = std::max(maxv, m);
        if (m > 30.f) {
            chroma += std::fabs(p[0] - p[2]);
            lum += m;
        }
    }
    auto t1 = std::chrono::steady_clock::now();
    st.ms = std::chrono::duration<double, std::milli>(t1 - t0).count();
    st.sum = sum;
    st.maxv = maxv;
    st.chroma = lum > 1.f ? chroma / lum : 0.f;
    st.cells = cells;
    std::printf("beam cells %d sum %.3e max %.1f chroma %.3f  %.1f ms\n", cells, sum, maxv, st.chroma, st.ms);
    return st;
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
    ihdr[10] = ihdr[11] = ihdr[12] = 0;
    chunk("IHDR", ihdr, 13);
    chunk("IDAT", z.data(), (uint32_t)bound);
    chunk("IEND", nullptr, 0);
    std::fclose(f);
    return true;
}
static int fail(const char* m) {
    std::printf("FAIL %s\n", m);
    return 1;
}

static void push_vert(std::vector<float>& v, V3 p, V3 T, V3 N) {
    V3 n = N - T * dot(N, T);
    if (len(n) < 1e-5f) n = cross(std::fabs(T.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0}, T);
    n = norm(n);
    V3 B = norm(cross(n, T));
    quatf q = mat3f::packTangentFrame(mat3f{float3{T.x, T.y, T.z}, float3{B.x, B.y, B.z}, float3{n.x, n.y, n.z}});
    float f[7] = {p.x, p.y, p.z, q.x, q.y, q.z, q.w};
    v.insert(v.end(), f, f + 7);
}


#include "prism_2.h"
