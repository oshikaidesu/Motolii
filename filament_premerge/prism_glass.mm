// prism.mm plus one closed glass 3D path (the "3dpath" of beauty.mm) floating around the cube.
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

#include "caustic_tri.h"

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
#include <functional>
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

// ---- the glass path: copied from beauty.mm, one closed 3D path ----
static const float TAU = PI * 2.f;
struct Cubic {
    V3 p[4];
};
struct Sample {
    V3 p, T, N, B;
    float s = 0;
};
struct Built {
    std::vector<float> v;
    std::vector<uint32_t> idx;
};

static V3 bezier(const Cubic& c, float t) {
    float u = 1.f - t;
    return c.p[0] * (u * u * u) + c.p[1] * (3.f * u * u * t) + c.p[2] * (3.f * u * t * t) + c.p[3] * (t * t * t);
}
static V3 bezier_d(const Cubic& c, float t) {
    float u = 1.f - t;
    return (c.p[1] - c.p[0]) * (3.f * u * u) + (c.p[2] - c.p[1]) * (6.f * u * t) + (c.p[3] - c.p[2]) * (3.f * t * t);
}
static V3 rot_about(V3 v, V3 T, float ang) {
    return v * std::cos(ang) + cross(T, v) * std::sin(ang);
}
static V3 limit_tangent(const Cubic& c, float t) {
    V3 d = bezier_d(c, t);
    if (len(d) >= 1e-4f) return d;
    float tin = std::clamp(t + (t < 0.5f ? 1e-3f : -1e-3f), 0.f, 1.f);
    V3 side = bezier_d(c, tin);
    if (len(side) >= 1e-4f) return side;
    V3 chord = c.p[3] - c.p[0];
    return len(chord) >= 1e-4f ? chord : V3{1, 0, 0};
}
static V3 transport_n(V3 n, V3 t_from, V3 t_to) {
    V3 axis = cross(t_from, t_to);
    float axis_len = len(axis);
    if (axis_len < 1e-8f) {
        if (dot(t_from, t_to) < 0.f) n = cross(t_to, cross(t_from, n));
    } else {
        float ang = std::atan2(axis_len, std::clamp(dot(t_from, t_to), -1.f, 1.f));
        n = rot_about(n, norm(axis), ang);
    }
    return n - t_to * dot(n, t_to);
}
static float signed_about(V3 T, V3 from, V3 to) {
    return std::atan2(dot(T, cross(from, to)), std::clamp(dot(from, to), -1.f, 1.f));
}

static std::vector<Cubic> catmull(const std::vector<V3>& p) {
    std::vector<Cubic> out;
    int n = (int)p.size();
    auto at = [&](int i) { return p[(i % n + n) % n]; };
    for (int i = 0; i < n; ++i) {
        V3 p0 = at(i), p3 = at(i + 1);
        V3 p1 = p0 + (at(i + 1) - at(i - 1)) * (1.f / 6.f);
        V3 p2 = p3 - (at(i + 2) - at(i)) * (1.f / 6.f);
        out.push_back(Cubic{p0, p1, p2, p3});
    }
    return out;
}

static std::vector<Cubic> the_path() {
    return catmull({{-0.78f, -0.22f, 0.12f},
                    {-0.22f, -0.48f, 0.38f},
                    {0.55f, -0.12f, 0.22f},
                    {0.42f, 0.46f, -0.22f},
                    {-0.18f, 0.38f, -0.48f},
                    {-0.68f, 0.08f, -0.12f}});
}

static void push_sample(std::vector<Sample>& out, const Cubic& c, float t) {
    Sample s;
    bool has = !out.empty();
    Sample prev = has ? out.back() : Sample{};
    s.p = bezier(c, t);
    s.T = norm(limit_tangent(c, t));
    if (len(s.T) < 0.5f) s.T = has ? prev.T : V3{1, 0, 0};
    V3 up = std::fabs(s.T.z) < 0.9f ? V3{0, 0, 1} : V3{0, 1, 0};
    V3 n = has ? prev.N - s.T * dot(prev.N, s.T) : cross(up, s.T);
    if (len(n) < 1e-4f) {
        V3 axis = cross(has ? prev.T : up, s.T);
        float axis_len = len(axis);
        if (!has || axis_len < 1e-5f) n = cross(up, s.T);
        else n = rot_about(prev.N, norm(axis), std::atan2(axis_len, std::clamp(dot(prev.T, s.T), -1.f, 1.f)));
        n = n - s.T * dot(n, s.T);
    }
    s.N = norm(n);
    s.B = norm(cross(s.T, s.N));
    s.s = has ? prev.s + len(s.p - prev.p) : 0;
    out.push_back(s);
}

static std::vector<Sample> sample_uniform(const std::vector<Cubic>& spans, int per) {
    std::vector<Sample> out;
    for (const Cubic& c : spans)
        for (int i = 0; i < per; ++i) push_sample(out, c, (float)i / (float)per);
    return out;
}

static std::vector<Sample> close_frames(const std::vector<Cubic>& spans, int per) {
    std::vector<Sample> in = sample_uniform(spans, per);
    float alpha = 0;
    {
        V3 n = transport_n(in.back().N, in.back().T, in.front().T);
        alpha = signed_about(in.front().T, in.front().N, norm(n));
    }
    float L = in.back().s + len(in.front().p - in.back().p);
    for (Sample& a : in) {
        a.N = norm(rot_about(a.N, a.T, -alpha * a.s / L));
        a.B = norm(cross(a.T, a.N));
    }
    return in;
}

static void pack(std::vector<float>& v, V3 p, V3 T, V3 normal) {
    V3 N = normal - T * dot(normal, T);
    if (len(N) < 1e-5f) {
        V3 up = std::fabs(T.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0};
        N = cross(up, T);
    }
    N = norm(N);
    V3 B = norm(cross(T, N));
    quatf q = mat3f::packTangentFrame(mat3f{float3{T.x, T.y, T.z}, float3{B.x, B.y, B.z}, float3{N.x, N.y, N.z}});
    float packf[7] = {p.x, p.y, p.z, q.x, q.y, q.z, q.w};
    v.insert(v.end(), packf, packf + 7);
}

static float closed_len(const std::vector<Sample>& s) {
    return s.back().s + len(s.front().p - s.back().p);
}

static Built tube(const std::vector<Sample>& s, int sides, const std::function<float(float)>& radius) {
    Built m;
    int rings = (int)s.size();
    float L = closed_len(s);
    for (int i = 0; i < rings; ++i) {
        float u = s[i].s / L;
        float r = radius(u);
        for (int k = 0; k < sides; ++k) {
            float ang = (float)k / (float)sides * TAU;
            V3 radial = norm(s[i].N * std::cos(ang) + s[i].B * std::sin(ang));
            pack(m.v, s[i].p + radial * r, s[i].T, radial);
        }
    }
    for (int i = 0; i < rings; ++i) {
        int i1 = (i + 1) % rings;
        for (int k = 0; k < sides; ++k) {
            int k1 = (k + 1) % sides;
            uint32_t a = (uint32_t)(i * sides + k);
            uint32_t b = (uint32_t)(i * sides + k1);
            uint32_t c = (uint32_t)(i1 * sides + k);
            uint32_t d = (uint32_t)(i1 * sides + k1);
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
    ihdr[10] = ihdr[11] = ihdr[12] = 0;
    chunk("IHDR", ihdr, 13);
    chunk("IDAT", z.data(), (uint32_t)bound);
    chunk("IEND", nullptr, 0);
    std::fclose(f);
    return true;
}
static float env_f(const char* k, float d) {
    const char* v = std::getenv(k);
    return v ? (float)std::atof(v) : d;
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
    std::vector<float> tex, plain;
    Stats full = build(cu, sun, lux, nd, dispersion, tex);
    Stats mono = build(cu, sun, lux, nd, 0.f, plain);
    std::printf("CHECK dispersed chroma %.3f   same-ior chroma %.3f\n", full.chroma, mono.chroma);
    std::printf("CHECK max %.1f (RGBA16F holds this; upload clamps only above 60000)\n", full.maxv);
    if (argc > 1 && std::string(argv[1]) == "dump") {
        auto save = [](const char* path, const std::vector<float>& src) {
            FILE* f = std::fopen(path, "wb");
            if (!f) return false;
            uint32_t n = (uint32_t)TEX;
            std::fwrite(&n, 4, 1, f);
            std::fwrite(src.data(), sizeof(float), src.size(), f);
            std::fclose(f);
            return true;
        };
        if (!save("out/prism-spec.f32", tex) || !save("out/prism-mono.f32", plain)) return fail("dump");
        std::printf("dumped pre-blur spec and mono %d\n", TEX);
        return 0;
    }

    std::vector<uint8_t> map((size_t)TEX * TEX * 4);
    for (int i = 0; i < TEX * TEX; ++i) {
        float* p = tex.data() + (size_t)i * 4;
        auto enc = [](float v) {
            float x = v / (v + 3500.f);
            return (uint8_t)std::clamp(int(std::pow(x, 0.85f) * 255.f), 0, 255);
        };
        map[(size_t)i * 4 + 0] = enc(p[0]);
        map[(size_t)i * 4 + 1] = enc(p[1]);
        map[(size_t)i * 4 + 2] = enc(p[2]);
        map[(size_t)i * 4 + 3] = 255;
    }
    write_png("out/prism-map.png", TEX, TEX, map.data());

    for (float& v : tex) v = std::min(v, 60000.f);
    {
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
    }

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

    auto fpkg = load_file("floor_glass.filamat");
    auto epkg = load_file("emissive.filamat");
    auto ppkg = load_file("xcube_drei.filamat");
    if (fpkg.empty() || epkg.empty() || ppkg.empty()) return fail("filamat");
    Material* fmat = Material::Builder().package(fpkg.data(), fpkg.size()).build(*engine);
    Material* emat = Material::Builder().package(epkg.data(), epkg.size()).build(*engine);
    Material* pmat = Material::Builder().package(ppkg.data(), ppkg.size()).build(*engine);
    if (!fmat || !emat || !pmat) return fail("material");
    TextureSampler lin(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR);
    lin.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    lin.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    MaterialInstance* floor_mi = fmat->createInstance();
    floor_mi->setParameter("color", float4{0.86f, 0.85f, 0.84f, 1});
    floor_mi->setParameter("roughness", 0.42f);
    floor_mi->setParameter("caustic", cau, lin);
    floor_mi->setParameter("tubeBlur", env_f("TUBE_BLUR", 1.6f) / 1024.f);
    floor_mi->setParameter("tubeScale", lux * std::fabs(normalize(float3{sun.x, sun.y, sun.z}).y));
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

    // The prism: not a cube any more but a chamfered gem. Same box for the sun / floor caustic, new look.
    std::vector<float> gem_v;
    std::vector<uint32_t> gem_i;
    {
        const float h = cu.h, b = env_f("GEM_BEVEL", 0.f), hb = h - b;
        auto W = [&](const float c[3]) { return cu.c + cu.a[0] * c[0] + cu.a[1] * c[1] + cu.a[2] * c[2]; };
        auto tri = [&](V3 p0, V3 p1, V3 p2, V3 N) {
            if (dot(cross(p1 - p0, p2 - p0), N) < 0) std::swap(p1, p2);
            V3 T = norm(p1 - p0);
            uint32_t base = (uint32_t)(gem_v.size() / 7);
            push_vert(gem_v, p0, T, N);
            push_vert(gem_v, p1, T, N);
            push_vert(gem_v, p2, T, N);
            gem_i.insert(gem_i.end(), {base, base + 1, base + 2});
        };
        auto quad = [&](V3 a, V3 bq, V3 c, V3 d, V3 N) {
            tri(a, bq, c, N);
            tri(a, c, d, N);
        };
        const float sg[2] = {-1.f, 1.f};
        for (int k = 0; k < 3; ++k)
            for (float s : sg) {  // six flat faces
                int i = (k + 1) % 3, j = (k + 2) % 3;
                float c[4][3];
                const float uu[4] = {-1, 1, 1, -1}, vv[4] = {-1, -1, 1, 1};
                for (int q = 0; q < 4; ++q) {
                    c[q][k] = s * h;
                    c[q][i] = uu[q] * hb;
                    c[q][j] = vv[q] * hb;
                }
                quad(W(c[0]), W(c[1]), W(c[2]), W(c[3]), cu.a[k] * s);
            }
        if (b > 1e-6f)
        for (int i = 0; i < 3; ++i)
            for (int j = i + 1; j < 3; ++j) {
                int l = 3 - i - j;
                for (float si : sg)
                    for (float sj : sg) {  // twelve bevel strips
                        float p[4][3];
                        for (int q = 0; q < 4; ++q) {
                            bool first = (q == 0 || q == 3);
                            p[q][i] = si * (first ? h : hb);
                            p[q][j] = sj * (first ? hb : h);
                            p[q][l] = (q < 2) ? -hb : hb;
                        }
                        quad(W(p[0]), W(p[1]), W(p[2]), W(p[3]), norm(cu.a[i] * si + cu.a[j] * sj));
                    }
            }
        if (b > 1e-6f)
        for (float sx : sg)
            for (float sy : sg)
                for (float sz : sg) {  // eight corner triangles
                    float p0[3] = {sx * h, sy * hb, sz * hb}, p1[3] = {sx * hb, sy * h, sz * hb}, p2[3] = {sx * hb, sy * hb, sz * h};
                    tri(W(p0), W(p1), W(p2), norm(cu.a[0] * sx + cu.a[1] * sy + cu.a[2] * sz));
                }
    }
    auto gempkg = load_file("gem.filamat");
    if (gempkg.empty()) return fail("gem filamat");
    Material* gemmat = Material::Builder().package(gempkg.data(), gempkg.size()).build(*engine);
    if (!gemmat) return fail("gem material");
    MaterialInstance* gem_mi = gemmat->createInstance();
    gem_mi->setParameter("ior", env_f("GEM_IOR", 2.2f));
    gem_mi->setParameter("transmission", env_f("GEM_TRANS", 1.0f));
    gem_mi->setParameter("thickness", 0.32f);
    gem_mi->setParameter("dispersion", env_f("GEM_DISPERSION", 1.6f));
    gem_mi->setParameter("fire", env_f("GEM_FIRE", 2500.f));
    gem_mi->setParameter("film", env_f("GEM_FILM", 0.7f));
    gem_mi->setParameter("tint", env_f("GEM_TINT", 3.2f));
    gem_mi->setParameter("glint", env_f("GEM_GLINT", 14000.f));
    gem_mi->setParameter("cells", env_f("GEM_CELLS", 1.f));
    std::vector<float> panel_v;
    std::vector<uint32_t> panel_i = {0, 1, 2, 0, 2, 3};
    {
        V3 c{0.0f, 0.55f, -1.15f};
        V3 right{1, 0, 0}, upn{0, 1, 0};
        float hw = 1.6f, hh = 0.7f;
        V3 p[4] = {c - right * hw - upn * hh, c + right * hw - upn * hh, c + right * hw + upn * hh, c - right * hw + upn * hh};
        for (int i = 0; i < 4; ++i) push_vert(panel_v, p[i], right, {0, 0, 1});
    }

    std::vector<float> triPos;
    triPos.reserve(cube_i.size() * 4);
    for (uint32_t idx : cube_i) {
        const float* p = cube_v.data() + (size_t)idx * 7;
        triPos.insert(triPos.end(), {p[0], p[1], p[2], 1.f});
    }
    const int ntri = (int)(cube_i.size() / 3);
    Texture* triTex = Texture::Builder().width((uint32_t)ntri * 3).height(1).levels(1)
                          .sampler(Texture::Sampler::SAMPLER_2D)
                          .format(Texture::InternalFormat::RGBA32F)
                          .usage(Texture::Usage::DEFAULT)
                          .build(*engine);
    triTex->setImage(*engine, 0, Texture::PixelBufferDescriptor(triPos.data(), triPos.size() * sizeof(float),
                                                                Texture::Format::RGBA, Texture::Type::FLOAT));
    MaterialInstance* path = pmat->createInstance();
    TextureSampler nearest(TextureSampler::MinFilter::NEAREST, TextureSampler::MagFilter::NEAREST);
    nearest.setWrapModeS(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    nearest.setWrapModeT(TextureSampler::WrapMode::CLAMP_TO_EDGE);
    path->setParameter("tris", triTex, nearest);
    path->setParameter("eye", float3{0.55f, 0.78f, 0.70f});
    path->setParameter("triCount", (float)ntri);
    path->setParameter("bounces", 3.f);
    path->setParameter("ior", nd);
    path->setParameter("dispersion", dispersion);
    path->setParameter("aberration", 0.f);
    path->setParameter("chroma", 0.f);
    path->setParameter("fresnel", 0.f);
    path->setParameter("grad", 0.f);
    path->setParameter("envScale", 1.f);

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
    add(floor_v, floor_i, 36, true, floor_mi, false, true);
    utils::Entity cubeE = add(gem_v, gem_i, 28, false, gem_mi, true, false);
    add(panel_v, panel_i, 28, false, panel, false, false);

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

    // GPU caustic of the glass loop, drawn inside Filament every frame. Same loop as the visible glass below.
    ctri::CausticPass* causticPass = nullptr;
    {
        const float srcc[6][3] = {{-0.78f, -0.22f, 0.12f}, {-0.22f, -0.48f, 0.38f}, {0.55f, -0.12f, 0.22f},
                                  {0.42f, 0.46f, -0.22f},  {-0.18f, 0.38f, -0.48f}, {-0.68f, 0.08f, -0.12f}};
        std::vector<V3> ringc;
        for (const auto& q : srcc) ringc.push_back({(q[0] + 0.095f) * 0.44f, 0.13f + q[2] * 0.16f, q[1] * 0.64f});
        std::vector<Sample> loopc = close_frames(catmull(ringc), 24);
        std::vector<float3> centre;
        for (const Sample& sp : loopc) centre.push_back({sp.p.x, sp.p.y, sp.p.z});
        ctri::Materials cm;
        if (!ctri::load_materials(engine, "caustic_tri", cm)) return fail("caustic materials (run caustic_tri/build.sh)");
        ctri::Params cp;
        cp.ior = 1.55f;
        cp.dispersion = env_f("CAUSTIC_DISPERSION", 0.04f);
        cp.intensity = 1.f;
        cp.cap = (int)env_f("CAUSTIC_CAPTURE", 256);
        cp.map = (int)env_f("CAUSTIC_MAP", 1024);
        cp.footprint = false;  // the tube casts a real Filament shadow instead
        cp.rsize = 2.f * FLOOR_HW;
        cp.boxCenter = {cu.c.x, cu.c.y, cu.c.z};
        cp.boxAxis0 = {cu.a[0].x, cu.a[0].y, cu.a[0].z};
        cp.boxAxis1 = {cu.a[1].x, cu.a[1].y, cu.a[1].z};
        cp.boxAxis2 = {cu.a[2].x, cu.a[2].y, cu.a[2].z};
        cp.boxHalf = cu.h;
        causticPass = new ctri::CausticPass(engine, cm, {ctri::tube(centre, 64, 0.02f)}, float3{sun.x, sun.y, sun.z}, cp);
        // draw once now: the room capture below sees the floor too, so its texture must not be empty
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) break;
        causticPass->draw(renderer);
        renderer->endFrame();
        engine->flushAndWait();
        renderer->setClearOptions({});
        floor_mi->setParameter("tubeCaustic", causticPass->caustic(), lin);
    }

    // The path leaves the glass into this room. Capture it without the cube, before the grade.
    scene->remove(cubeE);
    const int FACE = 512;
    Texture* faceColor = Texture::Builder()
                             .width(FACE).height(FACE).levels(1)
                             .usage(Texture::Usage::COLOR_ATTACHMENT | Texture::Usage::BLIT_SRC)
                             .format(Texture::InternalFormat::RGBA16F)
                             .build(*engine);
    Texture* faceDepth = Texture::Builder()
                             .width(FACE).height(FACE).levels(1)
                             .usage(Texture::Usage::DEPTH_ATTACHMENT)
                             .format(Texture::InternalFormat::DEPTH24)
                             .build(*engine);
    RenderTarget* faceRT = RenderTarget::Builder()
                               .texture(RenderTarget::AttachmentPoint::COLOR, faceColor)
                               .texture(RenderTarget::AttachmentPoint::DEPTH, faceDepth)
                               .build(*engine);
    view->setPostProcessingEnabled(false);
    view->setViewport({0, 0, (uint32_t)FACE, (uint32_t)FACE});
    view->setRenderTarget(faceRT);
    camera->setProjection(90.0, 1.0, 0.02, 20.0, Camera::Fov::VERTICAL);
    float3 origin{cu.c.x, cu.c.y, cu.c.z};
    float3 fwd[6] = {{1, 0, 0}, {-1, 0, 0}, {0, 1, 0}, {0, -1, 0}, {0, 0, 1}, {0, 0, -1}};
    float3 upv[6] = {{0, -1, 0}, {0, -1, 0}, {0, 0, 1}, {0, 0, -1}, {0, -1, 0}, {0, -1, 0}};
    static std::vector<float> roomFace[6];
    for (int f = 0; f < 6; ++f) {
        camera->lookAt(origin, origin + fwd[f], upv[f]);
        roomFace[f].assign((size_t)FACE * FACE * 4, 0.f);
        bool opened = false;
        for (int i = 0; i < 4000; ++i)
            if (renderer->beginFrame(swap)) {
                opened = true;
                break;
            }
        if (!opened) return fail("room face");
        renderer->render(view);
        renderer->readPixels(faceRT, 0, 0, FACE, FACE,
                             backend::PixelBufferDescriptor(roomFace[f].data(), roomFace[f].size() * sizeof(float),
                                                            backend::PixelDataFormat::RGBA, backend::PixelDataType::FLOAT));
        renderer->endFrame();
        engine->flushAndWait();
        float mx = 0.f;
        for (int i = 0; i < FACE * FACE; ++i) mx = std::max(mx, roomFace[f][i * 4]);
        std::printf("room face %d max %.2f\n", f, mx);
    }
    Texture* room = Texture::Builder().width(FACE).height(FACE).levels(1)
                        .sampler(Texture::Sampler::SAMPLER_CUBEMAP)
                        .format(Texture::InternalFormat::RGBA16F)
                        .usage(Texture::Usage::DEFAULT)
                        .build(*engine);
    for (int f = 0; f < 6; ++f) {
        room->setImage(*engine, 0, 0, 0, (uint32_t)f, (uint32_t)FACE, (uint32_t)FACE, 1,
                       Texture::PixelBufferDescriptor(roomFace[f].data(), roomFace[f].size() * sizeof(float),
                                                      Texture::Format::RGBA, Texture::Type::FLOAT));
    }
    path->setParameter("env", room, TextureSampler(TextureSampler::MagFilter::LINEAR));
    scene->addEntity(cubeE);
    {
        // One closed glass path wrapped around the cube. The six control points are beauty.mm's, laid flat:
        // beauty's x,y become the floor plane, beauty's z becomes the height above the floor.
        const float src[6][3] = {{-0.78f, -0.22f, 0.12f}, {-0.22f, -0.48f, 0.38f}, {0.55f, -0.12f, 0.22f},
                                 {0.42f, 0.46f, -0.22f},  {-0.18f, 0.38f, -0.48f}, {-0.68f, 0.08f, -0.12f}};
        std::vector<V3> ring;
        for (const auto& q : src)
            ring.push_back({(q[0] + 0.095f) * 0.44f, 0.13f + q[2] * 0.16f, q[1] * 0.64f});
        std::vector<Sample> loop = close_frames(catmull(ring), 24);
        MaterialInstance* gmi = gemmat->createInstance();
        gmi->setParameter("ior", 1.75f);
        gmi->setParameter("transmission", 1.0f);
        gmi->setParameter("thickness", 0.10f);
        gmi->setParameter("dispersion", 1.2f);
        gmi->setParameter("fire", env_f("TUBE_FIRE", 1800.f));
        gmi->setParameter("film", 0.5f);
        gmi->setParameter("tint", env_f("TUBE_TINT", 1.4f));
        gmi->setParameter("glint", env_f("TUBE_GLINT", 6000.f));
        gmi->setParameter("cells", env_f("TUBE_CELLS", 0.f));
        Built t = tube(loop, 40, [](float) { return 0.02f; });
        add(t.v, t.idx, 28, false, gmi, true, false);
        std::printf("glass path rings %zu radius 0.02\n", loop.size());
    }
    view->setPostProcessingEnabled(true);
    view->setViewport({0, 0, (uint32_t)W, (uint32_t)H});
    view->setRenderTarget(rt);
    camera->setExposure(2.8f, 1.0f / 125.0f, 100.0f);
    camera->lookAt({0.55f, 0.78f, 0.70f}, {0.0f, 0.04f, 0.0f}, {0, 1, 0});
    camera->setProjection(30.0, double(W) / double(H), 0.05, 20.0, Camera::Fov::VERTICAL);
    std::printf("path triangles %d bounces 3 ior %.2f\n", ntri, nd);

    std::vector<uint8_t> pic((size_t)W * H * 4);
    auto t0 = std::chrono::steady_clock::now();
    bool opened = false;
    for (int i = 0; i < 4000; ++i)
        if (renderer->beginFrame(swap)) {
            opened = true;
            break;
        }
    if (!opened) return fail("frame");
    view->setRenderTarget(rt);
    causticPass->draw(renderer);
    renderer->setClearOptions({});
    renderer->render(view);
    renderer->readPixels(rt, 0, 0, W, H,
                         backend::PixelBufferDescriptor(pic.data(), pic.size(), backend::PixelDataFormat::RGBA,
                                                        backend::PixelDataType::UBYTE));
    renderer->endFrame();
    engine->flushAndWait();
    auto t1 = std::chrono::steady_clock::now();
    if (!write_png("out/prism-glass.png", W, H, pic.data())) return fail("png");
    std::printf("frame %.1f ms wall (flushAndWait + readPixels)\n", std::chrono::duration<double, std::milli>(t1 - t0).count());
    std::printf("PASS prism-glass\n");
    return 0;
}
