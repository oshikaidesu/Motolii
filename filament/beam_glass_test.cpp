// Desk check of beam_glass.h. No renderer.
//   1. Energy: sunlight in = light on the floor + light to the sky + branches dropped, for every case.
//   2. Placement: the polygons agree with rays walked one by one through the same solid (single wavelength).
//   3. The claim under test: a perfect box throws no colour, a chamfered one does.
//   4. Top-view pictures of the floor light (out/beam-top-*.png) to look at.
// Build and run: ./beam_test.sh
#include "beam_glass.h"

#include <zlib.h>

#include <algorithm>
#include <chrono>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

using namespace beam;

static bool write_png(const char* path, int w, int h, const uint8_t* rgba) {
    std::vector<uint8_t> raw((size_t)h * ((size_t)w * 4 + 1));
    for (int y = 0; y < h; ++y) {
        raw[(size_t)y * ((size_t)w * 4 + 1)] = 0;
        std::memcpy(raw.data() + (size_t)y * ((size_t)w * 4 + 1) + 1, rgba + (size_t)y * w * 4, (size_t)w * 4);
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
        uint32_t crc = (uint32_t)crc32(0, (const Bytef*)tag, 4);
        if (n) crc = (uint32_t)crc32(crc, data, n);
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

// Floor image: W x W pixels over [cx - R, cx + R] x [cz - R, cz + R], linear RGB irradiance.
struct Image {
    int W;
    double cx, cz, R;
    std::vector<double> rgb;
    Image(int w, double cx_, double cz_, double r) : W(w), cx(cx_), cz(cz_), R(r), rgb((size_t)w * w * 3, 0.0) {}
    double px() const { return 2.0 * R / W; }
};

// Adds convex polygons with S x S point samples per pixel.
static void raster(Image& im, const Result& r, double sign, int S = 4) {
    const double px = im.px();
    for (const Patch& p : r.patches) {
        double x0 = 1e30, x1 = -1e30, z0 = 1e30, z1 = -1e30;
        for (int i = 0; i < p.count; ++i) {
            double x = r.xz[(size_t)(p.first + i) * 2], z = r.xz[(size_t)(p.first + i) * 2 + 1];
            x0 = std::min(x0, x); x1 = std::max(x1, x);
            z0 = std::min(z0, z); z1 = std::max(z1, z);
        }
        int ix0 = std::max(0, (int)std::floor((x0 - (im.cx - im.R)) / px)), ix1 = std::min(im.W - 1, (int)std::floor((x1 - (im.cx - im.R)) / px));
        int iz0 = std::max(0, (int)std::floor((z0 - (im.cz - im.R)) / px)), iz1 = std::min(im.W - 1, (int)std::floor((z1 - (im.cz - im.R)) / px));
        // orientation
        double a2 = 0;
        for (int i = 0; i < p.count; ++i) {
            int j = (i + 1) % p.count;
            a2 += r.xz[(size_t)(p.first + i) * 2] * r.xz[(size_t)(p.first + j) * 2 + 1] - r.xz[(size_t)(p.first + j) * 2] * r.xz[(size_t)(p.first + i) * 2 + 1];
        }
        const double sg = a2 >= 0 ? 1.0 : -1.0;
        for (int iz = iz0; iz <= iz1; ++iz)
            for (int ix = ix0; ix <= ix1; ++ix) {
                int hit = 0;
                for (int sz = 0; sz < S; ++sz)
                    for (int sx = 0; sx < S; ++sx) {
                        double x = im.cx - im.R + (ix + (sx + 0.5) / S) * px, z = im.cz - im.R + (iz + (sz + 0.5) / S) * px;
                        bool in = true;
                        for (int i = 0; i < p.count && in; ++i) {
                            int j = (i + 1) % p.count;
                            double ax = r.xz[(size_t)(p.first + i) * 2], az = r.xz[(size_t)(p.first + i) * 2 + 1];
                            double bx = r.xz[(size_t)(p.first + j) * 2], bz = r.xz[(size_t)(p.first + j) * 2 + 1];
                            if (sg * ((bx - ax) * (z - az) - (bz - az) * (x - ax)) < 0) in = false;
                        }
                        hit += in ? 1 : 0;
                    }
                if (!hit) continue;
                double cov = sign * (double)hit / (S * S);
                double* o = &im.rgb[((size_t)iz * im.W + ix) * 3];
                o[0] += p.rgb.x * cov;
                o[1] += p.rgb.y * cov;
                o[2] += p.rgb.z * cov;
            }
    }
}

// Rays walked one at a time, same physics, no polygons. One wavelength; deposits flux into the image.
static void walk_inside(const Solid& s, const Params& par, double n, V3 o, V3 d, int from, double flux, double weight, int bounces, Image& im, double& floorFlux) {
    int best = -1;
    double tb = 1e30;
    for (int j = 0; j < (int)s.planes.size(); ++j) {
        if (j == from || s.faces[(size_t)j].n == 0) continue;
        double dj = dot(s.planes[(size_t)j].n, d);
        if (dj <= 1e-12) continue;
        double t = (s.planes[(size_t)j].o - dot(s.planes[(size_t)j].n, o)) / dj;
        if (t < tb) { tb = t; best = j; }
    }
    if (best < 0) return;
    V3 p = o + d * tb, nj = s.planes[(size_t)best].n;
    double ci = dot(nj, d), Rf = fresnel(ci, n, 1.0);
    if (Rf < 1.0) {
        V3 out;
        refract(d, nj * -1.0, n, out);
        if (out.y < -1e-9) {
            double t = std::max(0.0, (par.floorY - p.y) / out.y);
            V3 q = p + out * t;
            int ix = (int)std::floor((q.x - (im.cx - im.R)) / im.px()), iz = (int)std::floor((q.z - (im.cz - im.R)) / im.px());
            double f = flux * (1.0 - Rf);
            floorFlux += f;
            if (ix >= 0 && iz >= 0 && ix < im.W && iz < im.W) im.rgb[((size_t)iz * im.W + ix) * 3] += f / (im.px() * im.px());
        }
    }
    double w = weight * Rf;
    if (bounces < par.maxBounces && w > par.cutoff) walk_inside(s, par, n, p, d - nj * (2.0 * ci), best, flux * Rf, w, bounces + 1, im, floorFlux);
}
static double walk_all(const Solid& s, const Params& par, double nm, Image& im, int raysPerSide) {
    const V3 L = normalize(par.sun);
    const double n = par.glass.n(nm);
    V3 up = std::fabs(L.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
    V3 U = normalize(cross(up, L)), V = cross(L, U);
    // bounds of the solid seen along the sun
    double u0 = 1e30, u1 = -1e30, v0 = 1e30, v1 = -1e30;
    V3 c{0, 0, 0};
    for (const Poly& P : s.faces)
        for (int i = 0; i < P.n; ++i) {
            u0 = std::min(u0, dot(P.p[(size_t)i], U)); u1 = std::max(u1, dot(P.p[(size_t)i], U));
            v0 = std::min(v0, dot(P.p[(size_t)i], V)); v1 = std::max(v1, dot(P.p[(size_t)i], V));
            c = P.p[(size_t)i];
        }
    const double du = (u1 - u0) / raysPerSide, dv = (v1 - v0) / raysPerSide;
    const double back = dot(c, L) - 100.0;
    double floorFlux = 0;
    for (int iv = 0; iv < raysPerSide; ++iv)
        for (int iu = 0; iu < raysPerSide; ++iu) {
            V3 o = U * (u0 + (iu + 0.5) * du) + V * (v0 + (iv + 0.5) * dv) + L * back;
            double tin = -1e30, tout = 1e30;
            int fin = -1;
            for (int j = 0; j < (int)s.planes.size(); ++j) {
                double dj = dot(s.planes[(size_t)j].n, L);
                double t = (s.planes[(size_t)j].o - dot(s.planes[(size_t)j].n, o)) / dj;
                if (dj < -1e-12) { if (t > tin) { tin = t; fin = j; } }
                else if (dj > 1e-12) tout = std::min(tout, t);
            }
            if (fin < 0 || tin >= tout) continue;
            V3 p = o + L * tin, nf = s.planes[(size_t)fin].n;
            double ci = -dot(nf, L), Rf = fresnel(ci, 1.0, n), flux = du * dv;  // unit sun irradiance on its own cross-section
            if (par.outerReflection) {
                V3 r = L + nf * (2.0 * ci);
                if (r.y < -1e-9) {
                    V3 q = p + r * ((par.floorY - p.y) / r.y);
                    int ix = (int)std::floor((q.x - (im.cx - im.R)) / im.px()), iz = (int)std::floor((q.z - (im.cz - im.R)) / im.px());
                    floorFlux += flux * Rf;
                    if (ix >= 0 && iz >= 0 && ix < im.W && iz < im.W) im.rgb[((size_t)iz * im.W + ix) * 3] += flux * Rf / (im.px() * im.px());
                }
            }
            V3 in;
            refract(L, nf, 1.0 / n, in);
            walk_inside(s, par, n, p, in, fin, flux * (1.0 - Rf), 1.0 - Rf, 0, im, floorFlux);
        }
    return floorFlux;
}

// The same walk through a whole scene: a ray that leaves one solid may meet another.
struct Walk {
    const Scene* sc;
    const Params* par;
    double n;
    Image* im;
    double floorFlux = 0;
};
static bool nearest(const Scene& sc, V3 o, V3 d, int from, int& S, int& g, double& t) {
    t = 1e30;
    S = -1;
    for (int k = 0; k < (int)sc.solids.size(); ++k) {
        if (k == from) continue;
        const Solid& so = sc.solids[(size_t)k];
        double tin = -1e30, tout = 1e30;
        int gin = -1;
        bool miss = false;
        for (int j = 0; j < (int)so.planes.size() && !miss; ++j) {
            if (so.faces[(size_t)j].n == 0) continue;
            const double dj = dot(so.planes[(size_t)j].n, d), off = so.planes[(size_t)j].o - dot(so.planes[(size_t)j].n, o);
            if (dj < -1e-12) { if (off / dj > tin) { tin = off / dj; gin = j; } }
            else if (dj > 1e-12) tout = std::min(tout, off / dj);
            else if (off < 0) miss = true;
        }
        if (!miss && gin >= 0 && tin < tout && tin >= -1e-9 && tin < t) { t = tin; S = k; g = gin; }
    }
    return S >= 0;
}
static void w_enter(Walk& w, int S, int g, V3 p, V3 d, double flux, double weight, int entries);
static void w_air(Walk& w, V3 o, V3 d, int from, double flux, double weight, int entries) {
    int S, g;
    double t;
    if (nearest(*w.sc, o, d, from, S, g, t)) {
        w_enter(w, S, g, o + d * t, d, flux, weight, entries);
        return;
    }
    if (d.y < -1e-9) {
        const double tt = std::max(0.0, (w.par->floorY - o.y) / d.y);
        const V3 q = o + d * tt;
        Image& im = *w.im;
        const int ix = (int)std::floor((q.x - (im.cx - im.R)) / im.px()), iz = (int)std::floor((q.z - (im.cz - im.R)) / im.px());
        w.floorFlux += flux;
        if (ix >= 0 && iz >= 0 && ix < im.W && iz < im.W) im.rgb[((size_t)iz * im.W + ix) * 3] += flux / (im.px() * im.px());
    }
}
static void w_inside(Walk& w, int S, V3 o, V3 d, int from, double flux, double weight, int bounces, int entries) {
    const Solid& s = w.sc->solids[(size_t)S];
    int best = -1;
    double tb = 1e30;
    for (int j = 0; j < (int)s.planes.size(); ++j) {
        if (j == from || s.faces[(size_t)j].n == 0) continue;
        const double dj = dot(s.planes[(size_t)j].n, d);
        if (dj <= 1e-12) continue;
        const double t = (s.planes[(size_t)j].o - dot(s.planes[(size_t)j].n, o)) / dj;
        if (t < tb) { tb = t; best = j; }
    }
    if (best < 0) return;
    const V3 p = o + d * tb, nj = s.planes[(size_t)best].n;
    const double ci = dot(nj, d), Rf = fresnel(ci, w.n, 1.0);
    if (Rf < 1.0) {
        V3 out;
        refract(d, nj * -1.0, w.n, out);
        w_air(w, p, out, S, flux * (1.0 - Rf), weight * (1.0 - Rf), entries);
    }
    const double ww = weight * Rf;
    if (bounces < w.par->maxBounces && ww > w.par->cutoff) w_inside(w, S, p, d - nj * (2.0 * ci), best, flux * Rf, ww, bounces + 1, entries);
}
static void w_enter(Walk& w, int S, int g, V3 p, V3 d, double flux, double weight, int entries) {
    const V3 ng = w.sc->solids[(size_t)S].planes[(size_t)g].n;
    const double ci = -dot(ng, d), Rf = fresnel(ci, 1.0, w.n);
    if (w.par->outerReflection && weight * Rf > w.par->cutoff * 0.25) w_air(w, p, d + ng * (2.0 * ci), S, flux * Rf, weight * Rf, entries);
    if (entries < w.par->maxSolids) {
        V3 in;
        refract(d, ng, 1.0 / w.n, in);
        w_inside(w, S, p, in, g, flux * (1.0 - Rf), weight * (1.0 - Rf), 0, entries + 1);
    }
}
static double walk_scene(const Scene& sc, const Params& par, double nm, Image& im, int raysPerSide) {
    const V3 L = normalize(par.sun);
    V3 up = std::fabs(L.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
    V3 U = normalize(cross(up, L)), V = cross(L, U);
    double u0 = 1e30, u1 = -1e30, v0 = 1e30, v1 = -1e30, back = 1e30;
    for (const Solid& s : sc.solids)
        for (const Poly& P : s.faces)
            for (int i = 0; i < P.n; ++i) {
                u0 = std::min(u0, dot(P.p[(size_t)i], U)); u1 = std::max(u1, dot(P.p[(size_t)i], U));
                v0 = std::min(v0, dot(P.p[(size_t)i], V)); v1 = std::max(v1, dot(P.p[(size_t)i], V));
                back = std::min(back, dot(P.p[(size_t)i], L));
            }
    const double du = (u1 - u0) / raysPerSide, dv = (v1 - v0) / raysPerSide;
    Walk w{&sc, &par, par.glass.n(nm), &im};
    for (int iv = 0; iv < raysPerSide; ++iv)
        for (int iu = 0; iu < raysPerSide; ++iu) {
            const V3 o = U * (u0 + (iu + 0.5) * du) + V * (v0 + (iv + 0.5) * dv) + L * (back - 10.0);
            int S, g;
            double t;
            if (nearest(sc, o, L, -1, S, g, t)) w_enter(w, S, g, o + L * t, L, du * dv, 1.0, 0);
        }
    return w.floorFlux;
}

// Rays walked one by one through a scene of any kinds of solid (flat or round), for the cheap trace to be
// compared with. One wavelength; splits at every face like the real thing.
struct WalkAny {
    const Scene* sc;
    const Params* par;
    double n;
    Image* im;
    int channel;
    double scale;
    double floorFlux = 0;
};
static void wa_enter(WalkAny& w, int S, V3 p, V3 ng, int g, V3 d, double flux, double weight, int entries);
static void wa_air(WalkAny& w, V3 o, V3 d, int from, double flux, double weight, int entries) {
    int S, g;
    double t;
    V3 n;
    if (detail::first_solid(*w.sc, o, d, from, S, g, t, n)) {
        wa_enter(w, S, o + d * t, n, g, d, flux, weight, entries);
        return;
    }
    if (d.y < -1e-9) {
        const double tt = std::max(0.0, (w.par->floorY - o.y) / d.y);
        const V3 q = o + d * tt;
        Image& im = *w.im;
        const int ix = (int)std::floor((q.x - (im.cx - im.R)) / im.px()), iz = (int)std::floor((q.z - (im.cz - im.R)) / im.px());
        w.floorFlux += flux;
        if (ix >= 0 && iz >= 0 && ix < im.W && iz < im.W) im.rgb[((size_t)iz * im.W + ix) * 3 + (size_t)w.channel] += w.scale * flux / (im.px() * im.px());
    }
}
static void wa_inside(WalkAny& w, int S, V3 o, V3 d, int from, double flux, double weight, int bounces, int entries) {
    const Solid& s = w.sc->solids[(size_t)S];
    double t;
    V3 nj;
    int best;
    if (!detail::leave(s, o, d, from, t, nj, best)) return;
    const V3 p = o + d * t;
    const double ci = dot(nj, d), Rf = fresnel(ci, w.n, 1.0);
    if (Rf < 1.0) {
        V3 out;
        refract(d, nj * -1.0, w.n, out);
        wa_air(w, p, out, S, flux * (1.0 - Rf), weight * (1.0 - Rf), entries);
    }
    const double ww = weight * Rf;
    if (bounces < w.par->maxBounces && ww > w.par->cutoff) wa_inside(w, S, p, d - nj * (2.0 * ci), s.kind == Solid::FLAT ? best : -1, flux * Rf, ww, bounces + 1, entries);
}
static void wa_enter(WalkAny& w, int S, V3 p, V3 ng, int g, V3 d, double flux, double weight, int entries) {
    const double ci = -dot(ng, d), Rf = fresnel(ci, 1.0, w.n);
    if (w.par->outerReflection && weight * Rf > w.par->cutoff * 0.25) wa_air(w, p, d + ng * (2.0 * ci), S, flux * Rf, weight * Rf, entries);
    if (entries < w.par->maxSolids) {
        V3 in;
        refract(d, ng, 1.0 / w.n, in);
        wa_inside(w, S, p, in, w.sc->solids[(size_t)S].kind == Solid::FLAT ? g : -1, flux * (1.0 - Rf), weight * (1.0 - Rf), 0, entries + 1);
    }
}
// Adds one wavelength's light to one colour channel of the picture.
static double walk_any(const Scene& sc, const Params& par, double n, Image& im, int channel, double scale, int raysPerSide) {
    const V3 L = normalize(par.sun);
    V3 up = std::fabs(L.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
    V3 U = normalize(cross(up, L)), V = cross(L, U);
    double u0 = 1e30, u1 = -1e30, v0 = 1e30, v1 = -1e30, back = 1e30;
    for (size_t k = 0; k < sc.solids.size(); ++k) {
        const double cu = dot(sc.centre[k], U), cv = dot(sc.centre[k], V), cl = dot(sc.centre[k], L), r = sc.radius[k];
        u0 = std::min(u0, cu - r); u1 = std::max(u1, cu + r);
        v0 = std::min(v0, cv - r); v1 = std::max(v1, cv + r);
        back = std::min(back, cl - r);
    }
    const double du = (u1 - u0) / raysPerSide, dv = (v1 - v0) / raysPerSide;
    WalkAny w{&sc, &par, n, &im, channel, scale};
    for (int iv = 0; iv < raysPerSide; ++iv)
        for (int iu = 0; iu < raysPerSide; ++iu) {
            const V3 o = U * (u0 + (iu + 0.5) * du) + V * (v0 + (iv + 0.5) * dv) + L * (back - 10.0);
            int S, g;
            double t;
            V3 nn;
            if (detail::first_solid(sc, o, L, -1, S, g, t, nn)) wa_enter(w, S, o + L * t, nn, g, L, du * dv, 1.0, 0);
        }
    return w.floorFlux;
}

static void save_top(const char* path, const Solid& s, const Params& par, const Result& r, double cx, double cz, double R, int W) {
    Image im(W, cx, cz, R);
    const V3 L = normalize(par.sun);
    for (size_t i = 0; i < im.rgb.size(); ++i) im.rgb[i] = -L.y;  // bare floor in the sun
    raster(im, silhouette(s, par.sun, par.floorY), -1.0);
    raster(im, r, 1.0);
    std::vector<uint8_t> px((size_t)W * W * 4);
    const double ambient = 0.10, albedo = 0.80, exposure = 0.95;
    for (int i = 0; i < W * W; ++i) {
        for (int c = 0; c < 3; ++c) {
            double v = (std::max(0.0, im.rgb[(size_t)i * 3 + c]) + ambient) * albedo * exposure;
            v = v / (1.0 + 0.18 * v);  // gentle shoulder
            v = std::min(1.0, std::max(0.0, v));
            v = v <= 0.0031308 ? 12.92 * v : 1.055 * std::pow(v, 1.0 / 2.4) - 0.055;
            px[(size_t)i * 4 + c] = (uint8_t)std::lround(v * 255.0);
        }
        px[(size_t)i * 4 + 3] = 255;
    }
    write_png(path, W, W, px.data());
}

// How coloured the floor light is: mean over lit pixels of (max - min) / max of the patch light alone.
static double chroma(const Result& r, double cx, double cz, double R) {
    Image im(384, cx, cz, R);
    raster(im, r, 1.0, 3);
    double num = 0, den = 0;
    for (size_t i = 0; i < im.rgb.size(); i += 3) {
        double a = std::max(0.0, im.rgb[i]), b = std::max(0.0, im.rgb[i + 1]), c = std::max(0.0, im.rgb[i + 2]);
        double mx = std::max(a, std::max(b, c)), mn = std::min(a, std::min(b, c));
        num += mx - mn;
        den += mx;
    }
    return den > 0 ? num / den : 0;
}

int main() {
    int bad = 0;
    std::system("mkdir -p out");
    const double half = 0.5;
    const V3 center{0, half, 0};
    struct Case { const char* name; V3 sun; double yaw, chamfer; };
    const Case cases[] = {
        {"oblique, perfect box", {0.62, -0.62, 0.28}, 0.35, 0.0},
        {"oblique, chamfer 1.5%", {0.62, -0.62, 0.28}, 0.35, 0.015},
        {"low sun, chamfer 1.5%", {0.85, -0.38, -0.20}, 0.80, 0.015},
        {"high sun, chamfer 3%", {0.20, -0.95, 0.15}, 0.10, 0.03},
        {"face-on, chamfer 1.5%", {0.70, -0.70, 0.0}, 0.0, 0.015},
    };

    std::printf("1. energy (per unit of sun; floor + sky + dropped must equal in)\n");
    for (const Case& c : cases) {
        Solid s = chamfered_box(center, c.yaw, half, c.chamfer * 2.0 * half);
        Params par;
        par.sun = c.sun;
        par.bands = 8;
        Result r = trace(s, par);
        double out = r.fluxFloor + r.fluxSky + r.fluxDropped, err = std::fabs(out - r.fluxIn) / r.fluxIn;
        bool ok = err < 1e-9;
        bad += ok ? 0 : 1;
        std::printf("   %-24s in %.6f  floor %.4f  sky %.4f  dropped %.4f  error %.1e  patches %zu  beams %d  %s\n", c.name, r.fluxIn, r.fluxFloor / r.fluxIn,
                    r.fluxSky / r.fluxIn, r.fluxDropped / r.fluxIn, err, r.patches.size() / 8, r.beams / 8, ok ? "ok" : "FAIL");
    }

#include "beam_glass_test_2.inc"
#include "beam_glass_test_3.inc"
