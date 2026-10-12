// ring.mm: a twisted ring of clear glass standing on pale paper, and a glass bead beside it; after a photograph.
// Everything is kit.h's and Filament's: the glass is Filament's own, twice (poster_glass.mat shows the paper behind,
// room_glass.mat the room on its rims and folds); the paper, its shade and the light the glass throws are kit.h's. No switches but the view:
// ./ring.sh -> out/ring.png (the whole ring)     ./ring.sh close -> out/ring_close.png (close to the band)
#include "kit.h"

static const int W = 1080;
static const int H = 1920;

// The ring: a flat ribbon of glass with round edges, bent round a tall upright loop and twisted a turn and a half on
// the way round (its section is the same turned half round, so it closes), wide at its folds and thin between.
static float3 ring_point(float u, float ang) {
    auto centre = [](float a) { return float3{0.82f * std::cos(a) + 0.10f * std::sin(2.f * a), 1.45f * std::sin(a), 0.16f * std::sin(a + 0.8f)}; };
    const float a = u * TAU, round = 2.6f;
    const float3 T = normalize(centre(a + 1e-3f) - centre(a - 1e-3f));
    const float3 N = normalize(cross(float3{0, 0, 1}, T)), B = cross(T, N);
    const float twist = 1.5f * a + 0.5f * std::sin(2.f * a) + 0.4f;
    const float3 n = N * std::cos(twist) + B * std::sin(twist), b = cross(T, n);
    const float wide = 0.27f * (1.f + 0.40f * std::sin(3.f * a + 1.f)), thick = 0.10f * (1.f + 0.3f * std::sin(2.f * a));
    const float c = std::cos(ang), s = std::sin(ang);
    return centre(a) + n * (std::copysign(std::pow(std::fabs(c), 2.f / round), c) * thick)
                     + b * (std::copysign(std::pow(std::fabs(s), 2.f / round), s) * wide);
}
static Mesh* ring() {
    const int rings = 480, sides = 72;
    Mesh* m = new Mesh;
    float low = 1e9f;
    for (int i = 0; i < rings; ++i)
        for (int k = 0; k < sides; ++k) {
            const float u = (float)i / rings, ang = (float)k / sides * TAU, d = 2e-3f;
            const float3 p = ring_point(u, ang);
            float3 n = normalize(cross(ring_point(u, ang + d) - ring_point(u, ang - d), ring_point(u + d, ang) - ring_point(u - d, ang)));
            if (dot(n, p - (ring_point(u, ang) + ring_point(u, ang + 0.5f * TAU)) * 0.5f) < 0.f) n = -n;
            m->pos.push_back(p);
            m->nrm.push_back(n);
            m->uv.push_back({u, (float)k / sides});
            low = std::min(low, p.y);
            // (the half turn: the last ring meets the first one turned half round)
            const int i1 = (i + 1) % rings, turn = i1 == 0 ? sides / 2 : 0;
            const uint32_t p0 = (uint32_t)(i * sides + k), p1 = (uint32_t)(i * sides + (k + 1) % sides);
            const uint32_t q0 = (uint32_t)(i1 * sides + (k + turn) % sides), q1 = (uint32_t)(i1 * sides + (k + 1 + turn) % sides);
            m->idx.insert(m->idx.end(), {p0, p1, q0, p1, q1, q0});
        }
    for (float3& p : m->pos) p.y -= low;   // standing on the paper
    return m;
}

int main(int argc, char** argv) {
    const bool close = argc > 1 && argv[1][0] == 'c';
    Stage s = open_stage(W, H);
    if (close) {
        s.camera->lookAt({-0.2f, 1.1f, 3.2f}, {-0.75f, 1.15f, 0.f}, {0, 1, 0});
        s.camera->setProjection(16.0, double(W) / double(H), 0.1, 60.0, Camera::Fov::VERTICAL);
    } else {
        const float3 aim{-0.30f, 1.50f, 0.f};   // seen from a little above, as the photograph is: nothing but paper behind
        s.camera->lookAt(aim + normalize(float3{0.0f, 0.34f, 0.94f}) * 11.f, aim, {0, 1, 0});
        s.camera->setProjection(23.0, double(W) / double(H), 0.5, 80.0, Camera::Fov::VERTICAL);
    }
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.1f).build(*s.engine));
    Sunlit lit;
    lit.paper = {0.62f, 0.62f, 0.61f};                        // pale warm grey
    lit.sunWay = normalize(float3{0.42f, -0.86f, -0.28f});    // from high on the left in front: a short shadow to the right and behind
    lit.sunWide = 0.07f;                                      // a broad soft lamp
    lit.studio = "out/small_studio/small_studio_ibl.ktx";     // a dark studio with soft lamps: dark rims and bright streaks in the glass
    lit.sunLux = 50000.f;
    lit.sky = 9000.f;                                         // a bright room: pale shadows
    if (!paper_and_sun(s, lit)) return fail("paper and sun (paper.filamat, floor_shade.filamat, out/lightroom)");

    struct Thing { Mesh* mesh; mat4f place; float thick; };
    const Thing things[] = {
        {ring(), mat4f::rotation(0.80f, float3{0, 1, 0}), 0.22f},   // turned three quarters to the eye
        {ball(48, 32), mat4f::translation(float3{0.70f, 0.045f, 0.95f}) * mat4f::scaling(float3{0.045f}), 0.09f},   // the bead
    };
    for (const Thing& t : things) {
        MaterialInstance* glass = add(s, t.mesh, "poster_glass.filamat", {t.place}, 2);
        if (!glass) return fail("poster_glass.filamat");
        glass->setParameter("thickness", t.thick);
        glass->setParameter("lean", 0.f);
        glass->setParameter("film", 1.f);
        glass->setParameter("split", 6.f);
        MaterialInstance* rims = add(s, t.mesh, "room_glass.filamat", {t.place}, 2, false, 1, 6);   // over it: the room, on the rims and folds
        if (!rims) return fail("room_glass.filamat");
        rims->setParameter("thickness", t.thick);
        rims->setParameter("film", 1.f);
        rims->setParameter("split", 20.f);
        rims->setParameter("rim", 1.6f);
        rims->setParameter("strength", 1.f);
        if (!shadow_of(s, lit, t.mesh, {t.place})) return fail("floor_shade.filamat");
        if (!glass_light(s, lit, t.mesh, {t.place})) return fail("glasslight.filamat");
    }

    const char* out = close ? "out/ring_close.png" : "out/ring.png";
    if (!shoot(s, out)) return fail("png");
    std::printf("wrote %s\n", out);
    return 0;
}
