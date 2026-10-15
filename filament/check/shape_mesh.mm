// Forty stars waving, two ways, timed (kit.h clock): `sheets`, each drawn by ThorVG into a picture of its own and bent
// as it is drawn (drawn.h warp_at), or `meshes`, each cut into triangles once (libtess2) and drawn by Filament itself,
// bent in its material (shape.mat), the view antialiased by MSAA.
//   check/shape_mesh.sh sheets|meshes [time]
#include "../kit.h"
#include "../drawn.h"
#include <tesselator.h>

// A star of 5 points, 1 high, from its middle, its sides cut in pieces 1/64 long, cut into triangles by libtess2.
static Mesh* star_mesh() {
    std::vector<float2> ring;
    const int n = 10;
    for (int k = 0; k < n; ++k) {
        const float a = (float)(-TAU / 4.0 + TAU * k / n), r = (k & 1) ? 0.25f : 0.5f;
        const float b = (float)(-TAU / 4.0 + TAU * (k + 1) / n), q = ((k + 1) & 1) ? 0.25f : 0.5f;
        const float2 from{r * std::cos(a), -r * std::sin(a)}, to{q * std::cos(b), -q * std::sin(b)};
        const int pieces = std::max(1, (int)std::ceil(length(to - from) * 64.f));
        for (int i = 0; i < pieces; ++i) ring.push_back(from + (to - from) * ((float)i / (float)pieces));
    }
    TESStesselator* t = tessNewTess(nullptr);
    tessAddContour(t, 2, ring.data(), sizeof(float2), (int)ring.size());
    tessTesselate(t, TESS_WINDING_NONZERO, TESS_POLYGONS, 3, 2, nullptr);
    Mesh* m = new Mesh;
    const TESSreal* v = tessGetVertices(t);
    for (int i = 0; i < tessGetVertexCount(t); ++i) m->pos.push_back({v[2 * i], v[2 * i + 1], 0.f}), m->nrm.push_back({0, 0, 1}), m->uv.push_back({v[2 * i], v[2 * i + 1]});
    const TESSindex* e = tessGetElements(t);
    for (int i = 0; i < 3 * tessGetElementCount(t); ++i) m->idx.push_back((uint32_t)e[i]);
    tessDeleteTess(t);
    return m;
}

int main(int argc, char** argv) {
    Stage s = open_stage(1920, 1080);
    s.camera->lookAt({0.f, 0.f, 6.f}, {0.f, 0.f, 0.f}, {0, 1, 0});
    s.camera->setProjection(30.0, 1920.0 / 1080.0, 0.05, 60.0, Camera::Fov::VERTICAL);
    const bool meshes = argc > 1 && !strcmp(argv[1], "meshes");
    std::vector<Drawn*> sheets;
    std::vector<MaterialInstance*> looks;
    Mesh* star = meshes ? star_mesh() : nullptr;
    if (meshes) s.view->setMultiSampleAntiAliasingOptions({.enabled = true, .sampleCount = 4});
    for (int i = 0; i < 40; ++i) {
        const mat4f place = mat4f::translation(float3{-3.f + 0.85f * (float)(i % 8), 1.6f - 0.8f * (float)(i / 8), 0.f}) * mat4f::scaling(float3{0.5f});
        if (meshes) {
            MaterialInstance* look = add(s, star, "shape.filamat", {place});
            if (!look) return fail("shape.filamat");
            look->setParameter("color", float4{0x64 / 255.f, 0x37 / 255.f, 1.f, 1.f});
            looks.push_back(look);
        } else {
            DrawnFx fx;
            fx.warped = true, fx.reach = 0.05f;
            Drawn* d = flat(s, "star", 1.f, mat4f::translation(float3{-3.f + 0.85f * (float)(i % 8), 1.6f - 0.8f * (float)(i / 8), 0.f}), 0.5f, float3{0x64 / 255.f, 0x37 / 255.f, 1.f}, nullptr, float4{0.f}, fx, 5, 0.5f);
            if (!d) return fail("flat (WebGPU backend only)");
            sheets.push_back(d);
        }
    }
    return show(s, argc - 1, argv + 1, meshes ? "shape_meshes" : "shape_sheets", 240, [&](float f) {
        const float along = 4.f * f;   // (four waves go by in the film)
        for (MaterialInstance* look : looks) look->setParameter("wave", float4{0.05f, 0.3f, along, (float)TAU / 4.f});
        for (Drawn* d : sheets) warp_at(s, d, 0.05f, 0.3f, 90.f, along);
    });
}
