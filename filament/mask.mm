// mask.mm: mattes across the two kinds of thing, as After Effects' track mattes cut one layer by another.
// Above: a flat picture cuts a solid. A lump of glass is seen only inside After Effects' own figures (a film,
// text2.json, never shown itself). Below: a solid cuts a flat picture. A film on a sheet (gradient_background.json)
// is seen only inside the shape of a turning gem (never shown itself). Behind both, a film on the wall (text_anim.json), uncut.
// The films are After Effects' files, ThorVG's examples (third_party/thorvg_example). WebGPU backend only.
//   ./dawn_probe/kit_wgpu.sh mask          ->  out/wgpu_kit/out/mask.png       (then: cd out/wgpu_kit && ../mask_wgpu film)
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
    const std::string films = "../../third_party/thorvg_example/res/lottie/";
    const float dist = 6.6f;
    Drawn* wall = picture(s, (films + "text_anim.json").c_str(), mat4f::translation(float3{eye.x, eye.y, eye.z - dist}),
                          2.f * dist * std::tan(0.5f * fov * TAU / 360.f));
    if (!wall || wall->frames <= 0.f) return fail("picture (WebGPU backend only): the example films (third_party/thorvg_example)");

    // a flat picture cuts a solid
    Mesh* lump = gem(1.3f);
    const mat4f above = mat4f::translation(float3{0.f, 2.15f, 0.6f});
    MaterialInstance* glass = thick_glass(s, lump, {above});
    Drawn* figures = picture(s, (films + "text2.json").c_str(), above, 2.7f, float3{-1.f}, false);
    if (!glass || !figures) return fail("thick_glass.filamat, text2.json");
    const int byFigures = matte(s, figures->sheet, {above}, figures->texture);

    // a solid cuts a flat picture
    const mat4f below = mat4f::translation(float3{0.f, 0.62f, 0.4f});
    Drawn* colours = picture(s, (films + "gradient_background.json").c_str(), below, 1.45f);
    Mesh* window = gem(0.72f);
    if (!colours) return fail("gradient_background.json");
    const int byGem = matte(s, window, {below});
    if (byFigures < 0 || byGem < 0) return fail("matte.filamat");
    matted(glass, byFigures);
    matted(colours->look, byGem);

    const auto pose = [&](float t) {
        // (each film about at its own speed, a whole number of times over these 8 seconds, so that the loop closes)
        for (Drawn* d : {wall, figures, colours}) picture_at(s, d, std::fmod(t * std::max(1.f, std::round(8.f / d->seconds)), 1.f) * d->seconds);
        move(s, lump, [&](int, const mat4f& at) { return at * mat4f::rotation(TAU * t, normalize(float3{0.2f, 1, 0})); });
        move(s, window, [&](int, const mat4f& at) { return at * mat4f::rotation(-TAU * t, normalize(float3{0.5f, 1, 0.3f})); });
    };
    pose(0.3f);
    return show(s, argc, argv, "mask", 480, 2.f, pose);
}
