// mask.mm: mattes across the two kinds of thing, as After Effects' track mattes cut one layer by another, of the
// two kinds a matte can be: in the film's picture (a window on the screen) or on a thing (a sticker).
// Behind all, a film on the wall (text_anim.json), uncut. Before it, three rows:
// above, a flat picture cuts a solid, in the picture: a lump of glass is seen only inside After Effects' own figures
//   (a film, text2.json, never shown itself), its edge blurred;
// in the middle, solids cut a flat picture, in the picture: a film on a swinging sheet (gradient_background.json) is
//   seen only inside the shape of a turning gem and outside the shape of a turning ring (two mattes on one thing;
//   neither is shown): the sheet swings behind its window; a glass ring passes before it, and shows it cut as it is;
// below, a picture cuts a picture by its light, on the thing: the same film on another swinging sheet, as much of it
//   as a third film (dancing_star.json, never shown) is light there: the star swings with its sheet.
// The film's camera goes round all of it. GOD=1: the same looked at by another eye, standing aside (out/mask_god.*):
// the mattes stay the film camera's and the things' own, so the things are cut just the same; a small gem marks
// where the film's camera is.
// The films are After Effects' files, ThorVG's examples (third_party/thorvg_example). WebGPU backend only.
//   ./dawn_probe/kit_wgpu.sh mask          ->  out/wgpu_kit/out/mask.png       (then: cd out/wgpu_kit && ../mask_wgpu film)
#include "kit.h"
#include "drawn.h"

static const int W = 1080;
static const int H = 1920;

int main(int argc, char** argv) {
    Stage s = open_stage(W, H);
    const bool aside = getenv("GOD") != nullptr;
    Camera* god = aside ? god_view(s) : nullptr;
    const float3 eye{0.f, 1.4f, 6.f};
    const float fov = 30.f;
    s.film->lookAt(eye, {0.f, 1.4f, 0.f}, {0, 1, 0});
    s.film->setProjection(fov, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);
    if (god) {
        god->lookAt({3.9f, 5.0f, 10.4f}, {0.55f, 1.2f, 3.1f}, {0, 1, 0});   // (from above and aside: the film's camera near, the wall far)
        god->setProjection(42.0, double(W) / double(H), 0.05, 60.0, Camera::Fov::VERTICAL);
    }
    static FilmicToneMapper filmic;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&filmic).contrast(1.2f).saturation(1.1f).build(*s.engine));
    if (!room(s, "out/small_studio/small_studio_ibl.ktx", 60000.f)) return fail("out/small_studio/small_studio_ibl.ktx");
    const std::string films = "../../third_party/thorvg_example/res/lottie/";
    const auto file = [&](const char* name) { static std::string path; path = films + name; return path.c_str(); };
    const float dist = 6.6f;
    Drawn* wall = picture(s, file("text_anim.json"), mat4f::translation(float3{eye.x, eye.y, eye.z - dist}), (god ? 2.6f : 1.6f) * 2.f * dist * std::tan(0.5f * fov * TAU / 360.f), float3{1.f});   // (on white; larger than the view head-on: the eye goes round)
    if (!wall || wall->frames <= 0.f) return fail("picture (WebGPU backend only): the example films (third_party/thorvg_example)");

    // a pale floor under it all, so that the eye standing aside sees where things stand
    Mesh* floor = new Mesh;
    for (float2 c : {float2{-1, 1}, float2{1, 1}, float2{1, -1}, float2{-1, -1}}) {
        floor->pos.push_back({9.f * c.x, -0.25f, 3.f + 9.f * c.y});
        floor->nrm.push_back({0, 1, 0});
        floor->uv.push_back(c * 0.5f + 0.5f);
    }
    floor->idx = {0, 1, 2, 0, 2, 3};
    if (MaterialInstance* pale = add(s, floor, "flat.filamat")) pale->setParameter("picture", white_point(s), TextureSampler());
    else return fail("flat.filamat");

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
    const int byLight = matte(s, light->sheet, {below}, light->texture, true, 1.f, -1, more->sheet);   // (on the sheet: it rides)
    Mesh* marker = god ? gem(0.3f) : nullptr;   // (where the film's camera is, for the eye that stands aside)
    if (marker && !thick_glass(s, marker, {mat4f{}})) return fail("thick_glass.filamat");

    if (byFigures < 0 || byGem < 0 || byRing < 0 || byLight < 0) return fail("matte.filamat");
    if (!matted(s, glass, byFigures, true, 6.f) || !matted(s, colours->look, byGem) || !matted(s, colours->look, byRing, false) || !matted(s, more->look, byLight)) return fail("matted");

    const auto pose = [&](float t) {
        // the film's camera goes round what it looks at: 25 degrees to either side, a little up and down.
        const float round = 0.44f * std::sin(TAU * t), up = 0.16f * std::sin(2.f * TAU * t);
        const float3 from = float3{0.f, 1.4f, 0.f} + 6.f * float3{std::sin(round) * std::cos(up), std::sin(up), std::cos(round) * std::cos(up)};
        s.film->lookAt(from, {0.f, 1.4f, 0.f}, {0, 1, 0});
        if (marker) move(s, marker, [&](int, const mat4f&) { return mat4f::translation(from) * mat4f::rotation(3.f * TAU * t, float3{0, 1, 0}); });
        // the two sheets swing alike: the one behind its window, the other with its sticker
        for (Drawn* d : {colours, more}) move(s, d->sheet, [&](int, const mat4f& at) { return at * mat4f::rotation(0.7f * std::sin(2.f * TAU * t), float3{0, 1, 0}); });
        move(s, lump, [&](int, const mat4f& at) { return at * mat4f::rotation(TAU * t, normalize(float3{0.2f, 1, 0})); });
        move(s, window, [&](int, const mat4f& at) { return at * mat4f::rotation(-TAU * t, normalize(float3{0.5f, 1, 0.3f})); });
        move(s, hole, [&](int, const mat4f& at) { return at * mat4f::rotation(TAU * t, normalize(float3{1, 0.4f, 0})); });
        move(s, ring, [&](int, const mat4f& at) { return mat4f::translation(float3{0.75f * std::sin(TAU * t), 0.f, 0.f}) * at * mat4f::rotation(2.f * TAU * t, normalize(float3{0.3f, 1, 0.2f})); });
    };
    if (getenv("LIGHT")) {   // (and glass that throws its light on paper, as blocks.mm: both at once, timed)
        Sunlit lit;
        if (!paper_and_sun(s, lit)) return fail("paper and sun");
        std::vector<mat4f> places;
        for (int k = 0; k < 3; ++k) places.push_back(mat4f::translation(float3{-0.9f + 0.9f * (float)k, 0.2f, -0.5f}) * mat4f::rotation(0.6f, float3{0, 1, 0}) * mat4f::scaling(float3{0.4f}));
        if (!glass_blocks(s, places, 0.1f) || !shadow_of(s, lit, cube(0.1f), places) || !box_light(s, lit, places, 0.1f)) return fail("glass blocks and their light");
    }
    return show(s, argc, argv, god ? "mask_god" : getenv("LIGHT") ? "mask_light" : "mask", 480, pose);
}
