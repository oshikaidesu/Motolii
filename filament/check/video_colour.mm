// (a check, not a scene: one film filling the view with nothing done to its colours, a still at one moment, to set
//  beside what ffmpeg makes of the same file)   video_colour <film> <seconds> <out.png> <wide> <high>
#include "../kit.h"
#include "../video.h"
int main(int argc, char** argv) {
    if (argc < 6) return fail("video_colour <film> <seconds> <out.png> <wide> <high>");
    const int W = atoi(argv[4]), H = atoi(argv[5]);   // (the film's own shape as it is shown, turned if it asks to be)
    Stage s = open_stage(W, H);
    s.camera->lookAt({0.f, 0.f, 1.f}, {0.f, 0.f, 0.f}, {0, 1, 0});
    s.camera->setProjection(Camera::Projection::ORTHO, -double(W) / double(H), double(W) / double(H), -1.0, 1.0, 0.1, 10.0);
    static LinearToneMapper as_it_is;
    s.view->setColorGrading(ColorGrading::Builder().toneMapper(&as_it_is).build(*s.engine));
    if (!video(s, argv[1], mat4f{}, 2.f)) return fail(video_trouble);
    time_is(s, atof(argv[2]));
    return shoot(s, argv[3]) ? 0 : fail("png");
}
