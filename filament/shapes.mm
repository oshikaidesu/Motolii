// shapes.mm: glass things that are not boxes, on the same paper in the same sun: a ring, a solid of twenty faces,
// letters. Their bodies are Filament's own glass (poster_glass.mat); their light on the paper is glasslight.mat, the
// same material as for the blocks: it asks a mesh only where its surface is and which way it faces.
// One picture, no switches.   ./shapes.sh  ->  out/shapes.png
#include "kit.h"

static const int W = 1920;
static const int H = 1080;

int main() {
    Stage s = open_stage(W, H);
    s.camera->lookAt({2.9f, 5.6f, 4.9f}, {0.1f, 0.2f, 0.25f}, {0, 1, 0});
    s.camera->setProjection(30.0, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.4f).saturation(1.15f).build(*s.engine));
    Sunlit lit;
    if (!paper_and_sun(s, lit)) return fail("paper and sun (floor_lit.filamat, out/lightroom)");

    struct Thing { Mesh* mesh; mat4f place; float thick; };
    Mesh* word = letters("GLASS", 0.25f, 0.02f);
    if (!word) return fail("letters");
    const Thing things[] = {
        {torus(0.55f, 0.20f), mat4f::translation(float3{-1.55f, 0.75f, 0.95f}) * mat4f::rotation(0.5f, float3{0, 1, 0}), 0.4f},      // a ring, standing
        {gem(0.55f), mat4f::translation(float3{1.25f, 0.47f, 1.05f}) * mat4f::rotation(0.4f, normalize(float3{1, 0.3f, 0.2f})), 0.8f},   // twenty faces
        {word, mat4f::translation(float3{-1.75f, 0.0f, -1.15f}) * mat4f::rotation(0.12f, float3{0, 1, 0}), 0.25f},                     // letters, standing
    };
    for (const Thing& t : things) {
        MaterialInstance* glass = add(s, t.mesh, "poster_glass.filamat", {t.place}, 2);
        if (!glass) return fail("poster_glass.filamat");
        glass->setParameter("thickness", t.thick);
        glass->setParameter("lean", 0.f);
        if (!shadow_of(s, t.mesh, {t.place})) return fail("shadow_only.filamat");
        if (!glass_light(s, lit, t.mesh, {t.place})) return fail("glasslight.filamat");
    }

    if (!shoot(s, "out/shapes.png")) return fail("png");
    std::printf("wrote out/shapes.png\n");
    return 0;
}
