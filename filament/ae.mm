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
    const char* file = argv[1];
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
    const int W = 2 * (int)std::lround(0.5f * w * fit), H = 2 * (int)std::lround(0.5f * h * fit);   // (even, for the film)

    Stage s = open_stage(W, H);
    s.camera->lookAt({0.f, 0.f, 1.f}, {0.f, 0.f, 0.f}, {0, 1, 0});
    s.camera->setProjection(Camera::Projection::ORTHO, -double(W) / double(H), double(W) / double(H), -1.0, 1.0, 0.1, 10.0);
    static LinearToneMapper as_it_is;   // (nothing is done to the colours)
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&as_it_is).build(*s.engine));
    s.view->setAntiAliasing(View::AntiAliasing::NONE);   // (ThorVG has smoothed its own edges)

    // the film: the one thing in the scene, filling the view
    Drawn* film = picture(s, file, mat4f{}, 2.f);
    if (!film) return fail("picture (WebGPU backend only)");

    const auto pose = [&](float t) { picture_at(s, film, t * film->seconds); };
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
