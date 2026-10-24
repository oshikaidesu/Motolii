// caustics.mm, part 2 (cut there by size, as it was): read only through caustics.mm, in order.
#pragma once


static TraceStats trace(const Field& field, V3 sun_dir, float lux, bool same_ior, std::vector<float>& tex) {
    TraceStats st;
    tex.assign((size_t)TEX * TEX * 4, 0.f);
    if (field.segs.empty() || lux <= 0.f) return st;
    auto t0 = std::chrono::steady_clock::now();
    V3 D = norm(sun_dir);
    V3 up = std::fabs(D.y) < 0.9f ? V3{0, 1, 0} : V3{1, 0, 0};
    V3 U = norm(cross(up, D));
    V3 V = cross(D, U);
    V3 C{0, 0, 0};
    for (const Seg& s : field.segs) C = C + s.a;
    C = C * (1.f / (float)field.segs.size());
    const int G = 200;
    const float extent = 1.15f;
    const float cell = (2.f * extent) / (float)G;
    const float texel_area = (FLOOR_HW * 2.f * FLOOR_HH * 2.f) / (float)(TEX * TEX);
    const float flux = lux * cell * cell / texel_area;
    int hits = 0, floors = 0;
    for (int j = 0; j < G; ++j) {
        for (int i = 0; i < G; ++i) {
            float su = -extent + (i + 0.5f) * cell;
            float sv = -extent + (j + 0.5f) * cell;
            V3 ro = C - D * 2.4f + U * su + V * sv;
            for (int w = 0; w < 4; ++w) {
                V3 o = ro, d = D;
                float transmit = 1.f;
                bool live = false;
                for (int pass = 0; pass < 3; ++pass) {
                    float best = 1e9f;
                    int best_seg = -1;
                    for (int si = 0; si < (int)field.segs.size(); ++si) {
                        const Seg& s = field.segs[si];
                        V3 mid = (s.a + s.b) * 0.5f;
                        V3 w = mid - o;
                        float along = dot(w, d);
                        if (along < -(s.rad + s.len) || along > 6.f) continue;
                        V3 perp = w - d * along;
                        if (dot(perp, perp) > (s.rad + s.len * 0.5f + 0.02f) * (s.rad + s.len * 0.5f + 0.02f)) continue;
                        float t = capsule_enter(o, d, s);
                        if (t > 0.006f && t < best) {
                            best = t;
                            best_seg = si;
                        }
                    }
                    if (best_seg < 0) break;
                    const Seg& hit = field.segs[best_seg];
                    V3 hp = o + d * best;
                    float u = std::clamp(dot(hp - hit.a, hit.ax), 0.f, hit.len);
                    V3 axis = hit.a + hit.ax * u;
                    V3 hn = norm(hp - axis);
                    int tube = hit.tube;
                    float ior = hit.ior, disp = hit.disp;
                    float absorb[3] = {hit.absorb[0], hit.absorb[1], hit.absorb[2]};
                    if (dot(d, hn) > 0.f) hn = hn * -1.f;
                    float n = ior;
                    if (!same_ior) n = ior + (disp / 20.f) * (ior - 1.f) * kOff[w];
                    if (n < 1.01f) n = 1.01f;
                    float cosi = std::clamp(-dot(d, hn), 0.f, 1.f);
                    transmit *= 1.f - fresnel(cosi, 1.f, n);
                    V3 inside;
                    if (!refract(d, hn, 1.f / n, inside)) break;
                    auto sdf_tube = [&](V3 p, V3& axis) {
                        float best = 1e9f;
                        for (const Seg& s : field.segs) {
                            if (s.tube != tube) continue;
                            float uu = std::clamp(dot(p - s.a, s.ax), 0.f, s.len);
                            V3 q = s.a + s.ax * uu;
                            float dd = len(p - q) - s.rad;
                            if (dd < best) {
                                best = dd;
                                axis = q;
                            }
                        }
                        return best;
                    };
                    float lo = 0.f, hi = 0.008f;
                    if (sdf_tube(hp + inside * hi, axis) > 0.f) break;
                    for (int step = 0; step < 24 && sdf_tube(hp + inside * hi, axis) < 0.f; ++step) hi *= 1.45f;
                    if (sdf_tube(hp + inside * hi, axis) < 0.f) break;
                    for (int step = 0; step < 12; ++step) {
                        float mid = 0.5f * (lo + hi);
                        if (sdf_tube(hp + inside * mid, axis) < 0.f) lo = mid;
                        else hi = mid;
                    }
                    V3 xp = hp + inside * hi;
                    sdf_tube(xp, axis);
                    V3 xn = norm(xp - axis);
                    float path = len(xp - hp);
                    transmit *= std::exp(-(absorb[0] + absorb[1] + absorb[2]) * 0.15f * path);
                    if (dot(inside, xn) > 0.f) xn = xn * -1.f;
                    float cosi2 = std::clamp(-dot(inside, xn), 0.f, 1.f);
                    transmit *= 1.f - fresnel(cosi2, n, 1.f);
                    V3 out;
                    if (!refract(inside, xn, n, out)) break;
                    o = xp + out * 0.01f;
                    d = out;
                    live = true;
                    hits++;
                }
                if (!live || d.y >= -1e-4f || transmit < 1e-4f) continue;
                float tf = (FLOOR_Y - o.y) / d.y;
                if (tf < 0.f || tf > 8.f) continue;
                V3 hitp = o + d * tf;
                float u, v;
                floor_uv(hitp, u, v);
                if (u < 0.f || u > 1.f || v < 0.f || v > 1.f) continue;
                float x = u * (TEX - 1), y = v * (TEX - 1);
                const int rad = 8;
                const float sig2 = 2.f * 3.1f * 3.1f;
                float accw = 0.f;
                for (int dy = -rad; dy <= rad; ++dy)
                    for (int dx = -rad; dx <= rad; ++dx) accw += std::exp(-(dx * dx + dy * dy) / sig2);
                for (int dy = -rad; dy <= rad; ++dy)
                    for (int dx = -rad; dx <= rad; ++dx) {
                        int xx = (int)std::lround(x) + dx;
                        int yy = (int)std::lround(y) + dy;
                        if (xx < 0 || yy < 0 || xx >= TEX || yy >= TEX) continue;
                        float* p = tex.data() + ((size_t)yy * TEX + xx) * 4;
                        float e = flux * transmit * std::exp(-(dx * dx + dy * dy) / sig2) / accw;
                        p[0] += kW[w][0] * e;
                        p[1] += kW[w][1] * e;
                        p[2] += kW[w][2] * e;
                    }
                floors++;
            }
        }
    }
    float sr = 0, sb = 0, wr = 0, wb = 0, chroma = 0, lum = 0, sum = 0, maxv = 0;
    int nch = 0;
    for (int y = 0; y < TEX; ++y)
        for (int x = 0; x < TEX; ++x) {
            float* p = tex.data() + ((size_t)y * TEX + x) * 4;
            p[0] = std::max(p[0], 0.f);
            p[1] = std::max(p[1], 0.f);
            p[2] = std::max(p[2], 0.f);
            float m = std::max(p[0], std::max(p[1], p[2]));
            sum += m;
            maxv = std::max(maxv, m);
            if (m > 20.f) {
                chroma += std::fabs(p[0] - p[2]);
                lum += m;
                sr += p[0] * x;
                wr += p[0];
                sb += p[2] * x;
                wb += p[2];
                nch++;
            }
        }
    auto t1 = std::chrono::steady_clock::now();
    st.ms = std::chrono::duration<double, std::milli>(t1 - t0).count();
    st.hit = hits;
    st.floor = floors;
    st.sum = sum;
    st.maxv = maxv;
    st.cx_r = wr > 1.f ? sr / wr : 0;
    st.cx_b = wb > 1.f ? sb / wb : 0;
    st.sep = std::fabs(st.cx_r - st.cx_b);
    st.chroma = lum > 1.f ? chroma / lum : 0;
    std::printf("trace hits %d floor %d sum %.3e max %.1f sep %.2f chroma %.3f  %.1f ms\n", st.hit, st.floor, st.sum,
                st.maxv, st.sep, st.chroma, st.ms);
    return st;
}

struct Hold {
    utils::Entity entity{};
    VertexBuffer* vb = nullptr;
    IndexBuffer* ib = nullptr;
    Built mesh;
    std::vector<float> uv;
};

