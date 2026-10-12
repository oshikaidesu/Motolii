// ring.mm: a twisted ring of clear glass standing on pale paper, and a glass bead beside it; after a photograph.
// Everything is kit.h's and Filament's: the glass is Filament's own, twice (poster_glass.mat shows the paper behind,
// room_glass.mat the room on its rims and folds); the paper, its shade and the light the glass throws are kit.h's. No switches but the view:
// ./ring.sh -> out/ring.png (the whole ring)     ./ring.sh close -> out/ring_close.png (close to the band)
#include "kit.h"

static const int W = 1080;
static const int H = 1920;

// The ring: a ribbon of glass of one width, a flat band with small round edges (four times as wide as thick), bent round a tall upright
// loop. It turns over twice on the way round, half a turn each time, at its upper left and at its lower right.
static float3 ring_point(float u, float ang) {
    auto centre = [](float a) { return float3{0.86f * std::cos(a) + 0.08f * std::sin(2.f * a), 1.42f * std::sin(a), 0.14f * std::sin(a + 0.8f)}; };
    auto turn_over = [](float a, float at) {   // half a turn, made between 0.45 before `at` and 0.45 after it
        const float t = std::min(std::max((a - at + 0.45f) / 0.9f, 0.f), 1.f);
        return 0.5f * TAU * t * t * (3.f - 2.f * t);
    };
    const float a = u * TAU, round = 5.0f, wide = 0.24f, thick = 0.065f;
    const float3 T = normalize(centre(a + 1e-3f) - centre(a - 1e-3f));
    const float3 N = normalize(cross(float3{0, 0, 1}, T)), B = cross(T, N);
    const float twist = 0.25f + turn_over(a, 2.55f) + turn_over(a, 5.45f);
    const float3 n = N * std::cos(twist) + B * std::sin(twist), b = cross(T, n);
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
            const int i1 = (i + 1) % rings;
            const uint32_t p0 = (uint32_t)(i * sides + k), p1 = (uint32_t)(i * sides + (k + 1) % sides);
            const uint32_t q0 = (uint32_t)(i1 * sides + k), q1 = (uint32_t)(i1 * sides + (k + 1) % sides);
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
        const float3 aim{0.42f, 1.68f, 0.f};   // seen from a little above, as the photograph is: nothing but paper behind
        s.camera->lookAt(aim + normalize(float3{0.0f, 0.34f, 0.94f}) * 8.6f, aim, {0, 1, 0});
        s.camera->setProjection(23.0, double(W) / double(H), 0.5, 80.0, Camera::Fov::VERTICAL);
    }
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.1f).saturation(1.5f).build(*s.engine));
    Sunlit lit;
    lit.paper = {0.60f, 0.63f, 0.66f};                        // pale warm grey
    lit.sunWay = normalize(float3{0.42f, -0.86f, -0.28f});    // from high on the left in front: a short shadow to the right and behind
    lit.sunWide = 0.14f;                                      // a broad soft lamp
    lit.sunLux = 34000.f;
    lit.sky = 13000.f;
    lit.studio = "out/graffiti_shelter_2k/graffiti_shelter_2k_ibl.ktx";   // a room full of light and dark and colour (it comes with Filament): glass shows only what is around it
    if (!paper_and_sun(s, lit)) return fail("paper and sun (paper.filamat, floor_shade.filamat, out/lightroom)");

    struct Thing { Mesh* mesh; mat4f place; float thick; };
    const Thing things[] = {
        {ring(), mat4f::rotation(0.75f, float3{0, 1, 0}) * mat4f::rotation(-0.28f, float3{0, 0, 1}), 0.17f},   // turned three quarters to the eye, leaning to the right
        {ball(48, 32), mat4f::translation(float3{1.15f, 0.045f, 0.55f}) * mat4f::scaling(float3{0.045f}), 0.09f},   // the bead
    };
    for (const Thing& t : things) {
        MaterialInstance* glass = add(s, t.mesh, "poster_glass.filamat", {t.place}, 2);
        if (!glass) return fail("poster_glass.filamat");
        glass->setParameter("thickness", t.thick);
        glass->setParameter("lean", 0.f);
        glass->setParameter("film", 1.f);
        glass->setParameter("split", 2.f);
        // over it: the room, on the rims and folds; first on the thing's FAR side (its inside faces), then on its near
        // side, so the far rims show through the glass and the bar reads as thick
        for (int side = 0; side < 2; ++side) {
            MaterialInstance* rims = add(s, t.mesh, "room_glass.filamat", {t.place}, 2, false, 1, 5 + side);
            if (!rims) return fail("room_glass.filamat");
            if (side == 0) rims->setCullingMode(backend::CullingMode::FRONT);
            rims->setParameter("thickness", t.thick);
            rims->setParameter("film", 1.f);
            rims->setParameter("filmIor", 2.2f);
            rims->setParameter("filmThick", float2{250.f, 800.f});
            rims->setParameter("mirror", side == 0 ? 0.5f : 1.0f);
            rims->setParameter("split", 20.f);
            rims->setParameter("rim", side == 0 ? 2.4f : 1.6f);
            rims->setParameter("strength", side == 0 ? 0.8f : 1.f);
        }
        if (!shadow_of(s, lit, t.mesh, {t.place})) return fail("floor_shade.filamat");
        if (!glass_light(s, lit, t.mesh, {t.place})) return fail("glasslight.filamat");
    }

    const char* out = close ? "out/ring_close.png" : "out/ring.png";
    if (!shoot(s, out)) return fail("png");
    std::printf("wrote %s\n", out);
    return 0;
}
