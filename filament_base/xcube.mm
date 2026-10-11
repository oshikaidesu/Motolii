// A glass cube bends a sunbeam onto the floor.
// Filament draws the cube, the shadow, and the grade.
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

// Dichroic split, reimplemented from Optical_bench (MIT):
// https://github.com/gmike92/Optical_bench  optical_bench.jsx
// shortReflect: wavelength < cutoff reflects, otherwise the ray continues.
// longReflect is the opposite test in that same branch.
// Ray Optics (Apache-2.0) uses the same idea as a band on a mirror:
// https://github.com/ricktu288/ray-optics  src/core/sceneObjs/BaseFilter.js
//   checkRayIntersectFilter()
// v-prism has no license file, so none of its code is copied.
enum class CoatMode { Off, Hard, Soft };

struct Recipe {
    int reflA;
    int reflB;
    int exitAt;
};

static const Recipe kGlass[] = {{0, 0, 0}, {0, 0, 1}, {0, 0, 2}};
static const Recipe kX[] = {
    {0, 0, 0}, {0, 0, 1}, {0, 0, 2}, {1, 0, 0}, {0, 1, 0}, {1, 1, 0}, {1, 0, 1}, {0, 1, 1},
};

static float coat_R(int which, float nm, CoatMode mode) {
    if (mode == CoatMode::Off) return 0.f;
    const float cutoff = which == 0 ? 600.f : 490.f;
    const bool longR = which == 0;
    if (mode == CoatMode::Hard) return longR ? (nm >= cutoff ? 1.f : 0.f) : (nm < cutoff ? 1.f : 0.f);
    float s = 1.f / (1.f + std::exp(-(nm - cutoff) / 22.f));
    return longR ? s : (1.f - s);
}

static bool plane_t(V3 o, V3 d, V3 c, V3 n, float& t) {
    float den = dot(d, n);
    if (std::fabs(den) < 1e-6f) return false;
    t = dot(c - o, n) / den;
    return t > 1.2e-3f;
}

static bool in_cube(V3 p, const Cube& cu) {
    for (int i = 0; i < 3; ++i)
        if (std::fabs(dot(p - cu.c, cu.a[i])) > cu.h + 1e-3f) return false;
    return true;
}

static Spot trace_path(V3 ro, V3 rd, const Cube& cu, float n, float nm, CoatMode mode, Recipe rec) {
    Spot s{0, 0, 0, false};
    float t0, t1;
    int f0, f1;
    if (!ray_cube(ro, rd, cu, t0, t1, f0, f1) || t0 < 0.001f) return s;
    V3 p = ro + rd * t0;
    V3 N = face_normal(cu, f0);
    if (dot(rd, N) > 0.f) N = N * -1.f;
    float cosi = std::clamp(-dot(rd, N), 0.f, 1.f);
    V3 T;
    if (!refract(rd, N, 1.f / n, T)) return s;
    V3 o = p + T * 2e-4f;
    V3 d = T;
    float e = 1.f - fresnel(cosi, 1.f, n);
    if (mode != CoatMode::Off && rec.reflA == 0 && rec.reflB == 0) e *= 0.22f;
    int exits = 0;
    bool didA = false, didB = false;
    V3 nA = norm(cu.a[0] - cu.a[2]);
    V3 nB = norm(cu.a[0] + cu.a[2]);
    for (int step = 0; step < 8 && e > 0.02f; ++step) {
        if (!ray_cube(o, d, cu, t0, t1, f0, f1) || t1 < 1e-4f) return s;
        float tHit = t1;
        int kind = 0;
        V3 hn = face_normal(cu, f1);
        float tp;
        if (!didA && plane_t(o, d, cu.c, nA, tp) && tp < tHit && in_cube(o + d * tp, cu)) {
            tHit = tp;
            kind = 1;
            hn = nA;
        }
        if (!didB && plane_t(o, d, cu.c, nB, tp) && tp < tHit && in_cube(o + d * tp, cu)) {
            tHit = tp;
            kind = 2;
            hn = nB;
        }
        p = o + d * tHit;
        if (kind == 0) {
            V3 Nin = dot(d, hn) > 0.f ? hn * -1.f : hn;
            float c2 = std::clamp(-dot(d, Nin), 0.f, 1.f);
            float F2 = fresnel(c2, n, 1.f);
            V3 Tout;
            if (refract(d, Nin, n, Tout)) {
                if (exits == rec.exitAt) {
                    to_floor(p + Tout * 2e-4f, Tout, e * (1.f - F2), s);
                    return s;
                }
                ++exits;
                V3 R = reflect(d, hn);
                o = p + R * 2e-4f;
                d = R;
                e *= F2;
            } else {
                V3 R = reflect(d, hn);
                o = p + R * 2e-4f;
                d = R;
            }
        } else {
            if (dot(d, hn) > 0.f) hn = hn * -1.f;
            bool want = kind == 1 ? rec.reflA : rec.reflB;
            float Rcoat = coat_R(kind - 1, nm, mode);
            if (want) {
                e *= Rcoat * (mode == CoatMode::Soft ? 3.2f : 2.4f);
                V3 Rdir = reflect(d, hn);
                o = p + Rdir * 2e-4f;
                d = Rdir;
            } else {
                e *= 1.f - Rcoat;
                o = p + d * 2e-4f;
            }
            if (kind == 1) didA = true;
            else didB = true;
        }
    }
    return s;
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

static Stats build(const Cube& cu, V3 sun, float lux, float nd, float dispersion, CoatMode mode, std::vector<float>& tex) {
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
    const Recipe* recs = mode == CoatMode::Off ? kGlass : kX;
    const int npath = mode == CoatMode::Off ? 3 : 8;
    std::vector<Spot> spots((size_t)NW * npath * G * G);
    auto at = [&](int w, int b, int j, int i) -> Spot& {
        return spots[(((size_t)w * npath + b) * G + j) * G + i];
    };
    for (int j = 0; j < G; ++j) {
        for (int i = 0; i < G; ++i) {
            float su = -extent + i * cell;
            float sv = -extent + j * cell;
            V3 ro = cu.c - D * 1.6f + U * su + V * sv;
            for (int w = 0; w < NW; ++w) {
                float nm = nm0 + (nm1 - nm0) * ((w + 0.5f) / NW);
                float n = n_of(nm, nd, dispersion);
                for (int b = 0; b < npath; ++b) at(w, b, j, i) = trace_path(ro, D, cu, n, nm, mode, recs[b]);
            }
        }
    }
    int cells = 0;
    for (int w = 0; w < NW; ++w)
        for (int b = 0; b < npath; ++b)
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
    double sr = 0, sb = 0, xr = 0, xb = 0, yr = 0, yb = 0;
    for (int y = 0; y < TEX; ++y)
        for (int x = 0; x < TEX; ++x) {
            float* p = tex.data() + ((size_t)y * TEX + x) * 4;
            if (p[0] > p[2] + 80.f) {
                sr += p[0];
                xr += x * p[0];
                yr += y * p[0];
            }
            if (p[2] > p[0] + 80.f) {
                sb += p[2];
                xb += x * p[2];
                yb += y * p[2];
            }
        }
    std::printf("beam cells %d sum %.3e max %.1f chroma %.3f  %.1f ms\n", cells, sum, maxv, st.chroma, st.ms);
    if (sr > 1.0) std::printf("  red centroid uv %.3f %.3f\n", xr / sr / TEX, yr / sr / TEX);
    if (sb > 1.0) std::printf("  blue centroid uv %.3f %.3f\n", xb / sb / TEX, yb / sb / TEX);
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

int main(int argc, char** argv) {
    Cube cu;
    cu.h = 0.16f;
    cu.c = {0.f, cu.h + 0.001f, 0.f};
    cu.a[0] = {1, 0, 0};
    cu.a[1] = {0, 1, 0};
    cu.a[2] = {0, 0, 1};
    for (int i = 0; i < 3; ++i) {
        rotate(cu.a[i], 1, 0.62f);
    }
    for (int i = 0; i < 3; ++i) cu.a[i] = norm(cu.a[i]);
    const float nd = 1.55f;
    const float dispersion = 36.f;
    const float lux = 22000.f;
    V3 sun{0.92f, -0.42f, 0.22f};
    auto blur_map = [](std::vector<float>& tex) {
        for (float& v : tex) v = std::min(std::max(v, 0.f), 60000.f);
        std::vector<float> blurred(tex.size());
        for (int y = 0; y < TEX; ++y)
            for (int x = 0; x < TEX; ++x) {
                float r = 0, g = 0, b = 0, wsum = 0;
                for (int dy = -4; dy <= 4; ++dy)
                    for (int dx = -4; dx <= 4; ++dx) {
                        int xx = std::clamp(x + dx, 0, TEX - 1);
                        int yy = std::clamp(y + dy, 0, TEX - 1);
                        float w = std::exp(-(dx * dx + dy * dy) / 3.2f);
                        float* s = tex.data() + ((size_t)yy * TEX + xx) * 4;
                        r += s[0] * w;
                        g += s[1] * w;
                        b += s[2] * w;
                        wsum += w;
                    }
                float* d = blurred.data() + ((size_t)y * TEX + x) * 4;
                d[0] = r / wsum;
                d[1] = g / wsum;
                d[2] = b / wsum;
            }
        tex.swap(blurred);
    };
    std::vector<float> mapA, mapB, mapC, mapDraw;
    Stats sa = build(cu, sun, lux, nd, dispersion, CoatMode::Off, mapA);
    Stats sb = build(cu, sun, lux, nd, dispersion, CoatMode::Hard, mapB);
    Stats sc = build(cu, sun, lux, nd, dispersion, CoatMode::Soft, mapC);
    std::printf("A glass cells %d chroma %.3f  %.1f ms\n", sa.cells, sa.chroma, sa.ms);
    std::printf("B hard  cells %d chroma %.3f  %.1f ms\n", sb.cells, sb.chroma, sb.ms);
    std::printf("C soft  cells %d chroma %.3f  %.1f ms\n", sc.cells, sc.chroma, sc.ms);
    mapDraw = mapC;
    {
        auto encmap = [](const std::vector<float>& src, const char* path) {
            std::vector<uint8_t> pic((size_t)TEX * TEX * 4);
            for (int i = 0; i < TEX * TEX; ++i) {
                const float* p = src.data() + (size_t)i * 4;
                auto enc = [](float v) {
                    float x = v / (v + 3500.f);
                    return (uint8_t)std::clamp(int(std::pow(std::max(x, 0.f), 0.85f) * 255.f), 0, 255);
                };
                pic[(size_t)i * 4 + 0] = enc(p[0]);
                pic[(size_t)i * 4 + 1] = enc(p[1]);
                pic[(size_t)i * 4 + 2] = enc(p[2]);
                pic[(size_t)i * 4 + 3] = 255;
            }
            write_png(path, TEX, TEX, pic.data());
        };
        encmap(mapB, "out/xcube-b-map.png");
        encmap(mapC, "out/xcube-c-map.png");
    }
    blur_map(mapA);
    blur_map(mapB);
    blur_map(mapC);
    std::vector<float>& tex = mapA;

    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    Scene* scene = engine->createScene();
    View* view = engine->createView();
    SwapChain* swap = engine->createSwapChain((uint32_t)W, (uint32_t)H, 0);
    utils::Entity cam_e = utils::EntityManager::get().create();
    Camera* camera = engine->createCamera(cam_e);
    camera->setExposure(2.8f, 1.0f / 125.0f, 100.0f);
    camera->lookAt({0.55f, 0.78f, 0.70f}, {0.0f, 0.04f, 0.0f}, {0, 1, 0});
    camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);

    Texture* color = Texture::Builder()
                         .width(W).height(H).levels(1)
                         .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                         .format(Texture::InternalFormat::RGBA8).build(*engine);
    Texture* depth = Texture::Builder()
                         .width(W).height(H).levels(1)
                         .usage(Texture::Usage::DEPTH_ATTACHMENT)
                         .format(Texture::InternalFormat::DEPTH24).build(*engine);
    RenderTarget* rt = RenderTarget::Builder()
                           .texture(RenderTarget::AttachmentPoint::COLOR, color)
                           .texture(RenderTarget::AttachmentPoint::DEPTH, depth).build(*engine);
    Texture* cau = Texture::Builder()
                       .width(TEX).height(TEX).levels(1)
                       .sampler(Texture::Sampler::SAMPLER_2D)
                       .format(Texture::InternalFormat::RGBA16F).build(*engine);
    cau->setImage(*engine, 0,
                  Texture::PixelBufferDescriptor(tex.data(), tex.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                 backend::PixelDataType::FLOAT, 1, 0, 0, TEX));

    scene->setSkybox(Skybox::Builder().color({0.72f, 0.74f, 0.78f, 1}).build(*engine));
    static AgxToneMapper agx(AgxToneMapper::AgxLook::PUNCHY);
    view->setColorGrading(ColorGrading::Builder().toneMapper(&agx).contrast(1.04f).saturation(1.08f).build(*engine));
    view->setScene(scene);
    view->setCamera(camera);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setDithering(View::Dithering::NONE);
    view->setAntiAliasing(View::AntiAliasing::FXAA);
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    view->setShadowType(View::ShadowType::VSM);
    VsmShadowOptions vsm;
    vsm.highPrecision = true;
    view->setVsmShadowOptions(vsm);
    BloomOptions bloom{};
    bloom.enabled = true;
    bloom.strength = 0.18f;
    bloom.levels = 6;
    bloom.resolution = 1080;
    bloom.threshold = true;
    bloom.highlight = 10.f;
    bloom.blendMode = BloomOptions::BlendMode::ADD;
    view->setBloomOptions(bloom);

    auto gpkg = load_file("glass_shadow.filamat");
    auto fpkg = load_file("floor_recv.filamat");
    auto epkg = load_file("emissive.filamat");
    if (gpkg.empty() || fpkg.empty() || epkg.empty()) return fail("filamat");
    Material* gmat = Material::Builder().package(gpkg.data(), gpkg.size()).build(*engine);
    Material* fmat = Material::Builder().package(fpkg.data(), fpkg.size()).build(*engine);
    Material* emat = Material::Builder().package(epkg.data(), epkg.size()).build(*engine);
    if (!gmat || !fmat || !emat) return fail("material");
    TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    lin.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    lin.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* floor_mi = fmat->createInstance();
    floor_mi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    floor_mi->setParameter("roughness", 0.42f);
    floor_mi->setParameter("caustic", cau, lin);
    MaterialInstance* glass = gmat->createInstance();
    glass->setParameter("ior", nd);
    glass->setParameter("transmission", 1.f);
    glass->setParameter("thickness", cu.h * 2.f);
    glass->setParameter("dispersion", dispersion);
    glass->setParameter("reflectance", 0.5f);
    glass->setParameter("alpha", 1.f);
    glass->setParameter("absorption", float3{0.02f, 0.015f, 0.02f});
    MaterialInstance* panel = emat->createInstance();
    panel->setParameter("color", float4{2200.f, 2300.f, 2600.f, 1});

    std::vector<float> floor_v;
    std::vector<uint32_t> floor_i = {0, 2, 1, 0, 3, 2};
    {
        V3 center{0, 0, 0}, right{1, 0, 0}, up{0, 0, 1};
        V3 n{0, 1, 0};
        V3 p[4] = {center - right * FLOOR_HW - up * FLOOR_HH, center + right * FLOOR_HW - up * FLOOR_HH,
                   center + right * FLOOR_HW + up * FLOOR_HH, center - right * FLOOR_HW + up * FLOOR_HH};
        float uv[4][2] = {{0, 0}, {1, 0}, {1, 1}, {0, 1}};
        quatf q = mat3f::packTangentFrame(mat3f{float3{1, 0, 0}, float3{0, 0, -1}, float3{0, 1, 0}});
        floor_v.resize(4 * 9);
        for (int i = 0; i < 4; ++i) {
            float* o = floor_v.data() + i * 9;
            o[0] = p[i].x;
            o[1] = p[i].y;
            o[2] = p[i].z;
            o[3] = uv[i][0];
            o[4] = uv[i][1];
            o[5] = q.x;
            o[6] = q.y;
            o[7] = q.z;
            o[8] = q.w;
            (void)n;
        }
    }
    std::vector<float> cube_v;
    std::vector<uint32_t> cube_i;
    for (int face = 0; face < 6; ++face) {
        V3 N = face_normal(cu, face);
        V3 T = cu.a[(face / 2 + 1) % 3];
        V3 B = cross(N, T);
        uint32_t base = (uint32_t)(cube_v.size() / 7);
        V3 c = cu.c + N * cu.h;
        push_vert(cube_v, c - T * cu.h - B * cu.h, T, N);
        push_vert(cube_v, c + T * cu.h - B * cu.h, T, N);
        push_vert(cube_v, c + T * cu.h + B * cu.h, T, N);
        push_vert(cube_v, c - T * cu.h + B * cu.h, T, N);
        cube_i.insert(cube_i.end(), {base, base + 1, base + 2, base, base + 2, base + 3});
    }
    std::vector<float> panel_v;
    std::vector<uint32_t> panel_i = {0, 1, 2, 0, 2, 3};
    {
        V3 c{0.0f, 0.55f, -1.15f};
        V3 right{1, 0, 0}, upn{0, 1, 0};
        float hw = 1.6f, hh = 0.7f;
        V3 p[4] = {c - right * hw - upn * hh, c + right * hw - upn * hh, c + right * hw + upn * hh, c - right * hw + upn * hh};
        for (int i = 0; i < 4; ++i) push_vert(panel_v, p[i], right, {0, 0, 1});
    }

    auto add = [&](std::vector<float>& verts, std::vector<uint32_t>& idx, int stride, bool uv, MaterialInstance* mi, bool cast_s,
                   bool recv) {
        utils::Entity e = utils::EntityManager::get().create();
        uint32_t nv = uv ? 4u : (uint32_t)(verts.size() / 7);
        auto b = VertexBuffer::Builder().vertexCount(nv).bufferCount(1);
        b.attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, (uint32_t)stride);
        if (uv) {
            b.attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, (uint32_t)stride);
            b.attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, (uint32_t)stride);
        } else b.attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 12, (uint32_t)stride);
        VertexBuffer* vb = b.build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(verts.data(), verts.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount((uint32_t)idx.size()).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(idx.data(), idx.size() * sizeof(uint32_t)));
        RenderableManager::Builder(1)
            .boundingBox({{-4, -4, -4}, {4, 4, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, mi).culling(false).castShadows(cast_s).receiveShadows(recv)
            .build(*engine, e);
        scene->addEntity(e);
        return e;
    };
    utils::Entity floorE = add(floor_v, floor_i, 36, true, floor_mi, false, true);
    utils::Entity cubeE = add(cube_v, cube_i, 28, false, glass, true, false);
    utils::Entity panelE = add(panel_v, panel_i, 28, false, panel, false, false);
    auto film_quad = [&](V3 nrm, std::vector<float>& v, std::vector<uint32_t>& idx) {
        V3 U = cu.a[1];
        V3 Vp = norm(cross(nrm, U));
        float s = cu.h * 0.92f;
        V3 q[4] = {cu.c - U * s - Vp * s, cu.c + U * s - Vp * s, cu.c + U * s + Vp * s, cu.c - U * s + Vp * s};
        for (int i = 0; i < 4; ++i) push_vert(v, q[i], Vp, nrm);
        idx = {0, 1, 2, 0, 2, 3};
    };
    std::vector<float> filmAv, filmBv;
    std::vector<uint32_t> filmAi, filmBi;
    film_quad(norm(cu.a[0] - cu.a[2]), filmAv, filmAi);
    film_quad(norm(cu.a[0] + cu.a[2]), filmBv, filmBi);
    MaterialInstance* filmA = emat->createInstance();
    MaterialInstance* filmB = emat->createInstance();
    filmA->setParameter("color", float4{1400.f, 90.f, 40.f, 1});
    filmB->setParameter("color", float4{40.f, 160.f, 1600.f, 1});
    utils::Entity filmAe = add(filmAv, filmAi, 28, false, filmA, false, false);
    utils::Entity filmBe = add(filmBv, filmBi, 28, false, filmB, false, false);
    scene->remove(filmAe);
    scene->remove(filmBe);

    utils::Entity sun_e = utils::EntityManager::get().create();
    LightManager::ShadowOptions shopt;
    shopt.mapSize = 2048;
    shopt.vsm.blurWidth = 8.f;
    shopt.vsm.elvsm = true;
    LightManager::Builder(LightManager::Type::SUN)
        .color(float3{1.f, 0.98f, 0.96f})
        .intensity(lux)
        .direction(normalize(float3{sun.x, sun.y, sun.z}))
        .sunAngularRadius(0.35f)
        .sunHaloSize(1.2f)
        .castShadows(true)
        .shadowOptions(shopt)
        .build(*engine, sun_e);
    scene->addEntity(sun_e);

    auto upload = [&](std::vector<float>& src) {
        cau->setImage(*engine, 0,
                      Texture::PixelBufferDescriptor(src.data(), src.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                     backend::PixelDataType::FLOAT, 1, 0, 0, TEX));
        engine->flushAndWait();
    };
    auto shot = [&](const char* path) -> bool {
        std::vector<uint8_t> pic((size_t)W * H * 4);
        bool opened = false;
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) {
                opened = true;
                break;
            }
        if (!opened) return false;
        view->setRenderTarget(rt);
        renderer->render(view);
        renderer->readPixels(rt, 0, 0, W, H,
                             backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                            backend::PixelDataType::UBYTE));
        renderer->endFrame();
        engine->flushAndWait();
        return write_png(path, W, H, pic.data());
    };
    if (!shot("out/xcube-a.png")) return fail("a");
    upload(mapB);
    if (!shot("out/xcube-b.png")) return fail("b");
    upload(mapC);
    if (!shot("out/xcube-c.png")) return fail("c");

    auto apkg = load_file("caustic_atrous.filamat");
    if (apkg.empty()) return fail("atrous");
    Material* atrousMat = Material::Builder().package(apkg.data(), apkg.size()).build(*engine);
    if (!atrousMat) return fail("atrous-mat");
    Texture* srcTex = Texture::Builder().width(TEX).height(TEX).levels(1).format(Texture::InternalFormat::RGBA16F).build(*engine);
    auto usage = Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC | Texture::Usage::BLIT_DST |
                 Texture::Usage::GEN_MIPMAPPABLE;
    Texture* tempTex = Texture::Builder().width(TEX).height(TEX).levels(10).usage(usage).format(Texture::InternalFormat::RGBA16F).build(*engine);
    Texture* tempDepth = Texture::Builder().width(TEX).height(TEX).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH32F).build(*engine);
    RenderTarget* tempRT = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, tempTex)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, tempDepth).build(*engine);
    Texture* filtTex = Texture::Builder().width(TEX).height(TEX).levels(1)
                           .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::SAMPLEABLE | Texture::Usage::BLIT_SRC)
                           .format(Texture::InternalFormat::RGBA16F).build(*engine);
    Texture* filtDepth = Texture::Builder().width(TEX).height(TEX).levels(1).usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH32F).build(*engine);
    RenderTarget* filtRT = RenderTarget::Builder().texture(RenderTarget::AttachmentPoint::COLOR, filtTex)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, filtDepth).build(*engine);
    TextureSampler mipped(TextureSampler::MinFilter::LINEAR_MIPMAP_LINEAR, TextureSampler::MagFilter::LINEAR);
    mipped.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    mipped.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* atrous1 = atrousMat->createInstance();
    atrous1->setParameter("src", srcTex, lin);
    atrous1->setParameter("texel", float2{1.f / TEX, 1.f / TEX});
    atrous1->setParameter("stride", 1.f);
    atrous1->setParameter("lumWeight", 0.5f);
    atrous1->setParameter("mip", 0.f);
    atrous1->setParameter("second", 0.f);
    MaterialInstance* atrous2 = atrousMat->createInstance();
    atrous2->setParameter("src", tempTex, mipped);
    atrous2->setParameter("texel", float2{1.f / TEX, 1.f / TEX});
    atrous2->setParameter("stride", 2.f);
    atrous2->setParameter("lumWeight", 0.5f);
    atrous2->setParameter("mip", 1.f);
    atrous2->setParameter("second", 1.f);
    utils::Entity filterE = utils::EntityManager::get().create();
    {
        auto b = VertexBuffer::Builder().vertexCount(4).bufferCount(1);
        b.attribute(VertexAttribute::POSITION, 0, VertexBuffer::AttributeType::FLOAT3, 0, 36);
        b.attribute(VertexAttribute::UV0, 0, VertexBuffer::AttributeType::FLOAT2, 12, 36);
        b.attribute(VertexAttribute::TANGENTS, 0, VertexBuffer::AttributeType::FLOAT4, 20, 36);
        VertexBuffer* vb = b.build(*engine);
        vb->setBufferAt(*engine, 0, VertexBuffer::BufferDescriptor(floor_v.data(), floor_v.size() * sizeof(float)));
        IndexBuffer* ib = IndexBuffer::Builder().indexCount(6).bufferType(IndexBuffer::IndexType::UINT).build(*engine);
        ib->setBuffer(*engine, IndexBuffer::BufferDescriptor(floor_i.data(), floor_i.size() * sizeof(uint32_t)));
        RenderableManager::Builder(1).boundingBox({{-4, -4, -4}, {4, 4, 4}})
            .geometry(0, RenderableManager::PrimitiveType::TRIANGLES, vb, ib)
            .material(0, atrous1).culling(false).castShadows(false).receiveShadows(false)
            .build(*engine, filterE);
    }
    srcTex->setImage(*engine, 0,
                     Texture::PixelBufferDescriptor(mapDraw.data(), mapDraw.size() * sizeof(float), backend::PixelDataFormat::RGBA,
                                                    backend::PixelDataType::FLOAT, 1, 0, 0, TEX));
    engine->flushAndWait();
    Skybox* sky = scene->getSkybox();
    scene->setSkybox(nullptr);
    scene->remove(floorE);
    scene->remove(cubeE);
    scene->remove(panelE);
    scene->remove(sun_e);
    scene->addEntity(filterE);
    view->setPostProcessingEnabled(false);
    view->setScreenSpaceRefractionEnabled(false);
    view->setBloomOptions(BloomOptions{});
    camera->setExposure(1.f);
    camera->lookAt({0, 4, 0}, {0, 0, 0}, {0, 0, -1});
    camera->setProjection(Camera::Projection::ORTHO, -1.55, 1.55, -1.55, 1.55, 0.01, 10.0);
    view->setViewport({0, 0, (uint32_t)TEX, (uint32_t)TEX});
    Renderer::ClearOptions clearOpt;
    clearOpt.clear = true;
    clearOpt.discard = true;
    clearOpt.clearColor = {0, 0, 0, 0};
    renderer->setClearOptions(clearOpt);
    auto pass = [&](RenderTarget* target, MaterialInstance* mi) -> bool {
        engine->getRenderableManager().setMaterialInstanceAt(engine->getRenderableManager().getInstance(filterE), 0, mi);
        bool opened = false;
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) {
                opened = true;
                break;
            }
        if (!opened) return false;
        view->setRenderTarget(target);
        renderer->render(view);
        renderer->endFrame();
        return true;
    };
    if (!pass(tempRT, atrous1)) return fail("atrous1");
    tempTex->generateMipmaps(*engine);
    engine->flushAndWait();
    if (!pass(filtRT, atrous2)) return fail("atrous2");
    engine->flushAndWait();
    scene->remove(filterE);
    scene->addEntity(floorE);
    scene->addEntity(cubeE);
    scene->addEntity(panelE);
    scene->addEntity(sun_e);
    scene->setSkybox(sky);
    floor_mi->setParameter("caustic", filtTex, lin);
    view->setPostProcessingEnabled(true);
    view->setScreenSpaceRefractionEnabled(true);
    view->setBloomOptions(bloom);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    camera->setExposure(2.8f, 1.f / 125.f, 100.f);
    camera->lookAt({0.55f, 0.78f, 0.70f}, {0.f, 0.04f, 0.f}, {0, 1, 0});
    camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
    if (!shot("out/xcube-d.png")) return fail("d");
    std::printf("PASS xcube-cpu\n");
    return 0;
}
