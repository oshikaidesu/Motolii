// blocks.mm: clear glass blocks standing on pale paper in hard sun, seen from above.
// Everything is kit.h's: the blocks (glassbox.mat), the paper and the sun with Filament's own shadow, and the light the
// blocks throw on the paper (glasslight.mat). One picture, no switches.   ./blocks.sh  ->  out/blocks.png
#include "kit.h"

static const int W = 1920;
static const int H = 1080;
static const float ROUND = 0.06f;   // the blocks' edges, for the glass and for its light alike

int main() {
    Stage s = open_stage(W, H);
    // seen from far through a long lens, as such still lifes are photographed: the blocks' edges stay parallel
    const float3 aim{-0.15f, 0.30f, 0.35f};
    s.camera->lookAt(aim + normalize(float3{0.25f, 0.78f, 1.0f}) * 22.f, aim, {0, 1, 0});
    s.camera->setProjection(9.0, double(W) / double(H), 1.0, 60.0, Camera::Fov::VERTICAL);
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.4f).saturation(1.15f).build(*s.engine));
    Sunlit lit;
    if (!paper_and_sun(s, lit)) return fail("paper and sun (floor_lit.filamat, out/lightroom)");

    // the blocks: standing close, all turned alike. {middle on the floor, along and across their common turn}, {half sizes}
    struct Placed { float2 at; float3 half; };
    const Placed all[] = {{{0.00f, 0.00f}, {0.50f, 0.50f, 0.50f}}, {{1.00f, -0.08f}, {0.45f, 0.45f, 0.45f}}, {{-0.95f, 0.12f}, {0.40f, 0.40f, 0.40f}},
                          {{0.10f, -1.10f}, {0.55f, 0.36f, 0.50f}}, {{0.98f, 0.82f}, {0.33f, 0.33f, 0.33f}}};
    const mat4f turn = mat4f::rotation(0.62f, float3{0, 1, 0});
    std::vector<mat4f> places;
    for (const Placed& p : all)
        places.push_back(turn * mat4f::translation(float3{p.at.x, p.half.y, p.at.y}) * mat4f::scaling(p.half * 2.f));
    MaterialInstance* blocks_mi = glass_blocks(s, places, ROUND);
    if (!blocks_mi) return fail("glassbox.filamat");
    blocks_mi->setParameter("abbe", 8.f);
    blocks_mi->setParameter("envGain", 0.5f);    // (glassbox's own studio, bright strips and black flags: glass shows only what is around it)
    blocks_mi->setParameter("reach", 0.02f);     // (how far behind a block the scene is taken to lie: these stand on the paper)
    blocks_mi->setParameter("absorption", float3{0.30f, 0.10f, 0.06f});   // (a breath of blue)
    blocks_mi->setParameter("lean", 0.02f);
    if (!shadow_of(s, cube(ROUND), places)) return fail("shadow_only.filamat");
    if (!glass_light(s, lit, cube(ROUND, 16), places, 0.01f, 0.5f)) return fail("glasslight.filamat");

    if (!shoot(s, "out/blocks.png")) return fail("png");
    std::printf("wrote out/blocks.png\n");
    return 0;
}
