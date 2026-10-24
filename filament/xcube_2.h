// xcube.mm, part 2 (cut there by size, as it was): read only through xcube.mm, in order.
#pragma once


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

