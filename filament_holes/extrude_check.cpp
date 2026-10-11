// scratch: the twin's light of extruded shapes next to rays walked one by one
#include "beam_glass.h"
#include <chrono>
#include <cstdio>
#include <string>
#include <zlib.h>
using namespace beam;
static bool write_png(const char* path, int w, int h, const uint8_t* rgba) {
    std::vector<uint8_t> raw((size_t)h * (w * 4 + 1));
    for (int y = 0; y < h; ++y) { raw[(size_t)y * (w * 4 + 1)] = 0; std::memcpy(raw.data() + (size_t)y * (w * 4 + 1) + 1, rgba + (size_t)y * w * 4, (size_t)w * 4); }
    uLongf bound = compressBound((uLong)raw.size()); std::vector<uint8_t> z(bound);
    if (compress(z.data(), &bound, raw.data(), (uLong)raw.size()) != Z_OK) return false;
    FILE* f = std::fopen(path, "wb"); if (!f) return false;
    auto chunk = [&](const char* tag, const uint8_t* data, uint32_t n) { uint32_t be = __builtin_bswap32(n); std::fwrite(&be, 4, 1, f); std::fwrite(tag, 1, 4, f); if (n) std::fwrite(data, 1, n, f);
        uint32_t crc = crc32(0, (const Bytef*)tag, 4); if (n) crc = crc32(crc, data, n); crc = __builtin_bswap32(crc); std::fwrite(&crc, 4, 1, f); };
    const uint8_t sig[8] = {137, 80, 78, 71, 13, 10, 26, 10}; std::fwrite(sig, 1, 8, f);
    uint8_t ihdr[13]; uint32_t bw = __builtin_bswap32((uint32_t)w), bh = __builtin_bswap32((uint32_t)h); std::memcpy(ihdr, &bw, 4); std::memcpy(ihdr + 4, &bh, 4); ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = ihdr[11] = ihdr[12] = 0;
    chunk("IHDR", ihdr, 13); chunk("IDAT", z.data(), (uint32_t)bound); chunk("IEND", nullptr, 0); std::fclose(f); return true;
}
struct Image {
    int W; double cx, cz, R; std::vector<double> rgb;
    Image(int w, double cx_, double cz_, double r) : W(w), cx(cx_), cz(cz_), R(r), rgb((size_t)w * w * 3, 0.0) {}
    double px() const { return 2.0 * R / W; }
};
// triangles with a colour per corner, S x S samples per pixel
static void raster(Image& im, const std::vector<float>& tri, int S = 3) {
    const double px = im.px(), x0 = im.cx - im.R, z0 = im.cz - im.R;
    for (size_t i = 0; i + 3 * TRI <= tri.size(); i += 3 * TRI) {
        const float *a = &tri[i], *b = a + TRI, *c = b + TRI;
        const double den = (double)(b[0] - a[0]) * (c[1] - a[1]) - (double)(c[0] - a[0]) * (b[1] - a[1]);
        if (std::fabs(den) < 1e-18) continue;
        const double lx = std::min(a[0], std::min(b[0], c[0])), hx = std::max(a[0], std::max(b[0], c[0])), lz = std::min(a[1], std::min(b[1], c[1])), hz = std::max(a[1], std::max(b[1], c[1]));
        const int ix0 = std::max(0, (int)std::floor((lx - x0) / px)), ix1 = std::min(im.W - 1, (int)std::floor((hx - x0) / px));
        const int iz0 = std::max(0, (int)std::floor((lz - z0) / px)), iz1 = std::min(im.W - 1, (int)std::floor((hz - z0) / px));
        for (int iz = iz0; iz <= iz1; ++iz)
            for (int ix = ix0; ix <= ix1; ++ix) {
                double acc[3] = {0, 0, 0};
                for (int sz = 0; sz < S; ++sz)
                    for (int sx = 0; sx < S; ++sx) {
                        const double x = x0 + (ix + (sx + 0.5) / S) * px, z = z0 + (iz + (sz + 0.5) / S) * px;
                        const double w1 = ((x - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (z - a[1])) / den, w2 = ((b[0] - a[0]) * (z - a[1]) - (x - a[0]) * (b[1] - a[1])) / den, w0 = 1 - w1 - w2;
                        if (w0 < 0 || w1 < 0 || w2 < 0) continue;
                        for (int k = 0; k < 3; ++k) acc[k] += w0 * a[2 + k] + w1 * b[2 + k] + w2 * c[2 + k];
                    }
                for (int k = 0; k < 3; ++k) im.rgb[((size_t)iz * im.W + ix) * 3 + k] += acc[k] / (S * S);
            }
    }
}
// rays, one by one
struct Walk { const Solid* so; const Params* par; double n; Image* im; int ch; bool again, split; double floorFlux = 0; };
static void w_in(Walk& w, V3 o, V3 d, double flux, int bounces, int entries);
static void w_air(Walk& w, V3 o, V3 d, double flux, int entries) {
    double t; V3 n; int g;
    if (w.again && entries < 4 && detail::meet(*w.so, o, d, t, n, g)) {
        const V3 p = o + d * t; const double ci = -dot(n, d), Rf = fresnel(ci, 1.0, w.n);
        V3 in; refract(d, n, 1.0 / w.n, in);
        if (w.split) w_air(w, p, d + n * (2.0 * ci), flux * Rf, 99);
        w_in(w, p, in, flux * (1.0 - Rf), 0, entries + 1);
        return;
    }
    if (d.y < -1e-9) {
        const double tt = std::max(0.0, (w.par->floorY - o.y) / d.y); const V3 q = o + d * tt; Image& im = *w.im;
        const int ix = (int)std::floor((q.x - (im.cx - im.R)) / im.px()), iz = (int)std::floor((q.z - (im.cz - im.R)) / im.px());
        w.floorFlux += flux;
        if (ix >= 0 && iz >= 0 && ix < im.W && iz < im.W) im.rgb[((size_t)iz * im.W + ix) * 3 + (size_t)w.ch] += flux / (im.px() * im.px());
    }
}
static void w_in(Walk& w, V3 o, V3 d, double flux, int bounces, int entries) {
    double t; V3 nj; int best;
    if (!detail::leave(*w.so, o, d, -1, t, nj, best)) return;
    const V3 p = o + d * t; const double ci = dot(nj, d), Rf = fresnel(ci, w.n, 1.0);
    if (Rf < 1.0) { V3 out; refract(d, nj * -1.0, w.n, out); w_air(w, p, out, flux * (1.0 - Rf), entries); }
    if (bounces < 12 && (Rf >= 1.0 || (w.split && flux * Rf > 1e-9 && Rf > 0.02 && bounces < 4))) w_in(w, p, d - nj * (2.0 * ci), flux * Rf, bounces + 1, entries);
}
static double walk(const Solid& so, const Params& par, double n, Image& im, int ch, int raysPerSide, bool again, bool split, int entry = 0) {
    const V3 L = normalize(par.sun); V3 up = std::fabs(L.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0}; V3 U = normalize(cross(up, L)), V = cross(L, U);
    V3 c; double r; extrude_bound(so, c, r);
    Walk w{&so, &par, n, &im, ch, again, split};
    const double du = 2 * r / raysPerSide;
    for (int iv = 0; iv < raysPerSide; ++iv)
        for (int iu = 0; iu < raysPerSide; ++iu) {
            const V3 o = c + U * (-r + (iu + 0.5) * du) + V * (-r + (iv + 0.5) * du) - L * (r + 10.0);
            double t; V3 nn; int g;
            if (!detail::meet(so, o, L, t, nn, g)) continue;
            if (entry == 1 && g < (int)so.shape->edges.size()) continue;
            if (entry == 2 && g >= (int)so.shape->edges.size()) continue;
            const V3 p = o + L * t; const double ci = -dot(nn, L), Rf = fresnel(ci, 1.0, n), flux = du * du;
            if (par.outerReflection) w_air(w, p, L + nn * (2.0 * ci), flux * Rf, 99);
            V3 in; refract(L, nn, 1.0 / n, in);
            w_in(w, p, in, flux * (1.0 - Rf), 0, 1);
        }
    return w.floorFlux;
}
static void tone(const Image& im, const Image& cov, double sunDown, uint8_t* px, int stride, int xoff) {
    for (int y = 0; y < im.W; ++y)
        for (int x = 0; x < im.W; ++x) {
            const size_t i = (size_t)y * im.W + x;
            for (int c = 0; c < 3; ++c) {
                const double shade = std::min(1.0, cov.rgb[i * 3]);
                double v = (std::max(0.0, im.rgb[i * 3 + c]) + sunDown * (1.0 - shade) + 0.10) * 0.80 * 0.95;
                v = v / (1.0 + 0.18 * v); v = std::min(1.0, std::max(0.0, v));
                v = v <= 0.0031308 ? 12.92 * v : 1.055 * std::pow(v, 1.0 / 2.4) - 0.055;
                px[((size_t)y * stride + xoff + x) * 4 + c] = (uint8_t)std::lround(v * 255.0);
            }
            px[((size_t)y * stride + xoff + x) * 4 + 3] = 255;
        }
}
int main(int argc, char** argv) {
    const char* want = argc > 1 ? argv[1] : nullptr;
    const int mode = argc > 2 ? std::atoi(argv[2]) : 0;
    const double Rr = argc > 3 ? std::atof(argv[3]) : 2.4, cx = argc > 4 ? std::atof(argv[4]) : 0.9;
    struct Set { const char* name; Solid local; V3 X, Y, Z; double lift; V3 sun; };
    const double c = 0.03;
    std::vector<Set> sets;
    const V3 sun1{0.62, -0.62, 0.28}, sun2{0.92, -0.42, 0.22};
    { Outline ol; ol.h = 0.12; ol.chamfer = c; ol.move(0, 0); ol.line(1, 0); ol.line(1, 0.7); ol.line(0, 0.7); ol.finish();
      sets.push_back({"box-flat", extruded(ol), normalize(V3{0.9, 0, -0.436}), normalize(V3{-0.436, 0, -0.9}), {0, 1, 0}, 0.1205, sun1});
      sets.push_back({"box-tilt", extruded(ol), normalize(V3{0.9, 0.2, -0.436}), normalize(V3{-0.436, 0.3, -0.9}), {0.1, 1, 0.3}, 0.5, sun1}); }
    sets.push_back({"star-flat", star(5, 0.5, 0.225, 0.12, c), {1, 0, 0}, {0, 0, -1}, {0, 1, 0}, 0.1205, sun1});
    sets.push_back({"star-up", star(5, 0.5, 0.225, 0.12, c), normalize(V3{0.8, 0, -0.6}), {0, 1, 0}, normalize(V3{0.6, 0, 0.8}), 0.4051, sun1});
    sets.push_back({"washer-flat", washer(0.5, 0.25, 0.11, c), {1, 0, 0}, {0, 0, -1}, {0, 1, 0}, 0.1105, sun2});
    sets.push_back({"washer-up", washer(0.5, 0.25, 0.11, c), normalize(V3{0.5, 0, -0.866}), {0, 1, 0}, normalize(V3{0.866, 0, 0.5}), 0.5005, sun1});
    sets.push_back({"A-up", letter('A', 0.15, c), normalize(V3{0.7, 0, -0.714}), {0, 1, 0}, normalize(V3{0.714, 0, 0.7}), 0.5005, sun1});
    sets.push_back({"B-up", letter('B', 0.15, c), normalize(V3{0.3, 0, -0.954}), {0, 1, 0}, normalize(V3{0.954, 0, 0.3}), 0.5005, sun2});
    sets.push_back({"M-flat", letter('M', 0.15, c), {1, 0, 0}, {0, 0, -1}, {0, 1, 0}, 0.1505, sun1});
    sets.push_back({"O-up", letter('O', 0.15, c), normalize(V3{0.9, 0, -0.436}), {0, 1, 0}, normalize(V3{0.436, 0, 0.9}), 0.5005, sun2});
    sets.push_back({"R-tumble", letter('R', 0.15, c), normalize(V3{0.6, 0.5, -0.62}), normalize(cross(normalize(V3{0.3, 0.2, 0.93}), normalize(V3{0.6, 0.5, -0.62}))), normalize(V3{0.3, 0.2, 0.93}), 0.9, sun1});
    for (const Set& st : sets) {
        if (want && std::string(st.name).find(want) == std::string::npos) continue;
        V3 Z = normalize(st.Z), X = normalize(st.X - Z * dot(st.X, Z)), Y = cross(Z, X);
        V3 R[3] = {X, Y, Z};
        const Solid so = placed(st.local, R, 1.0, {0, st.lift, 0});
        Params par; par.sun = st.sun; par.glass.exaggerate = 10;
        const int Wp = 900;
        if (mode) par.outerReflection = false;
        const V3 L = normalize(par.sun);
        const double nm3[3] = {640, 550, 460};
        Image ref(Wp, cx, 0, Rr), ref2(Wp, cx, 0, Rr);
        double rf = 0, rf2 = 0;
        for (int ch = 0; ch < 3; ++ch) { rf += walk(so, par, par.glass.n(nm3[ch]), ref, ch, 2200, false, false, mode) / 3; rf2 += walk(so, par, par.glass.n(nm3[ch]), ref2, ch, 2200, true, true, mode) / 3; }
        std::vector<float> all, sharp, fam[7], sil;
        auto t0 = std::chrono::steady_clock::now();
        extrude_light(so, par, all, nullptr, -1);
        const double ms = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
        const int ids[7] = {100, detail::EX_MIRROR, detail::EX_WALL, detail::EX_GLINT, detail::EX_FOOT, detail::EX_CUT, 0};
        auto light = [](const std::vector<float>& tc) { double s = 0; for (size_t i = 0; i + 3 * TRI <= tc.size(); i += 3 * TRI) { const float *a = &tc[i], *b = a + TRI, *c = b + TRI;
            const double ar = 0.5 * std::fabs((double)(b[0] - a[0]) * (c[1] - a[1]) - (double)(c[0] - a[0]) * (b[1] - a[1])); s += ar * ((double)a[2] + a[3] + a[4] + b[2] + b[3] + b[4] + c[2] + c[3] + c[4]) / 9.0; } return s; };
        double part[6];
        for (int f = 0; f < 6; ++f) { extrude_light(so, par, fam[f], nullptr, ids[f]); part[f] = light(fam[f]); if (f < 4 && (mode == 0 || (mode == 1 && f < 2) || (mode == 2 && f == 2))) sharp.insert(sharp.end(), fam[f].begin(), fam[f].end()); }
        t0 = std::chrono::steady_clock::now();
        extrude_silhouette(so, par.sun, 0.0, sil);
        const double msS = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - t0).count();
        Image tw(Wp, cx, 0, Rr), tws(Wp, cx, 0, Rr), cov(Wp, cx, 0, Rr);
        raster(tw, mode ? sharp : all); raster(tws, sharp); raster(cov, sil);
        // the shadow against the rays' own: where no ray of the sun reaches the floor
        double shadowTwin = 0, covMax = 0;
        for (size_t i = 0; i < cov.rgb.size(); i += 3) { shadowTwin += cov.rgb[i] * cov.px() * cov.px(); covMax = std::max(covMax, cov.rgb[i]); }
        // the true shadow area: the solid's outline along the sun = rays that meet it, times |L.y| ... per unit of floor: area = flux in / down
        double fluxIn = 0; { V3 cc; double r; extrude_bound(so, cc, r); V3 up = std::fabs(L.y) < 0.9 ? V3{0, 1, 0} : V3{1, 0, 0}; V3 U = normalize(cross(up, L)), V = cross(L, U); const int N = 1500; const double du = 2 * r / N;
            for (int iv = 0; iv < N; ++iv) for (int iu = 0; iu < N; ++iu) { const V3 o = cc + U * (-r + (iu + 0.5) * du) + V * (-r + (iv + 0.5) * du) - L * (r + 10.0); double t; V3 nn; int g; if (detail::meet(so, o, L, t, nn, g)) fluxIn += du * du; } }
        // picture difference, coarse
        const int B = 6; double l1 = 0, tot = 0, l1b = 0, totb = 0;
        for (int z = 0; z + B <= Wp; z += B) for (int x = 0; x + B <= Wp; x += B) for (int ch = 0; ch < 3; ++ch) { double a = 0, b = 0, d2 = 0;
            for (int dz = 0; dz < B; ++dz) for (int dx = 0; dx < B; ++dx) { const size_t i = ((size_t)(z + dz) * Wp + (x + dx)) * 3 + ch; a += ref.rgb[i]; b += tws.rgb[i]; d2 += ref2.rgb[i]; }
            l1 += std::fabs(a - b); tot += a; l1b += std::fabs(d2 - b); totb += d2; }
        const double px2 = tw.px() * tw.px();
        double sumS = 0; for (double v : tws.rgb) sumS += v * px2 / 3;
        std::printf("%-12s twin %.2f ms + shadow %.2f ms, %zu triangles | light: rays %.4f (with re-entry and part mirrors %.4f), twin sharp %.4f = %.1f%% (window %.4f mirror %.4f wall %.4f glint %.4f) + foot %.4f cut %.4f | picture %.1f%% from rays (%.1f%% from the full walk) | shadow area %.4f vs %.4f, most cover %.2f\n",
                    st.name, ms, msS, all.size() / (3 * TRI), rf, rf2, sumS, 100 * sumS / rf, part[0], part[1], part[2], part[3], part[4], part[5], 100 * l1 / tot, 100 * l1b / totb, shadowTwin, fluxIn / -L.y, covMax);
        std::vector<uint8_t> px((size_t)Wp * 3 * Wp * 4);
        tone(ref2, cov, -L.y, px.data(), Wp * 3, 0); tone(ref, cov, -L.y, px.data(), Wp * 3, Wp); tone(tw, cov, -L.y, px.data(), Wp * 3, 2 * Wp);
        write_png((std::string("out/ext-top-") + st.name + ".png").c_str(), Wp * 3, Wp, px.data());
    }
}
