// dance.mm: glass rings and twenty-faced solids on paper under a low sun, upright like the poster: what they throw on
// the paper (shade, and rainbow light, glasslight.mat) is the picture. Bodies are kit.h's thick glass, as it comes.
//   ./dance.sh  ->  out/dance.png        ./dance.sh film  ->  out/dance.mp4 (8 seconds, it loops)
#include "kit.h"

static const int W = 1080;
static const int H = 1920;

int main(int argc, char** argv) {
    Stage s = open_stage(W, H);
    const float3 eye{0.5f, 8.6f, 6.4f}, aim{0.f, 0.f, 0.55f};
    s.camera->lookAt(eye, aim, {0, 1, 0});
    s.camera->setProjection(30.0, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.4f).saturation(1.15f).build(*s.engine));
    Sunlit lit;
    lit.sunWay = normalize(float3{-0.12f, -0.50f, 0.86f});   // the sun low behind the things: shade and light run down the picture, towards the eye
    if (!paper_and_sun(s, lit)) return fail("paper and sun (paper.filamat, floor_shade.filamat, out/lightroom)");

    // {the thing, where it stands, how many times it turns in a film (the sign: which way)}
    struct Thing { Mesh* mesh; mat4f place; float turns; };
    const Thing things[] = {
        {torus(0.55f, 0.20f), mat4f::translation(float3{-0.80f, 0.75f, -1.55f}) * mat4f::rotation(0.5f, float3{0, 1, 0}), 1.f},
        {gem(0.55f), mat4f::translation(float3{0.85f, 0.47f, -0.75f}) * mat4f::rotation(0.4f, normalize(float3{1, 0.3f, 0.2f})), -1.f},
        {torus(0.36f, 0.13f), mat4f::translation(float3{0.95f, 0.49f, 0.95f}) * mat4f::rotation(-0.7f, float3{0, 1, 0}), -2.f},
        {gem(0.36f), mat4f::translation(float3{-0.75f, 0.31f, 0.55f}) * mat4f::rotation(0.9f, normalize(float3{0.2f, 1, 0.5f})), 2.f},
    };
    std::vector<MaterialInstance*> bodies, thrown;
    for (const Thing& t : things) {
        bodies.push_back(thick_glass(s, t.mesh, {t.place}));
        if (!bodies.back()) return fail("thick_glass.filamat");
        if (!shadow_of(s, lit, t.mesh, {t.place})) return fail("floor_shade.filamat");
        thrown.push_back(glass_light(s, lit, t.mesh, {t.place}));
        if (!thrown.back()) return fail("glasslight.filamat");
    }

    // The film, 8 seconds of 16 beats: the sun swings to and fro (shade and thrown light sweep the paper), each thing
    // turns on the spot, whole turns. On each beat the turning jumps on and eases, the colours in the glass burst apart
    // and the thrown light flashes.
    const auto pose = [&](float t) {
        const float beat = t * 16.f - std::floor(t * 16.f), kick = std::exp(-6.f * beat);
        const float go = 0.4f * t + 0.6f * (std::floor(t * 16.f) + 1.f - std::pow(1.f - beat, 4.f)) / 16.f;
        sun_to(s, normalize((mat4f::rotation(0.5f * std::sin(TAU * t), float3{0, 1, 0}) * float4{lit.sunWay, 0.f}).xyz));
        for (const Thing& thing : things)
            move(s, thing.mesh, [&](int, const mat4f& at) { return mat4f::translation(at[3].xyz) * mat4f::rotation(TAU * go * thing.turns, float3{0, 1, 0}) * mat4f::translation(-at[3].xyz) * at; });
        for (MaterialInstance* mi : bodies) mi->setParameter("split", 0.10f + 0.08f * kick);
        for (MaterialInstance* mi : thrown) mi->setParameter("gain", 0.9f + 0.6f * kick);
    };
    return show(s, argc, argv, "dance", 480, 2.f, pose);
}
