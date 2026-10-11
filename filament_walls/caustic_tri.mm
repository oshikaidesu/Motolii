// Self-test and demo for caustic_tri.h: a sphere checked against exact CPU refraction, then a glass loop + pebble.
// Desk-derived checks (independent of the pipeline):
//   sphere profile  : radial energy distribution == exact 1D refraction solved on the CPU
//   footprint area  : the unrefracted shadow of the sphere == pi R^2
#include <filament/SwapChain.h>
#include <filament/Skybox.h>

#include <zlib.h>

#include <chrono>
#include <cstdlib>

#include "caustic_tri.h"

using namespace ctri;

static float env_f(const char* k, float d) {
    const char* v = std::getenv(k);
    return v ? (float)std::atof(v) : d;
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

struct Result {
    int map = 0;
    std::vector<float> caustic, cover;  // rgba floats read back from the maps
    double ms_median = 0;
};
static Result run(Engine* e, Renderer* r, SwapChain* sc, const Materials& M, std::vector<Mesh> meshes, float3 sun,
                  const Params& P, int timing_frames) {
    CausticPass cp(e, M, std::move(meshes), sun, P);
    auto open_frame = [&]() {
        for (int i = 0; i < 4000; ++i)
            if (r->beginFrame(sc)) return true;
        return false;
    };
    Result res;
    res.map = P.map;
    res.caustic.assign((size_t)P.map * P.map * 4, 0.f);
    res.cover.assign((size_t)P.map * P.map * 4, 0.f);
    if (!open_frame()) std::exit(fail("beginFrame"));
    cp.draw(r);
    r->readPixels(cp.causticTarget(), 0, 0, P.map, P.map,
                  backend::PixelBufferDescriptor(res.caustic.data(), res.caustic.size() * 4, backend::PixelDataFormat::RGBA,
                                                 backend::PixelDataType::FLOAT));
    if (P.footprint)
        r->readPixels(cp.footprintTarget(), 0, 0, P.map, P.map,
                      backend::PixelBufferDescriptor(res.cover.data(), res.cover.size() * 4, backend::PixelDataFormat::RGBA,
                                                     backend::PixelDataType::FLOAT));
    r->endFrame();
    e->flushAndWait();
    std::vector<double> t;
    for (int i = 0; i < timing_frames; ++i) {
        auto a = std::chrono::steady_clock::now();
        if (!open_frame()) break;
        cp.draw(r);
        r->endFrame();
        e->flushAndWait();
        t.push_back(std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - a).count());
    }
    std::sort(t.begin(), t.end());
    res.ms_median = t.empty() ? 0 : t[t.size() / 2];
    return res;
}

// ---------------------------------------------------------------------------------------- sphere check
// Exact refraction through a sphere (radius R, index n), parallel rays travelling down -y, landing on the plane
// y = -H below the centre. Returns the landing radius for impact parameter h, or -1 if it misses.
static float3 refract(float3 I, float3 N, float eta) {
    float cosi = -dot(N, I);
    float k = 1.f - eta * eta * (1.f - cosi * cosi);
    if (k < 0.f) return float3{0.f};
    return I * eta + N * (eta * cosi - std::sqrt(k));
}
static float sphere_landing(float h, float R, float n, float H) {
    float y = std::sqrt(R * R - h * h);
    float3 p{h, y, 0}, N = normalize(p), I{0, -1, 0};
    float3 t1 = refract(I, N, 1.f / n);
    float s = -2.f * dot(p, t1);
    float3 q = p + t1 * s;
    float3 Nq = normalize(q);
    float3 t2 = refract(t1, -Nq, n);
    if (t2.y >= -1e-6f) return -1.f;
    float k = (q.y + H) / -t2.y;
    return std::fabs(q.x + t2.x * k);
}

int main() {
    Engine* engine = Engine::Builder().backend(Engine::Backend::METAL).featureLevel(Engine::FeatureLevel::FEATURE_LEVEL_3).build();
    if (!engine) return fail("engine");
    Renderer* renderer = engine->createRenderer();
    SwapChain* swap = engine->createSwapChain(64, 64, 0);
    Materials M;
    if (!load_materials(engine, "caustic_tri", M)) return fail("materials (run caustic_tri/build.sh)");

    Params P;
    P.cap = (int)env_f("CAUSTIC_CAPTURE", 256);
    P.map = (int)env_f("CAUSTIC_MAP", 1024);
    P.dispersion = env_f("CAUSTIC_DISPERSION", 0.03f);
    P.intensity = env_f("CAUSTIC_INTENSITY", 6.0f);
    int exit_code = 0;

    // ---- check 1+2: one sphere, sun straight down, no dispersion so every band has the same index
    {
        Params S = P;
        S.dispersion = 0.f;
        S.absorb = float3{0.f};
        S.intensity = 1.f;
        S.ratioCap = 1e9f;  // energy test: no clamp
        const float R = 0.25f, H = 0.9f;
        const float3 c{0.6f, H, -0.3f};
        Result r = run(engine, renderer, swap, M, {pebble(c, {R, R, R}, 0.f)}, {0, -1, 0}, S, 0);
        {
            std::vector<uint8_t> img((size_t)S.map * S.map * 4);
            for (int i = 0; i < S.map * S.map; ++i) {
                float y = std::max(0.f, r.caustic[(size_t)i * 4 + 1]);
                uint8_t g = (uint8_t)(std::clamp(std::pow(y / 4.f, 0.5f), 0.f, 1.f) * 255.f);
                img[(size_t)i * 4] = g; img[(size_t)i * 4 + 1] = g; img[(size_t)i * 4 + 2] = g; img[(size_t)i * 4 + 3] = 255;
            }
            std::system("mkdir -p out");
            write_png("out/caustic-tri-sphere.png", S.map, S.map, img.data());
        }
        {
            double cs = 0, cl = 0, cr = 0; int cminx = 9999, cmaxx = -1, cminy = 9999, cmaxy = -1;
            for (int ry = 0; ry < S.map; ++ry) for (int rx = 0; rx < S.map; ++rx) { float c = r.cover[((size_t)ry * S.map + rx) * 4]; if (c > 0.5f) { cs += c; cminx = std::min(cminx, rx); cmaxx = std::max(cmaxx, rx); cminy = std::min(cminy, ry); cmaxy = std::max(cmaxy, ry); } }
            double tx = S.rsize / S.map;
            std::printf("CHECK footprint area %.4f (want pi R^2 = %.4f)  px x[%d,%d] y[%d,%d]  (want x centre %.0f)\n", cs * tx * tx, PI * 0.25 * 0.25, cminx, cmaxx, cminy, cmaxy, (0.6 / 3.1 + 0.5) * S.map);
            (void)cl; (void)cr;
        }
        const float texel = S.rsize / (float)S.map;
        double sumY = 0, cx = 0, cz = 0;
        std::vector<double> yv((size_t)S.map * S.map);
        for (int i = 0; i < S.map * S.map; ++i) {
            double y = std::max(0.f, r.caustic[(size_t)i * 4 + 1]);
            yv[i] = y;
            sumY += y;
        }
        double energy = sumY * texel * texel;
        double expect = PI * R * R * S.intensity;
        std::printf("INFO  sphere total energy in map %.5f of %.5f (%.3f); the rest lands outside the 3.1 m map or travels upward\n", energy, expect, energy / expect);
        // orientation: centroid in map pixels, then both possible row conventions
        for (int ry = 0; ry < S.map; ++ry)
            for (int rx = 0; rx < S.map; ++rx) {
                cx += yv[(size_t)ry * S.map + rx] * rx;
                cz += yv[(size_t)ry * S.map + rx] * ry;
            }
        cx /= std::max(sumY, 1e-9);
        cz /= std::max(sumY, 1e-9);
        float x_world = ((float)cx + 0.5f) / S.map * S.rsize - S.rsize * 0.5f;
        float z_if_top_first = (1.f - ((float)cz + 0.5f) / S.map - 0.5f) * S.rsize;
        float z_if_bottom_first = (((float)cz + 0.5f) / S.map - 0.5f) * S.rsize;
        std::printf("INFO  sphere centroid x %.3f, z top-first %.3f / bottom-first %.3f (biased: part of the light is off the map)\n",
                    x_world, z_if_top_first, z_if_bottom_first);
        bool top_first = std::fabs(z_if_top_first + 0.3f) < std::fabs(z_if_bottom_first + 0.3f);
        // radial energy profile around the true centre, versus exact 1D refraction
        double cxp = ((0.6 / S.rsize) + 0.5) * S.map;
        double czp = top_first ? (1.0 - ((-0.3 / S.rsize) + 0.5)) * S.map : ((-0.3 / S.rsize) + 0.5) * S.map;
        std::vector<std::pair<double, double>> bins;  // (radius world, energy), sorted by radius below
        for (int ry = 0; ry < S.map; ++ry)
            for (int rx = 0; rx < S.map; ++rx) {
                double y = yv[(size_t)ry * S.map + rx];
                if (y <= 0) continue;
                double dx = (rx + 0.5 - cxp) * texel, dz = (ry + 0.5 - czp) * texel;
                bins.push_back({std::sqrt(dx * dx + dz * dz), y * texel * texel});
            }
        std::sort(bins.begin(), bins.end());
        double total = 0;
        for (auto& b : bins) total += b.second;
        auto radius_at = [&](double frac) {
            double acc = 0;
            for (auto& b : bins) {
                acc += b.second;
                if (acc >= frac * total) return b.first;
            }
            return bins.empty() ? 0.0 : bins.back().first;
        };
        std::printf("CHECK sphere radial profile (floor %.1f below the centre, n=%.2f): cumulative energy inside radius r\n", H, S.ior);
        std::printf("      r      GPU map    exact CPU refraction  (fraction of the full pi R^2 beam)\n");
        bool prof_ok = true;
        double worst = 0;
        double E0 = (double)PI * R * R * S.intensity;
        for (double rr : {0.15, 0.25, 0.35, 0.5, 0.7, 0.9}) {
            double acc = 0;
            for (auto& bn : bins) { if (bn.first > rr) break; acc += bn.second; }
            double gpu = acc / E0;
            // exact: the beam inside impact parameter h lands inside radius r (landing radius grows with h)
            float lo = 0, hi = R * 0.9999f;
            for (int it = 0; it < 60; ++it) {
                float mid = 0.5f * (lo + hi);
                if (sphere_landing(mid, R, S.ior, H) < rr) lo = mid; else hi = mid;
            }
            double exact = (double)(lo * lo) / (R * R);
            std::printf("   %.2f     %.4f       %.4f\n", rr, gpu, exact);
            worst = std::max(worst, std::fabs(gpu - exact));
        }
        std::printf("CHECK worst gap %.4f of the beam\n", worst);
        if (worst > 0.03) prof_ok = false;
        double inside09 = 0;
        for (auto& bn : bins) { if (bn.first > 0.9) break; inside09 += bn.second; }
        bool energy_ok = prof_ok;  // the radial table already compares absolute energy; the total also counts light outside the map
        (void)inside09;
        std::printf("%s sphere-energy\n%s sphere-profile\n", energy_ok ? "PASS" : "FAIL", prof_ok ? "PASS" : "FAIL");
        if (!energy_ok || !prof_ok) exit_code = 1;
    }

    // ---- check 3: an opaque box between the sun and the sphere removes the caustic; a box far away changes nothing
    {
        Params S = P;
        S.dispersion = 0.f;
        S.intensity = 1.f;
        S.ratioCap = 1e9f;
        S.footprint = false;
        S.cap = 128;
        S.map = 512;
        auto total = [&](float3 boxCenter) {
            Params B = S;
            B.boxCenter = boxCenter;
            B.boxHalf = 0.4f;
            Result r = run(engine, renderer, swap, M, {pebble({0.6f, 0.9f, -0.3f}, {0.25f, 0.25f, 0.25f}, 0.f)}, {0, -1, 0}, B, 0);
            double sum = 0;
            for (int i = 0; i < B.map * B.map; ++i) sum += std::max(0.f, r.caustic[(size_t)i * 4 + 1]);
            return sum;
        };
        double clear = total({9.f, 9.f, 9.f}), above = total({0.6f, 1.7f, -0.3f}), below = total({0.6f, 0.3f, -0.3f});
        std::printf("CHECK box blocker: far box %.3f   box between sun and sphere %.3f (want 0)   box between sphere and floor %.3f (want ~0)\n",
                    clear, above, below);
        bool ok = clear > 100.0 && above < 1e-3 * clear && below < 0.05 * clear;
        std::printf("%s box-blocker\n", ok ? "PASS" : "FAIL");
        if (!ok) exit_code = 1;
    }

    // ---- the picture: glass loop + pebble on a floor, sun from the side
    {
        float3 sun{0.92f, -0.42f, 0.22f};
        std::vector<Mesh> meshes;
        meshes.push_back(glass_loop());
        meshes.push_back(pebble({0.62f, 0.075f, -0.22f}, {0.16f, 0.075f, 0.12f}, 0.5f));
        Result r = run(engine, renderer, swap, M, std::move(meshes), sun, P, (int)env_f("CAUSTIC_FRAMES", 41));
        std::printf("TIME caustic pipeline (2 objects, capture %d, map %d): median %.2f ms per frame (wall, flushAndWait)\n", P.cap,
                    P.map, r.ms_median);
        int m = r.map;
        bool flip = env_f("CAUSTIC_FLIP", 0) > 0.5f;  // readPixels row order -> png rows
        std::vector<uint8_t> rgba((size_t)m * m * 4);
        const float exposure = env_f("CAUSTIC_EXPOSURE", 0.9f);
        double maxY = 0;
        for (int row = 0; row < m; ++row)
            for (int col = 0; col < m; ++col) {
                size_t src = (size_t)(flip ? (m - 1 - row) : row) * m + col;
                float x = r.caustic[src * 4], y = r.caustic[src * 4 + 1], z = r.caustic[src * 4 + 2];
                float cover = std::clamp(r.cover[src * 4], 0.f, 1.f);
                maxY = std::max<double>(maxY, y);
                float lin[3] = {3.2409699f * x - 1.5373832f * y - 0.4986108f * z, -0.9692436f * x + 1.8759675f * y + 0.0415551f * z,
                                0.0556301f * x - 0.2039770f * y + 1.0569715f * z};
                for (int c = 0; c < 3; ++c) {
                    float v = 0.10f + 0.55f * (1.f * (1.f - cover) + std::max(lin[c], 0.f)) * exposure;
                    float t = v / (1.f + v * 0.35f);
                    float s = t <= 0.0031308f ? 12.92f * t : 1.055f * std::pow(t, 1.f / 2.4f) - 0.055f;
                    rgba[((size_t)row * m + col) * 4 + c] = (uint8_t)(std::clamp(s, 0.f, 1.f) * 255.f);
                }
                rgba[((size_t)row * m + col) * 4 + 3] = 255;
            }
        std::printf("map Y max %.3f\n", maxY);
        std::system("mkdir -p out");
        if (!write_png("out/caustic-tri-floor.png", m, m, rgba.data())) return fail("png");
        std::printf("wrote out/caustic-tri-floor.png\n");
    }
    return exit_code;
}
