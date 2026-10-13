// shapes.mm: glass things that are not boxes, on the same paper in the same sun: a ring, a solid of twenty faces,
// letters. Their bodies are kit.h's thick glass, put down as it comes; their light on the paper is glasslight.mat, the
// same material as for the blocks: it asks a mesh only where its surface is and which way it faces.
// One picture, no switches.   ./shapes.sh  ->  out/shapes.png        ./shapes.sh film  ->  out/shapes.mp4 (8 seconds, it loops)
#include "kit.h"

static const int W = 1920;
static const int H = 1080;

int main(int argc, char** argv) {
    Stage s = open_stage(W, H);
    const float3 eye{2.9f, 5.6f, 4.9f}, aim{0.1f, 0.2f, 0.25f};
    s.camera->lookAt(eye, aim, {0, 1, 0});
    s.camera->setProjection(30.0, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.4f).saturation(1.15f).build(*s.engine));
    Sunlit lit;
    if (!paper_and_sun(s, lit)) return fail("paper and sun (paper.filamat, floor_shade.filamat, out/lightroom)");

    struct Thing { Mesh* mesh; mat4f place; };
    Mesh* word = letters("GLASS", 0.25f, 0.02f);
    if (!word) return fail("letters");
    const Thing things[] = {
        {torus(0.55f, 0.20f), mat4f::translation(float3{-1.55f, 0.75f, 0.95f}) * mat4f::rotation(0.5f, float3{0, 1, 0})},      // a ring, standing
        {gem(0.55f), mat4f::translation(float3{1.25f, 0.47f, 1.05f}) * mat4f::rotation(0.4f, normalize(float3{1, 0.3f, 0.2f}))},   // twenty faces
        {word, mat4f::translation(float3{-1.75f, 0.0f, -1.15f}) * mat4f::rotation(0.12f, float3{0, 1, 0})},                     // letters, standing
    };
    std::vector<MaterialInstance*> bodies, thrown;
    for (const Thing& t : things) {
        bodies.push_back(thick_glass(s, t.mesh, {t.place}));   // (put down as it comes: no numbers)
        if (!bodies.back()) return fail("thick_glass.filamat");
        if (!shadow_of(s, lit, t.mesh, {t.place})) return fail("floor_shade.filamat");
        thrown.push_back(glass_light(s, lit, t.mesh, {t.place}));
        if (!thrown.back()) return fail("glasslight.filamat");
    }

    // The film, 8 seconds of 16 beats: the sun swings to and fro (the shade and the thrown light sweep the paper), the
    // ring and the twenty-faced solid each turn once on the spot, the eye drifts a little round them. On each beat the
    // turning jumps on and eases, the colours in the glass burst apart and the thrown light flashes.
    const auto pose = [&](float t) {
        const float beat = t * 16.f - std::floor(t * 16.f), kick = std::exp(-6.f * beat);
        const float go = 0.4f * t + 0.6f * (std::floor(t * 16.f) + 1.f - std::pow(1.f - beat, 4.f)) / 16.f;
        sun_to(s, normalize((mat4f::rotation(0.45f * std::sin(TAU * t), float3{0, 1, 0}) * float4{lit.sunWay, 0.f}).xyz));
        const auto turned = [&](float turns) {
            return [=](int, const mat4f& at) { return mat4f::translation(at[3].xyz) * mat4f::rotation(TAU * go * turns, float3{0, 1, 0}) * mat4f::translation(-at[3].xyz) * at; };
        };
        move(s, things[0].mesh, turned(1.f));
        move(s, things[1].mesh, turned(-1.f));
        for (MaterialInstance* mi : bodies) mi->setParameter("split", 0.10f + 0.08f * kick);
        for (MaterialInstance* mi : thrown) mi->setParameter("gain", 0.9f + 0.6f * kick);
        s.camera->lookAt(aim + (mat4f::rotation(0.12f * std::sin(TAU * t), float3{0, 1, 0}) * float4{eye - aim, 0.f}).xyz, aim, {0, 1, 0});
    };
    return show(s, argc, argv, "shapes", 480, pose);
}
