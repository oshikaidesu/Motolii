// ae_ref.cpp: the answer ae.mm is checked against (ae_test.sh): the same file at the same moments, drawn by ThorVG
// alone on the CPU, with nothing of Filament or the GPU in the way.
//   ae_ref <file.json> <w> <h> <to> <how far>...   ->  <to>_0.ppm, <to>_1.ppm, ...
#include <thorvg.h>
#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <string>
#include <vector>

int main(int argc, char** argv) {
    if (argc < 6) return 1;
    const int W = atoi(argv[2]), H = atoi(argv[3]);
    tvg::Initializer::init(0);
    tvg::Animation* film = tvg::Animation::gen();
    if (film->picture()->load(argv[1]) != tvg::Result::Success) return 2;
    film->picture()->size((float)W, (float)H);
    std::vector<uint32_t> pic((size_t)W * H);
    tvg::SwCanvas* canvas = tvg::SwCanvas::gen();
    if (canvas->target(pic.data(), (uint32_t)W, (uint32_t)W, (uint32_t)H, tvg::ColorSpace::ABGR8888S) != tvg::Result::Success) return 3;
    tvg::Shape* ground = tvg::Shape::gen();
    ground->appendRect(0, 0, (float)W, (float)H);
    ground->fill(255, 255, 255, 255);
    canvas->add(ground);
    canvas->add(film->picture());
    const float frames = film->totalFrame();
    for (int i = 5; i < argc; ++i) {
        film->frame(std::floor(std::min((float)atof(argv[i]) * frames, frames - 1.f) + 1e-3f));   // (at the file's own frames, as drawn.h picture_at plays it)
        canvas->update();
        canvas->draw(true);
        canvas->sync();
        FILE* f = std::fopen((std::string(argv[4]) + "_" + std::to_string(i - 5) + ".ppm").c_str(), "wb");
        std::fprintf(f, "P6\n%d %d\n255\n", W, H);
        for (uint32_t p : pic) std::fwrite(&p, 1, 3, f);   // (red, green, blue lie first in memory)
        std::fclose(f);
    }
    return 0;
}
