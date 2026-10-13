// ring.mm: a twisted ring of clear glass standing on pale paper, and a glass bead beside it; after a photograph.
// Everything is kit.h's and Filament's: the glass is thick_glass.mat (what is behind, bent by its far side and by its
// near side, the colours parting; where the far side is seen steeply, the glass's own room instead); the paper, its
// shade and the light the glass throws are kit.h's. The glass is put down as it comes: no look numbers here.
// One picture, no switches.   ./ring.sh -> out/ring.png
#include "kit.h"

static const int W = 1080;
static const int H = 1920;

// The ring: a thick bar of glass of one width (twice as wide as thick, its edges rounded), bent round a tall upright
// loop. It turns over twice on the way round, half a turn each time, slowly, at its upper left and at its lower right.
// (Four turns gave more places of colour and read as a folded film with crossing lines inside.)
static float3 ring_point(float u, float ang) {
    auto centre = [](float a) { return float3{0.86f * std::cos(a) + 0.08f * std::sin(2.f * a), 1.42f * std::sin(a), 0.14f * std::sin(a + 0.8f)}; };
    auto turn_over = [](float a, float at) {   // half a turn, made between 0.9 before `at` and 0.9 after it (shorter, and it knots)
        const float t = std::min(std::max((a - at + 0.9f) / 1.8f, 0.f), 1.f);
        return 0.5f * TAU * t * t * (3.f - 2.f * t);
    };
    const float a = u * TAU, round = 4.0f, wide = 0.22f, thick = 0.115f;   // (twice as wide as thick: a bar, not a strap)
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
    Stage s = open_stage(W, H);
    const float3 aim{0.5f, 1.45f, 0.f};   // seen from above and close, as the photograph is: the floor's shade and light lie behind the lower bar
    const float3 eye = aim + normalize(float3{0.0f, 0.5f, 0.87f}) * 7.9f;
    s.camera->lookAt(eye, aim, {0, 1, 0});
    s.camera->setProjection(23.0, double(W) / double(H), 0.5, 80.0, Camera::Fov::VERTICAL);
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.1f).saturation(1.2f).build(*s.engine));
    Sunlit lit;
    lit.paper = {0.645f, 0.635f, 0.615f};                       // pale warm grey
    lit.sunWay = normalize(float3{0.42f, -0.86f, -0.28f});    // from high on the left in front: a short shadow to the right and behind
    lit.sunWide = 0.09f;                                      // a broad soft lamp
    lit.sunLux = 34000.f;
    lit.sky = 13000.f;
    lit.grain = 0.07f;                                        // (seen this low, a coarser tooth runs in streaks)
    if (!paper_and_sun(s, lit)) return fail("paper and sun (paper.filamat, floor_shade.filamat, out/lightroom)");

    struct Thing { Mesh* mesh; mat4f place; };
    const Thing things[] = {
        {ring(), mat4f::rotation(0.75f, float3{0, 1, 0}) * mat4f::rotation(-0.48f, float3{0, 0, 1})},   // turned three quarters to the eye, leaning to the right
        {ball(48, 32), mat4f::translation(float3{1.10f, 0.045f, 0.95f}) * mat4f::scaling(float3{0.045f})},   // the bead
    };
    std::vector<MaterialInstance*> bodies, thrown;
    for (const Thing& t : things) {
        bodies.push_back(thick_glass(s, t.mesh, {t.place}));
        if (!bodies.back()) return fail("thick_glass.filamat");
        if (!shadow_of(s, lit, t.mesh, {t.place})) return fail("floor_shade.filamat");
        thrown.push_back(glass_light(s, lit, t.mesh, {t.place}));
        if (!thrown.back()) return fail("glasslight.filamat");
    }

    // The film, 8 seconds of 16 beats: the ring turns once where it stands, like a thing on a turntable (its colours
    // wander along it as its bends come round), the sun swings to and fro, the eye comes a little nearer and goes back.
    // On each beat the turning jumps on and eases, the colours burst apart and the thrown light flashes.
    float3 middle{0.f};   // where the ring stands: the upright it turns about goes through here
    for (const float3& p : things[0].mesh->pos) middle += (things[0].place * float4{p, 1.f}).xyz / (float)things[0].mesh->pos.size();
    const auto pose = [&](float t) {
        const float beat = t * 16.f - std::floor(t * 16.f), kick = std::exp(-6.f * beat);
        const float go = 0.4f * t + 0.6f * (std::floor(t * 16.f) + 1.f - std::pow(1.f - beat, 4.f)) / 16.f;
        sun_to(s, normalize((mat4f::rotation(0.4f * std::sin(TAU * t), float3{0, 1, 0}) * float4{lit.sunWay, 0.f}).xyz));
        move(s, things[0].mesh, [&](int, const mat4f& at) { return mat4f::translation(middle) * mat4f::rotation(TAU * go, float3{0, 1, 0}) * mat4f::translation(-middle) * at; });
        for (MaterialInstance* mi : bodies) mi->setParameter("split", 0.10f + 0.08f * kick);
        for (MaterialInstance* mi : thrown) mi->setParameter("gain", 0.9f + 0.6f * kick);
        s.camera->lookAt(aim + (eye - aim) * (1.14f - 0.06f * (0.5f - 0.5f * std::cos(TAU * t))), aim, {0, 1, 0});   // (a little further off than the still: broad-on, the ring is wider than the still's frame)
    };
    return show(s, argc, argv, "ring", 480, 2.f, pose);
}
