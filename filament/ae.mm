// ae.mm: an After Effects file, played as it is. The file is a Lottie (what After Effects writes out); ThorVG reads it
// and draws it with all it holds (layers, keys, masks, mattes, precompositions, type), on Filament's own device
// (drawn.h), onto a sheet that fills the picture. The stage is the composition's size (no more than 1920 the long
// way) and nothing bends the colours, so what comes out is the film and nothing else: ae_test.sh checks it against
// ThorVG drawing alone. WebGPU backend only.
//   BUILDONLY=1 ./dawn_probe/kit_wgpu.sh ae    then, in out/wgpu_kit:
//   ../ae_wgpu <file.json>                         ->  out/ae.png   (the film's middle)
//   ../ae_wgpu <file.json> film                    ->  out/ae.mp4   (once through, at its own speed)
//   ../ae_wgpu <file.json> time                    how many pictures a second
//   ../ae_wgpu <file.json> stills <to> 0.1 0.5     ->  <to>_0.png, <to>_1.png   (how far through the film: 0 up to 1)
#include "kit.h"
#include "drawn.h"

int main(int argc, char** argv) {
    if (argc < 2) return fail("which file? ae_wgpu <file.json> [film | time | stills <to> <how far>...]");
    static const char* file = argv[1];
    // the composition's size and length (read once here, before anything is drawn)
    float w = 0.f, h = 0.f, frames = 0.f, seconds = 0.f;
    {
        tvg::Initializer::init(0);
        tvg::Animation* a = tvg::Animation::gen();
        if (a->picture()->load(file) == tvg::Result::Success) a->picture()->size(&w, &h);
        frames = a->totalFrame();
        seconds = a->duration();
        delete a;
    }
    if (w <= 0.f || h <= 0.f || frames <= 0.f) return fail("not a film ThorVG can read");
    const float fit = std::min(1.f, 1920.f / std::max(w, h));
    static const int W = 2 * (int)std::lround(0.5f * w * fit), H = 2 * (int)std::lround(0.5f * h * fit);   // (even, for the film)

    Stage s = open_stage(W, H);
    s.camera->lookAt({0.f, 0.f, 1.f}, {0.f, 0.f, 0.f}, {0, 1, 0});
    s.camera->setProjection(Camera::Projection::ORTHO, -1.0, 1.0, -1.0, 1.0, 0.1, 10.0);
    static LinearToneMapper as_it_is;
    // (nothing is done to the colours, and Filament is told it may then take its short way, one fine row of numbers
    // instead of a coarse block of them: with the block a white of 255 comes out 249, with the row it comes out 255)
    if (!s.engine->setFeatureFlag("engine.color_grading.use_1d_lut", true)) return fail("engine.color_grading.use_1d_lut");
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&as_it_is).build(*s.engine));
    s.view->setAntiAliasing(View::AntiAliasing::NONE);   // (ThorVG has smoothed its own edges)

    static tvg::Animation* film_of = nullptr;
    Drawn* drawn = drawn_open(s, W, H, [](tvg::WgCanvas* canvas) {
        film_of = tvg::Animation::gen();
        if (film_of->picture()->load(file) != tvg::Result::Success) return;
        film_of->picture()->size((float)W, (float)H);
        tvg::Shape* ground = tvg::Shape::gen();   // (a composition has no ground of its own: white)
        ground->appendRect(0, 0, (float)W, (float)H);
        ground->fill(255, 255, 255, 255);
        canvas->add(ground);
        canvas->add(film_of->picture());
    });
    if (!drawn) return fail("drawn_open: the WebGPU backend, ThorVG built against Dawn and the patched Filament are needed (BACKEND=webgpu)");

    Mesh* sheet = new Mesh;
    for (float2 c : {float2{-1, -1}, float2{1, -1}, float2{1, 1}, float2{-1, 1}}) {
        sheet->pos.push_back({c.x, c.y, 0.f});
        sheet->nrm.push_back({0, 0, 1});
        sheet->uv.push_back(c * 0.5f + 0.5f);
    }
    sheet->idx = {0, 1, 2, 0, 2, 3};
    MaterialInstance* mi = add(s, sheet, "flat.filamat");
    if (!mi) return fail("flat.filamat");
    mi->setParameter("picture", drawn->texture, TextureSampler(TextureSampler::MinFilter::LINEAR, TextureSampler::MagFilter::LINEAR));

    const auto pose = [&](float t) {
        const float frame = std::min(t * frames, frames - 1.f);
        drawn_now(s, drawn, [frame] { film_of->frame(frame); });
    };
    if (argc > 3 && !strcmp(argv[2], "stills")) {
        for (int i = 4; i < argc; ++i) {
            pose((float)atof(argv[i]));
            if (!shoot(s, (std::string(argv[3]) + "_" + std::to_string(i - 4) + ".png").c_str())) return fail("png");
        }
        return 0;
    }
    pose(0.5f);
    return show(s, argc - 1, argv + 1, "ae", (int)std::lround(seconds * 60.f), 0.f, pose);
}
