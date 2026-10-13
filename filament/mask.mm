// mask.mm: mattes across the two kinds of thing, as After Effects' track mattes cut one layer by another.
// Behind all, a film on the wall (text_anim.json), uncut. Before it, three rows:
// above, a flat picture cuts a solid: a lump of glass is seen only inside After Effects' own figures (a film,
//   text2.json, never shown itself), its edge blurred;
// in the middle, solids cut a flat picture: a film on a sheet (gradient_background.json) is seen only inside the shape
//   of a turning gem and outside the shape of a turning ring (two mattes on one thing; neither is shown); a glass ring
//   passes before it, and shows it cut as it is;
// below, a picture cuts a picture by its light: the same film on another sheet, as much of it as a third film
//   (dancing_star.json, never shown) is light there.
// The eye goes round all of it.
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
    const auto file = [&](const char* name) { static std::string path; path = films + name; return path.c_str(); };
    const float dist = 6.6f;
    Drawn* wall = picture(s, file("text_anim.json"), mat4f::translation(float3{eye.x, eye.y, eye.z - dist}), 1.6f * 2.f * dist * std::tan(0.5f * fov * TAU / 360.f));   // (larger than the view head-on: the eye goes round)
    if (!wall || wall->frames <= 0.f) return fail("picture (WebGPU backend only): the example films (third_party/thorvg_example)");

    // above: a flat picture cuts a solid, softly
    const mat4f above = mat4f::translation(float3{0.f, 2.36f, 0.6f});
    Mesh* lump = gem(0.66f);
    MaterialInstance* glass = thick_glass(s, lump, {above});
    Drawn* figures = picture(s, file("text2.json"), above, 1.5f, float3{-1.f}, false);
    if (!glass || !figures) return fail("thick_glass.filamat, text2.json");
    const int byFigures = matte(s, figures->sheet, {above}, figures->texture);

    // in the middle: two solids cut a flat picture, and glass passes before it
    const mat4f middle = mat4f::translation(float3{0.f, 1.4f, 0.5f});
    Drawn* colours = picture(s, file("gradient_background.json"), middle, 0.95f);
    Mesh* window = gem(0.5f);
    Mesh* hole = torus(0.24f, 0.07f);
    Mesh* ring = torus(0.3f, 0.11f);
    if (!colours) return fail("gradient_background.json");
    const int byGem = matte(s, window, {middle}), byRing = matte(s, hole, {middle});
    if (!thick_glass(s, ring, {mat4f::translation(float3{0.f, 1.4f, 1.3f})})) return fail("thick_glass.filamat");

    // below: a picture cuts a picture by its light
    const mat4f below = mat4f::translation(float3{0.f, 0.44f, 0.5f});
    Drawn* more = picture(s, file("gradient_background.json"), below, 0.95f);
    Drawn* light = picture(s, file("dancing_star.json"), below, 0.95f, float3{-1.f}, false);
    if (!more || !light) return fail("dancing_star.json");
    const int byLight = matte(s, light->sheet, {below}, light->texture, true);

    if (byFigures < 0 || byGem < 0 || byRing < 0 || byLight < 0) return fail("matte.filamat");
    if (!matted(s, glass, byFigures, true, 6.f) || !matted(s, colours->look, byGem) || !matted(s, colours->look, byRing, false) || !matted(s, more->look, byLight)) return fail("matted");

    const auto pose = [&](float t) {
        // the eye goes round what it looks at: 25 degrees to either side, a little up and down. The mattes' pictures
        // are taken by the same camera, so they go with it.
        const float round = 0.44f * std::sin(TAU * t), up = 0.16f * std::sin(2.f * TAU * t);
        s.camera->lookAt(float3{0.f, 1.4f, 0.f} + 6.f * float3{std::sin(round) * std::cos(up), std::sin(up), std::cos(round) * std::cos(up)}, {0.f, 1.4f, 0.f}, {0, 1, 0});
        // (each film about at its own speed, a whole number of times over these 8 seconds, so that the loop closes)
        for (Drawn* d : {wall, figures, colours, more, light}) picture_at(s, d, std::fmod(t * std::max(1.f, std::round(8.f / d->seconds)), 1.f) * d->seconds);
        move(s, lump, [&](int, const mat4f& at) { return at * mat4f::rotation(TAU * t, normalize(float3{0.2f, 1, 0})); });
        move(s, window, [&](int, const mat4f& at) { return at * mat4f::rotation(-TAU * t, normalize(float3{0.5f, 1, 0.3f})); });
        move(s, hole, [&](int, const mat4f& at) { return at * mat4f::rotation(TAU * t, normalize(float3{1, 0.4f, 0})); });
        move(s, ring, [&](int, const mat4f& at) { return mat4f::translation(float3{0.75f * std::sin(TAU * t), 0.f, 0.f}) * at * mat4f::rotation(2.f * TAU * t, normalize(float3{0.3f, 1, 0.2f})); });
    };
    pose(0.3f);
    return show(s, argc, argv, "mask", 480, 2.f, pose);
}
