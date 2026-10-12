// blocks.mm: clear glass blocks standing on pale paper in hard sun, seen from above.
// The blocks are kit.h's; their shadows are Filament's own sun shadow; the paper is Filament's lit material; the light
// the blocks throw on the paper is boxlight.mat (each pair of a block's faces drawn once per colour, cut by the GPU).
// One picture, no switches.   ./blocks.sh  ->  out/blocks.png
#include "kit.h"

static const int W = 1920;
static const int H = 1080;

int main() {
    Stage s = open_stage(W, H);
    s.camera->lookAt({2.4f, 4.4f, 3.8f}, {0.f, 0.35f, 0.f}, {0, 1, 0});
    s.camera->setProjection(30.0, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.2f).build(*s.engine));

    // light: a hard sun from the far right, and Filament's bright studio as the sky that fills the shadows
    if (!room(s, "out/lightroom/lightroom_ibl.ktx", 22000.f)) return fail("out/lightroom/lightroom_ibl.ktx");
    const float3 sunWay = normalize(float3{-0.34f, -0.75f, 0.67f});
    const float sunLux = 100000.f;
    LightManager::ShadowOptions shadow;
    shadow.mapSize = 2048;
    utils::Entity sun = utils::EntityManager::get().create();
    LightManager::Builder(LightManager::Type::SUN).color({1.0f, 0.98f, 0.95f}).intensity(sunLux).direction(sunWay)
        .castShadows(true).shadowOptions(shadow).build(*s.engine, sun);
    s.scene->addEntity(sun);

    // the paper
    const float3 paper{0.78f, 0.82f, 0.85f};
    Mesh* floor = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        floor->pos.push_back({30.f * c.x, 0.f, -30.f * c.y});
        floor->nrm.push_back({0, 1, 0});
        floor->uv.push_back(c * 0.5f + 0.5f);
    }
    floor->idx = {0, 1, 2, 0, 2, 3};
    MaterialInstance* floor_mi = add(s, floor, "floor_lit.filamat", {mat4f{}}, 1, true);
    if (!floor_mi) return fail("floor_lit.filamat");
    floor_mi->setParameter("color", float4{paper, 1.f});
    floor_mi->setParameter("roughness", 0.9f);

    // the blocks: {middle on the floor}, {half sizes}, turned about the upright by
    struct Placed { float2 at; float3 half; float turn; };
    const Placed all[] = {{{0.00f, 0.00f}, {0.50f, 0.50f, 0.50f}, 0.35f}, {{-1.15f, 0.25f}, {0.42f, 0.42f, 0.42f}, -0.20f},
                          {{1.05f, 0.45f}, {0.36f, 0.36f, 0.36f}, 0.50f}, {{0.55f, -1.10f}, {0.75f, 0.45f, 0.45f}, 0.10f},
                          {{-0.80f, -0.95f}, {0.30f, 0.30f, 0.30f}, 0.80f}};
    std::vector<mat4f> places;
    static float4 poseX[64], poseY[64], poseZ[64], poseAt[64];   // (boxlight.mat: axes with half sizes, middle)
    int count = 0;
    for (const Placed& p : all) {
        const mat4f turn = mat4f::rotation(p.turn, float3{0, 1, 0});
        places.push_back(mat4f::translation(float3{p.at.x, p.half.y, p.at.y}) * turn * mat4f::scaling(p.half * 2.f));
        poseX[count] = {turn[0].xyz, p.half.x};
        poseY[count] = {turn[1].xyz, p.half.y};
        poseZ[count] = {turn[2].xyz, p.half.z};
        poseAt[count++] = {p.at.x, p.half.y, p.at.y, 0.f};
    }
    MaterialInstance* blocks_mi = glass_blocks(s, places, 0.03f, true);
    if (!blocks_mi) return fail("glassbox.filamat");
    blocks_mi->setParameter("abbe", 8.f);
    blocks_mi->setParameter("envGain", 0.4f);    // (on pale paper the blocks want glassbox's own studio brighter than on black)
    blocks_mi->setParameter("lean", 0.02f);

    // the light the blocks throw: every ordered pair of faces (in by A, out by B) as B's four corners, 8 colours a block
    Mesh* pairs = new Mesh;
    for (int A = 0; A < 6; ++A)
        for (int B = 0; B < 6; ++B) {
            if (A == B) continue;
            const uint32_t base = (uint32_t)pairs->pos.size();
            for (int corner = 0; corner < 4; ++corner) {
                pairs->pos.push_back({(float)A, (float)B, (float)corner});
                pairs->nrm.push_back({0, 1, 0});
                pairs->uv.push_back({0, 0});
            }
            pairs->idx.insert(pairs->idx.end(), {base, base + 1, base + 2, base, base + 2, base + 3});
        }
    MaterialInstance* light_mi = add(s, pairs, "boxlight.filamat", {mat4f{}}, 1, false, count * 8);
    if (!light_mi) return fail("boxlight.filamat");
    light_mi->setParameter("poseX", poseX, 64);
    light_mi->setParameter("poseY", poseY, 64);
    light_mi->setParameter("poseZ", poseZ, 64);
    light_mi->setParameter("poseAt", poseAt, 64);
    light_mi->setParameter("sunWay", sunWay);
    light_mi->setParameter("sunOnFloor", paper * (sunLux / 3.14159265f));
    light_mi->setParameter("ior", 1.5f);
    light_mi->setParameter("abbe", 8.f);
    light_mi->setParameter("gain", 1.f);
    light_mi->setParameter("floorY", 0.f);

    if (!shoot(s, "out/blocks.png")) return fail("png");
    std::printf("wrote out/blocks.png\n");
    return 0;
}
