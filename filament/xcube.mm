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


#include "xcube_2.h"
#include "xcube_3.h"
