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

    std::printf("2. polygons against rays walked one by one (single wavelength, 96 x 96 bins)\n");
    for (const Case& c : cases) {
        Solid s = chamfered_box(center, c.yaw, half, c.chamfer * 2.0 * half);
        Params par;
        par.sun = c.sun;
        par.bands = 1;
        par.nmMin = par.nmMax = 550.0;
        Result r = trace(s, par);
        const double R = 4.0;
        Image a(96 * 8, 0, 0, R), b(96, 0, 0, R);
        raster(a, r, 1.0, 2);
        double rayFloor = walk_all(s, par, 550.0, b, 3000);
        // bring the polygon image down to the same bins (band weight for one band is (1, 1, 1); use the red channel)
        double l1 = 0, tot = 0, polyFloor = 0;
        for (int z = 0; z < 96; ++z)
            for (int x = 0; x < 96; ++x) {
                double sum = 0;
                for (int dz = 0; dz < 8; ++dz)
                    for (int dx = 0; dx < 8; ++dx) sum += a.rgb[((size_t)(z * 8 + dz) * a.W + (x * 8 + dx)) * 3];
                sum /= 64.0;
                double ref = b.rgb[((size_t)z * b.W + x) * 3];
                l1 += std::fabs(sum - ref);
                tot += ref;
                polyFloor += sum * b.px() * b.px();
            }
        double inside = 0;
        for (size_t i = 0; i < b.rgb.size(); i += 3) inside += b.rgb[i] * b.px() * b.px();
        bool ok = l1 / tot < 0.02 && std::fabs(r.fluxFloor - rayFloor) / rayFloor < 0.003;
        bad += ok ? 0 : 1;
        std::printf("   %-24s floor light: polygons %.5f  rays %.5f  (%.2f%% apart)   picture difference %.2f%%  (in frame %.1f%%)  %s\n", c.name, r.fluxFloor, rayFloor,
                    100.0 * std::fabs(r.fluxFloor - rayFloor) / rayFloor, 100.0 * l1 / tot, 100.0 * inside / rayFloor, ok ? "ok" : "FAIL");
        (void)polyFloor;
    }

    std::printf("2b. other solids: energy, and polygons against rays walked one by one\n");
    {
        struct Shape { const char* name; Solid s; };
        auto rot = [](double yaw, double tilt, V3 R[3]) {
            // tilt about x, then yaw about y
            const double cy = std::cos(yaw), sy = std::sin(yaw), ct = std::cos(tilt), st = std::sin(tilt);
            const V3 X{1, 0, 0}, Y{0, ct, st}, Z{0, -st, ct};
            auto yawed = [&](V3 v) { return V3{cy * v.x + sy * v.z, v.y, -sy * v.x + cy * v.z}; };
            R[0] = yawed(X); R[1] = yawed(Y); R[2] = yawed(Z);
        };
        auto rest = [&](const Solid& local, double yaw, double tilt) {
            V3 R[3];
            rot(yaw, tilt, R);
            Solid t = placed(local, R, 1.0, {0, 0, 0});
            double low = 1e30;
            for (const Poly& P : t.faces)
                for (int i = 0; i < P.n; ++i) low = std::min(low, P.p[(size_t)i].y);
            return placed(local, R, 1.0, {0, -low + 1e-4, 0});
        };
        const double pi = 3.14159265358979;
        std::vector<Shape> shapes;
        shapes.push_back({"slab", rest(chamfered(box(0.6, 0.12, 0.4), 0.012), 0.4, 0)});
        shapes.push_back({"pillar", rest(chamfered(box(0.2, 0.6, 0.2), 0.012), 0.7, 0)});
        shapes.push_back({"triangular prism, lying", rest(chamfered(prism(3, 0.4, 0.5), 0.012), 0.5, pi / 2)});
        shapes.push_back({"triangular prism, standing", rest(chamfered(prism(3, 0.4, 0.5), 0.012), 0.2, 0)});
        shapes.push_back({"hexagonal prism", rest(chamfered(prism(6, 0.4, 0.3), 0.012), 0.3, 0)});
        shapes.push_back({"octagonal prism, lying", rest(chamfered(prism(8, 0.3, 0.5), 0.012), 0.9, pi / 2)});
        shapes.push_back({"square pyramid", rest(chamfered(pyramid(4, 0.5, 0.6), 0.012), 0.3, 0)});
        shapes.push_back({"octahedron", rest(chamfered(octahedron(0.45), 0.012), 0.2, 0.6155)});
        shapes.push_back({"tetrahedron", rest(chamfered(pyramid(3, 0.5, 0.7), 0.012), 0.1, 0)});
        for (const Shape& sh : shapes) {
            Params par;
            par.sun = {0.62, -0.62, 0.28};
            par.bands = 1;
            par.nmMin = par.nmMax = 550.0;
            Result r = trace(sh.s, par);
            double out = r.fluxFloor + r.fluxSky + r.fluxDropped, err = std::fabs(out - r.fluxIn) / r.fluxIn;
            const double R = 5.0;
            Image a(96 * 8, 0, 0, R), b(96, 0, 0, R);
            raster(a, r, 1.0, 2);
            double rayFloor = walk_all(sh.s, par, 550.0, b, 2400);
            double l1 = 0, tot = 0;
            for (int z = 0; z < 96; ++z)
                for (int x = 0; x < 96; ++x) {
                    double sum = 0;
                    for (int dz = 0; dz < 8; ++dz)
                        for (int dx = 0; dx < 8; ++dx) sum += a.rgb[((size_t)(z * 8 + dz) * a.W + (x * 8 + dx)) * 3];
                    sum /= 64.0;
                    double ref = b.rgb[((size_t)z * b.W + x) * 3];
                    l1 += std::fabs(sum - ref);
                    tot += ref;
                }
            std::vector<float> tri;
            FastStats st;
            Params pc;
            pc.sun = par.sun;
            pc.glass.exaggerate = 10;
            auto t0 = std::chrono::steady_clock::now();
            trace_fast(sh.s, pc, 0.003, tri, &st);
            double ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
            bool ok = err < 1e-9 && l1 / tot < 0.03 && std::fabs(r.fluxFloor - rayFloor) / rayFloor < 0.005;
            bad += ok ? 0 : 1;
            std::printf("   %-28s %2zu faces  energy error %.1e  floor %.3f sky %.3f dropped %.3f   vs rays: light %.2f%% apart, picture %.2f%%   fast %.2f ms, %d ways  %s\n",
                        sh.name, sh.s.planes.size(), err, r.fluxFloor / r.fluxIn, r.fluxSky / r.fluxIn, r.fluxDropped / r.fluxIn,
                        100.0 * std::fabs(r.fluxFloor - rayFloor) / rayFloor, 100.0 * l1 / tot, ms, st.ways, ok ? "ok" : "FAIL");
        }
    }

    std::printf("2c. several solids together: energy, polygons against rays, fast against reference\n");
    {
        auto rest_on = [](const Solid& local, double yaw, double x, double y0, double z) {
            const double cy = std::cos(yaw), sy = std::sin(yaw);
            V3 R[3] = {{cy, 0, -sy}, {0, 1, 0}, {sy, 0, cy}};
            Solid t = placed(local, R, 1.0, {0, 0, 0});
            double low = 1e30;
            for (const Poly& P : t.faces)
                for (int i = 0; i < P.n; ++i) low = std::min(low, P.p[(size_t)i].y);
            return placed(local, R, 1.0, {x, y0 - low + 1e-4, z});
        };
        const Solid cube = chamfered(box(0.5, 0.5, 0.5), 0.02), slab = chamfered(box(0.7, 0.14, 0.5), 0.02);
        const Solid tri3 = chamfered(prism(3, 0.5, 0.5), 0.02), pil = chamfered(box(0.22, 0.7, 0.22), 0.02);
        struct Set { const char* name; Scene sc; V3 sun; };
        std::vector<Set> sets;
        {
            Scene sc;  // the second cube stands in the first one's shadow and light
            sc.solids = {rest_on(cube, 0.3, 0, 0, 0), rest_on(cube, 0.9, 1.25, 0, 0.45)};
            sc.bound();
            sets.push_back({"two cubes, one behind the other", sc, {0.92, -0.42, 0.22}});
        }
        {
            Scene sc;  // a slab lying on a cube
            sc.solids = {rest_on(cube, 0.2, 0, 0, 0), rest_on(slab, 0.7, 0.05, 1.0002, 0.0)};
            sc.bound();
            sets.push_back({"slab resting on a cube", sc, {0.62, -0.62, 0.28}});
        }
        {
            Scene sc;  // three shapes close together
            sc.solids = {rest_on(tri3, 0.4, -0.2, 0, 0.1), rest_on(cube, 1.1, 0.95, 0, -0.35), rest_on(pil, 0.2, 0.55, 0, 0.75)};
            sc.bound();
            sets.push_back({"prism, cube and pillar close together", sc, {0.80, -0.45, -0.25}});
        }
        for (const Set& st : sets) {
            Params par;
            par.sun = st.sun;
            par.bands = 1;
            par.nmMin = par.nmMax = 550.0;
            Result r = trace(st.sc, par);
            double out = r.fluxFloor + r.fluxSky + r.fluxDropped, err = std::fabs(out - r.fluxIn) / r.fluxIn;
            const double R = 6.0;
            Image a(96 * 8, 0.8, 0, R), b(96, 0.8, 0, R);
            raster(a, r, 1.0, 2);
            double rayFloor = walk_scene(st.sc, par, 550.0, b, 3000);
            double l1 = 0, tot = 0;
            for (int z = 0; z < 96; ++z)
                for (int x = 0; x < 96; ++x) {
                    double sum = 0;
                    for (int dz = 0; dz < 8; ++dz)
                        for (int dx = 0; dx < 8; ++dx) sum += a.rgb[((size_t)(z * 8 + dz) * a.W + (x * 8 + dx)) * 3];
                    sum /= 64.0;
                    double ref = b.rgb[((size_t)z * b.W + x) * 3];
                    l1 += std::fabs(sum - ref);
                    tot += ref;
                }
            // how much of the floor light went through more than one solid
            double shared = 0, all = 0;
            for (const Patch& p : r.patches) {
                double ar = 0;
                for (int i = 1; i + 1 < p.count; ++i) {
                    const double* q = &r.xz[(size_t)p.first * 2];
                    ar += 0.5 * std::fabs((q[i * 2] - q[0]) * (q[i * 2 + 3] - q[1]) - (q[i * 2 + 2] - q[0]) * (q[i * 2 + 1] - q[1]));
                }
                bool two = false;
                for (int i = 1; i < p.len; ++i) two = two || step_solid(p.seq[i]) != step_solid(p.seq[0]);
                all += ar * p.rgb.x;
                if (two) shared += ar * p.rgb.x;
            }
            // fast against the reference, all colours
            Params pc;
            pc.sun = st.sun;
            pc.glass.exaggerate = 10;
            const int Wd = 1280;
            const double detail = 2.0 * R / Wd;
            Result ref = trace(st.sc, pc);
            std::vector<float> tri;
            FastStats fs;
            auto t0 = std::chrono::steady_clock::now();
            trace_fast(st.sc, pc, std::vector<double>(st.sc.solids.size(), detail), tri, &fs);
            double ms1 = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
            pc.threads = 8;
            std::vector<float> tri8;
            t0 = std::chrono::steady_clock::now();
            trace_fast(st.sc, pc, std::vector<double>(st.sc.solids.size(), detail), tri8);
            double ms8 = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
            const bool sameThreads = tri == tri8;
            Result fr;
            for (size_t i = 0; i + 3 * TRI <= tri.size(); i += 3 * TRI) {
                Patch p;
                p.first = (int)(fr.xz.size() / 2);
                p.count = 3;
                p.rgb = {tri[i + 2], tri[i + 3], tri[i + 4]};
                for (int k = 0; k < 3; ++k) {
                    fr.xz.push_back(tri[i + (size_t)k * TRI]);
                    fr.xz.push_back(tri[i + (size_t)k * TRI + 1]);
                }
                fr.patches.push_back(p);
            }
            Image fa(Wd, 0.8, 0, R), fb(Wd, 0.8, 0, R);
            raster(fa, ref, 1.0, 2);
            raster(fb, fr, 1.0, 2);
            double fl1 = 0, ftot = 0;
            for (size_t i = 0; i < fa.rgb.size(); ++i) {
                fl1 += std::fabs(fa.rgb[i] - fb.rgb[i]);
                ftot += std::fabs(fa.rgb[i]);
            }
            bool ok = err < 1e-9 && l1 / tot < 0.03 && std::fabs(r.fluxFloor - rayFloor) / rayFloor < 0.005 && fl1 / ftot < 0.03 && sameThreads;
            bad += ok ? 0 : 1;
            std::printf("   %-38s energy error %.1e   vs rays: light %.2f%% apart, picture %.2f%%   through more than one solid: %.1f%% of the floor light\n", st.name, err,
                        100.0 * std::fabs(r.fluxFloor - rayFloor) / rayFloor, 100.0 * l1 / tot, 100.0 * shared / all);
            std::printf("   %-38s fast vs reference %.2f%%   %d ways, %.2f ms on 1 thread, %.2f ms on 8, same triangles: %s  %s\n", "", 100.0 * fl1 / ftot, fs.ways, ms1, ms8,
                        sameThreads ? "yes" : "NO", ok ? "ok" : "FAIL");
            // the cheap version next to it: time, how far its picture is from the exact one, and its own top view
            for (int pieces : {1, 2, 4}) {
                Params ph = pc;
                ph.threads = 1;
                std::vector<float> tc;
                CheapStats cs;
                double msC = 1e30;
                for (int k = 0; k < 20; ++k) {
                    t0 = std::chrono::steady_clock::now();
                    trace_cheap(st.sc, ph, std::vector<double>(st.sc.solids.size(), detail), pieces, tc, &cs);
                    msC = std::min(msC, std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
                }
                Result cr;
                for (size_t i = 0; i + 3 * TRI <= tc.size(); i += 3 * TRI) {
                    Patch p;
                    p.first = (int)(cr.xz.size() / 2);
                    p.count = 3;
                    p.rgb = {tc[i + 2], tc[i + 3], tc[i + 4]};
                    for (int k = 0; k < 3; ++k) {
                        cr.xz.push_back(tc[i + (size_t)k * TRI]);
                        cr.xz.push_back(tc[i + (size_t)k * TRI + 1]);
                    }
                    cr.patches.push_back(p);
                }
                Image ca(Wd, 0.8, 0, R);
                raster(ca, cr, 1.0, 2);
                double cl1 = 0, sumC = 0, sumE = 0;
                for (size_t i = 0; i < fa.rgb.size(); ++i) {
                    cl1 += std::fabs(fa.rgb[i] - ca.rgb[i]);
                    sumC += ca.rgb[i];
                    sumE += fa.rgb[i];
                }
                std::printf("   %-38s cheap, pieces %d: %.3f ms on 1 thread, %d pieces, %d patches, %d rays; light on the floor %.0f%% of exact, picture %.0f%% away from exact\n", "",
                            pieces, msC, cs.pieces, cs.patches, cs.rays, 100.0 * sumC / sumE, 100.0 * cl1 / ftot);
                Image im(1400, 0.8, 0, 3.4), cov(1400, 0.8, 0, 3.4);
                const V3 L = normalize(pc.sun);
                for (const Solid& so : st.sc.solids) raster(cov, silhouette(so, pc.sun, 0.0), 1.0);
                raster(im, cr, 1.0);
                std::vector<uint8_t> px((size_t)1400 * 1400 * 4);
                for (int i = 0; i < 1400 * 1400; ++i) {
                    for (int c = 0; c < 3; ++c) {
                        const double shade = std::min(1.0, cov.rgb[(size_t)i * 3] / (-L.y));
                        double v = (std::max(0.0, im.rgb[(size_t)i * 3 + c]) + (-L.y) * (1.0 - shade) + 0.10) * 0.80 * 0.95;
                        v = v / (1.0 + 0.18 * v);
                        v = std::min(1.0, std::max(0.0, v));
                        v = v <= 0.0031308 ? 12.92 * v : 1.055 * std::pow(v, 1.0 / 2.4) - 0.055;
                        px[(size_t)i * 4 + c] = (uint8_t)std::lround(v * 255.0);
                    }
                    px[(size_t)i * 4 + 3] = 255;
                }
                std::string cf = std::string("out/beam-top-scene-") + std::to_string(&st - &sets[0]) + "-cheap" + std::to_string(pieces) + ".png";
                write_png(cf.c_str(), 1400, 1400, px.data());
            }
            Params pv = pc;
            std::string file = std::string("out/beam-top-scene-") + std::to_string(&st - &sets[0]) + ".png";
            // top view picture: bare floor, minus every outline, plus the light
            {
                Image im(1400, 0.8, 0, 3.4);
                const V3 L = normalize(pv.sun);
                for (size_t i = 0; i < im.rgb.size(); ++i) im.rgb[i] = 0;
                Image cov(1400, 0.8, 0, 3.4);
                for (const Solid& so : st.sc.solids) raster(cov, silhouette(so, pv.sun, 0.0), 1.0);
                raster(im, ref, 1.0);
                std::vector<uint8_t> px((size_t)1400 * 1400 * 4);
                for (int i = 0; i < 1400 * 1400; ++i) {
                    for (int c = 0; c < 3; ++c) {
                        const double shade = std::min(1.0, cov.rgb[(size_t)i * 3] / (-L.y));
                        double v = (std::max(0.0, im.rgb[(size_t)i * 3 + c]) + (-L.y) * (1.0 - shade) + 0.10) * 0.80 * 0.95;
                        v = v / (1.0 + 0.18 * v);
                        v = std::min(1.0, std::max(0.0, v));
                        v = v <= 0.0031308 ? 12.92 * v : 1.055 * std::pow(v, 1.0 / 2.4) - 0.055;
                        px[(size_t)i * 4 + c] = (uint8_t)std::lround(v * 255.0);
                    }
                    px[(size_t)i * 4 + 3] = 255;
                }
                write_png(file.c_str(), 1400, 1400, px.data());
            }
        }
    }

    std::printf("2d. round solids: the cheap trace next to rays walked one by one (three wavelengths as red, green, blue)\n");
    {
        auto at = [](const Solid& local, V3 turnAxisTo, double x, double y, double z) {
            // the solid's +Y goes to `turnAxisTo`
            const V3 Y = normalize(turnAxisTo);
            const V3 up = std::fabs(Y.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0};
            const V3 X = normalize(cross(up, Y)), Z = cross(X, Y);
            V3 R[3] = {X, Y, Z};
            return placed(local, R, 1.0, {x, y, z});
        };
        struct Set { const char* name; const char* file; Scene sc; V3 sun; };
        std::vector<Set> sets;
        { Scene sc; sc.solids = {at(ball(0.5), {0, 1, 0}, 0, 0.5005, 0)}; sc.bound(); sets.push_back({"ball", "ball", sc, {0.62, -0.62, 0.28}}); }
        { Scene sc; sc.solids = {at(rod(0.3, 0.6), {0, 1, 0}, 0, 0.6005, 0)}; sc.bound(); sets.push_back({"rod, standing", "rod-up", sc, {0.62, -0.62, 0.28}}); }
        { Scene sc; sc.solids = {at(rod(0.3, 0.7), {0.3, 0, 1}, 0, 0.3005, 0)}; sc.bound(); sets.push_back({"rod, lying", "rod-flat", sc, {0.62, -0.62, 0.28}}); }
        {
            Scene sc;
            V3 I[3] = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
            sc.solids = {at(ball(0.4), {0, 1, 0}, 0, 0.4005, 0), placed(chamfered(box(0.4, 0.4, 0.4), 0.02), I, 1.0, {1.1, 0.4005, 0.35})};
            sc.bound();
            sets.push_back({"ball, with a cube in its light", "ball-cube", sc, {0.85, -0.45, 0.22}});
        }
        { Scene sc; sc.solids = {at(cone(0.45, 0.95), {0, 1, 0}, 0, 0.4755, 0)}; sc.bound(); sets.push_back({"cone, standing", "cone-up", sc, {0.62, -0.62, 0.28}}); }
        { Scene sc; sc.solids = {at(cone(0.45, 0.95), {0, 1, 0}, 0, 0.4755, 0)}; sc.bound(); sets.push_back({"cone, standing, low sun", "cone-low", sc, {0.85, -0.40, 0.22}}); }
        { Scene sc; sc.solids = {at(cone(0.45, 0.95), {0.9, -0.45, 0.3}, 0, 0.206, 0)}; sc.bound(); sets.push_back({"cone, lying", "cone-flat", sc, {0.62, -0.62, 0.28}}); }
        { Scene sc; sc.solids = {at(cone(0.45, 0.95), {0.2, -1, 0.1}, 0, 0.47, 0)}; sc.bound(); sets.push_back({"cone, tip down", "cone-down", sc, {0.62, -0.62, 0.28}}); }
        for (const Set& st : sets) {
            Params par;
            par.sun = st.sun;
            par.glass.exaggerate = 10;
            const int Wp = 900;
            const double R = 3.0, cx = 0.8, detail = 2.0 * R / Wp;
            // rays: three wavelengths into three channels
            Image ref(Wp, cx, 0, R);
            const double nm3[3] = {640, 550, 460};
            double rayFloor = 0;
            for (int c = 0; c < 3; ++c) rayFloor += walk_any(st.sc, par, par.glass.n(nm3[c]), ref, c, 1.0, 2600) / 3.0;
            std::vector<float> tc;
            CheapStats cs;
            double msC = 1e30;
            for (int k = 0; k < 10; ++k) {
                auto t0 = std::chrono::steady_clock::now();
                trace_cheap(st.sc, par, std::vector<double>(st.sc.solids.size(), detail), 4, tc, &cs);
                msC = std::min(msC, std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
            }
            Result cr;
            for (size_t i = 0; i + 3 * TRI <= tc.size(); i += 3 * TRI) {
                Patch p;
                p.first = (int)(cr.xz.size() / 2);
                p.count = 3;
                p.rgb = {tc[i + 2], tc[i + 3], tc[i + 4]};
                for (int k = 0; k < 3; ++k) {
                    cr.xz.push_back(tc[i + (size_t)k * TRI]);
                    cr.xz.push_back(tc[i + (size_t)k * TRI + 1]);
                }
                cr.patches.push_back(p);
            }
            Image ch(Wp, cx, 0, R);
            raster(ch, cr, 1.0, 2);
            double sumC = 0, sumR = 0;
            for (size_t i = 0; i < ch.rgb.size(); ++i) {
                sumC += ch.rgb[i];
                sumR += ref.rgb[i];
            }
            (void)sumC;
            (void)sumR;
            // the light that is drawn, added up from the triangles themselves (area x brightness, mean of the three channels)
            double drawnLight = 0;
            for (size_t i = 0; i + 3 * TRI <= tc.size(); i += 3 * TRI) {
                const float *a = &tc[i], *b = a + TRI, *c = b + TRI;
                const double ar = 0.5 * std::fabs((double)(b[0] - a[0]) * (c[1] - a[1]) - (double)(c[0] - a[0]) * (b[1] - a[1]));
                drawnLight += ar * ((double)a[2] + a[3] + a[4] + b[2] + b[3] + b[4] + c[2] + c[3] + c[4]) / 9.0;
            }
            std::printf("   %-32s cheap %.3f ms, %d triangles of surface, %d patches, %d rays; light on the floor %.0f%% of the rays' (%.4f)\n", st.name, msC, cs.pieces, cs.patches,
                        cs.rays, 100.0 * drawnLight / rayFloor, rayFloor);
            {
                // the same with the balls that are alike left to the renderer's one shared shape: what the CPU still does
                std::vector<float> tl;
                std::vector<int> balls;
                CheapStats ls;
                double msL = 1e30;
                for (int k = 0; k < 200; ++k) {
                    auto t0 = std::chrono::steady_clock::now();
                    trace_cheap(st.sc, par, std::vector<double>(st.sc.solids.size(), detail), 4, tl, &ls, &balls);
                    msL = std::min(msL, std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
                }
                if (!balls.empty())
                    std::printf("   %-32s   with the %zu ball(s) left to the renderer's shared shape: %.4f ms on the CPU for the whole scene\n", "", balls.size(), msL);
            }
            // pictures: rays | cheap, with the outline as shadow
            Image cov(Wp, cx, 0, R);
            std::vector<float> ot;
            silhouette_triangles(st.sc, par.sun, 0.0, ot);
            Result orr;
            for (size_t i = 0; i + 3 * TRI <= ot.size(); i += 3 * TRI) {
                Patch p;
                p.first = (int)(orr.xz.size() / 2);
                p.count = 3;
                p.rgb = {1, 1, 1};
                for (int k = 0; k < 3; ++k) {
                    orr.xz.push_back(ot[i + (size_t)k * TRI]);
                    orr.xz.push_back(ot[i + (size_t)k * TRI + 1]);
                }
                orr.patches.push_back(p);
            }
            raster(cov, orr, 1.0, 2);
            const V3 L = normalize(par.sun);
            std::vector<uint8_t> px((size_t)Wp * 2 * Wp * 4);
            for (int y = 0; y < Wp; ++y)
                for (int x = 0; x < Wp * 2; ++x) {
                    const Image& im = x < Wp ? ref : ch;
                    const size_t i = (size_t)y * Wp + (size_t)(x % Wp);
                    for (int c = 0; c < 3; ++c) {
                        const double shade = std::min(1.0, cov.rgb[i * 3]);
                        double v = (std::max(0.0, im.rgb[i * 3 + c]) + (-L.y) * (1.0 - shade) + 0.10) * 0.80 * 0.95;
                        v = v / (1.0 + 0.18 * v);
                        v = std::min(1.0, std::max(0.0, v));
                        v = v <= 0.0031308 ? 12.92 * v : 1.055 * std::pow(v, 1.0 / 2.4) - 0.055;
                        px[((size_t)y * Wp * 2 + x) * 4 + c] = (uint8_t)std::lround(v * 255.0);
                    }
                    px[((size_t)y * Wp * 2 + x) * 4 + 3] = 255;
                }
            write_png((std::string("out/beam-top-round-") + st.file + ".png").c_str(), Wp * 2, Wp, px.data());
        }
    }

    std::printf("3. colour on the floor (0 = white light only)\n");
    for (double ex : {1.0, 4.0}) {
        double v[3];
        int k = 0;
        for (double ch : {0.0, 0.015, 0.05}) {
            Solid s = chamfered_box(center, 0.35, half, ch * 2.0 * half);
            Params par;
            par.sun = {0.62, -0.62, 0.28};
            par.bands = 48;
            par.glass.exaggerate = ex;
            v[k++] = chroma(trace(s, par), 0, 0, 3.0);
        }
        std::printf("   dispersion x%.0f:  perfect box %.4f   chamfer 1.5%% %.4f   chamfer 5%% %.4f\n", ex, v[0], v[1], v[2]);
    }

    std::printf("4. pictures and time\n");
    struct Shot { const char* file; V3 sun; double yaw, chamfer, ex; int bands; };
    const Shot shots[] = {
        {"out/beam-top-perfect.png", {0.62, -0.62, 0.28}, 0.35, 0.0, 1.0, 64},
        {"out/beam-top-chamfer.png", {0.62, -0.62, 0.28}, 0.35, 0.015, 1.0, 64},
        {"out/beam-top-chamfer-x4.png", {0.62, -0.62, 0.28}, 0.35, 0.015, 4.0, 64},
        {"out/beam-top-lowsun-x4.png", {0.85, -0.38, -0.20}, 0.80, 0.015, 4.0, 64},
        {"out/beam-top-lowsun-perfect-x4.png", {0.85, -0.38, -0.20}, 0.80, 0.0, 4.0, 64},
    };
    for (const Shot& sh : shots) {
        Solid s = chamfered_box(center, sh.yaw, half, sh.chamfer * 2.0 * half);
        Params par;
        par.sun = sh.sun;
        par.bands = sh.bands;
        par.glass.exaggerate = sh.ex;
        auto t0 = std::chrono::steady_clock::now();
        Result r = trace(s, par);
        double ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
        std::vector<float> tri;
        to_triangles(r, tri);
        V3 L = normalize(sh.sun);
        save_top(sh.file, s, par, r, 1.2 * L.x / std::max(0.3, -L.y) * 0.6, 1.2 * L.z / std::max(0.3, -L.y) * 0.6, 2.6, 1400);
        std::printf("   %-36s %d bands: %.2f ms, %zu patches, %zu triangles\n", sh.file, sh.bands, ms, r.patches.size(), tri.size() / (3 * TRI));
    }
    std::printf("5. fast trace against the reference (64 bands traced in full)\n");
    for (const Case& c : cases) {
        for (double ex : {1.0, 4.0}) {
            Solid s = chamfered_box(center, c.yaw, half, c.chamfer * 2.0 * half);
            Params par;
            par.sun = c.sun;
            par.glass.exaggerate = ex;
            const double R = 4.0;
            const int Wd = 1536;
            const double detail = 2.0 * R / Wd;  // one pixel of the comparison picture
            auto t0 = std::chrono::steady_clock::now();
            Result ref = trace(s, par);
            double msRef = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
            std::vector<float> tri;
            FastStats st;
            double msFast = 1e30;
            for (int k = 0; k < 20; ++k) {
                t0 = std::chrono::steady_clock::now();
                trace_fast(s, par, detail, tri, &st);
                msFast = std::min(msFast, std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count());
            }
            Result fr;
            for (size_t i = 0; i + 3 * TRI <= tri.size(); i += 3 * TRI) {
                Patch p;
                p.first = (int)(fr.xz.size() / 2);
                p.count = 3;
                p.rgb = {tri[i + 2], tri[i + 3], tri[i + 4]};
                for (int k = 0; k < 3; ++k) {
                    fr.xz.push_back(tri[i + (size_t)k * TRI]);
                    fr.xz.push_back(tri[i + (size_t)k * TRI + 1]);
                }
                fr.patches.push_back(p);
            }
            Image a(Wd, 0, 0, R), b(Wd, 0, 0, R);
            raster(a, ref, 1.0, 2);
            raster(b, fr, 1.0, 2);
            double l1 = 0, tot = 0, worst = 0;
            for (size_t i = 0; i < a.rgb.size(); ++i) {
                double d = std::fabs(a.rgb[i] - b.rgb[i]);
                l1 += d;
                tot += std::fabs(a.rgb[i]);
                worst = std::max(worst, d);
            }
            bool ok = l1 / tot < 0.02;
            bad += ok ? 0 : 1;
            std::printf("   %-24s x%.0f: picture difference %.2f%% (worst pixel %.3f)   reference %.1f ms   fast %.3f ms   %d ways, %d coloured (%d smooth), %d follows, %zu triangles  %s\n", c.name, ex,
                        100.0 * l1 / tot, worst, msRef, msFast, st.ways, st.coloured, st.smooth, st.follows, tri.size() / (3 * TRI), ok ? "ok" : "FAIL");
        }
    }
    std::printf("receivers: every ray is kept on one receiver and one only (the rule of recv_plane.glsl / recv_lands.glsl, on the CPU)\n");
    {
        // a room with two walls meeting in a corner and three blocks; rays from anywhere inside it, any way
        const double pi = 3.14159265358979323846;
        std::vector<Half> walls = {Half{V3{0, 0, 1}, 1.2}, Half{V3{std::cos(0.2), 0, -std::sin(0.2)}, 1.5}};
        std::vector<Block> blocks(3);
        const double at[3][7] = {{-0.4, 0.07, -0.3, 0.17, 0.07, 0.17, 0.3}, {0.3, 0.20, 0.1, 0.12, 0.20, 0.12, -0.5}, {0.5, 0.09, -0.7, 0.22, 0.09, 0.10, 0.9}};
        for (int b = 0; b < 3; ++b) {
            blocks[(size_t)b].c = {at[b][0], at[b][1], at[b][2]};
            blocks[(size_t)b].ax[0] = {std::cos(at[b][6]), 0, -std::sin(at[b][6])};
            blocks[(size_t)b].ax[1] = {0, 1, 0};
            blocks[(size_t)b].ax[2] = {std::sin(at[b][6]), 0, std::cos(at[b][6])};
            for (int k = 0; k < 3; ++k) blocks[(size_t)b].h[k] = at[b][3 + k];
        }
        const Stage st = make_stage(0.0, walls, 3.0, blocks);
        uint64_t seed = 88172645463325252ull;
        auto rnd = [&]() {
            seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
            return (double)(seed >> 11) / 9007199254740992.0;
        };
        int rays = 0, none = 0, both = 0, wrong = 0, lost = 0, onBlocks = 0, onRoom = 0;
        while (rays < 400000) {
            const V3 E{-1.4 + 3.4 * rnd(), 0.01 + 1.2 * rnd(), -1.1 + 3.0 * rnd()};
            bool inside = true;
            for (const Half& h : st.room) inside = inside && dot(h.a, E) + h.b > 0.02;
            for (const Block& B : st.blocks) {
                bool in = true;
                for (int k = 0; k < 3; ++k) in = in && std::fabs(dot(B.ax[k], E - B.c)) < B.h[k] + 0.02;
                inside = inside && !in;
            }
            if (!inside) continue;
            const double z = 2.0 * rnd() - 1.0, ph = 2.0 * pi * rnd(), rr = std::sqrt(1.0 - z * z);
            const V3 D{rr * std::cos(ph), z, rr * std::sin(ph)};
            ++rays;
            // the truth: the first block the ray enters, else the first plane of the room (if that is not where the walls end)
            int truth = -2;  // -2 nothing, -1 the room, i a block
            double tTruth = 1e30;
            for (size_t j = 0; j < st.room.size(); ++j) {
                const double den = dot(st.room[j].a, D);
                if (den > -1e-12) continue;
                const double t = -(dot(st.room[j].a, E) + st.room[j].b) / den;
                if (t < tTruth) {
                    tTruth = t;
                    truth = j + 1 < st.room.size() ? -1 : -2;
                }
            }
            for (size_t i = 0; i < st.blocks.size(); ++i) {
                const Block& B = st.blocks[i];
                double t0 = 0, t1 = tTruth;
                bool miss = false;
                for (int k = 0; k < 3; ++k) {
                    const double o = dot(B.ax[k], E - B.c), d = dot(B.ax[k], D);
                    if (std::fabs(d) < 1e-12) {
                        miss = miss || std::fabs(o) > B.h[k];
                        continue;
                    }
                    const double ta = (-B.h[k] - o) / d, tb = (B.h[k] - o) / d;
                    t0 = std::max(t0, std::min(ta, tb));
                    t1 = std::min(t1, std::max(ta, tb));
                }
                if (!miss && t0 < t1) {
                    tTruth = t0;
                    truth = (int)i;
                }
            }
            // the rule: once dropped on the room, once on the blocks; which of the two is kept?
            // (the sun is put where no block can be in its way: straight above)
            const V3 sun{0, -1, 0};
            bool kept[2] = {false, false};
            int owner[2] = {-1, -1};
            for (int kind = 0; kind < 2; ++kind) {
                Plane N;
                if (!stage_choose(st, kind, E, D, N, owner[kind])) continue;
                const double heading = -dot(N.n, D);
                if (heading <= 0) continue;
                const double t = (dot(N.n, E) - N.o) / heading;
                if (t < 0) continue;
                const V3 X = E + D * t;
                bool clear = stage_on(st, X, owner[kind]);
                for (size_t i = 0; i < st.blocks.size() && clear; ++i)
                    if (stage_through(st.blocks[i], E, X - E, 0.9999)) clear = false;
                (void)sun;
                kept[kind] = clear;
            }
            const int count = (kept[0] ? 1 : 0) + (kept[1] ? 1 : 0);
            if (count == 2) ++both;
            if (count == 0 && truth != -2) ++lost;
            if (count == 0) ++none;
            if (count == 1) {
                const int got = kept[1] ? owner[1] : -1;
                if (got != truth) ++wrong;
                (got < 0 ? onRoom : onBlocks)++;
            }
        }
        // (a ray that grazes an edge of a block within the rule's own tolerance, a tenth of a millimetre, may go either
        // way or both: a few in 400 000)
        const bool ok = both + lost + wrong <= 40;
        bad += ok ? 0 : 1;
        std::printf("   %d rays: %d kept on the room, %d on a block, %d on nothing (out of the room); kept twice %d, lost %d, on the wrong thing %d  %s\n", rays, onRoom, onBlocks,
                    none, both, lost, wrong, ok ? "ok" : "FAIL");
    }
    std::printf(bad ? "FAILED: %d\n" : "ALL OK\n", bad);
    return bad ? 1 : 0;
}
