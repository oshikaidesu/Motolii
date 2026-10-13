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

    // the film, drawn by ThorVG to fill the picture's width
    static tvg::Animation* film_of = nullptr;
    Drawn* film = drawn_open(s, W, H, [](tvg::WgCanvas* canvas) {
        film_of = tvg::Animation::gen();
        tvg::Picture* p = film_of->picture();
        for (const char* at : {"third_party/thorvg_example/res/lottie/new_design.json", "../../third_party/thorvg_example/res/lottie/new_design.json"})
            if (p->load(at) == tvg::Result::Success) break;
        float w = 0.f, h = 0.f;
        p->size(&w, &h);
        if (w <= 0.f) return;
        const float fit = std::min((float)W / w, (float)H / h);
        p->size(w * fit, h * fit);
        p->translate(0.5f * ((float)W - w * fit), 0.5f * ((float)H - h * fit));
        tvg::Shape* ground = tvg::Shape::gen();   // (the film has no ground of its own)
        ground->appendRect(0, 0, (float)W, (float)H);
        ground->fill(245, 242, 235, 255);
        canvas->add(ground);
        canvas->add(p);
    });
    if (!film) return fail("drawn_open: the WebGPU backend, ThorVG built against Dawn and the patched Filament are needed (BACKEND=webgpu)");
    if (!film_of || film_of->totalFrame() <= 0.f) return fail("the example film (third_party/thorvg_example)");

    // the wall fills the view, 0.6 behind the middle
    const float dist = 6.6f, hh = dist * std::tan(0.5f * fov * TAU / 360.f), hw = hh * W / H;
    Mesh* sheet = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        sheet->pos.push_back({eye.x + hw * c.x, eye.y + hh * c.y, eye.z - dist});
        sheet->nrm.push_back({0, 0, 1});
        sheet->uv.push_back(c * 0.5f + 0.5f);
    }
    sheet->idx = {0, 1, 2, 0, 2, 3};
    MaterialInstance* sheet_mi = add(s, sheet, "poster.filamat");
    if (!sheet_mi) return fail("poster.filamat");
    sheet_mi->setParameter("color", film->texture, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));
    sheet_mi->setParameter("glow", 0.f);
    sheet_mi->setParameter("roll", 0.f);

    struct Thing { Mesh* mesh; mat4f place; float turns; };
    const Thing things[] = {
        {torus(0.62f, 0.22f), mat4f::translation(float3{-0.25f, 2.05f, 0.6f}) * mat4f::rotation(0.5f, float3{0.3f, 1, 0}), 1.f},
        {gem(0.55f), mat4f::translation(float3{0.45f, 0.75f, 0.75f}) * mat4f::rotation(0.4f, normalize(float3{1, 0.3f, 0.2f})), -1.f},
    };
    for (const Thing& t : things)
        if (!thick_glass(s, t.mesh, {t.place})) return fail("thick_glass.filamat");

    const float frames = film_of->totalFrame(), seconds = film_of->duration();
    const auto pose = [&](float t) {
        const float frame = std::fmod(t * 8.f / seconds, 1.f) * frames;   // (the film's own speed; it starts again when it ends)
        drawn_now(s, film, [frame] { film_of->frame(frame); });
        for (const Thing& thing : things)
            move(s, thing.mesh, [&](int, const mat4f& at) { return mat4f::translation(at[3].xyz) * mat4f::rotation(TAU * t * thing.turns, float3{0, 1, 0}) * mat4f::translation(-at[3].xyz) * at; });
    };
    pose(0.3f);
    return show(s, argc, argv, "wall", 480, 2.f, pose);
}
