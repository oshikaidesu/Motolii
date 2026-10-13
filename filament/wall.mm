// wall.mm: an After Effects film on the wall, glass before it. The film is a Lottie file After Effects wrote (one
// of ThorVG's examples, fetched: third_party/thorvg_example), played by ThorVG with all it holds (shapes, masks,
// precompositions) and drawn by ThorVG on Filament's own device (drawn.h); the wall is a sheet that shows that
// picture, and the glass bends it like anything else behind it. WebGPU backend only.
//   ./dawn_probe/kit_wgpu.sh wall          ->  out/wgpu_kit/out/wall.png       (then: cd out/wgpu_kit && ../wall_wgpu film)
#include "kit.h"
#include "drawn.h"

static const int W = 1080;
static const int H = 1920;

int main(int argc, char** argv) {
    Stage s = open_stage(W, H);
    const float3 eye{0.f, 1.4f, 6.f};
    const float fov = 30.f;
    s.camera->lookAt(eye, {0.f, 1.4f, 0.f}, {0, 1, 0});
    s.camera->setProjection(fov, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.2f).saturation(1.1f).build(*s.engine));
    if (!room(s, "out/small_studio/small_studio_ibl.ktx", 60000.f)) return fail("out/small_studio/small_studio_ibl.ktx");

    // the film: a thing on the wall, 0.6 behind the middle, as tall as the view there
    const float dist = 6.6f;
    Drawn* film = picture(s, "../../third_party/thorvg_example/res/lottie/new_design.json", mat4f::translation(float3{eye.x, eye.y, eye.z - dist}),
                          2.f * dist * std::tan(0.5f * fov * TAU / 360.f), float3{245.f, 242.f, 235.f} / 255.f);
    if (!film || film->frames <= 0.f) return fail("picture (WebGPU backend only): the example film (third_party/thorvg_example)");

    struct Thing { Mesh* mesh; mat4f place; float turns; };
    const Thing things[] = {
        {torus(0.62f, 0.22f), mat4f::translation(float3{-0.25f, 2.05f, 0.6f}) * mat4f::rotation(0.5f, float3{0.3f, 1, 0}), 1.f},
        {gem(0.55f), mat4f::translation(float3{0.45f, 0.75f, 0.75f}) * mat4f::rotation(0.4f, normalize(float3{1, 0.3f, 0.2f})), -1.f},
    };
    for (const Thing& t : things)
        if (!thick_glass(s, t.mesh, {t.place})) return fail("thick_glass.filamat");

    const auto pose = [&](float t) {
        picture_at(s, film, std::fmod(t * 8.f, film->seconds));   // (the film's own speed; it starts again when it ends)
        for (const Thing& thing : things)
            move(s, thing.mesh, [&](int, const mat4f& at) { return mat4f::translation(at[3].xyz) * mat4f::rotation(TAU * t * thing.turns, float3{0, 1, 0}) * mat4f::translation(-at[3].xyz) * at; });
    };
    pose(0.3f);
    return show(s, argc, argv, "wall", 480, 2.f, pose);
}
